//! Key management aggregate root
//!
//! REFACTORING IN PROGRESS:
//! The aggregate pattern is being refactored to work with the new modular command structure.
//! Command handlers are now in src/commands/ as standalone functions.
//! This file provides minimal type definitions for backwards compatibility.
//!
//! TODO: Fully refactor aggregate to coordinate the new command handlers

use cim_domain::AggregateRoot;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Key management aggregate root
///
/// This is a pure functional aggregate that processes commands and emits events.
/// State is reconstructed from the event stream via projections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementAggregate {
    pub id: Uuid,  // Aggregate ID
    pub version: u64,
}

impl KeyManagementAggregate {
    /// Create a new key management aggregate
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
        }
    }

    /// Handle a command (legacy stub for GUI compatibility)
    ///
    /// TODO: Implement actual command handling coordinating with new modular commands
    pub async fn handle_command<C>(
        &self,
        _command: C,
        _projection: &crate::projections::OfflineKeyProjection,
        _nats_port: Option<()>,
        #[cfg(feature = "policy")]
        _policy_engine: Option<()>,
    ) -> Result<Vec<crate::events::KeyEvent>, KeyManagementError> {
        // Stub implementation - returns empty events
        // In the future, this will delegate to the appropriate command handler
        Ok(vec![])
    }
}

/// Errors that can occur during key management operations
#[derive(Debug, thiserror::Error)]
pub enum KeyManagementError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Projection error: {0}")]
    ProjectionError(String),

    #[error("NATS error: {0}")]
    NatsError(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),
}

impl AggregateRoot for KeyManagementAggregate {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

// TODO: Re-implement command handlers to work with new modular command structure
// For now, command handlers are standalone functions in src/commands/
// - nats_identity.rs: handle_create_nats_operator, handle_create_nats_account, handle_create_nats_user
// - yubikey.rs: handle_configure_yubikey_security, handle_provision_yubikey_slot
// - pki.rs: handle_generate_key_pair, handle_generate_root_ca, handle_generate_certificate
// - export.rs: handle_export_to_encrypted_storage
