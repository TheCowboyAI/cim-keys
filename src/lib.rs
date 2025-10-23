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

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// Core modules following FRP/Event-sourcing
pub mod events;
pub mod commands;
pub mod aggregate;
pub mod projections;

// Ports & Adapters for external integrations
pub mod ports;
pub mod adapters;

// Master domain model - cim-keys owns the initial domain creation
pub mod domain;

// Certificate generation service
pub mod certificate_service;

// GUI for offline key generation (native and WASM)
#[cfg(feature = "gui")]
pub mod gui;

// Policy integration for PKI operations
#[cfg(feature = "policy")]
pub mod policy;

// Re-export core types
pub use events::{KeyEvent, KeyAlgorithm, KeyPurpose, KeyMetadata};
pub use commands::{KeyCommand, GenerateKeyCommand, GenerateCertificateCommand};
pub use aggregate::{KeyManagementAggregate, KeyManagementError};
pub use projections::{OfflineKeyProjection, KeyManifest, ProjectionError};
pub use domain::{
    Organization, OrganizationUnit, Person, PersonRole, Location,
    KeyOwnership, KeyStorageLocation, KeyContext, NatsIdentity,
    OrganizationalPKI, ServiceAccount
};

// Re-export from cim-domain for convenience
pub use cim_domain::{
    DomainEvent, Command, CommandId, EventId,
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
    pub use crate::commands::*;
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