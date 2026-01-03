// Copyright (c) 2025 - Cowboy AI, LLC.

//! Saga Lifecycle Events
//!
//! Events related to saga execution and lifecycle management.
//! Sagas coordinate multi-aggregate operations and these events
//! provide observability into saga progress and outcomes.
//!
//! ## Event Categories
//!
//! - **Lifecycle**: SagaStarted, SagaCompleted, SagaFailed
//! - **Progress**: StepStarted, StepCompleted, StepFailed
//! - **Compensation**: CompensationStarted, CompensationCompleted
//! - **Recovery**: SagaResumed, SagaRecovered

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Events for saga execution lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum SagaEvents {
    /// A saga was started
    SagaStarted(SagaStartedEvent),

    /// A saga step was started
    StepStarted(StepStartedEvent),

    /// A saga step completed successfully
    StepCompleted(StepCompletedEvent),

    /// A saga step failed
    StepFailed(StepFailedEvent),

    /// Saga completed successfully
    SagaCompleted(SagaCompletedEvent),

    /// Saga failed
    SagaFailed(SagaFailedEvent),

    /// Compensation was started
    CompensationStarted(CompensationStartedEvent),

    /// A compensation step completed
    CompensationStepCompleted(CompensationStepCompletedEvent),

    /// Compensation completed
    CompensationCompleted(CompensationCompletedEvent),

    /// Saga was resumed from persisted state
    SagaResumed(SagaResumedEvent),

    /// Saga was recovered after failure
    SagaRecovered(SagaRecoveredEvent),
}

/// A saga was started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStartedEvent {
    /// Unique saga ID
    pub saga_id: Uuid,
    /// Type of saga (e.g., "certificate_provisioning")
    pub saga_type: String,
    /// Correlation ID for tracking related events
    pub correlation_id: Uuid,
    /// ID of the command that triggered this saga
    pub triggered_by_command_id: Option<Uuid>,
    /// Who/what initiated the saga
    pub initiated_by: String,
    /// When the saga started
    pub started_at: DateTime<Utc>,
    /// Initial saga context (JSON-serializable)
    pub context: Option<String>,
}

/// A saga step was started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStartedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Step name (e.g., "GeneratingKey")
    pub step_name: String,
    /// Step sequence number
    pub step_number: u32,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// Previous step that caused this step
    pub causation_id: Option<Uuid>,
    /// When the step started
    pub started_at: DateTime<Utc>,
}

/// A saga step completed successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCompletedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Step name
    pub step_name: String,
    /// Step sequence number
    pub step_number: u32,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// The step start event that this completes
    pub causation_id: Uuid,
    /// When the step completed
    pub completed_at: DateTime<Utc>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Artifacts produced by this step (JSON)
    pub artifacts: Option<String>,
}

/// A saga step failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepFailedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Step name
    pub step_name: String,
    /// Step sequence number
    pub step_number: u32,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// The step start event that this fails
    pub causation_id: Uuid,
    /// When the step failed
    pub failed_at: DateTime<Utc>,
    /// Error message
    pub error_message: String,
    /// Error code (if applicable)
    pub error_code: Option<String>,
    /// Whether compensation is needed
    pub requires_compensation: bool,
    /// Retry count before failure
    pub retry_count: u32,
}

/// Saga completed successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaCompletedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Saga type
    pub saga_type: String,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// The saga start event
    pub causation_id: Uuid,
    /// When the saga completed
    pub completed_at: DateTime<Utc>,
    /// Total duration in milliseconds
    pub total_duration_ms: u64,
    /// Number of steps executed
    pub steps_executed: u32,
    /// Final result (JSON-serializable)
    pub result: Option<String>,
}

/// Saga failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaFailedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Saga type
    pub saga_type: String,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// The step failure or start event that caused this
    pub causation_id: Uuid,
    /// When the saga failed
    pub failed_at: DateTime<Utc>,
    /// Step where failure occurred
    pub failed_at_step: String,
    /// Error message
    pub error_message: String,
    /// Whether compensation was attempted
    pub compensation_attempted: bool,
    /// Compensation result if attempted
    pub compensation_result: Option<String>,
}

