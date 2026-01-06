//! Event Modules Organized by Aggregate Root
//!
//! Following Domain-Driven Design principles, events are organized into separate
//! modules per aggregate root. Each aggregate owns its own event ontology.
//!
//! # Aggregate Roots
//!
//! - **Person** - Individual identities with roles and permissions
//! - **Organization** - Organizational structure, units, roles, and policies
//! - **Location** - Physical and virtual storage locations
//! - **Certificate** - X.509 digital certificates and PKI hierarchy
//! - **Key** - Cryptographic key material and operations
//! - **NatsOperator** - Top-level NATS security authority
//! - **NatsAccount** - NATS tenant/namespace within an operator
//! - **NatsUser** - NATS authenticated identity within an account
//! - **YubiKey** - Hardware security module operations
//! - **Relationship** - Connections between domain entities
//! - **Manifest** - Export tracking and metadata

// Re-export shared domain ontologies for convenience
pub use crate::types::*;

// Aggregate-specific event modules
pub mod person;
pub mod organization;
pub mod location;
pub mod certificate;
pub mod key;
pub mod delegation;
pub mod nats_operator;
pub mod nats_account;
pub mod nats_user;
pub mod yubikey;
pub mod relationship;
pub mod manifest;
pub mod saga;

// Re-export all aggregate event enums at module level for convenience
pub use person::PersonEvents;
pub use organization::OrganizationEvents;
pub use location::LocationEvents;
pub use certificate::CertificateEvents;
pub use key::KeyEvents;
pub use delegation::DelegationEvents;
pub use nats_operator::NatsOperatorEvents;
pub use nats_account::NatsAccountEvents;
pub use nats_user::NatsUserEvents;
pub use yubikey::YubiKeyEvents;
pub use relationship::RelationshipEvents;
pub use manifest::ManifestEvents;
pub use saga::SagaEvents;

// Re-export delegation event types
pub use delegation::{
    DelegationCreatedEvent, DelegationRevokedEvent, DelegationCascadeRevokedEvent,
    RevocationReason as DelegationRevocationReason,
};

// Re-export commonly used event structs for convenience
// This allows using crate::events::CertificateGeneratedEvent instead of
// crate::events::certificate::CertificateGeneratedEvent
pub use certificate::{CertificateGeneratedEvent, CertificateSignedEvent, CertificateRenewedEvent, PkiHierarchyCreatedEvent};
pub use yubikey::{YubiKeyProvisionedEvent, YubiKeyDetectedEvent};
pub use key::{KeyGeneratedEvent, KeyRevokedEvent, KeyStoredOfflineEvent};

use serde::{Deserialize, Serialize};

/// Unified domain event type that wraps all aggregate events
///
/// This enum provides a single entry point for event handling while maintaining
/// proper aggregate boundaries. Each variant delegates to the appropriate aggregate's
/// event enum.
///
/// # Future Compatibility
///
/// This enum is marked `#[non_exhaustive]` to allow adding new aggregate types
/// in future versions without breaking existing code. Consumers should always
/// include a catch-all arm in match expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "aggregate", content = "event")]
#[non_exhaustive]
pub enum DomainEvent {
    Person(PersonEvents),
    Organization(OrganizationEvents),
    Location(LocationEvents),
    Certificate(CertificateEvents),
    Key(KeyEvents),
    Delegation(DelegationEvents),
    NatsOperator(NatsOperatorEvents),
    NatsAccount(NatsAccountEvents),
    NatsUser(NatsUserEvents),
    YubiKey(YubiKeyEvents),
    Relationship(RelationshipEvents),
    Manifest(ManifestEvents),
    Saga(SagaEvents),
}

