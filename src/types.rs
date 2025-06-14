//! Common types used throughout the cim-keys crate

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt;

/// Unique identifier for a key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId(pub Uuid);

impl KeyId {
    /// Create a new random key ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for KeyId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Key algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    /// RSA with specified bit size
    Rsa(RsaKeySize),
    /// Ed25519 elliptic curve
    Ed25519,
    /// ECDSA with specified curve
    Ecdsa(EcdsaCurve),
    /// AES symmetric encryption
    Aes(AesKeySize),
}

/// RSA key sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RsaKeySize {
    /// 2048 bits (minimum recommended)
    Rsa2048,
    /// 3072 bits
    Rsa3072,
    /// 4096 bits (recommended for long-term keys)
    Rsa4096,
}

/// ECDSA curves
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcdsaCurve {
    /// NIST P-256 (secp256r1)
    P256,
    /// NIST P-384 (secp384r1)
    P384,
    /// NIST P-521 (secp521r1)
    P521,
}

/// AES key sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AesKeySize {
    /// 128-bit key
    Aes128,
    /// 192-bit key
    Aes192,
    /// 256-bit key (recommended)
    Aes256,
}

/// Key usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyUsage {
    /// Can sign data
    pub sign: bool,
    /// Can verify signatures
    pub verify: bool,
    /// Can encrypt data
    pub encrypt: bool,
    /// Can decrypt data
    pub decrypt: bool,
    /// Can derive other keys
    pub derive: bool,
    /// Can authenticate
    pub authenticate: bool,
}

impl Default for KeyUsage {
    fn default() -> Self {
        Self {
            sign: true,
            verify: true,
            encrypt: true,
            decrypt: true,
            derive: false,
            authenticate: true,
        }
    }
}

/// Key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique key identifier
    pub id: KeyId,
    /// Key algorithm
    pub algorithm: KeyAlgorithm,
    /// Key usage flags
    pub usage: KeyUsage,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp (if any)
    pub expires_at: Option<DateTime<Utc>>,
    /// Key label/name
    pub label: String,
    /// Key description
    pub description: Option<String>,
    /// Associated email
    pub email: Option<String>,
    /// Key fingerprint
    pub fingerprint: Option<String>,
    /// Hardware token serial (if stored on hardware)
    pub hardware_serial: Option<String>,
}

/// Certificate metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateMetadata {
    /// Certificate subject
    pub subject: String,
    /// Certificate issuer
    pub issuer: String,
    /// Serial number
    pub serial_number: String,
    /// Not valid before
    pub not_before: DateTime<Utc>,
    /// Not valid after
    pub not_after: DateTime<Utc>,
    /// Subject alternative names
    pub san: Vec<String>,
    /// Key usage extensions
    pub key_usage: Vec<String>,
    /// Extended key usage
    pub extended_key_usage: Vec<String>,
    /// Is this a CA certificate
    pub is_ca: bool,
    /// Path length constraint (for CA certs)
    pub path_len_constraint: Option<u32>,
}

/// PIN/Passphrase holder with secure memory handling
#[derive(Clone)]
pub struct SecureString(secrecy::SecretString);

impl SecureString {
    /// Create a new secure string
    pub fn new(s: String) -> Self {
        Self(secrecy::SecretString::from(s))
    }

    /// Get the inner secret
    pub fn expose_secret(&self) -> &str {
        use secrecy::ExposeSecret;
        self.0.expose_secret()
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureString(***)")
    }
}

/// Hardware token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareTokenInfo {
    /// Token type (e.g., "YubiKey 5C")
    pub token_type: String,
    /// Serial number
    pub serial_number: String,
    /// Firmware version
    pub firmware_version: String,
    /// Available slots/applications
    pub available_slots: Vec<String>,
    /// Supported algorithms
    pub supported_algorithms: Vec<KeyAlgorithm>,
    /// PIN retry count
    pub pin_retries: Option<u8>,
    /// PUK retry count
    pub puk_retries: Option<u8>,
}

/// Key storage location
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyLocation {
    /// Stored in filesystem
    File(std::path::PathBuf),
    /// Stored on hardware token
    HardwareToken {
        /// Token serial number
        serial: String,
        /// Slot/application identifier
        slot: String,
    },
    /// Stored in memory (temporary)
    Memory,
    /// Stored in system keyring
    SystemKeyring(String),
    /// Remote key server
    RemoteServer {
        /// Server URL
        url: String,
        /// Key identifier on server
        key_id: String,
    },
}

/// Signature format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureFormat {
    /// Raw binary signature
    Raw,
    /// PEM encoded
    Pem,
    /// DER encoded
    Der,
    /// OpenPGP format
    OpenPgp,
    /// SSH signature format
    Ssh,
    /// PKCS#7/CMS
    Pkcs7,
}

/// Encryption format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionFormat {
    /// Raw encrypted data
    Raw,
    /// OpenPGP encrypted message
    OpenPgp,
    /// CMS/PKCS#7 enveloped data
    Cms,
    /// Age encryption format
    Age,
}

/// Certificate format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateFormat {
    /// PEM encoded
    Pem,
    /// DER encoded
    Der,
    /// PKCS#12/PFX
    Pkcs12,
}

/// Key export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyExportFormat {
    /// PEM encoded private key
    Pem,
    /// DER encoded private key
    Der,
    /// PKCS#8 format
    Pkcs8,
    /// OpenPGP format
    OpenPgp,
    /// SSH private key format
    SshPrivate,
    /// SSH public key format
    SshPublic,
    /// JWK (JSON Web Key)
    Jwk,
}
