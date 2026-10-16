// crypto.rs — AES-256-GCM Şifreleme/Şifre Çözme Katmanı
//
// Neden AES-256-GCM?
// - AES-256: NSA Suite B standardı, günümüzde kırılabilir değil (2^256 arama alanı)
// - GCM (Galois/Counter Mode): Hem gizlilik (confidentiality) hem bütünlük
//   (integrity/authentication) sağlar. Şifreli veri değiştirilirse GCM bunu tespit eder.
// - Alternatif olarak ChaCha20-Poly1305 düşünülebilirdi; AES-NI donanım
//   hızlandırması olan sistemlerde AES-GCM genelde daha hızlıdır.
//
// IV/Nonce yönetimi:
// - Her şifreleme işlemi için 96-bit (12-byte) rastgele IV üretilir.
// - IV'nin tekrar kullanımı AES-GCM'de FELAKET sonuçları doğurur (aynı key+IV
//   ile iki farklı mesaj şifrelenirse her ikisi de çözülür).
// - Bu implementasyonda IV her çağrıda kriptografik olarak güvenli rastgele
//   üretilir (ring::rand::SystemRandom), tekrar kullanım riski yoktur.
//
// Depolama formatı: BASE64(IV || ciphertext || auth_tag)
// - 12 byte IV + n byte şifreli veri + 16 byte GCM auth tag
// - Tek string olarak base64'e kodlanır, veritabanında TEXT olarak saklanır

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM,
    NONCE_LEN,
};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use zeroize::Zeroizing;

use crate::errors::AppError;

/// Tek kullanımlık nonce (IV) üreteci.
/// Her şifreleme çağrısında kriptografik olarak güvenli rastgele 12 byte üretir.
struct OneTimeNonce([u8; NONCE_LEN]);

impl OneTimeNonce {
    fn new(rng: &SystemRandom) -> Result<Self, Unspecified> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rng.fill(&mut nonce_bytes)?;
        Ok(Self(nonce_bytes))
    }

    fn bytes(&self) -> [u8; NONCE_LEN] {
        self.0
    }
}

impl NonceSequence for OneTimeNonce {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Nonce::try_assume_unique_for_key(&self.0)
    }
}

/// Verilen plaintext'i AES-256-GCM ile şifreler.
///
/// # Güvenlik
/// - Her çağrıda yeni rastgele IV üretilir — asla tekrar kullanılmaz.
/// - GCM auth tag, şifreli verinin bütünlüğünü garanti eder.
/// - key parametresi Zeroizing ile sarılmalıdır (çağıran tarafın sorumluluğu).
///
/// # Çıktı Formatı
/// BASE64(12-byte IV || şifreli veri || 16-byte GCM auth tag)
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<String, AppError> {
    let rng = SystemRandom::new();

    // Kriptografik olarak güvenli rastgele IV üret
    let nonce = OneTimeNonce::new(&rng)
        .map_err(|_| AppError::Crypto("IV üretimi başarısız".into()))?;
    let nonce_bytes = nonce.bytes();

    // ring kütüphanesi için UnboundKey oluştur
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|_| AppError::Crypto("AES anahtar oluşturma hatası".into()))?;

    let mut sealing_key = SealingKey::new(unbound_key, nonce);

    // Şifreleme işlemi: in-place (ring, veri + 16-byte tag ekler sona)
    let mut buffer = plaintext.to_vec();

    sealing_key
        .seal_in_place_append_tag(Aad::empty(), &mut buffer)
        .map_err(|_| AppError::Crypto("Şifreleme başarısız".into()))?;

    // Format: IV || şifreli_veri+tag → base64
    let mut result = Vec::with_capacity(NONCE_LEN + buffer.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&buffer);

    Ok(BASE64.encode(result))
}

