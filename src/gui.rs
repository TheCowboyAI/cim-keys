//! Native/WASM GUI for offline key generation using Iced 0.13+
//!
//! This module provides a pure Rust GUI that can run both as a native
//! application and as a WASM application in the browser.

use iced::{
    application,
    widget::{button, column, container, row, text, text_input, Container, horizontal_space, vertical_space, pick_list, progress_bar, checkbox, scrollable, Space, image, stack},
    Task, Element, Length, Border, Theme, Background, Shadow, Alignment, Point, Color,
};
use iced_futures::Subscription;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    aggregate::KeyManagementAggregate,
    domain::{Person, KeyOwnerRole},
    projections::OfflineKeyProjection,
    // MVI architecture
    mvi::{Intent, Model as MviModel},
    // Hexagonal ports
    ports::{StoragePort, X509Port, SshKeyPort, YubiKeyPort},
    // Adapters
    adapters::{
        InMemoryStorageAdapter,
        MockX509Adapter,
        MockSshKeyAdapter,
        YubiKeyCliAdapter,
    },
    // Icons
    icons::{self, ICON_WARNING},
};

pub mod graph;
pub mod graph_pki;
pub mod graph_nats;
pub mod graph_yubikey;
pub mod graph_events;
pub mod event_emitter;
pub mod cowboy_theme;
pub mod animated_background;
pub mod fireflies;
pub mod firefly_shader;
pub mod firefly_synchronization;
pub mod kuramoto_firefly_shader;
pub mod debug_firefly_shader;
pub mod firefly_math;
pub mod firefly_renderer;
pub mod context_menu;
pub mod property_card;
pub mod passphrase_dialog;
pub mod view_model;
pub mod edge_indicator;

#[cfg(test)]
mod graph_integration_tests;

use graph::{OrganizationGraph, GraphMessage};
use event_emitter::{CimEventEmitter, GuiEventSubscriber, InteractionType};
use view_model::ViewModel;
use cowboy_theme::{CowboyTheme, CowboyAppTheme as CowboyCustomTheme};
// use kuramoto_firefly_shader::KuramotoFireflyShader;
// use debug_firefly_shader::DebugFireflyShader;
use firefly_renderer::FireflyRenderer;
use context_menu::{ContextMenu, ContextMenuMessage};
use property_card::{PropertyCard, PropertyCardMessage};

/// Main application state
pub struct CimKeysApp {
    // Tab navigation
    active_tab: Tab,
    graph_view: GraphView,

    // Domain configuration
    domain_loaded: bool,
    _domain_path: PathBuf,  // Reserved for domain persistence path
    organization_name: String,
    organization_domain: String,
    organization_id: Option<Uuid>,  // Set when domain is created
    admin_email: String,  // Admin contact email for the organization

    // Master passphrase for encryption
    master_passphrase: String,
    master_passphrase_confirm: String,

    bootstrap_config: Option<BootstrapConfig>,
    aggregate: Arc<RwLock<KeyManagementAggregate>>,
    projection: Arc<RwLock<OfflineKeyProjection>>,

    // Application configuration (for NATS publishing, storage paths, etc.)
    config: Option<crate::config::Config>,

    // Event-driven communication
    event_emitter: CimEventEmitter,
    _event_subscriber: GuiEventSubscriber,  // Reserved for future NATS integration

    // Graph visualization
    org_graph: OrganizationGraph,
    graph_projector: crate::graph_projection::GraphProjector,  // Functorial projection to cim-graph
    selected_person: Option<Uuid>,
    selected_node_type: Option<String>,  // Node type selected in "Add Node" dropdown

    // Inline editing for newly created nodes
    editing_new_node: Option<Uuid>,  // Node being edited inline
    inline_edit_name: String,        // Name being edited

    // NATS infrastructure bootstrap (for graph visualization)
    nats_bootstrap: Option<crate::domain_projections::OrganizationBootstrap>,

    // Form fields for adding people
    new_person_name: String,
    new_person_email: String,
    new_person_role: Option<KeyOwnerRole>,

    // Form fields for adding locations
    new_location_name: String,
    new_location_type: Option<crate::domain::LocationType>,
    // Address fields for physical locations
    new_location_street: String,
    new_location_city: String,
    new_location_region: String,
    new_location_country: String,
    new_location_postal: String,

    // YubiKey fields
    yubikey_serial: String,
    yubikey_assigned_to: Option<Uuid>,
    detected_yubikeys: Vec<crate::ports::yubikey::YubiKeyDevice>,
    yubikey_detection_status: String,
    yubikey_configs: Vec<crate::domain::YubiKeyConfig>,  // Imported from secrets
    loaded_locations: Vec<crate::projections::LocationEntry>,  // Loaded from manifest
    loaded_people: Vec<crate::projections::PersonEntry>,  // Loaded from manifest
    loaded_certificates: Vec<crate::projections::CertificateEntry>,  // Loaded from manifest
    loaded_keys: Vec<crate::projections::KeyEntry>,  // Loaded from manifest

    // Key generation state
    key_generation_progress: f32,
    keys_generated: usize,
    total_keys_to_generate: usize,
    _certificates_generated: usize,  // Reserved for certificate generation tracking

    // Certificate generation fields
    intermediate_ca_name_input: String,
    server_cert_cn_input: String,
    server_cert_sans_input: String,
    selected_intermediate_ca: Option<String>,
    selected_cert_location: Option<String>,  // Storage location for certificates

    // Certificate metadata fields (editable before generation)
    cert_organization: String,
    cert_organizational_unit: String,
    cert_locality: String,
    cert_state_province: String,
    cert_country: String,
    cert_validity_days: String,  // String for text input, will parse to u32

    // Export configuration
    export_path: PathBuf,
    include_public_keys: bool,
    include_certificates: bool,
    include_nats_config: bool,
    include_private_keys: bool,
    export_password: String,

    // NATS hierarchy state
    nats_hierarchy_generated: bool,
    nats_operator_id: Option<Uuid>,
    nats_export_path: PathBuf,

    // Collapsible sections state
    root_ca_collapsed: bool,
    intermediate_ca_collapsed: bool,
    server_cert_collapsed: bool,
    yubikey_section_collapsed: bool,
    nats_section_collapsed: bool,
    certificates_collapsed: bool,
    keys_collapsed: bool,

    // Root passphrase for PKI
    root_passphrase: String,
    root_passphrase_confirm: String,
    show_passphrase: bool,  // Toggle to show/hide passphrase

    // Status
    status_message: String,
    error_message: Option<String>,
    overwrite_warning: Option<String>,  // Warning message when about to overwrite existing cert/key
    pending_generation_action: Option<Message>,  // Action to perform after overwrite confirmation

    // Animation
    animation_time: f32,
    firefly_shader: FireflyRenderer,

    // View Model (centralized sizing and layout)
    view_model: ViewModel,

    // MVI Integration (Option A: Quick Integration)
    mvi_model: MviModel,
    storage_port: Arc<dyn StoragePort>,
    x509_port: Arc<dyn X509Port>,
    ssh_port: Arc<dyn SshKeyPort>,
    yubikey_port: Arc<dyn YubiKeyPort>,

    // Phase 4: Interactive UI Components
    context_menu: ContextMenu,
    property_card: PropertyCard,
    passphrase_dialog: passphrase_dialog::PassphraseDialog,
    context_menu_node: Option<Uuid>,  // Node that context menu was opened on (if any)

    // Phase 5: Search and filtering
    search_query: String,
    search_results: Vec<Uuid>,  // Matching node IDs
    highlight_nodes: Vec<Uuid>,  // Nodes to highlight in graph

    // Phase 6: Help and tooltips
    show_help_overlay: bool,

    // Phase 7: Loading indicators
    loading_export: bool,
    loading_import: bool,
    #[allow(dead_code)]
    loading_graph_data: bool,

    // Phase 8: Node/edge type filtering
    filter_show_people: bool,
    filter_show_orgs: bool,
    filter_show_nats: bool,
    filter_show_pki: bool,
    filter_show_yubikey: bool,
    // Phase 9: Graph layout options
    current_layout: GraphLayout,
}

/// Graph layout algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphLayout {
    Manual,        // User-positioned nodes (default)
    Hierarchical,  // Top-down tree layout
    ForceDirected, // Spring/force simulation
    Circular,      // Concentric circles
}

/// Different tabs in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Welcome,
    Organization,
    Export,
}

/// Graph visualization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphView {
    /// Organization structure (default)
    Organization,
    /// NATS infrastructure (operator/accounts/users)
    NatsInfrastructure,
    /// PKI certificate trust chain
    PkiTrustChain,
    /// YubiKey provisioning and PIV slots
    YubiKeyDetails,
}

impl Default for GraphView {
    fn default() -> Self {
        GraphView::Organization
    }
}

/// Messages for the application
#[derive(Debug, Clone)]
pub enum Message {
    // Tab Navigation
    TabSelected(Tab),

    // Graph View Selection
    GraphViewSelected(GraphView),

    // Domain operations
    CreateNewDomain,
    LoadExistingDomain,
    ImportFromSecrets,
    DomainCreated(Result<String, String>),
    DomainLoaded(Result<BootstrapConfig, String>),
    SecretsImported(Result<(crate::domain::Organization, Vec<crate::domain::Person>, Vec<crate::domain::YubiKeyConfig>, Option<String>), String>),
    ManifestDataLoaded(Result<(crate::projections::OrganizationInfo, Vec<crate::projections::LocationEntry>, Vec<crate::projections::PersonEntry>, Vec<crate::projections::CertificateEntry>, Vec<crate::projections::KeyEntry>), String>),

    // Organization form inputs
    OrganizationNameChanged(String),
    OrganizationDomainChanged(String),
    MasterPassphraseChanged(String),
    MasterPassphraseConfirmChanged(String),

    // People operations
    NewPersonNameChanged(String),
    NewPersonEmailChanged(String),
    NewPersonRoleSelected(KeyOwnerRole),
    AddPerson,
    RemovePerson(Uuid),
    SelectPerson(Uuid),
    NodeTypeSelected(String),  // Context-aware node type selection from dropdown

    // Inline editing for newly created nodes
    InlineEditNameChanged(String),
    InlineEditSubmit,
    InlineEditCancel,

    // Location operations
    NewLocationNameChanged(String),
    NewLocationTypeSelected(crate::domain::LocationType),
    NewLocationStreetChanged(String),
    NewLocationCityChanged(String),
    NewLocationRegionChanged(String),
    NewLocationCountryChanged(String),
    NewLocationPostalChanged(String),
    AddLocation,
    RemoveLocation(Uuid),

    // YubiKey operations
    DetectYubiKeys,
    YubiKeysDetected(Result<Vec<crate::ports::yubikey::YubiKeyDevice>, String>),
    YubiKeySerialChanged(String),
    AssignYubiKeyToPerson(Uuid),
    ProvisionYubiKey,
    YubiKeyProvisioned(Result<String, String>),

    // Key generation
    GenerateRootCA,
    IntermediateCANameChanged(String),
    GenerateIntermediateCA,
    ServerCertCNChanged(String),
    ServerCertSANsChanged(String),
    SelectIntermediateCA(String),
    SelectCertLocation(String),
    CertOrganizationChanged(String),
    CertOrganizationalUnitChanged(String),
    CertLocalityChanged(String),
    CertStateProvinceChanged(String),
    CertCountryChanged(String),
    CertValidityDaysChanged(String),
    GenerateServerCert,
    GenerateSSHKeys,
    GenerateAllKeys,
    KeyGenerationProgress(f32),
    KeysGenerated(Result<usize, String>),

    // Export operations
    ExportPathChanged(String),
    TogglePublicKeys(bool),
    ToggleCertificates(bool),
    ToggleNatsConfig(bool),
    TogglePrivateKeys(bool),
    ExportPasswordChanged(String),
    ExportToSDCard,
    DomainExported(Result<String, String>),

    // NATS Hierarchy operations
    GenerateNatsHierarchy,
    NatsHierarchyGenerated(Result<String, String>),
    NatsBootstrapCreated(Box<crate::domain_projections::OrganizationBootstrap>),
    GenerateNatsFromGraph,  // Graph-first NATS generation
    NatsFromGraphGenerated(Result<Vec<(graph::GraphNode, Option<Uuid>)>, String>),
    ExportToNsc,
    NscExported(Result<String, String>),

    // PKI operations
    PkiCertificatesLoaded(Vec<crate::projections::CertificateEntry>),
    GeneratePkiFromGraph,  // Graph-first PKI generation
    PkiGenerated(Result<Vec<(graph::GraphNode, Option<Uuid>)>, String>),

    // YubiKey operations
    YubiKeyDataLoaded(Vec<crate::projections::YubiKeyEntry>, Vec<crate::projections::PersonEntry>),
    ProvisionYubiKeysFromGraph,  // Graph-first YubiKey provisioning
    YubiKeysProvisioned(Result<Vec<(graph::GraphNode, Uuid)>, String>),

    // Root passphrase operations
    RootPassphraseChanged(String),
    RootPassphraseConfirmChanged(String),
    TogglePassphraseVisibility,
    GenerateRandomPassphrase,

    // Overwrite confirmation
    ConfirmOverwrite(bool),  // true = proceed, false = cancel
    DismissOverwriteWarning,

    // Collapsible section toggles
    ToggleRootCA,
    ToggleIntermediateCA,
    ToggleServerCert,
    ToggleYubiKeySection,
    ToggleNatsSection,
    ToggleCertificatesSection,
    ToggleKeysSection,

    // Status messages
    UpdateStatus(String),
    ShowError(String),
    ClearError,

    // Graph interactions
    GraphMessage(GraphMessage),
    CreateContextAwareNode,  // SPACE key: create most likely next node

    // Phase 4: Interactive UI Component Messages
    ContextMenuMessage(ContextMenuMessage),
    PropertyCardMessage(PropertyCardMessage),
    PassphraseDialogMessage(passphrase_dialog::PassphraseDialogMessage),

    // MVI Integration
    MviIntent(Intent),

    // Animation
    AnimationTick,

    // UI Scaling
    IncreaseScale,
    DecreaseScale,
    ResetScale,

    // Search and filtering
    SearchQueryChanged(String),
    ClearSearch,
    HighlightSearchResults,

    // Graph export/import
    ExportGraph,
    GraphExported(Result<String, String>),
    ImportGraph,
    GraphImported(Result<Option<GraphExport>, String>),

    // Help and tooltips
    ToggleHelp,

    // Node/edge type filtering
    ToggleFilterPeople,
    ToggleFilterOrgs,
    ToggleFilterNats,
    ToggleFilterPki,
    ToggleFilterYubiKey,

    // Graph layout options
    ChangeLayout(GraphLayout),
    ApplyLayout,
}

