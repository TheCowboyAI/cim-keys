//! SSH key port
//!
//! This defines the interface for SSH key operations.
//! This is a **Functor** mapping from the SSH category to the Domain category.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: SSH (keys, signatures, authentication)
//! - **Target Category**: Domain (secure shell operations)
//! - **Functor**: SshKeyPort maps SSH operations to domain operations
//! - **Morphisms Preserved**: sign ∘ verify maintains authentication integrity

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ports::yubikey::SecureString;

/// Port for SSH key operations
///
/// This is a **Functor** F: SSH → Domain where:
/// - Objects: SSH keys and signatures → Domain authentication entities
/// - Morphisms: Sign/verify/authenticate → Domain security operations
///
/// **Functor Laws:**
/// 1. Identity: verify(sign(m)) = valid
/// 2. Composition: authenticate ∘ sign maintains identity
#[async_trait]
pub trait SshKeyPort: Send + Sync {
    /// Generate SSH keypair
    ///
    /// **Functor Mapping**: (key_type, bits) → Keypair
    async fn generate_keypair(
        &self,
        key_type: SshKeyType,
        bits: Option<u32>,
        comment: Option<String>,
    ) -> Result<SshKeypair, SshError>;

    /// Parse SSH public key
    ///
    /// **Functor Mapping**: bytes → PublicKey (object construction)
    async fn parse_public_key(&self, key_data: &[u8]) -> Result<SshPublicKey, SshError>;

    /// Parse SSH private key
    ///
    /// **Functor Mapping**: (bytes, passphrase) → PrivateKey (object construction)
    async fn parse_private_key(
        &self,
        key_data: &[u8],
        passphrase: Option<&SecureString>,
    ) -> Result<SshPrivateKey, SshError>;

    /// Sign data with private key
    ///
    /// **Functor Mapping**: (key, data) → Signature
    async fn sign(
        &self,
        private_key: &SshPrivateKey,
        data: &[u8],
    ) -> Result<SshSignature, SshError>;

    /// Verify signature with public key
    ///
    /// **Functor Mapping**: (key, data, signature) → bool
    async fn verify(
        &self,
        public_key: &SshPublicKey,
        data: &[u8],
        signature: &SshSignature,
    ) -> Result<bool, SshError>;

    /// Format public key for authorized_keys
    ///
    /// **Functor Mapping**: PublicKey → String (serialization)
    async fn format_authorized_key(
        &self,
        public_key: &SshPublicKey,
        comment: Option<String>,
    ) -> Result<String, SshError>;

    /// Export private key (optionally encrypted)
    ///
    /// **Functor Mapping**: (PrivateKey, passphrase) → bytes
    async fn export_private_key(
        &self,
        private_key: &SshPrivateKey,
        passphrase: Option<&SecureString>,
        format: SshPrivateKeyFormat,
    ) -> Result<Vec<u8>, SshError>;

    /// Export public key
    ///
    /// **Functor Mapping**: PublicKey → bytes
    async fn export_public_key(
        &self,
        public_key: &SshPublicKey,
        format: SshPublicKeyFormat,
    ) -> Result<Vec<u8>, SshError>;

    /// Get key fingerprint
    ///
    /// **Functor Mapping**: PublicKey → Fingerprint
    async fn get_fingerprint(
        &self,
        public_key: &SshPublicKey,
        hash_type: FingerprintHashType,
    ) -> Result<String, SshError>;

    /// Convert to other key formats (PKCS8, etc.)
    async fn convert_key_format(
        &self,
        private_key: &SshPrivateKey,
        format: KeyConversionFormat,
    ) -> Result<Vec<u8>, SshError>;
}

/// SSH key type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SshKeyType {
    Rsa,
    Dsa,
    Ecdsa,
    Ed25519,
    EcdsaSk, // Security key (FIDO)
    Ed25519Sk, // Security key (FIDO)
}

/// SSH keypair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeypair {
    pub public_key: SshPublicKey,
    pub private_key: SshPrivateKey,
    pub comment: Option<String>,
}

/// SSH public key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshPublicKey {
    pub key_type: SshKeyType,
    pub data: Vec<u8>,
    pub comment: Option<String>,
}

/// SSH private key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshPrivateKey {
    pub key_type: SshKeyType,
    pub data: Vec<u8>,
    pub public_key: SshPublicKey,
    pub is_encrypted: bool,
}

/// SSH signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSignature {
    pub algorithm: String,
    pub data: Vec<u8>,
}

/// SSH private key format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SshPrivateKeyFormat {
    OpenSsh, // New OpenSSH format (default)
    Pem,     // Traditional PEM format
    Pkcs8,   // PKCS#8 format
}

/// SSH public key format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SshPublicKeyFormat {
    OpenSsh,     // Standard SSH public key format
    Pkcs8,       // PKCS#8 SPKI format
    Rfc4253,     // RFC 4253 wire format
}

/// Fingerprint hash type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FingerprintHashType {
    Md5,    // Legacy MD5 fingerprint
    Sha256, // Modern SHA256 fingerprint (default)
}

/// Key conversion format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyConversionFormat {
    Pkcs1,  // RSA PKCS#1 format
    Pkcs8,  // PKCS#8 format
    Sec1,   // SEC1 EC private key format
}

/// SSH operation errors
#[derive(Debug, Error)]
pub enum SshError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Unsupported key type: {0:?}")]
    UnsupportedKeyType(SshKeyType),

    #[error("Key generation failed: {0}")]
    GenerationFailed(String),

    #[error("Parsing failed: {0}")]
    ParsingFailed(String),

    #[error("Signature failed: {0}")]
    SignatureFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid passphrase")]
    InvalidPassphrase,

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Conversion failed: {0}")]
    ConversionFailed(String),

    #[error("Invalid fingerprint format: {0}")]
    InvalidFingerprint(String),

    #[error("SSH operation error: {0}")]
    OperationError(String),
}
