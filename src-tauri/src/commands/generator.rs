// commands/generator.rs — Güvenli Şifre Üretici
//
// Ring crate'in SystemRandom'u kullanır: OS'in kriptografik rastgele
// sayı üretecine (CSPRNG) dayanır — /dev/urandom veya Windows CNG.
// Math.random() gibi pseudo-random üreticiler ASLA kullanılmaz.

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

#[derive(Debug, Deserialize)]
pub struct GeneratorOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool, // l, I, 1, O, 0 gibi karışan karakterler
}

#[derive(Debug, Serialize)]
pub struct GeneratedPassword {
    pub password: String,
}

const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const UPPERCASE_NO_AMBIGUOUS: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ"; // I, O çıkarıldı
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const LOWERCASE_NO_AMBIGUOUS: &[u8] = b"abcdefghjkmnpqrstuvwxyz"; // l çıkarıldı
const DIGITS: &[u8] = b"0123456789";
const DIGITS_NO_AMBIGUOUS: &[u8] = b"23456789"; // 0, 1 çıkarıldı
const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

/// Kriptografik olarak güvenli rastgele şifre üretir.
///
/// # Algoritma
/// Modulo bias'ını önlemek için rejection sampling kullanılır:
/// charset boyutu 256'nın tam böleni değilse, fazla kalan değerler reddedilir.
/// Bu, her karakterin eşit olasılıkla seçilmesini garanti eder.
#[tauri::command]
pub fn generate_password(options: GeneratorOptions) -> Result<GeneratedPassword, AppError> {
    // Parametre doğrulama
    if options.length < 4 || options.length > 128 {
        return Err(AppError::InvalidInput(
            "Şifre uzunluğu 4-128 karakter arasında olmalıdır".into(),
        ));
    }

    if !options.uppercase && !options.lowercase && !options.digits && !options.symbols {
        return Err(AppError::InvalidInput(
            "En az bir karakter seti seçilmelidir".into(),
        ));
    }

    // Karakter setini oluştur
    let mut charset: Vec<u8> = Vec::new();

    if options.uppercase {
        if options.exclude_ambiguous {
            charset.extend_from_slice(UPPERCASE_NO_AMBIGUOUS);
        } else {
            charset.extend_from_slice(UPPERCASE);
        }
    }
    if options.lowercase {
        if options.exclude_ambiguous {
            charset.extend_from_slice(LOWERCASE_NO_AMBIGUOUS);
        } else {
            charset.extend_from_slice(LOWERCASE);
        }
    }
    if options.digits {
        if options.exclude_ambiguous {
            charset.extend_from_slice(DIGITS_NO_AMBIGUOUS);
        } else {
            charset.extend_from_slice(DIGITS);
        }
    }
    if options.symbols {
        charset.extend_from_slice(SYMBOLS);
    }

    let charset_len = charset.len();
    // Rejection sampling için bias threshold hesapla
    // 256 % charset_len değerindeki byte'ları reddet
    let max_valid = 256 - (256 % charset_len);

    let rng = SystemRandom::new();
    let mut password = Vec::with_capacity(options.length);
    let mut random_buf = [0u8; 1];

    // Her pozisyon için rejection sampling
    while password.len() < options.length {
        rng.fill(&mut random_buf)
            .map_err(|_| AppError::Crypto("Rastgele sayı üretimi başarısız".into()))?;

        let byte = random_buf[0] as usize;

        // Modulo bias'ı önle: bias aralığındaki değerleri reddet
        if byte < max_valid {
            password.push(charset[byte % charset_len]);
        }
    }

    // Her seçilen karakter setinden en az biri olduğunu doğrula
    // (Zorunlu karakter garantisi — garanti yoksa tekrar üret)
    let generated = String::from_utf8(password)
        .map_err(|_| AppError::Internal("Karakter dönüşüm hatası".into()))?;

    // Her aktif set için en az 1 karakter kontrolü
    let has_upper = !options.uppercase || generated.chars().any(|c| c.is_uppercase());
    let has_lower = !options.lowercase || generated.chars().any(|c| c.is_lowercase());
    let has_digit = !options.digits || generated.chars().any(|c| c.is_ascii_digit());
    let has_symbol = !options.symbols || generated.chars().any(|c| SYMBOLS.contains(&(c as u8)));

    if !has_upper || !has_lower || !has_digit || !has_symbol {
        // Tekrar dene (recursion yerine loop ile yeniden çağır)
        return generate_password(options);
    }

    Ok(GeneratedPassword { password: generated })
}

/// Şifrenin SHA-1 hash'inin ilk 5 karakterini döndürür (HIBP k-anonymity için).
///
/// # Gizlilik
/// HIBP API'sine şifrenin kendisi asla gönderilmez.
/// Sadece SHA-1 hash'inin ilk 5 hex karakteri gönderilir.
/// API, bu prefix ile başlayan tüm hash'leri döndürür.
/// Eşleşme kontrolü tamamen istemci tarafında yapılır.
#[tauri::command]
pub fn get_hibp_prefix(password: String) -> Result<String, AppError> {
    use ring::digest::{digest, SHA1_FOR_LEGACY_USE_ONLY};

    let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, password.as_bytes());
    let hex = hex::encode(hash.as_ref()).to_uppercase();

    // İlk 5 karakter prefix, kalan suffix (istemci tarafı eşleşme için)
    let prefix = &hex[..5];
    let suffix = &hex[5..];

    // Her ikisini de döndür: prefix API'ye gider, suffix lokal karşılaştırma için
    Ok(format!("{}:{}", prefix, suffix))
}
