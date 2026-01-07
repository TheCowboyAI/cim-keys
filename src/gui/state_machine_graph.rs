// Copyright (c) 2025 - Cowboy AI, LLC.

//! State Machine Graph Visualization
//!
//! Visualizes state machines as directed graphs where:
//! - States are nodes (objects in categorical terms)
//! - Transitions are edges (morphisms)
//! - Current state is highlighted
//! - Terminal states are marked distinctly
//!
//! ## Category Theory Foundation
//!
//! A state machine is a **small category** where:
//! - Objects are states
//! - Morphisms are transitions
//! - Composition is sequential transition execution
//! - Identity morphisms are "stay in state" (implicit)
//!
//! The graph visualization is a functor from the state machine category
//! to the visual display category.

use iced::{Color, Point};
use std::collections::HashMap;
use uuid::Uuid;

use crate::gui::graph::{OrganizationConcept, ConceptEntity, EdgeType};
use crate::lifting::{LiftedNode, Injection};

/// State machine type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateMachineType {
    Key,
    Certificate,
    Policy,
    Person,
    Organization,
    Location,
    Relationship,
    Manifest,
    NatsOperator,
    NatsAccount,
    NatsUser,
    YubiKey,
    // Workflow state machines
    PkiBootstrap,
    YubiKeyProvisioning,
    ExportWorkflow,
    // Saga state machines
    CertificateProvisioning,
    PersonOnboarding,
    CompleteBootstrap,
}

impl StateMachineType {
    /// Get all available state machine types
    pub fn all() -> Vec<Self> {
        vec![
            Self::Key,
            Self::Certificate,
            Self::Policy,
            Self::Person,
            Self::Organization,
            Self::Location,
            Self::Relationship,
            Self::Manifest,
            Self::NatsOperator,
            Self::NatsAccount,
            Self::NatsUser,
            Self::YubiKey,
            Self::PkiBootstrap,
            Self::YubiKeyProvisioning,
            Self::ExportWorkflow,
            Self::CertificateProvisioning,
            Self::PersonOnboarding,
            Self::CompleteBootstrap,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Key => "Key Lifecycle",
            Self::Certificate => "Certificate Lifecycle",
            Self::Policy => "Policy Lifecycle",
            Self::Person => "Person Lifecycle",
            Self::Organization => "Organization Lifecycle",
            Self::Location => "Location Lifecycle",
            Self::Relationship => "Relationship Lifecycle",
            Self::Manifest => "Manifest Lifecycle",
            Self::NatsOperator => "NATS Operator Lifecycle",
            Self::NatsAccount => "NATS Account Lifecycle",
            Self::NatsUser => "NATS User Lifecycle",
            Self::YubiKey => "YubiKey Lifecycle",
            Self::PkiBootstrap => "PKI Bootstrap Workflow",
            Self::YubiKeyProvisioning => "YubiKey Provisioning Workflow",
            Self::ExportWorkflow => "Export Workflow",
            Self::CertificateProvisioning => "Certificate Provisioning Saga",
            Self::PersonOnboarding => "Person Onboarding Saga",
            Self::CompleteBootstrap => "Complete Bootstrap Saga",
        }
    }

    /// Get category description
    pub fn category(&self) -> &'static str {
        match self {
            Self::Key | Self::Certificate | Self::Policy => "Security",
            Self::Person | Self::Organization | Self::Location | Self::Relationship => "Domain",
            Self::NatsOperator | Self::NatsAccount | Self::NatsUser | Self::Manifest => "Infrastructure",
            Self::YubiKey | Self::YubiKeyProvisioning => "Hardware",
            Self::PkiBootstrap | Self::ExportWorkflow |
            Self::CertificateProvisioning | Self::PersonOnboarding | Self::CompleteBootstrap => "Workflow",
        }
    }
}

