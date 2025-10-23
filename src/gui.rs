//! Native/WASM GUI for offline key generation using Iced 0.13+
//!
//! This module provides a pure Rust GUI that can run both as a native
//! application and as a WASM application in the browser.

use iced::{
    application, executor,
    widget::{button, column, container, row, text, text_input, Column, Container, horizontal_space, pick_list, progress_bar, checkbox, scrollable},
    Task, Element, Length, Theme, Color, Background, Border, Font, Padding,
};
use iced_futures::Subscription;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    aggregate::KeyManagementAggregate,
    domain::{Organization, Person, Location, KeyOwnerRole},
    projections::OfflineKeyProjection,
};

pub mod graph;
pub mod event_emitter;

use graph::{OrganizationGraph, GraphMessage};
use event_emitter::{CimEventEmitter, GuiEventSubscriber, InteractionType};

/// Main application state
pub struct CimKeysApp {
    // Tab navigation
    active_tab: Tab,

    // Domain configuration
    domain_loaded: bool,
    domain_path: PathBuf,
    organization_name: String,
    organization_domain: String,

    bootstrap_config: Option<BootstrapConfig>,
    aggregate: Arc<RwLock<KeyManagementAggregate>>,
    projection: Arc<RwLock<OfflineKeyProjection>>,

    // Event-driven communication
    event_emitter: CimEventEmitter,
    event_subscriber: GuiEventSubscriber,

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
    certificates_generated: usize,

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
}

/// Different tabs in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
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
}

/// Bootstrap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub organization: Organization,
    pub people: Vec<Person>,
    pub locations: Vec<Location>,
    pub yubikey_assignments: Vec<YubiKeyAssignment>,
    pub nats_hierarchy: NatsHierarchy,
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

        (
            Self {
                active_tab: Tab::Welcome,
                domain_loaded: false,
                domain_path: PathBuf::from(&output_dir),
                organization_name: String::new(),
                organization_domain: String::new(),
                bootstrap_config: None,
                aggregate,
                projection,
                event_emitter: CimEventEmitter::new(default_org),
                event_subscriber: GuiEventSubscriber::new(default_org),
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
                certificates_generated: 0,
                export_path: PathBuf::from(&output_dir),
                include_public_keys: true,
                include_certificates: true,
                include_nats_config: true,
                include_private_keys: false,
                export_password: String::new(),
                status_message: String::from("🔐 Welcome to CIM Keys - Offline Key Management System"),
                error_message: None,
            },
            Task::none(),
        )
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn title(&self) -> String {
        String::from("CIM Keys - Offline Domain Bootstrap")
    }

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
                        self.bootstrap_config = Some(config.clone());
                        self.domain_loaded = true;
                        self.active_tab = Tab::Organization;
                        self.status_message = "Domain configuration loaded successfully".to_string();
                        // TODO: Populate graph from config
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
                self.status_message = format!("Removed person from organization");
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
                // TODO: Implement actual root CA generation
                Task::none()
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
                                &*projection,
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
                        self.status_message = format!("Selected person in graph");
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
        }
    }

    fn view(&self) -> Element<Message> {
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
        let error_display = if let Some(ref error) = self.error_message {
            Some(
                container(
                    row![
                        text(format!("❌ {}", error)).size(14),
                        horizontal_space(),
                        button("✕").on_press(Message::ClearError)
                    ]
                    .padding(10)
                )
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                    text_color: Some(Color::WHITE),
                    border: Border {
                        color: Color::from_rgb(0.6, 0.1, 0.1),
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                })
            )
        } else {
            None
        };

        let mut main_column = column![
            text("🔐 CIM Keys - Offline Key Management System").size(24),
            text(&self.status_message).size(12),
            container(tab_bar)
                .padding(10)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                    border: Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                }),
        ]
        .spacing(10);

        if let Some(error) = error_display {
            main_column = main_column.push(error);
        }

        main_column = main_column.push(content);

        container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

