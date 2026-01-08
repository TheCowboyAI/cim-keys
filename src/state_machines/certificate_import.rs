// Copyright (c) 2025 - Cowboy AI, LLC.

//! Certificate Import Workflow State Machine
//!
//! This module defines the lifecycle state machine for importing certificates
//! to YubiKey PIV slots. The workflow ensures RFC 5280 compliance validation
//! before allowing import operations.
//!
//! ## State Transitions (Behaviors)
//!
//! ```text
//! NoCertificateSelected
//!       │
//!       ▼ CertificateSelectedForImport
//! CertificateSelected { cert_id, slot }
//!       │
//!       ▼ CertificateValidationStarted
//! Validating { cert_id }
//!       │
//!       ├──▶ CertificateValidationFailed { errors }
//!       │         │
//!       │         ▼
//!       │    ValidationFailed { cert_id, errors }
//!       │         │
//!       │         ▼ CertificateDeselected
//!       │    NoCertificateSelected
//!       │
//!       ▼ CertificateValidationSucceeded { metadata }
//! Validated { cert_id, metadata }
//!       │
//!       ▼ PinRequested
//! AwaitingPin { cert_id, slot }
//!       │
//!       ├──▶ PinEntryFailed { reason }
//!       │         │
//!       │         ▼
//!       │    PinFailed { attempts_remaining }
//!       │         │
//!       │         ├──▶ retry → AwaitingPin
//!       │         └──▶ CertificateImportAborted → NoCertificateSelected
//!       │
//!       ▼ PinVerified
//! Importing { cert_id, slot }
//!       │
//!       ├──▶ CertificateImportFailed { reason }
//!       │         │
//!       │         ▼
//!       │    ImportFailed { cert_id, reason }
//!       │
//!       ▼ CertificateImportSucceeded
//! Imported { cert_id, slot, imported_at }
//!       │
//!       ▼ WorkflowReset
//! NoCertificateSelected
//! ```
//!
//! ## Invariants
//!
//! - Cannot import without RFC 5280 validation passing
//! - Cannot skip PIN verification step
//! - Import requires YubiKey to be in KeysGenerated state
//! - Only one certificate can be in import workflow at a time per YubiKey

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::rfc5280::CertificateMetadata;
use crate::ports::yubikey::PivSlot;

/// Lifecycle state machine for certificate import to YubiKey
///
/// Each state represents a step in the import workflow.
/// Transitions are triggered by domain events (behaviors).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateImportState {
    /// No certificate selected for import
    NoCertificateSelected,

    /// Certificate has been selected, awaiting validation
    CertificateSelected {
        cert_id: Uuid,
        yubikey_serial: String,
        slot: PivSlot,
        selected_at: DateTime<Utc>,
    },

    /// RFC 5280 validation in progress
    Validating {
        cert_id: Uuid,
        yubikey_serial: String,
        slot: PivSlot,
        started_at: DateTime<Utc>,
    },

    /// Validation failed - cannot proceed with import
    ValidationFailed {
        cert_id: Uuid,
        errors: Vec<String>,
        failed_at: DateTime<Utc>,
    },

    /// Validation succeeded - certificate metadata available
    Validated {
        cert_id: Uuid,
        yubikey_serial: String,
        slot: PivSlot,
        metadata: CertificateMetadata,
        validated_at: DateTime<Utc>,
    },

    /// Awaiting PIN entry from user
    AwaitingPin {
        cert_id: Uuid,
        yubikey_serial: String,
        slot: PivSlot,
        metadata: CertificateMetadata,
        attempts_remaining: u8,
    },

    /// PIN verification failed
    PinFailed {
        cert_id: Uuid,
        yubikey_serial: String,
        slot: PivSlot,
        attempts_remaining: u8,
        failed_at: DateTime<Utc>,
    },

    /// Import operation in progress
    Importing {
        cert_id: Uuid,
        yubikey_serial: String,
        slot: PivSlot,
        started_at: DateTime<Utc>,
    },

    /// Import failed
    ImportFailed {
        cert_id: Uuid,
        yubikey_serial: String,
        slot: PivSlot,
        reason: String,
        failed_at: DateTime<Utc>,
    },

    /// Certificate successfully imported (terminal success state)
    Imported {
        cert_id: Uuid,
        yubikey_serial: String,
        slot: PivSlot,
        imported_at: DateTime<Utc>,
    },
}

impl Default for CertificateImportState {
    fn default() -> Self {
        Self::NoCertificateSelected
    }
}