/// A state in a state machine
#[derive(Debug, Clone)]
pub struct StateMachineState {
    /// Unique ID for this state (derived from name)
    pub id: Uuid,
    /// State name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Is this a terminal state?
    pub is_terminal: bool,
    /// Is this the initial state?
    pub is_initial: bool,
    /// Color for visualization
    pub color: Color,
}

impl StateMachineState {
    /// Create a new state
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        // Generate deterministic UUID from name using simple hash (not v5)
        // We use the name bytes to seed a deterministic UUID
        let mut bytes = [0u8; 16];
        let name_bytes = name.as_bytes();
        for (i, b) in name_bytes.iter().enumerate() {
            bytes[i % 16] ^= *b;
        }
        // Set version to 8 (custom) and variant to RFC 4122
        bytes[6] = (bytes[6] & 0x0f) | 0x80;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let id = Uuid::from_bytes(bytes);
        Self {
            id,
            name,
            description: description.into(),
            is_terminal: false,
            is_initial: false,
            color: Color::from_rgb(0.6, 0.6, 0.6),
        }
    }

    pub fn terminal(mut self) -> Self {
        self.is_terminal = true;
        self.color = Color::from_rgb(0.8, 0.2, 0.2); // Red for terminal
        self
    }

    pub fn initial(mut self) -> Self {
        self.is_initial = true;
        self.color = Color::from_rgb(0.2, 0.8, 0.2); // Green for initial
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// A transition between states
#[derive(Debug, Clone)]
pub struct StateMachineTransition {
    /// Source state name
    pub from: String,
    /// Target state name
    pub to: String,
    /// Transition label (event or action)
    pub label: String,
    /// Is this the current/recent transition?
    pub is_active: bool,
}

impl StateMachineTransition {
    pub fn new(from: impl Into<String>, to: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: label.into(),
            is_active: false,
        }
    }

    pub fn active(mut self) -> Self {
        self.is_active = true;
        self
    }
}

/// Complete state machine definition for visualization
#[derive(Debug, Clone)]
pub struct StateMachineDefinition {
    /// Type of state machine
    pub machine_type: StateMachineType,
    /// All states
    pub states: Vec<StateMachineState>,
    /// All valid transitions
    pub transitions: Vec<StateMachineTransition>,
    /// Currently active state (if applicable)
    pub current_state: Option<String>,
}

impl StateMachineDefinition {
    pub fn new(machine_type: StateMachineType) -> Self {
        Self {
            machine_type,
            states: Vec::new(),
            transitions: Vec::new(),
            current_state: None,
        }
    }

    pub fn with_state(mut self, state: StateMachineState) -> Self {
        self.states.push(state);
        self
    }

    pub fn with_transition(mut self, transition: StateMachineTransition) -> Self {
        self.transitions.push(transition);
        self
    }

    pub fn with_current(mut self, state_name: impl Into<String>) -> Self {
        self.current_state = Some(state_name.into());
        self
    }

    /// Get state by name
    pub fn get_state(&self, name: &str) -> Option<&StateMachineState> {
        self.states.iter().find(|s| s.name == name)
    }
}

// ============================================================================
// State Machine Definitions - The 14 State Machines
// ============================================================================