impl CimKeysApp {
    fn tab_button_style(&self, _theme: &Theme, is_active: bool) -> button::Style {
        if is_active {
            button::Style {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.4, 0.8))),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::from_rgb(0.3, 0.5, 0.9),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }
        } else {
            button::Style {
                background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                text_color: Color::from_rgb(0.8, 0.8, 0.8),
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }
        }
    }

    fn view_welcome(&self) -> Element<Message> {
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
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.6, 0.2))),
                text_color: Some(Color::BLACK),
                border: Border {
                    color: Color::from_rgb(0.6, 0.4, 0.1),
                    width: 2.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            })
            .padding(20),

            if !self.domain_loaded {
                column![
                    text("Get Started").size(20),
                    row![
                        text_input("Organization", &self.organization_name)
                            .on_input(Message::OrganizationNameChanged)
                            .size(16),
                        text_input("Domain", &self.organization_domain)
                            .on_input(Message::OrganizationDomainChanged)
                            .size(16),
                    ]
                    .spacing(10),
                    row![
                        button("Load Existing Domain").on_press(Message::LoadExistingDomain),
                        button("Create New Domain").on_press(Message::CreateNewDomain),
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

    fn view_organization(&self) -> Element<Message> {
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
                    text("Add Person to Organization").size(16),
                    row![
                        text_input("Name", &self.new_person_name)
                            .on_input(Message::NewPersonNameChanged)
                            .size(14),
                        text_input("Email", &self.new_person_email)
                            .on_input(Message::NewPersonEmailChanged)
                            .size(14),
                        pick_list(
                            role_options,
                            self.new_person_role,
                            Message::NewPersonRoleSelected,
                        )
                        .placeholder("Select Role"),
                        button("Add Person").on_press(Message::AddPerson)
                    ]
                    .spacing(10),
                ]
                .spacing(10)
            )
            .padding(15)
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }),

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

    fn view_keys(&self) -> Element<Message> {
        let progress_percentage = self.key_generation_progress * 100.0;

        let content = column![
            text("Generate Keys for Organization").size(20),
            text("Generate cryptographic keys for all organization members").size(14),

            container(
                column![
                    text("Key Generation Options").size(16),
                    button("Generate Root CA")
                        .on_press(Message::GenerateRootCA)
                        .style(|theme: &Theme, _| button::primary(theme, button::Status::Active)),
                    button("Generate SSH Keys for All")
                        .on_press(Message::GenerateSSHKeys),
                    button("Provision YubiKeys")
                        .on_press(Message::ProvisionYubiKey),
                    button("Generate All Keys")
                        .on_press(Message::GenerateAllKeys)
                        .style(|theme: &Theme, _| button::success(theme, button::Status::Active)),
                ]
                .spacing(10)
            )
            .padding(15)
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }),

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

    fn view_export(&self) -> Element<Message> {
        let content = column![
            text("Export Domain Configuration").size(20),
            text("Export your domain configuration to encrypted storage").size(14),

            container(
                column![
                    text("Export Options").size(16),
                    text_input("Output Directory", &self.export_path.display().to_string())
                        .on_input(Message::ExportPathChanged),
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
                    } else {
                        text_input("", "").on_input(Message::ExportPasswordChanged)
                    },
                ]
                .spacing(10)
            )
            .padding(15)
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }),

            button("Export to Encrypted SD Card")
                .on_press(Message::ExportToSDCard)
                .style(|theme: &Theme, _| button::primary(theme, button::Status::Active))
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
    // TODO: Implement file loading in WASM
    Err("WASM file loading not yet implemented".to_string())
}

async fn generate_all_keys(
    aggregate: Arc<RwLock<KeyManagementAggregate>>,
    projection: Arc<RwLock<OfflineKeyProjection>>,
) -> Result<usize, String> {
    // TODO: Implement actual key generation
    Ok(0)
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
        .subscription(CimKeysApp::subscription)
        .theme(CimKeysApp::theme)
        .run_with(|| CimKeysApp::new(output_dir))
}