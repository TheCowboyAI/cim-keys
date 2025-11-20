// YubiKey Provisioning Projections
//
// Projects domain compositions (Person × Organization × KeyPurpose × PivSlot)
// into YubiKey PIV provisioning parameters.
//
// Each step emits events for complete audit trail:
//   SlotAllocationPlanned → SlotPlannedEvent
//   KeyGeneration → KeyGeneratedInSlotEvent
//   CertificateImport → CertificateImportedToSlotEvent

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::domain::{Person, Organization, KeyContext};
use crate::events::KeyPurpose;
use crate::state_machines::{PivSlot, PinPolicy, TouchPolicy, SlotPlan};
use crate::value_objects::{
    Certificate, PublicKey,
    YubiKeyPivConfiguration, FirmwareVersion, PinValue, PukValue,
    ManagementKeyValue, ManagementKeyAlgorithm, SlotState,
};

// ============================================================================
// PIV Slot Configuration (Intermediate Projection)
// ============================================================================

/// Complete PIV slot configuration
///
/// This is a projection of domain context into YubiKey PIV format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivSlotConfiguration {
    pub slot: PivSlot,
    pub purpose: KeyPurpose,
    pub algorithm: PivKeyAlgorithm,
    pub pin_policy: PinPolicy,
    pub touch_policy: TouchPolicy,
    pub subject: String,
    pub management_key_auth_required: bool,
}

/// PIV key algorithm (YubiKey specific)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PivKeyAlgorithm {
    /// RSA 2048-bit
    Rsa2048,
    /// RSA 3072-bit (YubiKey 4+)
    Rsa3072,
    /// RSA 4096-bit (YubiKey 4+)
    Rsa4096,
    /// ECDSA P-256
    EccP256,
    /// ECDSA P-384
    EccP384,
    /// Ed25519 (YubiKey 5.2+)
    Ed25519,
    /// X25519 (YubiKey 5.2+)
    X25519,
}

impl PivKeyAlgorithm {
    /// Get recommended algorithm for a purpose
    pub fn for_purpose(purpose: KeyPurpose) -> Self {
        match purpose {
            KeyPurpose::Signing => PivKeyAlgorithm::Ed25519,
            KeyPurpose::Encryption => PivKeyAlgorithm::X25519,
            KeyPurpose::Authentication => PivKeyAlgorithm::Ed25519,
            KeyPurpose::KeyAgreement => PivKeyAlgorithm::EccP256,
            KeyPurpose::CertificateAuthority => PivKeyAlgorithm::EccP384,
            _ => PivKeyAlgorithm::EccP256, // Default
        }
    }

    /// Check if algorithm is supported on YubiKey firmware version
    pub fn is_supported(&self, firmware_major: u8, firmware_minor: u8) -> bool {
        match self {
            PivKeyAlgorithm::Rsa2048 => true, // All versions
            PivKeyAlgorithm::Rsa3072 | PivKeyAlgorithm::Rsa4096 => firmware_major >= 4,
            PivKeyAlgorithm::EccP256 | PivKeyAlgorithm::EccP384 => firmware_major >= 4,
            PivKeyAlgorithm::Ed25519 | PivKeyAlgorithm::X25519 => {
                firmware_major > 5 || (firmware_major == 5 && firmware_minor >= 2)
            }
        }
    }
}

/// YubiKey provisioning plan
///
/// Complete plan for provisioning all slots on a YubiKey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyProvisioningPlan {
    pub yubikey_serial: String,
    pub owner_id: Uuid,
    pub owner_name: String,
    pub slot_configurations: HashMap<PivSlot, PivSlotConfiguration>,
    pub pin_policy: PinPolicyConfig,
    pub management_key_rotation: bool,
}

/// PIN policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinPolicyConfig {
    pub require_pin_change: bool,
    pub min_pin_length: u8,
    pub max_retries: u8,
}

impl Default for PinPolicyConfig {
    fn default() -> Self {
        Self {
            require_pin_change: true,
            min_pin_length: 6,
            max_retries: 3,
        }
    }
}

// ============================================================================
// Projection: Domain → YubiKey Provisioning Plan
// ============================================================================

/// Projection functor: Domain → YubiKey PIV Params
pub struct YubiKeyProvisioningProjection;

