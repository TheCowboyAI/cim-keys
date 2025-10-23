//! CIM Event Emission System for GUI
//!
//! This module handles the translation of GUI actions into CIM domain events
//! following the event-driven architecture principles.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::{
    commands::KeyCommand,
    domain::{Organization, Person, Location, KeyOwnerRole},
    events::KeyEvent,
};

/// GUI Event Emitter following CIM principles
///
/// IMPORTANT: The GUI never mutates state directly!
/// It only emits intentions as commands that will be processed
/// by the domain aggregate and result in events.
#[derive(Clone)]
pub struct CimEventEmitter {
    /// Queue of commands to be published
    command_queue: Arc<Mutex<VecDeque<DomainCommand>>>,

    /// Current correlation ID for tracking related operations
    correlation_id: Arc<Mutex<Uuid>>,

    /// Last event ID for causation tracking
    last_event_id: Arc<Mutex<Option<Uuid>>>,

    /// NATS subject prefix for this session
    subject_prefix: String,
}

/// Domain command with full CIM metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCommand {
    /// The actual command to execute
    pub command: KeyCommand,

    /// Correlation ID linking all related commands/events
    pub correlation_id: Uuid,

    /// ID of the event that caused this command
    pub causation_id: Option<Uuid>,

    /// Timestamp when the command was created
    pub timestamp: chrono::DateTime<Utc>,

    /// NATS subject for publishing
    pub subject: String,

    /// GUI context (screen, user action, etc.)
    pub context: GuiContext,
}

/// Context about where in the GUI the command originated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiContext {
    /// Current screen when command was issued
    pub screen: String,

    /// Type of user interaction
    pub interaction: InteractionType,

    /// Optional additional metadata
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    ButtonClick { button_id: String },
    TextInput { field_id: String },
    GraphNodeClick { node_id: Uuid },
    GraphEdgeCreate { from: Uuid, to: Uuid },
    FileLoad { filename: String },
    Navigation { from_screen: String, to_screen: String },
}

impl CimEventEmitter {
    /// Create a new event emitter for the GUI session
    pub fn new(organization_name: &str) -> Self {
        Self {
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
            correlation_id: Arc::new(Mutex::new(Uuid::now_v7())),
            last_event_id: Arc::new(Mutex::new(None)),
            subject_prefix: format!("{}.gui.commands", organization_name.to_lowercase()),
        }
    }

    /// Start a new correlation context (e.g., for a new user workflow)
    pub fn new_correlation(&mut self) {
        *self.correlation_id.lock().unwrap() = Uuid::now_v7();
        *self.last_event_id.lock().unwrap() = None;
    }

    /// Emit a command from a GUI action
    pub fn emit_command(
        &mut self,
        command: KeyCommand,
        screen: &str,
        interaction: InteractionType,
    ) -> DomainCommand {
        // Build the NATS subject based on command type
        let subject = self.build_subject(&command);

        let correlation_id = *self.correlation_id.lock().unwrap();
        let causation_id = *self.last_event_id.lock().unwrap();

        let domain_command = DomainCommand {
            command,
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            subject,
            context: GuiContext {
                screen: screen.to_string(),
                interaction,
                metadata: None,
            },
        };

        // Queue for publishing
        self.command_queue.lock().unwrap().push_back(domain_command.clone());

        domain_command
    }

    /// Build NATS subject for a command
    fn build_subject(&self, command: &KeyCommand) -> String {
        use crate::commands::KeyCommand::*;

        let operation = match command {
            GenerateKey(_) => "key.generate",
            ImportKey(_) => "key.import",
            GenerateCertificate(_) => "certificate.generate",
            SignCertificate(_) => "certificate.sign",
            ExportKey(_) => "key.export",
            StoreKeyOffline(_) => "key.store.offline",
            ProvisionYubiKey(_) => "yubikey.provision",
            GenerateSshKey(_) => "ssh.generate",
            GenerateGpgKey(_) => "gpg.generate",
            RevokeKey(_) => "key.revoke",
            EstablishTrust(_) => "trust.establish",
            CreatePkiHierarchy(_) => "pki.create",
            CreateNatsOperator(_) => "nats.create.operator",
            CreateNatsAccount(_) => "nats.create.account",
            CreateNatsUser(_) => "nats.create.user",
            GenerateNatsSigningKey(_) => "nats.generate.signing",
            SetNatsPermissions(_) => "nats.set.permissions",
            ExportNatsConfig(_) => "nats.export.config",
        };

        format!("{}.{}", self.subject_prefix, operation)
    }

    /// Process an event that was received
    pub fn process_event(&mut self, event: &KeyEvent) {
        // Update last event ID for causation tracking
        *self.last_event_id.lock().unwrap() = Some(event.id());
    }

    /// Get pending commands to publish
    pub fn drain_commands(&mut self) -> Vec<DomainCommand> {
        self.command_queue.lock().unwrap().drain(..).collect()
    }
}

/// Event subscription handler for GUI updates
pub struct GuiEventSubscriber {
    /// Subscribed NATS subjects
    subscriptions: Vec<String>,

    /// Event buffer for processing
    event_buffer: VecDeque<KeyEvent>,
}

