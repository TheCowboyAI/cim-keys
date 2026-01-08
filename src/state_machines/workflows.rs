// Workflow State Machines for CIM Keys
//
// This module defines explicit state machines for multi-aggregate workflows,
// ensuring proper sequencing and validation of complex operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::events::{KeyAlgorithm, KeyPurpose};

// ============================================================================
// Supporting Types
// ============================================================================

/// Location identifier (references a Location entity)
pub type LocationId = Uuid;

/// Certificate subject information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CertificateSubject {
    pub common_name: String,
    pub organization: String,
    pub organizational_unit: Option<String>,
    pub country: String,
    pub state: Option<String>,
    pub locality: Option<String>,
    pub email: Option<String>,
}

/// PIV slot on YubiKey
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PivSlot {
    /// 9a - PIV Authentication
    Authentication,
    /// 9c - Digital Signature
    Signature,
    /// 9d - Key Management
    KeyManagement,
    /// 9e - Card Authentication
    CardAuth,
    /// 82-95 - Retired slots
    Retired(u8),
}

impl PivSlot {
    /// Get the slot hex identifier
    pub fn hex(&self) -> String {
        match self {
            PivSlot::Authentication => "9a".to_string(),
            PivSlot::Signature => "9c".to_string(),
            PivSlot::KeyManagement => "9d".to_string(),
            PivSlot::CardAuth => "9e".to_string(),
            PivSlot::Retired(n) => format!("{:x}", 0x82 + n),
        }
    }
}

// ============================================================================
// PKI Bootstrap State Machine
// ============================================================================

/// State machine for PKI bootstrap workflow
///
/// This state machine ensures proper sequencing of PKI generation:
/// 1. Root CA must be generated offline (air-gapped, YubiKey ceremony)
/// 2. Intermediate CA must be signed by Root CA
/// 3. Leaf certificates must be signed by Intermediate CA
/// 4. Export can only happen when all artifacts are generated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PKIBootstrapState {
    /// Initial state - no PKI infrastructure exists
    Uninitialized,

    /// Root CA has been planned but not yet generated
    RootCAPlanned {
        subject: CertificateSubject,
        validity_years: u32,
        yubikey_serial: String,
    },

    /// Root CA has been generated (OFFLINE ceremony complete)
    RootCAGenerated {
        root_ca_cert_id: Uuid,
        root_ca_key_id: Uuid,
        generated_at: DateTime<Utc>,
    },

    /// Intermediate CA has been planned
    IntermediateCAPlanned {
        subject: CertificateSubject,
        validity_years: u32,
        path_len: Option<u32>,
    },

    /// Intermediate CA(s) have been generated
    IntermediateCAGenerated {
        intermediate_ca_ids: Vec<Uuid>,
    },

    /// Leaf certificates have been generated
    LeafCertsGenerated {
        leaf_cert_ids: Vec<Uuid>,
    },

    /// YubiKeys have been provisioned with keys
    YubiKeysProvisioned {
        yubikey_serials: Vec<String>,
    },

    /// Export is ready to proceed
    ExportReady {
        manifest_id: Uuid,
    },

    /// Bootstrap complete - exported to secure storage
    Bootstrapped {
        export_location: Uuid,
        export_checksum: String,
        bootstrapped_at: DateTime<Utc>,
    },
}

impl PKIBootstrapState {
    /// Can we plan a root CA?
    pub fn can_plan_root_ca(&self) -> bool {
        matches!(self, PKIBootstrapState::Uninitialized)
    }

    /// Can we generate a root CA?
    pub fn can_generate_root_ca(&self) -> bool {
        matches!(self, PKIBootstrapState::RootCAPlanned { .. })
    }

    /// Can we plan an intermediate CA?
    pub fn can_plan_intermediate_ca(&self) -> bool {
        matches!(
            self,
            PKIBootstrapState::RootCAGenerated { .. }
                | PKIBootstrapState::IntermediateCAPlanned { .. }
        )
    }

    /// Can we generate an intermediate CA?
    pub fn can_generate_intermediate_ca(&self) -> bool {
        matches!(
            self,
            PKIBootstrapState::RootCAGenerated { .. }
                | PKIBootstrapState::IntermediateCAPlanned { .. }
        )
    }

    /// Can we generate leaf certificates?
    pub fn can_generate_leaf_cert(&self) -> bool {
        matches!(
            self,
            PKIBootstrapState::IntermediateCAGenerated { .. }
                | PKIBootstrapState::LeafCertsGenerated { .. }
        )
    }

    /// Can we provision YubiKeys?
    pub fn can_provision_yubikey(&self) -> bool {
        matches!(
            self,
            PKIBootstrapState::LeafCertsGenerated { .. }
                | PKIBootstrapState::YubiKeysProvisioned { .. }
        )
    }

    /// Can we prepare export?
    pub fn can_prepare_export(&self) -> bool {
        matches!(
            self,
            PKIBootstrapState::YubiKeysProvisioned { .. }
        )
    }

