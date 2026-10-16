// kdf.rs — Anahtar Türetme Fonksiyonu (Key Derivation Function)
//
// Neden Argon2id?
// - PBKDF2 veya bcrypt, modern GPU/ASIC saldırılarına karşı zayıf.
// - Argon2id hem bellek-zorlu (memory-hard) hem de CPU-zorlu olduğu için
//   GPU ile paralel brute-force saldırısını ekonomik olarak imkânsız kılar.
// - Argon2id, 2015 Password Hashing Competition birincisidir ve
//   OWASP, NIST SP 800-63B tarafından tavsiye edilmektedir.
//
// Parametre seçimi:
// - memory_cost: 65536 KB (64 MB) — saldırganın her deneme için 64 MB RAM ayırmasını zorlar
// - time_cost: 3 iterasyon — ek zaman maliyeti
// - parallelism: 4 — çok çekirdekli sistemlerde kullanım
// Bu parametreler, ~0.3-1 sn. gecikme sağlar (kullanıcı deneyimini bozmadan güvenlik)

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params, Version,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use zeroize::Zeroizing;

use crate::errors::AppError;

// Argon2id parametreleri — Production değerleri
// Düşürme: yalnızca test ortamında (feature flag ile)
#[cfg(not(test))]
const MEMORY_COST_KB: u32 = 65536; // 64 MB
#[cfg(not(test))]
const TIME_COST: u32 = 3;
#[cfg(not(test))]
const PARALLELISM: u32 = 4;

// Test ortamında hızlı çalışması için düşük parametreler
#[cfg(test)]
const MEMORY_COST_KB: u32 = 8192; // 8 MB (test için)
#[cfg(test)]
const TIME_COST: u32 = 1;
#[cfg(test)]
const PARALLELISM: u32 = 1;

/// Master şifreden 256-bit (32-byte) şifreleme anahtarı türetir.
///
/// # Güvenlik Notları
/// - Döndürülen anahtar Zeroizing<Vec<u8>> ile sarılmıştır:
///   drop edildiğinde bellek sıfırlanır.
/// - Salt, her kullanıcı/vault için bir kez üretilmeli ve saklanmalı.
///   Salt gizli değildir; veritabanında açık saklanabilir.
/// - Aynı master_password + salt kombinasyonu her zaman aynı anahtarı üretir
///   (deterministic) — bu sayede şifre doğrulaması için anahtarı saklamak gerekmez.
pub fn derive_key(master_password: &str, salt_b64: &str) -> Result<Zeroizing<Vec<u8>>, AppError> {
    // Salt'ı base64'ten decode et
    let salt_bytes = BASE64
        .decode(salt_b64)
        .map_err(|e| AppError::Crypto(format!("Salt decode hatası: {}", e)))?;

    // Argon2id yapılandırması
    let params = Params::new(MEMORY_COST_KB, TIME_COST, PARALLELISM, Some(32))
        .map_err(|e| AppError::Crypto(format!("Argon2 parametre hatası: {}", e)))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    // 32-byte çıktı için buffer hazırla
    let mut key = Zeroizing::new(vec![0u8; 32]);

    // Argon2id ile anahtar türet (raw hash, PHC formatı değil)
    argon2
        .hash_password_into(master_password.as_bytes(), &salt_bytes, &mut key)
        .map_err(|e| AppError::Crypto(format!("Argon2id türetme hatası: {}", e)))?;

    Ok(key)
}

/// Master şifreden bir doğrulama hash'i oluşturur (PHC string formatında).
///
/// Bu hash, anahtarın kendisi değildir! Yalnızca "kullanıcı doğru şifreyi
/// girdi mi?" sorusunu cevaplamak için kullanılır.
///
/// # Neden Ayrı Hash?
/// Türetilen AES anahtarını doğrulamak için önce bir veriyi çözmeyi deneriz
/// diyebilirsiniz, ancak bu GCM auth tag sayesinde zaten yapılabilir.
/// Bu fonksiyon, daha hızlı ve açık bir "şifre doğru mu?" kontrolü sağlar.
pub fn hash_master_password(master_password: &str) -> Result<String, AppError> {
    let params = Params::new(MEMORY_COST_KB, TIME_COST, PARALLELISM, Some(32))
        .map_err(|e| AppError::Crypto(format!("Argon2 parametre hatası: {}", e)))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    // Yeni benzersiz salt üret (PHC formatındaki hash içinde gömülü gelir)
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(master_password.as_bytes(), &salt)
        .map_err(|e| AppError::Crypto(format!("Şifre hashleme hatası: {}", e)))?;

    Ok(password_hash.to_string())
}

/// Kullanıcının girdiği şifreyi kayıtlı hash ile doğrular.
pub fn verify_master_password(master_password: &str, hash: &str) -> Result<bool, AppError> {
    let params = Params::new(MEMORY_COST_KB, TIME_COST, PARALLELISM, Some(32))
        .map_err(|e| AppError::Crypto(format!("Argon2 parametre hatası: {}", e)))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Crypto(format!("Hash parse hatası: {}", e)))?;

    match argon2.verify_password(master_password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::Crypto(format!("Doğrulama hatası: {}", e))),
    }
}

/// Yeni vault için rastgele 32-byte salt üretir (base64 kodlanmış).
///
/// Salt gizli değildir; veritabanında açık olarak saklanır.
/// Her vault için bir kez üretilmeli, hiçbir zaman değiştirilmemeli.
pub fn generate_salt() -> String {
    use ring::rand::{SecureRandom, SystemRandom};
    let rng = SystemRandom::new();
    let mut salt = [0u8; 32];
    rng.fill(&mut salt).expect("Salt üretimi başarısız");
    BASE64.encode(salt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        // Aynı şifre + salt kombinasyonu her zaman aynı anahtarı üretmeli
        let salt = generate_salt();
        let key1 = derive_key("test_password_123!", &salt).unwrap();
        let key2 = derive_key("test_password_123!", &salt).unwrap();
        assert_eq!(*key1, *key2, "Aynı input için anahtar deterministik olmalı");
    }

    #[test]
    fn test_derive_key_different_passwords() {
        // Farklı şifreler farklı anahtar üretmeli
        let salt = generate_salt();
        let key1 = derive_key("password_one", &salt).unwrap();
        let key2 = derive_key("password_two", &salt).unwrap();
        assert_ne!(*key1, *key2, "Farklı şifreler farklı anahtar üretmeli");
    }

    #[test]
    fn test_derive_key_different_salts() {
        // Aynı şifre, farklı salt → farklı anahtar
        let key1 = derive_key("same_password", &generate_salt()).unwrap();
        let key2 = derive_key("same_password", &generate_salt()).unwrap();
        assert_ne!(*key1, *key2, "Farklı salt ile farklı anahtar üretilmeli");
    }

    #[test]
    fn test_key_length() {
        let salt = generate_salt();
        let key = derive_key("test", &salt).unwrap();
        assert_eq!(key.len(), 32, "AES-256 için anahtar 32 byte olmalı");
    }

    #[test]
    fn test_hash_and_verify() {
        let password = "my_secure_master_pass!";
        let hash = hash_master_password(password).unwrap();

        assert!(
            verify_master_password(password, &hash).unwrap(),
            "Doğru şifre doğrulanmalı"
        );
        assert!(
            !verify_master_password("wrong_password", &hash).unwrap(),
            "Yanlış şifre reddedilmeli"
        );
    }
}
