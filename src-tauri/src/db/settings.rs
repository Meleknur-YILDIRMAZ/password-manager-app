// settings.rs — Uygulama Ayarları (salt, verify hash, vb.)
//
// Vault metadata'sı (salt, doğrulama hash'i, ayarlar) burada yönetilir.
// Tüm veriler basit key-value çiftleri olarak settings tablosunda saklanır.

use rusqlite::{Connection, OptionalExtension};

use crate::errors::AppError;

// Settings anahtarları — sabit string olarak tanımlı (typo'lardan korur)
pub const KEY_SALT: &str = "argon2_salt";
pub const KEY_VERIFY_HASH: &str = "master_password_hash";
pub const KEY_VAULT_VERSION: &str = "vault_version";
pub const KEY_AUTO_LOCK_MINUTES: &str = "auto_lock_minutes";
pub const KEY_HIBP_ENABLED: &str = "hibp_enabled";
pub const KEY_THEME: &str = "theme";
pub const KEY_CLIPBOARD_CLEAR_SECONDS: &str = "clipboard_clear_seconds";

/// Bir ayar değerini kaydet (INSERT OR REPLACE).
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(AppError::from)?;
    Ok(())
}

/// Bir ayar değerini getir.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(AppError::from)?;

    let result = stmt
        .query_row(rusqlite::params![key], |row| row.get(0))
        .optional()
        .map_err(AppError::from)?;

    Ok(result)
}

/// Vault'un ilk kurulum yapılıp yapılmadığını kontrol eder.
pub fn is_vault_initialized(conn: &Connection) -> Result<bool, AppError> {
    let salt = get_setting(conn, KEY_SALT)?;
    let hash = get_setting(conn, KEY_VERIFY_HASH)?;
    Ok(salt.is_some() && hash.is_some())
}

/// İlk kurulum sırasında vault metadata'sını kaydeder.
pub fn initialize_vault_settings(
    conn: &Connection,
    salt: &str,
    verify_hash: &str,
) -> Result<(), AppError> {
    set_setting(conn, KEY_SALT, salt)?;
    set_setting(conn, KEY_VERIFY_HASH, verify_hash)?;
    set_setting(conn, KEY_VAULT_VERSION, "1")?;
    set_setting(conn, KEY_AUTO_LOCK_MINUTES, "5")?;
    set_setting(conn, KEY_HIBP_ENABLED, "false")?;
    set_setting(conn, KEY_THEME, "dark")?;
    set_setting(conn, KEY_CLIPBOARD_CLEAR_SECONDS, "30")?;
    Ok(())
}
