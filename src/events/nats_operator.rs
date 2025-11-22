//! NATS Operator Aggregate Events
//!
//! Events related to the NATS Operator aggregate root.
//! A NATS Operator is the top-level authority in NATS security.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import shared types from legacy module
use crate::events_legacy::{NatsEntityType, NatsExportFormat};

/// Events for the NATS Operator aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum NatsOperatorEvents {
    /// A new NATS operator was created
    NatsOperatorCreated(NatsOperatorCreatedEvent),

    /// NATS operator was updated
    NatsOperatorUpdated(NatsOperatorUpdatedEvent),

    /// NATS signing key was generated
    NatsSigningKeyGenerated(NatsSigningKeyGeneratedEvent),

    /// NATS configuration was exported
    NatsConfigExported(NatsConfigExportedEvent),

    /// NKey was generated for operator
    NKeyGenerated(NKeyGeneratedEvent),

    /// JWT claims were created
    JwtClaimsCreated(JwtClaimsCreatedEvent),

    /// JWT was signed
    JwtSigned(JwtSignedEvent),
}

/// A new NATS operator was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorCreatedEvent {
    pub operator_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub organization_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS operator was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorUpdatedEvent {
    pub operator_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS signing key was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSigningKeyGeneratedEvent {
    pub key_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: NatsEntityType,
    pub public_key: String,
    pub generated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS configuration was exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfigExportedEvent {
    pub export_id: Uuid,
    pub operator_id: Uuid,
    pub format: NatsExportFormat,
    pub exported_at: DateTime<Utc>,
    pub exported_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NKey was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NKeyGeneratedEvent {
    pub nkey_id: Uuid,
    pub key_type: String,
    pub public_key: String,
    pub seed: String,
    pub generated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// JWT claims were created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaimsCreatedEvent {
    pub claims_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: NatsEntityType,
    pub claims: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// JWT was signed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSignedEvent {
    pub jwt_id: Uuid,
    pub claims_id: Uuid,
    pub signed_by: Uuid,
    pub jwt_token: String,
    pub signed_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for NatsOperatorEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            NatsOperatorEvents::NatsOperatorCreated(e) => e.operator_id,
            NatsOperatorEvents::NatsOperatorUpdated(e) => e.operator_id,
            NatsOperatorEvents::NatsSigningKeyGenerated(e) => e.entity_id,
            NatsOperatorEvents::NatsConfigExported(e) => e.operator_id,
            NatsOperatorEvents::NKeyGenerated(e) => e.nkey_id,
            NatsOperatorEvents::JwtClaimsCreated(e) => e.entity_id,
            NatsOperatorEvents::JwtSigned(e) => e.jwt_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            NatsOperatorEvents::NatsOperatorCreated(_) => "NatsOperatorCreated",
            NatsOperatorEvents::NatsOperatorUpdated(_) => "NatsOperatorUpdated",
            NatsOperatorEvents::NatsSigningKeyGenerated(_) => "NatsSigningKeyGenerated",
            NatsOperatorEvents::NatsConfigExported(_) => "NatsConfigExported",
            NatsOperatorEvents::NKeyGenerated(_) => "NKeyGenerated",
            NatsOperatorEvents::JwtClaimsCreated(_) => "JwtClaimsCreated",
            NatsOperatorEvents::JwtSigned(_) => "JwtSigned",
        }
    }
}
