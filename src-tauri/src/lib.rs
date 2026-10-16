// lib.rs — Tauri Uygulama Kütüphane Giriş Noktası
// Tüm modülleri ve Tauri komutlarını kayıt eder.

pub mod commands;
pub mod core;
pub mod db;
pub mod errors;
pub mod state;

use std::path::PathBuf;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Uygulama veri dizinini belirle
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("App data dizini belirlenemedi");

            std::fs::create_dir_all(&app_dir)
                .expect("App data dizini oluşturulamadı");

            let db_path: PathBuf = app_dir.join("vault.db");

            // AppState'i Tauri'ye kayıt et
            app.manage(AppState::new(db_path));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth komutları
            commands::auth::is_setup,
            commands::auth::setup_vault,
            commands::auth::unlock_vault,
            commands::auth::lock_vault,
            commands::auth::is_locked,
            commands::auth::get_vault_settings,
            commands::auth::update_vault_settings,
            // Vault CRUD komutları
            commands::vault::create_vault_entry,
            commands::vault::update_vault_entry,
            commands::vault::delete_vault_entry,
            commands::vault::list_vault_entries,
            commands::vault::get_vault_entry,
            commands::vault::analyze_passwords,
            // Şifre üretici
            commands::generator::generate_password,
            commands::generator::get_hibp_prefix,
            // TOTP
            commands::totp::generate_totp,
            commands::totp::parse_otpauth_uri,
            // HIBP
            commands::hibp::check_hibp,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri uygulaması başlatılamadı");
}
