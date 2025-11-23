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
pub mod nats_operator;
pub mod nats_account;
pub mod nats_user;
pub mod yubikey;
pub mod relationship;
pub mod manifest;

// Re-export all aggregate event enums at module level for convenience
pub use person::PersonEvents;
pub use organization::OrganizationEvents;
pub use location::LocationEvents;
pub use certificate::CertificateEvents;
pub use key::KeyEvents;
pub use nats_operator::NatsOperatorEvents;
pub use nats_account::NatsAccountEvents;
pub use nats_user::NatsUserEvents;
pub use yubikey::YubiKeyEvents;
pub use relationship::RelationshipEvents;
pub use manifest::ManifestEvents;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "aggregate", content = "event")]
pub enum DomainEvent {
    Person(PersonEvents),
    Organization(OrganizationEvents),
    Location(LocationEvents),
    Certificate(CertificateEvents),
    Key(KeyEvents),
    NatsOperator(NatsOperatorEvents),
    NatsAccount(NatsAccountEvents),
    NatsUser(NatsUserEvents),
    YubiKey(YubiKeyEvents),
    Relationship(RelationshipEvents),
    Manifest(ManifestEvents),
}
