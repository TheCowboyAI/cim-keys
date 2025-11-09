//! Mock GPG adapter for testing
//!
//! This adapter implements the GpgPort trait using in-memory simulation.
//! It provides a functor from the OpenPGP category to the Domain category for testing.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: OpenPGP (simulated)
//! - **Target Category**: Domain (cryptographic operations)
//! - **Functor**: MockGpgAdapter maps simulated GPG to domain operations
//! - **Morphisms Preserved**: All sign/encrypt compositions are preserved

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::ports::gpg::*;
use crate::ports::yubikey::SecureString;

/// Mock GPG adapter for testing
///
/// This is a **Functor** F: OpenPGP_Mock → Domain where:
/// - Objects: Simulated PGP keys → Domain cryptographic entities
/// - Morphisms: Simulated GPG operations → Domain operations
///
/// **Functor Laws Verified:**
/// 1. Identity: decrypt(encrypt(m)) = m
/// 2. Composition: verify(sign(m)) = valid
#[derive(Clone)]
pub struct MockGpgAdapter {
    /// Keys stored by key ID
    keys: Arc<RwLock<HashMap<GpgKeyId, GpgKeypair>>>,

    /// Key metadata
    key_info: Arc<RwLock<HashMap<GpgKeyId, GpgKeyInfo>>>,

    /// Passphrases for encrypted keys
    passphrases: Arc<RwLock<HashMap<GpgKeyId, String>>>,

    /// Key counter for generating unique IDs
    key_counter: Arc<RwLock<u64>>,
}

impl MockGpgAdapter {
    /// Create a new mock adapter
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            key_info: Arc::new(RwLock::new(HashMap::new())),
            passphrases: Arc::new(RwLock::new(HashMap::new())),
            key_counter: Arc::new(RwLock::new(1)),
        }
    }

    /// Clear all state (for test isolation)
    pub fn clear(&self) {
        self.keys.write().unwrap().clear();
        self.key_info.write().unwrap().clear();
        self.passphrases.write().unwrap().clear();
        *self.key_counter.write().unwrap() = 1;
    }

    fn next_key_id(&self) -> GpgKeyId {
        let mut counter = self.key_counter.write().unwrap();
        let id = format!("{:016X}", *counter);
        *counter += 1;
        GpgKeyId(id)
    }

    fn generate_mock_keypair(&self, user_id: &str, key_type: GpgKeyType) -> GpgKeypair {
        let key_id = self.next_key_id();

        // Mock key material
        let (public_key, private_key) = match key_type {
            GpgKeyType::Rsa => (
                vec![0x99, 0x01, 0x00], // Mock RSA public key
                vec![0x95, 0x01, 0x00], // Mock RSA private key
            ),
            GpgKeyType::Ecdsa | GpgKeyType::Eddsa => (
                vec![0x99, 0x00, 0x40], // Mock EC public key
                vec![0x95, 0x00, 0x40], // Mock EC private key
            ),
            _ => (vec![0x99], vec![0x95]),
        };

        // Mock fingerprint (40 hex chars)
        let fingerprint = format!(
            "{:040X}",
            u128::from_be_bytes([
                key_id.0.as_bytes()[0],
                key_id.0.as_bytes()[1],
                0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0
            ])
        );

        GpgKeypair {
            key_id: key_id.clone(),
            public_key,
            private_key,
            fingerprint,
            user_id: user_id.to_string(),
        }
    }

    fn generate_mock_signature(&self, key_type: GpgKeyType, data: &[u8]) -> Vec<u8> {
        // Generate deterministic mock signature
        let mut sig = match key_type {
            GpgKeyType::Rsa => vec![0x88, 0x00, 0x80], // Mock RSA signature packet
            GpgKeyType::Ecdsa | GpgKeyType::Eddsa => vec![0x88, 0x00, 0x40], // Mock ECDSA signature
            _ => vec![0x88],
        };

        // Add some deterministic data based on input
        if !data.is_empty() {
            sig.push(data[0] ^ 0xFF);
        }

        sig
    }

    fn generate_mock_encrypted(&self, data: &[u8], _recipients: &[GpgKeyId]) -> Vec<u8> {
        // Mock encrypted message packet
        let mut encrypted = vec![0x84, 0x00]; // Mock encrypted packet header

        // Simple XOR "encryption" for testing
        for byte in data {
            encrypted.push(byte ^ 0xAA);
        }

        encrypted
    }

    fn mock_decrypt(&self, encrypted_data: &[u8]) -> Vec<u8> {
        // Reverse the mock encryption
        if encrypted_data.len() < 2 {
            return Vec::new();
        }

        let mut decrypted = Vec::new();
        for byte in &encrypted_data[2..] {
            decrypted.push(byte ^ 0xAA);
        }

        decrypted
    }
}

