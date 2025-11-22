//! Certificate Aggregate State Machine
//!
//! This module defines the lifecycle state machine for PKI certificates.
//! Certificates transition through 8 states from pending to archival.
//!
//! State Transitions:
//! - Pending → Issued (CertificateSigned)
//! - Issued → Active (time-based, not_before reached)
//! - Active → RenewalPending (CertificateRenewed initiated)
//! - RenewalPending → Renewed (new cert issued)
//! - Any → Revoked (CertificateRevoked - terminal)
//! - Active → Expired (time-based, not_after reached)
//! - Renewed/Revoked/Expired → Archived (terminal)
//!
//! Invariants:
//! - Can only use for TLS/signing if Active
//! - Can't renew if already in RenewalPending
//! - Revoked certs can't be reactivated
//! - Must publish to CRL when revoked

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Note: Event integration will be added in Phase 4

/// Lifecycle state machine for PKI certificates
///
/// Enforces certificate lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateState {
    /// Certificate requested, CSR created, awaiting signing
    Pending {
        csr_id: Option<Uuid>,
        pending_since: DateTime<Utc>,
        requested_by: Uuid, // Person ID
    },

    /// Certificate signed by CA but not yet valid (not_before in future)
    Issued {
        issued_at: DateTime<Utc>,
        issuer_id: Uuid, // CA certificate ID
        issued_by: Uuid, // Person ID who signed
    },

    /// Certificate is active and valid for use
    Active {
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        usage_count: u64,
        last_used: Option<DateTime<Utc>>,
    },

    /// Certificate renewal has been initiated
    RenewalPending {
        new_cert_id: Uuid,
        initiated_at: DateTime<Utc>,
        initiated_by: Uuid, // Person ID
    },

    /// Certificate has been renewed, new cert is active
    Renewed {
        new_cert_id: Uuid,
        renewed_at: DateTime<Utc>,
        renewed_by: Uuid, // Person ID
    },

    /// Certificate has been revoked (TERMINAL STATE)
    Revoked {
        reason: RevocationReason,
        revoked_at: DateTime<Utc>,
        revoked_by: Uuid,      // Person ID
        crl_published: bool,    // Has CRL been updated?
        ocsp_updated: bool,     // Has OCSP responder been notified?
    },

    /// Certificate has expired based on not_after date
    Expired {
        expired_at: DateTime<Utc>,
        not_after: DateTime<Utc>,
    },

    /// Certificate has been archived (TERMINAL STATE)
    Archived {
        archived_at: DateTime<Utc>,
        archived_by: Uuid, // Person ID
        previous_state: ArchivedFromState,
    },
}

impl CertificateState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the certificate active and valid?
    pub fn is_active(&self) -> bool {
        matches!(self, CertificateState::Active { .. })
    }

    /// Can the certificate be used for TLS/signing?
    pub fn can_use_for_crypto(&self) -> bool {
        matches!(self, CertificateState::Active { .. })
    }

    /// Can the certificate be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            CertificateState::Revoked { .. } | CertificateState::Archived { .. }
        )
    }

    /// Is certificate renewal in progress?
    pub fn is_renewal_pending(&self) -> bool {
        matches!(self, CertificateState::RenewalPending { .. })
    }

    /// Has the certificate been renewed?
    pub fn is_renewed(&self) -> bool {
        matches!(self, CertificateState::Renewed { .. })
    }

    /// Has the certificate expired?
    pub fn is_expired(&self) -> bool {
        matches!(self, CertificateState::Expired { .. })
    }

    /// Has the certificate been revoked?
    pub fn is_revoked(&self) -> bool {
        matches!(self, CertificateState::Revoked { .. })
    }

    /// Is the certificate pending issuance?
    pub fn is_pending(&self) -> bool {
        matches!(self, CertificateState::Pending { .. })
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &CertificateState) -> bool {
        match (self, target) {
            // Pending → Issued
            (CertificateState::Pending { .. }, CertificateState::Issued { .. }) => true,

            // Issued → Active
            (CertificateState::Issued { .. }, CertificateState::Active { .. }) => true,

            // Active → RenewalPending
            (CertificateState::Active { .. }, CertificateState::RenewalPending { .. }) => true,

            // RenewalPending → Renewed
            (CertificateState::RenewalPending { .. }, CertificateState::Renewed { .. }) => true,

            // Any non-terminal → Revoked
            (_, CertificateState::Revoked { .. }) if !self.is_terminal() => true,

            // Active → Expired
            (CertificateState::Active { .. }, CertificateState::Expired { .. }) => true,

            // Renewed → Archived
            (CertificateState::Renewed { .. }, CertificateState::Archived { .. }) => true,

            // Revoked → Archived
            (CertificateState::Revoked { .. }, CertificateState::Archived { .. }) => true,

            // Expired → Archived
            (CertificateState::Expired { .. }, CertificateState::Archived { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // TODO: Event validation and application will be implemented in Phase 4
    // when wiring state machines to aggregate event handlers.
    // For now, state transitions are managed through explicit transition methods.

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            CertificateState::Pending { .. } => "Pending (awaiting CA signature)",
            CertificateState::Issued { .. } => "Issued (not yet valid)",
            CertificateState::Active { .. } => "Active (valid for use)",
            CertificateState::RenewalPending { .. } => "Renewal Pending (new cert being issued)",
            CertificateState::Renewed { .. } => "Renewed (superseded by new certificate)",
            CertificateState::Revoked { .. } => "Revoked (TERMINAL - check CRL/OCSP)",
            CertificateState::Expired { .. } => "Expired (validity period ended)",
            CertificateState::Archived { .. } => "Archived (TERMINAL - long-term storage)",
        }
    }

    /// Check if certificate is currently valid (time-based)
    pub fn is_time_valid(&self, now: DateTime<Utc>) -> bool {
        match self {
            CertificateState::Active {
                not_before,
                not_after,
                ..
            } => now >= *not_before && now <= *not_after,
            _ => false,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Reason for certificate revocation (RFC 5280)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RevocationReason {
    /// Unspecified reason
    Unspecified,
    /// Private key compromised
    KeyCompromise,
    /// CA key compromised
    CACompromise,
    /// Certificate subject affiliation changed
    AffiliationChanged,
    /// Certificate superseded by newer one
    Superseded,
    /// No longer needed
    CessationOfOperation,
    /// Certificate on hold (temporary)
    CertificateHold,
    /// Remove from CRL (only from CertificateHold)
    RemoveFromCRL,
    /// Privilege withdrawn
    PrivilegeWithdrawn,
    /// AA (Attribute Authority) compromised
    AACompromise,
}

/// State from which certificate was archived
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ArchivedFromState {
    Renewed,
    Revoked,
    Expired,
}

/// Errors that can occur during state transitions
#[derive(Debug, Clone, thiserror::Error)]
pub enum StateError {
    #[error("Invalid state transition from {current} on event {event}: {reason}")]
    InvalidTransition {
        current: String,
        event: String,
        reason: String,
    },

    #[error("Terminal state reached: {0}")]
    TerminalState(String),

    #[error("State validation failed: {0}")]
    ValidationFailed(String),

    #[error("Time validation failed: {0}")]
    TimeValidationFailed(String),
}
