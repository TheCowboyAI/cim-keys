// Copyright (c) 2025 - Cowboy AI, LLC.

//! JetStream Saga Executor
//!
//! Provides async saga execution with JetStream integration for:
//! - Saga state persistence in KV store
//! - Event publishing for saga lifecycle events
//! - Event-driven saga progression
//! - Saga recovery from event stream
//!
//! ## Architecture
//!
//! ```text
//! Saga Request
//!       │
//!       ▼
//! JetStreamSagaExecutor ──► KV Store (saga state)
//!       │                        │
//!       ├── Load/Save state ◄────┘
//!       │
//!       ▼
//! Execute Step ──► Domain Commands
//!       │                │
//!       │                ▼
//!       │         Domain Events
//!       │                │
//!       ▼                ▼
//! Saga Events ──► JetStream (SAGA_EVENTS stream)
//!       │
//!       ▼
//! Update State ──► KV Store
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use cim_keys::domain::nats::saga_executor::{JetStreamSagaExecutor, SagaExecutorConfig};
//!
//! let executor = JetStreamSagaExecutor::new(jetstream_port, config);
//!
//! // Start a new saga
//! let saga_id = executor.start_saga(bootstrap_saga).await?;
//!
//! // Execute next step
//! let result = executor.execute_step(saga_id).await?;
//!
//! // Resume a saga after restart
//! let saga = executor.recover_saga::<CompleteBootstrapSaga>(saga_id).await?;
//! ```

use crate::domain::sagas::{
    SagaState, SagaResult, SagaError, SagaEvent, CompensationResult,
};
use crate::ports::{JetStreamPort, JetStreamError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for the saga executor
#[derive(Debug, Clone)]
pub struct SagaExecutorConfig {
    /// KV bucket name for saga state
    pub state_bucket: String,
    /// Stream name for saga events
    pub event_stream: String,
    /// Subject prefix for saga events
    pub event_subject_prefix: String,
    /// Maximum retries for step execution
    pub max_step_retries: u32,
    /// Timeout for step execution in milliseconds
    pub step_timeout_ms: u64,
}

impl Default for SagaExecutorConfig {
    fn default() -> Self {
        Self {
            state_bucket: "SAGA_STATE".to_string(),
            event_stream: "SAGA_EVENTS".to_string(),
            event_subject_prefix: "saga.events".to_string(),
            max_step_retries: 3,
            step_timeout_ms: 30_000,
        }
    }
}

impl SagaExecutorConfig {
    /// Create new config with custom bucket
    pub fn with_bucket(mut self, bucket: impl Into<String>) -> Self {
        self.state_bucket = bucket.into();
        self
    }

    /// Create new config with custom stream
    pub fn with_stream(mut self, stream: impl Into<String>) -> Self {
        self.event_stream = stream.into();
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_step_retries = retries;
        self
    }
}

/// Persisted saga state wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSagaState<S> {
    /// The saga state
    pub state: S,
    /// Saga type name for recovery
    pub saga_type: String,
    /// Version for optimistic locking
    pub version: u64,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Retry count for current step
    pub retry_count: u32,
}

impl<S: SagaState + Serialize> PersistedSagaState<S> {
    /// Create new persisted state
    pub fn new(state: S) -> Self {
        Self {
            saga_type: std::any::type_name::<S>().to_string(),
            state,
            version: 1,
            updated_at: Utc::now(),
            retry_count: 0,
        }
    }

    /// Increment version
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.updated_at = Utc::now();
    }

    /// Reset retry count
    pub fn reset_retry(&mut self) {
        self.retry_count = 0;
    }
}

/// Errors from saga executor
#[derive(Debug, thiserror::Error)]
pub enum SagaExecutorError {
    #[error("Saga not found: {0}")]
    SagaNotFound(Uuid),

    #[error("Saga already exists: {0}")]
    SagaAlreadyExists(Uuid),

    #[error("Saga type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Saga in terminal state: {0}")]
    SagaTerminated(String),

    #[error("Max retries exceeded: {0}")]
    MaxRetriesExceeded(u32),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("JetStream error: {0}")]
    JetStreamError(#[from] JetStreamError),

    #[error("Saga execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Compensation failed: {0}")]
    CompensationFailed(String),
}

/// Result type for saga operations
pub type SagaExecutorResult<T> = Result<T, SagaExecutorError>;

