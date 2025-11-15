//! Intent - Unified Event Source Abstraction
//!
//! ALL events in the cim-keys application flow through this single algebraic type.
//! This makes event origins explicit and enables cross-framework reuse.

use std::path::PathBuf;

/// Type of node that can be created in the graph
#[derive(Debug, Clone)]
pub enum NodeCreationType {
    Organization,
    OrganizationalUnit,
    Person,
    Location,
    Role,
    Policy,
}

/// Intent - unified event source abstraction for ALL inputs
///
/// **Design Principle**: Event source is explicit in the type.
/// Unlike traditional Message enums that mix UI and async events,
/// Intent variants explicitly name their origin:
/// - `Ui*` = User interface interactions
/// - `Domain*` = Domain events from aggregates
/// - `Port*` = Responses from hexagonal ports
/// - `System*` = System-level events
#[derive(Debug, Clone)]
pub enum Intent {
    // ===== UI-Originated Intents =====
    /// User selected a different tab
    UiTabSelected(super::model::Tab),

    /// User clicked "Create New Domain"
    UiCreateDomainClicked,

    /// User clicked "Load Existing Domain"
    UiLoadDomainClicked { path: PathBuf },

    /// User updated organization name input
    UiOrganizationNameChanged(String),

    /// User updated organization ID input
    UiOrganizationIdChanged(String),

    /// User clicked "Add Person"
    UiAddPersonClicked,

    /// User updated person name input
    UiPersonNameChanged { index: usize, name: String },

    /// User updated person email input
    UiPersonEmailChanged { index: usize, email: String },

    /// User clicked "Remove Person"
    UiRemovePersonClicked { index: usize },

    /// User clicked "Generate Root CA"
    UiGenerateRootCAClicked,

    /// User clicked "Generate Intermediate CA"
    UiGenerateIntermediateCAClicked { name: String },

    /// User clicked "Generate Server Certificate"
    UiGenerateServerCertClicked {
        common_name: String,
        san_entries: Vec<String>,
        intermediate_ca_name: String,
    },

    /// User clicked "Generate SSH Keys"
    UiGenerateSSHKeysClicked,

    /// User clicked "Generate All Keys"
    UiGenerateAllKeysClicked,

    /// User clicked "Export to SD Card"
    UiExportClicked { output_path: PathBuf },

    /// User clicked "Provision YubiKey"
    UiProvisionYubiKeyClicked { person_index: usize },

    /// User entered/changed master passphrase
    UiPassphraseChanged(String),

    /// User entered/changed passphrase confirmation
    UiPassphraseConfirmChanged(String),

    /// User clicked "Derive Master Seed"
    UiDeriveMasterSeedClicked,

    // ===== Domain-Originated Intents =====
    /// Domain was successfully created
    DomainCreated {
        organization_id: String,
        organization_name: String,
    },

    /// Person was added to organization
    PersonAdded {
        person_id: String,
        name: String,
        email: String,
    },

    /// Root CA was generated
    RootCAGenerated {
        certificate_id: String,
        subject: String,
    },

    /// SSH keypair was generated for a person
    SSHKeyGenerated {
        person_id: String,
        key_type: String,
        fingerprint: String,
    },

    /// YubiKey was provisioned for a person
    YubiKeyProvisioned {
        person_id: String,
        yubikey_serial: String,
        slot: String,
    },

    /// Master seed was successfully derived from passphrase
    MasterSeedDerived {
        organization_id: String,
        entropy_bits: f64,
        seed: crate::crypto::MasterSeed,
    },

    /// Master seed derivation failed
    MasterSeedDerivationFailed {
        error: String,
    },

    // ===== Graph-Originated Intents =====
    /// User clicked to create a new node in the graph
    UiGraphCreateNode {
        node_type: NodeCreationType,
        position: (f32, f32),  // x, y coordinates
    },

    /// User started creating an edge by clicking a source node
    UiGraphCreateEdgeStarted {
        from_node: String,  // Node ID
    },

    /// User completed edge creation by clicking a target node
    UiGraphCreateEdgeCompleted {
        from: String,       // Source node ID
        to: String,         // Target node ID
        edge_type: String,  // Edge type as string for serialization
    },

