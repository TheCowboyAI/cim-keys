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

    /// Handle a command by routing to the appropriate handler
    ///
    /// Routes KeyCommand variants to their corresponding handler functions.
    /// Each handler validates the command and emits domain events.
    pub async fn handle_command(
        &self,
        command: crate::commands::KeyCommand,
        _projection: &crate::projections::OfflineKeyProjection,
        _nats_port: Option<()>,
        #[cfg(feature = "policy")]
        _policy_engine: Option<()>,
    ) -> Result<Vec<crate::events::DomainEvent>, KeyManagementError> {
        use crate::commands::KeyCommand;

        // Route command to appropriate handler based on variant
        // Handlers are synchronous and return Result<EventType, String>
        // EventType has an `events` field containing Vec<DomainEvent>
        match command {
            KeyCommand::GenerateRootCA(cmd) => {
                let result = crate::commands::pki::handle_generate_root_ca(cmd)
                    .map_err(|e| KeyManagementError::CryptoError(e))?;
                Ok(result.events)
            }
            KeyCommand::GenerateCertificate(cmd) => {
                // Convert GUI GenerateCertificateCommand to handler GenerateCertificate
                // First generate a key pair for the certificate
                let key_purpose = if cmd.is_ca {
                    crate::value_objects::AuthKeyPurpose::X509ServerAuth // CA cert
                } else {
                    crate::value_objects::AuthKeyPurpose::X509ServerAuth // Server cert
                };

                // Create minimal key context for certificate generation
                let key_ownership = crate::domain::KeyOwnership {
                    person_id: Uuid::now_v7(), // System-generated
                    organization_id: Uuid::now_v7(), // From organization
                    role: crate::domain::KeyOwnerRole::SecurityAdmin,
                    delegations: vec![],
                };

                let key_context = crate::domain::KeyContext {
                    actor: key_ownership,
                    org_context: None,
                    nats_identity: None,
                    audit_requirements: vec![],
                };

                // Generate key pair first
                let key_pair_cmd = crate::commands::pki::GenerateKeyPair {
                    purpose: key_purpose,
                    algorithm: Some(crate::events::KeyAlgorithm::Ed25519),
                    owner_context: key_context,
                    correlation_id: Uuid::now_v7(),
                    causation_id: None,
                };

                let key_result = crate::commands::pki::handle_generate_key_pair(key_pair_cmd)
                    .map_err(|e| KeyManagementError::CryptoError(e))?;

                // Now generate certificate with the public key
                let cert_cmd = crate::commands::pki::GenerateCertificate {
                    subject: crate::value_objects::CertificateSubject {
                        common_name: cmd.subject.common_name,
                        organization: cmd.subject.organization,
                        organizational_unit: cmd.subject.organizational_unit,
                        country: cmd.subject.country,
                        state: cmd.subject.state_or_province,
                        locality: cmd.subject.locality,
                        email: None,
                    },
                    public_key: key_result.public_key,
                    key_id: key_result.key_id,
                    purpose: crate::events::KeyPurpose::Authentication,
                    validity_years: cmd.validity_days / 365,
                    ca_id: Uuid::now_v7(), // Self-signed for now
                    ca_certificate: None,
                    ca_algorithm: None,
                    correlation_id: Uuid::now_v7(),
                    causation_id: Some(key_result.key_id),
                };

                let cert_result = crate::commands::pki::handle_generate_certificate(cert_cmd)
                    .map_err(|e| KeyManagementError::CryptoError(e))?;

                // Combine events from key generation and certificate generation
                let mut all_events = key_result.events;
                all_events.extend(cert_result.events);
                Ok(all_events)
            }
            KeyCommand::GenerateSshKey(cmd) => {
                // Convert GUI GenerateSshKeyCommand to handler GenerateKeyPair
                // SSH keys use Ed25519 algorithm by default
                let key_purpose = crate::value_objects::AuthKeyPurpose::SshAuthentication;

                // Create key context from person_id
                let key_ownership = crate::domain::KeyOwnership {
                    person_id: cmd.person_id,
                    organization_id: Uuid::now_v7(), // Should come from projection
                    role: crate::domain::KeyOwnerRole::Developer,
                    delegations: vec![],
                };

                let key_context = crate::domain::KeyContext {
                    actor: key_ownership,
                    org_context: None,
                    nats_identity: None,
                    audit_requirements: vec![],
                };

                let key_pair_cmd = crate::commands::pki::GenerateKeyPair {
                    purpose: key_purpose,
                    algorithm: Some(crate::events::KeyAlgorithm::Ed25519), // Ed25519 for SSH
                    owner_context: key_context,
                    correlation_id: Uuid::now_v7(),
                    causation_id: None,
                };

                let result = crate::commands::pki::handle_generate_key_pair(key_pair_cmd)
                    .map_err(|e| KeyManagementError::CryptoError(e))?;

                Ok(result.events)
            }
            KeyCommand::ProvisionYubiKey(cmd) => {
                let result = crate::commands::yubikey::handle_provision_yubikey_slot(cmd)
                    .map_err(|e| KeyManagementError::InvalidCommand(e))?;
                Ok(result.events)
            }
            KeyCommand::ExportKeys(cmd) => {
                let result = crate::commands::export::handle_export_to_encrypted_storage(cmd)
                    .map_err(|e| KeyManagementError::ProjectionError(e))?;
                Ok(result.events)
            }
            KeyCommand::CreateOrganization(cmd) => {
                crate::commands::organization::handle_create_organization(cmd).await
            }
            KeyCommand::CreatePerson(cmd) => {
                crate::commands::organization::handle_create_person(cmd).await
            }
            KeyCommand::CreateLocation(cmd) => {
                crate::commands::organization::handle_create_location(cmd).await
            }
        }
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