/// Build KeyState machine definition
pub fn build_key_state_machine() -> StateMachineDefinition {
    StateMachineDefinition::new(StateMachineType::Key)
        .with_state(StateMachineState::new("Generated", "Key generated but not yet activated")
            .initial()
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("Imported", "Key imported from external source")
            .initial()
            .with_color(Color::from_rgb(0.4, 0.6, 0.8)))
        .with_state(StateMachineState::new("Active", "Key is active and usable")
            .with_color(Color::from_rgb(0.2, 0.8, 0.2)))
        .with_state(StateMachineState::new("RotationPending", "Key rotation in progress")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("Rotated", "Key has been superseded")
            .with_color(Color::from_rgb(0.6, 0.6, 0.6)))
        .with_state(StateMachineState::new("Revoked", "Key has been revoked")
            .terminal()
            .with_color(Color::from_rgb(0.8, 0.2, 0.2)))
        .with_state(StateMachineState::new("Expired", "Key validity period ended")
            .with_color(Color::from_rgb(0.8, 0.5, 0.2)))
        .with_state(StateMachineState::new("Archived", "Key archived for long-term storage")
            .terminal()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5)))
        // Transitions
        .with_transition(StateMachineTransition::new("Generated", "Active", "KeyActivated"))
        .with_transition(StateMachineTransition::new("Imported", "Active", "KeyActivated"))
        .with_transition(StateMachineTransition::new("Active", "RotationPending", "KeyRotationInitiated"))
        .with_transition(StateMachineTransition::new("RotationPending", "Rotated", "KeyRotationCompleted"))
        .with_transition(StateMachineTransition::new("Active", "Revoked", "KeyRevoked"))
        .with_transition(StateMachineTransition::new("Generated", "Revoked", "KeyRevoked"))
        .with_transition(StateMachineTransition::new("Imported", "Revoked", "KeyRevoked"))
        .with_transition(StateMachineTransition::new("RotationPending", "Revoked", "KeyRevoked"))
        .with_transition(StateMachineTransition::new("Active", "Expired", "KeyExpired"))
        .with_transition(StateMachineTransition::new("Rotated", "Archived", "KeyArchived"))
        .with_transition(StateMachineTransition::new("Revoked", "Archived", "KeyArchived"))
        .with_transition(StateMachineTransition::new("Expired", "Archived", "KeyArchived"))
}

/// Build CertificateState machine definition
pub fn build_certificate_state_machine() -> StateMachineDefinition {
    StateMachineDefinition::new(StateMachineType::Certificate)
        .with_state(StateMachineState::new("Pending", "Certificate requested, awaiting CA signature")
            .initial()
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("Issued", "Certificate signed but not yet valid")
            .with_color(Color::from_rgb(0.4, 0.6, 0.8)))
        .with_state(StateMachineState::new("Active", "Certificate is valid for use")
            .with_color(Color::from_rgb(0.2, 0.8, 0.2)))
        .with_state(StateMachineState::new("RenewalPending", "Certificate renewal in progress")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("Renewed", "Certificate has been superseded")
            .with_color(Color::from_rgb(0.6, 0.6, 0.6)))
        .with_state(StateMachineState::new("Revoked", "Certificate has been revoked")
            .terminal()
            .with_color(Color::from_rgb(0.8, 0.2, 0.2)))
        .with_state(StateMachineState::new("Expired", "Certificate validity period ended")
            .with_color(Color::from_rgb(0.8, 0.5, 0.2)))
        .with_state(StateMachineState::new("Archived", "Certificate archived")
            .terminal()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5)))
        // Transitions
        .with_transition(StateMachineTransition::new("Pending", "Issued", "CertificateSigned"))
        .with_transition(StateMachineTransition::new("Issued", "Active", "ValidityPeriodStarted"))
        .with_transition(StateMachineTransition::new("Active", "RenewalPending", "RenewalInitiated"))
        .with_transition(StateMachineTransition::new("RenewalPending", "Renewed", "NewCertIssued"))
        .with_transition(StateMachineTransition::new("Active", "Revoked", "CertificateRevoked"))
        .with_transition(StateMachineTransition::new("Pending", "Revoked", "CertificateRevoked"))
        .with_transition(StateMachineTransition::new("Issued", "Revoked", "CertificateRevoked"))
        .with_transition(StateMachineTransition::new("RenewalPending", "Revoked", "CertificateRevoked"))
        .with_transition(StateMachineTransition::new("Active", "Expired", "ValidityPeriodEnded"))
        .with_transition(StateMachineTransition::new("Renewed", "Archived", "CertificateArchived"))
        .with_transition(StateMachineTransition::new("Revoked", "Archived", "CertificateArchived"))
        .with_transition(StateMachineTransition::new("Expired", "Archived", "CertificateArchived"))
}

