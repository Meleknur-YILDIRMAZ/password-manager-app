// commands/auth.rs — Kimlik Doğrulama Komutları
//
// Vault kurulum, kilitleme ve kilit açma işlemleri.

use tauri::State;

use crate::core::kdf::{derive_key, generate_salt, hash_master_password, verify_master_password};
use crate::db::schema::initialize_schema;
use crate::db::settings::{
    get_setting, initialize_vault_settings, is_vault_initialized, set_setting,
    KEY_AUTO_LOCK_MINUTES, KEY_CLIPBOARD_CLEAR_SECONDS, KEY_HIBP_ENABLED, KEY_SALT,
    KEY_THEME, KEY_VERIFY_HASH,
};
use crate::errors::AppError;
use crate::state::AppState;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultSettings {
    pub auto_lock_minutes: u32,
    pub hibp_enabled: bool,
    pub theme: String,
    pub clipboard_clear_seconds: u32,
}

/// Vault daha önce kurulmuş mu?
#[tauri::command]
pub fn is_setup(state: State<AppState>) -> Result<bool, AppError> {
    let conn_guard = state.conn.lock().unwrap();
    match &*conn_guard {
        Some(conn) => is_vault_initialized(conn),
        None => {
            // Bağlantı henüz açılmamış — DB dosyasını kontrol et
            if state.db_path.exists() {
                let conn = Connection::open(&state.db_path).map_err(AppError::from)?;
                initialize_schema(&conn)?;
                is_vault_initialized(&conn)
            } else {
                Ok(false)
            }
        }
    }
}

/// İlk kurulum — master şifre belirle, vault'u başlat.
#[tauri::command]
pub fn setup_vault(
    master_password: String,
    state: State<AppState>,
) -> Result<(), AppError> {
    // Güvenlik: şifre en az 8 karakter olmalı
    if master_password.len() < 8 {
        return Err(AppError::InvalidInput(
            "Master şifre en az 8 karakter olmalıdır".into(),
        ));
    }

    let mut conn_guard = state.conn.lock().unwrap();

    // Bağlantı aç
    let conn = Connection::open(&state.db_path).map_err(AppError::from)?;
    initialize_schema(&conn)?;

    if is_vault_initialized(&conn)? {
        return Err(AppError::AlreadySetup);
    }

    // Benzersiz salt üret (her vault için bir kez, asla değiştirilmez)
    let salt = generate_salt();

    // Master şifrenin doğrulama hash'ini oluştur
    // NOT: Bu hash'ten şifreleme anahtarı türetilemez — sadece doğrulama içindir.
    let verify_hash = hash_master_password(&master_password)?;

    // Vault ayarlarını kaydet
    initialize_vault_settings(&conn, &salt, &verify_hash)?;

    // Şifreleme anahtarını türet ve belleğe yükle
    // Zeroizing wrapper: derive_key çıktısı scope dışına çıkınca silinir
    let key_vec: Zeroizing<Vec<u8>> = derive_key(&master_password, &salt)?;
    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&key_vec);

    // master_password string'ini belleğe almayı bitirince Rust drop eder
    // (Zeroizing<String> kullansaydık daha güvenli olurdu — iyileştirme notu)

    *conn_guard = Some(conn);
    drop(conn_guard);

    state.set_key(key_arr);

    Ok(())
}

/// Vault'u aç — master şifre ile kimlik doğrula.
#[tauri::command]
pub fn unlock_vault(
    master_password: String,
    state: State<AppState>,
) -> Result<(), AppError> {
    let mut conn_guard = state.conn.lock().unwrap();

    // Bağlantı yoksa aç
    if conn_guard.is_none() {
        if !state.db_path.exists() {
            return Err(AppError::NotSetup);
        }
        let conn = Connection::open(&state.db_path).map_err(AppError::from)?;
        initialize_schema(&conn)?;
        *conn_guard = Some(conn);
    }

    let conn = conn_guard.as_ref().unwrap();

    // Salt ve verify hash'i getir
    let salt = get_setting(conn, KEY_SALT)?
        .ok_or_else(|| AppError::NotSetup)?;
    let verify_hash = get_setting(conn, KEY_VERIFY_HASH)?
        .ok_or_else(|| AppError::NotSetup)?;

    // Şifreyi doğrula (zamanlanmış — sabit süre, timing oracle'ı önler)
    let is_valid = verify_master_password(&master_password, &verify_hash)?;
    if !is_valid {
        // Sabit hata mesajı — ayrıntı verme (oracle saldırısı önlemi)
        return Err(AppError::DecryptionFailed);
    }

    // Doğrulama başarılı → anahtarı türet ve belleğe yükle
    let key_vec = derive_key(&master_password, &salt)?;
    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&key_vec);

    drop(conn_guard);
    state.set_key(key_arr);

    Ok(())
}

/// Vault'u kilitle — bellekteki anahtarı sıfırla.
#[tauri::command]
pub fn lock_vault(state: State<AppState>) -> Result<(), AppError> {
    state.clear_key();
    Ok(())
}

/// Vault kilitli mi?
#[tauri::command]
pub fn is_locked(state: State<AppState>) -> bool {
    state.is_locked()
}

/// Vault ayarlarını getir.
#[tauri::command]
pub fn get_vault_settings(state: State<AppState>) -> Result<VaultSettings, AppError> {
    let conn_guard = state.conn.lock().unwrap();
    let conn = conn_guard
        .as_ref()
        .ok_or(AppError::VaultLocked)?;

    let auto_lock = get_setting(conn, KEY_AUTO_LOCK_MINUTES)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(5u32);

    let hibp = get_setting(conn, KEY_HIBP_ENABLED)?
        .map(|v| v == "true")
        .unwrap_or(false);

    let theme = get_setting(conn, KEY_THEME)?
        .unwrap_or_else(|| "dark".into());

    let clipboard_sec = get_setting(conn, KEY_CLIPBOARD_CLEAR_SECONDS)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(30u32);

    Ok(VaultSettings {
        auto_lock_minutes: auto_lock,
        hibp_enabled: hibp,
        theme,
        clipboard_clear_seconds: clipboard_sec,
    })
}

/// Vault ayarlarını güncelle.
#[tauri::command]
pub fn update_vault_settings(
    settings: VaultSettings,
    state: State<AppState>,
) -> Result<(), AppError> {
    if state.is_locked() {
        return Err(AppError::VaultLocked);
    }

    let conn_guard = state.conn.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or(AppError::VaultLocked)?;

    set_setting(conn, KEY_AUTO_LOCK_MINUTES, &settings.auto_lock_minutes.to_string())?;
    set_setting(conn, KEY_HIBP_ENABLED, if settings.hibp_enabled { "true" } else { "false" })?;
    set_setting(conn, KEY_THEME, &settings.theme)?;
    set_setting(conn, KEY_CLIPBOARD_CLEAR_SECONDS, &settings.clipboard_clear_seconds.to_string())?;

    Ok(())
}