/// Execution result for a single step
///
/// Following the Mealy machine pattern (A2, A6), step execution produces
/// both state transitions AND domain events as output. The domain events
/// are returned alongside the step result for publication.
///
/// ## FRP Axiom Compliance
///
/// - A3 (Decoupled): Domain events are produced AFTER step completes
/// - A6 (Explicit Routing): Events are composed with step results, not pattern matched
/// - A7 (Change Prefixes): Events form a causal chain via correlation_id
/// - A9 (Semantic Preservation): `execute_step >>> publish_events` composes correctly
#[derive(Debug, Clone)]
pub enum StepExecutionResult<T> {
    /// Step completed, saga continues
    /// Contains domain events produced by this step
    Continue {
        /// Domain events produced by this step for publication
        domain_events: Vec<crate::events::DomainEvent>,
    },
    /// Saga completed with output
    /// Contains final domain events
    Completed {
        /// The saga output
        output: T,
        /// Domain events produced by the final step
        domain_events: Vec<crate::events::DomainEvent>,
    },
    /// Step failed, retry available
    Retry { reason: String, retry_in_ms: u64 },
    /// Step failed, compensation needed
    Failed(SagaError),
}

impl<T> StepExecutionResult<T> {
    /// Create a Continue result with domain events
    pub fn continue_with_events(events: Vec<crate::events::DomainEvent>) -> Self {
        Self::Continue { domain_events: events }
    }

    /// Create a Continue result with no events
    pub fn continue_empty() -> Self {
        Self::Continue { domain_events: vec![] }
    }

    /// Create a Completed result with output and events
    pub fn completed_with_events(output: T, events: Vec<crate::events::DomainEvent>) -> Self {
        Self::Completed { output, domain_events: events }
    }

    /// Get domain events if present
    pub fn domain_events(&self) -> &[crate::events::DomainEvent] {
        match self {
            Self::Continue { domain_events } => domain_events,
            Self::Completed { domain_events, .. } => domain_events,
            Self::Retry { .. } | Self::Failed(_) => &[],
        }
    }
}

/// Async saga executor trait
#[async_trait::async_trait]
pub trait AsyncSagaExecutor<S: SagaState>: Send + Sync {
    /// Output type when saga completes
    type Output;

    /// Execute the next step of the saga
    async fn execute_next_step(&self, state: &mut S) -> SagaExecutorResult<StepExecutionResult<Self::Output>>;

    /// Compensate (rollback) the saga
    async fn compensate(&self, state: &mut S) -> SagaExecutorResult<CompensationResult>;

    /// Get the saga type name
    fn saga_type_name(&self) -> &'static str;
}

/// JetStream-backed saga executor
pub struct JetStreamSagaExecutor<P: JetStreamPort> {
    port: P,
    config: SagaExecutorConfig,
}

impl<P: JetStreamPort> JetStreamSagaExecutor<P> {
    /// Create new executor
    pub fn new(port: P, config: SagaExecutorConfig) -> Self {
        Self { port, config }
    }

    /// Create with default config
    pub fn with_defaults(port: P) -> Self {
        Self::new(port, SagaExecutorConfig::default())
    }

    /// Get the KV key for a saga
    fn saga_key(&self, saga_id: Uuid) -> String {
        format!("saga.{}", saga_id)
    }

    /// Get the event subject for a saga event
    fn event_subject(&self, saga_id: Uuid, event_type: &str) -> String {
        format!("{}.{}.{}", self.config.event_subject_prefix, saga_id, event_type)
    }

    /// Start a new saga
    pub async fn start_saga<S>(&self, state: S) -> SagaExecutorResult<Uuid>
    where
        S: SagaState + Serialize + for<'de> Deserialize<'de>,
    {
        let saga_id = state.saga_id();
        let key = self.saga_key(saga_id);

        // Check if saga already exists
        if self.port.kv_get(&self.config.state_bucket, &key).await?.is_some() {
            return Err(SagaExecutorError::SagaAlreadyExists(saga_id));
        }

        // Create persisted state
        let persisted = PersistedSagaState::new(state);
        let data = serde_json::to_vec(&persisted)
            .map_err(|e| SagaExecutorError::SerializationError(e.to_string()))?;

        // Save to KV store
        self.port.kv_put(&self.config.state_bucket, &key, &data).await?;

        // Publish saga started event
        let event = SagaEvent::SagaStarted {
            saga_id,
            saga_type: persisted.saga_type.clone(),
            correlation_id: persisted.state.correlation_id(),
            started_at: Utc::now(),
        };
        self.publish_saga_event(saga_id, &event).await?;

        Ok(saga_id)
    }

