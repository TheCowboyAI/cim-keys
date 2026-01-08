//! State Machines for CIM Keys
//!
//! This module contains all state machines used in cim-keys:
//!
//! ## Workflow State Machines
//! - PKIBootstrapState - Root CA → Intermediate → Leaf certificate workflow
//! - YubiKeyProvisioningState - YubiKey initialization and provisioning
//! - ExportWorkflowState - Export to encrypted storage
//!
//! ## Aggregate Lifecycle State Machines
//!
//! ### Phase 1: CRITICAL Security & Identity (IMPLEMENTED)
//! - KeyState - Cryptographic key lifecycle (8 states) ✅
//! - CertificateState - PKI certificate lifecycle (8 states) ✅
//! - PolicyState - Authorization policy lifecycle (5 states) ✅
//!
//! ### Phase 2: Core Domain (IMPLEMENTED)
//! - PersonState - Identity lifecycle (5 states) ✅
//! - OrganizationState - Organizational structure lifecycle (4 states) ✅
//! - LocationState - Physical/virtual location lifecycle (4 states) ✅
//! - RelationshipState - Graph relationship lifecycle (6 states) ✅
//!
//! ### Phase 3: Infrastructure & Export (IMPLEMENTED)
//! - ManifestState - Export manifest lifecycle (6 states) ✅
//! - NatsOperatorState - NATS operator lifecycle (5 states) ✅
//! - NatsAccountState - NATS account lifecycle (5 states) ✅
//! - NatsUserState - NATS user lifecycle (5 states) ✅
//! - YubiKeyState - Hardware security module lifecycle (6 states) ✅
//!
//! All state machines follow a common pattern:
//! - States are enums with data
//! - Transitions are validated before execution
//! - Events trigger state changes
//! - Terminal states prevent further modifications
//! - All state machines are serializable for event sourcing

// Workflow state machines (cross-aggregate workflows)
pub mod workflows;

// Aggregate lifecycle state machines (per-aggregate)
pub mod key;
pub mod certificate;
pub mod certificate_import;
pub mod policy;
pub mod person;
pub mod organization;
pub mod location;
pub mod relationship;
pub mod manifest;
pub mod nats_operator;
pub mod nats_account;
pub mod nats_user;
pub mod yubikey;

// Re-export workflow state machines
pub use workflows::{
    ArtifactType, CertificateSubject, ExportWorkflowState, GenerationProgress,
    PKIBootstrapError, PKIBootstrapState, PinPolicy, PivAlgorithm, PivSlot, SlotPlan,
    TouchPolicy, YubiKeyProvisioningState,
};

// Re-export aggregate state machines
pub use certificate::CertificateState;
pub use key::KeyState;
pub use policy::PolicyState;
pub use person::PersonState;
pub use organization::OrganizationState;
pub use location::{AccessGrant, AccessLevel, LocationState, LocationType};
pub use relationship::{
    RelationshipChange, RelationshipMetadata, RelationshipState, RelationshipStrength,
};
pub use manifest::{ManifestState, FailedStage};
pub use nats_operator::NatsOperatorState;
pub use nats_account::{NatsAccountState, NatsPermissions};
pub use nats_user::{NatsUserState, NatsUserPermissions};
pub use yubikey::{YubiKeyState, RetirementReason};
pub use certificate_import::{CertificateImportState, CertificateImportError};