/// Bootstrap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub organization: Option<OrganizationInfo>,
    pub people: Vec<PersonInfo>,
    #[serde(default)]
    pub locations: Vec<LocationInfo>,
    #[serde(default)]
    pub yubikey_assignments: Vec<YubiKeyAssignment>,
    #[serde(rename = "nats_config", default)]
    pub nats_hierarchy: Option<NatsHierarchy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInfo {
    pub name: String,
    pub display_name: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonInfo {
    pub person_id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub name: String,
    pub location_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyAssignment {
    pub serial: String,
    pub name: String,
    pub person_id: Uuid,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsHierarchy {
    pub operator_name: String,
    pub accounts: Vec<String>,
}

/// Serializable graph export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExport {
    pub version: String,
    pub exported_at: String,
    pub graph_view: String,
    pub nodes: Vec<GraphNodeExport>,
    pub edges: Vec<GraphEdgeExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeExport {
    pub id: Uuid,
    pub node_type: String,
    pub position_x: f32,
    pub position_y: f32,
    pub label: String,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdgeExport {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub edge_type: String,
}

impl CimKeysApp {
    fn new(output_dir: String, config: Option<crate::config::Config>) -> (Self, Task<Message>) {
        let aggregate = Arc::new(RwLock::new(KeyManagementAggregate::new(uuid::Uuid::now_v7())));
        let projection = Arc::new(RwLock::new(
            OfflineKeyProjection::new(&output_dir).expect("Failed to create projection")
        ));

        // Initialize with a default org name, will be updated when org is set up
        let default_org = "cim-domain";

        // Initialize MVI ports with mock adapters
        let storage_port: Arc<dyn StoragePort> = Arc::new(InMemoryStorageAdapter::new());
        let x509_port: Arc<dyn X509Port> = Arc::new(MockX509Adapter::new());
        let ssh_port: Arc<dyn SshKeyPort> = Arc::new(MockSshKeyAdapter::new());
        let yubikey_port: Arc<dyn YubiKeyPort> = Arc::new(YubiKeyCliAdapter::new());

        // Initialize MVI model
        let mvi_model = MviModel::new(PathBuf::from(&output_dir));

        // Load existing data from manifest if it exists
        let load_task = {
            let proj = projection.clone();
            Task::perform(
                async move {
                    let proj_read = proj.read().await;
                    let org = proj_read.get_organization();
                    let locations = proj_read.get_locations().to_vec();
                    let people = proj_read.get_people().to_vec();
                    let certificates = proj_read.get_certificates().to_vec();
                    let keys = proj_read.get_keys().to_vec();
                    Ok((org.clone(), locations, people, certificates, keys))
                },
                |result: Result<(crate::projections::OrganizationInfo, Vec<crate::projections::LocationEntry>, Vec<crate::projections::PersonEntry>, Vec<crate::projections::CertificateEntry>, Vec<crate::projections::KeyEntry>), String>| {
                    Message::ManifestDataLoaded(result)
                }
            )
        };

        (
            Self {
                active_tab: Tab::Welcome,
                graph_view: GraphView::default(),
                domain_loaded: false,
                _domain_path: PathBuf::from(&output_dir),
                organization_name: String::new(),
                organization_domain: String::new(),
                organization_id: None,
                admin_email: String::from("admin@example.com"),
                master_passphrase: String::new(),
                master_passphrase_confirm: String::new(),
                bootstrap_config: None,
                aggregate,
                projection,
                config,
                event_emitter: CimEventEmitter::new(default_org),
                _event_subscriber: GuiEventSubscriber::new(default_org),
                org_graph: OrganizationGraph::new(),
                graph_projector: crate::graph_projection::GraphProjector::new(),
                selected_person: None,
                selected_node_type: None,
                editing_new_node: None,
                inline_edit_name: String::new(),
                nats_bootstrap: None,
                new_person_name: String::new(),
                new_person_email: String::new(),
                new_person_role: None,
                new_location_name: String::new(),
                new_location_type: None,
                new_location_street: String::new(),
                new_location_city: String::new(),
                new_location_region: String::new(),
                new_location_country: String::new(),
                new_location_postal: String::new(),
                yubikey_serial: String::new(),
                yubikey_assigned_to: None,
                detected_yubikeys: Vec::new(),
                yubikey_detection_status: "Click 'Detect YubiKeys' to scan for hardware".to_string(),
                yubikey_configs: Vec::new(),
                loaded_locations: Vec::new(),
                loaded_people: Vec::new(),
                loaded_certificates: Vec::new(),
                loaded_keys: Vec::new(),
                key_generation_progress: 0.0,
                keys_generated: 0,
                total_keys_to_generate: 0,
                _certificates_generated: 0,
                intermediate_ca_name_input: String::new(),
                server_cert_cn_input: String::new(),
                server_cert_sans_input: String::new(),
                selected_intermediate_ca: None,
                selected_cert_location: None,
                cert_organization: String::from("CIM Organization"),
                cert_organizational_unit: String::from("Infrastructure"),
                cert_locality: String::from(""),
                cert_state_province: String::from(""),
                cert_country: String::from("US"),
                cert_validity_days: String::from("365"),
                export_path: PathBuf::from(&output_dir),
                include_public_keys: true,
                include_certificates: true,
                include_nats_config: true,
                include_private_keys: false,
                export_password: String::new(),
                nats_hierarchy_generated: false,
                nats_operator_id: None,
                nats_export_path: PathBuf::from(&output_dir).join("nsc"),
                root_ca_collapsed: false,
                intermediate_ca_collapsed: false,
                server_cert_collapsed: false,
                yubikey_section_collapsed: false,
                nats_section_collapsed: false,
                certificates_collapsed: true,
                keys_collapsed: true,
                root_passphrase: String::new(),
                root_passphrase_confirm: String::new(),
                show_passphrase: false,
                status_message: String::from("[LOCK] Welcome to CIM Keys - Offline Key Management System"),
                error_message: None,
                overwrite_warning: None,
                pending_generation_action: None,
                animation_time: 0.0,
                firefly_shader: FireflyRenderer::new(),
                view_model: ViewModel::default(),  // Initialize view model with scale 1.0
                // MVI integration
                mvi_model,
                storage_port,
                x509_port,
                ssh_port,
                yubikey_port,
                // Phase 4: Interactive UI Components
                context_menu: ContextMenu::new(),
                property_card: PropertyCard::new(),
                passphrase_dialog: passphrase_dialog::PassphraseDialog::new(),
                context_menu_node: None,
                // Phase 5: Search and filtering
                search_query: String::new(),
                search_results: Vec::new(),
                highlight_nodes: Vec::new(),
                // Phase 6: Help and tooltips
                show_help_overlay: false,
                // Phase 7: Loading indicators
                loading_export: false,
                loading_import: false,
                loading_graph_data: false,
                // Phase 8: Node/edge type filtering
                filter_show_people: true,
                filter_show_orgs: true,
                filter_show_nats: true,
                filter_show_pki: true,
                filter_show_yubikey: true,
                // Phase 9: Graph layout options
                current_layout: GraphLayout::Manual,
            },
            load_task,
        )
    }


    // Note: Title method removed - window title now set via iced::Settings

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Tab navigation
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                self.status_message = match tab {
                    Tab::Welcome => "Welcome to CIM Keys".to_string(),
                    Tab::Organization => format!("Organization Graph - Primary Interface ({} nodes, {} edges)",
                        self.org_graph.nodes.len(), self.org_graph.edges.len()),
                    Tab::Export => "Export Domain Configuration".to_string(),
                };
                Task::none()
            }

            Message::GraphViewSelected(view) => {
                self.graph_view = view;

                // Populate the graph based on the selected view
                match view {
                    GraphView::Organization => {
                        // Organization view is already populated
                        self.status_message = format!("Organization Structure (Graph: {} nodes, {} edges)",
                            self.org_graph.nodes.len(), self.org_graph.edges.len());
                    }
                    GraphView::NatsInfrastructure => {
                        if let Some(ref bootstrap) = self.nats_bootstrap {
                            // Clear the graph and populate with NATS infrastructure
                            self.org_graph.nodes.clear();
                            self.org_graph.edges.clear();
                            self.org_graph.populate_nats_infrastructure(bootstrap);
                            self.status_message = format!("NATS Infrastructure (Graph: {} nodes, {} edges)",
                                self.org_graph.nodes.len(), self.org_graph.edges.len());
                        } else {
                            self.status_message = "NATS Infrastructure View - Generate NATS hierarchy first".to_string();
                        }
                    }
                    GraphView::PkiTrustChain => {
                        // Get certificates from projection
                        let projection = self.projection.clone();
                        return Task::perform(
                            async move {
                                let proj = projection.read().await;
                                let certs = proj.get_certificates().to_vec();
                                certs
                            },
                            |certs| Message::PkiCertificatesLoaded(certs)
                        );
                    }
                    GraphView::YubiKeyDetails => {
                        // Get YubiKeys and people from projection
                        let projection = self.projection.clone();
                        return Task::perform(
                            async move {
                                let proj = projection.read().await;
                                let yubikeys = proj.get_yubikeys().to_vec();
                                let people = proj.get_people().to_vec();
                                (yubikeys, people)
                            },
                            |(yubikeys, people)| Message::YubiKeyDataLoaded(yubikeys, people)
                        );
                    }
                }
                Task::none()
            }

            // Domain operations
            Message::CreateNewDomain => {
                // Validate inputs
                if self.organization_name.is_empty() {
                    self.error_message = Some("Organization name is required".to_string());
                    return Task::none();
                }
                if self.organization_domain.is_empty() {
                    self.error_message = Some("Domain is required".to_string());
                    return Task::none();
                }

                // Validate passphrase
                let passphrase_matches = self.master_passphrase == self.master_passphrase_confirm;
                let long_enough = self.master_passphrase.len() >= 12;
                let has_upper = self.master_passphrase.chars().any(|c| c.is_uppercase());
                let has_lower = self.master_passphrase.chars().any(|c| c.is_lowercase());
                let has_digit = self.master_passphrase.chars().any(|c| c.is_numeric());
                let has_special = self.master_passphrase.chars().any(|c| !c.is_alphanumeric());

                if !passphrase_matches {
                    self.error_message = Some("Passphrases do not match".to_string());
                    return Task::none();
                }
                if !long_enough {
                    self.error_message = Some("Passphrase must be at least 12 characters".to_string());
                    return Task::none();
                }
                if !(has_upper && has_lower && has_digit && has_special) {
                    self.error_message = Some("Passphrase must contain uppercase, lowercase, number, and special character".to_string());
                    return Task::none();
                }

                // Create organization in projection
                let projection = self.projection.clone();
                let org_name = self.organization_name.clone();
                let org_domain = self.organization_domain.clone();
                let org_id = Uuid::now_v7();  // Generate organization ID

                // Store the org_id for use in person creation
                self.organization_id = Some(org_id);

                let country = self.cert_country.clone();
                let admin_email = self.admin_email.clone();

                Task::perform(
                    async move {
                        let mut proj = projection.write().await;
                        proj.set_organization(
                            org_name.clone(),
                            org_domain.clone(),
                            country,
                            admin_email,
                        )
                        .map(|_| format!("Created domain: {}", org_name))
                        .map_err(|e| format!("Failed to create domain: {}", e))
                    },
                    Message::DomainCreated
                )
            }

            Message::LoadExistingDomain => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Task::perform(load_config_native(), Message::DomainLoaded)
                }
                #[cfg(target_arch = "wasm32")]
                {
                    Task::perform(load_config_wasm(), Message::DomainLoaded)
                }
            }

            Message::ImportFromSecrets => {
                // Import from secrets/cowboyai.json and secrets/secrets.json
                Task::perform(
                    async move {
                        use crate::secrets_loader::SecretsLoader;
                        use std::path::PathBuf;

                        let secrets_path = PathBuf::from("secrets/secrets.json");
                        let cowboyai_path = PathBuf::from("secrets/cowboyai.json");

                        if !secrets_path.exists() || !cowboyai_path.exists() {
                            return Err("Secrets files not found. Please ensure secrets/secrets.json and secrets/cowboyai.json exist.".to_string());
                        }

                        match SecretsLoader::load_from_files(&secrets_path, &cowboyai_path) {
                            Ok(data) => Ok(data),
                            Err(e) => Err(format!("Failed to load secrets: {}", e)),
                        }
                    },
                    Message::SecretsImported
                )
            }

            Message::SecretsImported(result) => {
                match result {
                    Ok((org, people, yubikey_configs, master_passphrase)) => {
                        // Set organization info
                        self.organization_name = org.name.clone();
                        self.organization_domain = org.display_name.clone();
                        self.organization_id = Some(org.id);

                        // Set master passphrase if provided
                        if let Some(passphrase) = master_passphrase {
                            self.master_passphrase = passphrase.clone();
                            self.master_passphrase_confirm = passphrase;
                        }

                        // Populate graph with people
                        for person in &people {
                            // Determine role based on YubiKey configs
                            let role = yubikey_configs
                                .iter()
                                .find(|yk| yk.owner_email == person.email)
                                .map(|yk| match yk.role {
                                    crate::domain::YubiKeyRole::RootCA => KeyOwnerRole::RootAuthority,
                                    crate::domain::YubiKeyRole::Backup => KeyOwnerRole::BackupHolder,
                                    crate::domain::YubiKeyRole::Service => KeyOwnerRole::ServiceAccount,
                                    crate::domain::YubiKeyRole::User => KeyOwnerRole::Developer,
                                })
                                .unwrap_or(KeyOwnerRole::Developer);

                            self.org_graph.add_node(person.clone(), role);
                        }

                        // Store YubiKey configs
                        self.yubikey_configs = yubikey_configs.clone();

                        self.domain_loaded = true;
                        self.active_tab = Tab::Organization;
                        self.status_message = format!(
                            "Imported {} ({}) with {} people and {} YubiKeys",
                            org.display_name,
                            org.name,
                            people.len(),
                            yubikey_configs.len()
                        );
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }

            Message::ManifestDataLoaded(result) => {
                match result {
                    Ok((org_info, locations, people, certificates, keys)) => {
                        // Populate organization info if available
                        if !org_info.name.is_empty() {
                            self.organization_name = org_info.name.clone();
                            self.organization_domain = org_info.domain.clone();
                            self.status_message = format!("Loaded organization: {}", org_info.name);
                        }

                        // Store loaded data
                        self.loaded_locations = locations.clone();
                        self.loaded_people = people.clone();
                        self.loaded_certificates = certificates.clone();
                        self.loaded_keys = keys.clone();

                        // Build status message
                        let mut loaded_items = Vec::new();
                        if !locations.is_empty() {
                            loaded_items.push(format!("{} locations", locations.len()));
                        }
                        if !people.is_empty() {
                            loaded_items.push(format!("{} people", people.len()));
                        }
                        if !certificates.is_empty() {
                            loaded_items.push(format!("{} certificates", certificates.len()));
                        }
                        if !keys.is_empty() {
                            loaded_items.push(format!("{} keys", keys.len()));
                        }

                        if !loaded_items.is_empty() {
                            self.status_message = format!("Loaded from manifest: {}", loaded_items.join(", "));
                        }
                    }
                    Err(_e) => {
                        // Silently ignore - manifest might not exist yet
                    }
                }
                Task::none()
            }

            Message::DomainLoaded(result) => {
                match result {
                    Ok(config) => {
                        // Update organization info
                        if let Some(ref org) = config.organization {
                            self.organization_name = org.name.clone();
                            self.organization_domain = org.display_name.clone();
                        }

                        // Populate graph with people from config
                        for person_config in &config.people {
                            let person = Person {
                                id: person_config.person_id,
                                name: person_config.name.clone(),
                                email: person_config.email.clone(),
                                organization_id: Uuid::now_v7(), // TODO: Use actual org ID from config
                                unit_ids: vec![],
                                roles: vec![],
                                active: true,
                                created_at: chrono::Utc::now(),
                            };

                            // Map role string to enum
                            let role = match person_config.role.as_str() {
                                "RootAuthority" => KeyOwnerRole::RootAuthority,
                                "SecurityAdmin" => KeyOwnerRole::SecurityAdmin,
                                "Developer" => KeyOwnerRole::Developer,
                                "ServiceAccount" => KeyOwnerRole::ServiceAccount,
                                "BackupHolder" => KeyOwnerRole::BackupHolder,
                                "Auditor" => KeyOwnerRole::Auditor,
                                _ => KeyOwnerRole::Developer,
                            };

                            self.org_graph.add_node(person, role);
                        }

                        self.bootstrap_config = Some(config);
                        self.domain_loaded = true;
                        self.active_tab = Tab::Organization;
                        self.status_message = format!("Loaded {} people from configuration", self.org_graph.node_count());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load domain: {}", e));
                    }
                }
                Task::none()
            }

            // Organization inputs
            Message::OrganizationNameChanged(value) => {
                self.organization_name = value;
                Task::none()
            }

            Message::OrganizationDomainChanged(value) => {
                self.organization_domain = value;
                Task::none()
            }

            Message::MasterPassphraseChanged(value) => {
                self.master_passphrase = value;
                Task::none()
            }

            Message::MasterPassphraseConfirmChanged(value) => {
                self.master_passphrase_confirm = value;
                Task::none()
            }

            // People operations
            Message::NewPersonNameChanged(value) => {
                self.new_person_name = value;
                Task::none()
            }

            Message::NewPersonEmailChanged(value) => {
                self.new_person_email = value;
                Task::none()
            }

            Message::NewPersonRoleSelected(role) => {
                self.new_person_role = Some(role);
                Task::none()
            }

            Message::NodeTypeSelected(node_type) => {
                self.selected_node_type = Some(node_type.clone());
                self.status_message = format!("Click on canvas to place new {} node", node_type);
                Task::none()
            }

            Message::InlineEditNameChanged(name) => {
                self.inline_edit_name = name;
                Task::none()
            }

            Message::InlineEditSubmit => {
                if let Some(node_id) = self.editing_new_node {
                    // FRP: Create event instead of direct mutation
                    if let Some(node) = self.org_graph.nodes.get(&node_id) {
                        use crate::gui::graph::NodeType;
                        use crate::gui::graph_events::GraphEvent;
                        use chrono::Utc;

                        // Clone old state for compensating event (undo)
                        let old_node_type = node.node_type.clone();
                        let old_label = node.label.clone();

                        // Create new node type with updated name in domain entity
                        let new_node_type = match &node.node_type {
                            NodeType::Person { person, role } => {
                                let mut updated_person = person.clone();
                                updated_person.name = self.inline_edit_name.clone();
                                NodeType::Person { person: updated_person, role: *role }
                            }
                            NodeType::OrganizationalUnit(unit) => {
                                let mut updated_unit = unit.clone();
                                updated_unit.name = self.inline_edit_name.clone();
                                NodeType::OrganizationalUnit(updated_unit)
                            }
                            other => other.clone(), // No change for other types
                        };

                        // Create immutable event
                        let event = GraphEvent::NodePropertiesChanged {
                            node_id,
                            old_node_type,
                            old_label,
                            new_node_type,
                            new_label: self.inline_edit_name.clone(),
                            timestamp: Utc::now(),
                        };

                        // Apply event through event sourcing system
                        self.org_graph.event_stack.push(event.clone());
                        self.org_graph.apply_event(&event);

                        self.status_message = format!("Updated node name to '{}'", self.inline_edit_name);
                    }

                    // Clear editing state
                    self.editing_new_node = None;
                    self.inline_edit_name.clear();
                }
                Task::none()
            }

            Message::InlineEditCancel => {
                if self.editing_new_node.is_some() {
                    // Cancel inline editing
                    self.editing_new_node = None;
                    self.inline_edit_name.clear();
                    self.status_message = "Edit cancelled".to_string();
                } else {
                    // No inline edit active - cancel edge creation instead
                    self.org_graph.handle_message(crate::gui::graph::GraphMessage::CancelEdgeCreation);
                    if self.org_graph.edge_indicator.is_active() {
                        self.status_message = "Edge creation cancelled".to_string();
                    }
                }
                Task::none()
            }

            Message::AddPerson => {
                // Validate inputs
                if self.new_person_name.is_empty() || self.new_person_email.is_empty() {
                    self.error_message = Some("Please enter name and email".to_string());
                    return Task::none();
                }

                // Validate role is selected
                if self.new_person_role.is_none() {
                    self.error_message = Some("Please select a role".to_string());
                    return Task::none();
                }

                // Validate domain is created
                let org_id = match self.organization_id {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Please create a domain first".to_string());
                        return Task::none();
                    }
                };

                let person_id = Uuid::now_v7();
                let person = Person {
                    id: person_id,
                    name: self.new_person_name.clone(),
                    email: self.new_person_email.clone(),
                    organization_id: org_id,
                    unit_ids: vec![],
                    roles: vec![],
                    active: true,
                    created_at: chrono::Utc::now(),
                };

                let role = self.new_person_role.unwrap();

                // Add to graph for visualization
                self.org_graph.add_node(person.clone(), role);

                // Persist to projection
                let projection = self.projection.clone();
                let person_name = person.name.clone();
                let person_email = person.email.clone();
                let role_string = format!("{:?}", role);

                // Clear form fields immediately
                self.new_person_name.clear();
                self.new_person_email.clear();
                self.new_person_role = None;

                Task::perform(
                    async move {
                        let mut proj = projection.write().await;
                        proj.add_person(person_id, person_name.clone(), person_email, role_string, org_id)
                            .map(|_| format!("Added {} to organization", person_name))
                            .map_err(|e| format!("Failed to add person: {}", e))
                    },
                    |result| match result {
                        Ok(msg) => Message::UpdateStatus(msg),
                        Err(e) => Message::ShowError(e),
                    }
                )
            }

            Message::RemovePerson(person_id) => {
                // TODO: Remove person from graph and domain
                self.status_message = format!("Removed person {} from organization", person_id);
                Task::none()
            }

            Message::SelectPerson(person_id) => {
                self.selected_person = Some(person_id);
                self.org_graph.select_node(person_id);
                Task::none()
            }

            // Location operations
            Message::NewLocationNameChanged(value) => {
                self.new_location_name = value;
                Task::none()
            }

            Message::NewLocationTypeSelected(location_type) => {
                self.new_location_type = Some(location_type);
                Task::none()
            }

            Message::NewLocationStreetChanged(value) => {
                self.new_location_street = value;
                Task::none()
            }

            Message::NewLocationCityChanged(value) => {
                self.new_location_city = value;
                Task::none()
            }

            Message::NewLocationRegionChanged(value) => {
                self.new_location_region = value;
                Task::none()
            }

            Message::NewLocationCountryChanged(value) => {
                self.new_location_country = value;
                Task::none()
            }

            Message::NewLocationPostalChanged(value) => {
                self.new_location_postal = value;
                Task::none()
            }

            Message::AddLocation => {
                // Validate inputs
                if self.new_location_name.is_empty() {
                    self.error_message = Some("Location name is required".to_string());
                    return Task::none();
                }

                // Validate address fields for physical location
                if self.new_location_street.is_empty() {
                    self.error_message = Some("Street address is required".to_string());
                    return Task::none();
                }
                if self.new_location_city.is_empty() {
                    self.error_message = Some("City is required".to_string());
                    return Task::none();
                }
                if self.new_location_region.is_empty() {
                    self.error_message = Some("State/Region is required".to_string());
                    return Task::none();
                }
                if self.new_location_country.is_empty() {
                    self.error_message = Some("Country is required".to_string());
                    return Task::none();
                }
                if self.new_location_postal.is_empty() {
                    self.error_message = Some("Postal code is required".to_string());
                    return Task::none();
                }

                // Validate domain is created
                let org_id = match self.organization_id {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Please create a domain first".to_string());
                        return Task::none();
                    }
                };

                let location_id = Uuid::now_v7();
                let location_name = self.new_location_name.clone();
                let location_type = "Physical".to_string();

                // Clone address values for the async task
                let street = self.new_location_street.clone();
                let city = self.new_location_city.clone();
                let region = self.new_location_region.clone();
                let country = self.new_location_country.clone();
                let postal = self.new_location_postal.clone();

                // Clear form fields immediately
                self.new_location_name.clear();
                self.new_location_type = None;
                self.new_location_street.clear();
                self.new_location_city.clear();
                self.new_location_region.clear();
                self.new_location_country.clear();
                self.new_location_postal.clear();

                // Persist to projection with full address details
                let projection = self.projection.clone();

                Task::perform(
                    async move {
                        let mut proj = projection.write().await;
                        proj.add_location(
                            location_id,
                            location_name.clone(),
                            location_type,
                            org_id,
                            Some(street),
                            Some(city),
                            Some(region),
                            Some(country),
                            Some(postal),
                        )
                        .map(|_| format!("Added location: {}", location_name))
                        .map_err(|e| format!("Failed to add location: {}", e))
                    },
                    |result| match result {
                        Ok(msg) => Message::UpdateStatus(msg),
                        Err(e) => Message::ShowError(e),
                    }
                )
            }

            Message::RemoveLocation(location_id) => {
                let projection = self.projection.clone();

                Task::perform(
                    async move {
                        let mut proj = projection.write().await;
                        proj.remove_location(location_id)
                            .map(|_| "Location removed successfully".to_string())
                            .map_err(|e| format!("Failed to remove location: {}", e))
                    },
                    |result| match result {
                        Ok(msg) => Message::UpdateStatus(msg),
                        Err(e) => Message::ShowError(e),
                    }
                )
            }

            // YubiKey operations
            Message::DetectYubiKeys => {
                self.yubikey_detection_status = "Detecting YubiKeys...".to_string();
                let yubikey_port = self.yubikey_port.clone();

                Task::perform(
                    async move {
                        yubikey_port.list_devices().await
                    },
                    |result| Message::YubiKeysDetected(result.map_err(|e| format!("{:?}", e)))
                )
            }

            Message::YubiKeysDetected(result) => {
                match result {
                    Ok(devices) => {
                        self.detected_yubikeys = devices.clone();
                        self.yubikey_detection_status = format!("Found {} YubiKey(s)", devices.len());
                        if !devices.is_empty() {
                            self.status_message = format!("Detected: {}",
                                devices.iter()
                                    .map(|d| format!("{} (Serial: {})", d.model, d.serial))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                        }
                    }
                    Err(e) => {
                        self.yubikey_detection_status = "Detection failed".to_string();
                        self.error_message = Some(format!("YubiKey detection error: {}", e));
                    }
                }
                Task::none()
            }

            Message::YubiKeySerialChanged(value) => {
                self.yubikey_serial = value;
                Task::none()
            }

            Message::AssignYubiKeyToPerson(person_id) => {
                self.yubikey_assigned_to = Some(person_id);
                self.status_message = format!("YubiKey {} assigned to person", self.yubikey_serial);
                Task::none()
            }

            Message::ProvisionYubiKey => {
                // Provision all detected YubiKeys with their configurations
                if self.detected_yubikeys.is_empty() {
                    self.error_message = Some("No YubiKeys detected. Please run detection first.".to_string());
                    return Task::none();
                }

                if self.yubikey_configs.is_empty() {
                    self.error_message = Some("No YubiKey configurations loaded. Please import secrets first.".to_string());
                    return Task::none();
                }

                self.status_message = "Provisioning YubiKeys...".to_string();

                let detected = self.detected_yubikeys.clone();
                let configs = self.yubikey_configs.clone();
                let yubikey_port = self.yubikey_port.clone();

                Task::perform(
                    async move {
                        use crate::ports::yubikey::{PivSlot, KeyAlgorithm};
                        use crate::ports::yubikey::SecureString;

                        let mut results = Vec::new();

                        for device in &detected {
                            // Find matching config by serial number
                            if let Some(config) = configs.iter().find(|c| c.serial == device.serial) {
                                let serial = device.serial.clone();

                                // Determine PIV slot based on role
                                let slot = match config.role {
                                    crate::domain::YubiKeyRole::RootCA => PivSlot::Signature,  // 9C - for signing
                                    crate::domain::YubiKeyRole::Backup => PivSlot::KeyManagement,  // 9D - for backup
                                    crate::domain::YubiKeyRole::User => PivSlot::Authentication,  // 9A - for auth
                                    crate::domain::YubiKeyRole::Service => PivSlot::CardAuth,  // 9E - for service
                                };

                                // Use PIN from config
                                let pin = SecureString::new(config.piv.pin.as_bytes());

                                // Generate key in the slot
                                match yubikey_port.generate_key_in_slot(
                                    &serial,
                                    slot,
                                    KeyAlgorithm::EccP256,  // Default to P-256
                                    &pin
                                ).await {
                                    Ok(_public_key) => {
                                        results.push(format!(
                                            "[OK] {} ({}) - {} provisioned in slot {:?}",
                                            config.name,
                                            serial,
                                            match config.role {
                                                crate::domain::YubiKeyRole::RootCA => "Root CA",
                                                crate::domain::YubiKeyRole::Backup => "Backup",
                                                crate::domain::YubiKeyRole::User => "User",
                                                crate::domain::YubiKeyRole::Service => "Service",
                                            },
                                            slot
                                        ));
                                    }
                                    Err(e) => {
                                        results.push(format!(
                                            "✗ {} ({}) - Failed: {:?}",
                                            config.name,
                                            serial,
                                            e
                                        ));
                                    }
                                }
                            } else {
                                results.push(format!(
                                    "⚠ Serial {} detected but no configuration found",
                                    device.serial
                                ));
                            }
                        }

                        if results.iter().all(|r| r.starts_with("[OK]")) {
                            Ok(results.join("\n"))
                        } else {
                            Err(results.join("\n"))
                        }
                    },
                    Message::YubiKeyProvisioned
                )
            }

            // Key generation
            Message::GenerateRootCA => {
                self.status_message = "Generating Root CA certificate...".to_string();
                self.key_generation_progress = 0.1;

                // TODO: Convert to MVI pattern like intermediate CA and server cert generation
                // For now, using legacy aggregate/projection pattern
                let aggregate = self.aggregate.clone();
                let projection = self.projection.clone();

                Task::perform(
                    async move {
                        generate_root_ca(aggregate, projection).await
                    },
                    |result| match result {
                        Ok(cert_id) => Message::UpdateStatus(format!("Root CA generated: {}", cert_id)),
                        Err(e) => Message::ShowError(format!("Root CA generation failed: {}", e)),
                    }
                )
            }

            Message::GeneratePkiFromGraph => {
                self.status_message = "Generating PKI hierarchy from organizational graph...".to_string();

                // Clone the graph and passphrase for async task
                let graph = self.org_graph.clone();
                let passphrase = self.root_passphrase.clone();

                Task::perform(
                    async move {
                        // Generate PKI from graph structure
                        graph_pki::generate_pki_from_graph(&graph, &passphrase)
                    },
                    Message::PkiGenerated
                )
            }

            Message::PkiGenerated(result) => {
                match result {
                    Ok(certificate_nodes) => {
                        // Add certificate nodes to graph
                        graph_pki::add_pki_to_graph(&mut self.org_graph, certificate_nodes);

                        let count = self.org_graph.nodes.iter()
                            .filter(|(_, node)| matches!(
                                node.node_type,
                                graph::NodeType::RootCertificate{..} |
                                graph::NodeType::IntermediateCertificate{..} |
                                graph::NodeType::LeafCertificate{..}
                            ))
                            .count();

                        self.status_message = format!("✅ PKI hierarchy generated! {} certificates created from organizational structure", count);
                        tracing::info!("Graph-first PKI generation complete: {} certificates", count);
                    }
                    Err(e) => {
                        self.status_message = format!("❌ PKI generation failed: {}", e);
                        self.error_message = Some(e);
                        tracing::error!("Graph-first PKI generation failed");
                    }
                }
                Task::none()
            }

            Message::GenerateNatsFromGraph => {
                self.status_message = "Generating NATS infrastructure from organizational graph...".to_string();

                // Clone the graph for async task
                let graph = self.org_graph.clone();

                Task::perform(
                    async move {
                        // Generate NATS from graph structure
                        graph_nats::generate_nats_from_graph(&graph)
                    },
                    Message::NatsFromGraphGenerated
                )
            }

            Message::NatsFromGraphGenerated(result) => {
                match result {
                    Ok(nats_nodes) => {
                        // Add NATS nodes to graph
                        graph_nats::add_nats_to_graph(&mut self.org_graph, nats_nodes);

                        let count = self.org_graph.nodes.iter()
                            .filter(|(_, node)| matches!(
                                node.node_type,
                                graph::NodeType::NatsOperator(_) |
                                graph::NodeType::NatsAccount(_) |
                                graph::NodeType::NatsUser(_)
                            ))
                            .count();

                        self.status_message = format!("✅ NATS infrastructure generated! {} entities created from organizational structure", count);
                        tracing::info!("Graph-first NATS generation complete: {} entities", count);
                    }
                    Err(e) => {
                        self.status_message = format!("❌ NATS generation failed: {}", e);
                        self.error_message = Some(e);
                        tracing::error!("Graph-first NATS generation failed");
                    }
                }
                Task::none()
            }

            Message::ProvisionYubiKeysFromGraph => {
                self.status_message = "Analyzing YubiKey provisioning requirements from organizational graph...".to_string();

                // Clone the graph for async task
                let graph = self.org_graph.clone();

                Task::perform(
                    async move {
                        // Generate YubiKey provision plan from graph structure
                        graph_yubikey::generate_yubikey_provision_from_graph(&graph)
                    },
                    Message::YubiKeysProvisioned
                )
            }

            Message::YubiKeysProvisioned(result) => {
                match result {
                    Ok(yubikey_nodes) => {
                        // Add YubiKey status nodes to graph
                        graph_yubikey::add_yubikey_status_to_graph(&mut self.org_graph, yubikey_nodes);

                        let count = self.org_graph.nodes.iter()
                            .filter(|(_, node)| matches!(
                                node.node_type,
                                graph::NodeType::YubiKeyStatus{..}
                            ))
                            .count();

                        self.status_message = format!("✅ YubiKey provisioning analyzed! {} people require YubiKeys based on roles", count);
                        tracing::info!("Graph-first YubiKey analysis complete: {} provision plans", count);
                    }
                    Err(e) => {
                        self.status_message = format!("❌ YubiKey analysis failed: {}", e);
                        self.error_message = Some(e);
                        tracing::error!("Graph-first YubiKey analysis failed");
                    }
                }
                Task::none()
            }

            Message::IntermediateCANameChanged(name) => {
                self.intermediate_ca_name_input = name;
                Task::none()
            }

            Message::GenerateIntermediateCA => {
                // Update status message
                self.status_message = format!("Generating intermediate CA '{}'...", self.intermediate_ca_name_input);
                self.key_generation_progress = 0.2;

                // Create MVI Intent
                let intent = Intent::UiGenerateIntermediateCAClicked {
                    name: self.intermediate_ca_name_input.clone(),
                };

                // Call MVI update and wire back to MviIntent message
                let (updated_model, task) = crate::mvi::update(
                    self.mvi_model.clone(),
                    intent,
                    self.storage_port.clone(),
                    self.x509_port.clone(),
                    self.ssh_port.clone(),
                    self.yubikey_port.clone(),
                );

                self.mvi_model = updated_model;
                task.map(Message::MviIntent)
            }

            Message::ServerCertCNChanged(cn) => {
                self.server_cert_cn_input = cn;
                Task::none()
            }

            Message::ServerCertSANsChanged(sans) => {
                self.server_cert_sans_input = sans;
                Task::none()
            }

            Message::SelectIntermediateCA(ca_name) => {
                self.selected_intermediate_ca = Some(ca_name);
                Task::none()
            }

            Message::SelectCertLocation(location) => {
                self.selected_cert_location = Some(location);
                Task::none()
            }

            Message::CertOrganizationChanged(org) => {
                self.cert_organization = org;
                Task::none()
            }

            Message::CertOrganizationalUnitChanged(ou) => {
                self.cert_organizational_unit = ou;
                Task::none()
            }

            Message::CertLocalityChanged(locality) => {
                self.cert_locality = locality;
                Task::none()
            }

            Message::CertStateProvinceChanged(state) => {
                self.cert_state_province = state;
                Task::none()
            }

            Message::CertCountryChanged(country) => {
                self.cert_country = country;
                Task::none()
            }

            Message::CertValidityDaysChanged(days) => {
                self.cert_validity_days = days;
                Task::none()
            }

            Message::GenerateServerCert => {
                if let Some(ref ca_name) = self.selected_intermediate_ca {
                    // Check if a certificate with this CN already exists
                    let cn_to_check = format!("CN={}", self.server_cert_cn_input);
                    let existing_cert = self.loaded_certificates.iter()
                        .find(|cert| cert.subject.contains(&cn_to_check));

                    if let Some(existing) = existing_cert {
                        // Certificate already exists, show overwrite warning
                        self.overwrite_warning = Some(format!(
                            "A server certificate with CN='{}' already exists (created {}). Do you want to overwrite it?",
                            self.server_cert_cn_input,
                            existing.not_before.format("%Y-%m-%d")
                        ));
                        self.pending_generation_action = Some(Message::GenerateServerCert);
                        return Task::none();
                    }

                    self.status_message = format!(
                        "Generating server certificate for '{}' signed by '{}'...",
                        self.server_cert_cn_input, ca_name
                    );
                    self.key_generation_progress = 0.3;

                    // Parse SANs from comma-separated input
                    let san_entries: Vec<String> = self.server_cert_sans_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    // Create MVI Intent
                    let intent = Intent::UiGenerateServerCertClicked {
                        common_name: self.server_cert_cn_input.clone(),
                        san_entries,
                        intermediate_ca_name: ca_name.clone(),
                    };

                    // Call MVI update and wire back to MviIntent message
                    let (updated_model, task) = crate::mvi::update(
                        self.mvi_model.clone(),
                        intent,
                        self.storage_port.clone(),
                        self.x509_port.clone(),
                        self.ssh_port.clone(),
                        self.yubikey_port.clone(),
                    );

                    self.mvi_model = updated_model;
                    task.map(Message::MviIntent)
                } else {
                    self.error_message = Some("Please select an intermediate CA first".to_string());
                    Task::none()
                }
            }

            Message::GenerateSSHKeys => {
                self.status_message = "Generating SSH keys for all users...".to_string();
                self.key_generation_progress = 0.3;
                // TODO: Implement SSH key generation
                Task::none()
            }

            Message::GenerateAllKeys => {
                // Emit command to generate keys
                // In true CIM style, we don't directly call the aggregate
                // Instead we emit a command that will be processed asynchronously

                let root_ca_command = crate::commands::KeyCommand::GenerateCertificate(
                    crate::commands::GenerateCertificateCommand {
                        command_id: cim_domain::EntityId::new(),
                        key_id: uuid::Uuid::now_v7(),
                        subject: crate::commands::CertificateSubject {
                            common_name: self.organization_name.clone(),
                            organization: Some(self.organization_domain.clone()),
                            country: Some("US".to_string()),
                            organizational_unit: None,
                            locality: None,
                            state_or_province: None,
                        },
                        validity_days: 3650,
                        is_ca: true,
                        san: vec![],
                        key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
                        extended_key_usage: vec![],
                        requestor: "GUI".to_string(),
                        context: None,
                    }
                );

                self.event_emitter.emit_command(
                    root_ca_command,
                    "GenerateKeys",
                    InteractionType::ButtonClick {
                        button_id: "start_generation".to_string()
                    }
                );

                // Start a new correlation for the key generation workflow
                self.event_emitter.new_correlation();

                let aggregate = self.aggregate.clone();
                let projection = self.projection.clone();
                let mut emitter = self.event_emitter.clone();

                // Process commands through the aggregate
                Task::perform(
                    async move {
                        // Drain and process all pending commands
                        let commands = emitter.drain_commands();
                        let mut total_keys = 0;

                        for domain_command in commands {
                            let aggregate = aggregate.read().await;
                            let projection = projection.read().await;

                            match aggregate.handle_command(
                                domain_command.command,
                                &projection,
                                None,  // No NATS port in offline mode
                                #[cfg(feature = "policy")]
                                None   // No policy engine in GUI yet
                            ).await {
                                Ok(events) => {
                                    total_keys += events.len();
                                    // In a real system, events would be published to NATS
                                    // Here we're offline, so we just process them locally
                                }
                                Err(e) => {
                                    return Err(format!("Command failed: {}", e));
                                }
                            }
                        }

                        Ok(total_keys)
                    },
                    Message::KeysGenerated
                )
            }

            Message::KeysGenerated(result) => {
                match result {
                    Ok(count) => {
                        self.keys_generated = count;
                        self.key_generation_progress = 1.0;
                        self.status_message = format!("Generated {} keys successfully", count);
                        self.active_tab = Tab::Export;
                    }
                    Err(e) => {
                        self.status_message = format!("Key generation failed: {}", e);
                    }
                }
                Task::none()
            }

            // Export operations
            Message::ExportPathChanged(path) => {
                self.export_path = PathBuf::from(path);
                Task::none()
            }

            Message::TogglePublicKeys(enabled) => {
                self.include_public_keys = enabled;
                Task::none()
            }

            Message::ToggleCertificates(enabled) => {
                self.include_certificates = enabled;
                Task::none()
            }

            Message::ToggleNatsConfig(enabled) => {
                self.include_nats_config = enabled;
                Task::none()
            }

            Message::TogglePrivateKeys(enabled) => {
                self.include_private_keys = enabled;
                Task::none()
            }

            Message::ExportPasswordChanged(password) => {
                self.export_password = password;
                Task::none()
            }

            Message::ExportToSDCard => {
                let projection = self.projection.clone();
                Task::perform(export_domain(projection), Message::DomainExported)
            }

            Message::DomainExported(result) => {
                match result {
                    Ok(path) => {
                        self.status_message = format!("Domain exported to: {}", path);
                    }
                    Err(e) => {
                        self.status_message = format!("Export failed: {}", e);
                    }
                }
                Task::none()
            }

            // NATS Hierarchy operations
            Message::GenerateNatsHierarchy => {
                if self.organization_name.is_empty() {
                    self.error_message = Some("Create organization first".to_string());
                    return Task::none();
                }

                let org_name = self.organization_name.clone();
                let projection = self.projection.clone();

                Task::perform(
                    generate_nats_hierarchy(org_name, projection),
                    Message::NatsHierarchyGenerated
                )
            }

            Message::NatsHierarchyGenerated(result) => {
                match result {
                    Ok(operator_id) => {
                        self.nats_hierarchy_generated = true;
                        self.nats_operator_id = Some(Uuid::parse_str(&operator_id).unwrap_or_else(|_| Uuid::now_v7()));
                        self.status_message = format!("[OK] NATS hierarchy generated for {}", self.organization_name);

                        // Build OrganizationBootstrap for graph visualization
                        let org_id = self.organization_id.unwrap_or_else(|| Uuid::now_v7());
                        let org_name = self.organization_name.clone();
                        let org_domain = self.organization_domain.clone();
                        let projection = self.projection.clone();

                        return Task::perform(
                            async move {
                                let proj = projection.read().await;
                                let people_info = proj.get_people();

                                // Construct domain Organization for NATS projection
                                use crate::domain::{Organization, OrganizationUnit, OrganizationUnitType, Person, PersonRole, RoleType, RoleScope, Permission};
                                use std::collections::HashMap;

                                let org = Organization {
                                    id: org_id,
                                    name: org_name.clone(),
                                    display_name: org_name.clone(),
                                    description: Some(format!("Organization for {}", org_domain)),
                                    parent_id: None,
                                    units: vec![
                                        OrganizationUnit {
                                            id: org_id,  // Use same ID for default unit
                                            name: format!("{} - Default", org_name),
                                            unit_type: OrganizationUnitType::Infrastructure,
                                            parent_unit_id: None,
                                            responsible_person_id: None,
                                        }
                                    ],
                                    created_at: chrono::Utc::now(),
                                    metadata: HashMap::new(),
                                };

                                // Convert PersonEntry to domain Person
                                let people: Vec<Person> = people_info.iter().map(|p| Person {
                                    id: p.person_id,
                                    name: p.name.clone(),
                                    email: p.email.clone(),
                                    roles: vec![PersonRole {
                                        role_type: RoleType::Operator,  // Default role for visualization
                                        scope: RoleScope::Organization,
                                        permissions: vec![Permission::ViewAuditLogs],
                                    }],
                                    organization_id: org_id,
                                    unit_ids: vec![org_id],  // Assign to default unit
                                    created_at: p.created_at,
                                    active: true,
                                }).collect();

                                // Use NatsProjection to bootstrap the organization
                                use crate::domain_projections::NatsProjection;
                                let bootstrap = NatsProjection::bootstrap_organization(&org, &people);
                                bootstrap
                            },
                            |bootstrap| Message::NatsBootstrapCreated(Box::new(bootstrap))
                        );
                    }
                    Err(e) => {
                        self.error_message = Some(format!("NATS generation failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::NatsBootstrapCreated(bootstrap) => {
                // Store the bootstrap for graph visualization
                self.nats_bootstrap = Some(*bootstrap);

                // Automatically switch to NATS Infrastructure view
                self.graph_view = GraphView::NatsInfrastructure;

                // Populate the graph with NATS infrastructure
                if let Some(ref bootstrap) = self.nats_bootstrap {
                    self.org_graph.nodes.clear();
                    self.org_graph.edges.clear();
                    self.org_graph.populate_nats_infrastructure(bootstrap);
                    self.status_message = format!("[OK] NATS Infrastructure visualized ({} nodes, {} edges)",
                        self.org_graph.nodes.len(), self.org_graph.edges.len());
                }

                Task::none()
            }

            Message::PkiCertificatesLoaded(certificates) => {
                // Populate the PKI trust chain graph
                self.org_graph.nodes.clear();
                self.org_graph.edges.clear();

                if certificates.is_empty() {
                    self.status_message = "PKI Trust Chain View - No certificates generated yet".to_string();
                } else {
                    self.org_graph.populate_pki_trust_chain(&certificates);
                    self.status_message = format!("PKI Trust Chain ({} certificates, {} nodes, {} edges)",
                        certificates.len(),
                        self.org_graph.nodes.len(),
                        self.org_graph.edges.len());
                }

                Task::none()
            }

            Message::YubiKeyDataLoaded(yubikeys, people) => {
                // Populate the YubiKey hardware graph
                self.org_graph.nodes.clear();
                self.org_graph.edges.clear();

                if yubikeys.is_empty() {
                    self.status_message = "YubiKey Details View - No YubiKeys provisioned yet".to_string();
                } else {
                    self.org_graph.populate_yubikey_graph(&yubikeys, &people);
                    self.status_message = format!("YubiKey Details ({} devices, {} nodes, {} edges)",
                        yubikeys.len(),
                        self.org_graph.nodes.len(),
                        self.org_graph.edges.len());
                }

                Task::none()
            }

            Message::ExportToNsc => {
                if !self.nats_hierarchy_generated {
                    self.error_message = Some("Generate NATS hierarchy first".to_string());
                    return Task::none();
                }

                let export_path = self.nats_export_path.clone();
                let projection = self.projection.clone();

                Task::perform(
                    export_nats_to_nsc(export_path, projection),
                    Message::NscExported
                )
            }

            Message::NscExported(result) => {
                match result {
                    Ok(path) => {
                        self.status_message = format!("[OK] NATS hierarchy exported to NSC store: {}", path);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("NSC export failed: {}", e));
                    }
                }
                Task::none()
            }

            // Root passphrase operations
            Message::RootPassphraseChanged(passphrase) => {
                self.root_passphrase = passphrase;
                Task::none()
            }

            Message::RootPassphraseConfirmChanged(passphrase) => {
                self.root_passphrase_confirm = passphrase;
                Task::none()
            }

            Message::TogglePassphraseVisibility => {
                self.show_passphrase = !self.show_passphrase;
                Task::none()
            }

            Message::GenerateRandomPassphrase => {
                use rand::Rng;
                const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%^&*()-_=+[]{}|;:,.<>?";
                let mut rng = rand::thread_rng();
                let length = rng.gen_range(24..=32);
                let passphrase: String = (0..length)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect();

                self.root_passphrase = passphrase.clone();
                self.root_passphrase_confirm = passphrase;
                self.show_passphrase = true;  // Auto-show when generated
                Task::none()
            }

            // Overwrite confirmation
            Message::ConfirmOverwrite(proceed) => {
                if proceed {
                    // User confirmed, proceed with the pending action
                    if let Some(action) = self.pending_generation_action.take() {
                        self.overwrite_warning = None;
                        return self.update(action);
                    }
                } else {
                    // User cancelled, clear the warning and pending action
                    self.overwrite_warning = None;
                    self.pending_generation_action = None;
                    self.status_message = "[CANCEL] Generation cancelled by user".to_string();
                }
                Task::none()
            }

            Message::DismissOverwriteWarning => {
                self.overwrite_warning = None;
                self.pending_generation_action = None;
                Task::none()
            }

            // Collapsible section toggles
            Message::ToggleRootCA => {
                self.root_ca_collapsed = !self.root_ca_collapsed;
                Task::none()
            }

            Message::ToggleIntermediateCA => {
                self.intermediate_ca_collapsed = !self.intermediate_ca_collapsed;
                Task::none()
            }

            Message::ToggleServerCert => {
                self.server_cert_collapsed = !self.server_cert_collapsed;
                Task::none()
            }

            Message::ToggleYubiKeySection => {
                self.yubikey_section_collapsed = !self.yubikey_section_collapsed;
                Task::none()
            }

            Message::ToggleNatsSection => {
                self.nats_section_collapsed = !self.nats_section_collapsed;
                Task::none()
            }

            Message::ToggleCertificatesSection => {
                self.certificates_collapsed = !self.certificates_collapsed;
                Task::none()
            }

            Message::ToggleKeysSection => {
                self.keys_collapsed = !self.keys_collapsed;
                Task::none()
            }

            // Status messages
            Message::UpdateStatus(status) => {
                self.status_message = status;
                Task::none()
            }

            Message::ShowError(error) => {
                self.error_message = Some(error);
                Task::none()
            }

            Message::ClearError => {
                self.error_message = None;
                Task::none()
            }

            // Context-aware node creation (SPACE key)
            Message::CreateContextAwareNode => {
                use crate::gui::graph::NodeType;
                use crate::domain::{Organization, OrganizationUnit, OrganizationUnitType, Person, Location, LocationMarker, Address};
                use cim_domain::EntityId;
                use chrono::Utc;

                // Determine what type of node to create based on current graph state
                let has_org = self.org_graph.nodes.values().any(|n| matches!(n.node_type, NodeType::Organization(_)));
                let has_units = self.org_graph.nodes.values().any(|n| matches!(n.node_type, NodeType::OrganizationalUnit(_)));
                let has_people = self.org_graph.nodes.values().any(|n| matches!(n.node_type, NodeType::Person { .. }));

                if !has_org {
                    // No organization yet - create one
                    let org = Organization {
                        id: Uuid::now_v7(),
                        name: "New Organization".to_string(),
                        display_name: "New Organization".to_string(),
                        description: Some("Created with SPACE key".to_string()),
                        parent_id: None,
                        units: Vec::new(),
                        created_at: Utc::now(),
                        metadata: std::collections::HashMap::new(),
                    };
                    self.org_graph.add_organization_node(org);
                    self.status_message = "✨ Created new organization (SPACE)".to_string();
                } else if !has_units {
                    // Has org but no units - create a unit
                    let unit = OrganizationUnit {
                        id: Uuid::now_v7(),
                        name: "New Unit".to_string(),
                        unit_type: OrganizationUnitType::Department,
                        parent_unit_id: None,
                        responsible_person_id: None,
                    };
                    self.org_graph.add_org_unit_node(unit);
                    self.status_message = "✨ Created new organizational unit (SPACE)".to_string();
                } else if !has_people {
                    // Has org and units but no people - create a person
                    let org_id = self.org_graph.nodes.values()
                        .find_map(|n| if let NodeType::Organization(org) = &n.node_type { Some(org.id) } else { None })
                        .unwrap_or_else(Uuid::now_v7);

                    let person = Person {
                        id: Uuid::now_v7(),
                        name: "New Person".to_string(),
                        email: "person@example.com".to_string(),
                        roles: Vec::new(),
                        organization_id: org_id,
                        unit_ids: Vec::new(),
                        created_at: Utc::now(),
                        active: true,
                    };
                    self.org_graph.add_node(person, KeyOwnerRole::Developer);
                    self.status_message = "✨ Created new person (SPACE)".to_string();
                } else {
                    // Has everything - default to creating a location
                    let node_id = Uuid::now_v7();
                    let address = Address::new(
                        "123 Main St".to_string(),
                        "City".to_string(),
                        "State".to_string(),
                        "Country".to_string(),
                        "12345".to_string(),
                    );

                    let location = Location::new_physical(
                        EntityId::<LocationMarker>::from_uuid(node_id),
                        "New Location".to_string(),
                        address,
                    ).expect("Failed to create location");

                    self.org_graph.add_location_node(location);

                    // Also add to projection so it shows up in Locations tab
                    if let Some(org_id) = self.organization_id {
                        let projection = self.projection.clone();
                        let location_name = "New Location".to_string();
                        let location_type = "Physical".to_string();

                        self.status_message = "✨ Created new location (SPACE)".to_string();

                        return Task::perform(
                            async move {
                                let mut proj = projection.write().await;
                                proj.add_location(
                                    node_id,
                                    location_name.clone(),
                                    location_type,
                                    org_id,
                                    Some("123 Main St".to_string()),
                                    Some("City".to_string()),
                                    Some("State".to_string()),
                                    Some("Country".to_string()),
                                    Some("12345".to_string()),
                                )
                                .map(|_| format!("✨ Location added to both graph and list"))
                                .map_err(|e| format!("Location added to graph but failed to add to list: {}", e))
                            },
                            |result| match result {
                                Ok(msg) => Message::UpdateStatus(msg),
                                Err(e) => Message::ShowError(e),
                            }
                        );
                    } else {
                        self.status_message = "✨ Created new location (SPACE) - create organization to persist".to_string();
                    }
                }

                Task::none()
            }

            // Graph interactions
            Message::GraphMessage(graph_msg) => {
                match &graph_msg {
                    GraphMessage::NodeClicked(id) => {
                        self.status_message = format!("DEBUG: NodeClicked received for node {:?}", id);
                        // Check if we're in edge creation mode
                        if self.org_graph.edge_indicator.is_active() {
                            // Complete edge creation
                            if let Some(from_id) = self.org_graph.edge_indicator.from_node() {
                                if from_id != *id {  // Don't allow self-edges
                                    // Create edge event with default type
                                    use crate::gui::graph::EdgeType;
                                    use crate::gui::graph_events::GraphEvent;
                                    use chrono::Utc;

                                    let edge_type = EdgeType::Hierarchy;
                                    let color = self.view_model.colors.node_edge_highlight;

                                    let event = GraphEvent::EdgeCreated {
                                        from: from_id,
                                        to: *id,
                                        edge_type,
                                        color,
                                        timestamp: Utc::now(),
                                    };

                                    self.org_graph.event_stack.push(event.clone());
                                    self.org_graph.apply_event(&event);

                                    if let (Some(from_node), Some(to_node)) = (
                                        self.org_graph.nodes.get(&from_id),
                                        self.org_graph.nodes.get(id)
                                    ) {
                                        self.status_message = format!("Created edge from '{}' to '{}' (Total edges: {})",
                                            from_node.label, to_node.label, self.org_graph.edges.len());
                                    } else {
                                        self.status_message = format!("Edge created but nodes not found (Total edges: {})", self.org_graph.edges.len());
                                    }
                                } else {
                                    self.status_message = "Cannot create edge to same node".to_string();
                                }
                                self.org_graph.edge_indicator.complete();
                            }
                        } else {
                            // Normal node selection
                            self.selected_person = Some(*id);
                            self.org_graph.selected_node = Some(*id);  // Update graph's selected node
                            // Phase 4: Open property card when node is clicked
                            if let Some(node) = self.org_graph.nodes.get(id) {
                                self.property_card.set_node(*id, node.node_type.clone());
                                self.status_message = format!("Selected '{}' - property card: {}, selected_node: {:?}",
                                    node.label,
                                    if self.property_card.is_editing() { "open" } else { "closed" },
                                    self.org_graph.selected_node);
                            } else {
                                self.status_message = "Selected node in graph".to_string();
                            }
                        }
                    }
                    GraphMessage::NodeDragStarted { node_id, offset } => {
                        self.org_graph.dragging_node = Some(*node_id);
                        self.org_graph.drag_offset = *offset;
                        if let Some(node) = self.org_graph.nodes.get(node_id) {
                            self.org_graph.drag_start_position = Some(node.position);
                        }
                    }
                    GraphMessage::NodeDragged(cursor_position) => {
                        if let Some(node_id) = self.org_graph.dragging_node {
                            if let Some(node) = self.org_graph.nodes.get_mut(&node_id) {
                                // Update node position based on cursor and drag offset
                                let adjusted_x = (cursor_position.x - self.org_graph.pan_offset.x) / self.org_graph.zoom;
                                let adjusted_y = (cursor_position.y - self.org_graph.pan_offset.y) / self.org_graph.zoom;
                                node.position.x = adjusted_x - self.org_graph.drag_offset.x;
                                node.position.y = adjusted_y - self.org_graph.drag_offset.y;
                            }
                        }
                    }
                    GraphMessage::NodeDragEnded => {
                        if let Some(node_id) = self.org_graph.dragging_node {
                            // Check if node actually moved
                            if let (Some(start_pos), Some(node)) = (
                                self.org_graph.drag_start_position,
                                self.org_graph.nodes.get(&node_id)
                            ) {
                                let distance = ((node.position.x - start_pos.x).powi(2)
                                    + (node.position.y - start_pos.y).powi(2)).sqrt();

                                if distance > 5.0 {
                                    // Create NodeMoved event for undo/redo
                                    use crate::gui::graph_events::GraphEvent;
                                    use chrono::Utc;

                                    let event = GraphEvent::NodeMoved {
                                        node_id,
                                        old_position: start_pos,
                                        new_position: node.position,
                                        timestamp: Utc::now(),
                                    };
                                    self.org_graph.event_stack.push(event);
                                    self.status_message = format!("Moved node to ({:.0}, {:.0})",
                                        node.position.x, node.position.y);
                                }
                            }
                        }
                        self.org_graph.dragging_node = None;
                        self.org_graph.drag_start_position = None;
                    }
                    GraphMessage::AutoLayout => {
                        self.org_graph.auto_layout();
                        self.status_message = String::from("Graph layout updated");
                    }
                    GraphMessage::AddEdge { .. } => {
                        // Handled in graph.handle_message (line 1744)
                        self.status_message = String::from("Relationship added");
                    }
                    GraphMessage::EdgeSelected(index) => {
                        if let Some(edge) = self.org_graph.edges.get(*index) {
                            self.property_card.set_edge(*index, edge.from, edge.to, edge.edge_type.clone());
                            self.status_message = format!("Edge selected ({})", *index);
                        }
                    }
                    GraphMessage::EdgeDeleted(_index) => {
                        // Handled in graph.handle_message
                        self.property_card.clear();
                        self.status_message = String::from("Edge deleted");
                    }
                    GraphMessage::EdgeTypeChanged { .. } => {
                        // Handled in graph.handle_message
                        self.status_message = String::from("Edge type changed");
                    }
                    GraphMessage::EdgeCreationStarted(_node_id) => {
                        // Handled in graph.handle_message (starts edge indicator)
                        self.status_message = String::from("Drag to target node to create edge");
                    }
                    // Phase 4: Right-click shows context menu
                    GraphMessage::RightClick(position) => {
                        // Position is now canvas-relative, transform to graph coordinates for hit detection
                        let graph_x = (position.x - self.org_graph.pan_offset.x) / self.org_graph.zoom;
                        let graph_y = (position.y - self.org_graph.pan_offset.y) / self.org_graph.zoom;

                        // Check if we right-clicked on a node (using graph coords)
                        self.context_menu_node = None;
                        for (node_id, node) in &self.org_graph.nodes {
                            let dx = graph_x - node.position.x;
                            let dy = graph_y - node.position.y;
                            let distance = (dx * dx + dy * dy).sqrt();
                            if distance <= 25.0 {  // Within node radius
                                self.context_menu_node = Some(*node_id);
                                break;
                            }
                        }

                        // Adjust menu position to keep it on screen
                        // Menu dimensions scaled by ui_scale
                        let menu_width = 180.0 * self.view_model.scale;
                        let menu_height = 300.0 * self.view_model.scale;
                        const MIN_MARGIN: f32 = 10.0;

                        // Use typical window dimensions for bounds checking
                        // TODO: Track actual window size for precise bounds checking
                        const TYPICAL_WIDTH: f32 = 1920.0;
                        const TYPICAL_HEIGHT: f32 = 1080.0;

                        let mut menu_x = position.x;
                        let mut menu_y = position.y;

                        // Adjust horizontally if menu would go off right edge
                        if menu_x + menu_width + MIN_MARGIN > TYPICAL_WIDTH {
                            menu_x = TYPICAL_WIDTH - menu_width - MIN_MARGIN;
                        }
                        // Ensure not off left edge
                        if menu_x < MIN_MARGIN {
                            menu_x = MIN_MARGIN;
                        }

                        // Adjust vertically if menu would go off bottom edge
                        if menu_y + menu_height + MIN_MARGIN > TYPICAL_HEIGHT {
                            menu_y = TYPICAL_HEIGHT - menu_height - MIN_MARGIN;
                        }
                        // Ensure not off top edge
                        if menu_y < MIN_MARGIN {
                            menu_y = MIN_MARGIN;
                        }

                        // Use adjusted screen coordinates for menu positioning
                        self.context_menu.show(Point::new(menu_x, menu_y), self.view_model.scale);
                        if self.context_menu_node.is_some() {
                            self.status_message = format!(
                                "Node menu: click({:.0},{:.0}) → canvas({:.0},{:.0}) → menu({:.0},{:.0})",
                                position.x, position.y, graph_x, graph_y, menu_x, menu_y
                            );
                        } else {
                            self.status_message = format!(
                                "Canvas menu: click({:.0},{:.0}) → menu({:.0},{:.0})",
                                position.x, position.y, menu_x, menu_y
                            );
                        }
                    }
                    // Phase 4: Update edge indicator position during edge creation
                    GraphMessage::CursorMoved(position) => {
                        if self.org_graph.edge_indicator.is_active() {
                            self.org_graph.edge_indicator.update_position(*position);
                        }
                    }
                    // Phase 4: Cancel edge creation with Esc key
                    GraphMessage::CancelEdgeCreation => {
                        if self.org_graph.edge_indicator.is_active() {
                            self.status_message = "Edge creation cancelled".to_string();
                        }
                        // Cancellation handled in graph.handle_message
                    }
                    // Phase 4: Delete selected node with Delete key
                    GraphMessage::DeleteSelected => {
                        if let Some(node_id) = self.org_graph.selected_node {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                use crate::gui::graph_events::GraphEvent;
                                use chrono::Utc;

                                // Create NodeDeleted event with snapshot for redo
                                let event = GraphEvent::NodeDeleted {
                                    node_id,
                                    node_type: node.node_type.clone(),
                                    position: node.position,
                                    color: node.color,
                                    label: node.label.clone(),
                                    timestamp: Utc::now(),
                                };

                                let label = node.label.clone();
                                self.org_graph.event_stack.push(event.clone());
                                self.org_graph.apply_event(&event);

                                self.status_message = format!("Deleted '{}'", label);
                            } else {
                                self.status_message = "Node not found".to_string();
                            }
                        } else {
                            self.status_message = "No node selected to delete".to_string();
                        }
                    }
                    // Phase 4: Undo last action
                    GraphMessage::Undo => {
                        if let Some(description) = self.org_graph.event_stack.undo_description() {
                            self.status_message = format!("Undo: {}", description);
                        } else {
                            self.status_message = "Nothing to undo".to_string();
                        }
                        // Undo handled in graph.handle_message
                    }
                    // Phase 4: Redo last undone action
                    GraphMessage::Redo => {
                        if let Some(description) = self.org_graph.event_stack.redo_description() {
                            self.status_message = format!("Redo: {}", description);
                        } else {
                            self.status_message = "Nothing to redo".to_string();
                        }
                        // Redo handled in graph.handle_message
                    }
                    // Canvas clicked - place new node if node type is selected
                    GraphMessage::CanvasClicked(position) => {
                        if let Some(ref node_type_str) = self.selected_node_type {
                            use crate::domain::{OrganizationUnit, OrganizationUnitType, Person};
                            use crate::gui::graph::NodeType;
                            use crate::gui::graph_events::GraphEvent;
                            use chrono::Utc;
                            

                            let node_id = Uuid::now_v7();
                            let dummy_org_id = self.organization_id.unwrap_or_else(|| Uuid::now_v7());

                            // Create node based on selected type and current graph view
                            let (graph_node_type, label, color) = match node_type_str.as_str() {
                                // Organization graph nodes
                                "Person" => {
                                    let person = Person {
                                        id: node_id,
                                        name: "New Person".to_string(),
                                        email: format!("person{}@example.com", node_id),
                                        roles: vec![],
                                        organization_id: dummy_org_id,
                                        unit_ids: vec![],
                                        created_at: Utc::now(),
                                        active: true,
                                    };
                                    use crate::domain::KeyOwnerRole;
                                    (NodeType::Person { person, role: KeyOwnerRole::RootAuthority }, "New Person".to_string(), self.view_model.colors.node_person)
                                }
                                "Unit" => {
                                    let unit = OrganizationUnit {
                                        id: node_id,
                                        name: "New Unit".to_string(),
                                        unit_type: OrganizationUnitType::Department,
                                        parent_unit_id: None,
                                        responsible_person_id: None,
                                    };
                                    (NodeType::OrganizationalUnit(unit), "New Unit".to_string(), self.view_model.colors.node_unit)
                                }
                                "Location" => {
                                    use cim_domain::EntityId;
                                    use crate::domain::{Address, Location, LocationMarker};

                                    // Create a simple physical location with placeholder address
                                    let location = Location::new_physical(
                                        EntityId::<LocationMarker>::from_uuid(node_id),
                                        "New Location".to_string(),
                                        Address::new(
                                            "123 Main St".to_string(),
                                            "City".to_string(),
                                            "State".to_string(),
                                            "Country".to_string(),
                                            "12345".to_string(),
                                        ),
                                    ).expect("Failed to create location");
                                    (NodeType::Location(location), "New Location".to_string(), self.view_model.colors.node_location)
                                }
                                // TODO: Role - complex type, implement later
                                // TODO: Add NATS, PKI, and YubiKey node types
                                _ => {
                                    self.status_message = format!("Node type '{}' not yet implemented", node_type_str);
                                    self.selected_node_type = None;
                                    return Task::none();
                                }
                            };

                            // Clone node type for domain event projection before moving it
                            let node_type_for_domain = graph_node_type.clone();

                            // Create NodeCreated event
                            let event = GraphEvent::NodeCreated {
                                node_id,
                                node_type: graph_node_type,
                                position: *position,
                                color,
                                label: label.clone(),
                                timestamp: Utc::now(),
                            };

                            self.org_graph.event_stack.push(event.clone());
                            self.org_graph.apply_event(&event);

                            // Emit domain event and project to cim-graph (demonstration)
                            // In production, this would create a real PersonCreated domain event
                            // and persist it to NATS/IPLD via the GraphProjector
                            #[cfg(feature = "policy")]
                            {
                                use cim_domain_person::events::{PersonEvent, PersonCreated};
                                use cim_domain_person::value_objects::PersonName;
                                use cim_domain::EntityId;

                                if let NodeType::Person { person, .. } = &node_type_for_domain {
                                    // Create domain event (using new EntityId for demonstration)
                                    // In production, would properly convert person.id to EntityId
                                    let domain_event = PersonEvent::PersonCreated(PersonCreated {
                                        person_id: EntityId::new(),
                                        name: PersonName::new(person.name.clone(), "".to_string()),
                                        source: "gui".to_string(),
                                        created_at: Utc::now(),
                                    });

                                    // Lift to cim-graph events using GraphProjector
                                    match self.graph_projector.lift_person_event(&domain_event) {
                                        Ok(graph_events) => {
                                            // TODO: Persist graph_events to NATS/IPLD
                                            tracing::debug!("✨ Generated {} cim-graph events for PersonCreated", graph_events.len());
                                            for (i, evt) in graph_events.iter().enumerate() {
                                                tracing::debug!("  Event {}: {:?}", i+1, evt);
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to project PersonCreated event: {:?}", e);
                                        }
                                    }
                                }
                            }

                            // Start inline editing for the new node
                            self.editing_new_node = Some(node_id);
                            self.inline_edit_name = label.clone();
                            self.status_message = format!("Created '{}' - edit name and press Enter", label);
                            self.selected_node_type = None;  // Clear selection after placing
                        } else {
                            // No node type selected - just a normal canvas click
                            self.status_message = "Canvas clicked - select a node type from the dropdown to place a new node".to_string();
                        }
                    }
                    _ => {}
                }
                self.org_graph.handle_message(graph_msg);
                Task::none()
            }

            // Phase 4: Context Menu interactions
            Message::ContextMenuMessage(menu_msg) => {
                use crate::mvi::intent::NodeCreationType;
                use crate::domain::{Organization, OrganizationUnit, OrganizationUnitType, Location, Role, Policy};
                use crate::gui::graph::NodeType;
                use crate::gui::graph_events::GraphEvent;
                use chrono::Utc;
                use std::collections::HashMap;

                match menu_msg {
                    ContextMenuMessage::CreateNode(node_type) => {
                        let position = self.context_menu.position();
                        let node_id = Uuid::now_v7();
                        let dummy_org_id = self.organization_id.unwrap_or_else(|| Uuid::now_v7());

                        // Create placeholder domain entity and generate event
                        let (graph_node_type, label, color) = match node_type {
                            NodeCreationType::Organization => {
                                let org = Organization {
                                    id: node_id,
                                    name: "New Organization".to_string(),
                                    display_name: "New Organization".to_string(),
                                    description: Some("Edit this organization".to_string()),
                                    parent_id: None,
                                    units: vec![],
                                    created_at: Utc::now(),
                                    metadata: HashMap::new(),
                                };
                                let label = org.name.clone();
                                (NodeType::Organization(org), label, self.view_model.colors.node_organization)
                            }
                            NodeCreationType::OrganizationalUnit => {
                                let unit = OrganizationUnit {
                                    id: node_id,
                                    name: "New Unit".to_string(),
                                    unit_type: OrganizationUnitType::Department,
                                    parent_unit_id: None,
                                    responsible_person_id: None,
                                };
                                let label = unit.name.clone();
                                (NodeType::OrganizationalUnit(unit), label, self.view_model.colors.node_unit)
                            }
                            NodeCreationType::Person => {
                                let person = Person {
                                    id: node_id,
                                    name: "New Person".to_string(),
                                    email: "person@example.com".to_string(),
                                    roles: vec![],
                                    organization_id: dummy_org_id,
                                    unit_ids: vec![],
                                    created_at: Utc::now(),
                                    active: true,
                                };
                                let label = person.name.clone();
                                (NodeType::Person { person, role: KeyOwnerRole::Developer }, label, self.view_model.colors.node_person)
                            }
                            NodeCreationType::Location => {
                                use cim_domain::EntityId;
                                use crate::domain::{Address, LocationMarker};

                                // Create a placeholder physical location
                                // Click the node to edit details via property card
                                let address = Address::new(
                                    "123 Main St".to_string(),
                                    "City".to_string(),
                                    "State".to_string(),
                                    "Country".to_string(),
                                    "00000".to_string(),
                                );

                                let location = Location::new_physical(
                                    EntityId::<LocationMarker>::from_uuid(node_id),
                                    "New Location (Edit Me)".to_string(),
                                    address,
                                ).expect("Failed to create location");

                                let label = location.name.clone();
                                (NodeType::Location(location), label, self.view_model.colors.node_location)
                            }
                            NodeCreationType::Role => {
                                let role = Role {
                                    id: node_id,
                                    name: "New Role".to_string(),
                                    description: "Role description".to_string(),
                                    organization_id: dummy_org_id,
                                    unit_id: None,
                                    required_policies: vec![],
                                    responsibilities: vec![],
                                    created_at: Utc::now(),
                                    created_by: dummy_org_id,
                                    active: true,
                                };
                                let label = role.name.clone();
                                (NodeType::Role(role), label, self.view_model.colors.node_role)
                            }
                            NodeCreationType::Policy => {
                                let policy = Policy {
                                    id: node_id,
                                    name: "New Policy".to_string(),
                                    description: "Policy description".to_string(),
                                    claims: vec![],
                                    conditions: vec![],
                                    priority: 0,
                                    enabled: true,
                                    created_at: Utc::now(),
                                    created_by: dummy_org_id,
                                    metadata: HashMap::new(),
                                };
                                let label = policy.name.clone();
                                (NodeType::Policy(policy), label, self.view_model.colors.orange_warning)
                            }
                        };

                        // Create and apply NodeCreated event
                        let event = GraphEvent::NodeCreated {
                            node_id,
                            node_type: graph_node_type,
                            position,
                            color,
                            label,
                            timestamp: Utc::now(),
                        };

                        self.org_graph.event_stack.push(event.clone());
                        self.org_graph.apply_event(&event);

                        // Emit domain event and project to cim-graph (demonstration)
                        #[cfg(feature = "policy")]
                        {
                            use super::gui::graph_events::GraphEvent as GuiGraphEvent;
                            if let GuiGraphEvent::NodeCreated { node_type, .. } = &event {
                                match node_type {
                                    NodeType::Organization(org) => {
                                        use cim_domain_organization::events::{OrganizationEvent, OrganizationCreated};
                                        use cim_domain_organization::{OrganizationType, Organization as OrgMarker};
                                        use cim_domain::{EntityId, MessageIdentity, CorrelationId, CausationId};

                                        let event_id = Uuid::now_v7();
                                        let domain_event = OrganizationEvent::OrganizationCreated(OrganizationCreated {
                                            event_id,
                                            identity: MessageIdentity {
                                                correlation_id: CorrelationId::Single(event_id),
                                                causation_id: CausationId(event_id),
                                                message_id: event_id,
                                            },
                                            organization_id: EntityId::<OrgMarker>::new(),
                                            name: org.name.clone(),
                                            display_name: org.display_name.clone(),
                                            organization_type: OrganizationType::Corporation,
                                            parent_id: None,
                                            metadata: serde_json::json!({}),
                                            occurred_at: Utc::now(),
                                        });

                                        match self.graph_projector.lift_organization_event(&domain_event) {
                                            Ok(graph_events) => {
                                                tracing::debug!("✨ Generated {} cim-graph events for OrganizationCreated", graph_events.len());
                                                for (i, evt) in graph_events.iter().enumerate() {
                                                    tracing::debug!("  Event {}: {:?}", i+1, evt);
                                                }

                                                // Check if NATS publishing is enabled via configuration
                                                if let Some(ref cfg) = self.config {
                                                    if cfg.nats.enabled {
                                                        tracing::info!("📡 NATS publishing enabled - events would be published to {}", cfg.nats.url);
                                                        // TODO (v0.9.0): Publish to NATS here
                                                    } else {
                                                        tracing::debug!("📴 NATS disabled - events logged locally only");
                                                    }
                                                } else {
                                                    tracing::debug!("📴 No configuration loaded - events logged locally only");
                                                }
                                            }
                                            Err(e) => {
                                                tracing::warn!("Failed to project OrganizationCreated event: {:?}", e);
                                            }
                                        }
                                    }
                                    NodeType::Person { person, .. } => {
                                        use cim_domain_person::events::{PersonEvent, PersonCreated};
                                        use cim_domain_person::value_objects::PersonName;
                                        use cim_domain::EntityId;

                                        let domain_event = PersonEvent::PersonCreated(PersonCreated {
                                            person_id: EntityId::new(),
                                            name: PersonName::new(person.name.clone(), "".to_string()),
                                            source: "gui".to_string(),
                                            created_at: Utc::now(),
                                        });

                                        match self.graph_projector.lift_person_event(&domain_event) {
                                            Ok(graph_events) => {
                                                tracing::debug!("✨ Generated {} cim-graph events for PersonCreated", graph_events.len());
                                                for (i, evt) in graph_events.iter().enumerate() {
                                                    tracing::debug!("  Event {}: {:?}", i+1, evt);
                                                }

                                                // Check if NATS publishing is enabled via configuration
                                                if let Some(ref cfg) = self.config {
                                                    if cfg.nats.enabled {
                                                        tracing::info!("📡 NATS publishing enabled - events would be published to {}", cfg.nats.url);
                                                        // TODO (v0.9.0): Publish to NATS here
                                                    } else {
                                                        tracing::debug!("📴 NATS disabled - events logged locally only");
                                                    }
                                                } else {
                                                    tracing::debug!("📴 No configuration loaded - events logged locally only");
                                                }
                                            }
                                            Err(e) => {
                                                tracing::warn!("Failed to project PersonCreated event: {:?}", e);
                                            }
                                        }
                                    }
                                    _ => {} // Other node types don't have domain event demonstrations yet
                                }
                            }
                        }

                        // Auto-layout to position the new node and re-align hierarchy
                        self.org_graph.auto_layout();

                        // Open property card for the new node
                        if let Some(node) = self.org_graph.nodes.get(&node_id) {
                            self.property_card.set_node(node_id, node.node_type.clone());
                        }

                        self.context_menu.hide();
                        self.status_message = format!("Created {:?} node - edit properties", node_type);
                    }
                    ContextMenuMessage::CreateEdge => {
                        // Start edge creation from the node where context menu was opened
                        let source_node = self.context_menu_node.or(self.selected_person);

                        if let Some(from_id) = source_node {
                            if let Some(from_node) = self.org_graph.nodes.get(&from_id) {
                                // Start edge creation indicator
                                self.org_graph.edge_indicator.start(from_id, from_node.position);
                                self.status_message = format!("Edge creation mode active - click target node (from: '{}')", from_node.label);
                            } else {
                                self.status_message = "Error: Source node not found in graph".to_string();
                            }
                        } else {
                            self.status_message = "Please right-click on a node first to select edge source".to_string();
                        }

                        self.context_menu.hide();
                    }
                    ContextMenuMessage::Dismiss => {
                        self.context_menu.hide();
                    }
                }
                Task::none()
            }

            // Phase 4: Property Card interactions
            Message::PropertyCardMessage(card_msg) => {
                self.property_card.update(card_msg.clone());
                match card_msg {
                    PropertyCardMessage::Save => {
                        // Handle node or edge save
                        if let Some(edge_index) = self.property_card.edge_index() {
                            // Saving edge changes (edge type)
                            if edge_index < self.org_graph.edges.len() {
                                let new_edge_type = self.property_card.edge_type();
                                // Send EdgeTypeChanged message to graph
                                self.org_graph.handle_message(GraphMessage::EdgeTypeChanged {
                                    edge_index,
                                    new_type: new_edge_type,
                                });
                                self.status_message = "Edge type saved".to_string();
                            }
                            self.property_card.clear();
                        } else if let Some(node_id) = self.property_card.node_id() {
                            // Saving node changes
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                use crate::gui::graph_events::GraphEvent;
                                use chrono::Utc;

                                let new_name = self.property_card.name().to_string();
                                let new_description = self.property_card.description().to_string();
                                let new_email = self.property_card.email().to_string();
                                let new_enabled = self.property_card.enabled();

                                // Capture old state
                                let old_node_type = node.node_type.clone();
                                let old_label = node.label.clone();

                                // Create updated node type with new values
                                let new_node_type = match &node.node_type {
                                    graph::NodeType::Organization(org) => {
                                        let mut updated = org.clone();
                                        updated.name = new_name.clone();
                                        updated.display_name = new_name.clone();
                                        updated.description = Some(new_description);
                                        graph::NodeType::Organization(updated)
                                    }
                                    graph::NodeType::OrganizationalUnit(unit) => {
                                        let mut updated = unit.clone();
                                        updated.name = new_name.clone();
                                        graph::NodeType::OrganizationalUnit(updated)
                                    }
                                    graph::NodeType::Person { person, role } => {
                                        let mut updated = person.clone();
                                        updated.name = new_name.clone();
                                        updated.email = new_email;
                                        updated.active = new_enabled;
                                        graph::NodeType::Person { person: updated, role: *role }
                                    }
                                    graph::NodeType::Location(location) => {
                                        let mut updated = location.clone();
                                        updated.name = new_name.clone();
                                        graph::NodeType::Location(updated)
                                    }
                                    graph::NodeType::Role(role) => {
                                        let mut updated = role.clone();
                                        updated.name = new_name.clone();
                                        updated.description = new_description;
                                        updated.active = new_enabled;
                                        graph::NodeType::Role(updated)
                                    }
                                    graph::NodeType::Policy(policy) => {
                                        let mut updated = policy.clone();
                                        updated.name = new_name.clone();
                                        updated.description = new_description;
                                        updated.enabled = new_enabled;
                                        updated.claims = self.property_card.claims();
                                        graph::NodeType::Policy(updated)
                                    }
                                    // NATS Infrastructure - read-only, return unchanged
                                    graph::NodeType::NatsOperator(identity) => {
                                        graph::NodeType::NatsOperator(identity.clone())
                                    }
                                    graph::NodeType::NatsAccount(identity) => {
                                        graph::NodeType::NatsAccount(identity.clone())
                                    }
                                    graph::NodeType::NatsUser(identity) => {
                                        graph::NodeType::NatsUser(identity.clone())
                                    }
                                    graph::NodeType::NatsServiceAccount(identity) => {
                                        graph::NodeType::NatsServiceAccount(identity.clone())
                                    }
                                    // PKI Trust Chain - read-only, return unchanged
                                    graph::NodeType::RootCertificate { cert_id, subject, issuer, not_before, not_after, key_usage } => {
                                        graph::NodeType::RootCertificate {
                                            cert_id: *cert_id,
                                            subject: subject.clone(),
                                            issuer: issuer.clone(),
                                            not_before: *not_before,
                                            not_after: *not_after,
                                            key_usage: key_usage.clone(),
                                        }
                                    }
                                    graph::NodeType::IntermediateCertificate { cert_id, subject, issuer, not_before, not_after, key_usage } => {
                                        graph::NodeType::IntermediateCertificate {
                                            cert_id: *cert_id,
                                            subject: subject.clone(),
                                            issuer: issuer.clone(),
                                            not_before: *not_before,
                                            not_after: *not_after,
                                            key_usage: key_usage.clone(),
                                        }
                                    }
                                    graph::NodeType::LeafCertificate { cert_id, subject, issuer, not_before, not_after, key_usage, san } => {
                                        graph::NodeType::LeafCertificate {
                                            cert_id: *cert_id,
                                            subject: subject.clone(),
                                            issuer: issuer.clone(),
                                            not_before: *not_before,
                                            not_after: *not_after,
                                            key_usage: key_usage.clone(),
                                            san: san.clone(),
                                        }
                                    }
                                    // YubiKey Hardware - read-only, return unchanged
                                    graph::NodeType::YubiKey { device_id, serial, version, provisioned_at, slots_used } => {
                                        graph::NodeType::YubiKey {
                                            device_id: *device_id,
                                            serial: serial.clone(),
                                            version: version.clone(),
                                            provisioned_at: *provisioned_at,
                                            slots_used: slots_used.clone(),
                                        }
                                    }
                                    graph::NodeType::PivSlot { slot_id, slot_name, yubikey_serial, has_key, certificate_subject } => {
                                        graph::NodeType::PivSlot {
                                            slot_id: *slot_id,
                                            slot_name: slot_name.clone(),
                                            yubikey_serial: yubikey_serial.clone(),
                                            has_key: *has_key,
                                            certificate_subject: certificate_subject.clone(),
                                        }
                                    }
                                    graph::NodeType::YubiKeyStatus { person_id, yubikey_serial, slots_provisioned, slots_needed } => {
                                        graph::NodeType::YubiKeyStatus {
                                            person_id: *person_id,
                                            yubikey_serial: yubikey_serial.clone(),
                                            slots_provisioned: slots_provisioned.clone(),
                                            slots_needed: slots_needed.clone(),
                                        }
                                    }
                                };

                                // Create and apply NodePropertiesChanged event
                                let event = GraphEvent::NodePropertiesChanged {
                                    node_id,
                                    old_node_type,
                                    old_label,
                                    new_node_type,
                                    new_label: new_name.clone(),
                                    timestamp: Utc::now(),
                                };

                                self.org_graph.event_stack.push(event.clone());
                                self.org_graph.apply_event(&event);

                                self.status_message = format!("Saved changes to '{}'", new_name);
                            }
                        }
                        self.property_card.clear();
                    }
                    PropertyCardMessage::Cancel => {
                        self.property_card.clear();
                        self.status_message = "Property changes cancelled".to_string();
                    }
                    PropertyCardMessage::Close => {
                        self.property_card.clear();
                    }
                    PropertyCardMessage::DeleteEdge => {
                        if let Some(edge_index) = self.property_card.edge_index() {
                            // Send EdgeDeleted message to graph
                            self.org_graph.handle_message(GraphMessage::EdgeDeleted(edge_index));
                            self.property_card.clear();
                            self.status_message = "Edge deleted".to_string();
                        }
                    }
                    PropertyCardMessage::GenerateRootCA => {
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                if let graph::NodeType::Person { person, .. } = &node.node_type {
                                    // Show passphrase dialog for Root CA generation
                                    self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::RootCA);
                                    self.status_message = format!("Enter passphrase to generate Root CA for {}", person.name);
                                }
                            }
                        }
                    }
                    PropertyCardMessage::GeneratePersonalKeys => {
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                if let graph::NodeType::Person { person, .. } = &node.node_type {
                                    self.status_message = format!("Generating personal keys for {}... (Not yet implemented)", person.name);
                                    // TODO: Implement personal key generation
                                    // This would involve:
                                    // 1. Generating SSH key pair
                                    // 2. Generating GPG key pair
                                    // 3. Creating certificate signing request
                                    // 4. Storing in encrypted projection
                                }
                            }
                        }
                    }
                    PropertyCardMessage::ProvisionYubiKey => {
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                if let graph::NodeType::Person { person, .. } = &node.node_type {
                                    self.status_message = format!("Provisioning YubiKey for {}... (Not yet implemented)", person.name);
                                    // TODO: Implement YubiKey provisioning
                                    // This would involve:
                                    // 1. Detecting connected YubiKey
                                    // 2. Generating keys on YubiKey PIV slots
                                    // 3. Storing slot assignments
                                    // 4. Creating edge in graph from person to YubiKey node
                                }
                            }
                        }
                    }
                    _ => {}
                }
                Task::none()
            }

            Message::PassphraseDialogMessage(dialog_msg) => {
                use passphrase_dialog::PassphraseDialogMessage;
                match dialog_msg {
                    PassphraseDialogMessage::Submit => {
                        // User clicked OK with valid passphrase
                        if let Some(passphrase) = self.passphrase_dialog.get_passphrase() {
                            // TODO: Call actual crypto function with passphrase
                            // For now, just show success message
                            self.status_message = "Root CA generation in progress...".to_string();
                            self.passphrase_dialog.hide();
                            // TODO: Trigger actual Root CA generation task
                        }
                    }
                    PassphraseDialogMessage::Cancel => {
                        // User clicked Cancel
                        self.passphrase_dialog.hide();
                        self.status_message = "Root CA generation cancelled".to_string();
                    }
                    _ => {
                        // Pass other messages to the dialog for internal state updates
                        self.passphrase_dialog.update(dialog_msg);
                    }
                }
                Task::none()
            }

            // Messages we haven't implemented yet
            Message::DomainCreated(result) => {
                match result {
                    Ok(domain) => {
                        self.domain_loaded = true;
                        self.status_message = format!("Domain created: {}", domain);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to create domain: {}", e));
                    }
                }
                Task::none()
            }

            Message::YubiKeyProvisioned(result) => {
                match result {
                    Ok(serial) => {
                        self.status_message = format!("YubiKey {} provisioned", serial);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("YubiKey provisioning failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::KeyGenerationProgress(progress) => {
                self.key_generation_progress = progress;
                Task::none()
            }

            // MVI Integration Handler
            Message::MviIntent(intent) => {
                // Check for completion events and update status
                use crate::mvi::intent::Intent;
                match &intent {
                    Intent::PortX509IntermediateCAGenerated { name, fingerprint, .. } => {
                        self.status_message = format!("[OK] Intermediate CA '{}' generated successfully! Fingerprint: {}", name, fingerprint);
                        self.key_generation_progress = 0.4;
                    }
                    Intent::PortX509ServerCertGenerated { common_name, fingerprint, signed_by, .. } => {
                        self.status_message = format!("[OK] Server certificate for '{}' generated! Signed by: {} | Fingerprint: {}", common_name, signed_by, fingerprint);
                        self.key_generation_progress = 0.6;
                    }
                    Intent::PortX509GenerationFailed { error } => {
                        self.error_message = Some(format!("Certificate generation failed: {}", error));
                    }
                    Intent::PortSSHKeypairGenerated { person_id, fingerprint, .. } => {
                        self.status_message = format!("[OK] SSH key for person {} generated! Fingerprint: {}", person_id, fingerprint);
                    }
                    Intent::PortSSHGenerationFailed { person_id, error } => {
                        self.error_message = Some(format!("SSH key generation failed for {}: {}", person_id, error));
                    }
                    _ => {}
                }

                // Call the pure MVI update function
                let (updated_model, task) = crate::mvi::update(
                    self.mvi_model.clone(),
                    intent,
                    self.storage_port.clone(),
                    self.x509_port.clone(),
                    self.ssh_port.clone(),
                    self.yubikey_port.clone(),
                );

                // Update our MVI model
                self.mvi_model = updated_model;

                // Map the Intent task back to Message task
                task.map(Message::MviIntent)
            }

            Message::AnimationTick => {
                // Update animation time
                self.animation_time += 0.016; // ~60fps
                // Update Kuramoto synchronization
                self.firefly_shader.update(0.016);  // Update phases for synchronization
                Task::none()
            }

            Message::IncreaseScale => {
                let new_scale = (self.view_model.scale + 0.1).min(3.0);  // Max 3x scale
                self.view_model.set_scale(new_scale);
                self.status_message = format!("UI Scale: {:.0}%", new_scale * 100.0);
                Task::none()
            }

            Message::DecreaseScale => {
                let new_scale = (self.view_model.scale - 0.1).max(0.5);  // Min 0.5x scale
                self.view_model.set_scale(new_scale);
                self.status_message = format!("UI Scale: {:.0}%", new_scale * 100.0);
                Task::none()
            }

            Message::ResetScale => {
                self.view_model.set_scale(1.0);  // Reset to default 1x scale
                self.status_message = "UI Scale: 100%".to_string();
                Task::none()
            }

            // Search and filtering
            Message::SearchQueryChanged(query) => {
                self.search_query = query.clone();

                // Clear previous results
                self.search_results.clear();
                self.highlight_nodes.clear();

                // If query is empty, don't search
                if query.trim().is_empty() {
                    self.status_message = "Search cleared".to_string();
                    return Task::none();
                }

                // Search through all nodes in the graph
                let query_lower = query.to_lowercase();
                for (node_id, node) in &self.org_graph.nodes {
                    let matches = match &node.node_type {
                        graph::NodeType::Person { person, .. } => {
                            person.name.to_lowercase().contains(&query_lower) ||
                            person.email.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::Organization(org) => {
                            org.name.to_lowercase().contains(&query_lower) ||
                            org.display_name.to_lowercase().contains(&query_lower) ||
                            org.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
                        }
                        graph::NodeType::OrganizationalUnit(unit) => {
                            unit.name.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::Location(location) => {
                            location.name.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::Role(role) => {
                            role.name.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::Policy(policy) => {
                            policy.name.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::NatsOperator(_) => {
                            "nats operator".contains(&query_lower) || "operator".contains(&query_lower)
                        }
                        graph::NodeType::NatsAccount(_) => {
                            "nats account".contains(&query_lower) || "account".contains(&query_lower)
                        }
                        graph::NodeType::NatsUser(_) => {
                            "nats user".contains(&query_lower) || "user".contains(&query_lower)
                        }
                        graph::NodeType::NatsServiceAccount(_) => {
                            "service account".contains(&query_lower) || "service".contains(&query_lower)
                        }
                        graph::NodeType::RootCertificate { subject, .. } => {
                            "root".contains(&query_lower) || "certificate".contains(&query_lower) ||
                            subject.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::IntermediateCertificate { subject, .. } => {
                            "intermediate".contains(&query_lower) || "certificate".contains(&query_lower) ||
                            subject.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::LeafCertificate { subject, .. } => {
                            "leaf".contains(&query_lower) || "certificate".contains(&query_lower) ||
                            subject.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::YubiKey { serial, .. } => {
                            "yubikey".contains(&query_lower) || serial.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::PivSlot { slot_name, .. } => {
                            "piv".contains(&query_lower) || "slot".contains(&query_lower) ||
                            slot_name.to_lowercase().contains(&query_lower)
                        }
                        graph::NodeType::YubiKeyStatus { yubikey_serial, .. } => {
                            "yubikey".contains(&query_lower) || "status".contains(&query_lower) ||
                            yubikey_serial.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
                        }
                    };

                    if matches {
                        self.search_results.push(*node_id);
                        self.highlight_nodes.push(*node_id);
                    }
                }

                let result_count = self.search_results.len();
                self.status_message = if result_count == 0 {
                    format!("No results found for '{}'", query)
                } else if result_count == 1 {
                    "Found 1 matching node".to_string()
                } else {
                    format!("Found {} matching nodes", result_count)
                };

                Task::none()
            }

            Message::ClearSearch => {
                self.search_query.clear();
                self.search_results.clear();
                self.highlight_nodes.clear();
                self.status_message = "Search cleared".to_string();
                Task::none()
            }

            Message::HighlightSearchResults => {
                // This message can be used to re-apply highlighting or cycle through results
                if !self.search_results.is_empty() {
                    self.status_message = format!("Highlighting {} search results", self.search_results.len());
                }
                Task::none()
            }

            // Graph export/import
            Message::ExportGraph => {
                // Set loading state
                self.loading_export = true;
                self.status_message = "Exporting graph...".to_string();

                // Create export data from current graph state
                let export_data = GraphExport {
                    version: "1.0".to_string(),
                    exported_at: chrono::Utc::now().to_rfc3339(),
                    graph_view: format!("{:?}", self.graph_view),
                    nodes: self.org_graph.nodes.iter().map(|(id, node)| {
                        GraphNodeExport {
                            id: *id,
                            node_type: format!("{:?}", node.node_type).split('(').next().unwrap_or("Unknown").to_string(),
                            position_x: node.position.x,
                            position_y: node.position.y,
                            label: node.label.clone(),
                            color_r: node.color.r,
                            color_g: node.color.g,
                            color_b: node.color.b,
                        }
                    }).collect(),
                    edges: self.org_graph.edges.iter().map(|edge| {
                        GraphEdgeExport {
                            from_id: edge.from,
                            to_id: edge.to,
                            edge_type: format!("{:?}", edge.edge_type),
                        }
                    }).collect(),
                };

                // Serialize to JSON
                let output_dir = self._domain_path.clone();
                Task::perform(
                    async move {
                        match serde_json::to_string_pretty(&export_data) {
                            Ok(json) => {
                                let export_path = output_dir.join("graph_export.json");
                                match tokio::fs::write(&export_path, json).await {
                                    Ok(_) => Ok(format!("Graph exported to: {}", export_path.display())),
                                    Err(e) => Err(format!("Failed to write export file: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Failed to serialize graph: {}", e)),
                        }
                    },
                    Message::GraphExported
                )
            }

            Message::GraphExported(result) => {
                // Clear loading state
                self.loading_export = false;

                match result {
                    Ok(message) => {
                        self.status_message = message;
                        self.error_message = None;
                    }
                    Err(error) => {
                        self.error_message = Some(error.clone());
                        self.status_message = "Graph export failed".to_string();
                    }
                }
                Task::none()
            }

            Message::ImportGraph => {
                // Set loading state
                self.loading_import = true;
                self.status_message = "Importing graph...".to_string();
                let output_dir = self._domain_path.clone();
                Task::perform(
                    async move {
                        let import_path = output_dir.join("graph_export.json");
                        match tokio::fs::read_to_string(&import_path).await {
                            Ok(json) => {
                                match serde_json::from_str::<GraphExport>(&json) {
                                    Ok(data) => {
                                        // Return the parsed data for restoration
                                        Ok(Some(data))
                                    }
                                    Err(e) => Err(format!("Failed to parse import file: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Failed to read import file: {}", e)),
                        }
                    },
                    Message::GraphImported
                )
            }

            Message::GraphImported(result) => {
                // Clear loading state
                self.loading_import = false;

                match result {
                    Ok(Some(graph_data)) => {
                        // Restore node positions from imported data
                        let mut restored_count = 0;
                        for node_export in &graph_data.nodes {
                            if let Some(node) = self.org_graph.nodes.get_mut(&node_export.id) {
                                node.position = Point::new(node_export.position_x, node_export.position_y);
                                restored_count += 1;
                            }
                        }
                        self.status_message = format!(
                            "Graph imported: restored positions for {} nodes (exported at: {})",
                            restored_count,
                            graph_data.exported_at
                        );
                        self.error_message = None;
                    }
                    Ok(None) => {
                        self.status_message = "Graph import completed (no data)".to_string();
                        self.error_message = None;
                    }
                    Err(error) => {
                        self.error_message = Some(error.clone());
                        self.status_message = "Graph import failed".to_string();
                    }
                }
                Task::none()
            }

            // Help and tooltips
            Message::ToggleHelp => {
                self.show_help_overlay = !self.show_help_overlay;
                self.status_message = if self.show_help_overlay {
                    "Help overlay shown - Press F1 or Escape to close".to_string()
                } else {
                    "Help overlay hidden".to_string()
                };
                Task::none()
            }

            // Node/edge type filtering
            Message::ToggleFilterPeople => {
                self.filter_show_people = !self.filter_show_people;
                self.org_graph.filter_show_people = self.filter_show_people;
                self.status_message = format!("People nodes {}", if self.filter_show_people { "shown" } else { "hidden" });
                Task::none()
            }
            Message::ToggleFilterOrgs => {
                self.filter_show_orgs = !self.filter_show_orgs;
                self.org_graph.filter_show_orgs = self.filter_show_orgs;
                self.status_message = format!("Organization nodes {}", if self.filter_show_orgs { "shown" } else { "hidden" });
                Task::none()
            }
            Message::ToggleFilterNats => {
                self.filter_show_nats = !self.filter_show_nats;
                self.org_graph.filter_show_nats = self.filter_show_nats;
                self.status_message = format!("NATS nodes {}", if self.filter_show_nats { "shown" } else { "hidden" });
                Task::none()
            }
            Message::ToggleFilterPki => {
                self.filter_show_pki = !self.filter_show_pki;
                self.org_graph.filter_show_pki = self.filter_show_pki;
                self.status_message = format!("PKI nodes {}", if self.filter_show_pki { "shown" } else { "hidden" });
                Task::none()
            }
            Message::ToggleFilterYubiKey => {
                self.filter_show_yubikey = !self.filter_show_yubikey;
                self.org_graph.filter_show_yubikey = self.filter_show_yubikey;
                self.status_message = format!("YubiKey nodes {}", if self.filter_show_yubikey { "shown" } else { "hidden" });
                Task::none()
            }

            // Graph layout options
            Message::ChangeLayout(layout) => {
                self.current_layout = layout;
                let layout_name = match layout {
                    GraphLayout::Manual => "Manual",
                    GraphLayout::Hierarchical => "Hierarchical",
                    GraphLayout::ForceDirected => "Force-Directed",
                    GraphLayout::Circular => "Circular",
                };
                self.status_message = format!("Layout changed to {}", layout_name);
                Task::none()
            }
            Message::ApplyLayout => {
                self.status_message = "Applying layout...".to_string();
                // Apply the selected layout algorithm
                self.org_graph.apply_layout(self.current_layout);
                self.status_message = "Layout applied successfully".to_string();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::{stack, shader};

        // Cowboy AI logo with glass morphism styling
        let logo_container = container(
            image("assets/logo.png")
                .width(Length::Fixed(80.0 * self.view_model.scale))
                .height(Length::Fixed(80.0 * self.view_model.scale))
        )
        .width(Length::Fixed(100.0 * self.view_model.scale))
        .height(Length::Fixed(100.0 * self.view_model.scale))
        .padding(self.view_model.padding_md)
        .center(Length::Fixed(100.0 * self.view_model.scale))
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(CowboyTheme::logo_radial_gradient()),
                border: Border {
                    color: self.view_model.colors.with_alpha(self.view_model.colors.info, 0.5),  // Teal border
                    width: 2.0,
                    radius: 12.0.into(),
                },
                text_color: Some(CowboyTheme::text_primary()),
                shadow: Shadow {
                    color: self.view_model.colors.shadow_default,
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
            }
        });

        // Tab bar - Graph is the primary interface
        let tab_bar = row![
            button(text("Welcome").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::Welcome))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Welcome)),
            button(text("Organization Graph").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::Organization))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Organization)),
            button(text("Export").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::Export))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Export)),
        ]
        .spacing(self.view_model.spacing_md);

        // Tab content
        let content = match self.active_tab {
            Tab::Welcome => self.view_welcome(),
            Tab::Organization => self.view_organization(),
            Tab::Export => self.view_export(),
        };

        // Error display
        let error_display = self.error_message.as_ref().map(|error| container(
                    row![
                        text(format!("❌ {}", error))
                            .size(self.view_model.text_normal)
                            .color(CowboyTheme::text_primary()),
                        horizontal_space(),
                        button("✕")
                            .on_press(Message::ClearError)
                            .style(CowboyCustomTheme::glass_button())
                    ]
                    .padding(self.view_model.padding_md)
                )
                .style(|_theme: &Theme| container::Style {
                    background: Some(CowboyTheme::warning_gradient()),
                    text_color: Some(CowboyTheme::text_primary()),
                    border: Border {
                        color: self.view_model.colors.with_alpha(self.view_model.colors.text_light, 0.3),
                        width: 1.0,
                        radius: 10.0.into(),
                    },
                    shadow: CowboyTheme::glow_shadow(),
                }));

        // Header with logo and title
        let header = row![
            logo_container,
            column![
                text("CIM Keys - Offline Key Management System")
                    .size(self.view_model.text_header)
                    .color(CowboyTheme::text_primary()),
                text("The Cowboy AI Infrastructure")
                    .size(self.view_model.text_normal)
                    .color(CowboyTheme::text_secondary()),
            ]
            .spacing(self.view_model.spacing_sm),
        ]
        .spacing(self.view_model.spacing_xl)
        .align_y(iced::Alignment::Center);

        let mut main_column = column![
            header,
            text(&self.status_message)
                .size(self.view_model.text_small)
                .color(self.view_model.colors.text_primary),  // Light gray for visibility on dark background
            container(tab_bar)
                .padding(self.view_model.padding_md)
                .style(CowboyCustomTheme::glass_container()),
        ]
        .spacing(self.view_model.spacing_md);

        if let Some(error) = error_display {
            main_column = main_column.push(error);
        }

        // Add overwrite warning dialog if present
        if let Some(ref warning_msg) = self.overwrite_warning {
            let warning_dialog = container(
                column![
                    text(warning_msg)
                        .size(self.view_model.text_normal)
                        .color(self.view_model.colors.warning),  // Yellow warning color
                    row![
                        button("Cancel")
                            .on_press(Message::ConfirmOverwrite(false))
                            .style(CowboyCustomTheme::glass_button())
                            .padding(self.view_model.padding_md),
                        button("Proceed & Overwrite")
                            .on_press(Message::ConfirmOverwrite(true))
                            .style(CowboyCustomTheme::security_button())
                            .padding(self.view_model.padding_md),
                    ]
                    .spacing(self.view_model.spacing_md)
                ]
                .spacing(self.view_model.spacing_lg)
            )
            .padding(self.view_model.padding_xl)
            .style(|_theme| container::Style {
                text_color: None,
                background: Some(Background::Color(self.view_model.colors.orange_warning)),
                border: Border {
                    color: self.view_model.colors.yellow_warning,
                    width: 2.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: self.view_model.colors.shadow_yellow,
                    offset: iced::Vector::new(0.0, 0.0),
                    blur_radius: 16.0,
                },
            });

            main_column = main_column.push(warning_dialog);
        }

        main_column = main_column.push(content);

        // Layer the animated background behind the main content
        let main_content = container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        // Stack the background gradient, firefly shader, and main content
        let base_view = stack![
            // Background gradient matching www-egui
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(CowboyTheme::dark_background()),
                    ..Default::default()
                }),
            // Firefly shader on top of background
            shader(self.firefly_shader.clone())
                .width(Length::Fill)
                .height(Length::Fill),
            // Main content on top
            main_content
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // Add passphrase dialog overlay if visible
        if self.passphrase_dialog.is_visible() {
            stack![
                base_view,
                self.passphrase_dialog.view().map(Message::PassphraseDialogMessage)
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            base_view.into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::{time, keyboard, event};
        use iced::keyboard::Key;
        use std::time::Duration;

        Subscription::batch([
            // Update animation at 30 FPS instead of 60 to reduce resource usage
            time::every(Duration::from_millis(33)).map(|_| Message::AnimationTick),

            // Keyboard shortcuts for UI scaling
            event::listen_with(|event, _status, _window| {
                match event {
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        ..
                    }) => {
                        if modifiers.control() || modifiers.command() {
                            match key {
                                Key::Character(c) if c == "=" || c == "+" => {
                                    Some(Message::IncreaseScale)
                                }
                                Key::Character(c) if c == "-" || c == "_" => {
                                    Some(Message::DecreaseScale)
                                }
                                Key::Character(c) if c == "0" => {
                                    Some(Message::ResetScale)
                                }
                                // Phase 4: Undo/Redo shortcuts
                                Key::Character(c) if c == "z" && !modifiers.shift() => {
                                    Some(Message::GraphMessage(
                                        crate::gui::graph::GraphMessage::Undo
                                    ))
                                }
                                Key::Character(c) if c == "z" && modifiers.shift() => {
                                    Some(Message::GraphMessage(
                                        crate::gui::graph::GraphMessage::Redo
                                    ))
                                }
                                Key::Character(c) if c == "y" => {
                                    Some(Message::GraphMessage(
                                        crate::gui::graph::GraphMessage::Redo
                                    ))
                                }
                                // Phase 5: Graph export/import shortcuts
                                Key::Character(c) if c == "s" => {
                                    Some(Message::ExportGraph)
                                }
                                Key::Character(c) if c == "o" => {
                                    Some(Message::ImportGraph)
                                }
                                // Search shortcut
                                Key::Character(c) if c == "f" => {
                                    // Note: In a real implementation, this would focus the search input
                                    // For now, it just shows a status message
                                    Some(Message::UpdateStatus("Use search bar above to search nodes".to_string()))
                                }
                                // Clear search shortcut
                                Key::Named(keyboard::key::Named::Escape) if modifiers.control() => {
                                    Some(Message::ClearSearch)
                                }
                                _ => None,
                            }
                        } else {
                            // Phase 4: Keyboard shortcuts without modifiers
                            match key {
                                Key::Named(keyboard::key::Named::F1) => {
                                    Some(Message::ToggleHelp)
                                }
                                Key::Named(keyboard::key::Named::Escape) => {
                                    // Send InlineEditCancel - it will handle both cases
                                    Some(Message::InlineEditCancel)
                                }
                                Key::Named(keyboard::key::Named::Delete) => {
                                    Some(Message::GraphMessage(
                                        crate::gui::graph::GraphMessage::DeleteSelected
                                    ))
                                }
                                Key::Named(keyboard::key::Named::Space) => {
                                    // SPACE: Create context-aware node
                                    Some(Message::CreateContextAwareNode)
                                }
                                _ => None,
                            }
                        }
                    }
                    _ => None,
                }
            }),
        ])
    }
}

impl CimKeysApp {
    /// Returns the Cowboy AI theme for the application
    fn theme(&self) -> Theme {
        CowboyCustomTheme::dark()
    }

    fn tab_button_style(&self, _theme: &Theme, is_active: bool) -> button::Style {
        if is_active {
            button::Style {
                background: Some(CowboyTheme::primary_gradient()),
                text_color: CowboyTheme::text_primary(),
                border: Border {
                    color: CowboyTheme::border_hover_color(),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                shadow: CowboyTheme::glow_shadow(),
            }
        } else {
            button::Style {
                background: Some(CowboyTheme::glass_background()),
                text_color: CowboyTheme::text_secondary(),
                border: Border {
                    color: CowboyTheme::border_color(),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        }
    }

    fn view_welcome(&self) -> Element<'_, Message> {
        let content = column![
            text("Welcome to CIM Keys!").size(self.view_model.text_header),
            text("Generate and manage cryptographic keys for your CIM infrastructure").size(self.view_model.text_medium),
            container(
                column![
                    row![
                        icons::icon_sized(ICON_WARNING, self.view_model.text_large),
                        text(" Security Notice").size(self.view_model.text_large),
                    ].spacing(self.view_model.spacing_sm),
                    text("This application should be run on an air-gapped computer for maximum security.").size(self.view_model.text_normal),
                    text("All keys are generated offline and stored on encrypted SD cards.").size(self.view_model.text_normal),
                ]
                .spacing(self.view_model.spacing_sm)
            )
            .style(CowboyCustomTheme::pastel_coral_card())
            .padding(self.view_model.padding_xl),

            container(
                column![
                    text("Getting Started").size(self.view_model.text_xlarge),
                    text("Choose how you want to proceed:").size(self.view_model.text_normal),

                    row![
                        button("Import from Secrets")
                            .on_press(Message::ImportFromSecrets)
                            .style(CowboyCustomTheme::security_button()),
                        text("Load cowboyai.json and secrets.json files")
                            .size(self.view_model.text_small)
                            .color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(self.view_model.spacing_md)
                    .align_y(Alignment::Center),

                    row![
                        button("Go to Organization")
                            .on_press(Message::TabSelected(Tab::Organization))
                            .style(CowboyCustomTheme::primary_button()),
                        text("Manually create your organization")
                            .size(self.view_model.text_small)
                            .color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(self.view_model.spacing_md)
                    .align_y(Alignment::Center),
                ]
                .spacing(self.view_model.spacing_lg)
            )
            .padding(self.view_model.padding_xl)
            .style(CowboyCustomTheme::glass_container()),
        ]
        .spacing(self.view_model.spacing_xl)
        .padding(self.view_model.padding_md);

        scrollable(content).into()
    }

    fn view_organization(&self) -> Element<'_, Message> {
        use graph::view_graph;

        // ============================================================================
        // GRAPH-FIRST INTERFACE
        // Minimal toolbar + full-screen graph as primary working surface
        // ============================================================================

        // Minimal top toolbar - essential info only
        let toolbar = container(
            row![
                // Domain status (ultra-compact)
                if self.domain_loaded {
                    text(format!("🏢 {} | {} nodes, {} edges",
                        self.organization_name,
                        self.org_graph.nodes.len(),
                        self.org_graph.edges.len()))
                        .size(14)
                } else {
                    text("⚠️  No domain - Right-click canvas to create")
                        .size(14)
                        .color(self.view_model.colors.warning)
                },
                horizontal_space(),
                // View mode switcher buttons
                if self.graph_view == GraphView::Organization {
                    button(text("📊").size(14))
                        .on_press(Message::GraphViewSelected(GraphView::Organization))
                        .style(CowboyCustomTheme::primary_button())
                } else {
                    button(text("📊").size(14))
                        .on_press(Message::GraphViewSelected(GraphView::Organization))
                        .style(CowboyCustomTheme::glass_button())
                },
                if self.graph_view == GraphView::NatsInfrastructure {
                    button(text("🌐").size(14))
                        .on_press(Message::GraphViewSelected(GraphView::NatsInfrastructure))
                        .style(CowboyCustomTheme::primary_button())
                } else {
                    button(text("🌐").size(14))
                        .on_press(Message::GraphViewSelected(GraphView::NatsInfrastructure))
                        .style(CowboyCustomTheme::glass_button())
                },
                if self.graph_view == GraphView::PkiTrustChain {
                    button(text("🔐").size(14))
                        .on_press(Message::GraphViewSelected(GraphView::PkiTrustChain))
                        .style(CowboyCustomTheme::primary_button())
                } else {
                    button(text("🔐").size(14))
                        .on_press(Message::GraphViewSelected(GraphView::PkiTrustChain))
                        .style(CowboyCustomTheme::glass_button())
                },
                if self.graph_view == GraphView::YubiKeyDetails {
                    button(text("🔑").size(14))
                        .on_press(Message::GraphViewSelected(GraphView::YubiKeyDetails))
                        .style(CowboyCustomTheme::primary_button())
                } else {
                    button(text("🔑").size(14))
                        .on_press(Message::GraphViewSelected(GraphView::YubiKeyDetails))
                        .style(CowboyCustomTheme::glass_button())
                },
                horizontal_space(),
                // Context-aware "Add Node" dropdown - shows only valid nodes for current view
                {
                    let node_options = match self.graph_view {
                        GraphView::Organization => vec!["Person", "Unit", "Location", "Role"],
                        GraphView::NatsInfrastructure => vec!["Account", "User", "Service"],
                        GraphView::PkiTrustChain => vec!["Root CA", "Inter CA", "Leaf Cert", "CSR"],
                        GraphView::YubiKeyDetails => vec!["YubiKey", "PIV Slot"],
                    };

                    row![
                        text("➕").size(14),
                        pick_list(
                            node_options,
                            self.selected_node_type.as_ref().map(|s| s.as_str()),
                            |selected| Message::NodeTypeSelected(selected.to_string()),
                        )
                        .placeholder("Add node...")
                        .text_size(12)
                        .style(CowboyCustomTheme::glass_pick_list())
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center)
                },
                // Reset view button
                button(text("↻").size(14))
                    .on_press(Message::GraphMessage(graph::GraphMessage::ResetView))
                    .style(CowboyCustomTheme::glass_button()),
            ]
            .spacing(self.view_model.spacing_sm)
            .align_y(Alignment::Center)
        )
        .padding(self.view_model.padding_sm)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgba8(20, 20, 30, 0.8))),
            ..Default::default()
        });

        // THE GRAPH - this is the primary interface!
        let graph_canvas = {
            let graph_base = Container::new(
                view_graph(&self.org_graph)
                    .map(Message::GraphMessage)
            )
            .width(Length::Fill)
            .height(Length::Fill)  // FILL ALL SPACE!
            .style(|_theme| {
                container::Style {
                    background: Some(Background::Color(self.view_model.colors.background)),
                    border: Border {
                        color: self.view_model.colors.blue_bright,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            });

            // Build stack with overlays
            let mut stack_layers = vec![graph_base.into()];

            // Add context menu overlay if visible
            if self.context_menu.is_visible() {
                let pos = self.context_menu.position();
                const TOOLBAR_OFFSET: f32 = 36.0;

                let menu_overlay = column![
                    vertical_space().height(Length::Fixed(pos.y + TOOLBAR_OFFSET)),
                    row![
                        horizontal_space().width(Length::Fixed(pos.x)),
                        self.context_menu.view()
                            .map(Message::ContextMenuMessage)
                    ]
                ];

                stack_layers.push(menu_overlay.into());
            }

            // Add inline edit overlay if editing a newly created node
            if let Some(node_id) = self.editing_new_node {
                if let Some(node) = self.org_graph.nodes.get(&node_id) {
                    use iced::widget::text_input;
                    const TOOLBAR_OFFSET: f32 = 36.0;

                    // Transform node position to screen coordinates
                    let screen_x = node.position.x * self.org_graph.zoom + self.org_graph.pan_offset.x;
                    let screen_y = node.position.y * self.org_graph.zoom + self.org_graph.pan_offset.y + TOOLBAR_OFFSET;

                    // Position input slightly below the node
                    let input_y = screen_y + 30.0;

                    let edit_overlay = column![
                        vertical_space().height(Length::Fixed(input_y)),
                        row![
                            horizontal_space().width(Length::Fixed(screen_x - 75.0)), // Center on node (150px wide input)
                            container(
                                text_input("Node name...", &self.inline_edit_name)
                                    .on_input(Message::InlineEditNameChanged)
                                    .on_submit(Message::InlineEditSubmit)
                                    .size(14)
                                    .padding(6)
                                    .width(Length::Fixed(150.0))
                            )
                            .padding(4)
                            .style(|_theme| container::Style {
                                background: Some(Background::Color(Color::from_rgba8(40, 40, 50, 0.95))),
                                border: Border {
                                    color: Color::from_rgb(0.4, 0.6, 0.8),
                                    width: 2.0,
                                    radius: 4.0.into(),
                                },
                                ..Default::default()
                            })
                        ]
                    ];

                    stack_layers.push(edit_overlay.into());
                }
            }

            // Add property card overlay if editing
            if self.property_card.is_editing() {
                let card_overlay = container(
                    row![
                        horizontal_space(),
                        self.property_card.view()
                            .map(Message::PropertyCardMessage)
                    ]
                )
                .padding(20);

                stack_layers.push(card_overlay.into());
            }

            stack(stack_layers)
        };

        // Simple two-row layout: tiny toolbar + massive graph
        let content = column![
            toolbar,
            graph_canvas,
        ]
        .spacing(0)
        .height(Length::Fill);


        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_locations(&self) -> Element<'_, Message> {
        // TODO: Re-implement location type picker once Display is implemented in cim-domain-location

        let content = column![
            text("Locations Management").size(self.view_model.text_xlarge),
            text("Manage corporate locations where keys and certificates are stored").size(self.view_model.text_normal),

            // Add location form
            container(
                column![
                    text("Add New Physical Location")
                        .size(self.view_model.text_medium)
                        .color(CowboyTheme::text_primary()),
                    text("Default location type: Physical")
                        .size(self.view_model.text_small)
                        .color(CowboyTheme::text_secondary()),

                    // Location name
                    text_input("Location Name (e.g., Main Office, HQ)", &self.new_location_name)
                        .on_input(Message::NewLocationNameChanged)
                        .size(self.view_model.text_normal)
                        .style(CowboyCustomTheme::glass_input()),

                    // Address section
                    text("Address").size(self.view_model.text_normal).color(CowboyTheme::text_primary()),
                    text_input("Street Address", &self.new_location_street)
                        .on_input(Message::NewLocationStreetChanged)
                        .size(self.view_model.text_normal)
                        .style(CowboyCustomTheme::glass_input()),

                    row![
                        text_input("City", &self.new_location_city)
                            .on_input(Message::NewLocationCityChanged)
                            .size(self.view_model.text_normal)
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("State/Region", &self.new_location_region)
                            .on_input(Message::NewLocationRegionChanged)
                            .size(self.view_model.text_normal)
                            .style(CowboyCustomTheme::glass_input()),
                    ]
                    .spacing(self.view_model.spacing_md),

                    row![
                        text_input("Country", &self.new_location_country)
                            .on_input(Message::NewLocationCountryChanged)
                            .size(self.view_model.text_normal)
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("Postal Code", &self.new_location_postal)
                            .on_input(Message::NewLocationPostalChanged)
                            .size(self.view_model.text_normal)
                            .style(CowboyCustomTheme::glass_input()),
                    ]
                    .spacing(self.view_model.spacing_md),

                    button("Add Location")
                        .on_press(Message::AddLocation)
                        .style(CowboyCustomTheme::primary_button())
                ]
                .spacing(self.view_model.spacing_md)
            )
            .padding(self.view_model.padding_lg)
            .style(CowboyCustomTheme::pastel_mint_card()),

            {
                // Display list of locations - build container once, outside the if-let
                let location_container = if let Ok(projection) = self.projection.try_read() {
                    let locations = projection.get_locations().to_vec();
                    drop(projection); // Release the lock

                    if locations.is_empty() {
                        container(
                            column![
                                text("No locations added yet")
                                    .size(self.view_model.text_normal)
                                    .color(CowboyTheme::text_secondary()),
                            ]
                        )
                        .padding(self.view_model.padding_lg)
                        .style(CowboyCustomTheme::pastel_cream_card())
                    } else {
                        let mut location_list = column![].spacing(self.view_model.spacing_md);

                        for location in locations {
                            // Build address string from optional fields
                            let address_parts: Vec<String> = vec![
                                location.street.clone(),
                                location.city.clone(),
                                location.region.clone(),
                                location.country.clone(),
                                location.postal_code.clone(),
                            ]
                            .into_iter()
                            .flatten()
                            .collect();

                            let address_text = if address_parts.is_empty() {
                                "No address specified".to_string()
                            } else {
                                address_parts.join(", ")
                            };

                            location_list = location_list.push(
                                container(
                                    row![
                                        column![
                                            text(location.name)
                                                .size(self.view_model.text_medium)
                                                .color(CowboyTheme::text_primary()),
                                            text(format!("Type: {}",
                                                location.location_type))
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary()),
                                            text(format!("Address: {}", address_text))
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary()),
                                            text(format!("Created: {}",
                                                location.created_at.format("%Y-%m-%d %H:%M UTC")))
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary()),
                                        ]
                                        .spacing(self.view_model.spacing_sm),
                                        horizontal_space(),
                                        button("Remove")
                                            .on_press(Message::RemoveLocation(location.location_id))
                                            .style(CowboyCustomTheme::security_button())
                                    ]
                                    .align_y(Alignment::Center)
                                    .spacing(self.view_model.spacing_md)
                                )
                                .padding(self.view_model.padding_md)
                                .style(CowboyCustomTheme::pastel_teal_card())
                            );
                        }

                        container(location_list)
                            .padding(self.view_model.padding_lg)
                            .style(CowboyCustomTheme::pastel_cream_card())
                    }
                } else {
                    container(
                        text("Loading locations...")
                            .size(self.view_model.text_normal)
                            .color(CowboyTheme::text_secondary())
                    )
                    .padding(self.view_model.padding_lg)
                    .style(CowboyCustomTheme::pastel_cream_card())
                };

                location_container
            },
        ]
        .spacing(self.view_model.spacing_xl);

        scrollable(content).into()
    }

    fn view_keys(&self) -> Element<'_, Message> {
        let progress_percentage = self.key_generation_progress * 100.0;

        let content = column![
            text("Generate Keys for Organization").size(self.view_model.text_header),
            text("Generate cryptographic keys for all organization members").size(self.view_model.text_large),

            // Root Passphrase Section
            container(
                column![
                    row![
                        text("Root Passphrase (Required for PKI Operations)")
                            .size(self.view_model.text_large)
                            .color(CowboyTheme::text_primary()),
                        horizontal_space(),
                        button(if self.show_passphrase { icons::icon(icons::ICON_VISIBILITY_OFF) } else { icons::icon(icons::ICON_VISIBILITY) })
                            .on_press(Message::TogglePassphraseVisibility)
                            .style(CowboyCustomTheme::glass_button()),
                        button(text("Generate Random").size(self.view_model.text_small))
                            .on_press(Message::GenerateRandomPassphrase)
                            .style(CowboyCustomTheme::primary_button()),
                    ]
                    .spacing(self.view_model.spacing_sm)
                    .align_y(iced::Alignment::Center),
                    text_input("Root Passphrase", &self.root_passphrase)
                        .on_input(Message::RootPassphraseChanged)
                        .secure(!self.show_passphrase)  // Toggle based on show_passphrase
                        .size(self.view_model.text_medium)
                        .padding(self.view_model.padding_md)
                        .style(CowboyCustomTheme::glass_input()),
                    text_input("Confirm Root Passphrase", &self.root_passphrase_confirm)
                        .on_input(Message::RootPassphraseConfirmChanged)
                        .secure(!self.show_passphrase)  // Toggle based on show_passphrase
                        .size(self.view_model.text_medium)
                        .padding(self.view_model.padding_md)
                        .style(CowboyCustomTheme::glass_input()),
                    if !self.root_passphrase.is_empty() && self.root_passphrase == self.root_passphrase_confirm {
                        text("[OK] Passphrases match")
                            .size(self.view_model.text_normal)
                            .color(self.view_model.colors.green_success)
                    } else if !self.root_passphrase.is_empty() && !self.root_passphrase_confirm.is_empty() {
                        text("✗ Passphrases do not match")
                            .size(self.view_model.text_normal)
                            .color(self.view_model.colors.error)
                    } else {
                        text("")
                    }
                ]
                .spacing(self.view_model.spacing_md)
            )
            .padding(self.view_model.padding_xl)
            .style(CowboyCustomTheme::pastel_coral_card()),

            container(
                column![
                    text("PKI Hierarchy Generation")
                        .size(self.view_model.text_large)
                        .color(CowboyTheme::text_primary()),

                    // Graph-First PKI Generation (NEW!)
                    container(
                        column![
                            text("🎯 Graph-First PKI Generation")
                                .size(self.view_model.text_large)
                                .color(CowboyTheme::text_primary()),
                            text("Generate complete PKI hierarchy from your organizational graph")
                                .size(self.view_model.text_normal)
                                .color(CowboyTheme::text_secondary()),
                            text("• Organization → Root CA")
                                .size(self.view_model.text_small)
                                .color(CowboyTheme::text_muted()),
                            text("• Organizational Units → Intermediate CAs")
                                .size(self.view_model.text_small)
                                .color(CowboyTheme::text_muted()),
                            text("• People → Leaf Certificates")
                                .size(self.view_model.text_small)
                                .color(CowboyTheme::text_muted()),
                            button(
                                text("Generate PKI from Graph")
                                    .size(self.view_model.text_large)
                            )
                                .on_press_maybe(
                                    if !self.root_passphrase.is_empty() &&
                                       self.root_passphrase == self.root_passphrase_confirm &&
                                       self.org_graph.nodes.iter().any(|(_, n)| matches!(n.node_type, graph::NodeType::Organization(_))) {
                                        Some(Message::GeneratePkiFromGraph)
                                    } else {
                                        None
                                    }
                                )
                                .padding(self.view_model.padding_xl)
                                .width(Length::Fill)
                                .style(CowboyCustomTheme::primary_button()),
                            if self.root_passphrase.is_empty() || self.root_passphrase != self.root_passphrase_confirm {
                                text("⚠️  Enter and confirm root passphrase above")
                                    .size(self.view_model.text_small)
                                    .color(self.view_model.colors.warning)
                            } else if !self.org_graph.nodes.iter().any(|(_, n)| matches!(n.node_type, graph::NodeType::Organization(_))) {
                                text("⚠️  Create an organization in the Organization tab first")
                                    .size(self.view_model.text_small)
                                    .color(self.view_model.colors.warning)
                            } else {
                                text("✓ Ready to generate PKI from graph structure")
                                    .size(self.view_model.text_small)
                                    .color(self.view_model.colors.green_success)
                            }
                        ]
                        .spacing(self.view_model.spacing_sm)
                    )
                    .padding(self.view_model.padding_xl)
                    .style(CowboyCustomTheme::pastel_teal_card()),

                    // Divider
                    container(
                        text("─ OR Generate Individual Components ─")
                            .size(self.view_model.text_normal)
                            .color(CowboyTheme::text_muted())
                    )
                    .width(Length::Fill)
                    .center_x(Length::Fill),

                    // Root CA Section
                    text("1. Root CA (Trust Anchor)").size(self.view_model.text_medium),
                    button("Generate Root CA")
                        .on_press(Message::GenerateRootCA)
                        .padding(self.view_model.padding_lg)
                        .style(CowboyCustomTheme::security_button()),

                    // Intermediate CA Section
                    text("2. Intermediate CA (Signing-Only, pathlen:0)").size(self.view_model.text_medium),
                    text_input("CA Name (e.g., 'Engineering')", &self.intermediate_ca_name_input)
                        .on_input(Message::IntermediateCANameChanged)
                        .size(self.view_model.text_medium)
                        .padding(self.view_model.padding_md)
                        .style(CowboyCustomTheme::glass_input()),

                    // Storage location picker for intermediate CA
                    if !self.loaded_locations.is_empty() {
                        let location_names: Vec<String> = self.loaded_locations
                            .iter()
                            .map(|loc| loc.name.clone())
                            .collect();

                        row![
                            text("Storage Location:").size(self.view_model.text_normal),
                            pick_list(
                                location_names,
                                self.selected_cert_location.clone(),
                                Message::SelectCertLocation,
                            )
                            .placeholder("Select Storage Location")
                        ]
                        .spacing(self.view_model.spacing_md)
                    } else {
                        row![
                            text("Storage Location:").size(self.view_model.text_normal).color(self.view_model.colors.text_tertiary),
                            text("(no locations defined)").size(self.view_model.text_normal).color(self.view_model.colors.text_tertiary),
                        ]
                        .spacing(self.view_model.spacing_md)
                    },

                    button("Generate Intermediate CA")
                        .on_press(Message::GenerateIntermediateCA)
                        .padding(self.view_model.padding_lg)
                        .style(CowboyCustomTheme::primary_button()),

                    // Server Certificate Section
                    text("3. Server Certificates").size(self.view_model.text_medium),
                    text_input("Common Name (e.g., 'nats.example.com')", &self.server_cert_cn_input)
                        .on_input(Message::ServerCertCNChanged)
                        .size(self.view_model.text_medium)
                        .padding(self.view_model.padding_md)
                        .style(CowboyCustomTheme::glass_input()),
                    text_input("SANs (comma-separated DNS names or IPs)", &self.server_cert_sans_input)
                        .on_input(Message::ServerCertSANsChanged)
                        .size(self.view_model.text_medium)
                        .padding(self.view_model.padding_md)
                        .style(CowboyCustomTheme::glass_input()),

                    // Certificate metadata fields (collapsible section for details)
                    text("Certificate Details").size(self.view_model.text_normal).color(self.view_model.colors.text_secondary),

                    row![
                        text_input("Organization", &self.cert_organization)
                            .on_input(Message::CertOrganizationChanged)
                            .size(self.view_model.text_normal)
                            .padding(self.view_model.padding_md)
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("Organizational Unit", &self.cert_organizational_unit)
                            .on_input(Message::CertOrganizationalUnitChanged)
                            .size(self.view_model.text_normal)
                            .padding(self.view_model.padding_md)
                            .style(CowboyCustomTheme::glass_input()),
                    ]
                    .spacing(self.view_model.spacing_md),

                    row![
                        text_input("Locality/City", &self.cert_locality)
                            .on_input(Message::CertLocalityChanged)
                            .size(self.view_model.text_normal)
                            .padding(self.view_model.padding_md)
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("State/Province", &self.cert_state_province)
                            .on_input(Message::CertStateProvinceChanged)
                            .size(self.view_model.text_normal)
                            .padding(self.view_model.padding_md)
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("Country (2-letter code)", &self.cert_country)
                            .on_input(Message::CertCountryChanged)
                            .size(self.view_model.text_normal)
                            .padding(self.view_model.padding_md)
                            .style(CowboyCustomTheme::glass_input()),
                    ]
                    .spacing(self.view_model.spacing_md),

                    text_input("Validity (days)", &self.cert_validity_days)
                        .on_input(Message::CertValidityDaysChanged)
                        .size(self.view_model.text_normal)
                        .padding(self.view_model.padding_md)
                        .style(CowboyCustomTheme::glass_input()),

                    // CA selection picker
                    if !self.mvi_model.key_generation_status.intermediate_cas.is_empty() {
                        let ca_names: Vec<String> = self.mvi_model.key_generation_status.intermediate_cas
                            .iter()
                            .map(|ca| ca.name.clone())
                            .collect();

                        row![
                            text("Signing CA:").size(self.view_model.text_normal),
                            pick_list(
                                ca_names,
                                self.selected_intermediate_ca.clone(),
                                Message::SelectIntermediateCA,
                            )
                            .placeholder("Select Intermediate CA")
                        ]
                        .spacing(self.view_model.spacing_md)
                    } else {
                        row![
                            text("Signing CA:").size(self.view_model.text_normal).color(self.view_model.colors.text_tertiary),
                            text("(generate an intermediate CA first)").size(self.view_model.text_normal).color(self.view_model.colors.text_tertiary),
                        ]
                        .spacing(self.view_model.spacing_md)
                    },

                    // Storage location picker
                    if !self.loaded_locations.is_empty() {
                        let location_names: Vec<String> = self.loaded_locations
                            .iter()
                            .map(|loc| loc.name.clone())
                            .collect();

                        row![
                            text("Storage Location:").size(self.view_model.text_normal),
                            pick_list(
                                location_names,
                                self.selected_cert_location.clone(),
                                Message::SelectCertLocation,
                            )
                            .placeholder("Select Storage Location")
                        ]
                        .spacing(self.view_model.spacing_md)
                    } else {
                        row![
                            text("Storage Location:").size(self.view_model.text_normal).color(self.view_model.colors.text_tertiary),
                            text("(no locations defined)").size(self.view_model.text_normal).color(self.view_model.colors.text_tertiary),
                        ]
                        .spacing(self.view_model.spacing_md)
                    },

                    button("Generate Server Certificate")
                        .on_press(Message::GenerateServerCert)
                        .style(CowboyCustomTheme::primary_button()),

                    // Display generated certificates from MVI model
                    if !self.mvi_model.key_generation_status.intermediate_cas.is_empty()
                       || !self.mvi_model.key_generation_status.server_certificates.is_empty() {
                        container(
                            column![
                                text("Generated Certificates").size(self.view_model.text_medium).color(self.view_model.colors.success),

                                // Intermediate CAs
                                if !self.mvi_model.key_generation_status.intermediate_cas.is_empty() {
                                    iced::widget::Column::with_children(
                                        self.mvi_model.key_generation_status.intermediate_cas.iter().map(|ca| {
                                            text(format!("  [OK] CA: {} - {}", ca.name, &ca.fingerprint[..16]))
                                                .size(self.view_model.text_small)
                                                .color(self.view_model.colors.success)
                                                .into()
                                        }).collect::<Vec<_>>()
                                    )
                                    .spacing(self.view_model.spacing_xs)
                                } else {
                                    column![]
                                },

                                // Server Certificates
                                if !self.mvi_model.key_generation_status.server_certificates.is_empty() {
                                    iced::widget::Column::with_children(
                                        self.mvi_model.key_generation_status.server_certificates.iter().map(|cert| {
                                            column![
                                                text(format!("  [OK] Server: {} (signed by: {})", cert.common_name, cert.signed_by))
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.success),
                                                text(format!("    Fingerprint: {}", &cert.fingerprint[..16]))
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.text_disabled),
                                            ]
                                            .spacing(self.view_model.spacing_xs)
                                            .into()
                                        }).collect::<Vec<_>>()
                                    )
                                    .spacing(self.view_model.spacing_sm)
                                } else {
                                    column![]
                                },
                            ]
                            .spacing(self.view_model.spacing_md)
                        )
                        .padding(self.view_model.padding_md)
                        .style(CowboyCustomTheme::card_container())
                    } else {
                        container(text(""))
                    },

                    // Step 4: YubiKey Detection and Management (Card with Collapse)
                    container(
                        column![
                            row![
                                button(if self.yubikey_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleYubiKeySection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("4. YubiKey Hardware Detection")
                                    .size(self.view_model.text_large)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.yubikey_section_collapsed {
                                column![
                                    row![
                                        button("Detect YubiKeys")
                                            .on_press(Message::DetectYubiKeys)
                                            .padding(self.view_model.padding_lg)
                                            .style(CowboyCustomTheme::security_button()),
                                        text(&self.yubikey_detection_status)
                                            .size(self.view_model.text_normal)
                                            .color(CowboyTheme::text_secondary()),
                                    ]
                                    .spacing(self.view_model.spacing_md)
                                    .align_y(Alignment::Center),

                    // Display detected YubiKeys
                    if !self.detected_yubikeys.is_empty() {
                        let mut yubikey_list = column![].spacing(self.view_model.spacing_sm);

                        for device in &self.detected_yubikeys {
                            yubikey_list = yubikey_list.push(
                                row![
                                    text(format!("  [OK] {} v{} - Serial: {}",
                                        device.model,
                                        device.version,
                                        device.serial))
                                        .size(self.view_model.text_small)
                                        .color(if device.piv_enabled {
                                            self.view_model.colors.green_success
                                        } else {
                                            self.view_model.colors.orange_warning
                                        }),
                                    if !device.piv_enabled {
                                        text(" (PIV not enabled)")
                                            .size(self.view_model.text_small)
                                            .color(self.view_model.colors.orange_warning)
                                    } else {
                                        text("")
                                    }
                                ]
                                .spacing(self.view_model.spacing_sm)
                            );
                        }

                        container(yubikey_list)
                            .padding(self.view_model.padding_md)
                            .style(CowboyCustomTheme::card_container())
                    } else {
                        container(text(""))
                    },

                    // YubiKey Configurations (imported from secrets)
                    if !self.yubikey_configs.is_empty() {
                        text(format!("🔑 YubiKey Configurations ({} imported)", self.yubikey_configs.len()))
                            .size(self.view_model.text_normal)
                            .color(CowboyTheme::text_primary())
                    } else {
                        text("")
                    },

                    if !self.yubikey_configs.is_empty() {
                        let mut config_list = column![].spacing(self.view_model.spacing_sm);

                        for config in &self.yubikey_configs {
                            let role_str = match config.role {
                                crate::domain::YubiKeyRole::RootCA => "[LOCK] Root CA",
                                crate::domain::YubiKeyRole::Backup => "💾 Backup",
                                crate::domain::YubiKeyRole::User => "👤 User",
                                crate::domain::YubiKeyRole::Service => "⚙️ Service",
                            };

                            config_list = config_list.push(
                                container(
                                    column![
                                        text(format!("{} - {}", role_str, config.name))
                                            .size(self.view_model.text_normal)
                                            .color(CowboyTheme::text_primary()),
                                        row![
                                            column![
                                                text(format!("Serial: {}", config.serial))
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                                text(format!("Owner: {}", config.owner_email))
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                            ]
                                            .spacing(self.view_model.spacing_xs),
                                            horizontal_space(),
                                            column![
                                                text(format!("PIN: {}", config.piv.pin))
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.orange_warning),
                                                text(format!("PUK: {}", config.piv.puk))
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.orange_warning),
                                                text(format!("Mgmt: {}...", &config.piv.mgmt_key[..12]))
                                                    .size(self.view_model.text_tiny)
                                                    .color(self.view_model.colors.orange_warning),
                                            ]
                                            .spacing(self.view_model.spacing_xs)
                                            .align_x(iced::alignment::Horizontal::Right),
                                        ]
                                        .align_y(Alignment::Center),
                                    ]
                                    .spacing(self.view_model.spacing_sm)
                                )
                                .padding(self.view_model.padding_md)
                                .style(CowboyCustomTheme::pastel_coral_card())
                            );
                        }

                        container(config_list)
                            .padding(self.view_model.padding_md)
                            .style(CowboyCustomTheme::card_container())
                    } else {
                        container(text(""))
                    },
                                ]
                                .spacing(self.view_model.spacing_md)
                            } else {
                                column![]
                            }
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_xl)
                    .style(CowboyCustomTheme::pastel_mint_card()),

                    // Loaded Certificates from Manifest (Collapsible)
                    if !self.loaded_certificates.is_empty() {
                        container(
                            column![
                                row![
                                    button(if self.certificates_collapsed { "▶" } else { "▼" })
                                        .on_press(Message::ToggleCertificatesSection)
                                        .padding(self.view_model.padding_sm)
                                        .style(CowboyCustomTheme::glass_button()),
                                    text(format!("📜 Certificates from Manifest ({} loaded)", self.loaded_certificates.len()))
                                        .size(self.view_model.text_medium)
                                        .color(CowboyTheme::text_primary()),
                                ]
                                .spacing(self.view_model.spacing_md)
                                .align_y(Alignment::Center),

                                if !self.certificates_collapsed {
                                    {
                                        let mut cert_list = column![].spacing(self.view_model.spacing_sm);

                                        for cert in &self.loaded_certificates {
                                            cert_list = cert_list.push(
                                                container(
                                                    column![
                                                        text(format!("[LOCK] {}{}", cert.subject, if cert.is_ca { " (CA)" } else { "" }))
                                                            .size(self.view_model.text_small)
                                                            .color(if cert.is_ca { self.view_model.colors.red_error } else { self.view_model.colors.info }),
                                                        row![
                                                            column![
                                                                text(format!("Serial: {}...", &cert.serial_number.chars().take(16).collect::<String>()))
                                                                    .size(self.view_model.text_tiny)
                                                                    .color(CowboyTheme::text_secondary()),
                                                                if let Some(ref issuer) = cert.issuer {
                                                                    text(format!("Issuer: {}", issuer))
                                                                        .size(self.view_model.text_tiny)
                                                                        .color(CowboyTheme::text_secondary())
                                                                } else {
                                                                    text("")
                                                                },
                                                            ]
                                                            .spacing(self.view_model.spacing_xs),
                                                            horizontal_space(),
                                                            column![
                                                                text(format!("Valid: {} to {}",
                                                                    cert.not_before.format("%Y-%m-%d"),
                                                                    cert.not_after.format("%Y-%m-%d")))
                                                                    .size(self.view_model.text_tiny)
                                                                    .color(CowboyTheme::text_secondary()),
                                                            ]
                                                            .align_x(iced::alignment::Horizontal::Right),
                                                        ]
                                                    ]
                                                    .spacing(self.view_model.spacing_xs)
                                                )
                                                .padding(self.view_model.padding_sm)
                                                .style(CowboyCustomTheme::pastel_teal_card())
                                            );
                                        }

                                        container(cert_list)
                                            .padding(self.view_model.padding_md)
                                            .style(CowboyCustomTheme::card_container())
                                    }
                                } else {
                                    container(text(""))
                                }
                            ]
                            .spacing(self.view_model.spacing_md)
                        )
                        .padding(self.view_model.padding_xl)
                        .style(CowboyCustomTheme::pastel_teal_card())
                    } else {
                        container(text(""))
                    },

                    // Loaded Keys from Manifest (Collapsible)
                    if !self.loaded_keys.is_empty() {
                        container(
                            column![
                                row![
                                    button(if self.keys_collapsed { "▶" } else { "▼" })
                                        .on_press(Message::ToggleKeysSection)
                                        .padding(self.view_model.padding_sm)
                                        .style(CowboyCustomTheme::glass_button()),
                                    text(format!("🔑 Keys from Manifest ({} loaded)", self.loaded_keys.len()))
                                        .size(self.view_model.text_medium)
                                        .color(CowboyTheme::text_primary()),
                                ]
                                .spacing(self.view_model.spacing_md)
                                .align_y(Alignment::Center),

                                if !self.keys_collapsed {
                                    {
                                        let mut key_list = column![].spacing(self.view_model.spacing_sm);

                                        for key in &self.loaded_keys {
                            key_list = key_list.push(
                                container(
                                    column![
                                        text(format!("[LOCK] {} - {}", key.label, format!("{:?}", key.algorithm)))
                                            .size(self.view_model.text_small)
                                            .color(if key.revoked { self.view_model.colors.red_error } else { self.view_model.colors.green_success }),
                                        row![
                                            column![
                                                text(format!("Purpose: {:?}", key.purpose))
                                                    .size(self.view_model.text_tiny)
                                                    .color(CowboyTheme::text_secondary()),
                                                if key.hardware_backed {
                                                    if let Some(ref serial) = key.yubikey_serial {
                                                        text(format!("YubiKey: {}", serial))
                                                            .size(self.view_model.text_tiny)
                                                            .color(self.view_model.colors.info)
                                                    } else {
                                                        text("Hardware-backed")
                                                            .size(self.view_model.text_tiny)
                                                            .color(self.view_model.colors.info)
                                                    }
                                                } else {
                                                    text("Software key")
                                                        .size(self.view_model.text_tiny)
                                                        .color(CowboyTheme::text_secondary())
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_xs),
                                            horizontal_space(),
                                            column![
                                                text(format!("Created: {}", key.created_at.format("%Y-%m-%d")))
                                                    .size(self.view_model.text_tiny)
                                                    .color(CowboyTheme::text_secondary()),
                                                if key.revoked {
                                                    text("[WARNING] REVOKED")
                                                        .size(self.view_model.text_tiny)
                                                        .color(self.view_model.colors.red_error)
                                                } else {
                                                    text("")
                                                },
                                            ]
                                            .align_x(iced::alignment::Horizontal::Right),
                                        ]
                                    ]
                                    .spacing(self.view_model.spacing_xs)
                                )
                                .padding(self.view_model.padding_sm)
                                .style(CowboyCustomTheme::pastel_mint_card())
                            );
                        }

                                        container(key_list)
                                            .padding(self.view_model.padding_md)
                                            .style(CowboyCustomTheme::card_container())
                                    }
                                } else {
                                    container(text(""))
                                }
                            ]
                            .spacing(self.view_model.spacing_md)
                        )
                        .padding(self.view_model.padding_xl)
                        .style(CowboyCustomTheme::pastel_cream_card())
                    } else {
                        container(text(""))
                    },

                    // Step 5: NATS Hierarchy Generation (Card with Collapse)
                    container(
                        column![
                            row![
                                button(if self.nats_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleNatsSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("5. NATS Hierarchy (Operator/Account/User)")
                                    .size(self.view_model.text_large)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.nats_section_collapsed {
                                column![
                                    // Graph-First NATS Generation (NEW!)
                                    container(
                                        column![
                                            text("🎯 Graph-First NATS Generation")
                                                .size(self.view_model.text_large)
                                                .color(CowboyTheme::text_primary()),
                                            text("Generate complete NATS hierarchy from your organizational graph")
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_secondary()),
                                            text("• Organization → NATS Operator")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_muted()),
                                            text("• Organizational Units → NATS Accounts")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_muted()),
                                            text("• People → NATS Users")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_muted()),
                                            button(
                                                text("Generate NATS from Graph")
                                                    .size(self.view_model.text_large)
                                            )
                                                .on_press_maybe(
                                                    if self.org_graph.nodes.iter().any(|(_, n)| matches!(n.node_type, graph::NodeType::Organization(_))) {
                                                        Some(Message::GenerateNatsFromGraph)
                                                    } else {
                                                        None
                                                    }
                                                )
                                                .padding(self.view_model.padding_xl)
                                                .width(Length::Fill)
                                                .style(CowboyCustomTheme::primary_button()),
                                            if !self.org_graph.nodes.iter().any(|(_, n)| matches!(n.node_type, graph::NodeType::Organization(_))) {
                                                text("⚠️  Create an organization in the Organization tab first")
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.warning)
                                            } else {
                                                text("✓ Ready to generate NATS from graph structure")
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.green_success)
                                            }
                                        ]
                                        .spacing(self.view_model.spacing_sm)
                                    )
                                    .padding(self.view_model.padding_xl)
                                    .style(CowboyCustomTheme::pastel_coral_card()),

                                    // Divider
                                    container(
                                        text("─ OR Generate Individual Components ─")
                                            .size(self.view_model.text_normal)
                                            .color(CowboyTheme::text_muted())
                                    )
                                    .width(Length::Fill)
                                    .center_x(Length::Fill),

                                    row![
                                        button("Generate NATS Hierarchy")
                                            .on_press(Message::GenerateNatsHierarchy)
                                            .padding(self.view_model.padding_lg)
                                            .style(CowboyCustomTheme::security_button()),
                                        if self.nats_hierarchy_generated {
                                            text("[OK] NATS hierarchy generated")
                                                .size(self.view_model.text_normal)
                                                .color(self.view_model.colors.success)
                                        } else {
                                            text("Generate NATS operator, accounts, and users")
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_secondary())
                                        }
                                    ]
                                    .spacing(self.view_model.spacing_md)
                                    .align_y(Alignment::Center),
                                ]
                                .spacing(self.view_model.spacing_md)
                            } else {
                                column![]
                            }
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_xl)
                    .style(CowboyCustomTheme::pastel_coral_card()),

                    // Step 6: Other Key Generation (Card with Collapse)
                    container(
                        column![
                            row![
                                button(if self.keys_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleKeysSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("6. Other Keys")
                                    .size(self.view_model.text_large)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.keys_collapsed {
                                column![
                                    // Graph-First YubiKey Provisioning (NEW!)
                                    container(
                                        column![
                                            text("🎯 Graph-First YubiKey Provisioning")
                                                .size(self.view_model.text_large)
                                                .color(CowboyTheme::text_primary()),
                                            text("Analyze YubiKey requirements from your organizational graph")
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_secondary()),
                                            text("• Root Authority → Signature slot (9C)")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_muted()),
                                            text("• Security Admin → All slots (9A, 9C, 9D)")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_muted()),
                                            text("• Developers → Authentication slot (9A)")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_muted()),
                                            button(
                                                text("Analyze YubiKey Requirements from Graph")
                                                    .size(self.view_model.text_large)
                                            )
                                                .on_press_maybe(
                                                    if self.org_graph.nodes.iter().any(|(_, n)| matches!(n.node_type, graph::NodeType::Person{..})) {
                                                        Some(Message::ProvisionYubiKeysFromGraph)
                                                    } else {
                                                        None
                                                    }
                                                )
                                                .padding(self.view_model.padding_xl)
                                                .width(Length::Fill)
                                                .style(CowboyCustomTheme::primary_button()),
                                            if !self.org_graph.nodes.iter().any(|(_, n)| matches!(n.node_type, graph::NodeType::Person{..})) {
                                                text("⚠️  Add people with roles in the Organization tab first")
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.warning)
                                            } else {
                                                text("✓ Ready to analyze YubiKey requirements")
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.green_success)
                                            }
                                        ]
                                        .spacing(self.view_model.spacing_sm)
                                    )
                                    .padding(self.view_model.padding_xl)
                                    .style(CowboyCustomTheme::pastel_mint_card()),

                                    // Divider
                                    container(
                                        text("─ OR Generate Individual Keys ─")
                                            .size(self.view_model.text_normal)
                                            .color(CowboyTheme::text_muted())
                                    )
                                    .width(Length::Fill)
                                    .center_x(Length::Fill),

                                    button("Generate SSH Keys for All")
                                        .on_press(Message::GenerateSSHKeys)
                                        .padding(self.view_model.padding_lg)
                                        .style(CowboyCustomTheme::primary_button()),
                                    button("Provision YubiKeys (Manual)")
                                        .on_press(Message::ProvisionYubiKey)
                                        .padding(self.view_model.padding_lg)
                                        .style(CowboyCustomTheme::glass_button()),
                                    button("Generate All Keys")
                                        .on_press(Message::GenerateAllKeys)
                                        .padding(self.view_model.padding_lg)
                                        .style(CowboyCustomTheme::security_button()),
                                ]
                                .spacing(self.view_model.spacing_md)
                            } else {
                                column![]
                            }
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_xl)
                    .style(CowboyCustomTheme::pastel_mint_card()),
                ]
                .spacing(self.view_model.spacing_md)
            )
            .padding(self.view_model.padding_lg)
            .style(CowboyCustomTheme::pastel_mint_card()),

            if self.key_generation_progress > 0.0 {
                container(
                    column![
                        text(format!("Progress: {:.0}%", progress_percentage)).size(self.view_model.text_normal),
                        progress_bar(0.0..=1.0, self.key_generation_progress),
                        text(format!("{} of {} keys generated",
                            self.keys_generated,
                            self.total_keys_to_generate)).size(self.view_model.text_small),
                    ]
                    .spacing(self.view_model.spacing_sm)
                )
                .padding(self.view_model.padding_md)
            } else {
                container(text("No key generation in progress").size(self.view_model.text_normal))
            }
        ]
        .spacing(self.view_model.spacing_xl)
        .padding(self.view_model.padding_md);

        scrollable(content).into()
    }

    fn view_export(&self) -> Element<'_, Message> {
        let content = column![
            text("Export Domain Configuration").size(self.view_model.text_xlarge),
            text("Export your domain configuration to encrypted storage").size(self.view_model.text_normal),

            container(
                column![
                    text("Export Options")
                        .size(self.view_model.text_medium)
                        .color(CowboyTheme::text_primary()),
                    text_input("Output Directory", &self.export_path.display().to_string())
                        .on_input(Message::ExportPathChanged)
                        .style(CowboyCustomTheme::glass_input()),
                    checkbox("Include public keys", self.include_public_keys)
                        .on_toggle(Message::TogglePublicKeys)
                        .style(CowboyCustomTheme::light_checkbox()),
                    checkbox("Include certificates", self.include_certificates)
                        .on_toggle(Message::ToggleCertificates)
                        .style(CowboyCustomTheme::light_checkbox()),
                    checkbox("Generate NATS configuration", self.include_nats_config)
                        .on_toggle(Message::ToggleNatsConfig)
                        .style(CowboyCustomTheme::light_checkbox()),
                    checkbox("Include private keys (requires password)", self.include_private_keys)
                        .on_toggle(Message::TogglePrivateKeys)
                        .style(CowboyCustomTheme::light_checkbox()),
                    if self.include_private_keys {
                        text_input("Encryption Password", &self.export_password)
                            .on_input(Message::ExportPasswordChanged)
                            .secure(true)
                            .style(CowboyCustomTheme::glass_input())
                    } else {
                        text_input("", "")
                            .on_input(Message::ExportPasswordChanged)
                            .style(CowboyCustomTheme::glass_input())
                    },
                ]
                .spacing(self.view_model.spacing_md)
            )
            .padding(self.view_model.padding_lg)
            .style(CowboyCustomTheme::pastel_cream_card()),

            button("Export to Encrypted SD Card")
                .on_press(Message::ExportToSDCard)
                .style(CowboyCustomTheme::security_button()),

            // NATS NSC Export
            if self.nats_hierarchy_generated {
                container(
                    column![
                        text("Export NATS Hierarchy to NSC")
                            .size(self.view_model.text_medium)
                            .color(CowboyTheme::text_primary()),
                        text(format!("Export directory: {}", self.nats_export_path.display()))
                            .size(self.view_model.text_small)
                            .color(CowboyTheme::text_secondary()),
                        button("Export to NSC Store")
                            .on_press(Message::ExportToNsc)
                            .style(CowboyCustomTheme::primary_button()),
                    ]
                    .spacing(self.view_model.spacing_md)
                )
                .padding(self.view_model.padding_lg)
                .style(CowboyCustomTheme::pastel_teal_card())
            } else {
                container(
                    text("Generate NATS hierarchy first to enable NSC export")
                        .size(self.view_model.text_normal)
                        .color(self.view_model.colors.text_tertiary)
                )
                .padding(self.view_model.padding_lg)
                .style(CowboyCustomTheme::card_container())
            },
        ]
        .spacing(self.view_model.spacing_xl)
        .padding(self.view_model.padding_md);

        scrollable(content).into()
    }
}

