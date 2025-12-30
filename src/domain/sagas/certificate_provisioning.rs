// Copyright (c) 2025 - Cowboy AI, LLC.

//! Certificate Provisioning Saga
//!
//! Coordinates the provisioning of a certificate to hardware:
//! 1. Generate cryptographic key pair
//! 2. Generate certificate signed by appropriate CA
//! 3. Provision key and certificate to YubiKey slot
//! 4. Verify provisioning success
//!
//! This saga is used when provisioning certificates to YubiKeys outside
//! of the full person onboarding flow (e.g., certificate renewal, additional
//! certificates for existing users).
//!
//! ## State Machine
//!
//! ```text
//! Initial → GeneratingKey → GeneratingCertificate → ProvisioningToYubiKey
//!     ↓           ↓                  ↓                      ↓
//!   Failed     Failed             Failed                 Failed
//!                                                           ↓
//!                                                 VerifyingProvisioning → Completed
//!                                                          ↓
//!                                                       Failed
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{SagaState, SagaError};
use crate::domain::ids::*;
use crate::events::{KeyAlgorithm, KeyPurpose};
use crate::domain::yubikey::PIVSlot;

/// Certificate Provisioning Saga state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateProvisioningSaga {
    /// Unique saga ID
    pub saga_id: Uuid,
    /// Correlation ID for all events
    pub correlation_id: Uuid,
    /// Current state
    pub state: ProvisioningState,
    /// State at which failure occurred (for compensation)
    failed_at_state: Option<ProvisioningState>,
    /// Started at timestamp
    pub started_at: DateTime<Utc>,
    /// Completed at timestamp (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Provisioning request details
    pub request: ProvisioningRequest,
    /// Generated artifacts
    pub artifacts: ProvisioningArtifacts,
    /// Error if failed
    pub error: Option<SagaError>,
}

/// Provisioning state machine states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProvisioningState {
    /// Saga not started
    Initial,
    /// Generating cryptographic key pair
    GeneratingKey,
    /// Generating certificate
    GeneratingCertificate,
    /// Provisioning to YubiKey
    ProvisioningToYubiKey,
    /// Verifying the provisioning
    VerifyingProvisioning,
    /// Successfully completed
    Completed,
    /// Failed (see error field)
    Failed,
    /// Compensating (rolling back)
    Compensating(ProvisioningCompensationStep),
}

/// Compensation sub-steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProvisioningCompensationStep {
    /// Clear YubiKey slot
    ClearYubiKeySlot,
    /// Revoke certificate
    RevokeCertificate,
    /// Revoke key
    RevokeKey,
}

/// Provisioning request details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningRequest {
    /// Organization ID
    pub organization_id: BootstrapOrgId,
    /// Person ID the certificate is for
    pub person_id: Uuid,
    /// Person name (for certificate subject)
    pub person_name: String,
    /// Person email (for certificate SAN)
    pub person_email: String,
    /// Certificate purpose
    pub purpose: CertificatePurpose,
    /// Key algorithm to use
    pub key_algorithm: KeyAlgorithm,
    /// Issuing CA certificate ID
    pub issuing_ca_id: CertificateId,
    /// YubiKey device ID
    pub yubikey_device_id: YubiKeyDeviceId,
    /// YubiKey serial
    pub yubikey_serial: String,
    /// Target PIV slot
    pub target_slot: PIVSlot,
    /// Certificate validity in days
    pub validity_days: u32,
}

/// Certificate purpose determines key usage and extensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CertificatePurpose {
    /// Authentication (TLS client auth, SSH)
    Authentication,
    /// Digital signature (code signing, document signing)
    DigitalSignature,
    /// Key management (key agreement, encryption)
    KeyManagement,
    /// Card authentication
    CardAuthentication,
}

impl CertificatePurpose {
    /// Get the corresponding PIV slot for this purpose
    pub fn default_slot(&self) -> PIVSlot {
        match self {
            CertificatePurpose::Authentication => PIVSlot::Authentication,
            CertificatePurpose::DigitalSignature => PIVSlot::Signature,
            CertificatePurpose::KeyManagement => PIVSlot::KeyManagement,
            CertificatePurpose::CardAuthentication => PIVSlot::CardAuth,
        }
    }

    /// Convert to KeyPurpose for key generation
    pub fn to_key_purpose(&self) -> KeyPurpose {
        match self {
            CertificatePurpose::Authentication => KeyPurpose::Authentication,
            CertificatePurpose::DigitalSignature => KeyPurpose::Signing,
            CertificatePurpose::KeyManagement => KeyPurpose::KeyAgreement,
            CertificatePurpose::CardAuthentication => KeyPurpose::Authentication,
        }
    }

