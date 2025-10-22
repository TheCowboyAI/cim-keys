//! Native/WASM GUI for offline key generation using Iced 0.13+
//!
//! This module provides a pure Rust GUI that can run both as a native
//! application and as a WASM application in the browser.

use iced::{
    application, executor,
    widget::{button, column, container, row, text, text_input, Column, Container},
    Task, Element, Length, Theme, Color, Background, Border,
};
use iced_futures::Subscription;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    current_screen: Screen,
    bootstrap_config: Option<BootstrapConfig>,
    aggregate: Arc<RwLock<KeyManagementAggregate>>,
    projection: Arc<RwLock<OfflineKeyProjection>>,

    // Event-driven communication
    event_emitter: CimEventEmitter,
    event_subscriber: GuiEventSubscriber,

    // Graph visualization
    org_graph: OrganizationGraph,

    // Form fields
    org_name: String,
    org_display_name: String,
    person_name: String,
    person_email: String,
    yubikey_serial: String,

    // Status
    status_message: String,
    keys_generated: usize,
    certificates_generated: usize,
}

/// Different screens in the application
#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Welcome,
    LoadConfig,
    OrganizationSetup,
    OrganizationGraph,  // New graph visualization screen
    PeopleSetup,
    YubiKeySetup,
    GenerateKeys,
    Export,
}

