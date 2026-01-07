// Copyright (c) 2025 - Cowboy AI, LLC.

//! Delegation Aggregate Commands
//!
//! Commands for the Delegation aggregate root.
//! Delegations represent authority transfer from one person to another
//! with specific permissions and temporal bounds.

use chrono::{DateTime, Duration, Utc};
use cim_domain::{Command, EntityId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::KeyPermission;
use crate::events::{DomainEvent, DelegationEvents};
use crate::events::delegation::DelegationCreatedEvent;

/// Marker type for Delegation aggregate
#[derive(Debug, Clone, Copy)]
pub struct DelegationAggregate;

/// Command to create a new delegation
///
/// Delegations transfer authority from one person (delegator)
/// to another (delegate) with specific permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDelegation {
    pub command_id: Uuid,
    pub delegation_id: Uuid,
    pub delegator_id: Uuid,
    pub delegate_id: Uuid,
    pub permissions: Vec<KeyPermission>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub derives_from: Option<Uuid>,
    pub created_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

impl Command for CreateDelegation {
    type Aggregate = DelegationAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.delegation_id))
    }
}

/// Command to revoke a delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeDelegation {
    pub command_id: Uuid,
    pub delegation_id: Uuid,
    pub revoked_by: Uuid,
    pub reason: RevocationReason,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

impl Command for RevokeDelegation {
    type Aggregate = DelegationAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.delegation_id))
    }
}

/// Reason for revoking a delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

// ============================================================================
// Command Handlers
// ============================================================================

/// Handle CreateDelegation command
pub async fn handle_create_delegation(
    cmd: CreateDelegation,
) -> Result<Vec<DomainEvent>, crate::aggregate::KeyManagementError> {
    // Validate: delegator and delegate must be different
    if cmd.delegator_id == cmd.delegate_id {
        return Err(crate::aggregate::KeyManagementError::InvalidCommand(
            "Cannot delegate to self".to_string(),
        ));
    }

    // Validate: must have at least one permission
    if cmd.permissions.is_empty() {
        return Err(crate::aggregate::KeyManagementError::InvalidCommand(
            "Must delegate at least one permission".to_string(),
        ));
    }

    // Validate: valid_until must be after valid_from
    if let Some(until) = cmd.valid_until {
        if until <= cmd.valid_from {
            return Err(crate::aggregate::KeyManagementError::InvalidCommand(
                "Expiration must be after start date".to_string(),
            ));
        }
    }

    // Emit DelegationCreated event
    let event = DomainEvent::Delegation(DelegationEvents::DelegationCreated(
        DelegationCreatedEvent {
            delegation_id: cmd.delegation_id,
            delegator_id: cmd.delegator_id,
            delegate_id: cmd.delegate_id,
            permissions: cmd.permissions,
            derives_from: cmd.derives_from,
            valid_from: cmd.valid_from,
            valid_until: cmd.valid_until,
            created_at: cmd.timestamp,
            created_by: cmd.created_by,
            correlation_id: cmd.correlation_id,
            causation_id: cmd.causation_id,
        }
    ));

    Ok(vec![event])
}

/// Handle RevokeDelegation command
pub async fn handle_revoke_delegation(
    cmd: RevokeDelegation,
) -> Result<Vec<DomainEvent>, crate::aggregate::KeyManagementError> {
    use crate::events::delegation::{DelegationRevokedEvent, RevocationReason as EventRevocationReason};

    // Convert command reason to event reason (same variants, different module)
    let event_reason = match cmd.reason {
        RevocationReason::DelegatorRevoked => EventRevocationReason::DelegatorRevoked,
        RevocationReason::DelegateRequested => EventRevocationReason::DelegateRequested,
        RevocationReason::AdminRevoked => EventRevocationReason::AdminRevoked,
        RevocationReason::Expired => EventRevocationReason::Expired,
        RevocationReason::CascadeFromParent => EventRevocationReason::CascadeFromParent,
        RevocationReason::SecurityIncident => EventRevocationReason::SecurityIncident,
        RevocationReason::RoleChange => EventRevocationReason::RoleChange,
    };

    // Emit DelegationRevoked event
    let event = DomainEvent::Delegation(DelegationEvents::DelegationRevoked(
        DelegationRevokedEvent {
            delegation_id: cmd.delegation_id,
            revoked_by: cmd.revoked_by,
            reason: event_reason,
            revoked_at: cmd.timestamp,
            correlation_id: cmd.correlation_id,
            causation_id: cmd.causation_id,
        }
    ));

    Ok(vec![event])
}

