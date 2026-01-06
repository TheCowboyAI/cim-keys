//! Comprehensive Certificate Events Tests
//!
//! Target: 90%+ coverage of src/events/certificate.rs
//!
//! Tests all 10 event types for certificate lifecycle, PKI hierarchy, and validation.

#![allow(deprecated)]

use chrono::Utc;
use cim_keys::events::certificate::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_cert_id() -> Uuid { Uuid::now_v7() }
fn test_key_id() -> Uuid { Uuid::now_v7() }
fn test_ca_id() -> Uuid { Uuid::now_v7() }
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_org_id() -> Uuid { Uuid::now_v7() }
fn test_export_id() -> Uuid { Uuid::now_v7() }

fn sample_certificate_generated() -> CertificateGeneratedEvent {
    CertificateGeneratedEvent {
        cert_id: test_cert_id(),
        key_id: test_key_id(),
        subject: "CN=example.com,O=Example Org".to_string(),
        issuer: Some(test_ca_id()),
        not_before: Utc::now(),
        not_after: Utc::now() + chrono::Duration::days(365),
        is_ca: false,
        san: vec!["example.com".to_string(), "www.example.com".to_string()],
        key_usage: vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
        extended_key_usage: vec!["serverAuth".to_string(), "clientAuth".to_string()],
        correlation_id: Uuid::now_v7(),
        causation_id: None,
        // New typed fields (None for backward compat testing)
        subject_name: None,
        subject_alt_name: None,
        key_usage_ext: None,
        extended_key_usage_ext: None,
        validity: None,
        basic_constraints: None,
    }
}

