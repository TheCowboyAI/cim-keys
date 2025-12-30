// Copyright (c) 2025 - Cowboy AI, LLC.

//! Person Onboarding Saga
//!
//! Coordinates the onboarding of a new person across multiple aggregates:
//! 1. Create person in organization aggregate
//! 2. Generate cryptographic key for the person
//! 3. Generate leaf certificate signed by appropriate CA
//! 4. Create NATS user in the person's account
//! 5. Provision YubiKey if required
//!
//! ## State Machine
//!
//! ```text
//! Initial → CreatingPerson → GeneratingKey → GeneratingCertificate
//!     ↓           ↓               ↓                  ↓
//!   Failed     Failed          Failed             Failed
//!                                                    ↓
//!                                        CreatingNatsUser → ProvisioningYubiKey → Completed
//!                                              ↓                    ↓
//!                                           Failed               Failed
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{SagaState, SagaError};
use crate::domain::ids::*;
use crate::domain::bootstrap::KeyOwnerRole;

/// Person Onboarding Saga state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonOnboardingSaga {
    /// Unique saga ID
    pub saga_id: Uuid,
    /// Correlation ID for all events
    pub correlation_id: Uuid,
    /// Current state
    pub state: OnboardingState,
    /// State at which failure occurred (for compensation)
    failed_at_state: Option<OnboardingState>,
    /// Started at timestamp
    pub started_at: DateTime<Utc>,
    /// Completed at timestamp (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Person data
    pub person: PersonOnboardingData,
    /// Generated artifacts
    pub artifacts: OnboardingArtifacts,
    /// Error if failed
    pub error: Option<SagaError>,
}

/// Onboarding state machine states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OnboardingState {
    /// Saga not started
    Initial,
    /// Creating person in organization
    CreatingPerson,
    /// Generating cryptographic key
    GeneratingKey,
    /// Generating leaf certificate
    GeneratingCertificate,
    /// Creating NATS user
    CreatingNatsUser,
    /// Provisioning YubiKey
    ProvisioningYubiKey,
    /// Successfully completed
    Completed,
    /// Failed (see error field)
    Failed,
    /// Compensating (rolling back)
    Compensating(OnboardingCompensationStep),
}

/// Compensation sub-steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OnboardingCompensationStep {
    RevokeYubiKeySlots,
    DeleteNatsUser,
    RevokeCertificate,
    RevokeKey,
    DeactivatePerson,
}

/// Person data for onboarding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonOnboardingData {
    /// Person ID
    pub person_id: Uuid,
    /// Organization ID
    pub organization_id: BootstrapOrgId,
    /// Person name
    pub name: String,
    /// Person email
    pub email: String,
    /// Role (determines permissions and YubiKey slots)
    pub role: KeyOwnerRole,
    /// Unit IDs the person belongs to
    pub unit_ids: Vec<UnitId>,
    /// Whether this person needs a YubiKey
    pub needs_yubikey: bool,
    /// YubiKey serial (if assigning an existing device)
    pub yubikey_serial: Option<String>,
    /// Issuer CA for the person's certificate
    pub issuing_ca_id: Option<CertificateId>,
    /// NATS account for the person
    pub nats_account_id: Option<NatsAccountId>,
}

/// Artifacts generated during onboarding
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnboardingArtifacts {
    /// Generated key ID
    pub key_id: Option<KeyId>,
    /// Generated certificate ID
    pub certificate_id: Option<CertificateId>,
    /// Created NATS user ID
    pub nats_user_id: Option<NatsUserId>,
    /// Provisioned YubiKey device ID
    pub yubikey_device_id: Option<YubiKeyDeviceId>,
    /// Provisioned YubiKey slots
    pub yubikey_slots: Vec<String>,
}

impl PersonOnboardingSaga {
    /// Create a new person onboarding saga
    pub fn new(person: PersonOnboardingData) -> Self {
        Self {
            saga_id: Uuid::now_v7(),
            correlation_id: Uuid::now_v7(),
            state: OnboardingState::Initial,
            failed_at_state: None,
            started_at: Utc::now(),
            completed_at: None,
            person,
            artifacts: OnboardingArtifacts::default(),
            error: None,
        }
    }

