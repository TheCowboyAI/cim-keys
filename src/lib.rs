//! # CIM Keys - Event-Sourced Cryptographic Key Management
//!
//! This crate provides event-sourced, functional reactive key management for the CIM architecture.
//! All operations are modeled as immutable events with projections to offline storage.
//!
//! ## Architecture
//!
//! - **Event-Sourced**: All operations produce immutable events
//! - **FRP Design**: No mutable state, pure functions process commands to events
//! - **Offline-First**: Designed for air-gapped key generation and storage
//! - **Projection-Based**: State is projected to JSON files on encrypted partitions
//!
//! ## Key Features
//!
//! - YubiKey hardware token support for secure key storage
//! - Complete PKI hierarchy generation for enterprise deployments
//! - SSH and GPG key generation for developer authentication
//! - X.509 certificate generation and management
//! - Offline storage to encrypted SD cards for air-gapped security
//!
//! ## Usage Pattern
//!
//! ```rust,ignore
//! use cim_keys::{KeyCommand, KeyEvent, OfflineKeyProjection};
//!
//! // Create projection to encrypted partition
//! let mut projection = OfflineKeyProjection::new("/mnt/encrypted")?;
//!
//! // Process a command to generate events
//! let command = GenerateKeyCommand { ... };
//! let events = aggregate.handle_command(command)?;
//!
//! // Apply events to update projection (writes to disk)
//! for event in events {
//!     projection.apply(&event)?;
//! }
//! ```

// TODO: Re-enable missing_docs warnings after adding comprehensive documentation
#![allow(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// Core modules following FRP/Event-sourcing
pub mod types; // Shared domain ontologies (value objects, enums)
pub mod events; // DDD-organized events by aggregate root
pub mod commands;
pub mod aggregate;
pub mod projections;

// Graph projection layer - Functorial lifting of domain events to cim-graph
pub mod graph_projection;

// LiftableDomain - Faithful functor for domain composition (Sprint 7)
#[cfg(feature = "gui")]
pub mod lifting;

// Configuration management
pub mod config;

// Ports & Adapters for external integrations
pub mod ports;
pub mod adapters;

// Cryptographic primitives for deterministic key generation
pub mod crypto;

// MVI (Model-View-Intent) architecture for GUI
#[cfg(feature = "gui")]
pub mod mvi;

// N-ary FRP Signal System - Axioms A1 & A2
pub mod signals;

// Compositional Routing - Axiom A6
pub mod routing;

// Causality Enforcement - Axiom A4
pub mod causality;

// Feedback Combinators - Axiom A8
pub mod combinators;

// Master domain model - cim-keys owns the initial domain creation
pub mod domain;

// State machines for workflow control
pub mod state_machines;

// Value objects for cryptographic artifacts
pub mod value_objects;

// Policy types for authorization and governance
pub mod policy_types;

// NATS identity types for security configuration
pub mod nats_identity_types;

// IPLD support for content-addressed storage
pub mod ipld_support;

// Domain projections - functors mapping domain to library formats
pub mod domain_projections;

// Secrets loader for importing configuration from JSON
pub mod secrets_loader;

// CLAN bootstrap loader for clan-bootstrap.json configuration
pub mod clan_bootstrap;

// Policy bootstrap loader for policy-bootstrap.json configuration
pub mod policy_loader;

// Certificate generation service
pub mod certificate_service;

// GUI for offline key generation (native and WASM)
#[cfg(feature = "gui")]
pub mod gui;

// Material Icons for GUI
// TODO: Re-enable full implementation after GUI refactoring
// Using stub module for now to allow compilation
pub mod icons;

// Policy integration for PKI operations
#[cfg(feature = "policy")]
pub mod policy;

// Re-export core types
pub use events::DomainEvent;
pub use types::KeyMetadata;
// TODO: Re-export command types when modular structure is stabilized
// pub use commands::{...};
pub use aggregate::{KeyManagementAggregate, KeyManagementError};
pub use projections::{OfflineKeyProjection, KeyManifest, ProjectionError};
pub use domain::{
    // Bootstrap types (for JSON loading)
    Organization, OrganizationUnit, Person, PersonRole, Location,
    KeyOwnership, KeyContext, NatsIdentity,
    OrganizationalPKI, ServiceAccount, KeyOwnerRole,
    // Re-export Location types from cim-domain-location
    LocationMarker, Address, GeoCoordinates, LocationType, VirtualLocation,
};

// Re-export canonical domain types from cim-domain-* crates (when policy feature enabled)
#[cfg(feature = "policy")]
pub use domain::{
    // Organization domain
    DomainOrganization, DomainOrganizationUnit, DomainDepartment, DomainTeam, DomainRole,
    OrganizationType, OrganizationStatus, DomainOrganizationUnitType,
    // Person domain
    DomainPerson, PersonId, PersonMarker, PersonName,
    EmploymentRelationship, EmploymentRole,
    // Relationship domain
    EdgeConcept, RelationshipCategory, EntityRef, RelationshipEntityType, RelationshipQuality,
};

// Re-export from cim-domain for convenience
pub use cim_domain::{
    Command, CommandId, EventId,
    CausationId, CorrelationId, EntityId
};

// Re-export policy types when feature is enabled
#[cfg(feature = "policy")]
pub use policy::{
    KeyPolicyEngine, PkiPolicySet, PolicyError,
    KeyPolicyCommand, KeyPolicyEvent,
    YubikeyConfig
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::events::*;
    // TODO: Re-add command exports when modular structure is stabilized
    // pub use crate::commands::*;
    pub use crate::aggregate::*;
    pub use crate::projections::*;

    #[cfg(feature = "policy")]
    pub use crate::policy::*;
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the library version string
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}