/// Build PersonState machine definition
pub fn build_person_state_machine() -> StateMachineDefinition {
    StateMachineDefinition::new(StateMachineType::Person)
        .with_state(StateMachineState::new("Invited", "Person invited to organization")
            .initial()
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("Pending", "Awaiting identity verification")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("Active", "Person is an active member")
            .with_color(Color::from_rgb(0.2, 0.8, 0.2)))
        .with_state(StateMachineState::new("Suspended", "Person temporarily suspended")
            .with_color(Color::from_rgb(0.8, 0.5, 0.2)))
        .with_state(StateMachineState::new("Departed", "Person has left organization")
            .terminal()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5)))
        // Transitions
        .with_transition(StateMachineTransition::new("Invited", "Pending", "InvitationAccepted"))
        .with_transition(StateMachineTransition::new("Pending", "Active", "IdentityVerified"))
        .with_transition(StateMachineTransition::new("Active", "Suspended", "PersonSuspended"))
        .with_transition(StateMachineTransition::new("Suspended", "Active", "SuspensionLifted"))
        .with_transition(StateMachineTransition::new("Active", "Departed", "PersonDeparted"))
        .with_transition(StateMachineTransition::new("Suspended", "Departed", "PersonDeparted"))
        .with_transition(StateMachineTransition::new("Invited", "Departed", "InvitationRevoked"))
        .with_transition(StateMachineTransition::new("Pending", "Departed", "VerificationFailed"))
}

/// Build OrganizationState machine definition
pub fn build_organization_state_machine() -> StateMachineDefinition {
    StateMachineDefinition::new(StateMachineType::Organization)
        .with_state(StateMachineState::new("Founding", "Organization being established")
            .initial()
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("Active", "Organization is operational")
            .with_color(Color::from_rgb(0.2, 0.8, 0.2)))
        .with_state(StateMachineState::new("Restructuring", "Organization undergoing changes")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("Dissolved", "Organization has been dissolved")
            .terminal()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5)))
        // Transitions
        .with_transition(StateMachineTransition::new("Founding", "Active", "OrganizationEstablished"))
        .with_transition(StateMachineTransition::new("Active", "Restructuring", "RestructuringInitiated"))
        .with_transition(StateMachineTransition::new("Restructuring", "Active", "RestructuringCompleted"))
        .with_transition(StateMachineTransition::new("Active", "Dissolved", "OrganizationDissolved"))
        .with_transition(StateMachineTransition::new("Restructuring", "Dissolved", "OrganizationDissolved"))
}

/// Build YubiKeyState machine definition
pub fn build_yubikey_state_machine() -> StateMachineDefinition {
    StateMachineDefinition::new(StateMachineType::YubiKey)
        .with_state(StateMachineState::new("Uninitialized", "YubiKey not yet provisioned")
            .initial()
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("Initializing", "YubiKey being configured")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("Provisioned", "YubiKey ready with keys")
            .with_color(Color::from_rgb(0.2, 0.8, 0.2)))
        .with_state(StateMachineState::new("InUse", "YubiKey assigned and active")
            .with_color(Color::from_rgb(0.2, 0.6, 0.9)))
        .with_state(StateMachineState::new("Locked", "YubiKey temporarily locked")
            .with_color(Color::from_rgb(0.8, 0.5, 0.2)))
        .with_state(StateMachineState::new("Retired", "YubiKey decommissioned")
            .terminal()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5)))
        // Transitions
        .with_transition(StateMachineTransition::new("Uninitialized", "Initializing", "InitializationStarted"))
        .with_transition(StateMachineTransition::new("Initializing", "Provisioned", "ProvisioningCompleted"))
        .with_transition(StateMachineTransition::new("Provisioned", "InUse", "AssignedToPerson"))
        .with_transition(StateMachineTransition::new("InUse", "Provisioned", "Unassigned"))
        .with_transition(StateMachineTransition::new("InUse", "Locked", "TooManyAttempts"))
        .with_transition(StateMachineTransition::new("Locked", "InUse", "PinReset"))
        .with_transition(StateMachineTransition::new("Provisioned", "Retired", "YubiKeyRetired"))
        .with_transition(StateMachineTransition::new("InUse", "Retired", "YubiKeyRetired"))
        .with_transition(StateMachineTransition::new("Locked", "Retired", "YubiKeyRetired"))
}