    /// Can we execute export?
    pub fn can_export(&self) -> bool {
        matches!(self, PKIBootstrapState::ExportReady { .. })
    }

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            PKIBootstrapState::Uninitialized => "PKI infrastructure not initialized",
            PKIBootstrapState::RootCAPlanned { .. } => "Root CA planned, awaiting generation",
            PKIBootstrapState::RootCAGenerated { .. } => "Root CA generated (offline ceremony complete)",
            PKIBootstrapState::IntermediateCAPlanned { .. } => "Intermediate CA planned",
            PKIBootstrapState::IntermediateCAGenerated { .. } => "Intermediate CA(s) generated",
            PKIBootstrapState::LeafCertsGenerated { .. } => "Leaf certificates generated",
            PKIBootstrapState::YubiKeysProvisioned { .. } => "YubiKeys provisioned",
            PKIBootstrapState::ExportReady { .. } => "Export manifest ready",
            PKIBootstrapState::Bootstrapped { .. } => "Bootstrap complete",
        }
    }

    /// Get the state name for viewmodel display
    ///
    /// Returns the enum variant name (without data) for matching
    /// with StateMachineDefinition state names.
    pub fn state_name(&self) -> &'static str {
        match self {
            PKIBootstrapState::Uninitialized => "Uninitialized",
            PKIBootstrapState::RootCAPlanned { .. } => "RootCAPlanned",
            PKIBootstrapState::RootCAGenerated { .. } => "RootCAGenerated",
            PKIBootstrapState::IntermediateCAPlanned { .. } => "IntermediateCAPlanned",
            PKIBootstrapState::IntermediateCAGenerated { .. } => "IntermediateCAGenerated",
            PKIBootstrapState::LeafCertsGenerated { .. } => "LeafCertsGenerated",
            PKIBootstrapState::YubiKeysProvisioned { .. } => "YubiKeysProvisioned",
            PKIBootstrapState::ExportReady { .. } => "ExportReady",
            PKIBootstrapState::Bootstrapped { .. } => "Bootstrapped",
        }
    }

    // ========================================================================
    // Terminal State Detection
    // ========================================================================

    /// Is the workflow in a terminal state?
    pub fn is_terminal(&self) -> bool {
        matches!(self, PKIBootstrapState::Bootstrapped { .. })
    }

    /// Is the workflow complete?
    pub fn is_complete(&self) -> bool {
        matches!(self, PKIBootstrapState::Bootstrapped { .. })
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Validate if a transition to the target state is allowed
    pub fn can_transition_to(&self, target: &PKIBootstrapState) -> bool {
        match (self, target) {
            // Uninitialized → RootCAPlanned
            (PKIBootstrapState::Uninitialized, PKIBootstrapState::RootCAPlanned { .. }) => true,

            // RootCAPlanned → RootCAGenerated
            (PKIBootstrapState::RootCAPlanned { .. }, PKIBootstrapState::RootCAGenerated { .. }) => true,

            // RootCAGenerated → IntermediateCAPlanned
            (PKIBootstrapState::RootCAGenerated { .. }, PKIBootstrapState::IntermediateCAPlanned { .. }) => true,

            // RootCAGenerated → IntermediateCAGenerated (skip planning, generate directly)
            (PKIBootstrapState::RootCAGenerated { .. }, PKIBootstrapState::IntermediateCAGenerated { .. }) => true,

            // IntermediateCAPlanned → IntermediateCAGenerated
            (PKIBootstrapState::IntermediateCAPlanned { .. }, PKIBootstrapState::IntermediateCAGenerated { .. }) => true,

            // IntermediateCAGenerated → LeafCertsGenerated
            (PKIBootstrapState::IntermediateCAGenerated { .. }, PKIBootstrapState::LeafCertsGenerated { .. }) => true,

            // LeafCertsGenerated → LeafCertsGenerated (add more leaf certs)
            (PKIBootstrapState::LeafCertsGenerated { .. }, PKIBootstrapState::LeafCertsGenerated { .. }) => true,

            // LeafCertsGenerated → YubiKeysProvisioned
            (PKIBootstrapState::LeafCertsGenerated { .. }, PKIBootstrapState::YubiKeysProvisioned { .. }) => true,

            // YubiKeysProvisioned → YubiKeysProvisioned (provision more YubiKeys)
            (PKIBootstrapState::YubiKeysProvisioned { .. }, PKIBootstrapState::YubiKeysProvisioned { .. }) => true,

            // YubiKeysProvisioned → ExportReady
            (PKIBootstrapState::YubiKeysProvisioned { .. }, PKIBootstrapState::ExportReady { .. }) => true,

            // ExportReady → Bootstrapped
            (PKIBootstrapState::ExportReady { .. }, PKIBootstrapState::Bootstrapped { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Plan root CA generation
    pub fn plan_root_ca(
        &self,
        subject: CertificateSubject,
        validity_years: u32,
        yubikey_serial: String,
    ) -> Result<PKIBootstrapState, PKIBootstrapError> {
        if !self.can_plan_root_ca() {
            return Err(PKIBootstrapError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "PlanRootCA".to_string(),
                reason: "Can only plan root CA from Uninitialized state".to_string(),
            });
        }

        Ok(PKIBootstrapState::RootCAPlanned {
            subject,
            validity_years,
            yubikey_serial,
        })
    }

    /// Generate root CA (offline ceremony)
    pub fn generate_root_ca(
        &self,
        root_ca_cert_id: Uuid,
        root_ca_key_id: Uuid,
        generated_at: DateTime<Utc>,
    ) -> Result<PKIBootstrapState, PKIBootstrapError> {
        if !self.can_generate_root_ca() {
            return Err(PKIBootstrapError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "GenerateRootCA".to_string(),
                reason: "Can only generate root CA from RootCAPlanned state".to_string(),
            });
        }

        Ok(PKIBootstrapState::RootCAGenerated {
            root_ca_cert_id,
            root_ca_key_id,
            generated_at,
        })
    }

    /// Plan intermediate CA
    pub fn plan_intermediate_ca(
        &self,
        subject: CertificateSubject,
        validity_years: u32,
        path_len: Option<u32>,
    ) -> Result<PKIBootstrapState, PKIBootstrapError> {
        if !self.can_plan_intermediate_ca() {
            return Err(PKIBootstrapError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "PlanIntermediateCA".to_string(),
                reason: "Can only plan intermediate CA from RootCAGenerated state".to_string(),
            });
        }

        Ok(PKIBootstrapState::IntermediateCAPlanned {
            subject,
            validity_years,
            path_len,
        })
    }

    /// Generate intermediate CA
    pub fn generate_intermediate_ca(
        &self,
        intermediate_ca_ids: Vec<Uuid>,
    ) -> Result<PKIBootstrapState, PKIBootstrapError> {
        if !self.can_generate_intermediate_ca() {
            return Err(PKIBootstrapError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "GenerateIntermediateCA".to_string(),
                reason: "Can only generate intermediate CA from RootCAGenerated or IntermediateCAPlanned state".to_string(),
            });
        }

        if intermediate_ca_ids.is_empty() {
            return Err(PKIBootstrapError::ValidationFailed(
                "At least one intermediate CA ID required".to_string(),
            ));
        }

        Ok(PKIBootstrapState::IntermediateCAGenerated {
            intermediate_ca_ids,
        })
    }

    /// Generate leaf certificates
    pub fn generate_leaf_certs(
        &self,
        leaf_cert_ids: Vec<Uuid>,
    ) -> Result<PKIBootstrapState, PKIBootstrapError> {
        if !self.can_generate_leaf_cert() {
            return Err(PKIBootstrapError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "GenerateLeafCerts".to_string(),
                reason: "Can only generate leaf certs from IntermediateCAGenerated or LeafCertsGenerated state".to_string(),
            });
        }

        if leaf_cert_ids.is_empty() {
            return Err(PKIBootstrapError::ValidationFailed(
                "At least one leaf certificate ID required".to_string(),
            ));
        }

        Ok(PKIBootstrapState::LeafCertsGenerated { leaf_cert_ids })
    }

    /// Provision YubiKeys
    pub fn provision_yubikeys(
        &self,
        yubikey_serials: Vec<String>,
    ) -> Result<PKIBootstrapState, PKIBootstrapError> {
        if !self.can_provision_yubikey() {
            return Err(PKIBootstrapError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "ProvisionYubiKeys".to_string(),
                reason: "Can only provision YubiKeys from LeafCertsGenerated or YubiKeysProvisioned state".to_string(),
            });
        }

        if yubikey_serials.is_empty() {
            return Err(PKIBootstrapError::ValidationFailed(
                "At least one YubiKey serial required".to_string(),
            ));
        }

        Ok(PKIBootstrapState::YubiKeysProvisioned { yubikey_serials })
    }

    /// Prepare export manifest
    pub fn prepare_export(
        &self,
        manifest_id: Uuid,
    ) -> Result<PKIBootstrapState, PKIBootstrapError> {
        if !self.can_prepare_export() {
            return Err(PKIBootstrapError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "PrepareExport".to_string(),
                reason: "Can only prepare export from YubiKeysProvisioned state".to_string(),
            });
        }

        Ok(PKIBootstrapState::ExportReady { manifest_id })
    }

    /// Complete bootstrap with export
    pub fn complete_bootstrap(
        &self,
        export_location: Uuid,
        export_checksum: String,
        bootstrapped_at: DateTime<Utc>,
    ) -> Result<PKIBootstrapState, PKIBootstrapError> {
        if !self.can_export() {
            return Err(PKIBootstrapError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "CompleteBootstrap".to_string(),
                reason: "Can only complete bootstrap from ExportReady state".to_string(),
            });
        }

        Ok(PKIBootstrapState::Bootstrapped {
            export_location,
            export_checksum,
            bootstrapped_at,
        })
    }
}

