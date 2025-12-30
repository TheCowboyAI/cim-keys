// Copyright (c) 2025 - Cowboy AI, LLC.

//! Complete Bootstrap Saga
//!
//! Orchestrates the full CIM infrastructure bootstrap process:
//! 1. Create Organization with units
//! 2. Add People to organization
//! 3. Generate PKI hierarchy (Root CA → Intermediate CAs → Leaf certs)
//! 4. Setup NATS security (Operator → Accounts → Users)
//! 5. Provision YubiKeys for key holders
//!
//! ## State Machine
//!
//! ```text
//! Initial → CreatingOrganization → AddingPeople → GeneratingPKI
//!     ↓              ↓                  ↓             ↓
//!   Failed        Failed            Failed        Failed
//!                                                    ↓
//!                                            SettingUpNATS → ProvisioningYubiKeys → Completed
//!                                                 ↓                  ↓
//!                                              Failed             Failed
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::{SagaState, SagaError};
use crate::domain::ids::*;

/// Complete Bootstrap Saga state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteBootstrapSaga {
    /// Unique saga ID
    pub saga_id: Uuid,
    /// Correlation ID for all events
    pub correlation_id: Uuid,
    /// Current state
    pub state: BootstrapState,
    /// State at which failure occurred (for compensation)
    failed_at_state: Option<BootstrapState>,
    /// Started at timestamp
    pub started_at: DateTime<Utc>,
    /// Completed at timestamp (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Organization being bootstrapped
    pub organization: Option<OrganizationBootstrapData>,
    /// People to add
    pub people: Vec<PersonBootstrapData>,
    /// PKI state
    pub pki: PkiBootstrapState,
    /// NATS state
    pub nats: NatsBootstrapState,
    /// YubiKey state
    pub yubikey: YubiKeyBootstrapState,
    /// Error if failed
    pub error: Option<SagaError>,
}

/// Bootstrap state machine states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BootstrapState {
    /// Saga not started
    Initial,
    /// Creating organization and units
    CreatingOrganization,
    /// Adding people to organization
    AddingPeople,
    /// Generating PKI hierarchy
    GeneratingPKI(PkiGenerationStep),
    /// Setting up NATS infrastructure
    SettingUpNATS(NatsSetupStep),
    /// Provisioning YubiKeys
    ProvisioningYubiKeys,
    /// Successfully completed
    Completed,
    /// Failed (see error field)
    Failed,
    /// Compensating (rolling back)
    Compensating(CompensationStep),
}

/// PKI generation sub-steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PkiGenerationStep {
    GeneratingRootCA,
    GeneratingIntermediateCAs,
    GeneratingLeafCertificates,
}

/// NATS setup sub-steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NatsSetupStep {
    CreatingOperator,
    CreatingSystemAccount,
    CreatingAccounts,
    CreatingUsers,
}

/// Compensation sub-steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompensationStep {
    RollbackYubiKeys,
    RollbackNATS,
    RollbackPKI,
    RollbackOrganization,
}

/// Organization data for bootstrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationBootstrapData {
    pub id: BootstrapOrgId,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub units: Vec<UnitBootstrapData>,
}

/// Unit data for bootstrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitBootstrapData {
    pub id: UnitId,
    pub name: String,
    pub unit_type: String,
    pub parent_id: Option<UnitId>,
}

/// Person data for bootstrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonBootstrapData {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub unit_ids: Vec<UnitId>,
    /// Whether this person needs a YubiKey
    pub needs_yubikey: bool,
}

/// PKI bootstrap state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PkiBootstrapState {
    /// Root CA ID (if generated)
    pub root_ca_id: Option<CertificateId>,
    /// Intermediate CA IDs by unit
    pub intermediate_ca_ids: HashMap<UnitId, CertificateId>,
    /// Leaf certificate IDs by person
    pub leaf_cert_ids: HashMap<Uuid, CertificateId>,
    /// Generated key IDs
    pub key_ids: Vec<KeyId>,
}

/// NATS bootstrap state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NatsBootstrapState {
    /// Operator ID (if created)
    pub operator_id: Option<NatsOperatorId>,
    /// System account ID
    pub system_account_id: Option<NatsAccountId>,
    /// Account IDs by unit
    pub account_ids: HashMap<UnitId, NatsAccountId>,
    /// User IDs by person
    pub user_ids: HashMap<Uuid, NatsUserId>,
}

/// YubiKey bootstrap state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct YubiKeyBootstrapState {
    /// YubiKey assignments by person
    pub assignments: HashMap<Uuid, YubiKeyAssignment>,
}

/// YubiKey assignment for a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyAssignment {
    pub device_id: YubiKeyDeviceId,
    pub serial: String,
    pub slots_provisioned: Vec<String>,
}

