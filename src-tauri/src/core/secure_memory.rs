// secure_memory.rs — Güvenli Bellek Yönetimi Yardımcıları
//
// Sorun: Rust'ın normal bellek yönetimi, hassas verileri (şifreler, anahtarlar)
// bellekten siler gibi görünse de derleyici optimizasyonları bu silme işlemini
// atlatabilir (dead store elimination). Zeroize crate bu sorunu çözer.
//
// zeroize crate, volatile write kullanarak derleyicinin atlayamayacağı
// gerçek bellek silme işlemi yapar.

use zeroize::Zeroize;

/// 32-byte AES anahtarını güvenli şekilde bellekten temizler.
/// Drop edildiğinde otomatik çağrılmaz — explicit çağrı gerekir.
pub fn zero_key(key: &mut [u8; 32]) {
    key.zeroize();
}

/// Byte slice'ı güvenli şekilde bellekten temizler.
pub fn zero_bytes(data: &mut Vec<u8>) {
    data.zeroize();
}

/// String'i güvenli şekilde bellekten temizler.
pub fn zero_string(s: &mut String) {
    // String'in iç byte buffer'ını sıfırla
    unsafe {
        let bytes = s.as_bytes_mut();
        bytes.zeroize();
    }
    s.clear();
}

/// Güvenli string tipi — drop edildiğinde otomatik olarak bellekten temizlenir.
/// Master password gibi hassas string'ler için kullan.
pub use zeroize::Zeroizing;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_key() {
        let mut key = [0xFFu8; 32];
        zero_key(&mut key);
        assert_eq!(key, [0u8; 32], "Anahtar sıfırlanmış olmalı");
    }

    #[test]
    fn test_zero_bytes() {
        let mut data = vec![0x42u8; 100];
        zero_bytes(&mut data);
        assert!(data.iter().all(|&b| b == 0), "Tüm byte'lar sıfırlanmış olmalı");
    }
}