/// Errors for PKI bootstrap state transitions
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PKIBootstrapError {
    #[error("Invalid state transition from {current} on event {event}: {reason}")]
    InvalidTransition {
        current: String,
        event: String,
        reason: String,
    },

    #[error("Terminal state reached: {0}")]
    TerminalState(String),

    #[error("State validation failed: {0}")]
    ValidationFailed(String),
}

// ============================================================================
// YubiKey Provisioning State Machine
// ============================================================================

/// State machine for YubiKey provisioning workflow
///
/// This state machine ensures proper YubiKey initialization:
/// 1. Detect and authenticate to the YubiKey
/// 2. Change default PIN
/// 3. Rotate default management key
/// 4. Plan slot allocation
/// 5. Generate keys in slots
/// 6. Import certificates
/// 7. Attest keys (prove on-device generation)
/// 8. Seal configuration (lock final state)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum YubiKeyProvisioningState {
    /// YubiKey detected, serial number read
    Detected {
        serial: String,
        firmware_version: String,
    },

    /// Authenticated with current PIN
    Authenticated {
        pin_retries_remaining: u8,
    },

    /// PIN changed from default
    PINChanged {
        new_pin_hash: String, // SHA-256 hash for audit trail
    },

    /// Management key rotated from default
    ManagementKeyRotated {
        algorithm: PivAlgorithm,
    },

    /// Slot allocation planned
    SlotPlanned {
        slot_plan: HashMap<PivSlot, SlotPlan>,
    },

    /// Keys generated in slots
    KeysGenerated {
        slot_keys: HashMap<PivSlot, Vec<u8>>, // public key bytes
    },

    /// Certificates imported to slots
    CertificatesImported {
        slot_certs: HashMap<PivSlot, Uuid>, // certificate IDs
    },

    /// Keys attested (verified on-device generation)
    Attested {
        attestation_chain_verified: bool,
        attestation_cert_ids: Vec<Uuid>,
    },

    /// Configuration sealed (final, immutable)
    Sealed {
        sealed_at: DateTime<Utc>,
        final_config_hash: String,
    },
}

