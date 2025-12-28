//! Comprehensive NATS Operator Events Tests
//!
//! Target: 90%+ coverage of src/events/nats_operator.rs
//!
//! Tests all 12 event types for NATS operator lifecycle, keys, JWTs, and projections.

use chrono::Utc;
use cim_keys::events::nats_operator::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use cim_keys::types::{NatsEntityType, NatsExportFormat};
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_operator_id() -> Uuid { Uuid::now_v7() }
fn test_key_id() -> Uuid { Uuid::now_v7() }
fn test_entity_id() -> Uuid { Uuid::now_v7() }
fn test_export_id() -> Uuid { Uuid::now_v7() }
fn test_nkey_id() -> Uuid { Uuid::now_v7() }
fn test_claims_id() -> Uuid { Uuid::now_v7() }
fn test_jwt_id() -> Uuid { Uuid::now_v7() }
fn test_projection_id() -> Uuid { Uuid::now_v7() }
fn test_org_id() -> Uuid { Uuid::now_v7() }

fn sample_nats_operator_created() -> NatsOperatorCreatedEvent {
    NatsOperatorCreatedEvent {
        operator_id: test_operator_id(),
        name: "CowboyAI Operator".to_string(),
        public_key: "ODXYZ...".to_string(),
        created_at: Utc::now(),
        created_by: "admin".to_string(),
        organization_id: Some(test_org_id()),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_nats_operator_updated() -> NatsOperatorUpdatedEvent {
    NatsOperatorUpdatedEvent {
        operator_id: test_operator_id(),
        field_name: "name".to_string(),
        old_value: Some("Old Operator".to_string()),
        new_value: "Updated Operator".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_nats_signing_key_generated() -> NatsSigningKeyGeneratedEvent {
    NatsSigningKeyGeneratedEvent {
        key_id: test_key_id(),
        entity_id: test_entity_id(),
        entity_type: NatsEntityType::Operator,
        public_key: "SKXYZ...".to_string(),
        generated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_nats_config_exported() -> NatsConfigExportedEvent {
    NatsConfigExportedEvent {
        export_id: test_export_id(),
        operator_id: test_operator_id(),
        format: NatsExportFormat::NscStore,
        exported_at: Utc::now(),
        exported_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_nkey_generated() -> NKeyGeneratedEvent {
    NKeyGeneratedEvent {
        nkey_id: test_nkey_id(),
        key_type: "operator".to_string(),
        public_key: "OKXYZ...".to_string(),
        seed: "SOKXYZ...".to_string(),
        purpose: "Operator signing".to_string(),
        expires_at: None,
        generated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_jwt_claims_created() -> JwtClaimsCreatedEvent {
    JwtClaimsCreatedEvent {
        claims_id: test_claims_id(),
        issuer: "cowboyai-operator".to_string(),
        subject: "account-1".to_string(),
        audience: Some("nats-cluster".to_string()),
        permissions: "{\"publish\": [\">\"], \"subscribe\": [\">\"]}".to_string(),
        not_before: Utc::now(),
        expires_at: Some(Utc::now() + chrono::Duration::days(365)),
        created_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_jwt_signed() -> JwtSignedEvent {
    JwtSignedEvent {
        jwt_id: test_jwt_id(),
        claims_id: test_claims_id(),
        signed_by: test_operator_id(),
        signer_public_key: "OKXYZ...".to_string(),
        jwt_token: "eyJhbGciOiJFZDI1NTE5In0...".to_string(),
        signature_algorithm: "Ed25519".to_string(),
        signature_verification_data: Some("verification-data".to_string()),
        signed_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_nats_operator_suspended() -> NatsOperatorSuspendedEvent {
    NatsOperatorSuspendedEvent {
        operator_id: test_operator_id(),
        reason: "Security review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: "security_admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_nats_operator_reactivated() -> NatsOperatorReactivatedEvent {
    NatsOperatorReactivatedEvent {
        operator_id: test_operator_id(),
        reactivated_at: Utc::now(),
        reactivated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_nats_operator_revoked() -> NatsOperatorRevokedEvent {
    NatsOperatorRevokedEvent {
        operator_id: test_operator_id(),
        reason: "Decommissioned".to_string(),
        revoked_at: Utc::now(),
        revoked_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_jwks_exported() -> JwksExportedEvent {
    JwksExportedEvent {
        export_id: test_export_id(),
        operator_id: test_operator_id(),
        jwks_data: "{\"keys\": [...]}".to_string(),
        exported_at: Utc::now(),
        exported_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_projection_applied() -> ProjectionAppliedEvent {
    ProjectionAppliedEvent {
        projection_id: test_projection_id(),
        projection_type: "OperatorCreated".to_string(),
        entity_id: test_operator_id(),
        applied_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (12 event types)
// =============================================================================

#[test]
fn test_nats_operator_created_serialization() {
    let event = sample_nats_operator_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsOperatorCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.operator_id, deserialized.operator_id);
    assert_eq!(event.name, deserialized.name);
}

#[test]
fn test_nats_operator_updated_serialization() {
    let event = sample_nats_operator_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsOperatorUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.operator_id, deserialized.operator_id);
}

#[test]
fn test_nats_signing_key_generated_serialization() {
    let event = sample_nats_signing_key_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsSigningKeyGeneratedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.entity_id, deserialized.entity_id);
}

#[test]
fn test_nats_config_exported_serialization() {
    let event = sample_nats_config_exported();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsConfigExportedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.export_id, deserialized.export_id);
}

#[test]
fn test_nkey_generated_serialization() {
    let event = sample_nkey_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NKeyGeneratedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.nkey_id, deserialized.nkey_id);
}

#[test]
fn test_jwt_claims_created_serialization() {
    let event = sample_jwt_claims_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: JwtClaimsCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.claims_id, deserialized.claims_id);
}

#[test]
fn test_jwt_signed_serialization() {
    let event = sample_jwt_signed();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: JwtSignedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.jwt_id, deserialized.jwt_id);
    assert_eq!(event.claims_id, deserialized.claims_id);
}

#[test]
fn test_nats_operator_suspended_serialization() {
    let event = sample_nats_operator_suspended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsOperatorSuspendedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.operator_id, deserialized.operator_id);
}

#[test]
fn test_nats_operator_reactivated_serialization() {
    let event = sample_nats_operator_reactivated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsOperatorReactivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.operator_id, deserialized.operator_id);
}

#[test]
fn test_nats_operator_revoked_serialization() {
    let event = sample_nats_operator_revoked();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: NatsOperatorRevokedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.operator_id, deserialized.operator_id);
}

#[test]
fn test_jwks_exported_serialization() {
    let event = sample_jwks_exported();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: JwksExportedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.export_id, deserialized.export_id);
}

#[test]
fn test_projection_applied_serialization() {
    let event = sample_projection_applied();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ProjectionAppliedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.projection_id, deserialized.projection_id);
}

// =============================================================================
// NatsOperatorEvents Enum Serialization
// =============================================================================

#[test]
fn test_nats_operator_events_enum_serialization() {
    let events = vec![
        NatsOperatorEvents::NatsOperatorCreated(sample_nats_operator_created()),
        NatsOperatorEvents::NatsOperatorUpdated(sample_nats_operator_updated()),
        NatsOperatorEvents::NatsSigningKeyGenerated(sample_nats_signing_key_generated()),
        NatsOperatorEvents::NatsConfigExported(sample_nats_config_exported()),
        NatsOperatorEvents::NKeyGenerated(sample_nkey_generated()),
        NatsOperatorEvents::JwtClaimsCreated(sample_jwt_claims_created()),
        NatsOperatorEvents::JwtSigned(sample_jwt_signed()),
        NatsOperatorEvents::NatsOperatorSuspended(sample_nats_operator_suspended()),
        NatsOperatorEvents::NatsOperatorReactivated(sample_nats_operator_reactivated()),
        NatsOperatorEvents::NatsOperatorRevoked(sample_nats_operator_revoked()),
        NatsOperatorEvents::JwksExported(sample_jwks_exported()),
        NatsOperatorEvents::ProjectionApplied(sample_projection_applied()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: NatsOperatorEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let created = NatsOperatorCreatedEvent {
        causation_id: None,
        correlation_id,
        ..sample_nats_operator_created()
    };
    let key_generated = NatsSigningKeyGeneratedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_nats_signing_key_generated()
    };

    assert_eq!(created.causation_id, None);
    assert_eq!(key_generated.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let created = NatsOperatorCreatedEvent { correlation_id, ..sample_nats_operator_created() };
    let key_gen = NatsSigningKeyGeneratedEvent { correlation_id, ..sample_nats_signing_key_generated() };
    let exported = NatsConfigExportedEvent { correlation_id, ..sample_nats_config_exported() };

    assert_eq!(created.correlation_id, correlation_id);
    assert_eq!(key_gen.correlation_id, correlation_id);
    assert_eq!(exported.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let operator = sample_nats_operator_created();
    assert_ne!(operator.operator_id, Uuid::nil());
    assert_ne!(operator.correlation_id, Uuid::nil());

    let jwt = sample_jwt_signed();
    assert_ne!(jwt.jwt_id, Uuid::nil());
    assert_ne!(jwt.claims_id, Uuid::nil());
}

#[test]
fn test_nkey_has_seed() {
    let nkey = sample_nkey_generated();
    assert!(!nkey.seed.is_empty());
    assert!(!nkey.public_key.is_empty());
}

#[test]
fn test_jwt_has_token() {
    let jwt = sample_jwt_signed();
    assert!(!jwt.jwt_token.is_empty());
    assert!(!jwt.signature_algorithm.is_empty());
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let operator_id = test_operator_id();
    let entity_id = test_entity_id();
    let nkey_id = test_nkey_id();
    let claims_id = test_claims_id();
    let jwt_id = test_jwt_id();
    let export_id = test_export_id();
    let projection_id = test_projection_id();

    let events = vec![
        NatsOperatorEvents::NatsOperatorCreated(NatsOperatorCreatedEvent { operator_id, ..sample_nats_operator_created() }),
        NatsOperatorEvents::NatsOperatorUpdated(NatsOperatorUpdatedEvent { operator_id, ..sample_nats_operator_updated() }),
        NatsOperatorEvents::NatsSigningKeyGenerated(NatsSigningKeyGeneratedEvent { entity_id, ..sample_nats_signing_key_generated() }),
        NatsOperatorEvents::NatsConfigExported(NatsConfigExportedEvent { operator_id, ..sample_nats_config_exported() }),
        NatsOperatorEvents::NKeyGenerated(NKeyGeneratedEvent { nkey_id, ..sample_nkey_generated() }),
        NatsOperatorEvents::JwtClaimsCreated(JwtClaimsCreatedEvent { claims_id, ..sample_jwt_claims_created() }),
        NatsOperatorEvents::JwtSigned(JwtSignedEvent { jwt_id, ..sample_jwt_signed() }),
        NatsOperatorEvents::NatsOperatorSuspended(NatsOperatorSuspendedEvent { operator_id, ..sample_nats_operator_suspended() }),
        NatsOperatorEvents::NatsOperatorReactivated(NatsOperatorReactivatedEvent { operator_id, ..sample_nats_operator_reactivated() }),
        NatsOperatorEvents::NatsOperatorRevoked(NatsOperatorRevokedEvent { operator_id, ..sample_nats_operator_revoked() }),
        NatsOperatorEvents::JwksExported(JwksExportedEvent { export_id, ..sample_jwks_exported() }),
        NatsOperatorEvents::ProjectionApplied(ProjectionAppliedEvent { projection_id, ..sample_projection_applied() }),
    ];

    // Verify each event returns the correct aggregate ID
    assert_eq!(events[0].aggregate_id(), operator_id);
    assert_eq!(events[1].aggregate_id(), operator_id);
    assert_eq!(events[2].aggregate_id(), entity_id); // NatsSigningKeyGenerated uses entity_id
    assert_eq!(events[3].aggregate_id(), operator_id);
    assert_eq!(events[4].aggregate_id(), nkey_id); // NKeyGenerated uses nkey_id
    assert_eq!(events[5].aggregate_id(), claims_id); // JwtClaimsCreated uses claims_id
    assert_eq!(events[6].aggregate_id(), jwt_id); // JwtSigned uses jwt_id
    assert_eq!(events[7].aggregate_id(), operator_id);
    assert_eq!(events[8].aggregate_id(), operator_id);
    assert_eq!(events[9].aggregate_id(), operator_id);
    assert_eq!(events[10].aggregate_id(), export_id); // JwksExported uses export_id
    assert_eq!(events[11].aggregate_id(), projection_id); // ProjectionApplied uses projection_id
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(NatsOperatorEvents::NatsOperatorCreated(sample_nats_operator_created()).event_type(), "NatsOperatorCreated");
    assert_eq!(NatsOperatorEvents::NatsOperatorUpdated(sample_nats_operator_updated()).event_type(), "NatsOperatorUpdated");
    assert_eq!(NatsOperatorEvents::NatsSigningKeyGenerated(sample_nats_signing_key_generated()).event_type(), "NatsSigningKeyGenerated");
    assert_eq!(NatsOperatorEvents::NatsConfigExported(sample_nats_config_exported()).event_type(), "NatsConfigExported");
    assert_eq!(NatsOperatorEvents::NKeyGenerated(sample_nkey_generated()).event_type(), "NKeyGenerated");
    assert_eq!(NatsOperatorEvents::JwtClaimsCreated(sample_jwt_claims_created()).event_type(), "JwtClaimsCreated");
    assert_eq!(NatsOperatorEvents::JwtSigned(sample_jwt_signed()).event_type(), "JwtSigned");
    assert_eq!(NatsOperatorEvents::NatsOperatorSuspended(sample_nats_operator_suspended()).event_type(), "NatsOperatorSuspended");
    assert_eq!(NatsOperatorEvents::NatsOperatorReactivated(sample_nats_operator_reactivated()).event_type(), "NatsOperatorReactivated");
    assert_eq!(NatsOperatorEvents::NatsOperatorRevoked(sample_nats_operator_revoked()).event_type(), "NatsOperatorRevoked");
    assert_eq!(NatsOperatorEvents::JwksExported(sample_jwks_exported()).event_type(), "JwksExported");
    assert_eq!(NatsOperatorEvents::ProjectionApplied(sample_projection_applied()).event_type(), "ProjectionApplied");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_operator_lifecycle() {
    let operator_id = test_operator_id();
    let correlation_id = Uuid::now_v7();

    let created = NatsOperatorCreatedEvent {
        operator_id,
        correlation_id,
        ..sample_nats_operator_created()
    };
    let suspended = NatsOperatorSuspendedEvent {
        operator_id,
        correlation_id,
        ..sample_nats_operator_suspended()
    };
    let reactivated = NatsOperatorReactivatedEvent {
        operator_id,
        correlation_id,
        ..sample_nats_operator_reactivated()
    };
    let revoked = NatsOperatorRevokedEvent {
        operator_id,
        correlation_id,
        ..sample_nats_operator_revoked()
    };

    assert_eq!(created.operator_id, operator_id);
    assert_eq!(suspended.operator_id, operator_id);
    assert_eq!(reactivated.operator_id, operator_id);
    assert_eq!(revoked.operator_id, operator_id);
}

#[test]
fn test_jwt_workflow() {
    let correlation_id = Uuid::now_v7();
    let claims_id = test_claims_id();

    let claims_created = JwtClaimsCreatedEvent {
        claims_id,
        correlation_id,
        ..sample_jwt_claims_created()
    };
    let jwt_signed = JwtSignedEvent {
        claims_id,
        correlation_id,
        ..sample_jwt_signed()
    };

    assert_eq!(claims_created.claims_id, claims_id);
    assert_eq!(jwt_signed.claims_id, claims_id);
}

#[test]
fn test_nkey_generation_workflow() {
    let nkey_id = test_nkey_id();
    let correlation_id = Uuid::now_v7();

    let nkey = NKeyGeneratedEvent {
        nkey_id,
        correlation_id,
        ..sample_nkey_generated()
    };

    assert_eq!(nkey.nkey_id, nkey_id);
    assert!(!nkey.seed.is_empty());
    assert!(!nkey.public_key.is_empty());
}

#[test]
fn test_export_workflow() {
    let operator_id = test_operator_id();
    let correlation_id = Uuid::now_v7();

    let config_exported = NatsConfigExportedEvent {
        operator_id,
        correlation_id,
        ..sample_nats_config_exported()
    };
    let jwks_exported = JwksExportedEvent {
        operator_id,
        correlation_id,
        ..sample_jwks_exported()
    };

    assert_eq!(config_exported.operator_id, operator_id);
    assert_eq!(jwks_exported.operator_id, operator_id);
}

#[test]
fn test_projection_workflow() {
    let entity_id = test_operator_id();
    let correlation_id = Uuid::now_v7();

    let projection = ProjectionAppliedEvent {
        entity_id,
        correlation_id,
        ..sample_projection_applied()
    };

    assert_eq!(projection.entity_id, entity_id);
    assert!(!projection.projection_type.is_empty());
}