/// Compensation was started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationStartedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// The failure event that triggered compensation
    pub causation_id: Uuid,
    /// When compensation started
    pub started_at: DateTime<Utc>,
    /// Steps to compensate
    pub steps_to_compensate: Vec<String>,
}

/// A compensation step completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationStepCompletedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Step being compensated
    pub step_name: String,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// The compensation start event
    pub causation_id: Uuid,
    /// When the compensation step completed
    pub completed_at: DateTime<Utc>,
    /// Whether the compensation succeeded
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Compensation completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationCompletedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// The compensation start event
    pub causation_id: Uuid,
    /// When compensation completed
    pub completed_at: DateTime<Utc>,
    /// Compensation outcome
    pub outcome: CompensationOutcome,
    /// Steps that were successfully compensated
    pub compensated_steps: Vec<String>,
    /// Steps that failed compensation
    pub failed_steps: Vec<String>,
}

/// Outcome of compensation attempt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompensationOutcome {
    /// All steps successfully compensated
    FullyCompensated,
    /// Some steps failed compensation
    PartiallyCompensated,
    /// No compensation was needed
    NotNeeded,
    /// Compensation failed completely
    Failed,
}

/// Saga was resumed from persisted state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaResumedEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Saga type
    pub saga_type: String,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// When resumed
    pub resumed_at: DateTime<Utc>,
    /// State version when resumed
    pub state_version: u64,
    /// Step to resume from
    pub resume_from_step: String,
    /// Reason for resume (restart, manual, recovery)
    pub resume_reason: String,
}

/// Saga was recovered after failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaRecoveredEvent {
    /// Saga ID
    pub saga_id: Uuid,
    /// Saga type
    pub saga_type: String,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// When recovered
    pub recovered_at: DateTime<Utc>,
    /// How long the saga was stalled (milliseconds)
    pub stalled_duration_ms: u64,
    /// Recovery action taken
    pub recovery_action: RecoveryAction,
}

/// Action taken during saga recovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Saga was resumed from last checkpoint
    Resumed,
    /// Saga was compensated and marked as failed
    Compensated,
    /// Saga was marked as failed without compensation
    MarkedFailed,
    /// Saga was retried from beginning
    Retried,
}

