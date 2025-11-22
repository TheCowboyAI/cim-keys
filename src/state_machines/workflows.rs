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
