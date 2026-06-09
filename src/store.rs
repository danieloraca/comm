use std::{env, fs, path::PathBuf};

use sqlx::{FromRow, SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
    attachment_crypto::AttachmentCrypto,
    crypto::{CryptoError, MessageCrypto},
};

const DEFAULT_DATABASE_FILE: &str = "comm.sqlite3";
const DEFAULT_ATTACHMENTS_DIR: &str = "attachments";
const HISTORY_LIMIT: i64 = 100;
const ACTIVITY_LOG_LIMIT: i64 = 200;

#[derive(Clone)]
pub struct MessageStore {
    attachment_crypto: AttachmentCrypto,
    attachments_dir: PathBuf,
    crypto: MessageCrypto,
    pool: SqlitePool,
}

impl MessageStore {
    pub async fn load_from_env() -> Self {
        let path =
            env::var("COMM_DATABASE_FILE").unwrap_or_else(|_| DEFAULT_DATABASE_FILE.to_string());
        let attachments_dir = env::var("COMM_ATTACHMENTS_DIR")
            .unwrap_or_else(|_| DEFAULT_ATTACHMENTS_DIR.to_string());
        Self::open(
            &path,
            MessageCrypto::load_from_env(),
            AttachmentCrypto::load_from_env(),
            PathBuf::from(attachments_dir),
        )
        .await
    }

    async fn open(
        path: &str,
        crypto: MessageCrypto,
        attachment_crypto: AttachmentCrypto,
        attachments_dir: PathBuf,
    ) -> Self {
        fs::create_dir_all(&attachments_dir).unwrap_or_else(|error| {
            panic!(
                "failed to create attachments directory `{}`: {error}",
                attachments_dir.display()
            )
        });

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
        ensure_message_attachment_column(&pool).await;
        ensure_attachments_table(&pool).await;
        ensure_hidden_messages_table(&pool).await;
        ensure_read_receipts_table(&pool).await;
        ensure_activity_logs_table(&pool).await;

        Self {
            attachment_crypto,
            attachments_dir,
            crypto,
            pool,
        }
    }

    pub async fn save_message(
        &self,
        sender: &str,
        body: &str,
        attachment_id: Option<i64>,
    ) -> sqlx::Result<StoredMessage> {
        if let Some(attachment_id) = attachment_id {
            let exists: Option<i64> = sqlx::query_scalar(
                r#"
                SELECT id
                FROM attachments
                WHERE id = ? AND sender = ? AND deleted_at IS NULL
                "#,
            )
            .bind(attachment_id)
            .bind(sender)
            .fetch_optional(&self.pool)
            .await?;

            if exists.is_none() {
                return Err(sqlx::Error::RowNotFound);
            }
        }

        let encrypted = self
            .crypto
            .encrypt(body)
            .expect("message encryption should not fail");

        let row = sqlx::query_as::<_, MessageRow>(
            r#"
            INSERT INTO messages (sender, body, body_ciphertext, body_nonce, attachment_id)
            VALUES (?, '', ?, ?, ?)
            RETURNING
                id,
                sender,
                body,
                body_ciphertext,
                body_nonce,
                attachment_id,
                created_at,
                NULL AS read_at,
                NULL AS attachment_mime_type,
                NULL AS attachment_original_name,
                NULL AS attachment_size_bytes
            "#,
        )
        .bind(sender)
        .bind(encrypted.ciphertext)
        .bind(encrypted.nonce)
        .bind(attachment_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.decrypt_row(row).await)
    }

    pub async fn recent_messages(&self, username: &str) -> sqlx::Result<Vec<StoredMessage>> {
        let rows = sqlx::query_as::<_, MessageRow>(
            r#"
            SELECT
                id,
                sender,
                body,
                body_ciphertext,
                body_nonce,
                attachment_id,
                created_at,
                read_at,
                attachment_mime_type,
                attachment_original_name,
                attachment_size_bytes
            FROM (
                SELECT
                    messages.id,
                    messages.sender,
                    messages.body,
                    messages.body_ciphertext,
                    messages.body_nonce,
                    messages.attachment_id,
                    messages.created_at,
                    attachments.mime_type AS attachment_mime_type,
                    attachments.original_name AS attachment_original_name,
                    attachments.size_bytes AS attachment_size_bytes,
                    (
                        SELECT max(read_at)
                        FROM read_receipts
                        WHERE message_id = messages.id
                        AND username != messages.sender
                    ) AS read_at
                FROM messages
                LEFT JOIN attachments ON attachments.id = messages.attachment_id
                WHERE messages.deleted_at IS NULL
                AND messages.id NOT IN (
                    SELECT message_id
                    FROM hidden_messages
                    WHERE username = ?
                )
                ORDER BY messages.id DESC
                LIMIT ?
            )
            ORDER BY id ASC
            "#,
        )
        .bind(username)
        .bind(HISTORY_LIMIT)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::with_capacity(rows.len());
        for row in rows {
            messages.push(self.decrypt_row(row).await);
        }

        Ok(messages)
    }