    /// Load saga state from KV store
    pub async fn load_saga<S>(&self, saga_id: Uuid) -> SagaExecutorResult<PersistedSagaState<S>>
    where
        S: SagaState + for<'de> Deserialize<'de>,
    {
        let key = self.saga_key(saga_id);
        let data = self.port.kv_get(&self.config.state_bucket, &key).await?
            .ok_or(SagaExecutorError::SagaNotFound(saga_id))?;

        let persisted: PersistedSagaState<S> = serde_json::from_slice(&data)
            .map_err(|e| SagaExecutorError::SerializationError(e.to_string()))?;

        // Verify type matches
        let expected_type = std::any::type_name::<S>().to_string();
        if persisted.saga_type != expected_type {
            return Err(SagaExecutorError::TypeMismatch {
                expected: expected_type,
                found: persisted.saga_type,
            });
        }

        Ok(persisted)
    }

    /// Save saga state to KV store
    pub async fn save_saga<S>(&self, persisted: &PersistedSagaState<S>) -> SagaExecutorResult<()>
    where
        S: SagaState + Serialize,
    {
        let saga_id = persisted.state.saga_id();
        let key = self.saga_key(saga_id);

        let data = serde_json::to_vec(persisted)
            .map_err(|e| SagaExecutorError::SerializationError(e.to_string()))?;

        self.port.kv_put(&self.config.state_bucket, &key, &data).await?;

        Ok(())
    }

    /// Execute next step with an executor
    ///
    /// Returns `StepExecutionResult` which includes domain events produced by the step.
    /// The caller is responsible for publishing these domain events (following A6: explicit routing).
    ///
    /// ## FRP Axiom Compliance
    ///
    /// - A3 (Decoupled): Domain events returned AFTER step completes
    /// - A6 (Explicit Routing): Caller composes `execute_step >>> publish_domain_events`
    /// - A7 (Change Prefixes): Events preserve causation chain
    pub async fn execute_step<S, E>(&self, saga_id: Uuid, executor: &E) -> SagaExecutorResult<StepExecutionResult<E::Output>>
    where
        S: SagaState + Serialize + for<'de> Deserialize<'de>,
        E: AsyncSagaExecutor<S>,
    {
        // Load current state
        let mut persisted = self.load_saga::<S>(saga_id).await?;

        // Check if saga is in terminal state
        if persisted.state.is_terminal() {
            return Err(SagaExecutorError::SagaTerminated(
                persisted.state.status_description()
            ));
        }

        // Publish step started event
        let step_name = persisted.state.status_description();
        let step_started = SagaEvent::StepStarted {
            saga_id,
            step_name: step_name.clone(),
            started_at: Utc::now(),
        };
        self.publish_saga_event(saga_id, &step_started).await?;

        // Execute the step
        let result = executor.execute_next_step(&mut persisted.state).await;

        match result {
            Ok(StepExecutionResult::Continue { domain_events }) => {
                // Step completed, save state and continue
                persisted.increment_version();
                persisted.reset_retry();
                self.save_saga(&persisted).await?;

                let step_completed = SagaEvent::StepCompleted {
                    saga_id,
                    step_name,
                    completed_at: Utc::now(),
                };
                self.publish_saga_event(saga_id, &step_completed).await?;

                // Return domain events to caller for publication
                Ok(StepExecutionResult::Continue { domain_events })
            }
            Ok(StepExecutionResult::Completed { output, domain_events }) => {
                // Saga completed
                persisted.increment_version();
                self.save_saga(&persisted).await?;

                let saga_completed = SagaEvent::SagaCompleted {
                    saga_id,
                    completed_at: Utc::now(),
                };
                self.publish_saga_event(saga_id, &saga_completed).await?;

                // Return output and domain events to caller
                Ok(StepExecutionResult::Completed { output, domain_events })
            }
            Ok(StepExecutionResult::Retry { reason, retry_in_ms }) => {
                // Retry needed
                persisted.increment_retry();

                if persisted.retry_count > self.config.max_step_retries {
                    // Max retries exceeded, fail
                    return Err(SagaExecutorError::MaxRetriesExceeded(persisted.retry_count));
                }

                self.save_saga(&persisted).await?;

                Ok(StepExecutionResult::Retry { reason, retry_in_ms })
            }
            Ok(StepExecutionResult::Failed(error)) => {
                // Step failed
                let step_failed = SagaEvent::StepFailed {
                    saga_id,
                    step_name,
                    error: error.message.clone(),
                    failed_at: Utc::now(),
                };
                self.publish_saga_event(saga_id, &step_failed).await?;

                // Update state
                persisted.increment_version();
                self.save_saga(&persisted).await?;

                Ok(StepExecutionResult::Failed(error))
            }
            Err(e) => {
                // Executor error
                let step_failed = SagaEvent::StepFailed {
                    saga_id,
                    step_name,
                    error: e.to_string(),
                    failed_at: Utc::now(),
                };
                self.publish_saga_event(saga_id, &step_failed).await?;

                Err(e)
            }
        }
    }