impl CompleteBootstrapSaga {
    /// Create a new bootstrap saga
    pub fn new(correlation_id: Uuid) -> Self {
        Self {
            saga_id: Uuid::now_v7(),
            correlation_id,
            state: BootstrapState::Initial,
            failed_at_state: None,
            started_at: Utc::now(),
            completed_at: None,
            organization: None,
            people: Vec::new(),
            pki: PkiBootstrapState::default(),
            nats: NatsBootstrapState::default(),
            yubikey: YubiKeyBootstrapState::default(),
            error: None,
        }
    }

    /// Set organization data
    pub fn with_organization(mut self, org: OrganizationBootstrapData) -> Self {
        self.organization = Some(org);
        self
    }

    /// Add people to bootstrap
    pub fn with_people(mut self, people: Vec<PersonBootstrapData>) -> Self {
        self.people = people;
        self
    }

    /// Start the saga
    pub fn start(&mut self) -> Result<(), SagaError> {
        if self.organization.is_none() {
            return Err(SagaError::new(
                "Organization data required",
                "Initial",
            ));
        }
        self.state = BootstrapState::CreatingOrganization;
        Ok(())
    }

    /// Transition to the next state
    pub fn advance(&mut self) -> BootstrapState {
        self.state = match &self.state {
            BootstrapState::Initial => {
                if self.organization.is_some() {
                    BootstrapState::CreatingOrganization
                } else {
                    BootstrapState::Failed
                }
            }
            BootstrapState::CreatingOrganization => {
                if !self.people.is_empty() {
                    BootstrapState::AddingPeople
                } else {
                    BootstrapState::GeneratingPKI(PkiGenerationStep::GeneratingRootCA)
                }
            }
            BootstrapState::AddingPeople => {
                BootstrapState::GeneratingPKI(PkiGenerationStep::GeneratingRootCA)
            }
            BootstrapState::GeneratingPKI(step) => match step {
                PkiGenerationStep::GeneratingRootCA => {
                    BootstrapState::GeneratingPKI(PkiGenerationStep::GeneratingIntermediateCAs)
                }
                PkiGenerationStep::GeneratingIntermediateCAs => {
                    BootstrapState::GeneratingPKI(PkiGenerationStep::GeneratingLeafCertificates)
                }
                PkiGenerationStep::GeneratingLeafCertificates => {
                    BootstrapState::SettingUpNATS(NatsSetupStep::CreatingOperator)
                }
            },
            BootstrapState::SettingUpNATS(step) => match step {
                NatsSetupStep::CreatingOperator => {
                    BootstrapState::SettingUpNATS(NatsSetupStep::CreatingSystemAccount)
                }
                NatsSetupStep::CreatingSystemAccount => {
                    BootstrapState::SettingUpNATS(NatsSetupStep::CreatingAccounts)
                }
                NatsSetupStep::CreatingAccounts => {
                    BootstrapState::SettingUpNATS(NatsSetupStep::CreatingUsers)
                }
                NatsSetupStep::CreatingUsers => {
                    if self.people.iter().any(|p| p.needs_yubikey) {
                        BootstrapState::ProvisioningYubiKeys
                    } else {
                        self.completed_at = Some(Utc::now());
                        BootstrapState::Completed
                    }
                }
            },
            BootstrapState::ProvisioningYubiKeys => {
                self.completed_at = Some(Utc::now());
                BootstrapState::Completed
            }
            BootstrapState::Completed => BootstrapState::Completed,
            BootstrapState::Failed => BootstrapState::Failed,
            BootstrapState::Compensating(_) => BootstrapState::Failed,
        };
        self.state.clone()
    }

    /// Mark the saga as failed
    pub fn fail(&mut self, message: impl Into<String>, step: impl Into<String>) {
        self.failed_at_state = Some(self.state.clone());
        self.error = Some(SagaError::new(message, step));
        self.state = BootstrapState::Failed;
    }

    /// Start compensation
    pub fn start_compensation(&mut self) -> CompensationStep {
        // Use the state at which failure occurred, not the current (Failed) state
        let failed_state = self.failed_at_state.as_ref().unwrap_or(&self.state);
        let step = match failed_state {
            BootstrapState::ProvisioningYubiKeys => CompensationStep::RollbackYubiKeys,
            BootstrapState::SettingUpNATS(_) => CompensationStep::RollbackNATS,
            BootstrapState::GeneratingPKI(_) => CompensationStep::RollbackPKI,
            BootstrapState::AddingPeople | BootstrapState::CreatingOrganization => {
                CompensationStep::RollbackOrganization
            }
            _ => CompensationStep::RollbackOrganization,
        };
        self.state = BootstrapState::Compensating(step.clone());
        step
    }

    /// Record Root CA generation
    pub fn record_root_ca(&mut self, cert_id: CertificateId, key_id: KeyId) {
        self.pki.root_ca_id = Some(cert_id);
        self.pki.key_ids.push(key_id);
    }

