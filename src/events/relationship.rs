//! Relationship Aggregate Events
//!
//! Events related to the Relationship aggregate root.
//! Relationships represent connections between entities in the domain model.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import shared types
use crate::types::TrustLevel;
use crate::commands::organization::RelationshipType;

/// Events for the Relationship aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum RelationshipEvents {
    /// A relationship was established between entities
    RelationshipEstablished(RelationshipEstablishedEvent),

    /// A trust relationship was established
    TrustEstablished(TrustEstablishedEvent),

    /// A relationship was modified
    RelationshipModified(RelationshipModifiedEvent),

    /// A relationship was terminated
    RelationshipTerminated(RelationshipTerminatedEvent),

    /// Accountability was validated
    AccountabilityValidated(AccountabilityValidatedEvent),

    /// Accountability violation was detected
    AccountabilityViolated(AccountabilityViolatedEvent),
}

/// A relationship was established between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEstablishedEvent {
    pub relationship_id: Uuid,
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub relationship_type: RelationshipType,
    pub established_at: DateTime<Utc>,
    pub established_by: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A trust relationship was established
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEstablishedEvent {
    pub relationship_id: Uuid,
    pub trustor_id: Uuid,
    pub trustee_id: Uuid,
    pub trust_level: TrustLevel,
    pub established_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A relationship was modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipModifiedEvent {
    pub relationship_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub modified_at: DateTime<Utc>,
    pub modified_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A relationship was terminated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipTerminatedEvent {
    pub relationship_id: Uuid,
    pub reason: String,
    pub terminated_at: DateTime<Utc>,
    pub terminated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Accountability was validated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountabilityValidatedEvent {
    pub validation_id: Uuid,
    pub entity_id: Uuid,
    pub validator_id: Uuid,
    pub validation_type: String,
    pub result: bool,
    pub validated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Accountability violation was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountabilityViolatedEvent {
    pub violation_id: Uuid,
    pub entity_id: Uuid,
    pub violation_type: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub severity: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for RelationshipEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            RelationshipEvents::RelationshipEstablished(e) => e.relationship_id,
            RelationshipEvents::TrustEstablished(e) => e.relationship_id,
            RelationshipEvents::RelationshipModified(e) => e.relationship_id,
            RelationshipEvents::RelationshipTerminated(e) => e.relationship_id,
            RelationshipEvents::AccountabilityValidated(e) => e.validation_id,
            RelationshipEvents::AccountabilityViolated(e) => e.violation_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            RelationshipEvents::RelationshipEstablished(_) => "RelationshipEstablished",
            RelationshipEvents::TrustEstablished(_) => "TrustEstablished",
            RelationshipEvents::RelationshipModified(_) => "RelationshipModified",
            RelationshipEvents::RelationshipTerminated(_) => "RelationshipTerminated",
            RelationshipEvents::AccountabilityValidated(_) => "AccountabilityValidated",
            RelationshipEvents::AccountabilityViolated(_) => "AccountabilityViolated",
        }
    }
}
