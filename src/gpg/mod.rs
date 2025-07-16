//! GPG/OpenPGP support
//!
//! This module provides OpenPGP operations using Sequoia-PGP.

use crate::types::*;
use crate::traits::*;
use crate::{Result, KeyError};
use async_trait::async_trait;
use std::path::Path;
use std::io::Write;
use tracing::{debug, info, warn};
use chrono::{DateTime, Utc, Duration};
use sequoia_openpgp as openpgp;
use openpgp::cert::prelude::*;
use openpgp::crypto::{KeyPair, Password};
use openpgp::packet::prelude::*;
use openpgp::parse::{Parse, stream::*};
use openpgp::policy::StandardPolicy;
use openpgp::serialize::Serialize;
use openpgp::serialize::stream::{Encryptor, LiteralWriter, Message as StreamMessage};
use openpgp::types::{KeyFlags, HashAlgorithm as PgpHashAlgorithm, SymmetricAlgorithm};

/// GPG key manager implementation
pub struct GpgKeyManager {
    keyring_path: std::path::PathBuf,
    policy: StandardPolicy<'static>,
}

impl GpgKeyManager {
    /// Create a new GPG key manager
    pub fn new(keyring_path: impl AsRef<Path>) -> Self {
        Self {
            keyring_path: keyring_path.as_ref().to_path_buf(),
            policy: StandardPolicy::new(),
        }
    }
    
    /// Load a certificate from the keyring
    fn load_cert(&self, key_id: &KeyId) -> Result<Cert> {
        let cert_path = self.keyring_path.join(format!("{}.pgp", key_id));
        if !cert_path.exists() {
            return Err(KeyError::KeyNotFound(key_id.to_string()));
        }
        
        let cert_data = std::fs::read(&cert_path)
            .map_err(|e| KeyError::Io(e))?;
            
        Cert::from_bytes(&cert_data)
            .map_err(|e| KeyError::Other(format!("Failed to parse certificate: {}", e)))
    }
    
    /// Save a certificate to the keyring
    fn save_cert(&self, cert: &Cert, key_id: &KeyId) -> Result<()> {
        let cert_path = self.keyring_path.join(format!("{}.pgp", key_id));
        
        // Ensure keyring directory exists
        std::fs::create_dir_all(&self.keyring_path)
            .map_err(|e| KeyError::Io(e))?;
        
        let mut file = std::fs::File::create(&cert_path)
            .map_err(|e| KeyError::Io(e))?;
            
        cert.as_tsk().serialize(&mut file)
            .map_err(|e| KeyError::Other(format!("Failed to serialize certificate: {}", e)))?;
            
        Ok(())
    }
    
    /// Convert our hash algorithm to Sequoia's
    fn convert_hash_algorithm(alg: HashAlgorithm) -> PgpHashAlgorithm {
        match alg {
            HashAlgorithm::Sha256 => PgpHashAlgorithm::SHA256,
            HashAlgorithm::Sha384 => PgpHashAlgorithm::SHA384,
            HashAlgorithm::Sha512 => PgpHashAlgorithm::SHA512,
            _ => PgpHashAlgorithm::SHA256, // Default
        }
    }
}

