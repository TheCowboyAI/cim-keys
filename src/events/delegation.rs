// Copyright (c) 2025 - Cowboy AI, LLC.

//! Delegation Aggregate Events
//!
//! Events related to the Delegation aggregate root.
//! Delegations represent authority transfer from one person to another
//! with specific permissions and temporal bounds.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::KeyPermission;

/// Events for the Delegation aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum DelegationEvents {
    /// A new delegation was created
    DelegationCreated(DelegationCreatedEvent),

    /// A delegation was revoked
    DelegationRevoked(DelegationRevokedEvent),

    /// A delegation was cascade-revoked due to parent revocation
    DelegationCascadeRevoked(DelegationCascadeRevokedEvent),

    /// A delegation was extended (validity period changed)
    DelegationExtended(DelegationExtendedEvent),

    /// A delegation's permissions were modified
    DelegationPermissionsModified(DelegationPermissionsModifiedEvent),
}

/// A new delegation was created
///
/// This event represents the establishment of authority from one person
/// (the delegator) to another person (the delegate) with specific permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationCreatedEvent {
    /// Unique identifier for this delegation
    pub delegation_id: Uuid,

    /// Person granting the delegation
    pub delegator_id: Uuid,

    /// Person receiving the delegation
    pub delegate_id: Uuid,

    /// Permissions being delegated
    pub permissions: Vec<KeyPermission>,

    /// Optional parent delegation this derives from (for transitive delegations)
    pub derives_from: Option<Uuid>,

    /// When the delegation becomes effective
    pub valid_from: DateTime<Utc>,

    /// When the delegation expires (None = no expiration)
    pub valid_until: Option<DateTime<Utc>>,

    /// When this event occurred
    pub created_at: DateTime<Utc>,

    /// Who created this delegation (usually the delegator, but could be admin)
    pub created_by: Uuid,

    /// Correlation ID for event chain tracking
    pub correlation_id: Uuid,

    /// Causation ID linking to what triggered this event
    pub causation_id: Option<Uuid>,
}

/// A delegation was directly revoked
///
/// This event represents the intentional revocation of a delegation
/// by someone with authority to do so (typically the delegator or admin).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRevokedEvent {
    /// The delegation being revoked
    pub delegation_id: Uuid,

    /// Who revoked the delegation
    pub revoked_by: Uuid,

    /// Reason for revocation
    pub reason: RevocationReason,

    /// When the revocation occurred
    pub revoked_at: DateTime<Utc>,

    /// Correlation ID for event chain tracking
    pub correlation_id: Uuid,

    /// Causation ID linking to what triggered this event
    pub causation_id: Option<Uuid>,
}

/// A delegation was cascade-revoked due to parent delegation revocation
///
/// This event represents automatic revocation of a delegation because
/// its parent (derives_from) delegation was revoked. This ensures that
/// revoked authority doesn't persist through transitive delegations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationCascadeRevokedEvent {
    /// The delegation being cascade-revoked
    pub delegation_id: Uuid,

    /// The parent delegation that caused this cascade
    pub caused_by_delegation_id: Uuid,

    /// The original revocation event that started the cascade
    pub root_revocation_id: Uuid,

    /// Human-readable reason for cascade
    pub reason: String,

    /// When the cascade revocation occurred
    pub revoked_at: DateTime<Utc>,

    /// Correlation ID for event chain tracking
    pub correlation_id: Uuid,

    /// Causation ID linking to what triggered this event
    pub causation_id: Option<Uuid>,
}

/// A delegation was extended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationExtendedEvent {
    /// The delegation being extended
    pub delegation_id: Uuid,

    /// Previous expiration date
    pub previous_valid_until: Option<DateTime<Utc>>,

    /// New expiration date
    pub new_valid_until: Option<DateTime<Utc>>,

    /// Who extended the delegation
    pub extended_by: Uuid,

    /// When the extension occurred
    pub extended_at: DateTime<Utc>,

    /// Correlation ID for event chain tracking
    pub correlation_id: Uuid,

    /// Causation ID linking to what triggered this event
    pub causation_id: Option<Uuid>,
}

