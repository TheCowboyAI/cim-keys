// YubiKey Value Objects
//
// Immutable value objects representing YubiKey PIV security parameters.
// These are NOT entities - they are values that define the security posture.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::state_machines::PivSlot;

// ============================================================================
// YubiKey Security Parameters (Value Objects)
// ============================================================================

/// PIN (Personal Identification Number) value object
///
/// CRITICAL SECURITY: Never store plaintext PINs in production!
/// This structure is for projection/configuration only.
#[derive(Clone, Serialize, Deserialize)]
pub struct PinValue {
    /// PIN hash (for verification, never plaintext in production)
    pin_hash: String,
    /// Retry counter (decrements on failed attempts)
    pub retries_remaining: u8,
    /// Maximum retries before lockout
    pub max_retries: u8,
}

impl PinValue {
    /// Create a new PIN with hash
    pub fn new(pin_hash: String, max_retries: u8) -> Self {
        Self {
            pin_hash,
            retries_remaining: max_retries,
            max_retries,
        }
    }

    /// Default factory PIN (123456)
    pub fn factory_default() -> Self {
        Self {
            pin_hash: Self::hash_pin("123456"),
            retries_remaining: 3,
            max_retries: 3,
        }
    }

    /// Check if PIN is locked (no retries remaining)
    pub fn is_locked(&self) -> bool {
        self.retries_remaining == 0
    }

    /// Hash a PIN for storage (simplified - use proper KDF in production)
    fn hash_pin(pin: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(pin.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify PIN (for projection purposes)
    pub fn verify(&self, pin: &str) -> bool {
        self.pin_hash == Self::hash_pin(pin)
    }
}

impl fmt::Debug for PinValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PinValue")
            .field("pin_hash", &"[REDACTED]")
            .field("retries_remaining", &self.retries_remaining)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

impl fmt::Display for PinValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PIN({}/{} retries, {})",
            self.retries_remaining,
            self.max_retries,
            if self.is_locked() { "LOCKED" } else { "active" }
        )
    }
}

/// PUK (PIN Unblock Key) value object
///
/// Used to reset a locked PIN
#[derive(Clone, Serialize, Deserialize)]
pub struct PukValue {
    /// PUK hash (for verification)
    puk_hash: String,
    /// Retry counter
    pub retries_remaining: u8,
    /// Maximum retries
    pub max_retries: u8,
}

impl PukValue {
    /// Create a new PUK with hash
    pub fn new(puk_hash: String, max_retries: u8) -> Self {
        Self {
            puk_hash,
            retries_remaining: max_retries,
            max_retries,
        }
    }

    /// Default factory PUK (12345678)
    pub fn factory_default() -> Self {
        Self {
            puk_hash: Self::hash_puk("12345678"),
            retries_remaining: 3,
            max_retries: 3,
        }
    }

    /// Check if PUK is locked
    pub fn is_locked(&self) -> bool {
        self.retries_remaining == 0
    }

    fn hash_puk(puk: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(puk.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl fmt::Debug for PukValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PukValue")
            .field("puk_hash", &"[REDACTED]")
            .field("retries_remaining", &self.retries_remaining)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

/// Management Key value object
///
/// Used to authenticate administrative operations on the YubiKey
#[derive(Clone, Serialize, Deserialize)]
pub struct ManagementKeyValue {
    /// Management key bytes (encrypted in production!)
    key_bytes: Vec<u8>,
    /// Key algorithm
    pub algorithm: ManagementKeyAlgorithm,
    /// Whether this is the default factory key
    pub is_default: bool,
}

impl ManagementKeyValue {
    /// Create a new management key
    pub fn new(key_bytes: Vec<u8>, algorithm: ManagementKeyAlgorithm) -> Self {
        Self {
            key_bytes,
            algorithm,
            is_default: false,
        }
    }

    /// Default factory management key (3DES)
    pub fn factory_default() -> Self {
        Self {
            key_bytes: vec![
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            ],
            algorithm: ManagementKeyAlgorithm::TripleDes,
            is_default: true,
        }
    }

    /// Generate a random management key
    pub fn generate_random(algorithm: ManagementKeyAlgorithm) -> Self {
        let key_bytes = match algorithm {
            ManagementKeyAlgorithm::TripleDes => {
                // 24 bytes for 3DES
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..24).map(|_| rng.gen()).collect()
            }
            ManagementKeyAlgorithm::Aes128 => {
                // 16 bytes for AES-128
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..16).map(|_| rng.gen()).collect()
            }
            ManagementKeyAlgorithm::Aes192 => {
                // 24 bytes for AES-192
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..24).map(|_| rng.gen()).collect()
            }
            ManagementKeyAlgorithm::Aes256 => {
                // 32 bytes for AES-256
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..32).map(|_| rng.gen()).collect()
            }
        };