/// Event envelope that wraps domain events with routing and correlation metadata
///
/// The envelope provides a standardized wrapper for all domain events, enabling:
/// - **Event correlation**: Link related events across aggregates via correlation_id
/// - **Causal chains**: Track what event caused this one via causation_id
/// - **NATS routing**: Specify the subject for event publication
/// - **Temporal ordering**: Events have a timestamp for ordering
/// - **Content addressing**: Optional CID for immutable event identity (with `ipld` feature)
///
/// # NATS Subject Patterns
///
/// Events are routed using semantic subject naming:
/// - `organization.{org_id}.person.created`
/// - `organization.{org_id}.certificate.signed`
/// - `organization.{org_id}.yubikey.provisioned`
/// - `organization.{org_id}.nats.account.created`
///
/// # Content Addressing (IPLD)
///
/// When the `ipld` feature is enabled, events can be content-addressed using CIDs:
/// ```ignore
/// let envelope = EventEnvelope::new(event, correlation_id, None)
///     .with_cid()?;  // Generates CID from event content
/// ```
///
/// # Example
///
/// ```ignore
/// let envelope = EventEnvelope::new(
///     DomainEvent::Person(PersonEvents::Created(event)),
///     correlation_id,
///     Some(causation_id),
/// ).with_subject("organization.cowboyai.person.created");
/// ```
///
/// # Future Compatibility
///
/// This struct is marked `#[non_exhaustive]` to allow adding new fields
/// in future versions without breaking existing code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EventEnvelope {
    /// Unique identifier for this event instance
    pub event_id: uuid::Uuid,

    /// Correlation ID linking related events across a business process
    /// All events in a saga or workflow share the same correlation_id
    pub correlation_id: uuid::Uuid,

    /// ID of the event that caused this one (for causal chains)
    /// None for initial events in a chain
    pub causation_id: Option<uuid::Uuid>,

    /// NATS subject where this event should be published
    /// Follows semantic naming: {org}.{aggregate}.{action}
    pub nats_subject: String,

    /// Timestamp when this event was created (UTC)
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Content Identifier (CID) for immutable event identity
    /// Generated from the event content using SHA2-256 and DAG-CBOR codec.
    /// Present only when content addressing is explicitly requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,

    /// The wrapped domain event
    #[serde(flatten)]
    pub event: DomainEvent,
}