    /// User cancelled edge creation
    UiGraphCreateEdgeCancelled,

    /// User clicked on a node to select it
    UiGraphNodeClicked {
        node_id: String,
    },

    /// User requested to delete a node
    UiGraphDeleteNode {
        node_id: String,
    },

    /// User requested to delete an edge
    UiGraphDeleteEdge {
        from: String,
        to: String,
    },

    /// User opened property editor for a node
    UiGraphEditNodeProperties {
        node_id: String,
    },

    /// User changed a property value in the property editor
    UiGraphPropertyChanged {
        node_id: String,
        property: String,
        value: String,
    },

    /// User saved property changes
    UiGraphPropertiesSaved {
        node_id: String,
    },

    /// User cancelled property editing
    UiGraphPropertiesCancelled,

    /// User requested auto-layout of the graph
    UiGraphAutoLayout,

    // Domain events for graph changes
    /// A node was created in the domain
    DomainNodeCreated {
        node_id: String,
        node_type: String,
    },

    /// An edge was created in the domain
    DomainEdgeCreated {
        from: String,
        to: String,
        edge_type: String,
    },

    /// A node was deleted from the domain
    DomainNodeDeleted {
        node_id: String,
    },

    /// A node's properties were updated
    DomainNodeUpdated {
        node_id: String,
        properties: Vec<(String, String)>,
    },

    /// An organization was created
    DomainOrganizationCreated {
        org_id: String,
        name: String,
    },

    /// An organizational unit was created
    DomainOrgUnitCreated {
        unit_id: String,
        name: String,
        parent_id: Option<String>,
    },

    /// A location was created
    DomainLocationCreated {
        location_id: String,
        name: String,
        location_type: String,
    },

    /// A role was created
    DomainRoleCreated {
        role_id: String,
        name: String,
        organization_id: String,
    },

    /// A policy was created
    DomainPolicyCreated {
        policy_id: String,
        name: String,
        claims: Vec<String>,
    },

    /// A policy was bound to an entity
    DomainPolicyBound {
        policy_id: String,
        entity_id: String,
        entity_type: String,
    },

    // ===== Port-Originated Intents (Async Responses) =====
    /// Storage port completed write operation
    PortStorageWriteCompleted {
        path: String,
        bytes_written: usize,
    },

    /// Storage port failed write operation
    PortStorageWriteFailed {
        path: String,
        error: String,
    },

    /// X509 port completed root CA generation
    PortX509RootCAGenerated {
        certificate_pem: String,
        private_key_pem: String,
        fingerprint: String,
    },

    /// X509 port completed intermediate CA generation
    PortX509IntermediateCAGenerated {
        name: String,
        certificate_pem: String,
        private_key_pem: String,
        fingerprint: String,
    },

    /// X509 port completed server certificate generation
    PortX509ServerCertGenerated {
        common_name: String,
        certificate_pem: String,
        private_key_pem: String,
        fingerprint: String,
        signed_by: String,
    },

    /// X509 port failed certificate generation
    PortX509GenerationFailed {
        error: String,
    },

    /// SSH port completed keypair generation
    PortSSHKeypairGenerated {
        person_id: String,
        public_key: String,
        fingerprint: String,
    },

    /// SSH port failed keypair generation
    PortSSHGenerationFailed {
        person_id: String,
        error: String,
    },

    /// YubiKey port listed devices
    PortYubiKeyDevicesListed {
        devices: Vec<String>,
    },

    /// YubiKey port completed key generation in slot
    PortYubiKeyKeyGenerated {
        yubikey_serial: String,
        slot: String,
        public_key: Vec<u8>,
    },

    /// YubiKey port failed operation
    PortYubiKeyOperationFailed {
        error: String,
    },

    // ===== System-Originated Intents =====
    /// System file picker dialog returned a path
    SystemFileSelected(PathBuf),

    /// System file picker was cancelled
    SystemFilePickerCancelled,

    /// Error occurred in the system
    SystemErrorOccurred {
        context: String,
        error: String,
    },

    /// System clipboard was updated
    SystemClipboardUpdated(String),