        Self {
            key_bytes,
            algorithm,
            is_default: false,
        }
    }

    /// Get key bytes (should be encrypted in transit!)
    pub fn key_bytes(&self) -> &[u8] {
        &self.key_bytes
    }

    /// Get key length
    pub fn key_length(&self) -> usize {
        self.key_bytes.len()
    }
}

impl fmt::Debug for ManagementKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManagementKeyValue")
            .field("key_bytes", &format!("[{} bytes REDACTED]", self.key_bytes.len()))
            .field("algorithm", &self.algorithm)
            .field("is_default", &self.is_default)
            .finish()
    }
}

/// Management key algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManagementKeyAlgorithm {
    /// Triple DES (legacy, 24 bytes)
    TripleDes,
    /// AES-128 (YubiKey 5.4+, 16 bytes)
    Aes128,
    /// AES-192 (YubiKey 5.4+, 24 bytes)
    Aes192,
    /// AES-256 (YubiKey 5.4+, 32 bytes)
    Aes256,
}

impl ManagementKeyAlgorithm {
    /// Check if algorithm is supported on firmware version
    pub fn is_supported(&self, firmware_major: u8, firmware_minor: u8) -> bool {
        match self {
            ManagementKeyAlgorithm::TripleDes => true, // All versions
            ManagementKeyAlgorithm::Aes128
            | ManagementKeyAlgorithm::Aes192
            | ManagementKeyAlgorithm::Aes256 => {
                firmware_major > 5 || (firmware_major == 5 && firmware_minor >= 4)
            }
        }
    }

    /// Get key size in bytes
    pub fn key_size(&self) -> usize {
        match self {
            ManagementKeyAlgorithm::TripleDes => 24,
            ManagementKeyAlgorithm::Aes128 => 16,
            ManagementKeyAlgorithm::Aes192 => 24,
            ManagementKeyAlgorithm::Aes256 => 32,
        }
    }
}

/// PIV Slot state value object
///
/// Represents the complete state of a single PIV slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotState {
    pub slot: PivSlot,
    pub status: SlotStatus,
    pub key_algorithm: Option<SlotKeyAlgorithm>,
    pub certificate_present: bool,
    pub generated_on_device: bool,
    pub pin_policy: SlotPinPolicy,
    pub touch_policy: SlotTouchPolicy,
}

impl SlotState {
    /// Create an empty slot
    pub fn empty(slot: PivSlot) -> Self {
        Self {
            slot,
            status: SlotStatus::Empty,
            key_algorithm: None,
            certificate_present: false,
            generated_on_device: false,
            pin_policy: SlotPinPolicy::Default,
            touch_policy: SlotTouchPolicy::Default,
        }
    }

    /// Check if slot is provisioned
    pub fn is_provisioned(&self) -> bool {
        matches!(self.status, SlotStatus::Provisioned)
    }
}

/// Slot status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotStatus {
    Empty,
    KeyGenerated,
    CertificateImported,
    Provisioned,
}

/// Slot key algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotKeyAlgorithm {
    Rsa1024,
    Rsa2048,
    Rsa3072,
    Rsa4096,
    EccP256,
    EccP384,
    Ed25519,
    X25519,
}

/// Slot PIN policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotPinPolicy {
    Default,  // Follows YubiKey default (Once for most operations)
    Never,    // No PIN required
    Once,     // PIN required once per session
    Always,   // PIN required for every operation
}

/// Slot touch policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotTouchPolicy {
    Default,  // No touch required
    Never,    // No touch required
    Always,   // Touch required for every operation
    Cached,   // Touch required but cached for 15 seconds
}

