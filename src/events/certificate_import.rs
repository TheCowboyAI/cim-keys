// Copyright (c) 2025 - Cowboy AI, LLC.

//! Certificate Import Workflow Events
//!
//! Events for the certificate import state machine, tracking the complete
//! workflow from certificate selection through RFC 5280 validation to
//! YubiKey import.
//!
//! ## Event Flow
//!
//! ```text
//! CertificateSelectedForImport
//!       │
//!       ▼
//! CertificateValidationStarted
//!       │
//!       ├──▶ CertificateValidationFailed
//!       │
//!       ▼
//! CertificateValidationSucceeded
//!       │
//!       ▼
//! PinEntryRequested
//!       │
//!       ├──▶ PinEntryFailed (retry available)
//!       │
//!       ▼
//! PinVerified
//!       │
//!       ▼
//! CertificateImportStarted
//!       │
//!       ├──▶ CertificateImportFailed
//!       │
//!       ▼
//! CertificateImportSucceeded
//! ```

use chrono::{DateTime, Utc};
use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::rfc5280::CertificateMetadata;
use crate::ports::yubikey::PivSlot;

/// Events for the Certificate Import workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum CertificateImportEvents {
    /// A certificate was selected for import to YubiKey
    CertificateSelectedForImport(CertificateSelectedForImportEvent),

    /// Certificate selection was cancelled or deselected
    CertificateDeselected(CertificateDeselectedEvent),

    /// RFC 5280 validation started
    CertificateValidationStarted(CertificateValidationStartedEvent),

    /// RFC 5280 validation succeeded
    CertificateValidationSucceeded(CertificateValidationSucceededEvent),

    /// RFC 5280 validation failed
    CertificateValidationFailed(CertificateValidationFailedEvent),

    /// PIN entry was requested from user
    PinEntryRequested(PinEntryRequestedEvent),

    /// PIN was successfully verified
    PinVerified(PinVerifiedEvent),

    /// PIN entry failed (wrong PIN)
    PinEntryFailed(PinEntryFailedEvent),

    /// Certificate import to YubiKey started
    CertificateImportStarted(CertificateImportStartedEvent),

    /// Certificate successfully imported to YubiKey
    CertificateImportSucceeded(CertificateImportSucceededEvent),

    /// Certificate import to YubiKey failed
    CertificateImportFailed(CertificateImportFailedEvent),

    /// Import workflow was aborted by user
    CertificateImportAborted(CertificateImportAbortedEvent),

    /// Workflow was reset for next import
    WorkflowReset(WorkflowResetEvent),
}

// ============================================================================
// Selection Events
// ============================================================================

/// A certificate was selected for import to YubiKey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSelectedForImportEvent {
    /// Unique identifier for the certificate being selected
    pub cert_id: Uuid,
    /// Serial number of the target YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot on the YubiKey
    pub slot: PivSlot,
    /// When the certificate was selected
    pub selected_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// Certificate selection was cancelled or deselected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateDeselectedEvent {
    /// Certificate that was deselected (if any)
    pub cert_id: Option<Uuid>,
    /// Reason for deselection
    pub reason: String,
    /// When the deselection occurred
    pub deselected_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Validation Events
// ============================================================================

/// RFC 5280 validation started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidationStartedEvent {
    /// Certificate being validated
    pub cert_id: Uuid,
    /// Serial number of the target YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// When validation started
    pub started_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// RFC 5280 validation succeeded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidationSucceededEvent {
    /// Certificate that passed validation
    pub cert_id: Uuid,
    /// Serial number of the target YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// Extracted certificate metadata
    pub metadata: CertificateMetadata,
    /// When validation completed
    pub validated_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// RFC 5280 validation failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidationFailedEvent {
    /// Certificate that failed validation
    pub cert_id: Uuid,
    /// List of validation errors
    pub errors: Vec<String>,
    /// List of validation warnings (non-blocking)
    pub warnings: Vec<String>,
    /// When validation failed
    pub failed_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// PIN Events
// ============================================================================

/// PIN entry was requested from user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinEntryRequestedEvent {
    /// Certificate awaiting import after PIN verification
    pub cert_id: Uuid,
    /// Serial number of the target YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// Number of PIN attempts remaining before lockout
    pub attempts_remaining: u8,
    /// When PIN was requested
    pub requested_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// PIN was successfully verified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinVerifiedEvent {
    /// Certificate ready for import
    pub cert_id: Uuid,
    /// Serial number of the YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// When PIN was verified
    pub verified_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// PIN entry failed (wrong PIN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinEntryFailedEvent {
    /// Certificate that was being imported
    pub cert_id: Uuid,
    /// Serial number of the YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// Number of PIN attempts remaining
    pub attempts_remaining: u8,
    /// Reason for failure
    pub reason: String,
    /// When PIN entry failed
    pub failed_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Import Events
// ============================================================================

/// Certificate import to YubiKey started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateImportStartedEvent {
    /// Certificate being imported
    pub cert_id: Uuid,
    /// Serial number of the target YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// When import started
    pub started_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// Certificate successfully imported to YubiKey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateImportSucceededEvent {
    /// Certificate that was imported
    pub cert_id: Uuid,
    /// Serial number of the YubiKey
    pub yubikey_serial: String,
    /// PIV slot where certificate was stored
    pub slot: PivSlot,
    /// Certificate metadata
    pub metadata: CertificateMetadata,
    /// When import completed
    pub imported_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// Certificate import to YubiKey failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateImportFailedEvent {
    /// Certificate that failed to import
    pub cert_id: Uuid,
    /// Serial number of the YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// Reason for failure
    pub reason: String,
    /// When import failed
    pub failed_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// Import workflow was aborted by user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateImportAbortedEvent {
    /// Certificate that was being imported (if any)
    pub cert_id: Option<Uuid>,
    /// Serial number of the YubiKey (if known)
    pub yubikey_serial: Option<String>,
    /// Reason for abort
    pub reason: String,
    /// When workflow was aborted
    pub aborted_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