/// Messages for the application
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(Screen),

    // File operations
    LoadConfigClicked,
    ConfigLoaded(Result<BootstrapConfig, String>),

    // Form inputs
    OrgNameChanged(String),
    OrgDisplayNameChanged(String),
    PersonNameChanged(String),
    PersonEmailChanged(String),
    YubiKeySerialChanged(String),

    // Actions
    AddPerson,
    RemovePerson(usize),
    AddYubiKey,
    RemoveYubiKey(usize),
    GenerateKeys,
    KeysGenerated(Result<usize, String>),
    ExportDomain,
    DomainExported(Result<String, String>),

    // Async tasks
    TaskComplete(String),

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
                current_screen: Screen::Welcome,
                bootstrap_config: None,
                aggregate,
                projection,
                event_emitter: CimEventEmitter::new(default_org),
                event_subscriber: GuiEventSubscriber::new(default_org),
                org_graph: OrganizationGraph::new(),
                org_name: String::new(),
                org_display_name: String::new(),
                person_name: String::new(),
                person_email: String::new(),
                yubikey_serial: String::new(),
                status_message: String::from("Welcome to CIM Keys - Offline Key Generation"),
                keys_generated: 0,
                certificates_generated: 0,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("CIM Keys - Offline Domain Bootstrap")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.current_screen = screen;
                Task::none()
            }

            Message::LoadConfigClicked => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Task::perform(load_config_native(), Message::ConfigLoaded)
                }
                #[cfg(target_arch = "wasm32")]
                {
                    // In WASM, we need to use file input element
                    Task::perform(load_config_wasm(), Message::ConfigLoaded)
                }
            }

            Message::ConfigLoaded(result) => {
                match result {
                    Ok(config) => {
                        self.bootstrap_config = Some(config);
                        self.current_screen = Screen::OrganizationSetup;
                        self.status_message = String::from("Configuration loaded successfully");
                    }
                    Err(e) => {
                        self.status_message = format!("Failed to load config: {}", e);
                    }
                }
                Task::none()
            }

            Message::OrgNameChanged(value) => {
                self.org_name = value;
                Task::none()
            }

            Message::OrgDisplayNameChanged(value) => {
                self.org_display_name = value;
                Task::none()
            }

            Message::PersonNameChanged(value) => {
                self.person_name = value;
                Task::none()
            }

            Message::PersonEmailChanged(value) => {
                self.person_email = value;
                Task::none()
            }

            Message::AddPerson => {
                // Create a person and add to graph
                let person_id = Uuid::new_v4();
                let person = Person {
                    id: person_id,
                    name: self.person_name.clone(),
                    email: self.person_email.clone(),
                    organization_id: Uuid::new_v4(), // TODO: Use actual org ID
                    unit_ids: vec![],
                    roles: vec![],
                    active: true,
                    created_at: chrono::Utc::now(),
                };

                // Add to graph for visualization
                self.org_graph.add_node(person.clone(), KeyOwnerRole::Developer);

                // Emit domain command following CIM principles
                // GUI doesn't directly save - it emits an intention
                let command = crate::commands::KeyCommand::GenerateSshKey(
                    crate::commands::GenerateSshKeyCommand {
                        command_id: cim_domain::EntityId::new(),
                        user: person.name.to_lowercase().replace(" ", "-"),
                        comment: format!("{}@{}", person.email, self.org_name),
                        algorithm: crate::events::KeyAlgorithm::Ed25519,
                        passphrase: None,
                        requestor: person.email.clone(),
                        context: Some(crate::domain::KeyContext {
                            owner_id: person_id,
                            organization_id: person.organization_id,
                            location: crate::domain::KeyStorageLocation::LocalSystem,
                            owner: crate::domain::KeyOwnership::Person(person_id),
                        }),
                    }
                );

                self.event_emitter.emit_command(
                    command,
                    "PeopleSetup",
                    InteractionType::ButtonClick {
                        button_id: "add_person".to_string()
                    }
                );

                // Clear form fields
                self.person_name.clear();
                self.person_email.clear();
                self.status_message = format!(
                    "Person '{}' added - Keys will be generated",
                    person.name
                );

                Task::none()
            }

            Message::YubiKeySerialChanged(value) => {
                self.yubikey_serial = value;
                Task::none()
            }

            Message::GenerateKeys => {
                // Emit command to generate keys
                // In true CIM style, we don't directly call the aggregate
                // Instead we emit a command that will be processed asynchronously

                let root_ca_command = crate::commands::KeyCommand::GenerateCertificate(
                    crate::commands::GenerateCertificateCommand {
                        command_id: cim_domain::EntityId::new(),
                        key_id: uuid::Uuid::new_v4(),
                        subject: crate::commands::CertificateSubject {
                            common_name: self.org_name.clone(),
                            organization: Some(self.org_display_name.clone()),
                            country: Some("US".to_string()),
                            organizational_unit: None,
                            locality: None,
                            state_province: None,
                        },
                        validity_days: 3650,
                        is_ca: true,
                        san: vec![],
                        key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
                        extended_key_usage: vec![],
                        requestor: "GUI".to_string(),
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
                                None  // No NATS port in offline mode
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
                        self.status_message = format!("Generated {} keys successfully", count);
                        self.current_screen = Screen::Export;
                    }
                    Err(e) => {
                        self.status_message = format!("Key generation failed: {}", e);
                    }
                }
                Task::none()
            }

            Message::ExportDomain => {
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

            Message::GraphMessage(graph_msg) => {
                // Handle graph interactions
                match graph_msg {
                    GraphMessage::NodeClicked(id) => {
                        self.status_message = format!("Selected node: {}", id);
                    }
                    GraphMessage::AutoLayout => {
                        self.org_graph.auto_layout();
                        self.status_message = String::from("Graph layout updated");
                    }
                    GraphMessage::AddEdge(from, to, edge_type) => {
                        self.org_graph.add_edge(from, to, edge_type);
                        self.status_message = String::from("Relationship added");
                    }
                    _ => {}
                }
                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self.current_screen {
            Screen::Welcome => self.view_welcome(),
            Screen::LoadConfig => self.view_load_config(),
            Screen::OrganizationSetup => self.view_organization_setup(),
            Screen::OrganizationGraph => self.view_organization_graph(),
            Screen::PeopleSetup => self.view_people_setup(),
            Screen::YubiKeySetup => self.view_yubikey_setup(),
            Screen::GenerateKeys => self.view_generate_keys(),
            Screen::Export => self.view_export(),
        };

        container(column![
            text("CIM Keys - Offline Domain Bootstrap").size(24),
            text(&self.status_message).size(14),
            content
        ])
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
    fn view_welcome(&self) -> Element<Message> {
        column![
            text("Welcome to CIM Keys").size(20),
            text("This tool generates the initial cryptographic keys and domain structure for your CIM infrastructure."),
            text("⚠️  Ensure this computer is air-gapped from all networks!").size(16),
            button("Start Setup").on_press(Message::NavigateTo(Screen::LoadConfig)),
            button("Load Existing Config").on_press(Message::LoadConfigClicked)
        ]
        .spacing(20)
        .into()
    }

    fn view_load_config(&self) -> Element<Message> {
        column![
            text("Load Bootstrap Configuration").size(20),
            button("Choose File").on_press(Message::LoadConfigClicked),
            button("Create New").on_press(Message::NavigateTo(Screen::OrganizationSetup))
        ]
        .spacing(20)
        .into()
    }

    fn view_organization_setup(&self) -> Element<Message> {
        column![
            text("Organization Setup").size(20),
            text_input("Organization Name", &self.org_name)
                .on_input(Message::OrgNameChanged),
            text_input("Display Name", &self.org_display_name)
                .on_input(Message::OrgDisplayNameChanged),
            button("Next").on_press(Message::NavigateTo(Screen::PeopleSetup))
        ]
        .spacing(20)
        .into()
    }

    fn view_people_setup(&self) -> Element<Message> {
        column![
            text("People Setup").size(20),
            text_input("Name", &self.person_name)
                .on_input(Message::PersonNameChanged),
            text_input("Email", &self.person_email)
                .on_input(Message::PersonEmailChanged),
            button("Add Person").on_press(Message::AddPerson),
            button("Next").on_press(Message::NavigateTo(Screen::YubiKeySetup))
        ]
        .spacing(20)
        .into()
    }

    fn view_yubikey_setup(&self) -> Element<Message> {
        column![
            text("YubiKey Setup").size(20),
            text_input("YubiKey Serial", &self.yubikey_serial)
                .on_input(Message::YubiKeySerialChanged),
            button("Add YubiKey").on_press(Message::AddYubiKey),
            button("Generate Keys").on_press(Message::NavigateTo(Screen::GenerateKeys))
        ]
        .spacing(20)
        .into()
    }

    fn view_generate_keys(&self) -> Element<Message> {
        column![
            text("Generate Keys").size(20),
            text(format!("Keys Generated: {}", self.keys_generated)),
            text(format!("Certificates Generated: {}", self.certificates_generated)),
            button("Start Generation").on_press(Message::GenerateKeys)
        ]
        .spacing(20)
        .into()
    }

    fn view_export(&self) -> Element<Message> {
        column![
            text("Export Domain").size(20),
            text("Ready to export to encrypted storage"),
            button("Export").on_press(Message::ExportDomain)
        ]
        .spacing(20)
        .into()
    }

    fn view_organization_graph(&self) -> Element<Message> {
        use graph::view_graph;

        column![
            text("Organization Graph").size(20),
            text("Visual representation of organizational structure and key delegations"),

            // Graph visualization canvas
            Container::new(
                view_graph(&self.org_graph)
                    .map(Message::GraphMessage)
            )
            .width(Length::Fill)
            .height(Length::FillPortion(8))
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

            // Navigation buttons
            row![
                button("Back").on_press(Message::NavigateTo(Screen::PeopleSetup)),
                button("Continue").on_press(Message::NavigateTo(Screen::YubiKeySetup))
            ]
            .spacing(20)
        ]
        .spacing(20)
        .into()
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