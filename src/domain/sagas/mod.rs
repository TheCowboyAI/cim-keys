// Copyright (c) 2025 - Cowboy AI, LLC.

//! Saga Patterns for Cross-Aggregate Operations
//!
//! Sagas coordinate operations that span multiple aggregate boundaries.
//! Each saga is modeled as a state machine with well-defined transitions.
//!
//! ## Saga Pattern
//!
//! Following DDD principles, sagas are "aggregates of aggregates" that:
//! - Coordinate multi-aggregate business processes
//! - Use compensation actions for failure recovery
//! - Maintain eventual consistency across boundaries
//! - Track progress via state machine transitions
//!
//! ## Available Sagas
//!
//! - **CompleteBootstrapSaga**: Full CIM infrastructure bootstrap
//! - **PersonOnboardingSaga**: Person + Keys + NATS User + YubiKey
//! - **CertificateProvisioningSaga**: Key + Certificate + YubiKey slot
//!
//! ## State Machine Pattern
//!
//! Each saga follows a Markov chain pattern:
//! ```text
//! Initial → Step1 → Step2 → ... → Completed
//!     ↓         ↓        ↓
//!   Failed ← Failed ← Failed
//!     ↓         ↓        ↓
//! Compensating...
//! ```

pub mod bootstrap;
pub mod person_onboarding;
pub mod certificate_provisioning;

pub use bootstrap::*;
pub use person_onboarding::*;
pub use certificate_provisioning::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Common saga state trait
pub trait SagaState: Clone + Send + Sync {
    /// Get the saga's unique ID
    fn saga_id(&self) -> Uuid;

    /// Get the correlation ID for all events in this saga
    fn correlation_id(&self) -> Uuid;

    /// Check if the saga is in a terminal state
    fn is_terminal(&self) -> bool;

    /// Check if the saga completed successfully
    fn is_completed(&self) -> bool;

    /// Check if the saga failed
    fn is_failed(&self) -> bool;

    /// Get a human-readable status description
    fn status_description(&self) -> String;
}

/// Saga execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SagaResult<T> {
    /// Saga completed successfully
    Completed(T),
    /// Saga is still in progress (return current state)
    InProgress,
    /// Saga failed with error
    Failed(SagaError),
    /// Saga is compensating (rolling back)
    Compensating(CompensationProgress),
}

/// Saga error with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaError {
    /// Error message
    pub message: String,
    /// The step where the error occurred
    pub failed_step: String,
    /// Timestamp of failure
    pub failed_at: DateTime<Utc>,
    /// Whether compensation has been attempted
    pub compensation_attempted: bool,
    /// Compensation result
    pub compensation_result: Option<CompensationResult>,
}

impl SagaError {
    pub fn new(message: impl Into<String>, step: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            failed_step: step.into(),
            failed_at: Utc::now(),
            compensation_attempted: false,
            compensation_result: None,
        }
    }

    pub fn with_compensation(mut self, result: CompensationResult) -> Self {
        self.compensation_attempted = true;
        self.compensation_result = Some(result);
        self
    }
}

/// Progress of compensation actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationProgress {
    /// Steps that need compensation
    pub steps_to_compensate: Vec<String>,
    /// Steps already compensated
    pub steps_compensated: Vec<String>,
    /// Current compensation step
    pub current_step: Option<String>,
}

/// Result of compensation attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompensationResult {
    /// All compensation actions completed
    FullyCompensated,
    /// Partial compensation (some steps could not be rolled back)
    PartiallyCompensated { failed_steps: Vec<String> },
    /// Compensation not needed (no side effects to roll back)
    NotNeeded,
}

/// Events emitted by sagas for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SagaEvent {
    /// Saga started
    SagaStarted {
        saga_id: Uuid,
        saga_type: String,
        correlation_id: Uuid,
        started_at: DateTime<Utc>,
    },
    /// Saga step started
    StepStarted {
        saga_id: Uuid,
        step_name: String,
        started_at: DateTime<Utc>,
    },
    /// Saga step completed
    StepCompleted {
        saga_id: Uuid,
        step_name: String,
        completed_at: DateTime<Utc>,
    },
    /// Saga step failed
    StepFailed {
        saga_id: Uuid,
        step_name: String,
        error: String,
        failed_at: DateTime<Utc>,
    },
    /// Saga completed successfully
    SagaCompleted {
        saga_id: Uuid,
        completed_at: DateTime<Utc>,
    },
    /// Saga failed
    SagaFailed {
        saga_id: Uuid,
        error: SagaError,
        failed_at: DateTime<Utc>,
    },
    /// Compensation started
    CompensationStarted {
        saga_id: Uuid,
        started_at: DateTime<Utc>,
    },
    /// Compensation completed
    CompensationCompleted {
        saga_id: Uuid,
        result: CompensationResult,
        completed_at: DateTime<Utc>,
    },
}

/// Trait for saga coordinators that manage saga execution
pub trait SagaCoordinator<S: SagaState> {
    /// The output type when saga completes
    type Output;

    /// Execute the next step of the saga
    fn execute_next_step(&self, state: &mut S) -> SagaResult<Self::Output>;

    /// Compensate (rollback) the saga
    fn compensate(&self, state: &mut S) -> CompensationResult;

    /// Get saga events for the current execution
    fn get_saga_events(&self, state: &S) -> Vec<SagaEvent>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saga_error_creation() {
        let error = SagaError::new("Test error", "step1");
        assert_eq!(error.message, "Test error");
        assert_eq!(error.failed_step, "step1");
        assert!(!error.compensation_attempted);
    }

    #[test]
    fn test_saga_error_with_compensation() {
        let error = SagaError::new("Test error", "step1")
            .with_compensation(CompensationResult::FullyCompensated);
        assert!(error.compensation_attempted);
        assert!(matches!(error.compensation_result, Some(CompensationResult::FullyCompensated)));
    }
}
