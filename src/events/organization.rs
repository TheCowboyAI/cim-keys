//! Organization Aggregate Events
//!
//! Events related to the Organization aggregate root.
//! Includes organizational structure, units, roles, and policies.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::policy_types::{PolicyClaim, PolicyCondition};

/// Events for the Organization aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum OrganizationEvents {
    /// A new organization was created
    OrganizationCreated(OrganizationCreatedEvent),

    /// Organization information was updated
    OrganizationUpdated(OrganizationUpdatedEvent),

    /// An organizational unit was created
    OrganizationalUnitCreated(OrganizationalUnitCreatedEvent),

    /// An organizational unit was updated
    OrganizationalUnitUpdated(OrganizationalUnitUpdatedEvent),

    /// An organizational unit was dissolved
    OrganizationalUnitDissolved(OrganizationalUnitDissolvedEvent),

    /// A role was created
    RoleCreated(RoleCreatedEvent),

    /// A role was updated
    RoleUpdated(RoleUpdatedEvent),

    /// A role was deleted
    RoleDeleted(RoleDeletedEvent),

    /// A policy was created
    PolicyCreated(PolicyCreatedEvent),

    /// A policy was updated
    PolicyUpdated(PolicyUpdatedEvent),

    /// A policy was revoked
    PolicyRevoked(PolicyRevokedEvent),
}

/// A new organization was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCreatedEvent {
    pub organization_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Organization information was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUpdatedEvent {
    pub organization_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// An organizational unit was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalUnitCreatedEvent {
    pub unit_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub organization_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// An organizational unit was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalUnitUpdatedEvent {
    pub unit_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// An organizational unit was dissolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalUnitDissolvedEvent {
    pub unit_id: Uuid,
    pub reason: String,
    pub dissolved_at: DateTime<Utc>,
    pub dissolved_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A role was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCreatedEvent {
    pub role_id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A role was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleUpdatedEvent {
    pub role_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A role was deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDeletedEvent {
    pub role_id: Uuid,
    pub reason: String,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A policy was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCreatedEvent {
    pub policy_id: Uuid,
    pub name: String,
    pub description: String,
    pub claims: Vec<PolicyClaim>,
    pub conditions: Vec<PolicyCondition>,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A policy was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdatedEvent {
    pub policy_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A policy was revoked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRevokedEvent {
    pub policy_id: Uuid,
    pub reason: String,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for OrganizationEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            OrganizationEvents::OrganizationCreated(e) => e.organization_id,
            OrganizationEvents::OrganizationUpdated(e) => e.organization_id,
            OrganizationEvents::OrganizationalUnitCreated(e) => e.unit_id,
            OrganizationEvents::OrganizationalUnitUpdated(e) => e.unit_id,
            OrganizationEvents::OrganizationalUnitDissolved(e) => e.unit_id,
            OrganizationEvents::RoleCreated(e) => e.role_id,
            OrganizationEvents::RoleUpdated(e) => e.role_id,
            OrganizationEvents::RoleDeleted(e) => e.role_id,
            OrganizationEvents::PolicyCreated(e) => e.policy_id,
            OrganizationEvents::PolicyUpdated(e) => e.policy_id,
            OrganizationEvents::PolicyRevoked(e) => e.policy_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            OrganizationEvents::OrganizationCreated(_) => "OrganizationCreated",
            OrganizationEvents::OrganizationUpdated(_) => "OrganizationUpdated",
            OrganizationEvents::OrganizationalUnitCreated(_) => "OrganizationalUnitCreated",
            OrganizationEvents::OrganizationalUnitUpdated(_) => "OrganizationalUnitUpdated",
            OrganizationEvents::OrganizationalUnitDissolved(_) => "OrganizationalUnitDissolved",
            OrganizationEvents::RoleCreated(_) => "RoleCreated",
            OrganizationEvents::RoleUpdated(_) => "RoleUpdated",
            OrganizationEvents::RoleDeleted(_) => "RoleDeleted",
            OrganizationEvents::PolicyCreated(_) => "PolicyCreated",
            OrganizationEvents::PolicyUpdated(_) => "PolicyUpdated",
            OrganizationEvents::PolicyRevoked(_) => "PolicyRevoked",
        }
    }
}
