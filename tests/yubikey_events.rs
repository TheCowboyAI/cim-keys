//! Comprehensive YubiKey Events Tests
//!
//! Target: 90%+ coverage of src/events/yubikey.rs
//!
//! Tests all 8 event types for YubiKey hardware security module lifecycle.

use chrono::Utc;
use cim_keys::events::yubikey::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use cim_keys::types::{YubiKeySlot, KeyAlgorithm, KeyPurpose};
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_event_id() -> Uuid { Uuid::now_v7() }
fn test_key_id() -> Uuid { Uuid::now_v7() }
fn test_cert_id() -> Uuid { Uuid::now_v7() }
fn test_person_id() -> Uuid { Uuid::now_v7() }

fn test_yubikey_slot() -> YubiKeySlot {
    YubiKeySlot {
        slot_id: "9a".to_string(),
        key_id: test_key_id(),
        purpose: KeyPurpose::Authentication,
    }
}

fn sample_yubikey_detected() -> YubiKeyDetectedEvent {
    YubiKeyDetectedEvent {
        event_id: test_event_id(),
        yubikey_serial: "12345678".to_string(),
        firmware_version: "5.4.3".to_string(),
        detected_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_yubikey_provisioned() -> YubiKeyProvisionedEvent {
    YubiKeyProvisionedEvent {
        event_id: test_event_id(),
        yubikey_serial: "12345678".to_string(),
        slots_configured: vec![test_yubikey_slot()],
        provisioned_at: Utc::now(),
        provisioned_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_pin_configured() -> PinConfiguredEvent {
    PinConfiguredEvent {
        event_id: test_event_id(),
        yubikey_serial: "12345678".to_string(),
        pin_hash: "sha256:abcd1234...".to_string(),
        retry_count: 3,
        configured_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_puk_configured() -> PukConfiguredEvent {
    PukConfiguredEvent {
        event_id: test_event_id(),
        yubikey_serial: "12345678".to_string(),
        puk_hash: "sha256:efgh5678...".to_string(),
        retry_count: 5,
        configured_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_management_key_rotated() -> ManagementKeyRotatedEvent {
    ManagementKeyRotatedEvent {
        event_id: test_event_id(),
        yubikey_serial: "12345678".to_string(),
        algorithm: "AES256".to_string(),
        rotated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_slot_allocation_planned() -> SlotAllocationPlannedEvent {
    SlotAllocationPlannedEvent {
        event_id: test_event_id(),
        yubikey_serial: "12345678".to_string(),
        slot: "9a".to_string(),
        purpose: KeyPurpose::Authentication,
        person_id: test_person_id(),
        planned_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_key_generated_in_slot() -> KeyGeneratedInSlotEvent {
    KeyGeneratedInSlotEvent {
        event_id: test_event_id(),
        yubikey_serial: "12345678".to_string(),
        slot: "9a".to_string(),
        key_id: test_key_id(),
        algorithm: KeyAlgorithm::Rsa { bits: 2048 },
        public_key: vec![0x30, 0x82, 0x01, 0x22], // Simplified RSA public key header
        generated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_certificate_imported_to_slot() -> CertificateImportedToSlotEvent {
    CertificateImportedToSlotEvent {
        event_id: test_event_id(),
        yubikey_serial: "12345678".to_string(),
        slot: "9a".to_string(),
        cert_id: test_cert_id(),
        imported_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (8 event types)
// =============================================================================

#[test]
fn test_yubikey_detected_serialization() {
    let event = sample_yubikey_detected();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: YubiKeyDetectedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.yubikey_serial, deserialized.yubikey_serial);
    assert_eq!(event.firmware_version, deserialized.firmware_version);
}

#[test]
fn test_yubikey_provisioned_serialization() {
    let event = sample_yubikey_provisioned();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: YubiKeyProvisionedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.slots_configured.len(), deserialized.slots_configured.len());
}

#[test]
fn test_pin_configured_serialization() {
    let event = sample_pin_configured();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PinConfiguredEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.retry_count, deserialized.retry_count);
}

#[test]
fn test_puk_configured_serialization() {
    let event = sample_puk_configured();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PukConfiguredEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.retry_count, deserialized.retry_count);
}

#[test]
fn test_management_key_rotated_serialization() {
    let event = sample_management_key_rotated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ManagementKeyRotatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.algorithm, deserialized.algorithm);
}

#[test]
fn test_slot_allocation_planned_serialization() {
    let event = sample_slot_allocation_planned();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: SlotAllocationPlannedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.slot, deserialized.slot);
    assert_eq!(event.person_id, deserialized.person_id);
}

#[test]
fn test_key_generated_in_slot_serialization() {
    let event = sample_key_generated_in_slot();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: KeyGeneratedInSlotEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.public_key, deserialized.public_key);
}

#[test]
fn test_certificate_imported_to_slot_serialization() {
    let event = sample_certificate_imported_to_slot();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateImportedToSlotEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.cert_id, deserialized.cert_id);
}

// =============================================================================
// YubiKeyEvents Enum Serialization
// =============================================================================

#[test]
fn test_yubikey_events_enum_serialization() {
    let events = vec![
        YubiKeyEvents::YubiKeyDetected(sample_yubikey_detected()),
        YubiKeyEvents::YubiKeyProvisioned(sample_yubikey_provisioned()),
        YubiKeyEvents::PinConfigured(sample_pin_configured()),
        YubiKeyEvents::PukConfigured(sample_puk_configured()),
        YubiKeyEvents::ManagementKeyRotated(sample_management_key_rotated()),
        YubiKeyEvents::SlotAllocationPlanned(sample_slot_allocation_planned()),
        YubiKeyEvents::KeyGeneratedInSlot(sample_key_generated_in_slot()),
        YubiKeyEvents::CertificateImportedToSlot(sample_certificate_imported_to_slot()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: YubiKeyEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let detected = YubiKeyDetectedEvent {
        causation_id: None,
        correlation_id,
        ..sample_yubikey_detected()
    };
    let provisioned = YubiKeyProvisionedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_yubikey_provisioned()
    };

    assert_eq!(detected.causation_id, None);
    assert_eq!(provisioned.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let detected = YubiKeyDetectedEvent { correlation_id, ..sample_yubikey_detected() };
    let provisioned = YubiKeyProvisionedEvent { correlation_id, ..sample_yubikey_provisioned() };
    let pin_configured = PinConfiguredEvent { correlation_id, ..sample_pin_configured() };

    assert_eq!(detected.correlation_id, correlation_id);
    assert_eq!(provisioned.correlation_id, correlation_id);
    assert_eq!(pin_configured.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let detected = sample_yubikey_detected();
    assert_ne!(detected.event_id, Uuid::nil());
    assert_ne!(detected.correlation_id, Uuid::nil());

    let key_gen = sample_key_generated_in_slot();
    assert_ne!(key_gen.key_id, Uuid::nil());
}

#[test]
fn test_yubikey_serial_format() {
    let detected = sample_yubikey_detected();
    assert!(!detected.yubikey_serial.is_empty());
    assert!(detected.yubikey_serial.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_firmware_version_format() {
    let detected = sample_yubikey_detected();
    assert!(!detected.firmware_version.is_empty());
    assert!(detected.firmware_version.contains('.'));
}

#[test]
fn test_retry_counts() {
    let pin = sample_pin_configured();
    assert!(pin.retry_count > 0);
    assert!(pin.retry_count <= 10);

    let puk = sample_puk_configured();
    assert!(puk.retry_count > 0);
    assert!(puk.retry_count <= 10);
}

#[test]
fn test_slot_identifiers() {
    let slot_allocation = sample_slot_allocation_planned();
    assert!(!slot_allocation.slot.is_empty());
    // YubiKey PIV slots are typically 9a, 9c, 9d, 9e
    assert!(slot_allocation.slot.len() == 2);
}

#[test]
fn test_key_algorithm_variants() {
    let key_gen = sample_key_generated_in_slot();
    match key_gen.algorithm {
        KeyAlgorithm::Rsa { bits } => assert!(bits >= 2048),
        KeyAlgorithm::Ecdsa { ref curve } => assert!(!curve.is_empty()),
        KeyAlgorithm::Ed25519 => {},
        KeyAlgorithm::Secp256k1 => {},
    }
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let event_id = test_event_id();

    let events = vec![
        (YubiKeyEvents::YubiKeyDetected(YubiKeyDetectedEvent { event_id, ..sample_yubikey_detected() }), event_id),
        (YubiKeyEvents::YubiKeyProvisioned(YubiKeyProvisionedEvent { event_id, ..sample_yubikey_provisioned() }), event_id),
        (YubiKeyEvents::PinConfigured(PinConfiguredEvent { event_id, ..sample_pin_configured() }), event_id),
        (YubiKeyEvents::PukConfigured(PukConfiguredEvent { event_id, ..sample_puk_configured() }), event_id),
        (YubiKeyEvents::ManagementKeyRotated(ManagementKeyRotatedEvent { event_id, ..sample_management_key_rotated() }), event_id),
        (YubiKeyEvents::SlotAllocationPlanned(SlotAllocationPlannedEvent { event_id, ..sample_slot_allocation_planned() }), event_id),
        (YubiKeyEvents::KeyGeneratedInSlot(KeyGeneratedInSlotEvent { event_id, ..sample_key_generated_in_slot() }), event_id),
        (YubiKeyEvents::CertificateImportedToSlot(CertificateImportedToSlotEvent { event_id, ..sample_certificate_imported_to_slot() }), event_id),
    ];

    for (event, expected_id) in events {
        assert_eq!(event.aggregate_id(), expected_id);
    }
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(YubiKeyEvents::YubiKeyDetected(sample_yubikey_detected()).event_type(), "YubiKeyDetected");
    assert_eq!(YubiKeyEvents::YubiKeyProvisioned(sample_yubikey_provisioned()).event_type(), "YubiKeyProvisioned");
    assert_eq!(YubiKeyEvents::PinConfigured(sample_pin_configured()).event_type(), "PinConfigured");
    assert_eq!(YubiKeyEvents::PukConfigured(sample_puk_configured()).event_type(), "PukConfigured");
    assert_eq!(YubiKeyEvents::ManagementKeyRotated(sample_management_key_rotated()).event_type(), "ManagementKeyRotated");
    assert_eq!(YubiKeyEvents::SlotAllocationPlanned(sample_slot_allocation_planned()).event_type(), "SlotAllocationPlanned");
    assert_eq!(YubiKeyEvents::KeyGeneratedInSlot(sample_key_generated_in_slot()).event_type(), "KeyGeneratedInSlot");
    assert_eq!(YubiKeyEvents::CertificateImportedToSlot(sample_certificate_imported_to_slot()).event_type(), "CertificateImportedToSlot");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_yubikey_lifecycle() {
    let yubikey_serial = "12345678".to_string();
    let correlation_id = Uuid::now_v7();

    let detected = YubiKeyDetectedEvent {
        yubikey_serial: yubikey_serial.clone(),
        correlation_id,
        ..sample_yubikey_detected()
    };
    let pin_config = PinConfiguredEvent {
        yubikey_serial: yubikey_serial.clone(),
        correlation_id,
        ..sample_pin_configured()
    };
    let puk_config = PukConfiguredEvent {
        yubikey_serial: yubikey_serial.clone(),
        correlation_id,
        ..sample_puk_configured()
    };
    let provisioned = YubiKeyProvisionedEvent {
        yubikey_serial: yubikey_serial.clone(),
        correlation_id,
        ..sample_yubikey_provisioned()
    };

    assert_eq!(detected.yubikey_serial, yubikey_serial);
    assert_eq!(pin_config.yubikey_serial, yubikey_serial);
    assert_eq!(puk_config.yubikey_serial, yubikey_serial);
    assert_eq!(provisioned.yubikey_serial, yubikey_serial);
}

#[test]
fn test_slot_workflow() {
    let yubikey_serial = "12345678".to_string();
    let slot = "9a".to_string();
    let key_id = test_key_id();
    let cert_id = test_cert_id();
    let correlation_id = Uuid::now_v7();

    let allocation = SlotAllocationPlannedEvent {
        yubikey_serial: yubikey_serial.clone(),
        slot: slot.clone(),
        correlation_id,
        ..sample_slot_allocation_planned()
    };
    let key_gen = KeyGeneratedInSlotEvent {
        yubikey_serial: yubikey_serial.clone(),
        slot: slot.clone(),
        key_id,
        correlation_id,
        ..sample_key_generated_in_slot()
    };
    let cert_import = CertificateImportedToSlotEvent {
        yubikey_serial: yubikey_serial.clone(),
        slot: slot.clone(),
        cert_id,
        correlation_id,
        ..sample_certificate_imported_to_slot()
    };

    assert_eq!(allocation.slot, slot);
    assert_eq!(key_gen.slot, slot);
    assert_eq!(cert_import.slot, slot);
    assert_eq!(key_gen.key_id, key_id);
    assert_eq!(cert_import.cert_id, cert_id);
}

#[test]
fn test_management_key_rotation() {
    let yubikey_serial = "12345678".to_string();
    let correlation_id = Uuid::now_v7();

    let rotation = ManagementKeyRotatedEvent {
        yubikey_serial: yubikey_serial.clone(),
        algorithm: "AES256".to_string(),
        correlation_id,
        ..sample_management_key_rotated()
    };

    assert_eq!(rotation.yubikey_serial, yubikey_serial);
    assert_eq!(rotation.algorithm, "AES256");
}

#[test]
fn test_yubikey_slot_configuration() {
    let slot = test_yubikey_slot();
    assert_eq!(slot.slot_id, "9a");
    assert_ne!(slot.key_id, Uuid::nil());
    assert!(matches!(slot.purpose, KeyPurpose::Authentication));
}