    /// Create with explicit correlation ID (for linking to parent saga)
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    /// Start the saga
    pub fn start(&mut self) -> Result<(), SagaError> {
        if self.person.name.is_empty() {
            return Err(SagaError::new("Person name required", "Initial"));
        }
        if self.person.email.is_empty() {
            return Err(SagaError::new("Person email required", "Initial"));
        }
        self.state = OnboardingState::CreatingPerson;
        Ok(())
    }

    /// Transition to the next state
    pub fn advance(&mut self) -> OnboardingState {
        self.state = match &self.state {
            OnboardingState::Initial => OnboardingState::CreatingPerson,
            OnboardingState::CreatingPerson => OnboardingState::GeneratingKey,
            OnboardingState::GeneratingKey => OnboardingState::GeneratingCertificate,
            OnboardingState::GeneratingCertificate => {
                if self.person.nats_account_id.is_some() {
                    OnboardingState::CreatingNatsUser
                } else if self.person.needs_yubikey {
                    OnboardingState::ProvisioningYubiKey
                } else {
                    self.completed_at = Some(Utc::now());
                    OnboardingState::Completed
                }
            }
            OnboardingState::CreatingNatsUser => {
                if self.person.needs_yubikey {
                    OnboardingState::ProvisioningYubiKey
                } else {
                    self.completed_at = Some(Utc::now());
                    OnboardingState::Completed
                }
            }
            OnboardingState::ProvisioningYubiKey => {
                self.completed_at = Some(Utc::now());
                OnboardingState::Completed
            }
            OnboardingState::Completed => OnboardingState::Completed,
            OnboardingState::Failed => OnboardingState::Failed,
            OnboardingState::Compensating(_) => OnboardingState::Failed,
        };
        self.state.clone()
    }

    /// Mark the saga as failed
    pub fn fail(&mut self, message: impl Into<String>, step: impl Into<String>) {
        self.failed_at_state = Some(self.state.clone());
        self.error = Some(SagaError::new(message, step));
        self.state = OnboardingState::Failed;
    }

    /// Start compensation
    pub fn start_compensation(&mut self) -> OnboardingCompensationStep {
        // Use the state at which failure occurred, not the current (Failed) state
        let failed_state = self.failed_at_state.as_ref().unwrap_or(&self.state);
        let step = match failed_state {
            OnboardingState::ProvisioningYubiKey => OnboardingCompensationStep::RevokeYubiKeySlots,
            OnboardingState::CreatingNatsUser => OnboardingCompensationStep::DeleteNatsUser,
            OnboardingState::GeneratingCertificate => OnboardingCompensationStep::RevokeCertificate,
            OnboardingState::GeneratingKey => OnboardingCompensationStep::RevokeKey,
            _ => OnboardingCompensationStep::DeactivatePerson,
        };
        self.state = OnboardingState::Compensating(step.clone());
        step
    }

    /// Advance compensation to next step
    pub fn advance_compensation(&mut self) -> Option<OnboardingCompensationStep> {
        if let OnboardingState::Compensating(current) = &self.state {
            let next = match current {
                OnboardingCompensationStep::RevokeYubiKeySlots => {
                    Some(OnboardingCompensationStep::DeleteNatsUser)
                }
                OnboardingCompensationStep::DeleteNatsUser => {
                    Some(OnboardingCompensationStep::RevokeCertificate)
                }
                OnboardingCompensationStep::RevokeCertificate => {
                    Some(OnboardingCompensationStep::RevokeKey)
                }
                OnboardingCompensationStep::RevokeKey => {
                    Some(OnboardingCompensationStep::DeactivatePerson)
                }
                OnboardingCompensationStep::DeactivatePerson => None,
            };

            if let Some(step) = next.clone() {
                self.state = OnboardingState::Compensating(step);
            } else {
                self.state = OnboardingState::Failed;
            }

            next
        } else {
            None
        }
    }

    /// Record key generation
    pub fn record_key(&mut self, key_id: KeyId) {
        self.artifacts.key_id = Some(key_id);
    }

    /// Record certificate generation
    pub fn record_certificate(&mut self, cert_id: CertificateId) {
        self.artifacts.certificate_id = Some(cert_id);
    }