// Async functions for operations

#[cfg(not(target_arch = "wasm32"))]
async fn load_config_native() -> Result<BootstrapConfig, String> {
    use rfd::AsyncFileDialog;

    let file = AsyncFileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file()
        .await
        .ok_or_else(|| "No file selected".to_string())?;

    let contents = file.read().await;
    serde_json::from_slice(&contents).map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn load_config_wasm() -> Result<BootstrapConfig, String> {
    use wasm_bindgen::JsCast;
    use web_sys::{HtmlInputElement, Event};
    use wasm_bindgen_futures::JsFuture;
    use gloo_file::callbacks::FileReader;
    use gloo_file::File as GlooFile;

    // Create a file input element
    let document = web_sys::window()
        .ok_or_else(|| "No window object".to_string())?
        .document()
        .ok_or_else(|| "No document object".to_string())?;

    let input: HtmlInputElement = document
        .create_element("input")
        .map_err(|e| format!("Failed to create input element: {:?}", e))?
        .dyn_into()
        .map_err(|e| format!("Failed to cast to HtmlInputElement: {:?}", e))?;

    input.set_type("file");
    input.set_accept(".json");

    // Create a promise to wait for file selection
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<Vec<u8>, String>>();

    // Setup file change event listener
    let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: Event| {
        let input = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

        if let Some(input) = input {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let file = GlooFile::from(file);
                    let tx_clone = tx.clone();

                    // Read file content
                    let reader = FileReader::new();
                    let mut reader_clone = reader.clone();

                    reader.read_as_bytes(&file, move |result| {
                        if let Some(tx) = tx_clone.lock().unwrap().take() {
                            match result {
                                Ok(data) => {
                                    let _ = tx.send(Ok(data));
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(format!("Failed to read file: {:?}", e)));
                                }
                            }
                        }
                    });
                }
            }
        }
    }) as Box<dyn FnMut(_)>);

    input.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
        .map_err(|e| format!("Failed to add event listener: {:?}", e))?;

    // Trigger the file picker
    input.click();

    // Keep the closure alive
    closure.forget();

    // Wait for file to be selected and read
    let file_contents = rx.await
        .map_err(|e| format!("File selection cancelled or failed: {:?}", e))??;

    // Parse JSON
    serde_json::from_slice(&file_contents)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