impl CertificateImportState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is a certificate currently selected?
    pub fn has_certificate_selected(&self) -> bool {
        !matches!(self, CertificateImportState::NoCertificateSelected)
    }

    /// Can we select a certificate?
    pub fn can_select_certificate(&self) -> bool {
        matches!(
            self,
            CertificateImportState::NoCertificateSelected
                | CertificateImportState::ValidationFailed { .. }
                | CertificateImportState::ImportFailed { .. }
                | CertificateImportState::Imported { .. }
        )
    }

    /// Can we start validation?
    pub fn can_validate(&self) -> bool {
        matches!(self, CertificateImportState::CertificateSelected { .. })
    }

    /// Is validation in progress?
    pub fn is_validating(&self) -> bool {
        matches!(self, CertificateImportState::Validating { .. })
    }

    /// Did validation fail?
    pub fn is_validation_failed(&self) -> bool {
        matches!(self, CertificateImportState::ValidationFailed { .. })
    }

    /// Is certificate validated and ready for PIN?
    pub fn is_validated(&self) -> bool {
        matches!(self, CertificateImportState::Validated { .. })
    }

    /// Can we request PIN?
    pub fn can_request_pin(&self) -> bool {
        matches!(self, CertificateImportState::Validated { .. })
    }

    /// Is awaiting PIN entry?
    pub fn is_awaiting_pin(&self) -> bool {
        matches!(self, CertificateImportState::AwaitingPin { .. })
    }

    /// Can we attempt import?
    pub fn can_import(&self) -> bool {
        matches!(self, CertificateImportState::AwaitingPin { .. })
    }

    /// Is import in progress?
    pub fn is_importing(&self) -> bool {
        matches!(self, CertificateImportState::Importing { .. })
    }

    /// Was import successful?
    pub fn is_imported(&self) -> bool {
        matches!(self, CertificateImportState::Imported { .. })
    }

    /// Is in a failed state?
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            CertificateImportState::ValidationFailed { .. }
                | CertificateImportState::PinFailed { .. }
                | CertificateImportState::ImportFailed { .. }
        )
    }

    /// Is in a terminal state (success or unrecoverable failure)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, CertificateImportState::Imported { .. })
    }

    /// Can we abort/reset the workflow?
    pub fn can_abort(&self) -> bool {
        !matches!(
            self,
            CertificateImportState::NoCertificateSelected
                | CertificateImportState::Importing { .. }
        )
    }

    // ========================================================================
    // State Data Accessors
    // ========================================================================

    /// Get the certificate ID if one is selected
    pub fn cert_id(&self) -> Option<Uuid> {
        match self {
            CertificateImportState::NoCertificateSelected => None,
            CertificateImportState::CertificateSelected { cert_id, .. } => Some(*cert_id),
            CertificateImportState::Validating { cert_id, .. } => Some(*cert_id),
            CertificateImportState::ValidationFailed { cert_id, .. } => Some(*cert_id),
            CertificateImportState::Validated { cert_id, .. } => Some(*cert_id),
            CertificateImportState::AwaitingPin { cert_id, .. } => Some(*cert_id),
            CertificateImportState::PinFailed { cert_id, .. } => Some(*cert_id),
            CertificateImportState::Importing { cert_id, .. } => Some(*cert_id),
            CertificateImportState::ImportFailed { cert_id, .. } => Some(*cert_id),
            CertificateImportState::Imported { cert_id, .. } => Some(*cert_id),
        }
    }

    /// Get the YubiKey serial if in workflow
    pub fn yubikey_serial(&self) -> Option<&str> {
        match self {
            CertificateImportState::CertificateSelected { yubikey_serial, .. } => {
                Some(yubikey_serial)
            }
            CertificateImportState::Validating { yubikey_serial, .. } => Some(yubikey_serial),
            CertificateImportState::Validated { yubikey_serial, .. } => Some(yubikey_serial),
            CertificateImportState::AwaitingPin { yubikey_serial, .. } => Some(yubikey_serial),
            CertificateImportState::PinFailed { yubikey_serial, .. } => Some(yubikey_serial),
            CertificateImportState::Importing { yubikey_serial, .. } => Some(yubikey_serial),
            CertificateImportState::ImportFailed { yubikey_serial, .. } => Some(yubikey_serial),
            CertificateImportState::Imported { yubikey_serial, .. } => Some(yubikey_serial),
            _ => None,
        }
    }

    /// Get the target slot
    pub fn slot(&self) -> Option<PivSlot> {
        match self {
            CertificateImportState::CertificateSelected { slot, .. } => Some(*slot),
            CertificateImportState::Validating { slot, .. } => Some(*slot),
            CertificateImportState::Validated { slot, .. } => Some(*slot),
            CertificateImportState::AwaitingPin { slot, .. } => Some(*slot),
            CertificateImportState::PinFailed { slot, .. } => Some(*slot),
            CertificateImportState::Importing { slot, .. } => Some(*slot),
            CertificateImportState::ImportFailed { slot, .. } => Some(*slot),
            CertificateImportState::Imported { slot, .. } => Some(*slot),
            _ => None,
        }
    }

    /// Get certificate metadata if validated
    pub fn metadata(&self) -> Option<&CertificateMetadata> {
        match self {
            CertificateImportState::Validated { metadata, .. } => Some(metadata),
            CertificateImportState::AwaitingPin { metadata, .. } => Some(metadata),
            _ => None,
        }
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &CertificateImportState) -> bool {
        match (self, target) {
            // NoCertificateSelected → CertificateSelected
            (
                CertificateImportState::NoCertificateSelected,
                CertificateImportState::CertificateSelected { .. },
            ) => true,

            // CertificateSelected → Validating
            (
                CertificateImportState::CertificateSelected { .. },
                CertificateImportState::Validating { .. },
            ) => true,

            // Validating → ValidationFailed
            (
                CertificateImportState::Validating { .. },
                CertificateImportState::ValidationFailed { .. },
            ) => true,

            // Validating → Validated
            (
                CertificateImportState::Validating { .. },
                CertificateImportState::Validated { .. },
            ) => true,

            // ValidationFailed → NoCertificateSelected (retry with different cert)
            (
                CertificateImportState::ValidationFailed { .. },
                CertificateImportState::NoCertificateSelected,
            ) => true,

            // ValidationFailed → CertificateSelected (select different cert)
            (
                CertificateImportState::ValidationFailed { .. },
                CertificateImportState::CertificateSelected { .. },
            ) => true,

            // Validated → AwaitingPin
            (
                CertificateImportState::Validated { .. },
                CertificateImportState::AwaitingPin { .. },
            ) => true,

            // AwaitingPin → PinFailed
            (
                CertificateImportState::AwaitingPin { .. },
                CertificateImportState::PinFailed { .. },
            ) => true,

            // AwaitingPin → Importing
            (
                CertificateImportState::AwaitingPin { .. },
                CertificateImportState::Importing { .. },
            ) => true,

            // PinFailed → AwaitingPin (retry)
            (
                CertificateImportState::PinFailed { attempts_remaining, .. },
                CertificateImportState::AwaitingPin { .. },
            ) => *attempts_remaining > 0,

            // PinFailed → NoCertificateSelected (abort)
            (
                CertificateImportState::PinFailed { .. },
                CertificateImportState::NoCertificateSelected,
            ) => true,

            // Importing → ImportFailed
            (
                CertificateImportState::Importing { .. },
                CertificateImportState::ImportFailed { .. },
            ) => true,

            // Importing → Imported
            (
                CertificateImportState::Importing { .. },
                CertificateImportState::Imported { .. },
            ) => true,

            // ImportFailed → NoCertificateSelected (reset)
            (
                CertificateImportState::ImportFailed { .. },
                CertificateImportState::NoCertificateSelected,
            ) => true,

            // Imported → NoCertificateSelected (reset for next import)
            (
                CertificateImportState::Imported { .. },
                CertificateImportState::NoCertificateSelected,
            ) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            CertificateImportState::NoCertificateSelected => "No certificate selected",
            CertificateImportState::CertificateSelected { .. } => "Certificate selected",
            CertificateImportState::Validating { .. } => "Validating RFC 5280 compliance",
            CertificateImportState::ValidationFailed { .. } => "Validation failed",
            CertificateImportState::Validated { .. } => "Certificate validated",
            CertificateImportState::AwaitingPin { .. } => "Awaiting PIN entry",
            CertificateImportState::PinFailed { .. } => "PIN verification failed",
            CertificateImportState::Importing { .. } => "Importing certificate",
            CertificateImportState::ImportFailed { .. } => "Import failed",
            CertificateImportState::Imported { .. } => "Certificate imported",
        }
    }

    /// Get the state name for display/matching
    pub fn state_name(&self) -> &'static str {
        match self {
            CertificateImportState::NoCertificateSelected => "NoCertificateSelected",
            CertificateImportState::CertificateSelected { .. } => "CertificateSelected",
            CertificateImportState::Validating { .. } => "Validating",
            CertificateImportState::ValidationFailed { .. } => "ValidationFailed",
            CertificateImportState::Validated { .. } => "Validated",
            CertificateImportState::AwaitingPin { .. } => "AwaitingPin",
            CertificateImportState::PinFailed { .. } => "PinFailed",
            CertificateImportState::Importing { .. } => "Importing",
            CertificateImportState::ImportFailed { .. } => "ImportFailed",
            CertificateImportState::Imported { .. } => "Imported",
        }
    }
}