    /// Get X.509 key usage flags
    pub fn key_usage_flags(&self) -> Vec<&'static str> {
        match self {
            CertificatePurpose::Authentication => vec!["digitalSignature"],
            CertificatePurpose::DigitalSignature => vec!["digitalSignature", "nonRepudiation"],
            CertificatePurpose::KeyManagement => vec!["keyEncipherment", "keyAgreement"],
            CertificatePurpose::CardAuthentication => vec!["digitalSignature"],
        }
    }
}

/// Artifacts generated during provisioning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvisioningArtifacts {
    /// Generated key ID
    pub key_id: Option<KeyId>,
    /// Generated certificate ID
    pub certificate_id: Option<CertificateId>,
    /// Certificate fingerprint (for verification)
    pub certificate_fingerprint: Option<String>,
    /// Slot where certificate was provisioned
    pub provisioned_slot: Option<String>,
    /// Verification result
    pub verification_status: Option<VerificationStatus>,
}

/// Verification status after provisioning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Certificate verified in slot
    Verified,
    /// Certificate not found in slot
    NotFound,
    /// Certificate found but fingerprint mismatch
    FingerprintMismatch,
    /// Verification failed with error
    Error(String),
}

impl CertificateProvisioningSaga {
    /// Create a new certificate provisioning saga
    pub fn new(request: ProvisioningRequest) -> Self {
        Self {
            saga_id: Uuid::now_v7(),
            correlation_id: Uuid::now_v7(),
            state: ProvisioningState::Initial,
            failed_at_state: None,
            started_at: Utc::now(),
            completed_at: None,
            request,
            artifacts: ProvisioningArtifacts::default(),
            error: None,
        }
    }

    /// Create with explicit correlation ID
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    /// Start the saga
    pub fn start(&mut self) -> Result<(), SagaError> {
        // Validate request
        if self.request.person_name.is_empty() {
            return Err(SagaError::new("Person name required", "Initial"));
        }
        if self.request.yubikey_serial.is_empty() {
            return Err(SagaError::new("YubiKey serial required", "Initial"));
        }
        if self.request.validity_days == 0 {
            return Err(SagaError::new("Validity days must be > 0", "Initial"));
        }
        self.state = ProvisioningState::GeneratingKey;
        Ok(())
    }

    /// Transition to the next state
    pub fn advance(&mut self) -> ProvisioningState {
        self.state = match &self.state {
            ProvisioningState::Initial => ProvisioningState::GeneratingKey,
            ProvisioningState::GeneratingKey => ProvisioningState::GeneratingCertificate,
            ProvisioningState::GeneratingCertificate => ProvisioningState::ProvisioningToYubiKey,
            ProvisioningState::ProvisioningToYubiKey => ProvisioningState::VerifyingProvisioning,
            ProvisioningState::VerifyingProvisioning => {
                self.completed_at = Some(Utc::now());
                ProvisioningState::Completed
            }
            ProvisioningState::Completed => ProvisioningState::Completed,
            ProvisioningState::Failed => ProvisioningState::Failed,
            ProvisioningState::Compensating(_) => ProvisioningState::Failed,
        };
        self.state.clone()
    }

    /// Mark the saga as failed
    pub fn fail(&mut self, message: impl Into<String>, step: impl Into<String>) {
        self.failed_at_state = Some(self.state.clone());
        self.error = Some(SagaError::new(message, step));
        self.state = ProvisioningState::Failed;
    }

    /// Start compensation
    pub fn start_compensation(&mut self) -> ProvisioningCompensationStep {
        // Use the state at which failure occurred, not the current (Failed) state
        let failed_state = self.failed_at_state.as_ref().unwrap_or(&self.state);
        let step = match failed_state {
            ProvisioningState::VerifyingProvisioning | ProvisioningState::ProvisioningToYubiKey => {
                ProvisioningCompensationStep::ClearYubiKeySlot
            }
            ProvisioningState::GeneratingCertificate => {
                ProvisioningCompensationStep::RevokeCertificate
            }
            ProvisioningState::GeneratingKey | _ => ProvisioningCompensationStep::RevokeKey,
        };
        self.state = ProvisioningState::Compensating(step.clone());
        step
    }

