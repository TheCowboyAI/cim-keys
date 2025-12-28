//! Comprehensive NATS User Events Tests
//!
//! Target: 90%+ coverage of src/events/nats_user.rs
//!
//! Tests all 10 event types for NATS user lifecycle, permissions, and service accounts.

use chrono::Utc;
use cim_keys::events::nats_user::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use cim_keys::types::NatsPermissions;
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_user_id() -> Uuid { Uuid::now_v7() }
fn test_account_id() -> Uuid { Uuid::now_v7() }
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_org_unit_id() -> Uuid { Uuid::now_v7() }
fn test_org_id() -> Uuid { Uuid::now_v7() }
fn test_secret_id() -> Uuid { Uuid::now_v7() }

fn test_permissions() -> NatsPermissions {
    NatsPermissions {
        publish: vec!["user.>".to_string()],
        subscribe: vec!["user.>".to_string()],
        allow_responses: true,
        max_payload: Some(512 * 1024),
    }
}

fn sample_user_created() -> NatsUserCreatedEvent {
    NatsUserCreatedEvent {
        user_id: test_user_id(),
        account_id: test_account_id(),
        name: "test.user".to_string(),
        public_key: "UCAAABBBCCCDDDEEEFFFGGGHHHIIIJJJKKKLLLMMMNNNOOOPPPQQQRRR".to_string(),
        created_at: Utc::now(),
        created_by: "admin".to_string(),
        person_id: Some(test_person_id()),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_user_updated() -> NatsUserUpdatedEvent {
    NatsUserUpdatedEvent {
        user_id: test_user_id(),
        field_name: "name".to_string(),
        old_value: Some("old.user".to_string()),
        new_value: "new.user".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_user_permissions_set() -> NatsUserPermissionsSetEvent {
    NatsUserPermissionsSetEvent {
        user_id: test_user_id(),
        permissions: test_permissions(),
        set_at: Utc::now(),
        set_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_user_suspended() -> NatsUserSuspendedEvent {
    NatsUserSuspendedEvent {
        user_id: test_user_id(),
        reason: "Security review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: "security_team".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_user_reactivated() -> NatsUserReactivatedEvent {
    NatsUserReactivatedEvent {
        user_id: test_user_id(),
        permissions: Some(test_permissions()),
        reactivated_at: Utc::now(),
        reactivated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_service_account_created() -> ServiceAccountCreatedEvent {
    ServiceAccountCreatedEvent {
        service_account_id: test_user_id(),
        name: "api-service".to_string(),
        purpose: "Backend API service account".to_string(),
        owning_unit_id: test_org_unit_id(),
        responsible_person_id: test_person_id(),
        created_at: Utc::now(),
        correlation_id: Some(Uuid::now_v7()),
        causation_id: None,
    }
}

fn sample_agent_created() -> AgentCreatedEvent {
    AgentCreatedEvent {
        agent_id: test_user_id(),
        name: "monitoring-agent".to_string(),
        agent_type: "Monitoring".to_string(),
        responsible_person_id: test_person_id(),
        organization_id: test_org_id(),
        created_at: Utc::now(),
        correlation_id: Some(Uuid::now_v7()),
        causation_id: None,
    }
}

fn sample_user_activated() -> NatsUserActivatedEvent {
    NatsUserActivatedEvent {
        user_id: test_user_id(),
        permissions: Some(test_permissions()),
        activated_at: Utc::now(),
        activated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_user_deleted() -> NatsUserDeletedEvent {
    NatsUserDeletedEvent {
        user_id: test_user_id(),
        reason: "User left organization".to_string(),
        deleted_at: Utc::now(),
        deleted_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_totp_secret_generated() -> TotpSecretGeneratedEvent {
    TotpSecretGeneratedEvent {
        user_id: test_user_id(),
        secret_id: test_secret_id(),
        algorithm: "SHA256".to_string(),
        generated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (10 event types)
// =============================================================================

#[test]
fn test_user_created_serialization() {
    let event = sample_user_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsUserCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.user_id, deserialized.user_id);
    assert_eq!(event.account_id, deserialized.account_id);
    assert_eq!(event.person_id, deserialized.person_id);
}

#[test]
fn test_user_updated_serialization() {
    let event = sample_user_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsUserUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.user_id, deserialized.user_id);
    assert_eq!(event.field_name, deserialized.field_name);
}

#[test]
fn test_user_permissions_set_serialization() {
    let event = sample_user_permissions_set();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsUserPermissionsSetEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.user_id, deserialized.user_id);
    assert_eq!(event.permissions.allow_responses, deserialized.permissions.allow_responses);
}

#[test]
fn test_user_suspended_serialization() {
    let event = sample_user_suspended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsUserSuspendedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.user_id, deserialized.user_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_user_reactivated_serialization() {
    let event = sample_user_reactivated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsUserReactivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.user_id, deserialized.user_id);
    assert!(deserialized.permissions.is_some());
}

#[test]
fn test_service_account_created_serialization() {
    let event = sample_service_account_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ServiceAccountCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.service_account_id, deserialized.service_account_id);
    assert_eq!(event.purpose, deserialized.purpose);
}

#[test]
fn test_agent_created_serialization() {
    let event = sample_agent_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: AgentCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.agent_id, deserialized.agent_id);
    assert_eq!(event.agent_type, deserialized.agent_type);
}

#[test]
fn test_user_activated_serialization() {
    let event = sample_user_activated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsUserActivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.user_id, deserialized.user_id);
    assert!(deserialized.permissions.is_some());
}

#[test]
fn test_user_deleted_serialization() {
    let event = sample_user_deleted();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsUserDeletedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.user_id, deserialized.user_id);
    assert_eq!(event.reason, deserialized.reason);
}

#[test]
fn test_totp_secret_generated_serialization() {
    let event = sample_totp_secret_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: TotpSecretGeneratedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.user_id, deserialized.user_id);
    assert_eq!(event.secret_id, deserialized.secret_id);
    assert_eq!(event.algorithm, deserialized.algorithm);
}

// =============================================================================
// NatsUserEvents Enum Serialization
// =============================================================================

#[test]
fn test_user_events_enum_serialization() {
    let events = vec![
        NatsUserEvents::NatsUserCreated(sample_user_created()),
        NatsUserEvents::NatsUserUpdated(sample_user_updated()),
        NatsUserEvents::NatsUserPermissionsSet(sample_user_permissions_set()),
        NatsUserEvents::NatsUserSuspended(sample_user_suspended()),
        NatsUserEvents::NatsUserReactivated(sample_user_reactivated()),
        NatsUserEvents::ServiceAccountCreated(sample_service_account_created()),
        NatsUserEvents::AgentCreated(sample_agent_created()),
        NatsUserEvents::NatsUserActivated(sample_user_activated()),
        NatsUserEvents::NatsUserDeleted(sample_user_deleted()),
        NatsUserEvents::TotpSecretGenerated(sample_totp_secret_generated()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: NatsUserEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let created = NatsUserCreatedEvent {
        causation_id: None,
        correlation_id,
        ..sample_user_created()
    };
    let activated = NatsUserActivatedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_user_activated()
    };

    assert_eq!(created.causation_id, None);
    assert_eq!(activated.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let created = NatsUserCreatedEvent { correlation_id, ..sample_user_created() };
    let activated = NatsUserActivatedEvent { correlation_id, ..sample_user_activated() };
    let suspended = NatsUserSuspendedEvent { correlation_id, ..sample_user_suspended() };

    assert_eq!(created.correlation_id, correlation_id);
    assert_eq!(activated.correlation_id, correlation_id);
    assert_eq!(suspended.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let user = sample_user_created();
    assert_ne!(user.user_id, Uuid::nil());
    assert_ne!(user.account_id, Uuid::nil());
    assert_ne!(user.correlation_id, Uuid::nil());

    let service_account = sample_service_account_created();
    assert_ne!(service_account.service_account_id, Uuid::nil());
    assert_ne!(service_account.owning_unit_id, Uuid::nil());
}

#[test]
fn test_public_key_format() {
    let user = sample_user_created();
    // NATS user public keys should be base32-encoded and start with 'U'
    assert!(user.public_key.starts_with('U'));
    assert!(user.public_key.len() >= 56); // Typical NATS key length
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
fn test_totp_algorithm() {
    let totp = sample_totp_secret_generated();
    assert!(!totp.algorithm.is_empty());
    assert_ne!(totp.secret_id, Uuid::nil());
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let user_id = test_user_id();
    let service_account_id = test_user_id();
    let agent_id = test_user_id();

    let events = vec![
        (NatsUserEvents::NatsUserCreated(NatsUserCreatedEvent { user_id, ..sample_user_created() }), user_id),
        (NatsUserEvents::NatsUserUpdated(NatsUserUpdatedEvent { user_id, ..sample_user_updated() }), user_id),
        (NatsUserEvents::NatsUserPermissionsSet(NatsUserPermissionsSetEvent { user_id, ..sample_user_permissions_set() }), user_id),
        (NatsUserEvents::NatsUserSuspended(NatsUserSuspendedEvent { user_id, ..sample_user_suspended() }), user_id),
        (NatsUserEvents::NatsUserReactivated(NatsUserReactivatedEvent { user_id, ..sample_user_reactivated() }), user_id),
        (NatsUserEvents::ServiceAccountCreated(ServiceAccountCreatedEvent { service_account_id, ..sample_service_account_created() }), service_account_id),
        (NatsUserEvents::AgentCreated(AgentCreatedEvent { agent_id, ..sample_agent_created() }), agent_id),
        (NatsUserEvents::NatsUserActivated(NatsUserActivatedEvent { user_id, ..sample_user_activated() }), user_id),
        (NatsUserEvents::NatsUserDeleted(NatsUserDeletedEvent { user_id, ..sample_user_deleted() }), user_id),
        (NatsUserEvents::TotpSecretGenerated(TotpSecretGeneratedEvent { user_id, ..sample_totp_secret_generated() }), user_id),
    ];

    for (event, expected_id) in events {
        assert_eq!(event.aggregate_id(), expected_id);
    }
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(NatsUserEvents::NatsUserCreated(sample_user_created()).event_type(), "NatsUserCreated");
    assert_eq!(NatsUserEvents::NatsUserUpdated(sample_user_updated()).event_type(), "NatsUserUpdated");
    assert_eq!(NatsUserEvents::NatsUserPermissionsSet(sample_user_permissions_set()).event_type(), "NatsUserPermissionsSet");
    assert_eq!(NatsUserEvents::NatsUserSuspended(sample_user_suspended()).event_type(), "NatsUserSuspended");
    assert_eq!(NatsUserEvents::NatsUserReactivated(sample_user_reactivated()).event_type(), "NatsUserReactivated");
    assert_eq!(NatsUserEvents::ServiceAccountCreated(sample_service_account_created()).event_type(), "ServiceAccountCreated");
    assert_eq!(NatsUserEvents::AgentCreated(sample_agent_created()).event_type(), "AgentCreated");
    assert_eq!(NatsUserEvents::NatsUserActivated(sample_user_activated()).event_type(), "NatsUserActivated");
    assert_eq!(NatsUserEvents::NatsUserDeleted(sample_user_deleted()).event_type(), "NatsUserDeleted");
    assert_eq!(NatsUserEvents::TotpSecretGenerated(sample_totp_secret_generated()).event_type(), "TotpSecretGenerated");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_user_lifecycle() {
    let user_id = test_user_id();
    let correlation_id = Uuid::now_v7();

    let created = NatsUserCreatedEvent {
        user_id,
        correlation_id,
        ..sample_user_created()
    };
    let activated = NatsUserActivatedEvent {
        user_id,
        correlation_id,
        ..sample_user_activated()
    };
    let permissions_set = NatsUserPermissionsSetEvent {
        user_id,
        correlation_id,
        ..sample_user_permissions_set()
    };
    let suspended = NatsUserSuspendedEvent {
        user_id,
        correlation_id,
        ..sample_user_suspended()
    };
    let reactivated = NatsUserReactivatedEvent {
        user_id,
        correlation_id,
        ..sample_user_reactivated()
    };
    let deleted = NatsUserDeletedEvent {
        user_id,
        correlation_id,
        ..sample_user_deleted()
    };

    assert_eq!(created.user_id, user_id);
    assert_eq!(activated.user_id, user_id);
    assert_eq!(permissions_set.user_id, user_id);
    assert_eq!(suspended.user_id, user_id);
    assert_eq!(reactivated.user_id, user_id);
    assert_eq!(deleted.user_id, user_id);
}

#[test]
fn test_user_with_person_mapping() {
    let person_id = test_person_id();
    let user = NatsUserCreatedEvent {
        person_id: Some(person_id),
        ..sample_user_created()
    };

    assert_eq!(user.person_id, Some(person_id));
}

#[test]
fn test_service_account_workflow() {
    let owning_unit_id = test_org_unit_id();
    let responsible_person_id = test_person_id();
    let correlation_id = Uuid::now_v7();

    let service_account = ServiceAccountCreatedEvent {
        owning_unit_id,
        responsible_person_id,
        correlation_id: Some(correlation_id),
        ..sample_service_account_created()
    };

    assert_eq!(service_account.owning_unit_id, owning_unit_id);
    assert_eq!(service_account.responsible_person_id, responsible_person_id);
    assert_eq!(service_account.correlation_id, Some(correlation_id));
}

#[test]
fn test_agent_workflow() {
    let organization_id = test_org_id();
    let responsible_person_id = test_person_id();
    let correlation_id = Uuid::now_v7();

    let agent = AgentCreatedEvent {
        organization_id,
        responsible_person_id,
        correlation_id: Some(correlation_id),
        ..sample_agent_created()
    };

    assert_eq!(agent.organization_id, organization_id);
    assert_eq!(agent.responsible_person_id, responsible_person_id);
    assert_eq!(agent.correlation_id, Some(correlation_id));
}

#[test]
fn test_totp_secret_association() {
    let user_id = test_user_id();
    let secret_id = test_secret_id();
    let correlation_id = Uuid::now_v7();

    let totp = TotpSecretGeneratedEvent {
        user_id,
        secret_id,
        correlation_id,
        ..sample_totp_secret_generated()
    };

    assert_eq!(totp.user_id, user_id);
    assert_eq!(totp.secret_id, secret_id);
    assert_eq!(totp.correlation_id, correlation_id);
}

#[test]
fn test_permissions_update_workflow() {
    let user_id = test_user_id();
    let correlation_id = Uuid::now_v7();

    // Set initial permissions
    let initial = NatsUserPermissionsSetEvent {
        user_id,
        correlation_id,
        permissions: NatsPermissions {
            publish: vec!["basic.>".to_string()],
            subscribe: vec!["basic.>".to_string()],
            allow_responses: false,
            max_payload: Some(64 * 1024),
        },
        ..sample_user_permissions_set()
    };

    // Update permissions with expanded access
    let updated = NatsUserPermissionsSetEvent {
        user_id,
        correlation_id,
        permissions: NatsPermissions {
            publish: vec!["basic.>".to_string(), "advanced.>".to_string()],
            subscribe: vec!["basic.>".to_string(), "advanced.>".to_string()],
            allow_responses: true,
            max_payload: Some(1024 * 1024),
        },
        ..sample_user_permissions_set()
    };

    assert_eq!(initial.user_id, user_id);
    assert_eq!(updated.user_id, user_id);
    assert!(!initial.permissions.allow_responses);
    assert!(updated.permissions.allow_responses);
}