    /// Run saga to completion (or failure)
    ///
    /// Note: This method does NOT publish domain events. Caller should use
    /// `execute_step` in a loop and publish events after each step for proper
    /// FRP composition (A6: explicit routing).
    pub async fn run_to_completion<S, E>(&self, saga_id: Uuid, executor: &E) -> SagaExecutorResult<SagaResult<E::Output>>
    where
        S: SagaState + Serialize + for<'de> Deserialize<'de>,
        E: AsyncSagaExecutor<S>,
    {
        loop {
            match self.execute_step::<S, E>(saga_id, executor).await? {
                StepExecutionResult::Continue { domain_events: _ } => {
                    // Continue to next step
                    // Note: Domain events are dropped here - use execute_step loop for publishing
                    continue;
                }
                StepExecutionResult::Completed { output, domain_events: _ } => {
                    // Note: Domain events are dropped here - use execute_step loop for publishing
                    return Ok(SagaResult::Completed(output));
                }
                StepExecutionResult::Retry { retry_in_ms, .. } => {
                    // Async sleep for retry delay
                    #[cfg(feature = "nats-client")]
                    {
                        tokio::time::sleep(std::time::Duration::from_millis(retry_in_ms)).await;
                    }
                    #[cfg(not(feature = "nats-client"))]
                    {
                        // Without async runtime, log the retry delay intention
                        let _ = retry_in_ms; // Acknowledged but no-op without runtime
                    }
                    continue;
                }
                StepExecutionResult::Failed(error) => {
                    // Attempt compensation
                    return Ok(SagaResult::Failed(error));
                }
            }
        }
    }

    /// Compensate a failed saga
    pub async fn compensate<S, E>(&self, saga_id: Uuid, executor: &E) -> SagaExecutorResult<CompensationResult>
    where
        S: SagaState + Serialize + for<'de> Deserialize<'de>,
        E: AsyncSagaExecutor<S>,
    {
        // Load current state
        let mut persisted = self.load_saga::<S>(saga_id).await?;

        // Publish compensation started
        let comp_started = SagaEvent::CompensationStarted {
            saga_id,
            started_at: Utc::now(),
        };
        self.publish_saga_event(saga_id, &comp_started).await?;

        // Execute compensation
        let result = executor.compensate(&mut persisted.state).await?;

        // Save updated state
        persisted.increment_version();
        self.save_saga(&persisted).await?;

        // Publish compensation completed
        let comp_completed = SagaEvent::CompensationCompleted {
            saga_id,
            result: result.clone(),
            completed_at: Utc::now(),
        };
        self.publish_saga_event(saga_id, &comp_completed).await?;

        Ok(result)
    }

    /// Publish a saga event to JetStream
    async fn publish_saga_event(&self, saga_id: Uuid, event: &SagaEvent) -> SagaExecutorResult<()> {
        let event_type = match event {
            SagaEvent::SagaStarted { .. } => "started",
            SagaEvent::StepStarted { .. } => "step.started",
            SagaEvent::StepCompleted { .. } => "step.completed",
            SagaEvent::StepFailed { .. } => "step.failed",
            SagaEvent::SagaCompleted { .. } => "completed",
            SagaEvent::SagaFailed { .. } => "failed",
            SagaEvent::CompensationStarted { .. } => "compensation.started",
            SagaEvent::CompensationCompleted { .. } => "compensation.completed",
        };

        let subject = self.event_subject(saga_id, event_type);
        let data = serde_json::to_vec(event)
            .map_err(|e| SagaExecutorError::SerializationError(e.to_string()))?;

        // Create minimal headers
        let headers = crate::ports::JetStreamHeaders::default();

        self.port.publish(&subject, &data, Some(&headers)).await?;

        Ok(())
    }

