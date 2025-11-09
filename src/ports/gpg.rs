//! GPG/PGP cryptography port
//!
//! This defines the interface for OpenPGP operations.
//! This is a **Functor** mapping from the OpenPGP category to the Domain category.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: OpenPGP (keys, signatures, encryption)
//! - **Target Category**: Domain (cryptographic operations)
//! - **Functor**: GpgPort maps OpenPGP operations to domain operations
//! - **Morphisms Preserved**: encrypt ∘ sign maintains message integrity

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ports::yubikey::SecureString;

/// Port for GPG/PGP operations
///
/// This is a **Functor** F: OpenPGP → Domain where:
/// - Objects: PGP keys and messages → Domain cryptographic entities
/// - Morphisms: Sign/encrypt/verify → Domain security operations
///
/// **Functor Laws:**
/// 1. Identity: decrypt(encrypt(m)) = m
/// 2. Composition: verify(sign(m)) = valid
#[async_trait]
pub trait GpgPort: Send + Sync {
    /// Generate GPG keypair
    ///
    /// **Functor Mapping**: (user_id, key_type) → Keypair
    async fn generate_keypair(
        &self,
        user_id: &str,
        key_type: GpgKeyType,
        key_length: u32,
        expires_in_days: Option<u32>,
    ) -> Result<GpgKeypair, GpgError>;

    /// Import GPG key
    async fn import_key(&self, key_data: &[u8]) -> Result<GpgKeyId, GpgError>;

    /// Export public key
    async fn export_public_key(
        &self,
        key_id: &GpgKeyId,
        armor: bool,
    ) -> Result<Vec<u8>, GpgError>;

    /// Export private key (encrypted)
    async fn export_private_key(
        &self,
        key_id: &GpgKeyId,
        passphrase: &SecureString,
    ) -> Result<Vec<u8>, GpgError>;

    /// Sign data
    ///
    /// **Functor Mapping**: (key, data) → Signature
    async fn sign(
        &self,
        key_id: &GpgKeyId,
        data: &[u8],
        detached: bool,
    ) -> Result<Vec<u8>, GpgError>;

    /// Verify signature
    ///
    /// **Functor Mapping**: (data, signature) → bool
    async fn verify(
        &self,
        data: &[u8],
        signature: &[u8],
    ) -> Result<GpgVerification, GpgError>;

    /// Encrypt data
    ///
    /// **Functor Mapping**: ([recipients], data) → CiphertextMessage
    async fn encrypt(
        &self,
        recipient_keys: &[GpgKeyId],
        data: &[u8],
    ) -> Result<Vec<u8>, GpgError>;

    /// Decrypt data
    ///
    /// **Functor Mapping**: (key, ciphertext) → PlaintextMessage
    /// Inverse of encrypt: decrypt(encrypt(m)) = m
    async fn decrypt(
        &self,
        key_id: &GpgKeyId,
        encrypted_data: &[u8],
    ) -> Result<Vec<u8>, GpgError>;

    /// List keys in keyring
    async fn list_keys(&self, secret: bool) -> Result<Vec<GpgKeyInfo>, GpgError>;

    /// Revoke key
    async fn revoke_key(
        &self,
        key_id: &GpgKeyId,
        reason: RevocationReason,
    ) -> Result<Vec<u8>, GpgError>;

    /// Add subkey
    async fn add_subkey(
        &self,
        master_key_id: &GpgKeyId,
        subkey_type: GpgKeyType,
        usage: Vec<KeyUsage>,
    ) -> Result<GpgKeyId, GpgError>;
}

/// GPG key identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GpgKeyId(pub String);

/// GPG keypair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeypair {
    pub key_id: GpgKeyId,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub fingerprint: String,
    pub user_id: String,
}

/// GPG key type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GpgKeyType {
    Rsa,
    Dsa,
    Elgamal,
    Ecdsa,
    Eddsa,
}

/// Key usage flags
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyUsage {
    Certification,
    Signing,
    Encryption,
    Authentication,
}

/// GPG key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeyInfo {
    pub key_id: GpgKeyId,
    pub fingerprint: String,
    pub user_ids: Vec<String>,
    pub creation_time: i64,
    pub expiration_time: Option<i64>,
    pub is_revoked: bool,
    pub is_expired: bool,
}

/// Signature verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgVerification {
    pub valid: bool,
    pub key_id: Option<GpgKeyId>,
    pub signer_user_id: Option<String>,
    pub signature_time: Option<i64>,
}

/// Revocation reason
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RevocationReason {
    Unspecified,
    KeyCompromised,
    KeySuperseded,
    KeyRetired,
}

/// GPG operation errors
#[derive(Debug, Error)]
pub enum GpgError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Signature failed: {0}")]
    SignatureFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid passphrase")]
    InvalidPassphrase,

    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Import failed: {0}")]
    ImportFailed(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("GPG operation error: {0}")]
    OperationError(String),
}