impl YubiKeyProvisioningProjection {
    /// Project person context to YubiKey provisioning plan
    ///
    /// This creates a complete provisioning plan with all slots configured
    /// based on the person's roles and responsibilities.
    ///
    /// Each step will emit events:
    /// - SlotAllocationPlannedEvent
    /// - PinPolicyConfiguredEvent
    /// - ManagementKeyRotationPlannedEvent
    pub fn project_from_person(
        person: &Person,
        organization: &Organization,
        yubikey_serial: String,
    ) -> YubiKeyProvisioningPlan {
        let mut slot_configurations = HashMap::new();

        // Standard slot allocation for a person
        // 9a: Authentication (TLS client auth, SSH)
        slot_configurations.insert(
            PivSlot::Authentication,
            PivSlotConfiguration {
                slot: PivSlot::Authentication,
                purpose: KeyPurpose::Authentication,
                algorithm: PivKeyAlgorithm::Ed25519,
                pin_policy: PinPolicy::Once,
                touch_policy: TouchPolicy::Never,
                subject: format!("{} Authentication", person.name),
                management_key_auth_required: true,
            },
        );

        // 9c: Digital Signature (Code signing, email signing)
        slot_configurations.insert(
            PivSlot::Signature,
            PivSlotConfiguration {
                slot: PivSlot::Signature,
                purpose: KeyPurpose::Signing,
                algorithm: PivKeyAlgorithm::Ed25519,
                pin_policy: PinPolicy::Always,
                touch_policy: TouchPolicy::Always, // Extra security for signatures
                subject: format!("{} Digital Signature", person.name),
                management_key_auth_required: true,
            },
        );

        // 9d: Key Management (Encryption/decryption)
        slot_configurations.insert(
            PivSlot::KeyManagement,
            PivSlotConfiguration {
                slot: PivSlot::KeyManagement,
                purpose: KeyPurpose::Encryption,
                algorithm: PivKeyAlgorithm::X25519,
                pin_policy: PinPolicy::Once,
                touch_policy: TouchPolicy::Never,
                subject: format!("{} Encryption", person.name),
                management_key_auth_required: true,
            },
        );

        // 9e: Card Authentication (Physical access, passwordless login)
        slot_configurations.insert(
            PivSlot::CardAuth,
            PivSlotConfiguration {
                slot: PivSlot::CardAuth,
                purpose: KeyPurpose::Authentication,
                algorithm: PivKeyAlgorithm::EccP256,
                pin_policy: PinPolicy::Never, // No PIN for physical access
                touch_policy: TouchPolicy::Cached,
                subject: format!("{} Card Authentication", person.name),
                management_key_auth_required: true,
            },
        );

        YubiKeyProvisioningPlan {
            yubikey_serial,
            owner_id: Uuid::now_v7(), // Would be person.id in real implementation
            owner_name: person.name.clone(),
            slot_configurations,
            pin_policy: PinPolicyConfig::default(),
            management_key_rotation: true,
        }
    }

    /// Project for administrator YubiKey
    ///
    /// Administrators get additional slots for CA operations
    pub fn project_for_administrator(
        person: &Person,
        organization: &Organization,
        yubikey_serial: String,
    ) -> YubiKeyProvisioningPlan {
        let mut plan = Self::project_from_person(person, organization, yubikey_serial);

        // Add retired slot for CA key (extra secure)
        plan.slot_configurations.insert(
            PivSlot::Retired(0),
            PivSlotConfiguration {
                slot: PivSlot::Retired(0),
                purpose: KeyPurpose::CertificateAuthority,
                algorithm: PivKeyAlgorithm::EccP384,
                pin_policy: PinPolicy::Always,
                touch_policy: TouchPolicy::Always,
                subject: format!("{} CA Key", organization.name),
                management_key_auth_required: true,
            },
        );

        // Require touch for all operations (higher security)
        for config in plan.slot_configurations.values_mut() {
            if config.touch_policy == TouchPolicy::Never {
                config.touch_policy = TouchPolicy::Cached;
            }
        }

        plan
    }

    /// Project for root CA YubiKey (offline, maximum security)
    pub fn project_for_root_ca(
        organization: &Organization,
        yubikey_serial: String,
    ) -> YubiKeyProvisioningPlan {
        let mut slot_configurations = HashMap::new();

        // Only provision signature slot for root CA
        slot_configurations.insert(
            PivSlot::Signature,
            PivSlotConfiguration {
                slot: PivSlot::Signature,
                purpose: KeyPurpose::CertificateAuthority,
                algorithm: PivKeyAlgorithm::EccP384,
                pin_policy: PinPolicy::Always,
                touch_policy: TouchPolicy::Always,
                subject: format!("{} Root CA", organization.name),
                management_key_auth_required: true,
            },
        );

        YubiKeyProvisioningPlan {
            yubikey_serial,
            owner_id: Uuid::now_v7(), // Organization ID
            owner_name: format!("{} Root CA", organization.name),
            slot_configurations,
            pin_policy: PinPolicyConfig {
                require_pin_change: true,
                min_pin_length: 8, // Higher security for root CA
                max_retries: 3,
            },
            management_key_rotation: true,
        }
    }

    /// Convert SlotPlan (from state machine) to PivSlotConfiguration
    pub fn from_slot_plan(
        slot: PivSlot,
        plan: &SlotPlan,
        subject: String,
    ) -> PivSlotConfiguration {
        PivSlotConfiguration {
            slot,
            purpose: plan.purpose,
            algorithm: PivKeyAlgorithm::for_purpose(plan.purpose),
            pin_policy: plan.pin_policy,
            touch_policy: plan.touch_policy,
            subject,
            management_key_auth_required: true,
        }
    }