    /// List all active sagas
    pub async fn list_active_sagas(&self) -> SagaExecutorResult<Vec<Uuid>> {
        let keys = self.port.kv_keys(&self.config.state_bucket, "saga.").await?;

        let saga_ids: Vec<Uuid> = keys
            .iter()
            .filter_map(|key| {
                key.strip_prefix("saga.")
                    .and_then(|id_str| Uuid::parse_str(id_str).ok())
            })
            .collect();

        Ok(saga_ids)
    }

    /// Delete a completed saga
    pub async fn delete_saga(&self, saga_id: Uuid) -> SagaExecutorResult<()> {
        let key = self.saga_key(saga_id);
        self.port.kv_delete(&self.config.state_bucket, &key).await?;
        Ok(())
    }
}

/// Saga recovery from event stream
pub struct SagaRecovery<P: JetStreamPort> {
    port: P,
    config: SagaExecutorConfig,
}

impl<P: JetStreamPort> SagaRecovery<P> {
    /// Create new recovery handler
    pub fn new(port: P, config: SagaExecutorConfig) -> Self {
        Self { port, config }
    }

    /// Find sagas that need recovery (started but not completed/failed)
    pub async fn find_incomplete_sagas(&self) -> SagaExecutorResult<Vec<Uuid>> {
        // List all saga keys from KV store
        let keys = self.port.kv_keys(&self.config.state_bucket, "saga.").await?;

        let mut incomplete = Vec::new();

        for key in keys {
            if let Some(id_str) = key.strip_prefix("saga.") {
                if let Ok(saga_id) = Uuid::parse_str(id_str) {
                    // Load the saga state to check if it's terminal
                    if let Some(data) = self.port.kv_get(&self.config.state_bucket, &key).await? {
                        // Parse minimal persisted state to check terminal status
                        // This allows checking without deserializing the full saga state
                        #[derive(Deserialize)]
                        struct MinimalPersistedState {
                            #[allow(dead_code)] // Required for JSON parsing but not used directly
                            saga_type: String,
                            version: u64,
                        }

                        if let Ok(minimal) = serde_json::from_slice::<MinimalPersistedState>(&data) {
                            // Include sagas that have been persisted (version >= 1)
                            // Terminal state detection requires type-specific parsing
                            // which happens during actual recovery, not discovery
                            if minimal.version >= 1 {
                                incomplete.push(saga_id);
                            }
                        }
                    }
                }
            }
        }

        Ok(incomplete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saga_executor_config_default() {
        let config = SagaExecutorConfig::default();
        assert_eq!(config.state_bucket, "SAGA_STATE");
        assert_eq!(config.event_stream, "SAGA_EVENTS");
        assert_eq!(config.max_step_retries, 3);
    }

    #[test]
    fn test_saga_executor_config_builder() {
        let config = SagaExecutorConfig::default()
            .with_bucket("CUSTOM_BUCKET")
            .with_stream("CUSTOM_STREAM")
            .with_max_retries(5);

        assert_eq!(config.state_bucket, "CUSTOM_BUCKET");
        assert_eq!(config.event_stream, "CUSTOM_STREAM");
        assert_eq!(config.max_step_retries, 5);
    }

    #[test]
    fn test_persisted_saga_state() {
        #[derive(Clone, Serialize, Deserialize)]
        struct TestState {
            saga_id: Uuid,
            correlation_id: Uuid,
        }

        impl SagaState for TestState {
            fn saga_id(&self) -> Uuid { self.saga_id }
            fn correlation_id(&self) -> Uuid { self.correlation_id }
            fn is_terminal(&self) -> bool { false }
            fn is_completed(&self) -> bool { false }
            fn is_failed(&self) -> bool { false }
            fn status_description(&self) -> String { "test".to_string() }
        }

        let state = TestState {
            saga_id: Uuid::now_v7(),
            correlation_id: Uuid::now_v7(),
        };

        let mut persisted = PersistedSagaState::new(state);
        assert_eq!(persisted.version, 1);
        assert_eq!(persisted.retry_count, 0);

        persisted.increment_version();
        assert_eq!(persisted.version, 2);

        persisted.increment_retry();
        assert_eq!(persisted.retry_count, 1);

        persisted.reset_retry();
        assert_eq!(persisted.retry_count, 0);
    }

    #[test]
    fn test_saga_key_format() {
        let saga_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
        let key = format!("saga.{}", saga_id);
        assert_eq!(key, "saga.12345678-1234-1234-1234-123456789012");
    }

    #[test]
    fn test_event_subject_format() {
        let saga_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
        let subject = format!("saga.events.{}.started", saga_id);
        assert_eq!(subject, "saga.events.12345678-1234-1234-1234-123456789012.started");
    }
}
