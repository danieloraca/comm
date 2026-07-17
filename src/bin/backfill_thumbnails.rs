use std::{
    env, fs,
    path::{Path, PathBuf},
};

use image::codecs::jpeg::JpegEncoder;
use sqlx::{FromRow, SqlitePool, sqlite::SqliteConnectOptions};

#[path = "../attachment_crypto.rs"]
mod attachment_crypto;

use attachment_crypto::AttachmentCrypto;

const DEFAULT_DATABASE_FILE: &str = "comm.sqlite3";
const DEFAULT_ATTACHMENTS_DIR: &str = "attachments";
const THUMBNAIL_MAX_SIDE: u32 = 720;
const THUMBNAIL_QUALITY: u8 = 74;
const THUMBNAIL_MIME_TYPE: &str = "image/jpeg";

#[tokio::main]
async fn main() {
    let database_path =
        env::var("COMM_DATABASE_FILE").unwrap_or_else(|_| DEFAULT_DATABASE_FILE.to_owned());
    let attachments_dir =
        env::var("COMM_ATTACHMENTS_DIR").unwrap_or_else(|_| DEFAULT_ATTACHMENTS_DIR.to_owned());
    let attachments_dir = PathBuf::from(attachments_dir);

    if !Path::new(&database_path).exists() {
        eprintln!("database `{database_path}` does not exist");
        std::process::exit(1);
    }

    let pool = SqlitePool::connect_with(SqliteConnectOptions::new().filename(&database_path))
        .await
        .unwrap_or_else(|error| panic!("failed to open database `{database_path}`: {error}"));
    ensure_attachment_thumbnail_columns(&pool).await;

    let attachments = sqlx::query_as::<_, AttachmentRow>(
        r#"
        SELECT id, stored_name, nonce
        FROM attachments
        WHERE deleted_at IS NULL
        AND thumbnail_stored_name IS NULL
        ORDER BY id ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_else(|error| panic!("failed to load attachments: {error}"));

    if attachments.is_empty() {
        println!("no attachment thumbnails to backfill");
        return;
    }

    let crypto = AttachmentCrypto::load_from_env();
    let mut created = 0usize;
    let mut skipped = 0usize;

    for attachment in attachments {
        match backfill_attachment(&pool, &crypto, &attachments_dir, &attachment).await {
            Ok(()) => {
                created += 1;
                println!("created thumbnail for attachment {}", attachment.id);
            }
            Err(error) => {
                skipped += 1;
                eprintln!("skipped attachment {}: {error}", attachment.id);
            }
        }
    }

    println!("thumbnail backfill complete: {created} created, {skipped} skipped");
}

async fn backfill_attachment(
    pool: &SqlitePool,
    crypto: &AttachmentCrypto,
    attachments_dir: &Path,
    attachment: &AttachmentRow,
) -> Result<(), String> {
    let ciphertext = fs::read(attachments_dir.join(&attachment.stored_name))
        .map_err(|error| format!("failed to read encrypted file: {error}"))?;
    let bytes = crypto
        .decrypt(&ciphertext, &attachment.nonce)
        .map_err(|_| "failed to decrypt original image".to_owned())?;
    let thumbnail = create_thumbnail(&bytes)?;
    let encrypted_thumbnail = crypto
        .encrypt(&thumbnail)
        .map_err(|_| "failed to encrypt thumbnail".to_owned())?;
    let thumbnail_stored_name = random_stored_name();

    fs::write(
        attachments_dir.join(&thumbnail_stored_name),
        encrypted_thumbnail.ciphertext,
    )
    .map_err(|error| format!("failed to write encrypted thumbnail: {error}"))?;

    let result = sqlx::query(
        r#"
        UPDATE attachments
        SET
            thumbnail_stored_name = ?,
            thumbnail_mime_type = ?,
            thumbnail_size_bytes = ?,
            thumbnail_nonce = ?
        WHERE id = ?
        AND thumbnail_stored_name IS NULL
        "#,
    )
    .bind(&thumbnail_stored_name)
    .bind(THUMBNAIL_MIME_TYPE)
    .bind(thumbnail.len() as i64)
    .bind(encrypted_thumbnail.nonce)
    .bind(attachment.id)
    .execute(pool)
    .await
    .map_err(|error| format!("failed to update attachment row: {error}"))?;

    if result.rows_affected() == 0 {
        return Err("attachment already had a thumbnail".to_owned());
    }

    Ok(())
}

fn create_thumbnail(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let image =
        image::load_from_memory(bytes).map_err(|error| format!("unsupported image: {error}"))?;
    let thumbnail = image.thumbnail(THUMBNAIL_MAX_SIDE, THUMBNAIL_MAX_SIDE);
    let mut output = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut output, THUMBNAIL_QUALITY);

    encoder
        .encode_image(&thumbnail)
        .map_err(|error| format!("failed to encode thumbnail: {error}"))?;

    Ok(output)
}

async fn ensure_attachment_thumbnail_columns(pool: &SqlitePool) {
    let existing_columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('attachments')")
            .fetch_all(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to inspect attachment schema: {error}"));

    if !existing_columns
        .iter()
        .any(|column| column == "thumbnail_stored_name")
    {
        sqlx::query("ALTER TABLE attachments ADD COLUMN thumbnail_stored_name TEXT")
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to add thumbnail_stored_name column: {error}"));
    }

    if !existing_columns
        .iter()
        .any(|column| column == "thumbnail_mime_type")
    {
        sqlx::query("ALTER TABLE attachments ADD COLUMN thumbnail_mime_type TEXT")
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to add thumbnail_mime_type column: {error}"));
    }

    if !existing_columns
        .iter()
        .any(|column| column == "thumbnail_size_bytes")
    {
        sqlx::query("ALTER TABLE attachments ADD COLUMN thumbnail_size_bytes INTEGER")
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to add thumbnail_size_bytes column: {error}"));
    }

    if !existing_columns
        .iter()
        .any(|column| column == "thumbnail_nonce")
    {
        sqlx::query("ALTER TABLE attachments ADD COLUMN thumbnail_nonce BLOB")
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to add thumbnail_nonce column: {error}"));
    }
}

fn random_stored_name() -> String {
    let bytes: [u8; 32] = rand::random();
    format!("{}.bin", hex::encode(bytes))
}

#[derive(Debug, FromRow)]
struct AttachmentRow {
    id: i64,
    stored_name: String,
    nonce: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use image::{ImageFormat, Rgb, RgbImage};

    use super::create_thumbnail;

    #[test]
    fn create_thumbnail_outputs_jpeg() {
        let mut image = RgbImage::new(8, 8);
        for pixel in image.pixels_mut() {
            *pixel = Rgb([120, 40, 200]);
        }

        let mut png = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut png), ImageFormat::Png)
            .unwrap();

        let thumbnail = create_thumbnail(&png).unwrap();

        assert!(thumbnail.starts_with(&[0xFF, 0xD8, 0xFF]));
    }
}