    /// Advance compensation to next step
    pub fn advance_compensation(&mut self) -> Option<ProvisioningCompensationStep> {
        if let ProvisioningState::Compensating(current) = &self.state {
            let next = match current {
                ProvisioningCompensationStep::ClearYubiKeySlot => {
                    Some(ProvisioningCompensationStep::RevokeCertificate)
                }
                ProvisioningCompensationStep::RevokeCertificate => {
                    Some(ProvisioningCompensationStep::RevokeKey)
                }
                ProvisioningCompensationStep::RevokeKey => None,
            };

            if let Some(step) = next.clone() {
                self.state = ProvisioningState::Compensating(step);
            } else {
                self.state = ProvisioningState::Failed;
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
    pub fn record_certificate(&mut self, cert_id: CertificateId, fingerprint: String) {
        self.artifacts.certificate_id = Some(cert_id);
        self.artifacts.certificate_fingerprint = Some(fingerprint);
    }

    /// Record YubiKey provisioning
    pub fn record_provisioning(&mut self, slot: String) {
        self.artifacts.provisioned_slot = Some(slot);
    }

    /// Record verification result
    pub fn record_verification(&mut self, status: VerificationStatus) {
        self.artifacts.verification_status = Some(status);
    }

    /// Check if verification passed
    pub fn is_verification_successful(&self) -> bool {
        matches!(
            self.artifacts.verification_status,
            Some(VerificationStatus::Verified)
        )
    }

    /// Get certificate subject based on request
    pub fn certificate_subject(&self) -> String {
        format!(
            "CN={}, emailAddress={}",
            self.request.person_name, self.request.person_email
        )
    }

    /// Get current step name for logging
    pub fn current_step_name(&self) -> String {
        match &self.state {
            ProvisioningState::Initial => "Initial".to_string(),
            ProvisioningState::GeneratingKey => "GeneratingKey".to_string(),
            ProvisioningState::GeneratingCertificate => "GeneratingCertificate".to_string(),
            ProvisioningState::ProvisioningToYubiKey => "ProvisioningToYubiKey".to_string(),
            ProvisioningState::VerifyingProvisioning => "VerifyingProvisioning".to_string(),
            ProvisioningState::Completed => "Completed".to_string(),
            ProvisioningState::Failed => "Failed".to_string(),
            ProvisioningState::Compensating(step) => format!("Compensating:{:?}", step),
        }
    }
}

impl SagaState for CertificateProvisioningSaga {
    fn saga_id(&self) -> Uuid {
        self.saga_id
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            ProvisioningState::Completed | ProvisioningState::Failed
        )
    }

    fn is_completed(&self) -> bool {
        matches!(self.state, ProvisioningState::Completed)
    }

    fn is_failed(&self) -> bool {
        matches!(self.state, ProvisioningState::Failed)
    }

    fn status_description(&self) -> String {
        match &self.state {
            ProvisioningState::Initial => "Not started".to_string(),
            ProvisioningState::GeneratingKey => {
                format!(
                    "Generating {:?} key for {}",
                    self.request.key_algorithm, self.request.person_name
                )
            }
            ProvisioningState::GeneratingCertificate => {
                format!(
                    "Generating {:?} certificate for {}",
                    self.request.purpose, self.request.person_name
                )
            }
            ProvisioningState::ProvisioningToYubiKey => {
                format!(
                    "Provisioning to YubiKey {} slot {:?}",
                    self.request.yubikey_serial, self.request.target_slot
                )
            }
            ProvisioningState::VerifyingProvisioning => "Verifying provisioning".to_string(),
            ProvisioningState::Completed => {
                format!(
                    "Successfully provisioned certificate to YubiKey {} slot {:?}",
                    self.request.yubikey_serial, self.request.target_slot
                )
            }
            ProvisioningState::Failed => format!(
                "Provisioning failed: {}",
                self.error.as_ref().map_or("Unknown error", |e| &e.message)
            ),
            ProvisioningState::Compensating(step) => format!("Rolling back: {:?}", step),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request() -> ProvisioningRequest {
        ProvisioningRequest {
            organization_id: BootstrapOrgId::new(),
            person_id: Uuid::now_v7(),
            person_name: "Alice Engineer".to_string(),
            person_email: "alice@test.com".to_string(),
            purpose: CertificatePurpose::Authentication,
            key_algorithm: KeyAlgorithm::Ecdsa { curve: "P-256".to_string() },
            issuing_ca_id: CertificateId::new(),
            yubikey_device_id: YubiKeyDeviceId::new(),
            yubikey_serial: "12345678".to_string(),
            target_slot: PIVSlot::Authentication,
            validity_days: 365,
        }
    }

    #[test]
    fn test_saga_creation() {
        let request = create_test_request();
        let saga = CertificateProvisioningSaga::new(request);

        assert_eq!(saga.state, ProvisioningState::Initial);
        assert!(!saga.is_terminal());
    }

    #[test]
    fn test_saga_start_validation() {
        let mut request = create_test_request();
        request.person_name = "".to_string();

        let mut saga = CertificateProvisioningSaga::new(request);
        assert!(saga.start().is_err());

        let mut request2 = create_test_request();
        request2.yubikey_serial = "".to_string();

        let mut saga2 = CertificateProvisioningSaga::new(request2);
        assert!(saga2.start().is_err());

        let mut request3 = create_test_request();
        request3.validity_days = 0;

        let mut saga3 = CertificateProvisioningSaga::new(request3);
        assert!(saga3.start().is_err());
    }

    #[test]
    fn test_saga_full_flow() {
        let request = create_test_request();
        let mut saga = CertificateProvisioningSaga::new(request);

        saga.start().unwrap();

        let expected_states = [
            ProvisioningState::GeneratingCertificate,
            ProvisioningState::ProvisioningToYubiKey,
            ProvisioningState::VerifyingProvisioning,
            ProvisioningState::Completed,
        ];

        for expected in expected_states {
            saga.advance();
            assert_eq!(saga.state, expected);
        }

        assert!(saga.is_completed());
    }

    #[test]
    fn test_certificate_purpose_slots() {
        assert_eq!(
            CertificatePurpose::Authentication.default_slot(),
            PIVSlot::Authentication
        );
        assert_eq!(
            CertificatePurpose::DigitalSignature.default_slot(),
            PIVSlot::Signature
        );
        assert_eq!(
            CertificatePurpose::KeyManagement.default_slot(),
            PIVSlot::KeyManagement
        );
        assert_eq!(
            CertificatePurpose::CardAuthentication.default_slot(),
            PIVSlot::CardAuth
        );
    }

    #[test]
    fn test_key_usage_flags() {
        let auth_flags = CertificatePurpose::Authentication.key_usage_flags();
        assert!(auth_flags.contains(&"digitalSignature"));

        let sign_flags = CertificatePurpose::DigitalSignature.key_usage_flags();
        assert!(sign_flags.contains(&"digitalSignature"));
        assert!(sign_flags.contains(&"nonRepudiation"));

        let keymgmt_flags = CertificatePurpose::KeyManagement.key_usage_flags();
        assert!(keymgmt_flags.contains(&"keyEncipherment"));
        assert!(keymgmt_flags.contains(&"keyAgreement"));
    }

    #[test]
    fn test_saga_compensation() {
        let request = create_test_request();
        let mut saga = CertificateProvisioningSaga::new(request);

        saga.start().unwrap();
        saga.advance(); // GeneratingCertificate
        saga.advance(); // ProvisioningToYubiKey

        // Fail during provisioning
        saga.fail("YubiKey communication error", "ProvisioningToYubiKey");
        assert!(saga.is_failed());

        // Start compensation - should start with clearing slot
        let step = saga.start_compensation();
        assert_eq!(step, ProvisioningCompensationStep::ClearYubiKeySlot);

        // Continue compensation
        let step = saga.advance_compensation();
        assert_eq!(step, Some(ProvisioningCompensationStep::RevokeCertificate));

        let step = saga.advance_compensation();
        assert_eq!(step, Some(ProvisioningCompensationStep::RevokeKey));

        let step = saga.advance_compensation();
        assert_eq!(step, None);
    }

    #[test]
    fn test_record_artifacts() {
        let request = create_test_request();
        let mut saga = CertificateProvisioningSaga::new(request);

        let key_id = KeyId::new();
        saga.record_key(key_id);
        assert_eq!(saga.artifacts.key_id, Some(key_id));

        let cert_id = CertificateId::new();
        saga.record_certificate(cert_id, "SHA256:abc123".to_string());
        assert_eq!(saga.artifacts.certificate_id, Some(cert_id));
        assert_eq!(
            saga.artifacts.certificate_fingerprint,
            Some("SHA256:abc123".to_string())
        );

        saga.record_provisioning("9A".to_string());
        assert_eq!(saga.artifacts.provisioned_slot, Some("9A".to_string()));

        saga.record_verification(VerificationStatus::Verified);
        assert!(saga.is_verification_successful());
    }

    #[test]
    fn test_certificate_subject() {
        let request = create_test_request();
        let saga = CertificateProvisioningSaga::new(request);

        let subject = saga.certificate_subject();
        assert!(subject.contains("CN=Alice Engineer"));
        assert!(subject.contains("emailAddress=alice@test.com"));
    }
}
