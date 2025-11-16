//! Native/WASM GUI for offline key generation using Iced 0.13+
//!
//! This module provides a pure Rust GUI that can run both as a native
//! application and as a WASM application in the browser.

use iced::{
    application,
    widget::{button, column, container, row, text, text_input, Container, horizontal_space, pick_list, progress_bar, checkbox, scrollable, Space, image, stack},
    Task, Element, Length, Color, Border, Theme, Background, Shadow, Alignment,
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
};

pub mod graph;
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
pub mod edge_indicator;

use graph::{OrganizationGraph, GraphMessage};
use event_emitter::{CimEventEmitter, GuiEventSubscriber, InteractionType};
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

    // Domain configuration
    domain_loaded: bool,
    _domain_path: PathBuf,  // Reserved for domain persistence path
    organization_name: String,
    organization_domain: String,
    organization_id: Option<Uuid>,  // Set when domain is created

    // Master passphrase for encryption
    master_passphrase: String,
    master_passphrase_confirm: String,

    bootstrap_config: Option<BootstrapConfig>,
    aggregate: Arc<RwLock<KeyManagementAggregate>>,
    projection: Arc<RwLock<OfflineKeyProjection>>,

    // Event-driven communication
    event_emitter: CimEventEmitter,
    _event_subscriber: GuiEventSubscriber,  // Reserved for future NATS integration

    // Graph visualization
    org_graph: OrganizationGraph,
    selected_person: Option<Uuid>,

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

    // Status
    status_message: String,
    error_message: Option<String>,

    // Animation
    animation_time: f32,
    firefly_shader: FireflyRenderer,

    // Responsive scaling
    ui_scale: f32,  // Scale factor for UI elements (1.0 = base size at 1920x1080)

    // MVI Integration (Option A: Quick Integration)
    mvi_model: MviModel,
    storage_port: Arc<dyn StoragePort>,
    x509_port: Arc<dyn X509Port>,
    ssh_port: Arc<dyn SshKeyPort>,
    yubikey_port: Arc<dyn YubiKeyPort>,

    // Phase 4: Interactive UI Components
    context_menu: ContextMenu,
    property_card: PropertyCard,
    context_menu_node: Option<Uuid>,  // Node that context menu was opened on (if any)
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

/// Messages for the application
#[derive(Debug, Clone)]
pub enum Message {
    // Tab Navigation
    TabSelected(Tab),

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
    ExportToNsc,
    NscExported(Result<String, String>),

    // Root passphrase operations
    RootPassphraseChanged(String),
    RootPassphraseConfirmChanged(String),

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

    // Phase 4: Interactive UI Component Messages
    ContextMenuMessage(ContextMenuMessage),
    PropertyCardMessage(PropertyCardMessage),

    // MVI Integration
    MviIntent(Intent),

    // Animation
    AnimationTick,