impl Default for MockGpgAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GpgPort for MockGpgAdapter {
    /// **Functor Mapping**: (user_id, key_type) → Keypair
    async fn generate_keypair(
        &self,
        user_id: &str,
        key_type: GpgKeyType,
        key_length: u32,
        expires_in_days: Option<u32>,
    ) -> Result<GpgKeypair, GpgError> {
        let keypair = self.generate_mock_keypair(user_id, key_type);

        let now = Utc::now();
        let key_info = GpgKeyInfo {
            key_id: keypair.key_id.clone(),
            fingerprint: keypair.fingerprint.clone(),
            user_ids: vec![user_id.to_string()],
            creation_time: now.timestamp(),
            expiration_time: expires_in_days.map(|days| {
                (now + Duration::days(days as i64)).timestamp()
            }),
            is_revoked: false,
            is_expired: false,
        };

        // Store keypair and metadata
        self.keys.write().unwrap().insert(keypair.key_id.clone(), keypair.clone());
        self.key_info.write().unwrap().insert(keypair.key_id.clone(), key_info);

        Ok(keypair)
    }

    async fn import_key(&self, key_data: &[u8]) -> Result<GpgKeyId, GpgError> {
        // Mock import - just generate a new key
        if key_data.is_empty() {
            return Err(GpgError::ImportFailed("Empty key data".to_string()));
        }

        let key_id = self.next_key_id();
        let keypair = GpgKeypair {
            key_id: key_id.clone(),
            public_key: key_data.to_vec(),
            private_key: Vec::new(), // Public key only
            fingerprint: format!("{:040X}", 0),
            user_id: "Imported Key".to_string(),
        };

        let now = Utc::now();
        let key_info = GpgKeyInfo {
            key_id: key_id.clone(),
            fingerprint: keypair.fingerprint.clone(),
            user_ids: vec!["Imported Key".to_string()],
            creation_time: now.timestamp(),
            expiration_time: None,
            is_revoked: false,
            is_expired: false,
        };

        self.keys.write().unwrap().insert(key_id.clone(), keypair);
        self.key_info.write().unwrap().insert(key_id.clone(), key_info);

        Ok(key_id)
    }

    async fn export_public_key(
        &self,
        key_id: &GpgKeyId,
        armor: bool,
    ) -> Result<Vec<u8>, GpgError> {
        let keys = self.keys.read().unwrap();
        let keypair = keys
            .get(key_id)
            .ok_or_else(|| GpgError::KeyNotFound(key_id.0.clone()))?;

        if armor {
            // Return armored format
            let armored = format!(
                "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\n{}\n-----END PGP PUBLIC KEY BLOCK-----",
                base64::encode(&keypair.public_key)
            );
            Ok(armored.into_bytes())
        } else {
            Ok(keypair.public_key.clone())
        }
    }

    async fn export_private_key(
        &self,
        key_id: &GpgKeyId,
        passphrase: &SecureString,
    ) -> Result<Vec<u8>, GpgError> {
        let keys = self.keys.read().unwrap();
        let keypair = keys
            .get(key_id)
            .ok_or_else(|| GpgError::KeyNotFound(key_id.0.clone()))?;

        // Mock encryption with passphrase (just XOR for testing)
        let mut encrypted = keypair.private_key.clone();
        let pass_bytes = passphrase.as_bytes();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= pass_bytes[i % pass_bytes.len()];
        }

        // Store passphrase for later verification
        self.passphrases.write().unwrap().insert(
            key_id.clone(),
            String::from_utf8_lossy(pass_bytes).to_string(),
        );

