use std::env;

use sqlx::{FromRow, SqlitePool, sqlite::SqliteConnectOptions};

use crate::crypto::{CryptoError, MessageCrypto};

const DEFAULT_DATABASE_FILE: &str = "comm.sqlite3";
const HISTORY_LIMIT: i64 = 100;

#[derive(Clone)]
pub struct MessageStore {
    crypto: MessageCrypto,
    pool: SqlitePool,
}

impl MessageStore {
    pub async fn load_from_env() -> Self {
        let path =
            env::var("COMM_DATABASE_FILE").unwrap_or_else(|_| DEFAULT_DATABASE_FILE.to_string());
        Self::open(&path, MessageCrypto::load_from_env()).await
    }

    async fn open(path: &str, crypto: MessageCrypto) -> Self {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options)
            .await
            .unwrap_or_else(|error| panic!("failed to open database `{path}`: {error}"));

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sender TEXT NOT NULL,
                body TEXT,
                body_ciphertext BLOB,
                body_nonce BLOB,
                deleted_at TEXT,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap_or_else(|error| panic!("failed to create message schema: {error}"));

        ensure_encrypted_columns(&pool).await;

        Self { crypto, pool }
    }

    pub async fn save_message(&self, sender: &str, body: &str) -> sqlx::Result<StoredMessage> {
        let encrypted = self
            .crypto
            .encrypt(body)
            .expect("message encryption should not fail");

        let row = sqlx::query_as::<_, MessageRow>(
            r#"
            INSERT INTO messages (sender, body, body_ciphertext, body_nonce)
            VALUES (?, '', ?, ?)
            RETURNING id, sender, body, body_ciphertext, body_nonce, created_at
            "#,
        )
        .bind(sender)
        .bind(encrypted.ciphertext)
        .bind(encrypted.nonce)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.decrypt_row(row).await)
    }

    pub async fn recent_messages(&self) -> sqlx::Result<Vec<StoredMessage>> {
        let rows = sqlx::query_as::<_, MessageRow>(
            r#"
            SELECT id, sender, body, body_ciphertext, body_nonce, created_at
            FROM (
                SELECT id, sender, body, body_ciphertext, body_nonce, created_at
                FROM messages
                WHERE deleted_at IS NULL
                ORDER BY id DESC
                LIMIT ?
            )
            ORDER BY id ASC
            "#,
        )
        .bind(HISTORY_LIMIT)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::with_capacity(rows.len());
        for row in rows {
            messages.push(self.decrypt_row(row).await);
        }

        Ok(messages)
    }

    pub async fn delete_message(&self, id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn decrypt_row(&self, row: MessageRow) -> StoredMessage {
        let body = match (row.body_ciphertext.as_deref(), row.body_nonce.as_deref()) {
            (Some(ciphertext), Some(nonce)) => self
                .crypto
                .decrypt(ciphertext, nonce)
                .unwrap_or_else(|error| panic!("failed to decrypt message {}: {error:?}", row.id)),
            _ => {
                let plaintext = row.body.unwrap_or_default();
                self.encrypt_existing_row(row.id, &plaintext).await;
                plaintext
            }
        };

        StoredMessage {
            id: row.id,
            sender: row.sender,
            body,
            created_at: row.created_at,
        }
    }

    async fn encrypt_existing_row(&self, id: i64, body: &str) {
        if body.is_empty() {
            return;
        }

        let encrypted = self
            .crypto
            .encrypt(body)
            .expect("message encryption should not fail");

        sqlx::query(
            r#"
            UPDATE messages
            SET body_ciphertext = ?, body_nonce = ?, body = ''
            WHERE id = ?
            "#,
        )
        .bind(encrypted.ciphertext)
        .bind(encrypted.nonce)
        .bind(id)
        .execute(&self.pool)
        .await
        .unwrap_or_else(|error| panic!("failed to encrypt existing message {id}: {error}"));
    }
}

#[derive(Debug)]
pub struct StoredMessage {
    pub id: i64,
    pub sender: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, FromRow)]
struct MessageRow {
    id: i64,
    sender: String,
    body: Option<String>,
    body_ciphertext: Option<Vec<u8>>,
    body_nonce: Option<Vec<u8>>,
    created_at: String,
}

async fn ensure_encrypted_columns(pool: &SqlitePool) {
    let existing_columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('messages')")
            .fetch_all(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to inspect message schema: {error}"));

    if !existing_columns
        .iter()
        .any(|column| column == "body_ciphertext")
    {
        sqlx::query("ALTER TABLE messages ADD COLUMN body_ciphertext BLOB")
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to add body_ciphertext column: {error}"));
    }

    if !existing_columns.iter().any(|column| column == "body_nonce") {
        sqlx::query("ALTER TABLE messages ADD COLUMN body_nonce BLOB")
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to add body_nonce column: {error}"));
    }

    if !existing_columns.iter().any(|column| column == "deleted_at") {
        sqlx::query("ALTER TABLE messages ADD COLUMN deleted_at TEXT")
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to add deleted_at column: {error}"));
    }
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
