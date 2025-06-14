//! SSH key management
//!
//! This module provides SSH key generation, management, and operations.

use crate::{KeyError, Result};
use crate::types::*;
use crate::traits::*;
use async_trait::async_trait;
use ssh_key::{
    Algorithm, EcdsaCurve as SshEcdsaCurve, HashAlg, LineEnding,
    PrivateKey, PublicKey, Signature,
};
use ssh_encoding::Encode;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::Utc;
use tracing::{debug, info};

/// SSH key manager
pub struct SshKeyManager {
    /// In-memory key storage
    keys: Arc<RwLock<HashMap<KeyId, SshKeyEntry>>>,
}

/// Internal SSH key storage entry
struct SshKeyEntry {
    private_key: PrivateKey,
    public_key: PublicKey,
    metadata: KeyMetadata,
}

impl SshKeyManager {
    /// Create a new SSH key manager
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate an SSH key pair
    pub fn generate_ssh_key(
        &self,
        algorithm: KeyAlgorithm,
        label: String,
        comment: Option<String>,
    ) -> Result<KeyId> {
        let key_id = KeyId::new();

        // Generate the key based on algorithm
        let (private_key, public_key) = match algorithm {
            KeyAlgorithm::Ed25519 => {
                let private = PrivateKey::random(
                    &mut rand::thread_rng(),
                    Algorithm::Ed25519,
                ).map_err(|e| KeyError::SshKey(e))?;
                let public = private.public_key();
                (private, public)
            }
            KeyAlgorithm::Rsa(size) => {
                let bits = match size {
                    RsaKeySize::Rsa2048 => 2048,
                    RsaKeySize::Rsa3072 => 3072,
                    RsaKeySize::Rsa4096 => 4096,
                };
                let private = PrivateKey::random(
                    &mut rand::thread_rng(),
                    Algorithm::Rsa { hash: Some(HashAlg::Sha256) },
                ).map_err(|e| KeyError::SshKey(e))?;
                let public = private.public_key();
                (private, public)
            }
            KeyAlgorithm::Ecdsa(curve) => {
                let ssh_curve = match curve {
                    EcdsaCurve::P256 => SshEcdsaCurve::NistP256,
                    EcdsaCurve::P384 => SshEcdsaCurve::NistP384,
                    EcdsaCurve::P521 => SshEcdsaCurve::NistP521,
                };
                let private = PrivateKey::random(
                    &mut rand::thread_rng(),
                    Algorithm::Ecdsa { curve: ssh_curve },
                ).map_err(|e| KeyError::SshKey(e))?;
                let public = private.public_key();
                (private, public)
            }
            _ => return Err(KeyError::UnsupportedAlgorithm(
                format!("Algorithm {:?} not supported for SSH keys", algorithm)
            )),
        };

        // Set comment if provided
        let mut public_key = public_key;
        if let Some(comment) = comment {
            public_key.set_comment(&comment);
        }

        // Create metadata
        let metadata = KeyMetadata {
            id: key_id,
            algorithm,
            usage: KeyUsage {
                sign: true,
                verify: true,
                encrypt: false,
                decrypt: false,
                derive: false,
                authenticate: true,
            },
            created_at: Utc::now(),
            expires_at: None,
            label,
            description: Some("SSH key".to_string()),
            email: None,
            fingerprint: Some(public_key.fingerprint(HashAlg::Sha256).to_string()),
            hardware_serial: None,
        };

        // Store the key
        let entry = SshKeyEntry {
            private_key,
            public_key,
            metadata: metadata.clone(),
        };

        let mut keys = self.keys.write().unwrap();
        keys.insert(key_id, entry);

        info!("Generated SSH key {} with algorithm {:?}", key_id, algorithm);
        Ok(key_id)
    }