        Ok(encrypted)
    }

    /// **Functor Mapping**: (key, data) → Signature
    async fn sign(
        &self,
        key_id: &GpgKeyId,
        data: &[u8],
        detached: bool,
    ) -> Result<Vec<u8>, GpgError> {
        let keys = self.keys.read().unwrap();
        let keypair = keys
            .get(key_id)
            .ok_or_else(|| GpgError::KeyNotFound(key_id.0.clone()))?;

        // Determine key type from key material
        let key_type = if keypair.public_key.starts_with(&[0x99, 0x01]) {
            GpgKeyType::Rsa
        } else {
            GpgKeyType::Eddsa
        };

        let signature = self.generate_mock_signature(key_type, data);

        if detached {
            Ok(signature)
        } else {
            // Combine data and signature
            let mut combined = data.to_vec();
            combined.extend_from_slice(&signature);
            Ok(combined)
        }
    }

    /// **Functor Mapping**: (data, signature) → bool
    async fn verify(
        &self,
        data: &[u8],
        signature: &[u8],
    ) -> Result<GpgVerification, GpgError> {
        // Mock verification - just check signature format
        let valid = !signature.is_empty() && signature[0] == 0x88;

        Ok(GpgVerification {
            valid,
            key_id: if valid { Some(GpgKeyId("0000000000000001".to_string())) } else { None },
            signer_user_id: if valid { Some("Mock Signer".to_string()) } else { None },
            signature_time: if valid { Some(Utc::now().timestamp()) } else { None },
        })
    }

    /// **Functor Mapping**: ([recipients], data) → CiphertextMessage
    async fn encrypt(
        &self,
        recipient_keys: &[GpgKeyId],
        data: &[u8],
    ) -> Result<Vec<u8>, GpgError> {
        if recipient_keys.is_empty() {
            return Err(GpgError::EncryptionFailed("No recipients specified".to_string()));
        }

        // Verify all recipient keys exist
        let keys = self.keys.read().unwrap();
        for key_id in recipient_keys {
            if !keys.contains_key(key_id) {
                return Err(GpgError::KeyNotFound(key_id.0.clone()));
            }
        }

        Ok(self.generate_mock_encrypted(data, recipient_keys))
    }

    /// **Functor Mapping**: (key, ciphertext) → PlaintextMessage
    /// Inverse of encrypt: decrypt(encrypt(m)) = m
    async fn decrypt(
        &self,
        key_id: &GpgKeyId,
        encrypted_data: &[u8],
    ) -> Result<Vec<u8>, GpgError> {
        let keys = self.keys.read().unwrap();
        if !keys.contains_key(key_id) {
            return Err(GpgError::KeyNotFound(key_id.0.clone()));
        }

        Ok(self.mock_decrypt(encrypted_data))
    }

    async fn list_keys(&self, secret: bool) -> Result<Vec<GpgKeyInfo>, GpgError> {
        let info_map = self.key_info.read().unwrap();
        let keys = self.keys.read().unwrap();

        let mut result = Vec::new();
        for (key_id, info) in info_map.iter() {
            if secret {
                // Only include keys with private key material
                if let Some(keypair) = keys.get(key_id) {
                    if !keypair.private_key.is_empty() {
                        result.push(info.clone());
                    }
                }
            } else {
                result.push(info.clone());
            }
        }

        Ok(result)
    }

    async fn revoke_key(
        &self,
        key_id: &GpgKeyId,
        reason: RevocationReason,
    ) -> Result<Vec<u8>, GpgError> {
        let mut info_map = self.key_info.write().unwrap();
        let info = info_map
            .get_mut(key_id)
            .ok_or_else(|| GpgError::KeyNotFound(key_id.0.clone()))?;

        info.is_revoked = true;

        // Return mock revocation certificate
        Ok(vec![0x88, 0x00, 0x20, reason as u8])
    }

    async fn add_subkey(
        &self,
        master_key_id: &GpgKeyId,
        subkey_type: GpgKeyType,
        usage: Vec<KeyUsage>,
    ) -> Result<GpgKeyId, GpgError> {
        // Verify master key exists
        let keys = self.keys.read().unwrap();
        if !keys.contains_key(master_key_id) {
            return Err(GpgError::KeyNotFound(master_key_id.0.clone()));
        }
        drop(keys);

        // Generate subkey
        let subkey_id = self.next_key_id();
        let keypair = self.generate_mock_keypair("Subkey", subkey_type);

        self.keys.write().unwrap().insert(subkey_id.clone(), keypair);

        Ok(subkey_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_functor_identity_law() {
        // decrypt(encrypt(m)) = m
        let adapter = MockGpgAdapter::new();

        let keypair = adapter
            .generate_keypair("test@example.com", GpgKeyType::Rsa, 2048, None)
            .await
            .unwrap();

        let plaintext = b"Hello, World!";
        let ciphertext = adapter
            .encrypt(&[keypair.key_id.clone()], plaintext)
            .await
            .unwrap();

        let decrypted = adapter
            .decrypt(&keypair.key_id, &ciphertext)
            .await
            .unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[tokio::test]
    async fn test_functor_composition_law() {
        // verify(sign(m)) = valid
        let adapter = MockGpgAdapter::new();

        let keypair = adapter
            .generate_keypair("signer@example.com", GpgKeyType::Eddsa, 256, None)
            .await
            .unwrap();

        let data = b"Important message";
        let signature = adapter
            .sign(&keypair.key_id, data, true)
            .await
            .unwrap();

        let verification = adapter
            .verify(data, &signature)
            .await
            .unwrap();

        assert!(verification.valid);
    }

    #[tokio::test]
    async fn test_list_keys_secret_filter() {
        let adapter = MockGpgAdapter::new();

        // Generate keypair with private key
        let full_keypair = adapter
            .generate_keypair("full@example.com", GpgKeyType::Rsa, 2048, None)
            .await
            .unwrap();

        // Import public key only
        adapter
            .import_key(b"mock public key data")
            .await
            .unwrap();

        // List all keys
        let all_keys = adapter.list_keys(false).await.unwrap();
        assert_eq!(all_keys.len(), 2);

        // List secret keys only
        let secret_keys = adapter.list_keys(true).await.unwrap();
        assert_eq!(secret_keys.len(), 1);
        assert_eq!(secret_keys[0].key_id, full_keypair.key_id);
    }
}