/// Delegation permissions were modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationPermissionsModifiedEvent {
    /// The delegation being modified
    pub delegation_id: Uuid,

    /// Permissions that were removed
    pub permissions_removed: Vec<KeyPermission>,

    /// Permissions that were added
    pub permissions_added: Vec<KeyPermission>,

    /// Who modified the permissions
    pub modified_by: Uuid,

    /// When the modification occurred
    pub modified_at: DateTime<Utc>,

    /// Correlation ID for event chain tracking
    pub correlation_id: Uuid,

    /// Causation ID linking to what triggered this event
    pub causation_id: Option<Uuid>,
}

/// Reasons for delegation revocation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevocationReason {
    /// Delegator explicitly revoked
    DelegatorRevoked,

    /// Delegate requested revocation
    DelegateRequested,

    /// Administrator revoked for policy reasons
    AdminRevoked,

    /// Delegation expired (automatic)
    Expired,

    /// Cascade revocation from parent
    CascadeFromParent,

    /// Security incident
    SecurityIncident,

    /// Role change (delegate no longer needs access)
    RoleChange,

    /// Organizational restructuring
    OrganizationalChange,

    /// Other reason with description
    Other(String),
}

impl std::fmt::Display for RevocationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RevocationReason::DelegatorRevoked => write!(f, "Delegator revoked"),
            RevocationReason::DelegateRequested => write!(f, "Delegate requested"),
            RevocationReason::AdminRevoked => write!(f, "Administrator revoked"),
            RevocationReason::Expired => write!(f, "Expired"),
            RevocationReason::CascadeFromParent => write!(f, "Cascade from parent"),
            RevocationReason::SecurityIncident => write!(f, "Security incident"),
            RevocationReason::RoleChange => write!(f, "Role change"),
            RevocationReason::OrganizationalChange => write!(f, "Organizational change"),
            RevocationReason::Other(reason) => write!(f, "{}", reason),
        }
    }
}

impl DomainEvent for DelegationEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            DelegationEvents::DelegationCreated(e) => e.delegation_id,
            DelegationEvents::DelegationRevoked(e) => e.delegation_id,
            DelegationEvents::DelegationCascadeRevoked(e) => e.delegation_id,
            DelegationEvents::DelegationExtended(e) => e.delegation_id,
            DelegationEvents::DelegationPermissionsModified(e) => e.delegation_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            DelegationEvents::DelegationCreated(_) => "DelegationCreated",
            DelegationEvents::DelegationRevoked(_) => "DelegationRevoked",
            DelegationEvents::DelegationCascadeRevoked(_) => "DelegationCascadeRevoked",
            DelegationEvents::DelegationExtended(_) => "DelegationExtended",
            DelegationEvents::DelegationPermissionsModified(_) => "DelegationPermissionsModified",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revocation_reason_display() {
        assert_eq!(RevocationReason::DelegatorRevoked.to_string(), "Delegator revoked");
        assert_eq!(RevocationReason::SecurityIncident.to_string(), "Security incident");
        assert_eq!(RevocationReason::Other("Custom reason".to_string()).to_string(), "Custom reason");
    }

    #[test]
    fn test_delegation_event_aggregate_id() {
        let event = DelegationEvents::DelegationCreated(DelegationCreatedEvent {
            delegation_id: Uuid::now_v7(),
            delegator_id: Uuid::now_v7(),
            delegate_id: Uuid::now_v7(),
            permissions: vec![KeyPermission::Sign],
            derives_from: None,
            valid_from: Utc::now(),
            valid_until: None,
            created_at: Utc::now(),
            created_by: Uuid::now_v7(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        });

        assert!(event.aggregate_id().to_string().len() > 0);
    }
}