impl SagaEvents {
    /// Get the event type name for NATS subject routing
    pub fn event_type(&self) -> &'static str {
        match self {
            SagaEvents::SagaStarted(_) => "saga.started",
            SagaEvents::StepStarted(_) => "saga.step.started",
            SagaEvents::StepCompleted(_) => "saga.step.completed",
            SagaEvents::StepFailed(_) => "saga.step.failed",
            SagaEvents::SagaCompleted(_) => "saga.completed",
            SagaEvents::SagaFailed(_) => "saga.failed",
            SagaEvents::CompensationStarted(_) => "saga.compensation.started",
            SagaEvents::CompensationStepCompleted(_) => "saga.compensation.step.completed",
            SagaEvents::CompensationCompleted(_) => "saga.compensation.completed",
            SagaEvents::SagaResumed(_) => "saga.resumed",
            SagaEvents::SagaRecovered(_) => "saga.recovered",
        }
    }

    /// Get the saga ID from any event
    pub fn saga_id(&self) -> Uuid {
        match self {
            SagaEvents::SagaStarted(e) => e.saga_id,
            SagaEvents::StepStarted(e) => e.saga_id,
            SagaEvents::StepCompleted(e) => e.saga_id,
            SagaEvents::StepFailed(e) => e.saga_id,
            SagaEvents::SagaCompleted(e) => e.saga_id,
            SagaEvents::SagaFailed(e) => e.saga_id,
            SagaEvents::CompensationStarted(e) => e.saga_id,
            SagaEvents::CompensationStepCompleted(e) => e.saga_id,
            SagaEvents::CompensationCompleted(e) => e.saga_id,
            SagaEvents::SagaResumed(e) => e.saga_id,
            SagaEvents::SagaRecovered(e) => e.saga_id,
        }
    }

    /// Get the correlation ID from any event
    pub fn correlation_id(&self) -> Uuid {
        match self {
            SagaEvents::SagaStarted(e) => e.correlation_id,
            SagaEvents::StepStarted(e) => e.correlation_id,
            SagaEvents::StepCompleted(e) => e.correlation_id,
            SagaEvents::StepFailed(e) => e.correlation_id,
            SagaEvents::SagaCompleted(e) => e.correlation_id,
            SagaEvents::SagaFailed(e) => e.correlation_id,
            SagaEvents::CompensationStarted(e) => e.correlation_id,
            SagaEvents::CompensationStepCompleted(e) => e.correlation_id,
            SagaEvents::CompensationCompleted(e) => e.correlation_id,
            SagaEvents::SagaResumed(e) => e.correlation_id,
            SagaEvents::SagaRecovered(e) => e.correlation_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saga_started_event() {
        let event = SagaStartedEvent {
            saga_id: Uuid::now_v7(),
            saga_type: "certificate_provisioning".to_string(),
            correlation_id: Uuid::now_v7(),
            triggered_by_command_id: Some(Uuid::now_v7()),
            initiated_by: "user:alice".to_string(),
            started_at: Utc::now(),
            context: Some(r#"{"purpose": "authentication"}"#.to_string()),
        };

        let saga_event = SagaEvents::SagaStarted(event.clone());
        assert_eq!(saga_event.event_type(), "saga.started");
        assert_eq!(saga_event.saga_id(), event.saga_id);
    }

    #[test]
    fn test_step_completed_event() {
        let event = StepCompletedEvent {
            saga_id: Uuid::now_v7(),
            step_name: "GeneratingKey".to_string(),
            step_number: 1,
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            completed_at: Utc::now(),
            duration_ms: 150,
            artifacts: Some(r#"{"key_id": "abc123"}"#.to_string()),
        };

        let saga_event = SagaEvents::StepCompleted(event.clone());
        assert_eq!(saga_event.event_type(), "saga.step.completed");
    }

    #[test]
    fn test_saga_failed_event() {
        let event = SagaFailedEvent {
            saga_id: Uuid::now_v7(),
            saga_type: "certificate_provisioning".to_string(),
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            failed_at: Utc::now(),
            failed_at_step: "ProvisioningToYubiKey".to_string(),
            error_message: "YubiKey not connected".to_string(),
            compensation_attempted: true,
            compensation_result: Some("FullyCompensated".to_string()),
        };

        let saga_event = SagaEvents::SagaFailed(event.clone());
        assert_eq!(saga_event.event_type(), "saga.failed");
    }

    #[test]
    fn test_compensation_outcome_serialization() {
        let outcome = CompensationOutcome::PartiallyCompensated;
        let json = serde_json::to_string(&outcome).unwrap();
        assert_eq!(json, r#""PartiallyCompensated""#);

        let parsed: CompensationOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CompensationOutcome::PartiallyCompensated);
    }

    #[test]
    fn test_recovery_action_serialization() {
        let action = RecoveryAction::Compensated;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, r#""Compensated""#);
    }

    #[test]
    fn test_saga_event_json_roundtrip() {
        let event = SagaEvents::SagaCompleted(SagaCompletedEvent {
            saga_id: Uuid::now_v7(),
            saga_type: "test_saga".to_string(),
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            completed_at: Utc::now(),
            total_duration_ms: 5000,
            steps_executed: 4,
            result: Some(r#"{"success": true}"#.to_string()),
        });

        let json = serde_json::to_string(&event).unwrap();
        let parsed: SagaEvents = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.event_type(), "saga.completed");
    }

    #[test]
    fn test_all_saga_event_types() {
        // Verify all event variants have the correct event_type()
        let test_cases = [
            ("saga.started", SagaEvents::SagaStarted(SagaStartedEvent {
                saga_id: Uuid::now_v7(),
                saga_type: "test".to_string(),
                correlation_id: Uuid::now_v7(),
                triggered_by_command_id: None,
                initiated_by: "test".to_string(),
                started_at: Utc::now(),
                context: None,
            })),
            ("saga.step.started", SagaEvents::StepStarted(StepStartedEvent {
                saga_id: Uuid::now_v7(),
                step_name: "test_step".to_string(),
                step_number: 1,
                correlation_id: Uuid::now_v7(),
                causation_id: None,
                started_at: Utc::now(),
            })),
            ("saga.step.completed", SagaEvents::StepCompleted(StepCompletedEvent {
                saga_id: Uuid::now_v7(),
                step_name: "test_step".to_string(),
                step_number: 1,
                correlation_id: Uuid::now_v7(),
                causation_id: Uuid::now_v7(),
                completed_at: Utc::now(),
                duration_ms: 100,
                artifacts: None,
            })),
            ("saga.step.failed", SagaEvents::StepFailed(StepFailedEvent {
                saga_id: Uuid::now_v7(),
                step_name: "test_step".to_string(),
                step_number: 1,
                correlation_id: Uuid::now_v7(),
                causation_id: Uuid::now_v7(),
                failed_at: Utc::now(),
                error_message: "test error".to_string(),
                error_code: None,
                requires_compensation: false,
                retry_count: 0,
            })),
            ("saga.completed", SagaEvents::SagaCompleted(SagaCompletedEvent {
                saga_id: Uuid::now_v7(),
                saga_type: "test".to_string(),
                correlation_id: Uuid::now_v7(),
                causation_id: Uuid::now_v7(),
                completed_at: Utc::now(),
                total_duration_ms: 1000,
                steps_executed: 3,
                result: None,
            })),
            ("saga.failed", SagaEvents::SagaFailed(SagaFailedEvent {
                saga_id: Uuid::now_v7(),
                saga_type: "test".to_string(),
                correlation_id: Uuid::now_v7(),
                causation_id: Uuid::now_v7(),
                failed_at: Utc::now(),
                failed_at_step: "step_2".to_string(),
                error_message: "test failure".to_string(),
                compensation_attempted: false,
                compensation_result: None,
            })),
            ("saga.compensation.started", SagaEvents::CompensationStarted(CompensationStartedEvent {
                saga_id: Uuid::now_v7(),
                correlation_id: Uuid::now_v7(),
                causation_id: Uuid::now_v7(),
                started_at: Utc::now(),
                steps_to_compensate: vec!["step_1".to_string()],
            })),
            ("saga.compensation.step.completed", SagaEvents::CompensationStepCompleted(CompensationStepCompletedEvent {
                saga_id: Uuid::now_v7(),
                step_name: "step_1".to_string(),
                correlation_id: Uuid::now_v7(),
                causation_id: Uuid::now_v7(),
                completed_at: Utc::now(),
                success: true,
                error_message: None,
            })),
            ("saga.compensation.completed", SagaEvents::CompensationCompleted(CompensationCompletedEvent {
                saga_id: Uuid::now_v7(),
                correlation_id: Uuid::now_v7(),
                causation_id: Uuid::now_v7(),
                completed_at: Utc::now(),
                outcome: CompensationOutcome::FullyCompensated,
                compensated_steps: vec!["step_1".to_string()],
                failed_steps: vec![],
            })),
            ("saga.resumed", SagaEvents::SagaResumed(SagaResumedEvent {
                saga_id: Uuid::now_v7(),
                saga_type: "test".to_string(),
                correlation_id: Uuid::now_v7(),
                resumed_at: Utc::now(),
                state_version: 1,
                resume_from_step: "step_2".to_string(),
                resume_reason: "restart".to_string(),
            })),
            ("saga.recovered", SagaEvents::SagaRecovered(SagaRecoveredEvent {
                saga_id: Uuid::now_v7(),
                saga_type: "test".to_string(),
                correlation_id: Uuid::now_v7(),
                recovered_at: Utc::now(),
                stalled_duration_ms: 5000,
                recovery_action: RecoveryAction::Resumed,
            })),
        ];

        for (expected_type, event) in test_cases {
            assert_eq!(event.event_type(), expected_type, "Event type mismatch");
        }
    }

    #[test]
    fn test_saga_id_extraction() {
        let saga_id = Uuid::now_v7();

        let event = SagaEvents::StepStarted(StepStartedEvent {
            saga_id,
            step_name: "test".to_string(),
            step_number: 1,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            started_at: Utc::now(),
        });

        assert_eq!(event.saga_id(), saga_id);
    }

    #[test]
    fn test_correlation_id_extraction() {
        let correlation_id = Uuid::now_v7();

        let event = SagaEvents::StepCompleted(StepCompletedEvent {
            saga_id: Uuid::now_v7(),
            step_name: "test".to_string(),
            step_number: 1,
            correlation_id,
            causation_id: Uuid::now_v7(),
            completed_at: Utc::now(),
            duration_ms: 100,
            artifacts: None,
        });

        assert_eq!(event.correlation_id(), correlation_id);
    }

    #[test]
    fn test_compensation_outcome_all_variants() {
        let outcomes = [
            (CompensationOutcome::FullyCompensated, "\"FullyCompensated\""),
            (CompensationOutcome::PartiallyCompensated, "\"PartiallyCompensated\""),
            (CompensationOutcome::NotNeeded, "\"NotNeeded\""),
            (CompensationOutcome::Failed, "\"Failed\""),
        ];

        for (outcome, expected_json) in outcomes {
            let json = serde_json::to_string(&outcome).unwrap();
            assert_eq!(json, expected_json);
        }
    }

    #[test]
    fn test_recovery_action_all_variants() {
        let actions = [
            (RecoveryAction::Resumed, "\"Resumed\""),
            (RecoveryAction::Compensated, "\"Compensated\""),
            (RecoveryAction::MarkedFailed, "\"MarkedFailed\""),
            (RecoveryAction::Retried, "\"Retried\""),
        ];

        for (action, expected_json) in actions {
            let json = serde_json::to_string(&action).unwrap();
            assert_eq!(json, expected_json);
        }
    }

    #[test]
    fn test_step_failed_event_serialization() {
        let event = StepFailedEvent {
            saga_id: Uuid::now_v7(),
            step_name: "GeneratingCertificate".to_string(),
            step_number: 2,
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            failed_at: Utc::now(),
            error_message: "Certificate generation failed: Invalid CA".to_string(),
            error_code: Some("CERT_001".to_string()),
            requires_compensation: true,
            retry_count: 3,
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: StepFailedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.step_name, "GeneratingCertificate");
        assert_eq!(parsed.error_code, Some("CERT_001".to_string()));
        assert!(parsed.requires_compensation);
        assert_eq!(parsed.retry_count, 3);
    }

    #[test]
    fn test_domain_event_saga_variant() {
        use super::super::DomainEvent;

        let saga_event = SagaEvents::SagaStarted(SagaStartedEvent {
            saga_id: Uuid::now_v7(),
            saga_type: "certificate_provisioning".to_string(),
            correlation_id: Uuid::now_v7(),
            triggered_by_command_id: None,
            initiated_by: "test".to_string(),
            started_at: Utc::now(),
            context: None,
        });

        let domain_event = DomainEvent::Saga(saga_event);

        // Test serialization roundtrip
        let json = serde_json::to_string(&domain_event).unwrap();
        let parsed: DomainEvent = serde_json::from_str(&json).unwrap();

        match parsed {
            DomainEvent::Saga(evt) => {
                assert_eq!(evt.event_type(), "saga.started");
            }
            _ => panic!("Expected Saga variant"),
        }
    }

    #[test]
    fn test_event_envelope_with_saga_event() {
        use super::super::{DomainEvent, EventEnvelope};

        let saga_event = SagaEvents::SagaCompleted(SagaCompletedEvent {
            saga_id: Uuid::now_v7(),
            saga_type: "key_provisioning".to_string(),
            correlation_id: Uuid::now_v7(),
            causation_id: Uuid::now_v7(),
            completed_at: Utc::now(),
            total_duration_ms: 2500,
            steps_executed: 4,
            result: Some(r#"{"key_id": "abc-123"}"#.to_string()),
        });

        let domain_event = DomainEvent::Saga(saga_event);
        let correlation_id = Uuid::now_v7();

        let envelope = EventEnvelope::new(domain_event, correlation_id, None);

        assert_eq!(envelope.aggregate_type(), "Saga");
        assert_eq!(envelope.correlation_id, correlation_id);
        // Default subject should contain saga
        assert!(envelope.nats_subject.contains("saga"),
            "Subject should contain 'saga': {}", envelope.nats_subject);
    }
}
