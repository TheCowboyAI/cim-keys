// Copyright (c) 2025 - Cowboy AI, LLC.

//! Certificate Import Commands
//!
//! Commands for the CertificateImport aggregate. These commands drive state
//! transitions through the CertificateImportState machine and emit events.
//!
//! ## Command Flow
//!
//! 1. SelectCertificateForImport → CertificateSelectedForImport
//! 2. ValidateCertificate → CertificateValidationStarted
//! 3. ProvidePin → PinVerified or PinEntryFailed
//! 4. ImportCertificate → CertificateImportSucceeded or CertificateImportFailed
//! 5. AbortImport → CertificateImportAborted
//! 6. ResetWorkflow → WorkflowReset

use chrono::Utc;
use uuid::Uuid;

use crate::crypto::rfc5280::{CertificateMetadata, Rfc5280ValidationResult};
use crate::events::certificate_import::*;
use crate::events::{CertificateImportEvents, DomainEvent};
use crate::ports::yubikey::PivSlot;
use crate::state_machines::certificate_import::{CertificateImportError, CertificateImportState};

// ============================================================================
// Commands
// ============================================================================

/// Select a certificate for import to YubiKey
#[derive(Debug, Clone)]
pub struct SelectCertificateForImport {
    /// ID of the certificate to import
    pub cert_id: Uuid,
    /// Serial number of the target YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// Correlation ID for this workflow
    pub correlation_id: Uuid,
    /// Causation ID (what triggered this command)
    pub causation_id: Option<Uuid>,
}

/// Validate a selected certificate against RFC 5280
#[derive(Debug, Clone)]
pub struct ValidateCertificate {
    /// Certificate to validate
    pub cert_id: Uuid,
    /// Certificate PEM data for validation
    pub certificate_pem: Vec<u8>,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// Causation ID
    pub causation_id: Option<Uuid>,
}

/// Provide PIN for YubiKey import
#[derive(Debug, Clone)]
pub struct ProvidePin {
    /// Certificate being imported
    pub cert_id: Uuid,
    /// PIN attempt (not stored in events)
    pub pin: String,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// Causation ID
    pub causation_id: Option<Uuid>,
}

/// Execute the certificate import to YubiKey
#[derive(Debug, Clone)]
pub struct ImportCertificate {
    /// Certificate to import
    pub cert_id: Uuid,
    /// Certificate PEM data
    pub certificate_pem: Vec<u8>,
    /// Serial number of target YubiKey
    pub yubikey_serial: String,
    /// Target PIV slot
    pub slot: PivSlot,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// Causation ID
    pub causation_id: Option<Uuid>,
}

/// Abort the current import workflow
#[derive(Debug, Clone)]
pub struct AbortImport {
    /// Reason for aborting
    pub reason: String,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// Causation ID
    pub causation_id: Option<Uuid>,
}

/// Reset the workflow for a new import
#[derive(Debug, Clone)]
pub struct ResetWorkflow {
    /// Correlation ID
    pub correlation_id: Uuid,
    /// Causation ID
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Command Results
// ============================================================================

/// Result of command handling
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Events produced by the command
    pub events: Vec<DomainEvent>,
    /// New state after applying events
    pub new_state: CertificateImportState,
}

// ============================================================================
// Command Handlers
// ============================================================================

/// Handle SelectCertificateForImport command
pub fn handle_select_certificate(
    cmd: SelectCertificateForImport,
    current_state: &CertificateImportState,
) -> Result<CommandResult, CertificateImportError> {
    // Validate state allows selection
    if !current_state.can_select_certificate() {
        return Err(CertificateImportError::InvalidTransition {
            current: current_state.state_name().to_string(),
            target: "CertificateSelected".to_string(),
            reason: "Cannot select certificate in current state".to_string(),
        });
    }

    let now = Utc::now();

    // Create the event
    let event = CertificateSelectedForImportEvent {
        cert_id: cmd.cert_id,
        yubikey_serial: cmd.yubikey_serial.clone(),
        slot: cmd.slot,
        selected_at: now,
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    };

    // New state
    let new_state = CertificateImportState::CertificateSelected {
        cert_id: cmd.cert_id,
        yubikey_serial: cmd.yubikey_serial,
        slot: cmd.slot,
        selected_at: now,
    };

    Ok(CommandResult {
        events: vec![DomainEvent::CertificateImport(
            CertificateImportEvents::CertificateSelectedForImport(event),
        )],
        new_state,
    })
}

