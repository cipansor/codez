//! Port of `gitea.dev/modules/secret` — AES-CFB secret encryption.
//!
//! | upstream | here |
//! | --- | --- |
//! | `secret.go` | this module |

use aes::cipher::{AsyncStreamCipher, KeyIvInit};
use aes::{Aes128, Aes192, Aes256};
use base64::Engine as _;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// AES block size, used as the IV length.
const AES_BLOCK_SIZE: usize = 16;

/// `secret.AesEncrypt` — AES-CFB over base64(text), with the IV prefixed.
///
/// `aes.NewCipher` selects AES-128/192/256 from the key length, so the same
/// choice is made here; [`encrypt_secret`] always supplies a 32-byte key and so
/// always gets AES-256.
pub fn aes_encrypt(key: &[u8], text: &[u8]) -> Result<Vec<u8>, String> {
    let b = base64::engine::general_purpose::STANDARD.encode(text);

    let mut ciphertext = vec![0u8; AES_BLOCK_SIZE + b.len()];
    let (iv, body) = ciphertext.split_at_mut(AES_BLOCK_SIZE);
    rand::rngs::OsRng.fill_bytes(iv);

    body.copy_from_slice(b.as_bytes());
    cfb_encrypt(key, iv, body)?;

    Ok(ciphertext)
}

/// `secret.AesDecrypt` — reverses [`aes_encrypt`].
pub fn aes_decrypt(key: &[u8], text: &[u8]) -> Result<Vec<u8>, String> {
    if text.len() < AES_BLOCK_SIZE {
        return Err("AesDecrypt ciphertext too short".to_string());
    }
    let (iv, body) = text.split_at(AES_BLOCK_SIZE);
    let mut body = body.to_vec();

    cfb_decrypt(key, iv, &mut body)?;

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&body)
        .map_err(|e| format!("AesDecrypt invalid decrypted base64 string: {e}"))?;
    Ok(decoded)
}

/// `cipher.NewCFBEncrypter`, selecting the AES variant from the key length.
fn cfb_encrypt(key: &[u8], iv: &[u8], data: &mut [u8]) -> Result<(), String> {
    match key.len() {
        16 => cfb_mode::Encryptor::<Aes128>::new(key.into(), iv.into()).encrypt(data),
        24 => cfb_mode::Encryptor::<Aes192>::new(key.into(), iv.into()).encrypt(data),
        32 => cfb_mode::Encryptor::<Aes256>::new(key.into(), iv.into()).encrypt(data),
        _ => return Err(INVALID_KEY_SIZE.to_string()),
    }
    Ok(())
}

/// `cipher.NewCFBDecrypter`, selecting the AES variant from the key length.
fn cfb_decrypt(key: &[u8], iv: &[u8], data: &mut [u8]) -> Result<(), String> {
    match key.len() {
        16 => cfb_mode::Decryptor::<Aes128>::new(key.into(), iv.into()).decrypt(data),
        24 => cfb_mode::Decryptor::<Aes192>::new(key.into(), iv.into()).decrypt(data),
        32 => cfb_mode::Decryptor::<Aes256>::new(key.into(), iv.into()).decrypt(data),
        _ => return Err(INVALID_KEY_SIZE.to_string()),
    }
    Ok(())
}

/// The message Go's `aes.NewCipher` returns for an unsupported key length.
const INVALID_KEY_SIZE: &str = "crypto/aes: invalid key size";

/// `secret.EncryptSecret` — encrypts `str` under `key` into a hex string.
pub fn encrypt_secret(key: &str, value: &str) -> Result<String, String> {
    let key_hash = Sha256::digest(key.as_bytes());
    let ciphertext = aes_encrypt(&key_hash, value.as_bytes())
        .map_err(|e| format!("failed to encrypt by secret: {e}"))?;
    Ok(hex::encode(ciphertext))
}

/// `secret.DecryptSecret` — reverses [`encrypt_secret`].
pub fn decrypt_secret(key: &str, cipher_hex: &str) -> Result<String, String> {
    let key_hash = Sha256::digest(key.as_bytes());
    let ciphertext = hex::decode(cipher_hex)
        .map_err(|e| format!("failed to decrypt by secret, invalid hex string: {e}"))?;
    let plaintext = aes_decrypt(&key_hash, &ciphertext).map_err(|e| {
        format!("failed to decrypt by secret, the key (maybe SECRET_KEY?) might be incorrect: {e}")
    })?;
    String::from_utf8(plaintext).map_err(|e| {
        format!("failed to decrypt by secret, the key (maybe SECRET_KEY?) might be incorrect: {e}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestEncryptDecrypt` in modules/secret/secret_test.go.
    #[test]
    fn encrypt_decrypt_matches_upstream() {
        let hex = encrypt_secret("foo", "baz").unwrap();
        assert_eq!(decrypt_secret("foo", &hex).unwrap(), "baz");

        // A different key must not recover the plaintext.
        let hex = encrypt_secret("bar", "baz").unwrap();
        let got = decrypt_secret("foo", &hex);
        assert!(got.is_err() || got.as_deref() != Ok("baz"));

        let err = decrypt_secret("a", "b").unwrap_err();
        assert!(
            err.contains("invalid hex string"),
            "unexpected error: {err}"
        );

        let err = decrypt_secret("a", "bb").unwrap_err();
        assert!(
            err.contains(
                "the key (maybe SECRET_KEY?) might be incorrect: AesDecrypt ciphertext too short"
            ),
            "unexpected error: {err}"
        );

        let err = decrypt_secret(
            "a",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap_err();
        assert!(
            err.contains(
                "the key (maybe SECRET_KEY?) might be incorrect: AesDecrypt invalid decrypted base64 string"
            ),
            "unexpected error: {err}"
        );
    }

    /// A ciphertext produced by Go's `crypto/cipher` CFB over
    /// `base64("baz")` with key `SHA-256("foo")` and IV `00..0f`. Decrypting it
    /// here proves the wire format matches Gitea's, so stored secrets survive
    /// the port.
    #[test]
    fn decrypts_a_go_generated_ciphertext() {
        let key = Sha256::digest(b"foo");
        let ct = hex::decode("000102030405060708090a0b0c0d0e0f1e0f2529").unwrap();
        assert_eq!(aes_decrypt(&key, &ct).unwrap(), b"baz");
    }

    #[test]
    fn aes_round_trips_and_uses_a_fresh_iv() {
        let key = vec![7u8; 16];
        let a = aes_encrypt(&key, b"hello").unwrap();
        let b = aes_encrypt(&key, b"hello").unwrap();
        assert_ne!(a, b, "a random IV must make the ciphertext differ");
        assert_eq!(aes_decrypt(&key, &a).unwrap(), b"hello");
        assert_eq!(aes_decrypt(&key, &b).unwrap(), b"hello");
    }

    #[test]
    fn aes_rejects_a_short_ciphertext_and_a_bad_key() {
        assert!(aes_decrypt(&[0u8; 16], &[0u8; 8]).is_err());
        assert!(aes_encrypt(&[0u8; 7], b"x").is_err());
    }
}