    pub async fn save_attachment(
        &self,
        sender: &str,
        original_name: Option<&str>,
        mime_type: &str,
        bytes: &[u8],
    ) -> sqlx::Result<StoredAttachment> {
        let encrypted = self
            .attachment_crypto
            .encrypt(bytes)
            .expect("attachment encryption should not fail");
        let stored_name = random_stored_name();
        let path = self.attachments_dir.join(&stored_name);

        fs::write(&path, encrypted.ciphertext).map_err(sqlx::Error::Io)?;

        sqlx::query_as::<_, StoredAttachment>(
            r#"
            INSERT INTO attachments (sender, stored_name, original_name, mime_type, size_bytes, nonce)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, mime_type, original_name, size_bytes
            "#,
        )
        .bind(sender)
        .bind(stored_name)
        .bind(original_name)
        .bind(mime_type)
        .bind(bytes.len() as i64)
        .bind(encrypted.nonce)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn attachment_for_user(
        &self,
        username: &str,
        id: i64,
    ) -> sqlx::Result<Option<ServedAttachment>> {
        let Some(row) = sqlx::query_as::<_, AttachmentFileRow>(
            r#"
            SELECT attachments.stored_name, attachments.mime_type, attachments.nonce
            FROM attachments
            JOIN messages ON messages.attachment_id = attachments.id
            WHERE attachments.id = ?
            AND messages.deleted_at IS NULL
            AND attachments.deleted_at IS NULL
            AND messages.id NOT IN (
                SELECT message_id
                FROM hidden_messages
                WHERE username = ?
            )
            LIMIT 1
            "#,
        )
        .bind(id)
        .bind(username)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let ciphertext =
            fs::read(self.attachments_dir.join(&row.stored_name)).map_err(sqlx::Error::Io)?;
        let bytes = self
            .attachment_crypto
            .decrypt(&ciphertext, &row.nonce)
            .expect("failed to decrypt attachment");

        Ok(Some(ServedAttachment {
            bytes,
            mime_type: row.mime_type,
        }))
    }

    pub async fn hide_message_for_user(&self, username: &str, id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO hidden_messages (username, message_id)
            SELECT ?, id
            FROM messages
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(username)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_message_for_everyone(&self, username: &str, id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ? AND sender = ? AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(username)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn mark_message_read(
        &self,
        username: &str,
        id: i64,
    ) -> sqlx::Result<Option<ReadReceipt>> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO read_receipts (message_id, username)
            SELECT id, ?
            FROM messages
            WHERE id = ?
            AND sender != ?
            AND deleted_at IS NULL
            AND id NOT IN (
                SELECT message_id
                FROM hidden_messages
                WHERE username = ?
            )
            "#,
        )
        .bind(username)
        .bind(id)
        .bind(username)
        .bind(username)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, ReadReceipt>(
            r#"
            SELECT read_receipts.message_id, read_receipts.username, read_receipts.read_at
            FROM read_receipts
            JOIN messages ON messages.id = read_receipts.message_id
            WHERE read_receipts.message_id = ?
            AND read_receipts.username = ?
            AND messages.sender != ?
            AND messages.deleted_at IS NULL
            AND messages.id NOT IN (
                SELECT message_id
                FROM hidden_messages
                WHERE username = ?
            )
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(username)
        .bind(username)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn record_activity_log(
        &self,
        username: &str,
        action: &str,
    ) -> sqlx::Result<ActivityLog> {
        sqlx::query_as::<_, ActivityLog>(
            r#"
            INSERT INTO activity_logs (username, action)
            VALUES (?, ?)
            RETURNING occurred_at, username, action
            "#,
        )
        .bind(username)
        .bind(action)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn recent_activity_logs(&self) -> sqlx::Result<Vec<ActivityLog>> {
        sqlx::query_as::<_, ActivityLog>(
            r#"
            SELECT occurred_at, username, action
            FROM (
                SELECT id, occurred_at, username, action
                FROM activity_logs
                ORDER BY id DESC
                LIMIT ?
            )
            ORDER BY id ASC
            "#,
        )
        .bind(ACTIVITY_LOG_LIMIT)
        .fetch_all(&self.pool)
        .await
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
            attachment: row.attachment_id.map(|id| StoredAttachment {
                id,
                mime_type: row.attachment_mime_type.unwrap_or_default(),
                original_name: row.attachment_original_name,
                size_bytes: row.attachment_size_bytes.unwrap_or_default(),
            }),
            created_at: row.created_at,
            read_at: row.read_at,
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

async fn ensure_hidden_messages_table(pool: &SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS hidden_messages (
            username TEXT NOT NULL,
            message_id INTEGER NOT NULL,
            hidden_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            PRIMARY KEY (username, message_id),
            FOREIGN KEY (message_id) REFERENCES messages(id)
        )
        "#,
    )
    .execute(pool)
    .await
    .unwrap_or_else(|error| panic!("failed to create hidden message schema: {error}"));
}

async fn ensure_read_receipts_table(pool: &SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS read_receipts (
            message_id INTEGER NOT NULL,
            username TEXT NOT NULL,
            read_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            PRIMARY KEY (message_id, username),
            FOREIGN KEY (message_id) REFERENCES messages(id)
        )
        "#,
    )
    .execute(pool)
    .await
    .unwrap_or_else(|error| panic!("failed to create read receipt schema: {error}"));
}

async fn ensure_attachments_table(pool: &SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sender TEXT NOT NULL,
            stored_name TEXT NOT NULL UNIQUE,
            original_name TEXT,
            mime_type TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            nonce BLOB NOT NULL,
            deleted_at TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        )
        "#,
    )
    .execute(pool)
    .await
    .unwrap_or_else(|error| panic!("failed to create attachment schema: {error}"));
}

async fn ensure_activity_logs_table(pool: &SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activity_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
            username TEXT NOT NULL,
            action TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .unwrap_or_else(|error| panic!("failed to create activity log schema: {error}"));
}

#[derive(Debug)]
pub struct StoredMessage {
    pub id: i64,
    pub sender: String,
    pub body: String,
    pub attachment: Option<StoredAttachment>,
    pub created_at: String,
    pub read_at: Option<String>,
}

#[derive(Clone, Debug, FromRow)]
pub struct StoredAttachment {
    pub id: i64,
    pub mime_type: String,
    pub original_name: Option<String>,
    pub size_bytes: i64,
}

pub struct ServedAttachment {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct ReadReceipt {
    pub message_id: i64,
    pub username: String,
    pub read_at: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct ActivityLog {
    pub occurred_at: String,
    pub username: String,
    pub action: String,
}

#[derive(Debug, FromRow)]
struct MessageRow {
    id: i64,
    sender: String,
    body: Option<String>,
    body_ciphertext: Option<Vec<u8>>,
    body_nonce: Option<Vec<u8>>,
    attachment_id: Option<i64>,
    created_at: String,
    read_at: Option<String>,
    attachment_mime_type: Option<String>,
    attachment_original_name: Option<String>,
    attachment_size_bytes: Option<i64>,
}

#[derive(Debug, FromRow)]
struct AttachmentFileRow {
    stored_name: String,
    mime_type: String,
    nonce: Vec<u8>,
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

async fn ensure_message_attachment_column(pool: &SqlitePool) {
    let existing_columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('messages')")
            .fetch_all(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to inspect message schema: {error}"));

    if !existing_columns
        .iter()
        .any(|column| column == "attachment_id")
    {
        sqlx::query("ALTER TABLE messages ADD COLUMN attachment_id INTEGER")
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to add attachment_id column: {error}"));
    }
}

fn random_stored_name() -> String {
    let bytes: [u8; 32] = rand::random();
    format!("{}.bin", hex::encode(bytes))
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