/// Handle ValidateCertificate command
pub fn handle_validate_certificate(
    cmd: ValidateCertificate,
    current_state: &CertificateImportState,
    validation_result: &Rfc5280ValidationResult,
) -> Result<CommandResult, CertificateImportError> {
    // Validate state allows validation
    if !current_state.can_validate() {
        return Err(CertificateImportError::InvalidTransition {
            current: current_state.state_name().to_string(),
            target: "Validating".to_string(),
            reason: "Cannot validate in current state".to_string(),
        });
    }

    let now = Utc::now();
    let (yubikey_serial, slot) = match current_state {
        CertificateImportState::CertificateSelected {
            yubikey_serial,
            slot,
            ..
        } => (yubikey_serial.clone(), *slot),
        _ => {
            return Err(CertificateImportError::InvalidTransition {
                current: current_state.state_name().to_string(),
                target: "Validated".to_string(),
                reason: "Missing YubiKey info".to_string(),
            })
        }
    };

    // Check validation result
    if !validation_result.is_valid() {
        let event = CertificateValidationFailedEvent {
            cert_id: cmd.cert_id,
            errors: validation_result.errors().iter().map(|e| e.to_string()).collect(),
            warnings: validation_result.warnings().to_vec(),
            failed_at: now,
            correlation_id: cmd.correlation_id,
            causation_id: cmd.causation_id,
        };

        let new_state = CertificateImportState::ValidationFailed {
            cert_id: cmd.cert_id,
            errors: validation_result.errors().iter().map(|e| e.to_string()).collect(),
            failed_at: now,
        };

        return Ok(CommandResult {
            events: vec![DomainEvent::CertificateImport(
                CertificateImportEvents::CertificateValidationFailed(event),
            )],
            new_state,
        });
    }

    // Validation succeeded
    let metadata = validation_result
        .metadata
        .clone()
        .ok_or_else(|| CertificateImportError::ValidationFailed("No metadata extracted".to_string()))?;

    let event = CertificateValidationSucceededEvent {
        cert_id: cmd.cert_id,
        yubikey_serial: yubikey_serial.clone(),
        slot,
        metadata: metadata.clone(),
        validated_at: now,
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    };

    let new_state = CertificateImportState::Validated {
        cert_id: cmd.cert_id,
        yubikey_serial,
        slot,
        metadata,
        validated_at: now,
    };

    Ok(CommandResult {
        events: vec![DomainEvent::CertificateImport(
            CertificateImportEvents::CertificateValidationSucceeded(event),
        )],
        new_state,
    })
}

/// Handle request for PIN entry
pub fn handle_request_pin(
    current_state: &CertificateImportState,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
) -> Result<CommandResult, CertificateImportError> {
    // Validate state allows PIN request
    if !current_state.can_request_pin() {
        return Err(CertificateImportError::InvalidTransition {
            current: current_state.state_name().to_string(),
            target: "AwaitingPin".to_string(),
            reason: "Cannot request PIN in current state".to_string(),
        });
    }

    let (cert_id, yubikey_serial, slot, metadata) = match current_state {
        CertificateImportState::Validated {
            cert_id,
            yubikey_serial,
            slot,
            metadata,
            ..
        } => (*cert_id, yubikey_serial.clone(), *slot, metadata.clone()),
        _ => {
            return Err(CertificateImportError::InvalidTransition {
                current: current_state.state_name().to_string(),
                target: "AwaitingPin".to_string(),
                reason: "Missing validation data".to_string(),
            })
        }
    };

    let now = Utc::now();
    const INITIAL_PIN_ATTEMPTS: u8 = 3;

    let event = PinEntryRequestedEvent {
        cert_id,
        yubikey_serial: yubikey_serial.clone(),
        slot,
        attempts_remaining: INITIAL_PIN_ATTEMPTS,
        requested_at: now,
        correlation_id,
        causation_id,
    };

    let new_state = CertificateImportState::AwaitingPin {
        cert_id,
        yubikey_serial,
        slot,
        metadata,
        attempts_remaining: INITIAL_PIN_ATTEMPTS,
    };

    Ok(CommandResult {
        events: vec![DomainEvent::CertificateImport(
            CertificateImportEvents::PinEntryRequested(event),
        )],
        new_state,
    })
}

