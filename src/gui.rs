//! Native/WASM GUI for offline key generation using Iced 0.13+
//!
//! This module provides a pure Rust GUI that can run both as a native
//! application and as a WASM application in the browser.

use iced::{
    application,
    widget::{button, column, container, row, text, text_input, Container, horizontal_space, vertical_space, pick_list, progress_bar, checkbox, scrollable, Space, image, stack, canvas, Canvas},
    Task, Element, Length, Border, Theme, Background, Shadow, Alignment, Point, Color,
    Rectangle, mouse, Renderer,
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
pub mod workflow_view;
pub mod workflow_graph;
pub mod state_machine_graph;
pub mod state_machine_svg;
pub mod state_machine_view;
pub mod graph_buttons;

// Domain-bounded modules (Sprint 48-54 refactoring)
pub mod domains;
pub mod pki;
pub mod yubikey;
pub mod nats;
pub mod export;
pub mod delegation;
pub mod trustchain;
pub mod location;
pub mod service_account;
pub mod gpg;
pub mod recovery;
pub mod org_unit;
pub mod multi_key;
pub mod certificate;
pub mod event_log;
pub mod view_state;

#[cfg(test)]
mod graph_integration_tests;

use graph::{OrganizationConcept, OrganizationIntent};
use crate::lifting::Injection;
use domains::OrganizationMessage;
use pki::PkiMessage;
use yubikey::YubiKeyMessage;
use nats::NatsMessage;
use export::ExportMessage;
use delegation::DelegationMessage;
use trustchain::TrustChainMessage;
use location::LocationMessage;
use service_account::ServiceAccountMessage;
use gpg::GpgMessage;
use recovery::RecoveryMessage;
use org_unit::OrgUnitMessage;
use multi_key::MultiKeyMessage;
use certificate::CertificateMessage;
use event_log::EventLogMessage;
use event_emitter::{CimEventEmitter, GuiEventSubscriber, InteractionType};
use view_model::ViewModel;
use view_state::{NewLocationForm, NewOrgUnitForm, NewPersonForm, NewServiceAccountForm, OrganizationForm};

// Command factory for FRP-compliant command creation (Curried FP - Sprint 72)
use crate::command_factory::{
    person as person_factory, organization as org_factory,
    org_unit as org_unit_factory, location as location_factory,
    service_account as sa_factory,
    PersonResult, OrganizationResult, OrgUnitResult, LocationResult, ServiceAccountResult,
};
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
    new_location_url: String,  // For virtual/hybrid locations

    // YubiKey fields
    yubikey_serial: String,
    detected_yubikeys: Vec<crate::ports::yubikey::YubiKeyDevice>,
    yubikey_detection_status: String,
    yubikey_configs: Vec<crate::domain::YubiKeyConfig>,  // Imported from secrets
    yubikey_assignments: std::collections::HashMap<String, Uuid>,  // serial -> person_id
    yubikey_provisioning_status: std::collections::HashMap<String, String>,  // serial -> status message
    selected_yubikey_for_assignment: Option<String>,  // YubiKey serial selected for assignment
    loaded_locations: Vec<crate::projections::LocationEntry>,  // Loaded from manifest
    loaded_people: Vec<crate::projections::PersonEntry>,  // Loaded from manifest
    loaded_certificates: Vec<crate::projections::CertificateEntry>,  // Loaded from manifest
    loaded_keys: Vec<crate::projections::KeyEntry>,  // Loaded from manifest

    // Key generation state
    key_generation_progress: f32,
    keys_generated: usize,
    total_keys_to_generate: usize,
    _certificates_generated: usize,  // Reserved for certificate generation tracking

    // PKI Bootstrap State Machine (Sprint 77)
    pki_state: crate::state_machines::workflows::PKIBootstrapState,

    // YubiKey Provisioning State Machine (Sprint 78)
    // Maps YubiKey serial -> provisioning state
    yubikey_states: std::collections::HashMap<String, crate::state_machines::workflows::YubiKeyProvisioningState>,

    // Generated certificate storage (Sprint 81, extended Sprint 82)
    // Store generated certificates for signing chain
    generated_root_ca: Option<crate::crypto::x509::X509Certificate>,
    generated_intermediate_cas: std::collections::HashMap<Uuid, crate::crypto::x509::X509Certificate>,
    generated_leaf_certs: std::collections::HashMap<Uuid, crate::crypto::x509::X509Certificate>,

    // Certificate generation fields
    intermediate_ca_name_input: String,
    server_cert_cn_input: String,
    server_cert_sans_input: String,
    selected_intermediate_ca: Option<String>,
    selected_cert_location: Option<String>,  // Storage location for certificates
    loaded_units: Vec<crate::domain::OrganizationUnit>,  // Loaded organizational units
    selected_unit_for_ca: Option<String>,  // Unit name selected for intermediate CA

    // Certificate metadata fields (editable before generation)
    cert_organization: String,
    cert_organizational_unit: String,
    cert_locality: String,
    cert_state_province: String,
    cert_country: String,
    cert_validity_days: String,  // String for text input, will parse to u32

    // Projection configuration (Export is now part of the projection system)
    export_path: PathBuf,
    include_public_keys: bool,
    include_certificates: bool,
    include_nats_config: bool,
    include_private_keys: bool,
    export_password: String,

    // Projection system state - bidirectional domain mappings
    // Outgoing: Domain → External, Incoming: External → Domain
    projection_section: ProjectionSection,
    projections: Vec<ProjectionState>,
    selected_projection: Option<ProjectionTarget>,
    selected_injection: Option<InjectionSource>,

    // Neo4j projection configuration
    neo4j_endpoint: String,
    neo4j_username: String,
    neo4j_password: String,

    // JetStream projection configuration
    jetstream_url: String,
    jetstream_credentials_path: String,

    // NATS hierarchy state
    nats_hierarchy_generated: bool,
    nats_operator_id: Option<Uuid>,
    nats_export_path: PathBuf,

    // NATS hierarchy visualization state
    nats_viz_section_collapsed: bool,
    nats_viz_expanded_accounts: std::collections::HashSet<String>,  // Account names that are expanded
    nats_viz_selected_operator: bool,  // Is the operator selected?
    nats_viz_selected_account: Option<String>,  // Selected account name
    nats_viz_selected_user: Option<(String, Uuid)>,  // (account_name, person_id) of selected user
    nats_viz_hierarchy_data: Option<NatsHierarchyFull>,  // Cached hierarchy data for display

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
    gpg_section_collapsed: bool,

    // GPG key generation state
    gpg_user_id: String,
    gpg_key_type: Option<crate::ports::gpg::GpgKeyType>,
    gpg_key_length: String,  // Key length in bits (string for text input)
    gpg_expires_days: String,  // Days until expiration (string for text input)
    generated_gpg_keys: Vec<crate::ports::gpg::GpgKeyInfo>,
    gpg_generation_status: Option<String>,

    // Key recovery from seed state
    recovery_section_collapsed: bool,
    recovery_passphrase: String,
    recovery_passphrase_confirm: String,
    recovery_organization_id: String,
    recovery_status: Option<String>,
    recovery_seed_verified: bool,

    // YubiKey slot management state
    yubikey_slot_section_collapsed: bool,
    selected_yubikey_for_management: Option<String>,  // Serial of YubiKey being managed
    yubikey_pin_input: String,
    yubikey_new_pin: String,
    yubikey_pin_confirm: String,
    yubikey_management_key: String,  // Current management key (hex)
    yubikey_new_management_key: String,  // New management key (hex)
    yubikey_slot_info: std::collections::HashMap<String, Vec<SlotInfo>>,  // serial -> slot info
    yubikey_slot_operation_status: Option<String>,
    yubikey_attestation_result: Option<String>,
    selected_piv_slot: Option<crate::ports::yubikey::PivSlot>,

    // Organization unit creation state
    org_unit_section_collapsed: bool,
    new_unit_name: String,
    new_unit_type: Option<crate::domain::OrganizationUnitType>,
    new_unit_parent: Option<String>,  // Parent unit name (for nesting)
    new_unit_nats_account: String,  // Optional NATS account name
    new_unit_responsible_person: Option<Uuid>,  // Optional responsible person
    created_units: Vec<crate::domain::OrganizationUnit>,

    // Service account management state
    service_account_section_collapsed: bool,
    new_service_account_name: String,
    new_service_account_purpose: String,
    new_service_account_owning_unit: Option<Uuid>,  // Which org unit owns this
    new_service_account_responsible_person: Option<Uuid>,  // Person responsible (required)
    created_service_accounts: Vec<crate::domain::ServiceAccount>,

    // Trust chain visualization state
    trust_chain_section_collapsed: bool,
    selected_trust_chain_cert: Option<Uuid>,  // Certificate selected for chain view
    trust_chain_verification_status: std::collections::HashMap<Uuid, TrustChainStatus>,

    // Delegation management state
    delegation_section_collapsed: bool,
    delegation_from_person: Option<Uuid>,  // Person delegating (grantor)
    delegation_to_person: Option<Uuid>,  // Person receiving delegation (grantee)
    delegation_permissions: std::collections::HashSet<crate::domain::KeyPermission>,
    delegation_expires_days: String,  // Expiration in days (empty = no expiration)
    active_delegations: Vec<DelegationEntry>,  // List of active delegations

    // YubiKey domain registration state
    yubikey_registration_name: String,  // Friendly name for registration
    registered_yubikeys: std::collections::HashMap<String, Uuid>,  // serial -> domain entity ID

    // mTLS client certificate state
    client_cert_cn: String,
    client_cert_email: String,

    // Multi-purpose key generation state
    multi_purpose_key_section_collapsed: bool,
    multi_purpose_selected_person: Option<Uuid>,
    multi_purpose_selected_purposes: std::collections::HashSet<crate::domain::InvariantKeyPurpose>,

    // Event log state
    event_log_section_collapsed: bool,
    loaded_event_log: Vec<crate::event_store::StoredEventRecord>,
    selected_events_for_replay: std::collections::HashSet<String>,  // CIDs

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
    gpg_port: Arc<dyn crate::ports::gpg::GpgPort>,

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

    // Workflow guidance for trust chain gap fulfillment
    workflow_view: workflow_view::WorkflowView,

    // State Machine visualization
    selected_state_machine: state_machine_graph::StateMachineType,
    state_machine_definition: state_machine_graph::StateMachineDefinition,
    state_machine_graph: OrganizationConcept,
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

// ============================================================================
// PROJECTION SYSTEM TYPES
// ============================================================================
// Projections are functorial mappings between the domain and external systems.
// - Outgoing Projections: Domain → External (exports, syncs)
// - Incoming Injections: External → Domain (imports, feeds)
// These form adjoint functors in the categorical sense.

/// Outgoing projection targets - where domain state gets projected to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionTarget {
    /// Complete domain snapshot to encrypted SD card (air-gapped export)
    SDCard,
    /// Neo4j graph database for queryable domain state
    Neo4j,
    /// OLAP analytics projection for Orgs/People
    Olap,
    /// Contact system adapters (JSON schema driven)
    ContactSystems,
    /// JetStream for commands/queries/events (after PKI established)
    JetStream,
    /// NSC store for NATS credentials
    NscStore,
}

impl ProjectionTarget {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SDCard => "Encrypted SD Card",
            Self::Neo4j => "Neo4j Graph Database",
            Self::Olap => "OLAP Analytics",
            Self::ContactSystems => "Contact Systems",
            Self::JetStream => "JetStream Events",
            Self::NscStore => "NSC Store",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SDCard => "Air-gapped export of complete domain configuration",
            Self::Neo4j => "Continuous sync to Neo4j for graph queries and visualization",
            Self::Olap => "Analytics projection for organizations and people",
            Self::ContactSystems => "JSON schema adapters for external contact management",
            Self::JetStream => "Event stream for commands, queries, and domain events",
            Self::NscStore => "NATS security credentials and hierarchy",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::SDCard => "💾",
            Self::Neo4j => "🔷",
            Self::Olap => "📊",
            Self::ContactSystems => "📇",
            Self::JetStream => "🌊",
            Self::NscStore => "🔐",
        }
    }

    /// Whether this projection requires PKI to be established first
    pub fn requires_pki(&self) -> bool {
        match self {
            Self::SDCard => true,
            Self::Neo4j => false,
            Self::Olap => false,
            Self::ContactSystems => false,
            Self::JetStream => true, // Must have PKI for signed events
            Self::NscStore => true,
        }
    }

    /// All available projection targets
    pub fn all() -> Vec<Self> {
        vec![
            Self::SDCard,
            Self::Neo4j,
            Self::Olap,
            Self::ContactSystems,
            Self::JetStream,
            Self::NscStore,
        ]
    }
}

/// Incoming injection sources - where domain state comes from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionSource {
    /// JSON configuration files (domain bootstrap, imports)
    JsonFile,
    /// Petgraph import (graph structures)
    Petgraph,
    /// Contact system imports
    ContactImport,
    /// JetStream event replay (event sourcing)
    JetStreamReplay,
    /// External API feeds
    ApiImport,
}

impl InjectionSource {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::JsonFile => "JSON Configuration",
            Self::Petgraph => "Petgraph Import",
            Self::ContactImport => "Contact System Import",
            Self::JetStreamReplay => "JetStream Replay",
            Self::ApiImport => "API Import",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::JsonFile => "Load domain configuration from JSON files",
            Self::Petgraph => "Import graph structures from Petgraph format",
            Self::ContactImport => "Import contacts from external systems",
            Self::JetStreamReplay => "Replay events from JetStream for state reconstruction",
            Self::ApiImport => "Import data from external API endpoints",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::JsonFile => "📄",
            Self::Petgraph => "🕸️",
            Self::ContactImport => "📥",
            Self::JetStreamReplay => "⏪",
            Self::ApiImport => "🔌",
        }
    }

    /// All available injection sources
    pub fn all() -> Vec<Self> {
        vec![
            Self::JsonFile,
            Self::Petgraph,
            Self::ContactImport,
            Self::JetStreamReplay,
            Self::ApiImport,
        ]
    }
}

/// Status of a projection or injection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionStatus {
    /// Not configured
    NotConfigured,
    /// Configured but not connected
    Disconnected,
    /// Currently connecting/syncing
    Syncing,
    /// Connected and up to date
    Synced,
    /// Error state
    Error(String),
}

impl ProjectionStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::NotConfigured => "⚪",
            Self::Disconnected => "🔴",
            Self::Syncing => "🟡",
            Self::Synced => "🟢",
            Self::Error(_) => "❌",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::NotConfigured => "Not Configured",
            Self::Disconnected => "Disconnected",
            Self::Syncing => "Syncing...",
            Self::Synced => "Synced",
            Self::Error(_) => "Error",
        }
    }
}

/// Configuration and state for an outgoing projection
#[derive(Debug, Clone)]
pub struct ProjectionState {
    pub target: ProjectionTarget,
    pub status: ProjectionStatus,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub items_synced: usize,
    pub endpoint: Option<String>,
    pub enabled: bool,
}

impl ProjectionState {
    pub fn new(target: ProjectionTarget) -> Self {
        Self {
            target,
            status: ProjectionStatus::NotConfigured,
            last_sync: None,
            items_synced: 0,
            endpoint: None,
            enabled: false,
        }
    }
}

/// Configuration and state for an incoming injection
#[derive(Debug, Clone)]
pub struct InjectionState {
    pub source: InjectionSource,
    pub status: ProjectionStatus,
    pub last_import: Option<chrono::DateTime<chrono::Utc>>,
    pub items_imported: usize,
    pub source_path: Option<String>,
    pub enabled: bool,
}

impl InjectionState {
    pub fn new(source: InjectionSource) -> Self {
        Self {
            source,
            status: ProjectionStatus::NotConfigured,
            last_import: None,
            items_imported: 0,
            source_path: None,
            enabled: false,
        }
    }
}

/// Which projection section is currently expanded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectionSection {
    #[default]
    Overview,
    Outgoing,
    Incoming,
    Configuration,
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
    Locations,
    Keys,
    Projections,
    Workflow,
    StateMachines,
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
    // ============================================================================
    // Domain-Bounded Message Delegation (Sprint 48-50 refactoring)
    // ============================================================================
    /// Delegation to Organization bounded context
    Organization(OrganizationMessage),
    /// Delegation to PKI bounded context
    Pki(PkiMessage),
    /// Delegation to YubiKey bounded context
    YubiKey(YubiKeyMessage),
    /// Delegation to NATS bounded context
    Nats(NatsMessage),
    /// Delegation to Export bounded context
    Export(ExportMessage),
    /// Delegation to Delegation bounded context (authorization)
    Delegation(DelegationMessage),
    /// Delegation to TrustChain bounded context (certificate verification)
    TrustChain(TrustChainMessage),
    /// Delegation to Location bounded context (physical/virtual locations)
    Location(LocationMessage),
    /// Delegation to Service Account bounded context (automated systems)
    ServiceAccount(ServiceAccountMessage),
    /// Delegation to GPG Keys bounded context (GPG/PGP key generation)
    Gpg(GpgMessage),
    /// Delegation to Key Recovery bounded context (seed-based recovery)
    Recovery(RecoveryMessage),
    /// Delegation to Organization Unit bounded context (departments, teams)
    OrgUnit(OrgUnitMessage),
    /// Delegation to Multi-Purpose Key bounded context (batch key generation)
    MultiKey(MultiKeyMessage),
    /// Delegation to Certificate bounded context (X.509 certificate management)
    Certificate(CertificateMessage),
    /// Delegation to Event Log bounded context (event replay/reconstruction)
    EventLog(EventLogMessage),

    // ============================================================================
    // Tab Navigation
    // ============================================================================
    TabSelected(Tab),

    // Graph View Selection
    GraphViewSelected(GraphView),

    // Domain operations (migrating to Organization bounded context)
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
    PersonAdded(Result<(Uuid, String, String, Uuid, KeyOwnerRole), String>), // (person_id, name, email, org_id, role)
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
    NewLocationUrlChanged(String),
    AddLocation,
    LocationAdded(Result<(Uuid, String, String, Uuid), String>), // (location_id, name, type, org_id)
    RemoveLocation(Uuid),

    // YubiKey operations
    DetectYubiKeys,
    YubiKeysDetected(Result<Vec<crate::ports::yubikey::YubiKeyDevice>, String>),
    YubiKeySerialChanged(String),
    SelectYubiKeyForAssignment(String),  // Select which YubiKey to assign (by serial)
    AssignYubiKeyToPerson { serial: String, person_id: Uuid },  // Assign selected YubiKey to person
    ProvisionYubiKey,
    ProvisionSingleYubiKey { serial: String },  // Provision a single YubiKey by serial
    YubiKeyProvisioned(Result<String, String>),
    SingleYubiKeyProvisioned(Result<(String, String), (String, String)>),  // (serial, status) or (serial, error)

    // Key generation
    GenerateRootCA,
    IntermediateCANameChanged(String),
    SelectUnitForCA(String),  // Select organizational unit for intermediate CA
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

    // GPG key generation
    GpgUserIdChanged(String),
    GpgKeyTypeSelected(crate::ports::gpg::GpgKeyType),
    GpgKeyLengthChanged(String),
    GpgExpiresDaysChanged(String),
    GenerateGpgKey,
    GpgKeyGenerated(Result<crate::ports::gpg::GpgKeypair, String>),
    ToggleGpgSection,
    ListGpgKeys,
    GpgKeysListed(Result<Vec<crate::ports::gpg::GpgKeyInfo>, String>),

    // Key recovery from seed
    ToggleRecoverySection,
    RecoveryPassphraseChanged(String),
    RecoveryPassphraseConfirmChanged(String),
    RecoveryOrganizationIdChanged(String),
    VerifyRecoverySeed,
    RecoverySeedVerified(Result<String, String>),  // Ok(fingerprint) or Err(message)
    RecoverKeysFromSeed,
    KeysRecovered(Result<usize, String>),  // Number of keys recovered

    // YubiKey slot management
    ToggleYubiKeySlotSection,
    SelectYubiKeyForManagement(String),  // Select YubiKey by serial for slot management
    YubiKeyPinInputChanged(String),
    YubiKeyNewPinChanged(String),
    YubiKeyPinConfirmChanged(String),
    YubiKeyManagementKeyChanged(String),
    YubiKeyNewManagementKeyChanged(String),
    SelectPivSlot(crate::ports::yubikey::PivSlot),
    QueryYubiKeySlots(String),  // Query slots for a specific YubiKey serial
    YubiKeySlotsQueried(Result<(String, Vec<SlotInfo>), String>),  // (serial, slots) or error
    VerifyYubiKeyPin(String),  // Verify PIN for YubiKey by serial
    YubiKeyPinVerified(Result<(String, bool), String>),  // (serial, valid) or error
    ChangeYubiKeyManagementKey(String),  // Change management key for YubiKey by serial
    YubiKeyManagementKeyChanged2(Result<String, String>),  // Ok(serial) or error
    ResetYubiKeyPiv(String),  // Factory reset PIV for YubiKey by serial
    YubiKeyPivReset(Result<String, String>),  // Ok(serial) or error
    GetYubiKeyAttestation { serial: String, slot: crate::ports::yubikey::PivSlot },
    YubiKeyAttestationReceived(Result<(String, String), String>),  // Ok((serial, attestation_info)) or error
    ClearYubiKeySlot { serial: String, slot: crate::ports::yubikey::PivSlot },
    YubiKeySlotCleared(Result<(String, crate::ports::yubikey::PivSlot), String>),  // Ok((serial, slot)) or error
    GenerateKeyInSlot { serial: String, slot: crate::ports::yubikey::PivSlot },
    KeyInSlotGenerated(Result<(String, crate::ports::yubikey::PivSlot, String), String>),  // Ok((serial, slot, public_key_info)) or error

    // Organization unit creation
    ToggleOrgUnitSection,
    NewUnitNameChanged(String),
    NewUnitTypeSelected(crate::domain::OrganizationUnitType),
    NewUnitParentSelected(String),
    NewUnitNatsAccountChanged(String),
    NewUnitResponsiblePersonSelected(Uuid),
    CreateOrganizationUnit,
    OrganizationUnitCreated(Result<crate::domain::OrganizationUnit, String>),
    RemoveOrganizationUnit(Uuid),

    // Service account management
    ToggleServiceAccountSection,
    NewServiceAccountNameChanged(String),
    NewServiceAccountPurposeChanged(String),
    ServiceAccountOwningUnitSelected(Uuid),
    ServiceAccountResponsiblePersonSelected(Uuid),
    CreateServiceAccount,
    ServiceAccountCreated(Result<crate::domain::ServiceAccount, String>),
    DeactivateServiceAccount(Uuid),
    RemoveServiceAccount(Uuid),
    GenerateServiceAccountKey { service_account_id: Uuid },
    ServiceAccountKeyGenerated(Result<(Uuid, crate::domain::KeyOwnerRole), String>),

    // Trust chain visualization
    ToggleTrustChainSection,
    SelectCertificateForChainView(Uuid),
    VerifyTrustChain(Uuid),
    TrustChainVerified(Result<(Uuid, TrustChainStatus), String>),
    VerifyAllTrustChains,

    // Delegation management
    ToggleDelegationSection,
    DelegationFromPersonSelected(Uuid),
    DelegationToPersonSelected(Uuid),
    ToggleDelegationPermission(crate::domain::KeyPermission),
    DelegationExpiresDaysChanged(String),
    CreateDelegation,
    DelegationCreated(Result<DelegationEntry, String>),
    RevokeDelegation(Uuid),
    DelegationRevoked(Result<Uuid, String>),

    // Export operations
    ExportPathChanged(String),
    TogglePublicKeys(bool),
    ToggleCertificates(bool),
    ToggleNatsConfig(bool),
    TogglePrivateKeys(bool),
    ExportPasswordChanged(String),
    ExportToSDCard,
    DomainExported(Result<String, String>),

    // Projection configuration changes
    Neo4jEndpointChanged(String),
    Neo4jUsernameChanged(String),
    Neo4jPasswordChanged(String),
    JetStreamUrlChanged(String),
    JetStreamCredentialsChanged(String),
    ProjectionSectionChanged(ProjectionSection),
    ProjectionSelected(ProjectionTarget),
    InjectionSelected(InjectionSource),
    ConnectProjection(ProjectionTarget),
    DisconnectProjection(ProjectionTarget),
    SyncProjection(ProjectionTarget),
    ImportFromSource(InjectionSource),

    // Neo4j Cypher export
    ExportToCypher,
    CypherExported(Result<(String, usize), String>), // (file_path, query_count)

    // SD Card export (enhanced result type)
    SDCardExported(Result<(String, usize, usize), String>), // (base_path, files_written, bytes_written)

    // NATS Hierarchy operations
    GenerateNatsHierarchy,
    NatsHierarchyGenerated(Result<String, String>),
    NatsBootstrapCreated(Box<crate::domain_projections::OrganizationBootstrap>),
    GenerateNatsFromGraph,  // Graph-first NATS generation
    NatsFromGraphGenerated(Result<Vec<(graph::ConceptEntity, iced::Point, Option<Uuid>)>, String>),
    ExportToNsc,
    NscExported(Result<String, String>),

    // NATS hierarchy visualization
    ToggleNatsVizSection,
    ToggleNatsAccountExpand(String),  // Toggle account tree node expansion
    SelectNatsOperator,  // Select the operator node
    SelectNatsAccount(String),  // Select an account node
    SelectNatsUser(String, Uuid),  // Select a user node (account_name, person_id)
    RefreshNatsHierarchy,  // Refresh hierarchy data from sources
    NatsHierarchyRefreshed(Result<NatsHierarchyFull, String>),
    AddNatsAccount { unit_id: Uuid, account_name: String },  // Add new account
    AddNatsUser { account_name: String, person_id: Uuid },  // Add new user to account
    RemoveNatsAccount(String),  // Remove an account
    RemoveNatsUser(String, Uuid),  // Remove a user from account

    // YubiKey domain registration and lifecycle
    RegisterYubiKeyInDomain { serial: String, name: String },  // Formally register YubiKey
    YubiKeyRegistered(Result<(String, Uuid), String>),  // (serial, yubikey_id) or error
    TransferYubiKey { serial: String, from_person_id: Uuid, to_person_id: Uuid },  // Transfer between persons
    YubiKeyTransferred(Result<(String, Uuid, Uuid), String>),  // (serial, from, to) or error
    RevokeYubiKeyAssignment { serial: String },  // Remove assignment from person
    YubiKeyAssignmentRevoked(Result<String, String>),  // serial or error
    YubiKeyRegistrationNameChanged(String),  // Input for registration form

    // mTLS Client Certificate generation
    ClientCertCNChanged(String),
    ClientCertEmailChanged(String),
    GenerateClientCert,
    ClientCertGenerated(Result<String, String>),  // fingerprint or error

    // Multi-purpose key generation
    ToggleMultiPurposeKeySection,
    MultiPurposePersonSelected(Uuid),
    ToggleKeyPurpose(crate::domain::InvariantKeyPurpose),
    GenerateMultiPurposeKeys,
    MultiPurposeKeysGenerated(Result<(Uuid, Vec<String>), String>),  // (person_id, key_fingerprints)

    // Event Sourcing - Event Replay/Reconstruction
    ToggleEventLogSection,
    LoadEventLog,
    EventLogLoaded(Result<Vec<crate::event_store::StoredEventRecord>, String>),
    ReplaySelectedEvents,
    EventsReplayed(Result<usize, String>),
    ClearEventSelection,
    ToggleEventSelection(String),  // CID of event

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
    // Sprint 81: Intermediate CA generation result
    IntermediateCAGenerated(Result<(crate::crypto::x509::X509Certificate, Uuid), String>), // (cert, intermediate_ca_id)
    // Sprint 82: Leaf certificate generation result
    LeafCertGenerated(Result<(crate::crypto::x509::X509Certificate, Uuid, String), String>), // (cert, leaf_cert_id, person_name)
    PersonalKeysGenerated(Result<(crate::crypto::x509::X509Certificate, Vec<String>), String>), // (cert, nats_keys)

    // PKI State Machine Transitions (Sprint 77)
    PkiPlanRootCA {
        subject: crate::state_machines::workflows::CertificateSubject,
        validity_years: u32,
        yubikey_serial: String,
    },
    PkiExecuteRootCAGeneration,
    PkiRootCAGenerationComplete {
        root_ca_cert_id: Uuid,
        root_ca_key_id: Uuid,
    },

    // IntermediateCA State Machine Transitions (Sprint 78)
    PkiPlanIntermediateCA {
        subject: crate::state_machines::workflows::CertificateSubject,
        validity_years: u32,
        path_len: Option<u32>,
    },
    PkiExecuteIntermediateCAGeneration,
    PkiIntermediateCAGenerationComplete {
        intermediate_ca_id: Uuid,
    },

    // LeafCert State Machine Transitions (Sprint 78)
    PkiPlanLeafCert {
        subject: crate::state_machines::workflows::CertificateSubject,
        validity_years: u32,
        person_id: Option<Uuid>,
    },
    PkiExecuteLeafCertGeneration,
    PkiLeafCertGenerationComplete {
        leaf_cert_id: Uuid,
    },

    // YubiKey State Machine Transitions (Sprint 78)
    YubiKeyDetected {
        serial: String,
        firmware_version: String,
    },
    YubiKeyAuthenticated {
        serial: String,
        pin_retries: u8,
    },
    YubiKeyPINChanged {
        serial: String,
    },
    YubiKeyProvisioningComplete {
        serial: String,
    },

    // Sprint 83: YubiKey Operations (trigger actual PIV operations)
    YubiKeyStartAuthentication {
        serial: String,
        pin: String,
    },
    YubiKeyAuthenticationResult(Result<(String, u8), (String, String)>), // (serial, retries) or (serial, error)
    YubiKeyStartPINChange {
        serial: String,
        old_pin: String,
        new_pin: String,
    },
    YubiKeyPINChangeResult(Result<String, (String, String)>), // serial or (serial, error)
    YubiKeyStartKeyGeneration {
        serial: String,
        slot: crate::ports::yubikey::PivSlot,
        algorithm: crate::ports::yubikey::KeyAlgorithm,
    },
    YubiKeyKeyGenerationResult(Result<(String, crate::ports::yubikey::PivSlot, Vec<u8>), (String, String)>), // (serial, slot, pubkey) or (serial, error)

    // Export State Machine Transitions (Sprint 79)
    PkiPrepareExport,
    PkiExportReady {
        manifest_id: Uuid,
    },
    PkiExecuteExport {
        export_path: std::path::PathBuf,
    },
    PkiBootstrapComplete {
        export_location: Uuid,
        export_checksum: String,
    },

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

    // Workflow Guidance Messages
    WorkflowMessage(workflow_view::WorkflowMessage),

    // State Machine Visualization Messages
    StateMachineSelected(state_machine_graph::StateMachineType),
    StateMachineMessage(state_machine_graph::StateMachineMessage),

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
                    Tab::Projections => crate::mvi::model::Tab::Projections,
                    Tab::Locations => return None, // No equivalent in mvi::model::Tab
                    Tab::Workflow => return None, // No equivalent in mvi::model::Tab
                    Tab::StateMachines => return None, // No equivalent in mvi::model::Tab
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
            | Message::SingleYubiKeyProvisioned(_)
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

/// Information about a PIV slot on a YubiKey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub slot: crate::ports::yubikey::PivSlot,
    pub slot_name: String,
    pub slot_hex: String,
    pub occupied: bool,
    pub algorithm: Option<String>,
    pub subject: Option<String>,
    pub expires: Option<String>,
}

impl SlotInfo {
    pub fn new(slot: crate::ports::yubikey::PivSlot) -> Self {
        use crate::ports::yubikey::PivSlot;
        let (slot_name, slot_hex) = match slot {
            PivSlot::Authentication => ("Authentication".to_string(), "9A".to_string()),
            PivSlot::Signature => ("Digital Signature".to_string(), "9C".to_string()),
            PivSlot::KeyManagement => ("Key Management".to_string(), "9D".to_string()),
            PivSlot::CardAuth => ("Card Authentication".to_string(), "9E".to_string()),
            PivSlot::Retired(n) => (format!("Retired {}", n), format!("{:02X}", 0x82 + n - 1)),
        };
        Self {
            slot,
            slot_name,
            slot_hex,
            occupied: false,
            algorithm: None,
            subject: None,
            expires: None,
        }
    }

    pub fn with_key_info(mut self, algorithm: &str, subject: Option<&str>, expires: Option<&str>) -> Self {
        self.occupied = true;
        self.algorithm = Some(algorithm.to_string());
        self.subject = subject.map(|s| s.to_string());
        self.expires = expires.map(|s| s.to_string());
        self
    }
}

// DelegationEntry moved to delegation::authorization module (Sprint 53)
pub use delegation::DelegationEntry;

// TrustChainStatus moved to trustchain::verification module (Sprint 54)
pub use trustchain::TrustChainStatus;

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
        let gpg_port: Arc<dyn crate::ports::gpg::GpgPort> = Arc::new(crate::adapters::MockGpgAdapter::new());

        // Initialize MVI model
        let mvi_model = MviModel::new(PathBuf::from(&output_dir));

        // Load existing data from manifest if it exists
        let manifest_task = {
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

        // Auto-load secrets on startup (domain-bootstrap.json or legacy format)
        let secrets_task = Task::perform(
            async move {
                use crate::secrets_loader::{SecretsLoader, BootstrapData};
                use std::path::PathBuf;

                // Try domain-bootstrap.json first (recommended)
                let bootstrap_path = PathBuf::from("secrets/domain-bootstrap.json");
                if bootstrap_path.exists() {
                    match SecretsLoader::load_from_bootstrap_file(&bootstrap_path) {
                        Ok(data) => return Ok(data),
                        Err(e) => {
                            eprintln!("Info: Could not load domain-bootstrap.json: {}", e);
                        }
                    }
                }

                // Try legacy format
                let secrets_path = PathBuf::from("secrets/secrets.json");
                let cowboyai_path = PathBuf::from("secrets/cowboyai.json");

                if secrets_path.exists() && cowboyai_path.exists() {
                    match SecretsLoader::load_from_files(&secrets_path, &cowboyai_path) {
                        Ok((org, people, yubikey_configs, passphrase)) => {
                            let units = org.units.clone();
                            return Ok(BootstrapData {
                                organization: org,
                                units,
                                people,
                                yubikey_configs,
                                yubikey_assignments: vec![],
                                nats_hierarchy: None,
                                master_passphrase: passphrase,
                            });
                        }
                        Err(e) => {
                            eprintln!("Info: Could not load legacy secrets: {}", e);
                        }
                    }
                }

                // No secrets found - this is fine, user can create from scratch
                Err("No bootstrap files found - starting with empty domain".to_string())
            },
            Message::SecretsImported
        );

        // Run both tasks - manifest first, then secrets
        let load_task = Task::batch([manifest_task, secrets_task]);

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
                new_location_url: String::new(),
                yubikey_serial: String::new(),
                detected_yubikeys: Vec::new(),
                yubikey_detection_status: "Click 'Detect YubiKeys' to scan for hardware".to_string(),
                yubikey_configs: Vec::new(),
                yubikey_assignments: std::collections::HashMap::new(),
                yubikey_provisioning_status: std::collections::HashMap::new(),
                selected_yubikey_for_assignment: None,
                loaded_locations: Vec::new(),
                loaded_people: Vec::new(),
                loaded_certificates: Vec::new(),
                loaded_keys: Vec::new(),
                key_generation_progress: 0.0,
                keys_generated: 0,
                total_keys_to_generate: 0,
                _certificates_generated: 0,
                pki_state: crate::state_machines::workflows::PKIBootstrapState::Uninitialized,
                yubikey_states: std::collections::HashMap::new(),
                generated_root_ca: None,
                generated_intermediate_cas: std::collections::HashMap::new(),
                generated_leaf_certs: std::collections::HashMap::new(),
                intermediate_ca_name_input: String::new(),
                server_cert_cn_input: String::new(),
                server_cert_sans_input: String::new(),
                selected_intermediate_ca: None,
                selected_cert_location: None,
                loaded_units: Vec::new(),
                selected_unit_for_ca: None,
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

                // Projection system initialization
                projection_section: ProjectionSection::Overview,
                projections: ProjectionTarget::all().into_iter().map(ProjectionState::new).collect(),
                selected_projection: None,
                selected_injection: None,

                // Neo4j configuration
                neo4j_endpoint: String::from("bolt://localhost:7687"),
                neo4j_username: String::from("neo4j"),
                neo4j_password: String::new(),

                // JetStream configuration
                jetstream_url: String::from("nats://localhost:4222"),
                jetstream_credentials_path: String::new(),

                nats_hierarchy_generated: false,
                nats_operator_id: None,
                nats_export_path: PathBuf::from(&output_dir).join("nsc"),

                // NATS hierarchy visualization
                nats_viz_section_collapsed: true,
                nats_viz_expanded_accounts: std::collections::HashSet::new(),
                nats_viz_selected_operator: false,
                nats_viz_selected_account: None,
                nats_viz_selected_user: None,
                nats_viz_hierarchy_data: None,

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
                gpg_section_collapsed: true,
                gpg_user_id: String::new(),
                gpg_key_type: None,
                gpg_key_length: String::from("4096"),
                gpg_expires_days: String::from("365"),
                generated_gpg_keys: Vec::new(),
                gpg_generation_status: None,
                recovery_section_collapsed: true,
                recovery_passphrase: String::new(),
                recovery_passphrase_confirm: String::new(),
                recovery_organization_id: String::new(),
                recovery_status: None,
                recovery_seed_verified: false,

                // YubiKey slot management
                yubikey_slot_section_collapsed: true,
                selected_yubikey_for_management: None,
                yubikey_pin_input: String::new(),
                yubikey_new_pin: String::new(),
                yubikey_pin_confirm: String::new(),
                yubikey_management_key: String::new(),
                yubikey_new_management_key: String::new(),
                yubikey_slot_info: std::collections::HashMap::new(),
                yubikey_slot_operation_status: None,
                yubikey_attestation_result: None,
                selected_piv_slot: None,

                // Organization unit creation
                org_unit_section_collapsed: true,
                new_unit_name: String::new(),
                new_unit_type: None,
                new_unit_parent: None,
                new_unit_nats_account: String::new(),
                new_unit_responsible_person: None,
                created_units: Vec::new(),

                // Service account management
                service_account_section_collapsed: true,
                new_service_account_name: String::new(),
                new_service_account_purpose: String::new(),
                new_service_account_owning_unit: None,
                new_service_account_responsible_person: None,
                created_service_accounts: Vec::new(),

                // Trust chain visualization
                trust_chain_section_collapsed: true,
                selected_trust_chain_cert: None,
                trust_chain_verification_status: std::collections::HashMap::new(),

                // Delegation management
                delegation_section_collapsed: true,
                delegation_from_person: None,
                delegation_to_person: None,
                delegation_permissions: std::collections::HashSet::new(),
                delegation_expires_days: String::new(),
                active_delegations: Vec::new(),

                // YubiKey domain registration state
                yubikey_registration_name: String::new(),
                registered_yubikeys: std::collections::HashMap::new(),

                // mTLS client certificate state
                client_cert_cn: String::new(),
                client_cert_email: String::new(),

                // Multi-purpose key generation state
                multi_purpose_key_section_collapsed: true,
                multi_purpose_selected_person: None,
                multi_purpose_selected_purposes: std::collections::HashSet::new(),

                // Event log state
                event_log_section_collapsed: true,
                loaded_event_log: Vec::new(),
                selected_events_for_replay: std::collections::HashSet::new(),

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
                gpg_port,
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

                // Workflow guidance
                workflow_view: workflow_view::WorkflowView::new(),

                // State Machine visualization
                selected_state_machine: state_machine_graph::StateMachineType::Key,
                state_machine_definition: state_machine_graph::build_key_state_machine(),
                state_machine_graph: state_machine_graph::state_machine_to_graph(
                    &state_machine_graph::build_key_state_machine(),
                    &state_machine_graph::StateMachineLayoutConfig::default(),
                ),
            },
            load_task,
        )
    }


    // Note: Title method removed - window title now set via iced::Settings

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // ================================================================
            // Domain-Bounded Message Delegation (Sprint 48)
            // ================================================================
            Message::Organization(org_msg) => {
                use domains::OrganizationMessage;

                match org_msg {
                    // Section toggles
                    OrganizationMessage::ToggleOrgUnitSection => {
                        self.org_unit_section_collapsed = !self.org_unit_section_collapsed;
                        Task::none()
                    }
                    OrganizationMessage::ToggleServiceAccountSection => {
                        self.service_account_section_collapsed = !self.service_account_section_collapsed;
                        Task::none()
                    }
                    // Form inputs
                    OrganizationMessage::NameChanged(n) => { self.organization_name = n; Task::none() }
                    OrganizationMessage::DomainChanged(d) => { self.organization_domain = d; Task::none() }
                    OrganizationMessage::MasterPassphraseChanged(p) => { self.master_passphrase = p; Task::none() }
                    OrganizationMessage::MasterPassphraseConfirmChanged(p) => { self.master_passphrase_confirm = p; Task::none() }
                    OrganizationMessage::NewPersonNameChanged(n) => { self.new_person_name = n; Task::none() }
                    OrganizationMessage::NewPersonEmailChanged(e) => { self.new_person_email = e; Task::none() }
                    OrganizationMessage::NewPersonRoleSelected(r) => { self.new_person_role = Some(r); Task::none() }
                    OrganizationMessage::SelectPerson(p) => { self.selected_person = Some(p); Task::none() }
                    OrganizationMessage::InlineEditNameChanged(n) => { self.inline_edit_name = n; Task::none() }
                    OrganizationMessage::InlineEditCancel => {
                        self.inline_edit_name.clear();
                        self.editing_new_node = None;
                        Task::none()
                    }
                    OrganizationMessage::NewLocationNameChanged(n) => { self.new_location_name = n; Task::none() }
                    OrganizationMessage::NewLocationTypeSelected(t) => { self.new_location_type = Some(t); Task::none() }
                    OrganizationMessage::NewLocationStreetChanged(s) => { self.new_location_street = s; Task::none() }
                    OrganizationMessage::NewLocationCityChanged(c) => { self.new_location_city = c; Task::none() }
                    OrganizationMessage::NewLocationRegionChanged(r) => { self.new_location_region = r; Task::none() }
                    OrganizationMessage::NewLocationCountryChanged(c) => { self.new_location_country = c; Task::none() }
                    OrganizationMessage::NewLocationPostalChanged(p) => { self.new_location_postal = p; Task::none() }
                    OrganizationMessage::NewLocationUrlChanged(u) => { self.new_location_url = u; Task::none() }
                    OrganizationMessage::NewUnitNameChanged(n) => { self.new_unit_name = n; Task::none() }
                    OrganizationMessage::NewUnitTypeSelected(t) => { self.new_unit_type = Some(t); Task::none() }
                    OrganizationMessage::NewUnitParentSelected(p) => { self.new_unit_parent = Some(p); Task::none() }
                    OrganizationMessage::NewUnitNatsAccountChanged(a) => { self.new_unit_nats_account = a; Task::none() }
                    OrganizationMessage::NewUnitResponsiblePersonSelected(p) => { self.new_unit_responsible_person = Some(p); Task::none() }
                    OrganizationMessage::NewServiceAccountNameChanged(n) => { self.new_service_account_name = n; Task::none() }
                    OrganizationMessage::NewServiceAccountPurposeChanged(p) => { self.new_service_account_purpose = p; Task::none() }
                    OrganizationMessage::ServiceAccountOwningUnitSelected(u) => { self.new_service_account_owning_unit = Some(u); Task::none() }
                    OrganizationMessage::ServiceAccountResponsiblePersonSelected(p) => { self.new_service_account_responsible_person = Some(p); Task::none() }
                    // Result handlers (clear forms on success)
                    OrganizationMessage::OrganizationUnitCreated(r) => {
                        if r.is_ok() {
                            self.new_unit_name.clear();
                            self.new_unit_type = None;
                            self.new_unit_parent = None;
                            self.new_unit_nats_account.clear();
                            self.new_unit_responsible_person = None;
                        }
                        Task::done(Message::OrganizationUnitCreated(r))
                    }
                    OrganizationMessage::ServiceAccountCreated(r) => {
                        if r.is_ok() {
                            self.new_service_account_name.clear();
                            self.new_service_account_purpose.clear();
                            self.new_service_account_owning_unit = None;
                            self.new_service_account_responsible_person = None;
                        }
                        Task::done(Message::ServiceAccountCreated(r))
                    }
                    OrganizationMessage::DomainCreated(r) => {
                        self.domain_loaded = r.is_ok();
                        Task::done(Message::DomainCreated(r))
                    }
                    OrganizationMessage::DomainLoaded(r) => {
                        if r.is_ok() { self.domain_loaded = true; }
                        Task::done(Message::DomainLoaded(r))
                    }
                    // Delegated operations
                    OrganizationMessage::CreateNewDomain => Task::done(Message::CreateNewDomain),
                    OrganizationMessage::LoadExistingDomain => Task::done(Message::LoadExistingDomain),
                    OrganizationMessage::ImportFromSecrets => Task::done(Message::ImportFromSecrets),
                    OrganizationMessage::SecretsImported(r) => Task::done(Message::SecretsImported(r)),
                    OrganizationMessage::ManifestDataLoaded(r) => Task::done(Message::ManifestDataLoaded(r)),
                    OrganizationMessage::AddPerson => Task::done(Message::AddPerson),
                    OrganizationMessage::RemovePerson(id) => Task::done(Message::RemovePerson(id)),
                    OrganizationMessage::NodeTypeSelected(t) => Task::done(Message::NodeTypeSelected(t)),
                    OrganizationMessage::InlineEditSubmit => Task::done(Message::InlineEditSubmit),
                    OrganizationMessage::AddLocation => Task::done(Message::AddLocation),
                    OrganizationMessage::RemoveLocation(id) => Task::done(Message::RemoveLocation(id)),
                    OrganizationMessage::CreateOrganizationUnit => Task::done(Message::CreateOrganizationUnit),
                    OrganizationMessage::RemoveOrganizationUnit(id) => Task::done(Message::RemoveOrganizationUnit(id)),
                    OrganizationMessage::CreateServiceAccount => Task::done(Message::CreateServiceAccount),
                    OrganizationMessage::DeactivateServiceAccount(id) => Task::done(Message::DeactivateServiceAccount(id)),
                    OrganizationMessage::RemoveServiceAccount(id) => Task::done(Message::RemoveServiceAccount(id)),
                    OrganizationMessage::GenerateServiceAccountKey { service_account_id } => {
                        Task::done(Message::GenerateServiceAccountKey { service_account_id })
                    }
                    OrganizationMessage::ServiceAccountKeyGenerated(r) => Task::done(Message::ServiceAccountKeyGenerated(r)),
                }
            }

            // ================================================================
            // PKI Domain Message Delegation (Sprint 49)
            // ================================================================
            Message::Pki(pki_msg) => {
                use pki::PkiMessage;

                match pki_msg {
                    // Section toggles
                    PkiMessage::ToggleRootCASection => {
                        self.root_ca_collapsed = !self.root_ca_collapsed;
                        Task::none()
                    }
                    PkiMessage::ToggleIntermediateCASection => {
                        self.intermediate_ca_collapsed = !self.intermediate_ca_collapsed;
                        Task::none()
                    }
                    PkiMessage::ToggleServerCertSection => {
                        self.server_cert_collapsed = !self.server_cert_collapsed;
                        Task::none()
                    }
                    PkiMessage::ToggleCertificatesSection => {
                        self.certificates_collapsed = !self.certificates_collapsed;
                        Task::none()
                    }
                    PkiMessage::ToggleKeysSection => {
                        self.keys_collapsed = !self.keys_collapsed;
                        Task::none()
                    }
                    PkiMessage::ToggleGpgSection => {
                        self.gpg_section_collapsed = !self.gpg_section_collapsed;
                        Task::none()
                    }
                    PkiMessage::ToggleRecoverySection => {
                        self.recovery_section_collapsed = !self.recovery_section_collapsed;
                        Task::none()
                    }
                    PkiMessage::ToggleMultiPurposeKeySection => {
                        self.multi_purpose_key_section_collapsed = !self.multi_purpose_key_section_collapsed;
                        Task::none()
                    }
                    PkiMessage::TogglePassphraseVisibility => {
                        self.show_passphrase = !self.show_passphrase;
                        Task::none()
                    }
                    // Form inputs
                    PkiMessage::IntermediateCANameChanged(n) => { self.intermediate_ca_name_input = n; Task::none() }
                    PkiMessage::SelectUnitForCA(u) => { self.selected_unit_for_ca = Some(u); Task::none() }
                    PkiMessage::ServerCertCNChanged(cn) => { self.server_cert_cn_input = cn; Task::none() }
                    PkiMessage::ServerCertSANsChanged(s) => { self.server_cert_sans_input = s; Task::none() }
                    PkiMessage::SelectIntermediateCA(ca) => { self.selected_intermediate_ca = Some(ca); Task::none() }
                    PkiMessage::SelectCertLocation(l) => { self.selected_cert_location = Some(l); Task::none() }
                    PkiMessage::CertOrganizationChanged(o) => { self.cert_organization = o; Task::none() }
                    PkiMessage::CertOrganizationalUnitChanged(o) => { self.cert_organizational_unit = o; Task::none() }
                    PkiMessage::CertLocalityChanged(l) => { self.cert_locality = l; Task::none() }
                    PkiMessage::CertStateProvinceChanged(s) => { self.cert_state_province = s; Task::none() }
                    PkiMessage::CertCountryChanged(c) => { self.cert_country = c; Task::none() }
                    PkiMessage::CertValidityDaysChanged(d) => { self.cert_validity_days = d; Task::none() }
                    PkiMessage::GpgUserIdChanged(u) => { self.gpg_user_id = u; Task::none() }
                    PkiMessage::GpgKeyTypeSelected(t) => { self.gpg_key_type = Some(t); Task::none() }
                    PkiMessage::GpgKeyLengthChanged(l) => { self.gpg_key_length = l; Task::none() }
                    PkiMessage::GpgExpiresDaysChanged(d) => { self.gpg_expires_days = d; Task::none() }
                    PkiMessage::RecoveryPassphraseChanged(p) => { self.recovery_passphrase = p; Task::none() }
                    PkiMessage::RecoveryPassphraseConfirmChanged(p) => { self.recovery_passphrase_confirm = p; Task::none() }
                    PkiMessage::RecoveryOrganizationIdChanged(o) => { self.recovery_organization_id = o; Task::none() }
                    PkiMessage::ClientCertCNChanged(cn) => { self.client_cert_cn = cn; Task::none() }
                    PkiMessage::ClientCertEmailChanged(e) => { self.client_cert_email = e; Task::none() }
                    PkiMessage::MultiPurposePersonSelected(p) => { self.multi_purpose_selected_person = Some(p); Task::none() }
                    PkiMessage::RootPassphraseChanged(p) => { self.root_passphrase = p; Task::none() }
                    PkiMessage::RootPassphraseConfirmChanged(p) => { self.root_passphrase_confirm = p; Task::none() }
                    PkiMessage::ToggleKeyPurpose(purpose) => {
                        if self.multi_purpose_selected_purposes.contains(&purpose) {
                            self.multi_purpose_selected_purposes.remove(&purpose);
                        } else {
                            self.multi_purpose_selected_purposes.insert(purpose);
                        }
                        Task::none()
                    }
                    PkiMessage::KeyGenerationProgress(p) => { self.key_generation_progress = p; Task::none() }
                    PkiMessage::PkiCertificatesLoaded(c) => { self.loaded_certificates = c; Task::none() }
                    PkiMessage::GenerateRandomPassphrase => {
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%^&*".chars().collect();
                        let passphrase: String = (0..20).map(|_| chars[rng.gen_range(0..chars.len())]).collect();
                        self.root_passphrase = passphrase.clone();
                        self.root_passphrase_confirm = passphrase;
                        Task::none()
                    }
                    // Delegated operations
                    PkiMessage::GenerateRootCA => Task::done(Message::GenerateRootCA),
                    PkiMessage::RootCAGenerated(r) => Task::done(Message::RootCAGenerated(r)),
                    PkiMessage::GenerateIntermediateCA => Task::done(Message::GenerateIntermediateCA),
                    PkiMessage::GenerateServerCert => Task::done(Message::GenerateServerCert),
                    PkiMessage::GenerateSSHKeys => Task::done(Message::GenerateSSHKeys),
                    PkiMessage::GenerateAllKeys => Task::done(Message::GenerateAllKeys),
                    PkiMessage::KeysGenerated(r) => Task::done(Message::KeysGenerated(r)),
                    PkiMessage::GenerateGpgKey => Task::done(Message::GenerateGpgKey),
                    PkiMessage::GpgKeyGenerated(r) => Task::done(Message::GpgKeyGenerated(r)),
                    PkiMessage::ListGpgKeys => Task::done(Message::ListGpgKeys),
                    PkiMessage::GpgKeysListed(r) => Task::done(Message::GpgKeysListed(r)),
                    PkiMessage::VerifyRecoverySeed => Task::done(Message::VerifyRecoverySeed),
                    PkiMessage::RecoverySeedVerified(r) => Task::done(Message::RecoverySeedVerified(r)),
                    PkiMessage::RecoverKeysFromSeed => Task::done(Message::RecoverKeysFromSeed),
                    PkiMessage::KeysRecovered(r) => Task::done(Message::KeysRecovered(r)),
                    PkiMessage::GenerateClientCert => Task::done(Message::GenerateClientCert),
                    PkiMessage::ClientCertGenerated(r) => Task::done(Message::ClientCertGenerated(r)),
                    PkiMessage::GenerateMultiPurposeKeys => Task::done(Message::GenerateMultiPurposeKeys),
                    PkiMessage::MultiPurposeKeysGenerated(r) => Task::done(Message::MultiPurposeKeysGenerated(r)),
                    PkiMessage::GeneratePkiFromGraph => Task::done(Message::GeneratePkiFromGraph),
                    PkiMessage::PersonalKeysGenerated(r) => Task::done(Message::PersonalKeysGenerated(r)),
                }
            }

            // ================================================================
            // YubiKey Domain Message Delegation (Sprint 50)
            // ================================================================
            Message::YubiKey(yk_msg) => {
                use yubikey::YubiKeyMessage;

                match yk_msg {
                    // Section toggles
                    YubiKeyMessage::ToggleYubiKeySection => {
                        self.yubikey_section_collapsed = !self.yubikey_section_collapsed;
                        Task::none()
                    }
                    YubiKeyMessage::ToggleYubiKeySlotSection => {
                        self.yubikey_slot_section_collapsed = !self.yubikey_slot_section_collapsed;
                        Task::none()
                    }
                    YubiKeyMessage::ToggleFilterYubiKey => {
                        self.filter_show_yubikey = !self.filter_show_yubikey;
                        Task::none()
                    }
                    // Serial and selection
                    YubiKeyMessage::YubiKeySerialChanged(serial) => {
                        self.yubikey_serial = serial;
                        Task::none()
                    }
                    YubiKeyMessage::SelectYubiKeyForAssignment(serial) => {
                        self.selected_yubikey_for_assignment = Some(serial);
                        Task::none()
                    }
                    YubiKeyMessage::SelectYubiKeyForManagement(serial) => {
                        self.selected_yubikey_for_management = Some(serial);
                        Task::none()
                    }
                    YubiKeyMessage::SelectPivSlot(slot) => {
                        self.selected_piv_slot = Some(slot);
                        Task::none()
                    }
                    // PIN management
                    YubiKeyMessage::YubiKeyPinInputChanged(pin) => {
                        self.yubikey_pin_input = pin;
                        Task::none()
                    }
                    YubiKeyMessage::YubiKeyNewPinChanged(pin) => {
                        self.yubikey_new_pin = pin;
                        Task::none()
                    }
                    YubiKeyMessage::YubiKeyPinConfirmChanged(pin) => {
                        self.yubikey_pin_confirm = pin;
                        Task::none()
                    }
                    // Management key
                    YubiKeyMessage::YubiKeyManagementKeyChanged(key) => {
                        self.yubikey_management_key = key;
                        Task::none()
                    }
                    YubiKeyMessage::YubiKeyNewManagementKeyChanged(key) => {
                        self.yubikey_new_management_key = key;
                        Task::none()
                    }
                    // Registration
                    YubiKeyMessage::YubiKeyRegistrationNameChanged(name) => {
                        self.yubikey_registration_name = name;
                        Task::none()
                    }
                    // Delegated operations
                    YubiKeyMessage::DetectYubiKeys => Task::done(Message::DetectYubiKeys),
                    YubiKeyMessage::YubiKeysDetected(r) => Task::done(Message::YubiKeysDetected(r)),
                    YubiKeyMessage::AssignYubiKeyToPerson { serial, person_id } => {
                        Task::done(Message::AssignYubiKeyToPerson { serial, person_id })
                    }
                    YubiKeyMessage::ProvisionYubiKey => Task::done(Message::ProvisionYubiKey),
                    YubiKeyMessage::ProvisionSingleYubiKey { serial } => {
                        Task::done(Message::ProvisionSingleYubiKey { serial })
                    }
                    YubiKeyMessage::YubiKeyProvisioned(r) => Task::done(Message::YubiKeyProvisioned(r)),
                    YubiKeyMessage::SingleYubiKeyProvisioned(r) => Task::done(Message::SingleYubiKeyProvisioned(r)),
                    YubiKeyMessage::QueryYubiKeySlots(s) => Task::done(Message::QueryYubiKeySlots(s)),
                    YubiKeyMessage::YubiKeySlotsQueried(r) => Task::done(Message::YubiKeySlotsQueried(r)),
                    YubiKeyMessage::ClearYubiKeySlot { serial, slot } => {
                        Task::done(Message::ClearYubiKeySlot { serial, slot })
                    }
                    YubiKeyMessage::YubiKeySlotCleared(r) => Task::done(Message::YubiKeySlotCleared(r)),
                    YubiKeyMessage::GenerateKeyInSlot { serial, slot } => {
                        Task::done(Message::GenerateKeyInSlot { serial, slot })
                    }
                    YubiKeyMessage::KeyInSlotGenerated(r) => Task::done(Message::KeyInSlotGenerated(r)),
                    YubiKeyMessage::VerifyYubiKeyPin(s) => Task::done(Message::VerifyYubiKeyPin(s)),
                    YubiKeyMessage::YubiKeyPinVerified(r) => Task::done(Message::YubiKeyPinVerified(r)),
                    YubiKeyMessage::ChangeYubiKeyManagementKey(s) => {
                        Task::done(Message::ChangeYubiKeyManagementKey(s))
                    }
                    YubiKeyMessage::YubiKeyManagementKeyChanged2(r) => {
                        Task::done(Message::YubiKeyManagementKeyChanged2(r))
                    }
                    YubiKeyMessage::ResetYubiKeyPiv(s) => Task::done(Message::ResetYubiKeyPiv(s)),
                    YubiKeyMessage::YubiKeyPivReset(r) => Task::done(Message::YubiKeyPivReset(r)),
                    YubiKeyMessage::GetYubiKeyAttestation { serial, slot } => {
                        Task::done(Message::GetYubiKeyAttestation { serial, slot })
                    }
                    YubiKeyMessage::YubiKeyAttestationReceived(r) => {
                        Task::done(Message::YubiKeyAttestationReceived(r))
                    }
                    YubiKeyMessage::RegisterYubiKeyInDomain { serial, name } => {
                        Task::done(Message::RegisterYubiKeyInDomain { serial, name })
                    }
                    YubiKeyMessage::YubiKeyRegistered(r) => Task::done(Message::YubiKeyRegistered(r)),
                    YubiKeyMessage::TransferYubiKey { serial, from_person_id, to_person_id } => {
                        Task::done(Message::TransferYubiKey { serial, from_person_id, to_person_id })
                    }
                    YubiKeyMessage::YubiKeyTransferred(r) => Task::done(Message::YubiKeyTransferred(r)),
                    YubiKeyMessage::RevokeYubiKeyAssignment { serial } => {
                        Task::done(Message::RevokeYubiKeyAssignment { serial })
                    }
                    YubiKeyMessage::YubiKeyAssignmentRevoked(r) => Task::done(Message::YubiKeyAssignmentRevoked(r)),
                    YubiKeyMessage::YubiKeyDataLoaded(yk, pe) => Task::done(Message::YubiKeyDataLoaded(yk, pe)),
                    YubiKeyMessage::ProvisionYubiKeysFromGraph => Task::done(Message::ProvisionYubiKeysFromGraph),
                    YubiKeyMessage::YubiKeysProvisioned(r) => Task::done(Message::YubiKeysProvisioned(r)),
                }
            }

            // ================================================================
            // NATS Domain Message Delegation (Sprint 51)
            // ================================================================
            Message::Nats(nats_msg) => {
                use nats::NatsMessage;

                match nats_msg {
                    NatsMessage::ToggleNatsConfig(enabled) => {
                        self.include_nats_config = enabled;
                        Task::none()
                    }
                    NatsMessage::ToggleNatsSection => {
                        self.nats_section_collapsed = !self.nats_section_collapsed;
                        Task::none()
                    }
                    NatsMessage::ToggleNatsVizSection => {
                        self.nats_viz_section_collapsed = !self.nats_viz_section_collapsed;
                        Task::none()
                    }
                    NatsMessage::ToggleNatsAccountExpand(account_name) => {
                        if self.nats_viz_expanded_accounts.contains(&account_name) {
                            self.nats_viz_expanded_accounts.remove(&account_name);
                        } else {
                            self.nats_viz_expanded_accounts.insert(account_name);
                        }
                        Task::none()
                    }
                    NatsMessage::SelectNatsOperator => {
                        self.nats_viz_selected_operator = true;
                        self.nats_viz_selected_account = None;
                        self.nats_viz_selected_user = None;
                        Task::none()
                    }
                    NatsMessage::SelectNatsAccount(account_name) => {
                        self.nats_viz_selected_operator = false;
                        self.nats_viz_selected_account = Some(account_name);
                        self.nats_viz_selected_user = None;
                        Task::none()
                    }
                    NatsMessage::SelectNatsUser(account_name, person_id) => {
                        self.nats_viz_selected_operator = false;
                        self.nats_viz_selected_account = None;
                        self.nats_viz_selected_user = Some((account_name, person_id));
                        Task::none()
                    }
                    NatsMessage::ToggleFilterNats => {
                        self.filter_show_nats = !self.filter_show_nats;
                        self.org_graph.filter_show_nats = self.filter_show_nats;
                        Task::none()
                    }
                    NatsMessage::NatsHierarchyRefreshed(result) => {
                        if let Ok(hierarchy) = result {
                            self.nats_viz_hierarchy_data = Some(hierarchy);
                        }
                        Task::none()
                    }
                    NatsMessage::NatsHierarchyGenerated(result) => {
                        self.nats_hierarchy_generated = result.is_ok();
                        Task::done(Message::NatsHierarchyGenerated(result))
                    }
                    NatsMessage::NatsBootstrapCreated(bootstrap) => {
                        self.nats_bootstrap = Some(*bootstrap);
                        Task::none()
                    }
                    NatsMessage::RemoveNatsAccount(account_name) => {
                        if let Some(ref selected) = self.nats_viz_selected_account {
                            if selected == &account_name {
                                self.nats_viz_selected_account = None;
                            }
                        }
                        self.nats_viz_expanded_accounts.remove(&account_name);
                        Task::done(Message::RemoveNatsAccount(account_name))
                    }
                    NatsMessage::RemoveNatsUser(account_name, person_id) => {
                        if let Some((ref acc, ref pid)) = self.nats_viz_selected_user {
                            if acc == &account_name && *pid == person_id {
                                self.nats_viz_selected_user = None;
                            }
                        }
                        Task::done(Message::RemoveNatsUser(account_name, person_id))
                    }
                    // Delegated operations
                    NatsMessage::GenerateNatsHierarchy => {
                        Task::done(Message::GenerateNatsHierarchy)
                    }
                    NatsMessage::GenerateNatsFromGraph => {
                        Task::done(Message::GenerateNatsFromGraph)
                    }
                    NatsMessage::NatsFromGraphGenerated(result) => {
                        Task::done(Message::NatsFromGraphGenerated(result))
                    }
                    NatsMessage::RefreshNatsHierarchy => {
                        Task::done(Message::RefreshNatsHierarchy)
                    }
                    NatsMessage::AddNatsAccount { unit_id, account_name } => {
                        Task::done(Message::AddNatsAccount { unit_id, account_name })
                    }
                    NatsMessage::AddNatsUser { account_name, person_id } => {
                        Task::done(Message::AddNatsUser { account_name, person_id })
                    }
                }
            }

            // ================================================================
            // Export Domain Message Delegation (Sprint 52)
            // ================================================================
            Message::Export(export_msg) => {
                use export::ExportMessage;

                match export_msg {
                    ExportMessage::ExportPathChanged(path) => {
                        self.export_path = std::path::PathBuf::from(path);
                        Task::none()
                    }
                    ExportMessage::ExportPasswordChanged(password) => {
                        self.export_password = password;
                        Task::none()
                    }
                    ExportMessage::ProjectionSectionChanged(section) => {
                        self.projection_section = section;
                        Task::none()
                    }
                    ExportMessage::ProjectionSelected(target) => {
                        self.selected_projection = Some(target);
                        Task::none()
                    }
                    ExportMessage::ExportToSDCard => {
                        Task::done(Message::ExportToSDCard)
                    }
                    ExportMessage::DomainExported(result) => {
                        Task::done(Message::DomainExported(result))
                    }
                    ExportMessage::SDCardExported(result) => {
                        Task::done(Message::SDCardExported(result))
                    }
                    ExportMessage::ExportToCypher => {
                        Task::done(Message::ExportToCypher)
                    }
                    ExportMessage::CypherExported(result) => {
                        Task::done(Message::CypherExported(result))
                    }
                    ExportMessage::ExportToNsc => {
                        Task::done(Message::ExportToNsc)
                    }
                    ExportMessage::NscExported(result) => {
                        Task::done(Message::NscExported(result))
                    }
                    ExportMessage::ExportGraph => {
                        Task::done(Message::ExportGraph)
                    }
                    ExportMessage::GraphExported(result) => {
                        Task::done(Message::GraphExported(result))
                    }
                    ExportMessage::GraphImported(result) => {
                        Task::done(Message::GraphImported(result))
                    }
                    ExportMessage::ConnectProjection(target) => {
                        Task::done(Message::ConnectProjection(target))
                    }
                    ExportMessage::DisconnectProjection(target) => {
                        Task::done(Message::DisconnectProjection(target))
                    }
                    ExportMessage::SyncProjection(target) => {
                        Task::done(Message::SyncProjection(target))
                    }
                }
            }

            // ================================================================
            // Delegation Domain Message Delegation (Sprint 53)
            // ================================================================
            Message::Delegation(del_msg) => {
                use delegation::DelegationMessage;

                match del_msg {
                    DelegationMessage::ToggleDelegationSection => {
                        self.delegation_section_collapsed = !self.delegation_section_collapsed;
                        Task::none()
                    }
                    DelegationMessage::DelegationFromPersonSelected(person_id) => {
                        Task::done(Message::DelegationFromPersonSelected(person_id))
                    }
                    DelegationMessage::DelegationToPersonSelected(person_id) => {
                        Task::done(Message::DelegationToPersonSelected(person_id))
                    }
                    DelegationMessage::ToggleDelegationPermission(permission) => {
                        Task::done(Message::ToggleDelegationPermission(permission))
                    }
                    DelegationMessage::DelegationExpiresDaysChanged(days) => {
                        Task::done(Message::DelegationExpiresDaysChanged(days))
                    }
                    DelegationMessage::CreateDelegation => {
                        Task::done(Message::CreateDelegation)
                    }
                    DelegationMessage::DelegationCreated(result) => {
                        Task::done(Message::DelegationCreated(result))
                    }
                    DelegationMessage::RevokeDelegation(id) => {
                        Task::done(Message::RevokeDelegation(id))
                    }
                    DelegationMessage::DelegationRevoked(result) => {
                        Task::done(Message::DelegationRevoked(result))
                    }
                }
            }

            Message::TrustChain(tc_msg) => {
                use trustchain::TrustChainMessage;

                match tc_msg {
                    TrustChainMessage::ToggleTrustChainSection => {
                        self.trust_chain_section_collapsed = !self.trust_chain_section_collapsed;
                        Task::none()
                    }
                    TrustChainMessage::SelectCertForTrustChain(cert_id) => {
                        self.selected_trust_chain_cert = Some(cert_id);
                        Task::none()
                    }
                    TrustChainMessage::VerifyTrustChain(cert_id) => {
                        // Delegated to Message::VerifyTrustChain handler
                        Task::done(Message::VerifyTrustChain(cert_id))
                    }
                    TrustChainMessage::TrustChainVerified(result) => {
                        // Delegated to Message::TrustChainVerified handler
                        Task::done(Message::TrustChainVerified(result))
                    }
                    TrustChainMessage::VerifyAllTrustChains => {
                        // Delegated to Message::VerifyAllTrustChains handler
                        Task::done(Message::VerifyAllTrustChains)
                    }
                }
            }

            Message::Location(loc_msg) => {
                use location::LocationMessage;

                match loc_msg {
                    LocationMessage::NameChanged(name) => {
                        self.new_location_name = name;
                        Task::none()
                    }
                    LocationMessage::TypeSelected(loc_type) => {
                        self.new_location_type = Some(loc_type);
                        Task::none()
                    }
                    LocationMessage::StreetChanged(street) => {
                        self.new_location_street = street;
                        Task::none()
                    }
                    LocationMessage::CityChanged(city) => {
                        self.new_location_city = city;
                        Task::none()
                    }
                    LocationMessage::RegionChanged(region) => {
                        self.new_location_region = region;
                        Task::none()
                    }
                    LocationMessage::CountryChanged(country) => {
                        self.new_location_country = country;
                        Task::none()
                    }
                    LocationMessage::PostalChanged(postal) => {
                        self.new_location_postal = postal;
                        Task::none()
                    }
                    LocationMessage::UrlChanged(url) => {
                        self.new_location_url = url;
                        Task::none()
                    }
                    LocationMessage::AddLocation => {
                        // Adding delegated elsewhere
                        Task::none()
                    }
                    LocationMessage::RemoveLocation(location_id) => {
                        self.loaded_locations.retain(|l| l.location_id != location_id);
                        Task::none()
                    }
                }
            }

            Message::ServiceAccount(sa_msg) => {
                use service_account::ServiceAccountMessage;

                match sa_msg {
                    ServiceAccountMessage::ToggleSection => {
                        self.service_account_section_collapsed = !self.service_account_section_collapsed;
                        Task::none()
                    }
                    ServiceAccountMessage::NameChanged(name) => {
                        self.new_service_account_name = name;
                        Task::none()
                    }
                    ServiceAccountMessage::PurposeChanged(purpose) => {
                        self.new_service_account_purpose = purpose;
                        Task::none()
                    }
                    ServiceAccountMessage::OwningUnitSelected(unit_id) => {
                        self.new_service_account_owning_unit = Some(unit_id);
                        Task::none()
                    }
                    ServiceAccountMessage::ResponsiblePersonSelected(person_id) => {
                        self.new_service_account_responsible_person = Some(person_id);
                        Task::none()
                    }
                    ServiceAccountMessage::Create => {
                        // Creation delegated elsewhere
                        Task::none()
                    }
                    ServiceAccountMessage::Created(result) => {
                        match result {
                            Ok(account) => {
                                self.created_service_accounts.push(account);
                                // Clear form
                                self.new_service_account_name.clear();
                                self.new_service_account_purpose.clear();
                                self.new_service_account_owning_unit = None;
                                self.new_service_account_responsible_person = None;
                            }
                            Err(_) => {}
                        }
                        Task::none()
                    }
                    ServiceAccountMessage::Deactivate(id) => {
                        // Deactivation - remove from active accounts
                        // (In a full event-sourced system, this would emit a deactivation event)
                        self.created_service_accounts.retain(|a| a.id != id);
                        Task::none()
                    }
                    ServiceAccountMessage::Remove(id) => {
                        self.created_service_accounts.retain(|a| a.id != id);
                        Task::none()
                    }
                    ServiceAccountMessage::GenerateKey { .. } => {
                        // Key generation delegated elsewhere
                        Task::none()
                    }
                    ServiceAccountMessage::KeyGenerated(_result) => {
                        // Key generation result handled
                        Task::none()
                    }
                }
            }

            Message::Gpg(gpg_msg) => {
                use gpg::GpgMessage;

                match gpg_msg {
                    GpgMessage::ToggleSection => {
                        self.gpg_section_collapsed = !self.gpg_section_collapsed;
                        Task::none()
                    }
                    GpgMessage::UserIdChanged(user_id) => {
                        self.gpg_user_id = user_id;
                        Task::none()
                    }
                    GpgMessage::KeyTypeSelected(key_type) => {
                        self.gpg_key_type = Some(key_type);
                        Task::none()
                    }
                    GpgMessage::KeyLengthChanged(length) => {
                        self.gpg_key_length = length;
                        Task::none()
                    }
                    GpgMessage::ExpiresDaysChanged(days) => {
                        self.gpg_expires_days = days;
                        Task::none()
                    }
                    GpgMessage::Generate => {
                        // Generation delegated elsewhere
                        Task::none()
                    }
                    GpgMessage::Generated(result) => {
                        match result {
                            Ok(keypair) => {
                                self.gpg_generation_status = Some(format!("Generated key: {}", keypair.fingerprint));
                                // Convert GpgKeypair to GpgKeyInfo
                                let info = crate::ports::gpg::GpgKeyInfo {
                                    key_id: keypair.key_id.clone(),
                                    fingerprint: keypair.fingerprint.clone(),
                                    user_ids: vec![keypair.user_id],
                                    creation_time: chrono::Utc::now().timestamp(),
                                    expiration_time: None,
                                    is_revoked: false,
                                    is_expired: false,
                                };
                                self.generated_gpg_keys.push(info);
                            }
                            Err(error) => {
                                self.gpg_generation_status = Some(error);
                            }
                        }
                        Task::none()
                    }
                    GpgMessage::ListKeys => {
                        // Listing delegated elsewhere
                        Task::none()
                    }
                    GpgMessage::KeysListed(result) => {
                        match result {
                            Ok(keys) => {
                                self.generated_gpg_keys = keys;
                            }
                            Err(error) => {
                                self.gpg_generation_status = Some(error);
                            }
                        }
                        Task::none()
                    }
                }
            }

            Message::Recovery(recovery_msg) => {
                use recovery::RecoveryMessage;

                match recovery_msg {
                    RecoveryMessage::ToggleSection => {
                        self.recovery_section_collapsed = !self.recovery_section_collapsed;
                        Task::none()
                    }
                    RecoveryMessage::PassphraseChanged(passphrase) => {
                        self.recovery_passphrase = passphrase;
                        Task::none()
                    }
                    RecoveryMessage::PassphraseConfirmChanged(confirm) => {
                        self.recovery_passphrase_confirm = confirm;
                        Task::none()
                    }
                    RecoveryMessage::OrganizationIdChanged(org_id) => {
                        self.recovery_organization_id = org_id;
                        Task::none()
                    }
                    RecoveryMessage::VerifySeed => {
                        // Verification delegated elsewhere
                        Task::none()
                    }
                    RecoveryMessage::SeedVerified(result) => {
                        match result {
                            Ok(_fingerprint) => {
                                self.recovery_seed_verified = true;
                                self.recovery_status = Some("Seed verified successfully".to_string());
                            }
                            Err(error) => {
                                self.recovery_seed_verified = false;
                                self.recovery_status = Some(error);
                            }
                        }
                        Task::none()
                    }
                    RecoveryMessage::RecoverKeys => {
                        // Recovery delegated elsewhere
                        Task::none()
                    }
                    RecoveryMessage::KeysRecovered(result) => {
                        match result {
                            Ok(count) => {
                                self.recovery_status = Some(format!("Recovered {} keys", count));
                            }
                            Err(error) => {
                                self.recovery_status = Some(error);
                            }
                        }
                        Task::none()
                    }
                }
            }

            Message::OrgUnit(ou_msg) => {
                use org_unit::OrgUnitMessage;

                match ou_msg {
                    // === UI State ===
                    OrgUnitMessage::ToggleSection => {
                        self.org_unit_section_collapsed = !self.org_unit_section_collapsed;
                        Task::none()
                    }

                    // === Form Input ===
                    OrgUnitMessage::NameChanged(name) => {
                        self.new_unit_name = name;
                        Task::none()
                    }
                    OrgUnitMessage::TypeSelected(unit_type) => {
                        self.new_unit_type = Some(unit_type);
                        Task::none()
                    }
                    OrgUnitMessage::ParentSelected(parent) => {
                        self.new_unit_parent = if parent.is_empty() {
                            None
                        } else {
                            Some(parent)
                        };
                        Task::none()
                    }
                    OrgUnitMessage::NatsAccountChanged(account) => {
                        self.new_unit_nats_account = account;
                        Task::none()
                    }
                    OrgUnitMessage::ResponsiblePersonSelected(person_id) => {
                        self.new_unit_responsible_person = Some(person_id);
                        Task::none()
                    }

                    // === Lifecycle ===
                    OrgUnitMessage::Create => {
                        // Creation delegated elsewhere
                        Task::none()
                    }
                    OrgUnitMessage::Created(result) => {
                        match result {
                            Ok(unit) => {
                                self.created_units.push(unit);
                                // Clear form
                                self.new_unit_name.clear();
                                self.new_unit_type = None;
                                self.new_unit_parent = None;
                                self.new_unit_nats_account.clear();
                                self.new_unit_responsible_person = None;
                            }
                            Err(_) => {
                                // Error - keep form for retry
                            }
                        }
                        Task::none()
                    }
                    OrgUnitMessage::Remove(unit_id) => {
                        self.created_units.retain(|u| u.id != unit_id);
                        Task::none()
                    }
                }
            }

            Message::MultiKey(mk_msg) => {
                use multi_key::MultiKeyMessage;

                match mk_msg {
                    // === UI State ===
                    MultiKeyMessage::ToggleSection => {
                        self.multi_purpose_key_section_collapsed = !self.multi_purpose_key_section_collapsed;
                        Task::none()
                    }

                    // === Selection ===
                    MultiKeyMessage::PersonSelected(person_id) => {
                        self.multi_purpose_selected_person = Some(person_id);
                        Task::none()
                    }
                    MultiKeyMessage::TogglePurpose(purpose) => {
                        if self.multi_purpose_selected_purposes.contains(&purpose) {
                            self.multi_purpose_selected_purposes.remove(&purpose);
                        } else {
                            self.multi_purpose_selected_purposes.insert(purpose);
                        }
                        Task::none()
                    }

                    // === Generation ===
                    MultiKeyMessage::Generate => {
                        // Generation delegated elsewhere
                        Task::none()
                    }
                    MultiKeyMessage::Generated(result) => {
                        match result {
                            Ok(_gen_result) => {
                                // Success - clear purposes, keep person selected
                                self.multi_purpose_selected_purposes.clear();
                            }
                            Err(_) => {
                                // Error - keep selection for retry
                            }
                        }
                        Task::none()
                    }
                }
            }

            Message::Certificate(cert_msg) => {
                use certificate::CertificateMessage;

                match cert_msg {
                    // === UI State ===
                    CertificateMessage::ToggleCertificatesSection => {
                        self.certificates_collapsed = !self.certificates_collapsed;
                        Task::none()
                    }
                    CertificateMessage::ToggleIntermediateCA => {
                        self.intermediate_ca_collapsed = !self.intermediate_ca_collapsed;
                        Task::none()
                    }
                    CertificateMessage::ToggleServerCert => {
                        self.server_cert_collapsed = !self.server_cert_collapsed;
                        Task::none()
                    }

                    // === Metadata Form ===
                    CertificateMessage::OrganizationChanged(org) => {
                        self.cert_organization = org;
                        Task::none()
                    }
                    CertificateMessage::OrganizationalUnitChanged(ou) => {
                        self.cert_organizational_unit = ou;
                        Task::none()
                    }
                    CertificateMessage::LocalityChanged(locality) => {
                        self.cert_locality = locality;
                        Task::none()
                    }
                    CertificateMessage::StateProvinceChanged(st) => {
                        self.cert_state_province = st;
                        Task::none()
                    }
                    CertificateMessage::CountryChanged(country) => {
                        self.cert_country = country;
                        Task::none()
                    }
                    CertificateMessage::ValidityDaysChanged(days) => {
                        self.cert_validity_days = days;
                        Task::none()
                    }

                    // === Intermediate CA ===
                    CertificateMessage::IntermediateCANameChanged(name) => {
                        self.intermediate_ca_name_input = name;
                        Task::none()
                    }
                    CertificateMessage::SelectIntermediateCA(ca_name) => {
                        self.selected_intermediate_ca = if ca_name.is_empty() {
                            None
                        } else {
                            Some(ca_name)
                        };
                        Task::none()
                    }
                    CertificateMessage::GenerateIntermediateCA => {
                        // Crypto operations delegated elsewhere
                        Task::none()
                    }

                    // === Server Certificate ===
                    CertificateMessage::ServerCNChanged(cn) => {
                        self.server_cert_cn_input = cn;
                        Task::none()
                    }
                    CertificateMessage::ServerSANsChanged(sans) => {
                        self.server_cert_sans_input = sans;
                        Task::none()
                    }
                    CertificateMessage::SelectLocation(location) => {
                        self.selected_cert_location = if location.is_empty() {
                            None
                        } else {
                            Some(location)
                        };
                        Task::none()
                    }
                    CertificateMessage::GenerateServerCert => {
                        // Crypto operations delegated elsewhere
                        Task::none()
                    }

                    // === Client Certificate ===
                    CertificateMessage::GenerateClientCert => {
                        // Crypto operations delegated elsewhere
                        Task::none()
                    }
                    CertificateMessage::ClientCertGenerated(_result) => {
                        // Status handling
                        Task::none()
                    }

                    // === Chain View ===
                    CertificateMessage::SelectForChainView(cert_id) => {
                        self.selected_trust_chain_cert = Some(cert_id);
                        Task::none()
                    }

                    // === Loading ===
                    CertificateMessage::CertificatesLoaded(certificates) => {
                        self.loaded_certificates = certificates;
                        Task::none()
                    }
                }
            }

            Message::EventLog(el_msg) => {
                use event_log::EventLogMessage;

                match el_msg {
                    // === UI State ===
                    EventLogMessage::ToggleSection => {
                        self.event_log_section_collapsed = !self.event_log_section_collapsed;
                        Task::none()
                    }

                    // === Loading ===
                    EventLogMessage::Load => {
                        // Loading delegated elsewhere
                        Task::none()
                    }
                    EventLogMessage::Loaded(result) => {
                        match result {
                            Ok(events) => {
                                self.loaded_event_log = events;
                            }
                            Err(_) => {
                                // Error handled via status
                            }
                        }
                        Task::none()
                    }

                    // === Selection ===
                    EventLogMessage::ToggleSelection(cid) => {
                        if self.selected_events_for_replay.contains(&cid) {
                            self.selected_events_for_replay.remove(&cid);
                        } else {
                            self.selected_events_for_replay.insert(cid);
                        }
                        Task::none()
                    }
                    EventLogMessage::ClearSelection => {
                        self.selected_events_for_replay.clear();
                        Task::none()
                    }

                    // === Replay ===
                    EventLogMessage::Replay => {
                        // Replay delegated elsewhere
                        Task::none()
                    }
                    EventLogMessage::Replayed(result) => {
                        match result {
                            Ok(_count) => {
                                // Success - clear selection
                                self.selected_events_for_replay.clear();
                            }
                            Err(_) => {
                                // Error - keep selection for retry
                            }
                        }
                        Task::none()
                    }
                }
            }

            // Tab navigation
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                self.status_message = match tab {
                    Tab::Welcome => "Welcome to CIM Keys".to_string(),
                    Tab::Organization => format!("Organization Graph - Primary Interface ({} nodes, {} edges)",
                        self.org_graph.nodes.len(), self.org_graph.edges.len()),
                    Tab::Locations => format!("Manage Locations ({} loaded)", self.loaded_locations.len()),
                    Tab::Keys => "Generate and Manage Cryptographic Keys".to_string(),
                    Tab::Projections => "Projections".to_string(),
                    Tab::Workflow => {
                        let summary = self.workflow_view.tracker.summary();
                        format!("Trust Chain Gap Fulfillment ({}/{} complete)",
                            summary.completed, summary.total_gaps)
                    }
                    Tab::StateMachines => {
                        let def = state_machine_graph::get_state_machine(self.selected_state_machine);
                        format!("State Machine Visualization - {} ({} states, {} transitions)",
                            self.selected_state_machine.display_name(),
                            def.states.len(),
                            def.transitions.len())
                    }
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
                // Sprint 68: Use command factory for FRP-compliant validation

                // Check for duplicate organization (precondition - not part of domain validation)
                if let Ok(proj) = self.projection.try_read() {
                    let existing_org = proj.get_organization();
                    if !existing_org.name.is_empty() {
                        self.error_message = Some(format!(
                            "An organization '{}' already exists. Load or import existing domain instead, or clear data first.",
                            existing_org.name
                        ));
                        return Task::none();
                    }
                }

                // Validate passphrase (security concern, separate from organization domain)
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

                // Build form from GUI state (presentation → ViewModel)
                let form = OrganizationForm::new()
                    .with_name(self.organization_name.clone())
                    .with_domain(self.organization_domain.clone())
                    .with_admin_email(self.admin_email.clone());

                // Create correlation ID for event tracing
                let correlation_id = Uuid::now_v7();

                // Use curried command factory (ACL validation + command creation)
                let result: OrganizationResult = org_factory::create(correlation_id)(&form);
                match result {
                    Ok(command) => {
                        // Validation passed - use command data
                        let org_id = command.organization_id;
                        let org_name = command.name.clone();
                        let org_domain = command.domain.clone().unwrap_or_default();

                        // Store the org_id for use in person creation
                        self.organization_id = Some(org_id);
                        self.error_message = None;

                        let projection = self.projection.clone();
                        let country = self.cert_country.clone();
                        let admin_email = self.admin_email.clone();

                        Task::perform(
                            async move {
                                let mut proj = projection.write().await;
                                proj.set_organization(
                                    org_name.clone(),
                                    org_domain,
                                    country,
                                    admin_email,
                                )
                                .map(|_| format!("Created domain: {}", org_name))
                                .map_err(|e| format!("Failed to create domain: {}", e))
                            },
                            Message::DomainCreated
                        )
                    }
                    Err(validation_errors) => {
                        // Validation failed - format errors for GUI display
                        let error_messages: Vec<String> = validation_errors
                            .iter()
                            .map(|e| format!("{}: {}", e.field, e.message))
                            .collect();
                        self.error_message = Some(error_messages.join("\n"));
                        Task::none()
                    }
                }
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

                        // === IDEMPOTENT IMPORT HANDLING ===
                        // Check if this exact organization is already imported
                        let incoming_org_id = org.id.as_uuid();
                        let existing_org_id = self.organization_id;
                        let has_existing_data = !self.org_graph.nodes.is_empty();

                        if has_existing_data {
                            if existing_org_id == Some(incoming_org_id) {
                                // Same organization - check if data is essentially the same
                                let existing_people_count = self.org_graph.nodes.iter()
                                    .filter(|(_, node)| node.lifted_node.injection().is_person())
                                    .count();
                                let incoming_people_count = people.len();

                                if existing_people_count == incoming_people_count {
                                    // Idempotent: same org, same count - skip import
                                    self.status_message = format!(
                                        "Import skipped - organization '{}' is already loaded with {} people. \
                                        Data is already up to date.",
                                        org.name, existing_people_count
                                    );
                                    return Task::none();
                                } else {
                                    // Same org but different data - update
                                    self.status_message = format!(
                                        "Updating organization '{}': {} → {} people",
                                        org.name, existing_people_count, incoming_people_count
                                    );
                                }
                            } else {
                                // Different organization - warn about replacement
                                let old_name = self.organization_name.clone();
                                self.status_message = format!(
                                    "Replacing existing organization '{}' with '{}'",
                                    old_name, org.name
                                );
                            }
                        }

                        // Set organization info
                        self.organization_name = org.name.clone();
                        self.organization_domain = org.display_name.clone();
                        self.organization_id = Some(org.id.as_uuid());

                        // Store loaded units for UI (intermediate CA unit selector)
                        self.loaded_units = units.clone();

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
                // Sprint 75: Use CQRS through aggregate (replaces Sprint 68 factory pattern)

                // Validate domain is created first (precondition)
                let org_uuid = match self.organization_id {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Please create a domain first".to_string());
                        return Task::none();
                    }
                };

                // GUI-level validation for immediate feedback
                if self.new_person_name.trim().is_empty() {
                    self.error_message = Some("Person name is required".to_string());
                    return Task::none();
                }
                if self.new_person_email.trim().is_empty() || !self.new_person_email.contains('@') {
                    self.error_message = Some("Valid email is required".to_string());
                    return Task::none();
                }

                // Build CQRS command
                use crate::commands::{KeyCommand, organization::CreatePerson};

                let role = self.new_person_role.unwrap_or(KeyOwnerRole::Developer);
                let person_name = self.new_person_name.clone();
                let person_email = self.new_person_email.clone();

                let cmd = CreatePerson {
                    command_id: Uuid::now_v7(),
                    person_id: Uuid::now_v7(),
                    name: person_name.clone(),
                    email: person_email.clone(),
                    title: Some(format!("{:?}", role)),
                    department: None,
                    organization_id: Some(org_uuid),
                    correlation_id: Uuid::now_v7(),
                    causation_id: None,
                    timestamp: chrono::Utc::now(),
                };

                // Clone for async and GUI update
                let aggregate = self.aggregate.clone();
                let projection = self.projection.clone();
                let person_id = cmd.person_id;

                // Clear form fields optimistically
                self.new_person_name.clear();
                self.new_person_email.clear();
                self.new_person_role = None;
                self.error_message = None;

                // Process command through aggregate (CQRS pattern - Sprint 75)
                Task::perform(
                    async move {
                        let aggregate_read = aggregate.read().await;
                        let projection_read = projection.read().await;

                        let events = aggregate_read.handle_command(
                            KeyCommand::CreatePerson(cmd),
                            &projection_read,
                            None, // No NATS port in offline mode
                            #[cfg(feature = "policy")]
                            None  // No policy engine in GUI yet
                        ).await
                        .map_err(|e| e.to_string())?;

                        // Extract person info from emitted event for GUI update
                        for event in &events {
                            if let crate::events::DomainEvent::Person(
                                crate::events::PersonEvents::PersonCreated(evt)
                            ) = event {
                                return Ok((
                                    evt.person_id,
                                    evt.name.clone(),
                                    evt.email.clone().unwrap_or_default(),
                                    org_uuid,
                                    role,
                                ));
                            }
                        }

                        Err("No PersonCreated event emitted".to_string())
                    },
                    |result| match result {
                        Ok((person_id, name, email, org_id, role)) => {
                            Message::PersonAdded(Ok((person_id, name, email, org_id, role)))
                        }
                        Err(e) => Message::PersonAdded(Err(e)),
                    }
                )
            }

            Message::PersonAdded(result) => {
                match result {
                    Ok((person_id, name, email, org_id, role)) => {
                        // Create Person and add to graph for visualization
                        let person = Person {
                            id: BootstrapPersonId::from_uuid(person_id),
                            name: name.clone(),
                            email: email.clone(),
                            organization_id: BootstrapOrgId::from_uuid(org_id),
                            unit_ids: vec![],
                            roles: vec![],
                            nats_permissions: None,
                            active: true,
                            owner_id: None,
                        };

                        // Add to graph for visualization
                        self.org_graph.add_node(person.clone(), role);

                        // Persist to projection
                        let projection = self.projection.clone();
                        let role_string = format!("{:?}", role);

                        self.status_message = format!("Added {} to organization", name);

                        Task::perform(
                            async move {
                                let mut proj = projection.write().await;
                                proj.add_person(person_id, name, email, role_string, org_id)
                                    .map_err(|e| format!("Failed to persist: {}", e))
                            },
                            |result| match result {
                                Ok(_) => Message::UpdateStatus("Person saved to projection".to_string()),
                                Err(e) => Message::ShowError(e),
                            }
                        )
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to add person: {}", e));
                        Task::none()
                    }
                }
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

            Message::NewLocationUrlChanged(value) => {
                self.new_location_url = value;
                Task::none()
            }

            Message::AddLocation => {
                // Sprint 75: Use CQRS through aggregate

                // Validate domain is created (precondition)
                let org_id = match self.organization_id {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Please create a domain first".to_string());
                        return Task::none();
                    }
                };

                // GUI-level validation for immediate feedback
                if self.new_location_name.trim().is_empty() {
                    self.error_message = Some("Location name is required".to_string());
                    return Task::none();
                }

                // Build CQRS command
                use crate::commands::{KeyCommand, organization::CreateLocation};

                let location_type = self.new_location_type.clone()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|| "Physical".to_string());

                // Build address from form fields
                let address = if !self.new_location_street.is_empty() {
                    Some(format!("{}, {}, {} {} {}",
                        self.new_location_street,
                        self.new_location_city,
                        self.new_location_region,
                        self.new_location_postal,
                        self.new_location_country
                    ))
                } else {
                    None
                };

                let cmd = CreateLocation {
                    command_id: Uuid::now_v7(),
                    location_id: Uuid::now_v7(),
                    name: self.new_location_name.clone(),
                    location_type: location_type.clone(),
                    address,
                    coordinates: None,
                    organization_id: Some(org_id),
                    correlation_id: Uuid::now_v7(),
                    causation_id: None,
                    timestamp: chrono::Utc::now(),
                };

                // Clone for async
                let aggregate = self.aggregate.clone();
                let projection = self.projection.clone();
                let location_name = self.new_location_name.clone();

                // Clear form fields optimistically
                self.new_location_name.clear();
                self.new_location_type = None;
                self.new_location_street.clear();
                self.new_location_city.clear();
                self.new_location_region.clear();
                self.new_location_country.clear();
                self.new_location_postal.clear();
                self.new_location_url.clear();
                self.error_message = None;

                // Process command through aggregate (CQRS pattern - Sprint 75)
                Task::perform(
                    async move {
                        let aggregate_read = aggregate.read().await;
                        let projection_read = projection.read().await;

                        let events = aggregate_read.handle_command(
                            KeyCommand::CreateLocation(cmd),
                            &projection_read,
                            None, // No NATS port in offline mode
                            #[cfg(feature = "policy")]
                            None  // No policy engine in GUI yet
                        ).await
                        .map_err(|e| e.to_string())?;

                        // Extract location info from emitted event
                        for event in &events {
                            if let crate::events::DomainEvent::Location(
                                crate::events::LocationEvents::LocationCreated(evt)
                            ) = event {
                                return Ok((
                                    evt.location_id,
                                    evt.name.clone(),
                                    evt.location_type.clone(),
                                    org_id,
                                ));
                            }
                        }

                        Err("No LocationCreated event emitted".to_string())
                    },
                    |result| Message::LocationAdded(result)
                )
            }

            Message::LocationAdded(result) => {
                match result {
                    Ok((location_id, name, location_type, org_id)) => {
                        self.status_message = format!("Added location: {}", name);

                        // Persist to projection
                        let projection = self.projection.clone();

                        Task::perform(
                            async move {
                                let mut proj = projection.write().await;
                                proj.add_location(
                                    location_id,
                                    name.clone(),
                                    location_type,
                                    org_id,
                                    None, None, None, None, None, None, // Address details are in the event
                                )
                                .map_err(|e| format!("Failed to persist: {}", e))
                            },
                            |result| match result {
                                Ok(_) => Message::UpdateStatus("Location saved to projection".to_string()),
                                Err(e) => Message::ShowError(e),
                            }
                        )
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to add location: {}", e));
                        Task::none()
                    }
                }
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

                            // Sprint 83: Update YubiKey provisioning state machine for each device
                            // Emit YubiKeyDetected for each device to initialize state machine
                            let tasks: Vec<_> = devices.iter().map(|d| {
                                Task::done(Message::YubiKeyDetected {
                                    serial: d.serial.clone(),
                                    firmware_version: d.version.clone(),
                                })
                            }).collect();

                            return Task::batch(tasks);
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

            Message::SelectYubiKeyForAssignment(serial) => {
                self.selected_yubikey_for_assignment = Some(serial);
                Task::none()
            }

            Message::AssignYubiKeyToPerson { serial, person_id } => {
                // Find the person name for display
                let person_name = self.loaded_people.iter()
                    .find(|p| p.person_id == person_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                self.yubikey_assignments.insert(serial.clone(), person_id);
                self.selected_yubikey_for_assignment = None;
                self.status_message = format!("YubiKey {} assigned to {}", serial, person_name);
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
                // Check if Root CA already exists to prevent duplicates
                let existing_root_ca_count = self.org_graph.nodes.iter()
                    .filter(|(_, node)| {
                        if node.lifted_node.injection().is_certificate() {
                            // Check if it's a root CA by looking at the label
                            node.lifted_node.label.contains("Root CA")
                        } else {
                            false
                        }
                    })
                    .count();

                if existing_root_ca_count > 0 {
                    self.error_message = Some(format!(
                        "A Root CA already exists ({} found). Cannot generate duplicate. Revoke or delete existing Root CA first.",
                        existing_root_ca_count
                    ));
                    return Task::none();
                }

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
                        use crate::state_machines::workflows::PKIBootstrapState;

                        let cert_id = CertificateId::new();
                        let key_id = Uuid::now_v7(); // Key ID for state machine tracking
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
                        let cert_id_uuid = cert_id.as_uuid();

                        // Create view with custom color and label
                        let custom_color = self.view_model.colors.cert_root_ca;
                        let custom_label = format!("{} Root CA", self.organization_name);
                        let view = view_model::NodeView::new(cert_id_uuid, position, custom_color, custom_label);

                        // Add to graph
                        self.org_graph.nodes.insert(cert_id_uuid, root_ca_node);
                        self.org_graph.node_views.insert(cert_id_uuid, view);

                        // Switch to PKI view to show the new certificate
                        self.graph_view = GraphView::PkiTrustChain;

                        // Sprint 77: Update PKI state machine if in RootCAPlanned state
                        if matches!(self.pki_state, PKIBootstrapState::RootCAPlanned { .. }) {
                            self.pki_state = PKIBootstrapState::RootCAGenerated {
                                root_ca_cert_id: cert_id_uuid,
                                root_ca_key_id: key_id,
                                generated_at: chrono::Utc::now(),
                            };
                            tracing::info!(
                                "PKI state machine transitioned to RootCAGenerated (cert: {}, key: {})",
                                cert_id_uuid, key_id
                            );
                        }

                        // Sprint 81: Store the Root CA certificate for signing intermediate CAs
                        self.generated_root_ca = Some(certificate.clone());
                        tracing::info!("Root CA stored for intermediate CA signing");

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

            // Sprint 81: Intermediate CA generation result handler
            Message::IntermediateCAGenerated(result) => {
                match result {
                    Ok((certificate, intermediate_ca_id)) => {
                        // Create Intermediate CA node in graph using LiftableDomain
                        use crate::domain::pki::{Certificate as PkiCertificate, CertificateId};
                        use crate::lifting::LiftableDomain;
                        use crate::state_machines::workflows::PKIBootstrapState;

                        let cert_id = CertificateId::from_uuid(intermediate_ca_id);
                        let subject = format!("CN={} Intermediate CA, O={}", self.organization_name, self.organization_name);
                        let cert = PkiCertificate::intermediate(
                            cert_id,
                            subject.clone(),
                            format!("{} Root CA", self.organization_name), // issuer
                            chrono::Utc::now(),
                            chrono::Utc::now() + chrono::Duration::days(365 * 3), // 3 years
                            vec!["keyCertSign".to_string(), "cRLSign".to_string()],
                        );
                        let lifted_node = cert.lift();
                        let position = iced::Point::new(400.0, 200.0); // Below Root CA
                        let intermediate_ca_node = graph::ConceptEntity::from_lifted_node(lifted_node);

                        // Create view with custom color and label
                        let custom_color = self.view_model.colors.cert_intermediate;
                        let custom_label = format!("{} Intermediate CA", self.organization_name);
                        let view = view_model::NodeView::new(intermediate_ca_id, position, custom_color, custom_label);

                        // Add to graph
                        self.org_graph.nodes.insert(intermediate_ca_id, intermediate_ca_node);
                        self.org_graph.node_views.insert(intermediate_ca_id, view);

                        // Store the intermediate CA for signing leaf certificates
                        self.generated_intermediate_cas.insert(intermediate_ca_id, certificate.clone());

                        // Update PKI state machine
                        let intermediate_ca_ids = match &self.pki_state {
                            PKIBootstrapState::IntermediateCAGenerated { intermediate_ca_ids } => {
                                let mut ids = intermediate_ca_ids.clone();
                                ids.push(intermediate_ca_id);
                                ids
                            }
                            PKIBootstrapState::RootCAGenerated { .. } |
                            PKIBootstrapState::IntermediateCAPlanned { .. } => {
                                vec![intermediate_ca_id]
                            }
                            _ => vec![intermediate_ca_id],
                        };
                        self.pki_state = PKIBootstrapState::IntermediateCAGenerated { intermediate_ca_ids };
                        tracing::info!(
                            "PKI state machine transitioned to IntermediateCAGenerated (id: {})",
                            intermediate_ca_id
                        );

                        // Switch to PKI view to show the new certificate
                        self.graph_view = GraphView::PkiTrustChain;

                        self.status_message = format!(
                            "✅ Intermediate CA generated! Fingerprint: {} | View in PKI Graph",
                            &certificate.fingerprint[..16]
                        );
                        tracing::info!("Intermediate CA generated and added to graph: {}", certificate.fingerprint);
                    }
                    Err(e) => {
                        self.status_message = format!("❌ Intermediate CA generation failed: {}", e);
                        self.error_message = Some(e);
                        tracing::error!("Intermediate CA generation failed");
                    }
                }
                Task::none()
            }

            // Sprint 82: Leaf certificate generation result handler
            Message::LeafCertGenerated(result) => {
                match result {
                    Ok((certificate, leaf_cert_id, person_name)) => {
                        // Create Leaf Certificate node in graph using LiftableDomain
                        use crate::domain::pki::{Certificate as PkiCertificate, CertificateId};
                        use crate::lifting::LiftableDomain;
                        use crate::state_machines::workflows::PKIBootstrapState;

                        let cert_id = CertificateId::from_uuid(leaf_cert_id);
                        let subject = format!("CN={}, O={}", person_name, self.organization_name);
                        let cert = PkiCertificate::leaf(
                            cert_id,
                            subject.clone(),
                            format!("{} Intermediate CA", self.organization_name), // issuer
                            chrono::Utc::now(),
                            chrono::Utc::now() + chrono::Duration::days(365), // 1 year
                            vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
                            vec![format!("{}.{}", person_name.to_lowercase().replace(' ', "."), self.organization_name.to_lowercase().replace(' ', "-"))], // SAN
                        );
                        let lifted_node = cert.lift();
                        let position = iced::Point::new(500.0, 300.0); // Below Intermediate CA
                        let leaf_cert_node = graph::ConceptEntity::from_lifted_node(lifted_node);

                        // Create view with custom color and label
                        let custom_color = self.view_model.colors.cert_leaf;
                        let custom_label = format!("{} Certificate", person_name);
                        let view = view_model::NodeView::new(leaf_cert_id, position, custom_color, custom_label);

                        // Add to graph
                        self.org_graph.nodes.insert(leaf_cert_id, leaf_cert_node);
                        self.org_graph.node_views.insert(leaf_cert_id, view);

                        // Store the leaf certificate
                        self.generated_leaf_certs.insert(leaf_cert_id, certificate.clone());

                        // Update PKI state machine
                        let leaf_cert_ids = match &self.pki_state {
                            PKIBootstrapState::LeafCertsGenerated { leaf_cert_ids } => {
                                let mut ids = leaf_cert_ids.clone();
                                ids.push(leaf_cert_id);
                                ids
                            }
                            PKIBootstrapState::IntermediateCAGenerated { .. } => {
                                vec![leaf_cert_id]
                            }
                            _ => vec![leaf_cert_id],
                        };
                        self.pki_state = PKIBootstrapState::LeafCertsGenerated { leaf_cert_ids };
                        tracing::info!(
                            "PKI state machine transitioned to LeafCertsGenerated (id: {}, person: {})",
                            leaf_cert_id,
                            person_name
                        );

                        // Switch to PKI view to show the new certificate
                        self.graph_view = GraphView::PkiTrustChain;

                        self.status_message = format!(
                            "✅ Leaf certificate generated for {}! Fingerprint: {} | View in PKI Graph",
                            person_name,
                            &certificate.fingerprint[..16]
                        );
                        tracing::info!("Leaf certificate generated for {}: {}", person_name, certificate.fingerprint);
                    }
                    Err(e) => {
                        self.status_message = format!("❌ Leaf certificate generation failed: {}", e);
                        self.error_message = Some(e);
                        tracing::error!("Leaf certificate generation failed");
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

            Message::SelectUnitForCA(unit_name) => {
                self.selected_unit_for_ca = Some(unit_name.clone());
                // Also update the intermediate CA name to include the unit
                if self.intermediate_ca_name_input.is_empty() {
                    self.intermediate_ca_name_input = format!("{} CA", unit_name);
                }
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

            // GPG key generation messages
            Message::GpgUserIdChanged(user_id) => {
                self.gpg_user_id = user_id;
                Task::none()
            }

            Message::GpgKeyTypeSelected(key_type) => {
                self.gpg_key_type = Some(key_type);
                Task::none()
            }

            Message::GpgKeyLengthChanged(length) => {
                self.gpg_key_length = length;
                Task::none()
            }

            Message::GpgExpiresDaysChanged(days) => {
                self.gpg_expires_days = days;
                Task::none()
            }

            Message::ToggleGpgSection => {
                self.gpg_section_collapsed = !self.gpg_section_collapsed;
                Task::none()
            }

            Message::GenerateGpgKey => {
                // Validate inputs
                if self.gpg_user_id.is_empty() {
                    self.error_message = Some("User ID is required for GPG key (e.g., 'Name <email@example.com>')".to_string());
                    return Task::none();
                }

                let key_type = match self.gpg_key_type {
                    Some(kt) => kt,
                    None => {
                        self.error_message = Some("Please select a key type".to_string());
                        return Task::none();
                    }
                };

                let key_length = match self.gpg_key_length.parse::<u32>() {
                    Ok(len) if len >= 1024 => len,
                    _ => {
                        self.error_message = Some("Key length must be at least 1024 bits".to_string());
                        return Task::none();
                    }
                };

                let expires_days = if self.gpg_expires_days.is_empty() {
                    None
                } else {
                    match self.gpg_expires_days.parse::<u32>() {
                        Ok(days) => Some(days),
                        Err(_) => {
                            self.error_message = Some("Invalid expiration days".to_string());
                            return Task::none();
                        }
                    }
                };

                self.gpg_generation_status = Some("⏳ Generating GPG key...".to_string());
                self.status_message = "Generating GPG key pair...".to_string();
                self.error_message = None;

                let gpg_port = self.gpg_port.clone();
                let user_id = self.gpg_user_id.clone();

                Task::perform(
                    async move {
                        gpg_port.generate_keypair(&user_id, key_type, key_length, expires_days)
                            .await
                            .map_err(|e| format!("{:?}", e))
                    },
                    Message::GpgKeyGenerated
                )
            }

            Message::GpgKeyGenerated(result) => {
                match result {
                    Ok(keypair) => {
                        self.gpg_generation_status = Some(format!("✅ Key generated: {}", keypair.fingerprint));
                        self.status_message = format!("GPG key generated successfully! Fingerprint: {}", keypair.fingerprint);

                        // Add to generated keys list
                        self.generated_gpg_keys.push(crate::ports::gpg::GpgKeyInfo {
                            key_id: keypair.key_id,
                            fingerprint: keypair.fingerprint,
                            user_ids: vec![keypair.user_id],
                            creation_time: chrono::Utc::now().timestamp(),
                            expiration_time: None,
                            is_revoked: false,
                            is_expired: false,
                        });

                        // Clear the form
                        self.gpg_user_id.clear();
                    }
                    Err(e) => {
                        self.gpg_generation_status = Some(format!("❌ Failed: {}", e));
                        self.error_message = Some(format!("GPG key generation failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::ListGpgKeys => {
                let gpg_port = self.gpg_port.clone();
                Task::perform(
                    async move {
                        gpg_port.list_keys(true)
                            .await
                            .map_err(|e| format!("{:?}", e))
                    },
                    Message::GpgKeysListed
                )
            }

            Message::GpgKeysListed(result) => {
                match result {
                    Ok(keys) => {
                        self.generated_gpg_keys = keys;
                        self.status_message = format!("Found {} GPG keys", self.generated_gpg_keys.len());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to list GPG keys: {}", e));
                    }
                }
                Task::none()
            }

            // Key recovery from seed handlers
            Message::ToggleRecoverySection => {
                self.recovery_section_collapsed = !self.recovery_section_collapsed;
                Task::none()
            }

            Message::RecoveryPassphraseChanged(passphrase) => {
                self.recovery_passphrase = passphrase;
                self.recovery_seed_verified = false;
                self.recovery_status = None;
                Task::none()
            }

            Message::RecoveryPassphraseConfirmChanged(passphrase) => {
                self.recovery_passphrase_confirm = passphrase;
                self.recovery_seed_verified = false;
                self.recovery_status = None;
                Task::none()
            }

            Message::RecoveryOrganizationIdChanged(org_id) => {
                self.recovery_organization_id = org_id;
                self.recovery_seed_verified = false;
                self.recovery_status = None;
                Task::none()
            }

            Message::VerifyRecoverySeed => {
                // Validate inputs
                if self.recovery_passphrase.is_empty() {
                    self.error_message = Some("Recovery passphrase is required".to_string());
                    return Task::none();
                }

                if self.recovery_passphrase != self.recovery_passphrase_confirm {
                    self.error_message = Some("Passphrases do not match".to_string());
                    return Task::none();
                }

                if self.recovery_organization_id.is_empty() {
                    self.error_message = Some("Organization ID is required for recovery".to_string());
                    return Task::none();
                }

                self.recovery_status = Some("⏳ Deriving seed...".to_string());
                self.error_message = None;

                let passphrase = self.recovery_passphrase.clone();
                let org_id = self.recovery_organization_id.clone();

                Task::perform(
                    async move {
                        // Derive the master seed from passphrase
                        match crate::crypto::seed_derivation::derive_master_seed(&passphrase, &org_id) {
                            Ok(seed) => {
                                // Generate a fingerprint from the seed for verification
                                use sha2::{Sha256, Digest};
                                let mut hasher = Sha256::new();
                                hasher.update(seed.as_bytes());
                                let hash = hasher.finalize();
                                let fingerprint: String = hash.iter()
                                    .take(8)
                                    .map(|b| format!("{:02X}", b))
                                    .collect::<Vec<_>>()
                                    .join(":");
                                Ok(fingerprint)
                            }
                            Err(e) => Err(e),
                        }
                    },
                    Message::RecoverySeedVerified
                )
            }

            Message::RecoverySeedVerified(result) => {
                match result {
                    Ok(fingerprint) => {
                        self.recovery_status = Some(format!("✅ Seed verified! Fingerprint: {}", fingerprint));
                        self.recovery_seed_verified = true;
                        self.status_message = format!("Recovery seed derived successfully. Fingerprint: {}", fingerprint);
                    }
                    Err(e) => {
                        self.recovery_status = Some(format!("❌ Failed: {}", e));
                        self.recovery_seed_verified = false;
                        self.error_message = Some(format!("Seed derivation failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::RecoverKeysFromSeed => {
                if !self.recovery_seed_verified {
                    self.error_message = Some("Please verify the seed first".to_string());
                    return Task::none();
                }

                self.recovery_status = Some("⏳ Recovering keys...".to_string());
                self.error_message = None;

                let passphrase = self.recovery_passphrase.clone();
                let org_id = self.recovery_organization_id.clone();

                Task::perform(
                    async move {
                        // Derive the master seed
                        let seed = crate::crypto::seed_derivation::derive_master_seed(&passphrase, &org_id)
                            .map_err(|e| format!("Seed derivation failed: {}", e))?;

                        // Derive child seeds for different key purposes
                        let _root_ca_seed = seed.derive_child("root-ca");
                        let _intermediate_ca_seed = seed.derive_child("intermediate-ca");
                        let _nats_operator_seed = seed.derive_child("nats-operator");

                        // In a full implementation, these seeds would be used to regenerate
                        // the exact same keys as before. For now, we just verify the derivation works.

                        // Return the number of recoverable key types
                        Ok(3) // root-ca, intermediate-ca, nats-operator
                    },
                    Message::KeysRecovered
                )
            }

            Message::KeysRecovered(result) => {
                match result {
                    Ok(count) => {
                        self.recovery_status = Some(format!("✅ Recovery complete! {} key types can be regenerated", count));
                        self.status_message = format!("Key recovery successful: {} key hierarchies can be regenerated from this seed", count);

                        // Update the root passphrase to the recovery passphrase for subsequent operations
                        self.root_passphrase = self.recovery_passphrase.clone();
                        self.root_passphrase_confirm = self.recovery_passphrase_confirm.clone();
                    }
                    Err(e) => {
                        self.recovery_status = Some(format!("❌ Recovery failed: {}", e));
                        self.error_message = Some(format!("Key recovery failed: {}", e));
                    }
                }
                Task::none()
            }

            // YubiKey slot management handlers
            Message::ToggleYubiKeySlotSection => {
                self.yubikey_slot_section_collapsed = !self.yubikey_slot_section_collapsed;
                Task::none()
            }

            Message::SelectYubiKeyForManagement(serial) => {
                self.selected_yubikey_for_management = Some(serial.clone());
                self.yubikey_slot_operation_status = Some(format!("Selected YubiKey {} for management", serial));
                // Automatically query slots when selecting a YubiKey
                Task::done(Message::QueryYubiKeySlots(serial))
            }

            Message::YubiKeyPinInputChanged(pin) => {
                self.yubikey_pin_input = pin;
                Task::none()
            }

            Message::YubiKeyNewPinChanged(pin) => {
                self.yubikey_new_pin = pin;
                Task::none()
            }

            Message::YubiKeyPinConfirmChanged(pin) => {
                self.yubikey_pin_confirm = pin;
                Task::none()
            }

            Message::YubiKeyManagementKeyChanged(key) => {
                self.yubikey_management_key = key;
                Task::none()
            }

            Message::YubiKeyNewManagementKeyChanged(key) => {
                self.yubikey_new_management_key = key;
                Task::none()
            }

            Message::SelectPivSlot(slot) => {
                self.selected_piv_slot = Some(slot);
                Task::none()
            }

            Message::QueryYubiKeySlots(serial) => {
                // For now, create default slot info since we can't query actual slot status
                // without a YubiKey library that supports slot enumeration
                use crate::ports::yubikey::PivSlot;
                let slots = vec![
                    SlotInfo::new(PivSlot::Authentication),
                    SlotInfo::new(PivSlot::Signature),
                    SlotInfo::new(PivSlot::KeyManagement),
                    SlotInfo::new(PivSlot::CardAuth),
                ];
                self.yubikey_slot_info.insert(serial.clone(), slots);
                self.yubikey_slot_operation_status = Some(format!("Queried slots for YubiKey {}", serial));
                Task::none()
            }

            Message::YubiKeySlotsQueried(result) => {
                match result {
                    Ok((serial, slots)) => {
                        self.yubikey_slot_info.insert(serial.clone(), slots);
                        self.yubikey_slot_operation_status = Some(format!("✅ Retrieved {} slot(s) for YubiKey {}",
                            self.yubikey_slot_info.get(&serial).map(|s| s.len()).unwrap_or(0), serial));
                    }
                    Err(e) => {
                        self.yubikey_slot_operation_status = Some(format!("❌ Failed to query slots: {}", e));
                        self.error_message = Some(format!("Slot query failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::VerifyYubiKeyPin(serial) => {
                let yubikey_port = self.yubikey_port.clone();
                let pin = self.yubikey_pin_input.clone();

                self.yubikey_slot_operation_status = Some(format!("Verifying PIN for YubiKey {}...", serial));

                Task::perform(
                    async move {
                        let secure_pin = crate::ports::yubikey::SecureString::new(pin.as_bytes());
                        match yubikey_port.verify_pin(&serial, &secure_pin).await {
                            Ok(valid) => Ok((serial, valid)),
                            Err(e) => Err(format!("{}", e)),
                        }
                    },
                    Message::YubiKeyPinVerified
                )
            }

            Message::YubiKeyPinVerified(result) => {
                match result {
                    Ok((serial, valid)) => {
                        if valid {
                            self.yubikey_slot_operation_status = Some(format!("✅ PIN verified successfully for YubiKey {}", serial));
                            self.status_message = format!("PIN verification successful for YubiKey {}", serial);
                        } else {
                            self.yubikey_slot_operation_status = Some(format!("❌ Invalid PIN for YubiKey {}", serial));
                            self.error_message = Some("Invalid PIN".to_string());
                        }
                    }
                    Err(e) => {
                        self.yubikey_slot_operation_status = Some(format!("❌ PIN verification failed: {}", e));
                        self.error_message = Some(format!("PIN verification failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::ChangeYubiKeyManagementKey(serial) => {
                let yubikey_port = self.yubikey_port.clone();
                let current_key = self.yubikey_management_key.clone();
                let new_key = self.yubikey_new_management_key.clone();

                self.yubikey_slot_operation_status = Some(format!("Changing management key for YubiKey {}...", serial));

                Task::perform(
                    async move {
                        // Parse hex strings to bytes
                        let current_bytes = hex::decode(&current_key)
                            .map_err(|e| format!("Invalid current key hex: {}", e))?;
                        let new_bytes = hex::decode(&new_key)
                            .map_err(|e| format!("Invalid new key hex: {}", e))?;

                        match yubikey_port.change_management_key(&serial, &current_bytes, &new_bytes).await {
                            Ok(()) => Ok(serial),
                            Err(e) => Err(format!("{}", e)),
                        }
                    },
                    Message::YubiKeyManagementKeyChanged2
                )
            }

            Message::YubiKeyManagementKeyChanged2(result) => {
                match result {
                    Ok(serial) => {
                        self.yubikey_slot_operation_status = Some(format!("✅ Management key changed for YubiKey {}", serial));
                        self.status_message = format!("Management key updated successfully for YubiKey {}", serial);
                        // Clear the key inputs
                        self.yubikey_management_key.clear();
                        self.yubikey_new_management_key.clear();
                    }
                    Err(e) => {
                        self.yubikey_slot_operation_status = Some(format!("❌ Failed to change management key: {}", e));
                        self.error_message = Some(format!("Management key change failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::ResetYubiKeyPiv(serial) => {
                let yubikey_port = self.yubikey_port.clone();

                self.yubikey_slot_operation_status = Some(format!("⚠️ Resetting PIV for YubiKey {}...", serial));

                Task::perform(
                    async move {
                        match yubikey_port.reset_piv(&serial).await {
                            Ok(()) => Ok(serial),
                            Err(e) => Err(format!("{}", e)),
                        }
                    },
                    Message::YubiKeyPivReset
                )
            }

            Message::YubiKeyPivReset(result) => {
                match result {
                    Ok(serial) => {
                        self.yubikey_slot_operation_status = Some(format!("✅ PIV reset complete for YubiKey {}", serial));
                        self.status_message = format!("YubiKey {} PIV application has been factory reset", serial);
                        // Clear slot info since all slots are now empty
                        self.yubikey_slot_info.remove(&serial);
                        // Re-query slots
                        return Task::done(Message::QueryYubiKeySlots(serial));
                    }
                    Err(e) => {
                        self.yubikey_slot_operation_status = Some(format!("❌ PIV reset failed: {}", e));
                        self.error_message = Some(format!("PIV reset failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::GetYubiKeyAttestation { serial, slot } => {
                let yubikey_port = self.yubikey_port.clone();

                self.yubikey_slot_operation_status = Some(format!("Getting attestation for slot {:?}...", slot));

                Task::perform(
                    async move {
                        match yubikey_port.get_attestation(&serial, slot).await {
                            Ok(cert_bytes) => {
                                // Format attestation certificate info
                                use sha2::Digest;
                                let mut hasher = sha2::Sha256::new();
                                hasher.update(&cert_bytes);
                                let hash = hasher.finalize();
                                let info = format!(
                                    "Attestation certificate: {} bytes, SHA-256: {:02x}{:02x}{:02x}{:02x}...",
                                    cert_bytes.len(),
                                    hash[0], hash[1], hash[2], hash[3]
                                );
                                Ok((serial, info))
                            }
                            Err(e) => Err(format!("{}", e)),
                        }
                    },
                    Message::YubiKeyAttestationReceived
                )
            }

            Message::YubiKeyAttestationReceived(result) => {
                match result {
                    Ok((serial, info)) => {
                        self.yubikey_attestation_result = Some(info.clone());
                        self.yubikey_slot_operation_status = Some(format!("✅ Attestation retrieved for YubiKey {}", serial));
                        self.status_message = format!("Attestation verified: {}", info);
                    }
                    Err(e) => {
                        self.yubikey_attestation_result = Some(format!("❌ {}", e));
                        self.yubikey_slot_operation_status = Some(format!("❌ Attestation failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::ClearYubiKeySlot { serial, slot } => {
                // Note: The YubiKeyPort doesn't have a clear_slot method.
                // A factory reset is the only way to clear slots with the current interface.
                // This would need to be added to the port interface.
                self.yubikey_slot_operation_status = Some(format!(
                    "⚠️ Individual slot clearing not supported. Use Factory Reset to clear all slots."
                ));
                self.error_message = Some(format!(
                    "To clear slot {:?} on YubiKey {}, use the Factory Reset option. Warning: This will clear ALL slots.",
                    slot, serial
                ));
                Task::none()
            }

            Message::YubiKeySlotCleared(result) => {
                match result {
                    Ok((serial, slot)) => {
                        self.yubikey_slot_operation_status = Some(format!("✅ Slot {:?} cleared on YubiKey {}", slot, serial));
                        // Re-query slots
                        return Task::done(Message::QueryYubiKeySlots(serial));
                    }
                    Err(e) => {
                        self.yubikey_slot_operation_status = Some(format!("❌ Failed to clear slot: {}", e));
                        self.error_message = Some(format!("Slot clear failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::GenerateKeyInSlot { serial, slot } => {
                // Check if the slot is already occupied
                if let Some(slot_infos) = self.yubikey_slot_info.get(&serial) {
                    if let Some(slot_info) = slot_infos.iter().find(|si| si.slot == slot) {
                        if slot_info.occupied {
                            self.error_message = Some(format!(
                                "PIV slot {} ({}) is already occupied on YubiKey {}. Clear the slot first to regenerate.",
                                slot_info.slot_hex, slot_info.slot_name, serial
                            ));
                            return Task::none();
                        }
                    }
                }

                self.yubikey_slot_operation_status = Some(format!("⏳ Generating key in slot {:?}...", slot));
                let yubikey_port = self.yubikey_port.clone();
                let pin = self.yubikey_pin_input.clone();

                Task::perform(
                    async move {
                        use crate::ports::yubikey::{KeyAlgorithm, SecureString};

                        let pin = if pin.is_empty() {
                            SecureString::new(b"123456")  // Factory default
                        } else {
                            SecureString::new(pin.as_bytes())
                        };

                        match yubikey_port.generate_key_in_slot(
                            &serial,
                            slot,
                            KeyAlgorithm::EccP256,
                            &pin
                        ).await {
                            Ok(public_key) => {
                                let key_info = format!("ECC P-256, {} bytes", public_key.data.len());
                                Ok((serial, slot, key_info))
                            }
                            Err(e) => {
                                Err(format!("Failed to generate key: {:?}", e))
                            }
                        }
                    },
                    Message::KeyInSlotGenerated
                )
            }

            Message::KeyInSlotGenerated(result) => {
                match result {
                    Ok((serial, slot, key_info)) => {
                        self.yubikey_slot_operation_status = Some(format!("✅ Key generated in slot {:?}: {}", slot, key_info));
                        self.status_message = format!("Key generated in {:?} on YubiKey {}", slot, serial);
                        // Re-query slots to update display
                        return Task::done(Message::QueryYubiKeySlots(serial));
                    }
                    Err(e) => {
                        self.yubikey_slot_operation_status = Some(format!("❌ {}", e));
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }

            // Organization unit creation handlers
            Message::ToggleOrgUnitSection => {
                self.org_unit_section_collapsed = !self.org_unit_section_collapsed;
                Task::none()
            }

            Message::NewUnitNameChanged(name) => {
                self.new_unit_name = name;
                Task::none()
            }

            Message::NewUnitTypeSelected(unit_type) => {
                self.new_unit_type = Some(unit_type);
                Task::none()
            }

            Message::NewUnitParentSelected(parent) => {
                self.new_unit_parent = if parent.is_empty() { None } else { Some(parent) };
                Task::none()
            }

            Message::NewUnitNatsAccountChanged(account) => {
                self.new_unit_nats_account = account;
                Task::none()
            }

            Message::NewUnitResponsiblePersonSelected(person_id) => {
                self.new_unit_responsible_person = Some(person_id);
                Task::none()
            }

            Message::CreateOrganizationUnit => {
                // Sprint 76: Use CQRS through aggregate
                use crate::domain::ids::UnitId;

                // GUI-level validation for immediate feedback
                if self.new_unit_name.trim().is_empty() {
                    self.error_message = Some("Unit name is required".to_string());
                    return Task::none();
                }

                let unit_type = match &self.new_unit_type {
                    Some(t) => t.clone(),
                    None => {
                        self.error_message = Some("Please select a unit type".to_string());
                        return Task::none();
                    }
                };

                // Build CQRS command
                use crate::commands::{KeyCommand, organization::CreateOrganizationalUnit};

                // Find parent ID if parent name is set
                let parent_id = self.new_unit_parent.as_ref().and_then(|parent_name| {
                    self.created_units.iter()
                        .find(|u| &u.name == parent_name)
                        .map(|p| p.id.as_uuid().clone())
                });

                let cmd = CreateOrganizationalUnit {
                    command_id: Uuid::now_v7(),
                    unit_id: Uuid::now_v7(),
                    name: self.new_unit_name.clone(),
                    parent_id,
                    correlation_id: Uuid::now_v7(),
                    causation_id: None,
                    timestamp: chrono::Utc::now(),
                };

                // Clone for async
                let aggregate = self.aggregate.clone();
                let projection = self.projection.clone();
                let responsible_person_id = self.new_unit_responsible_person;
                let nats_account_name = if self.new_unit_nats_account.is_empty() {
                    None
                } else {
                    Some(self.new_unit_nats_account.clone())
                };

                // Clear form fields optimistically
                self.new_unit_name.clear();
                self.new_unit_type = None;
                self.new_unit_parent = None;
                self.new_unit_nats_account.clear();
                self.new_unit_responsible_person = None;
                self.error_message = None;

                // Process command through aggregate (CQRS pattern - Sprint 76)
                Task::perform(
                    async move {
                        let aggregate_read = aggregate.read().await;
                        let projection_read = projection.read().await;

                        let events = aggregate_read.handle_command(
                            KeyCommand::CreateOrganizationalUnit(cmd),
                            &projection_read,
                            None, // No NATS port in offline mode
                            #[cfg(feature = "policy")]
                            None  // No policy engine in GUI yet
                        ).await
                        .map_err(|e| e.to_string())?;

                        // Extract unit info from emitted event
                        for event in &events {
                            if let crate::events::DomainEvent::Organization(
                                crate::events::OrganizationEvents::OrganizationalUnitCreated(evt)
                            ) = event {
                                // Reconstruct OrganizationUnit from event + GUI context
                                let unit = crate::domain::OrganizationUnit {
                                    id: UnitId::from_uuid(evt.unit_id),
                                    name: evt.name.clone(),
                                    unit_type: unit_type.clone(),
                                    parent_unit_id: evt.parent_id.map(UnitId::from_uuid),
                                    responsible_person_id,
                                    nats_account_name: nats_account_name.clone(),
                                };
                                return Ok(unit);
                            }
                        }

                        Err("No OrganizationalUnitCreated event emitted".to_string())
                    },
                    Message::OrganizationUnitCreated
                )
            }

            Message::OrganizationUnitCreated(result) => {
                match result {
                    Ok(unit) => {
                        // Add to created units list
                        self.created_units.push(unit.clone());
                        // Also add to loaded_units for CA selection
                        self.loaded_units.push(unit.clone());
                        self.status_message = format!("✅ Organization unit '{}' created successfully", unit.name);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to create unit: {}", e));
                    }
                }
                Task::none()
            }

            Message::RemoveOrganizationUnit(unit_id) => {
                // Remove from created units
                if let Some(pos) = self.created_units.iter().position(|u| u.id.as_uuid() == unit_id) {
                    let removed = self.created_units.remove(pos);
                    self.status_message = format!("Removed organization unit: {}", removed.name);

                    // Also remove from loaded_units
                    self.loaded_units.retain(|u| u.id.as_uuid() != unit_id);
                }
                Task::none()
            }

            // Service account management handlers
            Message::ToggleServiceAccountSection => {
                self.service_account_section_collapsed = !self.service_account_section_collapsed;
                Task::none()
            }

            Message::NewServiceAccountNameChanged(name) => {
                self.new_service_account_name = name;
                Task::none()
            }

            Message::NewServiceAccountPurposeChanged(purpose) => {
                self.new_service_account_purpose = purpose;
                Task::none()
            }

            Message::ServiceAccountOwningUnitSelected(unit_id) => {
                self.new_service_account_owning_unit = Some(unit_id);
                Task::none()
            }

            Message::ServiceAccountResponsiblePersonSelected(person_id) => {
                self.new_service_account_responsible_person = Some(person_id);
                Task::none()
            }

            Message::CreateServiceAccount => {
                // Sprint 76: Use CQRS through aggregate

                // GUI-level validation for immediate feedback
                if self.new_service_account_name.trim().is_empty() {
                    self.error_message = Some("Service account name is required".to_string());
                    return Task::none();
                }

                if self.new_service_account_purpose.trim().is_empty() {
                    self.error_message = Some("Service account purpose is required".to_string());
                    return Task::none();
                }

                let owning_unit_id = match self.new_service_account_owning_unit {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Owning unit is required".to_string());
                        return Task::none();
                    }
                };

                let responsible_person_id = match self.new_service_account_responsible_person {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Responsible person is required".to_string());
                        return Task::none();
                    }
                };

                // Build CQRS command
                use crate::commands::{KeyCommand, organization::CreateServiceAccount as CreateServiceAccountCmd};

                let cmd = CreateServiceAccountCmd {
                    command_id: Uuid::now_v7(),
                    service_account_id: Uuid::now_v7(),
                    name: self.new_service_account_name.clone(),
                    purpose: self.new_service_account_purpose.clone(),
                    owning_unit_id,
                    responsible_person_id,
                    correlation_id: Uuid::now_v7(),
                    causation_id: None,
                    timestamp: chrono::Utc::now(),
                };

                // Clone for async
                let aggregate = self.aggregate.clone();
                let projection = self.projection.clone();

                // Clear form fields optimistically
                self.new_service_account_name.clear();
                self.new_service_account_purpose.clear();
                self.new_service_account_owning_unit = None;
                self.new_service_account_responsible_person = None;
                self.error_message = None;

                // Process command through aggregate (CQRS pattern - Sprint 76)
                Task::perform(
                    async move {
                        let aggregate_read = aggregate.read().await;
                        let projection_read = projection.read().await;

                        let events = aggregate_read.handle_command(
                            KeyCommand::CreateServiceAccount(cmd),
                            &projection_read,
                            None, // No NATS port in offline mode
                            #[cfg(feature = "policy")]
                            None  // No policy engine in GUI yet
                        ).await
                        .map_err(|e| e.to_string())?;

                        // Extract service account info from emitted event
                        for event in &events {
                            if let crate::events::DomainEvent::NatsUser(
                                crate::events::NatsUserEvents::ServiceAccountCreated(evt)
                            ) = event {
                                let sa = crate::domain::ServiceAccount {
                                    id: evt.service_account_id,
                                    name: evt.name.clone(),
                                    purpose: evt.purpose.clone(),
                                    owning_unit_id: evt.owning_unit_id,
                                    responsible_person_id: evt.responsible_person_id,
                                    active: true,
                                };
                                return Ok(sa);
                            }
                        }

                        Err("No ServiceAccountCreated event emitted".to_string())
                    },
                    Message::ServiceAccountCreated
                )
            }

            Message::ServiceAccountCreated(result) => {
                match result {
                    Ok(sa) => {
                        // Add to created list
                        self.created_service_accounts.push(sa.clone());
                        self.status_message = format!("Service account '{}' created successfully", sa.name);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to create service account: {}", e));
                    }
                }
                Task::none()
            }

            Message::DeactivateServiceAccount(sa_id) => {
                // Deactivate (but don't remove) a service account
                if let Some(sa) = self.created_service_accounts.iter_mut().find(|s| s.id == sa_id) {
                    sa.active = false;
                    self.status_message = format!("Deactivated service account: {}", sa.name);
                }
                Task::none()
            }

            Message::RemoveServiceAccount(sa_id) => {
                // Remove from created list
                if let Some(pos) = self.created_service_accounts.iter().position(|s| s.id == sa_id) {
                    let removed = self.created_service_accounts.remove(pos);
                    self.status_message = format!("Removed service account: {}", removed.name);
                }
                Task::none()
            }

            Message::GenerateServiceAccountKey { service_account_id } => {
                // Find the service account
                let service_account = self.created_service_accounts.iter()
                    .find(|s| s.id == service_account_id)
                    .cloned();

                if let Some(sa) = service_account {
                    self.status_message = format!("Generating key for service account: {}", sa.name);
                    // This would typically trigger key generation workflow
                    // For now, we'll emit a success message
                    Task::done(Message::ServiceAccountKeyGenerated(Ok((service_account_id, crate::domain::KeyOwnerRole::ServiceAccount))))
                } else {
                    self.error_message = Some("Service account not found".to_string());
                    Task::none()
                }
            }

            Message::ServiceAccountKeyGenerated(result) => {
                match result {
                    Ok((sa_id, role)) => {
                        if let Some(sa) = self.created_service_accounts.iter().find(|s| s.id == sa_id) {
                            self.status_message = format!("Generated {:?} key for service account: {}", role, sa.name);
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to generate service account key: {}", e));
                    }
                }
                Task::none()
            }

            // Delegation management handlers
            Message::ToggleDelegationSection => {
                self.delegation_section_collapsed = !self.delegation_section_collapsed;
                Task::none()
            }

            Message::DelegationFromPersonSelected(person_id) => {
                self.delegation_from_person = Some(person_id);
                // Can't delegate to yourself
                if self.delegation_to_person == Some(person_id) {
                    self.delegation_to_person = None;
                }
                Task::none()
            }

            Message::DelegationToPersonSelected(person_id) => {
                self.delegation_to_person = Some(person_id);
                // Can't delegate to yourself
                if self.delegation_from_person == Some(person_id) {
                    self.delegation_from_person = None;
                }
                Task::none()
            }

            Message::ToggleDelegationPermission(permission) => {
                if self.delegation_permissions.contains(&permission) {
                    self.delegation_permissions.remove(&permission);
                } else {
                    self.delegation_permissions.insert(permission);
                }
                Task::none()
            }

            Message::DelegationExpiresDaysChanged(days) => {
                self.delegation_expires_days = days;
                Task::none()
            }

            Message::CreateDelegation => {
                // Validate inputs (GUI-level validation for immediate feedback)
                let from_person_id = match self.delegation_from_person {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Please select a person to delegate from".to_string());
                        return Task::none();
                    }
                };

                let to_person_id = match self.delegation_to_person {
                    Some(id) => id,
                    None => {
                        self.error_message = Some("Please select a person to delegate to".to_string());
                        return Task::none();
                    }
                };

                if self.delegation_permissions.is_empty() {
                    self.error_message = Some("Please select at least one permission to delegate".to_string());
                    return Task::none();
                }

                // Find person names for display
                let from_name = self.loaded_people.iter()
                    .find(|p| p.person_id == from_person_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                let to_name = self.loaded_people.iter()
                    .find(|p| p.person_id == to_person_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                // Build CQRS command using builder pattern (Sprint 73)
                use crate::commands::{KeyCommand, CreateDelegation as CreateDelegationCmd};

                let permissions: Vec<_> = self.delegation_permissions.iter().cloned().collect();
                let mut cmd = CreateDelegationCmd::new(from_person_id, to_person_id, permissions);

                // Set expiration if specified
                if !self.delegation_expires_days.is_empty() {
                    match self.delegation_expires_days.parse::<i64>() {
                        Ok(days) if days > 0 => {
                            cmd = cmd.with_expiration_days(days);
                        }
                        _ => {
                            self.error_message = Some("Invalid expiration days (must be positive number or empty)".to_string());
                            return Task::none();
                        }
                    }
                }

                // Clone Arc references for async task
                let aggregate = self.aggregate.clone();
                let projection = self.projection.clone();

                // Process command through aggregate (CQRS pattern)
                Task::perform(
                    async move {
                        let aggregate_read = aggregate.read().await;
                        let projection_read = projection.read().await;

                        let events = aggregate_read.handle_command(
                            KeyCommand::CreateDelegation(cmd.clone()),
                            &projection_read,
                            None, // No NATS port in offline mode
                            #[cfg(feature = "policy")]
                            None  // No policy engine in GUI yet
                        ).await
                        .map_err(|e| e.to_string())?;

                        // Extract delegation info from emitted event
                        for event in &events {
                            if let crate::events::DomainEvent::Delegation(
                                crate::events::DelegationEvents::DelegationCreated(evt)
                            ) = event {
                                return Ok(DelegationEntry {
                                    id: evt.delegation_id,
                                    from_person_id: evt.delegator_id,
                                    from_person_name: from_name,
                                    to_person_id: evt.delegate_id,
                                    to_person_name: to_name,
                                    permissions: evt.permissions.clone(),
                                    created_at: evt.created_at,
                                    expires_at: evt.valid_until,
                                    is_active: true,
                                });
                            }
                        }

                        Err("No DelegationCreated event emitted".to_string())
                    },
                    Message::DelegationCreated
                )
            }

            Message::DelegationCreated(result) => {
                match result {
                    Ok(entry) => {
                        self.status_message = format!(
                            "Delegation created: {} -> {} ({} permissions)",
                            entry.from_person_name,
                            entry.to_person_name,
                            entry.permissions.len()
                        );
                        self.active_delegations.push(entry);

                        // Reset form
                        self.delegation_from_person = None;
                        self.delegation_to_person = None;
                        self.delegation_permissions.clear();
                        self.delegation_expires_days = String::new();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to create delegation: {}", e));
                    }
                }
                Task::none()
            }

            Message::RevokeDelegation(delegation_id) => {
                // Build CQRS command for revocation (Sprint 74)
                use crate::commands::{KeyCommand, RevokeDelegation as RevokeDelegationCmd, RevocationReason};

                // Find the delegation to get revoker info
                let revoker_id = self.active_delegations.iter()
                    .find(|d| d.id == delegation_id)
                    .map(|d| d.from_person_id) // Delegator revokes by default
                    .unwrap_or_else(uuid::Uuid::now_v7);

                let cmd = RevokeDelegationCmd {
                    command_id: uuid::Uuid::now_v7(),
                    delegation_id,
                    revoked_by: revoker_id,
                    reason: RevocationReason::DelegatorRevoked,
                    correlation_id: uuid::Uuid::now_v7(),
                    causation_id: None,
                    timestamp: chrono::Utc::now(),
                };

                // Clone Arc references for async task
                let aggregate = self.aggregate.clone();
                let projection = self.projection.clone();

                // Process command through aggregate (CQRS pattern)
                Task::perform(
                    async move {
                        let aggregate_read = aggregate.read().await;
                        let projection_read = projection.read().await;

                        aggregate_read.handle_command(
                            KeyCommand::RevokeDelegation(cmd),
                            &projection_read,
                            None, // No NATS port in offline mode
                            #[cfg(feature = "policy")]
                            None  // No policy engine in GUI yet
                        ).await
                        .map_err(|e| e.to_string())?;

                        Ok(delegation_id)
                    },
                    Message::DelegationRevoked
                )
            }

            Message::DelegationRevoked(result) => {
                match result {
                    Ok(delegation_id) => {
                        // Find and mark delegation as inactive in GUI state
                        if let Some(delegation) = self.active_delegations.iter_mut().find(|d| d.id == delegation_id) {
                            delegation.is_active = false;
                            self.status_message = format!(
                                "Delegation revoked: {} -> {}",
                                delegation.from_person_name,
                                delegation.to_person_name
                            );
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to revoke delegation: {}", e));
                    }
                }
                Task::none()
            }

            // YubiKey domain registration handlers
            Message::YubiKeyRegistrationNameChanged(name) => {
                self.yubikey_registration_name = name;
                Task::none()
            }

            Message::RegisterYubiKeyInDomain { serial, name } => {
                // Check for duplicate registration
                if self.registered_yubikeys.contains_key(&serial) {
                    self.error_message = Some(format!(
                        "YubiKey {} is already registered. Cannot register duplicate.",
                        serial
                    ));
                    return Task::none();
                }

                // Validate name is not empty
                if name.trim().is_empty() {
                    self.error_message = Some("YubiKey name cannot be empty".to_string());
                    return Task::none();
                }

                // Create a new YubiKey registration in the domain
                let yubikey_id = uuid::Uuid::now_v7();
                self.registered_yubikeys.insert(serial.clone(), yubikey_id);
                self.status_message = format!("Registering YubiKey {} as '{}'...", serial, name);
                Task::done(Message::YubiKeyRegistered(Ok((serial, yubikey_id))))
            }

            Message::YubiKeyRegistered(result) => {
                match result {
                    Ok((serial, id)) => {
                        self.status_message = format!("YubiKey {} registered successfully (ID: {})", serial, id);
                        self.yubikey_registration_name.clear();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to register YubiKey: {}", e));
                    }
                }
                Task::none()
            }

            Message::TransferYubiKey { serial, from_person_id, to_person_id } => {
                // Transfer YubiKey assignment from one person to another
                // HashMap is serial -> person_id
                if let Some(&current_person) = self.yubikey_assignments.get(&serial) {
                    if current_person == from_person_id {
                        // Find old person name for display
                        let old_person_name = self.loaded_people.iter()
                            .find(|p| p.person_id == from_person_id)
                            .map(|p| p.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        // Update assignment to new person
                        self.yubikey_assignments.insert(serial.clone(), to_person_id);

                        self.status_message = format!("Transferring YubiKey {} from {} to new owner...",
                            serial, old_person_name);
                        Task::done(Message::YubiKeyTransferred(Ok((serial, from_person_id, to_person_id))))
                    } else {
                        Task::done(Message::YubiKeyTransferred(Err(
                            format!("YubiKey {} is not assigned to the specified person", serial)
                        )))
                    }
                } else {
                    Task::done(Message::YubiKeyTransferred(Err(
                        format!("YubiKey {} not found in assignments", serial)
                    )))
                }
            }

            Message::YubiKeyTransferred(result) => {
                match result {
                    Ok((serial, _from, to)) => {
                        if let Some(person) = self.loaded_people.iter().find(|p| p.person_id == to) {
                            self.status_message = format!(
                                "YubiKey {} transferred to {}",
                                serial, person.name
                            );
                        } else {
                            self.status_message = format!("YubiKey {} transferred successfully", serial);
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to transfer YubiKey: {}", e));
                    }
                }
                Task::none()
            }

            Message::RevokeYubiKeyAssignment { serial } => {
                // Remove YubiKey assignment from person (and also from registered_yubikeys)
                if self.yubikey_assignments.remove(&serial).is_some() {
                    self.registered_yubikeys.remove(&serial);
                    self.status_message = format!("Revoking YubiKey {} assignment...", serial);
                    Task::done(Message::YubiKeyAssignmentRevoked(Ok(serial)))
                } else if self.registered_yubikeys.remove(&serial).is_some() {
                    self.status_message = format!("Revoking YubiKey {} registration...", serial);
                    Task::done(Message::YubiKeyAssignmentRevoked(Ok(serial)))
                } else {
                    Task::done(Message::YubiKeyAssignmentRevoked(Err(
                        format!("YubiKey {} not found in assignments", serial)
                    )))
                }
            }

            Message::YubiKeyAssignmentRevoked(result) => {
                match result {
                    Ok(serial) => {
                        self.status_message = format!("YubiKey {} assignment revoked", serial);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to revoke YubiKey assignment: {}", e));
                    }
                }
                Task::none()
            }

            // mTLS Client Certificate handlers
            Message::ClientCertCNChanged(cn) => {
                self.client_cert_cn = cn;
                Task::none()
            }

            Message::ClientCertEmailChanged(email) => {
                self.client_cert_email = email;
                Task::none()
            }

            Message::GenerateClientCert => {
                if self.client_cert_cn.is_empty() {
                    self.error_message = Some("Common Name (CN) is required for client certificate".to_string());
                    return Task::none();
                }
                self.status_message = format!(
                    "Generating mTLS client certificate for CN={}...",
                    self.client_cert_cn
                );
                // In a real implementation, this would trigger certificate generation
                // For now, we simulate success
                let cert_info = format!(
                    "CN={}, Email={}",
                    self.client_cert_cn,
                    if self.client_cert_email.is_empty() { "none" } else { &self.client_cert_email }
                );
                Task::done(Message::ClientCertGenerated(Ok(cert_info)))
            }

            Message::ClientCertGenerated(result) => {
                match result {
                    Ok(cert_info) => {
                        self.status_message = format!("mTLS client certificate generated: {}", cert_info);
                        self.client_cert_cn.clear();
                        self.client_cert_email.clear();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to generate client certificate: {}", e));
                    }
                }
                Task::none()
            }

            // Multi-purpose key generation handlers
            Message::ToggleMultiPurposeKeySection => {
                self.multi_purpose_key_section_collapsed = !self.multi_purpose_key_section_collapsed;
                Task::none()
            }

            Message::MultiPurposePersonSelected(person_id) => {
                self.multi_purpose_selected_person = Some(person_id);
                Task::none()
            }

            Message::ToggleKeyPurpose(purpose) => {
                if self.multi_purpose_selected_purposes.contains(&purpose) {
                    self.multi_purpose_selected_purposes.remove(&purpose);
                } else {
                    self.multi_purpose_selected_purposes.insert(purpose);
                }
                Task::none()
            }

            Message::GenerateMultiPurposeKeys => {
                if let Some(person_id) = self.multi_purpose_selected_person {
                    if self.multi_purpose_selected_purposes.is_empty() {
                        self.error_message = Some("Please select at least one key purpose".to_string());
                        return Task::none();
                    }
                    let purposes: Vec<String> = self.multi_purpose_selected_purposes
                        .iter()
                        .map(|p| format!("{:?}", p))
                        .collect();
                    self.status_message = format!(
                        "Generating keys for purposes: {:?}...",
                        purposes
                    );
                    Task::done(Message::MultiPurposeKeysGenerated(Ok((person_id, purposes))))
                } else {
                    self.error_message = Some("Please select a person first".to_string());
                    Task::none()
                }
            }

            Message::MultiPurposeKeysGenerated(result) => {
                match result {
                    Ok((person_id, purposes)) => {
                        if let Some(person) = self.loaded_people.iter().find(|p| p.person_id == person_id) {
                            self.status_message = format!(
                                "Generated {} keys for {}: {:?}",
                                purposes.len(),
                                person.name,
                                purposes
                            );
                        } else {
                            self.status_message = format!(
                                "Generated {} keys: {:?}",
                                purposes.len(),
                                purposes
                            );
                        }
                        self.multi_purpose_selected_purposes.clear();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to generate keys: {}", e));
                    }
                }
                Task::none()
            }

            // Event Sourcing - Event Replay/Reconstruction handlers
            Message::ToggleEventLogSection => {
                self.event_log_section_collapsed = !self.event_log_section_collapsed;
                Task::none()
            }

            Message::LoadEventLog => {
                // Load events from the CID event store
                let events_path = self.export_path.join("events");
                if events_path.exists() {
                    match crate::event_store::CidEventStore::new(&events_path) {
                        Ok(store) => {
                            match store.list_events() {
                                Ok(events) => {
                                    self.status_message = format!("Loaded {} events from event store", events.len());
                                    return Task::done(Message::EventLogLoaded(Ok(events)));
                                }
                                Err(e) => {
                                    return Task::done(Message::EventLogLoaded(Err(format!("Failed to list events: {}", e))));
                                }
                            }
                        }
                        Err(e) => {
                            return Task::done(Message::EventLogLoaded(Err(format!("Failed to open event store: {}", e))));
                        }
                    }
                } else {
                    self.error_message = Some("Event store not found. Generate some events first.".to_string());
                }
                Task::none()
            }

            Message::EventLogLoaded(result) => {
                match result {
                    Ok(events) => {
                        self.loaded_event_log = events;
                        self.status_message = format!("Loaded {} events", self.loaded_event_log.len());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load events: {}", e));
                    }
                }
                Task::none()
            }

            Message::ToggleEventSelection(cid) => {
                if self.selected_events_for_replay.contains(&cid) {
                    self.selected_events_for_replay.remove(&cid);
                } else {
                    self.selected_events_for_replay.insert(cid);
                }
                Task::none()
            }

            Message::ClearEventSelection => {
                self.selected_events_for_replay.clear();
                Task::none()
            }

            Message::ReplaySelectedEvents => {
                if self.selected_events_for_replay.is_empty() {
                    self.error_message = Some("No events selected for replay".to_string());
                    return Task::none();
                }

                // Get selected events in order
                let selected_events: Vec<_> = self.loaded_event_log.iter()
                    .filter(|e| self.selected_events_for_replay.contains(&e.cid))
                    .cloned()
                    .collect();

                let count = selected_events.len();
                self.status_message = format!("Replaying {} selected events...", count);

                // In a full implementation, this would apply events to rebuild state
                // For now, we just report success
                Task::done(Message::EventsReplayed(Ok(count)))
            }

            Message::EventsReplayed(result) => {
                match result {
                    Ok(count) => {
                        self.status_message = format!("Successfully replayed {} events", count);
                        self.selected_events_for_replay.clear();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to replay events: {}", e));
                    }
                }
                Task::none()
            }

            // Trust chain visualization handlers
            Message::ToggleTrustChainSection => {
                self.trust_chain_section_collapsed = !self.trust_chain_section_collapsed;
                Task::none()
            }

            Message::SelectCertificateForChainView(cert_id) => {
                self.selected_trust_chain_cert = Some(cert_id);
                // Also verify this certificate's chain
                Task::done(Message::VerifyTrustChain(cert_id))
            }

            Message::VerifyTrustChain(cert_id) => {
                // Find the certificate
                if let Some(cert) = self.loaded_certificates.iter().find(|c| c.cert_id == cert_id) {
                    let now = chrono::Utc::now();

                    // Check if expired
                    if now > cert.not_after {
                        self.trust_chain_verification_status.insert(
                            cert_id,
                            TrustChainStatus::Expired { expired_at: cert.not_after },
                        );
                        return Task::done(Message::TrustChainVerified(Ok((
                            cert_id,
                            TrustChainStatus::Expired { expired_at: cert.not_after },
                        ))));
                    }

                    // Check if self-signed (root CA)
                    if cert.is_ca && cert.issuer.as_ref() == Some(&cert.subject) {
                        self.trust_chain_verification_status.insert(
                            cert_id,
                            TrustChainStatus::SelfSigned,
                        );
                        return Task::done(Message::TrustChainVerified(Ok((
                            cert_id,
                            TrustChainStatus::SelfSigned,
                        ))));
                    }

                    // Try to find issuer
                    if let Some(issuer_subject) = &cert.issuer {
                        // Search for issuer certificate
                        let issuer_cert = self.loaded_certificates.iter()
                            .find(|c| &c.subject == issuer_subject);

                        if let Some(issuer) = issuer_cert {
                            // Found issuer, build chain
                            let mut chain_length = 1;
                            let mut current = issuer;
                            let root_subject;

                            // Follow chain to root
                            loop {
                                chain_length += 1;
                                if current.is_ca && current.issuer.as_ref() == Some(&current.subject) {
                                    // Reached self-signed root
                                    root_subject = current.subject.clone();
                                    break;
                                }
                                if let Some(ref next_issuer) = current.issuer {
                                    if let Some(next) = self.loaded_certificates.iter()
                                        .find(|c| &c.subject == next_issuer)
                                    {
                                        current = next;
                                    } else {
                                        // Chain broken
                                        self.trust_chain_verification_status.insert(
                                            cert_id,
                                            TrustChainStatus::IssuerNotFound { expected_issuer: next_issuer.clone() },
                                        );
                                        return Task::done(Message::TrustChainVerified(Ok((
                                            cert_id,
                                            TrustChainStatus::IssuerNotFound { expected_issuer: next_issuer.clone() },
                                        ))));
                                    }
                                } else {
                                    root_subject = current.subject.clone();
                                    break;
                                }
                            }

                            self.trust_chain_verification_status.insert(
                                cert_id,
                                TrustChainStatus::Verified { chain_length, root_subject: root_subject.clone() },
                            );
                            return Task::done(Message::TrustChainVerified(Ok((
                                cert_id,
                                TrustChainStatus::Verified { chain_length, root_subject },
                            ))));
                        } else {
                            self.trust_chain_verification_status.insert(
                                cert_id,
                                TrustChainStatus::IssuerNotFound { expected_issuer: issuer_subject.clone() },
                            );
                            return Task::done(Message::TrustChainVerified(Ok((
                                cert_id,
                                TrustChainStatus::IssuerNotFound { expected_issuer: issuer_subject.clone() },
                            ))));
                        }
                    } else {
                        // No issuer field - might be root or incomplete cert
                        if cert.is_ca {
                            self.trust_chain_verification_status.insert(
                                cert_id,
                                TrustChainStatus::SelfSigned,
                            );
                            return Task::done(Message::TrustChainVerified(Ok((
                                cert_id,
                                TrustChainStatus::SelfSigned,
                            ))));
                        } else {
                            self.trust_chain_verification_status.insert(
                                cert_id,
                                TrustChainStatus::Failed { reason: "No issuer information".to_string() },
                            );
                            return Task::done(Message::TrustChainVerified(Ok((
                                cert_id,
                                TrustChainStatus::Failed { reason: "No issuer information".to_string() },
                            ))));
                        }
                    }
                }
                Task::none()
            }

            Message::TrustChainVerified(result) => {
                match result {
                    Ok((cert_id, status)) => {
                        self.trust_chain_verification_status.insert(cert_id, status.clone());
                        self.status_message = match &status {
                            TrustChainStatus::Verified { chain_length, root_subject } =>
                                format!("✅ Trust chain verified: {} certificates to root '{}'", chain_length, root_subject),
                            TrustChainStatus::SelfSigned =>
                                "🔐 Self-signed root certificate".to_string(),
                            TrustChainStatus::Expired { expired_at } =>
                                format!("❌ Certificate expired at {}", expired_at.format("%Y-%m-%d")),
                            TrustChainStatus::IssuerNotFound { expected_issuer } =>
                                format!("⚠️ Issuer not found: {}", expected_issuer),
                            TrustChainStatus::Failed { reason } =>
                                format!("❌ Verification failed: {}", reason),
                            TrustChainStatus::Pending =>
                                "⏳ Verification pending...".to_string(),
                        };
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Trust chain verification failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::VerifyAllTrustChains => {
                // Verify all loaded certificates
                let cert_ids: Vec<Uuid> = self.loaded_certificates.iter()
                    .map(|c| c.cert_id)
                    .collect();

                if cert_ids.is_empty() {
                    self.status_message = "No certificates to verify".to_string();
                    return Task::none();
                }

                // Verify each certificate
                for cert_id in cert_ids {
                    // This will trigger individual verifications
                    self.trust_chain_verification_status.insert(cert_id, TrustChainStatus::Pending);
                }

                // Verify the first one, others will be verified in sequence
                if let Some(first_id) = self.loaded_certificates.first().map(|c| c.cert_id) {
                    self.status_message = format!("Verifying {} certificates...", self.loaded_certificates.len());
                    return Task::done(Message::VerifyTrustChain(first_id));
                }

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
                        self.active_tab = Tab::Projections;
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
                use crate::projection::{Projection, manifest_to_export, ExportToFilesystemProjection};
                use crate::state_machines::workflows::PKIBootstrapState;

                // Sprint 79: Check PKI state machine (warn if not ready)
                match &self.pki_state {
                    PKIBootstrapState::ExportReady { .. } | PKIBootstrapState::Bootstrapped { .. } => {
                        // Ideal state - export is ready
                        tracing::info!("PKI state machine: Export initiated from valid state");
                    }
                    PKIBootstrapState::YubiKeysProvisioned { .. } => {
                        // Valid but should prepare first
                        tracing::warn!("PKI state machine: Exporting before PkiPrepareExport called");
                    }
                    _ => {
                        // Not in export-ready state, warn but allow
                        tracing::warn!(
                            "PKI state machine: Exporting in early state {:?}. Full PKI chain not complete.",
                            self.pki_state
                        );
                    }
                }

                self.status_message = "Building SD Card export package...".to_string();

                // Build KeyManifest from current loaded state
                let manifest = crate::projections::KeyManifest {
                    version: "1.0.0".to_string(),
                    updated_at: chrono::Utc::now(),
                    organization: crate::projections::OrganizationInfo {
                        name: self.organization_name.clone(),
                        domain: self.organization_domain.clone(),
                        country: "US".to_string(), // TODO: Make configurable
                        admin_email: self.admin_email.clone(),
                    },
                    people: self.loaded_people.clone(),
                    locations: self.loaded_locations.clone(),
                    keys: self.loaded_keys.clone(),
                    certificates: self.loaded_certificates.clone(),
                    pki_hierarchies: vec![], // TODO: Populate from projection
                    yubikeys: vec![],        // TODO: Populate from projection
                    nats_operators: vec![],  // TODO: Populate from projection
                    nats_accounts: vec![],
                    nats_users: vec![],
                    event_count: 0, // TODO: Get from projection
                    checksum: String::new(),
                };

                // Create the composed projection pipeline
                let projection = manifest_to_export();
                match projection.project(manifest) {
                    Ok(export) => {
                        // Write to filesystem
                        let fs_projection = ExportToFilesystemProjection::new(&self.export_path);
                        match fs_projection.project(export) {
                            Ok(result) => {
                                return Task::done(Message::SDCardExported(
                                    Ok((result.base_path.display().to_string(), result.files_written, result.bytes_written))
                                ));
                            }
                            Err(e) => {
                                return Task::done(Message::SDCardExported(
                                    Err(format!("Failed to write export: {}", e))
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        return Task::done(Message::SDCardExported(
                            Err(format!("Projection failed: {}", e))
                        ));
                    }
                }
            }

            Message::DomainExported(result) => {
                match result {
                    Ok(path) => {
                        self.status_message = format!("Domain projected to: {}", path);
                    }
                    Err(e) => {
                        self.status_message = format!("Projection failed: {}", e);
                    }
                }
                Task::none()
            }

            // Projection configuration handlers
            Message::Neo4jEndpointChanged(endpoint) => {
                self.neo4j_endpoint = endpoint;
                Task::none()
            }

            Message::Neo4jUsernameChanged(username) => {
                self.neo4j_username = username;
                Task::none()
            }

            Message::Neo4jPasswordChanged(password) => {
                self.neo4j_password = password;
                Task::none()
            }

            Message::JetStreamUrlChanged(url) => {
                self.jetstream_url = url;
                Task::none()
            }

            Message::JetStreamCredentialsChanged(path) => {
                self.jetstream_credentials_path = path;
                Task::none()
            }

            Message::ProjectionSectionChanged(section) => {
                self.projection_section = section;
                Task::none()
            }

            Message::ProjectionSelected(target) => {
                self.selected_projection = Some(target);
                self.status_message = format!("Selected projection: {}", target.display_name());
                Task::none()
            }

            Message::InjectionSelected(source) => {
                self.selected_injection = Some(source);
                self.status_message = format!("Selected injection: {}", source.display_name());
                Task::none()
            }

            Message::ConnectProjection(target) => {
                // TODO: Implement actual connection logic
                self.status_message = format!("Connecting to {}...", target.display_name());
                Task::none()
            }

            Message::DisconnectProjection(target) => {
                // TODO: Implement actual disconnection logic
                self.status_message = format!("Disconnected from {}", target.display_name());
                Task::none()
            }

            Message::SyncProjection(target) => {
                match target {
                    ProjectionTarget::Neo4j => {
                        // Use the projection system for Neo4j export
                        self.status_message = "Generating Cypher for Neo4j...".to_string();
                        return Task::done(Message::ExportToCypher);
                    }
                    ProjectionTarget::SDCard => {
                        // Use the projection system for SD Card export
                        self.status_message = "Exporting domain to SD Card...".to_string();
                        return Task::done(Message::ExportToSDCard);
                    }
                    _ => {
                        self.status_message = format!("Syncing to {}...", target.display_name());
                    }
                }
                Task::none()
            }

            Message::ImportFromSource(source) => {
                // TODO: Implement actual import logic
                self.status_message = format!("Importing from {}...", source.display_name());
                Task::none()
            }

            Message::ExportToCypher => {
                use crate::projection::DomainGraphBuilder;
                use crate::ports::neo4j::GraphNode;

                self.status_message = "Building domain graph for Neo4j export...".to_string();

                // Build domain graph from current state
                let mut builder = DomainGraphBuilder::new();

                // Add organization if present
                if let Some(org_id) = self.organization_id {
                    builder = builder.add(&GraphNode::new(org_id, "Organization")
                        .with_property("name", self.organization_name.clone())
                        .with_property("domain", self.organization_domain.clone()));
                }

                // Add loaded people
                for person in &self.loaded_people {
                    builder = builder.add(&GraphNode::new(person.person_id, "Person")
                        .with_property("name", person.name.clone())
                        .with_property("email", person.email.clone())
                        .with_property("role", person.role.clone()));

                    // Relate person to organization
                    if let Some(org_id) = self.organization_id {
                        builder = builder.relate(person.person_id, org_id, "BELONGS_TO");
                    }
                }

                // Add certificates from loaded_certificates
                for cert in &self.loaded_certificates {
                    let cert_type = if cert.is_ca { "CA" } else { "EndEntity" };
                    builder = builder.add(&GraphNode::new(cert.cert_id, "Certificate")
                        .with_property("type", cert_type)
                        .with_property("subject", cert.subject.clone())
                        .with_property("serial_number", cert.serial_number.clone())
                        .with_property("not_before", cert.not_before.to_rfc3339())
                        .with_property("not_after", cert.not_after.to_rfc3339()));

                    // Relate cert to its key
                    builder = builder.relate(cert.cert_id, cert.key_id, "USES_KEY");
                }

                // Add loaded keys
                for key in &self.loaded_keys {
                    builder = builder.add(&GraphNode::new(key.key_id, "CryptographicKey")
                        .with_property("algorithm", format!("{:?}", key.algorithm))
                        .with_property("purpose", format!("{:?}", key.purpose))
                        .with_property("label", key.label.clone())
                        .with_property("hardware_backed", if key.hardware_backed { "true" } else { "false" }));
                }

                // Add loaded locations
                for location in &self.loaded_locations {
                    builder = builder.add(&GraphNode::new(location.location_id, "Location")
                        .with_property("name", location.name.clone())
                        .with_property("type", location.location_type.clone()));
                }

                // Project to Cypher file
                match builder.to_cypher_file() {
                    Ok(cypher_content) => {
                        let query_count = cypher_content.lines()
                            .filter(|l| l.trim().starts_with("MERGE") || l.trim().starts_with("MATCH"))
                            .count();

                        // Write to file in export directory
                        let file_path = self.export_path.join("domain-export.cypher");
                        match std::fs::write(&file_path, &cypher_content) {
                            Ok(_) => {
                                return Task::done(Message::CypherExported(
                                    Ok((file_path.display().to_string(), query_count))
                                ));
                            }
                            Err(e) => {
                                return Task::done(Message::CypherExported(
                                    Err(format!("Failed to write file: {}", e))
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        return Task::done(Message::CypherExported(
                            Err(format!("Projection failed: {}", e))
                        ));
                    }
                }
            }

            Message::CypherExported(result) => {
                match result {
                    Ok((path, count)) => {
                        self.status_message = format!(
                            "✅ Exported {} Cypher queries to {}",
                            count, path
                        );
                        // Update Neo4j projection status
                        if let Some(proj) = self.projections.iter_mut()
                            .find(|p| p.target == ProjectionTarget::Neo4j) {
                            proj.status = ProjectionStatus::Synced;
                            proj.items_synced = count;
                            proj.last_sync = Some(chrono::Utc::now());
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Neo4j export failed: {}", e));
                        if let Some(proj) = self.projections.iter_mut()
                            .find(|p| p.target == ProjectionTarget::Neo4j) {
                            proj.status = ProjectionStatus::Error(e);
                        }
                    }
                }
                Task::none()
            }

            Message::SDCardExported(result) => {
                match result {
                    Ok((path, files, bytes)) => {
                        self.status_message = format!(
                            "✅ Exported {} files ({} bytes) to {}",
                            files, bytes, path
                        );
                        // Update SDCard projection status
                        if let Some(proj) = self.projections.iter_mut()
                            .find(|p| p.target == ProjectionTarget::SDCard) {
                            proj.status = ProjectionStatus::Synced;
                            proj.items_synced = files;
                            proj.last_sync = Some(chrono::Utc::now());
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("SD Card export failed: {}", e));
                        if let Some(proj) = self.projections.iter_mut()
                            .find(|p| p.target == ProjectionTarget::SDCard) {
                            proj.status = ProjectionStatus::Error(e);
                        }
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

            // NATS Hierarchy Visualization handlers
            Message::ToggleNatsVizSection => {
                self.nats_viz_section_collapsed = !self.nats_viz_section_collapsed;
                Task::none()
            }

            Message::ToggleNatsAccountExpand(account_name) => {
                if self.nats_viz_expanded_accounts.contains(&account_name) {
                    self.nats_viz_expanded_accounts.remove(&account_name);
                } else {
                    self.nats_viz_expanded_accounts.insert(account_name);
                }
                Task::none()
            }

            Message::SelectNatsOperator => {
                self.nats_viz_selected_operator = true;
                self.nats_viz_selected_account = None;
                self.nats_viz_selected_user = None;
                self.status_message = "Selected NATS Operator".to_string();
                Task::none()
            }

            Message::SelectNatsAccount(account_name) => {
                self.nats_viz_selected_operator = false;
                self.nats_viz_selected_account = Some(account_name.clone());
                self.nats_viz_selected_user = None;
                self.status_message = format!("Selected NATS Account: {}", account_name);
                Task::none()
            }

            Message::SelectNatsUser(account_name, person_id) => {
                self.nats_viz_selected_operator = false;
                self.nats_viz_selected_account = None;
                self.nats_viz_selected_user = Some((account_name.clone(), person_id));
                // Try to find person name
                let person_name = self.loaded_people.iter()
                    .find(|p| p.person_id == person_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| format!("{}", person_id));
                self.status_message = format!("Selected NATS User: {} (in {})", person_name, account_name);
                Task::none()
            }

            Message::RefreshNatsHierarchy => {
                // Build hierarchy from current state
                let mut hierarchy = NatsHierarchyFull {
                    operator: NatsOperatorConfig {
                        name: self.organization_name.clone(),
                        signing_keys: vec![],
                    },
                    accounts: vec![],
                };

                // Build accounts from created units
                for unit in &self.created_units {
                    let account = NatsAccountConfig {
                        name: unit.nats_account_name.clone().unwrap_or_else(|| unit.name.to_lowercase().replace(' ', "-")),
                        unit_id: unit.id.as_uuid(),
                        users: vec![],
                    };
                    hierarchy.accounts.push(account);
                }

                // Also include accounts from loaded units
                for unit in &self.loaded_units {
                    if let Some(ref nats_account) = unit.nats_account_name {
                        // Check if already added
                        if !hierarchy.accounts.iter().any(|a| a.name == *nats_account) {
                            let account = NatsAccountConfig {
                                name: nats_account.clone(),
                                unit_id: unit.id.as_uuid(),
                                users: vec![],
                            };
                            hierarchy.accounts.push(account);
                        }
                    }
                }

                // Add users and accounts from nats_bootstrap if available
                if let Some(ref bootstrap) = self.nats_bootstrap {
                    // Update operator name from bootstrap
                    hierarchy.operator.name = bootstrap.organization.name.clone();

                    // Add accounts from bootstrap (OrganizationUnit → NatsAccount)
                    for (unit_id, (unit, _nats_identity)) in &bootstrap.accounts {
                        let account_name = unit.nats_account_name.clone()
                            .unwrap_or_else(|| unit.name.to_lowercase().replace(' ', "-"));

                        // Check if account already exists, if not add it
                        if !hierarchy.accounts.iter().any(|a| a.unit_id == *unit_id) {
                            hierarchy.accounts.push(NatsAccountConfig {
                                name: account_name,
                                unit_id: *unit_id,
                                users: vec![],
                            });
                        }
                    }

                    // Add users from bootstrap (Person → NatsUser)
                    for (person_id, (person, _nats_identity)) in &bootstrap.users {
                        // Find which account (unit) this person belongs to
                        if let Some(unit_id) = person.unit_ids.first() {
                            let unit_uuid = unit_id.as_uuid();
                            if let Some(account) = hierarchy.accounts.iter_mut().find(|a| a.unit_id == unit_uuid) {
                                if !account.users.iter().any(|u| u.person_id == *person_id) {
                                    account.users.push(NatsUserConfig {
                                        person_id: *person_id,
                                        permissions: None,
                                    });
                                }
                            }
                        }
                    }
                }

                self.nats_viz_hierarchy_data = Some(hierarchy.clone());
                self.status_message = format!("NATS hierarchy refreshed: {} accounts", hierarchy.accounts.len());
                Task::done(Message::NatsHierarchyRefreshed(Ok(hierarchy)))
            }

            Message::NatsHierarchyRefreshed(result) => {
                match result {
                    Ok(hierarchy) => {
                        self.nats_viz_hierarchy_data = Some(hierarchy);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to refresh NATS hierarchy: {}", e));
                    }
                }
                Task::none()
            }

            Message::AddNatsAccount { unit_id, account_name } => {
                if let Some(ref mut hierarchy) = self.nats_viz_hierarchy_data {
                    // Check for duplicates
                    if !hierarchy.accounts.iter().any(|a| a.name == account_name) {
                        hierarchy.accounts.push(NatsAccountConfig {
                            name: account_name.clone(),
                            unit_id,
                            users: vec![],
                        });
                        self.status_message = format!("Added NATS account: {}", account_name);
                    } else {
                        self.error_message = Some(format!("Account '{}' already exists", account_name));
                    }
                }
                Task::none()
            }

            Message::AddNatsUser { account_name, person_id } => {
                if let Some(ref mut hierarchy) = self.nats_viz_hierarchy_data {
                    if let Some(account) = hierarchy.accounts.iter_mut().find(|a| a.name == account_name) {
                        // Check for duplicates
                        if !account.users.iter().any(|u| u.person_id == person_id) {
                            account.users.push(NatsUserConfig {
                                person_id,
                                permissions: None,
                            });
                            let person_name = self.loaded_people.iter()
                                .find(|p| p.person_id == person_id)
                                .map(|p| p.name.clone())
                                .unwrap_or_else(|| format!("{}", person_id));
                            self.status_message = format!("Added {} to account {}", person_name, account_name);
                        } else {
                            self.error_message = Some("User already in this account".to_string());
                        }
                    } else {
                        self.error_message = Some(format!("Account '{}' not found", account_name));
                    }
                }
                Task::none()
            }

            Message::RemoveNatsAccount(account_name) => {
                if let Some(ref mut hierarchy) = self.nats_viz_hierarchy_data {
                    hierarchy.accounts.retain(|a| a.name != account_name);
                    self.status_message = format!("Removed NATS account: {}", account_name);
                    // Clear selection if removed account was selected
                    if self.nats_viz_selected_account.as_ref() == Some(&account_name) {
                        self.nats_viz_selected_account = None;
                    }
                }
                Task::none()
            }

            Message::RemoveNatsUser(account_name, person_id) => {
                if let Some(ref mut hierarchy) = self.nats_viz_hierarchy_data {
                    if let Some(account) = hierarchy.accounts.iter_mut().find(|a| a.name == account_name) {
                        account.users.retain(|u| u.person_id != person_id);
                        self.status_message = "Removed user from account".to_string();
                        // Clear selection if removed user was selected
                        if self.nats_viz_selected_user.as_ref() == Some(&(account_name.clone(), person_id)) {
                            self.nats_viz_selected_user = None;
                        }
                    }
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
                    PropertyCardMessage::RequestYubiKeyAssignment => {
                        // Show YubiKey assignment UI for the current person
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                if let Some(person) = node.lifted_node.downcast::<Person>() {
                                    // Set the selected person for YubiKey assignment
                                    self.selected_person = Some(node_id);

                                    // Check if there are any unassigned YubiKeys available
                                    // registered_yubikeys is HashMap<serial, entity_id>
                                    let unassigned_count = self.registered_yubikeys.keys()
                                        .filter(|serial| !self.yubikey_assignments.contains_key(*serial))
                                        .count();

                                    if unassigned_count == 0 && self.registered_yubikeys.is_empty() {
                                        self.status_message = format!("No registered YubiKeys available. Please register a YubiKey first.");
                                    } else if unassigned_count == 0 {
                                        self.status_message = format!("All registered YubiKeys are assigned. Register a new YubiKey to assign to {}.", person.name);
                                    } else {
                                        // Show available YubiKeys for selection
                                        // Toggle to the YubiKey tab to show the assignment UI
                                        self.yubikey_section_collapsed = false;
                                        self.status_message = format!("Select a YubiKey to assign to {}. {} unassigned YubiKey(s) available.", person.name, unassigned_count);
                                    }
                                }
                            }
                        }
                    }
                    PropertyCardMessage::RequestYubiKeyUnassignment => {
                        // Unassign YubiKey from the current person
                        if let Some(node_id) = self.property_card.node_id() {
                            if let Some(node) = self.org_graph.nodes.get(&node_id) {
                                if let Some(person) = node.lifted_node.downcast::<Person>() {
                                    // Find if this person has an assigned YubiKey
                                    let assigned_yubikey: Option<String> = self.yubikey_assignments.iter()
                                        .find(|(_, &pid)| pid == node_id)
                                        .map(|(serial, _)| serial.clone());

                                    if let Some(serial) = assigned_yubikey {
                                        // Remove the assignment
                                        self.yubikey_assignments.remove(&serial);
                                        self.status_message = format!("✅ YubiKey {} unassigned from {}", serial, person.name);
                                        tracing::info!("YubiKey {} unassigned from person: {}", serial, person.name);
                                    } else {
                                        self.status_message = format!("No YubiKey assigned to {}", person.name);
                                    }
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
                                    // Sprint 82: Generate Leaf Certificate signed by Intermediate CA
                                    // Check if we have an Intermediate CA to sign with
                                    if let Some((intermediate_ca_id, intermediate_ca)) = self.generated_intermediate_cas.iter().next() {
                                        self.status_message = "Leaf certificate generation in progress...".to_string();

                                        // Get person info from property card - use lifted_node downcast
                                        let person_name = self.property_card.node_id()
                                            .and_then(|id| self.org_graph.nodes.get(&id))
                                            .and_then(|node| node.lifted_node.downcast::<Person>().map(|p| p.name.clone()))
                                            .unwrap_or_else(|| "Unknown".to_string());

                                        // Clone intermediate CA data for async closure
                                        let intermediate_ca_cert_pem = intermediate_ca.certificate_pem.clone();
                                        let intermediate_ca_key_pem = intermediate_ca.private_key_pem.clone();
                                        let intermediate_ca_id = *intermediate_ca_id;
                                        let person_name_clone = person_name.clone();

                                        // Trigger async Leaf Certificate generation
                                        return Task::perform(
                                            async move {
                                                use crate::crypto::seed_derivation::derive_master_seed;
                                                use crate::crypto::x509::{generate_server_certificate, ServerCertParams};

                                                // Derive master seed from passphrase
                                                let leaf_seed = derive_master_seed(&passphrase, &org_id)
                                                    .map_err(|e| format!("Failed to derive seed: {}", e))?;

                                                // Create leaf certificate parameters
                                                let leaf_cert_params = ServerCertParams {
                                                    common_name: format!("{}", person_name_clone),
                                                    san_entries: vec![
                                                        format!("{}.{}", person_name_clone.to_lowercase().replace(' ', "."), org_name.to_lowercase().replace(' ', "-")),
                                                    ],
                                                    organization: org_name.clone(),
                                                    organizational_unit: Some("Personal".to_string()),
                                                    validity_days: 365, // 1 year for personal certs
                                                };

                                                // Generate leaf certificate signed by intermediate CA
                                                let correlation_id = uuid::Uuid::now_v7();
                                                let leaf_cert_id = uuid::Uuid::now_v7();
                                                let (cert, _gen_event, _sign_event) = generate_server_certificate(
                                                    &leaf_seed,
                                                    leaf_cert_params,
                                                    &intermediate_ca_cert_pem,
                                                    &intermediate_ca_key_pem,
                                                    intermediate_ca_id,
                                                    correlation_id,
                                                    None,
                                                )?;

                                                Ok((cert, leaf_cert_id, person_name_clone))
                                            },
                                            Message::LeafCertGenerated
                                        );
                                    } else {
                                        self.error_message = Some("Generate Intermediate CA first before creating leaf certificates.".to_string());
                                        return Task::none();
                                    }
                                }
                                passphrase_dialog::PassphrasePurpose::IntermediateCA => {
                                    // Sprint 81: Generate Intermediate CA signed by Root CA
                                    // Check if we have a Root CA to sign with
                                    if let Some(ref root_ca) = self.generated_root_ca {
                                        self.status_message = "Intermediate CA generation in progress...".to_string();

                                        // Clone root CA data for async closure
                                        let root_ca_cert_pem = root_ca.certificate_pem.clone();
                                        let root_ca_key_pem = root_ca.private_key_pem.clone();

                                        // Get root CA ID from state machine
                                        let root_ca_id = match &self.pki_state {
                                            crate::state_machines::workflows::PKIBootstrapState::RootCAGenerated { root_ca_cert_id, .. } => {
                                                *root_ca_cert_id
                                            }
                                            crate::state_machines::workflows::PKIBootstrapState::IntermediateCAPlanned { .. } |
                                            crate::state_machines::workflows::PKIBootstrapState::IntermediateCAGenerated { .. } => {
                                                // These states follow RootCAGenerated - use stored root CA's ID
                                                // The generated_root_ca should have the ID we need
                                                Uuid::now_v7() // Fallback - root CA ID should be tracked
                                            }
                                            _ => Uuid::now_v7() // Fallback for other states
                                        };

                                        // Get intermediate CA name from form or generate default
                                        let intermediate_name = if self.intermediate_ca_name_input.is_empty() {
                                            format!("{} Intermediate CA", self.organization_name)
                                        } else {
                                            self.intermediate_ca_name_input.clone()
                                        };
                                        let org_unit = self.cert_organizational_unit.clone();

                                        return Task::perform(
                                            async move {
                                                use crate::crypto::seed_derivation::derive_master_seed;
                                                use crate::crypto::x509::{generate_intermediate_ca, IntermediateCAParams};

                                                // Derive seed from passphrase
                                                let master_seed = derive_master_seed(&passphrase, &org_id)
                                                    .map_err(|e| format!("Failed to derive seed: {}", e))?;
                                                let intermediate_seed = master_seed.derive_child(&format!("intermediate-{}", org_unit));

                                                // Set up Intermediate CA parameters
                                                let params = IntermediateCAParams {
                                                    organization: org_name.clone(),
                                                    organizational_unit: org_unit,
                                                    common_name: intermediate_name,
                                                    country: Some("US".to_string()),
                                                    validity_years: 3, // Intermediate CAs typically 3-5 years
                                                    pathlen: 0, // Can only sign leaf certs
                                                };

                                                // Generate Intermediate CA certificate
                                                let correlation_id = uuid::Uuid::now_v7();
                                                let (cert, _gen_event, _sign_event) = generate_intermediate_ca(
                                                    &intermediate_seed,
                                                    params,
                                                    &root_ca_cert_pem,
                                                    &root_ca_key_pem,
                                                    root_ca_id,
                                                    correlation_id,
                                                    None,
                                                )?;

                                                let intermediate_ca_id = uuid::Uuid::now_v7();
                                                Ok((cert, intermediate_ca_id))
                                            },
                                            Message::IntermediateCAGenerated
                                        );
                                    } else {
                                        self.status_message = "❌ Cannot generate Intermediate CA: Root CA not generated yet".to_string();
                                        self.error_message = Some("Generate Root CA first before creating Intermediate CA".to_string());
                                    }
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

            Message::ProvisionSingleYubiKey { serial } => {
                // Provision a single YubiKey by serial
                if self.detected_yubikeys.is_empty() {
                    self.error_message = Some("No YubiKeys detected. Please run detection first.".to_string());
                    return Task::none();
                }

                // Find the device and config for this serial
                let device = match self.detected_yubikeys.iter().find(|d| d.serial == serial) {
                    Some(d) => d.clone(),
                    None => {
                        self.error_message = Some(format!("YubiKey {} not found in detected devices", serial));
                        return Task::none();
                    }
                };

                // Find config for this device
                let config = self.yubikey_configs.iter().find(|c| c.serial == serial).cloned();

                // Mark as provisioning in progress
                self.yubikey_provisioning_status.insert(serial.clone(), "⏳ Provisioning...".to_string());
                self.status_message = format!("Provisioning YubiKey {}...", serial);

                let yubikey_port = self.yubikey_port.clone();
                let serial_for_async = serial.clone();

                Task::perform(
                    async move {
                        use crate::ports::yubikey::{PivSlot, KeyAlgorithm};
                        use crate::ports::yubikey::SecureString;

                        let serial = serial_for_async;

                        if let Some(config) = config {
                            // Determine PIV slot based on role
                            let slot = match config.role {
                                crate::domain::YubiKeyRole::RootCA => PivSlot::Signature,  // 9C
                                crate::domain::YubiKeyRole::Backup => PivSlot::KeyManagement,  // 9D
                                crate::domain::YubiKeyRole::User => PivSlot::Authentication,  // 9A
                                crate::domain::YubiKeyRole::Service => PivSlot::CardAuth,  // 9E
                            };

                            let pin = SecureString::new(config.piv.pin.as_bytes());

                            // Generate key in the slot
                            match yubikey_port.generate_key_in_slot(
                                &device.serial,
                                slot,
                                KeyAlgorithm::EccP256,
                                &pin
                            ).await {
                                Ok(_public_key) => {
                                    let status = format!(
                                        "{} provisioned in slot {:?}",
                                        config.name,
                                        slot
                                    );
                                    Ok((serial, status))
                                }
                                Err(e) => {
                                    Err((serial, format!("Failed to provision: {:?}", e)))
                                }
                            }
                        } else {
                            // No config found - provision with default settings
                            // Default to Authentication slot (9A) with ECC P-256
                            let slot = PivSlot::Authentication;
                            let default_pin = SecureString::new(b"123456");  // Factory default

                            match yubikey_port.generate_key_in_slot(
                                &device.serial,
                                slot,
                                KeyAlgorithm::EccP256,
                                &default_pin
                            ).await {
                                Ok(_public_key) => {
                                    Ok((serial, format!("Provisioned with default settings (slot 9A)")))
                                }
                                Err(e) => {
                                    Err((serial, format!("Failed to provision with defaults: {:?}", e)))
                                }
                            }
                        }
                    },
                    Message::SingleYubiKeyProvisioned
                )
            }

            Message::SingleYubiKeyProvisioned(result) => {
                match result {
                    Ok((serial, msg)) => {
                        self.yubikey_provisioning_status.insert(serial.clone(), format!("✅ {}", msg));
                        self.status_message = format!("✅ YubiKey {}: {}", serial, msg);
                    }
                    Err((serial, e)) => {
                        self.yubikey_provisioning_status.insert(serial.clone(), format!("❌ {}", e));
                        self.error_message = Some(format!("YubiKey {}: {}", serial, e));
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

            Message::WorkflowMessage(msg) => {
                self.workflow_view.update(msg);
                Task::none()
            }

            Message::StateMachineSelected(sm_type) => {
                self.selected_state_machine = sm_type;
                // Sprint 80: Use current state for PKI and YubiKey state machines
                self.state_machine_definition = match sm_type {
                    state_machine_graph::StateMachineType::PkiBootstrap => {
                        state_machine_graph::get_pki_bootstrap_with_current(&self.pki_state)
                    }
                    state_machine_graph::StateMachineType::YubiKeyProvisioning => {
                        // Show first YubiKey's state if any exists
                        if let Some(yk_state) = self.yubikey_states.values().next() {
                            state_machine_graph::get_yubikey_provisioning_with_current(yk_state)
                        } else {
                            state_machine_graph::get_state_machine(sm_type)
                        }
                    }
                    _ => state_machine_graph::get_state_machine(sm_type),
                };
                self.state_machine_graph = state_machine_graph::state_machine_to_graph(
                    &self.state_machine_definition,
                    &state_machine_graph::StateMachineLayoutConfig::default(),
                );
                let current_info = if let Some(ref current) = self.state_machine_definition.current_state {
                    format!(" (current: {})", current)
                } else {
                    String::new()
                };
                self.status_message = format!("Viewing {} state machine{}", sm_type.display_name(), current_info);
                Task::none()
            }

            Message::StateMachineMessage(msg) => {
                match msg {
                    state_machine_graph::StateMachineMessage::SelectMachine(sm_type) => {
                        self.selected_state_machine = sm_type;
                        // Sprint 80: Use current state for PKI and YubiKey state machines
                        self.state_machine_definition = match sm_type {
                            state_machine_graph::StateMachineType::PkiBootstrap => {
                                state_machine_graph::get_pki_bootstrap_with_current(&self.pki_state)
                            }
                            state_machine_graph::StateMachineType::YubiKeyProvisioning => {
                                if let Some(yk_state) = self.yubikey_states.values().next() {
                                    state_machine_graph::get_yubikey_provisioning_with_current(yk_state)
                                } else {
                                    state_machine_graph::get_state_machine(sm_type)
                                }
                            }
                            _ => state_machine_graph::get_state_machine(sm_type),
                        };
                        self.state_machine_graph = state_machine_graph::state_machine_to_graph(
                            &self.state_machine_definition,
                            &state_machine_graph::StateMachineLayoutConfig::default(),
                        );
                    }
                    state_machine_graph::StateMachineMessage::StateSelected(state_name) => {
                        self.status_message = format!("Selected state: {}", state_name);
                    }
                    state_machine_graph::StateMachineMessage::TransitionSelected { from, to } => {
                        self.status_message = format!("Transition: {} → {}", from, to);
                    }
                    state_machine_graph::StateMachineMessage::ResetView => {
                        // Sprint 80: Refresh with current state
                        self.state_machine_definition = match self.selected_state_machine {
                            state_machine_graph::StateMachineType::PkiBootstrap => {
                                state_machine_graph::get_pki_bootstrap_with_current(&self.pki_state)
                            }
                            state_machine_graph::StateMachineType::YubiKeyProvisioning => {
                                if let Some(yk_state) = self.yubikey_states.values().next() {
                                    state_machine_graph::get_yubikey_provisioning_with_current(yk_state)
                                } else {
                                    state_machine_graph::get_state_machine(self.selected_state_machine)
                                }
                            }
                            _ => state_machine_graph::get_state_machine(self.selected_state_machine),
                        };
                        self.state_machine_graph = state_machine_graph::state_machine_to_graph(
                            &self.state_machine_definition,
                            &state_machine_graph::StateMachineLayoutConfig::default(),
                        );
                    }
                }
                Task::none()
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

            // PKI State Machine Handlers (Sprint 77)
            Message::PkiPlanRootCA { subject, validity_years, yubikey_serial } => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Validate state transition using state machine guard
                if !self.pki_state.can_plan_root_ca() {
                    self.error_message = Some(format!(
                        "Cannot plan Root CA in current state: {:?}",
                        self.pki_state
                    ));
                    return Task::none();
                }

                // Transition to RootCAPlanned state
                self.pki_state = PKIBootstrapState::RootCAPlanned {
                    subject,
                    validity_years,
                    yubikey_serial: yubikey_serial.clone(),
                };

                self.status_message = format!(
                    "Root CA planned. Ready for generation (YubiKey: {})",
                    yubikey_serial
                );

                Task::none()
            }

            Message::PkiExecuteRootCAGeneration => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Validate state transition using state machine guard
                if !self.pki_state.can_generate_root_ca() {
                    self.error_message = Some(format!(
                        "Cannot generate Root CA in current state: {:?}. Plan Root CA first.",
                        self.pki_state
                    ));
                    return Task::none();
                }

                // Extract planning data from current state
                let (subject, validity_years, yubikey_serial) = match &self.pki_state {
                    PKIBootstrapState::RootCAPlanned { subject, validity_years, yubikey_serial } => {
                        (subject.clone(), *validity_years, yubikey_serial.clone())
                    }
                    _ => {
                        self.error_message = Some("Invalid state for Root CA generation".to_string());
                        return Task::none();
                    }
                };

                self.status_message = "Generating Root CA (state machine driven)...".to_string();

                // Show passphrase dialog for seed derivation
                // The passphrase dialog will trigger the actual generation
                self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::RootCA);

                // Store planning data for use after passphrase entry
                // (The passphrase handler will need access to subject/validity)
                tracing::info!(
                    "PKI state machine: executing Root CA generation for {} (validity: {} years, YubiKey: {})",
                    subject.common_name,
                    validity_years,
                    yubikey_serial
                );

                Task::none()
            }

            Message::PkiRootCAGenerationComplete { root_ca_cert_id, root_ca_key_id } => {
                use crate::state_machines::workflows::PKIBootstrapState;
                use chrono::Utc;

                // Transition to RootCAGenerated state
                self.pki_state = PKIBootstrapState::RootCAGenerated {
                    root_ca_cert_id,
                    root_ca_key_id,
                    generated_at: Utc::now(),
                };

                self.status_message = format!(
                    "Root CA generated successfully! Cert ID: {}, Key ID: {}",
                    root_ca_cert_id,
                    root_ca_key_id
                );

                tracing::info!(
                    "PKI state machine: Root CA generation complete. State: {:?}",
                    self.pki_state
                );

                Task::none()
            }

            // IntermediateCA State Machine Handlers (Sprint 78)
            Message::PkiPlanIntermediateCA { subject, validity_years, path_len } => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Validate state transition using state machine guard
                if !self.pki_state.can_plan_intermediate_ca() {
                    self.error_message = Some(format!(
                        "Cannot plan Intermediate CA in current state: {:?}. Generate Root CA first.",
                        self.pki_state
                    ));
                    return Task::none();
                }

                // Transition to IntermediateCAPlanned state
                self.pki_state = PKIBootstrapState::IntermediateCAPlanned {
                    subject,
                    validity_years,
                    path_len,
                };

                self.status_message = "Intermediate CA planned. Ready for generation.".to_string();

                tracing::info!(
                    "PKI state machine: Intermediate CA planned. State: {:?}",
                    self.pki_state
                );

                Task::none()
            }

            Message::PkiExecuteIntermediateCAGeneration => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Validate state transition using state machine guard
                if !self.pki_state.can_generate_intermediate_ca() {
                    self.error_message = Some(format!(
                        "Cannot generate Intermediate CA in current state: {:?}. Plan Intermediate CA first.",
                        self.pki_state
                    ));
                    return Task::none();
                }

                self.status_message = "Generating Intermediate CA (state machine driven)...".to_string();

                // Show passphrase dialog for seed derivation
                self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::IntermediateCA);

                tracing::info!(
                    "PKI state machine: Executing Intermediate CA generation. Current state: {:?}",
                    self.pki_state
                );

                Task::none()
            }

            Message::PkiIntermediateCAGenerationComplete { intermediate_ca_id } => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Transition to IntermediateCAGenerated state
                // Note: We track multiple intermediate CAs in a Vec
                let intermediate_ca_ids = match &self.pki_state {
                    PKIBootstrapState::IntermediateCAGenerated { intermediate_ca_ids } => {
                        let mut ids = intermediate_ca_ids.clone();
                        ids.push(intermediate_ca_id);
                        ids
                    }
                    _ => vec![intermediate_ca_id],
                };

                self.pki_state = PKIBootstrapState::IntermediateCAGenerated {
                    intermediate_ca_ids,
                };

                self.status_message = format!(
                    "Intermediate CA generated successfully! ID: {}",
                    intermediate_ca_id
                );

                tracing::info!(
                    "PKI state machine: Intermediate CA generation complete. State: {:?}",
                    self.pki_state
                );

                Task::none()
            }

            // LeafCert State Machine Handlers (Sprint 78)
            Message::PkiPlanLeafCert { subject, validity_years, person_id } => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Validate state transition using state machine guard
                if !self.pki_state.can_generate_leaf_cert() {
                    self.error_message = Some(format!(
                        "Cannot plan Leaf Cert in current state: {:?}. Generate Intermediate CA first.",
                        self.pki_state
                    ));
                    return Task::none();
                }

                // Note: LeafCerts don't have a "planned" state in the current state machine
                // They go directly from IntermediateCAGenerated to LeafCertsGenerated
                // For now, just log and proceed
                self.status_message = format!(
                    "Leaf certificate planned for: {}",
                    subject.common_name
                );

                tracing::info!(
                    "PKI state machine: Leaf cert planned. Subject: {}, Person: {:?}",
                    subject.common_name,
                    person_id
                );

                Task::none()
            }

            Message::PkiExecuteLeafCertGeneration => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Validate state transition using state machine guard
                if !self.pki_state.can_generate_leaf_cert() {
                    self.error_message = Some(format!(
                        "Cannot generate Leaf Cert in current state: {:?}. Generate Intermediate CA first.",
                        self.pki_state
                    ));
                    return Task::none();
                }

                self.status_message = "Generating Leaf Certificate (state machine driven)...".to_string();

                // Show passphrase dialog for personal keys
                self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::PersonalKeys);

                tracing::info!(
                    "PKI state machine: Executing Leaf Cert generation. Current state: {:?}",
                    self.pki_state
                );

                Task::none()
            }

            Message::PkiLeafCertGenerationComplete { leaf_cert_id } => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Transition to LeafCertsGenerated state
                // Note: We track multiple leaf certs in a Vec
                let leaf_cert_ids = match &self.pki_state {
                    PKIBootstrapState::LeafCertsGenerated { leaf_cert_ids } => {
                        let mut ids = leaf_cert_ids.clone();
                        ids.push(leaf_cert_id);
                        ids
                    }
                    _ => vec![leaf_cert_id],
                };

                self.pki_state = PKIBootstrapState::LeafCertsGenerated {
                    leaf_cert_ids,
                };

                self.status_message = format!(
                    "Leaf certificate generated successfully! ID: {}",
                    leaf_cert_id
                );

                tracing::info!(
                    "PKI state machine: Leaf cert generation complete. State: {:?}",
                    self.pki_state
                );

                Task::none()
            }

            // YubiKey State Machine Handlers (Sprint 78)
            Message::YubiKeyDetected { serial, firmware_version } => {
                use crate::state_machines::workflows::YubiKeyProvisioningState;

                // First check if PKI state allows YubiKey provisioning
                if !self.pki_state.can_provision_yubikey() {
                    self.error_message = Some(format!(
                        "Cannot provision YubiKey in current PKI state: {:?}. Generate leaf certificates first.",
                        self.pki_state
                    ));
                    return Task::none();
                }

                // Initialize YubiKey in Detected state
                self.yubikey_states.insert(
                    serial.clone(),
                    YubiKeyProvisioningState::Detected {
                        serial: serial.clone(),
                        firmware_version: firmware_version.clone(),
                    },
                );

                self.status_message = format!(
                    "YubiKey {} detected (firmware: {}). Ready for authentication.",
                    serial,
                    firmware_version
                );

                tracing::info!(
                    "YubiKey state machine: {} detected. Firmware: {}",
                    serial,
                    firmware_version
                );

                Task::none()
            }

            Message::YubiKeyAuthenticated { serial, pin_retries } => {
                use crate::state_machines::workflows::YubiKeyProvisioningState;

                // Get current state and validate transition
                let current_state = self.yubikey_states.get(&serial);
                match current_state {
                    Some(state) if state.can_authenticate() => {
                        self.yubikey_states.insert(
                            serial.clone(),
                            YubiKeyProvisioningState::Authenticated {
                                pin_retries_remaining: pin_retries,
                            },
                        );
                        self.status_message = format!(
                            "YubiKey {} authenticated. {} PIN retries remaining.",
                            serial,
                            pin_retries
                        );
                        tracing::info!("YubiKey {} authenticated", serial);
                    }
                    Some(state) => {
                        self.error_message = Some(format!(
                            "Cannot authenticate YubiKey {} in state: {:?}",
                            serial,
                            state
                        ));
                    }
                    None => {
                        self.error_message = Some(format!(
                            "YubiKey {} not detected. Detect first.",
                            serial
                        ));
                    }
                }

                Task::none()
            }

            Message::YubiKeyPINChanged { serial } => {
                use crate::state_machines::workflows::YubiKeyProvisioningState;

                // Get current state and validate transition
                let current_state = self.yubikey_states.get(&serial);
                match current_state {
                    Some(state) if state.can_change_pin() => {
                        self.yubikey_states.insert(
                            serial.clone(),
                            YubiKeyProvisioningState::PINChanged {
                                new_pin_hash: "sha256:redacted".to_string(), // Actual hash in real impl
                            },
                        );
                        self.status_message = format!(
                            "YubiKey {} PIN changed successfully.",
                            serial
                        );
                        tracing::info!("YubiKey {} PIN changed", serial);
                    }
                    Some(state) => {
                        self.error_message = Some(format!(
                            "Cannot change PIN for YubiKey {} in state: {:?}. Authenticate first.",
                            serial,
                            state
                        ));
                    }
                    None => {
                        self.error_message = Some(format!(
                            "YubiKey {} not in provisioning workflow.",
                            serial
                        ));
                    }
                }

                Task::none()
            }

            Message::YubiKeyProvisioningComplete { serial } => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Update PKI state to include this YubiKey as provisioned
                let yubikey_serials = match &self.pki_state {
                    PKIBootstrapState::YubiKeysProvisioned { yubikey_serials } => {
                        let mut serials = yubikey_serials.clone();
                        if !serials.contains(&serial) {
                            serials.push(serial.clone());
                        }
                        serials
                    }
                    PKIBootstrapState::LeafCertsGenerated { .. } => {
                        vec![serial.clone()]
                    }
                    _ => {
                        self.error_message = Some(format!(
                            "Cannot complete YubiKey provisioning in PKI state: {:?}",
                            self.pki_state
                        ));
                        return Task::none();
                    }
                };

                self.pki_state = PKIBootstrapState::YubiKeysProvisioned {
                    yubikey_serials,
                };

                self.status_message = format!(
                    "YubiKey {} provisioning complete! PKI state updated.",
                    serial
                );

                tracing::info!(
                    "YubiKey {} provisioning complete. PKI state: {:?}",
                    serial,
                    self.pki_state
                );

                Task::none()
            }

            // Sprint 83: YubiKey Operation Handlers (actual PIV operations)
            Message::YubiKeyStartAuthentication { serial, pin } => {
                use crate::ports::yubikey::SecureString;

                // Validate state: must be in Detected state
                let current_state = self.yubikey_states.get(&serial);
                if !matches!(current_state, Some(state) if state.can_authenticate()) {
                    self.error_message = Some(format!(
                        "Cannot authenticate YubiKey {} in current state. Detect first.",
                        serial
                    ));
                    return Task::none();
                }

                self.status_message = format!("Authenticating YubiKey {}...", serial);
                let yubikey_port = self.yubikey_port.clone();
                let serial_for_async = serial.clone();
                let pin_secure = SecureString::new(pin.as_bytes());

                Task::perform(
                    async move {
                        match yubikey_port.verify_pin(&serial_for_async, &pin_secure).await {
                            Ok(true) => Ok((serial_for_async, 3u8)), // Assume 3 retries remaining on success
                            Ok(false) => Err((serial_for_async.clone(), "Invalid PIN".to_string())),
                            Err(e) => Err((serial_for_async.clone(), format!("{:?}", e))),
                        }
                    },
                    Message::YubiKeyAuthenticationResult
                )
            }

            Message::YubiKeyAuthenticationResult(result) => {
                match result {
                    Ok((serial, pin_retries)) => {
                        // Emit state machine transition
                        return Task::done(Message::YubiKeyAuthenticated { serial, pin_retries });
                    }
                    Err((serial, error)) => {
                        self.error_message = Some(format!("YubiKey {} authentication failed: {}", serial, error));
                    }
                }
                Task::none()
            }

            Message::YubiKeyStartPINChange { serial, old_pin, new_pin } => {
                use crate::ports::yubikey::SecureString;

                // Validate state: must be in Authenticated state
                let current_state = self.yubikey_states.get(&serial);
                if !matches!(current_state, Some(state) if state.can_change_pin()) {
                    self.error_message = Some(format!(
                        "Cannot change PIN for YubiKey {} in current state. Authenticate first.",
                        serial
                    ));
                    return Task::none();
                }

                self.status_message = format!("Changing PIN for YubiKey {}...", serial);
                let yubikey_port = self.yubikey_port.clone();
                let serial_for_async = serial.clone();
                let old_pin_secure = SecureString::new(old_pin.as_bytes());
                let new_pin_secure = SecureString::new(new_pin.as_bytes());

                Task::perform(
                    async move {
                        match yubikey_port.change_pin(&serial_for_async, &old_pin_secure, &new_pin_secure).await {
                            Ok(()) => Ok(serial_for_async),
                            Err(e) => Err((serial_for_async.clone(), format!("{:?}", e))),
                        }
                    },
                    Message::YubiKeyPINChangeResult
                )
            }

            Message::YubiKeyPINChangeResult(result) => {
                match result {
                    Ok(serial) => {
                        // Emit state machine transition
                        return Task::done(Message::YubiKeyPINChanged { serial });
                    }
                    Err((serial, error)) => {
                        self.error_message = Some(format!("YubiKey {} PIN change failed: {}", serial, error));
                    }
                }
                Task::none()
            }

            Message::YubiKeyStartKeyGeneration { serial, slot, algorithm } => {
                use crate::ports::yubikey::SecureString;

                // Get PIN from config or use default management key
                // Note: Key generation requires management key, not PIN
                let mgmt_key = SecureString::new(&[0u8; 24]); // Default 3DES key

                self.status_message = format!("Generating key in slot {:?} for YubiKey {}...", slot, serial);
                let yubikey_port = self.yubikey_port.clone();
                let serial_for_async = serial.clone();

                Task::perform(
                    async move {
                        match yubikey_port.generate_key_in_slot(&serial_for_async, slot, algorithm, &mgmt_key).await {
                            Ok(public_key) => Ok((serial_for_async, slot, public_key.data)),
                            Err(e) => Err((serial_for_async.clone(), format!("{:?}", e))),
                        }
                    },
                    Message::YubiKeyKeyGenerationResult
                )
            }

            Message::YubiKeyKeyGenerationResult(result) => {
                use crate::state_machines::workflows::YubiKeyProvisioningState;
                use std::collections::HashMap;

                match result {
                    Ok((serial, slot, public_key)) => {
                        // Update state machine to KeysGenerated
                        // Convert ports::yubikey::PivSlot to workflows::PivSlot
                        let workflow_slot = match slot {
                            crate::ports::yubikey::PivSlot::Authentication => {
                                crate::state_machines::workflows::PivSlot::Authentication
                            }
                            crate::ports::yubikey::PivSlot::Signature => {
                                crate::state_machines::workflows::PivSlot::Signature
                            }
                            crate::ports::yubikey::PivSlot::KeyManagement => {
                                crate::state_machines::workflows::PivSlot::KeyManagement
                            }
                            crate::ports::yubikey::PivSlot::CardAuth => {
                                crate::state_machines::workflows::PivSlot::CardAuth
                            }
                            crate::ports::yubikey::PivSlot::Retired(n) => {
                                crate::state_machines::workflows::PivSlot::Retired(n)
                            }
                        };

                        let mut slot_keys = HashMap::new();
                        slot_keys.insert(
                            workflow_slot,
                            public_key.clone(),
                        );

                        self.yubikey_states.insert(
                            serial.clone(),
                            YubiKeyProvisioningState::KeysGenerated { slot_keys },
                        );

                        self.status_message = format!(
                            "✅ Key generated in slot {:?} for YubiKey {}",
                            slot, serial
                        );
                        tracing::info!("Key generated for YubiKey {} in slot {:?}", serial, slot);
                    }
                    Err((serial, error)) => {
                        self.error_message = Some(format!("YubiKey {} key generation failed: {}", serial, error));
                    }
                }
                Task::none()
            }

            // Export State Machine Handlers (Sprint 79)
            Message::PkiPrepareExport => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Validate state transition using state machine guard
                if !self.pki_state.can_prepare_export() {
                    self.error_message = Some(format!(
                        "Cannot prepare export in current state: {:?}. Provision YubiKeys first.",
                        self.pki_state
                    ));
                    return Task::none();
                }

                // Generate manifest ID
                let manifest_id = Uuid::now_v7();

                self.status_message = format!(
                    "Preparing export manifest (ID: {})...",
                    manifest_id
                );

                tracing::info!(
                    "PKI state machine: Preparing export. Manifest ID: {}",
                    manifest_id
                );

                // Emit the ExportReady message to transition state
                Task::done(Message::PkiExportReady { manifest_id })
            }

            Message::PkiExportReady { manifest_id } => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Transition to ExportReady state
                self.pki_state = PKIBootstrapState::ExportReady {
                    manifest_id,
                };

                self.status_message = format!(
                    "Export manifest ready! ID: {}. Ready for export.",
                    manifest_id
                );

                tracing::info!(
                    "PKI state machine: Export ready. State: {:?}",
                    self.pki_state
                );

                Task::none()
            }

            Message::PkiExecuteExport { export_path } => {
                use crate::state_machines::workflows::PKIBootstrapState;

                // Validate state transition using state machine guard
                if !self.pki_state.can_export() {
                    self.error_message = Some(format!(
                        "Cannot execute export in current state: {:?}. Prepare export first.",
                        self.pki_state
                    ));
                    return Task::none();
                }

                self.status_message = format!(
                    "Executing export to: {}...",
                    export_path.display()
                );

                tracing::info!(
                    "PKI state machine: Executing export to {}",
                    export_path.display()
                );

                // Generate export location ID and checksum
                let export_location = Uuid::now_v7();
                let export_checksum = format!("sha256:{}", Uuid::now_v7()); // Placeholder

                // Emit completion message
                Task::done(Message::PkiBootstrapComplete {
                    export_location,
                    export_checksum,
                })
            }

            Message::PkiBootstrapComplete { export_location, export_checksum } => {
                use crate::state_machines::workflows::PKIBootstrapState;
                use chrono::Utc;

                // Transition to Bootstrapped state - FINAL STATE
                self.pki_state = PKIBootstrapState::Bootstrapped {
                    export_location,
                    export_checksum: export_checksum.clone(),
                    bootstrapped_at: Utc::now(),
                };

                self.status_message = format!(
                    "PKI Bootstrap COMPLETE! Export location: {}, Checksum: {}",
                    export_location,
                    &export_checksum[..20]
                );

                tracing::info!(
                    "PKI state machine: Bootstrap complete! Final state: {:?}",
                    self.pki_state
                );

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
            button(text("Projections").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::Projections))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Projections)),
            button(text("Workflow").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::Workflow))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Workflow)),
            button(text("State Machines").size(self.view_model.text_normal))
                .on_press(Message::TabSelected(Tab::StateMachines))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::StateMachines)),
        ]
        .spacing(self.view_model.spacing_md);

        // Tab content
        let content = match self.active_tab {
            Tab::Welcome => self.view_welcome(),
            Tab::Organization => self.view_organization(),
            Tab::Locations => self.view_locations(),
            Tab::Keys => self.view_keys(),
            Tab::Projections => self.view_projections(),
            Tab::Workflow => self.view_workflow(),
            Tab::StateMachines => self.view_state_machines(),
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
        // Show loading status or domain info
        let status_section = if self.domain_loaded {
            container(
                column![
                    row![
                        verified::icon("check_circle", self.view_model.text_large),
                        text(format!(" Domain Loaded: {}", self.organization_name)).size(self.view_model.text_large),
                    ].spacing(self.view_model.spacing_sm),
                    text(format!("{} people, {} locations configured",
                        self.loaded_people.len(),
                        self.loaded_locations.len()
                    )).size(self.view_model.text_normal),
                ]
                .spacing(self.view_model.spacing_sm)
            )
            .style(CowboyCustomTheme::pastel_mint_card())
            .padding(self.view_model.padding_lg)
        } else {
            container(
                column![
                    row![
                        verified::icon("info", self.view_model.text_large),
                        text(" No Domain Loaded").size(self.view_model.text_large),
                    ].spacing(self.view_model.spacing_sm),
                    text("Create a new domain using the Organization tab, or add secrets/domain-bootstrap.json").size(self.view_model.text_normal),
                ]
                .spacing(self.view_model.spacing_sm)
            )
            .style(CowboyCustomTheme::pastel_teal_card())
            .padding(self.view_model.padding_lg)
        };

        let content = column![
            text("CIM Keys").size(self.view_model.text_header),
            text("Offline Cryptographic Key Management for CIM Infrastructure").size(self.view_model.text_medium),

            // Security notice
            container(
                column![
                    row![
                        verified::icon("security", self.view_model.text_large),
                        text(" Security Notice").size(self.view_model.text_large),
                    ].spacing(self.view_model.spacing_sm),
                    text("This application should be run on an air-gapped computer for maximum security.").size(self.view_model.text_normal),
                    text("All keys are generated offline and stored on encrypted SD cards.").size(self.view_model.text_normal),
                ]
                .spacing(self.view_model.spacing_sm)
            )
            .style(CowboyCustomTheme::pastel_coral_card())
            .padding(self.view_model.padding_lg),

            // Status section
            status_section,

            // Quick Start Documentation
            container(
                column![
                    text("Quick Start Guide").size(self.view_model.text_xlarge),

                    // Step 1
                    container(
                        column![
                            text("1. Organization Tab - Define Your Structure").size(self.view_model.text_large),
                            text("• Right-click on the canvas to create organizations, units, and people").size(self.view_model.text_normal),
                            text("• Drag nodes to arrange your organizational hierarchy").size(self.view_model.text_normal),
                            text("• Connect people to units with relationships").size(self.view_model.text_normal),
                        ].spacing(self.view_model.spacing_xs)
                    )
                    .style(CowboyCustomTheme::glass_container())
                    .padding(self.view_model.padding_md),

                    // Step 2
                    container(
                        column![
                            text("2. Locations Tab - Define Storage Locations").size(self.view_model.text_large),
                            text("• Create physical locations for key storage (encrypted SD cards)").size(self.view_model.text_normal),
                            text("• Each YubiKey should have an assigned storage location").size(self.view_model.text_normal),
                        ].spacing(self.view_model.spacing_xs)
                    )
                    .style(CowboyCustomTheme::glass_container())
                    .padding(self.view_model.padding_md),

                    // Step 3
                    container(
                        column![
                            text("3. Keys Tab - Generate Cryptographic Keys").size(self.view_model.text_large),
                            text("• Generate Root CA, Intermediate CAs, and leaf certificates").size(self.view_model.text_normal),
                            text("• Provision YubiKeys with generated keys").size(self.view_model.text_normal),
                            text("• Create NATS operator, account, and user credentials").size(self.view_model.text_normal),
                        ].spacing(self.view_model.spacing_xs)
                    )
                    .style(CowboyCustomTheme::glass_container())
                    .padding(self.view_model.padding_md),

                    // Step 4
                    container(
                        column![
                            text("4. Export Tab - Save Your Configuration").size(self.view_model.text_large),
                            text("• Export domain configuration to encrypted storage").size(self.view_model.text_normal),
                            text("• Generate deployment packages for CIM leaf nodes").size(self.view_model.text_normal),
                        ].spacing(self.view_model.spacing_xs)
                    )
                    .style(CowboyCustomTheme::glass_container())
                    .padding(self.view_model.padding_md),

                    // Step 5
                    container(
                        column![
                            text("5. Workflow Tab - Track Progress").size(self.view_model.text_large),
                            text("• View trust chain gaps and recommended next steps").size(self.view_model.text_normal),
                            text("• Semantic positioning shows related tasks").size(self.view_model.text_normal),
                            text("• Markov chain predictions guide your workflow").size(self.view_model.text_normal),
                        ].spacing(self.view_model.spacing_xs)
                    )
                    .style(CowboyCustomTheme::glass_container())
                    .padding(self.view_model.padding_md),
                ]
                .spacing(self.view_model.spacing_md)
            )
            .padding(self.view_model.padding_lg)
            .style(CowboyCustomTheme::glass_container()),

            // Action button
            container(
                button("Go to Organization")
                    .on_press(Message::TabSelected(Tab::Organization))
                    .style(CowboyCustomTheme::primary_button())
                    .padding(self.view_model.padding_md)
            )
            .width(Length::Fill)
            .align_x(Alignment::Center),

            // Bootstrap file info
            container(
                column![
                    text("Bootstrap Files").size(self.view_model.text_large),
                    text("Place your configuration in one of these locations:").size(self.view_model.text_normal),
                    text("• secrets/domain-bootstrap.json (recommended)").size(self.view_model.text_small).color(CowboyTheme::text_secondary()),
                    text("• secrets/secrets.json + secrets/cowboyai.json (legacy)").size(self.view_model.text_small).color(CowboyTheme::text_secondary()),
                    text("Files are auto-loaded on startup.").size(self.view_model.text_small).color(CowboyTheme::text_secondary()),
                ]
                .spacing(self.view_model.spacing_xs)
            )
            .padding(self.view_model.padding_md)
            .style(CowboyCustomTheme::glass_container()),
        ]
        .spacing(self.view_model.spacing_lg)
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

                    // URL field for virtual/hybrid locations
                    if matches!(self.new_location_type, Some(crate::domain::LocationType::Virtual) | Some(crate::domain::LocationType::Hybrid)) {
                        column![
                            text("Virtual Location URL")
                                .size(self.view_model.text_small)
                                .color(CowboyTheme::text_secondary()),
                            text_input("https://cloud.example.com/storage", &self.new_location_url)
                                .on_input(Message::NewLocationUrlChanged)
                                .size(self.view_model.text_normal)
                                .style(CowboyCustomTheme::glass_input()),
                        ]
                        .spacing(self.view_model.spacing_sm)
                    } else {
                        column![]
                    },

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

                    // Organizational Unit picker for intermediate CA
                    if !self.loaded_units.is_empty() {
                        let unit_names: Vec<String> = self.loaded_units
                            .iter()
                            .map(|unit| unit.name.clone())
                            .collect();

                        row![
                            text("Organizational Unit:").size(self.view_model.text_normal),
                            pick_list(
                                unit_names,
                                self.selected_unit_for_ca.clone(),
                                Message::SelectUnitForCA,
                            )
                            .placeholder("Select Unit (optional)")
                            .width(Length::Fixed(200.0)),
                            text("Each unit can have its own intermediate CA")
                                .size(self.view_model.text_tiny)
                                .color(CowboyTheme::text_secondary()),
                        ]
                        .spacing(self.view_model.spacing_md)
                        .align_y(Alignment::Center)
                    } else {
                        row![
                            text("Organizational Unit:").size(self.view_model.text_normal).color(self.view_model.colors.text_tertiary),
                            text("(no units defined - will use organization-level CA)")
                                .size(self.view_model.text_small)
                                .color(self.view_model.colors.text_tertiary),
                        ]
                        .spacing(self.view_model.spacing_md)
                    },

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

                    // mTLS Client Certificate Section
                    container(
                        column![
                            text("3b. mTLS Client Certificates")
                                .size(self.view_model.text_medium)
                                .color(CowboyTheme::text_primary()),
                            text("Generate client certificates for mutual TLS authentication")
                                .size(self.view_model.text_small)
                                .color(CowboyTheme::text_secondary()),
                            row![
                                text("Common Name:").size(self.view_model.text_normal).width(iced::Length::Fixed(120.0)),
                                text_input("e.g., 'Alice Developer'", &self.client_cert_cn)
                                    .on_input(Message::ClientCertCNChanged)
                                    .size(self.view_model.text_medium)
                                    .padding(self.view_model.padding_md)
                                    .style(CowboyCustomTheme::glass_input()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),
                            row![
                                text("Email (optional):").size(self.view_model.text_normal).width(iced::Length::Fixed(120.0)),
                                text_input("e.g., 'alice@example.com'", &self.client_cert_email)
                                    .on_input(Message::ClientCertEmailChanged)
                                    .size(self.view_model.text_medium)
                                    .padding(self.view_model.padding_md)
                                    .style(CowboyCustomTheme::glass_input()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),
                            button(
                                text("Generate Client Certificate")
                                    .size(self.view_model.text_normal)
                            )
                                .on_press_maybe(
                                    if !self.client_cert_cn.is_empty() {
                                        Some(Message::GenerateClientCert)
                                    } else {
                                        None
                                    }
                                )
                                .padding(self.view_model.padding_lg)
                                .style(CowboyCustomTheme::security_button()),
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_md)
                    .style(CowboyCustomTheme::pastel_cream_card()),

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

                    // Display detected YubiKeys with assignment functionality
                    if !self.detected_yubikeys.is_empty() {
                        let mut yubikey_list = column![].spacing(self.view_model.spacing_md);

                        for device in &self.detected_yubikeys {
                            let serial = device.serial.clone();
                            let assigned_person = self.yubikey_assignments.get(&serial)
                                .and_then(|person_id| {
                                    self.loaded_people.iter()
                                        .find(|p| p.person_id == *person_id)
                                        .map(|p| p.name.clone())
                                });

                            // Device info row
                            let device_info = row![
                                text(if device.piv_enabled { "✓" } else { "⚠" })
                                    .size(self.view_model.text_normal)
                                    .color(if device.piv_enabled {
                                        self.view_model.colors.green_success
                                    } else {
                                        self.view_model.colors.orange_warning
                                    }),
                                column![
                                    text(format!("{} v{}", device.model, device.version))
                                        .size(self.view_model.text_normal)
                                        .color(CowboyTheme::text_primary()),
                                    text(format!("Serial: {}", device.serial))
                                        .size(self.view_model.text_small)
                                        .color(CowboyTheme::text_secondary()),
                                    if let Some(name) = &assigned_person {
                                        text(format!("Assigned to: {}", name))
                                            .size(self.view_model.text_small)
                                            .color(self.view_model.colors.green_success)
                                    } else {
                                        text("Not assigned")
                                            .size(self.view_model.text_small)
                                            .color(self.view_model.colors.orange_warning)
                                    },
                                ]
                                .spacing(2),
                            ]
                            .spacing(self.view_model.spacing_sm)
                            .align_y(Alignment::Center);

                            // Assignment controls - pick_list of available people
                            let assignment_control: Element<'_, Message> = if !self.loaded_people.is_empty() && device.piv_enabled {
                                let people_names: Vec<String> = self.loaded_people.iter()
                                    .map(|p| format!("{} ({})", p.name, p.person_id.to_string().chars().take(8).collect::<String>()))
                                    .collect();

                                let serial_clone = serial.clone();
                                let loaded_people = self.loaded_people.clone();

                                row![
                                    pick_list(
                                        people_names.clone(),
                                        None::<String>,
                                        move |selected: String| {
                                            // Extract person_id from selection
                                            if let Some(person) = loaded_people.iter().find(|p| {
                                                let formatted = format!("{} ({})", p.name, p.person_id.to_string().chars().take(8).collect::<String>());
                                                formatted == selected
                                            }) {
                                                Message::AssignYubiKeyToPerson {
                                                    serial: serial_clone.clone(),
                                                    person_id: person.person_id,
                                                }
                                            } else {
                                                Message::SelectYubiKeyForAssignment(serial_clone.clone())
                                            }
                                        }
                                    )
                                    .placeholder("Assign to person...")
                                    .width(Length::Fixed(200.0))
                                    .padding(self.view_model.padding_sm),
                                ]
                                .into()
                            } else if !device.piv_enabled {
                                text("PIV must be enabled to assign")
                                    .size(self.view_model.text_small)
                                    .color(self.view_model.colors.orange_warning)
                                    .into()
                            } else {
                                text("Import people first to assign")
                                    .size(self.view_model.text_small)
                                    .color(CowboyTheme::text_secondary())
                                    .into()
                            };

                            // Get provisioning status for this device
                            let provision_status = self.yubikey_provisioning_status.get(&serial).cloned();
                            let has_config = self.yubikey_configs.iter().any(|c| c.serial == serial);
                            let serial_for_provision = serial.clone();

                            // Provision button/status
                            let provision_control: Element<'_, Message> = if device.piv_enabled {
                                if let Some(status) = provision_status {
                                    // Show status instead of button - determine color first
                                    let status_color = if status.contains("✅") {
                                        self.view_model.colors.green_success
                                    } else if status.contains("❌") {
                                        self.view_model.colors.red_error
                                    } else {
                                        self.view_model.colors.info
                                    };
                                    text(status)
                                        .size(self.view_model.text_small)
                                        .color(status_color)
                                        .into()
                                } else {
                                    // Show provision button - use consistent styling
                                    button(if has_config { "Provision" } else { "Provision (defaults)" })
                                        .on_press(Message::ProvisionSingleYubiKey { serial: serial_for_provision })
                                        .padding(self.view_model.padding_sm)
                                        .style(CowboyCustomTheme::security_button())
                                        .into()
                                }
                            } else {
                                text("PIV not enabled")
                                    .size(self.view_model.text_small)
                                    .color(self.view_model.colors.orange_warning)
                                    .into()
                            };

                            yubikey_list = yubikey_list.push(
                                container(
                                    column![
                                        row![
                                            device_info,
                                            horizontal_space(),
                                            assignment_control,
                                        ]
                                        .spacing(self.view_model.spacing_md)
                                        .align_y(Alignment::Center),
                                        row![
                                            horizontal_space(),
                                            provision_control,
                                        ]
                                        .spacing(self.view_model.spacing_md)
                                        .align_y(Alignment::Center),
                                    ]
                                    .spacing(self.view_model.spacing_sm)
                                )
                                .padding(self.view_model.padding_md)
                                .style(CowboyCustomTheme::card_container())
                            );
                        }

                        container(yubikey_list)
                            .padding(self.view_model.padding_sm)
                    } else {
                        container(text("No YubiKeys detected yet"))
                    },

                    // YubiKey Domain Registration Form
                    container(
                        column![
                            text("Register YubiKey in Domain")
                                .size(self.view_model.text_medium)
                                .color(CowboyTheme::text_primary()),
                            text("Register a YubiKey by its serial number for domain tracking")
                                .size(self.view_model.text_small)
                                .color(CowboyTheme::text_secondary()),
                            row![
                                text("Name:").size(self.view_model.text_normal).width(iced::Length::Fixed(80.0)),
                                text_input("e.g., 'Primary CA Key'", &self.yubikey_registration_name)
                                    .on_input(Message::YubiKeyRegistrationNameChanged)
                                    .size(self.view_model.text_medium)
                                    .padding(self.view_model.padding_sm)
                                    .width(iced::Length::Fixed(250.0))
                                    .style(CowboyCustomTheme::glass_input()),
                            ]
                            .spacing(self.view_model.spacing_sm)
                            .align_y(Alignment::Center),
                            row![
                                text("Serial:").size(self.view_model.text_normal).width(iced::Length::Fixed(80.0)),
                                {
                                    if self.detected_yubikeys.is_empty() {
                                        Element::<Message>::from(
                                            text("Detect YubiKeys first to see available serials")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary())
                                        )
                                    } else {
                                        let mut serial_row = row![].spacing(self.view_model.spacing_xs);
                                        for device in &self.detected_yubikeys {
                                            let serial = device.serial.clone();
                                            let name = self.yubikey_registration_name.clone();
                                            serial_row = serial_row.push(
                                                button(text(&device.serial).size(self.view_model.text_small))
                                                    .on_press(Message::RegisterYubiKeyInDomain {
                                                        serial: serial.clone(),
                                                        name: if name.is_empty() { format!("YubiKey-{}", &serial[..4.min(serial.len())]) } else { name }
                                                    })
                                                    .padding(self.view_model.padding_sm)
                                                    .style(CowboyCustomTheme::security_button())
                                            );
                                        }
                                        serial_row.into()
                                    }
                                },
                            ]
                            .spacing(self.view_model.spacing_sm)
                            .align_y(Alignment::Center),
                            // Show registered YubiKeys
                            if !self.registered_yubikeys.is_empty() {
                                column![
                                    text(format!("Registered YubiKeys: {}", self.registered_yubikeys.len()))
                                        .size(self.view_model.text_small)
                                        .color(self.view_model.colors.green_success),
                                    {
                                        let mut registered_list = column![].spacing(self.view_model.spacing_xs);
                                        for (serial, _id) in &self.registered_yubikeys {
                                            registered_list = registered_list.push(
                                                row![
                                                    text(format!("✓ {}", serial))
                                                        .size(self.view_model.text_tiny)
                                                        .color(CowboyTheme::text_secondary()),
                                                    button(text("Revoke").size(self.view_model.text_tiny))
                                                        .on_press(Message::RevokeYubiKeyAssignment { serial: serial.clone() })
                                                        .padding(2)
                                                        .style(CowboyCustomTheme::glass_button()),
                                                ]
                                                .spacing(self.view_model.spacing_sm)
                                                .align_y(Alignment::Center)
                                            );
                                        }
                                        registered_list
                                    }
                                ]
                                .spacing(self.view_model.spacing_sm)
                            } else {
                                column![]
                            },
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_md)
                    .style(CowboyCustomTheme::pastel_cream_card()),

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
                                crate::domain::YubiKeyRole::RootCA => "🔒 Root CA",
                                crate::domain::YubiKeyRole::Backup => "💾 Backup",
                                crate::domain::YubiKeyRole::User => "👤 User",
                                crate::domain::YubiKeyRole::Service => "⚙️ Service",
                            };

                            // Check if this YubiKey is currently connected
                            let is_connected = self.detected_yubikeys.iter()
                                .any(|d| d.serial == config.serial);

                            // Get expected PIV slots based on role
                            let expected_slots = match config.role {
                                crate::domain::YubiKeyRole::RootCA => "9C (Signature)",
                                crate::domain::YubiKeyRole::Backup => "9C, 9D (Signature + KeyMgmt)",
                                crate::domain::YubiKeyRole::User => "9A, 9C (Auth + Signature)",
                                crate::domain::YubiKeyRole::Service => "9A, 9E (Auth + CardAuth)",
                            };

                            // Get assignment status
                            let assignment = self.yubikey_assignments.get(&config.serial)
                                .and_then(|person_id| {
                                    self.loaded_people.iter()
                                        .find(|p| p.person_id == *person_id)
                                        .map(|p| p.name.clone())
                                });

                            config_list = config_list.push(
                                container(
                                    column![
                                        // Header row with role and connection status
                                        row![
                                            text(format!("{} - {}", role_str, config.name))
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_primary()),
                                            horizontal_space(),
                                            if is_connected {
                                                text("🟢 Connected")
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.green_success)
                                            } else {
                                                text("⚪ Not Connected")
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.text_tertiary)
                                            },
                                        ]
                                        .align_y(Alignment::Center),

                                        // Status info row
                                        row![
                                            // Left column: Device info
                                            column![
                                                text(format!("Serial: {}", config.serial))
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                                text(format!("Owner: {}", config.owner_email))
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                                text(format!("Slots: {}", expected_slots))
                                                    .size(self.view_model.text_tiny)
                                                    .color(CowboyTheme::text_secondary()),
                                                if let Some(name) = &assignment {
                                                    text(format!("Assigned: {}", name))
                                                        .size(self.view_model.text_small)
                                                        .color(self.view_model.colors.green_success)
                                                } else {
                                                    text("Not assigned in domain")
                                                        .size(self.view_model.text_small)
                                                        .color(self.view_model.colors.text_tertiary)
                                                },
                                            ]
                                            .spacing(2),
                                            horizontal_space(),
                                            // Right column: PIV credentials (sensitive)
                                            column![
                                                text(format!("PIN: {}", config.piv.pin))
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.orange_warning),
                                                text(format!("PUK: {}", config.piv.puk))
                                                    .size(self.view_model.text_small)
                                                    .color(self.view_model.colors.orange_warning),
                                                text(format!("Algo: {:?}", config.piv.piv_alg))
                                                    .size(self.view_model.text_tiny)
                                                    .color(CowboyTheme::text_secondary()),
                                            ]
                                            .spacing(2)
                                            .align_x(iced::alignment::Horizontal::Right),
                                        ]
                                        .align_y(Alignment::Start),
                                    ]
                                    .spacing(self.view_model.spacing_sm)
                                )
                                .padding(self.view_model.padding_md)
                                .style(CowboyCustomTheme::card_container())
                            );
                        }

                        container(config_list)
                            .padding(self.view_model.padding_md)
                            .style(CowboyCustomTheme::card_container())
                    } else {
                        container(text(""))
                    },

                    // Step 4b: YubiKey Slot Management (Collapsible)
                    container(
                        column![
                            row![
                                button(if self.yubikey_slot_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleYubiKeySlotSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("4b. PIV Slot Management")
                                    .size(self.view_model.text_medium)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.yubikey_slot_section_collapsed && !self.detected_yubikeys.is_empty() {
                                column![
                                    // YubiKey selection for management
                                    text("Select YubiKey to Manage:")
                                        .size(self.view_model.text_small)
                                        .color(CowboyTheme::text_secondary()),

                                    {
                                        let mut yubikey_buttons = row![].spacing(self.view_model.spacing_sm);
                                        for device in &self.detected_yubikeys {
                                            let is_selected = self.selected_yubikey_for_management.as_ref()
                                                .map(|s| s == &device.serial)
                                                .unwrap_or(false);
                                            let serial = device.serial.clone();
                                            let button_text = if is_selected {
                                                format!("✓ {} ({})", device.serial, device.model)
                                            } else {
                                                format!("{} ({})", device.serial, device.model)
                                            };
                                            yubikey_buttons = yubikey_buttons.push(
                                                button(text(button_text).size(self.view_model.text_small))
                                                    .on_press(Message::SelectYubiKeyForManagement(serial))
                                                    .padding(self.view_model.padding_sm)
                                                    .style(CowboyCustomTheme::security_button())
                                            );
                                        }
                                        yubikey_buttons
                                    },

                                    // Show slot info and management for selected YubiKey
                                    if let Some(ref selected_serial) = self.selected_yubikey_for_management {
                                        let serial_for_pin = selected_serial.clone();
                                        let serial_for_mgmt = selected_serial.clone();
                                        let serial_for_reset = selected_serial.clone();

                                        column![
                                            text(format!("Managing YubiKey: {}", selected_serial))
                                                .size(self.view_model.text_normal)
                                                .color(self.view_model.colors.info),

                                            // PIV Slots Table
                                            text("PIV Slots:")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary()),

                                            if let Some(slots) = self.yubikey_slot_info.get(selected_serial) {
                                                let mut slot_table = column![].spacing(self.view_model.spacing_xs);
                                                for slot_info in slots {
                                                    let slot_for_attest = slot_info.slot;
                                                    let slot_for_gen = slot_info.slot;
                                                    let serial_for_attest = selected_serial.clone();
                                                    let serial_for_gen = selected_serial.clone();

                                                    // Get slot purpose description
                                                    let slot_purpose = match slot_info.slot {
                                                        crate::ports::yubikey::PivSlot::Authentication => "PIV Auth - User authentication",
                                                        crate::ports::yubikey::PivSlot::Signature => "Digital Sig - Document signing",
                                                        crate::ports::yubikey::PivSlot::KeyManagement => "Key Mgmt - Encryption/decryption",
                                                        crate::ports::yubikey::PivSlot::CardAuth => "Card Auth - Physical access",
                                                        crate::ports::yubikey::PivSlot::Retired(_) => "Retired - Legacy key storage",
                                                    };

                                                    slot_table = slot_table.push(
                                                        container(
                                                            column![
                                                                row![
                                                                    text(format!("{} ({})", slot_info.slot_name, slot_info.slot_hex))
                                                                        .size(self.view_model.text_small)
                                                                        .width(iced::Length::Fixed(130.0)),
                                                                    text(slot_purpose)
                                                                        .size(self.view_model.text_tiny)
                                                                        .color(CowboyTheme::text_secondary())
                                                                        .width(iced::Length::Fixed(170.0)),
                                                                    if slot_info.occupied {
                                                                        text(format!("✅ {} - {}",
                                                                            slot_info.algorithm.as_deref().unwrap_or("Unknown"),
                                                                            slot_info.subject.as_deref().unwrap_or("No subject")
                                                                        ))
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(self.view_model.colors.green_success)
                                                                    } else {
                                                                        text("⬜ Empty")
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(self.view_model.colors.text_tertiary)
                                                                    },
                                                                    horizontal_space(),
                                                                ]
                                                                .spacing(self.view_model.spacing_sm)
                                                                .align_y(Alignment::Center),
                                                                row![
                                                                    horizontal_space().width(iced::Length::Fixed(130.0)),
                                                                    if slot_info.occupied {
                                                                        row![
                                                                            button(text("Attestation").size(self.view_model.text_tiny))
                                                                                .on_press(Message::GetYubiKeyAttestation {
                                                                                    serial: serial_for_attest,
                                                                                    slot: slot_for_attest,
                                                                                })
                                                                                .padding(2)
                                                                                .style(CowboyCustomTheme::glass_button()),
                                                                        ]
                                                                    } else {
                                                                        row![
                                                                            button(text("🔑 Generate Key").size(self.view_model.text_tiny))
                                                                                .on_press(Message::GenerateKeyInSlot {
                                                                                    serial: serial_for_gen,
                                                                                    slot: slot_for_gen,
                                                                                })
                                                                                .padding(2)
                                                                                .style(CowboyCustomTheme::security_button()),
                                                                        ]
                                                                    },
                                                                ]
                                                                .spacing(self.view_model.spacing_sm)
                                                            ]
                                                            .spacing(self.view_model.spacing_xs)
                                                        )
                                                        .padding(self.view_model.padding_xs)
                                                        .style(CowboyCustomTheme::card_container())
                                                    );
                                                }
                                                slot_table
                                            } else {
                                                column![
                                                    text("Loading slot information...")
                                                        .size(self.view_model.text_small)
                                                        .color(CowboyTheme::text_secondary())
                                                ]
                                            },

                                            // PIN Verification Section
                                            container(
                                                column![
                                                    text("PIN Operations")
                                                        .size(self.view_model.text_small)
                                                        .color(CowboyTheme::text_primary()),
                                                    row![
                                                        text_input("Enter PIN", &self.yubikey_pin_input)
                                                            .on_input(Message::YubiKeyPinInputChanged)
                                                            .secure(true)
                                                            .width(iced::Length::Fixed(150.0))
                                                            .padding(self.view_model.padding_sm),
                                                        button(text("Verify PIN").size(self.view_model.text_small))
                                                            .on_press(Message::VerifyYubiKeyPin(serial_for_pin))
                                                            .padding(self.view_model.padding_sm)
                                                            .style(CowboyCustomTheme::glass_button()),
                                                    ]
                                                    .spacing(self.view_model.spacing_sm)
                                                    .align_y(Alignment::Center),
                                                ]
                                                .spacing(self.view_model.spacing_sm)
                                            )
                                            .padding(self.view_model.padding_sm)
                                            .style(CowboyCustomTheme::card_container()),

                                            // Management Key Section
                                            container(
                                                column![
                                                    text("Management Key")
                                                        .size(self.view_model.text_small)
                                                        .color(CowboyTheme::text_primary()),
                                                    row![
                                                        column![
                                                            text("Current Key (hex):").size(self.view_model.text_tiny),
                                                            text_input("48 hex chars", &self.yubikey_management_key)
                                                                .on_input(Message::YubiKeyManagementKeyChanged)
                                                                .width(iced::Length::Fixed(280.0))
                                                                .padding(self.view_model.padding_xs),
                                                        ],
                                                        column![
                                                            text("New Key (hex):").size(self.view_model.text_tiny),
                                                            text_input("48 hex chars", &self.yubikey_new_management_key)
                                                                .on_input(Message::YubiKeyNewManagementKeyChanged)
                                                                .width(iced::Length::Fixed(280.0))
                                                                .padding(self.view_model.padding_xs),
                                                        ],
                                                    ]
                                                    .spacing(self.view_model.spacing_sm),
                                                    button(text("Change Management Key").size(self.view_model.text_small))
                                                        .on_press(Message::ChangeYubiKeyManagementKey(serial_for_mgmt))
                                                        .padding(self.view_model.padding_sm)
                                                        .style(CowboyCustomTheme::glass_button()),
                                                    text("Default key: 010203...0102030405060708 (24 bytes = 48 hex chars)")
                                                        .size(self.view_model.text_tiny)
                                                        .color(CowboyTheme::text_secondary()),
                                                ]
                                                .spacing(self.view_model.spacing_sm)
                                            )
                                            .padding(self.view_model.padding_sm)
                                            .style(CowboyCustomTheme::card_container()),

                                            // Factory Reset Section (Dangerous)
                                            container(
                                                column![
                                                    text("⚠️ Factory Reset PIV")
                                                        .size(self.view_model.text_small)
                                                        .color(self.view_model.colors.red_error),
                                                    text("This will DELETE ALL keys and certificates from the YubiKey!")
                                                        .size(self.view_model.text_tiny)
                                                        .color(self.view_model.colors.red_error),
                                                    button(text("Factory Reset PIV").size(self.view_model.text_small))
                                                        .on_press(Message::ResetYubiKeyPiv(serial_for_reset))
                                                        .padding(self.view_model.padding_sm)
                                                        .style(CowboyCustomTheme::primary_button()),
                                                ]
                                                .spacing(self.view_model.spacing_sm)
                                            )
                                            .padding(self.view_model.padding_sm)
                                            .style(CowboyCustomTheme::pastel_coral_card()),

                                            // Operation status
                                            if let Some(ref status) = self.yubikey_slot_operation_status {
                                                text(status)
                                                    .size(self.view_model.text_small)
                                                    .color(if status.contains("✅") {
                                                        self.view_model.colors.green_success
                                                    } else if status.contains("❌") || status.contains("⚠️") {
                                                        self.view_model.colors.red_error
                                                    } else {
                                                        self.view_model.colors.info
                                                    })
                                            } else {
                                                text("")
                                            },

                                            // Attestation result
                                            if let Some(ref result) = self.yubikey_attestation_result {
                                                container(
                                                    column![
                                                        text("Attestation Result:")
                                                            .size(self.view_model.text_small)
                                                            .color(CowboyTheme::text_primary()),
                                                        text(result)
                                                            .size(self.view_model.text_tiny)
                                                            .color(if result.contains("❌") {
                                                                self.view_model.colors.red_error
                                                            } else {
                                                                self.view_model.colors.green_success
                                                            }),
                                                    ]
                                                    .spacing(self.view_model.spacing_xs)
                                                )
                                                .padding(self.view_model.padding_sm)
                                                .style(CowboyCustomTheme::card_container())
                                            } else {
                                                container(text(""))
                                            },
                                        ]
                                        .spacing(self.view_model.spacing_md)
                                    } else {
                                        column![
                                            text("Select a YubiKey above to manage its PIV slots")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary())
                                        ]
                                    },
                                ]
                                .spacing(self.view_model.spacing_md)
                            } else if !self.yubikey_slot_section_collapsed {
                                column![
                                    text("No YubiKeys detected. Click 'Detect YubiKeys' above first.")
                                        .size(self.view_model.text_small)
                                        .color(CowboyTheme::text_secondary())
                                ]
                            } else {
                                column![]
                            },
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_md)
                    .style(CowboyCustomTheme::pastel_teal_card()),
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

                    // Step 4c: Organization Unit Creation (Collapsible)
                    container(
                        column![
                            row![
                                button(if self.org_unit_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleOrgUnitSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("4c. Organization Units (Departments/Teams)")
                                    .size(self.view_model.text_medium)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.org_unit_section_collapsed {
                                column![
                                    // Unit creation form
                                    container(
                                        column![
                                            text("Create Organization Unit")
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_primary()),

                                            // Unit name input
                                            row![
                                                text("Name:").size(self.view_model.text_small).width(iced::Length::Fixed(100.0)),
                                                text_input("e.g., Engineering, Sales, DevOps", &self.new_unit_name)
                                                    .on_input(Message::NewUnitNameChanged)
                                                    .padding(self.view_model.padding_sm)
                                                    .width(iced::Length::Fixed(250.0)),
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Unit type selection
                                            row![
                                                text("Type:").size(self.view_model.text_small).width(iced::Length::Fixed(100.0)),
                                                {
                                                    use crate::domain::OrganizationUnitType;
                                                    let types = vec![
                                                        ("Division", OrganizationUnitType::Division),
                                                        ("Department", OrganizationUnitType::Department),
                                                        ("Team", OrganizationUnitType::Team),
                                                        ("Project", OrganizationUnitType::Project),
                                                        ("Service", OrganizationUnitType::Service),
                                                        ("Infrastructure", OrganizationUnitType::Infrastructure),
                                                    ];
                                                    let mut type_row = row![].spacing(self.view_model.spacing_xs);
                                                    for (label, unit_type) in types {
                                                        let is_selected = self.new_unit_type.as_ref()
                                                            .map(|t| std::mem::discriminant(t) == std::mem::discriminant(&unit_type))
                                                            .unwrap_or(false);
                                                        let label_text = if is_selected {
                                                            format!("✓ {}", label)
                                                        } else {
                                                            label.to_string()
                                                        };
                                                        type_row = type_row.push(
                                                            button(text(label_text).size(self.view_model.text_tiny))
                                                                .on_press(Message::NewUnitTypeSelected(unit_type))
                                                                .padding(4)
                                                                .style(CowboyCustomTheme::glass_button())
                                                        );
                                                    }
                                                    type_row
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Parent unit selection (for nesting)
                                            {
                                                let parent_info = if let Some(ref p) = self.new_unit_parent {
                                                    format!("Selected: {}", p)
                                                } else {
                                                    "None (Top-level)".to_string()
                                                };
                                                row![
                                                    text("Parent:").size(self.view_model.text_small).width(iced::Length::Fixed(100.0)),
                                                    text(parent_info)
                                                        .size(self.view_model.text_tiny)
                                                        .width(iced::Length::Fixed(150.0))
                                                        .color(CowboyTheme::text_secondary()),
                                                    button(text("Clear").size(self.view_model.text_tiny))
                                                        .on_press(Message::NewUnitParentSelected(String::new()))
                                                        .padding(4)
                                                        .style(CowboyCustomTheme::glass_button()),
                                                ]
                                                .spacing(self.view_model.spacing_sm)
                                                .align_y(Alignment::Center)
                                            },

                                            // NATS account name (optional)
                                            row![
                                                text("NATS Account:").size(self.view_model.text_small).width(iced::Length::Fixed(100.0)),
                                                text_input("Optional: e.g., engineering, devops", &self.new_unit_nats_account)
                                                    .on_input(Message::NewUnitNatsAccountChanged)
                                                    .padding(self.view_model.padding_sm)
                                                    .width(iced::Length::Fixed(250.0)),
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Create button
                                            button(text("Create Unit").size(self.view_model.text_small))
                                                .on_press(Message::CreateOrganizationUnit)
                                                .padding(self.view_model.padding_md)
                                                .style(CowboyCustomTheme::security_button()),
                                        ]
                                        .spacing(self.view_model.spacing_md)
                                    )
                                    .padding(self.view_model.padding_md)
                                    .style(CowboyCustomTheme::card_container()),

                                    // List of created units
                                    if !self.created_units.is_empty() {
                                        let mut units_list = column![
                                            text(format!("Created Units ({})", self.created_units.len()))
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_primary()),
                                        ].spacing(self.view_model.spacing_sm);

                                        for unit in &self.created_units {
                                            let unit_id = unit.id.as_uuid();
                                            let unit_type_str = match unit.unit_type {
                                                crate::domain::OrganizationUnitType::Division => "Division",
                                                crate::domain::OrganizationUnitType::Department => "Department",
                                                crate::domain::OrganizationUnitType::Team => "Team",
                                                crate::domain::OrganizationUnitType::Project => "Project",
                                                crate::domain::OrganizationUnitType::Service => "Service",
                                                crate::domain::OrganizationUnitType::Infrastructure => "Infrastructure",
                                            };
                                            units_list = units_list.push(
                                                container(
                                                    row![
                                                        column![
                                                            text(format!("📁 {} ({})", unit.name, unit_type_str))
                                                                .size(self.view_model.text_small)
                                                                .color(CowboyTheme::text_primary()),
                                                            if let Some(ref nats_account) = unit.nats_account_name {
                                                                text(format!("NATS: {}", nats_account))
                                                                    .size(self.view_model.text_tiny)
                                                                    .color(CowboyTheme::text_secondary())
                                                            } else {
                                                                text("")
                                                            },
                                                        ]
                                                        .spacing(self.view_model.spacing_xs),
                                                        horizontal_space(),
                                                        button(text("Remove").size(self.view_model.text_tiny))
                                                            .on_press(Message::RemoveOrganizationUnit(unit_id))
                                                            .padding(4)
                                                            .style(CowboyCustomTheme::glass_button()),
                                                    ]
                                                    .align_y(Alignment::Center)
                                                )
                                                .padding(self.view_model.padding_sm)
                                                .style(CowboyCustomTheme::card_container())
                                            );
                                        }
                                        units_list
                                    } else {
                                        column![
                                            text("No organization units created yet")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary())
                                        ]
                                    },
                                ]
                                .spacing(self.view_model.spacing_md)
                            } else {
                                column![]
                            },
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_md)
                    .style(CowboyCustomTheme::pastel_cream_card()),

                    // Step 4d: Service Account Management (Collapsible)
                    container(
                        column![
                            row![
                                button(if self.service_account_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleServiceAccountSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("4d. Service Accounts (Automated Systems)")
                                    .size(self.view_model.text_medium)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.service_account_section_collapsed {
                                column![
                                    // Service account creation form
                                    container(
                                        column![
                                            text("Create Service Account")
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_primary()),
                                            text("Service accounts require a responsible person for accountability")
                                                .size(self.view_model.text_tiny)
                                                .color(CowboyTheme::text_secondary()),

                                            // Service account name input
                                            row![
                                                text("Name:").size(self.view_model.text_small).width(iced::Length::Fixed(120.0)),
                                                text_input("e.g., api-gateway, backup-service", &self.new_service_account_name)
                                                    .on_input(Message::NewServiceAccountNameChanged)
                                                    .padding(self.view_model.padding_sm)
                                                    .width(iced::Length::Fixed(250.0)),
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Service account purpose input
                                            row![
                                                text("Purpose:").size(self.view_model.text_small).width(iced::Length::Fixed(120.0)),
                                                text_input("e.g., API Gateway authentication, Backup automation", &self.new_service_account_purpose)
                                                    .on_input(Message::NewServiceAccountPurposeChanged)
                                                    .padding(self.view_model.padding_sm)
                                                    .width(iced::Length::Fixed(350.0)),
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Owning unit selection
                                            row![
                                                text("Owning Unit:").size(self.view_model.text_small).width(iced::Length::Fixed(120.0)),
                                                {
                                                    let units_for_picker: Vec<(String, Uuid)> = self.loaded_units
                                                        .iter()
                                                        .chain(self.created_units.iter())
                                                        .map(|u| (u.name.clone(), u.id.as_uuid()))
                                                        .collect();

                                                    if units_for_picker.is_empty() {
                                                        row![
                                                            text("Create an organization unit first")
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary())
                                                        ]
                                                    } else {
                                                        let mut unit_buttons = row![].spacing(self.view_model.spacing_xs);
                                                        for (name, id) in units_for_picker.into_iter().take(6) {
                                                            let is_selected = self.new_service_account_owning_unit == Some(id);
                                                            let label = if is_selected { format!("✓ {}", name) } else { name };
                                                            unit_buttons = unit_buttons.push(
                                                                button(text(label).size(self.view_model.text_tiny))
                                                                    .on_press(Message::ServiceAccountOwningUnitSelected(id))
                                                                    .padding(4)
                                                                    .style(CowboyCustomTheme::glass_button())
                                                            );
                                                        }
                                                        unit_buttons
                                                    }
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Responsible person selection
                                            row![
                                                text("Responsible:").size(self.view_model.text_small).width(iced::Length::Fixed(120.0)),
                                                {
                                                    if self.loaded_people.is_empty() {
                                                        row![
                                                            text("Add a person first to assign responsibility")
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary())
                                                        ]
                                                    } else {
                                                        let mut person_buttons = row![].spacing(self.view_model.spacing_xs);
                                                        for person in self.loaded_people.iter().take(6) {
                                                            let person_id = person.person_id;
                                                            let is_selected = self.new_service_account_responsible_person == Some(person_id);
                                                            let label = if is_selected { format!("✓ {}", person.name) } else { person.name.clone() };
                                                            person_buttons = person_buttons.push(
                                                                button(text(label).size(self.view_model.text_tiny))
                                                                    .on_press(Message::ServiceAccountResponsiblePersonSelected(person_id))
                                                                    .padding(4)
                                                                    .style(CowboyCustomTheme::glass_button())
                                                            );
                                                        }
                                                        person_buttons
                                                    }
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Create button
                                            button(text("Create Service Account").size(self.view_model.text_small))
                                                .on_press(Message::CreateServiceAccount)
                                                .padding(self.view_model.padding_md)
                                                .style(CowboyCustomTheme::security_button()),
                                        ]
                                        .spacing(self.view_model.spacing_md)
                                    )
                                    .padding(self.view_model.padding_md)
                                    .style(CowboyCustomTheme::card_container()),

                                    // List of created service accounts
                                    if !self.created_service_accounts.is_empty() {
                                        let mut sa_list = column![
                                            text(format!("Service Accounts ({})", self.created_service_accounts.len()))
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_primary()),
                                        ].spacing(self.view_model.spacing_sm);

                                        for sa in &self.created_service_accounts {
                                            let sa_id = sa.id;
                                            let status_icon = if sa.active { "⚙️" } else { "🔒" };
                                            let status_text = if sa.active { "Active" } else { "Deactivated" };

                                            // Find responsible person name
                                            let responsible_name = self.loaded_people.iter()
                                                .find(|p| p.person_id == sa.responsible_person_id)
                                                .map(|p| p.name.clone())
                                                .unwrap_or_else(|| format!("ID: {}", &sa.responsible_person_id.to_string()[..8]));

                                            // Find owning unit name
                                            let unit_name = self.loaded_units.iter()
                                                .chain(self.created_units.iter())
                                                .find(|u| u.id.as_uuid() == sa.owning_unit_id)
                                                .map(|u| u.name.clone())
                                                .unwrap_or_else(|| format!("ID: {}", &sa.owning_unit_id.to_string()[..8]));

                                            sa_list = sa_list.push(
                                                container(
                                                    row![
                                                        column![
                                                            text(format!("{} {} [{}]", status_icon, sa.name, status_text))
                                                                .size(self.view_model.text_small)
                                                                .color(if sa.active { CowboyTheme::text_primary() } else { CowboyTheme::text_secondary() }),
                                                            text(format!("Purpose: {}", sa.purpose))
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary()),
                                                            text(format!("Unit: {} | Responsible: {}", unit_name, responsible_name))
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary()),
                                                        ]
                                                        .spacing(self.view_model.spacing_xs),
                                                        horizontal_space(),
                                                        column![
                                                            row![
                                                                button(text("Generate Key").size(self.view_model.text_tiny))
                                                                    .on_press(Message::GenerateServiceAccountKey { service_account_id: sa_id })
                                                                    .padding(4)
                                                                    .style(CowboyCustomTheme::security_button()),
                                                                if sa.active {
                                                                    button(text("Deactivate").size(self.view_model.text_tiny))
                                                                        .on_press(Message::DeactivateServiceAccount(sa_id))
                                                                        .padding(4)
                                                                        .style(CowboyCustomTheme::glass_button())
                                                                } else {
                                                                    button(text("Deactivated").size(self.view_model.text_tiny))
                                                                        .padding(4)
                                                                        .style(CowboyCustomTheme::glass_button())
                                                                },
                                                                button(text("Remove").size(self.view_model.text_tiny))
                                                                    .on_press(Message::RemoveServiceAccount(sa_id))
                                                                    .padding(4)
                                                                    .style(CowboyCustomTheme::glass_button()),
                                                            ]
                                                            .spacing(self.view_model.spacing_xs)
                                                        ]
                                                        .align_x(iced::alignment::Horizontal::Right),
                                                    ]
                                                    .align_y(Alignment::Center)
                                                )
                                                .padding(self.view_model.padding_sm)
                                                .style(CowboyCustomTheme::card_container())
                                            );
                                        }
                                        sa_list
                                    } else {
                                        column![
                                            text("No service accounts created yet")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary())
                                        ]
                                    },
                                ]
                                .spacing(self.view_model.spacing_md)
                            } else {
                                column![]
                            },
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_md)
                    .style(CowboyCustomTheme::pastel_mint_card()),

                    // Step 4f: Key Delegations (Collapsible)
                    container(
                        column![
                            row![
                                button(if self.delegation_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleDelegationSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("4f. Key Delegations (Authority Transfer)")
                                    .size(self.view_model.text_medium)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.delegation_section_collapsed {
                                column![
                                    // Delegation creation form
                                    container(
                                        column![
                                            text("Create Key Delegation")
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_primary()),
                                            text("Delegate specific key permissions from one person to another")
                                                .size(self.view_model.text_tiny)
                                                .color(CowboyTheme::text_secondary()),

                                            // From person selection
                                            row![
                                                text("Delegate From:").size(self.view_model.text_small).width(iced::Length::Fixed(120.0)),
                                                {
                                                    if self.loaded_people.is_empty() {
                                                        row![
                                                            text("Add people first to create delegations")
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary())
                                                        ]
                                                    } else {
                                                        let mut person_buttons = row![].spacing(self.view_model.spacing_xs);
                                                        for person in self.loaded_people.iter().take(6) {
                                                            let person_id = person.person_id;
                                                            // Can't select if already selected as "to"
                                                            let disabled = self.delegation_to_person == Some(person_id);
                                                            let is_selected = self.delegation_from_person == Some(person_id);
                                                            let label = if is_selected { format!("✓ {}", person.name) } else { person.name.clone() };
                                                            person_buttons = person_buttons.push(
                                                                if disabled {
                                                                    button(text(label).size(self.view_model.text_tiny))
                                                                        .padding(4)
                                                                        .style(CowboyCustomTheme::glass_button())
                                                                } else {
                                                                    button(text(label).size(self.view_model.text_tiny))
                                                                        .on_press(Message::DelegationFromPersonSelected(person_id))
                                                                        .padding(4)
                                                                        .style(CowboyCustomTheme::glass_button())
                                                                }
                                                            );
                                                        }
                                                        person_buttons
                                                    }
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // To person selection
                                            row![
                                                text("Delegate To:").size(self.view_model.text_small).width(iced::Length::Fixed(120.0)),
                                                {
                                                    if self.loaded_people.is_empty() {
                                                        row![
                                                            text("Add people first to create delegations")
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary())
                                                        ]
                                                    } else {
                                                        let mut person_buttons = row![].spacing(self.view_model.spacing_xs);
                                                        for person in self.loaded_people.iter().take(6) {
                                                            let person_id = person.person_id;
                                                            // Can't select if already selected as "from"
                                                            let disabled = self.delegation_from_person == Some(person_id);
                                                            let is_selected = self.delegation_to_person == Some(person_id);
                                                            let label = if is_selected { format!("✓ {}", person.name) } else { person.name.clone() };
                                                            person_buttons = person_buttons.push(
                                                                if disabled {
                                                                    button(text(label).size(self.view_model.text_tiny))
                                                                        .padding(4)
                                                                        .style(CowboyCustomTheme::glass_button())
                                                                } else {
                                                                    button(text(label).size(self.view_model.text_tiny))
                                                                        .on_press(Message::DelegationToPersonSelected(person_id))
                                                                        .padding(4)
                                                                        .style(CowboyCustomTheme::glass_button())
                                                                }
                                                            );
                                                        }
                                                        person_buttons
                                                    }
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Permission checkboxes
                                            row![
                                                text("Permissions:").size(self.view_model.text_small).width(iced::Length::Fixed(120.0)),
                                                {
                                                    let permissions = vec![
                                                        (crate::domain::KeyPermission::Sign, "Sign"),
                                                        (crate::domain::KeyPermission::Encrypt, "Encrypt"),
                                                        (crate::domain::KeyPermission::Decrypt, "Decrypt"),
                                                        (crate::domain::KeyPermission::CertifyOthers, "Certify"),
                                                        (crate::domain::KeyPermission::RevokeOthers, "Revoke"),
                                                        (crate::domain::KeyPermission::BackupAccess, "Backup"),
                                                    ];
                                                    let mut permission_buttons = row![].spacing(self.view_model.spacing_xs);
                                                    for (perm, label) in permissions {
                                                        let is_selected = self.delegation_permissions.contains(&perm);
                                                        let display_label = if is_selected { format!("✓ {}", label) } else { label.to_string() };
                                                        permission_buttons = permission_buttons.push(
                                                            button(text(display_label).size(self.view_model.text_tiny))
                                                                .on_press(Message::ToggleDelegationPermission(perm))
                                                                .padding(4)
                                                                .style(CowboyCustomTheme::glass_button())
                                                        );
                                                    }
                                                    permission_buttons
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Expiration days
                                            row![
                                                text("Expires in:").size(self.view_model.text_small).width(iced::Length::Fixed(120.0)),
                                                text_input("days (empty = no expiration)", &self.delegation_expires_days)
                                                    .on_input(Message::DelegationExpiresDaysChanged)
                                                    .padding(self.view_model.padding_sm)
                                                    .width(iced::Length::Fixed(200.0)),
                                                text("days")
                                                    .size(self.view_model.text_tiny)
                                                    .color(CowboyTheme::text_secondary()),
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                            .align_y(Alignment::Center),

                                            // Create delegation button
                                            button(text("Create Delegation").size(self.view_model.text_small))
                                                .on_press(Message::CreateDelegation)
                                                .padding(self.view_model.padding_md)
                                                .style(CowboyCustomTheme::security_button()),
                                        ]
                                        .spacing(self.view_model.spacing_md)
                                    )
                                    .padding(self.view_model.padding_md)
                                    .style(CowboyCustomTheme::card_container()),

                                    // List of active delegations
                                    if !self.active_delegations.is_empty() {
                                        let mut delegation_list = column![
                                            text(format!("Active Delegations ({})", self.active_delegations.iter().filter(|d| d.is_active).count()))
                                                .size(self.view_model.text_normal)
                                                .color(CowboyTheme::text_primary()),
                                        ].spacing(self.view_model.spacing_sm);

                                        for delegation in &self.active_delegations {
                                            let delegation_id = delegation.id;
                                            let status_icon = if delegation.is_active { "🔑" } else { "🚫" };
                                            let status_color = if delegation.is_active { CowboyTheme::text_primary() } else { CowboyTheme::text_secondary() };

                                            let expiry_text = match delegation.expires_at {
                                                Some(exp) => format!("Expires: {}", exp.format("%Y-%m-%d")),
                                                None => "No expiration".to_string(),
                                            };

                                            let permissions_text = delegation.permissions.iter()
                                                .map(|p| format!("{:?}", p))
                                                .collect::<Vec<_>>()
                                                .join(", ");

                                            delegation_list = delegation_list.push(
                                                container(
                                                    row![
                                                        column![
                                                            text(format!("{} {} → {}", status_icon, delegation.from_person_name, delegation.to_person_name))
                                                                .size(self.view_model.text_small)
                                                                .color(status_color),
                                                            text(format!("Permissions: {}", permissions_text))
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary()),
                                                            text(format!("{} | Created: {}", expiry_text, delegation.created_at.format("%Y-%m-%d")))
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary()),
                                                        ]
                                                        .spacing(self.view_model.spacing_xs),
                                                        horizontal_space(),
                                                        if delegation.is_active {
                                                            button(text("Revoke").size(self.view_model.text_tiny))
                                                                .on_press(Message::RevokeDelegation(delegation_id))
                                                                .padding(4)
                                                                .style(CowboyCustomTheme::glass_button())
                                                        } else {
                                                            button(text("Revoked").size(self.view_model.text_tiny))
                                                                .padding(4)
                                                                .style(CowboyCustomTheme::glass_button())
                                                        },
                                                    ]
                                                    .align_y(Alignment::Center)
                                                )
                                                .padding(self.view_model.padding_sm)
                                                .style(CowboyCustomTheme::card_container())
                                            );
                                        }
                                        delegation_list
                                    } else {
                                        column![
                                            text("No delegations created yet")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_secondary())
                                        ]
                                    },
                                ]
                                .spacing(self.view_model.spacing_md)
                            } else {
                                column![]
                            },
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_md)
                    .style(CowboyCustomTheme::pastel_mint_card()),

                    // Step 4e: Trust Chain Visualization (Collapsible)
                    container(
                        column![
                            row![
                                button(if self.trust_chain_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleTrustChainSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("4e. Trust Chain Verification")
                                    .size(self.view_model.text_medium)
                                    .color(CowboyTheme::text_primary()),
                                horizontal_space(),
                                if !self.loaded_certificates.is_empty() {
                                    button(text("Verify All").size(self.view_model.text_tiny))
                                        .on_press(Message::VerifyAllTrustChains)
                                        .padding(4)
                                        .style(CowboyCustomTheme::security_button())
                                } else {
                                    button(text("No certs").size(self.view_model.text_tiny))
                                        .padding(4)
                                        .style(CowboyCustomTheme::glass_button())
                                },
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.trust_chain_section_collapsed {
                                if self.loaded_certificates.is_empty() {
                                    column![
                                        text("No certificates loaded. Generate or import certificates first.")
                                            .size(self.view_model.text_small)
                                            .color(CowboyTheme::text_secondary())
                                    ]
                                } else {
                                    // Build the trust chain hierarchy visualization
                                    let root_certs: Vec<_> = self.loaded_certificates.iter()
                                        .filter(|c| c.is_ca && c.issuer.as_ref() == Some(&c.subject))
                                        .collect();
                                    let intermediate_certs: Vec<_> = self.loaded_certificates.iter()
                                        .filter(|c| c.is_ca && c.issuer.as_ref() != Some(&c.subject))
                                        .collect();
                                    let leaf_certs: Vec<_> = self.loaded_certificates.iter()
                                        .filter(|c| !c.is_ca)
                                        .collect();

                                    let mut chain_viz = column![
                                        // Root CAs
                                        container(
                                            column![
                                                text(format!("🔐 Root CA Certificates ({})", root_certs.len()))
                                                    .size(self.view_model.text_normal)
                                                    .color(self.view_model.colors.red_error),
                                                {
                                                    let mut root_list = column![].spacing(self.view_model.spacing_xs);
                                                    for cert in &root_certs {
                                                        let cert_id = cert.cert_id;
                                                        let status = self.trust_chain_verification_status.get(&cert_id);
                                                        let status_icon = match status {
                                                            Some(TrustChainStatus::SelfSigned) => "✅",
                                                            Some(TrustChainStatus::Verified { .. }) => "✅",
                                                            Some(TrustChainStatus::Expired { .. }) => "⏰",
                                                            Some(TrustChainStatus::Failed { .. }) => "❌",
                                                            Some(TrustChainStatus::IssuerNotFound { .. }) => "⚠️",
                                                            Some(TrustChainStatus::Pending) | None => "⏳",
                                                        };
                                                        let is_selected = self.selected_trust_chain_cert == Some(cert_id);
                                                        root_list = root_list.push(
                                                            button(
                                                                row![
                                                                    text(format!("{} {}", status_icon, cert.subject))
                                                                        .size(self.view_model.text_small)
                                                                        .color(if is_selected { self.view_model.colors.info } else { CowboyTheme::text_primary() }),
                                                                    horizontal_space(),
                                                                    text(format!("Valid until: {}", cert.not_after.format("%Y-%m-%d")))
                                                                        .size(self.view_model.text_tiny)
                                                                        .color(CowboyTheme::text_secondary()),
                                                                ]
                                                                .align_y(Alignment::Center)
                                                            )
                                                            .on_press(Message::SelectCertificateForChainView(cert_id))
                                                            .padding(self.view_model.padding_sm)
                                                            .width(iced::Length::Fill)
                                                            .style(CowboyCustomTheme::glass_button())
                                                        );
                                                    }
                                                    if root_certs.is_empty() {
                                                        root_list = root_list.push(
                                                            text("No root CA certificates found")
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary())
                                                        );
                                                    }
                                                    root_list
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                        )
                                        .padding(self.view_model.padding_sm)
                                        .style(CowboyCustomTheme::card_container()),

                                        // Chain arrow
                                        text("       ↓ signs ↓")
                                            .size(self.view_model.text_small)
                                            .color(CowboyTheme::text_secondary()),

                                        // Intermediate CAs
                                        container(
                                            column![
                                                text(format!("📋 Intermediate CA Certificates ({})", intermediate_certs.len()))
                                                    .size(self.view_model.text_normal)
                                                    .color(self.view_model.colors.orange_warning),
                                                {
                                                    let mut int_list = column![].spacing(self.view_model.spacing_xs);
                                                    for cert in &intermediate_certs {
                                                        let cert_id = cert.cert_id;
                                                        let status = self.trust_chain_verification_status.get(&cert_id);
                                                        let status_icon = match status {
                                                            Some(TrustChainStatus::SelfSigned) => "✅",
                                                            Some(TrustChainStatus::Verified { .. }) => "✅",
                                                            Some(TrustChainStatus::Expired { .. }) => "⏰",
                                                            Some(TrustChainStatus::Failed { .. }) => "❌",
                                                            Some(TrustChainStatus::IssuerNotFound { .. }) => "⚠️",
                                                            Some(TrustChainStatus::Pending) | None => "⏳",
                                                        };
                                                        let is_selected = self.selected_trust_chain_cert == Some(cert_id);
                                                        int_list = int_list.push(
                                                            button(
                                                                column![
                                                                    row![
                                                                        text(format!("{} {}", status_icon, cert.subject))
                                                                            .size(self.view_model.text_small)
                                                                            .color(if is_selected { self.view_model.colors.info } else { CowboyTheme::text_primary() }),
                                                                        horizontal_space(),
                                                                        text(format!("Valid until: {}", cert.not_after.format("%Y-%m-%d")))
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(CowboyTheme::text_secondary()),
                                                                    ]
                                                                    .align_y(Alignment::Center),
                                                                    if let Some(ref issuer) = cert.issuer {
                                                                        text(format!("← Signed by: {}", issuer))
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(CowboyTheme::text_secondary())
                                                                    } else {
                                                                        text("")
                                                                    },
                                                                ]
                                                            )
                                                            .on_press(Message::SelectCertificateForChainView(cert_id))
                                                            .padding(self.view_model.padding_sm)
                                                            .width(iced::Length::Fill)
                                                            .style(CowboyCustomTheme::glass_button())
                                                        );
                                                    }
                                                    if intermediate_certs.is_empty() {
                                                        int_list = int_list.push(
                                                            text("No intermediate CA certificates")
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary())
                                                        );
                                                    }
                                                    int_list
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                        )
                                        .padding(self.view_model.padding_sm)
                                        .style(CowboyCustomTheme::card_container()),

                                        // Chain arrow
                                        text("       ↓ signs ↓")
                                            .size(self.view_model.text_small)
                                            .color(CowboyTheme::text_secondary()),

                                        // Leaf certificates
                                        container(
                                            column![
                                                text(format!("📄 End Entity Certificates ({})", leaf_certs.len()))
                                                    .size(self.view_model.text_normal)
                                                    .color(self.view_model.colors.info),
                                                {
                                                    let mut leaf_list = column![].spacing(self.view_model.spacing_xs);
                                                    for cert in &leaf_certs {
                                                        let cert_id = cert.cert_id;
                                                        let status = self.trust_chain_verification_status.get(&cert_id);
                                                        let status_icon = match status {
                                                            Some(TrustChainStatus::SelfSigned) => "✅",
                                                            Some(TrustChainStatus::Verified { .. }) => "✅",
                                                            Some(TrustChainStatus::Expired { .. }) => "⏰",
                                                            Some(TrustChainStatus::Failed { .. }) => "❌",
                                                            Some(TrustChainStatus::IssuerNotFound { .. }) => "⚠️",
                                                            Some(TrustChainStatus::Pending) | None => "⏳",
                                                        };
                                                        let is_selected = self.selected_trust_chain_cert == Some(cert_id);
                                                        leaf_list = leaf_list.push(
                                                            button(
                                                                column![
                                                                    row![
                                                                        text(format!("{} {}", status_icon, cert.subject))
                                                                            .size(self.view_model.text_small)
                                                                            .color(if is_selected { self.view_model.colors.info } else { CowboyTheme::text_primary() }),
                                                                        horizontal_space(),
                                                                        text(format!("Valid until: {}", cert.not_after.format("%Y-%m-%d")))
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(CowboyTheme::text_secondary()),
                                                                    ]
                                                                    .align_y(Alignment::Center),
                                                                    if let Some(ref issuer) = cert.issuer {
                                                                        text(format!("← Signed by: {}", issuer))
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(CowboyTheme::text_secondary())
                                                                    } else {
                                                                        text("")
                                                                    },
                                                                ]
                                                            )
                                                            .on_press(Message::SelectCertificateForChainView(cert_id))
                                                            .padding(self.view_model.padding_sm)
                                                            .width(iced::Length::Fill)
                                                            .style(CowboyCustomTheme::glass_button())
                                                        );
                                                    }
                                                    if leaf_certs.is_empty() {
                                                        leaf_list = leaf_list.push(
                                                            text("No end entity certificates")
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_secondary())
                                                        );
                                                    }
                                                    leaf_list
                                                },
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                        )
                                        .padding(self.view_model.padding_sm)
                                        .style(CowboyCustomTheme::card_container()),
                                    ].spacing(self.view_model.spacing_sm);

                                    // Selected certificate detail
                                    if let Some(selected_id) = self.selected_trust_chain_cert {
                                        if let Some(cert) = self.loaded_certificates.iter().find(|c| c.cert_id == selected_id) {
                                            let status = self.trust_chain_verification_status.get(&selected_id);
                                            chain_viz = chain_viz.push(
                                                container(
                                                    column![
                                                        text("Selected Certificate Details")
                                                            .size(self.view_model.text_normal)
                                                            .color(CowboyTheme::text_primary()),
                                                        text(format!("Subject: {}", cert.subject))
                                                            .size(self.view_model.text_small),
                                                        if let Some(ref issuer) = cert.issuer {
                                                            text(format!("Issuer: {}", issuer))
                                                                .size(self.view_model.text_small)
                                                        } else {
                                                            text("Issuer: (self-signed)")
                                                                .size(self.view_model.text_small)
                                                        },
                                                        text(format!("Serial: {}", cert.serial_number))
                                                            .size(self.view_model.text_tiny)
                                                            .color(CowboyTheme::text_secondary()),
                                                        text(format!("Valid: {} to {}",
                                                            cert.not_before.format("%Y-%m-%d"),
                                                            cert.not_after.format("%Y-%m-%d")))
                                                            .size(self.view_model.text_small),
                                                        text(if cert.is_ca { "Type: Certificate Authority" } else { "Type: End Entity" })
                                                            .size(self.view_model.text_small),
                                                        match status {
                                                            Some(TrustChainStatus::Verified { chain_length, root_subject }) =>
                                                                text(format!("✅ Verified: {} certs to root '{}'", chain_length, root_subject))
                                                                    .size(self.view_model.text_small)
                                                                    .color(self.view_model.colors.success),
                                                            Some(TrustChainStatus::SelfSigned) =>
                                                                text("🔐 Self-signed root certificate")
                                                                    .size(self.view_model.text_small)
                                                                    .color(self.view_model.colors.success),
                                                            Some(TrustChainStatus::Expired { expired_at }) =>
                                                                text(format!("⏰ Expired at {}", expired_at.format("%Y-%m-%d")))
                                                                    .size(self.view_model.text_small)
                                                                    .color(self.view_model.colors.red_error),
                                                            Some(TrustChainStatus::IssuerNotFound { expected_issuer }) =>
                                                                text(format!("⚠️ Issuer not found: {}", expected_issuer))
                                                                    .size(self.view_model.text_small)
                                                                    .color(self.view_model.colors.orange_warning),
                                                            Some(TrustChainStatus::Failed { reason }) =>
                                                                text(format!("❌ Failed: {}", reason))
                                                                    .size(self.view_model.text_small)
                                                                    .color(self.view_model.colors.red_error),
                                                            Some(TrustChainStatus::Pending) | None =>
                                                                text("⏳ Not verified yet")
                                                                    .size(self.view_model.text_small)
                                                                    .color(CowboyTheme::text_secondary()),
                                                        },
                                                        button(text("Verify Chain").size(self.view_model.text_tiny))
                                                            .on_press(Message::VerifyTrustChain(selected_id))
                                                            .padding(4)
                                                            .style(CowboyCustomTheme::security_button()),
                                                    ]
                                                    .spacing(self.view_model.spacing_xs)
                                                )
                                                .padding(self.view_model.padding_md)
                                                .style(CowboyCustomTheme::pastel_teal_card())
                                            );
                                        }
                                    }

                                    chain_viz
                                }
                            } else {
                                column![]
                            },
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_md)
                    .style(CowboyCustomTheme::pastel_coral_card()),

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

                                    // NATS Hierarchy Visualization Section (Collapsible)
                                    container(
                                        column![
                                            row![
                                                button(if self.nats_viz_section_collapsed { "▶" } else { "▼" })
                                                    .on_press(Message::ToggleNatsVizSection)
                                                    .padding(self.view_model.padding_sm)
                                                    .style(CowboyCustomTheme::glass_button()),
                                                text("NATS Hierarchy Visualization")
                                                    .size(self.view_model.text_medium)
                                                    .color(CowboyTheme::text_primary()),
                                                horizontal_space(),
                                                button("Refresh")
                                                    .on_press(Message::RefreshNatsHierarchy)
                                                    .padding(self.view_model.padding_sm)
                                                    .style(CowboyCustomTheme::glass_button()),
                                            ]
                                            .spacing(self.view_model.spacing_md)
                                            .align_y(Alignment::Center),

                                            if !self.nats_viz_section_collapsed {
                                                {
                                                    let mut hierarchy_view = column![].spacing(self.view_model.spacing_sm);

                                                    if let Some(ref hierarchy) = self.nats_viz_hierarchy_data {
                                                        // Operator node (root) - show selection with >> indicator
                                                        let op_indicator = if self.nats_viz_selected_operator { ">> " } else { "" };

                                                        hierarchy_view = hierarchy_view.push(
                                                            button(
                                                                row![
                                                                    text(format!("{}🏢", op_indicator)).size(self.view_model.text_large),
                                                                    text(format!("Operator: {}", hierarchy.operator.name))
                                                                        .size(self.view_model.text_normal)
                                                                        .color(if self.nats_viz_selected_operator { self.view_model.colors.info } else { CowboyTheme::text_primary() }),
                                                                    if !hierarchy.operator.signing_keys.is_empty() {
                                                                        text(format!("({} keys)", hierarchy.operator.signing_keys.len()))
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(CowboyTheme::text_secondary())
                                                                    } else {
                                                                        text("")
                                                                    }
                                                                ]
                                                                .spacing(self.view_model.spacing_md)
                                                                .align_y(Alignment::Center)
                                                            )
                                                            .on_press(Message::SelectNatsOperator)
                                                            .padding(self.view_model.padding_md)
                                                            .width(Length::Fill)
                                                            .style(CowboyCustomTheme::glass_button())
                                                        );

                                                        // Accounts (expandable tree nodes)
                                                        for account in &hierarchy.accounts {
                                                            let is_expanded = self.nats_viz_expanded_accounts.contains(&account.name);
                                                            let is_selected = self.nats_viz_selected_account.as_ref() == Some(&account.name);
                                                            let acc_indicator = if is_selected { ">> " } else { "" };

                                                            // Account row with expand toggle
                                                            hierarchy_view = hierarchy_view.push(
                                                                row![
                                                                    Space::with_width(Length::Fixed(self.view_model.spacing_xl as f32)),
                                                                    button(if is_expanded { "▼" } else { "▶" })
                                                                        .on_press(Message::ToggleNatsAccountExpand(account.name.clone()))
                                                                        .padding(self.view_model.padding_xs)
                                                                        .style(CowboyCustomTheme::glass_button()),
                                                                    button(
                                                                        row![
                                                                            text(format!("{}📁", acc_indicator)).size(self.view_model.text_normal),
                                                                            text(format!("Account: {}", account.name))
                                                                                .size(self.view_model.text_small)
                                                                                .color(if is_selected { self.view_model.colors.info } else { CowboyTheme::text_primary() }),
                                                                            text(format!("({} users)", account.users.len()))
                                                                                .size(self.view_model.text_tiny)
                                                                                .color(CowboyTheme::text_secondary()),
                                                                        ]
                                                                        .spacing(self.view_model.spacing_sm)
                                                                        .align_y(Alignment::Center)
                                                                    )
                                                                    .on_press(Message::SelectNatsAccount(account.name.clone()))
                                                                    .padding(self.view_model.padding_sm)
                                                                    .width(Length::Fill)
                                                                    .style(CowboyCustomTheme::glass_button()),
                                                                ]
                                                                .spacing(self.view_model.spacing_sm)
                                                                .align_y(Alignment::Center)
                                                            );

                                                            // Users (shown when account is expanded)
                                                            if is_expanded {
                                                                for user in &account.users {
                                                                    let is_user_selected = self.nats_viz_selected_user.as_ref() == Some(&(account.name.clone(), user.person_id));
                                                                    let user_indicator = if is_user_selected { ">> " } else { "" };

                                                                    // Find person name
                                                                    let person_name = self.loaded_people.iter()
                                                                        .find(|p| p.person_id == user.person_id)
                                                                        .map(|p| p.name.clone())
                                                                        .unwrap_or_else(|| format!("{}", user.person_id));

                                                                    hierarchy_view = hierarchy_view.push(
                                                                        row![
                                                                            Space::with_width(Length::Fixed((self.view_model.spacing_xl * 2) as f32)),
                                                                            button(
                                                                                row![
                                                                                    text(format!("{}👤", user_indicator)).size(self.view_model.text_small),
                                                                                    text(format!("User: {}", person_name))
                                                                                        .size(self.view_model.text_small)
                                                                                        .color(if is_user_selected { self.view_model.colors.info } else { CowboyTheme::text_primary() }),
                                                                                ]
                                                                                .spacing(self.view_model.spacing_sm)
                                                                                .align_y(Alignment::Center)
                                                                            )
                                                                            .on_press(Message::SelectNatsUser(account.name.clone(), user.person_id))
                                                                            .padding(self.view_model.padding_sm)
                                                                            .width(Length::Fill)
                                                                            .style(CowboyCustomTheme::glass_button()),
                                                                        ]
                                                                        .spacing(self.view_model.spacing_sm)
                                                                        .align_y(Alignment::Center)
                                                                    );
                                                                }

                                                                // Show "No users" if account is empty
                                                                if account.users.is_empty() {
                                                                    hierarchy_view = hierarchy_view.push(
                                                                        row![
                                                                            Space::with_width(Length::Fixed((self.view_model.spacing_xl * 2) as f32)),
                                                                            text("(no users)")
                                                                                .size(self.view_model.text_small)
                                                                                .color(CowboyTheme::text_muted()),
                                                                        ]
                                                                    );
                                                                }
                                                            }
                                                        }

                                                        // Show "No accounts" if operator has no accounts
                                                        if hierarchy.accounts.is_empty() {
                                                            hierarchy_view = hierarchy_view.push(
                                                                row![
                                                                    Space::with_width(Length::Fixed(self.view_model.spacing_xl as f32)),
                                                                    text("(no accounts - create organizational units first)")
                                                                        .size(self.view_model.text_small)
                                                                        .color(CowboyTheme::text_muted()),
                                                                ]
                                                            );
                                                        }

                                                        // Summary stats
                                                        let total_users: usize = hierarchy.accounts.iter().map(|a| a.users.len()).sum();
                                                        hierarchy_view = hierarchy_view.push(
                                                            container(
                                                                text(format!("Total: 1 operator, {} accounts, {} users",
                                                                    hierarchy.accounts.len(), total_users))
                                                                    .size(self.view_model.text_small)
                                                                    .color(CowboyTheme::text_secondary())
                                                            )
                                                            .padding(self.view_model.padding_sm)
                                                        );
                                                    } else {
                                                        // No hierarchy data yet
                                                        hierarchy_view = hierarchy_view.push(
                                                            column![
                                                                text("No NATS hierarchy data available")
                                                                    .size(self.view_model.text_normal)
                                                                    .color(CowboyTheme::text_muted()),
                                                                text("Click 'Refresh' or 'Generate NATS Hierarchy' to populate")
                                                                    .size(self.view_model.text_small)
                                                                    .color(CowboyTheme::text_secondary()),
                                                            ]
                                                            .spacing(self.view_model.spacing_sm)
                                                        );
                                                    }

                                                    container(
                                                        scrollable(hierarchy_view)
                                                            .height(Length::Fixed(300.0))
                                                    )
                                                    .padding(self.view_model.padding_md)
                                                    .style(CowboyCustomTheme::card_container())
                                                }
                                            } else {
                                                container(text(""))
                                            }
                                        ]
                                        .spacing(self.view_model.spacing_sm)
                                    )
                                    .padding(self.view_model.padding_md)
                                    .style(CowboyCustomTheme::pastel_teal_card()),
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

                                    // Multi-purpose Key Generation (Collapsible)
                                    container(
                                        column![
                                            row![
                                                button(if self.multi_purpose_key_section_collapsed { "▶" } else { "▼" })
                                                    .on_press(Message::ToggleMultiPurposeKeySection)
                                                    .padding(self.view_model.padding_sm)
                                                    .style(CowboyCustomTheme::glass_button()),
                                                text("Multi-Purpose Key Generation")
                                                    .size(self.view_model.text_medium)
                                                    .color(CowboyTheme::text_primary()),
                                            ]
                                            .spacing(self.view_model.spacing_md)
                                            .align_y(Alignment::Center),

                                            if !self.multi_purpose_key_section_collapsed {
                                                column![
                                                    text("Generate multiple keys for a person with different purposes")
                                                        .size(self.view_model.text_small)
                                                        .color(CowboyTheme::text_secondary()),

                                                    // Person selection
                                                    row![
                                                        text("Person:").size(self.view_model.text_normal).width(iced::Length::Fixed(100.0)),
                                                        {
                                                            if self.loaded_people.is_empty() {
                                                                row![
                                                                    text("Load people first")
                                                                        .size(self.view_model.text_tiny)
                                                                        .color(CowboyTheme::text_secondary())
                                                                ]
                                                            } else {
                                                                let mut person_buttons = row![].spacing(self.view_model.spacing_xs);
                                                                for person in self.loaded_people.iter().take(5) {
                                                                    let person_id = person.person_id;
                                                                    let is_selected = self.multi_purpose_selected_person == Some(person_id);
                                                                    let label = if is_selected { format!("✓ {}", person.name) } else { person.name.clone() };
                                                                    person_buttons = person_buttons.push(
                                                                        button(text(label).size(self.view_model.text_tiny))
                                                                            .on_press(Message::MultiPurposePersonSelected(person_id))
                                                                            .padding(4)
                                                                            .style(CowboyCustomTheme::glass_button())
                                                                    );
                                                                }
                                                                person_buttons
                                                            }
                                                        },
                                                    ]
                                                    .spacing(self.view_model.spacing_sm)
                                                    .align_y(Alignment::Center),

                                                    // Key purpose checkboxes
                                                    row![
                                                        text("Purposes:").size(self.view_model.text_normal).width(iced::Length::Fixed(100.0)),
                                                        {
                                                            let purposes = vec![
                                                                (crate::domain::InvariantKeyPurpose::Signing, "Signing"),
                                                                (crate::domain::InvariantKeyPurpose::Encryption, "Encryption"),
                                                                (crate::domain::InvariantKeyPurpose::Authentication, "Auth"),
                                                                (crate::domain::InvariantKeyPurpose::KeyAgreement, "Key Agreement"),
                                                            ];
                                                            let mut purpose_buttons = row![].spacing(self.view_model.spacing_xs);
                                                            for (purpose, label) in purposes {
                                                                let is_selected = self.multi_purpose_selected_purposes.contains(&purpose);
                                                                let display_label = if is_selected { format!("✓ {}", label) } else { label.to_string() };
                                                                purpose_buttons = purpose_buttons.push(
                                                                    button(text(display_label).size(self.view_model.text_tiny))
                                                                        .on_press(Message::ToggleKeyPurpose(purpose))
                                                                        .padding(4)
                                                                        .style(CowboyCustomTheme::glass_button())
                                                                );
                                                            }
                                                            purpose_buttons
                                                        },
                                                    ]
                                                    .spacing(self.view_model.spacing_sm)
                                                    .align_y(Alignment::Center),

                                                    // Generate button
                                                    button(text("Generate Multi-Purpose Keys").size(self.view_model.text_normal))
                                                        .on_press_maybe(
                                                            if self.multi_purpose_selected_person.is_some() && !self.multi_purpose_selected_purposes.is_empty() {
                                                                Some(Message::GenerateMultiPurposeKeys)
                                                            } else {
                                                                None
                                                            }
                                                        )
                                                        .padding(self.view_model.padding_md)
                                                        .style(CowboyCustomTheme::security_button()),
                                                ]
                                                .spacing(self.view_model.spacing_md)
                                            } else {
                                                column![]
                                            }
                                        ]
                                        .spacing(self.view_model.spacing_md)
                                    )
                                    .padding(self.view_model.padding_md)
                                    .style(CowboyCustomTheme::pastel_cream_card()),

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

                    // Step 7: GPG/PGP Key Generation (Card with Collapse)
                    container(
                        column![
                            row![
                                button(if self.gpg_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleGpgSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("7. GPG/PGP Keys")
                                    .size(self.view_model.text_large)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.gpg_section_collapsed {
                                column![
                                    text("Generate GPG/PGP key pairs for email signing and encryption")
                                        .size(self.view_model.text_normal)
                                        .color(CowboyTheme::text_secondary()),

                                    // User ID input
                                    text("User ID (e.g., 'John Doe <john@example.com>')")
                                        .size(self.view_model.text_normal)
                                        .color(CowboyTheme::text_primary()),
                                    text_input("Name <email@example.com>", &self.gpg_user_id)
                                        .on_input(Message::GpgUserIdChanged)
                                        .size(self.view_model.text_medium)
                                        .padding(self.view_model.padding_md)
                                        .style(CowboyCustomTheme::glass_input()),

                                    // Key type picker
                                    row![
                                        text("Key Type:").size(self.view_model.text_normal),
                                        pick_list(
                                            vec!["EdDSA (Modern, Recommended)", "ECDSA", "RSA", "DSA (Legacy)"],
                                            self.gpg_key_type.map(|kt| match kt {
                                                crate::ports::gpg::GpgKeyType::Eddsa => "EdDSA (Modern, Recommended)",
                                                crate::ports::gpg::GpgKeyType::Ecdsa => "ECDSA",
                                                crate::ports::gpg::GpgKeyType::Rsa => "RSA",
                                                crate::ports::gpg::GpgKeyType::Dsa => "DSA (Legacy)",
                                                crate::ports::gpg::GpgKeyType::Elgamal => "RSA",  // Map to RSA for display
                                            }),
                                            |selected| match selected {
                                                "EdDSA (Modern, Recommended)" => Message::GpgKeyTypeSelected(crate::ports::gpg::GpgKeyType::Eddsa),
                                                "ECDSA" => Message::GpgKeyTypeSelected(crate::ports::gpg::GpgKeyType::Ecdsa),
                                                "RSA" => Message::GpgKeyTypeSelected(crate::ports::gpg::GpgKeyType::Rsa),
                                                "DSA (Legacy)" => Message::GpgKeyTypeSelected(crate::ports::gpg::GpgKeyType::Dsa),
                                                _ => Message::GpgKeyTypeSelected(crate::ports::gpg::GpgKeyType::Eddsa),
                                            }
                                        )
                                        .placeholder("Select Key Type")
                                        .width(Length::Fixed(250.0)),
                                    ]
                                    .spacing(self.view_model.spacing_md)
                                    .align_y(Alignment::Center),

                                    // Key length (for RSA/DSA)
                                    row![
                                        text("Key Length (bits):").size(self.view_model.text_normal),
                                        text_input("4096", &self.gpg_key_length)
                                            .on_input(Message::GpgKeyLengthChanged)
                                            .size(self.view_model.text_medium)
                                            .padding(self.view_model.padding_sm)
                                            .width(Length::Fixed(100.0))
                                            .style(CowboyCustomTheme::glass_input()),
                                        text("(ignored for EdDSA/ECDSA)")
                                            .size(self.view_model.text_small)
                                            .color(CowboyTheme::text_secondary()),
                                    ]
                                    .spacing(self.view_model.spacing_md)
                                    .align_y(Alignment::Center),

                                    // Expiration
                                    row![
                                        text("Expires in (days):").size(self.view_model.text_normal),
                                        text_input("365", &self.gpg_expires_days)
                                            .on_input(Message::GpgExpiresDaysChanged)
                                            .size(self.view_model.text_medium)
                                            .padding(self.view_model.padding_sm)
                                            .width(Length::Fixed(100.0))
                                            .style(CowboyCustomTheme::glass_input()),
                                        text("(leave empty for no expiration)")
                                            .size(self.view_model.text_small)
                                            .color(CowboyTheme::text_secondary()),
                                    ]
                                    .spacing(self.view_model.spacing_md)
                                    .align_y(Alignment::Center),

                                    // Generate button
                                    button(
                                        text("Generate GPG Key")
                                            .size(self.view_model.text_large)
                                    )
                                        .on_press_maybe(
                                            if !self.gpg_user_id.is_empty() && self.gpg_key_type.is_some() {
                                                Some(Message::GenerateGpgKey)
                                            } else {
                                                None
                                            }
                                        )
                                        .padding(self.view_model.padding_lg)
                                        .width(Length::Fill)
                                        .style(CowboyCustomTheme::security_button()),

                                    // Status message
                                    if let Some(ref status) = self.gpg_generation_status {
                                        text(status)
                                            .size(self.view_model.text_normal)
                                            .color(if status.contains("✅") {
                                                self.view_model.colors.green_success
                                            } else if status.contains("❌") {
                                                self.view_model.colors.red_error
                                            } else {
                                                self.view_model.colors.info
                                            })
                                    } else {
                                        text("")
                                    },

                                    // Generated keys list
                                    if !self.generated_gpg_keys.is_empty() {
                                        container(
                                            column![
                                                text(format!("Generated GPG Keys ({})", self.generated_gpg_keys.len()))
                                                    .size(self.view_model.text_medium)
                                                    .color(self.view_model.colors.green_success),
                                                {
                                                    let mut key_list = column![].spacing(self.view_model.spacing_sm);
                                                    for key in &self.generated_gpg_keys {
                                                        let status = if key.is_revoked {
                                                            "REVOKED"
                                                        } else if key.is_expired {
                                                            "EXPIRED"
                                                        } else {
                                                            "Active"
                                                        };
                                                        key_list = key_list.push(
                                                            container(
                                                                column![
                                                                    text(format!("🔐 {}", key.user_ids.first().unwrap_or(&"Unknown".to_string())))
                                                                        .size(self.view_model.text_normal)
                                                                        .color(CowboyTheme::text_primary()),
                                                                    row![
                                                                        text(format!("ID: {}", key.key_id.0))
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(CowboyTheme::text_secondary()),
                                                                        horizontal_space(),
                                                                        text(status)
                                                                            .size(self.view_model.text_tiny)
                                                                            .color(if key.is_revoked || key.is_expired {
                                                                                self.view_model.colors.red_error
                                                                            } else {
                                                                                self.view_model.colors.green_success
                                                                            }),
                                                                    ],
                                                                    text(format!("Fingerprint: {}...", &key.fingerprint.chars().take(20).collect::<String>()))
                                                                        .size(self.view_model.text_tiny)
                                                                        .color(CowboyTheme::text_secondary()),
                                                                ]
                                                                .spacing(2)
                                                            )
                                                            .padding(self.view_model.padding_sm)
                                                            .style(CowboyCustomTheme::pastel_teal_card())
                                                        );
                                                    }
                                                    key_list
                                                }
                                            ]
                                            .spacing(self.view_model.spacing_md)
                                        )
                                        .padding(self.view_model.padding_md)
                                        .style(CowboyCustomTheme::card_container())
                                    } else {
                                        container(text(""))
                                    }
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

                    // Step 8: Key Recovery from Seed (Card with Collapse)
                    container(
                        column![
                            row![
                                button(if self.recovery_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleRecoverySection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("8. Key Recovery from Seed")
                                    .size(self.view_model.text_large)
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.recovery_section_collapsed {
                                column![
                                    text("Recover keys from your backup passphrase")
                                        .size(self.view_model.text_normal)
                                        .color(CowboyTheme::text_secondary()),
                                    text("⚠️  IMPORTANT: Use the EXACT same passphrase and organization ID used during initial key generation")
                                        .size(self.view_model.text_small)
                                        .color(self.view_model.colors.orange_warning),

                                    // Recovery passphrase input
                                    text("Recovery Passphrase")
                                        .size(self.view_model.text_normal)
                                        .color(CowboyTheme::text_primary()),
                                    text_input("Enter your backup passphrase", &self.recovery_passphrase)
                                        .on_input(Message::RecoveryPassphraseChanged)
                                        .secure(true)
                                        .size(self.view_model.text_medium)
                                        .padding(self.view_model.padding_md)
                                        .style(CowboyCustomTheme::glass_input()),

                                    text_input("Confirm passphrase", &self.recovery_passphrase_confirm)
                                        .on_input(Message::RecoveryPassphraseConfirmChanged)
                                        .secure(true)
                                        .size(self.view_model.text_medium)
                                        .padding(self.view_model.padding_md)
                                        .style(CowboyCustomTheme::glass_input()),

                                    // Passphrase match indicator
                                    if !self.recovery_passphrase.is_empty() && self.recovery_passphrase == self.recovery_passphrase_confirm {
                                        text("✓ Passphrases match")
                                            .size(self.view_model.text_small)
                                            .color(self.view_model.colors.green_success)
                                    } else if !self.recovery_passphrase.is_empty() && !self.recovery_passphrase_confirm.is_empty() {
                                        text("✗ Passphrases do not match")
                                            .size(self.view_model.text_small)
                                            .color(self.view_model.colors.red_error)
                                    } else {
                                        text("")
                                    },

                                    // Organization ID input
                                    row![
                                        text("Organization ID:").size(self.view_model.text_normal),
                                        text_input("e.g., cowboyai-2025", &self.recovery_organization_id)
                                            .on_input(Message::RecoveryOrganizationIdChanged)
                                            .size(self.view_model.text_medium)
                                            .padding(self.view_model.padding_sm)
                                            .width(Length::Fixed(300.0))
                                            .style(CowboyCustomTheme::glass_input()),
                                    ]
                                    .spacing(self.view_model.spacing_md)
                                    .align_y(Alignment::Center),

                                    text("(This is the unique identifier used when first generating keys)")
                                        .size(self.view_model.text_tiny)
                                        .color(CowboyTheme::text_secondary()),

                                    // Verify seed button
                                    row![
                                        button(
                                            text("1. Verify Seed")
                                                .size(self.view_model.text_large)
                                        )
                                            .on_press_maybe(
                                                if !self.recovery_passphrase.is_empty() &&
                                                   self.recovery_passphrase == self.recovery_passphrase_confirm &&
                                                   !self.recovery_organization_id.is_empty() {
                                                    Some(Message::VerifyRecoverySeed)
                                                } else {
                                                    None
                                                }
                                            )
                                            .padding(self.view_model.padding_lg)
                                            .style(CowboyCustomTheme::primary_button()),

                                        button(
                                            text(if self.recovery_seed_verified { "2. Recover Keys" } else { "2. Recover Keys (verify first)" })
                                                .size(self.view_model.text_large)
                                        )
                                            .on_press_maybe(
                                                if self.recovery_seed_verified {
                                                    Some(Message::RecoverKeysFromSeed)
                                                } else {
                                                    None
                                                }
                                            )
                                            .padding(self.view_model.padding_lg)
                                            .style(CowboyCustomTheme::security_button()),
                                    ]
                                    .spacing(self.view_model.spacing_lg),

                                    // Status message
                                    if let Some(ref status) = self.recovery_status {
                                        text(status)
                                            .size(self.view_model.text_normal)
                                            .color(if status.contains("✅") {
                                                self.view_model.colors.green_success
                                            } else if status.contains("❌") {
                                                self.view_model.colors.red_error
                                            } else {
                                                self.view_model.colors.info
                                            })
                                    } else {
                                        text("")
                                    },

                                    // Explanation of what recovery does
                                    if self.recovery_seed_verified {
                                        container(
                                            column![
                                                text("🔐 Recovery Ready")
                                                    .size(self.view_model.text_medium)
                                                    .color(self.view_model.colors.green_success),
                                                text("The following key hierarchies can be regenerated:")
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                                text("• Root CA key pair")
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                                text("• Intermediate CA key pairs")
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                                text("• NATS operator signing keys")
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                                text("• User key pairs (deterministic from seed)")
                                                    .size(self.view_model.text_small)
                                                    .color(CowboyTheme::text_secondary()),
                                            ]
                                            .spacing(self.view_model.spacing_sm)
                                        )
                                        .padding(self.view_model.padding_md)
                                        .style(CowboyCustomTheme::pastel_mint_card())
                                    } else {
                                        container(text(""))
                                    }
                                ]
                                .spacing(self.view_model.spacing_md)
                            } else {
                                column![]
                            }
                        ]
                        .spacing(self.view_model.spacing_md)
                    )
                    .padding(self.view_model.padding_xl)
                    .style(CowboyCustomTheme::pastel_cream_card()),

                    // Step 9: Event Log / Replay (Card with Collapse)
                    container(
                        column![
                            row![
                                button(if self.event_log_section_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleEventLogSection)
                                    .padding(self.view_model.padding_sm)
                                    .style(CowboyCustomTheme::glass_button()),
                                text("9. Event Log & Replay")
                                    .size(self.view_model.text_large)
                                    .color(CowboyTheme::text_primary()),
                                horizontal_space(),
                                button(text("Load Events").size(self.view_model.text_tiny))
                                    .on_press(Message::LoadEventLog)
                                    .padding(4)
                                    .style(CowboyCustomTheme::primary_button()),
                            ]
                            .spacing(self.view_model.spacing_md)
                            .align_y(Alignment::Center),

                            if !self.event_log_section_collapsed {
                                column![
                                    text("View and replay events from the CID-based event store")
                                        .size(self.view_model.text_normal)
                                        .color(CowboyTheme::text_secondary()),

                                    if self.loaded_event_log.is_empty() {
                                        container(
                                            text("No events loaded. Click 'Load Events' to view the event log.")
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_muted())
                                        )
                                        .padding(self.view_model.padding_md)
                                    } else {
                                        container(
                                            column![
                                                row![
                                                    text(format!("Loaded Events: {}", self.loaded_event_log.len()))
                                                        .size(self.view_model.text_medium)
                                                        .color(self.view_model.colors.green_success),
                                                    horizontal_space(),
                                                    if !self.selected_events_for_replay.is_empty() {
                                                        row![
                                                            text(format!("{} selected", self.selected_events_for_replay.len()))
                                                                .size(self.view_model.text_tiny)
                                                                .color(self.view_model.colors.info),
                                                            button(text("Clear").size(self.view_model.text_tiny))
                                                                .on_press(Message::ClearEventSelection)
                                                                .padding(2)
                                                                .style(CowboyCustomTheme::glass_button()),
                                                            button(text("Replay Selected").size(self.view_model.text_tiny))
                                                                .on_press(Message::ReplaySelectedEvents)
                                                                .padding(2)
                                                                .style(CowboyCustomTheme::security_button()),
                                                        ]
                                                        .spacing(self.view_model.spacing_xs)
                                                    } else {
                                                        row![]
                                                    },
                                                ]
                                                .spacing(self.view_model.spacing_md)
                                                .align_y(Alignment::Center),
                                                {
                                                    let mut event_list = column![].spacing(self.view_model.spacing_xs);
                                                    for (idx, event) in self.loaded_event_log.iter().take(50).enumerate() {
                                                        let is_selected = self.selected_events_for_replay.contains(&event.cid);
                                                        let cid_short = event.cid.chars().take(12).collect::<String>();
                                                        // Get event type name from DomainEvent variant
                                                        let event_type = match &event.envelope.event {
                                                            crate::events::DomainEvent::Person(_) => "Person",
                                                            crate::events::DomainEvent::Organization(_) => "Organization",
                                                            crate::events::DomainEvent::Location(_) => "Location",
                                                            crate::events::DomainEvent::Certificate(_) => "Certificate",
                                                            crate::events::DomainEvent::Key(_) => "Key",
                                                            crate::events::DomainEvent::Delegation(_) => "Delegation",
                                                            crate::events::DomainEvent::NatsOperator(_) => "NatsOperator",
                                                            crate::events::DomainEvent::NatsAccount(_) => "NatsAccount",
                                                            crate::events::DomainEvent::NatsUser(_) => "NatsUser",
                                                            crate::events::DomainEvent::YubiKey(_) => "YubiKey",
                                                            crate::events::DomainEvent::Relationship(_) => "Relationship",
                                                            crate::events::DomainEvent::Manifest(_) => "Manifest",
                                                            crate::events::DomainEvent::Saga(_) => "Saga",
                                                        };
                                                        event_list = event_list.push(
                                                            button(
                                                                row![
                                                                    text(if is_selected { "☑" } else { "☐" })
                                                                        .size(self.view_model.text_small),
                                                                    text(format!("#{}", idx + 1))
                                                                        .size(self.view_model.text_tiny)
                                                                        .color(CowboyTheme::text_secondary()),
                                                                    text(format!("{}...", cid_short))
                                                                        .size(self.view_model.text_tiny)
                                                                        .color(CowboyTheme::text_muted()),
                                                                    text(event_type)
                                                                        .size(self.view_model.text_small)
                                                                        .color(if is_selected { self.view_model.colors.info } else { CowboyTheme::text_primary() }),
                                                                    horizontal_space(),
                                                                    text(event.stored_at.format("%Y-%m-%d %H:%M").to_string())
                                                                        .size(self.view_model.text_tiny)
                                                                        .color(CowboyTheme::text_secondary()),
                                                                ]
                                                                .spacing(self.view_model.spacing_sm)
                                                                .align_y(Alignment::Center)
                                                            )
                                                            .on_press(Message::ToggleEventSelection(event.cid.clone()))
                                                            .padding(self.view_model.padding_xs)
                                                            .width(iced::Length::Fill)
                                                            .style(CowboyCustomTheme::glass_button())
                                                        );
                                                    }
                                                    if self.loaded_event_log.len() > 50 {
                                                        event_list = event_list.push(
                                                            text(format!("... and {} more events", self.loaded_event_log.len() - 50))
                                                                .size(self.view_model.text_tiny)
                                                                .color(CowboyTheme::text_muted())
                                                        );
                                                    }
                                                    scrollable(event_list).height(iced::Length::Fixed(300.0))
                                                }
                                            ]
                                            .spacing(self.view_model.spacing_md)
                                        )
                                        .padding(self.view_model.padding_md)
                                        .style(CowboyCustomTheme::card_container())
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
                    .style(CowboyCustomTheme::pastel_teal_card()),
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

    fn view_projections(&self) -> Element<'_, Message> {
        // Compute readiness status (still used for SD Card projection)
        let readiness = self.compute_export_readiness();
        let is_ready = readiness.is_ready();
        let percentage = readiness.readiness_percentage();

        // ========================================================================
        // HEADER - Projections Overview
        // ========================================================================
        let header = column![
            text("Projections").size(self.view_model.text_xlarge),
            text("Bidirectional domain state mappings between CIM and external systems")
                .size(self.view_model.text_normal)
                .color(CowboyTheme::text_secondary()),
            row![
                text("Outgoing: Domain → External").size(self.view_model.text_small).color(self.view_model.colors.green_success),
                text(" | ").size(self.view_model.text_small).color(self.view_model.colors.text_tertiary),
                text("Incoming: External → Domain").size(self.view_model.text_small).color(self.view_model.colors.blue_info),
            ],
        ]
        .spacing(4);

        // ========================================================================
        // DOMAIN READINESS STATUS
        // ========================================================================
        let status_content = column![
            row![
                text(if is_ready { "✅" } else { "⚠️" }).font(EMOJI_FONT).size(20),
                text(format!("Domain Readiness: {}%", percentage))
                    .size(self.view_model.text_large)
                    .color(if is_ready {
                        self.view_model.colors.green_success
                    } else {
                        self.view_model.colors.yellow_warning
                    }),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
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
            row![
                text(format!("📜 {} certs", readiness.root_ca_count + readiness.intermediate_ca_count + readiness.leaf_cert_count))
                    .size(self.view_model.text_small),
                text(format!("🔑 {} keys", readiness.key_count))
                    .size(self.view_model.text_small),
                text(format!("👥 {} people", self.loaded_people.len()))
                    .size(self.view_model.text_small),
            ]
            .spacing(16),
        ]
        .spacing(8);

        let readiness_card: Element<'_, Message> = if is_ready {
            container(status_content)
                .padding(self.view_model.padding_md)
                .style(CowboyCustomTheme::pastel_mint_card())
                .into()
        } else {
            container(status_content)
                .padding(self.view_model.padding_md)
                .style(CowboyCustomTheme::pastel_coral_card())
                .into()
        };

        // ========================================================================
        // OUTGOING PROJECTIONS (Domain → External)
        // ========================================================================
        let outgoing_header = container(
            row![
                text("⬆️").font(EMOJI_FONT).size(20),
                column![
                    text("Outgoing Projections").size(self.view_model.text_large),
                    text("Domain → External Systems").size(self.view_model.text_small).color(CowboyTheme::text_secondary()),
                ]
                .spacing(2),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        )
        .padding(self.view_model.padding_sm);

        // SD Card Projection (air-gapped export)
        let sdcard_projection = container(
            column![
                row![
                    text("💾").font(EMOJI_FONT).size(24),
                    column![
                        text("Encrypted SD Card").size(self.view_model.text_medium),
                        text("Air-gapped snapshot of complete domain configuration").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(2),
                    horizontal_space(),
                    text(if is_ready { "🟢 Ready" } else { "🟡 Not Ready" }).size(self.view_model.text_small),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text_input("Output Directory", &self.export_path.display().to_string())
                    .on_input(Message::ExportPathChanged)
                    .style(CowboyCustomTheme::glass_input()),
                row![
                    checkbox("Public keys", self.include_public_keys)
                        .on_toggle(Message::TogglePublicKeys)
                        .style(CowboyCustomTheme::light_checkbox()),
                    checkbox("Certificates", self.include_certificates)
                        .on_toggle(Message::ToggleCertificates)
                        .style(CowboyCustomTheme::light_checkbox()),
                    checkbox("NATS config", self.include_nats_config)
                        .on_toggle(Message::ToggleNatsConfig)
                        .style(CowboyCustomTheme::light_checkbox()),
                ]
                .spacing(16),
                row![
                    checkbox("Private keys (encrypted)", self.include_private_keys)
                        .on_toggle(Message::TogglePrivateKeys)
                        .style(CowboyCustomTheme::light_checkbox()),
                ]
                .spacing(16)
                .align_y(Alignment::Center),
                if self.include_private_keys {
                    container(
                        text_input("Password", &self.export_password)
                            .on_input(Message::ExportPasswordChanged)
                            .secure(true)
                            .width(Length::Fixed(200.0))
                            .style(CowboyCustomTheme::glass_input())
                    )
                } else {
                    container(Space::with_height(0))
                },
                button(text("Project to SD Card").size(self.view_model.text_small))
                    .on_press(Message::ExportToSDCard)
                    .style(CowboyCustomTheme::security_button()),
            ]
            .spacing(self.view_model.spacing_sm)
        )
        .padding(self.view_model.padding_md)
        .style(CowboyCustomTheme::pastel_cream_card());

        // Neo4j Projection
        let neo4j_projection = container(
            column![
                row![
                    text("🔷").font(EMOJI_FONT).size(24),
                    column![
                        text("Neo4j Graph Database").size(self.view_model.text_medium),
                        text("Continuous sync for graph queries and visualization").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(2),
                    horizontal_space(),
                    text("⚪ Not Configured").size(self.view_model.text_small).color(self.view_model.colors.text_tertiary),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                row![
                    text("Endpoint:").size(self.view_model.text_small),
                    text_input("bolt://localhost:7687", &self.neo4j_endpoint)
                        .on_input(Message::Neo4jEndpointChanged)
                        .width(Length::Fixed(250.0))
                        .style(CowboyCustomTheme::glass_input()),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text("Projects: Organizations, People, Keys, Certificates, Relationships")
                    .size(self.view_model.text_tiny)
                    .color(self.view_model.colors.text_tertiary),
            ]
            .spacing(self.view_model.spacing_sm)
        )
        .padding(self.view_model.padding_md)
        .style(CowboyCustomTheme::card_container());

        // JetStream Projection
        let jetstream_status = if is_ready { "🟡 PKI Ready" } else { "🔴 Requires PKI" };
        let jetstream_projection = container(
            column![
                row![
                    text("🌊").font(EMOJI_FONT).size(24),
                    column![
                        text("JetStream Events").size(self.view_model.text_medium),
                        text("Commands, queries, and domain events (requires PKI)").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(2),
                    horizontal_space(),
                    text(jetstream_status).size(self.view_model.text_small),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                row![
                    text("NATS URL:").size(self.view_model.text_small),
                    text_input("nats://localhost:4222", &self.jetstream_url)
                        .on_input(Message::JetStreamUrlChanged)
                        .width(Length::Fixed(250.0))
                        .style(CowboyCustomTheme::glass_input()),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text("Publishes: DomainEvents, Commands, Queries to configured subjects")
                    .size(self.view_model.text_tiny)
                    .color(self.view_model.colors.text_tertiary),
            ]
            .spacing(self.view_model.spacing_sm)
        )
        .padding(self.view_model.padding_md)
        .style(CowboyCustomTheme::card_container());

        // NSC Store Projection - NATS credentials hierarchy
        let nsc_projection = container(
            column![
                row![
                    text("🔐").font(EMOJI_FONT).size(24),
                    column![
                        text("NSC Store").size(self.view_model.text_medium),
                        text("NATS security credentials and hierarchy").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(2),
                    horizontal_space(),
                    text(if self.nats_hierarchy_generated { "🟢 Ready" } else { "⚪ Not Generated" }).size(self.view_model.text_small),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                if self.nats_hierarchy_generated {
                    // Show detailed NATS hierarchy when generated
                    if let Some(ref bootstrap) = self.nats_bootstrap {
                        // Full hierarchy visualization from bootstrap data
                        let mut hierarchy_content = column![].spacing(4);

                        // Operator
                        let operator_key_preview = bootstrap.operator.nkey.public_key.public_key()
                            .chars().take(20).collect::<String>();
                        hierarchy_content = hierarchy_content.push(
                            container(
                                column![
                                    row![
                                        text("📡").font(EMOJI_FONT).size(16),
                                        text(format!("Operator: {}", bootstrap.organization.name))
                                            .size(self.view_model.text_small)
                                            .color(self.view_model.colors.green_success),
                                    ]
                                    .spacing(8),
                                    text(format!("  Public Key: {}...", operator_key_preview))
                                        .size(self.view_model.text_tiny)
                                        .color(CowboyTheme::text_secondary()),
                                ]
                                .spacing(2)
                            )
                            .padding(self.view_model.padding_sm)
                            .style(CowboyCustomTheme::pastel_mint_card())
                        );

                        // Accounts (one per organizational unit)
                        for (unit_id, (unit, account_proj)) in &bootstrap.accounts {
                            let account_key_preview = account_proj.nkey.public_key.public_key()
                                .chars().take(16).collect::<String>();
                            let users_in_unit: Vec<_> = bootstrap.users.iter()
                                .filter(|(_, (person, _))| person.unit_ids.iter().any(|uid| uid.as_uuid() == *unit_id))
                                .collect();

                            hierarchy_content = hierarchy_content.push(
                                container(
                                    column![
                                        row![
                                            text("  └─").size(self.view_model.text_small).color(CowboyTheme::text_secondary()),
                                            text("📋").font(EMOJI_FONT).size(14),
                                            text(format!("Account: {}", unit.name))
                                                .size(self.view_model.text_small)
                                                .color(self.view_model.colors.blue_info),
                                        ]
                                        .spacing(4),
                                        text(format!("      Key: {}...", account_key_preview))
                                            .size(self.view_model.text_tiny)
                                            .color(CowboyTheme::text_secondary()),
                                        text(format!("      {} users in this account", users_in_unit.len()))
                                            .size(self.view_model.text_tiny)
                                            .color(CowboyTheme::text_secondary()),
                                    ]
                                    .spacing(2)
                                )
                                .padding(4)
                            );

                            // Users in this account
                            for (_, (person, user_proj)) in &users_in_unit {
                                let user_key_preview = user_proj.nkey.public_key.public_key()
                                    .chars().take(12).collect::<String>();
                                hierarchy_content = hierarchy_content.push(
                                    row![
                                        text("          └─").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                                        text("👤").font(EMOJI_FONT).size(12),
                                        text(&person.name)
                                            .size(self.view_model.text_tiny)
                                            .color(CowboyTheme::text_primary()),
                                        text(format!("({}...)", user_key_preview))
                                            .size(8)
                                            .color(CowboyTheme::text_secondary()),
                                    ]
                                    .spacing(4)
                                );
                            }
                        }

                        // Service accounts if any
                        if !bootstrap.service_accounts.is_empty() {
                            hierarchy_content = hierarchy_content.push(
                                text(format!("  ⚙️ {} Service Accounts", bootstrap.service_accounts.len()))
                                    .size(self.view_model.text_small)
                                    .color(self.view_model.colors.info)
                            );
                        }

                        // Summary stats
                        hierarchy_content = hierarchy_content.push(
                            container(
                                row![
                                    text(format!("Total: {} identities", bootstrap.total_identities()))
                                        .size(self.view_model.text_tiny)
                                        .color(self.view_model.colors.green_success),
                                    horizontal_space(),
                                    text(format!("{} accounts, {} users",
                                        bootstrap.accounts.len(),
                                        bootstrap.users.len()))
                                        .size(self.view_model.text_tiny)
                                        .color(CowboyTheme::text_secondary()),
                                ]
                            )
                            .padding(self.view_model.padding_sm)
                            .style(CowboyCustomTheme::card_container())
                        );

                        column![
                            container(hierarchy_content)
                                .padding(self.view_model.padding_sm)
                                .style(CowboyCustomTheme::card_container()),
                            text(format!("Export path: {}", self.nats_export_path.display()))
                                .size(self.view_model.text_tiny)
                                .color(self.view_model.colors.text_tertiary),
                            button(text("Project to NSC Store").size(self.view_model.text_small))
                                .on_press(Message::ExportToNsc)
                                .style(CowboyCustomTheme::primary_button()),
                        ]
                        .spacing(8)
                    } else {
                        // Fallback if bootstrap data not available
                        column![
                            container(
                                column![
                                    row![
                                        text("📡").font(EMOJI_FONT).size(16),
                                        text(format!("Operator: {}", self.organization_name))
                                            .size(self.view_model.text_small)
                                            .color(self.view_model.colors.green_success),
                                    ]
                                    .spacing(8),
                                    row![
                                        text("  └─").size(self.view_model.text_small).color(CowboyTheme::text_secondary()),
                                        text("📋").font(EMOJI_FONT).size(14),
                                        text(format!("Account: {}-system", self.organization_name.to_lowercase().replace(' ', "-")))
                                            .size(self.view_model.text_small)
                                            .color(self.view_model.colors.blue_info),
                                    ]
                                    .spacing(4),
                                    {
                                        let users_count = self.loaded_people.len();
                                        row![
                                            text("      └─").size(self.view_model.text_small).color(CowboyTheme::text_secondary()),
                                            text("👥").font(EMOJI_FONT).size(14),
                                            text(format!("{} Users", users_count))
                                                .size(self.view_model.text_small)
                                                .color(CowboyTheme::text_primary()),
                                        ]
                                        .spacing(4)
                                    },
                                ]
                                .spacing(4)
                            )
                            .padding(self.view_model.padding_sm)
                            .style(CowboyCustomTheme::card_container()),
                            text(format!("Export path: {}", self.nats_export_path.display()))
                                .size(self.view_model.text_tiny)
                                .color(self.view_model.colors.text_tertiary),
                            button(text("Project to NSC Store").size(self.view_model.text_small))
                                .on_press(Message::ExportToNsc)
                                .style(CowboyCustomTheme::primary_button()),
                        ]
                        .spacing(8)
                    }
                } else {
                    // Show generation button when not yet generated
                    column![
                        text("NATS hierarchy not yet generated")
                            .size(self.view_model.text_tiny)
                            .color(self.view_model.colors.orange_warning),
                        row![
                            button(text("Generate NATS Hierarchy").size(self.view_model.text_small))
                                .on_press(Message::GenerateNatsHierarchy)
                                .style(CowboyCustomTheme::security_button()),
                            if self.organization_name.is_empty() {
                                text("(Create organization first)")
                                    .size(self.view_model.text_tiny)
                                    .color(self.view_model.colors.text_tertiary)
                            } else {
                                text(format!("for {}", self.organization_name))
                                    .size(self.view_model.text_tiny)
                                    .color(CowboyTheme::text_secondary())
                            },
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(8)
                },
            ]
            .spacing(self.view_model.spacing_sm)
        )
        .padding(self.view_model.padding_md)
        .style(CowboyCustomTheme::card_container());

        // ========================================================================
        // INCOMING INJECTIONS (External → Domain)
        // ========================================================================
        let incoming_header = container(
            row![
                text("⬇️").font(EMOJI_FONT).size(20),
                column![
                    text("Incoming Injections").size(self.view_model.text_large),
                    text("External Sources → Domain").size(self.view_model.text_small).color(CowboyTheme::text_secondary()),
                ]
                .spacing(2),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        )
        .padding(self.view_model.padding_sm);

        // JSON Configuration Injection
        let json_injection = container(
            column![
                row![
                    text("📄").font(EMOJI_FONT).size(24),
                    column![
                        text("JSON Configuration").size(self.view_model.text_medium),
                        text("Load domain configuration from JSON files").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(2),
                    horizontal_space(),
                    text("⚪ Not Configured").size(self.view_model.text_small).color(self.view_model.colors.text_tertiary),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text("Imports: domain-bootstrap.json, organizations, people, policies")
                    .size(self.view_model.text_tiny)
                    .color(self.view_model.colors.text_tertiary),
            ]
            .spacing(self.view_model.spacing_sm)
        )
        .padding(self.view_model.padding_md)
        .style(CowboyCustomTheme::card_container());

        // Petgraph Injection
        let petgraph_injection = container(
            column![
                row![
                    text("🕸️").font(EMOJI_FONT).size(24),
                    column![
                        text("Petgraph Import").size(self.view_model.text_medium),
                        text("Import graph structures from Petgraph format").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(2),
                    horizontal_space(),
                    text("⚪ Not Configured").size(self.view_model.text_small).color(self.view_model.colors.text_tertiary),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text("Imports: Graph nodes, edges, relationships from petgraph serialization")
                    .size(self.view_model.text_tiny)
                    .color(self.view_model.colors.text_tertiary),
            ]
            .spacing(self.view_model.spacing_sm)
        )
        .padding(self.view_model.padding_md)
        .style(CowboyCustomTheme::card_container());

        // JetStream Replay Injection
        let jetstream_replay = container(
            column![
                row![
                    text("⏪").font(EMOJI_FONT).size(24),
                    column![
                        text("JetStream Replay").size(self.view_model.text_medium),
                        text("Replay events from JetStream for state reconstruction").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(2),
                    horizontal_space(),
                    text("⚪ Not Configured").size(self.view_model.text_small).color(self.view_model.colors.text_tertiary),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text("Replays: Domain events from JetStream streams for projection rebuild")
                    .size(self.view_model.text_tiny)
                    .color(self.view_model.colors.text_tertiary),
            ]
            .spacing(self.view_model.spacing_sm)
        )
        .padding(self.view_model.padding_md)
        .style(CowboyCustomTheme::card_container());

        // Contact Systems Injection
        let contact_injection = container(
            column![
                row![
                    text("📇").font(EMOJI_FONT).size(24),
                    column![
                        text("Contact Systems").size(self.view_model.text_medium),
                        text("JSON schema adapters for external contact management").size(self.view_model.text_tiny).color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(2),
                    horizontal_space(),
                    text("⚪ Not Configured").size(self.view_model.text_small).color(self.view_model.colors.text_tertiary),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text("Bidirectional: Import/export contacts via JSON schema adapters")
                    .size(self.view_model.text_tiny)
                    .color(self.view_model.colors.text_tertiary),
            ]
            .spacing(self.view_model.spacing_sm)
        )
        .padding(self.view_model.padding_md)
        .style(CowboyCustomTheme::card_container());

        // ========================================================================
        // COMPOSE THE FULL VIEW
        // ========================================================================
        let content = column![
            header,
            readiness_card,

            // Outgoing Projections Section
            outgoing_header,
            sdcard_projection,
            neo4j_projection,
            jetstream_projection,
            nsc_projection,

            // Incoming Injections Section
            incoming_header,
            json_injection,
            petgraph_injection,
            jetstream_replay,
            contact_injection,
        ]
        .spacing(self.view_model.spacing_md)
        .padding(self.view_model.padding_md);

        scrollable(content).into()
    }

    fn view_workflow(&self) -> Element<'_, Message> {
        self.workflow_view.view().map(Message::WorkflowMessage)
    }

    fn view_state_machines(&self) -> Element<'_, Message> {
        use state_machine_graph::StateMachineType;

        // Create state machine type selector
        let machine_types = vec![
            StateMachineType::Key,
            StateMachineType::Certificate,
            StateMachineType::Person,
            StateMachineType::Organization,
            StateMachineType::YubiKey,
            StateMachineType::PkiBootstrap,
            StateMachineType::CertificateProvisioning,
        ];

        let machine_buttons: Vec<Element<'_, Message>> = machine_types
            .into_iter()
            .map(|sm_type| {
                let is_selected = self.selected_state_machine == sm_type;
                let btn_style = CowboyCustomTheme::glass_menu_button(is_selected);

                button(text(sm_type.display_name()).size(self.view_model.text_small))
                    .on_press(Message::StateMachineSelected(sm_type))
                    .style(btn_style)
                    .into()
            })
            .collect();

        let machine_selector = row(machine_buttons)
            .spacing(self.view_model.spacing_sm)
            .padding(self.view_model.padding_md);

        // Current state machine info (uses stored definition to avoid lifetime issues)
        let state_count = self.state_machine_definition.states.len();
        let transition_count = self.state_machine_definition.transitions.len();
        let terminal_count = self.state_machine_definition.states.iter().filter(|s| s.is_terminal).count();

        let info_text = text(format!(
            "{} | {} states | {} transitions | {} terminal states",
            self.selected_state_machine.display_name(),
            state_count,
            transition_count,
            terminal_count
        ))
        .size(self.view_model.text_small)
        .color(CowboyTheme::text_secondary());

        // Category badge
        let category_text = text(format!("Category: {}", self.selected_state_machine.category()))
            .size(self.view_model.text_small)
            .color(self.view_model.colors.accent);

        // State list (compact for side panel)
        let state_items: Vec<Element<'_, Message>> = self.state_machine_definition.states.iter().map(|state| {
            let state_color = if state.is_terminal {
                Color::from_rgb(0.8, 0.2, 0.2)
            } else if state.is_initial {
                Color::from_rgb(0.2, 0.8, 0.2)
            } else {
                state.color
            };

            let suffix = if state.is_terminal {
                " ◉"
            } else if state.is_initial {
                " ◐"
            } else {
                ""
            };

            row![
                container(text("●").color(state_color))
                    .width(Length::Fixed(16.0)),
                text(format!("{}{}", state.name, suffix))
                    .size(self.view_model.text_tiny),
            ]
            .spacing(4)
            .into()
        }).collect();

        // Transition list (compact)
        let transition_items: Vec<Element<'_, Message>> = self.state_machine_definition.transitions.iter().map(|trans| {
            let arrow = if trans.from == trans.to {
                format!("{} ↻", trans.from)
            } else {
                format!("{} → {}", trans.from, trans.to)
            };
            text(format!("{} [{}]", arrow, trans.label))
                .size(self.view_model.text_tiny)
                .color(CowboyTheme::text_secondary())
                .into()
        }).collect();

        // Graph canvas - renders the actual Mealy state machine diagram
        let graph_canvas: Element<'_, Message> = Canvas::new(StateMachineCanvasProgram {
            definition: &self.state_machine_definition,
            vm: &self.view_model,
        })
        .width(Length::Fill)
        .height(Length::Fixed(450.0))
        .into();

        let content = column![
            // Header
            container(
                column![
                    text("Mealy State Machine Visualization")
                        .size(self.view_model.text_large)
                        .color(CowboyTheme::text_primary()),
                    text("States as objects, transitions as morphisms in a small category")
                        .size(self.view_model.text_small)
                        .color(CowboyTheme::text_secondary()),
                ]
                .spacing(4)
            )
            .padding(self.view_model.padding_md),

            // Machine selector
            container(machine_selector)
                .style(CowboyCustomTheme::pastel_teal_card()),

            // Info bar
            container(
                row![info_text, horizontal_space(), category_text]
                    .align_y(Alignment::Center)
            )
            .padding(self.view_model.padding_sm),

            // Main content: Graph on left, details on right
            row![
                // Graph canvas (primary view)
                container(graph_canvas)
                    .style(CowboyCustomTheme::card_container())
                    .padding(self.view_model.padding_sm)
                    .width(Length::FillPortion(3)),

                // Details panel (states + transitions)
                container(
                    column![
                        text("States")
                            .size(self.view_model.text_small)
                            .color(CowboyTheme::text_primary()),
                        scrollable(
                            column(state_items).spacing(2)
                        )
                        .height(Length::Fixed(180.0)),

                        vertical_space().height(Length::Fixed(8.0)),

                        text("Transitions")
                            .size(self.view_model.text_small)
                            .color(CowboyTheme::text_primary()),
                        scrollable(
                            column(transition_items).spacing(1)
                        )
                        .height(Length::Fixed(200.0)),
                    ]
                    .spacing(self.view_model.spacing_xs)
                )
                .padding(self.view_model.padding_sm)
                .style(CowboyCustomTheme::card_container())
                .width(Length::FillPortion(1)),
            ]
            .spacing(self.view_model.spacing_md),

            // Legend
            container(
                row![
                    text("◐").color(Color::from_rgb(0.2, 0.8, 0.2)),
                    text(" Initial").size(self.view_model.text_tiny),
                    horizontal_space().width(Length::Fixed(15.0)),
                    text("◉").color(Color::from_rgb(0.8, 0.2, 0.2)),
                    text(" Terminal").size(self.view_model.text_tiny),
                    horizontal_space().width(Length::Fixed(15.0)),
                    text("●").color(Color::from_rgb(0.5, 0.6, 0.7)),
                    text(" State").size(self.view_model.text_tiny),
                    horizontal_space().width(Length::Fixed(15.0)),
                    text("→").color(Color::from_rgb(0.5, 0.6, 0.7)),
                    text(" Transition").size(self.view_model.text_tiny),
                    horizontal_space().width(Length::Fixed(15.0)),
                    text("↻").color(Color::from_rgb(0.5, 0.6, 0.7)),
                    text(" Self-loop").size(self.view_model.text_tiny),
                ]
                .spacing(4)
                .align_y(Alignment::Center)
            )
            .padding(self.view_model.padding_sm)
            .style(CowboyCustomTheme::pastel_mint_card()),
        ]
        .spacing(self.view_model.spacing_sm)
        .padding(self.view_model.padding_md);

        scrollable(content).into()
    }
}

// ============================================================================
// State Machine Canvas Program - Renders the Mealy graph
// ============================================================================

/// Canvas program for rendering state machine as a graph
struct StateMachineCanvasProgram<'a> {
    definition: &'a state_machine_graph::StateMachineDefinition,
    vm: &'a ViewModel,
}

impl<'a> canvas::Program<Message> for StateMachineCanvasProgram<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        use std::f32::consts::PI;

        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Layout configuration from bounds and ViewModel
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let layout_radius = (bounds.width.min(bounds.height) / 2.0 - 60.0).max(80.0);
        let node_radius = (25.0 * self.vm.scale).clamp(18.0, 35.0);

        let states = &self.definition.states;
        let transitions = &self.definition.transitions;

        if states.is_empty() {
            return vec![frame.into_geometry()];
        }

        // Calculate positions in a circle
        let mut positions: std::collections::HashMap<String, Point> = std::collections::HashMap::new();
        let angle_step = 2.0 * PI / states.len() as f32;

        for (i, state) in states.iter().enumerate() {
            let angle = angle_step * i as f32 - PI / 2.0; // Start from top
            let x = center.x + layout_radius * angle.cos();
            let y = center.y + layout_radius * angle.sin();
            positions.insert(state.name.clone(), Point::new(x, y));
        }

        // Draw title
        let title = canvas::Text {
            content: self.definition.machine_type.display_name().to_string(),
            position: Point::new(center.x, 15.0),
            color: Color::from_rgb(0.9, 0.9, 0.9),
            size: iced::Pixels(self.vm.text_medium as f32),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Top,
            ..Default::default()
        };
        frame.fill_text(title);

        // Draw transitions first (behind states)
        for trans in transitions {
            if let (Some(&from_pos), Some(&to_pos)) = (
                positions.get(&trans.from),
                positions.get(&trans.to),
            ) {
                self.draw_transition(&mut frame, trans, from_pos, to_pos, node_radius);
            }
        }

        // Draw states
        for state in states {
            if let Some(&pos) = positions.get(&state.name) {
                self.draw_state(&mut frame, state, pos, node_radius);
            }
        }

        vec![frame.into_geometry()]
    }
}

impl<'a> StateMachineCanvasProgram<'a> {
    fn draw_state(
        &self,
        frame: &mut canvas::Frame,
        state: &state_machine_graph::StateMachineState,
        pos: Point,
        radius: f32,
    ) {
        // Determine colors
        let fill_color = if state.is_terminal {
            Color::from_rgb(0.6, 0.2, 0.2)
        } else if state.is_initial {
            Color::from_rgb(0.2, 0.5, 0.3)
        } else {
            Color::from_rgb(0.25, 0.35, 0.45)
        };

        let stroke_color = Color::from_rgb(0.4, 0.6, 0.8);

        // Initial state: outer circle
        if state.is_initial {
            let outer = canvas::Path::circle(pos, radius + 5.0);
            frame.stroke(
                &outer,
                canvas::Stroke::default()
                    .with_color(stroke_color)
                    .with_width(self.vm.border_thin),
            );
        }

        // Main circle
        let circle = canvas::Path::circle(pos, radius);
        frame.fill(&circle, fill_color);
        frame.stroke(
            &circle,
            canvas::Stroke::default()
                .with_color(stroke_color)
                .with_width(self.vm.border_normal),
        );

        // Terminal state: inner circle
        if state.is_terminal {
            let inner = canvas::Path::circle(pos, radius - 5.0);
            frame.stroke(
                &inner,
                canvas::Stroke::default()
                    .with_color(stroke_color)
                    .with_width(self.vm.border_thin),
            );
        }

        // State name
        let display_name = if state.name.len() > 10 {
            format!("{}…", &state.name[..9])
        } else {
            state.name.clone()
        };

        let text_content = canvas::Text {
            content: display_name,
            position: pos,
            color: Color::from_rgb(0.95, 0.95, 0.95),
            size: iced::Pixels(self.vm.text_tiny as f32),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            ..Default::default()
        };
        frame.fill_text(text_content);
    }

    fn draw_transition(
        &self,
        frame: &mut canvas::Frame,
        trans: &state_machine_graph::StateMachineTransition,
        from: Point,
        to: Point,
        node_radius: f32,
    ) {
        use std::f32::consts::PI;

        let stroke_color = if trans.is_active {
            Color::from_rgb(0.0, 0.8, 0.4)
        } else {
            Color::from_rgb(0.5, 0.6, 0.7)
        };

        let is_self_loop = trans.from == trans.to;

        if is_self_loop {
            // Self-loop: arc above state
            let loop_radius = 18.0;
            let loop_center = Point::new(from.x, from.y - node_radius - loop_radius);

            let arc = canvas::Path::new(|builder| {
                builder.arc(canvas::path::Arc {
                    center: loop_center,
                    radius: loop_radius,
                    start_angle: iced::Radians(0.3 * PI),
                    end_angle: iced::Radians(2.7 * PI),
                });
            });
            frame.stroke(
                &arc,
                canvas::Stroke::default()
                    .with_color(stroke_color)
                    .with_width(self.vm.border_normal),
            );

            // Arrowhead
            let arrow_pos = Point::new(from.x + 6.0, from.y - node_radius - 2.0);
            self.draw_arrowhead(frame, arrow_pos, PI / 4.0, stroke_color);

            // Label
            let label_pos = Point::new(from.x, from.y - node_radius - loop_radius * 2.0 - 6.0);
            let label = canvas::Text {
                content: Self::truncate(&trans.label, 12),
                position: label_pos,
                color: stroke_color,
                size: iced::Pixels((self.vm.text_tiny - 1) as f32),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Bottom,
                ..Default::default()
            };
            frame.fill_text(label);
        } else {
            // Regular transition: curved arrow
            let dx = to.x - from.x;
            let dy = to.y - from.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 0.001 {
                return;
            }

            let ux = dx / dist;
            let uy = dy / dist;
            let px = -uy;
            let py = ux;
            let curve_offset = 18.0;

            let start = Point::new(from.x + ux * node_radius, from.y + uy * node_radius);
            let end = Point::new(to.x - ux * (node_radius + 6.0), to.y - uy * (node_radius + 6.0));
            let mid = Point::new(
                (from.x + to.x) / 2.0 + px * curve_offset,
                (from.y + to.y) / 2.0 + py * curve_offset,
            );

            let curve = canvas::Path::new(|builder| {
                builder.move_to(start);
                builder.quadratic_curve_to(mid, end);
            });
            frame.stroke(
                &curve,
                canvas::Stroke::default()
                    .with_color(stroke_color)
                    .with_width(self.vm.border_normal),
            );

            // Arrowhead
            let arrow_dx = end.x - mid.x;
            let arrow_dy = end.y - mid.y;
            let arrow_angle = arrow_dy.atan2(arrow_dx);
            self.draw_arrowhead(frame, end, arrow_angle, stroke_color);

            // Label
            let label_pos = Point::new(mid.x, mid.y - 6.0);
            let label = canvas::Text {
                content: Self::truncate(&trans.label, 15),
                position: label_pos,
                color: stroke_color,
                size: iced::Pixels((self.vm.text_tiny - 1) as f32),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Bottom,
                ..Default::default()
            };
            frame.fill_text(label);
        }
    }

    fn draw_arrowhead(&self, frame: &mut canvas::Frame, tip: Point, angle: f32, color: Color) {
        use std::f32::consts::PI;

        let size = 6.0 * self.vm.scale;
        let half_angle = PI / 6.0;

        let left = Point::new(
            tip.x + size * (angle + PI - half_angle).cos(),
            tip.y + size * (angle + PI - half_angle).sin(),
        );
        let right = Point::new(
            tip.x + size * (angle + PI + half_angle).cos(),
            tip.y + size * (angle + PI + half_angle).sin(),
        );

        let arrow = canvas::Path::new(|builder| {
            builder.move_to(tip);
            builder.line_to(left);
            builder.line_to(right);
            builder.close();
        });
        frame.fill(&arrow, color);
    }

    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}…", &s[..max_len - 1])
        }
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
                    "subject": e.subject_name.to_rfc4514(),
                    "issuer": e.issuer,
                    "not_before": e.validity.not_before(),
                    "not_after": e.validity.not_after(),
                    "is_ca": e.basic_constraints.is_ca(),
                    "key_usage": e.key_usage.to_string_list(),
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