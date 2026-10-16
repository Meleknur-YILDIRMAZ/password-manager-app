// vault.rs — Vault Kayıt CRUD İşlemleri
//
// Tüm hassas alanlar (username, password, notes, totp_secret) bu katmanda
// şifrelenerek veritabanına yazılır ve okunurken çözülür.
// UI katmanı hiçbir zaman ham şifreli veri görmez.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::crypto::{decrypt, encrypt};
use crate::errors::AppError;

/// UI'a gönderilen / UI'dan alınan vault kaydı (plaintext, bellekte kısa süre yaşar)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultEntry {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub category: String,
    pub tags: String,
    pub totp_secret: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Sadece liste görünümü için — şifre gibi hassas alanlar dahil değil
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultEntryMeta {
    pub id: String,
    pub name: String,
    pub username: Option<String>, // Görüntülemek için çözülür (şifre değil)
    pub url: Option<String>,
    pub category: String,
    pub tags: String,
    pub is_favorite: bool,
    pub has_totp: bool,
    pub updated_at: i64,
}

/// Yeni kayıt oluşturur (hassas alanlar şifrelenmiş olarak saklanır).
pub fn create_entry(
    conn: &Connection,
    key: &[u8; 32],
    entry: &VaultEntry,
) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = unix_timestamp();

    let username_enc = encrypt_opt(key, entry.username.as_deref())?;
    let password_enc = encrypt_opt(key, entry.password.as_deref())?;
    let notes_enc = encrypt_opt(key, entry.notes.as_deref())?;
    let totp_enc = encrypt_opt(key, entry.totp_secret.as_deref())?;

    conn.execute(
        "INSERT INTO vault_entries
            (id, name, username_enc, password_enc, url, notes_enc, category, tags, totp_enc, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            id, entry.name,
            username_enc, password_enc,
            entry.url, notes_enc,
            entry.category, entry.tags,
            totp_enc,
            entry.is_favorite as i64,
            now, now
        ],
    )
    .map_err(AppError::from)?;

    Ok(id)
}

/// Mevcut kaydı günceller.
pub fn update_entry(
    conn: &Connection,
    key: &[u8; 32],
    entry: &VaultEntry,
) -> Result<(), AppError> {
    let now = unix_timestamp();

    let username_enc = encrypt_opt(key, entry.username.as_deref())?;
    let password_enc = encrypt_opt(key, entry.password.as_deref())?;
    let notes_enc = encrypt_opt(key, entry.notes.as_deref())?;
    let totp_enc = encrypt_opt(key, entry.totp_secret.as_deref())?;

    let rows = conn.execute(
        "UPDATE vault_entries SET
            name = ?1, username_enc = ?2, password_enc = ?3, url = ?4,
            notes_enc = ?5, category = ?6, tags = ?7, totp_enc = ?8,
            is_favorite = ?9, updated_at = ?10
         WHERE id = ?11",
        rusqlite::params![
            entry.name, username_enc, password_enc,
            entry.url, notes_enc,
            entry.category, entry.tags,
            totp_enc,
            entry.is_favorite as i64,
            now, entry.id
        ],
    )
    .map_err(AppError::from)?;

    if rows == 0 {
        return Err(AppError::NotFound(entry.id.clone()));
    }

    Ok(())
}

/// Kaydı siler.
pub fn delete_entry(conn: &Connection, id: &str) -> Result<(), AppError> {
    let rows = conn
        .execute("DELETE FROM vault_entries WHERE id = ?1", rusqlite::params![id])
        .map_err(AppError::from)?;

    if rows == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    Ok(())
}

/// Tüm kayıtların meta verilerini getirir (username çözülür, şifre çözülmez).
pub fn list_entries(conn: &Connection, key: &[u8; 32]) -> Result<Vec<VaultEntryMeta>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, username_enc, url, category, tags, is_favorite, totp_enc, updated_at
             FROM vault_entries
             ORDER BY is_favorite DESC, updated_at DESC",
        )
        .map_err(AppError::from)?;

    let entries = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, i64>(8)?,
            ))
        })
        .map_err(AppError::from)?
        .filter_map(|r| r.ok())
        .map(|(id, name, username_enc, url, category, tags, is_fav, totp_enc, updated_at)| {
            let username = username_enc
                .as_deref()
                .and_then(|enc| decrypt(key, enc).ok())
                .and_then(|b| String::from_utf8(b.to_vec()).ok());

            Ok(VaultEntryMeta {
                id,
                name,
                username,
                url,
                category,
                tags,
                is_favorite: is_fav != 0,
                has_totp: totp_enc.is_some(),
                updated_at,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(entries)
}

/// Tek bir kaydın tüm alanlarını (şifre dahil) çözerek getirir.
pub fn get_entry(conn: &Connection, key: &[u8; 32], id: &str) -> Result<VaultEntry, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, username_enc, password_enc, url, notes_enc,
                    category, tags, totp_enc, is_favorite, created_at, updated_at
             FROM vault_entries WHERE id = ?1",
        )
        .map_err(AppError::from)?;

    let entry = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, i64>(11)?,
            ))
        })
        .map_err(|_| AppError::NotFound(id.to_string()))?;

    let (eid, name, username_enc, password_enc, url, notes_enc,
         category, tags, totp_enc, is_fav, created_at, updated_at) = entry;

    let username = decrypt_opt(key, username_enc.as_deref())?;
    let password = decrypt_opt(key, password_enc.as_deref())?;
    let notes = decrypt_opt(key, notes_enc.as_deref())?;
    let totp_secret = decrypt_opt(key, totp_enc.as_deref())?;

    Ok(VaultEntry {
        id: eid,
        name,
        username,
        password,
        url,
        notes,
        category,
        tags,
        totp_secret,
        is_favorite: is_fav != 0,
        created_at,
        updated_at,
    })
}

/// Tüm kayıtların şifrelerini getirir (zayıf/tekrar şifre tespiti için).
pub fn get_all_passwords(conn: &Connection, key: &[u8; 32]) -> Result<Vec<(String, String)>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, password_enc FROM vault_entries WHERE password_enc IS NOT NULL")
        .map_err(AppError::from)?;

    let results = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(AppError::from)?
        .filter_map(|r| r.ok())
        .filter_map(|(id, enc)| {
            decrypt(key, &enc)
                .ok()
                .and_then(|b| String::from_utf8(b.to_vec()).ok())
                .map(|pwd| (id, pwd))
        })
        .collect();

    Ok(results)
}

// ─── Yardımcı fonksiyonlar ───────────────────────────────────────────────────

fn encrypt_opt(key: &[u8; 32], value: Option<&str>) -> Result<Option<String>, AppError> {
    match value {
        Some(v) if !v.is_empty() => Ok(Some(encrypt(key, v.as_bytes())?)),
        _ => Ok(None),
    }
}

fn decrypt_opt(key: &[u8; 32], value: Option<&str>) -> Result<Option<String>, AppError> {
    match value {
        Some(enc) => {
            let bytes = decrypt(key, enc)?;
            String::from_utf8(bytes.to_vec())
                .map(Some)
                .map_err(|e| AppError::Crypto(format!("UTF-8 hatası: {}", e)))
        }
        None => Ok(None),
    }
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
