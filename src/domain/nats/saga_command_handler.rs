// Copyright (c) 2025 - Cowboy AI, LLC.

//! Saga Command Handler
//!
//! Bridges the command processing pipeline with saga execution.
//! Commands that trigger multi-aggregate workflows are routed through
//! sagas for coordination and compensation support.
//!
//! ## Architecture
//!
//! ```text
//! Command → SagaCommandHandler → JetStreamSagaExecutor
//!                    ↓                    ↓
//!              Create Saga         Persist State
//!                    ↓                    ↓
//!              Execute Steps       Publish Events
//!                    ↓                    ↓
//!              Return Result       Update KV
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use cim_keys::domain::nats::saga_command_handler::SagaCommandHandler;
//!
//! let handler = SagaCommandHandler::new(jetstream_port, publisher);
//!
//! // Handle a saga-triggering command
//! let result = handler.handle_provision_certificate(request).await?;
//! ```

use crate::domain::sagas::{
    CertificateProvisioningSaga, ProvisioningRequest, ProvisioningState,
    SagaState, SagaError, CompensationResult, CertificatePurpose,
};
use crate::value_objects::ActorId;
use crate::domain::nats::saga_executor::{
    JetStreamSagaExecutor, SagaExecutorConfig, SagaExecutorError,
    AsyncSagaExecutor, StepExecutionResult, PersistedSagaState,
};
use crate::domain::nats::publisher::{EventPublisher, EventPublishError};
use crate::ports::JetStreamPort;

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Saga command handler for orchestrating multi-aggregate workflows
///
/// ## FRP Architecture (Axiom A6: Explicit Routing)
///
/// The handler composes saga execution with domain event publishing:
///
/// ```text
/// execute_step >>> publish_domain_events >>> publish_saga_events
/// ```
///
/// This follows the categorical composition pattern where:
/// - Saga executor produces domain events as step outputs
/// - Handler publishes events after each successful step
/// - Causality chain is preserved via correlation_id
pub struct SagaCommandHandler<P: JetStreamPort> {
    executor: JetStreamSagaExecutor<P>,
    /// Publisher for domain events (KeyGenerated, CertificateIssued, etc.)
    publisher: EventPublisher<P>,
    config: SagaHandlerConfig,
}

/// Configuration for the saga command handler
#[derive(Debug, Clone)]
pub struct SagaHandlerConfig {
    /// Maximum retries per step
    pub max_step_retries: u32,
    /// Whether to auto-compensate on failure
    pub auto_compensate: bool,
    /// Timeout for saga execution in seconds
    pub saga_timeout_secs: u64,
}

impl Default for SagaHandlerConfig {
    fn default() -> Self {
        Self {
            max_step_retries: 3,
            auto_compensate: true,
            saga_timeout_secs: 300, // 5 minutes
        }
    }
}

/// Result of handling a saga command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaCommandResult<T> {
    /// The saga ID
    pub saga_id: Uuid,
    /// Correlation ID for tracking
    pub correlation_id: Uuid,
    /// Final status
    pub status: SagaCommandStatus,
    /// Result data (if completed successfully)
    pub result: Option<T>,
    /// Error information (if failed)
    pub error: Option<String>,
}

/// Status of saga command execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SagaCommandStatus {
    /// Saga completed successfully
    Completed,
    /// Saga failed and was compensated
    FailedAndCompensated,
    /// Saga failed, compensation also failed
    FailedCompensationFailed,
    /// Saga is still in progress (async execution)
    InProgress,
    /// Saga was already running
    AlreadyRunning,
}

impl<P: JetStreamPort + Clone> SagaCommandHandler<P> {
    /// Create a new saga command handler
    pub fn new(port: P, config: SagaHandlerConfig) -> Self {
        let executor_config = SagaExecutorConfig::default()
            .with_bucket("saga-states")
            .with_stream("SAGA_EVENTS")
            .with_max_retries(config.max_step_retries);

        Self {
            executor: JetStreamSagaExecutor::new(port.clone(), executor_config),
            publisher: EventPublisher::new(port),
            config,
        }
    }

    /// Create with custom executor config
    pub fn with_executor_config(port: P, handler_config: SagaHandlerConfig, executor_config: SagaExecutorConfig) -> Self {
        Self {
            executor: JetStreamSagaExecutor::new(port.clone(), executor_config),
            publisher: EventPublisher::new(port),
            config: handler_config,
        }
    }

