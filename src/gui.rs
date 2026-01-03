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
    domain::{Person, KeyOwnerRole, Organization},
    domain::ids::{BootstrapOrgId, BootstrapPersonId},
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
    icons::{verified, EMOJI_FONT, FONT_BODY},
    // Kan extension pattern for domain lifting
    lifting::LiftableDomain,
};

pub mod graph;
pub mod graph_pki;
pub mod graph_nats;
pub mod graph_yubikey;
pub mod graph_events;
pub mod folds;

// Aggregate-specific graph views for DDD organization
pub mod graph_person;
pub mod graph_location;
pub mod graph_key;
pub mod graph_relationship;
pub mod graph_organization;
pub mod graph_certificate;
pub mod graph_manifest;

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
pub mod role_palette;
pub mod graph_projection;

#[cfg(test)]
mod graph_integration_tests;

use graph::{OrganizationConcept, OrganizationIntent};
use crate::lifting::Injection;
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
    org_graph: OrganizationConcept,
    // Filtered graphs for different contexts (cached for performance)
    pki_graph: OrganizationConcept,
    nats_graph: OrganizationConcept,
    yubikey_graph: OrganizationConcept,
    location_graph: OrganizationConcept,
    policy_graph: OrganizationConcept,
    aggregates_graph: OrganizationConcept,
    empty_graph: OrganizationConcept,
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

    // CLAN bootstrap credential generation
    clan_bootstrap_path: String,
    clan_bootstrap_loaded: bool,
    clan_credentials_generated: bool,
    clan_nsc_store_path: Option<PathBuf>,
    clan_accounts_exported: usize,
    clan_users_exported: usize,

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

    // Phase 10: Node type selector for SPACE key
    show_node_type_selector: bool,
    node_selector_position: Point,
    selected_menu_index: usize,  // Currently selected menu item (for keyboard navigation)

    // Phase 8: Node/edge type filtering
    filter_show_people: bool,
    filter_show_orgs: bool,
    filter_show_nats: bool,
    filter_show_pki: bool,
    filter_show_yubikey: bool,
    // Phase 9: Graph layout options
    current_layout: GraphLayout,

    // Phase 10: Policy visualization
    policy_data: Option<crate::policy_loader::PolicyBootstrapData>,
    show_role_nodes: bool,  // Toggle between expanded (role nodes) and compact (badges) view

    // Role palette for drag-and-drop assignment
    role_palette: role_palette::RolePalette,

    // Progressive disclosure state for policy graph
    policy_expansion_level: PolicyExpansionLevel,
    expanded_separation_classes: std::collections::HashSet<crate::policy::SeparationClass>,
    expanded_categories: std::collections::HashSet<String>,
}

/// Expansion level for progressive disclosure in policy graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PolicyExpansionLevel {
    /// Show only separation class groups (6 nodes)
    #[default]
    Classes,
    /// Show separation classes + roles (26+ nodes)
    Roles,
    /// Show roles + claims (294+ nodes)
    Claims,
}

/// Export readiness status - tracks what's needed for PKI export
#[derive(Debug, Clone, Default)]
pub struct ExportReadiness {
    /// Core requirements
    pub has_organization: bool,
    pub has_root_ca: bool,
    pub has_people: bool,
    pub has_locations: bool,

    /// Key infrastructure
    pub root_ca_count: usize,
    pub intermediate_ca_count: usize,
    pub leaf_cert_count: usize,
    pub key_count: usize,

    /// NATS infrastructure (optional)
    pub has_nats_operator: bool,
    pub nats_account_count: usize,
    pub nats_user_count: usize,

    /// Policy assignments (optional)
    pub has_policy_data: bool,
    pub role_assignment_count: usize,
    pub people_with_roles: usize,

    /// YubiKey provisioning (optional)
    pub yubikey_count: usize,
    pub provisioned_yubikeys: usize,

    /// Missing items list for display
    pub missing_items: Vec<ExportMissingItem>,
    pub warnings: Vec<String>,
}

/// A missing item required for export
#[derive(Debug, Clone)]
pub struct ExportMissingItem {
    pub category: &'static str,
    pub description: String,
    pub severity: MissingSeverity,
    pub action: Option<String>,
}

/// Severity of missing item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingSeverity {
    /// Export cannot proceed without this
    Required,
    /// Export can proceed but functionality is limited
    Recommended,
    /// Nice to have but not essential
    Optional,
}

impl ExportReadiness {
    /// Check if all required items are present
    pub fn is_ready(&self) -> bool {
        self.has_organization && self.has_root_ca && self.has_people
    }

    /// Get the overall readiness percentage (0-100)
    pub fn readiness_percentage(&self) -> u8 {
        let mut score = 0u8;
        let mut total = 0u8;

        // Required items (weighted heavily)
        total += 30;
        if self.has_organization { score += 10; }
        if self.has_root_ca { score += 10; }
        if self.has_people { score += 10; }

        // Recommended items
        total += 30;
        if self.has_locations { score += 10; }
        if self.intermediate_ca_count > 0 { score += 10; }
        if self.key_count > 0 { score += 10; }

        // Optional items
        total += 40;
        if self.has_nats_operator { score += 10; }
        if self.has_policy_data { score += 10; }
        if self.people_with_roles > 0 { score += 10; }
        if self.provisioned_yubikeys > 0 { score += 10; }

        ((score as f32 / total as f32) * 100.0) as u8
    }

    /// Get count of required items missing
    pub fn required_missing_count(&self) -> usize {
        self.missing_items.iter()
            .filter(|item| matches!(item.severity, MissingSeverity::Required))
            .count()
    }

    /// Get count of recommended items missing
    pub fn recommended_missing_count(&self) -> usize {
        self.missing_items.iter()
            .filter(|item| matches!(item.severity, MissingSeverity::Recommended))
            .count()
    }
}

// NOTE: OrganizationalNodeType has been removed in favor of lifting::Injection
// which provides a proper categorical coproduct with 28 variants.
// Use Injection::creatable() for the subset that can be manually created.

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
    Locations,
    Keys,
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
    /// Policies - Roles, claims, and separation of duties
    Policies,
    /// Aggregate state machines - State diagrams for each aggregate type
    Aggregates,
    /// Command history - CQRS write audit trail
    CommandHistory,
    /// Causality chains - Correlation/causation tracking for distributed tracing
    CausalityChains,
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
    SecretsImported(Result<crate::secrets_loader::BootstrapData, String>),
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
    NatsFromGraphGenerated(Result<Vec<(graph::ConceptEntity, iced::Point, Option<Uuid>)>, String>),
    ExportToNsc,
    NscExported(Result<String, String>),

    // CLAN Bootstrap credential generation
    ClanBootstrapPathChanged(String),
    LoadClanBootstrap,
    ClanBootstrapLoaded(Result<(crate::domain::Organization, Vec<crate::domain::OrganizationUnit>, Vec<crate::domain::Person>), String>),
    GenerateClanCredentials,
    ClanCredentialsGenerated(Result<(PathBuf, usize, usize), String>),  // (nsc_store_path, accounts, users)

    // PKI operations
    PkiCertificatesLoaded(Vec<crate::projections::CertificateEntry>),
    GeneratePkiFromGraph,  // Graph-first PKI generation
    PkiGenerated(Result<Vec<(graph::ConceptEntity, iced::Point, Option<Uuid>)>, String>),
    RootCAGenerated(Result<crate::crypto::x509::X509Certificate, String>),
    PersonalKeysGenerated(Result<(crate::crypto::x509::X509Certificate, Vec<String>), String>), // (cert, nats_keys)

    // YubiKey operations
    YubiKeyDataLoaded(Vec<crate::projections::YubiKeyEntry>, Vec<crate::projections::PersonEntry>),
    ProvisionYubiKeysFromGraph,  // Graph-first YubiKey provisioning
    YubiKeysProvisioned(Result<Vec<(graph::ConceptEntity, iced::Point, iced::Color, String, Uuid)>, String>),

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
    OrganizationIntent(OrganizationIntent),
    CreateContextAwareNode,  // SPACE key: show node type selector
    SelectNodeType(Injection),  // User selects node type from menu
    CancelNodeTypeSelector,  // Escape key cancels selector
    MenuNavigateUp,  // Arrow up: navigate menu selection up
    MenuNavigateDown,  // Arrow down: navigate menu selection down
    MenuAcceptSelection,  // Space/Enter: accept current menu selection
    CycleEdgeType,  // TAB key: cycle selected edge through common types

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

    // Policy visualization
    PolicyDataLoaded(Result<crate::policy_loader::PolicyBootstrapData, String>),
    ToggleRoleDisplay,  // Toggle between expanded (role nodes) and compact (badges) view
    // Progressive disclosure for policy graph
    SetPolicyExpansionLevel(PolicyExpansionLevel),
    ToggleSeparationClassExpansion(crate::policy::SeparationClass),
    ToggleCategoryExpansion(String),

    // Role palette messages
    RolePaletteMessage(role_palette::RolePaletteMessage),
    ToggleRolePalette,  // Toggle palette visibility

    // Role drag-and-drop subscription messages
    RoleDragMove(Point),   // Mouse moved during drag
    RoleDragDrop,          // Mouse button released (drop)
    RoleDragCancel,        // Escape pressed (cancel)
}

impl Message {
    /// Convert a Message to an Intent if applicable
    ///
    /// This method provides a bridge from the Iced Message type to the
    /// MVI Intent type. Messages that have direct Intent equivalents are
    /// converted; Iced-specific messages return None.
    ///
    /// ## Intent Routing Categories
    ///
    /// Messages are categorized by their origin and handling:
    ///
    /// 1. **Ui* Intents**: User interface interactions (button clicks, form inputs)
    /// 2. **Domain* Intents**: Domain events from aggregates
    /// 3. **Port* Intents**: Async responses from hexagonal ports
    /// 4. **System* Intents**: System-level events (file picker, clipboard)
    /// 5. **Iced-specific**: Component wrappers, animations (remain as Message)
    pub fn to_intent(&self) -> Option<Intent> {
        match self {
            // === Direct Ui* Intent mappings ===
            Message::TabSelected(tab) => {
                // Convert gui::Tab to mvi::model::Tab
                let mvi_tab = match tab {
                    Tab::Welcome => crate::mvi::model::Tab::Welcome,
                    Tab::Organization => crate::mvi::model::Tab::Organization,
                    Tab::Keys => crate::mvi::model::Tab::Keys,
                    Tab::Export => crate::mvi::model::Tab::Export,
                    Tab::Locations => return None, // No equivalent in mvi::model::Tab
                };
                Some(Intent::UiTabSelected(mvi_tab))
            }
            Message::CreateNewDomain => Some(Intent::UiCreateDomainClicked),
            Message::OrganizationNameChanged(name) => {
                Some(Intent::UiOrganizationNameChanged(name.clone()))
            }
            Message::MasterPassphraseChanged(pass) => {
                Some(Intent::UiPassphraseChanged(pass.clone()))
            }
            Message::MasterPassphraseConfirmChanged(pass) => {
                Some(Intent::UiPassphraseConfirmChanged(pass.clone()))
            }
            Message::AddPerson => Some(Intent::UiAddPersonClicked),
            Message::GenerateRootCA => Some(Intent::UiGenerateRootCAClicked),
            Message::GenerateSSHKeys => Some(Intent::UiGenerateSSHKeysClicked),
            Message::GenerateAllKeys => Some(Intent::UiGenerateAllKeysClicked),

            // === Already an Intent - unwrap ===
            Message::MviIntent(intent) => Some(intent.clone()),

            // === Iced-specific - remain as Message ===
            // Component wrappers
            Message::OrganizationIntent(_)
            | Message::ContextMenuMessage(_)
            | Message::PropertyCardMessage(_)
            | Message::PassphraseDialogMessage(_)
            | Message::RolePaletteMessage(_) => None,

            // Animations and subscriptions
            Message::AnimationTick
            | Message::RoleDragMove(_)
            | Message::RoleDragDrop
            | Message::RoleDragCancel => None,

            // All other messages don't have Intent equivalents yet
            _ => None,
        }
    }

    /// Check if this message can be converted to an Intent
    #[inline]
    pub fn has_intent_equivalent(&self) -> bool {
        self.to_intent().is_some()
    }

    /// Get the origin category of this message
    ///
    /// Returns a string describing the origin for debugging and logging.
    pub fn origin_category(&self) -> &'static str {
        match self {
            // UI-originated messages
            Message::TabSelected(_)
            | Message::CreateNewDomain
            | Message::LoadExistingDomain
            | Message::OrganizationNameChanged(_)
            | Message::MasterPassphraseChanged(_)
            | Message::AddPerson
            | Message::GenerateRootCA
            | Message::GenerateSSHKeys
            | Message::GenerateAllKeys
            | Message::ToggleHelp
            | Message::IncreaseScale
            | Message::DecreaseScale
            | Message::ResetScale
            | Message::SearchQueryChanged(_)
            | Message::ClearSearch => "Ui",

            // Port-originated messages (async results)
            Message::DomainCreated(_)
            | Message::DomainLoaded(_)
            | Message::SecretsImported(_)
            | Message::YubiKeysDetected(_)
            | Message::YubiKeyProvisioned(_)
            | Message::DomainExported(_)
            | Message::KeysGenerated(_)
            | Message::NatsHierarchyGenerated(_)
            | Message::PolicyDataLoaded(_)
            | Message::GraphExported(_)
            | Message::GraphImported(_) => "Port",

            // Component messages (delegated to sub-components)
            Message::OrganizationIntent(_)
            | Message::ContextMenuMessage(_)
            | Message::PropertyCardMessage(_)
            | Message::PassphraseDialogMessage(_)
            | Message::RolePaletteMessage(_) => "Component",

            // System messages
            Message::AnimationTick => "System",

            // MVI bridge
            Message::MviIntent(_) => "MviIntent",

            // Default for unclassified
            _ => "Other",
        }
    }
}

/// Bootstrap configuration - supports both full and simplified formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BootstrapConfig {
    /// Full format with complete Organization struct (from domain-bootstrap.json)
    Full(FullBootstrapConfig),
    /// Simplified format with basic info
    Simple(SimpleBootstrapConfig),
}

/// Full bootstrap config matching domain-bootstrap.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullBootstrapConfig {
    pub organization: Organization,
    #[serde(default)]
    pub people: Vec<Person>,
    #[serde(default)]
    pub locations: Vec<serde_json::Value>,
    #[serde(default)]
    pub yubikey_assignments: Vec<YubiKeyAssignmentFull>,
    #[serde(default)]
    pub nats_hierarchy: Option<NatsHierarchyFull>,
}