// Note: Full key generation workflow is now handled through MVI architecture
// See mvi/update.rs for the event-driven approach to key generation
#[allow(dead_code)]
async fn generate_all_keys(
    aggregate: Arc<RwLock<KeyManagementAggregate>>,
    projection: Arc<RwLock<OfflineKeyProjection>>,
) -> Result<usize, String> {
    use crate::commands::{KeyCommand, GenerateSshKeyCommand};
    use crate::events::KeyEvent;

    let mut total_keys = 0;
    let mut events_to_apply = Vec::new();

    // Generate a test SSH key for demonstration
    let ssh_cmd = GenerateSshKeyCommand {
        command_id: cim_domain::EntityId::new(),
        person_id: uuid::Uuid::now_v7(),
        key_type: "ed25519".to_string(),
        comment: Some("test@example.com".to_string()),
        requestor: "GUI User".to_string(),
    };

    // Process the command through the aggregate
    let aggregate = aggregate.read().await;
    let projection_read = projection.read().await;

    let events = aggregate.handle_command(
        KeyCommand::GenerateSshKey(ssh_cmd),
        &projection_read,
        None,  // No NATS port in offline mode
        #[cfg(feature = "policy")]
        None   // No policy engine in GUI yet
    ).await
    .map_err(|e| format!("Failed to generate SSH key: {}", e))?;

    total_keys += events.len();
    events_to_apply.extend(events);

    // Drop read locks before getting write lock
    drop(projection_read);
    drop(aggregate);

    // Apply events to the projection
    if !events_to_apply.is_empty() {
        let projection_write = projection.write().await;
        for event in events_to_apply {
            match event {
                KeyEvent::SshKeyGenerated(e) => {
                    // Save metadata about the SSH key generation
                    // Actual key generation would happen in a dedicated service
                    let key_dir = projection_write.root_path.join("keys").join("ssh");
                    std::fs::create_dir_all(&key_dir)
                        .map_err(|e| format!("Failed to create SSH key directory: {}", e))?;

                    // Save key metadata
                    let metadata = serde_json::json!({
                        "key_id": e.key_id,
                        "key_type": format!("{:?}", e.key_type),
                        "comment": e.comment,
                        "generated_at": e.generated_at,
                    });

                    let metadata_file = key_dir.join(format!("{}.json", e.key_id));
                    std::fs::write(&metadata_file, serde_json::to_string_pretty(&metadata).unwrap())
                        .map_err(|e| format!("Failed to save SSH key metadata: {}", e))?;

                    // Update the manifest
                    projection_write.save_manifest()
                        .map_err(|e| format!("Failed to update manifest: {}", e))?;
                },
                _ => {
                    // Handle other event types as needed
                }
            }
        }
    }

    Ok(total_keys)
}

