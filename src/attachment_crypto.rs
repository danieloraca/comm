use std::{env, fs, path::Path};

use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};

const DEFAULT_ATTACHMENT_KEY_FILE: &str = "attachment.key";

#[derive(Clone)]
pub struct AttachmentCrypto {
    cipher: XChaCha20Poly1305,
}

impl AttachmentCrypto {
    pub fn load_from_env() -> Self {
        let path = env::var("COMM_ATTACHMENT_KEY_FILE")
            .unwrap_or_else(|_| DEFAULT_ATTACHMENT_KEY_FILE.to_string());
        Self::load_or_create(&path)
    }

    fn load_or_create(path: &str) -> Self {
        let key = if Path::new(path).exists() {
            read_key(path)
        } else {
            create_key(path)
        };

        Self {
            cipher: XChaCha20Poly1305::new(&key.into()),
        }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedAttachment, AttachmentCryptoError> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| AttachmentCryptoError::Encrypt)?;

        Ok(EncryptedAttachment {
            ciphertext,
            nonce: nonce.to_vec(),
        })
    }

    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, AttachmentCryptoError> {
        let nonce = XNonce::from_slice(nonce);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| AttachmentCryptoError::Decrypt)
    }
}

pub struct EncryptedAttachment {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Debug)]
pub enum AttachmentCryptoError {
    Decrypt,
    Encrypt,
}

fn read_key(path: &str) -> [u8; 32] {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read attachment key file `{path}`: {error}"));
    decode_key(path, contents.trim())
}

fn create_key(path: &str) -> [u8; 32] {
    let key = XChaCha20Poly1305::generate_key(&mut OsRng);
    fs::write(path, hex::encode(key))
        .unwrap_or_else(|error| panic!("failed to write attachment key file `{path}`: {error}"));

    key.into()
}

fn decode_key(path: &str, encoded: &str) -> [u8; 32] {
    let bytes = hex::decode(encoded)
        .unwrap_or_else(|error| panic!("attachment key file `{path}` is not valid hex: {error}"));
    bytes
        .try_into()
        .unwrap_or_else(|_| panic!("attachment key file `{path}` must contain a 32-byte hex key"))
}