/// Build PKI Bootstrap workflow state machine
///
/// State names MUST match PKIBootstrapState enum variants exactly for
/// current_state highlighting to work correctly.
pub fn build_pki_bootstrap_state_machine() -> StateMachineDefinition {
    StateMachineDefinition::new(StateMachineType::PkiBootstrap)
        // States match PKIBootstrapState enum variants exactly
        .with_state(StateMachineState::new("Uninitialized", "PKI infrastructure not initialized")
            .initial()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5)))
        .with_state(StateMachineState::new("RootCAPlanned", "Root CA planned, awaiting generation")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("RootCAGenerated", "Root CA generated (offline ceremony complete)")
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("IntermediateCAPlanned", "Intermediate CA planned")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("IntermediateCAGenerated", "Intermediate CA(s) generated")
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("LeafCertsGenerated", "Leaf certificates generated")
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("YubiKeysProvisioned", "YubiKeys provisioned with keys")
            .with_color(Color::from_rgb(0.4, 0.6, 0.8)))
        .with_state(StateMachineState::new("ExportReady", "Export manifest ready")
            .with_color(Color::from_rgb(0.3, 0.6, 0.9)))
        .with_state(StateMachineState::new("Bootstrapped", "PKI bootstrap complete")
            .terminal()
            .with_color(Color::from_rgb(0.2, 0.8, 0.2)))
        // Transitions match state machine guards
        .with_transition(StateMachineTransition::new("Uninitialized", "RootCAPlanned", "PkiPlanRootCA"))
        .with_transition(StateMachineTransition::new("RootCAPlanned", "RootCAGenerated", "PkiRootCAGenerationComplete"))
        .with_transition(StateMachineTransition::new("RootCAGenerated", "IntermediateCAPlanned", "PkiPlanIntermediateCA"))
        .with_transition(StateMachineTransition::new("IntermediateCAPlanned", "IntermediateCAGenerated", "PkiIntermediateCAGenerationComplete"))
        .with_transition(StateMachineTransition::new("RootCAGenerated", "IntermediateCAGenerated", "PkiIntermediateCAGenerationComplete"))
        .with_transition(StateMachineTransition::new("IntermediateCAGenerated", "LeafCertsGenerated", "PkiLeafCertGenerationComplete"))
        .with_transition(StateMachineTransition::new("LeafCertsGenerated", "YubiKeysProvisioned", "YubiKeyProvisioningComplete"))
        .with_transition(StateMachineTransition::new("YubiKeysProvisioned", "ExportReady", "PkiExportReady"))
        .with_transition(StateMachineTransition::new("ExportReady", "Bootstrapped", "PkiBootstrapComplete"))
}