async fn generate_root_ca(
    aggregate: Arc<RwLock<KeyManagementAggregate>>,
    projection: Arc<RwLock<OfflineKeyProjection>>,
) -> Result<Uuid, String> {
    use crate::commands::{KeyCommand, GenerateCertificateCommand, CertificateSubject};
    use crate::events::KeyEvent;
    use crate::certificate_service;

    // Create Root CA command
    let _cert_id = Uuid::now_v7();  // Reserved for certificate tracking
    let root_ca_cmd = GenerateCertificateCommand {
        command_id: cim_domain::EntityId::new(),
        key_id: Uuid::now_v7(),
        subject: CertificateSubject {
            common_name: "CIM Root CA".to_string(),
            organization: Some("CIM Infrastructure".to_string()),
            country: Some("US".to_string()),
            organizational_unit: Some("Security".to_string()),
            locality: None,
            state_or_province: None,
        },
        validity_days: 3650,
        is_ca: true,
        san: vec![],
        key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
        extended_key_usage: vec![],
        requestor: "GUI User".to_string(),
        context: None,
    };

    // Process through aggregate to get event
    let aggregate = aggregate.read().await;
    let projection_read = projection.read().await;

    let events = aggregate.handle_command(
        KeyCommand::GenerateCertificate(root_ca_cmd),
        &projection_read,
        None,
        #[cfg(feature = "policy")]
        None
    ).await
    .map_err(|e| format!("Failed to generate Root CA: {}", e))?;

    drop(projection_read);
    drop(aggregate);

    // Process the certificate generation event
    let mut cert_id = Uuid::nil();
    if !events.is_empty() {
        let projection_write = projection.write().await;

        for event in events {
            if let KeyEvent::CertificateGenerated(e) = event {
                cert_id = e.cert_id;

                // Generate actual certificate using the service
                let generated = certificate_service::generate_root_ca_from_event(&e)
                    .map_err(|e| format!("Certificate generation failed: {}", e))?;

                // Save certificate to projection
                let cert_dir = projection_write.root_path.join("certificates").join("root-ca");
                std::fs::create_dir_all(&cert_dir)
                    .map_err(|e| format!("Failed to create certificate directory: {}", e))?;

                // Save certificate PEM
                let cert_file = cert_dir.join(format!("{}.crt", e.cert_id));
                std::fs::write(&cert_file, generated.certificate_pem.as_bytes())
                    .map_err(|e| format!("Failed to save certificate: {}", e))?;

                // Save private key PEM (should be encrypted in production)
                let key_file = cert_dir.join(format!("{}.key", e.cert_id));
                std::fs::write(&key_file, generated.private_key_pem.as_bytes())
                    .map_err(|e| format!("Failed to save private key: {}", e))?;

                // Save metadata
                let metadata = serde_json::json!({
                    "cert_id": e.cert_id,
                    "subject": e.subject,
                    "issuer": e.issuer,
                    "not_before": e.not_before,
                    "not_after": e.not_after,
                    "is_ca": e.is_ca,
                    "key_usage": e.key_usage,
                    "fingerprint": generated.fingerprint,
                });

                let metadata_file = cert_dir.join(format!("{}.json", e.cert_id));
                std::fs::write(&metadata_file, serde_json::to_string_pretty(&metadata).unwrap())
                    .map_err(|e| format!("Failed to save certificate metadata: {}", e))?;

                // Update manifest
                projection_write.save_manifest()
                    .map_err(|e| format!("Failed to update manifest: {}", e))?;
            }
        }
    }

    Ok(cert_id)
}

