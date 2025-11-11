//! Native/WASM GUI for offline key generation using Iced 0.13+
//!
//! This module provides a pure Rust GUI that can run both as a native
//! application and as a WASM application in the browser.

use iced::{
    application,
    widget::{button, column, container, row, text, text_input, Container, horizontal_space, pick_list, progress_bar, checkbox, scrollable, Space},
    Task, Element, Length, Color, Border, Font, Theme, Background,
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
    // Mock adapters
    adapters::{
        InMemoryStorageAdapter,
        MockX509Adapter,
        MockSshKeyAdapter,
        MockYubiKeyAdapter,
    },
};

pub mod graph;
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

use graph::{OrganizationGraph, GraphMessage};
use event_emitter::{CimEventEmitter, GuiEventSubscriber, InteractionType};
use cowboy_theme::{CowboyTheme, CowboyAppTheme as CowboyCustomTheme};
// use kuramoto_firefly_shader::KuramotoFireflyShader;
// use debug_firefly_shader::DebugFireflyShader;
use firefly_renderer::FireflyRenderer;

/// Main application state
pub struct CimKeysApp {
    // Tab navigation
    active_tab: Tab,

    // Domain configuration
    domain_loaded: bool,
    _domain_path: PathBuf,  // Reserved for domain persistence path
    organization_name: String,
    organization_domain: String,

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

    // YubiKey fields
    yubikey_serial: String,
    yubikey_assigned_to: Option<Uuid>,

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

    // Status
    status_message: String,
    error_message: Option<String>,

    // Animation
    animation_time: f32,
    firefly_shader: FireflyRenderer,

    // MVI Integration (Option A: Quick Integration)
    mvi_model: MviModel,
    storage_port: Arc<dyn StoragePort>,
    x509_port: Arc<dyn X509Port>,
    ssh_port: Arc<dyn SshKeyPort>,
    yubikey_port: Arc<dyn YubiKeyPort>,
}

