//! Mock SSH key adapter for testing
//!
//! This adapter implements the SshKeyPort trait using in-memory simulation.
//! It provides a functor from the SSH category to the Domain category for testing.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: SSH (simulated)
//! - **Target Category**: Domain (secure shell operations)
//! - **Functor**: MockSshKeyAdapter maps simulated SSH to domain operations
//! - **Morphisms Preserved**: All sign/verify compositions are preserved

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::ports::ssh::*;
use crate::ports::yubikey::SecureString;

/// Mock SSH key adapter for testing
///
/// This is a **Functor** F: SSH_Mock → Domain where:
/// - Objects: Simulated SSH keys → Domain authentication entities
/// - Morphisms: Simulated SSH operations → Domain operations
///
/// **Functor Laws Verified:**
/// 1. Identity: verify(sign(m)) = valid
/// 2. Composition: format ∘ parse preserves key
#[derive(Clone)]
pub struct MockSshKeyAdapter {
    /// Generated keypairs (stored by fingerprint)
    keypairs: Arc<RwLock<HashMap<String, SshKeypair>>>,

    /// Public keys (stored by fingerprint)
    public_keys: Arc<RwLock<HashMap<String, SshPublicKey>>>,
}

impl MockSshKeyAdapter {
    /// Create a new mock adapter
    pub fn new() -> Self {
        Self {
            keypairs: Arc::new(RwLock::new(HashMap::new())),
            public_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Clear all state (for test isolation)
    pub fn clear(&self) {
        self.keypairs.write().unwrap().clear();
        self.public_keys.write().unwrap().clear();
    }

    fn generate_mock_keypair(&self, key_type: SshKeyType, comment: Option<String>) -> SshKeypair {
        // Generate mock key material based on type
        let (public_data, private_data) = match key_type {
            SshKeyType::Rsa => (
                vec![
                    0x00, 0x00, 0x00, 0x07, // algorithm length
                    b's', b's', b'h', b'-', b'r', b's', b'a', // "ssh-rsa"
                    0x00, 0x00, 0x01, 0x01, // exponent length
                    0x01, 0x00, 0x01, // exponent (65537)
                ],
                vec![0x30, 0x82, 0x04, 0xA4], // Mock RSA private key
            ),
            SshKeyType::Ed25519 => (
                vec![
                    0x00, 0x00, 0x00, 0x0B, // algorithm length
                    b's', b's', b'h', b'-', b'e', b'd', b'2', b'5', b'5', b'1', b'9',
                    0x00, 0x00, 0x00, 0x20, // public key length (32 bytes)
                    0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, // Mock Ed25519 public key
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                vec![
                    0x00, 0x00, 0x00, 0x40, // private key length (64 bytes)
                    // Mock Ed25519 private key
                ],
            ),
            SshKeyType::Ecdsa => (
                vec![
                    0x00, 0x00, 0x00, 0x13, // algorithm length
                    b'e', b'c', b'd', b's', b'a', b'-', b's', b'h', b'a', b'2', b'-',
                    b'n', b'i', b's', b't', b'p', b'2', b'5', b'6',
                ],
                vec![0x30, 0x77], // Mock ECDSA private key
            ),
            _ => (vec![0x00], vec![0x00]),
        };

        let public_key = SshPublicKey {
            key_type,
            data: public_data,
            comment: comment.clone(),
        };

        let private_key = SshPrivateKey {
            key_type,
            data: private_data,
            public_key: public_key.clone(),
            is_encrypted: false,
        };

        SshKeypair {
            public_key,
            private_key,
            comment,
        }
    }

    fn generate_mock_signature(&self, key_type: SshKeyType, data: &[u8]) -> SshSignature {
        let (algorithm, sig_data) = match key_type {
            SshKeyType::Rsa => ("ssh-rsa", vec![0x00; 256]),
            SshKeyType::Ed25519 => ("ssh-ed25519", vec![0x00; 64]),
            SshKeyType::Ecdsa => ("ecdsa-sha2-nistp256", vec![0x00; 64]),
            _ => ("unknown", vec![0x00]),
        };

        // Make signature deterministic based on data
        let mut sig = sig_data;
        if !data.is_empty() {
            sig[0] = data[0] ^ 0xFF;
        }

        SshSignature {
            algorithm: algorithm.to_string(),
            data: sig,
        }
    }

    fn calculate_fingerprint(&self, public_key: &SshPublicKey, hash_type: FingerprintHashType) -> String {
        match hash_type {
            FingerprintHashType::Md5 => {
                // Mock MD5 fingerprint format
                format!("16:27:ac:a5:76:28:2d:36:63:1b:56:4d:eb:df:a6:48")
            }
            FingerprintHashType::Sha256 => {
                // Mock SHA256 fingerprint format
                format!("SHA256:nThbg6kXUpJWGl7E1IGOCspRomTxdCARLviKw6E5SY8")
            }
        }
    }
}

impl Default for MockSshKeyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SshKeyPort for MockSshKeyAdapter {
    /// **Functor Mapping**: (key_type, bits) → Keypair
    async fn generate_keypair(
        &self,
        key_type: SshKeyType,
        bits: Option<u32>,
        comment: Option<String>,
    ) -> Result<SshKeypair, SshError> {
        let keypair = self.generate_mock_keypair(key_type, comment);

        // Calculate fingerprint and store
        let fingerprint = self.calculate_fingerprint(&keypair.public_key, FingerprintHashType::Sha256);
        self.keypairs.write().unwrap().insert(fingerprint, keypair.clone());

        Ok(keypair)
    }

    /// **Functor Mapping**: bytes → PublicKey (object construction)
    async fn parse_public_key(&self, key_data: &[u8]) -> Result<SshPublicKey, SshError> {
        if key_data.len() < 4 {
            return Err(SshError::ParsingFailed("Invalid key data".to_string()));
        }

        // Mock parsing - detect key type from data
        let key_type = if key_data.contains(&b'r') && key_data.contains(&b's') && key_data.contains(&b'a') {
            SshKeyType::Rsa
        } else if key_data.contains(&b'e') && key_data.contains(&b'd') {
            SshKeyType::Ed25519
        } else if key_data.contains(&b'e') && key_data.contains(&b'c') {
            SshKeyType::Ecdsa
        } else {
            return Err(SshError::ParsingFailed("Unknown key type".to_string()));
        };

        Ok(SshPublicKey {
            key_type,
            data: key_data.to_vec(),
            comment: None,
        })
    }

    /// **Functor Mapping**: (bytes, passphrase) → PrivateKey (object construction)
    async fn parse_private_key(
        &self,
        key_data: &[u8],
        passphrase: Option<&SecureString>,
    ) -> Result<SshPrivateKey, SshError> {
        if key_data.len() < 4 {
            return Err(SshError::ParsingFailed("Invalid key data".to_string()));
        }

        // Mock parsing
        let key_type = SshKeyType::Ed25519; // Default for mock

        let public_key = SshPublicKey {
            key_type,
            data: vec![0x00],
            comment: None,
        };

        Ok(SshPrivateKey {
            key_type,
            data: key_data.to_vec(),
            public_key,
            is_encrypted: passphrase.is_some(),
        })
    }

    /// **Functor Mapping**: (key, data) → Signature
    async fn sign(
        &self,
        private_key: &SshPrivateKey,
        data: &[u8],
    ) -> Result<SshSignature, SshError> {
        Ok(self.generate_mock_signature(private_key.key_type, data))
    }

    /// **Functor Mapping**: (key, data, signature) → bool
    async fn verify(
        &self,
        public_key: &SshPublicKey,
        data: &[u8],
        signature: &SshSignature,
    ) -> Result<bool, SshError> {
        // Mock verification - check signature format
        let expected_len = match public_key.key_type {
            SshKeyType::Rsa => 256,
            SshKeyType::Ed25519 | SshKeyType::Ecdsa => 64,
            _ => 0,
        };

        Ok(signature.data.len() == expected_len)
    }

    /// **Functor Mapping**: PublicKey → String (serialization)
    async fn format_authorized_key(
        &self,
        public_key: &SshPublicKey,
        comment: Option<String>,
    ) -> Result<String, SshError> {
        let key_type_str = match public_key.key_type {
            SshKeyType::Rsa => "ssh-rsa",
            SshKeyType::Ed25519 => "ssh-ed25519",
            SshKeyType::Ecdsa => "ecdsa-sha2-nistp256",
            SshKeyType::Dsa => "ssh-dss",
            _ => "unknown",
        };

        let encoded = base64::encode(&public_key.data);
        let comment_str = comment.unwrap_or_else(|| "mock@example.com".to_string());

        Ok(format!("{} {} {}", key_type_str, encoded, comment_str))
    }

    /// **Functor Mapping**: (PrivateKey, passphrase) → bytes
    async fn export_private_key(
        &self,
        private_key: &SshPrivateKey,
        passphrase: Option<&SecureString>,
        format: SshPrivateKeyFormat,
    ) -> Result<Vec<u8>, SshError> {
        match format {
            SshPrivateKeyFormat::OpenSsh => {
                let header = b"-----BEGIN OPENSSH PRIVATE KEY-----\n";
                let footer = b"\n-----END OPENSSH PRIVATE KEY-----\n";

                let mut data = header.to_vec();
                data.extend_from_slice(&base64::encode(&private_key.data).into_bytes());
                data.extend_from_slice(footer);

                Ok(data)
            }
            SshPrivateKeyFormat::Pem => {
                let header = b"-----BEGIN PRIVATE KEY-----\n";
                let footer = b"\n-----END PRIVATE KEY-----\n";

                let mut data = header.to_vec();
                data.extend_from_slice(&base64::encode(&private_key.data).into_bytes());
                data.extend_from_slice(footer);

                Ok(data)
            }
            SshPrivateKeyFormat::Pkcs8 => {
                Ok(private_key.data.clone())
            }
        }
    }

    /// **Functor Mapping**: PublicKey → bytes
    async fn export_public_key(
        &self,
        public_key: &SshPublicKey,
        format: SshPublicKeyFormat,
    ) -> Result<Vec<u8>, SshError> {
        match format {
            SshPublicKeyFormat::OpenSsh => {
                let authorized_key = self.format_authorized_key(public_key, None).await?;
                Ok(authorized_key.into_bytes())
            }
            SshPublicKeyFormat::Pkcs8 | SshPublicKeyFormat::Rfc4253 => {
                Ok(public_key.data.clone())
            }
        }
    }

    /// **Functor Mapping**: PublicKey → Fingerprint
    async fn get_fingerprint(
        &self,
        public_key: &SshPublicKey,
        hash_type: FingerprintHashType,
    ) -> Result<String, SshError> {
        Ok(self.calculate_fingerprint(public_key, hash_type))
    }

    async fn convert_key_format(
        &self,
        private_key: &SshPrivateKey,
        format: KeyConversionFormat,
    ) -> Result<Vec<u8>, SshError> {
        // Mock conversion - just return the key data
        Ok(private_key.data.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_functor_identity_law() {
        // verify(sign(m)) = valid
        let adapter = MockSshKeyAdapter::new();

        let keypair = adapter
            .generate_keypair(SshKeyType::Ed25519, None, Some("test@example.com".to_string()))
            .await
            .unwrap();

        let data = b"Test message";
        let signature = adapter
            .sign(&keypair.private_key, data)
            .await
            .unwrap();

        let is_valid = adapter
            .verify(&keypair.public_key, data, &signature)
            .await
            .unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_functor_composition_law() {
        // format ∘ parse preserves key structure
        let adapter = MockSshKeyAdapter::new();

        let keypair = adapter
            .generate_keypair(SshKeyType::Rsa, Some(2048), None)
            .await
            .unwrap();

        // Export and parse public key
        let exported = adapter
            .export_public_key(&keypair.public_key, SshPublicKeyFormat::OpenSsh)
            .await
            .unwrap();

        let parsed = adapter
            .parse_public_key(&exported)
            .await
            .unwrap();

        assert_eq!(parsed.key_type, keypair.public_key.key_type);
    }

    #[tokio::test]
    async fn test_authorized_keys_format() {
        let adapter = MockSshKeyAdapter::new();

        let keypair = adapter
            .generate_keypair(SshKeyType::Ed25519, None, Some("user@host".to_string()))
            .await
            .unwrap();

        let authorized_key = adapter
            .format_authorized_key(&keypair.public_key, Some("user@host".to_string()))
            .await
            .unwrap();

        // Verify format: "ssh-ed25519 <base64> user@host"
        assert!(authorized_key.starts_with("ssh-ed25519 "));
        assert!(authorized_key.ends_with(" user@host"));
    }

    #[tokio::test]
    async fn test_fingerprint_generation() {
        let adapter = MockSshKeyAdapter::new();

        let keypair = adapter
            .generate_keypair(SshKeyType::Rsa, Some(2048), None)
            .await
            .unwrap();

        let md5_fingerprint = adapter
            .get_fingerprint(&keypair.public_key, FingerprintHashType::Md5)
            .await
            .unwrap();

        let sha256_fingerprint = adapter
            .get_fingerprint(&keypair.public_key, FingerprintHashType::Sha256)
            .await
            .unwrap();

        // MD5 format: xx:xx:xx:...
        assert!(md5_fingerprint.contains(':'));

        // SHA256 format: SHA256:...
        assert!(sha256_fingerprint.starts_with("SHA256:"));
    }
}
