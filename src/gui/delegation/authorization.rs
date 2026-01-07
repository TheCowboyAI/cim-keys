// Copyright (c) 2025 - Cowboy AI, LLC.

//! Delegation Message Definitions
//!
//! This module defines the message types for the Delegation bounded context.
//! Handlers are in gui.rs - this module only provides message organization
//! and the DelegationEntry value type.
//!
//! ## Sub-domains
//!
//! 1. **Person Selection**: Grantor (from) and grantee (to)
//! 2. **Permission Management**: Permission set toggles
//! 3. **Expiration**: Time-limited delegations
//! 4. **Lifecycle**: Create, revoke, track delegations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::KeyPermission;

/// Entry for displaying active delegations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationEntry {
    pub id: Uuid,
    pub from_person_id: Uuid,
    pub from_person_name: String,
    pub to_person_id: Uuid,
    pub to_person_name: String,
    pub permissions: Vec<KeyPermission>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl DelegationEntry {
    /// Check if delegation has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if delegation is currently valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    /// Get days until expiration (None if no expiration)
    pub fn days_until_expiration(&self) -> Option<i64> {
        self.expires_at.map(|exp| {
            let duration = exp - Utc::now();
            duration.num_days()
        })
    }
}

/// Delegation Message
///
/// Organized by sub-domain:
/// - Section Toggle (1 message)
/// - Person Selection (2 messages)
/// - Permission Management (1 message)
/// - Expiration (1 message)
/// - Lifecycle (4 messages)
#[derive(Debug, Clone)]
pub enum DelegationMessage {
    // === Section Toggle ===
    /// Toggle delegation section visibility
    ToggleDelegationSection,

    // === Person Selection ===
    /// Select person to delegate from (grantor)
    DelegationFromPersonSelected(Uuid),
    /// Select person to delegate to (grantee)
    DelegationToPersonSelected(Uuid),

    // === Permission Management ===
    /// Toggle a specific permission in the delegation set
    ToggleDelegationPermission(KeyPermission),

    // === Expiration ===
    /// Change expiration days (empty = no expiration)
    DelegationExpiresDaysChanged(String),

    // === Lifecycle ===
    /// Create a new delegation
    CreateDelegation,
    /// Delegation creation completed
    DelegationCreated(Result<DelegationEntry, String>),
    /// Revoke an existing delegation
    RevokeDelegation(Uuid),
    /// Delegation revocation completed
    DelegationRevoked(Result<Uuid, String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_entry_validity() {
        let entry = DelegationEntry {
            id: Uuid::now_v7(),
            from_person_id: Uuid::now_v7(),
            from_person_name: "Alice".to_string(),
            to_person_id: Uuid::now_v7(),
            to_person_name: "Bob".to_string(),
            permissions: vec![KeyPermission::Sign],
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
        };

        assert!(entry.is_valid());
        assert!(!entry.is_expired());
        assert!(entry.days_until_expiration().is_none());
    }

    #[test]
    fn test_delegation_entry_expired() {
        let entry = DelegationEntry {
            id: Uuid::now_v7(),
            from_person_id: Uuid::now_v7(),
            from_person_name: "Alice".to_string(),
            to_person_id: Uuid::now_v7(),
            to_person_name: "Bob".to_string(),
            permissions: vec![KeyPermission::Sign],
            created_at: Utc::now() - chrono::Duration::days(10),
            expires_at: Some(Utc::now() - chrono::Duration::days(1)), // Expired yesterday
            is_active: true,
        };

        assert!(!entry.is_valid()); // Active but expired
        assert!(entry.is_expired());
    }

    #[test]
    fn test_delegation_message_variants() {
        let _ = DelegationMessage::ToggleDelegationSection;
        let _ = DelegationMessage::DelegationFromPersonSelected(Uuid::nil());
        let _ = DelegationMessage::DelegationToPersonSelected(Uuid::nil());
        let _ = DelegationMessage::ToggleDelegationPermission(KeyPermission::Sign);
        let _ = DelegationMessage::DelegationExpiresDaysChanged("30".to_string());
        let _ = DelegationMessage::CreateDelegation;
        let _ = DelegationMessage::RevokeDelegation(Uuid::nil());
    }
}