    // UI Scaling
    IncreaseScale,
    DecreaseScale,
    ResetScale,
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

impl CimKeysApp {
    fn new(output_dir: String) -> (Self, Task<Message>) {
        let aggregate = Arc::new(RwLock::new(KeyManagementAggregate::new()));
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
                domain_loaded: false,
                _domain_path: PathBuf::from(&output_dir),
                organization_name: String::new(),
                organization_domain: String::new(),
                organization_id: None,
                master_passphrase: String::new(),
                master_passphrase_confirm: String::new(),
                bootstrap_config: None,
                aggregate,
                projection,
                event_emitter: CimEventEmitter::new(default_org),
                _event_subscriber: GuiEventSubscriber::new(default_org),
                org_graph: OrganizationGraph::new(),
                selected_person: None,
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
                status_message: String::from("🔐 Welcome to CIM Keys - Offline Key Management System"),
                error_message: None,
                animation_time: 0.0,
                firefly_shader: FireflyRenderer::new(),
                ui_scale: 1.0,  // Default scale for 1920x1080
                // MVI integration
                mvi_model,
                storage_port,
                x509_port,
                ssh_port,
                yubikey_port,
                // Phase 4: Interactive UI Components
                context_menu: ContextMenu::new(),
                property_card: PropertyCard::new(),
                context_menu_node: None,
            },
            load_task,
        )
    }

    // Helper methods for responsive scaling
    fn scaled_text_size(&self, base_size: u16) -> u16 {
        (base_size as f32 * self.ui_scale).round() as u16
    }

    fn scaled_padding(&self, base_padding: u16) -> u16 {
        (base_padding as f32 * self.ui_scale).round() as u16
    }

    fn scaled_size(&self, base_size: f32) -> f32 {
        base_size * self.ui_scale
    }

    // Note: Title method removed - window title now set via iced::Settings

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Tab navigation
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                self.status_message = match tab {
                    Tab::Welcome => "Welcome to CIM Keys".to_string(),
                    Tab::Organization => format!("Organization Structure and Key Ownership (Graph: {} nodes, {} edges)",
                        self.org_graph.nodes.len(), self.org_graph.edges.len()),
                    Tab::Locations => "Manage Corporate Locations".to_string(),
                    Tab::Keys => "Generate Cryptographic Keys".to_string(),
                    Tab::Export => "Export Domain Configuration".to_string(),
                };
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

                Task::perform(
                    async move {
                        let mut proj = projection.write().await;
                        proj.set_organization(
                            org_name.clone(),
                            org_domain.clone(),
                            "US".to_string(),  // TODO: Make configurable
                            "admin@example.com".to_string(),  // TODO: Make configurable
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
                let _street = self.new_location_street.clone();
                let _city = self.new_location_city.clone();
                let _region = self.new_location_region.clone();
                let _country = self.new_location_country.clone();
                let _postal = self.new_location_postal.clone();

                // TODO: Once projection supports full Location objects, create proper Address
                // and pass Location with all address details

                // Clear form fields immediately
                self.new_location_name.clear();
                self.new_location_type = None;
                self.new_location_street.clear();
                self.new_location_city.clear();
                self.new_location_region.clear();
                self.new_location_country.clear();
                self.new_location_postal.clear();

                // Persist to projection
                let projection = self.projection.clone();

                Task::perform(
                    async move {
                        let mut proj = projection.write().await;
                        // TODO: Update projection to use new Location from cim-domain-location
                        proj.add_location(location_id, location_name.clone(), location_type, org_id)
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
                                            "✓ {} ({}) - {} provisioned in slot {:?}",
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

                        if results.iter().all(|r| r.starts_with('✓')) {
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

            Message::GenerateServerCert => {
                if let Some(ref ca_name) = self.selected_intermediate_ca {
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
                        self.status_message = format!("✓ NATS hierarchy generated for {}", self.organization_name);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("NATS generation failed: {}", e));
                    }
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
                        self.status_message = format!("✓ NATS hierarchy exported to NSC store: {}", path);
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

            // Graph interactions
            Message::GraphMessage(graph_msg) => {
                match &graph_msg {
                    GraphMessage::NodeClicked(id) => {
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
                                    let color = Color::from_rgb(0.3, 0.3, 0.7);

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
                    GraphMessage::AddEdge { from, to, edge_type } => {
                        self.org_graph.add_edge(*from, *to, edge_type.clone());
                        self.status_message = String::from("Relationship added");
                    }
                    // Phase 4: Right-click shows context menu
                    GraphMessage::RightClick(position) => {
                        // Check if we right-clicked on a node
                        self.context_menu_node = None;
                        for (node_id, node) in &self.org_graph.nodes {
                            let dx = position.x - node.position.x;
                            let dy = position.y - node.position.y;
                            let distance = (dx * dx + dy * dy).sqrt();
                            if distance <= 25.0 {  // Within node radius
                                self.context_menu_node = Some(*node_id);
                                break;
                            }
                        }

                        self.context_menu.show(*position);
                        if self.context_menu_node.is_some() {
                            self.status_message = "Node context menu opened".to_string();
                        } else {
                            self.status_message = "Context menu opened".to_string();
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
                                (NodeType::Organization(org), label, Color::from_rgb(0.2, 0.3, 0.6))
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
                                (NodeType::OrganizationalUnit(unit), label, Color::from_rgb(0.4, 0.5, 0.8))
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
                                (NodeType::Person { person, role: KeyOwnerRole::Developer }, label, Color::from_rgb(0.5, 0.7, 0.3))
                            }
                            NodeCreationType::Location => {
                                use cim_domain::EntityId;
                                use crate::domain::{Address, LocationMarker};

                                // Create a placeholder physical location
                                // Note: Use the "Locations" tab to create locations with full address details
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
                                (NodeType::Location(location), label, Color::from_rgb(0.6, 0.5, 0.4))
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
                                (NodeType::Role(role), label, Color::from_rgb(0.6, 0.3, 0.8))
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
                                (NodeType::Policy(policy), label, Color::from_rgb(0.8, 0.6, 0.2))
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
                        // Create property change event
                        if let Some(node_id) = self.property_card.node_id() {
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
                                        graph::NodeType::Policy(updated)
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
                    _ => {}
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
                self.ui_scale = (self.ui_scale + 0.1).min(2.0);  // Max 2x scale
                self.status_message = format!("UI Scale: {:.0}%", self.ui_scale * 100.0);
                Task::none()
            }

            Message::DecreaseScale => {
                self.ui_scale = (self.ui_scale - 0.1).max(0.5);  // Min 0.5x scale
                self.status_message = format!("UI Scale: {:.0}%", self.ui_scale * 100.0);
                Task::none()
            }

            Message::ResetScale => {
                self.ui_scale = 1.0;
                self.status_message = "UI Scale: 100%".to_string();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::{stack, shader};

        // Cowboy AI logo with glass morphism styling
        let logo_container = container(
            image("assets/logo.png")
                .width(Length::Fixed(self.scaled_size(80.0)))
                .height(Length::Fixed(self.scaled_size(80.0)))
        )
        .width(Length::Fixed(self.scaled_size(100.0)))
        .height(Length::Fixed(self.scaled_size(100.0)))
        .padding(self.scaled_padding(10))
        .center(Length::Fixed(self.scaled_size(100.0)))
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(CowboyTheme::logo_radial_gradient()),
                border: Border {
                    color: Color::from_rgba(0.4, 0.7, 0.75, 0.5),  // Teal border
                    width: 2.0,
                    radius: 12.0.into(),
                },
                text_color: Some(CowboyTheme::text_primary()),
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
            }
        });

        // Tab bar
        let tab_bar = row![
            button(text("Welcome").size(self.scaled_text_size(14)))
                .on_press(Message::TabSelected(Tab::Welcome))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Welcome)),
            button(text("Organization").size(self.scaled_text_size(14)))
                .on_press(Message::TabSelected(Tab::Organization))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Organization)),
            button(text("Locations").size(self.scaled_text_size(14)))
                .on_press(Message::TabSelected(Tab::Locations))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Locations)),
            button(text("Keys").size(self.scaled_text_size(14)))
                .on_press(Message::TabSelected(Tab::Keys))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Keys)),
            button(text("Export").size(self.scaled_text_size(14)))
                .on_press(Message::TabSelected(Tab::Export))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Export)),
        ]
        .spacing((5.0 * self.ui_scale) as u16);

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
                            .size(self.scaled_text_size(14))
                            .color(CowboyTheme::text_primary()),
                        horizontal_space(),
                        button("✕")
                            .on_press(Message::ClearError)
                            .style(CowboyCustomTheme::glass_button())
                    ]
                    .padding(self.scaled_padding(10))
                )
                .style(|_theme: &Theme| container::Style {
                    background: Some(CowboyTheme::warning_gradient()),
                    text_color: Some(CowboyTheme::text_primary()),
                    border: Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.3),
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
                    .size(self.scaled_text_size(24))
                    .color(CowboyTheme::text_primary()),
                text("The Cowboy AI Infrastructure")
                    .size(self.scaled_text_size(14))
                    .color(CowboyTheme::text_secondary()),
            ]
            .spacing((5.0 * self.ui_scale) as u16),
        ]
        .spacing(self.scaled_padding(20))
        .align_y(iced::Alignment::Center);

        let mut main_column = column![
            header,
            text(&self.status_message)
                .size(self.scaled_text_size(12))
                .color(CowboyTheme::text_secondary()),
            container(tab_bar)
                .padding(self.scaled_padding(10))
                .style(CowboyCustomTheme::glass_container()),
        ]
        .spacing(self.scaled_padding(10));

        if let Some(error) = error_display {
            main_column = main_column.push(error);
        }

        main_column = main_column.push(content);

        // Layer the animated background behind the main content
        let main_content = container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        // Stack the background gradient, firefly shader, and main content
        stack![
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
        .height(Length::Fill)
        .into()
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
                                _ => None,
                            }
                        } else {
                            // Phase 4: Keyboard shortcuts without modifiers
                            match key {
                                Key::Named(keyboard::key::Named::Escape) => {
                                    Some(Message::GraphMessage(
                                        crate::gui::graph::GraphMessage::CancelEdgeCreation
                                    ))
                                }
                                Key::Named(keyboard::key::Named::Delete) => {
                                    Some(Message::GraphMessage(
                                        crate::gui::graph::GraphMessage::DeleteSelected
                                    ))
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
            text("Welcome to CIM Keys!").size(self.scaled_text_size(28)),
            text("Generate and manage cryptographic keys for your CIM infrastructure").size(self.scaled_text_size(16)),
            container(
                column![
                    text("⚠️ Security Notice").size(self.scaled_text_size(18)),
                    text("This application should be run on an air-gapped computer for maximum security.").size(self.scaled_text_size(14)),
                    text("All keys are generated offline and stored on encrypted SD cards.").size(self.scaled_text_size(14)),
                ]
                .spacing((5.0 * self.ui_scale) as u16)
            )
            .style(CowboyCustomTheme::pastel_coral_card())
            .padding(self.scaled_padding(20)),

            container(
                column![
                    text("Getting Started").size(self.scaled_text_size(20)),
                    text("Choose how you want to proceed:").size(self.scaled_text_size(14)),

                    row![
                        button("Import from Secrets")
                            .on_press(Message::ImportFromSecrets)
                            .style(CowboyCustomTheme::security_button()),
                        text("Load cowboyai.json and secrets.json files")
                            .size(self.scaled_text_size(12))
                            .color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(self.scaled_padding(10))
                    .align_y(Alignment::Center),

                    row![
                        button("Go to Organization")
                            .on_press(Message::TabSelected(Tab::Organization))
                            .style(CowboyCustomTheme::primary_button()),
                        text("Manually create your organization")
                            .size(self.scaled_text_size(12))
                            .color(CowboyTheme::text_secondary()),
                    ]
                    .spacing(self.scaled_padding(10))
                    .align_y(Alignment::Center),
                ]
                .spacing(self.scaled_padding(15))
            )
            .padding(self.scaled_padding(20))
            .style(CowboyCustomTheme::glass_container()),
        ]
        .spacing(self.scaled_padding(20))
        .padding(self.scaled_padding(10));

        scrollable(content).into()
    }

    fn view_organization(&self) -> Element<'_, Message> {
        use graph::view_graph;

        let role_options = vec![
            KeyOwnerRole::RootAuthority,
            KeyOwnerRole::SecurityAdmin,
            KeyOwnerRole::Developer,
            KeyOwnerRole::ServiceAccount,
            KeyOwnerRole::BackupHolder,
            KeyOwnerRole::Auditor,
        ];

        let content = column![
            text("Organization Structure").size(self.scaled_text_size(20)),
            text("Visualize and manage your organization's key ownership hierarchy").size(self.scaled_text_size(14)),

            // Debug: Show graph state
            text(format!("Graph State: {} nodes, {} edges",
                self.org_graph.nodes.len(),
                self.org_graph.edges.len()))
                .size(self.scaled_text_size(12))
                .color(Color::from_rgb(0.5, 0.5, 0.5)),

            // Domain creation/loading form (shows only if domain not loaded)
            if !self.domain_loaded {
                container(
                    column![
                        text("Create or Load Domain")
                            .size(self.scaled_text_size(16))
                            .color(CowboyTheme::text_primary()),
                        row![
                            text_input("Organization Name", &self.organization_name)
                                .on_input(Message::OrganizationNameChanged)
                                .size(self.scaled_text_size(16))
                                .style(CowboyCustomTheme::glass_input()),
                            text_input("Domain", &self.organization_domain)
                                .on_input(Message::OrganizationDomainChanged)
                                .size(self.scaled_text_size(16))
                                .style(CowboyCustomTheme::glass_input()),
                        ]
                        .spacing(self.scaled_padding(10)),

                        text("Master Passphrase (encrypts SD card storage)")
                            .size(self.scaled_text_size(14))
                            .color(CowboyTheme::text_primary()),
                        row![
                            text_input("Master Passphrase", &self.master_passphrase)
                                .on_input(Message::MasterPassphraseChanged)
                                .size(self.scaled_text_size(16))
                                .secure(true)
                                .style(CowboyCustomTheme::glass_input()),
                            text_input("Confirm Passphrase", &self.master_passphrase_confirm)
                                .on_input(Message::MasterPassphraseConfirmChanged)
                                .size(self.scaled_text_size(16))
                                .secure(true)
                                .style(CowboyCustomTheme::glass_input()),
                        ]
                        .spacing(self.scaled_padding(10)),

                        // Passphrase validation feedback
                        if !self.master_passphrase.is_empty() || !self.master_passphrase_confirm.is_empty() {
                            let passphrase_matches = self.master_passphrase == self.master_passphrase_confirm;
                            let passphrase_strong = self.master_passphrase.len() >= 12;
                            let has_uppercase = self.master_passphrase.chars().any(|c| c.is_uppercase());
                            let has_lowercase = self.master_passphrase.chars().any(|c| c.is_lowercase());
                            let has_digit = self.master_passphrase.chars().any(|c| c.is_numeric());
                            let has_special = self.master_passphrase.chars().any(|c| !c.is_alphanumeric());

                            column![
                                if passphrase_strong {
                                    text("✅ Length: 12+ characters").size(self.scaled_text_size(12)).color(Color::from_rgb(0.0, 0.6, 0.0))
                                } else {
                                    text(format!("❌ Length: {}/12 characters", self.master_passphrase.len())).size(self.scaled_text_size(12)).color(Color::from_rgb(0.8, 0.0, 0.0))
                                },
                                if has_uppercase {
                                    text("✅ Contains uppercase letter").size(self.scaled_text_size(12)).color(Color::from_rgb(0.0, 0.6, 0.0))
                                } else {
                                    text("❌ Needs uppercase letter").size(self.scaled_text_size(12)).color(Color::from_rgb(0.8, 0.0, 0.0))
                                },
                                if has_lowercase {
                                    text("✅ Contains lowercase letter").size(self.scaled_text_size(12)).color(Color::from_rgb(0.0, 0.6, 0.0))
                                } else {
                                    text("❌ Needs lowercase letter").size(self.scaled_text_size(12)).color(Color::from_rgb(0.8, 0.0, 0.0))
                                },
                                if has_digit {
                                    text("✅ Contains number").size(self.scaled_text_size(12)).color(Color::from_rgb(0.0, 0.6, 0.0))
                                } else {
                                    text("❌ Needs number").size(self.scaled_text_size(12)).color(Color::from_rgb(0.8, 0.0, 0.0))
                                },
                                if has_special {
                                    text("✅ Contains special character").size(self.scaled_text_size(12)).color(Color::from_rgb(0.0, 0.6, 0.0))
                                } else {
                                    text("❌ Needs special character").size(self.scaled_text_size(12)).color(Color::from_rgb(0.8, 0.0, 0.0))
                                },
                                if passphrase_matches && !self.master_passphrase.is_empty() {
                                    text("✅ Passphrases match").size(self.scaled_text_size(12)).color(Color::from_rgb(0.0, 0.6, 0.0))
                                } else if !self.master_passphrase_confirm.is_empty() {
                                    text("❌ Passphrases do not match").size(self.scaled_text_size(12)).color(Color::from_rgb(0.8, 0.0, 0.0))
                                } else {
                                    text("").size(self.scaled_text_size(12))
                                },
                            ]
                            .spacing(self.scaled_padding(5))
                        } else {
                            column![].spacing(0)
                        },

                        {
                            // Validate passphrase before allowing domain creation
                            let passphrase_valid = {
                                let matches = self.master_passphrase == self.master_passphrase_confirm;
                                let long_enough = self.master_passphrase.len() >= 12;
                                let has_upper = self.master_passphrase.chars().any(|c| c.is_uppercase());
                                let has_lower = self.master_passphrase.chars().any(|c| c.is_lowercase());
                                let has_digit = self.master_passphrase.chars().any(|c| c.is_numeric());
                                let has_special = self.master_passphrase.chars().any(|c| !c.is_alphanumeric());

                                matches && long_enough && has_upper && has_lower && has_digit && has_special
                            };

                            let create_button = if passphrase_valid {
                                button("Create New Domain")
                                    .on_press(Message::CreateNewDomain)
                                    .style(CowboyCustomTheme::primary_button())
                            } else {
                                button("Create New Domain")
                                    .style(CowboyCustomTheme::glass_button())
                            };

                            row![
                                button("Load Existing Domain")
                                    .on_press(Message::LoadExistingDomain)
                                    .style(CowboyCustomTheme::glass_button()),
                                create_button,
                            ]
                            .spacing(self.scaled_padding(10))
                        },
                    ]
                    .spacing(self.scaled_padding(10))
                )
                .padding(self.scaled_padding(15))
                .style(CowboyCustomTheme::pastel_cream_card())
            } else {
                container(
                    column![
                        text(format!("✅ Domain: {} ({})", self.organization_name, self.organization_domain))
                            .size(self.scaled_text_size(16))
                            .color(CowboyTheme::text_primary()),
                    ]
                )
                .padding(self.scaled_padding(15))
                .style(CowboyCustomTheme::pastel_cream_card())
            },


            // Add person form
            container(
                column![
                    text("Add Person to Organization")
                        .size(self.scaled_text_size(16))
                        .color(CowboyTheme::text_primary()),
                    row![
                        text_input("Name", &self.new_person_name)
                            .on_input(Message::NewPersonNameChanged)
                            .size(self.scaled_text_size(14))
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("Email", &self.new_person_email)
                            .on_input(Message::NewPersonEmailChanged)
                            .size(self.scaled_text_size(14))
                            .style(CowboyCustomTheme::glass_input()),
                        pick_list(
                            role_options,
                            self.new_person_role,
                            Message::NewPersonRoleSelected,
                        )
                        .placeholder("Select Role"),
                        button("Add Person")
                            .on_press(Message::AddPerson)
                            .style(CowboyCustomTheme::primary_button())
                    ]
                    .spacing(self.scaled_padding(10)),
                ]
                .spacing(self.scaled_padding(10))
            )
            .padding(self.scaled_padding(15))
            .style(CowboyCustomTheme::pastel_teal_card()),

            // Graph visualization with overlays (using stack for absolute positioning)
            {
                let graph_base = Container::new(
                    view_graph(&self.org_graph)
                        .map(Message::GraphMessage)
                )
                .width(Length::Fill)
                .height(Length::Fixed(600.0))  // Explicit height so canvas is actually visible and clickable
                .style(|_theme| {
                    container::Style {
                        background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                        border: Border {
                            color: Color::from_rgb(0.7, 0.7, 0.7),
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        ..Default::default()
                    }
                });

                // Build stack with overlays
                let mut stack_layers = vec![graph_base.into()];

                // Add context menu overlay if visible
                if self.context_menu.is_visible() {
                    let menu_overlay = container(
                        self.context_menu.view()
                            .map(Message::ContextMenuMessage)
                    )
                    .padding(0);

                    stack_layers.push(menu_overlay.into());
                }

                // Add property card overlay if visible
                if self.property_card.is_editing() {
                    // Position card in top-right without blocking the rest of the canvas
                    let card_overlay = container(
                        row![
                            // Spacer to push card to the right
                            horizontal_space(),
                            // The actual property card
                            self.property_card.view()
                                .map(Message::PropertyCardMessage)
                        ]
                    )
                    .padding(20);

                    stack_layers.push(card_overlay.into());
                }

                stack(stack_layers)
            },
        ]
        .spacing(self.scaled_padding(20))
        .height(Length::Fill);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_locations(&self) -> Element<'_, Message> {
        // TODO: Re-implement location type picker once Display is implemented in cim-domain-location

        let content = column![
            text("Locations Management").size(self.scaled_text_size(20)),
            text("Manage corporate locations where keys and certificates are stored").size(self.scaled_text_size(14)),

            // Add location form
            container(
                column![
                    text("Add New Physical Location")
                        .size(self.scaled_text_size(16))
                        .color(CowboyTheme::text_primary()),
                    text("Default location type: Physical")
                        .size(self.scaled_text_size(12))
                        .color(CowboyTheme::text_secondary()),

                    // Location name
                    text_input("Location Name (e.g., Main Office, HQ)", &self.new_location_name)
                        .on_input(Message::NewLocationNameChanged)
                        .size(self.scaled_text_size(14))
                        .style(CowboyCustomTheme::glass_input()),

                    // Address section
                    text("Address").size(self.scaled_text_size(14)).color(CowboyTheme::text_primary()),
                    text_input("Street Address", &self.new_location_street)
                        .on_input(Message::NewLocationStreetChanged)
                        .size(self.scaled_text_size(14))
                        .style(CowboyCustomTheme::glass_input()),

                    row![
                        text_input("City", &self.new_location_city)
                            .on_input(Message::NewLocationCityChanged)
                            .size(self.scaled_text_size(14))
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("State/Region", &self.new_location_region)
                            .on_input(Message::NewLocationRegionChanged)
                            .size(self.scaled_text_size(14))
                            .style(CowboyCustomTheme::glass_input()),
                    ]
                    .spacing(self.scaled_padding(10)),

                    row![
                        text_input("Country", &self.new_location_country)
                            .on_input(Message::NewLocationCountryChanged)
                            .size(self.scaled_text_size(14))
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("Postal Code", &self.new_location_postal)
                            .on_input(Message::NewLocationPostalChanged)
                            .size(self.scaled_text_size(14))
                            .style(CowboyCustomTheme::glass_input()),
                    ]
                    .spacing(self.scaled_padding(10)),

                    button("Add Location")
                        .on_press(Message::AddLocation)
                        .style(CowboyCustomTheme::primary_button())
                ]
                .spacing(self.scaled_padding(10))
            )
            .padding(self.scaled_padding(15))
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
                                    .size(self.scaled_text_size(14))
                                    .color(CowboyTheme::text_secondary()),
                            ]
                        )
                        .padding(self.scaled_padding(15))
                        .style(CowboyCustomTheme::pastel_cream_card())
                    } else {
                        let mut location_list = column![].spacing(self.scaled_padding(10));

                        for location in locations {
                            location_list = location_list.push(
                                container(
                                    row![
                                        column![
                                            text(location.name)
                                                .size(self.scaled_text_size(16))
                                                .color(CowboyTheme::text_primary()),
                                            text(format!("Type: {}",
                                                location.location_type))
                                                .size(self.scaled_text_size(12))
                                                .color(CowboyTheme::text_secondary()),
                                            text(format!("Created: {}",
                                                location.created_at.format("%Y-%m-%d %H:%M UTC")))
                                                .size(self.scaled_text_size(11))
                                                .color(CowboyTheme::text_secondary()),
                                        ]
                                        .spacing(self.scaled_padding(5)),
                                        horizontal_space(),
                                        button("Remove")
                                            .on_press(Message::RemoveLocation(location.location_id))
                                            .style(CowboyCustomTheme::security_button())
                                    ]
                                    .align_y(Alignment::Center)
                                    .spacing(self.scaled_padding(10))
                                )
                                .padding(self.scaled_padding(12))
                                .style(CowboyCustomTheme::pastel_teal_card())
                            );
                        }

                        container(location_list)
                            .padding(self.scaled_padding(15))
                            .style(CowboyCustomTheme::pastel_cream_card())
                    }
                } else {
                    container(
                        text("Loading locations...")
                            .size(self.scaled_text_size(14))
                            .color(CowboyTheme::text_secondary())
                    )
                    .padding(self.scaled_padding(15))
                    .style(CowboyCustomTheme::pastel_cream_card())
                };

                location_container
            },
        ]
        .spacing(self.scaled_padding(20));

        scrollable(content).into()
    }

    fn view_keys(&self) -> Element<'_, Message> {
        let progress_percentage = self.key_generation_progress * 100.0;

        let content = column![
            text("Generate Keys for Organization").size(self.scaled_text_size(24)),
            text("Generate cryptographic keys for all organization members").size(self.scaled_text_size(15)),

            // Root Passphrase Section
            container(
                column![
                    text("Root Passphrase (Required for PKI Operations)")
                        .size(self.scaled_text_size(18))
                        .color(CowboyTheme::text_primary()),
                    text_input("Root Passphrase", &self.root_passphrase)
                        .on_input(Message::RootPassphraseChanged)
                        .secure(true)
                        .size(self.scaled_text_size(16))
                        .padding(self.scaled_padding(12))
                        .style(CowboyCustomTheme::glass_input()),
                    text_input("Confirm Root Passphrase", &self.root_passphrase_confirm)
                        .on_input(Message::RootPassphraseConfirmChanged)
                        .secure(true)
                        .size(self.scaled_text_size(16))
                        .padding(self.scaled_padding(12))
                        .style(CowboyCustomTheme::glass_input()),
                    if !self.root_passphrase.is_empty() && self.root_passphrase == self.root_passphrase_confirm {
                        text("✓ Passphrases match")
                            .size(self.scaled_text_size(14))
                            .color(Color::from_rgb(0.2, 0.9, 0.2))
                    } else if !self.root_passphrase.is_empty() && !self.root_passphrase_confirm.is_empty() {
                        text("✗ Passphrases do not match")
                            .size(self.scaled_text_size(14))
                            .color(Color::from_rgb(0.9, 0.2, 0.2))
                    } else {
                        text("")
                    }
                ]
                .spacing(self.scaled_padding(12))
            )
            .padding(self.scaled_padding(20))
            .style(CowboyCustomTheme::pastel_coral_card()),

            container(
                column![
                    text("PKI Hierarchy Generation")
                        .size(self.scaled_text_size(18))
                        .color(CowboyTheme::text_primary()),

                    // Root CA Section
                    text("1. Root CA (Trust Anchor)").size(self.scaled_text_size(16)),
                    button("Generate Root CA")
                        .on_press(Message::GenerateRootCA)
                        .padding(self.scaled_padding(14))
                        .style(CowboyCustomTheme::security_button()),

                    // Intermediate CA Section
                    text("2. Intermediate CA (Signing-Only, pathlen:0)").size(self.scaled_text_size(16)),
                    text_input("CA Name (e.g., 'Engineering')", &self.intermediate_ca_name_input)
                        .on_input(Message::IntermediateCANameChanged)
                        .size(self.scaled_text_size(16))
                        .padding(self.scaled_padding(12))
                        .style(CowboyCustomTheme::glass_input()),
                    button("Generate Intermediate CA")
                        .on_press(Message::GenerateIntermediateCA)
                        .padding(self.scaled_padding(14))
                        .style(CowboyCustomTheme::primary_button()),

                    // Server Certificate Section
                    text("3. Server Certificates").size(self.scaled_text_size(16)),
                    text_input("Common Name (e.g., 'nats.example.com')", &self.server_cert_cn_input)
                        .on_input(Message::ServerCertCNChanged)
                        .size(self.scaled_text_size(16))
                        .padding(self.scaled_padding(12))
                        .style(CowboyCustomTheme::glass_input()),
                    text_input("SANs (comma-separated DNS names or IPs)", &self.server_cert_sans_input)
                        .on_input(Message::ServerCertSANsChanged)
                        .size(self.scaled_text_size(16))
                        .padding(self.scaled_padding(12))
                        .style(CowboyCustomTheme::glass_input()),

                    // CA selection picker
                    if !self.mvi_model.key_generation_status.intermediate_cas.is_empty() {
                        let ca_names: Vec<String> = self.mvi_model.key_generation_status.intermediate_cas
                            .iter()
                            .map(|ca| ca.name.clone())
                            .collect();

                        row![
                            text("Signing CA:").size(self.scaled_text_size(14)),
                            pick_list(
                                ca_names,
                                self.selected_intermediate_ca.clone(),
                                Message::SelectIntermediateCA,
                            )
                            .placeholder("Select Intermediate CA")
                        ]
                        .spacing(self.scaled_padding(10))
                    } else {
                        row![
                            text("Signing CA:").size(self.scaled_text_size(14)).color(Color::from_rgb(0.6, 0.6, 0.6)),
                            text("(generate an intermediate CA first)").size(self.scaled_text_size(14)).color(Color::from_rgb(0.6, 0.6, 0.6)),
                        ]
                        .spacing(self.scaled_padding(10))
                    },

                    button("Generate Server Certificate")
                        .on_press(Message::GenerateServerCert)
                        .style(CowboyCustomTheme::primary_button()),

                    // Display generated certificates from MVI model
                    if !self.mvi_model.key_generation_status.intermediate_cas.is_empty()
                       || !self.mvi_model.key_generation_status.server_certificates.is_empty() {
                        container(
                            column![
                                text("Generated Certificates").size(self.scaled_text_size(16)).color(Color::from_rgb(0.3, 0.8, 0.3)),

                                // Intermediate CAs
                                if !self.mvi_model.key_generation_status.intermediate_cas.is_empty() {
                                    iced::widget::Column::with_children(
                                        self.mvi_model.key_generation_status.intermediate_cas.iter().map(|ca| {
                                            text(format!("  ✓ CA: {} - {}", ca.name, &ca.fingerprint[..16]))
                                                .size(self.scaled_text_size(12))
                                                .color(Color::from_rgb(0.3, 0.8, 0.3))
                                                .into()
                                        }).collect::<Vec<_>>()
                                    )
                                    .spacing((3.0 * self.ui_scale) as u16)
                                } else {
                                    column![]
                                },

                                // Server Certificates
                                if !self.mvi_model.key_generation_status.server_certificates.is_empty() {
                                    iced::widget::Column::with_children(
                                        self.mvi_model.key_generation_status.server_certificates.iter().map(|cert| {
                                            column![
                                                text(format!("  ✓ Server: {} (signed by: {})", cert.common_name, cert.signed_by))
                                                    .size(self.scaled_text_size(12))
                                                    .color(Color::from_rgb(0.3, 0.8, 0.3)),
                                                text(format!("    Fingerprint: {}", &cert.fingerprint[..16]))
                                                    .size(self.scaled_text_size(11))
                                                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                                            ]
                                            .spacing((2.0 * self.ui_scale) as u16)
                                            .into()
                                        }).collect::<Vec<_>>()
                                    )
                                    .spacing((5.0 * self.ui_scale) as u16)
                                } else {
                                    column![]
                                },
                            ]
                            .spacing(self.scaled_padding(10))
                        )
                        .padding(self.scaled_padding(10))
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
                                    .padding(self.scaled_padding(8))
                                    .style(CowboyCustomTheme::glass_button()),
                                text("4. YubiKey Hardware Detection")
                                    .size(self.scaled_text_size(18))
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.scaled_padding(10))
                            .align_y(Alignment::Center),

                            if !self.yubikey_section_collapsed {
                                column![
                                    row![
                                        button("Detect YubiKeys")
                                            .on_press(Message::DetectYubiKeys)
                                            .padding(self.scaled_padding(14))
                                            .style(CowboyCustomTheme::security_button()),
                                        text(&self.yubikey_detection_status)
                                            .size(self.scaled_text_size(14))
                                            .color(CowboyTheme::text_secondary()),
                                    ]
                                    .spacing(self.scaled_padding(10))
                                    .align_y(Alignment::Center),

                    // Display detected YubiKeys
                    if !self.detected_yubikeys.is_empty() {
                        let mut yubikey_list = column![].spacing(self.scaled_padding(5));

                        for device in &self.detected_yubikeys {
                            yubikey_list = yubikey_list.push(
                                row![
                                    text(format!("  ✓ {} v{} - Serial: {}",
                                        device.model,
                                        device.version,
                                        device.serial))
                                        .size(self.scaled_text_size(12))
                                        .color(if device.piv_enabled {
                                            Color::from_rgb(0.3, 0.8, 0.3)
                                        } else {
                                            Color::from_rgb(0.8, 0.5, 0.2)
                                        }),
                                    if !device.piv_enabled {
                                        text(" (PIV not enabled)")
                                            .size(self.scaled_text_size(11))
                                            .color(Color::from_rgb(0.8, 0.5, 0.2))
                                    } else {
                                        text("")
                                    }
                                ]
                                .spacing(self.scaled_padding(5))
                            );
                        }

                        container(yubikey_list)
                            .padding(self.scaled_padding(10))
                            .style(CowboyCustomTheme::card_container())
                    } else {
                        container(text(""))
                    },

                    // YubiKey Configurations (imported from secrets)
                    if !self.yubikey_configs.is_empty() {
                        text(format!("🔑 YubiKey Configurations ({} imported)", self.yubikey_configs.len()))
                            .size(self.scaled_text_size(14))
                            .color(CowboyTheme::text_primary())
                    } else {
                        text("")
                    },

                    if !self.yubikey_configs.is_empty() {
                        let mut config_list = column![].spacing(self.scaled_padding(8));

                        for config in &self.yubikey_configs {
                            let role_str = match config.role {
                                crate::domain::YubiKeyRole::RootCA => "🔐 Root CA",
                                crate::domain::YubiKeyRole::Backup => "💾 Backup",
                                crate::domain::YubiKeyRole::User => "👤 User",
                                crate::domain::YubiKeyRole::Service => "⚙️ Service",
                            };

                            config_list = config_list.push(
                                container(
                                    column![
                                        text(format!("{} - {}", role_str, config.name))
                                            .size(self.scaled_text_size(13))
                                            .color(CowboyTheme::text_primary()),
                                        row![
                                            column![
                                                text(format!("Serial: {}", config.serial))
                                                    .size(self.scaled_text_size(11))
                                                    .color(CowboyTheme::text_secondary()),
                                                text(format!("Owner: {}", config.owner_email))
                                                    .size(self.scaled_text_size(11))
                                                    .color(CowboyTheme::text_secondary()),
                                            ]
                                            .spacing(self.scaled_padding(2)),
                                            horizontal_space(),
                                            column![
                                                text(format!("PIN: {}", config.piv.pin))
                                                    .size(self.scaled_text_size(11))
                                                    .color(Color::from_rgb(0.8, 0.6, 0.2)),
                                                text(format!("PUK: {}", config.piv.puk))
                                                    .size(self.scaled_text_size(11))
                                                    .color(Color::from_rgb(0.8, 0.6, 0.2)),
                                                text(format!("Mgmt: {}...", &config.piv.mgmt_key[..12]))
                                                    .size(self.scaled_text_size(10))
                                                    .color(Color::from_rgb(0.8, 0.6, 0.2)),
                                            ]
                                            .spacing(self.scaled_padding(2))
                                            .align_x(iced::alignment::Horizontal::Right),
                                        ]
                                        .align_y(Alignment::Center),
                                    ]
                                    .spacing(self.scaled_padding(5))
                                )
                                .padding(self.scaled_padding(10))
                                .style(CowboyCustomTheme::pastel_coral_card())
                            );
                        }

                        container(config_list)
                            .padding(self.scaled_padding(10))
                            .style(CowboyCustomTheme::card_container())
                    } else {
                        container(text(""))
                    },
                                ]
                                .spacing(self.scaled_padding(12))
                            } else {
                                column![]
                            }
                        ]
                        .spacing(self.scaled_padding(12))
                    )
                    .padding(self.scaled_padding(20))
                    .style(CowboyCustomTheme::pastel_mint_card()),

                    // Loaded Certificates from Manifest (Collapsible)
                    if !self.loaded_certificates.is_empty() {
                        container(
                            column![
                                row![
                                    button(if self.certificates_collapsed { "▶" } else { "▼" })
                                        .on_press(Message::ToggleCertificatesSection)
                                        .padding(self.scaled_padding(8))
                                        .style(CowboyCustomTheme::glass_button()),
                                    text(format!("📜 Certificates from Manifest ({} loaded)", self.loaded_certificates.len()))
                                        .size(self.scaled_text_size(16))
                                        .color(CowboyTheme::text_primary()),
                                ]
                                .spacing(self.scaled_padding(10))
                                .align_y(Alignment::Center),

                                if !self.certificates_collapsed {
                                    {
                                        let mut cert_list = column![].spacing(self.scaled_padding(6));

                                        for cert in &self.loaded_certificates {
                                            cert_list = cert_list.push(
                                                container(
                                                    column![
                                                        text(format!("🔐 {}{}", cert.subject, if cert.is_ca { " (CA)" } else { "" }))
                                                            .size(self.scaled_text_size(12))
                                                            .color(if cert.is_ca { Color::from_rgb(0.8, 0.3, 0.3) } else { Color::from_rgb(0.3, 0.6, 0.8) }),
                                                        row![
                                                            column![
                                                                text(format!("Serial: {}...", &cert.serial_number.chars().take(16).collect::<String>()))
                                                                    .size(self.scaled_text_size(10))
                                                                    .color(CowboyTheme::text_secondary()),
                                                                if let Some(ref issuer) = cert.issuer {
                                                                    text(format!("Issuer: {}", issuer))
                                                                        .size(self.scaled_text_size(10))
                                                                        .color(CowboyTheme::text_secondary())
                                                                } else {
                                                                    text("")
                                                                },
                                                            ]
                                                            .spacing(self.scaled_padding(2)),
                                                            horizontal_space(),
                                                            column![
                                                                text(format!("Valid: {} to {}",
                                                                    cert.not_before.format("%Y-%m-%d"),
                                                                    cert.not_after.format("%Y-%m-%d")))
                                                                    .size(self.scaled_text_size(10))
                                                                    .color(CowboyTheme::text_secondary()),
                                                            ]
                                                            .align_x(iced::alignment::Horizontal::Right),
                                                        ]
                                                    ]
                                                    .spacing(self.scaled_padding(3))
                                                )
                                                .padding(self.scaled_padding(8))
                                                .style(CowboyCustomTheme::pastel_teal_card())
                                            );
                                        }

                                        container(cert_list)
                                            .padding(self.scaled_padding(10))
                                            .style(CowboyCustomTheme::card_container())
                                    }
                                } else {
                                    container(text(""))
                                }
                            ]
                            .spacing(self.scaled_padding(12))
                        )
                        .padding(self.scaled_padding(20))
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
                                        .padding(self.scaled_padding(8))
                                        .style(CowboyCustomTheme::glass_button()),
                                    text(format!("🔑 Keys from Manifest ({} loaded)", self.loaded_keys.len()))
                                        .size(self.scaled_text_size(16))
                                        .color(CowboyTheme::text_primary()),
                                ]
                                .spacing(self.scaled_padding(10))
                                .align_y(Alignment::Center),

                                if !self.keys_collapsed {
                                    {
                                        let mut key_list = column![].spacing(self.scaled_padding(6));

                                        for key in &self.loaded_keys {
                            key_list = key_list.push(
                                container(
                                    column![
                                        text(format!("🔐 {} - {}", key.label, format!("{:?}", key.algorithm)))
                                            .size(self.scaled_text_size(12))
                                            .color(if key.revoked { Color::from_rgb(0.8, 0.3, 0.3) } else { Color::from_rgb(0.3, 0.8, 0.3) }),
                                        row![
                                            column![
                                                text(format!("Purpose: {:?}", key.purpose))
                                                    .size(self.scaled_text_size(10))
                                                    .color(CowboyTheme::text_secondary()),
                                                if key.hardware_backed {
                                                    if let Some(ref serial) = key.yubikey_serial {
                                                        text(format!("YubiKey: {}", serial))
                                                            .size(self.scaled_text_size(10))
                                                            .color(Color::from_rgb(0.3, 0.6, 0.8))
                                                    } else {
                                                        text("Hardware-backed")
                                                            .size(self.scaled_text_size(10))
                                                            .color(Color::from_rgb(0.3, 0.6, 0.8))
                                                    }
                                                } else {
                                                    text("Software key")
                                                        .size(self.scaled_text_size(10))
                                                        .color(CowboyTheme::text_secondary())
                                                },
                                            ]
                                            .spacing(self.scaled_padding(2)),
                                            horizontal_space(),
                                            column![
                                                text(format!("Created: {}", key.created_at.format("%Y-%m-%d")))
                                                    .size(self.scaled_text_size(10))
                                                    .color(CowboyTheme::text_secondary()),
                                                if key.revoked {
                                                    text("⚠️ REVOKED")
                                                        .size(self.scaled_text_size(10))
                                                        .color(Color::from_rgb(0.8, 0.3, 0.3))
                                                } else {
                                                    text("")
                                                },
                                            ]
                                            .align_x(iced::alignment::Horizontal::Right),
                                        ]
                                    ]
                                    .spacing(self.scaled_padding(3))
                                )
                                .padding(self.scaled_padding(8))
                                .style(CowboyCustomTheme::pastel_mint_card())
                            );
                        }

                                        container(key_list)
                                            .padding(self.scaled_padding(10))
                                            .style(CowboyCustomTheme::card_container())
                                    }
                                } else {
                                    container(text(""))
                                }
                            ]
                            .spacing(self.scaled_padding(12))
                        )
                        .padding(self.scaled_padding(20))
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
                                    .padding(self.scaled_padding(8))
                                    .style(CowboyCustomTheme::glass_button()),
                                text("5. NATS Hierarchy (Operator/Account/User)")
                                    .size(self.scaled_text_size(18))
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.scaled_padding(10))
                            .align_y(Alignment::Center),

                            if !self.nats_section_collapsed {
                                column![
                                    row![
                                        button("Generate NATS Hierarchy")
                                            .on_press(Message::GenerateNatsHierarchy)
                                            .padding(self.scaled_padding(14))
                                            .style(CowboyCustomTheme::security_button()),
                                        if self.nats_hierarchy_generated {
                                            text("✓ NATS hierarchy generated")
                                                .size(self.scaled_text_size(14))
                                                .color(Color::from_rgb(0.3, 0.8, 0.3))
                                        } else {
                                            text("Generate NATS operator, accounts, and users")
                                                .size(self.scaled_text_size(14))
                                                .color(CowboyTheme::text_secondary())
                                        }
                                    ]
                                    .spacing(self.scaled_padding(10))
                                    .align_y(Alignment::Center),
                                ]
                                .spacing(self.scaled_padding(12))
                            } else {
                                column![]
                            }
                        ]
                        .spacing(self.scaled_padding(12))
                    )
                    .padding(self.scaled_padding(20))
                    .style(CowboyCustomTheme::pastel_coral_card()),

                    // Step 6: Other Key Generation (Card with Collapse)
                    container(
                        column![
                            row![
                                button(if self.keys_collapsed { "▶" } else { "▼" })
                                    .on_press(Message::ToggleKeysSection)
                                    .padding(self.scaled_padding(8))
                                    .style(CowboyCustomTheme::glass_button()),
                                text("6. Other Keys")
                                    .size(self.scaled_text_size(18))
                                    .color(CowboyTheme::text_primary()),
                            ]
                            .spacing(self.scaled_padding(10))
                            .align_y(Alignment::Center),

                            if !self.keys_collapsed {
                                column![
                                    button("Generate SSH Keys for All")
                                        .on_press(Message::GenerateSSHKeys)
                                        .padding(self.scaled_padding(14))
                                        .style(CowboyCustomTheme::primary_button()),
                                    button("Provision YubiKeys")
                                        .on_press(Message::ProvisionYubiKey)
                                        .padding(self.scaled_padding(14))
                                        .style(CowboyCustomTheme::glass_button()),
                                    button("Generate All Keys")
                                        .on_press(Message::GenerateAllKeys)
                                        .padding(self.scaled_padding(14))
                                        .style(CowboyCustomTheme::security_button()),
                                ]
                                .spacing(self.scaled_padding(12))
                            } else {
                                column![]
                            }
                        ]
                        .spacing(self.scaled_padding(12))
                    )
                    .padding(self.scaled_padding(20))
                    .style(CowboyCustomTheme::pastel_mint_card()),
                ]
                .spacing(self.scaled_padding(10))
            )
            .padding(self.scaled_padding(15))
            .style(CowboyCustomTheme::pastel_mint_card()),

            if self.key_generation_progress > 0.0 {
                container(
                    column![
                        text(format!("Progress: {:.0}%", progress_percentage)).size(self.scaled_text_size(14)),
                        progress_bar(0.0..=1.0, self.key_generation_progress),
                        text(format!("{} of {} keys generated",
                            self.keys_generated,
                            self.total_keys_to_generate)).size(self.scaled_text_size(12)),
                    ]
                    .spacing((5.0 * self.ui_scale) as u16)
                )
                .padding(self.scaled_padding(10))
            } else {
                container(text("No key generation in progress").size(self.scaled_text_size(14)))
            }
        ]
        .spacing(self.scaled_padding(20))
        .padding(self.scaled_padding(10));

        scrollable(content).into()
    }

    fn view_export(&self) -> Element<'_, Message> {
        let content = column![
            text("Export Domain Configuration").size(self.scaled_text_size(20)),
            text("Export your domain configuration to encrypted storage").size(self.scaled_text_size(14)),

            container(
                column![
                    text("Export Options")
                        .size(self.scaled_text_size(16))
                        .color(CowboyTheme::text_primary()),
                    text_input("Output Directory", &self.export_path.display().to_string())
                        .on_input(Message::ExportPathChanged)
                        .style(CowboyCustomTheme::glass_input()),
                    checkbox("Include public keys", self.include_public_keys)
                        .on_toggle(Message::TogglePublicKeys),
                    checkbox("Include certificates", self.include_certificates)
                        .on_toggle(Message::ToggleCertificates),
                    checkbox("Generate NATS configuration", self.include_nats_config)
                        .on_toggle(Message::ToggleNatsConfig),
                    checkbox("Include private keys (requires password)", self.include_private_keys)
                        .on_toggle(Message::TogglePrivateKeys),
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
                .spacing(self.scaled_padding(10))
            )
            .padding(self.scaled_padding(15))
            .style(CowboyCustomTheme::pastel_cream_card()),

            button("Export to Encrypted SD Card")
                .on_press(Message::ExportToSDCard)
                .style(CowboyCustomTheme::security_button()),

            // NATS NSC Export
            if self.nats_hierarchy_generated {
                container(
                    column![
                        text("Export NATS Hierarchy to NSC")
                            .size(self.scaled_text_size(16))
                            .color(CowboyTheme::text_primary()),
                        text(format!("Export directory: {}", self.nats_export_path.display()))
                            .size(self.scaled_text_size(12))
                            .color(CowboyTheme::text_secondary()),
                        button("Export to NSC Store")
                            .on_press(Message::ExportToNsc)
                            .style(CowboyCustomTheme::primary_button()),
                    ]
                    .spacing(self.scaled_padding(10))
                )
                .padding(self.scaled_padding(15))
                .style(CowboyCustomTheme::pastel_teal_card())
            } else {
                container(
                    text("Generate NATS hierarchy first to enable NSC export")
                        .size(self.scaled_text_size(14))
                        .color(Color::from_rgb(0.6, 0.6, 0.6))
                )
                .padding(self.scaled_padding(15))
                .style(CowboyCustomTheme::card_container())
            },
        ]
        .spacing(self.scaled_padding(20))
        .padding(self.scaled_padding(10));

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
        command_id: cim_domain::CommandId::new(),
        key_type: "ed25519".to_string(),
        comment: "test@example.com".to_string(),
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
pub async fn run(output_dir: String) -> iced::Result {
    application("CIM Keys", CimKeysApp::update, CimKeysApp::view)
        .subscription(|app| app.subscription())
        .theme(|app| app.theme())
        .font(include_bytes!("../assets/fonts/NotoColorEmoji.ttf"))
        .run_with(|| CimKeysApp::new(output_dir))
}