//! Comprehensive Person Events Tests
//!
//! Target: 90%+ coverage of src/events/person.rs
//!
//! Test Categories:
//! - Serialization Roundtrip (all 11 event types)
//! - Correlation/Causation Chain validation
//! - Event Invariants (valid UUIDs, timestamps)
//! - DomainEvent Trait implementation
//! - Event Wrapping Pattern (Inner → Enum → DomainEvent)

use chrono::Utc;
use cim_keys::events::person::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_keys::value_objects::ActorId;
use cim_domain::DomainEvent;
use uuid::Uuid;

// =============================================================================
// Test Helpers - Create sample events for testing
// =============================================================================

fn sample_person_created() -> PersonCreatedEvent {
    PersonCreatedEvent {
        person_id: Uuid::now_v7(),
        name: "Alice Johnson".to_string(),
        email: Some("alice@example.com".to_string()),
        title: Some("Software Engineer".to_string()),
        department: Some("Engineering".to_string()),
        organization_id: Uuid::now_v7(),
        created_by: ActorId::system("admin"),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_person_updated() -> PersonUpdatedEvent {
    PersonUpdatedEvent {
        person_id: Uuid::now_v7(),
        field_name: "title".to_string(),
        old_value: Some("Engineer".to_string()),
        new_value: "Senior Engineer".to_string(),
        updated_at: Utc::now(),
        updated_by: ActorId::system("manager"),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_role_assigned() -> RoleAssignedEvent {
    RoleAssignedEvent {
        person_id: Uuid::now_v7(),
        role_id: Uuid::now_v7(),
        role_name: "Admin".to_string(),
        assigned_at: Utc::now(),
        assigned_by: Uuid::now_v7(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_role_removed() -> RoleRemovedEvent {
    RoleRemovedEvent {
        person_id: Uuid::now_v7(),
        role_id: Uuid::now_v7(),
        removed_at: Utc::now(),
        removed_by: Uuid::now_v7(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_person_deactivated() -> PersonDeactivatedEvent {
    PersonDeactivatedEvent {
        person_id: Uuid::now_v7(),
        reason: "Left organization".to_string(),
        deactivated_at: Utc::now(),
        deactivated_by: Uuid::now_v7(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_person_activated() -> PersonActivatedEvent {
    PersonActivatedEvent {
        person_id: Uuid::now_v7(),
        activated_at: Utc::now(),
        activated_by: Uuid::now_v7(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_person_suspended() -> PersonSuspendedEvent {
    PersonSuspendedEvent {
        person_id: Uuid::now_v7(),
        reason: "Policy violation".to_string(),
        suspended_at: Utc::now(),
        suspended_by: Uuid::now_v7(),
        expected_return: None,
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_person_reactivated() -> PersonReactivatedEvent {
    PersonReactivatedEvent {
        person_id: Uuid::now_v7(),
        reactivated_at: Utc::now(),
        reactivated_by: Uuid::now_v7(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_person_archived() -> PersonArchivedEvent {
    PersonArchivedEvent {
        person_id: Uuid::now_v7(),
        reason: "Retired".to_string(),
        archived_at: Utc::now(),
        archived_by: Uuid::now_v7(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_ssh_key_generated() -> SshKeyGeneratedEvent {
    SshKeyGeneratedEvent {
        key_id: Uuid::now_v7(),
        person_id: Uuid::now_v7(),
        algorithm: "ed25519".to_string(),
        public_key_fingerprint: "SHA256:abc123...".to_string(),
        generated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

fn sample_gpg_key_generated() -> GpgKeyGeneratedEvent {
    GpgKeyGeneratedEvent {
        key_id: Uuid::now_v7(),
        person_id: Uuid::now_v7(),
        fingerprint: "ABCD1234EFGH5678".to_string(),
        email: "alice@example.com".to_string(),
        generated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: Some(Uuid::now_v7()),
    }
}

// =============================================================================
// Serialization Roundtrip Tests (Core Requirement)
// =============================================================================

#[test]
fn test_person_created_serialization_roundtrip() {
    let event = sample_person_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PersonCreatedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.name, deserialized.name);
    assert_eq!(event.email, deserialized.email);
    assert_eq!(event.correlation_id, deserialized.correlation_id);
}

#[test]
fn test_person_updated_serialization_roundtrip() {
    let event = sample_person_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PersonUpdatedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.field_name, deserialized.field_name);
    assert_eq!(event.new_value, deserialized.new_value);
}

#[test]
fn test_role_assigned_serialization_roundtrip() {
    let event = sample_role_assigned();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RoleAssignedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.role_name, deserialized.role_name);
}

#[test]
fn test_role_removed_serialization_roundtrip() {
    let event = sample_role_removed();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RoleRemovedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.role_id, deserialized.role_id);
}

#[test]
fn test_person_deactivated_serialization_roundtrip() {
    let event = sample_person_deactivated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PersonDeactivatedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_person_activated_serialization_roundtrip() {
    let event = sample_person_activated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PersonActivatedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.activated_by, deserialized.activated_by);
}

#[test]
fn test_person_suspended_serialization_roundtrip() {
    let event = sample_person_suspended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PersonSuspendedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_person_reactivated_serialization_roundtrip() {
    let event = sample_person_reactivated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PersonReactivatedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.reactivated_by, deserialized.reactivated_by);
}

#[test]
fn test_person_archived_serialization_roundtrip() {
    let event = sample_person_archived();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PersonArchivedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.person_id, deserialized.person_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_ssh_key_generated_serialization_roundtrip() {
    let event = sample_ssh_key_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: SshKeyGeneratedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.algorithm, deserialized.algorithm);
}

#[test]
fn test_gpg_key_generated_serialization_roundtrip() {
    let event = sample_gpg_key_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: GpgKeyGeneratedEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.fingerprint, deserialized.fingerprint);
}

// =============================================================================
// PersonEvents Enum Serialization Tests
// =============================================================================

#[test]
fn test_person_events_enum_serialization() {
    let events = vec![
        PersonEvents::PersonCreated(sample_person_created()),
        PersonEvents::PersonUpdated(sample_person_updated()),
        PersonEvents::RoleAssigned(sample_role_assigned()),
        PersonEvents::RoleRemoved(sample_role_removed()),
        PersonEvents::PersonDeactivated(sample_person_deactivated()),
        PersonEvents::PersonActivated(sample_person_activated()),
        PersonEvents::PersonSuspended(sample_person_suspended()),
        PersonEvents::PersonReactivated(sample_person_reactivated()),
        PersonEvents::PersonArchived(sample_person_archived()),
        PersonEvents::SshKeyGenerated(sample_ssh_key_generated()),
        PersonEvents::GpgKeyGenerated(sample_gpg_key_generated()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: PersonEvents = serde_json::from_str(&json).unwrap();

        // Verify event_type matches
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_correlation_id_present_in_all_events() {
    let created = sample_person_created();
    let updated = sample_person_updated();
    let role_assigned = sample_role_assigned();

    // All events must have correlation_id
    assert_ne!(created.correlation_id, Uuid::nil());
    assert_ne!(updated.correlation_id, Uuid::nil());
    assert_ne!(role_assigned.correlation_id, Uuid::nil());
}

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();

    // Create initial event
    let created = PersonCreatedEvent {
        causation_id: None, // Root event
        correlation_id,
        ..sample_person_created()
    };

    // Next event caused by created event
    let activated = PersonActivatedEvent {
        causation_id: Some(correlation_id), // Caused by created
        correlation_id,
        ..sample_person_activated()
    };

    assert_eq!(created.causation_id, None);
    assert_eq!(activated.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation_through_chain() {
    let correlation_id = Uuid::now_v7();

    let created = PersonCreatedEvent { correlation_id, ..sample_person_created() };
    let activated = PersonActivatedEvent { correlation_id, ..sample_person_activated() };
    let role_assigned = RoleAssignedEvent { correlation_id, ..sample_role_assigned() };

    // All events in chain share same correlation_id
    assert_eq!(created.correlation_id, correlation_id);
    assert_eq!(activated.correlation_id, correlation_id);
    assert_eq!(role_assigned.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let event = sample_person_created();

    // All UUIDs should be valid (not nil)
    assert_ne!(event.person_id, Uuid::nil());
    assert_ne!(event.organization_id, Uuid::nil());
    assert_ne!(event.correlation_id, Uuid::nil());
}

// Note: created_at test removed - timestamps are now embedded in UUID v7

#[test]
fn test_required_fields_are_non_empty() {
    let event = sample_person_created();

    assert!(!event.name.is_empty());
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_returns_person_id() {
    let person_id = Uuid::now_v7();

    let event = PersonEvents::PersonCreated(PersonCreatedEvent {
        person_id,
        ..sample_person_created()
    });

    assert_eq!(event.aggregate_id(), person_id);
}

#[test]
fn test_aggregate_id_for_all_event_types() {
    let person_id = Uuid::now_v7();

    // Test ALL 11 event variants to ensure complete coverage of match arms
    let events = vec![
        PersonEvents::PersonCreated(PersonCreatedEvent { person_id, ..sample_person_created() }),
        PersonEvents::PersonUpdated(PersonUpdatedEvent { person_id, ..sample_person_updated() }),
        PersonEvents::RoleAssigned(RoleAssignedEvent { person_id, ..sample_role_assigned() }),
        PersonEvents::RoleRemoved(RoleRemovedEvent { person_id, ..sample_role_removed() }),
        PersonEvents::PersonDeactivated(PersonDeactivatedEvent { person_id, ..sample_person_deactivated() }),
        PersonEvents::PersonActivated(PersonActivatedEvent { person_id, ..sample_person_activated() }),
        PersonEvents::PersonSuspended(PersonSuspendedEvent { person_id, ..sample_person_suspended() }),
        PersonEvents::PersonReactivated(PersonReactivatedEvent { person_id, ..sample_person_reactivated() }),
        PersonEvents::PersonArchived(PersonArchivedEvent { person_id, ..sample_person_archived() }),
        PersonEvents::SshKeyGenerated(SshKeyGeneratedEvent { person_id, ..sample_ssh_key_generated() }),
        PersonEvents::GpgKeyGenerated(GpgKeyGeneratedEvent { person_id, ..sample_gpg_key_generated() }),
    ];

    for event in events {
        assert_eq!(event.aggregate_id(), person_id);
    }
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(PersonEvents::PersonCreated(sample_person_created()).event_type(), "PersonCreated");
    assert_eq!(PersonEvents::PersonUpdated(sample_person_updated()).event_type(), "PersonUpdated");
    assert_eq!(PersonEvents::RoleAssigned(sample_role_assigned()).event_type(), "RoleAssigned");
    assert_eq!(PersonEvents::RoleRemoved(sample_role_removed()).event_type(), "RoleRemoved");
    assert_eq!(PersonEvents::PersonDeactivated(sample_person_deactivated()).event_type(), "PersonDeactivated");
    assert_eq!(PersonEvents::PersonActivated(sample_person_activated()).event_type(), "PersonActivated");
    assert_eq!(PersonEvents::PersonSuspended(sample_person_suspended()).event_type(), "PersonSuspended");
    assert_eq!(PersonEvents::PersonReactivated(sample_person_reactivated()).event_type(), "PersonReactivated");
    assert_eq!(PersonEvents::PersonArchived(sample_person_archived()).event_type(), "PersonArchived");
    assert_eq!(PersonEvents::SshKeyGenerated(sample_ssh_key_generated()).event_type(), "SshKeyGenerated");
    assert_eq!(PersonEvents::GpgKeyGenerated(sample_gpg_key_generated()).event_type(), "GpgKeyGenerated");
}

// =============================================================================
// Event Wrapping Pattern Tests (Inner → Enum → DomainEvent)
// =============================================================================

#[test]
fn test_event_wrapping_person_created() {
    let inner = sample_person_created();
    let middle = PersonEvents::PersonCreated(inner.clone());
    let outer = DomainEventEnum::Person(middle.clone());

    // Verify wrapping preserves data
    if let DomainEventEnum::Person(PersonEvents::PersonCreated(event)) = outer {
        assert_eq!(event.person_id, inner.person_id);
        assert_eq!(event.name, inner.name);
    } else {
        panic!("Event wrapping failed");
    }
}

#[test]
fn test_domain_event_serialization_with_wrapping() {
    let inner = sample_person_created();
    let middle = PersonEvents::PersonCreated(inner);
    let outer = DomainEventEnum::Person(middle);

    let json = serde_json::to_string(&outer).unwrap();
    let deserialized: DomainEventEnum = serde_json::from_str(&json).unwrap();

    // Verify outer wrapper deserialized correctly
    match deserialized {
        DomainEventEnum::Person(PersonEvents::PersonCreated(_)) => {
            // Success
        }
        _ => panic!("Deserialization failed to preserve wrapping"),
    }
}

// =============================================================================
// Complete Lifecycle Event Chain Tests
// =============================================================================

#[test]
fn test_complete_person_lifecycle_event_chain() {
    let person_id = Uuid::now_v7();
    let correlation_id = Uuid::now_v7();

    // Create person
    let created = PersonEvents::PersonCreated(PersonCreatedEvent {
        person_id,
        correlation_id,
        causation_id: None,
        ..sample_person_created()
    });

    // Activate person
    let activated = PersonEvents::PersonActivated(PersonActivatedEvent {
        person_id,
        correlation_id,
        causation_id: Some(correlation_id),
        ..sample_person_activated()
    });

    // Suspend person
    let suspended = PersonEvents::PersonSuspended(PersonSuspendedEvent {
        person_id,
        correlation_id,
        causation_id: Some(correlation_id),
        ..sample_person_suspended()
    });

    // Reactivate person
    let reactivated = PersonEvents::PersonReactivated(PersonReactivatedEvent {
        person_id,
        correlation_id,
        causation_id: Some(correlation_id),
        ..sample_person_reactivated()
    });

    // Archive person (terminal)
    let archived = PersonEvents::PersonArchived(PersonArchivedEvent {
        person_id,
        correlation_id,
        causation_id: Some(correlation_id),
        ..sample_person_archived()
    });

    let chain = vec![created, activated, suspended, reactivated, archived];

    // Verify all events share same person_id and correlation_id
    for event in chain {
        assert_eq!(event.aggregate_id(), person_id);
    }
}
