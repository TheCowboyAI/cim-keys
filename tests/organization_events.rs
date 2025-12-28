//! Comprehensive Organization Events Tests
//!
//! Target: 90%+ coverage of src/events/organization.rs
//!
//! Tests all 17 event types across organization lifecycle, units, roles, and policies.

use chrono::Utc;
use cim_keys::events::organization::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use cim_keys::policy_types::{PolicyClaim, PolicyCondition, SecurityClearance};
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_org_id() -> Uuid { Uuid::now_v7() }
fn test_unit_id() -> Uuid { Uuid::now_v7() }
fn test_role_id() -> Uuid { Uuid::now_v7() }
fn test_policy_id() -> Uuid { Uuid::now_v7() }
fn test_person_id() -> Uuid { Uuid::now_v7() }

fn sample_policy_claims() -> Vec<PolicyClaim> {
    vec![PolicyClaim::CanGenerateKeys, PolicyClaim::CanSignCode]
}

fn sample_policy_conditions() -> Vec<PolicyCondition> {
    vec![
        PolicyCondition::MinimumSecurityClearance(SecurityClearance::Secret),
        PolicyCondition::MFAEnabled(true),
    ]
}

fn sample_organization_created() -> OrganizationCreatedEvent {
    OrganizationCreatedEvent {
        organization_id: test_org_id(),
        name: "Acme Corp".to_string(),
        domain: Some("acme.com".to_string()),
        created_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_organization_updated() -> OrganizationUpdatedEvent {
    OrganizationUpdatedEvent {
        organization_id: test_org_id(),
        field_name: "name".to_string(),
        old_value: Some("Acme Corp".to_string()),
        new_value: "Acme Corporation".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_organizational_unit_created() -> OrganizationalUnitCreatedEvent {
    OrganizationalUnitCreatedEvent {
        unit_id: test_unit_id(),
        name: "Engineering".to_string(),
        parent_id: None,
        organization_id: test_org_id(),
        created_at: Utc::now(),
        created_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_organizational_unit_updated() -> OrganizationalUnitUpdatedEvent {
    OrganizationalUnitUpdatedEvent {
        unit_id: test_unit_id(),
        field_name: "name".to_string(),
        old_value: Some("Engineering".to_string()),
        new_value: "Engineering Department".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_organizational_unit_dissolved() -> OrganizationalUnitDissolvedEvent {
    OrganizationalUnitDissolvedEvent {
        unit_id: test_unit_id(),
        reason: "Restructuring".to_string(),
        dissolved_at: Utc::now(),
        dissolved_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_role_created() -> RoleCreatedEvent {
    RoleCreatedEvent {
        role_id: test_role_id(),
        name: "Developer".to_string(),
        description: "Software developer role".to_string(),
        organization_id: Some(test_org_id()),
        created_at: Utc::now(),
        created_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_role_updated() -> RoleUpdatedEvent {
    RoleUpdatedEvent {
        role_id: test_role_id(),
        field_name: "description".to_string(),
        old_value: Some("Software developer role".to_string()),
        new_value: "Senior software developer role".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_role_deleted() -> RoleDeletedEvent {
    RoleDeletedEvent {
        role_id: test_role_id(),
        reason: "Role obsolete".to_string(),
        deleted_at: Utc::now(),
        deleted_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_policy_created() -> PolicyCreatedEvent {
    PolicyCreatedEvent {
        policy_id: test_policy_id(),
        name: "Code Signing Policy".to_string(),
        description: "Policy for code signing permissions".to_string(),
        claims: sample_policy_claims(),
        conditions: sample_policy_conditions(),
        organization_id: Some(test_org_id()),
        created_at: Utc::now(),
        created_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_policy_updated() -> PolicyUpdatedEvent {
    PolicyUpdatedEvent {
        policy_id: test_policy_id(),
        field_name: "name".to_string(),
        old_value: Some("Code Signing Policy".to_string()),
        new_value: "Production Code Signing Policy".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_policy_revoked() -> PolicyRevokedEvent {
    PolicyRevokedEvent {
        policy_id: test_policy_id(),
        reason: "Policy outdated".to_string(),
        revoked_at: Utc::now(),
        revoked_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_organization_activated() -> OrganizationActivatedEvent {
    OrganizationActivatedEvent {
        organization_id: test_org_id(),
        activated_at: Utc::now(),
        activated_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_organization_suspended() -> OrganizationSuspendedEvent {
    OrganizationSuspendedEvent {
        organization_id: test_org_id(),
        reason: "Compliance review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_organization_dissolved() -> OrganizationDissolvedEvent {
    OrganizationDissolvedEvent {
        organization_id: test_org_id(),
        reason: "Company closure".to_string(),
        dissolved_at: Utc::now(),
        dissolved_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_policy_activated() -> PolicyActivatedEvent {
    PolicyActivatedEvent {
        policy_id: test_policy_id(),
        activated_at: Utc::now(),
        activated_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_policy_amended() -> PolicyAmendedEvent {
    PolicyAmendedEvent {
        policy_id: test_policy_id(),
        amendment_description: "Added new security requirements".to_string(),
        amended_at: Utc::now(),
        amended_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_policy_suspended() -> PolicySuspendedEvent {
    PolicySuspendedEvent {
        policy_id: test_policy_id(),
        reason: "Under review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (17 event types)
// =============================================================================

#[test]
fn test_organization_created_serialization() {
    let event = sample_organization_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: OrganizationCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.organization_id, deserialized.organization_id);
    assert_eq!(event.name, deserialized.name);
}

#[test]
fn test_organization_updated_serialization() {
    let event = sample_organization_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: OrganizationUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.organization_id, deserialized.organization_id);
}

#[test]
fn test_organizational_unit_created_serialization() {
    let event = sample_organizational_unit_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: OrganizationalUnitCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.unit_id, deserialized.unit_id);
}

#[test]
fn test_organizational_unit_updated_serialization() {
    let event = sample_organizational_unit_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: OrganizationalUnitUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.unit_id, deserialized.unit_id);
}

#[test]
fn test_organizational_unit_dissolved_serialization() {
    let event = sample_organizational_unit_dissolved();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: OrganizationalUnitDissolvedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.unit_id, deserialized.unit_id);
}

#[test]
fn test_role_created_serialization() {
    let event = sample_role_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RoleCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.role_id, deserialized.role_id);
}

#[test]
fn test_role_updated_serialization() {
    let event = sample_role_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RoleUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.role_id, deserialized.role_id);
}

#[test]
fn test_role_deleted_serialization() {
    let event = sample_role_deleted();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RoleDeletedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.role_id, deserialized.role_id);
}

#[test]
fn test_policy_created_serialization() {
    let event = sample_policy_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PolicyCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.policy_id, deserialized.policy_id);
    assert_eq!(event.claims.len(), deserialized.claims.len());
    assert_eq!(event.conditions.len(), deserialized.conditions.len());
}

#[test]
fn test_policy_updated_serialization() {
    let event = sample_policy_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PolicyUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.policy_id, deserialized.policy_id);
}

#[test]
fn test_policy_revoked_serialization() {
    let event = sample_policy_revoked();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PolicyRevokedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.policy_id, deserialized.policy_id);
}

#[test]
fn test_organization_activated_serialization() {
    let event = sample_organization_activated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: OrganizationActivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.organization_id, deserialized.organization_id);
}

#[test]
fn test_organization_suspended_serialization() {
    let event = sample_organization_suspended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: OrganizationSuspendedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.organization_id, deserialized.organization_id);
}

#[test]
fn test_organization_dissolved_serialization() {
    let event = sample_organization_dissolved();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: OrganizationDissolvedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.organization_id, deserialized.organization_id);
}

#[test]
fn test_policy_activated_serialization() {
    let event = sample_policy_activated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PolicyActivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.policy_id, deserialized.policy_id);
}

#[test]
fn test_policy_amended_serialization() {
    let event = sample_policy_amended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PolicyAmendedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.policy_id, deserialized.policy_id);
}

#[test]
fn test_policy_suspended_serialization() {
    let event = sample_policy_suspended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: PolicySuspendedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.policy_id, deserialized.policy_id);
}

// =============================================================================
// OrganizationEvents Enum Serialization
// =============================================================================

#[test]
fn test_organization_events_enum_serialization() {
    let events = vec![
        OrganizationEvents::OrganizationCreated(sample_organization_created()),
        OrganizationEvents::OrganizationUpdated(sample_organization_updated()),
        OrganizationEvents::OrganizationalUnitCreated(sample_organizational_unit_created()),
        OrganizationEvents::OrganizationalUnitUpdated(sample_organizational_unit_updated()),
        OrganizationEvents::OrganizationalUnitDissolved(sample_organizational_unit_dissolved()),
        OrganizationEvents::RoleCreated(sample_role_created()),
        OrganizationEvents::RoleUpdated(sample_role_updated()),
        OrganizationEvents::RoleDeleted(sample_role_deleted()),
        OrganizationEvents::PolicyCreated(sample_policy_created()),
        OrganizationEvents::PolicyUpdated(sample_policy_updated()),
        OrganizationEvents::PolicyRevoked(sample_policy_revoked()),
        OrganizationEvents::OrganizationActivated(sample_organization_activated()),
        OrganizationEvents::OrganizationSuspended(sample_organization_suspended()),
        OrganizationEvents::OrganizationDissolved(sample_organization_dissolved()),
        OrganizationEvents::PolicyActivated(sample_policy_activated()),
        OrganizationEvents::PolicyAmended(sample_policy_amended()),
        OrganizationEvents::PolicySuspended(sample_policy_suspended()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OrganizationEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let created = OrganizationCreatedEvent {
        causation_id: None, // Root event
        correlation_id,
        ..sample_organization_created()
    };
    let activated = OrganizationActivatedEvent {
        causation_id: Some(correlation_id), // Caused by created
        correlation_id,
        ..sample_organization_activated()
    };

    assert_eq!(created.causation_id, None);
    assert_eq!(activated.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let created = OrganizationCreatedEvent { correlation_id, ..sample_organization_created() };
    let activated = OrganizationActivatedEvent { correlation_id, ..sample_organization_activated() };
    let suspended = OrganizationSuspendedEvent { correlation_id, ..sample_organization_suspended() };

    assert_eq!(created.correlation_id, correlation_id);
    assert_eq!(activated.correlation_id, correlation_id);
    assert_eq!(suspended.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let org_event = sample_organization_created();
    assert_ne!(org_event.organization_id, Uuid::nil());
    assert_ne!(org_event.correlation_id, Uuid::nil());

    let policy_event = sample_policy_created();
    assert_ne!(policy_event.policy_id, Uuid::nil());
    assert_ne!(policy_event.correlation_id, Uuid::nil());
}

#[test]
fn test_policy_claims_and_conditions() {
    let policy = sample_policy_created();
    assert!(!policy.claims.is_empty());
    assert!(!policy.conditions.is_empty());
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let org_id = test_org_id();
    let unit_id = test_unit_id();
    let role_id = test_role_id();
    let policy_id = test_policy_id();

    // Organization events
    let org_events = vec![
        OrganizationEvents::OrganizationCreated(OrganizationCreatedEvent { organization_id: org_id, ..sample_organization_created() }),
        OrganizationEvents::OrganizationUpdated(OrganizationUpdatedEvent { organization_id: org_id, ..sample_organization_updated() }),
        OrganizationEvents::OrganizationActivated(OrganizationActivatedEvent { organization_id: org_id, ..sample_organization_activated() }),
        OrganizationEvents::OrganizationSuspended(OrganizationSuspendedEvent { organization_id: org_id, ..sample_organization_suspended() }),
        OrganizationEvents::OrganizationDissolved(OrganizationDissolvedEvent { organization_id: org_id, ..sample_organization_dissolved() }),
    ];
    for event in org_events {
        assert_eq!(event.aggregate_id(), org_id);
    }

    // Unit events
    let unit_events = vec![
        OrganizationEvents::OrganizationalUnitCreated(OrganizationalUnitCreatedEvent { unit_id, ..sample_organizational_unit_created() }),
        OrganizationEvents::OrganizationalUnitUpdated(OrganizationalUnitUpdatedEvent { unit_id, ..sample_organizational_unit_updated() }),
        OrganizationEvents::OrganizationalUnitDissolved(OrganizationalUnitDissolvedEvent { unit_id, ..sample_organizational_unit_dissolved() }),
    ];
    for event in unit_events {
        assert_eq!(event.aggregate_id(), unit_id);
    }

    // Role events
    let role_events = vec![
        OrganizationEvents::RoleCreated(RoleCreatedEvent { role_id, ..sample_role_created() }),
        OrganizationEvents::RoleUpdated(RoleUpdatedEvent { role_id, ..sample_role_updated() }),
        OrganizationEvents::RoleDeleted(RoleDeletedEvent { role_id, ..sample_role_deleted() }),
    ];
    for event in role_events {
        assert_eq!(event.aggregate_id(), role_id);
    }

    // Policy events
    let policy_events = vec![
        OrganizationEvents::PolicyCreated(PolicyCreatedEvent { policy_id, ..sample_policy_created() }),
        OrganizationEvents::PolicyUpdated(PolicyUpdatedEvent { policy_id, ..sample_policy_updated() }),
        OrganizationEvents::PolicyRevoked(PolicyRevokedEvent { policy_id, ..sample_policy_revoked() }),
        OrganizationEvents::PolicyActivated(PolicyActivatedEvent { policy_id, ..sample_policy_activated() }),
        OrganizationEvents::PolicyAmended(PolicyAmendedEvent { policy_id, ..sample_policy_amended() }),
        OrganizationEvents::PolicySuspended(PolicySuspendedEvent { policy_id, ..sample_policy_suspended() }),
    ];
    for event in policy_events {
        assert_eq!(event.aggregate_id(), policy_id);
    }
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(OrganizationEvents::OrganizationCreated(sample_organization_created()).event_type(), "OrganizationCreated");
    assert_eq!(OrganizationEvents::OrganizationUpdated(sample_organization_updated()).event_type(), "OrganizationUpdated");
    assert_eq!(OrganizationEvents::OrganizationalUnitCreated(sample_organizational_unit_created()).event_type(), "OrganizationalUnitCreated");
    assert_eq!(OrganizationEvents::OrganizationalUnitUpdated(sample_organizational_unit_updated()).event_type(), "OrganizationalUnitUpdated");
    assert_eq!(OrganizationEvents::OrganizationalUnitDissolved(sample_organizational_unit_dissolved()).event_type(), "OrganizationalUnitDissolved");
    assert_eq!(OrganizationEvents::RoleCreated(sample_role_created()).event_type(), "RoleCreated");
    assert_eq!(OrganizationEvents::RoleUpdated(sample_role_updated()).event_type(), "RoleUpdated");
    assert_eq!(OrganizationEvents::RoleDeleted(sample_role_deleted()).event_type(), "RoleDeleted");
    assert_eq!(OrganizationEvents::PolicyCreated(sample_policy_created()).event_type(), "PolicyCreated");
    assert_eq!(OrganizationEvents::PolicyUpdated(sample_policy_updated()).event_type(), "PolicyUpdated");
    assert_eq!(OrganizationEvents::PolicyRevoked(sample_policy_revoked()).event_type(), "PolicyRevoked");
    assert_eq!(OrganizationEvents::OrganizationActivated(sample_organization_activated()).event_type(), "OrganizationActivated");
    assert_eq!(OrganizationEvents::OrganizationSuspended(sample_organization_suspended()).event_type(), "OrganizationSuspended");
    assert_eq!(OrganizationEvents::OrganizationDissolved(sample_organization_dissolved()).event_type(), "OrganizationDissolved");
    assert_eq!(OrganizationEvents::PolicyActivated(sample_policy_activated()).event_type(), "PolicyActivated");
    assert_eq!(OrganizationEvents::PolicyAmended(sample_policy_amended()).event_type(), "PolicyAmended");
    assert_eq!(OrganizationEvents::PolicySuspended(sample_policy_suspended()).event_type(), "PolicySuspended");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_organization_lifecycle() {
    let org_id = test_org_id();
    let correlation_id = Uuid::now_v7();

    let created = OrganizationCreatedEvent {
        organization_id: org_id,
        correlation_id,
        ..sample_organization_created()
    };
    let activated = OrganizationActivatedEvent {
        organization_id: org_id,
        correlation_id,
        ..sample_organization_activated()
    };
    let suspended = OrganizationSuspendedEvent {
        organization_id: org_id,
        correlation_id,
        ..sample_organization_suspended()
    };
    let dissolved = OrganizationDissolvedEvent {
        organization_id: org_id,
        correlation_id,
        ..sample_organization_dissolved()
    };

    // All events should have same org_id and correlation_id
    assert_eq!(created.organization_id, org_id);
    assert_eq!(activated.organization_id, org_id);
    assert_eq!(suspended.organization_id, org_id);
    assert_eq!(dissolved.organization_id, org_id);

    assert_eq!(created.correlation_id, correlation_id);
    assert_eq!(activated.correlation_id, correlation_id);
    assert_eq!(suspended.correlation_id, correlation_id);
    assert_eq!(dissolved.correlation_id, correlation_id);
}

#[test]
fn test_complete_policy_lifecycle() {
    let policy_id = test_policy_id();
    let correlation_id = Uuid::now_v7();

    let created = PolicyCreatedEvent {
        policy_id,
        correlation_id,
        ..sample_policy_created()
    };
    let activated = PolicyActivatedEvent {
        policy_id,
        correlation_id,
        ..sample_policy_activated()
    };
    let amended = PolicyAmendedEvent {
        policy_id,
        correlation_id,
        ..sample_policy_amended()
    };
    let suspended = PolicySuspendedEvent {
        policy_id,
        correlation_id,
        ..sample_policy_suspended()
    };

    assert_eq!(created.policy_id, policy_id);
    assert_eq!(activated.policy_id, policy_id);
    assert_eq!(amended.policy_id, policy_id);
    assert_eq!(suspended.policy_id, policy_id);
}