/// Handle PIN verification result
pub fn handle_pin_result(
    current_state: &CertificateImportState,
    pin_valid: bool,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
) -> Result<CommandResult, CertificateImportError> {
    let (cert_id, yubikey_serial, slot, metadata, attempts_remaining) = match current_state {
        CertificateImportState::AwaitingPin {
            cert_id,
            yubikey_serial,
            slot,
            metadata,
            attempts_remaining,
        } => (
            *cert_id,
            yubikey_serial.clone(),
            *slot,
            metadata.clone(),
            *attempts_remaining,
        ),
        CertificateImportState::PinFailed {
            cert_id,
            yubikey_serial,
            slot,
            attempts_remaining,
            ..
        } => {
            // When retrying from PinFailed, we need to get metadata from somewhere
            // For now, we'll just transition without metadata check
            return Err(CertificateImportError::InvalidTransition {
                current: current_state.state_name().to_string(),
                target: "Importing".to_string(),
                reason: "Cannot verify PIN after failure without retrying from AwaitingPin".to_string(),
            });
        }
        _ => {
            return Err(CertificateImportError::InvalidTransition {
                current: current_state.state_name().to_string(),
                target: "Importing".to_string(),
                reason: "Cannot verify PIN in current state".to_string(),
            })
        }
    };

    let now = Utc::now();

    if pin_valid {
        let event = PinVerifiedEvent {
            cert_id,
            yubikey_serial: yubikey_serial.clone(),
            slot,
            verified_at: now,
            correlation_id,
            causation_id,
        };

        let new_state = CertificateImportState::Importing {
            cert_id,
            yubikey_serial,
            slot,
            started_at: now,
        };

        Ok(CommandResult {
            events: vec![DomainEvent::CertificateImport(
                CertificateImportEvents::PinVerified(event),
            )],
            new_state,
        })
    } else {
        let new_attempts = attempts_remaining.saturating_sub(1);

        if new_attempts == 0 {
            return Err(CertificateImportError::PinLocked);
        }

        let event = PinEntryFailedEvent {
            cert_id,
            yubikey_serial: yubikey_serial.clone(),
            slot,
            attempts_remaining: new_attempts,
            reason: "Invalid PIN".to_string(),
            failed_at: now,
            correlation_id,
            causation_id,
        };

        let new_state = CertificateImportState::PinFailed {
            cert_id,
            yubikey_serial,
            slot,
            attempts_remaining: new_attempts,
            failed_at: now,
        };

        Ok(CommandResult {
            events: vec![DomainEvent::CertificateImport(
                CertificateImportEvents::PinEntryFailed(event),
            )],
            new_state,
        })
    }
}

