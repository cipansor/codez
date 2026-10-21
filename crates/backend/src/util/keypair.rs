//! Port of `modules/util/keypair.go`.

use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::EncodePublicKey;
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};

/// `util.GenerateKeyPair` — returns `(private_pem, public_pem)`.
///
/// The PEM block types match upstream exactly (`RSA PRIVATE KEY` in PKCS#1
/// form, `PUBLIC KEY` in PKIX form), because consumers parse these strings.
pub fn generate_key_pair(bits: usize) -> Result<(String, String), String> {
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, bits).map_err(|e| format!("generate key: {e}"))?;
    let priv_pem = pem_block_for_priv(&priv_key)?;
    let pub_pem = pem_block_for_pub(&priv_key.to_public_key())?;
    Ok((priv_pem, pub_pem))
}

/// `util.pemBlockForPriv` — PKCS#1 `RSA PRIVATE KEY` PEM.
fn pem_block_for_priv(priv_key: &RsaPrivateKey) -> Result<String, String> {
    let doc = priv_key
        .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
        .map_err(|e| format!("encode private key: {e}"))?;
    Ok(doc.to_string())
}

/// `util.pemBlockForPub` — PKIX `PUBLIC KEY` PEM.
fn pem_block_for_pub(pub_key: &RsaPublicKey) -> Result<String, String> {
    let doc = pub_key
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .map_err(|e| format!("encode public key: {e}"))?;
    Ok(doc)
}

/// `util.CreatePublicKeyFingerprint` — SHA-256 of the PKIX DER encoding.
pub fn create_public_key_fingerprint(pub_key: &RsaPublicKey) -> Result<Vec<u8>, String> {
    let der = pub_key
        .to_public_key_der()
        .map_err(|e| format!("encode public key: {e}"))?;
    Ok(super::misc::fingerprint_of_pkix_der(der.as_bytes()))
}

/// Public exponent accessor, used by tests to sanity-check a generated key.
pub fn public_exponent(pub_key: &RsaPublicKey) -> Vec<u8> {
    pub_key.e().to_bytes_be()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::pkcs8::DecodePublicKey;

    // Transcribed from `TestKeygen` in modules/util/keypair_test.go.
    #[test]
    fn keygen_produces_pem_blocks() {
        let (priv_pem, pub_pem) = generate_key_pair(2048).unwrap();

        assert!(!priv_pem.is_empty());
        assert!(!pub_pem.is_empty());
        assert!(
            priv_pem.starts_with("-----BEGIN RSA PRIVATE KEY-----"),
            "private key header: {}",
            priv_pem.lines().next().unwrap_or("")
        );
        assert!(
            pub_pem.starts_with("-----BEGIN PUBLIC KEY-----"),
            "public key header: {}",
            pub_pem.lines().next().unwrap_or("")
        );
    }

    // Transcribed from `TestSignUsingKeys` in modules/util/keypair_test.go:
    // the PEM strings must parse back into usable keys that verify a signature.
    #[test]
    fn keys_parse_back_and_verify_a_signature() {
        use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
        use rsa::signature::{SignatureEncoding, Signer, Verifier};
        use sha2::Sha256;

        let (priv_pem, pub_pem) = generate_key_pair(2048).unwrap();

        let priv_key = RsaPrivateKey::from_pkcs1_pem(&priv_pem).expect("parse private pem");
        let pub_key = RsaPublicKey::from_public_key_pem(&pub_pem).expect("parse public pem");

        let signing_key = SigningKey::<Sha256>::new(priv_key);
        let sig = signing_key.sign(b"activity pub is great!");

        let verifying_key = VerifyingKey::<Sha256>::new(pub_key);
        let parsed = Signature::try_from(sig.to_bytes().as_ref()).expect("parse signature");
        verifying_key
            .verify(b"activity pub is great!", &parsed)
            .expect("signature must verify");
    }

    #[test]
    fn fingerprint_is_stable_and_32_bytes() {
        let (_, pub_pem) = generate_key_pair(2048).unwrap();
        let pub_key = RsaPublicKey::from_public_key_pem(&pub_pem).unwrap();
        let fp = create_public_key_fingerprint(&pub_key).unwrap();
        assert_eq!(fp.len(), 32);
        assert_eq!(fp, create_public_key_fingerprint(&pub_key).unwrap());
    }

    #[test]
    fn public_exponent_is_65537() {
        let (_, pub_pem) = generate_key_pair(2048).unwrap();
        let pub_key = RsaPublicKey::from_public_key_pem(&pub_pem).unwrap();
        assert_eq!(public_exponent(&pub_key), vec![0x01, 0x00, 0x01]);
    }
}