/// Build YubiKey Provisioning workflow state machine
///
/// State names MUST match YubiKeyProvisioningState enum variants exactly for
/// current_state highlighting to work correctly.
pub fn build_yubikey_provisioning_state_machine() -> StateMachineDefinition {
    StateMachineDefinition::new(StateMachineType::YubiKeyProvisioning)
        // States match YubiKeyProvisioningState enum variants exactly
        .with_state(StateMachineState::new("Detected", "YubiKey detected, serial number read")
            .initial()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5)))
        .with_state(StateMachineState::new("Authenticated", "Authenticated with current PIN")
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("PINChanged", "PIN changed from default")
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("ManagementKeyRotated", "Management key rotated from default")
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("SlotPlanned", "Slot allocation planned")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("KeysGenerated", "Keys generated in slots")
            .with_color(Color::from_rgb(0.4, 0.6, 0.8)))
        .with_state(StateMachineState::new("CertificatesImported", "Certificates imported to slots")
            .with_color(Color::from_rgb(0.4, 0.6, 0.8)))
        .with_state(StateMachineState::new("Attested", "Keys attested (verified on-device generation)")
            .with_color(Color::from_rgb(0.3, 0.7, 0.5)))
        .with_state(StateMachineState::new("Sealed", "Configuration sealed (final, immutable)")
            .terminal()
            .with_color(Color::from_rgb(0.2, 0.8, 0.2)))
        // Transitions match state machine guards
        .with_transition(StateMachineTransition::new("Detected", "Authenticated", "YubiKeyAuthenticated"))
        .with_transition(StateMachineTransition::new("Authenticated", "PINChanged", "YubiKeyPINChanged"))
        .with_transition(StateMachineTransition::new("PINChanged", "ManagementKeyRotated", "ManagementKeyRotated"))
        .with_transition(StateMachineTransition::new("ManagementKeyRotated", "SlotPlanned", "SlotAllocationPlanned"))
        .with_transition(StateMachineTransition::new("SlotPlanned", "KeysGenerated", "KeyGenerationComplete"))
        .with_transition(StateMachineTransition::new("KeysGenerated", "CertificatesImported", "CertificatesImported"))
        .with_transition(StateMachineTransition::new("CertificatesImported", "Attested", "AttestationComplete"))
        .with_transition(StateMachineTransition::new("Attested", "Sealed", "ConfigurationSealed"))
}

/// Build Certificate Provisioning Saga state machine
pub fn build_certificate_provisioning_saga() -> StateMachineDefinition {
    StateMachineDefinition::new(StateMachineType::CertificateProvisioning)
        .with_state(StateMachineState::new("Initial", "Saga initialized")
            .initial()
            .with_color(Color::from_rgb(0.6, 0.6, 0.6)))
        .with_state(StateMachineState::new("GeneratingKey", "Generating key pair")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("GeneratingCertificate", "Generating certificate")
            .with_color(Color::from_rgb(0.9, 0.7, 0.2)))
        .with_state(StateMachineState::new("ProvisioningToYubiKey", "Loading to YubiKey")
            .with_color(Color::from_rgb(0.4, 0.6, 0.8)))
        .with_state(StateMachineState::new("VerifyingProvisioning", "Verifying YubiKey slot")
            .with_color(Color::from_rgb(0.4, 0.7, 0.4)))
        .with_state(StateMachineState::new("Completed", "Saga completed successfully")
            .terminal()
            .with_color(Color::from_rgb(0.2, 0.8, 0.2)))
        .with_state(StateMachineState::new("Failed", "Saga failed")
            .terminal()
            .with_color(Color::from_rgb(0.8, 0.2, 0.2)))
        .with_state(StateMachineState::new("Compensating", "Rolling back changes")
            .with_color(Color::from_rgb(0.8, 0.5, 0.2)))
        // Transitions
        .with_transition(StateMachineTransition::new("Initial", "GeneratingKey", "Start"))
        .with_transition(StateMachineTransition::new("GeneratingKey", "GeneratingCertificate", "KeyGenerated"))
        .with_transition(StateMachineTransition::new("GeneratingCertificate", "ProvisioningToYubiKey", "CertificateGenerated"))
        .with_transition(StateMachineTransition::new("ProvisioningToYubiKey", "VerifyingProvisioning", "YubiKeyProvisioned"))
        .with_transition(StateMachineTransition::new("VerifyingProvisioning", "Completed", "Verified"))
        .with_transition(StateMachineTransition::new("GeneratingKey", "Failed", "Error"))
        .with_transition(StateMachineTransition::new("GeneratingCertificate", "Failed", "Error"))
        .with_transition(StateMachineTransition::new("ProvisioningToYubiKey", "Failed", "Error"))
        .with_transition(StateMachineTransition::new("VerifyingProvisioning", "Failed", "VerificationFailed"))
        .with_transition(StateMachineTransition::new("Failed", "Compensating", "StartCompensation"))
        .with_transition(StateMachineTransition::new("Compensating", "Failed", "CompensationComplete"))
}