/// Handle certificate import result
pub fn handle_import_result(
    current_state: &CertificateImportState,
    import_success: bool,
    error_reason: Option<String>,
    metadata: CertificateMetadata,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
) -> Result<CommandResult, CertificateImportError> {
    let (cert_id, yubikey_serial, slot) = match current_state {
        CertificateImportState::Importing {
            cert_id,
            yubikey_serial,
            slot,
            ..
        } => (*cert_id, yubikey_serial.clone(), *slot),
        _ => {
            return Err(CertificateImportError::InvalidTransition {
                current: current_state.state_name().to_string(),
                target: "Imported".to_string(),
                reason: "Cannot record import result in current state".to_string(),
            })
        }
    };

    let now = Utc::now();

    if import_success {
        let event = CertificateImportSucceededEvent {
            cert_id,
            yubikey_serial: yubikey_serial.clone(),
            slot,
            metadata,
            imported_at: now,
            correlation_id,
            causation_id,
        };

        let new_state = CertificateImportState::Imported {
            cert_id,
            yubikey_serial,
            slot,
            imported_at: now,
        };

        Ok(CommandResult {
            events: vec![DomainEvent::CertificateImport(
                CertificateImportEvents::CertificateImportSucceeded(event),
            )],
            new_state,
        })
    } else {
        let reason = error_reason.unwrap_or_else(|| "Unknown error".to_string());

        let event = CertificateImportFailedEvent {
            cert_id,
            yubikey_serial: yubikey_serial.clone(),
            slot,
            reason: reason.clone(),
            failed_at: now,
            correlation_id,
            causation_id,
        };

        let new_state = CertificateImportState::ImportFailed {
            cert_id,
            yubikey_serial,
            slot,
            reason,
            failed_at: now,
        };

        Ok(CommandResult {
            events: vec![DomainEvent::CertificateImport(
                CertificateImportEvents::CertificateImportFailed(event),
            )],
            new_state,
        })
    }
}

/// Handle abort command
pub fn handle_abort(
    cmd: AbortImport,
    current_state: &CertificateImportState,
) -> Result<CommandResult, CertificateImportError> {
    if !current_state.can_abort() {
        return Err(CertificateImportError::InvalidTransition {
            current: current_state.state_name().to_string(),
            target: "NoCertificateSelected".to_string(),
            reason: "Cannot abort in current state".to_string(),
        });
    }

    let now = Utc::now();

    let event = CertificateImportAbortedEvent {
        cert_id: current_state.cert_id(),
        yubikey_serial: current_state.yubikey_serial().map(|s| s.to_string()),
        reason: cmd.reason,
        aborted_at: now,
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    };

    Ok(CommandResult {
        events: vec![DomainEvent::CertificateImport(
            CertificateImportEvents::CertificateImportAborted(event),
        )],
        new_state: CertificateImportState::NoCertificateSelected,
    })
}

/// Handle workflow reset command
pub fn handle_reset(
    cmd: ResetWorkflow,
    current_state: &CertificateImportState,
) -> Result<CommandResult, CertificateImportError> {
    let now = Utc::now();

    let event = WorkflowResetEvent {
        previous_cert_id: current_state.cert_id(),
        reset_at: now,
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    };

    Ok(CommandResult {
        events: vec![DomainEvent::CertificateImport(
            CertificateImportEvents::WorkflowReset(event),
        )],
        new_state: CertificateImportState::NoCertificateSelected,
    })
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
    fn test_select_certificate_from_initial() {
        let state = CertificateImportState::NoCertificateSelected;
        let cmd = SelectCertificateForImport {
            cert_id: Uuid::now_v7(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_select_certificate(cmd, &state).unwrap();
        assert_eq!(result.events.len(), 1);
        assert!(matches!(
            result.new_state,
            CertificateImportState::CertificateSelected { .. }
        ));
    }

    #[test]
    fn test_cannot_select_while_importing() {
        let state = CertificateImportState::Importing {
            cert_id: Uuid::now_v7(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            started_at: Utc::now(),
        };

        let cmd = SelectCertificateForImport {
            cert_id: Uuid::now_v7(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Signature,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_select_certificate(cmd, &state);
        assert!(result.is_err());
    }

    #[test]
    fn test_abort_workflow() {
        let state = CertificateImportState::Validated {
            cert_id: Uuid::now_v7(),
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            metadata: create_test_metadata(),
            validated_at: Utc::now(),
        };

        let cmd = AbortImport {
            reason: "User cancelled".to_string(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_abort(cmd, &state).unwrap();
        assert!(matches!(
            result.new_state,
            CertificateImportState::NoCertificateSelected
        ));
    }
}
