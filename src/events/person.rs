//! Person Aggregate Events
//!
//! Events related to the Person aggregate root in the Organization bounded context.
//! A Person represents an individual with identity, roles, and ownership of cryptographic assets.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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

    /// SSH key was generated for this person
    SshKeyGenerated(SshKeyGeneratedEvent),

    /// GPG key was generated for this person
    GpgKeyGenerated(GpgKeyGeneratedEvent),
}

/// A new person was created in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreatedEvent {
    pub person_id: Uuid,
    pub legal_name: String,
    pub email: Option<String>,
    pub organization_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Person information was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonUpdatedEvent {
    pub person_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
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
            PersonEvents::SshKeyGenerated(_) => "SshKeyGenerated",
            PersonEvents::GpgKeyGenerated(_) => "GpgKeyGenerated",
        }
    }
}