    /// Publish domain events produced by a saga step
    ///
    /// ## FRP Axiom Compliance
    ///
    /// - A6 (Explicit Routing): Composed with execute_step via `>>>`
    /// - A7 (Change Prefixes): Events form causal chain via correlation_id
    async fn publish_domain_events(
        &self,
        events: &[crate::events::DomainEvent],
        correlation_id: Uuid,
    ) -> Result<(), SagaCommandHandlerError> {
        if events.is_empty() {
            return Ok(());
        }

        self.publisher
            .publish_batch(events, correlation_id)
            .await
            .map_err(|e| SagaCommandHandlerError::EventPublishFailed(e.to_string()))?;

        Ok(())
    }

    /// Handle a certificate provisioning command
    ///
    /// This command triggers the CertificateProvisioningSaga which:
    /// 1. Generates a key pair
    /// 2. Generates a certificate
    /// 3. Provisions to YubiKey
    /// 4. Verifies provisioning
    pub async fn handle_provision_certificate(
        &self,
        request: ProvisioningRequest,
    ) -> Result<SagaCommandResult<CertificateProvisioningResult>, SagaCommandHandlerError> {
        // Create the saga
        let mut saga = CertificateProvisioningSaga::new(request);
        let saga_id = saga.saga_id();
        let correlation_id = saga.correlation_id();

        // Start the saga
        saga.start().map_err(|e| SagaCommandHandlerError::SagaStartFailed(e.message))?;

        // Persist initial state
        self.executor
            .start_saga(saga.clone())
            .await
            .map_err(|e| SagaCommandHandlerError::PersistenceFailed(e.to_string()))?;

        // Create executor for this saga type
        let saga_executor = CertificateProvisioningExecutor::new();

        // Execute the saga to completion
        let result = self.execute_saga_to_completion(
            &mut saga,
            &saga_executor,
        ).await;

        match result {
            Ok(output) => Ok(SagaCommandResult {
                saga_id,
                correlation_id,
                status: SagaCommandStatus::Completed,
                result: Some(output),
                error: None,
            }),
            Err(SagaCommandHandlerError::SagaStepFailed { step, message }) => {
                // Attempt compensation if configured
                if self.config.auto_compensate {
                    let comp_result = self.compensate_saga(&mut saga, &saga_executor).await;
                    let status = match comp_result {
                        Ok(CompensationResult::FullyCompensated) => SagaCommandStatus::FailedAndCompensated,
                        Ok(CompensationResult::NotNeeded) => SagaCommandStatus::FailedAndCompensated,
                        _ => SagaCommandStatus::FailedCompensationFailed,
                    };
                    Ok(SagaCommandResult {
                        saga_id,
                        correlation_id,
                        status,
                        result: None,
                        error: Some(format!("Step '{}' failed: {}", step, message)),
                    })
                } else {
                    Ok(SagaCommandResult {
                        saga_id,
                        correlation_id,
                        status: SagaCommandStatus::FailedCompensationFailed,
                        result: None,
                        error: Some(format!("Step '{}' failed: {}", step, message)),
                    })
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Execute a saga to completion, handling retries and publishing domain events
    ///
    /// ## FRP Axiom Compliance
    ///
    /// - A3 (Decoupled): Domain events published AFTER step completes
    /// - A6 (Explicit Routing): Composition: `execute_step >>> publish_domain_events`
    /// - A7 (Change Prefixes): All events share correlation_id for causal chain
    /// - A9 (Semantic Preservation): Event publishing preserves saga semantics
    async fn execute_saga_to_completion<S, E>(
        &self,
        saga: &mut S,
        executor: &E,
    ) -> Result<E::Output, SagaCommandHandlerError>
    where
        S: SagaState + Serialize + for<'de> Deserialize<'de>,
        E: AsyncSagaExecutor<S>,
    {
        let mut retry_count = 0;
        let correlation_id = saga.correlation_id();

        loop {
            if saga.is_terminal() {
                if saga.is_completed() {
                    // Return a default output - the actual result is in the saga state
                    break;
                } else {
                    return Err(SagaCommandHandlerError::SagaStepFailed {
                        step: "terminal".to_string(),
                        message: "Saga reached failed terminal state".to_string(),
                    });
                }
            }

            match executor.execute_next_step(saga).await {
                Ok(StepExecutionResult::Completed { output, domain_events }) => {
                    // Publish domain events produced by final step (A6: explicit routing)
                    self.publish_domain_events(&domain_events, correlation_id).await?;

                    // Update persisted state
                    let persisted = PersistedSagaState::new(saga.clone());
                    self.executor
                        .save_saga(&persisted)
                        .await
                        .map_err(|e| SagaCommandHandlerError::PersistenceFailed(e.to_string()))?;
                    return Ok(output);
                }
                Ok(StepExecutionResult::Continue { domain_events }) => {
                    // Publish domain events produced by this step (A6: explicit routing)
                    self.publish_domain_events(&domain_events, correlation_id).await?;

                    // Save progress and continue
                    let persisted = PersistedSagaState::new(saga.clone());
                    self.executor
                        .save_saga(&persisted)
                        .await
                        .map_err(|e| SagaCommandHandlerError::PersistenceFailed(e.to_string()))?;
                    retry_count = 0;
                }
                Ok(StepExecutionResult::Retry { retry_in_ms, reason }) => {
                    retry_count += 1;
                    if retry_count > self.config.max_step_retries {
                        return Err(SagaCommandHandlerError::SagaStepFailed {
                            step: saga.status_description(),
                            message: format!("Max retries exceeded: {}", reason),
                        });
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry_in_ms)).await;
                }
                Ok(StepExecutionResult::Failed(err)) => {
                    return Err(SagaCommandHandlerError::SagaStepFailed {
                        step: saga.status_description(),
                        message: err.message,
                    });
                }
                Err(e) => {
                    return Err(SagaCommandHandlerError::ExecutorError(e.to_string()));
                }
            }
        }

        // If we exit the loop, saga completed
        Err(SagaCommandHandlerError::SagaStepFailed {
            step: "completion".to_string(),
            message: "Saga did not produce output".to_string(),
        })
    }

    /// Compensate a failed saga
    async fn compensate_saga<S, E>(
        &self,
        saga: &mut S,
        executor: &E,
    ) -> Result<CompensationResult, SagaCommandHandlerError>
    where
        S: SagaState + Serialize + for<'de> Deserialize<'de>,
        E: AsyncSagaExecutor<S>,
    {
        match executor.compensate(saga).await {
            Ok(result) => {
                // Save final state
                let persisted = PersistedSagaState::new(saga.clone());
                self.executor
                    .save_saga(&persisted)
                    .await
                    .map_err(|e| SagaCommandHandlerError::PersistenceFailed(e.to_string()))?;
                Ok(result)
            }
            Err(e) => Err(SagaCommandHandlerError::CompensationFailed(e.to_string())),
        }
    }

    /// Resume a saga from persisted state
    pub async fn resume_saga<S>(
        &self,
        saga_id: Uuid,
    ) -> Result<PersistedSagaState<S>, SagaCommandHandlerError>
    where
        S: SagaState + for<'de> Deserialize<'de>,
    {
        self.executor
            .load_saga(saga_id)
            .await
            .map_err(|e| SagaCommandHandlerError::PersistenceFailed(e.to_string()))
    }

    /// List active sagas
    pub async fn list_active_sagas(&self) -> Result<Vec<Uuid>, SagaCommandHandlerError> {
        self.executor
            .list_active_sagas()
            .await
            .map_err(|e| SagaCommandHandlerError::PersistenceFailed(e.to_string()))
    }
}

/// Result of certificate provisioning saga
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateProvisioningResult {
    /// Generated key ID
    pub key_id: Uuid,
    /// Generated certificate ID
    pub certificate_id: Uuid,
    /// YubiKey slot where provisioned
    pub slot: String,
    /// Certificate fingerprint
    pub fingerprint: String,
}

/// Executor for the CertificateProvisioningSaga
pub struct CertificateProvisioningExecutor {
    _phantom: PhantomData<()>,
}

impl CertificateProvisioningExecutor {
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl Default for CertificateProvisioningExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AsyncSagaExecutor<CertificateProvisioningSaga> for CertificateProvisioningExecutor {
    type Output = CertificateProvisioningResult;

    /// Execute the next step of the saga, returning domain events
    ///
    /// ## FRP Axiom Compliance
    ///
    /// - A3 (Decoupled): Events created AFTER operation completes
    /// - A6 (Explicit Routing): Caller composes step execution with event publishing
    /// - A7 (Change Prefixes): Events use correlation_id from saga
    async fn execute_next_step(
        &self,
        state: &mut CertificateProvisioningSaga,
    ) -> Result<StepExecutionResult<Self::Output>, SagaExecutorError> {
        use crate::events::{DomainEvent, KeyEvents, CertificateEvents, YubiKeyEvents};
        use crate::events::key::KeyGeneratedEvent;
        use crate::events::certificate::CertificateGeneratedEvent;
        use crate::events::yubikey::YubiKeyProvisionedEvent;
        use crate::types::{KeyMetadata, YubiKeySlot};
        use chrono::Utc;

        let correlation_id = state.correlation_id();

        match &state.state {
            ProvisioningState::Initial => {
                // Already started, advance to key generation
                state.advance();
                Ok(StepExecutionResult::continue_empty())
            }
            ProvisioningState::GeneratingKey => {
                // In a real implementation, this would call the key generation port
                // For now, simulate success
                let key_id = crate::domain::ids::KeyId::new();
                state.record_key(key_id);
                state.advance();

                // Emit KeyGenerated domain event (A7: event log) with typed ActorId
                #[allow(deprecated)]
                let event = DomainEvent::Key(KeyEvents::KeyGenerated(KeyGeneratedEvent {
                    key_id: key_id.as_uuid(),
                    algorithm: state.request.key_algorithm.clone(),
                    purpose: state.request.purpose.to_key_purpose(),
                    generated_at: Utc::now(),
                    generated_by: state.request.person_email.clone(), // Legacy field for backward compat
                    generated_by_actor: Some(ActorId::legacy(&state.request.person_email)),
                    hardware_backed: true, // YubiKey-backed
                    metadata: KeyMetadata {
                        label: format!("{} - {:?}", state.request.person_name, state.request.purpose),
                        description: Some(format!("Generated for saga {}", state.saga_id())),
                        tags: vec!["saga-generated".to_string()],
                        attributes: std::collections::HashMap::new(),
                        jwt_kid: None,
                        jwt_alg: None,
                        jwt_use: None,
                    },
                    ownership: None,
                    correlation_id,
                    causation_id: Some(state.saga_id()), // Saga is the cause
                }));

                Ok(StepExecutionResult::continue_with_events(vec![event]))
            }
            ProvisioningState::GeneratingCertificate => {
                // In a real implementation, this would call the certificate generation port
                let cert_id = crate::domain::ids::CertificateId::new();
                let fingerprint = format!("SHA256:{}", Uuid::now_v7());
                state.record_certificate(cert_id, fingerprint.clone());
                state.advance();

                // Emit CertificateGenerated domain event
                let key_id = state.artifacts.key_id.expect("Key should exist after GeneratingKey step");
                #[allow(deprecated)]
                let cert_gen_event = CertificateGeneratedEvent::new_legacy(
                    cert_id.as_uuid(),
                    key_id.as_uuid(),
                    format!("CN={}, O=CIM, OU={:?}", state.request.person_name, state.request.purpose),
                    Some(state.request.issuing_ca_id.as_uuid()),
                    Utc::now(),
                    Utc::now() + chrono::Duration::days(state.request.validity_days as i64),
                    false, // is_ca
                    vec![state.request.person_email.clone()],
                    vec!["digitalSignature".to_string()],
                    match state.request.purpose {
                        CertificatePurpose::Authentication => vec!["clientAuth".to_string()],
                        CertificatePurpose::DigitalSignature => vec!["codeSigning".to_string()],
                        CertificatePurpose::KeyManagement => vec!["emailProtection".to_string(), "keyAgreement".to_string()],
                        CertificatePurpose::CardAuthentication => vec!["smartcardLogon".to_string()],
                    },
                    correlation_id,
                    Some(state.saga_id()),
                );
                let event = DomainEvent::Certificate(CertificateEvents::CertificateGenerated(cert_gen_event));

                Ok(StepExecutionResult::continue_with_events(vec![event]))
            }
            ProvisioningState::ProvisioningToYubiKey => {
                // In a real implementation, this would call the YubiKey port
                let slot_str = format!("{:?}", state.request.target_slot);
                state.record_provisioning(slot_str.clone());
                state.advance();

                // Emit YubiKeyProvisioned domain event
                let key_id = state.artifacts.key_id.expect("Key should exist after GeneratingKey step");
                let event = DomainEvent::YubiKey(YubiKeyEvents::YubiKeyProvisioned(YubiKeyProvisionedEvent {
                    event_id: Uuid::now_v7(),
                    yubikey_serial: state.request.yubikey_serial.clone(),
                    slots_configured: vec![YubiKeySlot {
                        slot_id: slot_str,
                        key_id: key_id.as_uuid(),
                        purpose: state.request.purpose.to_key_purpose(),
                    }],
                    provisioned_at: Utc::now(),
                    provisioned_by: state.request.person_email.clone(),
                    correlation_id,
                    causation_id: Some(state.saga_id()),
                }));

                Ok(StepExecutionResult::continue_with_events(vec![event]))
            }
            ProvisioningState::VerifyingProvisioning => {
                // In a real implementation, this would verify the certificate is in the slot
                state.record_verification(crate::domain::sagas::VerificationStatus::Verified);
                state.advance();

                // Build result
                let result = CertificateProvisioningResult {
                    key_id: state.artifacts.key_id.map(|k| k.as_uuid()).unwrap_or_default(),
                    certificate_id: state.artifacts.certificate_id.map(|c| c.as_uuid()).unwrap_or_default(),
                    slot: state.artifacts.provisioned_slot.clone().unwrap_or_default(),
                    fingerprint: state.artifacts.certificate_fingerprint.clone().unwrap_or_default(),
                };

                // Verification complete - no additional domain event needed
                // (The saga completion event from the executor covers this)
                Ok(StepExecutionResult::completed_with_events(result, vec![]))
            }
            ProvisioningState::Completed => {
                // Already complete
                let result = CertificateProvisioningResult {
                    key_id: state.artifacts.key_id.map(|k| k.as_uuid()).unwrap_or_default(),
                    certificate_id: state.artifacts.certificate_id.map(|c| c.as_uuid()).unwrap_or_default(),
                    slot: state.artifacts.provisioned_slot.clone().unwrap_or_default(),
                    fingerprint: state.artifacts.certificate_fingerprint.clone().unwrap_or_default(),
                };
                Ok(StepExecutionResult::completed_with_events(result, vec![]))
            }
            ProvisioningState::Failed => {
                let err = state.error.clone().unwrap_or_else(|| SagaError::new("Unknown error", "unknown"));
                Ok(StepExecutionResult::Failed(err))
            }
            ProvisioningState::Compensating(_) => {
                // In compensating state, treat as failed
                let err = state.error.clone().unwrap_or_else(|| SagaError::new("Compensating", "compensation"));
                Ok(StepExecutionResult::Failed(err))
            }
        }
    }

    async fn compensate(
        &self,
        state: &mut CertificateProvisioningSaga,
    ) -> Result<CompensationResult, SagaExecutorError> {
        use crate::domain::sagas::ProvisioningCompensationStep;

        // Start compensation
        let mut current_step = state.start_compensation();

        loop {
            match current_step {
                ProvisioningCompensationStep::ClearYubiKeySlot => {
                    // In a real implementation, clear the YubiKey slot
                    // For now, simulate success
                }
                ProvisioningCompensationStep::RevokeCertificate => {
                    // In a real implementation, revoke the certificate
                }
                ProvisioningCompensationStep::RevokeKey => {
                    // In a real implementation, revoke the key
                }
            }

            match state.advance_compensation() {
                Some(next) => current_step = next,
                None => break,
            }
        }

        Ok(CompensationResult::FullyCompensated)
    }

    fn saga_type_name(&self) -> &'static str {
        "certificate_provisioning"
    }
}

/// Errors that can occur during saga command handling
#[derive(Debug, thiserror::Error)]
pub enum SagaCommandHandlerError {
    #[error("Saga start failed: {0}")]
    SagaStartFailed(String),

    #[error("Saga step failed at '{step}': {message}")]
    SagaStepFailed { step: String, message: String },

    #[error("Persistence failed: {0}")]
    PersistenceFailed(String),

    #[error("Executor error: {0}")]
    ExecutorError(String),

    #[error("Compensation failed: {0}")]
    CompensationFailed(String),

    #[error("Event publishing failed: {0}")]
    EventPublishFailed(String),

    #[error("Saga not found: {0}")]
    SagaNotFound(Uuid),

    #[error("Invalid saga state: {0}")]
    InvalidState(String),
}

impl From<SagaExecutorError> for SagaCommandHandlerError {
    fn from(err: SagaExecutorError) -> Self {
        SagaCommandHandlerError::ExecutorError(err.to_string())
    }
}

impl From<EventPublishError> for SagaCommandHandlerError {
    fn from(err: EventPublishError) -> Self {
        SagaCommandHandlerError::EventPublishFailed(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::sagas::{CertificatePurpose, ProvisioningRequest};
    use crate::domain::ids::*;
    use crate::domain::yubikey::PIVSlot;
    use crate::events::KeyAlgorithm;
    use crate::ports::{JetStreamHeaders, JetStreamError, PublishAck};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock JetStream port for testing
    struct MockJetStreamPort {
        kv_store: Arc<Mutex<std::collections::HashMap<String, Vec<u8>>>>,
    }

    impl MockJetStreamPort {
        fn new() -> Self {
            Self {
                kv_store: Arc::new(Mutex::new(std::collections::HashMap::new())),
            }
        }
    }

    impl Clone for MockJetStreamPort {
        fn clone(&self) -> Self {
            Self {
                kv_store: Arc::clone(&self.kv_store),
            }
        }
    }

    #[async_trait::async_trait]
    impl JetStreamPort for MockJetStreamPort {
        async fn publish(
            &self,
            _subject: &str,
            _payload: &[u8],
            _headers: Option<&JetStreamHeaders>,
        ) -> Result<PublishAck, JetStreamError> {
            Ok(PublishAck {
                stream: "TEST".to_string(),
                sequence: 1,
                duplicate: false,
                domain: None,
            })
        }

        async fn publish_with_id(
            &self,
            _subject: &str,
            _payload: &[u8],
            _message_id: &str,
            _headers: Option<&JetStreamHeaders>,
        ) -> Result<PublishAck, JetStreamError> {
            Ok(PublishAck {
                stream: "TEST".to_string(),
                sequence: 1,
                duplicate: false,
                domain: None,
            })
        }

        async fn subscribe(
            &self,
            _stream: &str,
            _consumer: &str,
            _filter_subject: Option<&str>,
        ) -> Result<Box<dyn crate::ports::JetStreamSubscription>, JetStreamError> {
            Err(JetStreamError::SubscribeFailed("Not implemented".to_string()))
        }

        async fn stream_info(&self, _stream: &str) -> Result<crate::ports::StreamInfo, JetStreamError> {
            Err(JetStreamError::StreamNotFound("Not implemented".to_string()))
        }

        async fn create_stream(
            &self,
            _config: &crate::ports::JetStreamStreamConfig,
        ) -> Result<crate::ports::StreamInfo, JetStreamError> {
            Err(JetStreamError::StreamCreationFailed("Not implemented".to_string()))
        }

        async fn create_consumer(
            &self,
            _stream: &str,
            _config: &crate::ports::JetStreamConsumerConfig,
        ) -> Result<crate::ports::ConsumerInfo, JetStreamError> {
            Err(JetStreamError::ConsumerCreationFailed("Not implemented".to_string()))
        }

        async fn is_connected(&self) -> bool {
            true
        }

        async fn kv_get(&self, bucket: &str, key: &str) -> Result<Option<Vec<u8>>, JetStreamError> {
            let store = self.kv_store.lock().await;
            let full_key = format!("{}.{}", bucket, key);
            Ok(store.get(&full_key).cloned())
        }

        async fn kv_put(&self, bucket: &str, key: &str, value: &[u8]) -> Result<u64, JetStreamError> {
            let mut store = self.kv_store.lock().await;
            let full_key = format!("{}.{}", bucket, key);
            store.insert(full_key, value.to_vec());
            Ok(1)
        }

        async fn kv_delete(&self, bucket: &str, key: &str) -> Result<(), JetStreamError> {
            let mut store = self.kv_store.lock().await;
            let full_key = format!("{}.{}", bucket, key);
            store.remove(&full_key);
            Ok(())
        }

        async fn kv_keys(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, JetStreamError> {
            let store = self.kv_store.lock().await;
            let bucket_prefix = format!("{}.", bucket);
            let full_prefix = format!("{}{}", bucket_prefix, prefix);
            Ok(store
                .keys()
                .filter(|k| k.starts_with(&full_prefix))
                .map(|k| k.strip_prefix(&bucket_prefix).unwrap_or(k).to_string())
                .collect())
        }

        async fn kv_create_bucket(&self, _bucket: &str, _config: &crate::ports::KvBucketConfig) -> Result<(), JetStreamError> {
            Ok(())
        }
    }

    fn create_test_request() -> ProvisioningRequest {
        ProvisioningRequest {
            organization_id: BootstrapOrgId::new(),
            person_id: Uuid::now_v7(),
            person_name: "Test User".to_string(),
            person_email: "test@example.com".to_string(),
            purpose: CertificatePurpose::Authentication,
            key_algorithm: KeyAlgorithm::Ed25519,
            issuing_ca_id: CertificateId::new(),
            yubikey_device_id: YubiKeyDeviceId::new(),
            yubikey_serial: "12345678".to_string(),
            target_slot: PIVSlot::Authentication,
            validity_days: 365,
        }
    }

    #[tokio::test]
    async fn test_saga_handler_config_default() {
        let config = SagaHandlerConfig::default();
        assert_eq!(config.max_step_retries, 3);
        assert!(config.auto_compensate);
        assert_eq!(config.saga_timeout_secs, 300);
    }

    #[tokio::test]
    async fn test_handle_provision_certificate() {
        let port = MockJetStreamPort::new();
        let handler = SagaCommandHandler::new(port, SagaHandlerConfig::default());

        let request = create_test_request();
        let result = handler.handle_provision_certificate(request).await;

        assert!(result.is_ok());
        let saga_result = result.unwrap();
        assert_eq!(saga_result.status, SagaCommandStatus::Completed);
        assert!(saga_result.result.is_some());
        assert!(saga_result.error.is_none());
    }

    #[tokio::test]
    async fn test_certificate_provisioning_executor() {
        let executor = CertificateProvisioningExecutor::new();
        let request = create_test_request();
        let mut saga = CertificateProvisioningSaga::new(request);

        // Start the saga - transitions to GeneratingKey
        saga.start().unwrap();

        // Step 1: GeneratingKey -> GeneratingCertificate (emits KeyGenerated event)
        let step1 = executor.execute_next_step(&mut saga).await.unwrap();
        assert!(matches!(step1, StepExecutionResult::Continue { .. }));
        assert_eq!(step1.domain_events().len(), 1);

        // Step 2: GeneratingCertificate -> ProvisioningToYubiKey (emits CertificateGenerated)
        let step2 = executor.execute_next_step(&mut saga).await.unwrap();
        assert!(matches!(step2, StepExecutionResult::Continue { .. }));
        assert_eq!(step2.domain_events().len(), 1);

        // Step 3: ProvisioningToYubiKey -> VerifyingProvisioning (emits YubiKeyProvisioned)
        let step3 = executor.execute_next_step(&mut saga).await.unwrap();
        assert!(matches!(step3, StepExecutionResult::Continue { .. }));
        assert_eq!(step3.domain_events().len(), 1);

        // Step 4: VerifyingProvisioning -> Completed (with result, no additional events)
        let step4 = executor.execute_next_step(&mut saga).await.unwrap();
        assert!(matches!(step4, StepExecutionResult::Completed { .. }));
        assert_eq!(step4.domain_events().len(), 0);

        // Verify saga is in completed state
        assert!(saga.is_completed());
    }

    #[tokio::test]
    async fn test_saga_command_result() {
        let result: SagaCommandResult<String> = SagaCommandResult {
            saga_id: Uuid::now_v7(),
            correlation_id: Uuid::now_v7(),
            status: SagaCommandStatus::Completed,
            result: Some("test".to_string()),
            error: None,
        };

        assert_eq!(result.status, SagaCommandStatus::Completed);
        assert!(result.result.is_some());
    }

    #[tokio::test]
    async fn test_compensation() {
        let executor = CertificateProvisioningExecutor::new();
        let request = create_test_request();
        let mut saga = CertificateProvisioningSaga::new(request);

        saga.start().unwrap();
        saga.advance(); // GeneratingKey
        saga.advance(); // GeneratingCertificate
        saga.fail("Test failure", "GeneratingCertificate");

        let result = executor.compensate(&mut saga).await.unwrap();
        assert!(matches!(result, CompensationResult::FullyCompensated));
    }

    #[tokio::test]
    async fn test_list_active_sagas() {
        let port = MockJetStreamPort::new();
        let handler = SagaCommandHandler::new(port, SagaHandlerConfig::default());

        // Initially no active sagas
        let sagas = handler.list_active_sagas().await.unwrap();
        assert!(sagas.is_empty());
    }

    /// Test that domain events are correctly formed during saga execution
    ///
    /// ## FRP Axiom Validation
    ///
    /// - A3 (Decoupled): Events created after step completion
    /// - A7 (Change Prefixes): Events form causal chain via correlation_id
    #[tokio::test]
    async fn test_domain_event_formation_and_causality() {
        use crate::events::DomainEvent;

        let executor = CertificateProvisioningExecutor::new();
        let request = create_test_request();
        let mut saga = CertificateProvisioningSaga::new(request);
        let correlation_id = saga.correlation_id();
        let saga_id = saga.saga_id();

        // Start the saga
        saga.start().unwrap();

        // Step 1: KeyGenerated event
        let step1 = executor.execute_next_step(&mut saga).await.unwrap();
        let events1 = step1.domain_events();
        assert_eq!(events1.len(), 1);

        // Verify KeyGenerated event structure
        match &events1[0] {
            DomainEvent::Key(key_event) => {
                match key_event {
                    crate::events::KeyEvents::KeyGenerated(e) => {
                        // A7: Verify causality chain
                        assert_eq!(e.correlation_id, correlation_id);
                        assert_eq!(e.causation_id, Some(saga_id));
                        // Verify key was hardware-backed
                        assert!(e.hardware_backed);
                    }
                    _ => panic!("Expected KeyGenerated event"),
                }
            }
            _ => panic!("Expected Key domain event"),
        }

        // Step 2: CertificateGenerated event
        let step2 = executor.execute_next_step(&mut saga).await.unwrap();
        let events2 = step2.domain_events();
        assert_eq!(events2.len(), 1);

        match &events2[0] {
            DomainEvent::Certificate(cert_event) => {
                match cert_event {
                    crate::events::CertificateEvents::CertificateGenerated(e) => {
                        // A7: Verify causality chain
                        assert_eq!(e.correlation_id, correlation_id);
                        assert_eq!(e.causation_id, Some(saga_id));
                        // Verify certificate metadata
                        assert!(!e.is_ca);
                    }
                    _ => panic!("Expected CertificateGenerated event"),
                }
            }
            _ => panic!("Expected Certificate domain event"),
        }

        // Step 3: YubiKeyProvisioned event
        let step3 = executor.execute_next_step(&mut saga).await.unwrap();
        let events3 = step3.domain_events();
        assert_eq!(events3.len(), 1);

        match &events3[0] {
            DomainEvent::YubiKey(yk_event) => {
                match yk_event {
                    crate::events::YubiKeyEvents::YubiKeyProvisioned(e) => {
                        // A7: Verify causality chain
                        assert_eq!(e.correlation_id, correlation_id);
                        assert_eq!(e.causation_id, Some(saga_id));
                        // Verify slot was configured
                        assert_eq!(e.slots_configured.len(), 1);
                    }
                    _ => panic!("Expected YubiKeyProvisioned event"),
                }
            }
            _ => panic!("Expected YubiKey domain event"),
        }

        // Step 4: Completed (no additional events)
        let step4 = executor.execute_next_step(&mut saga).await.unwrap();
        assert!(matches!(step4, StepExecutionResult::Completed { .. }));
        assert!(step4.domain_events().is_empty());
    }

    /// Test FRP Axiom A9: Semantic Preservation Under Composition
    ///
    /// Verify that the order of events respects the saga step sequence
    #[tokio::test]
    async fn test_axiom_a9_composition_semantics() {
        let executor = CertificateProvisioningExecutor::new();
        let request = create_test_request();
        let mut saga = CertificateProvisioningSaga::new(request);
        saga.start().unwrap();

        // Collect all events in order
        let mut all_events = Vec::new();

        for _ in 0..4 {
            if saga.is_terminal() { break; }
            let result = executor.execute_next_step(&mut saga).await.unwrap();
            all_events.extend(result.domain_events().to_vec());
        }

        // Total should be 3 domain events (Key, Certificate, YubiKey)
        assert_eq!(all_events.len(), 3);

        // Verify correct event order (composition semantics preserved)
        assert!(matches!(&all_events[0], crate::events::DomainEvent::Key(_)));
        assert!(matches!(&all_events[1], crate::events::DomainEvent::Certificate(_)));
        assert!(matches!(&all_events[2], crate::events::DomainEvent::YubiKey(_)));
    }
}
