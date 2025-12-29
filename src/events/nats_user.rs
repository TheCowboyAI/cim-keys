//! NATS User Aggregate Events
//!
//! Events related to the NATS User aggregate root.
//! A NATS User represents an authenticated identity within an account.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import shared types from legacy module
use crate::types::NatsPermissions;

/// Events for the NATS User aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum NatsUserEvents {
    /// A new NATS user was created
    NatsUserCreated(NatsUserCreatedEvent),

    /// NATS user was updated
    NatsUserUpdated(NatsUserUpdatedEvent),

    /// NATS user permissions were set
    NatsUserPermissionsSet(NatsUserPermissionsSetEvent),

    /// NATS user was suspended
    NatsUserSuspended(NatsUserSuspendedEvent),

    /// NATS user was reactivated
    NatsUserReactivated(NatsUserReactivatedEvent),

    /// Service account was created
    ServiceAccountCreated(ServiceAccountCreatedEvent),

    /// Agent was created
    AgentCreated(AgentCreatedEvent),

    /// NATS user was activated
    NatsUserActivated(NatsUserActivatedEvent),

    /// NATS user was deleted
    NatsUserDeleted(NatsUserDeletedEvent),

    /// TOTP secret was generated for user
    TotpSecretGenerated(TotpSecretGeneratedEvent),
}

/// A new NATS user was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserCreatedEvent {
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub created_by: String,
    pub person_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS user was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserUpdatedEvent {
    pub user_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS user permissions were set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserPermissionsSetEvent {
    pub user_id: Uuid,
    pub permissions: NatsPermissions,
    pub set_at: DateTime<Utc>,
    pub set_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS user was suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserSuspendedEvent {
    pub user_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS user was reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserReactivatedEvent {
    pub user_id: Uuid,
    pub permissions: Option<NatsPermissions>,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Service account was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountCreatedEvent {
    pub service_account_id: Uuid,
    pub name: String,
    pub purpose: String,
    pub owning_unit_id: Uuid,
    pub responsible_person_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
}

/// Agent was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCreatedEvent {
    pub agent_id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub responsible_person_id: Uuid,
    pub organization_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
}

/// NATS user was activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserActivatedEvent {
    pub user_id: Uuid,
    pub permissions: Option<NatsPermissions>,
    pub activated_at: DateTime<Utc>,
    pub activated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS user was deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserDeletedEvent {
    pub user_id: Uuid,
    pub reason: String,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// TOTP secret was generated for user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecretGeneratedEvent {
    pub user_id: Uuid,
    pub secret_id: Uuid,
    pub algorithm: String,
    pub generated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for NatsUserEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            NatsUserEvents::NatsUserCreated(e) => e.user_id,
            NatsUserEvents::NatsUserUpdated(e) => e.user_id,
            NatsUserEvents::NatsUserPermissionsSet(e) => e.user_id,
            NatsUserEvents::NatsUserSuspended(e) => e.user_id,
            NatsUserEvents::NatsUserReactivated(e) => e.user_id,
            NatsUserEvents::ServiceAccountCreated(e) => e.service_account_id,
            NatsUserEvents::AgentCreated(e) => e.agent_id,
            NatsUserEvents::NatsUserActivated(e) => e.user_id,
            NatsUserEvents::NatsUserDeleted(e) => e.user_id,
            NatsUserEvents::TotpSecretGenerated(e) => e.user_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            NatsUserEvents::NatsUserCreated(_) => "NatsUserCreated",
            NatsUserEvents::NatsUserUpdated(_) => "NatsUserUpdated",
            NatsUserEvents::NatsUserPermissionsSet(_) => "NatsUserPermissionsSet",
            NatsUserEvents::NatsUserSuspended(_) => "NatsUserSuspended",
            NatsUserEvents::NatsUserReactivated(_) => "NatsUserReactivated",
            NatsUserEvents::ServiceAccountCreated(_) => "ServiceAccountCreated",
            NatsUserEvents::AgentCreated(_) => "AgentCreated",
            NatsUserEvents::NatsUserActivated(_) => "NatsUserActivated",
            NatsUserEvents::NatsUserDeleted(_) => "NatsUserDeleted",
            NatsUserEvents::TotpSecretGenerated(_) => "TotpSecretGenerated",
        }
    }
}
