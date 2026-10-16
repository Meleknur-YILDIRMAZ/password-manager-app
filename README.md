# 🔐 Şifre Yöneticisi (Offline-First Desktop Password Manager)

**Tauri v2 (Rust)** ve **React + TypeScript** ile geliştirilmiş, tamamen yerel (offline-first) çalışan güvenli masaüstü şifre yöneticisi uygulaması.

## ✨ Temel Özellikler

- **🛡️ Argon2id Anahtar Türetme:** Ana şifre (Master Password) hiçbir zaman diske kaydedilmez; 64 MB bellek, 3 iterasyon ve 4 paralellik parametreleriyle Argon2id kullanılarak 256-bit şifreleme anahtarı türetilir.
- **🔒 AES-256-GCM Alan Bazlı Şifreleme:** Kullanıcı adı, şifre, notlar, URL ve TOTP gizli anahtarları veritabanına yazılmadan önce her kayıt ve alan için benzersiz 96-bit IV (nonce) ile şifrelenir.
- **🧹 Güvenli Bellek Yönetimi (Zeroize):** Kasa kilitlendiğinde veya uygulama kapatıldığında şifreleme anahtarı RAM üzerinden güvenli bir şekilde sıfırlanır (`zeroize`).
- **⏱️ Otomatik Kilitleme ve Pano Temizleme:** Hareketsizlik durumunda otomatik kasa kilitleme (5 dk) ve panoya kopyalanan şifreleri 30 saniye sonra otomatik temizleme.
- **🔑 Güçlü Şifre Üretici ve Güç Analizi:** CSPRNG tabanlı şifre üretici ve `zxcvbn` ile gerçek zamanlı kırılma süresi/güç analizi.
- **📱 TOTP (2FA) Üretici:** Google Authenticator uyumlu (RFC 6238) 6 haneli zaman tabanlı doğrulama kodu üretimi.
- **🕵️ HIBP Sızıntı Kontrolü (k-Anonymity):** İsteğe bağlı olarak şifrenin kendisini göndermeden yalnızca SHA-1 özetinin ilk 5 karakteriyle sızıntı kontrolü.

## 🛠️ Teknoloji Yığını

- **Backend / Core:** Rust, Tauri v2, `ring` (AES-256-GCM), `argon2` (Argon2id), `rusqlite` (SQLite), `zeroize`
- **Frontend:** React 19, TypeScript, Vite, Zustand, Lucide Icons, `zxcvbn`

## 🚀 Kurulum ve Çalıştırma

```bash
# Bağımlılıkları yükleyin
npm install

# Geliştirme modunda başlatın
npm run tauri dev

# Rust birim testlerini çalıştırın
cd src-tauri && cargo test

# Kurulum paketi (.exe) üretin
npm run tauri build
```