    /// Record NATS user creation
    pub fn record_nats_user(&mut self, user_id: NatsUserId) {
        self.artifacts.nats_user_id = Some(user_id);
    }

    /// Record YubiKey provisioning
    pub fn record_yubikey(&mut self, device_id: YubiKeyDeviceId, slots: Vec<String>) {
        self.artifacts.yubikey_device_id = Some(device_id);
        self.artifacts.yubikey_slots = slots;
    }

    /// Get the slots needed based on role
    pub fn required_yubikey_slots(&self) -> Vec<&'static str> {
        match self.person.role {
            KeyOwnerRole::RootAuthority => vec!["9C"], // Digital Signature
            KeyOwnerRole::SecurityAdmin => vec!["9A", "9C", "9D"], // Auth, Sign, KeyMgmt
            KeyOwnerRole::Developer => vec!["9A"], // Authentication
            KeyOwnerRole::ServiceAccount => vec!["9E"], // Card Auth
            KeyOwnerRole::BackupHolder => vec!["9A", "9D"], // Auth, KeyMgmt
            KeyOwnerRole::Auditor => vec!["9A"], // Authentication
        }
    }

    /// Get current step name for logging
    pub fn current_step_name(&self) -> String {
        match &self.state {
            OnboardingState::Initial => "Initial".to_string(),
            OnboardingState::CreatingPerson => "CreatingPerson".to_string(),
            OnboardingState::GeneratingKey => "GeneratingKey".to_string(),
            OnboardingState::GeneratingCertificate => "GeneratingCertificate".to_string(),
            OnboardingState::CreatingNatsUser => "CreatingNatsUser".to_string(),
            OnboardingState::ProvisioningYubiKey => "ProvisioningYubiKey".to_string(),
            OnboardingState::Completed => "Completed".to_string(),
            OnboardingState::Failed => "Failed".to_string(),
            OnboardingState::Compensating(step) => format!("Compensating:{:?}", step),
        }
    }
}