    /// Export SSH private key
    pub fn export_private_key(
        &self,
        key_id: &KeyId,
        passphrase: Option<SecureString>,
    ) -> Result<Vec<u8>> {
        let keys = self.keys.read().unwrap();
        let entry = keys.get(key_id)
            .ok_or_else(|| KeyError::KeyNotFound(key_id.to_string()))?;

        let pem = if let Some(pass) = passphrase {
            entry.private_key.to_openssh(LineEnding::LF)
                .map_err(|e| KeyError::SshKey(e))?
                .to_bytes()
        } else {
            entry.private_key.to_openssh(LineEnding::LF)
                .map_err(|e| KeyError::SshKey(e))?
                .to_bytes()
        };

        Ok(pem)
    }

    /// Export SSH public key
    pub fn export_public_key(
        &self,
        key_id: &KeyId,
    ) -> Result<Vec<u8>> {
        let keys = self.keys.read().unwrap();
        let entry = keys.get(key_id)
            .ok_or_else(|| KeyError::KeyNotFound(key_id.to_string()))?;

        let public_str = entry.public_key.to_openssh()
            .map_err(|e| KeyError::SshKey(e))?;

        Ok(public_str.as_bytes().to_vec())
    }

    /// Import SSH private key
    pub fn import_private_key(
        &self,
        key_data: &[u8],
        passphrase: Option<SecureString>,
        label: String,
    ) -> Result<KeyId> {
        let key_str = std::str::from_utf8(key_data)
            .map_err(|_| KeyError::InvalidKeyFormat("Invalid UTF-8".to_string()))?;

        let private_key = if let Some(pass) = passphrase {
            PrivateKey::from_openssh(key_str)
                .map_err(|e| KeyError::SshKey(e))?
        } else {
            PrivateKey::from_openssh(key_str)
                .map_err(|e| KeyError::SshKey(e))?
        };

        let public_key = private_key.public_key();
        let key_id = KeyId::new();

        // Determine algorithm
        let algorithm = match private_key.algorithm() {
            Algorithm::Ed25519 => KeyAlgorithm::Ed25519,
            Algorithm::Rsa { .. } => KeyAlgorithm::Rsa(RsaKeySize::Rsa2048), // Default
            Algorithm::Ecdsa { curve } => {
                let ec_curve = match curve {
                    SshEcdsaCurve::NistP256 => EcdsaCurve::P256,
                    SshEcdsaCurve::NistP384 => EcdsaCurve::P384,
                    SshEcdsaCurve::NistP521 => EcdsaCurve::P521,
                };
                KeyAlgorithm::Ecdsa(ec_curve)
            }
            _ => return Err(KeyError::UnsupportedAlgorithm(
                "Unsupported SSH key algorithm".to_string()
            )),
        };

        // Create metadata
        let metadata = KeyMetadata {
            id: key_id,
            algorithm,
            usage: KeyUsage {
                sign: true,
                verify: true,
                encrypt: false,
                decrypt: false,
                derive: false,
                authenticate: true,
            },
            created_at: Utc::now(),
            expires_at: None,
            label,
            description: Some("Imported SSH key".to_string()),
            email: None,
            fingerprint: Some(public_key.fingerprint(HashAlg::Sha256).to_string()),
            hardware_serial: None,
        };

        // Store the key
        let entry = SshKeyEntry {
            private_key,
            public_key,
            metadata: metadata.clone(),
        };

        let mut keys = self.keys.write().unwrap();
        keys.insert(key_id, entry);

        info!("Imported SSH key {}", key_id);
        Ok(key_id)
    }

    /// Sign data with SSH key
    pub fn sign_ssh(
        &self,
        key_id: &KeyId,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        let keys = self.keys.read().unwrap();
        let entry = keys.get(key_id)
            .ok_or_else(|| KeyError::KeyNotFound(key_id.to_string()))?;

        let signature = entry.private_key
            .sign(&mut rand::thread_rng(), data)
            .map_err(|e| KeyError::SshKey(e))?;

        // Encode signature to bytes
        let mut sig_bytes = Vec::new();
        signature.encode(&mut sig_bytes)
            .map_err(|e| KeyError::Other(format!("Failed to encode signature: {}", e)))?;

        Ok(sig_bytes)
    }