impl EventEnvelope {
    /// Create a new event envelope with a domain event
    pub fn new(
        event: DomainEvent,
        correlation_id: uuid::Uuid,
        causation_id: Option<uuid::Uuid>,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            correlation_id,
            causation_id,
            nats_subject: Self::default_subject(&event),
            timestamp: chrono::Utc::now(),
            cid: None,
            event,
        }
    }

    /// Generate and attach a CID (Content Identifier) to this envelope
    ///
    /// The CID is generated from the event content using SHA2-256 hashing
    /// and DAG-CBOR codec. This provides content-addressed identity for
    /// the event, enabling deduplication and integrity verification.
    ///
    /// # Feature
    ///
    /// Requires the `ipld` feature to be enabled. Without it, this method
    /// returns an error.
    #[cfg(feature = "ipld")]
    pub fn with_cid(mut self) -> Result<Self, crate::ipld_support::IpldError> {
        let cid = crate::ipld_support::generate_cid(&self.event)?;
        self.cid = Some(cid.to_string());
        Ok(self)
    }

    /// Generate and attach a CID (stub when IPLD feature disabled)
    #[cfg(not(feature = "ipld"))]
    pub fn with_cid(self) -> Result<Self, crate::ipld_support::IpldError> {
        Err(crate::ipld_support::IpldError::FeatureNotEnabled)
    }

    /// Verify that the event content matches its CID
    ///
    /// Returns `Ok(true)` if the CID matches, `Ok(false)` if it doesn't,
    /// or an error if verification cannot be performed.
    ///
    /// # Feature
    ///
    /// Requires the `ipld` feature to be enabled.
    #[cfg(feature = "ipld")]
    pub fn verify_cid(&self) -> Result<bool, crate::ipld_support::IpldError> {
        match &self.cid {
            Some(cid_str) => {
                let expected_cid = cid::Cid::try_from(cid_str.as_str())
                    .map_err(|e| crate::ipld_support::IpldError::CidParseError(e.to_string()))?;
                crate::ipld_support::verify_cid(&self.event, &expected_cid)
            }
            None => Ok(true), // No CID to verify
        }
    }

    /// Verify that the event content matches its CID (stub when IPLD feature disabled)
    #[cfg(not(feature = "ipld"))]
    pub fn verify_cid(&self) -> Result<bool, crate::ipld_support::IpldError> {
        Ok(true) // No verification when IPLD disabled
    }

    /// Check if this envelope has a CID attached
    pub fn has_cid(&self) -> bool {
        self.cid.is_some()
    }

    /// Get the CID string if present
    pub fn cid_string(&self) -> Option<&str> {
        self.cid.as_deref()
    }

    /// Set a custom NATS subject for this event
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.nats_subject = subject.into();
        self
    }

    /// Create with an organization-scoped subject
    pub fn with_org_subject(mut self, org_id: &str, aggregate: &str, action: &str) -> Self {
        self.nats_subject = format!("organization.{}.{}.{}", org_id, aggregate, action);
        self
    }

    /// Generate a default NATS subject based on event type
    fn default_subject(event: &DomainEvent) -> String {
        match event {
            DomainEvent::Person(_) => "cim.person.event".to_string(),
            DomainEvent::Organization(_) => "cim.organization.event".to_string(),
            DomainEvent::Location(_) => "cim.location.event".to_string(),
            DomainEvent::Certificate(_) => "cim.certificate.event".to_string(),
            DomainEvent::Key(_) => "cim.key.event".to_string(),
            DomainEvent::Delegation(_) => "cim.delegation.event".to_string(),
            DomainEvent::NatsOperator(_) => "cim.nats.operator.event".to_string(),
            DomainEvent::NatsAccount(_) => "cim.nats.account.event".to_string(),
            DomainEvent::NatsUser(_) => "cim.nats.user.event".to_string(),
            DomainEvent::YubiKey(_) => "cim.yubikey.event".to_string(),
            DomainEvent::Relationship(_) => "cim.relationship.event".to_string(),
            DomainEvent::Manifest(_) => "cim.manifest.event".to_string(),
            DomainEvent::Saga(e) => format!("cim.{}", e.event_type()),
        }
    }

    /// Get the aggregate type from the event
    pub fn aggregate_type(&self) -> &'static str {
        match &self.event {
            DomainEvent::Person(_) => "Person",
            DomainEvent::Organization(_) => "Organization",
            DomainEvent::Location(_) => "Location",
            DomainEvent::Certificate(_) => "Certificate",
            DomainEvent::Key(_) => "Key",
            DomainEvent::Delegation(_) => "Delegation",
            DomainEvent::NatsOperator(_) => "NatsOperator",
            DomainEvent::NatsAccount(_) => "NatsAccount",
            DomainEvent::NatsUser(_) => "NatsUser",
            DomainEvent::YubiKey(_) => "YubiKey",
            DomainEvent::Relationship(_) => "Relationship",
            DomainEvent::Manifest(_) => "Manifest",
            DomainEvent::Saga(_) => "Saga",
        }
    }

    /// Check if this event is part of the same correlation chain
    pub fn is_correlated_with(&self, other: &EventEnvelope) -> bool {
        self.correlation_id == other.correlation_id
    }

    /// Check if this event was caused by another event
    pub fn is_caused_by(&self, other: &EventEnvelope) -> bool {
        self.causation_id == Some(other.event_id)
    }
}

/// Builder for creating event chains with proper correlation
pub struct EventChainBuilder {
    correlation_id: uuid::Uuid,
    last_event_id: Option<uuid::Uuid>,
    org_id: Option<String>,
}

impl EventChainBuilder {
    /// Start a new event chain with a fresh correlation ID
    pub fn new() -> Self {
        Self {
            correlation_id: uuid::Uuid::now_v7(),
            last_event_id: None,
            org_id: None,
        }
    }

    /// Start a new chain with a specific correlation ID (e.g., from incoming request)
    pub fn with_correlation_id(correlation_id: uuid::Uuid) -> Self {
        Self {
            correlation_id,
            last_event_id: None,
            org_id: None,
        }
    }

    /// Set the organization ID for subject generation
    pub fn for_organization(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    /// Create an envelope for the next event in the chain
    pub fn envelope(&mut self, event: DomainEvent) -> EventEnvelope {
        let mut envelope = EventEnvelope::new(
            event,
            self.correlation_id,
            self.last_event_id,
        );

        // Update the chain with this event's ID for the next causation
        self.last_event_id = Some(envelope.event_id);

        // Apply org scoping if set
        if let Some(ref org_id) = self.org_id {
            let aggregate = envelope.aggregate_type().to_lowercase();
            envelope = envelope.with_org_subject(org_id, &aggregate, "event");
        }

        envelope
    }

    /// Get the current correlation ID
    pub fn correlation_id(&self) -> uuid::Uuid {
        self.correlation_id
    }
}

impl Default for EventChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}
