// schema.rs — SQLite Veritabanı Şeması
//
// Tasarım kararları:
// - Hassas alanlar (password, username, notes, totp_secret) TEXT olarak saklanır,
//   ancak içerikleri AES-256-GCM ile şifrelenmiş base64 string'lerdir.
// - name ve url alanları düz metin: arama/filtreleme için gerekli.
//   Bu alanlar hassas kabul edilmez (site adı ve URL bilgisi).
// - Her kayıt için UUID (TEXT) primary key: platform bağımsız, çakışma riski yok.
// - created_at ve updated_at UNIX timestamp (INTEGER): hafif, sıralama için ideal.

use rusqlite::{Connection, Result};

/// Veritabanı şemasını oluşturur (idempotent — birden fazla çağrılabilir).
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;
        PRAGMA synchronous=NORMAL;

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS vault_entries (
            id           TEXT PRIMARY KEY NOT NULL,
            name         TEXT NOT NULL,
            username_enc TEXT,
            password_enc TEXT,
            url          TEXT,
            notes_enc    TEXT,
            category     TEXT DEFAULT 'Genel',
            tags         TEXT DEFAULT '',
            totp_enc     TEXT,
            is_favorite  INTEGER DEFAULT 0,
            created_at   INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_vault_entries_name
            ON vault_entries(name);
        CREATE INDEX IF NOT EXISTS idx_vault_entries_category
            ON vault_entries(category);
        CREATE INDEX IF NOT EXISTS idx_vault_entries_updated_at
            ON vault_entries(updated_at DESC);
    ")?;

    Ok(())
}