impl SagaState for PersonOnboardingSaga {
    fn saga_id(&self) -> Uuid {
        self.saga_id
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    fn is_terminal(&self) -> bool {
        matches!(self.state, OnboardingState::Completed | OnboardingState::Failed)
    }

    fn is_completed(&self) -> bool {
        matches!(self.state, OnboardingState::Completed)
    }

    fn is_failed(&self) -> bool {
        matches!(self.state, OnboardingState::Failed)
    }

    fn status_description(&self) -> String {
        match &self.state {
            OnboardingState::Initial => "Not started".to_string(),
            OnboardingState::CreatingPerson => format!("Creating person: {}", self.person.name),
            OnboardingState::GeneratingKey => format!("Generating key for: {}", self.person.name),
            OnboardingState::GeneratingCertificate => {
                format!("Generating certificate for: {}", self.person.name)
            }
            OnboardingState::CreatingNatsUser => {
                format!("Creating NATS user for: {}", self.person.name)
            }
            OnboardingState::ProvisioningYubiKey => {
                format!("Provisioning YubiKey for: {}", self.person.name)
            }
            OnboardingState::Completed => format!("Onboarding completed for: {}", self.person.name),
            OnboardingState::Failed => format!(
                "Onboarding failed for {}: {}",
                self.person.name,
                self.error.as_ref().map_or("Unknown error", |e| &e.message)
            ),
            OnboardingState::Compensating(step) => {
                format!("Rolling back onboarding for {}: {:?}", self.person.name, step)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_person() -> PersonOnboardingData {
        PersonOnboardingData {
            person_id: Uuid::now_v7(),
            organization_id: BootstrapOrgId::new(),
            name: "Alice Engineer".to_string(),
            email: "alice@test.com".to_string(),
            role: KeyOwnerRole::Developer,
            unit_ids: vec![UnitId::new()],
            needs_yubikey: true,
            yubikey_serial: None,
            issuing_ca_id: Some(CertificateId::new()),
            nats_account_id: Some(NatsAccountId::new()),
        }
    }

    #[test]
    fn test_saga_creation() {
        let person = create_test_person();
        let saga = PersonOnboardingSaga::new(person.clone());

        assert_eq!(saga.person.name, person.name);
        assert_eq!(saga.state, OnboardingState::Initial);
        assert!(!saga.is_terminal());
    }

    #[test]
    fn test_saga_start_validation() {
        let mut person = create_test_person();
        person.name = "".to_string();

        let mut saga = PersonOnboardingSaga::new(person);
        let result = saga.start();
        assert!(result.is_err());

        let mut person2 = create_test_person();
        person2.email = "".to_string();

        let mut saga2 = PersonOnboardingSaga::new(person2);
        let result2 = saga2.start();
        assert!(result2.is_err());
    }

    #[test]
    fn test_saga_full_flow() {
        let person = create_test_person();
        let mut saga = PersonOnboardingSaga::new(person);

        saga.start().unwrap();

        // Progress through all states
        let expected_states = [
            OnboardingState::GeneratingKey,
            OnboardingState::GeneratingCertificate,
            OnboardingState::CreatingNatsUser,
            OnboardingState::ProvisioningYubiKey,
            OnboardingState::Completed,
        ];

        for expected in expected_states {
            saga.advance();
            assert_eq!(saga.state, expected);
        }

        assert!(saga.is_completed());
    }

    #[test]
    fn test_saga_skip_optional_steps() {
        let mut person = create_test_person();
        person.nats_account_id = None;
        person.needs_yubikey = false;

        let mut saga = PersonOnboardingSaga::new(person);
        saga.start().unwrap();

        saga.advance(); // GeneratingKey
        saga.advance(); // GeneratingCertificate
        saga.advance(); // Should skip to Completed

        assert!(saga.is_completed());
    }

    #[test]
    fn test_required_slots_by_role() {
        let mut person = create_test_person();

        person.role = KeyOwnerRole::RootAuthority;
        let saga = PersonOnboardingSaga::new(person.clone());
        assert_eq!(saga.required_yubikey_slots(), vec!["9C"]);

        person.role = KeyOwnerRole::SecurityAdmin;
        let saga = PersonOnboardingSaga::new(person.clone());
        assert_eq!(saga.required_yubikey_slots(), vec!["9A", "9C", "9D"]);

        person.role = KeyOwnerRole::Developer;
        let saga = PersonOnboardingSaga::new(person.clone());
        assert_eq!(saga.required_yubikey_slots(), vec!["9A"]);
    }

    #[test]
    fn test_saga_compensation() {
        let person = create_test_person();
        let mut saga = PersonOnboardingSaga::new(person);

        saga.start().unwrap();
        saga.advance(); // GeneratingKey
        saga.advance(); // GeneratingCertificate

        // Simulate failure during certificate generation
        saga.fail("Certificate generation failed", "GeneratingCertificate");
        assert!(saga.is_failed());

        // Start compensation
        let step = saga.start_compensation();
        assert_eq!(step, OnboardingCompensationStep::RevokeCertificate);

        // Continue compensation
        let step = saga.advance_compensation();
        assert_eq!(step, Some(OnboardingCompensationStep::RevokeKey));

        let step = saga.advance_compensation();
        assert_eq!(step, Some(OnboardingCompensationStep::DeactivatePerson));

        let step = saga.advance_compensation();
        assert_eq!(step, None);
    }

    #[test]
    fn test_record_artifacts() {
        let person = create_test_person();
        let mut saga = PersonOnboardingSaga::new(person);

        let key_id = KeyId::new();
        saga.record_key(key_id);
        assert_eq!(saga.artifacts.key_id, Some(key_id));

        let cert_id = CertificateId::new();
        saga.record_certificate(cert_id);
        assert_eq!(saga.artifacts.certificate_id, Some(cert_id));

        let user_id = NatsUserId::new();
        saga.record_nats_user(user_id);
        assert_eq!(saga.artifacts.nats_user_id, Some(user_id));

        let device_id = YubiKeyDeviceId::new();
        saga.record_yubikey(device_id, vec!["9A".to_string()]);
        assert_eq!(saga.artifacts.yubikey_device_id, Some(device_id));
        assert_eq!(saga.artifacts.yubikey_slots, vec!["9A".to_string()]);
    }
}