// NATS Hierarchy generation
async fn generate_nats_hierarchy(
    org_name: String,
    projection: Arc<RwLock<OfflineKeyProjection>>,
) -> Result<String, String> {
    use crate::adapters::NscAdapter;
    use crate::ports::nats::NatsKeyPort;

    // Create NSC adapter
    let nsc_adapter = NscAdapter::new("./nsc_store", false); // use_cli = false for native implementation

    // Generate operator for organization
    let operator = nsc_adapter.generate_operator(&org_name)
        .await
        .map_err(|e| format!("Failed to generate operator: {}", e))?;

    // Get organizational units and people from projection
    let proj = projection.read().await;
    let people = proj.get_people().to_vec();

    // For now, create a default "Engineering" account if we have people
    // TODO: Map actual organizational units to accounts
    let mut accounts = Vec::new();
    let mut users = Vec::new();

    if !people.is_empty() {
        // Create a default account
        let account = nsc_adapter.generate_account(&operator.id.to_string(), "Engineering")
            .await
            .map_err(|e| format!("Failed to generate account: {}", e))?;
        accounts.push(account.clone());

        // Generate user for each person
        for person in people.iter() {
            let user = nsc_adapter.generate_user(&account.id.to_string(), &person.name)
                .await
                .map_err(|e| format!("Failed to generate user for {}: {}", person.name, e))?;
            users.push(user);
        }
    }

    // Store the generated hierarchy in projection
    // TODO: Add methods to projection to store NATS hierarchy
    tracing::info!(
        "Generated NATS hierarchy: 1 operator, {} accounts, {} users",
        accounts.len(),
        users.len()
    );

    Ok(operator.id.to_string())
}

