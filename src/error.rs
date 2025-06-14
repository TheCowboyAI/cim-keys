//! Error types for cryptographic key operations

use thiserror::Error;

/// Result type alias for key operations
pub type Result<T> = std::result::Result<T, KeyError>;

/// Main error type for key management operations
#[derive(Error, Debug)]
pub enum KeyError {
    /// YubiKey-specific errors
    #[cfg(feature = "yubikey-support")]
    #[error("YubiKey error: {0}")]
    YubiKey(#[from] yubikey::Error),

    /// PC/SC smart card errors
    #[cfg(feature = "yubikey-support")]
    #[error("PC/SC error: {0}")]
    PcSc(#[from] pcsc::Error),

    /// GPG/OpenPGP errors
    #[cfg(feature = "gpg-support")]
    #[error("OpenPGP error: {0}")]
    OpenPgp(String),

    /// SSH key errors
    #[error("SSH key error: {0}")]
    SshKey(#[from] ssh_key::Error),

    /// X.509 certificate errors
    #[error("X.509 error: {0}")]
    X509(String),

    /// TLS errors
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    /// Certificate generation errors
    #[error("Certificate generation error: {0}")]
    CertGen(#[from] rcgen::Error),

    /// RSA errors
    #[error("RSA error: {0}")]
    Rsa(#[from] rsa::Error),

    /// Ed25519 errors
    #[error("Ed25519 error: {0}")]
    Ed25519(#[from] ed25519_dalek::SignatureError),

    /// ECDSA errors
    #[error("ECDSA error: {0}")]
    Ecdsa(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Base64 decoding errors
    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// PEM parsing errors
    #[error("PEM error: {0}")]
    Pem(#[from] pem::PemError),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    /// Unsupported algorithm
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Hardware token not available
    #[error("Hardware token not available: {0}")]
    HardwareTokenNotAvailable(String),

    /// PIN required
    #[error("PIN required for operation")]
    PinRequired,

    /// Invalid PIN
    #[error("Invalid PIN")]
    InvalidPin,

    /// Certificate validation failed
    #[error("Certificate validation failed: {0}")]
    CertificateValidationFailed(String),

    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Signature verification failed
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    /// Encryption failed
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption failed
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}