impl GuiEventSubscriber {
    /// Create a new subscriber for GUI-relevant events
    pub fn new(organization_name: &str) -> Self {
        let base = organization_name.to_lowercase();

        Self {
            subscriptions: vec![
                format!("{}.events.>", base),  // All events for this org
                format!("{}.projections.updated", base),  // Projection changes
                format!("{}.graph.changed", base),  // Graph structure changes
            ],
            event_buffer: VecDeque::new(),
        }
    }

    /// Get the subjects this GUI should subscribe to
    pub fn subjects(&self) -> &[String] {
        &self.subscriptions
    }

    /// Buffer an incoming event
    pub fn buffer_event(&mut self, event: KeyEvent) {
        self.event_buffer.push_back(event);
    }

    /// Process buffered events and return GUI update messages
    pub fn process_events(&mut self) -> Vec<GuiUpdateMessage> {
        let mut messages = Vec::new();

        while let Some(event) = self.event_buffer.pop_front() {
            use KeyEvent::*;

            let message = match event {
                KeyGenerated(e) => GuiUpdateMessage::KeyAdded {
                    owner_id: e.owner_id,
                    key_type: format!("{:?}", e.key_type),
                },

                NatsOperatorCreated(e) => GuiUpdateMessage::StatusUpdate {
                    message: format!("NATS operator '{}' created", e.name),
                },

                TrustEstablished(e) => GuiUpdateMessage::GraphEdgeAdded {
                    from: e.from_entity,
                    to: e.to_entity,
                    edge_type: "trust".to_string(),
                },

                KeyRevoked(e) => GuiUpdateMessage::KeyRemoved {
                    key_id: e.key_id,
                    reason: e.reason,
                },

                _ => GuiUpdateMessage::StatusUpdate {
                    message: format!("Event processed: {:?}", event.event_type()),
                },
            };

            messages.push(message);
        }

        messages
    }
}

/// Messages for updating the GUI based on events
#[derive(Debug, Clone)]
pub enum GuiUpdateMessage {
    KeyAdded {
        owner_id: Uuid,
        key_type: String,
    },
    KeyRemoved {
        key_id: Uuid,
        reason: String,
    },
    GraphNodeAdded {
        person: Person,
        role: KeyOwnerRole,
    },
    GraphEdgeAdded {
        from: Uuid,
        to: Uuid,
        edge_type: String,
    },
    StatusUpdate {
        message: String,
    },
    ProjectionReloaded,
}

// Extension trait for KeyEvent to get metadata
impl KeyEvent {
    pub fn id(&self) -> Uuid {
        match self {
            KeyEvent::KeyGenerated(e) => e.event_id,
            KeyEvent::CertificateGenerated(e) => e.event_id,
            KeyEvent::KeyStoredInYubiKey(e) => e.event_id,
            KeyEvent::NatsOperatorCreated(e) => e.event_id,
            KeyEvent::NatsAccountCreated(e) => e.event_id,
            KeyEvent::NatsUserCreated(e) => e.event_id,
            KeyEvent::TrustEstablished(e) => e.event_id,
            KeyEvent::KeyRevoked(e) => e.event_id,
            KeyEvent::DomainCreated(e) => e.event_id,
            KeyEvent::PersonAdded(e) => e.event_id,
            KeyEvent::OrganizationCreated(e) => e.event_id,
            KeyEvent::LocationAdded(e) => e.event_id,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            KeyEvent::KeyGenerated(_) => "KeyGenerated",
            KeyEvent::CertificateGenerated(_) => "CertificateGenerated",
            KeyEvent::KeyStoredInYubiKey(_) => "KeyStoredInYubiKey",
            KeyEvent::NatsOperatorCreated(_) => "NatsOperatorCreated",
            KeyEvent::NatsAccountCreated(_) => "NatsAccountCreated",
            KeyEvent::NatsUserCreated(_) => "NatsUserCreated",
            KeyEvent::TrustEstablished(_) => "TrustEstablished",
            KeyEvent::KeyRevoked(_) => "KeyRevoked",
            KeyEvent::DomainCreated(_) => "DomainCreated",
            KeyEvent::PersonAdded(_) => "PersonAdded",
            KeyEvent::OrganizationCreated(_) => "OrganizationCreated",
            KeyEvent::LocationAdded(_) => "LocationAdded",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_emission() {
        let mut emitter = CimEventEmitter::new("cowboyai");

        let command = KeyCommand::GenerateSSHKeypair {
            owner_id: Uuid::now_v7(),
            key_name: "test-key".to_string(),
            key_type: crate::types::KeyType::Ed25519,
            bits: None,
        };

        let domain_cmd = emitter.emit_command(
            command,
            "KeyGeneration",
            InteractionType::ButtonClick { button_id: "generate_ssh".to_string() },
        );

        assert_eq!(domain_cmd.subject, "cowboyai.gui.commands.ssh.generate.keypair");
        assert_eq!(emitter.drain_commands().len(), 1);
    }

    #[test]
    fn test_event_subscription() {
        let subscriber = GuiEventSubscriber::new("cowboyai");
        let subjects = subscriber.subjects();

        assert!(subjects.contains(&"cowboyai.events.>".to_string()));
        assert!(subjects.contains(&"cowboyai.projections.updated".to_string()));
    }
}