// Export NATS hierarchy to NSC store
async fn export_nats_to_nsc(
    export_path: std::path::PathBuf,
    projection: Arc<RwLock<OfflineKeyProjection>>,
) -> Result<String, String> {
    use crate::adapters::NscAdapter;
    use crate::ports::nats::{NatsKeyPort, NatsKeys};

    // Re-generate the hierarchy (in real implementation, we'd load from projection)
    let proj = projection.read().await;
    let org = proj.get_organization();
    let people = proj.get_people().to_vec();

    // Create NSC adapter
    let nsc_adapter = NscAdapter::new(export_path.clone(), false);

    // Generate hierarchy again for export
    let operator = nsc_adapter.generate_operator(&org.name)
        .await
        .map_err(|e| format!("Failed to generate operator: {}", e))?;

    let mut accounts = Vec::new();
    let mut users = Vec::new();

    if !people.is_empty() {
        let account = nsc_adapter.generate_account(&operator.id.to_string(), "Engineering")
            .await
            .map_err(|e| format!("Failed to generate account: {}", e))?;
        accounts.push(account.clone());

        for person in people.iter() {
            let user = nsc_adapter.generate_user(&account.id.to_string(), &person.name)
                .await
                .map_err(|e| format!("Failed to generate user: {}", e))?;
            users.push(user);
        }
    }

    // Build NatsKeys structure
    let nats_keys = NatsKeys {
        operator,
        accounts,
        users,
        signing_keys: Vec::new(),
    };

    // Export to NSC directory structure
    nsc_adapter.export_to_nsc_store(&nats_keys, &export_path)
        .await
        .map_err(|e| format!("Failed to export to NSC: {}", e))?;

    Ok(export_path.display().to_string())
}

async fn export_domain(
    projection: Arc<RwLock<OfflineKeyProjection>>,
) -> Result<String, String> {
    let projection = projection.read().await;
    projection.save_manifest()
        .map(|_| projection.root_path.display().to_string())
        .map_err(|e| e.to_string())
}

/// Run the GUI application
pub async fn run(output_dir: String, config: Option<crate::config::Config>) -> iced::Result {
    application("CIM Keys", CimKeysApp::update, CimKeysApp::view)
        .subscription(|app| app.subscription())
        .theme(|app| app.theme())
        .font(include_bytes!("../assets/fonts/MaterialIcons-Regular.ttf"))
        .run_with(|| CimKeysApp::new(output_dir, config))
}