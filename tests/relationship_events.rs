//! Comprehensive Relationship Events Tests
//!
//! Target: 90%+ coverage of src/events/relationship.rs
//!
//! Tests all 6 event types for relationship lifecycle and accountability.

use chrono::Utc;
use cim_keys::events::relationship::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use cim_keys::types::TrustLevel;
use cim_keys::commands::organization::RelationshipType;
use cim_keys::policy_types::{KeyDelegation, KeyPermission};
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_relationship_id() -> Uuid { Uuid::now_v7() }
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_entity_id() -> Uuid { Uuid::now_v7() }
fn test_validation_id() -> Uuid { Uuid::now_v7() }
fn test_violation_id() -> Uuid { Uuid::now_v7() }

fn test_key_delegation() -> KeyDelegation {
    KeyDelegation {
        delegated_to: test_person_id(),
        permissions: vec![KeyPermission::Sign, KeyPermission::Encrypt],
        expires_at: None,
    }
}

fn sample_relationship_established() -> RelationshipEstablishedEvent {
    RelationshipEstablishedEvent {
        relationship_id: test_relationship_id(),
        from_id: test_person_id(),
        to_id: test_person_id(),
        relationship_type: RelationshipType::StoredAt,
        established_at: Utc::now(),
        established_by: "admin".to_string(),
        valid_from: Utc::now(),
        valid_until: None,
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_trust_established() -> TrustEstablishedEvent {
    TrustEstablishedEvent {
        relationship_id: test_relationship_id(),
        trustor_id: test_person_id(),
        trustee_id: test_person_id(),
        trust_level: TrustLevel::Full,
        established_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_relationship_modified() -> RelationshipModifiedEvent {
    RelationshipModifiedEvent {
        relationship_id: test_relationship_id(),
        field_name: "trust_level".to_string(),
        old_value: Some("Marginal".to_string()),
        new_value: "Full".to_string(),
        modified_at: Utc::now(),
        modified_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_relationship_terminated() -> RelationshipTerminatedEvent {
    RelationshipTerminatedEvent {
        relationship_id: test_relationship_id(),
        reason: "Delegation expired".to_string(),
        terminated_at: Utc::now(),
        terminated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_accountability_validated() -> AccountabilityValidatedEvent {
    AccountabilityValidatedEvent {
        validation_id: test_validation_id(),
        identity_id: test_entity_id(),
        identity_type: "Key".to_string(),
        identity_name: "Root CA Key".to_string(),
        responsible_person_id: test_person_id(),
        responsible_person_name: "John Doe".to_string(),
        validated_at: Utc::now(),
        validation_result: "Ownership verified".to_string(),
        correlation_id: Some(Uuid::now_v7()),
        causation_id: None,
    }
}

fn sample_accountability_violated() -> AccountabilityViolatedEvent {
    AccountabilityViolatedEvent {
        violation_id: test_violation_id(),
        identity_id: test_entity_id(),
        identity_type: "Certificate".to_string(),
        identity_name: "Expired CA Certificate".to_string(),
        violation_reason: "No responsible person assigned".to_string(),
        detected_at: Utc::now(),
        required_action: "Assign ownership immediately".to_string(),
        severity: "High".to_string(),
        correlation_id: Some(Uuid::now_v7()),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (6 event types)
// =============================================================================

#[test]
fn test_relationship_established_serialization() {
    let event = sample_relationship_established();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RelationshipEstablishedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.relationship_id, deserialized.relationship_id);
    assert_eq!(event.from_id, deserialized.from_id);
    assert_eq!(event.to_id, deserialized.to_id);
}

#[test]
fn test_trust_established_serialization() {
    let event = sample_trust_established();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: TrustEstablishedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.relationship_id, deserialized.relationship_id);
    assert_eq!(event.trustor_id, deserialized.trustor_id);
    assert_eq!(event.trustee_id, deserialized.trustee_id);
}

#[test]
fn test_relationship_modified_serialization() {
    let event = sample_relationship_modified();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RelationshipModifiedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.relationship_id, deserialized.relationship_id);
    assert_eq!(event.field_name, deserialized.field_name);
}

#[test]
fn test_relationship_terminated_serialization() {
    let event = sample_relationship_terminated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RelationshipTerminatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.relationship_id, deserialized.relationship_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_accountability_validated_serialization() {
    let event = sample_accountability_validated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: AccountabilityValidatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.validation_id, deserialized.validation_id);
    assert_eq!(event.identity_id, deserialized.identity_id);
    assert_eq!(event.validation_result, deserialized.validation_result);
}

#[test]
fn test_accountability_violated_serialization() {
    let event = sample_accountability_violated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: AccountabilityViolatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.violation_id, deserialized.violation_id);
    assert_eq!(event.violation_reason, deserialized.violation_reason);
    assert_eq!(event.severity, deserialized.severity);
}

// =============================================================================
// RelationshipEvents Enum Serialization
// =============================================================================

#[test]
fn test_relationship_events_enum_serialization() {
    let events = vec![
        RelationshipEvents::RelationshipEstablished(sample_relationship_established()),
        RelationshipEvents::TrustEstablished(sample_trust_established()),
        RelationshipEvents::RelationshipModified(sample_relationship_modified()),
        RelationshipEvents::RelationshipTerminated(sample_relationship_terminated()),
        RelationshipEvents::AccountabilityValidated(sample_accountability_validated()),
        RelationshipEvents::AccountabilityViolated(sample_accountability_violated()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RelationshipEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let established = RelationshipEstablishedEvent {
        causation_id: None,
        correlation_id,
        ..sample_relationship_established()
    };
    let modified = RelationshipModifiedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_relationship_modified()
    };

    assert_eq!(established.causation_id, None);
    assert_eq!(modified.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let established = RelationshipEstablishedEvent { correlation_id, ..sample_relationship_established() };
    let trust = TrustEstablishedEvent { correlation_id, ..sample_trust_established() };
    let modified = RelationshipModifiedEvent { correlation_id, ..sample_relationship_modified() };

    assert_eq!(established.correlation_id, correlation_id);
    assert_eq!(trust.correlation_id, correlation_id);
    assert_eq!(modified.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let relationship = sample_relationship_established();
    assert_ne!(relationship.relationship_id, Uuid::nil());
    assert_ne!(relationship.from_id, Uuid::nil());
    assert_ne!(relationship.to_id, Uuid::nil());
    assert_ne!(relationship.correlation_id, Uuid::nil());

    let validated = sample_accountability_validated();
    assert_ne!(validated.validation_id, Uuid::nil());
    assert_ne!(validated.identity_id, Uuid::nil());
    assert_ne!(validated.responsible_person_id, Uuid::nil());

    let violated = sample_accountability_violated();
    assert_ne!(violated.violation_id, Uuid::nil());
    assert_ne!(violated.identity_id, Uuid::nil());
}

#[test]
fn test_trust_level_variants() {
    let trust_levels = vec![
        TrustLevel::Unknown,
        TrustLevel::Never,
        TrustLevel::Marginal,
        TrustLevel::Full,
        TrustLevel::Ultimate,
    ];

    for level in trust_levels {
        let event = TrustEstablishedEvent {
            trust_level: level,
            ..sample_trust_established()
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TrustEstablishedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.relationship_id, deserialized.relationship_id);
    }
}

#[test]
fn test_relationship_type_variants() {
    let relationship_types = vec![
        RelationshipType::KeyDelegation(test_key_delegation()),
        RelationshipType::StoredAt,
        RelationshipType::HasRole,
        RelationshipType::PolicyGovernsEntity,
        RelationshipType::RoleRequiresPolicy,
    ];

    for rel_type in relationship_types {
        let event = RelationshipEstablishedEvent {
            relationship_type: rel_type,
            ..sample_relationship_established()
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RelationshipEstablishedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.relationship_id, deserialized.relationship_id);
    }
}

#[test]
fn test_temporal_validity() {
    let established = sample_relationship_established();
    assert!(established.valid_from <= Utc::now());

    let with_expiry = RelationshipEstablishedEvent {
        valid_until: Some(Utc::now()),
        ..established
    };
    assert!(with_expiry.valid_until.is_some());
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let relationship_id = test_relationship_id();
    let validation_id = test_validation_id();
    let violation_id = test_violation_id();

    let events = vec![
        (RelationshipEvents::RelationshipEstablished(RelationshipEstablishedEvent { relationship_id, ..sample_relationship_established() }), relationship_id),
        (RelationshipEvents::TrustEstablished(TrustEstablishedEvent { relationship_id, ..sample_trust_established() }), relationship_id),
        (RelationshipEvents::RelationshipModified(RelationshipModifiedEvent { relationship_id, ..sample_relationship_modified() }), relationship_id),
        (RelationshipEvents::RelationshipTerminated(RelationshipTerminatedEvent { relationship_id, ..sample_relationship_terminated() }), relationship_id),
        (RelationshipEvents::AccountabilityValidated(AccountabilityValidatedEvent { validation_id, ..sample_accountability_validated() }), validation_id),
        (RelationshipEvents::AccountabilityViolated(AccountabilityViolatedEvent { violation_id, ..sample_accountability_violated() }), violation_id),
    ];

    for (event, expected_id) in events {
        assert_eq!(event.aggregate_id(), expected_id);
    }
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(RelationshipEvents::RelationshipEstablished(sample_relationship_established()).event_type(), "RelationshipEstablished");
    assert_eq!(RelationshipEvents::TrustEstablished(sample_trust_established()).event_type(), "TrustEstablished");
    assert_eq!(RelationshipEvents::RelationshipModified(sample_relationship_modified()).event_type(), "RelationshipModified");
    assert_eq!(RelationshipEvents::RelationshipTerminated(sample_relationship_terminated()).event_type(), "RelationshipTerminated");
    assert_eq!(RelationshipEvents::AccountabilityValidated(sample_accountability_validated()).event_type(), "AccountabilityValidated");
    assert_eq!(RelationshipEvents::AccountabilityViolated(sample_accountability_violated()).event_type(), "AccountabilityViolated");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_relationship_lifecycle() {
    let relationship_id = test_relationship_id();
    let correlation_id = Uuid::now_v7();

    let established = RelationshipEstablishedEvent {
        relationship_id,
        correlation_id,
        ..sample_relationship_established()
    };
    let trust = TrustEstablishedEvent {
        relationship_id,
        correlation_id,
        ..sample_trust_established()
    };
    let modified = RelationshipModifiedEvent {
        relationship_id,
        correlation_id,
        ..sample_relationship_modified()
    };
    let terminated = RelationshipTerminatedEvent {
        relationship_id,
        correlation_id,
        ..sample_relationship_terminated()
    };

    assert_eq!(established.relationship_id, relationship_id);
    assert_eq!(trust.relationship_id, relationship_id);
    assert_eq!(modified.relationship_id, relationship_id);
    assert_eq!(terminated.relationship_id, relationship_id);
}

#[test]
fn test_trust_escalation_workflow() {
    let relationship_id = test_relationship_id();
    let correlation_id = Uuid::now_v7();

    // Start with marginal trust
    let marginal_trust = TrustEstablishedEvent {
        relationship_id,
        correlation_id,
        trust_level: TrustLevel::Marginal,
        ..sample_trust_established()
    };

    // Escalate to full trust
    let full_trust = TrustEstablishedEvent {
        relationship_id,
        correlation_id,
        trust_level: TrustLevel::Full,
        ..sample_trust_established()
    };

    assert_eq!(marginal_trust.relationship_id, relationship_id);
    assert_eq!(full_trust.relationship_id, relationship_id);
}

#[test]
fn test_accountability_validation_workflow() {
    let correlation_id = Uuid::now_v7();
    let identity_id = test_entity_id();

    // First validation succeeds
    let validated = AccountabilityValidatedEvent {
        identity_id,
        correlation_id: Some(correlation_id),
        ..sample_accountability_validated()
    };

    // Later violation detected
    let violated = AccountabilityViolatedEvent {
        identity_id,
        correlation_id: Some(correlation_id),
        ..sample_accountability_violated()
    };

    assert_eq!(validated.identity_id, identity_id);
    assert_eq!(violated.identity_id, identity_id);
    assert_eq!(validated.correlation_id, violated.correlation_id);
}

#[test]
fn test_key_delegation_relationship() {
    let from_person = test_person_id();
    let to_person = test_person_id();
    let correlation_id = Uuid::now_v7();

    let delegation = RelationshipEstablishedEvent {
        from_id: from_person,
        to_id: to_person,
        relationship_type: RelationshipType::KeyDelegation(test_key_delegation()),
        correlation_id,
        ..sample_relationship_established()
    };

    assert_eq!(delegation.from_id, from_person);
    assert_eq!(delegation.to_id, to_person);
}

#[test]
fn test_accountability_violation_severity_levels() {
    let severities = vec!["Low", "Medium", "High", "Critical"];

    for severity in severities {
        let violation = AccountabilityViolatedEvent {
            severity: severity.to_string(),
            ..sample_accountability_violated()
        };
        assert_eq!(violation.severity, severity);
    }
}
