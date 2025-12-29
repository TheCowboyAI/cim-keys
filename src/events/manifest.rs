//! Manifest Aggregate Events
//!
//! Events related to the Manifest aggregate root.
//! Manifests track exports and provide metadata about cryptographic assets.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Events for the Manifest aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum ManifestEvents {
    /// A manifest was created
    ManifestCreated(ManifestCreatedEvent),

    /// A manifest was updated
    ManifestUpdated(ManifestUpdatedEvent),

    /// JWKS export was created
    JwksExported(JwksExportedEvent),

    /// Projection was applied
    ProjectionApplied(ProjectionAppliedEvent),
}

/// A manifest was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestCreatedEvent {
    pub manifest_id: Uuid,
    pub manifest_path: String,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub keys_count: usize,
    pub certificates_count: usize,
    pub nats_configs_count: usize,
    pub created_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A manifest was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestUpdatedEvent {
    pub manifest_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// JWKS export was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksExportedEvent {
    pub export_id: Uuid,
    pub organization_id: Uuid,
    pub jwks_path: String,
    pub keys_exported: usize,
    pub exported_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Projection was applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionAppliedEvent {
    pub projection_id: Uuid,
    pub projection_type: String,
    pub entity_id: Uuid,
    pub entity_type: String,
    pub applied_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for ManifestEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            ManifestEvents::ManifestCreated(e) => e.manifest_id,
            ManifestEvents::ManifestUpdated(e) => e.manifest_id,
            ManifestEvents::JwksExported(e) => e.export_id,
            ManifestEvents::ProjectionApplied(e) => e.projection_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            ManifestEvents::ManifestCreated(_) => "ManifestCreated",
            ManifestEvents::ManifestUpdated(_) => "ManifestUpdated",
            ManifestEvents::JwksExported(_) => "JwksExported",
            ManifestEvents::ProjectionApplied(_) => "ProjectionApplied",
        }
    }
}
