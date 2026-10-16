// state.rs — Uygulama State Yönetimi
//
// AppState, Tauri'nin managed state sistemi ile enjekte edilir.
// Vault'un kilitli/açık durumu ve bellekteki şifreleme anahtarı burada tutulur.
//
// Güvenlik Notları:
// - encryption_key, Option<[u8; 32]> — None = vault kilitli
// - Vault kilitlendiğinde key alanı zeroize() ile sıfırlanır
// - Mutex ile thread-safe erişim sağlanır

use std::sync::Mutex;
use std::path::PathBuf;
use rusqlite::Connection;
use zeroize::Zeroize;

/// Uygulama genelinde paylaşılan state.
/// Tauri'nin `manage()` sistemi ile Arc<Mutex<T>> olarak sarılır.
pub struct AppState {
    /// SQLite bağlantısı
    pub conn: Mutex<Option<Connection>>,

    /// AES-256-GCM şifreleme anahtarı — yalnızca vault açıkken dolu
    /// Vault kilitlendiğinde zeroize edilip None yapılır.
    pub encryption_key: Mutex<Option<[u8; 32]>>,

    /// Vault veritabanı dosya yolu
    pub db_path: PathBuf,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            conn: Mutex::new(None),
            encryption_key: Mutex::new(None),
            db_path,
        }
    }

    /// Vault kilitli mi?
    pub fn is_locked(&self) -> bool {
        self.encryption_key
            .lock()
            .unwrap()
            .is_none()
    }

    /// Vault açıksa şifreleme anahtarının kopyasını döndürür.
    /// Kopyalama kaçınılmazdır (Mutex guard lifetime sınırlaması).
    pub fn get_key(&self) -> Option<[u8; 32]> {
        *self.encryption_key.lock().unwrap()
    }

    /// Anahtarı belleğe yükle (vault açma).
    pub fn set_key(&self, key: [u8; 32]) {
        *self.encryption_key.lock().unwrap() = Some(key);
    }

    /// Anahtarı güvenli şekilde temizle (vault kilitleme).
    pub fn clear_key(&self) {
        let mut guard = self.encryption_key.lock().unwrap();
        if let Some(ref mut key) = *guard {
            key.zeroize(); // Belleği sıfırla, sadece None yapma
        }
        *guard = None;
    }
}
