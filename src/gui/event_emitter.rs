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
    domain::{Person, KeyOwnerRole},
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
        let subject = self.build_subject("key.command");

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
    /// TODO: Update this to work with new modular command structure
    /// For now, commands are handled directly by their handlers
    fn build_subject(&self, _operation: &str) -> String {
        // TODO: Implement proper subject building for new command structure
        format!("{}.command", self.subject_prefix)
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
                    owner_id: e.ownership.as_ref().map(|o| match o {
                        crate::domain::KeyOwnership { person_id, .. } => *person_id
                    }).unwrap_or_else(Uuid::nil),
                    key_type: format!("{:?}", e.algorithm),
                },

                NatsOperatorCreated(e) => GuiUpdateMessage::StatusUpdate {
                    message: format!("NATS operator '{}' created", e.name),
                },

                TrustEstablished(e) => GuiUpdateMessage::GraphEdgeAdded {
                    from: e.trustor_id,
                    to: e.trustee_id,
                    edge_type: "trust".to_string(),
                },

                KeyRevoked(e) => GuiUpdateMessage::KeyRemoved {
                    key_id: e.key_id,
                    reason: format!("{:?}", e.reason),  // Convert enum to string
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
            KeyEvent::KeyGenerated(e) => e.key_id,
            KeyEvent::KeyImported(e) => e.key_id,
            KeyEvent::CertificateGenerated(e) => e.cert_id,
            KeyEvent::CertificateSigned(e) => e.cert_id,
            KeyEvent::KeyExported(e) => e.key_id,
            KeyEvent::KeyStoredOffline(e) => e.key_id,
            KeyEvent::YubiKeyProvisioned(_e) => Uuid::now_v7(), // Generate ID for YubiKey events based on event data
            KeyEvent::PinConfigured(e) => e.event_id,
            KeyEvent::PukConfigured(e) => e.event_id,
            KeyEvent::ManagementKeyRotated(e) => e.event_id,
            KeyEvent::YubiKeyDetected(e) => e.event_id,
            KeyEvent::KeyGeneratedInSlot(e) => e.event_id,
            KeyEvent::CertificateImportedToSlot(e) => e.event_id,
            KeyEvent::SlotAllocationPlanned(e) => e.event_id,
            KeyEvent::SshKeyGenerated(e) => e.key_id,
            KeyEvent::GpgKeyGenerated(e) => e.key_id,
            KeyEvent::KeyRevoked(e) => e.key_id,
            KeyEvent::TrustEstablished(e) => e.trustor_id, // Use trustor_id as event ID
            KeyEvent::PkiHierarchyCreated(e) => e.root_ca_id,
            KeyEvent::NatsOperatorCreated(e) => e.operator_id,
            KeyEvent::NatsAccountCreated(e) => e.account_id,
            KeyEvent::NatsUserCreated(e) => e.user_id,
            // NATS Operator State Transitions
            KeyEvent::NatsOperatorSuspended(e) => e.operator_id,
            KeyEvent::NatsOperatorReactivated(e) => e.operator_id,
            KeyEvent::NatsOperatorRevoked(e) => e.operator_id,
            // NATS Account State Transitions
            KeyEvent::NatsAccountActivated(e) => e.account_id,
            KeyEvent::NatsAccountSuspended(e) => e.account_id,
            KeyEvent::NatsAccountReactivated(e) => e.account_id,
            KeyEvent::NatsAccountDeleted(e) => e.account_id,
            // NATS User State Transitions
            KeyEvent::NatsUserActivated(e) => e.user_id,
            KeyEvent::NatsUserSuspended(e) => e.user_id,
            KeyEvent::NatsUserReactivated(e) => e.user_id,
            KeyEvent::NatsUserDeleted(e) => e.user_id,
            KeyEvent::NatsSigningKeyGenerated(_e) => Uuid::now_v7(), // Generate ID from event
            KeyEvent::NKeyGenerated(e) => e.nkey_id,
            KeyEvent::JwtClaimsCreated(e) => e.claims_id,
            KeyEvent::JwtSigned(e) => e.jwt_id,
            KeyEvent::ProjectionApplied(e) => e.projection_id,
            KeyEvent::NatsPermissionsSet(_e) => Uuid::now_v7(), // Generate ID from event
            KeyEvent::NatsConfigExported(_e) => Uuid::now_v7(), // Generate ID from event
            KeyEvent::JwksExported(e) => e.export_id,
            KeyEvent::KeyRotationInitiated(e) => e.rotation_id,
            KeyEvent::KeyRotationCompleted(e) => e.rotation_id,
            KeyEvent::TotpSecretGenerated(e) => e.secret_id,
            KeyEvent::ServiceAccountCreated(e) => e.service_account_id,
            KeyEvent::AgentCreated(e) => e.agent_id,
            KeyEvent::AccountabilityValidated(e) => e.validation_id,
            KeyEvent::AccountabilityViolated(e) => e.violation_id,
            KeyEvent::CertificateExported(e) => e.export_id,
            KeyEvent::ManifestCreated(e) => e.manifest_id,
            KeyEvent::PersonCreated(e) => e.person_id,
            KeyEvent::LocationCreated(e) => e.location_id,
            KeyEvent::OrganizationCreated(e) => e.organization_id,
            KeyEvent::OrganizationalUnitCreated(e) => e.unit_id,
            KeyEvent::RoleCreated(e) => e.role_id,
            KeyEvent::PolicyCreated(e) => e.policy_id,
            KeyEvent::RelationshipEstablished(e) => e.from_id,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            KeyEvent::KeyGenerated(_) => "KeyGenerated",
            KeyEvent::KeyImported(_) => "KeyImported",
            KeyEvent::CertificateGenerated(_) => "CertificateGenerated",
            KeyEvent::CertificateSigned(_) => "CertificateSigned",
            KeyEvent::KeyExported(_) => "KeyExported",
            KeyEvent::KeyStoredOffline(_) => "KeyStoredOffline",
            KeyEvent::YubiKeyProvisioned(_) => "YubiKeyProvisioned",
            KeyEvent::PinConfigured(_) => "PinConfigured",
            KeyEvent::PukConfigured(_) => "PukConfigured",
            KeyEvent::ManagementKeyRotated(_) => "ManagementKeyRotated",
            KeyEvent::YubiKeyDetected(_) => "YubiKeyDetected",
            KeyEvent::KeyGeneratedInSlot(_) => "KeyGeneratedInSlot",
            KeyEvent::CertificateImportedToSlot(_) => "CertificateImportedToSlot",
            KeyEvent::SlotAllocationPlanned(_) => "SlotAllocationPlanned",
            KeyEvent::SshKeyGenerated(_) => "SshKeyGenerated",
            KeyEvent::GpgKeyGenerated(_) => "GpgKeyGenerated",
            KeyEvent::KeyRevoked(_) => "KeyRevoked",
            KeyEvent::TrustEstablished(_) => "TrustEstablished",
            KeyEvent::PkiHierarchyCreated(_) => "PkiHierarchyCreated",
            KeyEvent::NatsOperatorCreated(_) => "NatsOperatorCreated",
            KeyEvent::NatsAccountCreated(_) => "NatsAccountCreated",
            KeyEvent::NatsUserCreated(_) => "NatsUserCreated",
            // NATS Operator State Transitions
            KeyEvent::NatsOperatorSuspended(_) => "NatsOperatorSuspended",
            KeyEvent::NatsOperatorReactivated(_) => "NatsOperatorReactivated",
            KeyEvent::NatsOperatorRevoked(_) => "NatsOperatorRevoked",
            // NATS Account State Transitions
            KeyEvent::NatsAccountActivated(_) => "NatsAccountActivated",
            KeyEvent::NatsAccountSuspended(_) => "NatsAccountSuspended",
            KeyEvent::NatsAccountReactivated(_) => "NatsAccountReactivated",
            KeyEvent::NatsAccountDeleted(_) => "NatsAccountDeleted",
            // NATS User State Transitions
            KeyEvent::NatsUserActivated(_) => "NatsUserActivated",
            KeyEvent::NatsUserSuspended(_) => "NatsUserSuspended",
            KeyEvent::NatsUserReactivated(_) => "NatsUserReactivated",
            KeyEvent::NatsUserDeleted(_) => "NatsUserDeleted",
            KeyEvent::NatsSigningKeyGenerated(_) => "NatsSigningKeyGenerated",
            KeyEvent::NKeyGenerated(_) => "NKeyGenerated",
            KeyEvent::JwtClaimsCreated(_) => "JwtClaimsCreated",
            KeyEvent::JwtSigned(_) => "JwtSigned",
            KeyEvent::ProjectionApplied(_) => "ProjectionApplied",
            KeyEvent::NatsPermissionsSet(_) => "NatsPermissionsSet",
            KeyEvent::NatsConfigExported(_) => "NatsConfigExported",
            KeyEvent::JwksExported(_) => "JwksExported",
            KeyEvent::KeyRotationInitiated(_) => "KeyRotationInitiated",
            KeyEvent::KeyRotationCompleted(_) => "KeyRotationCompleted",
            KeyEvent::TotpSecretGenerated(_) => "TotpSecretGenerated",
            KeyEvent::ServiceAccountCreated(_) => "ServiceAccountCreated",
            KeyEvent::AgentCreated(_) => "AgentCreated",
            KeyEvent::AccountabilityValidated(_) => "AccountabilityValidated",
            KeyEvent::AccountabilityViolated(_) => "AccountabilityViolated",
            KeyEvent::CertificateExported(_) => "CertificateExported",
            KeyEvent::ManifestCreated(_) => "ManifestCreated",
            KeyEvent::PersonCreated(_) => "PersonCreated",
            KeyEvent::LocationCreated(_) => "LocationCreated",
            KeyEvent::OrganizationCreated(_) => "OrganizationCreated",
            KeyEvent::OrganizationalUnitCreated(_) => "OrganizationalUnitCreated",
            KeyEvent::RoleCreated(_) => "RoleCreated",
            KeyEvent::PolicyCreated(_) => "PolicyCreated",
            KeyEvent::RelationshipEstablished(_) => "RelationshipEstablished",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Update test to use current KeyCommand variants
    // #[test]
    // fn test_event_emission() {
    //     let mut emitter = CimEventEmitter::new("cowboyai");
    //
    //     let command = KeyCommand::GenerateSshKey(...);
    //
    //     let domain_cmd = emitter.emit_command(
    //         command,
    //         "KeyGeneration",
    //         InteractionType::ButtonClick { button_id: "generate_ssh".to_string() },
    //     );
    //
    //     assert_eq!(domain_cmd.subject, "cowboyai.gui.commands.ssh.generate.keypair");
    //     assert_eq!(emitter.drain_commands().len(), 1);
    // }

    #[test]
    fn test_event_subscription() {
        let subscriber = GuiEventSubscriber::new("cowboyai");
        let subjects = subscriber.subjects();

        assert!(subjects.contains(&"cowboyai.events.>".to_string()));
        assert!(subjects.contains(&"cowboyai.projections.updated".to_string()));
    }
}