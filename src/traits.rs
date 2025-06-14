//! Trait definitions for cryptographic operations

use async_trait::async_trait;
use crate::{KeyError, Result};
use crate::types::*;

/// Core trait for key management operations
#[async_trait]
pub trait KeyManager: Send + Sync {
    /// Generate a new key pair
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        label: String,
        usage: KeyUsage,
    ) -> Result<KeyId>;

    /// Import an existing key
    async fn import_key(
        &self,
        key_data: &[u8],
        format: KeyExportFormat,
        label: String,
    ) -> Result<KeyId>;

    /// Export a key
    async fn export_key(
        &self,
        key_id: &KeyId,
        format: KeyExportFormat,
        include_private: bool,
    ) -> Result<Vec<u8>>;

    /// Delete a key
    async fn delete_key(&self, key_id: &KeyId) -> Result<()>;

    /// List all keys
    async fn list_keys(&self) -> Result<Vec<KeyMetadata>>;

    /// Get key metadata
    async fn get_key_metadata(&self, key_id: &KeyId) -> Result<KeyMetadata>;
}

/// Trait for digital signature operations
#[async_trait]
pub trait Signer: Send + Sync {
    /// Sign data with a key
    async fn sign(
        &self,
        key_id: &KeyId,
        data: &[u8],
        format: SignatureFormat,
    ) -> Result<Vec<u8>>;

    /// Verify a signature
    async fn verify(
        &self,
        key_id: &KeyId,
        data: &[u8],
        signature: &[u8],
        format: SignatureFormat,
    ) -> Result<bool>;
}

/// Trait for encryption operations
#[async_trait]
pub trait Encryptor: Send + Sync {
    /// Encrypt data for one or more recipients
    async fn encrypt(
        &self,
        recipient_keys: &[KeyId],
        data: &[u8],
        format: EncryptionFormat,
    ) -> Result<Vec<u8>>;

    /// Decrypt data
    async fn decrypt(
        &self,
        key_id: &KeyId,
        encrypted_data: &[u8],
        format: EncryptionFormat,
    ) -> Result<Vec<u8>>;
}

/// Trait for certificate operations
#[async_trait]
pub trait CertificateManager: Send + Sync {
    /// Generate a certificate signing request (CSR)
    async fn generate_csr(
        &self,
        key_id: &KeyId,
        subject: &str,
        san: Vec<String>,
    ) -> Result<Vec<u8>>;

    /// Import a certificate
    async fn import_certificate(
        &self,
        cert_data: &[u8],
        format: CertificateFormat,
    ) -> Result<String>; // Returns certificate ID

    /// Export a certificate
    async fn export_certificate(
        &self,
        cert_id: &str,
        format: CertificateFormat,
        include_chain: bool,
    ) -> Result<Vec<u8>>;

    /// Get certificate metadata
    async fn get_certificate_metadata(
        &self,
        cert_id: &str,
    ) -> Result<CertificateMetadata>;

    /// Validate a certificate
    async fn validate_certificate(
        &self,
        cert_id: &str,
        ca_cert_id: Option<&str>,
    ) -> Result<bool>;
}

/// Trait for hardware token operations
#[async_trait]
pub trait HardwareTokenManager: Send + Sync {
    /// List available hardware tokens
    async fn list_tokens(&self) -> Result<Vec<HardwareTokenInfo>>;

    /// Connect to a hardware token
    async fn connect_token(&self, serial: &str) -> Result<()>;

    /// Disconnect from a hardware token
    async fn disconnect_token(&self, serial: &str) -> Result<()>;

    /// Authenticate to a hardware token
    async fn authenticate_token(
        &self,
        serial: &str,
        pin: SecureString,
    ) -> Result<()>;

    /// Change PIN on a hardware token
    async fn change_pin(
        &self,
        serial: &str,
        old_pin: SecureString,
        new_pin: SecureString,
    ) -> Result<()>;

    /// Reset a hardware token
    async fn reset_token(
        &self,
        serial: &str,
        puk: SecureString,
    ) -> Result<()>;
}

/// Trait for key derivation operations
#[async_trait]
pub trait KeyDerivation: Send + Sync {
    /// Derive a key from a master key
    async fn derive_key(
        &self,
        master_key_id: &KeyId,
        derivation_path: &str,
        algorithm: KeyAlgorithm,
    ) -> Result<KeyId>;

    /// Generate a key from a passphrase
    async fn key_from_passphrase(
        &self,
        passphrase: SecureString,
        salt: &[u8],
        algorithm: KeyAlgorithm,
        iterations: u32,
    ) -> Result<KeyId>;
}

/// Trait for key agreement operations
#[async_trait]
pub trait KeyAgreement: Send + Sync {
    /// Perform ECDH key agreement
    async fn ecdh(
        &self,
        private_key_id: &KeyId,
        public_key: &[u8],
    ) -> Result<Vec<u8>>;

    /// Generate ephemeral key pair for key agreement
    async fn generate_ephemeral(
        &self,
        algorithm: KeyAlgorithm,
    ) -> Result<(KeyId, Vec<u8>)>; // Returns (private_key_id, public_key)
}

/// Trait for secure key storage
#[async_trait]
pub trait KeyStorage: Send + Sync {
    /// Store a key securely
    async fn store_key(
        &self,
        key_id: &KeyId,
        key_data: &[u8],
        metadata: KeyMetadata,
        location: KeyLocation,
    ) -> Result<()>;

    /// Retrieve a key
    async fn retrieve_key(
        &self,
        key_id: &KeyId,
    ) -> Result<(Vec<u8>, KeyMetadata)>;

    /// Update key metadata
    async fn update_metadata(
        &self,
        key_id: &KeyId,
        metadata: KeyMetadata,
    ) -> Result<()>;

    /// Check if a key exists
    async fn key_exists(&self, key_id: &KeyId) -> Result<bool>;
}

/// Combined trait for full PKI functionality
#[async_trait]
pub trait PkiOperations:
    KeyManager +
    Signer +
    Encryptor +
    CertificateManager +
    Send +
    Sync
{
    /// Issue a certificate from a CA
    async fn issue_certificate(
        &self,
        ca_key_id: &KeyId,
        csr: &[u8],
        validity_days: u32,
        is_ca: bool,
        path_len_constraint: Option<u32>,
    ) -> Result<Vec<u8>>;

    /// Create a certificate chain
    async fn create_certificate_chain(
        &self,
        cert_ids: &[String],
    ) -> Result<Vec<u8>>;

    /// Verify a certificate chain
    async fn verify_certificate_chain(
        &self,
        chain: &[u8],
        trusted_roots: &[String],
    ) -> Result<bool>;
}