/// Different tabs in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Welcome,
    Organization,
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
    DomainCreated(Result<String, String>),
    DomainLoaded(Result<BootstrapConfig, String>),

    // Organization form inputs
    OrganizationNameChanged(String),
    OrganizationDomainChanged(String),

    // People operations
    NewPersonNameChanged(String),
    NewPersonEmailChanged(String),
    NewPersonRoleSelected(KeyOwnerRole),
    AddPerson,
    RemovePerson(Uuid),
    SelectPerson(Uuid),

    // YubiKey operations
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

    // Status messages
    UpdateStatus(String),
    ShowError(String),
    ClearError,

    // Graph interactions
    GraphMessage(GraphMessage),

    // MVI Integration
    MviIntent(Intent),

    // Animation
    AnimationTick,
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
        let yubikey_port: Arc<dyn YubiKeyPort> = Arc::new(MockYubiKeyAdapter::default());

        // Initialize MVI model
        let mvi_model = MviModel::new(PathBuf::from(&output_dir));

        (
            Self {
                active_tab: Tab::Welcome,
                domain_loaded: false,
                _domain_path: PathBuf::from(&output_dir),
                organization_name: String::new(),
                organization_domain: String::new(),
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
                yubikey_serial: String::new(),
                yubikey_assigned_to: None,
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
                status_message: String::from("🔐 Welcome to CIM Keys - Offline Key Management System"),
                error_message: None,
                animation_time: 0.0,
                firefly_shader: FireflyRenderer::new(),
                // MVI integration
                mvi_model,
                storage_port,
                x509_port,
                ssh_port,
                yubikey_port,
            },
            Task::none(),
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
                    Tab::Organization => "Organization Structure and Key Ownership".to_string(),
                    Tab::Keys => "Generate Cryptographic Keys".to_string(),
                    Tab::Export => "Export Domain Configuration".to_string(),
                };
                Task::none()
            }

            // Domain operations
            Message::CreateNewDomain => {
                self.domain_loaded = false;
                self.active_tab = Tab::Organization;
                self.status_message = "Creating new domain - configure organization".to_string();
                Task::none()
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
                if self.new_person_name.is_empty() || self.new_person_email.is_empty() {
                    self.error_message = Some("Please enter name and email".to_string());
                    return Task::none();
                }

                let person_id = Uuid::now_v7();
                let person = Person {
                    id: person_id,
                    name: self.new_person_name.clone(),
                    email: self.new_person_email.clone(),
                    organization_id: Uuid::now_v7(), // TODO: Use actual org ID
                    unit_ids: vec![],
                    roles: vec![],
                    active: true,
                    created_at: chrono::Utc::now(),
                };

                let role = self.new_person_role.unwrap_or(KeyOwnerRole::Developer);

                // Add to graph for visualization
                self.org_graph.add_node(person.clone(), role);

                // Clear form fields
                self.new_person_name.clear();
                self.new_person_email.clear();
                self.new_person_role = None;
                self.status_message = format!("Added {} to organization", person.name);

                Task::none()
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

            // YubiKey operations
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
                // TODO: Implement YubiKey provisioning
                self.status_message = "YubiKey provisioning started".to_string();
                Task::none()
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
                        self.selected_person = Some(*id);
                        self.status_message = "Selected person in graph".to_string();
                    }
                    GraphMessage::AutoLayout => {
                        self.org_graph.auto_layout();
                        self.status_message = String::from("Graph layout updated");
                    }
                    GraphMessage::AddEdge { from, to, edge_type } => {
                        self.org_graph.add_edge(*from, *to, edge_type.clone());
                        self.status_message = String::from("Relationship added");
                    }
                    _ => {}
                }
                self.org_graph.handle_message(graph_msg);
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::{stack, shader};

        // Create a text-based logo since iced doesn't support SVG directly
        let logo_text = container(
            column![
                text("CIM").size(32).font(Font {
                    family: iced::font::Family::Monospace,
                    weight: iced::font::Weight::Bold,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                }),
                text("KEYS").size(24).font(Font {
                    family: iced::font::Family::Monospace,
                    weight: iced::font::Weight::Bold,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                }),
            ]
            .align_x(iced::Alignment::Center)
            .spacing(0)
        )
        .width(Length::Fixed(80.0))
        .height(Length::Fixed(80.0))
        .center(Length::Fixed(80.0))
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            let base_style = container::Style::default();
            container::Style {
                background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.2, 1.0))),
                border: Border {
                    color: palette.primary.strong.color,
                    width: 2.0,
                    radius: 8.0.into(),
                },
                text_color: Some(palette.primary.strong.color),
                ..base_style  // Use base_style instead of direct default()
            }
        });

        // Tab bar
        let tab_bar = row![
            button(text("Welcome").size(14))
                .on_press(Message::TabSelected(Tab::Welcome))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Welcome)),
            button(text("Organization").size(14))
                .on_press(Message::TabSelected(Tab::Organization))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Organization)),
            button(text("Keys").size(14))
                .on_press(Message::TabSelected(Tab::Keys))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Keys)),
            button(text("Export").size(14))
                .on_press(Message::TabSelected(Tab::Export))
                .style(|theme: &Theme, _| self.tab_button_style(theme, self.active_tab == Tab::Export)),
        ]
        .spacing(5);

        // Tab content
        let content = match self.active_tab {
            Tab::Welcome => self.view_welcome(),
            Tab::Organization => self.view_organization(),
            Tab::Keys => self.view_keys(),
            Tab::Export => self.view_export(),
        };

        // Error display
        let error_display = self.error_message.as_ref().map(|error| container(
                    row![
                        text(format!("❌ {}", error))
                            .size(14)
                            .color(CowboyTheme::text_primary()),
                        horizontal_space(),
                        button("✕")
                            .on_press(Message::ClearError)
                            .style(CowboyCustomTheme::glass_button())
                    ]
                    .padding(10)
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
            logo_text,
            column![
                text("CIM Keys - Offline Key Management System")
                    .size(24)
                    .color(CowboyTheme::text_primary()),
                text("The Cowboy AI Infrastructure")
                    .size(14)
                    .color(CowboyTheme::text_secondary()),
            ]
            .spacing(5),
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);

        let mut main_column = column![
            header,
            text(&self.status_message)
                .size(12)
                .color(CowboyTheme::text_secondary()),
            container(tab_bar)
                .padding(10)
                .style(CowboyCustomTheme::glass_container()),
        ]
        .spacing(10);

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
        use iced::time;
        use std::time::Duration;

        // Update animation at 30 FPS instead of 60 to reduce resource usage
        time::every(Duration::from_millis(33)).map(|_| Message::AnimationTick)
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
            text("Welcome to CIM Keys!").size(28),
            text("Generate and manage cryptographic keys for your CIM infrastructure").size(16),
            container(
                column![
                    text("⚠️ Security Notice").size(18),
                    text("This application should be run on an air-gapped computer for maximum security."),
                    text("All keys are generated offline and stored on encrypted SD cards."),
                ]
                .spacing(5)
            )
            .style(CowboyCustomTheme::card_container())
            .padding(20),

            if !self.domain_loaded {
                column![
                    text("Get Started").size(20),
                    row![
                        text_input("Organization", &self.organization_name)
                            .on_input(Message::OrganizationNameChanged)
                            .size(16)
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("Domain", &self.organization_domain)
                            .on_input(Message::OrganizationDomainChanged)
                            .size(16)
                            .style(CowboyCustomTheme::glass_input()),
                    ]
                    .spacing(10),
                    row![
                        button("Load Existing Domain")
                            .on_press(Message::LoadExistingDomain)
                            .style(CowboyCustomTheme::glass_button()),
                        button("Create New Domain")
                            .on_press(Message::CreateNewDomain)
                            .style(CowboyCustomTheme::primary_button()),
                    ]
                    .spacing(10),
                ]
                .spacing(15)
            } else {
                column![
                    text(format!("✅ Domain Loaded: {}", self.organization_name)).size(18),
                    text(format!("Domain: {}", self.organization_domain)).size(16),
                    button("Go to Organization").on_press(Message::TabSelected(Tab::Organization))
                ]
                .spacing(10)
            }
        ]
        .spacing(20)
        .padding(10);

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
            text("Organization Structure").size(20),
            text("Visualize and manage your organization's key ownership hierarchy").size(14),

            // Add person form
            container(
                column![
                    text("Add Person to Organization")
                        .size(16)
                        .color(CowboyTheme::text_primary()),
                    row![
                        text_input("Name", &self.new_person_name)
                            .on_input(Message::NewPersonNameChanged)
                            .size(14)
                            .style(CowboyCustomTheme::glass_input()),
                        text_input("Email", &self.new_person_email)
                            .on_input(Message::NewPersonEmailChanged)
                            .size(14)
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
                    .spacing(10),
                ]
                .spacing(10)
            )
            .padding(15)
            .style(CowboyCustomTheme::card_container()),

            // Graph visualization canvas
            Container::new(
                view_graph(&self.org_graph)
                    .map(Message::GraphMessage)
            )
            .width(Length::Fill)
            .height(Length::Fixed(500.0))
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
            }),
        ]
        .spacing(20);

        scrollable(content).into()
    }

    fn view_keys(&self) -> Element<'_, Message> {
        let progress_percentage = self.key_generation_progress * 100.0;

        let content = column![
            text("Generate Keys for Organization").size(20),
            text("Generate cryptographic keys for all organization members").size(14),

            container(
                column![
                    text("PKI Hierarchy Generation")
                        .size(16)
                        .color(CowboyTheme::text_primary()),

                    // Root CA Section
                    text("1. Root CA (Trust Anchor)").size(14),
                    button("Generate Root CA")
                        .on_press(Message::GenerateRootCA)
                        .style(CowboyCustomTheme::security_button()),

                    // Intermediate CA Section
                    text("2. Intermediate CA (Signing-Only, pathlen:0)").size(14),
                    row![
                        text_input("CA Name (e.g., 'Engineering')", &self.intermediate_ca_name_input)
                            .on_input(Message::IntermediateCANameChanged)
                            .size(14)
                            .style(CowboyCustomTheme::glass_input()),
                        button("Generate Intermediate CA")
                            .on_press(Message::GenerateIntermediateCA)
                            .style(CowboyCustomTheme::primary_button()),
                    ]
                    .spacing(10),

                    // Server Certificate Section
                    text("3. Server Certificates").size(14),
                    text_input("Common Name (e.g., 'nats.example.com')", &self.server_cert_cn_input)
                        .on_input(Message::ServerCertCNChanged)
                        .size(14)
                        .style(CowboyCustomTheme::glass_input()),
                    text_input("SANs (comma-separated DNS names or IPs)", &self.server_cert_sans_input)
                        .on_input(Message::ServerCertSANsChanged)
                        .size(14)
                        .style(CowboyCustomTheme::glass_input()),

                    // CA selection picker
                    if !self.mvi_model.key_generation_status.intermediate_cas.is_empty() {
                        let ca_names: Vec<String> = self.mvi_model.key_generation_status.intermediate_cas
                            .iter()
                            .map(|ca| ca.name.clone())
                            .collect();

                        row![
                            text("Signing CA:").size(14),
                            pick_list(
                                ca_names,
                                self.selected_intermediate_ca.clone(),
                                Message::SelectIntermediateCA,
                            )
                            .placeholder("Select Intermediate CA")
                        ]
                        .spacing(10)
                    } else {
                        row![
                            text("Signing CA:").size(14).color(Color::from_rgb(0.6, 0.6, 0.6)),
                            text("(generate an intermediate CA first)").size(14).color(Color::from_rgb(0.6, 0.6, 0.6)),
                        ]
                        .spacing(10)
                    },

                    button("Generate Server Certificate")
                        .on_press(Message::GenerateServerCert)
                        .style(CowboyCustomTheme::primary_button()),

                    // Display generated certificates from MVI model
                    if !self.mvi_model.key_generation_status.intermediate_cas.is_empty()
                       || !self.mvi_model.key_generation_status.server_certificates.is_empty() {
                        container(
                            column![
                                text("Generated Certificates").size(16).color(Color::from_rgb(0.3, 0.8, 0.3)),

                                // Intermediate CAs
                                if !self.mvi_model.key_generation_status.intermediate_cas.is_empty() {
                                    iced::widget::Column::with_children(
                                        self.mvi_model.key_generation_status.intermediate_cas.iter().map(|ca| {
                                            text(format!("  ✓ CA: {} - {}", ca.name, &ca.fingerprint[..16]))
                                                .size(12)
                                                .color(Color::from_rgb(0.3, 0.8, 0.3))
                                                .into()
                                        }).collect::<Vec<_>>()
                                    )
                                    .spacing(3)
                                } else {
                                    column![]
                                },

                                // Server Certificates
                                if !self.mvi_model.key_generation_status.server_certificates.is_empty() {
                                    iced::widget::Column::with_children(
                                        self.mvi_model.key_generation_status.server_certificates.iter().map(|cert| {
                                            column![
                                                text(format!("  ✓ Server: {} (signed by: {})", cert.common_name, cert.signed_by))
                                                    .size(12)
                                                    .color(Color::from_rgb(0.3, 0.8, 0.3)),
                                                text(format!("    Fingerprint: {}", &cert.fingerprint[..16]))
                                                    .size(11)
                                                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                                            ]
                                            .spacing(2)
                                            .into()
                                        }).collect::<Vec<_>>()
                                    )
                                    .spacing(5)
                                } else {
                                    column![]
                                },
                            ]
                            .spacing(10)
                        )
                        .padding(10)
                        .style(CowboyCustomTheme::card_container())
                    } else {
                        container(text(""))
                    },

                    // Other Key Generation
                    text("4. Other Keys").size(14),
                    button("Generate SSH Keys for All")
                        .on_press(Message::GenerateSSHKeys)
                        .style(CowboyCustomTheme::primary_button()),
                    button("Provision YubiKeys")
                        .on_press(Message::ProvisionYubiKey)
                        .style(CowboyCustomTheme::glass_button()),
                    button("Generate All Keys")
                        .on_press(Message::GenerateAllKeys)
                        .style(CowboyCustomTheme::security_button()),
                ]
                .spacing(10)
            )
            .padding(15)
            .style(CowboyCustomTheme::card_container()),

            if self.key_generation_progress > 0.0 {
                container(
                    column![
                        text(format!("Progress: {:.0}%", progress_percentage)).size(14),
                        progress_bar(0.0..=1.0, self.key_generation_progress),
                        text(format!("{} of {} keys generated",
                            self.keys_generated,
                            self.total_keys_to_generate)).size(12),
                    ]
                    .spacing(5)
                )
                .padding(10)
            } else {
                container(text("No key generation in progress").size(14))
            }
        ]
        .spacing(20)
        .padding(10);

        scrollable(content).into()
    }

    fn view_export(&self) -> Element<'_, Message> {
        let content = column![
            text("Export Domain Configuration").size(20),
            text("Export your domain configuration to encrypted storage").size(14),

            container(
                column![
                    text("Export Options")
                        .size(16)
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
                .spacing(10)
            )
            .padding(15)
            .style(CowboyCustomTheme::card_container()),

            button("Export to Encrypted SD Card")
                .on_press(Message::ExportToSDCard)
                .style(CowboyCustomTheme::security_button())
        ]
        .spacing(20)
        .padding(10);

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
        .run_with(|| CimKeysApp::new(output_dir))
}