#[async_trait]
impl KeyManager for GpgKeyManager {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        label: String,
        usage: KeyUsage,
    ) -> Result<KeyId> {
        info!("Generating GPG key with algorithm {:?}", algorithm);
        
        // Build certificate
        let mut builder = CertBuilder::new();
        
        // Set user ID
        builder = builder.add_userid(label.clone());
        
        // Set cipher suite based on algorithm
        match algorithm {
            KeyAlgorithm::Rsa(size) => {
                let bits = match size {
                    RsaKeySize::Rsa2048 => 2048,
                    RsaKeySize::Rsa3072 => 3072,
                    RsaKeySize::Rsa4096 => 4096,
                    _ => 2048,
                };
                builder = builder.set_cipher_suite(CipherSuite::RSA(bits));
            }
            KeyAlgorithm::Ed25519 => {
                builder = builder.set_cipher_suite(CipherSuite::Cv25519);
            }
            KeyAlgorithm::Ecdsa(curve) => {
                match curve {
                    EcdsaCurve::P256 => {
                        builder = builder.set_cipher_suite(CipherSuite::P256);
                    }
                    EcdsaCurve::P384 => {
                        builder = builder.set_cipher_suite(CipherSuite::P384);
                    }
                    EcdsaCurve::P521 => {
                        builder = builder.set_cipher_suite(CipherSuite::P521);
                    }
                }
            }
            _ => return Err(KeyError::UnsupportedAlgorithm(
                format!("Algorithm {:?} not supported for GPG keys", algorithm)
            )),
        }
        
        // Set key flags based on usage
        let mut flags = KeyFlags::empty();
        match usage {
            KeyUsage::Signing => {
                flags |= KeyFlags::empty().set_signing();
            }
            KeyUsage::Encryption => {
                flags |= KeyFlags::empty().set_transport_encryption();
                flags |= KeyFlags::empty().set_storage_encryption();
            }
            KeyUsage::KeyAgreement => {
                // GPG doesn't have a direct key agreement flag
                flags |= KeyFlags::empty().set_transport_encryption();
            }
            KeyUsage::Authentication => {
                flags |= KeyFlags::empty().set_authentication();
            }
        }
        
        // Add appropriate subkeys based on flags
        if flags.for_signing() {
            builder = builder.add_signing_subkey();
        }
        if flags.for_transport_encryption() || flags.for_storage_encryption() {
            builder = builder.add_transport_encryption_subkey();
            builder = builder.add_storage_encryption_subkey();
        }
        if flags.for_authentication() {
            builder = builder.add_authentication_subkey();
        }
        
        // Generate the certificate
        let (cert, _revocation) = builder.generate()
            .map_err(|e| KeyError::Other(format!("Failed to generate certificate: {}", e)))?;
        
        // Generate key ID from fingerprint
        let key_id = KeyId::from(cert.fingerprint().to_hex());
        
        // Save certificate
        self.save_cert(&cert, &key_id)?;
        
        info!("Generated GPG key {} with fingerprint {}", label, cert.fingerprint());
        Ok(key_id)
    }
    
    async fn import_key(
        &self,
        key_data: &[u8],
        format: KeyExportFormat,
        label: String,
    ) -> Result<KeyId> {
        info!("Importing GPG key");
        
        let cert = match format {
            KeyExportFormat::Pem => {
                // For PEM, assume it's ASCII armored
                Cert::from_bytes(key_data)
                    .map_err(|e| KeyError::Other(format!("Failed to parse PEM key: {}", e)))?
            }
            KeyExportFormat::Der => {
                // For DER, it should be binary OpenPGP format
                Cert::from_bytes(key_data)
                    .map_err(|e| KeyError::Other(format!("Failed to parse DER key: {}", e)))?
            }
            _ => return Err(KeyError::InvalidKeyFormat(
                format!("Format {:?} not supported for GPG import", format)
            )),
        };
        
        let key_id = KeyId::from(cert.fingerprint().to_hex());
        
        // Save the certificate
        self.save_cert(&cert, &key_id)?;
        
        info!("Imported GPG key {} with fingerprint {}", label, cert.fingerprint());
        Ok(key_id)
    }

    async fn export_key(
        &self,
        key_id: &KeyId,
        format: KeyExportFormat,
        include_private: bool,
    ) -> Result<Vec<u8>> {
        debug!("Exporting GPG key {}", key_id);
        
        let cert = self.load_cert(key_id)?;
        
        let mut buffer = Vec::new();
        
        match format {
            KeyExportFormat::Pem => {
                // ASCII armored format
                let mut writer = openpgp::armor::Writer::new(
                    &mut buffer,
                    if include_private {
                        openpgp::armor::Kind::SecretKey
                    } else {
                        openpgp::armor::Kind::PublicKey
                    }
                ).map_err(|e| KeyError::Other(format!("Failed to create armor writer: {}", e)))?;
                
                if include_private {
                    cert.as_tsk().serialize(&mut writer)
                } else {
                    cert.serialize(&mut writer)
                }.map_err(|e| KeyError::Other(format!("Failed to serialize key: {}", e)))?;
                
                writer.finalize()
                    .map_err(|e| KeyError::Other(format!("Failed to finalize armor: {}", e)))?;
            }
            KeyExportFormat::Der => {
                // Binary OpenPGP format
                if include_private {
                    cert.as_tsk().serialize(&mut buffer)
                } else {
                    cert.serialize(&mut buffer)
                }.map_err(|e| KeyError::Other(format!("Failed to serialize key: {}", e)))?;
            }
            _ => return Err(KeyError::InvalidKeyFormat(
                format!("Format {:?} not supported for GPG export", format)
            )),
        }
        
        Ok(buffer)
    }

    async fn delete_key(&self, key_id: &KeyId) -> Result<()> {
        info!("Deleting GPG key {}", key_id);
        
        let cert_path = self.keyring_path.join(format!("{}.pgp", key_id));
        if cert_path.exists() {
            std::fs::remove_file(&cert_path)
                .map_err(|e| KeyError::Io(e))?;
        }
        
        Ok(())
    }
    
    async fn list_keys(&self) -> Result<Vec<KeyMetadata>> {
        debug!("Listing GPG keys");
        
        let mut keys = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&self.keyring_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "pgp" {
                        if let Ok(cert_data) = std::fs::read(entry.path()) {
                            if let Ok(cert) = Cert::from_bytes(&cert_data) {
                                let key_id = KeyId::from(cert.fingerprint().to_hex());
                                
                                // Get primary key info
                                if let Ok(primary) = cert.primary_key().key() {
                                    let algorithm = match primary.pk_algo() {
                                        openpgp::types::PublicKeyAlgorithm::RSAEncryptSign => {
                                            KeyAlgorithm::Rsa(RsaKeySize::Rsa2048) // Default
                                        }
                                        openpgp::types::PublicKeyAlgorithm::EdDSA => {
                                            KeyAlgorithm::Ed25519
                                        }
                                        openpgp::types::PublicKeyAlgorithm::ECDSA => {
                                            KeyAlgorithm::Ecdsa(EcdsaCurve::P256) // Default
                                        }
                                        _ => KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
                                    };
                                    
                                    let label = cert.userids()
                                        .next()
                                        .and_then(|uid| uid.userid().name().ok())
                                        .flatten()
                                        .unwrap_or_else(|| "Unknown".to_string());
                                    
                                    let metadata = KeyMetadata {
                                        key_id: key_id.clone(),
                                        algorithm,
                                        label,
                                        created_at: primary.creation_time().into(),
                                        usage: KeyUsage::Signing, // Default
                                        hardware_backed: false,
                                        exportable: true,
                                    };
                                    
                                    keys.push(metadata);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(keys)
    }
    
    async fn get_key_metadata(&self, key_id: &KeyId) -> Result<KeyMetadata> {
        let cert = self.load_cert(key_id)?;
        
        let primary = cert.primary_key().key()
            .map_err(|e| KeyError::Other(format!("Failed to get primary key: {}", e)))?;
            
        let algorithm = match primary.pk_algo() {
            openpgp::types::PublicKeyAlgorithm::RSAEncryptSign => {
                KeyAlgorithm::Rsa(RsaKeySize::Rsa2048) // Default, could inspect actual size
            }
            openpgp::types::PublicKeyAlgorithm::EdDSA => {
                KeyAlgorithm::Ed25519
            }
            openpgp::types::PublicKeyAlgorithm::ECDSA => {
                KeyAlgorithm::Ecdsa(EcdsaCurve::P256) // Default
            }
            _ => KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
        };
        
        let label = cert.userids()
            .next()
            .and_then(|uid| uid.userid().name().ok())
            .flatten()
            .unwrap_or_else(|| "Unknown".to_string());
        
        Ok(KeyMetadata {
            key_id: key_id.clone(),
            algorithm,
            label,
            created_at: primary.creation_time().into(),
            usage: KeyUsage::Signing, // Default
            hardware_backed: false,
            exportable: true,
        })
    }
}

// Additional GPG-specific functionality
impl GpgKeyManager {
    /// Sign data with a GPG key
    pub async fn sign(&self, key_id: &KeyId, data: &[u8]) -> Result<Vec<u8>> {
        debug!("Signing data with GPG key {}", key_id);
        
        let cert = self.load_cert(key_id)?;
        
        // Get signing key
        let signing_key = cert.keys()
            .unencrypted_secret()
            .with_policy(&self.policy, None)
            .supported()
            .alive()
            .revoked(false)
            .for_signing()
            .next()
            .ok_or_else(|| KeyError::Other("No signing key found".to_string()))?;
        
        let keypair = signing_key.key().clone()
            .parts_into_secret()
            .map_err(|e| KeyError::Other(format!("Failed to get secret key: {}", e)))?
            .into_keypair()
            .map_err(|e| KeyError::Other(format!("Failed to create keypair: {}", e)))?;
        
        // Create signature
        let mut signature_bytes = Vec::new();
        let message = StreamMessage::new(&mut signature_bytes);
        let mut signer = openpgp::serialize::stream::Signer::new(message, keypair)
            .build()
            .map_err(|e| KeyError::Other(format!("Failed to create signer: {}", e)))?;
        
        signer.write_all(data)
            .map_err(|e| KeyError::Io(e))?;
        
        signer.finalize()
            .map_err(|e| KeyError::Other(format!("Failed to finalize signature: {}", e)))?;
        
        Ok(signature_bytes)
    }
    
    /// Verify a GPG signature
    pub async fn verify(&self, signature: &[u8], data: &[u8]) -> Result<bool> {
        debug!("Verifying GPG signature");
        
        let mut verifier = VerifierBuilder::from_bytes(signature)
            .map_err(|e| KeyError::Other(format!("Failed to create verifier: {}", e)))?;
        
        let mut verified = false;
        let helper = VerificationHelper {
            certs: vec![], // Would need to load relevant certs
            verified: &mut verified,
        };
        
        // Note: This is simplified - real implementation would need proper cert loading
        // and verification logic
        
        Ok(false) // Placeholder
    }
    
    /// Encrypt data for a recipient
    pub async fn encrypt(&self, recipient: &KeyId, data: &[u8]) -> Result<Vec<u8>> {
        debug!("Encrypting data for GPG recipient {}", recipient);
        
        let cert = self.load_cert(recipient)?;
        
        // Get encryption key
        let recipient_key = cert.keys()
            .with_policy(&self.policy, None)
            .supported()
            .alive()
            .revoked(false)
            .for_transport_encryption()
            .next()
            .ok_or_else(|| KeyError::Other("No encryption key found".to_string()))?;
        
        let mut encrypted = Vec::new();
        let message = StreamMessage::new(&mut encrypted);
        
        let encryptor = Encryptor::for_recipients(message, vec![recipient_key])
            .build()
            .map_err(|e| KeyError::Other(format!("Failed to create encryptor: {}", e)))?;
        
        let mut literal_writer = LiteralWriter::new(encryptor)
            .build()
            .map_err(|e| KeyError::Other(format!("Failed to create literal writer: {}", e)))?;
        
        literal_writer.write_all(data)
            .map_err(|e| KeyError::Io(e))?;
        
        literal_writer.finalize()
            .map_err(|e| KeyError::Other(format!("Failed to finalize encryption: {}", e)))?;
        
        Ok(encrypted)
    }
    
    /// Decrypt data
    pub async fn decrypt(&self, key_id: &KeyId, data: &[u8]) -> Result<Vec<u8>> {
        debug!("Decrypting data with GPG key {}", key_id);
        
        let cert = self.load_cert(key_id)?;
        
        // This is simplified - real implementation would need proper decryption logic
        // with DecryptorBuilder and proper key handling
        
        Err(KeyError::Other("GPG decryption not fully implemented".to_string()))
    }
}

/// Helper struct for signature verification
struct VerificationHelper<'a> {
    certs: Vec<Cert>,
    verified: &'a mut bool,
}

impl<'a> VerificationAidV1 for VerificationHelper<'a> {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> openpgp::Result<Vec<Cert>> {
        Ok(self.certs.clone())
    }
    
    fn check(&mut self, structure: MessageStructure) -> openpgp::Result<()> {
        for layer in structure {
            match layer {
                MessageLayer::SignatureGroup { results } => {
                    for result in results {
                        if result.is_ok() {
                            *self.verified = true;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_gpg_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GpgKeyManager::new(temp_dir.path());
        
        // Generate an Ed25519 key
        let key_id = manager.generate_key(
            KeyAlgorithm::Ed25519,
            "Test User <test@example.com>".to_string(),
            KeyUsage::Signing,
        ).await.unwrap();
        
        // Verify key exists
        let metadata = manager.get_key_metadata(&key_id).await.unwrap();
        assert_eq!(metadata.algorithm, KeyAlgorithm::Ed25519);
        assert_eq!(metadata.label, "Test User <test@example.com>");
    }
    
    #[tokio::test]
    async fn test_gpg_key_export_import() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GpgKeyManager::new(temp_dir.path());
        
        // Generate a key
        let key_id = manager.generate_key(
            KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
            "Export Test <export@example.com>".to_string(),
            KeyUsage::Encryption,
        ).await.unwrap();
        
        // Export public key
        let public_key = manager.export_key(
            &key_id,
            KeyExportFormat::Pem,
            false,
        ).await.unwrap();
        
        // Should be ASCII armored
        let key_str = String::from_utf8(public_key.clone()).unwrap();
        assert!(key_str.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----"));
        
        // Import the key with a new ID
        let temp_dir2 = TempDir::new().unwrap();
        let manager2 = GpgKeyManager::new(temp_dir2.path());
        
        let imported_id = manager2.import_key(
            &public_key,
            KeyExportFormat::Pem,
            "Imported Key".to_string(),
        ).await.unwrap();
        
        // Verify imported key
        let metadata = manager2.get_key_metadata(&imported_id).await.unwrap();
        assert!(matches!(metadata.algorithm, KeyAlgorithm::Rsa(_)));
    }
}