    /// Project complete YubiKey security configuration
    ///
    /// This creates the complete PIV configuration including:
    /// - PIN/PUK values (with custom values, not factory defaults!)
    /// - Management key (rotated from default)
    /// - All slot states
    ///
    /// Emits events:
    /// - YubiKeyDetectedEvent
    /// - PinConfiguredEvent
    /// - ManagementKeyRotatedEvent
    /// - SlotAllocationPlannedEvent (for each slot)
    pub fn project_complete_configuration(
        person: &Person,
        organization: &Organization,
        yubikey_serial: String,
        firmware_version: FirmwareVersion,
        custom_pin: Option<String>,
        custom_puk: Option<String>,
    ) -> YubiKeyPivConfiguration {
        // Start with factory-fresh config
        let mut config = YubiKeyPivConfiguration::factory_fresh(
            yubikey_serial.clone(),
            firmware_version,
        );

        // Set custom PIN (or keep factory default for now - will be changed during provisioning)
        if let Some(pin) = custom_pin {
            config.pin = PinValue::new(Self::hash_value(&pin), 3);
        }

        // Set custom PUK
        if let Some(puk) = custom_puk {
            config.puk = PukValue::new(Self::hash_value(&puk), 3);
        }

        // Generate new management key (AES-256 if firmware supports it)
        let mgmt_key_algo = if firmware_version.supports(5, 4) {
            ManagementKeyAlgorithm::Aes256
        } else {
            ManagementKeyAlgorithm::TripleDes
        };
        config.management_key = ManagementKeyValue::generate_random(mgmt_key_algo);

        // Initialize standard slots (will be provisioned later)
        config.slots = vec![
            SlotState::empty(PivSlot::Authentication),
            SlotState::empty(PivSlot::Signature),
            SlotState::empty(PivSlot::KeyManagement),
            SlotState::empty(PivSlot::CardAuth),
        ];

        config
    }

    /// Project security initialization parameters
    ///
    /// Returns the security parameters that must be set during initialization:
    /// - New PIN
    /// - New PUK
    /// - New management key
    ///
    /// These should be securely generated and stored!
    pub fn project_security_initialization(
        firmware_version: FirmwareVersion,
    ) -> SecurityInitialization {
        // Determine best management key algorithm for firmware
        let mgmt_key_algo = if firmware_version.supports(5, 4) {
            ManagementKeyAlgorithm::Aes256 // Modern, secure
        } else {
            ManagementKeyAlgorithm::TripleDes // Legacy
        };

        SecurityInitialization {
            require_pin_change: true,
            require_puk_change: true,
            require_mgmt_key_rotation: true,
            recommended_mgmt_key_algorithm: mgmt_key_algo,
            min_pin_length: 6,
            max_pin_retries: 3,
        }
    }

    // Helper: Hash a value for storage
    fn hash_value(value: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Security initialization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInitialization {
    pub require_pin_change: bool,
    pub require_puk_change: bool,
    pub require_mgmt_key_rotation: bool,
    pub recommended_mgmt_key_algorithm: ManagementKeyAlgorithm,
    pub min_pin_length: u8,
    pub max_pin_retries: u8,
}

// ============================================================================
// Adapter: PIV Config → Library Params
// ============================================================================

/// Adapter for different YubiKey libraries
///
/// This would convert PivSlotConfiguration to library-specific formats
/// (e.g., yubikey-piv, ykman, etc.)
pub struct PivLibraryAdapter;

impl PivLibraryAdapter {
    /// Convert to library-agnostic command parameters
    pub fn to_command_params(config: &PivSlotConfiguration) -> Vec<String> {
        vec![
            format!("--slot={}", config.slot.hex()),
            format!("--algorithm={:?}", config.algorithm),
            format!("--pin-policy={:?}", config.pin_policy),
            format!("--touch-policy={:?}", config.touch_policy),
            format!("--subject={}", config.subject),
        ]
    }

    /// Get yubikey-piv crate algorithm identifier
    pub fn to_yubikey_algorithm(algo: PivKeyAlgorithm) -> u8 {
        match algo {
            PivKeyAlgorithm::Rsa2048 => 0x07,
            PivKeyAlgorithm::Rsa3072 => 0x05,
            PivKeyAlgorithm::Rsa4096 => 0x06,
            PivKeyAlgorithm::EccP256 => 0x11,
            PivKeyAlgorithm::EccP384 => 0x14,
            PivKeyAlgorithm::Ed25519 => 0xE0,
            PivKeyAlgorithm::X25519 => 0xE1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_support() {
        assert!(PivKeyAlgorithm::Rsa2048.is_supported(3, 0));
        assert!(PivKeyAlgorithm::Ed25519.is_supported(5, 2));
        assert!(!PivKeyAlgorithm::Ed25519.is_supported(5, 1));
    }

    #[test]
    fn test_slot_hex() {
        assert_eq!(PivSlot::Authentication.hex(), "9a");
        assert_eq!(PivSlot::Signature.hex(), "9c");
        assert_eq!(PivSlot::Retired(0).hex(), "82");
    }
}