/// Workflow was reset for next import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResetEvent {
    /// Previous certificate ID (if any)
    pub previous_cert_id: Option<Uuid>,
    /// When workflow was reset
    pub reset_at: DateTime<Utc>,
    /// Correlation ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// DomainEvent Implementation
// ============================================================================

impl DomainEvent for CertificateImportEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            CertificateImportEvents::CertificateSelectedForImport(e) => e.cert_id,
            CertificateImportEvents::CertificateDeselected(e) => e.cert_id.unwrap_or(Uuid::nil()),
            CertificateImportEvents::CertificateValidationStarted(e) => e.cert_id,
            CertificateImportEvents::CertificateValidationSucceeded(e) => e.cert_id,
            CertificateImportEvents::CertificateValidationFailed(e) => e.cert_id,
            CertificateImportEvents::PinEntryRequested(e) => e.cert_id,
            CertificateImportEvents::PinVerified(e) => e.cert_id,
            CertificateImportEvents::PinEntryFailed(e) => e.cert_id,
            CertificateImportEvents::CertificateImportStarted(e) => e.cert_id,
            CertificateImportEvents::CertificateImportSucceeded(e) => e.cert_id,
            CertificateImportEvents::CertificateImportFailed(e) => e.cert_id,
            CertificateImportEvents::CertificateImportAborted(e) => {
                e.cert_id.unwrap_or(Uuid::nil())
            }
            CertificateImportEvents::WorkflowReset(e) => e.previous_cert_id.unwrap_or(Uuid::nil()),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            CertificateImportEvents::CertificateSelectedForImport(_) => {
                "CertificateSelectedForImport"
            }
            CertificateImportEvents::CertificateDeselected(_) => "CertificateDeselected",
            CertificateImportEvents::CertificateValidationStarted(_) => {
                "CertificateValidationStarted"
            }
            CertificateImportEvents::CertificateValidationSucceeded(_) => {
                "CertificateValidationSucceeded"
            }
            CertificateImportEvents::CertificateValidationFailed(_) => {
                "CertificateValidationFailed"
            }
            CertificateImportEvents::PinEntryRequested(_) => "PinEntryRequested",
            CertificateImportEvents::PinVerified(_) => "PinVerified",
            CertificateImportEvents::PinEntryFailed(_) => "PinEntryFailed",
            CertificateImportEvents::CertificateImportStarted(_) => "CertificateImportStarted",
            CertificateImportEvents::CertificateImportSucceeded(_) => "CertificateImportSucceeded",
            CertificateImportEvents::CertificateImportFailed(_) => "CertificateImportFailed",
            CertificateImportEvents::CertificateImportAborted(_) => "CertificateImportAborted",
            CertificateImportEvents::WorkflowReset(_) => "WorkflowReset",
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> CertificateMetadata {
        CertificateMetadata {
            version: 3,
            serial_number: "01".to_string(),
            subject_cn: Some("Test Certificate".to_string()),
            subject_org: Some("Test Org".to_string()),
            issuer_cn: Some("Test CA".to_string()),
            not_before: Utc::now(),
            not_after: Utc::now(),
            is_ca: false,
            path_length: None,
            key_usage: vec!["digitalSignature".to_string()],
            extended_key_usage: vec![],
            subject_alt_names: vec![],
            fingerprint_sha256: "abc123".to_string(),
        }
    }

    #[test]
    fn test_certificate_selected_event() {
        let event = CertificateSelectedForImportEvent {
            cert_id: Uuid::now_v7(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            selected_at: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let wrapped = CertificateImportEvents::CertificateSelectedForImport(event.clone());
        assert_eq!(wrapped.aggregate_id(), event.cert_id);
        assert_eq!(wrapped.event_type(), "CertificateSelectedForImport");
    }

    #[test]
    fn test_validation_succeeded_event() {
        let event = CertificateValidationSucceededEvent {
            cert_id: Uuid::now_v7(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Signature,
            metadata: create_test_metadata(),
            validated_at: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let wrapped = CertificateImportEvents::CertificateValidationSucceeded(event.clone());
        assert_eq!(wrapped.aggregate_id(), event.cert_id);
        assert_eq!(wrapped.event_type(), "CertificateValidationSucceeded");
    }

    #[test]
    fn test_import_succeeded_event() {
        let event = CertificateImportSucceededEvent {
            cert_id: Uuid::now_v7(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            metadata: create_test_metadata(),
            imported_at: Utc::now(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let wrapped = CertificateImportEvents::CertificateImportSucceeded(event.clone());
        assert_eq!(wrapped.aggregate_id(), event.cert_id);
        assert_eq!(wrapped.event_type(), "CertificateImportSucceeded");
    }

    #[test]
    fn test_event_serialization() {
        let event = CertificateImportEvents::CertificateSelectedForImport(
            CertificateSelectedForImportEvent {
                cert_id: Uuid::now_v7(),
                yubikey_serial: "12345678".to_string(),
                slot: PivSlot::Authentication,
                selected_at: Utc::now(),
                correlation_id: Uuid::now_v7(),
                causation_id: None,
            },
        );

        let serialized = serde_json::to_string(&event).expect("serialization should succeed");
        let deserialized: CertificateImportEvents =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(event.event_type(), deserialized.event_type());
    }
}
