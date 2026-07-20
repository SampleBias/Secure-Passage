//! AES-256-GCM session keys for app-layer encryption.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use thiserror::Error;

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug, Error)]
pub enum SessionKeyError {
    #[error("invalid session key encoding")]
    InvalidEncoding,
    #[error("invalid session key length")]
    InvalidLength,
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed")]
    Decrypt,
}

/// 32-byte session key shared out-of-band (copy / QR).
#[derive(Clone)]
pub struct SessionKey {
    key: [u8; KEY_LEN],
}

impl std::fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionKey([redacted])")
    }
}

impl SessionKey {
    pub fn generate() -> Self {
        let mut key = [0u8; KEY_LEN];
        rand::thread_rng().fill_bytes(&mut key);
        Self { key }
    }

    pub fn from_base64(s: &str) -> Result<Self, SessionKeyError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(s.trim())
            .map_err(|_| SessionKeyError::InvalidEncoding)?;
        if bytes.len() != KEY_LEN {
            return Err(SessionKeyError::InvalidLength);
        }
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&bytes);
        Ok(Self { key })
    }

    pub fn to_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.key)
    }

    fn cipher(&self) -> Aes256Gcm {
        Aes256Gcm::new_from_slice(&self.key).expect("32-byte key")
    }

    /// Encrypt plaintext; output is nonce || ciphertext.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SessionKeyError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let mut out = nonce_bytes.to_vec();
        let ct = self
            .cipher()
            .encrypt(nonce, plaintext)
            .map_err(|_| SessionKeyError::Encrypt)?;
        out.extend_from_slice(&ct);
        Ok(out)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SessionKeyError> {
        if data.len() < NONCE_LEN + 1 {
            return Err(SessionKeyError::Decrypt);
        }
        let (nonce_bytes, ct) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher()
            .decrypt(nonce, ct)
            .map_err(|_| SessionKeyError::Decrypt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = SessionKey::generate();
        let enc = key.encrypt(b"hello secure passage").unwrap();
        let dec = key.decrypt(&enc).unwrap();
        assert_eq!(dec, b"hello secure passage");
        let parsed = SessionKey::from_base64(&key.to_base64()).unwrap();
        assert_eq!(parsed.decrypt(&enc).unwrap(), b"hello secure passage");
    }
}
