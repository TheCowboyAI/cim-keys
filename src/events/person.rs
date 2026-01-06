//! Person Aggregate Events
//!
//! Events related to the Person aggregate root in the Organization bounded context.
//! A Person represents an individual with identity, roles, and ownership of cryptographic assets.
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

// Import value objects
use crate::value_objects::ActorId;

/// Events for the Person aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum PersonEvents {
    /// A new person was added to the organization
    PersonCreated(PersonCreatedEvent),

    /// Person information was updated
    PersonUpdated(PersonUpdatedEvent),

    /// A role was assigned to the person
    RoleAssigned(RoleAssignedEvent),

    /// A role was removed from the person
    RoleRemoved(RoleRemovedEvent),

    /// Person was deactivated
    PersonDeactivated(PersonDeactivatedEvent),

    // Lifecycle State Transitions (Phase 12)
    /// Person activated
    PersonActivated(PersonActivatedEvent),

    /// Person suspended
    PersonSuspended(PersonSuspendedEvent),

    /// Person reactivated
    PersonReactivated(PersonReactivatedEvent),

    /// Person archived (terminal)
    PersonArchived(PersonArchivedEvent),

    /// SSH key was generated for this person
    SshKeyGenerated(SshKeyGeneratedEvent),

    /// GPG key was generated for this person
    GpgKeyGenerated(GpgKeyGeneratedEvent),
}

/// A new person was created in the system
///
/// This event uses dual-path fields for backward compatibility:
/// - Legacy string field (`created_by`) for existing events
/// - Typed value object (`created_by_actor`) for new events
///
/// Use the accessor method `created_by_actor()` which prefers
/// typed field and falls back to parsing legacy string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreatedEvent {
    pub person_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub organization_id: Uuid,

    // ========================================================================
    // Legacy fields (deprecated, kept for backward compatibility)
    // ========================================================================

    /// Legacy: Actor who created the person (use created_by_actor instead)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use created_by_actor field instead")]
    pub created_by: Option<String>,

    // ========================================================================
    // Typed value object fields (preferred)
    // ========================================================================

    /// Typed: Actor who created the person
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_actor: Option<ActorId>,

    // ========================================================================
    // Other fields
    // ========================================================================

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl PersonCreatedEvent {
    /// Create a new event using legacy string field (for backward compatibility)
    #[allow(clippy::too_many_arguments)]
    pub fn new_legacy(
        person_id: Uuid,
        name: String,
        email: Option<String>,
        title: Option<String>,
        department: Option<String>,
        organization_id: Uuid,
        created_by: Option<String>,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
    ) -> Self {
        Self {
            person_id,
            name,
            email,
            title,
            department,
            organization_id,
            created_by,
            created_by_actor: None,
            correlation_id,
            causation_id,
        }
    }

    /// Create a new event using typed value objects (preferred for new code)
    #[allow(clippy::too_many_arguments)]
    pub fn new_typed(
        person_id: Uuid,
        name: String,
        email: Option<String>,
        title: Option<String>,
        department: Option<String>,
        organization_id: Uuid,
        created_by_actor: ActorId,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
    ) -> Self {
        Self {
            person_id,
            name,
            email,
            title,
            department,
            organization_id,
            // Legacy field populated for backward compat serialization
            created_by: Some(created_by_actor.to_legacy_string()),
            created_by_actor: Some(created_by_actor),
            correlation_id,
            causation_id,
        }
    }

    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn created_by_value_object(&self) -> Option<ActorId> {
        if let Some(ref actor) = self.created_by_actor {
            return Some(actor.clone());
        }
        // Fall back to parsing legacy string
        self.created_by.as_ref().map(|s| ActorId::parse(s))
    }

    /// Check if this event uses typed value objects (new format)
    pub fn uses_typed_fields(&self) -> bool {
        self.created_by_actor.is_some()
    }
}

/// Person information was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonUpdatedEvent {
    pub person_id: Uuid,
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
impl PersonUpdatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn updated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.updated_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.updated_by)
    }
}

/// Role assigned to person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignedEvent {
    pub person_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Role removed from person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRemovedEvent {
    pub person_id: Uuid,
    pub role_id: Uuid,
    pub removed_at: DateTime<Utc>,
    pub removed_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Person deactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonDeactivatedEvent {
    pub person_id: Uuid,
    pub reason: String,
    pub deactivated_at: DateTime<Utc>,
    pub deactivated_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Person Lifecycle State Transitions (Phase 12)
// ============================================================================

/// Person activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonActivatedEvent {
    pub person_id: Uuid,
    pub activated_at: DateTime<Utc>,
    pub activated_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Person suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSuspendedEvent {
    pub person_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid,
    pub expected_return: Option<DateTime<Utc>>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Person reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonReactivatedEvent {
    pub person_id: Uuid,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Person archived (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonArchivedEvent {
    pub person_id: Uuid,
    pub reason: String,
    pub archived_at: DateTime<Utc>,
    pub archived_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// SSH key generated for person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyGeneratedEvent {
    pub key_id: Uuid,
    pub person_id: Uuid,
    pub algorithm: String,
    pub public_key_fingerprint: String,
    pub generated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// GPG key generated for person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeyGeneratedEvent {
    pub key_id: Uuid,
    pub person_id: Uuid,
    pub fingerprint: String,
    pub email: String,
    pub generated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for PersonEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            PersonEvents::PersonCreated(e) => e.person_id,
            PersonEvents::PersonUpdated(e) => e.person_id,
            PersonEvents::RoleAssigned(e) => e.person_id,
            PersonEvents::RoleRemoved(e) => e.person_id,
            PersonEvents::PersonDeactivated(e) => e.person_id,
            PersonEvents::PersonActivated(e) => e.person_id,
            PersonEvents::PersonSuspended(e) => e.person_id,
            PersonEvents::PersonReactivated(e) => e.person_id,
            PersonEvents::PersonArchived(e) => e.person_id,
            PersonEvents::SshKeyGenerated(e) => e.person_id,
            PersonEvents::GpgKeyGenerated(e) => e.person_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            PersonEvents::PersonCreated(_) => "PersonCreated",
            PersonEvents::PersonUpdated(_) => "PersonUpdated",
            PersonEvents::RoleAssigned(_) => "RoleAssigned",
            PersonEvents::RoleRemoved(_) => "RoleRemoved",
            PersonEvents::PersonDeactivated(_) => "PersonDeactivated",
            PersonEvents::PersonActivated(_) => "PersonActivated",
            PersonEvents::PersonSuspended(_) => "PersonSuspended",
            PersonEvents::PersonReactivated(_) => "PersonReactivated",
            PersonEvents::PersonArchived(_) => "PersonArchived",
            PersonEvents::SshKeyGenerated(_) => "SshKeyGenerated",
            PersonEvents::GpgKeyGenerated(_) => "GpgKeyGenerated",
        }
    }
}
