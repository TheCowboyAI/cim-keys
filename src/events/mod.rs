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

    /// Domain CID for fast content addressing using Blake3
    /// Generated from the event content using cim_domain::cid infrastructure.
    /// Use this for NATS JetStream deduplication and internal operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_cid: Option<String>,

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
            domain_cid: None,
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

    /// Generate and attach a Domain CID using cim_domain::cid infrastructure
    ///
    /// Uses Blake3 hashing for fast content addressing. This is optimized for:
    /// - NATS JetStream message deduplication
    /// - Internal event deduplication
    /// - Fast integrity checks
    ///
    /// For standard IPLD compatibility (IPFS ecosystem), use `with_cid()` instead.
    pub fn with_domain_cid(mut self) -> Result<Self, String> {
        use cim_domain::cid::{generate_cid, ContentType};
        let cid = generate_cid(&self.event, ContentType::Event)?;
        self.domain_cid = Some(cid.to_string());
        Ok(self)
    }

    /// Check if this envelope has a Domain CID attached
    pub fn has_domain_cid(&self) -> bool {
        self.domain_cid.is_some()
    }

    /// Get the Domain CID string if present
    pub fn domain_cid_string(&self) -> Option<&str> {
        self.domain_cid.as_deref()
    }

    /// Get the preferred CID for NATS message ID (domain CID if present, else IPLD CID)
    ///
    /// Returns the domain CID (Blake3) if available, falling back to IPLD CID.
    /// Domain CID is preferred for NATS because it's faster to compute.
    pub fn message_id(&self) -> Option<&str> {
        self.domain_cid.as_deref().or(self.cid.as_deref())
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_test_event() -> DomainEvent {
        use organization::OrganizationCreatedEvent;
        DomainEvent::Organization(OrganizationEvents::OrganizationCreated(OrganizationCreatedEvent {
            organization_id: Uuid::parse_str("019447d8-1234-7000-8000-000000000001").unwrap(),
            name: "Test Org".to_string(),
            domain: Some("testorg.com".to_string()),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        }))
    }

    #[test]
    fn test_event_envelope_new() {
        let event = make_test_event();
        let correlation_id = Uuid::now_v7();
        let envelope = EventEnvelope::new(event, correlation_id, None);

        assert_eq!(envelope.correlation_id, correlation_id);
        assert!(envelope.causation_id.is_none());
        assert!(!envelope.has_cid());
        assert!(!envelope.has_domain_cid());
    }

    #[test]
    fn test_domain_cid_generation() {
        let event = make_test_event();
        let correlation_id = Uuid::now_v7();
        let envelope = EventEnvelope::new(event, correlation_id, None)
            .with_domain_cid()
            .expect("Domain CID generation should succeed");

        assert!(envelope.has_domain_cid());
        assert!(envelope.domain_cid_string().is_some());
        let cid = envelope.domain_cid_string().unwrap();
        assert!(!cid.is_empty());
    }

    #[test]
    fn test_same_event_same_domain_cid() {
        // Create two identical events with deterministic IDs
        let fixed_correlation = Uuid::parse_str("019447d8-5678-7000-8000-000000000099").unwrap();
        let event1 = DomainEvent::Organization(OrganizationEvents::OrganizationCreated(
            organization::OrganizationCreatedEvent {
                organization_id: Uuid::parse_str("019447d8-1234-7000-8000-000000000001").unwrap(),
                name: "Test Org".to_string(),
                domain: Some("testorg.com".to_string()),
                correlation_id: fixed_correlation,
                causation_id: None,
            },
        ));
        let event2 = DomainEvent::Organization(OrganizationEvents::OrganizationCreated(
            organization::OrganizationCreatedEvent {
                organization_id: Uuid::parse_str("019447d8-1234-7000-8000-000000000001").unwrap(),
                name: "Test Org".to_string(),
                domain: Some("testorg.com".to_string()),
                correlation_id: fixed_correlation,
                causation_id: None,
            },
        ));

        // Generate CIDs directly from events (not envelopes)
        use cim_domain::cid::{generate_cid, ContentType};
        let cid1 = generate_cid(&event1, ContentType::Event).unwrap();
        let cid2 = generate_cid(&event2, ContentType::Event).unwrap();

        // Same event content = same CID (deterministic)
        assert_eq!(cid1, cid2);
    }

    #[test]
    fn test_message_id_prefers_domain_cid() {
        let event = make_test_event();
        let correlation_id = Uuid::now_v7();

        // Without any CID
        let envelope = EventEnvelope::new(event.clone(), correlation_id, None);
        assert!(envelope.message_id().is_none());

        // With domain CID
        let envelope_with_cid = EventEnvelope::new(event, correlation_id, None)
            .with_domain_cid()
            .unwrap();
        assert!(envelope_with_cid.message_id().is_some());
        assert_eq!(
            envelope_with_cid.message_id(),
            envelope_with_cid.domain_cid_string()
        );
    }

    #[test]
    fn test_event_chain_builder() {
        let mut builder = EventChainBuilder::new();
        let event1 = make_test_event();
        let event2 = make_test_event();

        let env1 = builder.envelope(event1);
        let env2 = builder.envelope(event2);

        // Events should be correlated
        assert!(env1.is_correlated_with(&env2));

        // Second event should be caused by first
        assert!(env2.is_caused_by(&env1));
    }

    #[test]
    fn test_event_chain_with_domain_cid() {
        let mut builder = EventChainBuilder::new();
        let event = make_test_event();

        let envelope = builder
            .envelope(event)
            .with_domain_cid()
            .expect("CID generation should succeed");

        assert!(envelope.has_domain_cid());
        assert!(envelope.message_id().is_some());
    }
}
