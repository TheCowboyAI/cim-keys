//! Comprehensive NATS Account Events Tests
//!
//! Target: 90%+ coverage of src/events/nats_account.rs
//!
//! Tests all 7 event types for NATS account lifecycle and permissions management.

use chrono::Utc;
use cim_keys::events::nats_account::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use cim_keys::types::{NatsEntityType, NatsPermissions};
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_account_id() -> Uuid { Uuid::now_v7() }
fn test_operator_id() -> Uuid { Uuid::now_v7() }
fn test_org_unit_id() -> Uuid { Uuid::now_v7() }
fn test_entity_id() -> Uuid { Uuid::now_v7() }

fn test_permissions() -> NatsPermissions {
    NatsPermissions {
        publish: vec!["account.>".to_string(), "service.>".to_string()],
        subscribe: vec!["account.>".to_string(), "service.>".to_string()],
        allow_responses: true,
        max_payload: Some(1024 * 1024),
    }
}

fn sample_account_created() -> NatsAccountCreatedEvent {
    NatsAccountCreatedEvent {
        account_id: test_account_id(),
        operator_id: test_operator_id(),
        name: "Test Account".to_string(),
        public_key: "NCAAABBBCCCDDDEEEFFFGGGHHHIIIJJJKKKLLLMMMNNNOOOPPPQQQRRR".to_string(),
        is_system: false,
        created_by: "admin".to_string(),
        organization_unit_id: Some(test_org_unit_id()),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_account_updated() -> NatsAccountUpdatedEvent {
    NatsAccountUpdatedEvent {
        account_id: test_account_id(),
        field_name: "name".to_string(),
        old_value: Some("Old Name".to_string()),
        new_value: "New Name".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_permissions_set() -> NatsPermissionsSetEvent {
    NatsPermissionsSetEvent {
        account_id: test_account_id(),
        entity_id: test_entity_id(),
        entity_type: NatsEntityType::Account,
        permissions: test_permissions(),
        set_at: Utc::now(),
        set_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_account_suspended() -> NatsAccountSuspendedEvent {
    NatsAccountSuspendedEvent {
        account_id: test_account_id(),
        reason: "Payment overdue".to_string(),
        suspended_at: Utc::now(),
        suspended_by: "billing_system".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_account_reactivated() -> NatsAccountReactivatedEvent {
    NatsAccountReactivatedEvent {
        account_id: test_account_id(),
        permissions: Some(test_permissions()),
        reactivated_at: Utc::now(),
        reactivated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_account_activated() -> NatsAccountActivatedEvent {
    NatsAccountActivatedEvent {
        account_id: test_account_id(),
        permissions: Some(test_permissions()),
        activated_at: Utc::now(),
        activated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_account_deleted() -> NatsAccountDeletedEvent {
    NatsAccountDeletedEvent {
        account_id: test_account_id(),
        reason: "Account closure requested".to_string(),
        deleted_at: Utc::now(),
        deleted_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (7 event types)
// =============================================================================

#[test]
fn test_account_created_serialization() {
    let event = sample_account_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsAccountCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.account_id, deserialized.account_id);
    assert_eq!(event.operator_id, deserialized.operator_id);
    assert_eq!(event.is_system, deserialized.is_system);
}

#[test]
fn test_account_updated_serialization() {
    let event = sample_account_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsAccountUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.account_id, deserialized.account_id);
    assert_eq!(event.field_name, deserialized.field_name);
}

#[test]
fn test_permissions_set_serialization() {
    let event = sample_permissions_set();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsPermissionsSetEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.account_id, deserialized.account_id);
    assert_eq!(event.entity_id, deserialized.entity_id);
    assert_eq!(event.permissions.allow_responses, deserialized.permissions.allow_responses);
}

#[test]
fn test_account_suspended_serialization() {
    let event = sample_account_suspended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsAccountSuspendedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.account_id, deserialized.account_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_account_reactivated_serialization() {
    let event = sample_account_reactivated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsAccountReactivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.account_id, deserialized.account_id);
    assert!(deserialized.permissions.is_some());
}

#[test]
fn test_account_activated_serialization() {
    let event = sample_account_activated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsAccountActivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.account_id, deserialized.account_id);
    assert!(deserialized.permissions.is_some());
}

#[test]
fn test_account_deleted_serialization() {
    let event = sample_account_deleted();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsAccountDeletedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.account_id, deserialized.account_id);
    assert_eq!(event.reason, deserialized.reason);
}

// =============================================================================
// NatsAccountEvents Enum Serialization
// =============================================================================

#[test]
fn test_account_events_enum_serialization() {
    let events = vec![
        NatsAccountEvents::NatsAccountCreated(sample_account_created()),
        NatsAccountEvents::NatsAccountUpdated(sample_account_updated()),
        NatsAccountEvents::NatsPermissionsSet(sample_permissions_set()),
        NatsAccountEvents::NatsAccountSuspended(sample_account_suspended()),
        NatsAccountEvents::NatsAccountReactivated(sample_account_reactivated()),
        NatsAccountEvents::NatsAccountActivated(sample_account_activated()),
        NatsAccountEvents::NatsAccountDeleted(sample_account_deleted()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: NatsAccountEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let created = NatsAccountCreatedEvent {
        causation_id: None,
        correlation_id,
        ..sample_account_created()
    };
    let activated = NatsAccountActivatedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_account_activated()
    };

    assert_eq!(created.causation_id, None);
    assert_eq!(activated.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let created = NatsAccountCreatedEvent { correlation_id, ..sample_account_created() };
    let activated = NatsAccountActivatedEvent { correlation_id, ..sample_account_activated() };
    let suspended = NatsAccountSuspendedEvent { correlation_id, ..sample_account_suspended() };

    assert_eq!(created.correlation_id, correlation_id);
    assert_eq!(activated.correlation_id, correlation_id);
    assert_eq!(suspended.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let account = sample_account_created();
    assert_ne!(account.account_id, Uuid::nil());
    assert_ne!(account.operator_id, Uuid::nil());
    assert_ne!(account.correlation_id, Uuid::nil());

    let permissions = sample_permissions_set();
    assert_ne!(permissions.entity_id, Uuid::nil());
}

#[test]
fn test_public_key_format() {
    let account = sample_account_created();
    // NATS public keys should be base32-encoded and start with 'N'
    assert!(account.public_key.starts_with('N'));
    assert!(account.public_key.len() >= 56); // Typical NATS key length
}

#[test]
fn test_permissions_structure() {
    let permissions = test_permissions();
    assert!(!permissions.publish.is_empty());
    assert!(!permissions.subscribe.is_empty());
    assert!(permissions.allow_responses);
    assert!(permissions.max_payload.is_some());
}

#[test]
fn test_entity_types() {
    let event = sample_permissions_set();
    // Test that entity_type can be serialized/deserialized
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsPermissionsSetEvent = serde_json::from_str(&json).unwrap();

    // Verify entity type is preserved
    assert!(matches!(deserialized.entity_type, NatsEntityType::Account));
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let account_id = test_account_id();

    let events = vec![
        NatsAccountEvents::NatsAccountCreated(NatsAccountCreatedEvent { account_id, ..sample_account_created() }),
        NatsAccountEvents::NatsAccountUpdated(NatsAccountUpdatedEvent { account_id, ..sample_account_updated() }),
        NatsAccountEvents::NatsPermissionsSet(NatsPermissionsSetEvent { account_id, ..sample_permissions_set() }),
        NatsAccountEvents::NatsAccountSuspended(NatsAccountSuspendedEvent { account_id, ..sample_account_suspended() }),
        NatsAccountEvents::NatsAccountReactivated(NatsAccountReactivatedEvent { account_id, ..sample_account_reactivated() }),
        NatsAccountEvents::NatsAccountActivated(NatsAccountActivatedEvent { account_id, ..sample_account_activated() }),
        NatsAccountEvents::NatsAccountDeleted(NatsAccountDeletedEvent { account_id, ..sample_account_deleted() }),
    ];

    for event in events {
        assert_eq!(event.aggregate_id(), account_id);
    }
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(NatsAccountEvents::NatsAccountCreated(sample_account_created()).event_type(), "NatsAccountCreated");
    assert_eq!(NatsAccountEvents::NatsAccountUpdated(sample_account_updated()).event_type(), "NatsAccountUpdated");
    assert_eq!(NatsAccountEvents::NatsPermissionsSet(sample_permissions_set()).event_type(), "NatsPermissionsSet");
    assert_eq!(NatsAccountEvents::NatsAccountSuspended(sample_account_suspended()).event_type(), "NatsAccountSuspended");
    assert_eq!(NatsAccountEvents::NatsAccountReactivated(sample_account_reactivated()).event_type(), "NatsAccountReactivated");
    assert_eq!(NatsAccountEvents::NatsAccountActivated(sample_account_activated()).event_type(), "NatsAccountActivated");
    assert_eq!(NatsAccountEvents::NatsAccountDeleted(sample_account_deleted()).event_type(), "NatsAccountDeleted");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_account_lifecycle() {
    let account_id = test_account_id();
    let correlation_id = Uuid::now_v7();

    let created = NatsAccountCreatedEvent {
        account_id,
        correlation_id,
        ..sample_account_created()
    };
    let activated = NatsAccountActivatedEvent {
        account_id,
        correlation_id,
        ..sample_account_activated()
    };
    let permissions_set = NatsPermissionsSetEvent {
        account_id,
        correlation_id,
        ..sample_permissions_set()
    };
    let suspended = NatsAccountSuspendedEvent {
        account_id,
        correlation_id,
        ..sample_account_suspended()
    };
    let reactivated = NatsAccountReactivatedEvent {
        account_id,
        correlation_id,
        ..sample_account_reactivated()
    };
    let deleted = NatsAccountDeletedEvent {
        account_id,
        correlation_id,
        ..sample_account_deleted()
    };

    assert_eq!(created.account_id, account_id);
    assert_eq!(activated.account_id, account_id);
    assert_eq!(permissions_set.account_id, account_id);
    assert_eq!(suspended.account_id, account_id);
    assert_eq!(reactivated.account_id, account_id);
    assert_eq!(deleted.account_id, account_id);
}

#[test]
fn test_system_account_creation() {
    let system_account = NatsAccountCreatedEvent {
        is_system: true,
        name: "SYS".to_string(),
        ..sample_account_created()
    };

    assert!(system_account.is_system);
    assert_eq!(system_account.name, "SYS");
}

#[test]
fn test_account_with_organization_unit_mapping() {
    let org_unit_id = test_org_unit_id();
    let account = NatsAccountCreatedEvent {
        organization_unit_id: Some(org_unit_id),
        ..sample_account_created()
    };

    assert_eq!(account.organization_unit_id, Some(org_unit_id));
}

#[test]
fn test_permissions_update_workflow() {
    let account_id = test_account_id();
    let correlation_id = Uuid::now_v7();

    // Set initial permissions
    let initial = NatsPermissionsSetEvent {
        account_id,
        correlation_id,
        permissions: NatsPermissions {
            publish: vec!["basic.>".to_string()],
            subscribe: vec!["basic.>".to_string()],
            allow_responses: false,
            max_payload: Some(64 * 1024),
        },
        ..sample_permissions_set()
    };

    // Update permissions with expanded access
    let updated = NatsPermissionsSetEvent {
        account_id,
        correlation_id,
        permissions: NatsPermissions {
            publish: vec!["basic.>".to_string(), "advanced.>".to_string()],
            subscribe: vec!["basic.>".to_string(), "advanced.>".to_string()],
            allow_responses: true,
            max_payload: Some(1024 * 1024),
        },
        ..sample_permissions_set()
    };

    assert_eq!(initial.account_id, account_id);
    assert_eq!(updated.account_id, account_id);
    assert!(!initial.permissions.allow_responses);
    assert!(updated.permissions.allow_responses);
}