/// Complete YubiKey PIV configuration value object
///
/// This is the complete security posture of a YubiKey
#[derive(Clone, Serialize, Deserialize)]
pub struct YubiKeyPivConfiguration {
    /// YubiKey serial number
    pub serial: String,
    /// Firmware version
    pub firmware_version: FirmwareVersion,
    /// PIN configuration
    pub pin: PinValue,
    /// PUK configuration
    pub puk: PukValue,
    /// Management key configuration
    pub management_key: ManagementKeyValue,
    /// State of all PIV slots
    pub slots: Vec<SlotState>,
    /// Whether YubiKey has been provisioned
    pub provisioned: bool,
    /// Whether configuration is sealed (immutable)
    pub sealed: bool,
}

impl YubiKeyPivConfiguration {
    /// Create a factory-fresh YubiKey configuration
    pub fn factory_fresh(serial: String, firmware_version: FirmwareVersion) -> Self {
        // Initialize all standard slots as empty
        let slots = vec![
            SlotState::empty(PivSlot::Authentication),
            SlotState::empty(PivSlot::Signature),
            SlotState::empty(PivSlot::KeyManagement),
            SlotState::empty(PivSlot::CardAuth),
        ];

        Self {
            serial,
            firmware_version,
            pin: PinValue::factory_default(),
            puk: PukValue::factory_default(),
            management_key: ManagementKeyValue::factory_default(),
            slots,
            provisioned: false,
            sealed: false,
        }
    }

    /// Check if using factory defaults (INSECURE!)
    pub fn has_factory_defaults(&self) -> bool {
        self.management_key.is_default
    }

    /// Get slot state by slot ID
    pub fn get_slot(&self, slot: PivSlot) -> Option<&SlotState> {
        self.slots.iter().find(|s| s.slot == slot)
    }

    /// Check if all security parameters have been changed from defaults
    pub fn is_secure(&self) -> bool {
        !self.has_factory_defaults() && self.pin.max_retries <= 5
    }
}

impl fmt::Debug for YubiKeyPivConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YubiKeyPivConfiguration")
            .field("serial", &self.serial)
            .field("firmware_version", &self.firmware_version)
            .field("pin", &"[REDACTED]")
            .field("puk", &"[REDACTED]")
            .field("management_key", &"[REDACTED]")
            .field("slots", &self.slots)
            .field("provisioned", &self.provisioned)
            .field("sealed", &self.sealed)
            .finish()
    }
}

/// Firmware version value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl FirmwareVersion {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse from version string (e.g., "5.7.2")
    pub fn parse(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version));
        }

        Ok(Self {
            major: parts[0].parse().map_err(|e| format!("Invalid major: {}", e))?,
            minor: parts[1].parse().map_err(|e| format!("Invalid minor: {}", e))?,
            patch: parts[2].parse().map_err(|e| format!("Invalid patch: {}", e))?,
        })
    }

    /// Check if this version supports a feature
    pub fn supports(&self, min_major: u8, min_minor: u8) -> bool {
        self.major > min_major || (self.major == min_major && self.minor >= min_minor)
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firmware_version_parse() {
        let version = FirmwareVersion::parse("5.7.2").unwrap();
        assert_eq!(version.major, 5);
        assert_eq!(version.minor, 7);
        assert_eq!(version.patch, 2);
    }

    #[test]
    fn test_firmware_version_supports() {
        let version = FirmwareVersion::new(5, 7, 2);
        assert!(version.supports(5, 2));
        assert!(version.supports(4, 0));
        assert!(!version.supports(6, 0));
    }

    #[test]
    fn test_pin_locked() {
        let mut pin = PinValue::factory_default();
        pin.retries_remaining = 0;
        assert!(pin.is_locked());
    }

    #[test]
    fn test_management_key_algorithm_size() {
        assert_eq!(ManagementKeyAlgorithm::TripleDes.key_size(), 24);
        assert_eq!(ManagementKeyAlgorithm::Aes128.key_size(), 16);
        assert_eq!(ManagementKeyAlgorithm::Aes256.key_size(), 32);
    }

    #[test]
    fn test_factory_fresh_config() {
        let config = YubiKeyPivConfiguration::factory_fresh(
            "12345678".to_string(),
            FirmwareVersion::new(5, 7, 2),
        );
        assert!(config.has_factory_defaults());
        assert!(!config.is_secure());
        assert_eq!(config.slots.len(), 4);
    }
}