impl YubiKeyProvisioningState {
    /// Can we authenticate?
    pub fn can_authenticate(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::Detected { .. })
    }

    /// Can we change PIN?
    pub fn can_change_pin(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::Authenticated { .. })
    }

    /// Can we rotate management key?
    pub fn can_rotate_management_key(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::PINChanged { .. })
    }

    /// Can we plan slot allocation?
    pub fn can_plan_slots(&self) -> bool {
        matches!(
            self,
            YubiKeyProvisioningState::ManagementKeyRotated { .. }
        )
    }

    /// Can we generate keys?
    pub fn can_generate_keys(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::SlotPlanned { .. })
    }

    /// Can we import certificates?
    pub fn can_import_certs(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::KeysGenerated { .. })
    }

    /// Can we attest keys?
    pub fn can_attest(&self) -> bool {
        matches!(
            self,
            YubiKeyProvisioningState::CertificatesImported { .. }
        )
    }

    /// Can we seal configuration?
    pub fn can_seal(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::Attested { .. })
    }

    /// Is the YubiKey sealed (immutable)?
    pub fn is_sealed(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::Sealed { .. })
    }

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            YubiKeyProvisioningState::Detected { .. } => "YubiKey detected",
            YubiKeyProvisioningState::Authenticated { .. } => "Authenticated with PIN",
            YubiKeyProvisioningState::PINChanged { .. } => "PIN changed",
            YubiKeyProvisioningState::ManagementKeyRotated { .. } => "Management key rotated",
            YubiKeyProvisioningState::SlotPlanned { .. } => "Slot allocation planned",
            YubiKeyProvisioningState::KeysGenerated { .. } => "Keys generated in slots",
            YubiKeyProvisioningState::CertificatesImported { .. } => "Certificates imported",
            YubiKeyProvisioningState::Attested { .. } => "Keys attested",
            YubiKeyProvisioningState::Sealed { .. } => "Configuration sealed",
        }
    }

    /// Get the state name for viewmodel display
    ///
    /// Returns the enum variant name (without data) for matching
    /// with StateMachineDefinition state names.
    pub fn state_name(&self) -> &'static str {
        match self {
            YubiKeyProvisioningState::Detected { .. } => "Detected",
            YubiKeyProvisioningState::Authenticated { .. } => "Authenticated",
            YubiKeyProvisioningState::PINChanged { .. } => "PINChanged",
            YubiKeyProvisioningState::ManagementKeyRotated { .. } => "ManagementKeyRotated",
            YubiKeyProvisioningState::SlotPlanned { .. } => "SlotPlanned",
            YubiKeyProvisioningState::KeysGenerated { .. } => "KeysGenerated",
            YubiKeyProvisioningState::CertificatesImported { .. } => "CertificatesImported",
            YubiKeyProvisioningState::Attested { .. } => "Attested",
            YubiKeyProvisioningState::Sealed { .. } => "Sealed",
        }
    }

    // ========================================================================
    // Terminal State Detection
    // ========================================================================

    /// Is the provisioning in a terminal state?
    pub fn is_terminal(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::Sealed { .. })
    }

    /// Is the provisioning complete?
    pub fn is_complete(&self) -> bool {
        matches!(self, YubiKeyProvisioningState::Sealed { .. })
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Validate if a transition to the target state is allowed
    pub fn can_transition_to(&self, target: &YubiKeyProvisioningState) -> bool {
        match (self, target) {
            // Detected → Authenticated
            (
                YubiKeyProvisioningState::Detected { .. },
                YubiKeyProvisioningState::Authenticated { .. },
            ) => true,

            // Authenticated → PINChanged
            (
                YubiKeyProvisioningState::Authenticated { .. },
                YubiKeyProvisioningState::PINChanged { .. },
            ) => true,

            // PINChanged → ManagementKeyRotated
            (
                YubiKeyProvisioningState::PINChanged { .. },
                YubiKeyProvisioningState::ManagementKeyRotated { .. },
            ) => true,

            // ManagementKeyRotated → SlotPlanned
            (
                YubiKeyProvisioningState::ManagementKeyRotated { .. },
                YubiKeyProvisioningState::SlotPlanned { .. },
            ) => true,

            // SlotPlanned → KeysGenerated
            (
                YubiKeyProvisioningState::SlotPlanned { .. },
                YubiKeyProvisioningState::KeysGenerated { .. },
            ) => true,

            // KeysGenerated → CertificatesImported
            (
                YubiKeyProvisioningState::KeysGenerated { .. },
                YubiKeyProvisioningState::CertificatesImported { .. },
            ) => true,

            // CertificatesImported → Attested
            (
                YubiKeyProvisioningState::CertificatesImported { .. },
                YubiKeyProvisioningState::Attested { .. },
            ) => true,

            // Attested → Sealed
            (
                YubiKeyProvisioningState::Attested { .. },
                YubiKeyProvisioningState::Sealed { .. },
            ) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Authenticate to the YubiKey
    pub fn authenticate(
        &self,
        pin_retries_remaining: u8,
    ) -> Result<YubiKeyProvisioningState, YubiKeyProvisioningError> {
        if !self.can_authenticate() {
            return Err(YubiKeyProvisioningError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "Authenticate".to_string(),
                reason: "Can only authenticate from Detected state".to_string(),
            });
        }

        Ok(YubiKeyProvisioningState::Authenticated {
            pin_retries_remaining,
        })
    }

    /// Change the PIN
    pub fn change_pin(
        &self,
        new_pin_hash: String,
    ) -> Result<YubiKeyProvisioningState, YubiKeyProvisioningError> {
        if !self.can_change_pin() {
            return Err(YubiKeyProvisioningError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "ChangePIN".to_string(),
                reason: "Can only change PIN from Authenticated state".to_string(),
            });
        }

        if new_pin_hash.is_empty() {
            return Err(YubiKeyProvisioningError::ValidationFailed(
                "PIN hash cannot be empty".to_string(),
            ));
        }

        Ok(YubiKeyProvisioningState::PINChanged { new_pin_hash })
    }

    /// Rotate the management key
    pub fn rotate_management_key(
        &self,
        algorithm: PivAlgorithm,
    ) -> Result<YubiKeyProvisioningState, YubiKeyProvisioningError> {
        if !self.can_rotate_management_key() {
            return Err(YubiKeyProvisioningError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "RotateManagementKey".to_string(),
                reason: "Can only rotate management key from PINChanged state".to_string(),
            });
        }

        Ok(YubiKeyProvisioningState::ManagementKeyRotated { algorithm })
    }

    /// Plan slot allocation
    pub fn plan_slots(
        &self,
        slot_plan: HashMap<PivSlot, SlotPlan>,
    ) -> Result<YubiKeyProvisioningState, YubiKeyProvisioningError> {
        if !self.can_plan_slots() {
            return Err(YubiKeyProvisioningError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "PlanSlots".to_string(),
                reason: "Can only plan slots from ManagementKeyRotated state".to_string(),
            });
        }

        if slot_plan.is_empty() {
            return Err(YubiKeyProvisioningError::ValidationFailed(
                "At least one slot must be planned".to_string(),
            ));
        }

        Ok(YubiKeyProvisioningState::SlotPlanned { slot_plan })
    }

    /// Generate keys in slots
    pub fn generate_keys(
        &self,
        slot_keys: HashMap<PivSlot, Vec<u8>>,
    ) -> Result<YubiKeyProvisioningState, YubiKeyProvisioningError> {
        if !self.can_generate_keys() {
            return Err(YubiKeyProvisioningError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "GenerateKeys".to_string(),
                reason: "Can only generate keys from SlotPlanned state".to_string(),
            });
        }

        if slot_keys.is_empty() {
            return Err(YubiKeyProvisioningError::ValidationFailed(
                "At least one key must be generated".to_string(),
            ));
        }

        Ok(YubiKeyProvisioningState::KeysGenerated { slot_keys })
    }

    /// Import certificates to slots
    pub fn import_certificates(
        &self,
        slot_certs: HashMap<PivSlot, Uuid>,
    ) -> Result<YubiKeyProvisioningState, YubiKeyProvisioningError> {
        if !self.can_import_certs() {
            return Err(YubiKeyProvisioningError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "ImportCertificates".to_string(),
                reason: "Can only import certificates from KeysGenerated state".to_string(),
            });
        }

        if slot_certs.is_empty() {
            return Err(YubiKeyProvisioningError::ValidationFailed(
                "At least one certificate must be imported".to_string(),
            ));
        }

        Ok(YubiKeyProvisioningState::CertificatesImported { slot_certs })
    }

    /// Attest keys
    pub fn attest_keys(
        &self,
        attestation_chain_verified: bool,
        attestation_cert_ids: Vec<Uuid>,
    ) -> Result<YubiKeyProvisioningState, YubiKeyProvisioningError> {
        if !self.can_attest() {
            return Err(YubiKeyProvisioningError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "AttestKeys".to_string(),
                reason: "Can only attest keys from CertificatesImported state".to_string(),
            });
        }

        Ok(YubiKeyProvisioningState::Attested {
            attestation_chain_verified,
            attestation_cert_ids,
        })
    }

    /// Seal the configuration (terminal state)
    pub fn seal(
        &self,
        sealed_at: DateTime<Utc>,
        final_config_hash: String,
    ) -> Result<YubiKeyProvisioningState, YubiKeyProvisioningError> {
        if !self.can_seal() {
            return Err(YubiKeyProvisioningError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "Seal".to_string(),
                reason: "Can only seal from Attested state".to_string(),
            });
        }

        if final_config_hash.is_empty() {
            return Err(YubiKeyProvisioningError::ValidationFailed(
                "Configuration hash cannot be empty".to_string(),
            ));
        }

        Ok(YubiKeyProvisioningState::Sealed {
            sealed_at,
            final_config_hash,
        })
    }
}

/// Errors for YubiKey provisioning state transitions
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum YubiKeyProvisioningError {
    #[error("Invalid state transition from {current} on event {event}: {reason}")]
    InvalidTransition {
        current: String,
        event: String,
        reason: String,
    },

    #[error("Terminal state reached: {0}")]
    TerminalState(String),

    #[error("State validation failed: {0}")]
    ValidationFailed(String),
}

/// Slot configuration plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlotPlan {
    pub purpose: KeyPurpose,
    pub algorithm: KeyAlgorithm,
    pub pin_policy: PinPolicy,
    pub touch_policy: TouchPolicy,
}

/// PIN policy for YubiKey slot
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PinPolicy {
    /// Never require PIN
    Never,
    /// Require PIN once per session
    Once,
    /// Always require PIN
    Always,
}

