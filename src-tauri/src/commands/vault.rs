// commands/vault.rs — Vault CRUD Komutları

use tauri::State;
use serde::{Deserialize, Serialize};

use crate::db::vault::{
    create_entry, delete_entry, get_all_passwords, get_entry, list_entries,
    update_entry, VaultEntry, VaultEntryMeta,
};
use crate::errors::AppError;
use crate::state::AppState;

/// Yeni vault kaydı oluştur.
#[tauri::command]
pub fn create_vault_entry(
    entry: VaultEntry,
    state: State<AppState>,
) -> Result<String, AppError> {
    if state.is_locked() {
        return Err(AppError::VaultLocked);
    }

    let key = state.get_key().ok_or(AppError::VaultLocked)?;
    let conn_guard = state.conn.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or(AppError::VaultLocked)?;

    create_entry(conn, &key, &entry)
}

/// Mevcut kaydı güncelle.
#[tauri::command]
pub fn update_vault_entry(
    entry: VaultEntry,
    state: State<AppState>,
) -> Result<(), AppError> {
    if state.is_locked() {
        return Err(AppError::VaultLocked);
    }

    let key = state.get_key().ok_or(AppError::VaultLocked)?;
    let conn_guard = state.conn.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or(AppError::VaultLocked)?;

    update_entry(conn, &key, &entry)
}

/// Kaydı sil.
#[tauri::command]
pub fn delete_vault_entry(id: String, state: State<AppState>) -> Result<(), AppError> {
    if state.is_locked() {
        return Err(AppError::VaultLocked);
    }

    let conn_guard = state.conn.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or(AppError::VaultLocked)?;

    delete_entry(conn, &id)
}

/// Tüm kayıtların listesini getir (meta veri düzeyi).
#[tauri::command]
pub fn list_vault_entries(state: State<AppState>) -> Result<Vec<VaultEntryMeta>, AppError> {
    if state.is_locked() {
        return Err(AppError::VaultLocked);
    }

    let key = state.get_key().ok_or(AppError::VaultLocked)?;
    let conn_guard = state.conn.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or(AppError::VaultLocked)?;

    list_entries(conn, &key)
}

/// Tek kaydın tüm alanlarını getir (şifre dahil).
#[tauri::command]
pub fn get_vault_entry(id: String, state: State<AppState>) -> Result<VaultEntry, AppError> {
    if state.is_locked() {
        return Err(AppError::VaultLocked);
    }

    let key = state.get_key().ok_or(AppError::VaultLocked)?;
    let conn_guard = state.conn.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or(AppError::VaultLocked)?;

    get_entry(conn, &key, &id)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordAnalysis {
    pub id: String,
    pub name: String,
    pub is_weak: bool,
    pub is_duplicate: bool,
    pub duplicate_with: Vec<String>,
}

/// Tüm şifreleri analiz eder: zayıf ve tekrar eden şifreleri tespit eder.
#[tauri::command]
pub fn analyze_passwords(state: State<AppState>) -> Result<Vec<PasswordAnalysis>, AppError> {
    if state.is_locked() {
        return Err(AppError::VaultLocked);
    }

    let key = state.get_key().ok_or(AppError::VaultLocked)?;
    let conn_guard = state.conn.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or(AppError::VaultLocked)?;

    let passwords = get_all_passwords(conn, &key)?;

    // Tekrar eden şifreleri tespit et
    let mut analysis: Vec<PasswordAnalysis> = Vec::new();

    for (i, (id_i, pwd_i)) in passwords.iter().enumerate() {
        let mut duplicates = Vec::new();

        for (j, (id_j, pwd_j)) in passwords.iter().enumerate() {
            if i != j && pwd_i == pwd_j {
                duplicates.push(id_j.clone());
            }
        }

        // Basit zayıf şifre tespiti: 8 karakterden kısa
        // (Frontend'de zxcvbn ile daha gelişmiş analiz yapılır)
        let is_weak = pwd_i.len() < 8;

        analysis.push(PasswordAnalysis {
            id: id_i.clone(),
            name: String::new(), // Frontend dolduracak
            is_weak,
            is_duplicate: !duplicates.is_empty(),
            duplicate_with: duplicates,
        });
    }

    Ok(analysis)
}