/// Simplified bootstrap config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBootstrapConfig {
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
    #[serde(default)]
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
pub struct YubiKeyAssignmentFull {
    pub serial: String,
    pub name: String,
    pub person_id: Uuid,
    pub role: String,
    #[serde(default)]
    pub pin: Option<String>,
    #[serde(default)]
    pub puk: Option<String>,
    #[serde(default)]
    pub management_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsHierarchy {
    pub operator_name: String,
    pub accounts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsHierarchyFull {
    pub operator: NatsOperatorConfig,
    #[serde(default)]
    pub accounts: Vec<NatsAccountConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorConfig {
    pub name: String,
    #[serde(default)]
    pub signing_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountConfig {
    pub name: String,
    pub unit_id: Uuid,
    #[serde(default)]
    pub users: Vec<NatsUserConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserConfig {
    pub person_id: Uuid,
    #[serde(default)]
    pub permissions: Option<serde_json::Value>,
}

/// Serializable graph export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExport {
    pub version: String,
    pub exported_at: String,
    pub graph_view: String,
    pub nodes: Vec<ConceptEntityExport>,
    pub edges: Vec<ConceptRelationExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEntityExport {
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
pub struct ConceptRelationExport {
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
                org_graph: OrganizationConcept::new(),
                // Initialize filtered graphs (empty initially)
                pki_graph: OrganizationConcept::new(),
                nats_graph: OrganizationConcept::new(),
                yubikey_graph: OrganizationConcept::new(),
                location_graph: OrganizationConcept::new(),
                policy_graph: OrganizationConcept::new(),
                aggregates_graph: OrganizationConcept::new(),
                empty_graph: OrganizationConcept::new(),
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
                // CLAN bootstrap
                clan_bootstrap_path: String::from("examples/clan-bootstrap.json"),
                clan_bootstrap_loaded: false,
                clan_credentials_generated: false,
                clan_nsc_store_path: None,
                clan_accounts_exported: 0,
                clan_users_exported: 0,
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
                // Phase 10: Node type selector
                show_node_type_selector: false,
                node_selector_position: Point::new(400.0, 300.0),
                selected_menu_index: 0,  // Default to first item
                // Phase 8: Node/edge type filtering
                filter_show_people: true,
                filter_show_orgs: true,
                filter_show_nats: true,
                filter_show_pki: true,
                filter_show_yubikey: true,
                // Phase 9: Graph layout options
                current_layout: GraphLayout::Manual,
                // Phase 10: Policy visualization
                policy_data: None,
                show_role_nodes: false,  // Default to compact badge view

                // Role palette for drag-and-drop assignment
                role_palette: role_palette::RolePalette::new(),

                // Progressive disclosure state - start at collapsed (Classes) level
                policy_expansion_level: PolicyExpansionLevel::default(),
                expanded_separation_classes: std::collections::HashSet::new(),
                expanded_categories: std::collections::HashSet::new(),
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
                    Tab::Locations => format!("Manage Locations ({} loaded)", self.loaded_locations.len()),
                    Tab::Keys => "Generate and Manage Cryptographic Keys".to_string(),
                    Tab::Export => "Export Domain Configuration".to_string(),
                };
                Task::none()
            }

            Message::GraphViewSelected(view) => {
                // Simple: just change the context
                self.graph_view = view;

                // Populate aggregates graph when switching to Aggregates view
                if view == GraphView::Aggregates {
                    self.populate_aggregates_graph();
                }

                // Context changed - space menu will update automatically based on self.graph_view
                // Graph rendering will show the appropriate graph for this context
                self.status_message = format!("Graph Context: {:?}", view);

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
                // Try domain-bootstrap.json first (hierarchical), fall back to cowboyai.json + secrets.json
                Task::perform(
                    async move {
                        use crate::secrets_loader::{SecretsLoader, BootstrapData};
                        use std::path::PathBuf;

                        // Try domain-bootstrap.json first (recommended by cim-expert)
                        let bootstrap_path = PathBuf::from("secrets/domain-bootstrap.json");
                        if bootstrap_path.exists() {
                            match SecretsLoader::load_from_bootstrap_file(&bootstrap_path) {
                                Ok(data) => return Ok(data),
                                Err(e) => {
                                    // Log but continue to fallback
                                    eprintln!("Warning: Could not load domain-bootstrap.json: {}", e);
                                }
                            }
                        }

                        // Fallback to legacy format
                        let secrets_path = PathBuf::from("secrets/secrets.json");
                        let cowboyai_path = PathBuf::from("secrets/cowboyai.json");

                        if !secrets_path.exists() || !cowboyai_path.exists() {
                            return Err("Secrets files not found. Please ensure secrets/domain-bootstrap.json or (secrets/secrets.json and secrets/cowboyai.json) exist.".to_string());
                        }

                        match SecretsLoader::load_from_files(&secrets_path, &cowboyai_path) {
                            Ok((org, people, yubikey_configs, passphrase)) => {
                                // Convert to BootstrapData for consistency
                                let units = org.units.clone();
                                Ok(BootstrapData {
                                    organization: org,
                                    units,
                                    people,
                                    yubikey_configs,
                                    yubikey_assignments: vec![],
                                    nats_hierarchy: None,
                                    master_passphrase: passphrase,
                                })
                            }
                            Err(e) => Err(format!("Failed to load secrets: {}", e)),
                        }
                    },
                    Message::SecretsImported
                )
            }

            Message::SecretsImported(result) => {
                match result {
                    Ok(data) => {
                        let org = data.organization;
                        let units = data.units;
                        let people = data.people;
                        let yubikey_configs = data.yubikey_configs;
                        let yubikey_assignments = data.yubikey_assignments;
                        let nats_hierarchy = data.nats_hierarchy;
                        let master_passphrase = data.master_passphrase;

                        // Set organization info
                        self.organization_name = org.name.clone();
                        self.organization_domain = org.display_name.clone();
                        self.organization_id = Some(org.id.as_uuid());

                        // Set master passphrase if provided
                        if let Some(passphrase) = master_passphrase {
                            self.master_passphrase = passphrase.clone();
                            self.master_passphrase_confirm = passphrase;
                        }

                        // === BUILD ORGANIZATION GRAPH ===
                        self.org_graph.clear();

                        // 1. Add Organization node first (root of the graph)
                        self.org_graph.add_organization_node(org.clone());

                        // 2. Add OrganizationUnit nodes
                        for unit in &units {
                            self.org_graph.add_org_unit_node(unit.clone());
                        }

                        // 3. Add Person nodes
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

                        // 4. Add edges
                        // 4a. Organization → OrganizationUnit edges (ParentChild)
                        for unit in &units {
                            self.org_graph.add_edge(
                                org.id.as_uuid(),
                                unit.id.as_uuid(),
                                crate::gui::graph::EdgeType::ParentChild,
                            );

                            // Unit → Parent Unit edges (if nested units)
                            if let Some(parent_id) = unit.parent_unit_id {
                                self.org_graph.add_edge(
                                    parent_id.as_uuid(),
                                    unit.id.as_uuid(),
                                    crate::gui::graph::EdgeType::ParentChild,
                                );
                            }
                        }

                        // 4b. Person → OrganizationUnit edges (MemberOf)
                        for person in &people {
                            for unit_id in &person.unit_ids {
                                self.org_graph.add_edge(
                                    person.id.as_uuid(),
                                    unit_id.as_uuid(),
                                    crate::gui::graph::EdgeType::MemberOf,
                                );
                            }
                        }

                        // 4c. Owner → Service edges (Manages) for service accounts
                        for person in &people {
                            if let Some(owner_id) = person.owner_id {
                                self.org_graph.add_edge(
                                    owner_id.as_uuid(),
                                    person.id.as_uuid(),
                                    crate::gui::graph::EdgeType::Manages,
                                );
                            }
                        }

                        // 5. Apply hierarchical layout (Org at top, Units middle, People bottom)
                        self.org_graph.layout_hierarchical();

                        // === BUILD NATS GRAPH ===
                        self.nats_graph.clear();
                        if let Some(nats) = &nats_hierarchy {
                            use std::collections::HashMap;

                            // Parse organization_id from string
                            let org_uuid = Uuid::parse_str(&nats.operator.organization_id).ok();

                            // Create operator node
                            let operator_id = Uuid::now_v7();
                            self.nats_graph.add_nats_operator_simple(
                                operator_id,
                                nats.operator.name.clone(),
                                org_uuid,
                            );

                            // Create account nodes and track IDs
                            let mut account_ids: HashMap<String, Uuid> = HashMap::new();
                            for account in &nats.accounts {
                                let account_id = Uuid::now_v7();
                                account_ids.insert(account.name.clone(), account_id);

                                // Parse unit_id from string if present
                                let unit_uuid = account.unit_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

                                self.nats_graph.add_nats_account_simple(
                                    account_id,
                                    account.name.clone(),
                                    unit_uuid,
                                    account.is_system,
                                );

                                // Operator → Account edge
                                self.nats_graph.add_edge(
                                    operator_id,
                                    account_id,
                                    crate::gui::graph::EdgeType::ParentChild,
                                );
                            }

                            // Create user nodes
                            for user in &nats.users {
                                let user_id = Uuid::now_v7();
                                let parent_account_id = account_ids.get(&user.account).copied();

                                // Parse person_id from string if present
                                let person_uuid = user.person_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

                                self.nats_graph.add_nats_user_simple(
                                    user_id,
                                    user.name.clone(),
                                    person_uuid,
                                    user.account.clone(),
                                );

                                // Account → User edge
                                if let Some(account_id) = parent_account_id {
                                    self.nats_graph.add_edge(
                                        account_id,
                                        user_id,
                                        crate::gui::graph::EdgeType::ParentChild,
                                    );
                                }
                            }

                            self.nats_graph.layout_nats_hierarchical();
                        }

                        // === BUILD YUBIKEY GRAPH ===
                        self.yubikey_graph.clear();
                        for assignment in &yubikey_assignments {
                            let yubikey_id = Uuid::now_v7();

                            // Add YubiKey node
                            self.yubikey_graph.add_yubikey_node(
                                yubikey_id,
                                assignment.serial.clone(),
                                "5.x".to_string(),
                                Some(chrono::Utc::now()),
                                vec!["9A".to_string(), "9C".to_string(), "9D".to_string(), "9E".to_string()],
                            );

                            // Add PIV slot nodes
                            let slots = [
                                ("9A", "Authentication"),
                                ("9C", "Digital Signature"),
                                ("9D", "Key Management"),
                                ("9E", "Card Authentication"),
                            ];

                            for (slot_num, slot_desc) in &slots {
                                let slot_id = Uuid::now_v7();
                                self.yubikey_graph.add_piv_slot_node(
                                    slot_id,
                                    format!("{} - {}", slot_num, slot_desc),
                                    assignment.serial.clone(),
                                    true, // Assume slot has key for visualization
                                    None,
                                );

                                // YubiKey → Slot edge
                                self.yubikey_graph.add_edge(
                                    yubikey_id,
                                    slot_id,
                                    crate::gui::graph::EdgeType::HasSlot,
                                );
                            }
                        }
                        // Use specialized YubiKey layout for better grouping
                        self.yubikey_graph.layout_yubikey_grouped();

                        // Store YubiKey configs
                        self.yubikey_configs = yubikey_configs.clone();

                        // Count items for status
                        let nats_count = nats_hierarchy.as_ref().map(|n| {
                            1 + n.accounts.len() + n.users.len()
                        }).unwrap_or(0);

                        self.domain_loaded = true;
                        self.active_tab = Tab::Organization;
                        self.status_message = format!(
                            "Imported {} ({}) with {} units, {} people, {} YubiKeys, {} NATS entities",
                            org.display_name,
                            org.name,
                            units.len(),
                            people.len(),
                            yubikey_assignments.len(),
                            nats_count
                        );

                        // Trigger policy loading after domain data is loaded
                        return Task::perform(
                            async {
                                use crate::policy_loader::PolicyLoader;
                                let path = PolicyLoader::default_path();
                                if path.exists() {
                                    PolicyLoader::load_from_file(&path)
                                        .map_err(|e| e.to_string())
                                } else {
                                    Err("Policy bootstrap file not found (secrets/policy-bootstrap.json)".to_string())
                                }
                            },
                            Message::PolicyDataLoaded
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
                        // Clear existing graph
                        self.org_graph = graph::OrganizationConcept::new();

                        match config {
                            BootstrapConfig::Full(full_config) => {
                                // Full format: domain-bootstrap.json style
                                let org = &full_config.organization;

                                // Update organization info
                                self.organization_name = org.name.clone();
                                self.organization_domain = org.display_name.clone();
                                self.organization_id = Some(org.id.as_uuid());

                                // 1. Add Organization node
                                self.org_graph.add_organization_node(org.clone());

                                // 2. Add OrganizationUnit nodes and edges to org
                                for unit in &org.units {
                                    self.org_graph.add_org_unit_node(unit.clone());
                                    // Edge: Organization -> Unit
                                    self.org_graph.add_edge(org.id.as_uuid(), unit.id.as_uuid(), graph::EdgeType::ManagesUnit);
                                }

                                // 3. Add Person nodes and edges to their units
                                for person in &full_config.people {
                                    // Determine role from person's roles
                                    let role = person.roles.first()
                                        .map(|r| match r.role_type {
                                            crate::domain::RoleType::Executive => KeyOwnerRole::RootAuthority,
                                            crate::domain::RoleType::Administrator => KeyOwnerRole::SecurityAdmin,
                                            crate::domain::RoleType::Developer => KeyOwnerRole::Developer,
                                            crate::domain::RoleType::Auditor => KeyOwnerRole::Auditor,
                                            crate::domain::RoleType::Operator => KeyOwnerRole::ServiceAccount,
                                            _ => KeyOwnerRole::Developer,
                                        })
                                        .unwrap_or(KeyOwnerRole::Developer);

                                    self.org_graph.add_node(person.clone(), role);

                                    // Edge: Unit -> Person (for each unit they belong to)
                                    for unit_id in &person.unit_ids {
                                        self.org_graph.add_edge(unit_id.as_uuid(), person.id.as_uuid(), graph::EdgeType::MemberOf);
                                    }

                                    // If no unit, connect directly to org
                                    if person.unit_ids.is_empty() {
                                        self.org_graph.add_edge(org.id.as_uuid(), person.id.as_uuid(), graph::EdgeType::MemberOf);
                                    }
                                }

                                // Store config for later use
                                self.bootstrap_config = Some(BootstrapConfig::Full(full_config.clone()));

                                let node_count = self.org_graph.node_count();
                                let edge_count = self.org_graph.edges.len();
                                self.status_message = format!(
                                    "Loaded full domain: {} nodes, {} relationships (1 org, {} units, {} people)",
                                    node_count, edge_count,
                                    full_config.organization.units.len(),
                                    full_config.people.len()
                                );
                            }
                            BootstrapConfig::Simple(simple_config) => {
                                // Simple format: basic info only
                                if let Some(ref org_info) = simple_config.organization {
                                    self.organization_name = org_info.name.clone();
                                    self.organization_domain = org_info.display_name.clone();
                                }

                                // Create synthetic Organization node using proper EntityId
                                let org = Organization::new(
                                    self.organization_name.clone(),
                                    self.organization_domain.clone(),
                                );
                                let org_uuid = org.id.as_uuid();
                                self.organization_id = Some(org_uuid);
                                self.org_graph.add_organization_node(org.clone());

                                // Add people
                                for person_config in &simple_config.people {
                                    let person = Person {
                                        id: BootstrapPersonId::from_uuid(person_config.person_id),
                                        name: person_config.name.clone(),
                                        email: person_config.email.clone(),
                                        organization_id: org.id.clone(),
                                        unit_ids: vec![],
                                        roles: vec![],
                                        nats_permissions: None,
                                        active: true,
                                        owner_id: None,
                                    };

                                    let role = match person_config.role.as_str() {
                                        "RootAuthority" => KeyOwnerRole::RootAuthority,
                                        "SecurityAdmin" => KeyOwnerRole::SecurityAdmin,
                                        "Developer" => KeyOwnerRole::Developer,
                                        "ServiceAccount" => KeyOwnerRole::ServiceAccount,
                                        "BackupHolder" => KeyOwnerRole::BackupHolder,
                                        "Auditor" => KeyOwnerRole::Auditor,
                                        _ => KeyOwnerRole::Developer,
                                    };

                                    self.org_graph.add_node(person.clone(), role);
                                    self.org_graph.add_edge(org_uuid, person.id.as_uuid(), graph::EdgeType::MemberOf);
                                }

                                self.bootstrap_config = Some(BootstrapConfig::Simple(simple_config));
                                self.status_message = format!("Loaded {} people from simple configuration", self.org_graph.node_count() - 1);
                            }
                        }

                        self.domain_loaded = true;
                        self.active_tab = Tab::Organization;
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
                        use crate::gui::graph_events::GraphEvent;
                        use chrono::Utc;

                        // Clone old state for compensating event (undo)
                        let old_lifted_node = node.lifted_node.clone();
                        let old_label = node.visualization().primary_text.clone();

                        // Create new lifted node with updated primary text
                        let new_lifted_node = old_lifted_node.clone()
                            .with_primary(self.inline_edit_name.clone());

                        // Create immutable event
                        let event = GraphEvent::NodePropertiesChanged {
                            node_id,
                            old_lifted_node,
                            old_label,
                            new_lifted_node,
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
                if self.show_node_type_selector {
                    // Cancel node type selector
                    self.show_node_type_selector = false;
                    self.status_message = "Node creation cancelled".to_string();
                } else if self.editing_new_node.is_some() {
                    // Cancel inline editing
                    self.editing_new_node = None;
                    self.inline_edit_name.clear();
                    self.status_message = "Edit cancelled".to_string();
                } else {
                    // No inline edit active - cancel edge creation instead
                    self.org_graph.handle_message(crate::gui::graph::OrganizationIntent::CancelEdgeCreation);
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
                let org_uuid = match self.organization_id {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Please create a domain first".to_string());
                        return Task::none();
                    }
                };

                let person = Person {
                    id: BootstrapPersonId::new(),
                    name: self.new_person_name.clone(),
                    email: self.new_person_email.clone(),
                    organization_id: BootstrapOrgId::from_uuid(org_uuid),
                    unit_ids: vec![],
                    roles: vec![],
                    nats_permissions: None,
                    active: true,
                    owner_id: None,
                };

                let role = self.new_person_role.unwrap();

                // Add to graph for visualization
                self.org_graph.add_node(person.clone(), role);

                // Persist to projection - capture ids as Uuid for projection interface
                let projection = self.projection.clone();
                let person_id = person.id.as_uuid();
                let org_id = person.organization_id.as_uuid();
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
                            .filter(|(_, node)| node.injection().is_certificate())
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

            Message::RootCAGenerated(result) => {
                match result {
                    Ok(certificate) => {
                        // Create Root CA node in graph using LiftableDomain
                        use crate::domain::pki::{Certificate as PkiCertificate, CertificateId};
                        use crate::lifting::LiftableDomain;

                        let cert_id = CertificateId::new();
                        let subject = format!("CN={} Root CA, O={}", self.organization_name, self.organization_name);
                        let cert = PkiCertificate::root(
                            cert_id,
                            subject.clone(),
                            chrono::Utc::now(),
                            chrono::Utc::now() + chrono::Duration::days(365 * 20), // 20 years
                            vec!["keyCertSign".to_string(), "cRLSign".to_string()],
                        );
                        let lifted_node = cert.lift();
                        let position = iced::Point::new(400.0, 100.0); // Center top of graph
                        let root_ca_node = graph::ConceptEntity::from_lifted_node(lifted_node);
                        let cert_id = cert_id.as_uuid();

                        // Create view with custom color and label
                        let custom_color = self.view_model.colors.cert_root_ca;
                        let custom_label = format!("{} Root CA", self.organization_name);
                        let view = view_model::NodeView::new(cert_id, position, custom_color, custom_label);

                        // Add to graph
                        self.org_graph.nodes.insert(cert_id, root_ca_node);
                        self.org_graph.node_views.insert(cert_id, view);

                        // Switch to PKI view to show the new certificate
                        self.graph_view = GraphView::PkiTrustChain;

                        // TODO: Store certificate PEM data in projection
                        // TODO: Store private key securely
                        // TODO: Emit CertificateGeneratedEvent

                        self.status_message = format!("✅ Root CA generated! Fingerprint: {} | View in PKI Graph", &certificate.fingerprint[..16]);
                        tracing::info!("Root CA generated and added to graph: {}", certificate.fingerprint);
                    }
                    Err(e) => {
                        self.status_message = format!("❌ Root CA generation failed: {}", e);
                        self.error_message = Some(e);
                        tracing::error!("Root CA generation failed");
                    }
                }
                Task::none()
            }

            Message::PersonalKeysGenerated(result) => {
                match result {
                    Ok((certificate, nats_keys)) => {
                        // Create Leaf Certificate node in graph using LiftableDomain
                        use crate::domain::pki::{Certificate as PkiCertificate, CertificateId};
                        use crate::lifting::LiftableDomain;

                        let cert_id = CertificateId::new();
                        let subject = certificate.certificate_pem.lines().nth(0).unwrap_or("Personal Cert").to_string();
                        let cert = PkiCertificate::leaf(
                            cert_id,
                            subject,
                            "Self-Signed (Temporary)".to_string(),
                            chrono::Utc::now(),
                            chrono::Utc::now() + chrono::Duration::days(365),
                            vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
                            vec![], // san
                        );
                        let lifted_node = cert.lift();
                        let position = iced::Point::new(400.0, 300.0); // Below Root CA
                        let leaf_cert_node = graph::ConceptEntity::from_lifted_node(lifted_node);
                        let cert_id = cert_id.as_uuid();

                        // Create view with custom color and label
                        let custom_color = self.view_model.colors.cert_leaf;
                        let custom_label = "Personal Certificate".to_string();
                        let view = view_model::NodeView::new(cert_id, position, custom_color, custom_label);

                        // Add to graph
                        self.org_graph.nodes.insert(cert_id, leaf_cert_node);
                        self.org_graph.node_views.insert(cert_id, view);

                        // Switch to PKI view
                        self.graph_view = GraphView::PkiTrustChain;

                        // TODO: Create NATS identity nodes
                        // TODO: Store keys in projection

                        self.status_message = format!("✅ Personal keys generated! {} NATS keys + Certificate | View in PKI Graph", nats_keys.len());
                        tracing::info!("Personal keys generated: cert + {} NATS keys", nats_keys.len());
                    }
                    Err(e) => {
                        self.status_message = format!("❌ Personal keys generation failed: {}", e);
                        self.error_message = Some(e);
                        tracing::error!("Personal keys generation failed");
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
                            .filter(|(_, node)| node.injection().is_nats())
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

                        // Count nodes by checking injection type
                        let count = self.org_graph.nodes.iter()
                            .filter(|(_, node)| node.injection().is_yubikey_status())
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
                        let org_uuid = self.organization_id.unwrap_or_else(|| Uuid::now_v7());
                        let org_name = self.organization_name.clone();
                        let org_domain = self.organization_domain.clone();
                        let projection = self.projection.clone();

                        return Task::perform(
                            async move {
                                let proj = projection.read().await;
                                let people_info = proj.get_people();

                                // Construct domain Organization for NATS projection
                                use crate::domain::{Organization, OrganizationUnit, OrganizationUnitType, Person, PersonRole, RoleType, RoleScope, Permission};
                                use crate::domain::ids::{BootstrapOrgId, BootstrapPersonId, UnitId};
                                use std::collections::HashMap;

                                let org_entity_id = BootstrapOrgId::from_uuid(org_uuid);
                                let unit_entity_id = UnitId::from_uuid(org_uuid);
                                let org = Organization {
                                    id: org_entity_id.clone(),
                                    name: org_name.clone(),
                                    display_name: org_name.clone(),
                                    description: Some(format!("Organization for {}", org_domain)),
                                    parent_id: None,
                                    units: vec![
                                        OrganizationUnit {
                                            id: unit_entity_id.clone(),  // Use EntityId for unit
                                            name: format!("{} - Default", org_name),
                                            unit_type: OrganizationUnitType::Infrastructure,
                                            parent_unit_id: None,
                                            nats_account_name: None,
                                            responsible_person_id: None,
                                        }
                                    ],
                                    metadata: HashMap::new(),
                                };

                                // Convert PersonEntry to domain Person
                                let people: Vec<Person> = people_info.iter().map(|p| Person {
                                    id: BootstrapPersonId::from_uuid(p.person_id),
                                    name: p.name.clone(),
                                    email: p.email.clone(),
                                    roles: vec![PersonRole {
                                        role_type: RoleType::Operator,  // Default role for visualization
                                        scope: RoleScope::Organization,
                                        permissions: vec![Permission::ViewAuditLogs],
                                    }],
                                    organization_id: org_entity_id.clone(),
                                    unit_ids: vec![unit_entity_id.clone()],  // Assign to default unit
                                    nats_permissions: None,
                                    active: true,
                                    owner_id: None,
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

            // CLAN Bootstrap credential generation
            Message::ClanBootstrapPathChanged(path) => {
                self.clan_bootstrap_path = path;
                Task::none()
            }

            Message::LoadClanBootstrap => {
                let path = self.clan_bootstrap_path.clone();
                Task::perform(
                    async move {
                        use crate::clan_bootstrap::ClanBootstrapLoader;
                        let config = ClanBootstrapLoader::load_from_file(&path)
                            .map_err(|e| format!("Failed to load configuration: {}", e))?;
                        ClanBootstrapLoader::to_domain_models(config)
                            .map_err(|e| format!("Failed to convert to domain models: {}", e))
                    },
                    Message::ClanBootstrapLoaded
                )
            }

            Message::ClanBootstrapLoaded(result) => {
                match result {
                    Ok((org, units, people)) => {
                        self.clan_bootstrap_loaded = true;
                        self.status_message = format!("[OK] Loaded CLAN bootstrap: {} ({} units, {} services)",
                            org.name, units.len(), people.len());

                        // Store in org_graph for visualization
                        self.organization_name = org.name.clone();
                        self.organization_domain = org.name.clone();
                        self.organization_id = Some(org.id.as_uuid());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load CLAN bootstrap: {}", e));
                        self.clan_bootstrap_loaded = false;
                    }
                }
                Task::none()
            }

            Message::GenerateClanCredentials => {
                if !self.clan_bootstrap_loaded {
                    self.error_message = Some("Load CLAN bootstrap configuration first".to_string());
                    return Task::none();
                }

                let path = self.clan_bootstrap_path.clone();
                let output_dir = self.export_path.clone();

                Task::perform(
                    async move {
                        use crate::clan_bootstrap::ClanBootstrapLoader;
                        use crate::commands::nsc_export::generate_and_export_credentials;

                        // Load configuration
                        let config = ClanBootstrapLoader::load_from_file(&path)
                            .map_err(|e| format!("Failed to load configuration: {}", e))?;

                        // Convert to domain models
                        let (org, units, people) = ClanBootstrapLoader::to_domain_models(config)
                            .map_err(|e| format!("Failed to convert to domain models: {}", e))?;

                        // Generate and export credentials
                        let result = generate_and_export_credentials(
                            &output_dir,
                            org,
                            units,
                            people,
                        ).map_err(|e| format!("Failed to generate credentials: {}", e))?;

                        Ok((result.nsc_store_path, result.accounts_exported, result.users_exported))
                    },
                    Message::ClanCredentialsGenerated
                )
            }

            Message::ClanCredentialsGenerated(result) => {
                match result {
                    Ok((nsc_store_path, accounts, users)) => {
                        self.clan_credentials_generated = true;
                        self.clan_nsc_store_path = Some(nsc_store_path.clone());
                        self.clan_accounts_exported = accounts;
                        self.clan_users_exported = users;
                        self.status_message = format!("[OK] Generated {} accounts and {} users. NSC store: {}",
                            accounts, users, nsc_store_path.display());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Credential generation failed: {}", e));
                        self.clan_credentials_generated = false;
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
                // Close property card if it's showing (don't stack menu on top of it)
                if self.property_card.is_editing() {
                    self.property_card.clear();
                }

                // ALWAYS position menu at property card location (consistent, predictable)
                self.node_selector_position = Point::new(800.0, 400.0);

                // Reset menu selection to first item
                self.selected_menu_index = 0;

                // Show node type selector menu
                self.show_node_type_selector = true;
                self.status_message = "Select node type to create (↑↓ to navigate, Space/Enter to select)".to_string();
                Task::none()
            }

            Message::CancelNodeTypeSelector => {
                self.show_node_type_selector = false;
                self.status_message = "Node creation cancelled".to_string();
                Task::none()
            }

            Message::MenuNavigateUp => {
                if self.show_node_type_selector {
                    let total_items = Injection::creatable().len();
                    self.selected_menu_index = if self.selected_menu_index == 0 {
                        total_items - 1  // Wrap to bottom
                    } else {
                        self.selected_menu_index - 1
                    };
                }
                Task::none()
            }

            Message::MenuNavigateDown => {
                if self.show_node_type_selector {
                    let total_items = Injection::creatable().len();
                    self.selected_menu_index = (self.selected_menu_index + 1) % total_items;
                }
                Task::none()
            }

            Message::MenuAcceptSelection => {
                if self.show_node_type_selector {
                    // Menu is open - accept selected item
                    let selected_type = Injection::creatable()[self.selected_menu_index];
                    // Close menu and trigger selection
                    self.show_node_type_selector = false;
                    // Trigger SelectNodeType message with selected type
                    return self.update(Message::SelectNodeType(selected_type));
                } else {
                    // Menu is not open - open it (SPACE key)
                    return self.update(Message::CreateContextAwareNode);
                }
            }

            Message::CycleEdgeType => {
                // Cycle the selected edge through common organizational edge types
                use crate::gui::graph::EdgeType;
                use crate::gui::graph_events::GraphEvent;
                use chrono::Utc;

                if let Some(edge_index) = self.org_graph.selected_edge {
                    if edge_index < self.org_graph.edges.len() {
                        let current_edge = &self.org_graph.edges[edge_index];
                        let old_type = current_edge.edge_type.clone();

                        // Define common organizational edge types to cycle through
                        let common_types = vec![
                            EdgeType::MemberOf,
                            EdgeType::Manages,
                            EdgeType::ResponsibleFor,
                            EdgeType::ParentChild,
                            EdgeType::HasRole { valid_from: chrono::Utc::now(), valid_until: None },
                            EdgeType::DefinesRole,
                            EdgeType::DefinesPolicy,
                            EdgeType::Hierarchy,
                        ];

                        // Find current type index, or start at 0 if not in common types
                        let current_index = common_types.iter().position(|t| t == &old_type).unwrap_or(0);
                        let next_index = (current_index + 1) % common_types.len();
                        let new_type = common_types[next_index].clone();

                        // Create EdgeTypeChanged event (FRP event-driven pattern)
                        let event = GraphEvent::EdgeTypeChanged {
                            from: current_edge.from,
                            to: current_edge.to,
                            old_type: old_type.clone(),
                            new_type: new_type.clone(),
                            timestamp: Utc::now(),
                        };

                        // Push event to stack and apply transformation
                        self.org_graph.event_stack.push(event.clone());
                        self.org_graph.apply_event(&event);

                        self.status_message = format!("Edge type changed to {:?}", new_type);
                    } else {
                        self.status_message = "No edge selected to cycle type".to_string();
                    }
                } else {
                    self.status_message = "Select an edge first (click on edge), then press Tab to cycle type".to_string();
                }

                Task::none()
            }

            Message::SelectNodeType(node_type) => {
                self.show_node_type_selector = false;

                use crate::gui::graph_events::GraphEvent;
                use crate::domain::{Organization, OrganizationUnit, OrganizationUnitType, Person, Location, LocationMarker, Address, Role, Policy};
                use cim_domain::EntityId;
                use chrono::Utc;

                // FRP Event-Driven Pattern: Create immutable GraphEvent
                let event = match node_type {
                    Injection::Organization => {
                        let org = Organization::new(
                            "New Organization",
                            "New Organization",
                        );
                        // Add description
                        let org = Organization {
                            description: Some("Edit name by clicking node".to_string()),
                            ..org
                        };

                        let node_id = org.id.as_uuid();
                        let label = org.name.clone();
                        let position = self.node_selector_position;
                        let color = self.view_model.colors.node_organization;

                        self.status_message = "✨ Created Organization - click to edit name".to_string();

                        // Use Kan extension pattern: lift domain type to graph representation
                        let lifted_node = org.lift();

                        GraphEvent::NodeCreated {
                            node_id,
                            lifted_node,
                            position,
                            color,
                            label,
                            timestamp: Utc::now(),
                        }
                    }
                    Injection::OrganizationUnit => {
                        let unit = OrganizationUnit::new(
                            "New Unit",
                            OrganizationUnitType::Department,
                        );

                        let node_id = unit.id.as_uuid();
                        let label = unit.name.clone();
                        let position = self.node_selector_position;
                        let color = self.view_model.colors.node_unit;

                        self.status_message = "✨ Created Organizational Unit - click to edit name".to_string();

                        // Use Kan extension pattern: lift domain type to graph representation
                        let lifted_node = unit.lift();

                        GraphEvent::NodeCreated {
                            node_id,
                            lifted_node,
                            position,
                            color,
                            label,
                            timestamp: Utc::now(),
                        }
                    }
                    Injection::Person => {
                        use crate::domain::ids::{BootstrapOrgId, BootstrapPersonId};

                        // Find org_id from existing Organization nodes
                        let org_id = self.org_graph.nodes.values()
                            .find_map(|n| n.lifted_node.downcast::<Organization>().map(|o| o.id.clone()))
                            .unwrap_or_else(BootstrapOrgId::new);

                        let person = Person {
                            id: BootstrapPersonId::new(),
                            name: "New Person".to_string(),
                            email: "person@example.com".to_string(),
                            roles: Vec::new(),
                            organization_id: org_id,
                            unit_ids: Vec::new(),
                            nats_permissions: None,
                            active: true,
                            owner_id: None,
                        };

                        let node_id = person.id.as_uuid();
                        let label = person.name.clone();
                        let position = self.node_selector_position;
                        let _role = KeyOwnerRole::Developer;
                        let color = self.view_model.colors.node_person;

                        self.status_message = "✨ Created Person - click to edit details".to_string();

                        // Use Kan extension pattern: lift domain type to graph representation
                        let lifted_node = person.lift();

                        GraphEvent::NodeCreated {
                            node_id,
                            lifted_node,
                            position,
                            color,
                            label,
                            timestamp: Utc::now(),
                        }
                    }
                    Injection::Location => {
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

                        let label = location.name.clone();
                        let position = self.node_selector_position;
                        let color = self.view_model.colors.node_location;

                        self.status_message = "✨ Created Location - click to edit address".to_string();

                        // Use Kan extension pattern: lift domain type to graph representation
                        let lifted_node = location.lift();

                        GraphEvent::NodeCreated {
                            node_id,
                            lifted_node,
                            position,
                            color,
                            label,
                            timestamp: Utc::now(),
                        }
                    }
                    Injection::Role => {
                        use crate::domain::ids::{BootstrapOrgId, BootstrapRoleId, BootstrapPersonId};

                        // Find org_id from existing Organization nodes
                        let org_id = self.org_graph.nodes.values()
                            .find_map(|n| n.lifted_node.downcast::<Organization>().map(|o| o.id.clone()))
                            .unwrap_or_else(BootstrapOrgId::new);

                        let creator_id = BootstrapPersonId::new(); // TODO: Get actual user ID

                        let role_data = Role {
                            id: BootstrapRoleId::new(),
                            name: "New Role".to_string(),
                            description: "Define role responsibilities".to_string(),
                            organization_id: org_id,
                            unit_id: None,
                            required_policies: Vec::new(),
                            responsibilities: Vec::new(),
                            created_by: creator_id,
                            active: true,
                        };

                        let node_id = role_data.id.as_uuid();
                        let label = role_data.name.clone();
                        let position = self.node_selector_position;
                        let color = self.view_model.colors.node_role;

                        self.status_message = "✨ Created Role - define responsibilities".to_string();

                        // Use Kan extension pattern: lift domain type to graph representation
                        let lifted_node = role_data.lift();

                        GraphEvent::NodeCreated {
                            node_id,
                            lifted_node,
                            position,
                            color,
                            label,
                            timestamp: Utc::now(),
                        }
                    }
                    Injection::Policy => {
                        use crate::domain::ids::{BootstrapPolicyId, BootstrapPersonId};

                        let creator_id = BootstrapPersonId::new(); // TODO: Get actual user ID

                        let policy = Policy {
                            id: BootstrapPolicyId::new(),
                            name: "New Policy".to_string(),
                            description: "Define policy claims and conditions".to_string(),
                            claims: Vec::new(),
                            conditions: Vec::new(),
                            priority: 0,
                            enabled: true,
                            created_by: creator_id,
                            metadata: std::collections::HashMap::new(),
                        };

                        let node_id = policy.id.as_uuid();
                        let label = policy.name.clone();
                        let position = self.node_selector_position;
                        let color = self.view_model.colors.node_policy;

                        self.status_message = "✨ Created Policy - define claims and conditions".to_string();

                        // Use Kan extension pattern: lift domain type to graph representation
                        let lifted_node = policy.lift();

                        GraphEvent::NodeCreated {
                            node_id,
                            lifted_node,
                            position,
                            color,
                            label,
                            timestamp: Utc::now(),
                        }
                    }
                    // Non-creatable injection types - these are created through other workflows
                    _ => {
                        self.status_message = format!("Cannot manually create {} nodes", node_type.display_name());
                        return Task::none();
                    }
                };

                // Pure FRP: Push immutable event to stack and apply transformation
                self.org_graph.event_stack.push(event.clone());
                self.org_graph.apply_event(&event);

                Task::none()
            }

            // OLD CODE REMOVED: Context-aware hierarchy creation
            // Now using flexible node type selector menu

            // Graph interactions
            Message::OrganizationIntent(graph_msg) => {
                match &graph_msg {
                    OrganizationIntent::NodeClicked(id) => {
                        // Check if we have a role selected for assignment
                        if let Some(ref drag) = self.org_graph.dragging_role.clone() {
                            // Check if clicked node is a Person using downcast
                            let is_person = self.org_graph.nodes.get(id)
                                .map(|n| n.downcast::<Person>().is_some())
                                .unwrap_or(false);

                            if is_person {
                                // Get the role being assigned
                                let role_name = match &drag.source {
                                    graph::DragSource::RoleFromPalette { role_name, .. } => role_name.clone(),
                                    graph::DragSource::RoleFromPerson { role_name, .. } => role_name.clone(),
                                };
                                let separation_class = match &drag.source {
                                    graph::DragSource::RoleFromPalette { separation_class, .. } => separation_class.clone(),
                                    graph::DragSource::RoleFromPerson { separation_class, .. } => separation_class.clone(),
                                };

                                // Check for SoD conflicts
                                let mut sod_conflicts = Vec::new();
                                if let Some(ref policy_data) = self.policy_data {
                                    let current_roles = policy_data.get_roles_for_person(*id);
                                    for current_role in current_roles {
                                        if policy_data.are_roles_incompatible(&role_name, &current_role.name) {
                                            let reason = policy_data.separation_of_duties_rules.iter()
                                                .find(|rule| {
                                                    (rule.role_a == role_name && rule.conflicts_with.contains(&current_role.name)) ||
                                                    (rule.role_a == current_role.name && rule.conflicts_with.contains(&role_name))
                                                })
                                                .and_then(|rule| rule.reason.clone())
                                                .unwrap_or_else(|| "Separation of duties violation".to_string());
                                            sod_conflicts.push((current_role.name.clone(), reason));
                                        }
                                    }
                                }

                                if sod_conflicts.is_empty() {
                                    // No conflicts - assign the role
                                    let badge = graph::RoleBadge {
                                        name: role_name.clone(),
                                        separation_class: separation_class.clone(),
                                        level: 1,
                                    };
                                    let person_badges = self.org_graph.role_badges
                                        .entry(*id)
                                        .or_insert_with(graph::PersonRoleBadges::default);
                                    // Only add if not already assigned
                                    if !person_badges.badges.iter().any(|b| b.name == role_name) {
                                        person_badges.badges.push(badge);
                                    }

                                    self.status_message = format!("Assigned role '{}' to person", role_name);
                                } else {
                                    // Has conflicts - show warning
                                    let conflict_names: Vec<_> = sod_conflicts.iter().map(|(n, _)| n.as_str()).collect();
                                    self.status_message = format!("Cannot assign '{}': conflicts with {}",
                                        role_name, conflict_names.join(", "));
                                }

                                // Clear the selected role
                                self.org_graph.cancel_role_drag();
                                return Task::none();
                            } else {
                                // Clicked on non-Person node - cancel role selection
                                self.org_graph.cancel_role_drag();
                                self.status_message = "Role assignment cancelled - click on a Person node".to_string();
                            }
                        }

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
                                            from_node.visualization().primary_text, to_node.visualization().primary_text, self.org_graph.edges.len());
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

                            // Check policy_graph for policy nodes - clicking selects, NOT expands
                            // (expansion is done via the +/- indicator below the node)
                            if let Some(node) = self.policy_graph.nodes.get(id) {
                                self.policy_graph.selected_node = Some(*id);
                                self.property_card.set_node(*id, node.lifted_node.clone());
                                self.status_message = format!("Selected '{}'", node.visualization().primary_text);
                                return Task::none();
                            }

                            // Phase 4: Open property card when node is clicked (org_graph)
                            if let Some(node) = self.org_graph.nodes.get(id) {
                                self.property_card.set_node(*id, node.lifted_node.clone());
                                self.status_message = format!("Selected '{}' - property card: {}, selected_node: {:?}",
                                    node.visualization().primary_text,
                                    if self.property_card.is_editing() { "open" } else { "closed" },
                                    self.org_graph.selected_node);
                            } else {
                                self.status_message = "Selected node in graph".to_string();
                            }
                        }
                    }
                    OrganizationIntent::ExpandIndicatorClicked(id) => {
                        // Handle +/- indicator click for expandable nodes
                        if let Some(node) = self.policy_graph.nodes.get(id) {
                            // PolicyGroup expansion - use LiftedNode downcast
                            use crate::domain::visualization::{PolicyGroup, PolicyCategory};
                            if let Some(policy_group) = node.lifted_node.downcast::<PolicyGroup>() {
                                let class = policy_group.separation_class;
                                let node_label = node.visualization().primary_text;
                                if self.expanded_separation_classes.contains(&class) {
                                    self.expanded_separation_classes.remove(&class);
                                    self.status_message = format!("Collapsed {} class", node_label);
                                } else {
                                    self.expanded_separation_classes.insert(class);
                                    self.status_message = format!("Expanded {} class - showing roles", node_label);
                                }
                                self.populate_policy_graph();
                                return Task::none();
                            }

                            // PolicyCategory expansion - use LiftedNode downcast
                            if let Some(policy_category) = node.lifted_node.downcast::<PolicyCategory>() {
                                let cat_name = policy_category.name.clone();
                                let node_label = node.visualization().primary_text;
                                if self.expanded_categories.contains(&cat_name) {
                                    self.expanded_categories.remove(&cat_name);
                                    self.status_message = format!("Collapsed {} category", node_label);
                                } else {
                                    self.expanded_categories.insert(cat_name);
                                    self.status_message = format!("Expanded {} category - showing claims", node_label);
                                }
                                self.populate_policy_graph();
                                return Task::none();
                            }
                        }
                    }
                    OrganizationIntent::NodeDragStarted { node_id, offset } => {
                        self.org_graph.dragging_node = Some(*node_id);
                        self.org_graph.drag_offset = *offset;
                        if let Some(view) = self.org_graph.node_views.get(node_id) {
                            self.org_graph.drag_start_position = Some(view.position);
                        }
                    }
                    OrganizationIntent::NodeDragged(cursor_position) => {
                        if let Some(node_id) = self.org_graph.dragging_node {
                            if let Some(view) = self.org_graph.node_views.get_mut(&node_id) {
                                // Update node position based on cursor and drag offset
                                let adjusted_x = (cursor_position.x - self.org_graph.pan_offset.x) / self.org_graph.zoom;
                                let adjusted_y = (cursor_position.y - self.org_graph.pan_offset.y) / self.org_graph.zoom;
                                view.position.x = adjusted_x - self.org_graph.drag_offset.x;
                                view.position.y = adjusted_y - self.org_graph.drag_offset.y;
                            }
                        }
                    }
                    OrganizationIntent::NodeDragEnded => {
                        if let Some(node_id) = self.org_graph.dragging_node {
                            // Check if node actually moved
                            if let (Some(start_pos), Some(view)) = (
                                self.org_graph.drag_start_position,
                                self.org_graph.node_views.get(&node_id)
                            ) {
                                let distance = ((view.position.x - start_pos.x).powi(2)
                                    + (view.position.y - start_pos.y).powi(2)).sqrt();

                                if distance > 5.0 {
                                    // Create NodeMoved event for undo/redo
                                    use crate::gui::graph_events::GraphEvent;
                                    use chrono::Utc;

                                    let event = GraphEvent::NodeMoved {
                                        node_id,
                                        old_position: start_pos,
                                        new_position: view.position,
                                        timestamp: Utc::now(),
                                    };
                                    self.org_graph.event_stack.push(event);
                                    self.status_message = format!("Moved node to ({:.0}, {:.0})",
                                        view.position.x, view.position.y);
                                }
                            }
                        }
                        self.org_graph.dragging_node = None;
                        self.org_graph.drag_start_position = None;
                    }
                    OrganizationIntent::AutoLayout => {
                        self.org_graph.auto_layout();
                        self.status_message = String::from("Graph layout updated");
                    }
                    OrganizationIntent::AddEdge { .. } => {
                        // Handled in graph.handle_message (line 1744)
                        self.status_message = String::from("Relationship added");
                    }
                    OrganizationIntent::EdgeSelected(index) => {
                        if let Some(edge) = self.org_graph.edges.get(*index) {
                            self.property_card.set_edge(*index, edge.from, edge.to, edge.edge_type.clone());
                            self.status_message = format!("Edge selected ({})", *index);
                        }
                    }
                    OrganizationIntent::EdgeDeleted(_index) => {
                        // Handled in graph.handle_message
                        self.property_card.clear();
                        self.status_message = String::from("Edge deleted");
                    }
                    OrganizationIntent::EdgeTypeChanged { .. } => {
                        // Handled in graph.handle_message
                        self.status_message = String::from("Edge type changed");
                    }
                    OrganizationIntent::EdgeCreationStarted(_node_id) => {
                        // Handled in graph.handle_message (starts edge indicator)
                        self.status_message = String::from("Drag to target node to create edge");
                    }
                    // Phase 4: Right-click shows node selector menu (same as SPACE key)
                    OrganizationIntent::RightClick(_position) => {
                        // Close property card if it's showing (don't stack menu on top of it)
                        if self.property_card.is_editing() {
                            self.property_card.clear();
                        }

                        // ALWAYS position menu at property card location (consistent, predictable)
                        self.node_selector_position = Point::new(800.0, 400.0);
                        self.show_node_type_selector = true;
                        self.status_message = "Select node type to create (Esc to cancel)".to_string();
                    }
                    // Phase 4: Update edge indicator position during edge creation
                    OrganizationIntent::CursorMoved(position) => {
                        if self.org_graph.edge_indicator.is_active() {
                            self.org_graph.edge_indicator.update_position(*position);
                        }
                    }
                    // Phase 4: Cancel edge creation with Esc key
                    OrganizationIntent::CancelEdgeCreation => {
                        if self.org_graph.edge_indicator.is_active() {
                            self.status_message = "Edge creation cancelled".to_string();
                        }
                        // Cancellation handled in graph.handle_message
                    }
                    // Phase 4: Delete selected node with Delete key
                    OrganizationIntent::DeleteSelected => {
                        if let Some(node_id) = self.org_graph.selected_node {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                use crate::gui::graph_events::GraphEvent;
                                use chrono::Utc;

                                // Get view data for event snapshot
                                let (position, color, label) = if let Some(view) = self.org_graph.node_views.get(&node_id) {
                                    (view.position, view.color, view.label.clone())
                                } else {
                                    let viz = node.visualization();
                                    (iced::Point::ORIGIN, viz.color, viz.primary_text)
                                };

                                // Create NodeDeleted event with snapshot for redo
                                let event = GraphEvent::NodeDeleted {
                                    node_id,
                                    lifted_node: node.lifted_node.clone(),
                                    position,
                                    color,
                                    label: label.clone(),
                                    timestamp: Utc::now(),
                                };

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
                    OrganizationIntent::Undo => {
                        if let Some(description) = self.org_graph.event_stack.undo_description() {
                            self.status_message = format!("Undo: {}", description);
                        } else {
                            self.status_message = "Nothing to undo".to_string();
                        }
                        // Undo handled in graph.handle_message
                    }
                    // Phase 4: Redo last undone action
                    OrganizationIntent::Redo => {
                        if let Some(description) = self.org_graph.event_stack.redo_description() {
                            self.status_message = format!("Redo: {}", description);
                        } else {
                            self.status_message = "Nothing to redo".to_string();
                        }
                        // Redo handled in graph.handle_message
                    }
                    // Canvas clicked - place new node if node type is selected
                    OrganizationIntent::CanvasClicked(position) => {
                        // If a role is selected, clicking canvas cancels the selection
                        if self.org_graph.dragging_role.is_some() {
                            self.org_graph.cancel_role_drag();
                            self.status_message = "Role selection cancelled".to_string();
                            return Task::none();
                        }

                        if let Some(ref node_type_str) = self.selected_node_type {
                            use crate::domain::{OrganizationUnit, OrganizationUnitType, Person};
                            use crate::gui::graph_events::GraphEvent;
                            use chrono::Utc;


                            use crate::domain::ids::{BootstrapPersonId, BootstrapOrgId, UnitId};
                            let node_id = Uuid::now_v7();
                            let dummy_org_id = BootstrapOrgId::from_uuid(
                                self.organization_id.unwrap_or_else(|| Uuid::now_v7())
                            );

                            // Create node based on selected type using Kan extension pattern
                            let (lifted_node, label, color) = match node_type_str.as_str() {
                                // Organization graph nodes
                                "Person" => {
                                    let person = Person {
                                        id: BootstrapPersonId::from_uuid(node_id),
                                        name: "New Person".to_string(),
                                        email: format!("person{}@example.com", node_id),
                                        roles: vec![],
                                        organization_id: dummy_org_id.clone(),
                                        unit_ids: vec![],
                                        nats_permissions: None,
                                        active: true,
                                        owner_id: None,
                                    };
                                    (person.lift(), "New Person".to_string(), self.view_model.colors.node_person)
                                }
                                "Unit" => {
                                    let unit = OrganizationUnit {
                                        id: UnitId::from_uuid(node_id),
                                        name: "New Unit".to_string(),
                                        unit_type: OrganizationUnitType::Department,
                                        parent_unit_id: None,
                                        nats_account_name: None,
                                        responsible_person_id: None,
                                    };
                                    (unit.lift(), "New Unit".to_string(), self.view_model.colors.node_unit)
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
                                    (location.lift(), "New Location".to_string(), self.view_model.colors.node_location)
                                }
                                // TODO: Role - complex type, implement later
                                // TODO: Add NATS, PKI, and YubiKey node types
                                _ => {
                                    self.status_message = format!("Node type '{}' not yet implemented", node_type_str);
                                    self.selected_node_type = None;
                                    return Task::none();
                                }
                            };

                            // Create NodeCreated event using lifted node
                            let event = GraphEvent::NodeCreated {
                                node_id,
                                lifted_node,
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

                                // Use lifted_node downcast to get Person from newly created node
                                if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                    if let Some(person) = node.lifted_node.downcast::<Person>() {
                                        // Create domain event (using new EntityId for demonstration)
                                        // In production, would properly convert person.id to EntityId
                                        let domain_event = PersonEvent::PersonCreated(PersonCreated {
                                            person_id: EntityId::new(),
                                            name: PersonName::new(person.name.clone(), "".to_string()),
                                            source: "gui".to_string(),
                                            created_at: chrono::Utc::now(),
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
                // Route to the active graph based on current view
                self.active_graph_mut().handle_message(graph_msg);
                Task::none()
            }

            // Phase 4: Context Menu interactions
            Message::ContextMenuMessage(menu_msg) => {
                use crate::mvi::intent::NodeCreationType;
                use crate::domain::{Organization, OrganizationUnit, OrganizationUnitType, Location, Role, Policy};
                use crate::gui::graph_events::GraphEvent;
                use crate::lifting::LiftableDomain;
                use chrono::Utc;
                use std::collections::HashMap;

                match menu_msg {
                    ContextMenuMessage::CreateNode(node_type) => {
                        use crate::domain::ids::{BootstrapOrgId, BootstrapPersonId, BootstrapRoleId, BootstrapPolicyId};
                        let position = self.context_menu.position();
                        let node_id = Uuid::now_v7();
                        let dummy_org_id = BootstrapOrgId::from_uuid(
                            self.organization_id.unwrap_or_else(|| Uuid::now_v7())
                        );

                        // Create placeholder domain entity and generate event
                        // Uses LiftableDomain::lift() for type-erased Kan extension
                        let (lifted_node, label, color) = match node_type {
                            NodeCreationType::Organization => {
                                let org = Organization::new(
                                    "New Organization",
                                    "New Organization",
                                );
                                let label = org.name.clone();
                                (org.lift(), label, self.view_model.colors.node_organization)
                            }
                            NodeCreationType::OrganizationalUnit => {
                                let unit = OrganizationUnit::new(
                                    "New Unit",
                                    OrganizationUnitType::Department,
                                );
                                let label = unit.name.clone();
                                (unit.lift(), label, self.view_model.colors.node_unit)
                            }
                            NodeCreationType::Person => {
                                let person = Person {
                                    id: BootstrapPersonId::from_uuid(node_id),
                                    name: "New Person".to_string(),
                                    email: "person@example.com".to_string(),
                                    roles: vec![],
                                    organization_id: dummy_org_id.clone(),
                                    unit_ids: vec![],
                                    nats_permissions: None,
                                    active: true,
                                    owner_id: None,
                                };
                                let label = person.name.clone();
                                (person.lift(), label, self.view_model.colors.node_person)
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
                                (location.lift(), label, self.view_model.colors.node_location)
                            }
                            NodeCreationType::Role => {
                                let role_data = Role {
                                    id: BootstrapRoleId::from_uuid(node_id),
                                    name: "New Role".to_string(),
                                    description: "Role description".to_string(),
                                    organization_id: dummy_org_id.clone(),
                                    unit_id: None,
                                    required_policies: vec![],
                                    responsibilities: vec![],
                                    created_by: BootstrapPersonId::from_uuid(node_id), // TODO: Get actual user
                                    active: true,
                                };
                                let label = role_data.name.clone();
                                (role_data.lift(), label, self.view_model.colors.node_role)
                            }
                            NodeCreationType::Policy => {
                                let policy = Policy {
                                    id: BootstrapPolicyId::from_uuid(node_id),
                                    name: "New Policy".to_string(),
                                    description: "Policy description".to_string(),
                                    claims: vec![],
                                    conditions: vec![],
                                    priority: 0,
                                    enabled: true,
                                    created_by: BootstrapPersonId::from_uuid(node_id), // TODO: Get actual user
                                    metadata: HashMap::new(),
                                };
                                let label = policy.name.clone();
                                (policy.lift(), label, self.view_model.colors.orange_warning)
                            }
                        };

                        // Create and apply NodeCreated event
                        let event = GraphEvent::NodeCreated {
                            node_id,
                            lifted_node,
                            position,
                            color,
                            label,
                            timestamp: Utc::now(),
                        };

                        self.org_graph.event_stack.push(event.clone());
                        self.org_graph.apply_event(&event);

                        // Mark domain as loaded when first organization is created
                        // Use lifted_node downcast to get Organization data
                        if let Some(node) = self.org_graph.nodes.get(&node_id) {
                            if let Some(org) = node.lifted_node.downcast::<Organization>() {
                                if !self.domain_loaded {
                                    self.domain_loaded = true;
                                    self.organization_name = org.display_name.clone();
                                    self.organization_id = Some(org.id.as_uuid());
                                    self.status_message = format!("Created organization: {}", org.display_name);
                                }
                            }
                        }

                        // Emit domain event and project to cim-graph (demonstration)
                        #[cfg(feature = "policy")]
                        {
                            use super::gui::graph_events::GraphEvent as GuiGraphEvent;
                            if let GuiGraphEvent::NodeCreated { lifted_node, .. } = &event {
                                // Use LiftedNode::downcast to extract data for domain event projection
                                if let Some(org) = lifted_node.downcast::<Organization>() {
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
                                                    tracing::info!("📡 NATS publishing enabled - events would be published to {}", crate::config::NATS_URL);
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
                                } else if let Some(person) = lifted_node.downcast::<Person>() {
                                    use cim_domain_person::events::{PersonEvent, PersonCreated};
                                    use cim_domain_person::value_objects::PersonName;
                                    use cim_domain::EntityId;

                                    let domain_event = PersonEvent::PersonCreated(PersonCreated {
                                        person_id: EntityId::new(),
                                        name: PersonName::new(person.name.clone(), "".to_string()),
                                        source: "gui".to_string(),
                                        created_at: chrono::Utc::now(),
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
                                                    tracing::info!("📡 NATS publishing enabled - events would be published to {}", crate::config::NATS_URL);
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
                                // Other node types don't have domain event demonstrations yet
                            }
                        }

                        // Auto-layout to position the new node and re-align hierarchy
                        self.org_graph.auto_layout();

                        // Open property card for the new node
                        if let Some(node) = self.org_graph.nodes.get(&node_id) {
                            self.property_card.set_node(node_id, node.lifted_node.clone());
                        }

                        self.context_menu.hide();
                        self.status_message = format!("Created {:?} node - edit properties", node_type);
                    }
                    ContextMenuMessage::CreateEdge => {
                        // Start edge creation from the node where context menu was opened
                        let source_node = self.context_menu_node.or(self.selected_person);

                        if let Some(from_id) = source_node {
                            // Get position from view and label from node
                            let from_position = self.org_graph.node_views.get(&from_id).map(|v| v.position);
                            let from_label = self.org_graph.nodes.get(&from_id).map(|n| n.visualization().primary_text);

                            if let (Some(position), Some(label)) = (from_position, from_label) {
                                // Start edge creation indicator
                                self.org_graph.edge_indicator.start(from_id, position);
                                self.status_message = format!("Edge creation mode active - click target node (from: '{}')", label);
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
                                self.org_graph.handle_message(OrganizationIntent::EdgeTypeChanged {
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
                                use crate::lifting::PropertyUpdate;
                                use chrono::Utc;

                                let new_name = self.property_card.name().to_string();
                                let new_description = self.property_card.description().to_string();
                                let new_email = self.property_card.email().to_string();
                                let new_enabled = self.property_card.enabled();

                                // Capture old state using lifted_node (Kan extension pattern)
                                let old_lifted_node = node.lifted_node.clone();
                                let old_label = node.visualization().primary_text;

                                // Create PropertyUpdate with all changed values
                                let update = PropertyUpdate::new()
                                    .with_name(new_name.clone())
                                    .with_description(new_description)
                                    .with_email(new_email)
                                    .with_enabled(new_enabled)
                                    .with_claims(self.property_card.claims());

                                // Apply property updates directly to LiftedNode (non-deprecated path)
                                let new_lifted_node = node.lifted_node.apply_properties(&update);

                                // Create and apply NodePropertiesChanged event
                                let event = GraphEvent::NodePropertiesChanged {
                                    node_id,
                                    old_lifted_node,
                                    old_label,
                                    new_lifted_node,
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
                            self.org_graph.handle_message(OrganizationIntent::EdgeDeleted(edge_index));
                            self.property_card.clear();
                            self.status_message = "Edge deleted".to_string();
                        }
                    }
                    PropertyCardMessage::GenerateRootCA => {
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                // Use lifted_node downcast for data extraction
                                if let Some(person) = node.lifted_node.downcast::<Person>() {
                                    // Show passphrase dialog for Root CA generation
                                    self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::RootCA);
                                    self.status_message = format!("Enter passphrase to generate Root CA for {}", person.name);
                                } else if let Some(org) = node.lifted_node.downcast::<Organization>() {
                                    // Organization generates root CA (top-level authority)
                                    self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::RootCA);
                                    self.status_message = format!("Enter passphrase to generate Root CA for organization '{}'", org.display_name);
                                } else {
                                    self.status_message = "Root CA can only be generated for Organizations or Persons".to_string();
                                }
                            }
                        }
                    }
                    PropertyCardMessage::GeneratePersonalKeys => {
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                // Use lifted_node downcast for person name
                                if let Some(person) = node.lifted_node.downcast::<Person>() {
                                    // Show passphrase dialog for Personal Keys generation
                                    self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::PersonalKeys);
                                    self.status_message = format!("Enter passphrase to generate personal keys for {}", person.name);
                                }
                            }
                        }
                    }
                    PropertyCardMessage::ProvisionYubiKey => {
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                // Use lifted_node downcast for person name
                                if let Some(person) = node.lifted_node.downcast::<Person>() {
                                    // For now, show status message about YubiKey provisioning
                                    // In production, this would:
                                    // 1. Detect connected YubiKey
                                    // 2. Show passphrase dialog
                                    // 3. Provision PIV slots with keys
                                    // 4. Create YubiKey node in graph

                                    self.status_message = format!("✅ YubiKey provisioning simulated for {} (hardware integration optional)", person.name);
                                    tracing::info!("YubiKey provisioning requested for person: {}", person.name);

                                    // TODO: Implement full YubiKey provisioning when hardware available:
                                    // - Show passphrase dialog
                                    // - Detect YubiKey serial number
                                    // - Generate keys in PIV slots (9A, 9C, 9D, 9E)
                                    // - Import certificates to slots
                                    // - Create YubiKey node and edge in graph
                                }
                            }
                        }
                    }
                    PropertyCardMessage::GenerateIntermediateCA => {
                        use crate::domain::OrganizationUnit;
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                // Use lifted_node downcast for data extraction
                                if let Some(org) = node.lifted_node.downcast::<Organization>() {
                                    // Organization can also generate intermediate CAs (not just root)
                                    self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::IntermediateCA);
                                    self.status_message = format!("Enter passphrase to generate Intermediate CA for organization '{}'", org.display_name);
                                } else if let Some(unit) = node.lifted_node.downcast::<OrganizationUnit>() {
                                    // OrganizationalUnit generates intermediate CAs
                                    self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::IntermediateCA);
                                    self.status_message = format!("Enter passphrase to generate Intermediate CA for unit '{}'", unit.name);
                                } else {
                                    self.status_message = "Intermediate CA can only be generated for Organizations or Units".to_string();
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
                            let purpose = self.passphrase_dialog.purpose().clone();
                            self.passphrase_dialog.hide();

                            // Get organization info for seed derivation
                            let org_id = self.organization_id
                                .map(|id| id.to_string())
                                .unwrap_or_else(|| self.organization_domain.clone());

                            let org_name = self.organization_name.clone();

                            // Dispatch based on purpose
                            match purpose {
                                passphrase_dialog::PassphrasePurpose::RootCA => {
                                    self.status_message = "Root CA generation in progress...".to_string();

                                    // Trigger async Root CA generation
                                    return Task::perform(
                                        async move {
                                            use crate::crypto::seed_derivation::derive_master_seed;
                                            use crate::crypto::x509::{generate_root_ca, RootCAParams};

                                            // Derive master seed from passphrase
                                            let seed = derive_master_seed(&passphrase, &org_id)
                                                .map_err(|e| format!("Failed to derive seed: {}", e))?;

                                            // Set up Root CA parameters
                                            let params = RootCAParams {
                                                organization: org_name.clone(),
                                                common_name: format!("{} Root CA", org_name),
                                                country: Some("US".to_string()),
                                                state: None,
                                                locality: None,
                                                validity_years: 20,
                                                pathlen: 1, // Allow one intermediate CA level
                                            };

                                            // Generate Root CA certificate (US-021: with event emission)
                                            let correlation_id = uuid::Uuid::now_v7();
                                            generate_root_ca(&seed, params, correlation_id, None)
                                                .map(|(cert, _event)| cert) // Extract cert, discard event for now
                                        },
                                        Message::RootCAGenerated
                                    );
                                }
                                passphrase_dialog::PassphrasePurpose::PersonalKeys => {
                                    self.status_message = "Personal keys generation in progress...".to_string();

                                    // Get person info from property card - use lifted_node downcast
                                    let person_name = self.property_card.node_id()
                                        .and_then(|id| self.org_graph.nodes.get(&id))
                                        .and_then(|node| node.lifted_node.downcast::<Person>().map(|p| p.name.clone()))
                                        .unwrap_or_else(|| "Unknown".to_string());

                                    // Trigger async Personal Keys generation
                                    return Task::perform(
                                        async move {
                                            use crate::crypto::seed_derivation::derive_master_seed;
                                            use crate::crypto::x509::{generate_root_ca, RootCAParams};

                                            // Derive master seed from passphrase
                                            let seed = derive_master_seed(&passphrase, &org_id)
                                                .map_err(|e| format!("Failed to derive seed: {}", e))?;

                                            // TODO: Generate proper leaf certificate signed by intermediate CA
                                            // For now, generate a temporary self-signed cert as placeholder
                                            let temp_cert_params = RootCAParams {
                                                organization: org_name.clone(),
                                                common_name: format!("{} Personal", person_name),
                                                country: Some("US".to_string()),
                                                state: None,
                                                locality: None,
                                                validity_years: 1,
                                                pathlen: 0, // Personal cert doesn't sign other certs
                                            };
                                            // US-021: Generate with event emission, extract cert
                                            let correlation_id = uuid::Uuid::now_v7();
                                            let (cert, _event) = generate_root_ca(&seed, temp_cert_params, correlation_id, None)?;

                                            // TODO: Generate NATS keys (Operator, Account, User)
                                            // For now, return placeholder keys
                                            let nats_keys = vec![
                                                "NATS Operator Key: (placeholder)".to_string(),
                                                "NATS Account Key: (placeholder)".to_string(),
                                                "NATS User Key: (placeholder)".to_string(),
                                            ];

                                            Ok((cert, nats_keys))
                                        },
                                        Message::PersonalKeysGenerated
                                    );
                                }
                                passphrase_dialog::PassphrasePurpose::IntermediateCA => {
                                    self.status_message = "Intermediate CA not yet implemented".to_string();
                                }
                            }
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

                // Search through all nodes using visualization data
                for (node_id, node) in &self.org_graph.nodes {
                    // Use visualization data for searchable text
                    let viz = node.visualization();
                    let query_lower = query.to_lowercase();
                    let matches = viz.primary_text.to_lowercase().contains(&query_lower) ||
                        viz.secondary_text.to_lowercase().contains(&query_lower);
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
                        // Get view data (position, label, color) - fall back to defaults if not found
                        let (pos_x, pos_y, label, color_r, color_g, color_b) =
                            if let Some(view) = self.org_graph.node_views.get(id) {
                                (view.position.x, view.position.y, view.label.clone(), view.color.r, view.color.g, view.color.b)
                            } else {
                                let viz = node.visualization();
                                (0.0, 0.0, viz.primary_text, viz.color.r, viz.color.g, viz.color.b)
                            };
                        ConceptEntityExport {
                            id: *id,
                            node_type: node.injection().display_name().to_string(),
                            position_x: pos_x,
                            position_y: pos_y,
                            label,
                            color_r,
                            color_g,
                            color_b,
                        }
                    }).collect(),
                    edges: self.org_graph.edges.iter().map(|edge| {
                        ConceptRelationExport {
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
                            if let Some(view) = self.org_graph.node_views.get_mut(&node_export.id) {
                                view.position = Point::new(node_export.position_x, node_export.position_y);
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

            // Policy visualization
            Message::PolicyDataLoaded(result) => {
                match result {
                    Ok(data) => {
                        let role_count = data.standard_roles.len();
                        let assignment_count = data.role_assignments.len();
                        let c_level_count = data.get_c_level_list().iter().filter(|(_, p)| p.is_some()).count();

                        self.policy_data = Some(data);

                        // Populate roles in the graph if show_role_nodes is true
                        if self.show_role_nodes {
                            self.populate_role_nodes();
                        }

                        // Populate the policy graph for the Policies view
                        self.populate_policy_graph();

                        self.status_message = format!(
                            "Loaded policy: {} roles, {} assignments, {} C-Level executives",
                            role_count, assignment_count, c_level_count
                        );
                    }
                    Err(e) => {
                        // Policy loading is optional, just log it
                        tracing::warn!("Could not load policy bootstrap: {}", e);
                    }
                }
                Task::none()
            }

            Message::ToggleRoleDisplay => {
                self.show_role_nodes = !self.show_role_nodes;

                if self.show_role_nodes {
                    // Expand: create role nodes and edges
                    self.populate_role_nodes();
                    self.status_message = "Role nodes expanded".to_string();
                } else {
                    // Collapse: remove role nodes (keep badges on person nodes)
                    self.remove_role_nodes();
                    self.status_message = "Role nodes collapsed (showing badges)".to_string();
                }

                Task::none()
            }

            // Progressive disclosure handlers
            Message::SetPolicyExpansionLevel(level) => {
                self.policy_expansion_level = level;
                self.populate_policy_graph();
                self.status_message = format!("Policy view: {:?} level", level);
                Task::none()
            }

            Message::ToggleSeparationClassExpansion(class) => {
                if self.expanded_separation_classes.contains(&class) {
                    self.expanded_separation_classes.remove(&class);
                } else {
                    self.expanded_separation_classes.insert(class);
                }
                self.populate_policy_graph();
                Task::none()
            }

            Message::ToggleCategoryExpansion(category) => {
                if self.expanded_categories.contains(&category) {
                    self.expanded_categories.remove(&category);
                } else {
                    self.expanded_categories.insert(category.clone());
                }
                self.populate_policy_graph();
                Task::none()
            }

            Message::RolePaletteMessage(palette_msg) => {
                match palette_msg {
                    role_palette::RolePaletteMessage::ToggleCategory(class) => {
                        self.role_palette.toggle_category(class);
                    }
                    role_palette::RolePaletteMessage::StartDrag { role_name, separation_class } => {
                        // Select role for assignment - click on Person node to assign
                        let source = graph::DragSource::RoleFromPalette {
                            role_name: role_name.clone(),
                            separation_class,
                        };
                        self.org_graph.start_role_drag(source, Point::ORIGIN);
                        self.status_message = format!("Role '{}' selected - click on a Person to assign, Esc to cancel", role_name);
                    }
                    role_palette::RolePaletteMessage::CancelDrag => {
                        self.org_graph.cancel_role_drag();
                        self.status_message = "Drag cancelled".to_string();
                    }
                }
                Task::none()
            }

            Message::ToggleRolePalette => {
                self.role_palette.toggle_collapsed();
                Task::none()
            }

            Message::RoleDragMove(position) => {
                // Only process if a drag is in progress
                if self.org_graph.dragging_role.is_some() {
                    self.org_graph.update_role_drag(position);

                    // Perform SoD validation if hovering over a person
                    if let Some(ref mut drag) = self.org_graph.dragging_role {
                        if let Some(person_id) = drag.hover_person {
                            // Get the role being dragged
                            let dragged_role_name = match &drag.source {
                                graph::DragSource::RoleFromPalette { role_name, .. } => role_name.clone(),
                                graph::DragSource::RoleFromPerson { role_name, .. } => role_name.clone(),
                            };

                            // Check for SoD conflicts
                            drag.sod_conflicts.clear();
                            if let Some(ref policy_data) = self.policy_data {
                                // Get the person's current roles
                                let current_roles = policy_data.get_roles_for_person(person_id);

                                // Check each current role for incompatibility with the dragged role
                                for current_role in current_roles {
                                    if policy_data.are_roles_incompatible(&dragged_role_name, &current_role.name) {
                                        // Find the separation rule for the reason
                                        let reason = policy_data.separation_of_duties_rules.iter()
                                            .find(|rule| {
                                                (rule.role_a == dragged_role_name && rule.conflicts_with.contains(&current_role.name)) ||
                                                (rule.role_a == current_role.name && rule.conflicts_with.contains(&dragged_role_name))
                                            })
                                            .and_then(|rule| rule.reason.clone())
                                            .unwrap_or_else(|| "Separation of duties violation".to_string());

                                        drag.sod_conflicts.push(graph::SoDConflict {
                                            conflicting_role: current_role.name.clone(),
                                            reason,
                                        });
                                    }
                                }
                            }
                        } else {
                            // Not hovering over anyone - clear conflicts
                            drag.sod_conflicts.clear();
                        }
                    }
                }
                Task::none()
            }

            Message::RoleDragDrop => {
                // Handle drop - assign role to person if hovering over one
                if let Some(ref drag) = self.org_graph.dragging_role.clone() {
                    if let Some(person_id) = drag.hover_person {
                        if drag.sod_conflicts.is_empty() {
                            // No conflicts - assign the role
                            let role_name = match &drag.source {
                                graph::DragSource::RoleFromPalette { role_name, .. } => role_name.clone(),
                                graph::DragSource::RoleFromPerson { role_name, .. } => role_name.clone(),
                            };

                            // Find person name for status message
                            let person_name = self.org_graph.nodes.get(&person_id)
                                .map(|n| n.visualization().primary_text)
                                .unwrap_or_else(|| "Unknown".to_string());

                            // Get role details from policy data for badge
                            let separation_class = match &drag.source {
                                graph::DragSource::RoleFromPalette { separation_class, .. } => *separation_class,
                                graph::DragSource::RoleFromPerson { separation_class, .. } => *separation_class,
                            };

                            let level = self.policy_data.as_ref()
                                .and_then(|p| p.get_role_by_name(&role_name))
                                .map(|r| r.level)
                                .unwrap_or(0);

                            // Add badge to person's role badges
                            let badge = graph::RoleBadge {
                                name: role_name.clone(),
                                separation_class,
                                level,
                            };

                            self.org_graph.role_badges
                                .entry(person_id)
                                .or_insert_with(|| graph::PersonRoleBadges::default())
                                .badges.push(badge);

                            self.status_message = format!("Assigned role '{}' to {}", role_name, person_name);
                            tracing::info!("Role '{}' assigned to person {:?}", role_name, person_id);
                        } else {
                            // Has conflicts - show warning
                            let conflict_names: Vec<_> = drag.sod_conflicts.iter()
                                .map(|c| c.conflicting_role.as_str())
                                .collect();
                            self.status_message = format!(
                                "Cannot assign: SoD conflict with {:?}",
                                conflict_names
                            );
                        }
                    }
                }
                self.org_graph.cancel_role_drag();
                Task::none()
            }

            Message::RoleDragCancel => {
                self.org_graph.cancel_role_drag();
                self.status_message = "Drag cancelled".to_string();
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
            button(text("Locations").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::Locations))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Locations)),
            button(text("Keys").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::Keys))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Keys)),
            button(text("Export").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::Export))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Export)),
        ]
        .spacing(self.view_model.spacing_md);

        // Tab content
        let content = match self.active_tab {
            Tab::Welcome => self.view_welcome(),
            Tab::Organization => self.view_organization(),
            Tab::Locations => self.view_locations(),
            Tab::Keys => self.view_keys(),
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

        // Add passphrase dialog overlay if visible (node selector is now in graph canvas)
        if self.passphrase_dialog.is_visible() {
            stack![
                base_view,
                self.passphrase_dialog.view(&self.view_model).map(Message::PassphraseDialogMessage)
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            base_view.into()
        }
    }

    /// Render the node type selector menu (FRP event-driven UI)
    fn view_node_selector_menu(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, container, text};
        use iced::Length;

        // Create menu with all node types
        let mut menu_column = column![]
            .spacing(self.view_model.spacing_xs);

        // Title
        menu_column = menu_column.push(
            text("Select Node Type")
                .size(self.view_model.text_normal)
                .color(CowboyTheme::text_primary())
        );

        // Add button for each node type with selection highlighting
        for (index, node_type) in Injection::creatable().iter().enumerate() {
            let is_selected = index == self.selected_menu_index;

            let button_text = if is_selected {
                text(format!("▶ {}", node_type.display_name()))
                    .size(self.view_model.text_normal)
            } else {
                text(format!("  {}", node_type.display_name()))
                    .size(self.view_model.text_normal)
            };

            menu_column = menu_column.push(
                button(button_text)
                .on_press(Message::SelectNodeType(*node_type))
                .style(CowboyCustomTheme::glass_menu_button(is_selected))
                .width(Length::Fill)
                .padding(self.view_model.padding_md)
            );
        }

        // Cancel button
        menu_column = menu_column.push(
            button(text("Cancel").size(self.view_model.text_normal))
                .on_press(Message::CancelNodeTypeSelector)
                .style(CowboyCustomTheme::glass_menu_button(false))
                .width(Length::Fill)
                .padding(self.view_model.padding_md)
        );

        // Instructions
        menu_column = menu_column.push(
            text("↑↓: Navigate • Enter: Select • Esc: Cancel")
                .size(self.view_model.text_small)
                .color(self.view_model.colors.text_light)
        );

        // Return styled menu container (positioning handled by caller)
        container(menu_column)
            .padding(self.view_model.padding_lg)
            .width(Length::Fixed(320.0 * self.view_model.scale))
            .style(|_theme: &Theme| container::Style {
                background: Some(CowboyTheme::glass_background()),
                text_color: Some(CowboyTheme::text_primary()),
                border: Border {
                    color: self.view_model.colors.with_alpha(self.view_model.colors.info, 0.8),
                    width: 2.0,
                    radius: 12.0.into(),
                },
                shadow: CowboyTheme::glow_shadow(),
            })
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::{time, keyboard, event};
        use iced::keyboard::Key;
        use std::time::Duration;

        Subscription::batch([
            // Update animation at 30 FPS instead of 60 to reduce resource usage
            time::every(Duration::from_millis(33)).map(|_| Message::AnimationTick),

            // Graph layout animation subscriptions (only active when animating)
            self.org_graph.subscription().map(Message::OrganizationIntent),
            self.pki_graph.subscription().map(Message::OrganizationIntent),
            self.nats_graph.subscription().map(Message::OrganizationIntent),
            self.yubikey_graph.subscription().map(Message::OrganizationIntent),
            self.location_graph.subscription().map(Message::OrganizationIntent),
            self.policy_graph.subscription().map(Message::OrganizationIntent),

            // Mouse movement tracking for role drag-and-drop
            // Only process drag events when a drag is actually in progress
            // This avoids flooding the message queue with mouse moves
            if self.org_graph.dragging_role.is_some() {
                event::listen_with(|event, _status, _window| {
                    match event {
                        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                            Some(Message::RoleDragMove(position))
                        }
                        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                            Some(Message::RoleDragDrop)
                        }
                        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                            ..
                        }) => {
                            Some(Message::RoleDragCancel)
                        }
                        _ => None,
                    }
                })
            } else {
                Subscription::none()
            },

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
                                    Some(Message::OrganizationIntent(
                                        crate::gui::graph::OrganizationIntent::Undo
                                    ))
                                }
                                Key::Character(c) if c == "z" && modifiers.shift() => {
                                    Some(Message::OrganizationIntent(
                                        crate::gui::graph::OrganizationIntent::Redo
                                    ))
                                }
                                Key::Character(c) if c == "y" => {
                                    Some(Message::OrganizationIntent(
                                        crate::gui::graph::OrganizationIntent::Redo
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
                                    // Send InlineEditCancel - it will handle both node selector and inline edit
                                    Some(Message::InlineEditCancel)
                                }
                                Key::Named(keyboard::key::Named::Delete) => {
                                    Some(Message::OrganizationIntent(
                                        crate::gui::graph::OrganizationIntent::DeleteSelected
                                    ))
                                }
                                Key::Named(keyboard::key::Named::ArrowUp) => {
                                    // Arrow up: navigate menu selection up
                                    Some(Message::MenuNavigateUp)
                                }
                                Key::Named(keyboard::key::Named::ArrowDown) => {
                                    // Arrow down: navigate menu selection down
                                    Some(Message::MenuNavigateDown)
                                }
                                Key::Named(keyboard::key::Named::Space) => {
                                    // SPACE: Accept menu selection if menu is open, otherwise create node
                                    Some(Message::MenuAcceptSelection)
                                }
                                Key::Named(keyboard::key::Named::Enter) => {
                                    // Enter: Accept menu selection if menu is open
                                    Some(Message::MenuAcceptSelection)
                                }
                                Key::Named(keyboard::key::Named::Tab) => {
                                    // TAB: Cycle selected edge type
                                    Some(Message::CycleEdgeType)
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

    /// Returns a mutable reference to the currently active graph based on graph_view
    fn active_graph_mut(&mut self) -> &mut graph::OrganizationConcept {
        match self.graph_view {
            GraphView::Organization => &mut self.org_graph,
            GraphView::PkiTrustChain => &mut self.pki_graph,
            GraphView::NatsInfrastructure => &mut self.nats_graph,
            GraphView::YubiKeyDetails => &mut self.yubikey_graph,
            GraphView::Policies => &mut self.policy_graph,
            GraphView::Aggregates => &mut self.aggregates_graph,
            GraphView::CommandHistory | GraphView::CausalityChains => &mut self.empty_graph,
        }
    }

    /// Populate role nodes and edges from policy data
    fn populate_role_nodes(&mut self) {
        if let Some(ref policy_data) = self.policy_data {
            // Track role nodes that we add
            let mut role_node_ids: std::collections::HashMap<String, Uuid> = std::collections::HashMap::new();

            // Add role nodes for each assigned role (only roles that are actually assigned)
            for role_name in policy_data.get_assigned_role_names() {
                if let Some(role_entry) = policy_data.get_role_by_name(role_name) {
                    let role_id = Uuid::now_v7();
                    role_node_ids.insert(role_name.to_string(), role_id);

                    // Add role node to the org graph
                    self.org_graph.add_role_node(
                        role_id,
                        role_entry.name.clone(),
                        role_entry.purpose.clone(),
                        role_entry.level,
                        role_entry.separation_class_enum(),
                        role_entry.claims.len(),
                    );
                }
            }

            // Add HasRole edges from people to their roles
            for assignment in &policy_data.role_assignments {
                for role_name in &assignment.roles {
                    if let Some(&role_id) = role_node_ids.get(role_name) {
                        self.org_graph.add_edge(
                            assignment.person_id,
                            role_id,
                            crate::gui::graph::EdgeType::HasRole {
                                valid_from: assignment.valid_from.unwrap_or_else(chrono::Utc::now),
                                valid_until: assignment.valid_until,
                            },
                        );
                    }
                }
            }

            // Also add edges for roles defined in people entries
            for person in &policy_data.people {
                for role_name in &person.assigned_roles {
                    if let Some(&role_id) = role_node_ids.get(role_name) {
                        self.org_graph.add_edge(
                            person.id,
                            role_id,
                            crate::gui::graph::EdgeType::HasRole {
                                valid_from: chrono::Utc::now(),
                                valid_until: None,
                            },
                        );
                    }
                }
            }

            // Add IncompatibleWith edges between roles (separation of duties)
            for rule in &policy_data.separation_of_duties_rules {
                if let Some(&role_a_id) = role_node_ids.get(&rule.role_a) {
                    for conflicting_role in &rule.conflicts_with {
                        if let Some(&role_b_id) = role_node_ids.get(conflicting_role) {
                            self.org_graph.add_edge(
                                role_a_id,
                                role_b_id,
                                crate::gui::graph::EdgeType::IncompatibleWith,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Remove role nodes from the graph (collapse to badges)
    fn remove_role_nodes(&mut self) {
        // Remove all PolicyRole nodes and their edges
        // Use injection helper for type checking
        self.org_graph.nodes.retain(|_, node| {
            !node.injection().is_policy_role()
        });

        // Remove edges that connect to role nodes (HasRole, IncompatibleWith)
        self.org_graph.edges.retain(|edge| {
            !matches!(edge.edge_type, crate::gui::graph::EdgeType::HasRole { .. })
                && !matches!(edge.edge_type, crate::gui::graph::EdgeType::IncompatibleWith)
        });

        // Populate role badges for compact mode display
        self.populate_role_badges();
    }

    /// Populate role badges on person nodes for compact display mode
    fn populate_role_badges(&mut self) {
        if let Some(ref policy_data) = self.policy_data {
            // Build role badges for each person
            for (person_id, roles) in policy_data.get_all_person_roles() {
                let badges: Vec<crate::gui::graph::RoleBadge> = roles
                    .iter()
                    .take(3) // Max 3 badges visible
                    .map(|role| crate::gui::graph::RoleBadge {
                        name: role.name.clone(),
                        separation_class: role.separation_class_enum(),
                        level: role.level,
                    })
                    .collect();
                let has_more = roles.len() > 3;
                self.org_graph.set_person_role_badges(person_id, badges, has_more);
            }
        }
    }

    /// Get roles for a person from policy data
    pub fn get_person_roles(&self, person_id: Uuid) -> Vec<&crate::policy_loader::StandardRoleEntry> {
        if let Some(ref policy_data) = self.policy_data {
            policy_data.get_roles_for_person(person_id)
        } else {
            Vec::new()
        }
    }

    /// Populate the policy graph with progressive disclosure
    /// - Classes level: Show only 6 PolicyGroup nodes
    /// - Roles level: Show roles in expanded classes, or all if level is Roles
    /// - Claims level: Show roles + claims for expanded categories
    fn populate_policy_graph(&mut self) {
        // Clear existing policy graph
        self.policy_graph.nodes.clear();
        self.policy_graph.edges.clear();

        if let Some(ref policy_data) = self.policy_data {
            let class_order = [
                crate::policy::SeparationClass::Operational,
                crate::policy::SeparationClass::Administrative,
                crate::policy::SeparationClass::Financial,
                crate::policy::SeparationClass::Audit,
                crate::policy::SeparationClass::Personnel,
                crate::policy::SeparationClass::Emergency,
            ];

            // Group roles by separation class
            let mut roles_by_class: std::collections::HashMap<crate::policy::SeparationClass, Vec<&crate::policy_loader::StandardRoleEntry>> = std::collections::HashMap::new();
            for role in &policy_data.standard_roles {
                let class = role.separation_class_enum();
                roles_by_class.entry(class).or_default().push(role);
            }

            let base_x = 100.0;
            let base_y = 100.0;
            let class_spacing_x = 300.0;
            let role_spacing_y = 80.0;

            let mut class_node_ids: std::collections::HashMap<crate::policy::SeparationClass, Uuid> = std::collections::HashMap::new();
            let mut role_node_ids: std::collections::HashMap<String, Uuid> = std::collections::HashMap::new();
            let mut category_node_ids: std::collections::HashMap<String, Uuid> = std::collections::HashMap::new();
            let mut claim_node_ids: std::collections::HashMap<String, Uuid> = std::collections::HashMap::new();

            // Always show separation class groups at the top level
            for (class_idx, class) in class_order.iter().enumerate() {
                let role_count = roles_by_class.get(class).map(|r| r.len()).unwrap_or(0);
                let class_id = Uuid::now_v7();
                class_node_ids.insert(*class, class_id);
                let is_expanded = self.expanded_separation_classes.contains(class)
                    || matches!(self.policy_expansion_level, PolicyExpansionLevel::Roles | PolicyExpansionLevel::Claims);

                let class_name = match class {
                    crate::policy::SeparationClass::Operational => "Operational",
                    crate::policy::SeparationClass::Administrative => "Administrative",
                    crate::policy::SeparationClass::Financial => "Financial",
                    crate::policy::SeparationClass::Audit => "Audit",
                    crate::policy::SeparationClass::Personnel => "Personnel",
                    crate::policy::SeparationClass::Emergency => "Emergency",
                };

                // Add separation class group node
                self.policy_graph.add_separation_class_node(
                    class_id,
                    class_name.to_string(),
                    *class,
                    role_count,
                    is_expanded,
                );

                // Position at top
                let x = base_x + (class_idx as f32) * class_spacing_x;
                if let Some(view) = self.policy_graph.node_views.get_mut(&class_id) {
                    view.position = Point::new(x, base_y);
                }

                // If expanded, show roles in this class
                if is_expanded {
                    if let Some(roles) = roles_by_class.get(class) {
                        let mut sorted_roles = roles.clone();
                        sorted_roles.sort_by(|a, b| b.level.cmp(&a.level).then(a.name.cmp(&b.name)));

                        for (role_idx, role) in sorted_roles.iter().enumerate() {
                            let role_id = Uuid::now_v7();
                            role_node_ids.insert(role.name.clone(), role_id);

                            let x = base_x + (class_idx as f32) * class_spacing_x;
                            let y = base_y + 100.0 + (role_idx as f32) * role_spacing_y;

                            self.policy_graph.add_role_node(
                                role_id,
                                role.name.clone(),
                                role.purpose.clone(),
                                role.level,
                                role.separation_class_enum(),
                                role.claims.len(),
                            );

                            if let Some(view) = self.policy_graph.node_views.get_mut(&role_id) {
                                view.position = Point::new(x, y);
                            }

                            // Add edge from class to role
                            self.policy_graph.add_edge(
                                class_id,
                                role_id,
                                crate::gui::graph::EdgeType::ClassContainsRole,
                            );
                        }
                    }
                }
            }

            // Add IncompatibleWith edges between roles (only if roles are visible)
            if matches!(self.policy_expansion_level, PolicyExpansionLevel::Roles | PolicyExpansionLevel::Claims)
                || !self.expanded_separation_classes.is_empty()
            {
                for rule in &policy_data.separation_of_duties_rules {
                    if let Some(&role_a_id) = role_node_ids.get(&rule.role_a) {
                        for conflicting_role in &rule.conflicts_with {
                            if let Some(&role_b_id) = role_node_ids.get(conflicting_role) {
                                self.policy_graph.add_edge(
                                    role_a_id,
                                    role_b_id,
                                    crate::gui::graph::EdgeType::IncompatibleWith,
                                );
                            }
                        }
                    }
                }
            }

            // At Claims level, show category nodes and their claims
            if matches!(self.policy_expansion_level, PolicyExpansionLevel::Claims) {
                let claim_base_y = base_y + 600.0;
                let category_spacing_x = 200.0;
                let claim_spacing_y = 40.0;

                for (cat_idx, (category, claims)) in policy_data.claim_categories.iter().enumerate() {
                    let category_id = Uuid::now_v7();
                    category_node_ids.insert(category.clone(), category_id);
                    let is_cat_expanded = self.expanded_categories.contains(category);

                    // Add category node
                    self.policy_graph.add_category_node(
                        category_id,
                        category.clone(),
                        claims.len(),
                        is_cat_expanded,
                    );

                    let x = base_x + (cat_idx as f32) * category_spacing_x;
                    if let Some(view) = self.policy_graph.node_views.get_mut(&category_id) {
                        view.position = Point::new(x, claim_base_y);
                    }

                    // If category is expanded, show individual claims
                    if is_cat_expanded {
                        for (claim_idx, claim_name) in claims.iter().enumerate() {
                            let claim_id = Uuid::now_v7();
                            claim_node_ids.insert(claim_name.clone(), claim_id);

                            let y = claim_base_y + 60.0 + (claim_idx as f32) * claim_spacing_y;

                            self.policy_graph.add_claim_node(
                                claim_id,
                                claim_name.clone(),
                                category.clone(),
                            );

                            if let Some(view) = self.policy_graph.node_views.get_mut(&claim_id) {
                                view.position = Point::new(x, y);
                            }

                            // Add edge from category to claim
                            self.policy_graph.add_edge(
                                category_id,
                                claim_id,
                                crate::gui::graph::EdgeType::CategoryContainsClaim,
                            );
                        }
                    }
                }

                // Add RoleContainsClaim edges for expanded claims
                for role in &policy_data.standard_roles {
                    if let Some(&role_id) = role_node_ids.get(&role.name) {
                        for claim_name in &role.claims {
                            if let Some(&claim_id) = claim_node_ids.get(claim_name) {
                                self.policy_graph.add_edge(
                                    role_id,
                                    claim_id,
                                    crate::gui::graph::EdgeType::RoleContainsClaim,
                                );
                            }
                        }
                    }
                }
            }

            // Apply barycenter ordering to claims to minimize edge crossings
            if matches!(self.policy_expansion_level, PolicyExpansionLevel::Claims) {
                self.order_claims_by_barycenter(&category_node_ids, &claim_node_ids, &role_node_ids);
            }

            let node_count = self.policy_graph.nodes.len();
            let edge_count = self.policy_graph.edges.len();
            tracing::info!(
                "Policy graph populated with {} nodes and {} edges (level: {:?})",
                node_count,
                edge_count,
                self.policy_expansion_level
            );
        }
    }

    /// Order claims within each category by barycenter to minimize edge crossings with roles
    /// Barycenter = average x-position of connected role nodes
    fn order_claims_by_barycenter(
        &mut self,
        category_node_ids: &std::collections::HashMap<String, Uuid>,
        claim_node_ids: &std::collections::HashMap<String, Uuid>,
        role_node_ids: &std::collections::HashMap<String, Uuid>,
    ) {
        // Get policy data for claim-to-role mapping
        let policy_data = match &self.policy_data {
            Some(pd) => pd.clone(),
            None => return,
        };

        // Build reverse mapping: claim -> roles that contain it
        let mut claim_to_roles: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for role in &policy_data.standard_roles {
            for claim in &role.claims {
                claim_to_roles.entry(claim.clone()).or_default().push(role.name.clone());
            }
        }

        // For each category, reorder claims by barycenter
        for (category, claims) in &policy_data.claim_categories {
            // Skip if category is not expanded
            if !self.expanded_categories.contains(category) {
                continue;
            }

            // Calculate barycenter for each claim
            let mut claim_barycenters: Vec<(String, f32)> = Vec::new();
            for claim_name in claims {
                if let Some(&claim_id) = claim_node_ids.get(claim_name) {
                    // Get x positions of all connected roles
                    let connected_roles = claim_to_roles.get(claim_name);
                    let mut x_sum = 0.0;
                    let mut count = 0;

                    if let Some(roles) = connected_roles {
                        for role_name in roles {
                            if let Some(&role_id) = role_node_ids.get(role_name) {
                                if let Some(role_view) = self.policy_graph.node_views.get(&role_id) {
                                    x_sum += role_view.position.x;
                                    count += 1;
                                }
                            }
                        }
                    }

                    // Barycenter is average x, or original position if no connections
                    let barycenter = if count > 0 {
                        x_sum / count as f32
                    } else if let Some(view) = self.policy_graph.node_views.get(&claim_id) {
                        view.position.x // Keep original position
                    } else {
                        0.0
                    };

                    claim_barycenters.push((claim_name.clone(), barycenter));
                }
            }

            // Sort claims by barycenter
            claim_barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            // Get category position for base x
            let category_x = category_node_ids.get(category)
                .and_then(|id| self.policy_graph.node_views.get(id))
                .map(|v| v.position.x)
                .unwrap_or(100.0);

            let category_y = category_node_ids.get(category)
                .and_then(|id| self.policy_graph.node_views.get(id))
                .map(|v| v.position.y)
                .unwrap_or(700.0);

            // Reposition claims in sorted order
            let claim_spacing_y = 40.0;
            for (idx, (claim_name, _barycenter)) in claim_barycenters.iter().enumerate() {
                if let Some(&claim_id) = claim_node_ids.get(claim_name) {
                    if let Some(view) = self.policy_graph.node_views.get_mut(&claim_id) {
                        view.position = Point::new(
                            category_x,
                            category_y + 60.0 + (idx as f32) * claim_spacing_y,
                        );
                    }
                }
            }
        }

        tracing::debug!("Applied barycenter ordering to claims");
    }

    /// Populate the aggregates graph with DDD aggregate state
    fn populate_aggregates_graph(&mut self) {
        // Clear existing aggregates graph
        self.aggregates_graph.nodes.clear();
        self.aggregates_graph.edges.clear();
        self.aggregates_graph.node_views.clear();

        let base_x = 200.0;
        let base_y = 150.0;
        let spacing_x = 350.0;

        // Organization Aggregate
        let org_id = Uuid::now_v7();
        let people_count = self.org_graph.nodes.values()
            .filter(|n| n.injection().is_person())
            .count();
        let units_count = self.org_graph.nodes.values()
            .filter(|n| n.injection().is_organization_unit())
            .count();
        self.aggregates_graph.add_aggregate_organization_node(
            org_id,
            "Organization".to_string(),
            0,  // Version - calculated from event count
            people_count,
            units_count,
        );
        if let Some(view) = self.aggregates_graph.node_views.get_mut(&org_id) {
            view.position = Point::new(base_x, base_y);
            view.color = self.view_model.colors.aggregate_organization;
        }

        // PKI Certificate Chain Aggregate
        let pki_id = Uuid::now_v7();
        // Count PKI nodes from pki_graph using injection helpers
        let certs_count = self.pki_graph.nodes.values()
            .filter(|n| n.injection().is_certificate())
            .count();
        let keys_count = self.pki_graph.nodes.values()
            .filter(|n| n.injection().is_key())
            .count();
        self.aggregates_graph.add_aggregate_pki_chain_node(
            pki_id,
            "PKI Certificate Chain".to_string(),
            certs_count as u64,
            certs_count,
            keys_count,
        );
        if let Some(view) = self.aggregates_graph.node_views.get_mut(&pki_id) {
            view.position = Point::new(base_x + spacing_x, base_y);
            view.color = self.view_model.colors.aggregate_pki_chain;
        }

        // NATS Security Aggregate - using injection helpers
        let nats_id = Uuid::now_v7();
        let operators_count = self.nats_graph.nodes.values()
            .filter(|n| n.injection().is_nats_operator())
            .count();
        let accounts_count = self.nats_graph.nodes.values()
            .filter(|n| n.injection().is_nats_account())
            .count();
        let users_count = self.nats_graph.nodes.values()
            .filter(|n| n.injection().is_nats_user())
            .count();
        self.aggregates_graph.add_aggregate_nats_security_node(
            nats_id,
            "NATS Security".to_string(),
            0,  // Version
            operators_count,
            accounts_count,
            users_count,
        );
        if let Some(view) = self.aggregates_graph.node_views.get_mut(&nats_id) {
            view.position = Point::new(base_x, base_y + 200.0);
            view.color = self.view_model.colors.aggregate_nats_security;
        }

        // YubiKey Provisioning Aggregate - using injection helpers
        let yubikey_id = Uuid::now_v7();
        let devices_count = self.yubikey_graph.nodes.values()
            .filter(|n| n.injection().is_yubikey_device())
            .count();
        let slots_count = self.yubikey_graph.nodes.values()
            .filter(|n| n.injection().is_piv_slot())
            .count();
        self.aggregates_graph.add_aggregate_yubikey_provisioning_node(
            yubikey_id,
            "YubiKey Provisioning".to_string(),
            0,  // Version
            devices_count,
            slots_count,
        );
        if let Some(view) = self.aggregates_graph.node_views.get_mut(&yubikey_id) {
            view.position = Point::new(base_x + spacing_x, base_y + 200.0);
            view.color = self.view_model.colors.aggregate_yubikey;
        }

        tracing::info!(
            "Aggregates graph populated with {} nodes: Org({} people, {} units), PKI({} certs, {} keys), NATS({} ops, {} accts, {} users), YubiKey({} devices, {} slots)",
            self.aggregates_graph.nodes.len(),
            people_count, units_count,
            certs_count, keys_count,
            operators_count, accounts_count, users_count,
            devices_count, slots_count
        );
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
                        verified::icon("warning", self.view_model.text_large),
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
                        .font(FONT_BODY)
                        .size(14)
                        .color(Color::WHITE)
                } else {
                    text("⚠️  No domain loaded - Right-click canvas to create")
                        .font(FONT_BODY)
                        .size(14)
                        .color(Color::from_rgb(1.0, 0.8, 0.2))  // Bright yellow/orange for visibility
                },
                // Graph context switcher - simple icon buttons centered in toolbar
                horizontal_space(),
                row![
                    // Organization context - org structure
                    button(text("🏢").font(EMOJI_FONT).size(16))
                        .on_press(Message::GraphViewSelected(GraphView::Organization))
                        .padding(8)
                        .style(CowboyCustomTheme::glass_menu_button(self.graph_view == GraphView::Organization)),

                    // Identity context - composing identities, matching people to roles and policies
                    button(text("🆔").font(EMOJI_FONT).size(16))
                        .on_press(Message::GraphViewSelected(GraphView::NatsInfrastructure))
                        .padding(8)
                        .style(CowboyCustomTheme::glass_menu_button(self.graph_view == GraphView::NatsInfrastructure)),

                    // Policies context - roles, claims, separation of duties
                    button(text("📜").font(EMOJI_FONT).size(16))
                        .on_press(Message::GraphViewSelected(GraphView::Policies))
                        .padding(8)
                        .style(CowboyCustomTheme::glass_menu_button(self.graph_view == GraphView::Policies)),

                    // Aggregates context - domain aggregate state machines (placeholder)
                    button(text("⚙️").font(EMOJI_FONT).size(16))
                        .on_press(Message::GraphViewSelected(GraphView::Aggregates))
                        .padding(8)
                        .style(CowboyCustomTheme::glass_menu_button(self.graph_view == GraphView::Aggregates)),

                    // PKI context - all cryptographic keys
                    button(text("🔐").font(EMOJI_FONT).size(16))
                        .on_press(Message::GraphViewSelected(GraphView::PkiTrustChain))
                        .padding(8)
                        .style(CowboyCustomTheme::glass_menu_button(self.graph_view == GraphView::PkiTrustChain)),

                    // YubiKey context - hardware security keys
                    button(text("🔑").font(EMOJI_FONT).size(16))
                        .on_press(Message::GraphViewSelected(GraphView::YubiKeyDetails))
                        .padding(8)
                        .style(CowboyCustomTheme::glass_menu_button(self.graph_view == GraphView::YubiKeyDetails)),
                ]
                .spacing(6),
                horizontal_space(),
                // Context-aware "Add Node" dropdown - shows only valid nodes for current view
                {
                    let node_options = match self.graph_view {
                        GraphView::Organization => vec!["Person", "Unit", "Location", "Role"],
                        GraphView::NatsInfrastructure => vec!["Account", "User", "Service"],
                        GraphView::PkiTrustChain => vec!["Root CA", "Inter CA", "Leaf Cert", "CSR"],
                        GraphView::YubiKeyDetails => vec!["YubiKey", "PIV Slot"],
                        GraphView::Policies => vec!["Role", "Claim", "SoD Rule"], // Compose policies from claims, define separation rules
                        GraphView::Aggregates => vec![], // Aggregates are read-only (state machines)
                        GraphView::CommandHistory => vec![], // Command history is read-only
                        GraphView::CausalityChains => vec![], // Causality is read-only (derived from events)
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
                    .on_press(Message::OrganizationIntent(graph::OrganizationIntent::ResetView))
                    .style(CowboyCustomTheme::glass_button()),
                // Role display toggle button
                button(text(if self.show_role_nodes { "📋" } else { "👤" }).font(EMOJI_FONT).size(14))
                    .on_press(Message::ToggleRoleDisplay)
                    .padding(6)
                    .style(CowboyCustomTheme::glass_menu_button(self.show_role_nodes)),
                // Progressive disclosure level buttons (only visible in Policies view)
                {
                    let disclosure_buttons: Element<'_, Message> = if matches!(self.graph_view, GraphView::Policies) {
                        row![
                            // Classes level (collapsed, 6 nodes)
                            button(text("📁").font(EMOJI_FONT).size(14))
                                .on_press(Message::SetPolicyExpansionLevel(PolicyExpansionLevel::Classes))
                                .padding(6)
                                .style(CowboyCustomTheme::glass_menu_button(
                                    matches!(self.policy_expansion_level, PolicyExpansionLevel::Classes)
                                )),
                            // Roles level (26+ nodes)
                            button(text("📋").font(EMOJI_FONT).size(14))
                                .on_press(Message::SetPolicyExpansionLevel(PolicyExpansionLevel::Roles))
                                .padding(6)
                                .style(CowboyCustomTheme::glass_menu_button(
                                    matches!(self.policy_expansion_level, PolicyExpansionLevel::Roles)
                                )),
                            // Claims level (full disclosure, 294+ nodes)
                            button(text("📑").font(EMOJI_FONT).size(14))
                                .on_press(Message::SetPolicyExpansionLevel(PolicyExpansionLevel::Claims))
                                .padding(6)
                                .style(CowboyCustomTheme::glass_menu_button(
                                    matches!(self.policy_expansion_level, PolicyExpansionLevel::Claims)
                                )),
                        ]
                        .spacing(2)
                        .into()
                    } else {
                        Space::with_width(0).into()
                    };
                    disclosure_buttons
                },
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
            // Show appropriate graph based on current context (using cached filtered graphs)
            let graph_content: Element<'_, graph::OrganizationIntent> = match self.graph_view {
                GraphView::Organization => {
                    // Show full organization graph (all nodes)
                    view_graph(&self.org_graph, &self.view_model)
                }
                GraphView::PkiTrustChain => {
                    // Show only PKI-related nodes
                    view_graph(&self.pki_graph, &self.view_model)
                }
                GraphView::NatsInfrastructure => {
                    // Show only NATS identity nodes
                    view_graph(&self.nats_graph, &self.view_model)
                }
                GraphView::YubiKeyDetails => {
                    // Show only YubiKey hardware and keys
                    view_graph(&self.yubikey_graph, &self.view_model)
                }
                GraphView::Policies => {
                    // Show policy roles, claims, and SoD relationships
                    view_graph(&self.policy_graph, &self.view_model)
                }
                GraphView::Aggregates => {
                    // Show aggregate state machines
                    view_graph(&self.aggregates_graph, &self.view_model)
                }
                GraphView::CommandHistory | GraphView::CausalityChains => {
                    // Empty graph for read-only derived views
                    view_graph(&self.empty_graph, &self.view_model)
                }
            };

            let graph_base = Container::new(
                graph_content.map(Message::OrganizationIntent)
            )
            .width(Length::Fill)
            .height(Length::Fill)  // FILL ALL SPACE!
            .style(|_theme| {
                container::Style {
                    background: Some(Background::Color(self.view_model.colors.background)),
                    border: Border {
                        color: self.view_model.colors.blue_bright,
                        width: 1.0,
                        radius: 12.0.into(),  // Rounded corners for canvas
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
                        self.context_menu.view(&self.view_model)
                            .map(Message::ContextMenuMessage)
                    ]
                ];

                stack_layers.push(menu_overlay.into());
            }

            // Add inline edit overlay if editing a newly created node
            if let Some(node_id) = self.editing_new_node {
                if let Some(view) = self.org_graph.node_views.get(&node_id) {
                    use iced::widget::text_input;
                    const TOOLBAR_OFFSET: f32 = 36.0;

                    // Transform node position to screen coordinates
                    let screen_x = view.position.x * self.org_graph.zoom + self.org_graph.pan_offset.x;
                    let screen_y = view.position.y * self.org_graph.zoom + self.org_graph.pan_offset.y + TOOLBAR_OFFSET;

                    // Position input slightly below the node
                    let input_y = screen_y + 30.0;

                    let edit_overlay = column![
                        vertical_space().height(Length::Fixed(input_y)),
                        row![
                            horizontal_space().width(Length::Fixed(screen_x - 100.0)), // Center on node (200px wide container)
                            container(
                                column![
                                    text_input("Node name...", &self.inline_edit_name)
                                        .on_input(Message::InlineEditNameChanged)
                                        .on_submit(Message::InlineEditSubmit)
                                        .size(14)
                                        .padding(6)
                                        .width(Length::Fixed(180.0)),
                                    row![
                                        button(text("OK").size(12))
                                            .on_press(Message::InlineEditSubmit)
                                            .padding(4)
                                            .style(CowboyCustomTheme::glass_menu_button(false)),
                                        horizontal_space().width(Length::Fixed(8.0)),
                                        button(text("Cancel").size(12))
                                            .on_press(Message::InlineEditCancel)
                                            .padding(4)
                                            .style(CowboyCustomTheme::glass_menu_button(false)),
                                    ].spacing(4)
                                ].spacing(6)
                            )
                            .padding(8)
                            .style(|_theme| container::Style {
                                background: Some(Background::Color(Color::from_rgba8(40, 40, 50, 0.95))),
                                border: Border {
                                    color: Color::from_rgb(0.4, 0.6, 0.8),
                                    width: 2.0,
                                    radius: 8.0.into(),
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
                        self.property_card.view(&self.view_model)
                            .map(Message::PropertyCardMessage)
                    ]
                )
                .padding(self.view_model.padding_xl);

                stack_layers.push(card_overlay.into());
            }

            // Add node selector menu overlay if visible
            if self.show_node_type_selector {
                let menu_overlay = container(
                    row![
                        horizontal_space(),
                        self.view_node_selector_menu()
                    ]
                )
                .padding(20);

                stack_layers.push(menu_overlay.into());
            }

            stack(stack_layers)
        };

        // Simple two-row layout: tiny toolbar + massive graph
        // Role palette visible in views where policies apply (Organization for assignment, Policies for composition)
        let show_role_palette = matches!(self.graph_view, GraphView::Organization | GraphView::Policies)
            && self.policy_data.is_some();

        let graph_row: Element<'_, Message> = if show_role_palette {
            // Show role palette on the left side
            row![
                self.role_palette.view(self.policy_data.as_ref(), &self.view_model)
                    .map(Message::RolePaletteMessage),
                graph_canvas,
            ]
            .spacing(self.view_model.padding_xs)
            .height(Length::Fill)
            .into()
        } else {
            // Other views: just the graph canvas
            graph_canvas.into()
        };

        let content = column![
            toolbar,
            graph_row,
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
                        button(if self.show_passphrase { verified::icon("visibility_off", 16) } else { verified::icon("visibility", 16) })
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
                                       self.org_graph.nodes.iter().any(|(_, n)| n.injection().is_organization()) {
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
                            } else if !self.org_graph.nodes.iter().any(|(_, n)| n.injection().is_organization()) {
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
                                                    if self.org_graph.nodes.iter().any(|(_, n)| n.injection().is_organization()) {
                                                        Some(Message::GenerateNatsFromGraph)
                                                    } else {
                                                        None
                                                    }
                                                )
                                                .padding(self.view_model.padding_xl)
                                                .width(Length::Fill)
                                                .style(CowboyCustomTheme::primary_button()),
                                            if !self.org_graph.nodes.iter().any(|(_, n)| n.injection().is_organization()) {
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
                                                    if self.org_graph.nodes.iter().any(|(_, n)| n.injection().is_person()) {
                                                        Some(Message::ProvisionYubiKeysFromGraph)
                                                    } else {
                                                        None
                                                    }
                                                )
                                                .padding(self.view_model.padding_xl)
                                                .width(Length::Fill)
                                                .style(CowboyCustomTheme::primary_button()),
                                            if !self.org_graph.nodes.iter().any(|(_, n)| n.injection().is_person()) {
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

    /// Compute export readiness status from current app state
    fn compute_export_readiness(&self) -> ExportReadiness {
        let mut readiness = ExportReadiness::default();

        // Check organization
        readiness.has_organization = self.domain_loaded && !self.organization_name.is_empty();
        if !readiness.has_organization {
            readiness.missing_items.push(ExportMissingItem {
                category: "Organization",
                description: "No organization defined".to_string(),
                severity: MissingSeverity::Required,
                action: Some("Load domain or create new organization".to_string()),
            });
        }

        // Check people
        let people_count = self.loaded_people.len();
        readiness.has_people = people_count > 0;
        if !readiness.has_people {
            readiness.missing_items.push(ExportMissingItem {
                category: "People",
                description: "No people defined in domain".to_string(),
                severity: MissingSeverity::Required,
                action: Some("Add people via Organization graph".to_string()),
            });
        }

        // Check certificates - use is_ca and issuer to determine certificate type
        // Root CA: is_ca=true and no issuer (self-signed)
        // Intermediate CA: is_ca=true and has issuer (signed by root)
        // Leaf: is_ca=false
        let root_ca_count = self.loaded_certificates.iter()
            .filter(|c| c.is_ca && c.issuer.is_none())
            .count();
        let intermediate_count = self.loaded_certificates.iter()
            .filter(|c| c.is_ca && c.issuer.is_some())
            .count();
        let leaf_count = self.loaded_certificates.iter()
            .filter(|c| !c.is_ca)
            .count();

        readiness.root_ca_count = root_ca_count;
        readiness.intermediate_ca_count = intermediate_count;
        readiness.leaf_cert_count = leaf_count;
        readiness.has_root_ca = root_ca_count > 0;

        if !readiness.has_root_ca {
            readiness.missing_items.push(ExportMissingItem {
                category: "PKI",
                description: "No Root CA certificate generated".to_string(),
                severity: MissingSeverity::Required,
                action: Some("Generate Root CA in Keys tab".to_string()),
            });
        }

        if intermediate_count == 0 && readiness.has_root_ca {
            readiness.missing_items.push(ExportMissingItem {
                category: "PKI",
                description: "No Intermediate CA certificates".to_string(),
                severity: MissingSeverity::Recommended,
                action: Some("Generate Intermediate CAs for departments".to_string()),
            });
        }

        // Check keys
        readiness.key_count = self.loaded_keys.len();
        if readiness.key_count == 0 && readiness.has_people {
            readiness.missing_items.push(ExportMissingItem {
                category: "Keys",
                description: "No cryptographic keys generated".to_string(),
                severity: MissingSeverity::Recommended,
                action: Some("Generate keys for people".to_string()),
            });
        }

        // Check locations
        readiness.has_locations = !self.loaded_locations.is_empty();
        if !readiness.has_locations {
            readiness.missing_items.push(ExportMissingItem {
                category: "Locations",
                description: "No storage locations defined".to_string(),
                severity: MissingSeverity::Recommended,
                action: Some("Add locations for key storage".to_string()),
            });
        }

        // Check NATS infrastructure
        readiness.has_nats_operator = self.nats_operator_id.is_some() || self.nats_hierarchy_generated;
        if self.include_nats_config && !readiness.has_nats_operator {
            readiness.missing_items.push(ExportMissingItem {
                category: "NATS",
                description: "NATS hierarchy not generated".to_string(),
                severity: MissingSeverity::Recommended,
                action: Some("Generate NATS hierarchy in Identity tab".to_string()),
            });
        }

        // Count NATS entities from graph using injection helpers
        for node in self.org_graph.nodes.values() {
            if node.injection().is_nats_account() {
                readiness.nats_account_count += 1;
            } else if node.injection().is_nats_user() {
                readiness.nats_user_count += 1;
            }
        }

        // Check policy data
        readiness.has_policy_data = self.policy_data.is_some();
        if let Some(ref policy) = self.policy_data {
            readiness.role_assignment_count = policy.role_assignments.len();
            // Count unique people with role assignments
            let people_with_roles: std::collections::HashSet<_> = policy.role_assignments.iter()
                .map(|a| a.person_id)
                .collect();
            readiness.people_with_roles = people_with_roles.len();

            if readiness.role_assignment_count == 0 && readiness.has_people {
                readiness.missing_items.push(ExportMissingItem {
                    category: "Policy",
                    description: "No role assignments defined".to_string(),
                    severity: MissingSeverity::Optional,
                    action: Some("Assign roles to people via drag-drop".to_string()),
                });
            }
        } else if readiness.has_people {
            readiness.missing_items.push(ExportMissingItem {
                category: "Policy",
                description: "Policy bootstrap not loaded".to_string(),
                severity: MissingSeverity::Optional,
                action: Some("Load policy-bootstrap.json".to_string()),
            });
        }

        // Check YubiKeys - consider provisioned if PIN has been changed from default
        readiness.yubikey_count = self.yubikey_configs.len();
        readiness.provisioned_yubikeys = self.yubikey_configs.iter()
            .filter(|yk| yk.piv.pin != yk.piv.default_pin)
            .count();

        if readiness.yubikey_count > 0 && readiness.provisioned_yubikeys == 0 {
            readiness.missing_items.push(ExportMissingItem {
                category: "YubiKey",
                description: format!("{} YubiKeys configured but none provisioned", readiness.yubikey_count),
                severity: MissingSeverity::Optional,
                action: Some("Provision YubiKeys with keys".to_string()),
            });
        }

        // Add warnings for optional but recommended items
        if people_count > 0 && readiness.key_count < people_count {
            readiness.warnings.push(format!(
                "Only {} keys for {} people - some people have no keys",
                readiness.key_count, people_count
            ));
        }

        if self.include_private_keys && self.export_password.is_empty() {
            readiness.warnings.push("Private key export selected but no password set".to_string());
        }

        readiness
    }

    fn view_export(&self) -> Element<'_, Message> {
        // Compute readiness status
        let readiness = self.compute_export_readiness();
        let is_ready = readiness.is_ready();
        let percentage = readiness.readiness_percentage();

        // Build status content separately to avoid opaque type conflicts
        let status_content = column![
            row![
                text(if is_ready { "✅" } else { "⚠️" }).font(EMOJI_FONT).size(20),
                text(format!("Export Readiness: {}%", percentage))
                    .size(self.view_model.text_large)
                    .color(if is_ready {
                        self.view_model.colors.green_success
                    } else {
                        self.view_model.colors.yellow_warning
                    }),
            ]
            .spacing(8)
            .align_y(Alignment::Center),

            // Progress bar - capture bar color for closure
            {
                let bar_color = if is_ready {
                    self.view_model.colors.green_success
                } else {
                    self.view_model.colors.yellow_warning
                };
                progress_bar(0.0..=100.0, percentage as f32)
                    .height(8)
                    .style(move |_theme| progress_bar::Style {
                        background: Background::Color(Color::from_rgba(0.2, 0.2, 0.3, 0.8)),
                        bar: Background::Color(bar_color),
                        border: Border::default(),
                    })
            },

            // Summary stats
            row![
                text(format!("📜 {} certs", readiness.root_ca_count + readiness.intermediate_ca_count + readiness.leaf_cert_count))
                    .size(self.view_model.text_small),
                text(format!("🔑 {} keys", readiness.key_count))
                    .size(self.view_model.text_small),
                text(format!("👥 {} people", self.loaded_people.len()))
                    .size(self.view_model.text_small),
                if readiness.has_nats_operator {
                    text(format!("🌐 NATS: {} accounts", readiness.nats_account_count))
                        .size(self.view_model.text_small)
                } else {
                    text("🌐 NATS: none")
                        .size(self.view_model.text_small)
                        .color(self.view_model.colors.text_tertiary)
                },
            ]
            .spacing(16),
        ]
        .spacing(8);

        // Readiness status card - use conditional rendering to avoid opaque type conflict
        let readiness_card: Element<'_, Message> = if is_ready {
            container(status_content)
                .padding(self.view_model.padding_lg)
                .style(CowboyCustomTheme::pastel_mint_card())
                .into()
        } else {
            container(status_content)
                .padding(self.view_model.padding_lg)
                .style(CowboyCustomTheme::pastel_coral_card())
                .into()
        };

        let content = column![
            text("Export Domain Configuration").size(self.view_model.text_xlarge),
            text("Export your domain configuration to encrypted storage").size(self.view_model.text_normal),

            readiness_card,

            // Missing items list (if any)
            if !readiness.missing_items.is_empty() {
                container(
                    column![
                        text(format!("Missing Items ({})", readiness.missing_items.len()))
                            .size(self.view_model.text_medium)
                            .color(CowboyTheme::text_primary()),
                        {
                            let mut missing_column = column![].spacing(6);
                            for item in &readiness.missing_items {
                                let (icon, color) = match item.severity {
                                    MissingSeverity::Required => ("🔴", self.view_model.colors.red_error),
                                    MissingSeverity::Recommended => ("🟡", self.view_model.colors.yellow_warning),
                                    MissingSeverity::Optional => ("🔵", self.view_model.colors.blue_info),
                                };
                                let mut item_row = row![
                                    text(icon).font(EMOJI_FONT).size(14),
                                    text(format!("[{}] {}", item.category, item.description))
                                        .size(self.view_model.text_small)
                                        .color(color),
                                ]
                                .spacing(8)
                                .align_y(Alignment::Center);

                                if let Some(ref action) = item.action {
                                    item_row = item_row.push(
                                        text(format!("→ {}", action))
                                            .size(self.view_model.text_tiny)
                                            .color(self.view_model.colors.text_tertiary)
                                    );
                                }
                                missing_column = missing_column.push(item_row);
                            }
                            missing_column
                        }
                    ]
                    .spacing(self.view_model.spacing_md)
                )
                .padding(self.view_model.padding_lg)
                .style(CowboyCustomTheme::pastel_cream_card())
            } else {
                container(
                    text("All requirements met - ready for export!")
                        .size(self.view_model.text_normal)
                        .color(self.view_model.colors.green_success)
                )
                .padding(self.view_model.padding_md)
            },

            // Warnings (if any)
            if !readiness.warnings.is_empty() {
                container(
                    column![
                        text("⚠️ Warnings").size(self.view_model.text_medium),
                        {
                            let mut warn_column = column![].spacing(4);
                            for warning in &readiness.warnings {
                                warn_column = warn_column.push(
                                    text(format!("• {}", warning))
                                        .size(self.view_model.text_small)
                                        .color(self.view_model.colors.yellow_warning)
                                );
                            }
                            warn_column
                        }
                    ]
                    .spacing(self.view_model.spacing_sm)
                )
                .padding(self.view_model.padding_md)
                .style(CowboyCustomTheme::pastel_cream_card())
            } else {
                container(Space::with_height(0))
            },

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
    use crate::events::DomainEvent;

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
                DomainEvent::Key(crate::events::KeyEvents::SshKeyGenerated(e)) => {
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
    use crate::events::DomainEvent;
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
            if let DomainEvent::Certificate(crate::events::CertificateEvents::CertificateGenerated(e)) = event {
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
/// Note: This is synchronous because iced's run_with() blocks until window closes
/// and manages its own async runtime internally
pub fn run(output_dir: String, config: Option<crate::config::Config>) -> iced::Result {
    // Load all 4 custom fonts:
    // 1. Rec Mono Linear - body text (monospace)
    // 2. Poller One - headings (display)
    // 3. Material Icons - UI icons
    // 4. Noto Color Emoji - emoji rendering
    application("CIM Keys", CimKeysApp::update, CimKeysApp::view)
        .subscription(|app| app.subscription())
        .theme(|app| app.theme())
        .font(include_bytes!("../assets/fonts/RecMonoLinear-Regular.ttf"))
        .font(include_bytes!("../assets/fonts/PollerOne-Regular.ttf"))
        .font(include_bytes!("../assets/fonts/MaterialIcons-Regular.ttf"))
        .font(include_bytes!("../assets/fonts/NotoColorEmoji.ttf"))
        .run_with(|| CimKeysApp::new(output_dir, config))
}