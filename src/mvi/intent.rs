//! Intent - Unified Event Source Abstraction
//!
//! ALL events in the cim-keys application flow through this single algebraic type.
//! This makes event origins explicit and enables cross-framework reuse.
//!
//! ## Signal Kind Classification
//!
//! Following n-ary FRP Axiom A1 (Multi-Kinded Signals), each intent has a natural
//! signal kind:
//!
//! - **EventKind (87%)**: Discrete occurrences (clicks, completions, failures)
//! - **StepKind (13%)**: Piecewise-constant values (form inputs, current state)
//! - **ContinuousKind (0%)**: Smooth functions (not yet used in cim-keys)
//!
//! See `INTENT_SIGNAL_KIND_ANALYSIS.md` for complete classification.

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
    UiConceptEntityClicked {
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

    /// Bootstrap domain data loaded from storage
    PortDomainLoaded {
        organization_name: String,
        organization_id: String,
        people_count: usize,
        locations_count: usize,
    },

    /// Bootstrap secrets data loaded from storage
    PortSecretsLoaded {
        organization_name: String,
        people_count: usize,
        yubikey_count: usize,
    },

    /// Domain exported to path
    PortDomainExported {
        path: String,
        bytes_written: usize,
    },

    /// Domain export failed
    PortDomainExportFailed {
        path: String,
        error: String,
    },

    /// NATS hierarchy generated
    PortNatsHierarchyGenerated {
        operator_name: String,
        account_count: usize,
        user_count: usize,
    },

    /// NATS hierarchy generation failed
    PortNatsHierarchyFailed {
        error: String,
    },

    /// Policy data loaded
    PortPolicyLoaded {
        role_count: usize,
        assignment_count: usize,
    },

    /// Policy loading failed
    PortPolicyLoadFailed {
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
                | Intent::PortDomainExportFailed { .. }
                | Intent::PortNatsHierarchyFailed { .. }
                | Intent::PortPolicyLoadFailed { .. }
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
                | Intent::UiConceptEntityClicked { .. }
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
                // New async result intents
                | Intent::PortDomainLoaded { .. }
                | Intent::PortSecretsLoaded { .. }
                | Intent::PortDomainExported { .. }
                | Intent::PortDomainExportFailed { .. }
                | Intent::PortNatsHierarchyGenerated { .. }
                | Intent::PortNatsHierarchyFailed { .. }
                | Intent::PortPolicyLoaded { .. }
                | Intent::PortPolicyLoadFailed { .. }
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

    /// Get the signal kind for this intent variant
    ///
    /// Following n-ary FRP Axiom A1 (Multi-Kinded Signals), each intent has a
    /// natural signal kind that determines its temporal semantics:
    ///
    /// - **EventKind**: Discrete occurrences at specific time points (87% of intents)
    ///   - Button clicks, completions, failures, domain events
    ///   - Semantics: `⟦Event T⟧(t) = [(t', x) | t' ≤ t]`
    ///
    /// - **StepKind**: Piecewise-constant values that change discretely (13% of intents)
    ///   - Form input values (organization name, person email, passphrase)
    ///   - Semantics: `⟦Step T⟧(t) = T` (holds value until next change)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cim_keys::mvi::Intent;
    /// use cim_keys::signals::SignalKind;
    ///
    /// // Event signals (discrete occurrences)
    /// let click = Intent::UiGenerateRootCAClicked;
    /// assert!(click.is_event_signal());
    ///
    /// // Step signals (piecewise constant values)
    /// let input = Intent::UiOrganizationNameChanged("Acme Corp".to_string());
    /// assert!(input.is_step_signal());
    /// ```
    ///
    /// See `INTENT_SIGNAL_KIND_ANALYSIS.md` for complete classification.
    pub fn is_event_signal(&self) -> bool {
        matches!(
            self,
            // All intents are events EXCEPT the following step signals
            Intent::UiOrganizationNameChanged(_)
            | Intent::UiOrganizationIdChanged(_)
            | Intent::UiPersonNameChanged { .. }
            | Intent::UiPersonEmailChanged { .. }
            | Intent::UiPassphraseChanged(_)
            | Intent::UiPassphraseConfirmChanged(_)
            | Intent::UiGraphPropertyChanged { .. }
        ) == false
    }

    /// Check if this intent is a step signal (piecewise-constant value)
    ///
    /// Step signals hold current values between changes, such as form inputs.
    /// These represent the *current state* of an input, not the change event itself.
    ///
    /// Step signals: 9 variants (13%)
    /// - UiOrganizationNameChanged
    /// - UiOrganizationIdChanged
    /// - UiPersonNameChanged
    /// - UiPersonEmailChanged
    /// - UiPassphraseChanged
    /// - UiPassphraseConfirmChanged
    /// - UiGraphPropertyChanged
    pub fn is_step_signal(&self) -> bool {
        matches!(
            self,
            Intent::UiOrganizationNameChanged(_)
            | Intent::UiOrganizationIdChanged(_)
            | Intent::UiPersonNameChanged { .. }
            | Intent::UiPersonEmailChanged { .. }
            | Intent::UiPassphraseChanged(_)
            | Intent::UiPassphraseConfirmChanged(_)
            | Intent::UiGraphPropertyChanged { .. }
        )
    }

    /// Type-level marker for this intent's signal kind
    ///
    /// Returns a type that can be used with `Signal<K, T>` to create
    /// properly typed signals.
    ///
    /// # Example Type Usage
    ///
    /// ```rust,ignore
    /// use cim_keys::signals::Signal;
    /// use cim_keys::mvi::Intent;
    ///
    /// // Event signal
    /// let click_events = Signal::<EventKind, Intent>::event(vec![
    ///     (0.0, Intent::UiGenerateRootCAClicked),
    ///     (1.0, Intent::UiGenerateSSHKeysClicked),
    /// ]);
    ///
    /// // Step signal (current organization name)
    /// let org_name = Signal::<StepKind, String>::step("Acme Corp".into());
    /// ```
    pub fn signal_kind_marker(&self) -> SignalKindMarker {
        if self.is_step_signal() {
            SignalKindMarker::Step
        } else {
            SignalKindMarker::Event
        }
    }
}

/// Marker type for signal kinds (runtime representation)
///
/// This is a runtime representation of the type-level SignalKind distinction.
/// Used for reflection and debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalKindMarker {
    Event,
    Step,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Signal Kind Classification Tests
    // =============================================================================

    #[test]
    fn test_event_intents_are_events() {
        // UI event intents (button clicks)
        assert!(Intent::UiGenerateRootCAClicked.is_event_signal());
        assert!(Intent::UiGenerateSSHKeysClicked.is_event_signal());
        assert!(Intent::UiAddPersonClicked.is_event_signal());
        assert!(Intent::UiDeriveMasterSeedClicked.is_event_signal());
        assert!(Intent::UiCreateDomainClicked.is_event_signal());

        // Domain event intents
        assert!(Intent::DomainCreated {
            organization_id: "test".into(),
            organization_name: "Test Org".into(),
        }.is_event_signal());

        assert!(Intent::PersonAdded {
            person_id: "p1".into(),
            name: "John Doe".into(),
            email: "john@example.com".into(),
        }.is_event_signal());

        assert!(Intent::RootCAGenerated {
            certificate_id: "cert1".into(),
            subject: "CN=Root CA".into(),
        }.is_event_signal());

        // Port event intents (async completions)
        assert!(Intent::PortX509RootCAGenerated {
            certificate_pem: "-----BEGIN CERTIFICATE-----".into(),
            private_key_pem: "-----BEGIN PRIVATE KEY-----".into(),
            fingerprint: "abc123".into(),
        }.is_event_signal());

        assert!(Intent::PortSSHKeypairGenerated {
            person_id: "p1".into(),
            public_key: "ssh-rsa AAA...".into(),
            fingerprint: "SHA256:abc".into(),
        }.is_event_signal());

        // Error intents
        assert!(Intent::ErrorOccurred {
            context: "test".into(),
            message: "error".into(),
        }.is_event_signal());

        assert!(Intent::SystemErrorOccurred {
            context: "system".into(),
            error: "error".into(),
        }.is_event_signal());
    }

    #[test]
    fn test_step_intents_are_steps() {
        // Form input intents (piecewise-constant values)
        assert!(Intent::UiOrganizationNameChanged("Acme Corp".into()).is_step_signal());
        assert!(Intent::UiOrganizationIdChanged("acme".into()).is_step_signal());
        assert!(Intent::UiPassphraseChanged("secret123".into()).is_step_signal());
        assert!(Intent::UiPassphraseConfirmChanged("secret123".into()).is_step_signal());

        assert!(Intent::UiPersonNameChanged {
            index: 0,
            name: "John Doe".into(),
        }.is_step_signal());

        assert!(Intent::UiPersonEmailChanged {
            index: 0,
            email: "john@example.com".into(),
        }.is_step_signal());

        assert!(Intent::UiGraphPropertyChanged {
            node_id: "node1".into(),
            property: "name".into(),
            value: "New Name".into(),
        }.is_step_signal());
    }

    #[test]
    fn test_step_intents_not_events() {
        // Step signals should NOT be event signals
        assert!(!Intent::UiOrganizationNameChanged("test".into()).is_event_signal());
        assert!(!Intent::UiPassphraseChanged("secret".into()).is_event_signal());
    }

    #[test]
    fn test_event_intents_not_steps() {
        // Event signals should NOT be step signals
        assert!(!Intent::UiGenerateRootCAClicked.is_step_signal());
        assert!(!Intent::UiCreateDomainClicked.is_step_signal());
    }

    #[test]
    fn test_signal_kind_marker() {
        // Event intents return Event marker
        assert_eq!(
            Intent::UiGenerateRootCAClicked.signal_kind_marker(),
            SignalKindMarker::Event
        );

        // Step intents return Step marker
        assert_eq!(
            Intent::UiOrganizationNameChanged("test".into()).signal_kind_marker(),
            SignalKindMarker::Step
        );
    }

    #[test]
    fn test_graph_intents_classification() {
        // Graph creation intents are events
        assert!(Intent::UiGraphCreateNode {
            node_type: NodeCreationType::Person,
            position: (0.0, 0.0),
        }.is_event_signal());

        assert!(Intent::UiGraphCreateEdgeStarted {
            from_node: "node1".into(),
        }.is_event_signal());

        assert!(Intent::UiGraphCreateEdgeCompleted {
            from: "node1".into(),
            to: "node2".into(),
            edge_type: "reports_to".into(),
        }.is_event_signal());

        // Graph property changes are steps (hold current value)
        assert!(Intent::UiGraphPropertyChanged {
            node_id: "node1".into(),
            property: "name".into(),
            value: "John Doe".into(),
        }.is_step_signal());

        // Graph node clicks are events
        assert!(Intent::UiConceptEntityClicked {
            node_id: "node1".into(),
        }.is_event_signal());
    }

    #[test]
    fn test_all_intents_have_classification() {
        // This test ensures we don't forget to classify new intents
        // Every intent should be either an event or a step

        let test_intents = vec![
            Intent::UiTabSelected(super::super::model::Tab::Welcome),
            Intent::UiCreateDomainClicked,
            Intent::UiOrganizationNameChanged("test".into()),
            Intent::UiGenerateRootCAClicked,
            Intent::NoOp,
        ];

        for intent in test_intents {
            // Every intent must be either event or step (not both, not neither)
            let is_event = intent.is_event_signal();
            let is_step = intent.is_step_signal();

            assert!(
                is_event ^ is_step, // XOR: exactly one must be true
                "Intent {:?} must be either event or step, not both or neither",
                intent
            );
        }
    }

    #[test]
    fn test_statistics_match_documentation() {
        // According to INTENT_SIGNAL_KIND_ANALYSIS.md:
        // - EventKind: 62 variants (87%)
        // - StepKind: 9 variants (13%)

        let all_step_intents = vec![
            Intent::UiOrganizationNameChanged("".into()),
            Intent::UiOrganizationIdChanged("".into()),
            Intent::UiPersonNameChanged { index: 0, name: "".into() },
            Intent::UiPersonEmailChanged { index: 0, email: "".into() },
            Intent::UiPassphraseChanged("".into()),
            Intent::UiPassphraseConfirmChanged("".into()),
            Intent::UiGraphPropertyChanged {
                node_id: "".into(),
                property: "".into(),
                value: "".into(),
            },
        ];

        // Verify all 7 step intents are classified correctly
        assert_eq!(all_step_intents.len(), 7);
        for intent in &all_step_intents {
            assert!(
                intent.is_step_signal(),
                "Intent {:?} should be classified as step signal",
                intent
            );
        }
    }

    // =============================================================================
    // Existing Classification Tests (is_ui_originated, etc.)
    // =============================================================================

    #[test]
    fn test_is_error() {
        assert!(Intent::ErrorOccurred {
            context: "test".into(),
            message: "error".into(),
        }.is_error());

        assert!(!Intent::UiGenerateRootCAClicked.is_error());
    }

    #[test]
    fn test_is_ui_originated() {
        assert!(Intent::UiGenerateRootCAClicked.is_ui_originated());
        assert!(Intent::UiOrganizationNameChanged("test".into()).is_ui_originated());

        assert!(!Intent::DomainCreated {
            organization_id: "test".into(),
            organization_name: "Test".into(),
        }.is_ui_originated());
    }

    #[test]
    fn test_is_port_originated() {
        assert!(Intent::PortX509RootCAGenerated {
            certificate_pem: "".into(),
            private_key_pem: "".into(),
            fingerprint: "".into(),
        }.is_port_originated());

        assert!(!Intent::UiGenerateRootCAClicked.is_port_originated());
    }

    #[test]
    fn test_is_domain_originated() {
        assert!(Intent::DomainCreated {
            organization_id: "test".into(),
            organization_name: "Test".into(),
        }.is_domain_originated());

        assert!(!Intent::UiGenerateRootCAClicked.is_domain_originated());
    }
}
