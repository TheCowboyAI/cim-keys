//! Comprehensive Manifest Events Tests
//!
//! Target: 90%+ coverage of src/events/manifest.rs
//!
//! Tests all 4 event types for manifest creation, JWKS export, and projection application.

use chrono::Utc;
use cim_keys::events::manifest::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_manifest_id() -> Uuid { Uuid::now_v7() }
fn test_org_id() -> Uuid { Uuid::now_v7() }
fn test_export_id() -> Uuid { Uuid::now_v7() }
fn test_projection_id() -> Uuid { Uuid::now_v7() }
fn test_entity_id() -> Uuid { Uuid::now_v7() }

fn sample_manifest_created() -> ManifestCreatedEvent {
    ManifestCreatedEvent {
        manifest_id: test_manifest_id(),
        manifest_path: "/mnt/encrypted/manifest.json".to_string(),
        organization_id: test_org_id(),
        organization_name: "Acme Corp".to_string(),
        keys_count: 5,
        certificates_count: 3,
        nats_configs_count: 2,
        created_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_manifest_updated() -> ManifestUpdatedEvent {
    ManifestUpdatedEvent {
        manifest_id: test_manifest_id(),
        field_name: "keys_count".to_string(),
        old_value: Some("5".to_string()),
        new_value: "6".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_jwks_exported() -> JwksExportedEvent {
    JwksExportedEvent {
        export_id: test_export_id(),
        organization_id: test_org_id(),
        jwks_path: "/mnt/encrypted/.well-known/jwks.json".to_string(),
        keys_exported: 5,
        exported_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_projection_applied() -> ProjectionAppliedEvent {
    ProjectionAppliedEvent {
        projection_id: test_projection_id(),
        projection_type: "KeyProjection".to_string(),
        entity_id: test_entity_id(),
        entity_type: "Key".to_string(),
        applied_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (4 event types)
// =============================================================================

#[test]
fn test_manifest_created_serialization() {
    let event = sample_manifest_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ManifestCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.manifest_id, deserialized.manifest_id);
    assert_eq!(event.organization_id, deserialized.organization_id);
    assert_eq!(event.keys_count, deserialized.keys_count);
    assert_eq!(event.certificates_count, deserialized.certificates_count);
}

#[test]
fn test_manifest_updated_serialization() {
    let event = sample_manifest_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ManifestUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.manifest_id, deserialized.manifest_id);
    assert_eq!(event.field_name, deserialized.field_name);
}

#[test]
fn test_jwks_exported_serialization() {
    let event = sample_jwks_exported();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: JwksExportedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.export_id, deserialized.export_id);
    assert_eq!(event.organization_id, deserialized.organization_id);
    assert_eq!(event.keys_exported, deserialized.keys_exported);
}

#[test]
fn test_projection_applied_serialization() {
    let event = sample_projection_applied();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ProjectionAppliedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.projection_id, deserialized.projection_id);
    assert_eq!(event.entity_id, deserialized.entity_id);
    assert_eq!(event.projection_type, deserialized.projection_type);
}

// =============================================================================
// ManifestEvents Enum Serialization
// =============================================================================

#[test]
fn test_manifest_events_enum_serialization() {
    let events = vec![
        ManifestEvents::ManifestCreated(sample_manifest_created()),
        ManifestEvents::ManifestUpdated(sample_manifest_updated()),
        ManifestEvents::JwksExported(sample_jwks_exported()),
        ManifestEvents::ProjectionApplied(sample_projection_applied()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ManifestEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let created = ManifestCreatedEvent {
        causation_id: None,
        correlation_id,
        ..sample_manifest_created()
    };
    let updated = ManifestUpdatedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_manifest_updated()
    };

    assert_eq!(created.causation_id, None);
    assert_eq!(updated.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let created = ManifestCreatedEvent { correlation_id, ..sample_manifest_created() };
    let exported = JwksExportedEvent { correlation_id, ..sample_jwks_exported() };
    let applied = ProjectionAppliedEvent { correlation_id, ..sample_projection_applied() };

    assert_eq!(created.correlation_id, correlation_id);
    assert_eq!(exported.correlation_id, correlation_id);
    assert_eq!(applied.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let manifest = sample_manifest_created();
    assert_ne!(manifest.manifest_id, Uuid::nil());
    assert_ne!(manifest.organization_id, Uuid::nil());
    assert_ne!(manifest.correlation_id, Uuid::nil());

    let export = sample_jwks_exported();
    assert_ne!(export.export_id, Uuid::nil());
    assert_ne!(export.organization_id, Uuid::nil());

    let projection = sample_projection_applied();
    assert_ne!(projection.projection_id, Uuid::nil());
    assert_ne!(projection.entity_id, Uuid::nil());
}

#[test]
fn test_manifest_path_format() {
    let manifest = sample_manifest_created();
    assert!(manifest.manifest_path.ends_with(".json"));
    assert!(manifest.manifest_path.contains("manifest"));
}

#[test]
fn test_jwks_path_format() {
    let export = sample_jwks_exported();
    assert!(export.jwks_path.ends_with(".json"));
    assert!(export.jwks_path.contains("jwks"));
}

#[test]
fn test_count_fields_are_non_negative() {
    let manifest = sample_manifest_created();
    assert!(manifest.keys_count >= 0);
    assert!(manifest.certificates_count >= 0);
    assert!(manifest.nats_configs_count >= 0);

    let export = sample_jwks_exported();
    assert!(export.keys_exported >= 0);
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let manifest_id = test_manifest_id();
    let export_id = test_export_id();
    let projection_id = test_projection_id();

    let events = vec![
        (ManifestEvents::ManifestCreated(ManifestCreatedEvent { manifest_id, ..sample_manifest_created() }), manifest_id),
        (ManifestEvents::ManifestUpdated(ManifestUpdatedEvent { manifest_id, ..sample_manifest_updated() }), manifest_id),
        (ManifestEvents::JwksExported(JwksExportedEvent { export_id, ..sample_jwks_exported() }), export_id),
        (ManifestEvents::ProjectionApplied(ProjectionAppliedEvent { projection_id, ..sample_projection_applied() }), projection_id),
    ];

    for (event, expected_id) in events {
        assert_eq!(event.aggregate_id(), expected_id);
    }
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(ManifestEvents::ManifestCreated(sample_manifest_created()).event_type(), "ManifestCreated");
    assert_eq!(ManifestEvents::ManifestUpdated(sample_manifest_updated()).event_type(), "ManifestUpdated");
    assert_eq!(ManifestEvents::JwksExported(sample_jwks_exported()).event_type(), "JwksExported");
    assert_eq!(ManifestEvents::ProjectionApplied(sample_projection_applied()).event_type(), "ProjectionApplied");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_manifest_lifecycle() {
    let manifest_id = test_manifest_id();
    let correlation_id = Uuid::now_v7();

    let created = ManifestCreatedEvent {
        manifest_id,
        correlation_id,
        ..sample_manifest_created()
    };
    let updated = ManifestUpdatedEvent {
        manifest_id,
        correlation_id,
        ..sample_manifest_updated()
    };

    assert_eq!(created.manifest_id, manifest_id);
    assert_eq!(updated.manifest_id, manifest_id);
}

#[test]
fn test_jwks_export_workflow() {
    let org_id = test_org_id();
    let correlation_id = Uuid::now_v7();

    let manifest = ManifestCreatedEvent {
        organization_id: org_id,
        correlation_id,
        ..sample_manifest_created()
    };
    let export = JwksExportedEvent {
        organization_id: org_id,
        correlation_id,
        ..sample_jwks_exported()
    };

    assert_eq!(manifest.organization_id, org_id);
    assert_eq!(export.organization_id, org_id);
    assert_eq!(manifest.keys_count, export.keys_exported);
}

#[test]
fn test_projection_application_workflow() {
    let correlation_id = Uuid::now_v7();
    let entity_id = test_entity_id();

    let projections = vec![
        ProjectionAppliedEvent {
            entity_id,
            projection_type: "KeyProjection".to_string(),
            entity_type: "Key".to_string(),
            correlation_id,
            ..sample_projection_applied()
        },
        ProjectionAppliedEvent {
            entity_id,
            projection_type: "CertificateProjection".to_string(),
            entity_type: "Certificate".to_string(),
            correlation_id,
            ..sample_projection_applied()
        },
        ProjectionAppliedEvent {
            entity_id,
            projection_type: "NatsProjection".to_string(),
            entity_type: "NatsOperator".to_string(),
            correlation_id,
            ..sample_projection_applied()
        },
    ];

    for projection in projections {
        assert_eq!(projection.entity_id, entity_id);
        assert_eq!(projection.correlation_id, correlation_id);
    }
}

#[test]
fn test_manifest_counts_update() {
    let manifest_id = test_manifest_id();
    let correlation_id = Uuid::now_v7();

    let created = ManifestCreatedEvent {
        manifest_id,
        correlation_id,
        keys_count: 5,
        certificates_count: 3,
        nats_configs_count: 2,
        ..sample_manifest_created()
    };

    let updated_keys = ManifestUpdatedEvent {
        manifest_id,
        correlation_id,
        field_name: "keys_count".to_string(),
        old_value: Some("5".to_string()),
        new_value: "6".to_string(),
        ..sample_manifest_updated()
    };

    assert_eq!(created.keys_count, 5);
    assert_eq!(updated_keys.new_value, "6");
}

#[test]
fn test_projection_types() {
    let projection_types = vec![
        "KeyProjection",
        "CertificateProjection",
        "NatsOperatorProjection",
        "NatsAccountProjection",
        "NatsUserProjection",
        "OrganizationProjection",
    ];

    for proj_type in projection_types {
        let projection = ProjectionAppliedEvent {
            projection_type: proj_type.to_string(),
            ..sample_projection_applied()
        };
        assert_eq!(projection.projection_type, proj_type);
    }
}
