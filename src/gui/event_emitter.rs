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
    events::DomainEvent,
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
    pub fn process_event(&mut self, event: &DomainEvent) {
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
    event_buffer: VecDeque<DomainEvent>,
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
    pub fn buffer_event(&mut self, event: DomainEvent) {
        self.event_buffer.push_back(event);
    }

    /// Process buffered events and return GUI update messages
    pub fn process_events(&mut self) -> Vec<GuiUpdateMessage> {
        let mut messages = Vec::new();

        while let Some(event) = self.event_buffer.pop_front() {
            let message = match &event {
                DomainEvent::Key(crate::events::KeyEvents::KeyGenerated(e)) => GuiUpdateMessage::KeyAdded {
                    owner_id: e.ownership.as_ref().map(|o| match o {
                        crate::domain::KeyOwnership { person_id, .. } => *person_id
                    }).unwrap_or_else(Uuid::nil),
                    key_type: format!("{:?}", e.algorithm),
                },

                DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorCreated(e)) => GuiUpdateMessage::StatusUpdate {
                    message: format!("NATS operator '{}' created", e.name),
                },

                DomainEvent::Relationship(crate::events::RelationshipEvents::TrustEstablished(e)) => GuiUpdateMessage::GraphEdgeAdded {
                    from: e.trustor_id,
                    to: e.trustee_id,
                    edge_type: "trust".to_string(),
                },

                DomainEvent::Key(crate::events::KeyEvents::KeyRevoked(e)) => GuiUpdateMessage::KeyRemoved {
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

// Extension trait for DomainEvent to get metadata
impl DomainEvent {
    pub fn id(&self) -> Uuid {
        match self {
            // Key aggregate events
            DomainEvent::Key(crate::events::KeyEvents::KeyGenerated(e)) => e.key_id,
            DomainEvent::Key(crate::events::KeyEvents::KeyImported(e)) => e.key_id,
            DomainEvent::Key(crate::events::KeyEvents::KeyExported(e)) => e.key_id,
            DomainEvent::Key(crate::events::KeyEvents::KeyStoredOffline(e)) => e.key_id,
            DomainEvent::Key(crate::events::KeyEvents::SshKeyGenerated(e)) => e.key_id,
            DomainEvent::Key(crate::events::KeyEvents::GpgKeyGenerated(e)) => e.key_id,
            DomainEvent::Key(crate::events::KeyEvents::KeyRevoked(e)) => e.key_id,
            DomainEvent::Key(crate::events::KeyEvents::KeyRotationInitiated(e)) => e.rotation_id,
            DomainEvent::Key(crate::events::KeyEvents::KeyRotationCompleted(e)) => e.rotation_id,
            // Certificate aggregate events
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateGenerated(e)) => e.cert_id,
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateSigned(e)) => e.cert_id,
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateActivated(e)) => e.cert_id,
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateSuspended(e)) => e.cert_id,
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateRevoked(e)) => e.cert_id,
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateExpired(e)) => e.cert_id,
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateRenewed(e)) => e.new_cert_id,
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateExported(e)) => e.export_id,
            DomainEvent::Certificate(crate::events::CertificateEvents::PkiHierarchyCreated(e)) => e.root_ca_id,
            // YubiKey aggregate events
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::YubiKeyProvisioned(_e)) => Uuid::now_v7(),
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::PinConfigured(e)) => e.event_id,
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::PukConfigured(e)) => e.event_id,
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::ManagementKeyRotated(e)) => e.event_id,
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::YubiKeyDetected(e)) => e.event_id,
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::KeyGeneratedInSlot(e)) => e.event_id,
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::CertificateImportedToSlot(e)) => e.event_id,
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::SlotAllocationPlanned(e)) => e.event_id,
            // Relationship aggregate events
            DomainEvent::Relationship(crate::events::RelationshipEvents::TrustEstablished(e)) => e.trustor_id,
            DomainEvent::Relationship(crate::events::RelationshipEvents::RelationshipEstablished(e)) => e.from_id,
            DomainEvent::Relationship(crate::events::RelationshipEvents::AccountabilityValidated(e)) => e.validation_id,
            DomainEvent::Relationship(crate::events::RelationshipEvents::AccountabilityViolated(e)) => e.violation_id,
            // NATS Operator aggregate events
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorCreated(e)) => e.operator_id,
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorSuspended(e)) => e.operator_id,
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorReactivated(e)) => e.operator_id,
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorRevoked(e)) => e.operator_id,
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsSigningKeyGenerated(_e)) => Uuid::now_v7(),
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NKeyGenerated(e)) => e.nkey_id,
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwtClaimsCreated(e)) => e.claims_id,
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwtSigned(e)) => e.jwt_id,
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsConfigExported(_e)) => Uuid::now_v7(),
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwksExported(e)) => e.export_id,
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::ProjectionApplied(e)) => e.projection_id,
            // NATS Account aggregate events
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsAccountCreated(e)) => e.account_id,
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsAccountSuspended(e)) => e.account_id,
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsAccountReactivated(e)) => e.account_id,
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsPermissionsSet(_e)) => Uuid::now_v7(),
            // NATS User aggregate events
            DomainEvent::NatsUser(crate::events::NatsUserEvents::NatsUserCreated(e)) => e.user_id,
            DomainEvent::NatsUser(crate::events::NatsUserEvents::ServiceAccountCreated(e)) => e.service_account_id,
            DomainEvent::NatsUser(crate::events::NatsUserEvents::AgentCreated(e)) => e.agent_id,
            DomainEvent::NatsUser(crate::events::NatsUserEvents::TotpSecretGenerated(e)) => e.secret_id,
            // Person aggregate events
            DomainEvent::Person(crate::events::PersonEvents::PersonCreated(e)) => e.person_id,
            DomainEvent::Person(crate::events::PersonEvents::PersonActivated(e)) => e.person_id,
            DomainEvent::Person(crate::events::PersonEvents::PersonSuspended(e)) => e.person_id,
            DomainEvent::Person(crate::events::PersonEvents::PersonReactivated(e)) => e.person_id,
            DomainEvent::Person(crate::events::PersonEvents::PersonArchived(e)) => e.person_id,
            // Location aggregate events
            DomainEvent::Location(crate::events::LocationEvents::LocationCreated(e)) => e.location_id,
            DomainEvent::Location(crate::events::LocationEvents::LocationActivated(e)) => e.location_id,
            DomainEvent::Location(crate::events::LocationEvents::LocationSuspended(e)) => e.location_id,
            DomainEvent::Location(crate::events::LocationEvents::LocationReactivated(e)) => e.location_id,
            DomainEvent::Location(crate::events::LocationEvents::LocationDecommissioned(e)) => e.location_id,
            // Organization aggregate events
            DomainEvent::Organization(crate::events::OrganizationEvents::OrganizationCreated(e)) => e.organization_id,
            DomainEvent::Organization(crate::events::OrganizationEvents::OrganizationalUnitCreated(e)) => e.unit_id,
            DomainEvent::Organization(crate::events::OrganizationEvents::RoleCreated(e)) => e.role_id,
            DomainEvent::Organization(crate::events::OrganizationEvents::PolicyCreated(e)) => e.policy_id,
            // Manifest aggregate events
            DomainEvent::Manifest(crate::events::ManifestEvents::ManifestCreated(e)) => e.manifest_id,
            // Catch-all for any events not explicitly handled
            _ => Uuid::now_v7(),
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            // Key aggregate
            DomainEvent::Key(crate::events::KeyEvents::KeyGenerated(_)) => "KeyGenerated",
            DomainEvent::Key(crate::events::KeyEvents::KeyImported(_)) => "KeyImported",
            DomainEvent::Key(crate::events::KeyEvents::KeyExported(_)) => "KeyExported",
            DomainEvent::Key(crate::events::KeyEvents::KeyStoredOffline(_)) => "KeyStoredOffline",
            DomainEvent::Key(crate::events::KeyEvents::SshKeyGenerated(_)) => "SshKeyGenerated",
            DomainEvent::Key(crate::events::KeyEvents::GpgKeyGenerated(_)) => "GpgKeyGenerated",
            DomainEvent::Key(crate::events::KeyEvents::KeyRevoked(_)) => "KeyRevoked",
            DomainEvent::Key(crate::events::KeyEvents::KeyRotationInitiated(_)) => "KeyRotationInitiated",
            DomainEvent::Key(crate::events::KeyEvents::KeyRotationCompleted(_)) => "KeyRotationCompleted",
            // Certificate aggregate
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateGenerated(_)) => "CertificateGenerated",
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateSigned(_)) => "CertificateSigned",
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateActivated(_)) => "CertificateActivated",
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateSuspended(_)) => "CertificateSuspended",
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateRevoked(_)) => "CertificateRevoked",
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateExpired(_)) => "CertificateExpired",
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateRenewed(_)) => "CertificateRenewed",
            DomainEvent::Certificate(crate::events::CertificateEvents::CertificateExported(_)) => "CertificateExported",
            DomainEvent::Certificate(crate::events::CertificateEvents::PkiHierarchyCreated(_)) => "PkiHierarchyCreated",
            // YubiKey aggregate
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::YubiKeyProvisioned(_)) => "YubiKeyProvisioned",
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::PinConfigured(_)) => "PinConfigured",
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::PukConfigured(_)) => "PukConfigured",
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::ManagementKeyRotated(_)) => "ManagementKeyRotated",
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::YubiKeyDetected(_)) => "YubiKeyDetected",
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::KeyGeneratedInSlot(_)) => "KeyGeneratedInSlot",
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::CertificateImportedToSlot(_)) => "CertificateImportedToSlot",
            DomainEvent::YubiKey(crate::events::YubiKeyEvents::SlotAllocationPlanned(_)) => "SlotAllocationPlanned",
            // Relationship aggregate
            DomainEvent::Relationship(crate::events::RelationshipEvents::TrustEstablished(_)) => "TrustEstablished",
            DomainEvent::Relationship(crate::events::RelationshipEvents::RelationshipEstablished(_)) => "RelationshipEstablished",
            DomainEvent::Relationship(crate::events::RelationshipEvents::AccountabilityValidated(_)) => "AccountabilityValidated",
            DomainEvent::Relationship(crate::events::RelationshipEvents::AccountabilityViolated(_)) => "AccountabilityViolated",
            // NATS Operator aggregate
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorCreated(_)) => "NatsOperatorCreated",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorSuspended(_)) => "NatsOperatorSuspended",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorReactivated(_)) => "NatsOperatorReactivated",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorRevoked(_)) => "NatsOperatorRevoked",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsSigningKeyGenerated(_)) => "NatsSigningKeyGenerated",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NKeyGenerated(_)) => "NKeyGenerated",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwtClaimsCreated(_)) => "JwtClaimsCreated",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwtSigned(_)) => "JwtSigned",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::ProjectionApplied(_)) => "ProjectionApplied",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsConfigExported(_)) => "NatsConfigExported",
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwksExported(_)) => "JwksExported",
            // NATS Account aggregate
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsAccountCreated(_)) => "NatsAccountCreated",
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsAccountSuspended(_)) => "NatsAccountSuspended",
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsAccountReactivated(_)) => "NatsAccountReactivated",
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsPermissionsSet(_)) => "NatsPermissionsSet",
            // NATS User aggregate
            DomainEvent::NatsUser(crate::events::NatsUserEvents::NatsUserCreated(_)) => "NatsUserCreated",
            DomainEvent::NatsUser(crate::events::NatsUserEvents::ServiceAccountCreated(_)) => "ServiceAccountCreated",
            DomainEvent::NatsUser(crate::events::NatsUserEvents::AgentCreated(_)) => "AgentCreated",
            DomainEvent::NatsUser(crate::events::NatsUserEvents::TotpSecretGenerated(_)) => "TotpSecretGenerated",
            // Person aggregate
            DomainEvent::Person(crate::events::PersonEvents::PersonCreated(_)) => "PersonCreated",
            DomainEvent::Person(crate::events::PersonEvents::PersonActivated(_)) => "PersonActivated",
            DomainEvent::Person(crate::events::PersonEvents::PersonSuspended(_)) => "PersonSuspended",
            DomainEvent::Person(crate::events::PersonEvents::PersonReactivated(_)) => "PersonReactivated",
            DomainEvent::Person(crate::events::PersonEvents::PersonArchived(_)) => "PersonArchived",
            // Location aggregate
            DomainEvent::Location(crate::events::LocationEvents::LocationCreated(_)) => "LocationCreated",
            DomainEvent::Location(crate::events::LocationEvents::LocationActivated(_)) => "LocationActivated",
            DomainEvent::Location(crate::events::LocationEvents::LocationSuspended(_)) => "LocationSuspended",
            DomainEvent::Location(crate::events::LocationEvents::LocationReactivated(_)) => "LocationReactivated",
            DomainEvent::Location(crate::events::LocationEvents::LocationDecommissioned(_)) => "LocationDecommissioned",
            // Organization aggregate
            DomainEvent::Organization(crate::events::OrganizationEvents::OrganizationCreated(_)) => "OrganizationCreated",
            DomainEvent::Organization(crate::events::OrganizationEvents::OrganizationalUnitCreated(_)) => "OrganizationalUnitCreated",
            DomainEvent::Organization(crate::events::OrganizationEvents::RoleCreated(_)) => "RoleCreated",
            DomainEvent::Organization(crate::events::OrganizationEvents::PolicyCreated(_)) => "PolicyCreated",
            // Manifest aggregate
            DomainEvent::Manifest(crate::events::ManifestEvents::ManifestCreated(_)) => "ManifestCreated",
            // Catch-all for any events not explicitly handled
            _ => "UnknownEvent",
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