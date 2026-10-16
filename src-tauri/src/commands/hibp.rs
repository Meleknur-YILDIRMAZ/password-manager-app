// commands/hibp.rs — Have I Been Pwned k-Anonymity Kontrolü
//
// Gizlilik garantisi:
// 1. Şifre SHA-1 ile hashlenir (RFC 5.1)
// 2. Hash'in SADECE ilk 5 hex karakteri HIBP API'sine gönderilir
// 3. API bu prefix ile başlayan tüm hash'leri (suffix kısmıyla) döndürür
// 4. Gerçek eşleşme kontrolü yerel olarak yapılır
// 5. Şifrenin kendisi veya tam hash'i asla ağa çıkmaz
//
// Bu özellik varsayılan olarak KAPALI gelir.
// Kullanıcı Ayarlar'dan açıkça etkinleştirmeli ve "Kontrol Et" butonuna basmalıdır.

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct HibpResult {
    pub is_pwned: bool,
    pub pwn_count: Option<u64>, // Kaç veri ihlalinde görülmüş
}

/// Şifrenin HIBP veritabanında yer alıp almadığını kontrol eder.
///
/// # Güvenlik
/// k-anonymity modeli kullanılır: şifrenin SHA-1 hash'inin ilk 5 karakteri gönderilir.
/// Şifrenin kendisi veya tam hash'i asla API'ye gönderilmez.
///
/// # Not
/// Bu fonksiyon internet bağlantısı gerektirir.
/// Yalnızca kullanıcı açıkça "kontrol et" dediğinde çağrılmalıdır.
#[tauri::command]
pub async fn check_hibp(password: String) -> Result<HibpResult, AppError> {
    use ring::digest::{digest, SHA1_FOR_LEGACY_USE_ONLY};

    // SHA-1 hash hesapla
    let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, password.as_bytes());
    let hex_hash = hex::encode(hash.as_ref()).to_uppercase();

    let prefix = &hex_hash[..5];
    let suffix = &hex_hash[5..];

    // HIBP API'sine prefix gönder
    let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("PasswordManager/1.0")
        .build()
        .map_err(|e| AppError::Network(e.to_string()))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("HIBP API bağlantı hatası: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Network(format!(
            "HIBP API hatası: {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    // Yanıt formatı: "SUFFIX:COUNT\r\n" satırları
    for line in body.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 && parts[0].eq_ignore_ascii_case(suffix) {
            let count: u64 = parts[1].trim().parse().unwrap_or(0);
            return Ok(HibpResult {
                is_pwned: true,
                pwn_count: Some(count),
            });
        }
    }

    Ok(HibpResult {
        is_pwned: false,
        pwn_count: None,
    })
}
