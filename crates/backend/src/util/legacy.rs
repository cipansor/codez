//! Port of `modules/util/legacy.go`.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes128Gcm, Aes256Gcm, Nonce};
use rand::RngCore;
use std::io;
use std::path::Path;

/// GCM nonce size, matching Go's `cipher.NewGCM`.
const NONCE_SIZE: usize = 12;

/// `util.CopyFile` — copies `src` to `dest`, preserving mtime and mode.
pub fn copy_file(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dest = dest.as_ref();

    let meta = std::fs::metadata(src)?;
    std::fs::copy(src, dest)?;

    if let Ok(mtime) = meta.modified() {
        let times = std::fs::FileTimes::new().set_modified(mtime);
        let f = std::fs::File::options().write(true).open(dest)?;
        f.set_times(times)?;
    }
    std::fs::set_permissions(dest, meta.permissions())?;
    Ok(())
}

/// `util.AESGCMEncrypt` — AES-GCM with the nonce prefixed to the ciphertext.
///
/// Output layout is `nonce || ciphertext || tag`, matching upstream. Go's
/// `aes.NewCipher` selects AES-128/192/256 from the key length; this port
/// supports the 128- and 256-bit cases, which is what Gitea's callers use.
pub fn aes_gcm_encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = match key.len() {
        16 => Aes128Gcm::new_from_slice(key)
            .map_err(|e| e.to_string())?
            .encrypt(nonce, plaintext),
        32 => Aes256Gcm::new_from_slice(key)
            .map_err(|e| e.to_string())?
            .encrypt(nonce, plaintext),
        24 => return Err("unsupported key size 24: use 16 or 32 bytes".to_string()),
        n => return Err(format!("invalid key size: {n}")),
    }
    .map_err(|e| format!("aes-gcm encrypt: {e}"))?;

    let mut out = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// `util.AESGCMDecrypt` — reverses [`aes_gcm_encrypt`].
pub fn aes_gcm_decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    // Upstream checks `len(ciphertext)-size <= 0`, so a buffer of exactly the
    // nonce size is rejected as empty.
    if ciphertext.len() <= NONCE_SIZE {
        return Err("ciphertext is empty".to_string());
    }
    let (nonce, body) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = match key.len() {
        16 => Aes128Gcm::new_from_slice(key)
            .map_err(|e| e.to_string())?
            .decrypt(nonce, body),
        32 => Aes256Gcm::new_from_slice(key)
            .map_err(|e| e.to_string())?
            .decrypt(nonce, body),
        24 => return Err("unsupported key size 24: use 16 or 32 bytes".to_string()),
        n => return Err(format!("invalid key size: {n}")),
    }
    .map_err(|e| format!("aes-gcm decrypt: {e}"))?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestCopyFile` in modules/util/legacy_test.go.
    #[test]
    fn copy_file_preserves_content() {
        let content = b"hello";
        let dir = std::env::temp_dir().join(format!("codez-copy-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.txt");
        let dst = dir.join("dst.txt");
        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);

        std::fs::write(&src, content).unwrap();
        copy_file(&src, &dst).unwrap();
        assert_eq!(std::fs::read(&dst).unwrap(), content);

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
        let _ = std::fs::remove_dir(&dir);
    }

    // Transcribed from `TestAESGCM` in modules/util/legacy_test.go, which uses
    // `aes.BlockSize` (16) bytes of random key material.
    #[test]
    fn aes_gcm_round_trips_with_a_16_byte_key() {
        let mut key = vec![0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut key);
        let plaintext = b"this will be encrypted";

        let ciphertext = aes_gcm_encrypt(&key, plaintext).unwrap();
        assert_ne!(ciphertext.as_slice(), plaintext.as_slice());
        assert_eq!(aes_gcm_decrypt(&key, &ciphertext).unwrap(), plaintext);
    }

    #[test]
    fn aes_gcm_round_trips_with_a_32_byte_key() {
        let mut key = vec![0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        let plaintext = b"another secret";
        let ciphertext = aes_gcm_encrypt(&key, plaintext).unwrap();
        assert_eq!(aes_gcm_decrypt(&key, &ciphertext).unwrap(), plaintext);
    }

    #[test]
    fn aes_gcm_rejects_an_empty_ciphertext() {
        let key = vec![0u8; 16];
        assert!(aes_gcm_decrypt(&key, &[]).is_err());
        assert!(aes_gcm_decrypt(&key, &[0u8; NONCE_SIZE]).is_err());
    }

    #[test]
    fn aes_gcm_rejects_a_wrong_key() {
        let key = vec![1u8; 16];
        let other = vec![2u8; 16];
        let ciphertext = aes_gcm_encrypt(&key, b"secret").unwrap();
        assert!(aes_gcm_decrypt(&other, &ciphertext).is_err());
    }

    #[test]
    fn aes_gcm_rejects_an_invalid_key_size() {
        assert!(aes_gcm_encrypt(&[0u8; 7], b"x").is_err());
    }
}
