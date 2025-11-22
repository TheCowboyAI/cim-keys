//! Policy and delegation types extracted from deprecated domain module
//!
//! These types are used across commands, events, and policy modules.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Key Delegation Types
// ============================================================================

/// Key delegation to another person
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyDelegation {
    pub delegated_to: Uuid,
    pub permissions: Vec<KeyPermission>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Permissions that can be delegated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyPermission {
    Sign,
    Encrypt,
    Decrypt,
    CertifyOthers,
    RevokeOthers,
    BackupAccess,
}

// ============================================================================
// Policy Types
// ============================================================================

/// Individual claim (capability/permission)
///
/// Claims represent atomic permissions. They compose additively:
/// Policy A: [CanSignCode] + Policy B: [CanAccessProd] = [CanSignCode, CanAccessProd]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyClaim {
    // ===== Key Management Claims =====
    /// Generate new cryptographic keys
    CanGenerateKeys,
    /// Sign code artifacts (binaries, containers, etc.)
    CanSignCode,
    /// Sign certificates (act as CA)
    CanSignCertificates,
    /// Revoke keys or certificates
    CanRevokeKeys,
    /// Delegate key permissions to others
    CanDelegateKeys,
    /// Export private keys from secure storage
    CanExportKeys,
    /// Backup keys to offline storage
    CanBackupKeys,

    // ===== Environment Access Claims =====
    /// Access production environment
    CanAccessProduction,
    /// Access staging environment
    CanAccessStaging,
    /// Access development environment
    CanAccessDevelopment,

    // ===== Administrative Claims =====
    /// Manage users and roles
    CanManageUsers,
    /// Manage policies
    CanManagePolicies,
    /// View audit logs
    CanViewAuditLogs,

    // ===== Custom Claim =====
    /// Custom claim with arbitrary string identifier
    Custom(String),
}

/// Security clearance levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecurityClearance {
    Unclassified,
    Confidential,
    Secret,
    TopSecret,
}

/// Conditions that must be met for policy to be active
///
/// ALL conditions must be satisfied for the policy to activate.
/// If any condition fails, the policy is inactive (claims don't apply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    /// Minimum security clearance level required
    MinimumSecurityClearance(SecurityClearance),

    /// MFA must be enabled and verified
    MFAEnabled(bool),

    /// YubiKey must be present
    YubiKeyRequired(bool),

    /// Must be at one of these physical locations
    LocationRestriction(Vec<Uuid>),

    /// Must be within time window
    TimeWindow {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },

    /// Must have witness(es) present
    RequiresWitness {
        minimum_count: usize,
        witness_roles: Vec<String>,
    },

    /// Custom condition with arbitrary validation logic
    Custom {
        description: String,
        validator: String, // Reference to validation function/script
    },
}