fn sample_certificate_signed() -> CertificateSignedEvent {
    CertificateSignedEvent {
        cert_id: test_cert_id(),
        signed_by: test_ca_id(),
        signature_algorithm: "SHA256-RSA".to_string(),
        signed_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_certificate_revoked() -> CertificateRevokedEvent {
    CertificateRevokedEvent {
        cert_id: test_cert_id(),
        reason: "Key compromise suspected".to_string(),
        revoked_at: Utc::now(),
        revoked_by: "security_admin".to_string(),
        crl_distribution_point: Some("http://crl.example.com/crl.pem".to_string()),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_certificate_renewed() -> CertificateRenewedEvent {
    CertificateRenewedEvent {
        old_cert_id: test_cert_id(),
        new_cert_id: Uuid::now_v7(),
        renewed_at: Utc::now(),
        renewed_by: "cert_admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_certificate_validated() -> CertificateValidatedEvent {
    CertificateValidatedEvent {
        cert_id: test_cert_id(),
        validation_method: "OCSP".to_string(),
        validation_result: true,
        validation_errors: vec![],
        validated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_certificate_exported() -> CertificateExportedEvent {
    CertificateExportedEvent {
        export_id: test_export_id(),
        cert_id: test_cert_id(),
        export_format: "PEM".to_string(),
        destination_path: "/mnt/encrypted/certs/cert.pem".to_string(),
        exported_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_pki_hierarchy_created() -> PkiHierarchyCreatedEvent {
    PkiHierarchyCreatedEvent {
        root_ca_id: test_cert_id(),
        intermediate_cas: vec![Uuid::now_v7(), Uuid::now_v7()],
        created_by: "pki_admin".to_string(),
        organization_id: test_org_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_certificate_activated() -> CertificateActivatedEvent {
    CertificateActivatedEvent {
        cert_id: test_cert_id(),
        activated_at: Utc::now(),
        activated_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_certificate_suspended() -> CertificateSuspendedEvent {
    CertificateSuspendedEvent {
        cert_id: test_cert_id(),
        reason: "Security review in progress".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_certificate_expired() -> CertificateExpiredEvent {
    CertificateExpiredEvent {
        cert_id: test_cert_id(),
        expired_at: Utc::now(),
        not_after: Utc::now() - chrono::Duration::seconds(1),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (10 event types)
// =============================================================================

#[test]
fn test_certificate_generated_serialization() {
    let event = sample_certificate_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateGeneratedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.cert_id, deserialized.cert_id);
    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.san, deserialized.san);
    assert_eq!(event.key_usage, deserialized.key_usage);
}

#[test]
fn test_certificate_signed_serialization() {
    let event = sample_certificate_signed();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateSignedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.cert_id, deserialized.cert_id);
    assert_eq!(event.signed_by, deserialized.signed_by);
}

#[test]
fn test_certificate_revoked_serialization() {
    let event = sample_certificate_revoked();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateRevokedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.cert_id, deserialized.cert_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_certificate_renewed_serialization() {
    let event = sample_certificate_renewed();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateRenewedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.old_cert_id, deserialized.old_cert_id);
    assert_eq!(event.new_cert_id, deserialized.new_cert_id);
}

#[test]
fn test_certificate_validated_serialization() {
    let event = sample_certificate_validated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateValidatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.cert_id, deserialized.cert_id);
    assert_eq!(event.validation_result, deserialized.validation_result);
}

#[test]
fn test_certificate_exported_serialization() {
    let event = sample_certificate_exported();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateExportedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.export_id, deserialized.export_id);
    assert_eq!(event.cert_id, deserialized.cert_id);
}

#[test]
fn test_pki_hierarchy_created_serialization() {
    let event = sample_pki_hierarchy_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PkiHierarchyCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.root_ca_id, deserialized.root_ca_id);
    assert_eq!(event.intermediate_cas, deserialized.intermediate_cas);
}

#[test]
fn test_certificate_activated_serialization() {
    let event = sample_certificate_activated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateActivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.cert_id, deserialized.cert_id);
}

#[test]
fn test_certificate_suspended_serialization() {
    let event = sample_certificate_suspended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateSuspendedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.cert_id, deserialized.cert_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_certificate_expired_serialization() {
    let event = sample_certificate_expired();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: CertificateExpiredEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.cert_id, deserialized.cert_id);
}

// =============================================================================
// CertificateEvents Enum Serialization
// =============================================================================

#[test]
fn test_certificate_events_enum_serialization() {
    let events = vec![
        CertificateEvents::CertificateGenerated(sample_certificate_generated()),
        CertificateEvents::CertificateSigned(sample_certificate_signed()),
        CertificateEvents::CertificateRevoked(sample_certificate_revoked()),
        CertificateEvents::CertificateRenewed(sample_certificate_renewed()),
        CertificateEvents::CertificateValidated(sample_certificate_validated()),
        CertificateEvents::CertificateExported(sample_certificate_exported()),
        CertificateEvents::PkiHierarchyCreated(sample_pki_hierarchy_created()),
        CertificateEvents::CertificateActivated(sample_certificate_activated()),
        CertificateEvents::CertificateSuspended(sample_certificate_suspended()),
        CertificateEvents::CertificateExpired(sample_certificate_expired()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: CertificateEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let generated = CertificateGeneratedEvent {
        causation_id: None,
        correlation_id,
        ..sample_certificate_generated()
    };
    let signed = CertificateSignedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_certificate_signed()
    };

    assert_eq!(generated.causation_id, None);
    assert_eq!(signed.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let generated = CertificateGeneratedEvent { correlation_id, ..sample_certificate_generated() };
    let signed = CertificateSignedEvent { correlation_id, ..sample_certificate_signed() };
    let validated = CertificateValidatedEvent { correlation_id, ..sample_certificate_validated() };

    assert_eq!(generated.correlation_id, correlation_id);
    assert_eq!(signed.correlation_id, correlation_id);
    assert_eq!(validated.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let cert = sample_certificate_generated();
    assert_ne!(cert.cert_id, Uuid::nil());
    assert_ne!(cert.key_id, Uuid::nil());
    assert_ne!(cert.correlation_id, Uuid::nil());

    let hierarchy = sample_pki_hierarchy_created();
    assert_ne!(hierarchy.root_ca_id, Uuid::nil());
    assert_ne!(hierarchy.organization_id, Uuid::nil());
}

#[test]
fn test_certificate_validity_period() {
    let cert = sample_certificate_generated();
    assert!(cert.not_after > cert.not_before);
}

#[test]
fn test_san_and_key_usage_fields() {
    let cert = sample_certificate_generated();
    assert!(!cert.san.is_empty());
    assert!(!cert.key_usage.is_empty());
    assert!(!cert.extended_key_usage.is_empty());
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let cert_id = test_cert_id();
    let old_cert_id = test_cert_id();
    let root_ca_id = test_cert_id();

    let events = vec![
        CertificateEvents::CertificateGenerated(CertificateGeneratedEvent { cert_id, ..sample_certificate_generated() }),
        CertificateEvents::CertificateSigned(CertificateSignedEvent { cert_id, ..sample_certificate_signed() }),
        CertificateEvents::CertificateRevoked(CertificateRevokedEvent { cert_id, ..sample_certificate_revoked() }),
        CertificateEvents::CertificateRenewed(CertificateRenewedEvent { old_cert_id, ..sample_certificate_renewed() }),
        CertificateEvents::CertificateValidated(CertificateValidatedEvent { cert_id, ..sample_certificate_validated() }),
        CertificateEvents::CertificateExported(CertificateExportedEvent { cert_id, ..sample_certificate_exported() }),
        CertificateEvents::PkiHierarchyCreated(PkiHierarchyCreatedEvent { root_ca_id, ..sample_pki_hierarchy_created() }),
        CertificateEvents::CertificateActivated(CertificateActivatedEvent { cert_id, ..sample_certificate_activated() }),
        CertificateEvents::CertificateSuspended(CertificateSuspendedEvent { cert_id, ..sample_certificate_suspended() }),
        CertificateEvents::CertificateExpired(CertificateExpiredEvent { cert_id, ..sample_certificate_expired() }),
    ];

    // Verify each event returns the correct aggregate ID
    assert_eq!(events[0].aggregate_id(), cert_id);
    assert_eq!(events[1].aggregate_id(), cert_id);
    assert_eq!(events[2].aggregate_id(), cert_id);
    assert_eq!(events[3].aggregate_id(), old_cert_id); // CertificateRenewed uses old_cert_id
    assert_eq!(events[4].aggregate_id(), cert_id);
    assert_eq!(events[5].aggregate_id(), cert_id);
    assert_eq!(events[6].aggregate_id(), root_ca_id); // PkiHierarchyCreated uses root_ca_id
    assert_eq!(events[7].aggregate_id(), cert_id);
    assert_eq!(events[8].aggregate_id(), cert_id);
    assert_eq!(events[9].aggregate_id(), cert_id);
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(CertificateEvents::CertificateGenerated(sample_certificate_generated()).event_type(), "CertificateGenerated");
    assert_eq!(CertificateEvents::CertificateSigned(sample_certificate_signed()).event_type(), "CertificateSigned");
    assert_eq!(CertificateEvents::CertificateRevoked(sample_certificate_revoked()).event_type(), "CertificateRevoked");
    assert_eq!(CertificateEvents::CertificateRenewed(sample_certificate_renewed()).event_type(), "CertificateRenewed");
    assert_eq!(CertificateEvents::CertificateValidated(sample_certificate_validated()).event_type(), "CertificateValidated");
    assert_eq!(CertificateEvents::CertificateExported(sample_certificate_exported()).event_type(), "CertificateExported");
    assert_eq!(CertificateEvents::PkiHierarchyCreated(sample_pki_hierarchy_created()).event_type(), "PkiHierarchyCreated");
    assert_eq!(CertificateEvents::CertificateActivated(sample_certificate_activated()).event_type(), "CertificateActivated");
    assert_eq!(CertificateEvents::CertificateSuspended(sample_certificate_suspended()).event_type(), "CertificateSuspended");
    assert_eq!(CertificateEvents::CertificateExpired(sample_certificate_expired()).event_type(), "CertificateExpired");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_certificate_lifecycle() {
    let cert_id = test_cert_id();
    let correlation_id = Uuid::now_v7();

    let generated = CertificateGeneratedEvent {
        cert_id,
        correlation_id,
        ..sample_certificate_generated()
    };
    let signed = CertificateSignedEvent {
        cert_id,
        correlation_id,
        ..sample_certificate_signed()
    };
    let activated = CertificateActivatedEvent {
        cert_id,
        correlation_id,
        ..sample_certificate_activated()
    };
    let validated = CertificateValidatedEvent {
        cert_id,
        correlation_id,
        ..sample_certificate_validated()
    };
    let suspended = CertificateSuspendedEvent {
        cert_id,
        correlation_id,
        ..sample_certificate_suspended()
    };
    let revoked = CertificateRevokedEvent {
        cert_id,
        correlation_id,
        ..sample_certificate_revoked()
    };

    assert_eq!(generated.cert_id, cert_id);
    assert_eq!(signed.cert_id, cert_id);
    assert_eq!(activated.cert_id, cert_id);
    assert_eq!(validated.cert_id, cert_id);
    assert_eq!(suspended.cert_id, cert_id);
    assert_eq!(revoked.cert_id, cert_id);
}

#[test]
fn test_pki_hierarchy_workflow() {
    let correlation_id = Uuid::now_v7();
    let root_ca_id = test_cert_id();

    let hierarchy = PkiHierarchyCreatedEvent {
        root_ca_id,
        correlation_id,
        ..sample_pki_hierarchy_created()
    };

    assert_eq!(hierarchy.root_ca_id, root_ca_id);
    assert!(!hierarchy.intermediate_cas.is_empty());
}

#[test]
fn test_certificate_renewal_workflow() {
    let old_cert_id = test_cert_id();
    let new_cert_id = Uuid::now_v7();
    let correlation_id = Uuid::now_v7();

    let renewed = CertificateRenewedEvent {
        old_cert_id,
        new_cert_id,
        correlation_id,
        ..sample_certificate_renewed()
    };

    assert_eq!(renewed.old_cert_id, old_cert_id);
    assert_eq!(renewed.new_cert_id, new_cert_id);
    assert_ne!(renewed.old_cert_id, renewed.new_cert_id);
}

#[test]
fn test_certificate_validation_workflow() {
    let cert_id = test_cert_id();
    let correlation_id = Uuid::now_v7();

    // Successful validation
    let valid = CertificateValidatedEvent {
        cert_id,
        correlation_id,
        validation_result: true,
        validation_errors: vec![],
        ..sample_certificate_validated()
    };

    // Failed validation
    let invalid = CertificateValidatedEvent {
        cert_id,
        correlation_id,
        validation_result: false,
        validation_errors: vec!["Expired".to_string(), "Invalid signature".to_string()],
        ..sample_certificate_validated()
    };

    assert!(valid.validation_result);
    assert!(valid.validation_errors.is_empty());
    assert!(!invalid.validation_result);
    assert!(!invalid.validation_errors.is_empty());
}