// ============================================================================
// Builder for CreateDelegation (FP-style)
// ============================================================================

impl CreateDelegation {
    /// Create a new delegation command with required fields
    pub fn new(
        delegator_id: Uuid,
        delegate_id: Uuid,
        permissions: Vec<KeyPermission>,
    ) -> Self {
        let now = Utc::now();
        Self {
            command_id: Uuid::now_v7(),
            delegation_id: Uuid::now_v7(),
            delegator_id,
            delegate_id,
            permissions,
            valid_from: now,
            valid_until: None,
            derives_from: None,
            created_by: delegator_id, // Default: delegator creates
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            timestamp: now,
        }
    }

    /// Set expiration in days from now
    pub fn with_expiration_days(mut self, days: i64) -> Self {
        self.valid_until = Some(Utc::now() + Duration::days(days));
        self
    }

    /// Set explicit expiration time
    pub fn with_valid_until(mut self, until: DateTime<Utc>) -> Self {
        self.valid_until = Some(until);
        self
    }

    /// Set parent delegation for transitive delegations
    pub fn with_derives_from(mut self, parent_id: Uuid) -> Self {
        self.derives_from = Some(parent_id);
        self
    }

    /// Set who is creating the delegation (for admin scenarios)
    pub fn with_created_by(mut self, creator_id: Uuid) -> Self {
        self.created_by = creator_id;
        self
    }

    /// Set correlation ID for event chain tracking
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    /// Set causation ID linking to what triggered this
    pub fn with_causation_id(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_delegation_success() {
        let delegator = Uuid::now_v7();
        let delegate = Uuid::now_v7();
        let permissions = vec![KeyPermission::Sign, KeyPermission::Encrypt];

        let cmd = CreateDelegation::new(delegator, delegate, permissions.clone());

        let result = handle_create_delegation(cmd).await;
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            DomainEvent::Delegation(DelegationEvents::DelegationCreated(evt)) => {
                assert_eq!(evt.delegator_id, delegator);
                assert_eq!(evt.delegate_id, delegate);
                assert_eq!(evt.permissions.len(), 2);
            }
            _ => panic!("Expected DelegationCreated event"),
        }
    }

    #[tokio::test]
    async fn test_cannot_delegate_to_self() {
        let person = Uuid::now_v7();
        let permissions = vec![KeyPermission::Sign];

        let cmd = CreateDelegation::new(person, person, permissions);

        let result = handle_create_delegation(cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_must_have_permissions() {
        let delegator = Uuid::now_v7();
        let delegate = Uuid::now_v7();

        let cmd = CreateDelegation::new(delegator, delegate, vec![]);

        let result = handle_create_delegation(cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_expiration_must_be_future() {
        let delegator = Uuid::now_v7();
        let delegate = Uuid::now_v7();
        let permissions = vec![KeyPermission::Sign];

        let past = Utc::now() - Duration::days(1);
        let cmd = CreateDelegation::new(delegator, delegate, permissions)
            .with_valid_until(past);

        let result = handle_create_delegation(cmd).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_command_implements_command_trait() {
        let delegator = Uuid::now_v7();
        let delegate = Uuid::now_v7();
        let permissions = vec![KeyPermission::Sign];

        let cmd = CreateDelegation::new(delegator, delegate, permissions);

        // Command trait requires aggregate_id
        let agg_id = cmd.aggregate_id();
        assert!(agg_id.is_some());
    }

    #[tokio::test]
    async fn test_revoke_delegation() {
        let delegation_id = Uuid::now_v7();
        let revoker = Uuid::now_v7();

        let cmd = RevokeDelegation {
            command_id: Uuid::now_v7(),
            delegation_id,
            revoked_by: revoker,
            reason: RevocationReason::DelegatorRevoked,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            timestamp: Utc::now(),
        };

        let result = handle_revoke_delegation(cmd).await;
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            DomainEvent::Delegation(DelegationEvents::DelegationRevoked(evt)) => {
                assert_eq!(evt.delegation_id, delegation_id);
                assert_eq!(evt.revoked_by, revoker);
            }
            _ => panic!("Expected DelegationRevoked event"),
        }
    }
}