    /// Record Intermediate CA generation
    pub fn record_intermediate_ca(&mut self, unit_id: UnitId, cert_id: CertificateId, key_id: KeyId) {
        self.pki.intermediate_ca_ids.insert(unit_id, cert_id);
        self.pki.key_ids.push(key_id);
    }

    /// Record Leaf certificate generation
    pub fn record_leaf_cert(&mut self, person_id: Uuid, cert_id: CertificateId, key_id: KeyId) {
        self.pki.leaf_cert_ids.insert(person_id, cert_id);
        self.pki.key_ids.push(key_id);
    }

    /// Record NATS Operator creation
    pub fn record_operator(&mut self, operator_id: NatsOperatorId) {
        self.nats.operator_id = Some(operator_id);
    }

    /// Record NATS System Account creation
    pub fn record_system_account(&mut self, account_id: NatsAccountId) {
        self.nats.system_account_id = Some(account_id);
    }

    /// Record NATS Account creation
    pub fn record_account(&mut self, unit_id: UnitId, account_id: NatsAccountId) {
        self.nats.account_ids.insert(unit_id, account_id);
    }

    /// Record NATS User creation
    pub fn record_user(&mut self, person_id: Uuid, user_id: NatsUserId) {
        self.nats.user_ids.insert(person_id, user_id);
    }

    /// Record YubiKey assignment
    pub fn record_yubikey_assignment(
        &mut self,
        person_id: Uuid,
        device_id: YubiKeyDeviceId,
        serial: String,
        slots: Vec<String>,
    ) {
        self.yubikey.assignments.insert(person_id, YubiKeyAssignment {
            device_id,
            serial,
            slots_provisioned: slots,
        });
    }

    /// Get current step name for logging
    pub fn current_step_name(&self) -> String {
        match &self.state {
            BootstrapState::Initial => "Initial".to_string(),
            BootstrapState::CreatingOrganization => "CreatingOrganization".to_string(),
            BootstrapState::AddingPeople => "AddingPeople".to_string(),
            BootstrapState::GeneratingPKI(step) => format!("GeneratingPKI:{:?}", step),
            BootstrapState::SettingUpNATS(step) => format!("SettingUpNATS:{:?}", step),
            BootstrapState::ProvisioningYubiKeys => "ProvisioningYubiKeys".to_string(),
            BootstrapState::Completed => "Completed".to_string(),
            BootstrapState::Failed => "Failed".to_string(),
            BootstrapState::Compensating(step) => format!("Compensating:{:?}", step),
        }
    }
}

