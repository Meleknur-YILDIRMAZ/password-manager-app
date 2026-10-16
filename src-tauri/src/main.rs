// main.rs — Tauri Uygulama Ana Giriş Noktası
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sifre_yoneticisi_lib::run();
}