    /// Verify SSH signature
    pub fn verify_ssh(
        &self,
        key_id: &KeyId,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        let keys = self.keys.read().unwrap();
        let entry = keys.get(key_id)
            .ok_or_else(|| KeyError::KeyNotFound(key_id.to_string()))?;

        // Decode signature
        let sig = Signature::try_from(signature)
            .map_err(|e| KeyError::Other(format!("Failed to decode signature: {}", e)))?;

        // Verify
        match entry.public_key.verify(data, &sig) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[async_trait]
impl KeyManager for SshKeyManager {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        label: String,
        usage: KeyUsage,
    ) -> Result<KeyId> {
        if !usage.authenticate || !usage.sign {
            return Err(KeyError::Other(
                "SSH keys must have authenticate and sign usage".to_string()
            ));
        }
        self.generate_ssh_key(algorithm, label, None)
    }

    async fn import_key(
        &self,
        key_data: &[u8],
        format: KeyExportFormat,
        label: String,
    ) -> Result<KeyId> {
        match format {
            KeyExportFormat::SshPrivate => self.import_private_key(key_data, None, label),
            _ => Err(KeyError::InvalidKeyFormat(
                format!("Format {:?} not supported for SSH key import", format)
            )),
        }
    }

    async fn export_key(
        &self,
        key_id: &KeyId,
        format: KeyExportFormat,
        include_private: bool,
    ) -> Result<Vec<u8>> {
        match format {
            KeyExportFormat::SshPrivate if include_private => {
                self.export_private_key(key_id, None)
            }
            KeyExportFormat::SshPublic => {
                self.export_public_key(key_id)
            }
            _ => Err(KeyError::InvalidKeyFormat(
                format!("Format {:?} not supported for SSH key export", format)
            )),
        }
    }

    async fn delete_key(&self, key_id: &KeyId) -> Result<()> {
        let mut keys = self.keys.write().unwrap();
        keys.remove(key_id)
            .ok_or_else(|| KeyError::KeyNotFound(key_id.to_string()))?;
        info!("Deleted SSH key {}", key_id);
        Ok(())
    }

    async fn list_keys(&self) -> Result<Vec<KeyMetadata>> {
        let keys = self.keys.read().unwrap();
        Ok(keys.values().map(|entry| entry.metadata.clone()).collect())
    }

    async fn get_key_metadata(&self, key_id: &KeyId) -> Result<KeyMetadata> {
        let keys = self.keys.read().unwrap();
        keys.get(key_id)
            .map(|entry| entry.metadata.clone())
            .ok_or_else(|| KeyError::KeyNotFound(key_id.to_string()))
    }
}

#[async_trait]
impl Signer for SshKeyManager {
    async fn sign(
        &self,
        key_id: &KeyId,
        data: &[u8],
        format: SignatureFormat,
    ) -> Result<Vec<u8>> {
        match format {
            SignatureFormat::Ssh | SignatureFormat::Raw => self.sign_ssh(key_id, data),
            _ => Err(KeyError::Other(
                format!("Signature format {:?} not supported for SSH", format)
            )),
        }
    }

    async fn verify(
        &self,
        key_id: &KeyId,
        data: &[u8],
        signature: &[u8],
        format: SignatureFormat,
    ) -> Result<bool> {
        match format {
            SignatureFormat::Ssh | SignatureFormat::Raw => self.verify_ssh(key_id, data, signature),
            _ => Err(KeyError::Other(
                format!("Signature format {:?} not supported for SSH", format)
            )),
        }
    }
}

impl Default for SshKeyManager {
    fn default() -> Self {
        Self::new()
    }
}
