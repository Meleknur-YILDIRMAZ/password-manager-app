// commands/totp.rs — TOTP (RFC 6238) Kod Üretici
//
// TOTP (Time-based One-Time Password) Google Authenticator ile uyumlu
// RFC 6238 standardına göre implementasyonu.
// TOTP secret'ları vault_entries.totp_enc alanında AES-256-GCM ile şifreli saklanır.

use std::time::{SystemTime, UNIX_EPOCH};

use base32::Alphabet;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;

use crate::errors::AppError;

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpCode {
    pub code: String,        // 6 haneli kod
    pub remaining_seconds: u64, // Geçerlilik süresi (0-29 saniye)
    pub period: u64,         // Periyot (genelde 30 saniye)
}

/// Verilen TOTP secret için güncel kodu üretir.
///
/// # Standart Uyumluluk
/// - RFC 6238 (TOTP) + RFC 4226 (HOTP) temel alınmıştır
/// - Base32 kodlu secret (Google Authenticator formatı)
/// - 30 saniyelik periyot, SHA-1 HMAC, 6 haneli kod
/// - Hesap: floor(current_unix_time / 30)
#[tauri::command]
pub fn generate_totp(secret_base32: String) -> Result<TotpCode, AppError> {
    // Base32 decode (boşluk ve büyük/küçük harf normalize et)
    let secret_clean = secret_base32
        .replace(' ', "")
        .replace('-', "")
        .to_uppercase();

    let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, &secret_clean)
        .ok_or_else(|| AppError::InvalidInput("Geçersiz TOTP secret (Base32 formatı bekleniyor)".into()))?;

    // Mevcut UNIX timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::Internal("Sistem saati hatası".into()))?
        .as_secs();

    let period = 30u64;
    let counter = now / period;
    let remaining = period - (now % period);

    // HOTP hesapla (RFC 4226)
    let code = compute_hotp(&secret_bytes, counter)?;

    Ok(TotpCode {
        code: format!("{:06}", code),
        remaining_seconds: remaining,
        period,
    })
}

/// HOTP (HMAC-based One-Time Password) hesapla (RFC 4226).
fn compute_hotp(secret: &[u8], counter: u64) -> Result<u32, AppError> {
    // Counter'ı big-endian 8 byte'a çevir
    let counter_bytes = counter.to_be_bytes();

    // HMAC-SHA1 hesapla
    let mut mac = HmacSha1::new_from_slice(secret)
        .map_err(|_| AppError::Crypto("HMAC oluşturma hatası".into()))?;
    mac.update(&counter_bytes);
    let hmac_result = mac.finalize().into_bytes();

    // Dynamic truncation (RFC 4226 Section 5.3)
    let offset = (hmac_result[19] & 0x0F) as usize;
    let binary = ((hmac_result[offset] as u32 & 0x7F) << 24)
        | ((hmac_result[offset + 1] as u32) << 16)
        | ((hmac_result[offset + 2] as u32) << 8)
        | (hmac_result[offset + 3] as u32);

    // 6 haneli kod (10^6 = 1_000_000)
    Ok(binary % 1_000_000)
}

/// otpauth:// URI'sinden secret'ı parse et.
/// Örnek: otpauth://totp/Example:user@example.com?secret=BASE32SECRET&issuer=Example
#[tauri::command]
pub fn parse_otpauth_uri(uri: String) -> Result<String, AppError> {
    if !uri.starts_with("otpauth://totp/") {
        return Err(AppError::InvalidInput("Geçersiz otpauth URI formatı".into()));
    }

    // Secret parametresini bul
    let secret = uri
        .split('?')
        .nth(1)
        .ok_or_else(|| AppError::InvalidInput("URI parametre kısmı bulunamadı".into()))?
        .split('&')
        .find(|p| p.starts_with("secret="))
        .ok_or_else(|| AppError::InvalidInput("secret parametresi bulunamadı".into()))?
        .strip_prefix("secret=")
        .unwrap()
        .to_string();

    Ok(secret)
}