/// Touch policy for YubiKey slot
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TouchPolicy {
    /// Never require touch
    Never,
    /// Always require touch
    Always,
    /// Touch required but cached for 15 seconds
    Cached,
}

/// PIV algorithm selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PivAlgorithm {
    /// Ed25519 (signing)
    Ed25519,
    /// X25519 (encryption)
    X25519,
    /// ECDSA P-256
    EcdsaP256,
    /// RSA 2048
    Rsa2048,
    /// RSA 4096
    Rsa4096,
}

// ============================================================================
// Export Workflow State Machine
// ============================================================================

/// State machine for export workflow
///
/// This state machine ensures proper export sequencing:
/// 1. Plan what artifacts to export and where
/// 2. Generate all artifacts
/// 3. Encrypt sensitive data
/// 4. Write to target location
/// 5. Verify checksums
/// 6. Mark as complete
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExportWorkflowState {
    /// Planning export - selecting artifacts and location
    Planning {
        artifacts_to_export: Vec<ArtifactType>,
        target_location: LocationId,
    },

    /// Generating artifacts
    Generating {
        progress: HashMap<ArtifactType, GenerationProgress>,
    },

    /// Encrypting sensitive data
    Encrypting {
        encryption_key_id: Uuid,
        progress_percent: u8,
    },

    /// Writing to target location
    Writing {
        bytes_written: u64,
        total_bytes: u64,
    },

    /// Verifying checksums
    Verifying {
        checksums: HashMap<String, String>, // filename -> checksum
    },

    /// Export completed successfully
    Completed {
        manifest_checksum: String,
        exported_at: DateTime<Utc>,
    },

    /// Export failed
    Failed {
        error: String,
        failed_at: DateTime<Utc>,
    },
}

impl ExportWorkflowState {
    /// Can we start generating artifacts?
    pub fn can_generate(&self) -> bool {
        matches!(self, ExportWorkflowState::Planning { .. })
    }

    /// Can we start encrypting?
    pub fn can_encrypt(&self) -> bool {
        matches!(self, ExportWorkflowState::Generating { .. })
    }

    /// Can we start writing?
    pub fn can_write(&self) -> bool {
        matches!(self, ExportWorkflowState::Encrypting { .. })
    }

    /// Can we verify?
    pub fn can_verify(&self) -> bool {
        matches!(self, ExportWorkflowState::Writing { .. })
    }

    /// Is export complete?
    pub fn is_complete(&self) -> bool {
        matches!(self, ExportWorkflowState::Completed { .. })
    }

    /// Has export failed?
    pub fn has_failed(&self) -> bool {
        matches!(self, ExportWorkflowState::Failed { .. })
    }

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            ExportWorkflowState::Planning { .. } => "Planning export",
            ExportWorkflowState::Generating { .. } => "Generating artifacts",
            ExportWorkflowState::Encrypting { .. } => "Encrypting sensitive data",
            ExportWorkflowState::Writing { .. } => "Writing to target location",
            ExportWorkflowState::Verifying { .. } => "Verifying checksums",
            ExportWorkflowState::Completed { .. } => "Export complete",
            ExportWorkflowState::Failed { .. } => "Export failed",
        }
    }

    /// Get state name for error messages
    pub fn state_name(&self) -> &str {
        match self {
            ExportWorkflowState::Planning { .. } => "Planning",
            ExportWorkflowState::Generating { .. } => "Generating",
            ExportWorkflowState::Encrypting { .. } => "Encrypting",
            ExportWorkflowState::Writing { .. } => "Writing",
            ExportWorkflowState::Verifying { .. } => "Verifying",
            ExportWorkflowState::Completed { .. } => "Completed",
            ExportWorkflowState::Failed { .. } => "Failed",
        }
    }

    // ========================================================================
    // Terminal State Detection
    // ========================================================================

    /// Is the export in a terminal state?
    /// Both Completed and Failed are terminal states
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ExportWorkflowState::Completed { .. } | ExportWorkflowState::Failed { .. }
        )
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Validate if a transition to the target state is allowed
    pub fn can_transition_to(&self, target: &ExportWorkflowState) -> bool {
        match (self, target) {
            // Planning → Generating
            (ExportWorkflowState::Planning { .. }, ExportWorkflowState::Generating { .. }) => true,

            // Generating → Encrypting
            (ExportWorkflowState::Generating { .. }, ExportWorkflowState::Encrypting { .. }) => true,

            // Encrypting → Writing
            (ExportWorkflowState::Encrypting { .. }, ExportWorkflowState::Writing { .. }) => true,

            // Writing → Verifying
            (ExportWorkflowState::Writing { .. }, ExportWorkflowState::Verifying { .. }) => true,

            // Verifying → Completed
            (ExportWorkflowState::Verifying { .. }, ExportWorkflowState::Completed { .. }) => true,

            // Any non-terminal state → Failed (error path)
            (current, ExportWorkflowState::Failed { .. }) if !current.is_terminal() => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Start generating artifacts
    pub fn start_generating(
        &self,
        artifacts: &[ArtifactType],
    ) -> Result<ExportWorkflowState, ExportWorkflowError> {
        if !self.can_generate() {
            return Err(ExportWorkflowError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "StartGenerating".to_string(),
                reason: "Can only start generating from Planning state".to_string(),
            });
        }

        if artifacts.is_empty() {
            return Err(ExportWorkflowError::ValidationFailed(
                "Artifacts list cannot be empty".to_string(),
            ));
        }

        let progress: HashMap<ArtifactType, GenerationProgress> = artifacts
            .iter()
            .map(|a| (*a, GenerationProgress::Pending))
            .collect();

        Ok(ExportWorkflowState::Generating { progress })
    }

    /// Start encrypting sensitive data
    pub fn start_encrypting(
        &self,
        encryption_key_id: Uuid,
    ) -> Result<ExportWorkflowState, ExportWorkflowError> {
        if !self.can_encrypt() {
            return Err(ExportWorkflowError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "StartEncrypting".to_string(),
                reason: "Can only start encrypting from Generating state".to_string(),
            });
        }

        Ok(ExportWorkflowState::Encrypting {
            encryption_key_id,
            progress_percent: 0,
        })
    }

    /// Start writing to target location
    pub fn start_writing(
        &self,
        total_bytes: u64,
    ) -> Result<ExportWorkflowState, ExportWorkflowError> {
        if !self.can_write() {
            return Err(ExportWorkflowError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "StartWriting".to_string(),
                reason: "Can only start writing from Encrypting state".to_string(),
            });
        }

        if total_bytes == 0 {
            return Err(ExportWorkflowError::ValidationFailed(
                "Total bytes cannot be zero".to_string(),
            ));
        }

        Ok(ExportWorkflowState::Writing {
            bytes_written: 0,
            total_bytes,
        })
    }

    /// Start verifying checksums
    pub fn start_verifying(
        &self,
        checksums: HashMap<String, String>,
    ) -> Result<ExportWorkflowState, ExportWorkflowError> {
        if !self.can_verify() {
            return Err(ExportWorkflowError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "StartVerifying".to_string(),
                reason: "Can only start verifying from Writing state".to_string(),
            });
        }

        if checksums.is_empty() {
            return Err(ExportWorkflowError::ValidationFailed(
                "Checksums map cannot be empty".to_string(),
            ));
        }

        Ok(ExportWorkflowState::Verifying { checksums })
    }

    /// Complete the export successfully
    pub fn complete(
        &self,
        manifest_checksum: String,
        exported_at: DateTime<Utc>,
    ) -> Result<ExportWorkflowState, ExportWorkflowError> {
        match self {
            ExportWorkflowState::Verifying { .. } => {
                if manifest_checksum.is_empty() {
                    return Err(ExportWorkflowError::ValidationFailed(
                        "Manifest checksum cannot be empty".to_string(),
                    ));
                }

                Ok(ExportWorkflowState::Completed {
                    manifest_checksum,
                    exported_at,
                })
            }
            _ if self.is_terminal() => Err(ExportWorkflowError::TerminalState(format!(
                "Export is already in terminal state: {}",
                self.state_name()
            ))),
            _ => Err(ExportWorkflowError::InvalidTransition {
                current: self.state_name().to_string(),
                event: "Complete".to_string(),
                reason: "Can only complete from Verifying state".to_string(),
            }),
        }
    }

    /// Fail the export with an error
    pub fn fail(
        &self,
        error: String,
        failed_at: DateTime<Utc>,
    ) -> Result<ExportWorkflowState, ExportWorkflowError> {
        if self.is_terminal() {
            return Err(ExportWorkflowError::TerminalState(format!(
                "Export is already in terminal state: {}",
                self.state_name()
            )));
        }

        if error.is_empty() {
            return Err(ExportWorkflowError::ValidationFailed(
                "Error message cannot be empty".to_string(),
            ));
        }

        Ok(ExportWorkflowState::Failed { error, failed_at })
    }
}

