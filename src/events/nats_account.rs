//! NATS Account Aggregate Events
//!
//! Events related to the NATS Account aggregate root.
//! A NATS Account represents a tenant/namespace within an operator.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import shared types from legacy module
use crate::types::NatsPermissions;

/// Events for the NATS Account aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum NatsAccountEvents {
    /// A new NATS account was created
    NatsAccountCreated(NatsAccountCreatedEvent),

    /// NATS account was updated
    NatsAccountUpdated(NatsAccountUpdatedEvent),

    /// NATS account permissions were set
    NatsPermissionsSet(NatsPermissionsSetEvent),

    /// NATS account was suspended
    NatsAccountSuspended(NatsAccountSuspendedEvent),

    /// NATS account was reactivated
    NatsAccountReactivated(NatsAccountReactivatedEvent),

    /// NATS account was activated
    NatsAccountActivated(NatsAccountActivatedEvent),

    /// NATS account was deleted
    NatsAccountDeleted(NatsAccountDeletedEvent),
}

/// A new NATS account was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountCreatedEvent {
    pub account_id: Uuid,
    pub operator_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub organization_unit_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS account was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountUpdatedEvent {
    pub account_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS permissions were set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsPermissionsSetEvent {
    pub account_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: crate::types::NatsEntityType,
    pub permissions: NatsPermissions,
    pub set_at: DateTime<Utc>,
    pub set_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS account was suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountSuspendedEvent {
    pub account_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS account was reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountReactivatedEvent {
    pub account_id: Uuid,
    pub permissions: Option<NatsPermissions>,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS account was activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountActivatedEvent {
    pub account_id: Uuid,
    pub permissions: Option<NatsPermissions>,
    pub activated_at: DateTime<Utc>,
    pub activated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS account was deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountDeletedEvent {
    pub account_id: Uuid,
    pub reason: String,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for NatsAccountEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            NatsAccountEvents::NatsAccountCreated(e) => e.account_id,
            NatsAccountEvents::NatsAccountUpdated(e) => e.account_id,
            NatsAccountEvents::NatsPermissionsSet(e) => e.account_id,
            NatsAccountEvents::NatsAccountSuspended(e) => e.account_id,
            NatsAccountEvents::NatsAccountReactivated(e) => e.account_id,
            NatsAccountEvents::NatsAccountActivated(e) => e.account_id,
            NatsAccountEvents::NatsAccountDeleted(e) => e.account_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            NatsAccountEvents::NatsAccountCreated(_) => "NatsAccountCreated",
            NatsAccountEvents::NatsAccountUpdated(_) => "NatsAccountUpdated",
            NatsAccountEvents::NatsPermissionsSet(_) => "NatsPermissionsSet",
            NatsAccountEvents::NatsAccountSuspended(_) => "NatsAccountSuspended",
            NatsAccountEvents::NatsAccountReactivated(_) => "NatsAccountReactivated",
            NatsAccountEvents::NatsAccountActivated(_) => "NatsAccountActivated",
            NatsAccountEvents::NatsAccountDeleted(_) => "NatsAccountDeleted",
        }
    }
}