/// AES-256-GCM ile şifrelenmiş veriyi çözer.
///
/// # Güvenlik
/// - GCM auth tag doğrulama başarısız olursa hata döner (veri değiştirilmiş demektir).
/// - Döndürülen veri Zeroizing ile sarılmıştır: drop edildiğinde bellek temizlenir.
pub fn decrypt(key: &[u8; 32], ciphertext_b64: &str) -> Result<Zeroizing<Vec<u8>>, AppError> {
    // Base64 decode
    let raw = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| AppError::Crypto(format!("Base64 decode hatası: {}", e)))?;

    // Minimum boyut: 12 (IV) + 16 (GCM tag) = 28 byte — plaintext 0 byte olabilir
    if raw.len() < NONCE_LEN + 16 {
        return Err(AppError::Crypto("Şifreli veri çok kısa".into()));
    }

    // IV'yi ayır
    let (nonce_bytes, ciphertext_with_tag) = raw.split_at(NONCE_LEN);
    let nonce_arr: [u8; NONCE_LEN] = nonce_bytes
        .try_into()
        .map_err(|_| AppError::Crypto("IV boyutu hatalı".into()))?;

    let nonce = OneTimeNonce(nonce_arr);

    // ring için UnboundKey oluştur
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|_| AppError::Crypto("AES anahtar oluşturma hatası".into()))?;

    let mut opening_key = OpeningKey::new(unbound_key, nonce);

    // Şifre çözme + auth tag doğrulama (in-place)
    let mut buffer = ciphertext_with_tag.to_vec();
    let plaintext = opening_key
        .open_in_place(Aad::empty(), &mut buffer)
        .map_err(|_| {
            // Kasıtlı olarak genel hata mesajı: saldırgana oracle bilgisi verme
            AppError::DecryptionFailed
        })?;

    Ok(Zeroizing::new(plaintext.to_vec()))
}

/// Şifreli string'in beklenilen formatta olup olmadığını kontrol eder.
/// UI tarafında basit validation için kullanılabilir.
pub fn is_valid_encrypted_format(s: &str) -> bool {
    match BASE64.decode(s) {
        Ok(bytes) => bytes.len() >= NONCE_LEN + 16, // IV + GCM tag minimum
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        // Test için sabit anahtar (production'da asla böyle yapma!)
        [0x42u8; 32]
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"Bu gizli bir sifre: P@ssw0rd!123";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice(), "Şifre çözme başarısız");
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32]; // Farklı anahtar

        let encrypted = encrypt(&key1, b"gizli veri").unwrap();

        // Yanlış anahtarla çözme GCM auth tag hatası vermeli
        let result = decrypt(&key2, &encrypted);
        assert!(result.is_err(), "Yanlış anahtar ile şifre çözme başarısız olmalı");
        assert!(
            matches!(result.unwrap_err(), AppError::DecryptionFailed),
            "DecryptionFailed hatası döndürülmeli"
        );
    }

    #[test]
    fn test_iv_uniqueness() {
        // Her şifreleme farklı IV kullanmalı → farklı çıktı üretmeli
        let key = test_key();
        let plaintext = b"ayni metin";

        let enc1 = encrypt(&key, plaintext).unwrap();
        let enc2 = encrypt(&key, plaintext).unwrap();

        assert_ne!(enc1, enc2, "Her şifreleme benzersiz IV kullanmalı");

        // Ancak her ikisi de doğru çözülmeli
        let dec1 = decrypt(&key, &enc1).unwrap();
        let dec2 = decrypt(&key, &enc2).unwrap();
        assert_eq!(dec1.as_slice(), plaintext.as_slice());
        assert_eq!(dec2.as_slice(), plaintext.as_slice());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        // Şifreli verinin tek bir biti değiştirilirse GCM auth tag başarısız olmalı
        let key = test_key();
        let encrypted = encrypt(&key, b"kritik veri").unwrap();

        // Base64 decode, bit flip, tekrar encode
        let mut raw = BASE64.decode(&encrypted).unwrap();
        raw[15] ^= 0xFF; // Şifreli verinin ortasını boz
        let tampered = BASE64.encode(&raw);

        let result = decrypt(&key, &tampered);
        assert!(result.is_err(), "Değiştirilmiş veri reddedilmeli");
    }

    #[test]
    fn test_empty_plaintext() {
        let key = test_key();
        let encrypted = encrypt(&key, b"").unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted.as_slice(), b"");
    }

    #[test]
    fn test_unicode_plaintext() {
        let key = test_key();
        let plaintext = "Türkçe şifre: ğüşıöçÇÖŞİĞÜ 🔐".as_bytes();
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }
}