// ============================================================================
// State Transition Errors
// ============================================================================

/// Errors that can occur during state transitions
#[derive(Debug, Clone, thiserror::Error)]
pub enum CertificateImportError {
    #[error("Invalid state transition from {current} to {target}: {reason}")]
    InvalidTransition {
        current: String,
        target: String,
        reason: String,
    },

    #[error("RFC 5280 validation failed: {0}")]
    ValidationFailed(String),

    #[error("PIN verification failed: {attempts_remaining} attempts remaining")]
    PinFailed { attempts_remaining: u8 },

    #[error("PIN locked: no attempts remaining")]
    PinLocked,

    #[error("Import failed: {0}")]
    ImportFailed(String),

    #[error("YubiKey not in correct state for import")]
    YubiKeyStateInvalid,

    #[error("Certificate not found: {0}")]
    CertificateNotFound(Uuid),

    #[error("Operation aborted by user")]
    Aborted,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = CertificateImportState::default();
        assert!(matches!(state, CertificateImportState::NoCertificateSelected));
        assert!(!state.has_certificate_selected());
        assert!(state.can_select_certificate());
    }

    #[test]
    fn test_valid_transitions() {
        let initial = CertificateImportState::NoCertificateSelected;
        let selected = CertificateImportState::CertificateSelected {
            cert_id: Uuid::new_v4(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            selected_at: Utc::now(),
        };

        assert!(initial.can_transition_to(&selected));
    }

    #[test]
    fn test_invalid_transitions() {
        let initial = CertificateImportState::NoCertificateSelected;
        let importing = CertificateImportState::Importing {
            cert_id: Uuid::new_v4(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            started_at: Utc::now(),
        };

        // Can't jump from NoCertificateSelected directly to Importing
        assert!(!initial.can_transition_to(&importing));
    }

    #[test]
    fn test_state_queries() {
        let validated = CertificateImportState::Validated {
            cert_id: Uuid::new_v4(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            metadata: create_test_metadata(),
            validated_at: Utc::now(),
        };

        assert!(validated.is_validated());
        assert!(validated.can_request_pin());
        assert!(!validated.is_importing());
        assert!(!validated.is_terminal());
    }

    #[test]
    fn test_pin_retry_allowed() {
        let pin_failed = CertificateImportState::PinFailed {
            cert_id: Uuid::new_v4(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            attempts_remaining: 2,
            failed_at: Utc::now(),
        };

        let awaiting_pin = CertificateImportState::AwaitingPin {
            cert_id: Uuid::new_v4(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            metadata: create_test_metadata(),
            attempts_remaining: 2,
        };

        // Can retry when attempts remaining
        assert!(pin_failed.can_transition_to(&awaiting_pin));
    }

    #[test]
    fn test_pin_retry_blocked_when_locked() {
        let pin_locked = CertificateImportState::PinFailed {
            cert_id: Uuid::new_v4(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            attempts_remaining: 0,
            failed_at: Utc::now(),
        };

        let awaiting_pin = CertificateImportState::AwaitingPin {
            cert_id: Uuid::new_v4(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            metadata: create_test_metadata(),
            attempts_remaining: 0,
        };

        // Cannot retry when no attempts remaining
        assert!(!pin_locked.can_transition_to(&awaiting_pin));
    }

    fn create_test_metadata() -> CertificateMetadata {
        CertificateMetadata {
            version: 3,
            serial_number: "01".to_string(),
            subject_cn: Some("Test Cert".to_string()),
            subject_org: Some("Test Org".to_string()),
            issuer_cn: Some("Test CA".to_string()),
            not_before: Utc::now(),
            not_after: Utc::now(),
            is_ca: false,
            path_length: None,
            key_usage: vec![],
            extended_key_usage: vec![],
            subject_alt_names: vec![],
            fingerprint_sha256: "abc123".to_string(),
        }
    }
}
