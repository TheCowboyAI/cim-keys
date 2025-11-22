// YubiKey Commands
//
// Command handlers for YubiKey provisioning and security configuration.
//
// User Stories: US-014, US-015, US-016, US-018

use uuid::Uuid;

use crate::domain::{Organization, Person};
use crate::domain_projections::YubiKeyProvisioningProjection;
use crate::events::DomainEvent;
use crate::state_machines::PivSlot;
use crate::value_objects::{
    FirmwareVersion, ManagementKeyAlgorithm, ManagementKeyValue, PinValue, PukValue,
    YubiKeyPivConfiguration,
};

// ============================================================================
// Command: Configure YubiKey Security (US-014, US-015)
// ============================================================================

/// Command to configure YubiKey security parameters
#[derive(Debug, Clone)]
pub struct ConfigureYubiKeySecurity {
    pub yubikey_serial: String,
    pub firmware_version: FirmwareVersion,
    pub new_pin: Option<String>,
    pub new_puk: Option<String>,
    pub rotate_management_key: bool,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Result of configuring YubiKey security
#[derive(Debug, Clone)]
pub struct YubiKeySecurityConfigured {
    pub configuration: YubiKeyPivConfiguration,
    pub warnings: Vec<String>,
    pub events: Vec<DomainEvent>,
}

/// Handle ConfigureYubiKeySecurity command
///
/// Validates security configuration and detects factory defaults.
///
/// Emits:
/// - YubiKeyDetectedEvent
/// - PinConfiguredEvent (if PIN changed)
/// - PukConfiguredEvent (if PUK changed)
/// - ManagementKeyRotatedEvent (if management key rotated)
/// - SecurityViolationEvent (if factory defaults detected)
///
/// User Story: US-014, US-015
pub fn handle_configure_yubikey_security(
    cmd: ConfigureYubiKeySecurity,
) -> Result<YubiKeySecurityConfigured, String> {
    use crate::events::{
        KeyEvent, ManagementKeyRotatedEvent, PinConfiguredEvent, PukConfiguredEvent,
        YubiKeyDetectedEvent,
    };
    use chrono::Utc;

    let mut events = Vec::new();
    let mut warnings = Vec::new();

    // Emit YubiKeyDetectedEvent
    events.push(DomainEvent::YubiKey(crate::events::YubiKeyEvents::YubiKeyDetected(crate::events::yubikey::YubiKeyDetectedEvent {
        event_id: Uuid::now_v7(),
        yubikey_serial: cmd.yubikey_serial.clone(),
        firmware_version: format!(
            "{}.{}.{}",
            cmd.firmware_version.major, cmd.firmware_version.minor, cmd.firmware_version.patch
        ),
        detected_at: Utc::now(),
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    })));

    // Step 1: Create factory-fresh configuration
    let mut config =
        YubiKeyPivConfiguration::factory_fresh(cmd.yubikey_serial.clone(), cmd.firmware_version);

    // Step 2: Check for factory defaults (CRITICAL SECURITY!)
    if config.has_factory_defaults() {
        warnings.push("CRITICAL: YubiKey has factory default management key! This is INSECURE!".to_string());
        warnings.push("Action required: Rotate management key immediately".to_string());
    }

    // Step 3: Configure PIN if provided
    if let Some(new_pin) = cmd.new_pin {
        if new_pin.len() < 6 {
            return Err("PIN must be at least 6 characters".to_string());
        }
        if new_pin == "123456" {
            return Err("Cannot use factory default PIN (123456)".to_string());
        }

        let pin_hash = hash_pin(&new_pin);
        config.pin = PinValue::new(pin_hash.clone(), 3);

        // Emit PinConfiguredEvent
        events.push(DomainEvent::YubiKey(crate::events::YubiKeyEvents::PinConfigured(crate::events::yubikey::PinConfiguredEvent {
            event_id: Uuid::now_v7(),
            yubikey_serial: cmd.yubikey_serial.clone(),
            pin_hash,
            retry_count: 3,
            configured_at: Utc::now(),
            correlation_id: cmd.correlation_id,
            causation_id: cmd.causation_id,
        })));
    }

    // Step 4: Configure PUK if provided
    if let Some(new_puk) = cmd.new_puk {
        if new_puk.len() < 8 {
            return Err("PUK must be at least 8 characters".to_string());
        }
        if new_puk == "12345678" {
            return Err("Cannot use factory default PUK (12345678)".to_string());
        }

        let puk_hash = hash_puk(&new_puk);
        config.puk = PukValue::new(puk_hash.clone(), 3);

        // Emit PukConfiguredEvent
        events.push(DomainEvent::YubiKey(crate::events::YubiKeyEvents::PukConfigured(crate::events::yubikey::PukConfiguredEvent {
            event_id: Uuid::now_v7(),
            yubikey_serial: cmd.yubikey_serial.clone(),
            puk_hash,
            retry_count: 3,
            configured_at: Utc::now(),
            correlation_id: cmd.correlation_id,
            causation_id: cmd.causation_id,
        })));
    }

    // Step 5: Rotate management key if requested (firmware-aware)
    if cmd.rotate_management_key {
        let mgmt_key_algo = if cmd.firmware_version.supports(5, 4) {
            ManagementKeyAlgorithm::Aes256 // Modern, secure
        } else {
            ManagementKeyAlgorithm::TripleDes // Legacy
        };

        config.management_key = ManagementKeyValue::generate_random(mgmt_key_algo);

        // Emit ManagementKeyRotatedEvent
        events.push(DomainEvent::YubiKey(crate::events::YubiKeyEvents::ManagementKeyRotated(crate::events::yubikey::ManagementKeyRotatedEvent {
            event_id: Uuid::now_v7(),
            yubikey_serial: cmd.yubikey_serial.clone(),
            algorithm: format!("{:?}", mgmt_key_algo),
            rotated_at: Utc::now(),
            correlation_id: cmd.correlation_id,
            causation_id: cmd.causation_id,
        })));
    }

    // Step 6: Validate final configuration
    if !config.is_secure() {
        warnings.push("WARNING: YubiKey configuration is not fully secure".to_string());
        if config.has_factory_defaults() {
            warnings.push("  - Still using factory default management key".to_string());
        }
    }

    Ok(YubiKeySecurityConfigured {
        configuration: config,
        warnings,
        events,
    })
}

// ============================================================================
// Command: Provision YubiKey Slot (US-016, US-018)
// ============================================================================

/// Command to provision a YubiKey PIV slot
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvisionYubiKeySlot {
    pub yubikey_serial: String,
    pub slot: PivSlot,
    pub person: Person,
    pub organization: Organization,
    pub purpose: crate::value_objects::AuthKeyPurpose,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Result of provisioning YubiKey slot
#[derive(Debug, Clone)]
pub struct YubiKeySlotProvisioned {
    pub slot: PivSlot,
    pub key_generated: bool,
    pub certificate_imported: bool,
    pub events: Vec<DomainEvent>,
}

/// Handle ProvisionYubiKeySlot command
///
/// Generates key in slot, creates certificate, and imports to YubiKey.
///
/// Emits:
/// - SlotAllocationPlannedEvent
/// - KeyGeneratedInSlotEvent
/// - CertificateGeneratedEvent
/// - CertificateImportedToSlotEvent
///
/// User Story: US-016, US-018
pub fn handle_provision_yubikey_slot(
    cmd: ProvisionYubiKeySlot,
) -> Result<YubiKeySlotProvisioned, String> {
    use crate::events::{
        CertificateGeneratedEvent, CertificateImportedToSlotEvent, KeyAlgorithm,
        KeyEvent, KeyGeneratedInSlotEvent, SlotAllocationPlannedEvent,
    };
    use chrono::Utc;

    let mut events = Vec::new();

    // Step 1: Plan slot allocation
    events.push(DomainEvent::YubiKey(crate::events::YubiKeyEvents::SlotAllocationPlanned(crate::events::yubikey::SlotAllocationPlannedEvent {
        event_id: Uuid::now_v7(),
        yubikey_serial: cmd.yubikey_serial.clone(),
        slot: cmd.slot.hex(),
        purpose: map_auth_purpose_to_key_purpose(&cmd.purpose),
        person_id: cmd.person.id,
        planned_at: Utc::now(),
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    })));

    // Step 2: Generate key in slot (on-device generation)
    // NOTE: This is a stub - actual hardware interaction would happen through YubiKeyPort
    let key_id = Uuid::now_v7();
    let algorithm = KeyAlgorithm::Ecdsa {
        curve: "P-256".to_string(),
    }; // Default to P-256 for modern security

    events.push(DomainEvent::YubiKey(crate::events::YubiKeyEvents::KeyGeneratedInSlot(crate::events::yubikey::KeyGeneratedInSlotEvent {
        event_id: Uuid::now_v7(),
        yubikey_serial: cmd.yubikey_serial.clone(),
        slot: cmd.slot.hex(),
        key_id,
        algorithm: algorithm.clone(),
        public_key: vec![], // Would be populated by actual hardware
        generated_at: Utc::now(),
        correlation_id: cmd.correlation_id,
        causation_id: Some(cmd.correlation_id),
    })));

    // Step 3: Generate certificate for key
    let cert_id = Uuid::now_v7();
    let subject = format!(
        "CN={},O={},OU={}",
        cmd.person.name,
        cmd.organization.name,
        cmd.organization.units.first().map(|u| u.name.as_str()).unwrap_or("Default")
    );

    events.push(DomainEvent::Certificate(crate::events::CertificateEvents::CertificateGenerated(crate::events::certificate::CertificateGeneratedEvent {
        cert_id,
        key_id,
        subject,
        issuer: None, // Would be set to CA cert ID in real implementation
        not_before: Utc::now(),
        not_after: Utc::now() + chrono::Duration::days(365),
        is_ca: false,
        san: vec![format!("email:{}", cmd.person.email)],
        key_usage: vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
        extended_key_usage: match cmd.purpose {
            crate::value_objects::AuthKeyPurpose::SsoAuthentication => {
                vec!["clientAuth".to_string()]
            }
            crate::value_objects::AuthKeyPurpose::X509CodeSigning => {
                vec!["codeSigning".to_string()]
            }
            _ => vec!["clientAuth".to_string()],
        },
        correlation_id: cmd.correlation_id,
        causation_id: Some(key_id), // Certificate caused by key generation
    })));

    // Step 4: Import certificate to slot
    events.push(DomainEvent::YubiKey(crate::events::YubiKeyEvents::CertificateImportedToSlot(crate::events::yubikey::CertificateImportedToSlotEvent {
            event_id: Uuid::now_v7(),
            yubikey_serial: cmd.yubikey_serial.clone(),
            slot: cmd.slot.hex(),
            cert_id,
            imported_at: Utc::now(),
            correlation_id: cmd.correlation_id,
            causation_id: Some(cmd.correlation_id),
    })));

    // Step 5: Attest slot (verify key was generated on device)
    // NOTE: Attestation would be done through YubiKeyPort in real implementation
    // For now, we mark as successful based on event emission

    Ok(YubiKeySlotProvisioned {
        slot: cmd.slot,
        key_generated: true,
        certificate_imported: true,
        events,
    })
}

/// Helper to map AuthKeyPurpose to KeyPurpose
fn map_auth_purpose_to_key_purpose(
    auth_purpose: &crate::value_objects::AuthKeyPurpose,
) -> crate::events::KeyPurpose {
    use crate::events::KeyPurpose;

    match auth_purpose {
        crate::value_objects::AuthKeyPurpose::SsoAuthentication => KeyPurpose::Authentication,
        crate::value_objects::AuthKeyPurpose::SessionTokenSigning => KeyPurpose::JwtSigning,
        crate::value_objects::AuthKeyPurpose::X509CodeSigning => KeyPurpose::Signing,
        crate::value_objects::AuthKeyPurpose::GpgEncryption => KeyPurpose::Encryption,
        crate::value_objects::AuthKeyPurpose::SshAuthentication => KeyPurpose::Authentication,
        crate::value_objects::AuthKeyPurpose::GpgSigning => KeyPurpose::Signing,
        crate::value_objects::AuthKeyPurpose::GpgAuthentication => KeyPurpose::Authentication,
        _ => KeyPurpose::Authentication, // Default fallback
    }
}

// ============================================================================
// Command: Complete YubiKey Provisioning (US-018)
// ============================================================================

/// Command to provision complete YubiKey for a person
#[derive(Debug, Clone)]
pub struct ProvisionCompleteYubiKey {
    pub yubikey_serial: String,
    pub firmware_version: FirmwareVersion,
    pub person: Person,
    pub organization: Organization,
    pub is_administrator: bool,
    pub correlation_id: Uuid,
}

/// Result of complete YubiKey provisioning
#[derive(Debug, Clone)]
pub struct YubiKeyCompletelyProvisioned {
    pub security_config: YubiKeySecurityConfigured,
    pub provisioned_slots: Vec<YubiKeySlotProvisioned>,
    pub events: Vec<DomainEvent>,
}

/// Handle ProvisionCompleteYubiKey command
///
/// Complete workflow:
/// 1. Configure security (PIN/PUK/management key)
/// 2. Provision standard slots (9a/9c/9d/9e)
/// 3. Provision administrator slots if needed (CA key)
/// 4. Generate attestation
/// 5. Seal configuration
///
/// User Story: US-018
pub fn handle_provision_complete_yubikey(
    cmd: ProvisionCompleteYubiKey,
) -> Result<YubiKeyCompletelyProvisioned, String> {
    let mut all_events = Vec::new();

    // Step 1: Configure security with generated PIN/PUK
    let secure_pin = generate_secure_pin()?;
    let secure_puk = generate_secure_puk()?;

    let security_config = handle_configure_yubikey_security(ConfigureYubiKeySecurity {
        yubikey_serial: cmd.yubikey_serial.clone(),
        firmware_version: cmd.firmware_version,
        new_pin: Some(secure_pin),
        new_puk: Some(secure_puk),
        rotate_management_key: true,
        correlation_id: cmd.correlation_id,
        causation_id: None,
    })?;
    all_events.extend(security_config.events.clone());

    // Step 2: Get provisioning plan from projection
    let plan = if cmd.is_administrator {
        YubiKeyProvisioningProjection::project_for_administrator(
            &cmd.person,
            &cmd.organization,
            cmd.yubikey_serial.clone(),
        )
    } else {
        YubiKeyProvisioningProjection::project_from_person(
            &cmd.person,
            &cmd.organization,
            cmd.yubikey_serial.clone(),
        )
    };

    // Step 3: Provision each slot in the plan
    let mut provisioned_slots = Vec::new();
    for (slot, slot_config) in &plan.slot_configurations {
        // Map KeyPurpose to AuthKeyPurpose
        let auth_purpose = match slot_config.purpose {
            crate::events::KeyPurpose::Authentication => {
                crate::value_objects::AuthKeyPurpose::SsoAuthentication
            }
            crate::events::KeyPurpose::Signing => {
                crate::value_objects::AuthKeyPurpose::X509CodeSigning
            }
            crate::events::KeyPurpose::Encryption => {
                crate::value_objects::AuthKeyPurpose::GpgEncryption
            }
            crate::events::KeyPurpose::JwtSigning => {
                crate::value_objects::AuthKeyPurpose::SessionTokenSigning
            }
            crate::events::KeyPurpose::CertificateAuthority => {
                crate::value_objects::AuthKeyPurpose::X509CodeSigning // Use code signing for CA
            }
            _ => crate::value_objects::AuthKeyPurpose::SsoAuthentication, // Default
        };

        let provision_cmd = ProvisionYubiKeySlot {
            yubikey_serial: cmd.yubikey_serial.clone(),
            slot: *slot,
            person: cmd.person.clone(),
            organization: cmd.organization.clone(),
            purpose: auth_purpose,
            correlation_id: cmd.correlation_id,
            causation_id: Some(cmd.correlation_id), // Link to parent provisioning command
        };

        let slot_result = handle_provision_yubikey_slot(provision_cmd)?;
        all_events.extend(slot_result.events.clone());
        provisioned_slots.push(slot_result);
    }

    // Step 4: Generate attestation
    // TODO: Attest that keys were generated on device
    // This requires actual YubiKey hardware interaction via yubikey-manager

    // Step 5: Seal configuration
    // TODO: Mark YubiKey as sealed/immutable
    // This would emit a YubiKeySealedEvent to indicate provisioning is complete

    Ok(YubiKeyCompletelyProvisioned {
        security_config,
        provisioned_slots,
        events: all_events,
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a cryptographically secure random PIN (6-8 digits)
///
/// Returns a PIN that:
/// - Is 8 digits long for maximum security
/// - Uses cryptographically secure random number generation
/// - Never contains repeating patterns (e.g., 111111, 123456)
/// - Never matches common weak PINs
fn generate_secure_pin() -> Result<String, String> {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    // Generate 10 candidates and pick the first valid one
    for _ in 0..10 {
        let pin: String = (0..8)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect();

        // Validate against weak patterns
        if is_weak_pin(&pin) {
            continue;
        }

        return Ok(pin);
    }

    Err("Failed to generate secure PIN after 10 attempts".to_string())
}

/// Generate a cryptographically secure random PUK (8 digits)
///
/// Returns a PUK that:
/// - Is 8 digits long
/// - Uses cryptographically secure random number generation
/// - Never contains repeating patterns
/// - Never matches common weak PUKs
fn generate_secure_puk() -> Result<String, String> {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    // Generate 10 candidates and pick the first valid one
    for _ in 0..10 {
        let puk: String = (0..8)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect();

        // Validate against weak patterns
        if is_weak_puk(&puk) {
            continue;
        }

        return Ok(puk);
    }

    Err("Failed to generate secure PUK after 10 attempts".to_string())
}

/// Check if a PIN is weak (common patterns to avoid)
fn is_weak_pin(pin: &str) -> bool {
    // Check for common weak patterns
    let weak_patterns = [
        "00000000", "11111111", "22222222", "33333333", "44444444", "55555555", "66666666",
        "77777777", "88888888", "99999999", "12345678", "87654321", "01234567", "123456",
        "000000", "111111", "222222", "333333", "444444", "555555", "666666", "777777",
        "888888", "999999",
    ];

    if weak_patterns.contains(&pin) {
        return true;
    }

    // Check for all same digits
    if pin.chars().all(|c| c == pin.chars().next().unwrap()) {
        return true;
    }

    // Check for sequential ascending
    let is_sequential_asc = pin
        .chars()
        .zip(pin.chars().skip(1))
        .all(|(a, b)| b as u8 == a as u8 + 1);
    if is_sequential_asc {
        return true;
    }

    // Check for sequential descending
    let is_sequential_desc = pin
        .chars()
        .zip(pin.chars().skip(1))
        .all(|(a, b)| a as u8 == b as u8 + 1);
    if is_sequential_desc {
        return true;
    }

    false
}

/// Check if a PUK is weak (common patterns to avoid)
fn is_weak_puk(puk: &str) -> bool {
    // Same validation as PIN
    is_weak_pin(puk) || puk == "12345678"
}

fn hash_pin(pin: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hex::encode(hasher.finalize())
}

fn hash_puk(puk: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(puk.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_security_detects_factory_defaults() {
        let cmd = ConfigureYubiKeySecurity {
            yubikey_serial: "12345678".to_string(),
            firmware_version: FirmwareVersion::new(5, 7, 2),
            new_pin: Some("654321".to_string()),
            new_puk: Some("87654321".to_string()),
            rotate_management_key: false,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_configure_yubikey_security(cmd).unwrap();

        // Should warn about factory default management key
        assert!(!result.warnings.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("factory default")));
    }

    #[test]
    fn test_configure_security_rejects_weak_pin() {
        let cmd = ConfigureYubiKeySecurity {
            yubikey_serial: "12345678".to_string(),
            firmware_version: FirmwareVersion::new(5, 7, 2),
            new_pin: Some("123".to_string()), // Too short
            new_puk: None,
            rotate_management_key: false,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_configure_yubikey_security(cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 6"));
    }

    #[test]
    fn test_configure_security_uses_firmware_aware_algorithms() {
        // Test with modern firmware (5.4+)
        let cmd_modern = ConfigureYubiKeySecurity {
            yubikey_serial: "12345678".to_string(),
            firmware_version: FirmwareVersion::new(5, 7, 2),
            new_pin: None,
            new_puk: None,
            rotate_management_key: true,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result_modern = handle_configure_yubikey_security(cmd_modern).unwrap();
        assert_eq!(
            result_modern.configuration.management_key.algorithm,
            ManagementKeyAlgorithm::Aes256
        );

        // Test with legacy firmware (<5.4)
        let cmd_legacy = ConfigureYubiKeySecurity {
            yubikey_serial: "12345678".to_string(),
            firmware_version: FirmwareVersion::new(4, 0, 0),
            new_pin: None,
            new_puk: None,
            rotate_management_key: true,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result_legacy = handle_configure_yubikey_security(cmd_legacy).unwrap();
        assert_eq!(
            result_legacy.configuration.management_key.algorithm,
            ManagementKeyAlgorithm::TripleDes
        );
    }
}