/// Get all state machine definitions
pub fn all_state_machines() -> Vec<StateMachineDefinition> {
    vec![
        build_key_state_machine(),
        build_certificate_state_machine(),
        build_person_state_machine(),
        build_organization_state_machine(),
        build_yubikey_state_machine(),
        build_pki_bootstrap_state_machine(),
        build_yubikey_provisioning_state_machine(),
        build_certificate_provisioning_saga(),
    ]
}

/// Get a state machine definition by type
pub fn get_state_machine(machine_type: StateMachineType) -> StateMachineDefinition {
    match machine_type {
        StateMachineType::Key => build_key_state_machine(),
        StateMachineType::Certificate => build_certificate_state_machine(),
        StateMachineType::Person => build_person_state_machine(),
        StateMachineType::Organization => build_organization_state_machine(),
        StateMachineType::YubiKey => build_yubikey_state_machine(),
        StateMachineType::PkiBootstrap => build_pki_bootstrap_state_machine(),
        StateMachineType::YubiKeyProvisioning => build_yubikey_provisioning_state_machine(),
        StateMachineType::CertificateProvisioning => build_certificate_provisioning_saga(),
        // Default to Key for unimplemented types
        _ => build_key_state_machine(),
    }
}

/// Get a state machine definition with current state set from the actual PKI state
pub fn get_pki_bootstrap_with_current(pki_state: &crate::state_machines::workflows::PKIBootstrapState) -> StateMachineDefinition {
    let current_state_name = pki_state.state_name();
    build_pki_bootstrap_state_machine().with_current(current_state_name)
}

/// Get a state machine definition with current state set from the actual YubiKey state
pub fn get_yubikey_provisioning_with_current(yubikey_state: &crate::state_machines::workflows::YubiKeyProvisioningState) -> StateMachineDefinition {
    let current_state_name = yubikey_state.state_name();
    build_yubikey_provisioning_state_machine().with_current(current_state_name)
}

// ============================================================================
// Graph Conversion - Lift state machines to OrganizationConcept
// ============================================================================

/// Configuration for state machine graph layout
#[derive(Debug, Clone)]
pub struct StateMachineLayoutConfig {
    /// Scale factor for positions
    pub scale: f32,
    /// Center point
    pub center: Point,
    /// Radius for circular layout
    pub radius: f32,
}

impl Default for StateMachineLayoutConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            center: Point::new(400.0, 300.0),
            radius: 200.0,
        }
    }
}

