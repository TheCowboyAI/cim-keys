//! YubiKey hardware token port
//!
//! This defines the interface for YubiKey PIV operations.
//! This is a **Functor** mapping from the YubiKey Hardware category to the Domain category.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: YubiKey Hardware (PCSC, USB devices)
//! - **Target Category**: Domain (key management operations)
//! - **Functor**: YubiKeyPort maps hardware operations to domain operations
//! - **Morphisms Preserved**: generate_key ∘ import_certificate maintains PIV state

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Port for YubiKey hardware token operations
///
/// This is a **Functor** F: YubiKey → Domain where:
/// - Objects: Hardware devices → Domain key entities
/// - Morphisms: PIV operations (generate/sign/verify) → Domain cryptographic operations
///
/// **Functor Laws:**
/// 1. Identity: F(id) = id - No-op on device maps to no domain change
/// 2. Composition: F(generate ∘ import_cert) = F(generate) ∘ F(import_cert)
#[async_trait]
pub trait YubiKeyPort: Send + Sync {
    /// List available YubiKey devices
    ///
    /// **Functor Mapping**: () → [YubiKeyDevice]
    async fn list_devices(&self) -> Result<Vec<YubiKeyDevice>, YubiKeyError>;

    /// Generate key in PIV slot
    ///
    /// **Functor Mapping**: (device, slot, algorithm) → PublicKey
    /// Preserves composition with import_certificate
    async fn generate_key_in_slot(
        &self,
        serial: &str,
        slot: PivSlot,
        algorithm: KeyAlgorithm,
        pin: &SecureString,
    ) -> Result<PublicKey, YubiKeyError>;

    /// Import certificate to PIV slot
    ///
    /// **Functor Mapping**: (device, slot, certificate) → ()
    /// Must compose with generate_key_in_slot
    async fn import_certificate(
        &self,
        serial: &str,
        slot: PivSlot,
        certificate: &[u8],
        pin: &SecureString,
    ) -> Result<(), YubiKeyError>;

    /// Sign data using PIV key
    ///
    /// **Functor Mapping**: (device, slot, data) → Signature
    async fn sign_with_slot(
        &self,
        serial: &str,
        slot: PivSlot,
        data: &[u8],
        pin: &SecureString,
    ) -> Result<Signature, YubiKeyError>;

    /// Verify PIN
    ///
    /// **Functor Mapping**: (device, pin) → bool
    async fn verify_pin(&self, serial: &str, pin: &SecureString) -> Result<bool, YubiKeyError>;

    /// Change management key
    ///
    /// **Functor Mapping**: (device, old_key, new_key) → ()
    async fn change_management_key(
        &self,
        serial: &str,
        current_key: &[u8],
        new_key: &[u8],
    ) -> Result<(), YubiKeyError>;

    /// Reset PIV application (factory reset)
    ///
    /// **Functor Mapping**: device → ()
    /// This is a terminal morphism - resets all state
    async fn reset_piv(&self, serial: &str) -> Result<(), YubiKeyError>;

    /// Get attestation certificate for a key
    ///
    /// **Functor Mapping**: (device, slot) → Certificate
    async fn get_attestation(
        &self,
        serial: &str,
        slot: PivSlot,
    ) -> Result<Vec<u8>, YubiKeyError>;

    /// Set CHUID (Cardholder Unique Identifier)
    async fn set_chuid(
        &self,
        serial: &str,
        chuid: &[u8],
        pin: &SecureString,
    ) -> Result<(), YubiKeyError>;

    /// Set CCC (Card Capability Container)
    async fn set_ccc(
        &self,
        serial: &str,
        ccc: &[u8],
        pin: &SecureString,
    ) -> Result<(), YubiKeyError>;
}

/// YubiKey device information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct YubiKeyDevice {
    /// Device serial number
    pub serial: String,

    /// Firmware version
    pub version: String,

    /// Device model (e.g., "YubiKey 5 NFC")
    pub model: String,

    /// Is PIV application available?
    pub piv_enabled: bool,
}

/// PIV slot identifiers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PivSlot {
    /// Authentication slot (9a)
    Authentication,

    /// Digital Signature slot (9c)
    Signature,

    /// Key Management slot (9d)
    KeyManagement,

    /// Card Authentication slot (9e)
    CardAuth,

    /// Retired key slots (82-95)
    Retired(u8),
}

impl PivSlot {
    pub fn to_slot_id(&self) -> u8 {
        match self {
            PivSlot::Authentication => 0x9a,
            PivSlot::Signature => 0x9c,
            PivSlot::KeyManagement => 0x9d,
            PivSlot::CardAuth => 0x9e,
            PivSlot::Retired(n) => {
                assert!(*n >= 1 && *n <= 20, "Retired slot must be 1-20");
                0x82 + (*n - 1)
            }
        }
    }
}

/// Key algorithms supported by YubiKey PIV
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyAlgorithm {
    /// RSA 1024-bit
    Rsa1024,

    /// RSA 2048-bit
    Rsa2048,

    /// RSA 3072-bit (YubiKey 5.7+)
    Rsa3072,

    /// RSA 4096-bit (YubiKey 5.7+)
    Rsa4096,

    /// ECDSA P-256
    EccP256,

    /// ECDSA P-384
    EccP384,

    /// Ed25519 (YubiKey 5.7+)
    Ed25519,
}

/// Public key returned from key generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    /// Algorithm used
    pub algorithm: KeyAlgorithm,

    /// Public key bytes (DER encoded)
    pub data: Vec<u8>,

    /// Subject Public Key Info (SPKI) format
    pub spki: Vec<u8>,
}

/// Signature result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signature algorithm
    pub algorithm: KeyAlgorithm,

    /// Signature bytes
    pub data: Vec<u8>,
}

/// Secure string wrapper (zeroized on drop)
#[derive(Clone)]
pub struct SecureString(Vec<u8>);

impl SecureString {
    pub fn new(s: impl AsRef<[u8]>) -> Self {
        Self(s.as_ref().to_vec())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Zero out memory
        self.0.iter_mut().for_each(|b| *b = 0);
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// YubiKey operation errors
#[derive(Debug, Error)]
pub enum YubiKeyError {
    #[error("No YubiKey devices found")]
    NoDeviceFound,

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Invalid PIN")]
    InvalidPin,

    #[error("PIN locked")]
    PinLocked,

    #[error("Invalid slot: {0:?}")]
    InvalidSlot(PivSlot),

    #[error("Slot already contains a key: {0:?}")]
    SlotOccupied(PivSlot),

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Hardware error: {0}")]
    HardwareError(String),

    #[error("PCSC error: {0}")]
    PcscError(String),

    #[error("Invalid key algorithm: {0}")]
    InvalidAlgorithm(String),

    #[error("Certificate import failed: {0}")]
    CertificateImportFailed(String),

    #[error("Attestation failed: {0}")]
    AttestationFailed(String),

    #[error("YubiKey operation error: {0}")]
    OperationError(String),
}