/// Errors that can occur during export workflow state transitions
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ExportWorkflowError {
    /// Invalid state transition attempted
    #[error("Invalid state transition from {current} on event {event}: {reason}")]
    InvalidTransition {
        current: String,
        event: String,
        reason: String,
    },

    /// Terminal state reached
    #[error("Terminal state reached: {0}")]
    TerminalState(String),

    /// State validation failed
    #[error("State validation failed: {0}")]
    ValidationFailed(String),
}

/// Type of artifact to export
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ArtifactType {
    RootCACertificate,
    IntermediateCACertificate,
    LeafCertificate,
    PublicKey,
    EncryptedPrivateKey,
    NatsOperatorJWT,
    NatsAccountJWT,
    NatsUserCreds,
    DidDocument,
    VerifiableCredential,
    Manifest,
}

/// Generation progress for an artifact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GenerationProgress {
    Pending,
    InProgress { percent: u8 },
    Completed { artifact_id: Uuid },
    Failed { error: String },
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // ========================================================================
    // PKIBootstrapState Tests
    // ========================================================================

    #[test]
    fn test_pki_bootstrap_initial_state() {
        let state = PKIBootstrapState::Uninitialized;
        assert!(state.can_plan_root_ca());
        assert!(!state.can_generate_root_ca());
        assert!(!state.is_terminal());
    }

    #[test]
    fn test_pki_bootstrap_can_transition_to() {
        let uninitialized = PKIBootstrapState::Uninitialized;
        let root_planned = PKIBootstrapState::RootCAPlanned {
            subject: CertificateSubject {
                common_name: "Test CA".to_string(),
                organization: "Test Org".to_string(),
                organizational_unit: None,
                country: "US".to_string(),
                state: None,
                locality: None,
                email: None,
            },
            validity_years: 10,
            yubikey_serial: "12345678".to_string(),
        };

        // Valid transitions
        assert!(uninitialized.can_transition_to(&root_planned));

        // Invalid transitions
        let bootstrapped = PKIBootstrapState::Bootstrapped {
            export_location: Uuid::now_v7(),
            export_checksum: "abc123".to_string(),
            bootstrapped_at: Utc::now(),
        };
        assert!(!uninitialized.can_transition_to(&bootstrapped));
    }

    #[test]
    fn test_pki_bootstrap_full_workflow() {
        // Start uninitialized
        let state = PKIBootstrapState::Uninitialized;

        // Plan root CA
        let subject = CertificateSubject {
            common_name: "CowboyAI Root CA".to_string(),
            organization: "CowboyAI".to_string(),
            organizational_unit: Some("Security".to_string()),
            country: "US".to_string(),
            state: Some("TX".to_string()),
            locality: Some("Austin".to_string()),
            email: None,
        };
        let state = state
            .plan_root_ca(subject.clone(), 20, "12345678".to_string())
            .expect("Should plan root CA");
        assert!(matches!(state, PKIBootstrapState::RootCAPlanned { .. }));

        // Generate root CA
        let state = state
            .generate_root_ca(Uuid::now_v7(), Uuid::now_v7(), Utc::now())
            .expect("Should generate root CA");
        assert!(matches!(state, PKIBootstrapState::RootCAGenerated { .. }));

        // Generate intermediate CA (skip planning)
        let state = state
            .generate_intermediate_ca(vec![Uuid::now_v7()])
            .expect("Should generate intermediate CA");
        assert!(matches!(
            state,
            PKIBootstrapState::IntermediateCAGenerated { .. }
        ));

        // Generate leaf certs
        let state = state
            .generate_leaf_certs(vec![Uuid::now_v7(), Uuid::now_v7()])
            .expect("Should generate leaf certs");
        assert!(matches!(state, PKIBootstrapState::LeafCertsGenerated { .. }));

        // Provision YubiKeys
        let state = state
            .provision_yubikeys(vec!["YUBIKEY001".to_string(), "YUBIKEY002".to_string()])
            .expect("Should provision YubiKeys");
        assert!(matches!(
            state,
            PKIBootstrapState::YubiKeysProvisioned { .. }
        ));

        // Prepare export
        let state = state
            .prepare_export(Uuid::now_v7())
            .expect("Should prepare export");
        assert!(matches!(state, PKIBootstrapState::ExportReady { .. }));

        // Complete bootstrap
        let state = state
            .complete_bootstrap(Uuid::now_v7(), "sha256:abc123".to_string(), Utc::now())
            .expect("Should complete bootstrap");
        assert!(matches!(state, PKIBootstrapState::Bootstrapped { .. }));
        assert!(state.is_terminal());
        assert!(state.is_complete());
    }

    #[test]
    fn test_pki_bootstrap_invalid_transition() {
        let state = PKIBootstrapState::Uninitialized;

        // Can't generate root CA without planning first
        let result = state.generate_root_ca(Uuid::now_v7(), Uuid::now_v7(), Utc::now());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PKIBootstrapError::InvalidTransition { .. }
        ));
    }

    #[test]
    fn test_pki_bootstrap_validation_errors() {
        // Can't generate intermediate CA with empty list
        let state = PKIBootstrapState::RootCAGenerated {
            root_ca_cert_id: Uuid::now_v7(),
            root_ca_key_id: Uuid::now_v7(),
            generated_at: Utc::now(),
        };
        let result = state.generate_intermediate_ca(vec![]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PKIBootstrapError::ValidationFailed(_)
        ));
    }

    #[test]
    fn test_pki_bootstrap_state_descriptions() {
        let state = PKIBootstrapState::Uninitialized;
        assert_eq!(state.description(), "PKI infrastructure not initialized");
        assert_eq!(state.state_name(), "Uninitialized");
    }

    // ========================================================================
    // YubiKeyProvisioningState Tests
    // ========================================================================

    #[test]
    fn test_yubikey_provisioning_initial_state() {
        let state = YubiKeyProvisioningState::Detected {
            serial: "12345678".to_string(),
            firmware_version: "5.4.3".to_string(),
        };
        assert!(state.can_authenticate());
        assert!(!state.can_change_pin());
        assert!(!state.is_terminal());
    }

    #[test]
    fn test_yubikey_provisioning_can_transition_to() {
        let detected = YubiKeyProvisioningState::Detected {
            serial: "12345678".to_string(),
            firmware_version: "5.4.3".to_string(),
        };
        let authenticated = YubiKeyProvisioningState::Authenticated {
            pin_retries_remaining: 3,
        };

        // Valid transitions
        assert!(detected.can_transition_to(&authenticated));

        // Invalid transitions - can't skip steps
        let sealed = YubiKeyProvisioningState::Sealed {
            sealed_at: Utc::now(),
            final_config_hash: "abc123".to_string(),
        };
        assert!(!detected.can_transition_to(&sealed));
    }

    #[test]
    fn test_yubikey_provisioning_full_workflow() {
        // Start with detected YubiKey
        let state = YubiKeyProvisioningState::Detected {
            serial: "12345678".to_string(),
            firmware_version: "5.4.3".to_string(),
        };

        // Authenticate
        let state = state.authenticate(3).expect("Should authenticate");
        assert!(matches!(
            state,
            YubiKeyProvisioningState::Authenticated { .. }
        ));

        // Change PIN
        let state = state
            .change_pin("sha256:newpinhash".to_string())
            .expect("Should change PIN");
        assert!(matches!(state, YubiKeyProvisioningState::PINChanged { .. }));

        // Rotate management key
        let state = state
            .rotate_management_key(PivAlgorithm::EcdsaP256)
            .expect("Should rotate management key");
        assert!(matches!(
            state,
            YubiKeyProvisioningState::ManagementKeyRotated { .. }
        ));

        // Plan slots
        let mut slot_plan = HashMap::new();
        slot_plan.insert(
            PivSlot::Signature,
            SlotPlan {
                purpose: KeyPurpose::Signing,
                algorithm: KeyAlgorithm::Ed25519,
                pin_policy: PinPolicy::Once,
                touch_policy: TouchPolicy::Always,
            },
        );
        let state = state.plan_slots(slot_plan).expect("Should plan slots");
        assert!(matches!(
            state,
            YubiKeyProvisioningState::SlotPlanned { .. }
        ));

        // Generate keys
        let mut slot_keys = HashMap::new();
        slot_keys.insert(PivSlot::Signature, vec![1, 2, 3, 4]);
        let state = state
            .generate_keys(slot_keys)
            .expect("Should generate keys");
        assert!(matches!(
            state,
            YubiKeyProvisioningState::KeysGenerated { .. }
        ));

        // Import certificates
        let mut slot_certs = HashMap::new();
        slot_certs.insert(PivSlot::Signature, Uuid::now_v7());
        let state = state
            .import_certificates(slot_certs)
            .expect("Should import certs");
        assert!(matches!(
            state,
            YubiKeyProvisioningState::CertificatesImported { .. }
        ));

        // Attest keys
        let state = state
            .attest_keys(true, vec![Uuid::now_v7()])
            .expect("Should attest");
        assert!(matches!(state, YubiKeyProvisioningState::Attested { .. }));

        // Seal
        let state = state
            .seal(Utc::now(), "sha256:finalconfig".to_string())
            .expect("Should seal");
        assert!(matches!(state, YubiKeyProvisioningState::Sealed { .. }));
        assert!(state.is_terminal());
        assert!(state.is_complete());
        assert!(state.is_sealed());
    }

    #[test]
    fn test_yubikey_provisioning_invalid_transition() {
        let state = YubiKeyProvisioningState::Detected {
            serial: "12345678".to_string(),
            firmware_version: "5.4.3".to_string(),
        };

        // Can't change PIN without authenticating first
        let result = state.change_pin("newhash".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            YubiKeyProvisioningError::InvalidTransition { .. }
        ));
    }

    #[test]
    fn test_yubikey_provisioning_validation_errors() {
        // Can't seal with empty hash
        let state = YubiKeyProvisioningState::Attested {
            attestation_chain_verified: true,
            attestation_cert_ids: vec![Uuid::now_v7()],
        };
        let result = state.seal(Utc::now(), "".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            YubiKeyProvisioningError::ValidationFailed(_)
        ));
    }

    #[test]
    fn test_yubikey_provisioning_state_descriptions() {
        let state = YubiKeyProvisioningState::Detected {
            serial: "12345678".to_string(),
            firmware_version: "5.4.3".to_string(),
        };
        assert_eq!(state.description(), "YubiKey detected");
        assert_eq!(state.state_name(), "Detected");
    }

    // ========================================================================
    // ExportWorkflowState Tests
    // ========================================================================

    #[test]
    fn test_export_workflow_initial_state() {
        let state = ExportWorkflowState::Planning {
            artifacts_to_export: vec![ArtifactType::RootCACertificate],
            target_location: Uuid::now_v7(),
        };
        assert!(state.can_generate());
        assert!(!state.can_encrypt());
        assert!(!state.is_terminal());
        assert!(!state.is_complete());
        assert!(!state.has_failed());
    }

    #[test]
    fn test_export_workflow_can_transition_to() {
        let planning = ExportWorkflowState::Planning {
            artifacts_to_export: vec![ArtifactType::RootCACertificate],
            target_location: Uuid::now_v7(),
        };
        let generating = ExportWorkflowState::Generating {
            progress: HashMap::new(),
        };
        let encrypting = ExportWorkflowState::Encrypting {
            encryption_key_id: Uuid::now_v7(),
            progress_percent: 0,
        };
        let writing = ExportWorkflowState::Writing {
            bytes_written: 0,
            total_bytes: 1000,
        };
        let verifying = ExportWorkflowState::Verifying {
            checksums: HashMap::new(),
        };
        let completed = ExportWorkflowState::Completed {
            manifest_checksum: "abc123".to_string(),
            exported_at: Utc::now(),
        };
        let failed = ExportWorkflowState::Failed {
            error: "Test error".to_string(),
            failed_at: Utc::now(),
        };

        // Valid transitions
        assert!(planning.can_transition_to(&generating));
        assert!(generating.can_transition_to(&encrypting));
        assert!(encrypting.can_transition_to(&writing));
        assert!(writing.can_transition_to(&verifying));
        assert!(verifying.can_transition_to(&completed));

        // Error path: any non-terminal can transition to Failed
        assert!(planning.can_transition_to(&failed));
        assert!(generating.can_transition_to(&failed));
        assert!(encrypting.can_transition_to(&failed));
        assert!(writing.can_transition_to(&failed));
        assert!(verifying.can_transition_to(&failed));

        // Invalid transitions
        assert!(!planning.can_transition_to(&completed)); // Must go through workflow
        assert!(!generating.can_transition_to(&writing)); // Must encrypt first
        assert!(!completed.can_transition_to(&failed)); // Terminal can't transition
        assert!(!failed.can_transition_to(&completed)); // Terminal can't transition
    }

    #[test]
    fn test_export_workflow_full_workflow() {
        // Start in Planning
        let state = ExportWorkflowState::Planning {
            artifacts_to_export: vec![ArtifactType::RootCACertificate],
            target_location: Uuid::now_v7(),
        };

        // Planning → Generating
        let artifacts = vec![ArtifactType::RootCACertificate, ArtifactType::Manifest];
        let state = state.start_generating(&artifacts).unwrap();
        assert!(matches!(state, ExportWorkflowState::Generating { .. }));

        // Generating → Encrypting
        let encryption_key_id = Uuid::now_v7();
        let state = state.start_encrypting(encryption_key_id).unwrap();
        assert!(matches!(state, ExportWorkflowState::Encrypting { .. }));

        // Encrypting → Writing
        let state = state.start_writing(1024).unwrap();
        assert!(matches!(state, ExportWorkflowState::Writing { .. }));

        // Writing → Verifying
        let mut checksums = HashMap::new();
        checksums.insert("manifest.json".to_string(), "sha256:abc123".to_string());
        let state = state.start_verifying(checksums).unwrap();
        assert!(matches!(state, ExportWorkflowState::Verifying { .. }));

        // Verifying → Completed
        let state = state
            .complete("sha256:final".to_string(), Utc::now())
            .unwrap();
        assert!(state.is_complete());
        assert!(state.is_terminal());
    }

    #[test]
    fn test_export_workflow_fail_from_any_state() {
        // Test failure from each non-terminal state
        let states = vec![
            ExportWorkflowState::Planning {
                artifacts_to_export: vec![ArtifactType::RootCACertificate],
                target_location: Uuid::now_v7(),
            },
            ExportWorkflowState::Generating {
                progress: HashMap::new(),
            },
            ExportWorkflowState::Encrypting {
                encryption_key_id: Uuid::now_v7(),
                progress_percent: 50,
            },
            ExportWorkflowState::Writing {
                bytes_written: 512,
                total_bytes: 1024,
            },
            ExportWorkflowState::Verifying {
                checksums: HashMap::new(),
            },
        ];

        for state in states {
            let failed = state.fail("Disk full".to_string(), Utc::now());
            assert!(failed.is_ok());
            assert!(failed.unwrap().has_failed());
        }
    }

    #[test]
    fn test_export_workflow_invalid_transitions() {
        // Try to encrypt from Planning (should fail)
        let state = ExportWorkflowState::Planning {
            artifacts_to_export: vec![ArtifactType::RootCACertificate],
            target_location: Uuid::now_v7(),
        };
        let result = state.start_encrypting(Uuid::now_v7());
        assert!(matches!(
            result,
            Err(ExportWorkflowError::InvalidTransition { .. })
        ));

        // Try to complete from a terminal state
        let completed = ExportWorkflowState::Completed {
            manifest_checksum: "abc123".to_string(),
            exported_at: Utc::now(),
        };
        let result = completed.fail("Should not work".to_string(), Utc::now());
        assert!(matches!(
            result,
            Err(ExportWorkflowError::TerminalState(_))
        ));
    }

    #[test]
    fn test_export_workflow_validation_errors() {
        // Empty artifacts list
        let state = ExportWorkflowState::Planning {
            artifacts_to_export: vec![ArtifactType::RootCACertificate],
            target_location: Uuid::now_v7(),
        };
        let result = state.start_generating(&[]);
        assert!(matches!(
            result,
            Err(ExportWorkflowError::ValidationFailed(_))
        ));

        // Zero bytes for writing
        let state = ExportWorkflowState::Encrypting {
            encryption_key_id: Uuid::now_v7(),
            progress_percent: 100,
        };
        let result = state.start_writing(0);
        assert!(matches!(
            result,
            Err(ExportWorkflowError::ValidationFailed(_))
        ));

        // Empty checksums
        let state = ExportWorkflowState::Writing {
            bytes_written: 1024,
            total_bytes: 1024,
        };
        let result = state.start_verifying(HashMap::new());
        assert!(matches!(
            result,
            Err(ExportWorkflowError::ValidationFailed(_))
        ));

        // Empty manifest checksum
        let state = ExportWorkflowState::Verifying {
            checksums: {
                let mut m = HashMap::new();
                m.insert("file.txt".to_string(), "sha256:xyz".to_string());
                m
            },
        };
        let result = state.complete(String::new(), Utc::now());
        assert!(matches!(
            result,
            Err(ExportWorkflowError::ValidationFailed(_))
        ));

        // Empty error message
        let state = ExportWorkflowState::Planning {
            artifacts_to_export: vec![ArtifactType::RootCACertificate],
            target_location: Uuid::now_v7(),
        };
        let result = state.fail(String::new(), Utc::now());
        assert!(matches!(
            result,
            Err(ExportWorkflowError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_export_workflow_state_descriptions() {
        let state = ExportWorkflowState::Planning {
            artifacts_to_export: vec![ArtifactType::RootCACertificate],
            target_location: Uuid::now_v7(),
        };
        assert_eq!(state.description(), "Planning export");
        assert_eq!(state.state_name(), "Planning");

        let state = ExportWorkflowState::Completed {
            manifest_checksum: "abc".to_string(),
            exported_at: Utc::now(),
        };
        assert_eq!(state.description(), "Export complete");
        assert_eq!(state.state_name(), "Completed");
        assert!(state.is_terminal());

        let state = ExportWorkflowState::Failed {
            error: "Test".to_string(),
            failed_at: Utc::now(),
        };
        assert_eq!(state.description(), "Export failed");
        assert_eq!(state.state_name(), "Failed");
        assert!(state.is_terminal());
    }
}
