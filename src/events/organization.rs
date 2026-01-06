//! Organization Aggregate Events
//!
//! Events related to the Organization aggregate root.
//! Includes organizational structure, units, roles, and policies.
//!
//! ## Value Object Migration
//!
//! Events use dual-path fields for backward compatibility:
//! - Old string fields are kept for deserializing existing events
//! - New typed fields use Option<T> for gradual migration
//! - Accessor methods prefer typed fields, fall back to strings

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::policy_types::{PolicyClaim, PolicyCondition};
use crate::value_objects::ActorId;

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

    // Organization Lifecycle State Transitions (Phase 13)
    /// Organization activated
    OrganizationActivated(OrganizationActivatedEvent),

    /// Organization suspended
    OrganizationSuspended(OrganizationSuspendedEvent),

    /// Organization dissolved (terminal)
    OrganizationDissolved(OrganizationDissolvedEvent),

    // Policy Lifecycle State Transitions (Phase 13)
    /// Policy activated
    PolicyActivated(PolicyActivatedEvent),

    /// Policy amended
    PolicyAmended(PolicyAmendedEvent),

    /// Policy suspended
    PolicySuspended(PolicySuspendedEvent),
}

/// A new organization was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCreatedEvent {
    pub organization_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
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

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use updated_by_actor field instead")]
    pub updated_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl OrganizationUpdatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn updated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.updated_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.updated_by)
    }
}

/// An organizational unit was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalUnitCreatedEvent {
    pub unit_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub organization_id: Uuid,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use created_by_actor field instead")]
    pub created_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl OrganizationalUnitCreatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn created_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.created_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.created_by)
    }
}

/// An organizational unit was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalUnitUpdatedEvent {
    pub unit_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use updated_by_actor field instead")]
    pub updated_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl OrganizationalUnitUpdatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn updated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.updated_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.updated_by)
    }
}

/// An organizational unit was dissolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalUnitDissolvedEvent {
    pub unit_id: Uuid,
    pub reason: String,
    pub dissolved_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use dissolved_by_actor field instead")]
    pub dissolved_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dissolved_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl OrganizationalUnitDissolvedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn dissolved_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.dissolved_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.dissolved_by)
    }
}

/// A role was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCreatedEvent {
    pub role_id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Option<Uuid>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use created_by_actor field instead")]
    pub created_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl RoleCreatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn created_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.created_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.created_by)
    }
}

/// A role was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleUpdatedEvent {
    pub role_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use updated_by_actor field instead")]
    pub updated_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl RoleUpdatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn updated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.updated_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.updated_by)
    }
}

/// A role was deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDeletedEvent {
    pub role_id: Uuid,
    pub reason: String,
    pub deleted_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use deleted_by_actor field instead")]
    pub deleted_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl RoleDeletedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn deleted_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.deleted_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.deleted_by)
    }
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

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use created_by_actor field instead")]
    pub created_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl PolicyCreatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn created_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.created_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.created_by)
    }
}

/// A policy was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdatedEvent {
    pub policy_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use updated_by_actor field instead")]
    pub updated_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl PolicyUpdatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn updated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.updated_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.updated_by)
    }
}

/// A policy was revoked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRevokedEvent {
    pub policy_id: Uuid,
    pub reason: String,
    pub revoked_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use revoked_by_actor field instead")]
    pub revoked_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl PolicyRevokedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn revoked_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.revoked_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.revoked_by)
    }
}

// ============================================================================
// Organization Lifecycle State Transitions (Phase 13)
// ============================================================================

/// Organization activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationActivatedEvent {
    pub organization_id: Uuid,
    pub activated_at: DateTime<Utc>,
    pub activated_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Organization suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSuspendedEvent {
    pub organization_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Organization dissolved (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationDissolvedEvent {
    pub organization_id: Uuid,
    pub reason: String,
    pub dissolved_at: DateTime<Utc>,
    pub dissolved_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Policy Lifecycle State Transitions (Phase 13)
// ============================================================================

/// Policy activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActivatedEvent {
    pub policy_id: Uuid,
    pub activated_at: DateTime<Utc>,
    pub activated_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Policy amended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAmendedEvent {
    pub policy_id: Uuid,
    pub amendment_description: String,
    pub amended_at: DateTime<Utc>,
    pub amended_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Policy suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySuspendedEvent {
    pub policy_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid,
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
            OrganizationEvents::OrganizationActivated(e) => e.organization_id,
            OrganizationEvents::OrganizationSuspended(e) => e.organization_id,
            OrganizationEvents::OrganizationDissolved(e) => e.organization_id,
            OrganizationEvents::PolicyActivated(e) => e.policy_id,
            OrganizationEvents::PolicyAmended(e) => e.policy_id,
            OrganizationEvents::PolicySuspended(e) => e.policy_id,
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
            OrganizationEvents::OrganizationActivated(_) => "OrganizationActivated",
            OrganizationEvents::OrganizationSuspended(_) => "OrganizationSuspended",
            OrganizationEvents::OrganizationDissolved(_) => "OrganizationDissolved",
            OrganizationEvents::PolicyActivated(_) => "PolicyActivated",
            OrganizationEvents::PolicyAmended(_) => "PolicyAmended",
            OrganizationEvents::PolicySuspended(_) => "PolicySuspended",
        }
    }
}
