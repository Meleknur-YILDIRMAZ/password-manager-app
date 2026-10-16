// errors.rs — Merkezi Hata Tipleri
//
// Tauri komutları serde::Serialize gerektirdiği için hata tipimiz
// String'e dönüştürülebilir olmalı.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum AppError {
    #[error("Şifreleme hatası: {0}")]
    Crypto(String),

    #[error("Şifre çözme başarısız — anahtar yanlış veya veri bozuk")]
    DecryptionFailed,

    #[error("Veritabanı hatası: {0}")]
    Database(String),

    #[error("Vault kilitli — önce kilidi açın")]
    VaultLocked,

    #[error("Vault zaten kurulmuş")]
    AlreadySetup,

    #[error("Vault henüz kurulmamış")]
    NotSetup,

    #[error("Geçersiz girdi: {0}")]
    InvalidInput(String),

    #[error("Kayıt bulunamadı: {0}")]
    NotFound(String),

    #[error("Ağ hatası: {0}")]
    Network(String),

    #[error("Beklenmeyen hata: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