/// Convert a state machine definition to an OrganizationConcept graph
pub fn state_machine_to_graph(
    definition: &StateMachineDefinition,
    config: &StateMachineLayoutConfig,
) -> OrganizationConcept {
    let mut graph = OrganizationConcept::new();

    let state_count = definition.states.len();
    let angle_step = 2.0 * std::f32::consts::PI / state_count as f32;

    // Create state name -> UUID mapping
    let mut state_ids: HashMap<String, Uuid> = HashMap::new();

    // Add states as nodes in circular layout
    for (i, state) in definition.states.iter().enumerate() {
        let angle = angle_step * i as f32 - std::f32::consts::PI / 2.0; // Start from top
        let x = config.center.x + config.radius * angle.cos();
        let y = config.center.y + config.radius * angle.sin();

        // Determine color based on state properties
        let color = if definition.current_state.as_ref() == Some(&state.name) {
            Color::from_rgb(0.0, 0.6, 1.0) // Highlight current state
        } else {
            state.color
        };

        // Lift state to graph node (Kan extension: Graph → Domain)
        let node = LiftedNode::new(
            state.id,
            Injection::StateMachineState,
            state.name.clone(),
            color,
            state.clone(),
        ).with_secondary(state.description.clone());

        let entity = ConceptEntity::from_lifted_node(node);
        let mut view = entity.create_view(Point::new(x, y));

        // Mark terminal states with special border
        if state.is_terminal {
            view.color = Color::from_rgb(
                view.color.r * 0.7,
                view.color.g * 0.3,
                view.color.b * 0.3,
            );
        }

        graph.nodes.insert(state.id, entity);
        graph.node_views.insert(state.id, view);
        state_ids.insert(state.name.clone(), state.id);
    }

    // Add transitions as edges
    for transition in &definition.transitions {
        if let (Some(&from_id), Some(&to_id)) = (
            state_ids.get(&transition.from),
            state_ids.get(&transition.to),
        ) {
            let edge_type = if transition.is_active {
                EdgeType::WorkflowDependency // Active transition
            } else {
                EdgeType::SemanticNeighbor // Potential transition
            };

            graph.add_edge(from_id, to_id, edge_type);
        }
    }

    graph.rebuild_adjacency_indices();
    graph
}

/// Message type for state machine graph interactions
#[derive(Debug, Clone)]
pub enum StateMachineMessage {
    /// Selected a different state machine type to view
    SelectMachine(StateMachineType),
    /// Clicked on a state
    StateSelected(String),
    /// Clicked on a transition
    TransitionSelected { from: String, to: String },
    /// Reset view
    ResetView,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state_machine_states() {
        let sm = build_key_state_machine();
        assert_eq!(sm.states.len(), 8);
        assert_eq!(sm.machine_type, StateMachineType::Key);
    }

    #[test]
    fn test_key_state_machine_transitions() {
        let sm = build_key_state_machine();
        // Should have many transitions
        assert!(sm.transitions.len() >= 10);

        // Check specific transition exists
        let has_activate = sm.transitions.iter()
            .any(|t| t.from == "Generated" && t.to == "Active");
        assert!(has_activate);
    }

    #[test]
    fn test_certificate_state_machine() {
        let sm = build_certificate_state_machine();
        assert_eq!(sm.states.len(), 8);

        // Check terminal states
        let terminal_states: Vec<_> = sm.states.iter()
            .filter(|s| s.is_terminal)
            .collect();
        assert_eq!(terminal_states.len(), 2); // Revoked and Archived
    }

    #[test]
    fn test_state_machine_to_graph() {
        let sm = build_key_state_machine();
        let config = StateMachineLayoutConfig::default();

        let graph = state_machine_to_graph(&sm, &config);

        // Should have same number of nodes as states
        assert_eq!(graph.nodes.len(), sm.states.len());

        // Should have edges for transitions
        assert!(!graph.edges.is_empty());
    }

    #[test]
    fn test_current_state_highlight() {
        let mut sm = build_key_state_machine();
        sm.current_state = Some("Active".to_string());

        let config = StateMachineLayoutConfig::default();
        let graph = state_machine_to_graph(&sm, &config);

        // Find the Active state node
        let active_state = sm.get_state("Active").unwrap();
        let node_view = graph.node_views.get(&active_state.id).unwrap();

        // Should be highlighted (blue)
        assert!(node_view.color.b > 0.8);
    }

    #[test]
    fn test_all_state_machine_types() {
        let all = StateMachineType::all();
        assert!(all.len() >= 14);

        // Verify each has a display name
        for sm_type in all {
            assert!(!sm_type.display_name().is_empty());
            assert!(!sm_type.category().is_empty());
        }
    }

    #[test]
    fn test_saga_state_machine() {
        let sm = build_certificate_provisioning_saga();
        assert_eq!(sm.machine_type, StateMachineType::CertificateProvisioning);

        // Should have compensating state
        let has_compensating = sm.states.iter()
            .any(|s| s.name == "Compensating");
        assert!(has_compensating);
    }
}