    // ===== Error Intents =====
    /// Generic error occurred
    ErrorOccurred {
        context: String,
        message: String,
    },

    /// Error was dismissed by user
    ErrorDismissed {
        error_id: String,
    },

    /// No operation (used for Task::none())
    NoOp,
}

impl Intent {
    /// Check if this intent represents an error state
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Intent::ErrorOccurred { .. }
                | Intent::PortStorageWriteFailed { .. }
                | Intent::PortX509GenerationFailed { .. }
                | Intent::PortSSHGenerationFailed { .. }
                | Intent::PortYubiKeyOperationFailed { .. }
                | Intent::SystemErrorOccurred { .. }
        )
    }

    /// Check if this intent originated from the UI
    pub fn is_ui_originated(&self) -> bool {
        matches!(
            self,
            Intent::UiTabSelected(_)
                | Intent::UiCreateDomainClicked
                | Intent::UiLoadDomainClicked { .. }
                | Intent::UiOrganizationNameChanged(_)
                | Intent::UiOrganizationIdChanged(_)
                | Intent::UiAddPersonClicked
                | Intent::UiPersonNameChanged { .. }
                | Intent::UiPersonEmailChanged { .. }
                | Intent::UiRemovePersonClicked { .. }
                | Intent::UiGenerateRootCAClicked
                | Intent::UiGenerateIntermediateCAClicked { .. }
                | Intent::UiGenerateServerCertClicked { .. }
                | Intent::UiGenerateSSHKeysClicked
                | Intent::UiGenerateAllKeysClicked
                | Intent::UiExportClicked { .. }
                | Intent::UiProvisionYubiKeyClicked { .. }
                // Graph intents
                | Intent::UiGraphCreateNode { .. }
                | Intent::UiGraphCreateEdgeStarted { .. }
                | Intent::UiGraphCreateEdgeCompleted { .. }
                | Intent::UiGraphCreateEdgeCancelled
                | Intent::UiGraphNodeClicked { .. }
                | Intent::UiGraphDeleteNode { .. }
                | Intent::UiGraphDeleteEdge { .. }
                | Intent::UiGraphEditNodeProperties { .. }
                | Intent::UiGraphPropertyChanged { .. }
                | Intent::UiGraphPropertiesSaved { .. }
                | Intent::UiGraphPropertiesCancelled
                | Intent::UiGraphAutoLayout
        )
    }

    /// Check if this intent originated from a hexagonal port
    pub fn is_port_originated(&self) -> bool {
        matches!(
            self,
            Intent::PortStorageWriteCompleted { .. }
                | Intent::PortStorageWriteFailed { .. }
                | Intent::PortX509RootCAGenerated { .. }
                | Intent::PortX509IntermediateCAGenerated { .. }
                | Intent::PortX509ServerCertGenerated { .. }
                | Intent::PortX509GenerationFailed { .. }
                | Intent::PortSSHKeypairGenerated { .. }
                | Intent::PortSSHGenerationFailed { .. }
                | Intent::PortYubiKeyDevicesListed { .. }
                | Intent::PortYubiKeyKeyGenerated { .. }
                | Intent::PortYubiKeyOperationFailed { .. }
        )
    }

    /// Check if this intent originated from the domain layer
    pub fn is_domain_originated(&self) -> bool {
        matches!(
            self,
            Intent::DomainCreated { .. }
                | Intent::PersonAdded { .. }
                | Intent::RootCAGenerated { .. }
                | Intent::SSHKeyGenerated { .. }
                | Intent::YubiKeyProvisioned { .. }
                // Graph domain events
                | Intent::DomainNodeCreated { .. }
                | Intent::DomainEdgeCreated { .. }
                | Intent::DomainNodeDeleted { .. }
                | Intent::DomainNodeUpdated { .. }
                | Intent::DomainOrganizationCreated { .. }
                | Intent::DomainOrgUnitCreated { .. }
                | Intent::DomainLocationCreated { .. }
                | Intent::DomainRoleCreated { .. }
                | Intent::DomainPolicyCreated { .. }
                | Intent::DomainPolicyBound { .. }
        )
    }
}