impl SagaState for CompleteBootstrapSaga {
    fn saga_id(&self) -> Uuid {
        self.saga_id
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    fn is_terminal(&self) -> bool {
        matches!(self.state, BootstrapState::Completed | BootstrapState::Failed)
    }

    fn is_completed(&self) -> bool {
        matches!(self.state, BootstrapState::Completed)
    }

    fn is_failed(&self) -> bool {
        matches!(self.state, BootstrapState::Failed)
    }

    fn status_description(&self) -> String {
        match &self.state {
            BootstrapState::Initial => "Not started".to_string(),
            BootstrapState::CreatingOrganization => "Creating organization structure".to_string(),
            BootstrapState::AddingPeople => "Adding people to organization".to_string(),
            BootstrapState::GeneratingPKI(step) => match step {
                PkiGenerationStep::GeneratingRootCA => "Generating Root CA".to_string(),
                PkiGenerationStep::GeneratingIntermediateCAs => "Generating Intermediate CAs".to_string(),
                PkiGenerationStep::GeneratingLeafCertificates => "Generating leaf certificates".to_string(),
            },
            BootstrapState::SettingUpNATS(step) => match step {
                NatsSetupStep::CreatingOperator => "Creating NATS Operator".to_string(),
                NatsSetupStep::CreatingSystemAccount => "Creating NATS System Account".to_string(),
                NatsSetupStep::CreatingAccounts => "Creating NATS Accounts".to_string(),
                NatsSetupStep::CreatingUsers => "Creating NATS Users".to_string(),
            },
            BootstrapState::ProvisioningYubiKeys => "Provisioning YubiKeys".to_string(),
            BootstrapState::Completed => "Bootstrap completed successfully".to_string(),
            BootstrapState::Failed => format!(
                "Failed: {}",
                self.error.as_ref().map_or("Unknown error", |e| &e.message)
            ),
            BootstrapState::Compensating(step) => format!("Rolling back: {:?}", step),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_org() -> OrganizationBootstrapData {
        OrganizationBootstrapData {
            id: BootstrapOrgId::new(),
            name: "TestOrg".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("Test org for saga".to_string()),
            units: vec![UnitBootstrapData {
                id: UnitId::new(),
                name: "Engineering".to_string(),
                unit_type: "Department".to_string(),
                parent_id: None,
            }],
        }
    }

    fn create_test_people(org_units: &[UnitId]) -> Vec<PersonBootstrapData> {
        vec![PersonBootstrapData {
            id: Uuid::now_v7(),
            name: "Alice Engineer".to_string(),
            email: "alice@test.com".to_string(),
            role: "Developer".to_string(),
            unit_ids: org_units.to_vec(),
            needs_yubikey: true,
        }]
    }

    #[test]
    fn test_saga_creation() {
        let correlation_id = Uuid::now_v7();
        let saga = CompleteBootstrapSaga::new(correlation_id);

        assert_eq!(saga.correlation_id(), correlation_id);
        assert_eq!(saga.state, BootstrapState::Initial);
        assert!(!saga.is_terminal());
    }

    #[test]
    fn test_saga_start_without_org_fails() {
        let mut saga = CompleteBootstrapSaga::new(Uuid::now_v7());
        let result = saga.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_saga_start_with_org_succeeds() {
        let org = create_test_org();
        let mut saga = CompleteBootstrapSaga::new(Uuid::now_v7())
            .with_organization(org);

        let result = saga.start();
        assert!(result.is_ok());
        assert_eq!(saga.state, BootstrapState::CreatingOrganization);
    }

    #[test]
    fn test_saga_state_transitions() {
        let org = create_test_org();
        let unit_ids: Vec<UnitId> = org.units.iter().map(|u| u.id).collect();
        let people = create_test_people(&unit_ids);

        let mut saga = CompleteBootstrapSaga::new(Uuid::now_v7())
            .with_organization(org)
            .with_people(people);

        saga.start().unwrap();

        // Progress through all states
        let states = [
            BootstrapState::AddingPeople,
            BootstrapState::GeneratingPKI(PkiGenerationStep::GeneratingRootCA),
            BootstrapState::GeneratingPKI(PkiGenerationStep::GeneratingIntermediateCAs),
            BootstrapState::GeneratingPKI(PkiGenerationStep::GeneratingLeafCertificates),
            BootstrapState::SettingUpNATS(NatsSetupStep::CreatingOperator),
            BootstrapState::SettingUpNATS(NatsSetupStep::CreatingSystemAccount),
            BootstrapState::SettingUpNATS(NatsSetupStep::CreatingAccounts),
            BootstrapState::SettingUpNATS(NatsSetupStep::CreatingUsers),
            BootstrapState::ProvisioningYubiKeys,
            BootstrapState::Completed,
        ];

        for expected_state in states {
            saga.advance();
            assert_eq!(saga.state, expected_state);
        }

        assert!(saga.is_completed());
        assert!(saga.is_terminal());
    }

    #[test]
    fn test_saga_failure_and_compensation() {
        let org = create_test_org();
        let mut saga = CompleteBootstrapSaga::new(Uuid::now_v7())
            .with_organization(org);

        saga.start().unwrap();
        saga.advance(); // Move to AddingPeople
        saga.advance(); // Move to GeneratingPKI

        // Simulate failure
        saga.fail("Root CA generation failed", "GeneratingPKI:GeneratingRootCA");

        assert!(saga.is_failed());
        assert!(saga.error.is_some());

        // Start compensation
        let comp_step = saga.start_compensation();
        assert!(matches!(comp_step, CompensationStep::RollbackPKI));
    }

    #[test]
    fn test_record_pki_progress() {
        let mut saga = CompleteBootstrapSaga::new(Uuid::now_v7());

        let root_cert_id = CertificateId::new();
        let root_key_id = KeyId::new();
        saga.record_root_ca(root_cert_id, root_key_id);

        assert_eq!(saga.pki.root_ca_id, Some(root_cert_id));
        assert!(saga.pki.key_ids.contains(&root_key_id));

        let unit_id = UnitId::new();
        let int_cert_id = CertificateId::new();
        let int_key_id = KeyId::new();
        saga.record_intermediate_ca(unit_id, int_cert_id, int_key_id);

        assert_eq!(saga.pki.intermediate_ca_ids.get(&unit_id), Some(&int_cert_id));
    }

    #[test]
    fn test_record_nats_progress() {
        let mut saga = CompleteBootstrapSaga::new(Uuid::now_v7());

        let op_id = NatsOperatorId::new();
        saga.record_operator(op_id);
        assert_eq!(saga.nats.operator_id, Some(op_id));

        let sys_acc_id = NatsAccountId::new();
        saga.record_system_account(sys_acc_id);
        assert_eq!(saga.nats.system_account_id, Some(sys_acc_id));

        let unit_id = UnitId::new();
        let acc_id = NatsAccountId::new();
        saga.record_account(unit_id, acc_id);
        assert_eq!(saga.nats.account_ids.get(&unit_id), Some(&acc_id));
    }
}
