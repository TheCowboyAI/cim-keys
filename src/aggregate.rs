//! Key management aggregate root
//!
//! The KeyManagement aggregate is the consistency boundary for all key operations.
//! It processes commands and emits events without holding mutable state.

use cim_domain::AggregateRoot;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::events::{KeyEvent, KeyGeneratedEvent, CertificateGeneratedEvent,
    NatsOperatorCreatedEvent, NatsAccountCreatedEvent, NatsUserCreatedEvent,
    NatsEntityType, NatsPermissions as EventNatsPermissions};
use crate::commands::{KeyCommand, GenerateKeyCommand, GenerateCertificateCommand,
    CreateNatsOperatorCommand, CreateNatsAccountCommand, CreateNatsUserCommand};
use crate::projections::OfflineKeyProjection;
use crate::ports::nats::NatsKeyPort;

#[cfg(feature = "policy")]
use crate::policy::{KeyPolicyEngine, YubikeyConfig};

/// Key management aggregate root
///
/// This is a pure functional aggregate that processes commands and emits events.
/// State is reconstructed from the event stream via projections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementAggregate {
    pub id: uuid::Uuid,  // Aggregate ID
    pub version: u64,
}

impl Default for KeyManagementAggregate {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyManagementAggregate {
    /// Create a new key management aggregate
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::now_v7(),
            version: 0,
        }
    }

    /// Process a command and return resulting events
    ///
    /// This is a pure function - it takes the current projection state, a command,
    /// an optional NATS port for key operations, and an optional policy engine.
    /// Returns the events that should be emitted. No mutable state is held.
    pub async fn handle_command(
        &self,
        command: KeyCommand,
        projection: &OfflineKeyProjection,
        nats_port: Option<&dyn NatsKeyPort>,
        #[cfg(feature = "policy")]
        policy_engine: Option<&mut KeyPolicyEngine>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        match command {
            KeyCommand::GenerateKey(cmd) => self.handle_generate_key(
                cmd,
                projection,
                #[cfg(feature = "policy")]
                policy_engine,
            ),
            KeyCommand::GenerateCertificate(cmd) => self.handle_generate_certificate(
                cmd,
                projection,
                #[cfg(feature = "policy")]
                policy_engine,
            ),
            KeyCommand::SignCertificate(cmd) => self.handle_sign_certificate(cmd, projection),
            KeyCommand::ImportKey(cmd) => self.handle_import_key(cmd, projection),
            KeyCommand::ExportKey(cmd) => self.handle_export_key(cmd, projection),
            KeyCommand::StoreKeyOffline(cmd) => self.handle_store_key_offline(cmd, projection),
            KeyCommand::ProvisionYubiKey(cmd) => self.handle_provision_yubikey(
                cmd,
                projection,
                #[cfg(feature = "policy")]
                policy_engine,
            ),
            KeyCommand::GenerateSshKey(cmd) => self.handle_generate_ssh_key(cmd, projection),
            KeyCommand::GenerateGpgKey(cmd) => self.handle_generate_gpg_key(cmd, projection),
            KeyCommand::RevokeKey(cmd) => self.handle_revoke_key(cmd, projection),
            KeyCommand::EstablishTrust(cmd) => self.handle_establish_trust(cmd, projection),
            KeyCommand::CreatePkiHierarchy(cmd) => self.handle_create_pki_hierarchy(cmd, projection),
            KeyCommand::CreateNatsOperator(cmd) => self.handle_create_nats_operator(
                cmd,
                nats_port,
                #[cfg(feature = "policy")]
                policy_engine,
            ).await,
            KeyCommand::CreateNatsAccount(cmd) => self.handle_create_nats_account(cmd, nats_port).await,
            KeyCommand::CreateNatsUser(cmd) => self.handle_create_nats_user(cmd, nats_port).await,
            KeyCommand::GenerateNatsSigningKey(cmd) => self.handle_generate_nats_signing_key(cmd, nats_port).await,
            KeyCommand::SetNatsPermissions(cmd) => self.handle_set_nats_permissions(cmd, projection),
            KeyCommand::ExportNatsConfig(cmd) => self.handle_export_nats_config(cmd, nats_port).await,
        }
    }

    fn handle_generate_key(
        &self,
        cmd: GenerateKeyCommand,
        _projection: &OfflineKeyProjection,
        #[cfg(feature = "policy")]
        policy_engine: Option<&mut KeyPolicyEngine>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        // Validate command
        if cmd.label.is_empty() {
            return Err(KeyManagementError::InvalidCommand("Key label cannot be empty".to_string()));
        }

        // Extract domain context and prepare for policy evaluation
        let context = cmd.context.as_ref();

        // Policy evaluation for key generation
        #[cfg(feature = "policy")]
        if let Some(engine) = policy_engine {
            if let Some(ctx) = context {
                // Create a Person struct for policy evaluation
                let person = crate::domain::Person {
                    id: ctx.actor.person_id,
                    name: cmd.requestor.clone(),
                    email: format!("{}@example.com", cmd.requestor), // Placeholder
                    organization_id: ctx.actor.organization_id,
                    unit_ids: vec![],
                    roles: vec![crate::domain::PersonRole {
                        role_type: match ctx.actor.role {
                            crate::domain::KeyOwnerRole::RootAuthority => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::SecurityAdmin => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::Developer => crate::domain::RoleType::Developer,
                            crate::domain::KeyOwnerRole::ServiceAccount => crate::domain::RoleType::Service,
                            crate::domain::KeyOwnerRole::BackupHolder => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::Auditor => crate::domain::RoleType::Auditor,
                        },
                        scope: crate::domain::RoleScope::Organization,
                        permissions: vec![],
                    }],
                    created_at: chrono::Utc::now(),
                    active: true,
                };

                // Get algorithm string and key size
                let (algorithm, key_size) = match &cmd.algorithm {
                    crate::events::KeyAlgorithm::Rsa { bits } => ("RSA", *bits),
                    crate::events::KeyAlgorithm::Ecdsa { curve } => {
                        let size = match curve.as_str() {
                            "P-256" => 256,
                            "P-384" => 384,
                            "P-521" => 521,
                            _ => 256,
                        };
                        ("ECDSA", size)
                    },
                    crate::events::KeyAlgorithm::Ed25519 => ("Ed25519", 256),
                    crate::events::KeyAlgorithm::Secp256k1 => ("Secp256k1", 256),
                };

                let purpose_str = format!("{:?}", cmd.purpose);

                // Evaluate policy
                if let Err(e) = engine.evaluate_key_generation(
                    &person,
                    algorithm,
                    key_size,
                    &purpose_str,
                ) {
                    return Err(KeyManagementError::PolicyViolation(e.to_string()));
                }
            }
        }

        // Generate the key ID
        let key_id = Uuid::now_v7();

        // Extract domain context if provided
        let (ownership, storage_location) = if let Some(context) = cmd.context {
            (Some(context.actor), context.location)
        } else {
            (None, None)
        };

        // Create the event
        let event = KeyGeneratedEvent {
            key_id,
            algorithm: cmd.algorithm,
            purpose: cmd.purpose,
            generated_at: chrono::Utc::now(),
            generated_by: cmd.requestor,
            hardware_backed: cmd.hardware_backed,
            metadata: crate::events::KeyMetadata {
                label: cmd.label,
                description: None,
                tags: vec![],
                attributes: HashMap::new(),
            },
            ownership,
            storage_location,
        };

        Ok(vec![KeyEvent::KeyGenerated(event)])
    }

    fn handle_generate_certificate(
        &self,
        cmd: GenerateCertificateCommand,
        projection: &OfflineKeyProjection,
        #[cfg(feature = "policy")]
        policy_engine: Option<&mut KeyPolicyEngine>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        // Check if key exists in projection
        if !projection.key_exists(&cmd.key_id) {
            return Err(KeyManagementError::KeyNotFound(cmd.key_id));
        }

        // Policy evaluation for certificate issuance
        #[cfg(feature = "policy")]
        if let Some(engine) = policy_engine {
            if let Some(ctx) = cmd.context.as_ref() {
                // Create a Person struct for policy evaluation
                let person = crate::domain::Person {
                    id: ctx.actor.person_id,
                    name: cmd.requestor.clone(),
                    email: format!("{}@example.com", cmd.requestor),
                    organization_id: ctx.actor.organization_id,
                    unit_ids: vec![],
                    roles: vec![crate::domain::PersonRole {
                        role_type: match ctx.actor.role {
                            crate::domain::KeyOwnerRole::RootAuthority => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::SecurityAdmin => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::Developer => crate::domain::RoleType::Developer,
                            crate::domain::KeyOwnerRole::ServiceAccount => crate::domain::RoleType::Service,
                            crate::domain::KeyOwnerRole::BackupHolder => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::Auditor => crate::domain::RoleType::Auditor,
                        },
                        scope: crate::domain::RoleScope::Organization,
                        permissions: vec![],
                    }],
                    created_at: chrono::Utc::now(),
                    active: true,
                };

                // Prepare key usage strings
                let key_usage_strs: Vec<String> = cmd.key_usage.iter()
                    .map(|u| format!("{:?}", u))
                    .collect();

                // Evaluate policy
                if let Err(e) = engine.evaluate_certificate_issuance(
                    &person,
                    cmd.validity_days,
                    &cmd.subject.organization.clone().unwrap_or_default(),
                    key_usage_strs,
                ) {
                    return Err(KeyManagementError::PolicyViolation(e.to_string()));
                }
            }
        }

        // Generate certificate ID
        let cert_id = Uuid::now_v7();

        // Create the event
        let event = CertificateGeneratedEvent {
            cert_id,
            key_id: cmd.key_id,
            subject: cmd.subject.common_name,
            issuer: None, // Self-signed for now
            not_before: chrono::Utc::now(),
            not_after: chrono::Utc::now() + chrono::Duration::days(cmd.validity_days as i64),
            is_ca: cmd.is_ca,
            san: cmd.san,
            key_usage: cmd.key_usage,
            extended_key_usage: cmd.extended_key_usage,
        };

        Ok(vec![KeyEvent::CertificateGenerated(event)])
    }

    // Stub implementations for other handlers
    fn handle_sign_certificate(
        &self,
        _cmd: crate::commands::SignCertificateCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        Ok(vec![])
    }

    fn handle_import_key(
        &self,
        _cmd: crate::commands::ImportKeyCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        Ok(vec![])
    }

    fn handle_export_key(
        &self,
        _cmd: crate::commands::ExportKeyCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        Ok(vec![])
    }

    fn handle_store_key_offline(
        &self,
        _cmd: crate::commands::StoreKeyOfflineCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        Ok(vec![])
    }

    fn handle_provision_yubikey(
        &self,
        cmd: crate::commands::ProvisionYubiKeyCommand,
        _projection: &OfflineKeyProjection,
        #[cfg(feature = "policy")]
        policy_engine: Option<&mut KeyPolicyEngine>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        // Policy evaluation for YubiKey provisioning
        #[cfg(feature = "policy")]
        if let Some(engine) = policy_engine {
            if let Some(ctx) = cmd.context.as_ref() {
                // Create a Person struct for policy evaluation
                let person = crate::domain::Person {
                    id: ctx.actor.person_id,
                    name: cmd.requestor.clone(),
                    email: format!("{}@example.com", cmd.requestor),
                    organization_id: ctx.actor.organization_id,
                    unit_ids: vec![],
                    roles: vec![crate::domain::PersonRole {
                        role_type: match ctx.actor.role {
                            crate::domain::KeyOwnerRole::RootAuthority => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::SecurityAdmin => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::Developer => crate::domain::RoleType::Developer,
                            crate::domain::KeyOwnerRole::ServiceAccount => crate::domain::RoleType::Service,
                            crate::domain::KeyOwnerRole::BackupHolder => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::Auditor => crate::domain::RoleType::Auditor,
                        },
                        scope: crate::domain::RoleScope::Organization,
                        permissions: vec![],
                    }],
                    created_at: chrono::Utc::now(),
                    active: true,
                };

                // Create YubiKey configuration for evaluation
                // This is a simplified version - in production, would query actual YubiKey state
                let yubikey_config = YubikeyConfig {
                    pin_configured: true, // Assume PIN will be configured
                    puk_configured: true, // Assume PUK will be configured
                    touch_policy: "always".to_string(), // Default to always require touch
                    management_key_status: if cmd.management_key.is_some() {
                        "custom".to_string()
                    } else {
                        "default".to_string()
                    },
                    firmware_version: "5.4".to_string(), // Assume modern YubiKey
                };

                // Evaluate policy
                if let Err(e) = engine.evaluate_yubikey_provisioning(&person, &yubikey_config) {
                    return Err(KeyManagementError::PolicyViolation(e.to_string()));
                }
            }
        }

        // TODO: Implement actual YubiKey provisioning logic
        // For now, return empty event list as this is a stub
        Ok(vec![])
    }

    fn handle_generate_ssh_key(
        &self,
        cmd: crate::commands::GenerateSshKeyCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        // Generate key ID
        let key_id = Uuid::now_v7();

        // Determine SSH key type enum from string
        let ssh_key_type = match cmd.key_type.as_str() {
            "ed25519" => crate::events::SshKeyType::Ed25519,
            "rsa" | "rsa-4096" => crate::events::SshKeyType::Rsa,
            "ecdsa" | "ecdsa-256" => crate::events::SshKeyType::Ecdsa,
            _ => return Err(KeyManagementError::InvalidCommand(
                format!("Unsupported SSH key type: {}", cmd.key_type)
            )),
        };

        // Create SSH key generated event
        // Note: Actual cryptographic key generation would happen in the projection
        // or in a dedicated service that handles the event
        let event = crate::events::SshKeyGeneratedEvent {
            key_id,
            key_type: ssh_key_type,
            comment: cmd.comment.clone(),
            generated_at: chrono::Utc::now(),
        };

        // Note: In a real implementation:
        // 1. The actual key generation happens when processing the event
        // 2. Keys are stored securely in the projection
        // 3. Private keys are encrypted before storage
        // 4. Public keys and fingerprints are calculated during projection

        Ok(vec![KeyEvent::SshKeyGenerated(event)])
    }

    fn handle_generate_gpg_key(
        &self,
        _cmd: crate::commands::GenerateGpgKeyCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        Ok(vec![])
    }

    fn handle_revoke_key(
        &self,
        _cmd: crate::commands::RevokeKeyCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        Ok(vec![])
    }

    fn handle_establish_trust(
        &self,
        _cmd: crate::commands::EstablishTrustCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        Ok(vec![])
    }

    fn handle_create_pki_hierarchy(
        &self,
        _cmd: crate::commands::CreatePkiHierarchyCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        Ok(vec![])
    }

    // NATS command handlers
    async fn handle_create_nats_operator(
        &self,
        cmd: CreateNatsOperatorCommand,
        nats_port: Option<&dyn NatsKeyPort>,
        #[cfg(feature = "policy")]
        policy_engine: Option<&mut KeyPolicyEngine>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        // Policy evaluation for NATS operator key generation
        #[cfg(feature = "policy")]
        if let Some(engine) = policy_engine {
            if let Some(ctx) = cmd.context.as_ref() {
                // Create a Person struct for policy evaluation
                let person = crate::domain::Person {
                    id: ctx.actor.person_id,
                    name: cmd.requestor.clone(),
                    email: format!("{}@example.com", cmd.requestor),
                    organization_id: ctx.actor.organization_id,
                    unit_ids: vec![],
                    roles: vec![crate::domain::PersonRole {
                        role_type: match ctx.actor.role {
                            crate::domain::KeyOwnerRole::RootAuthority => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::SecurityAdmin => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::Developer => crate::domain::RoleType::Developer,
                            crate::domain::KeyOwnerRole::ServiceAccount => crate::domain::RoleType::Service,
                            crate::domain::KeyOwnerRole::BackupHolder => crate::domain::RoleType::Administrator,
                            crate::domain::KeyOwnerRole::Auditor => crate::domain::RoleType::Auditor,
                        },
                        scope: crate::domain::RoleScope::Organization,
                        permissions: vec![],
                    }],
                    created_at: chrono::Utc::now(),
                    active: true,
                };

                // Evaluate policy for NATS operator key
                // NATS always uses Ed25519
                if let Err(e) = engine.evaluate_nats_operator_key(
                    &person,
                    "Ed25519",
                    "offline", // Operator keys should be stored offline
                    2,         // Require at least 2 signatures for operator modifications
                    true,      // Backup must exist
                ) {
                    return Err(KeyManagementError::PolicyViolation(e.to_string()));
                }
            }
        }

        let port = nats_port.ok_or_else(|| {
            KeyManagementError::InvalidCommand("NATS port not configured".to_string())
        })?;

        // Use the port to generate operator keys
        let operator_keys = port.generate_operator(&cmd.name)
            .await
            .map_err(|e| KeyManagementError::OperationFailed(format!("Failed to generate operator: {}", e)))?;

        // Create event
        let event = NatsOperatorCreatedEvent {
            operator_id: operator_keys.id,
            name: operator_keys.name,
            public_key: operator_keys.public_key,
            created_at: chrono::Utc::now(),
            created_by: cmd.requestor,
            organization_id: cmd.organization_id,
        };

        Ok(vec![KeyEvent::NatsOperatorCreated(event)])
    }

    async fn handle_create_nats_account(
        &self,
        cmd: CreateNatsAccountCommand,
        nats_port: Option<&dyn NatsKeyPort>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        let port = nats_port.ok_or_else(|| {
            KeyManagementError::InvalidCommand("NATS port not configured".to_string())
        })?;

        // Use the port to generate account keys
        let account_keys = port.generate_account(&cmd.operator_id.to_string(), &cmd.name)
            .await
            .map_err(|e| KeyManagementError::OperationFailed(format!("Failed to generate account: {}", e)))?;

        // Create event
        let event = NatsAccountCreatedEvent {
            account_id: account_keys.id,
            operator_id: cmd.operator_id,
            name: account_keys.name,
            public_key: account_keys.public_key,
            is_system: cmd.is_system,
            created_at: chrono::Utc::now(),
            created_by: cmd.requestor,
            organization_unit_id: cmd.organization_unit_id,
        };

        Ok(vec![KeyEvent::NatsAccountCreated(event)])
    }

    async fn handle_create_nats_user(
        &self,
        cmd: CreateNatsUserCommand,
        nats_port: Option<&dyn NatsKeyPort>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        let port = nats_port.ok_or_else(|| {
            KeyManagementError::InvalidCommand("NATS port not configured".to_string())
        })?;

        // Use the port to generate user keys
        let user_keys = port.generate_user(&cmd.account_id.to_string(), &cmd.name)
            .await
            .map_err(|e| KeyManagementError::OperationFailed(format!("Failed to generate user: {}", e)))?;

        // Create event
        let event = NatsUserCreatedEvent {
            user_id: user_keys.id,
            account_id: cmd.account_id,
            name: user_keys.name,
            public_key: user_keys.public_key,
            created_at: chrono::Utc::now(),
            created_by: cmd.requestor,
            person_id: cmd.person_id,
        };

        Ok(vec![KeyEvent::NatsUserCreated(event)])
    }

    async fn handle_generate_nats_signing_key(
        &self,
        cmd: crate::commands::GenerateNatsSigningKeyCommand,
        nats_port: Option<&dyn NatsKeyPort>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        let port = nats_port.ok_or_else(|| {
            KeyManagementError::InvalidCommand("NATS port not configured".to_string())
        })?;

        // Use the port to generate signing key
        let signing_key = port.generate_signing_key(&cmd.entity_id.to_string())
            .await
            .map_err(|e| KeyManagementError::OperationFailed(format!("Failed to generate signing key: {}", e)))?;

        // Map entity type
        let entity_type = match cmd.entity_type.as_str() {
            "operator" => NatsEntityType::Operator,
            "account" => NatsEntityType::Account,
            "user" => NatsEntityType::User,
            _ => return Err(KeyManagementError::InvalidCommand(format!("Invalid entity type: {}", cmd.entity_type))),
        };

        // Create event
        let event = crate::events::NatsSigningKeyGeneratedEvent {
            key_id: signing_key.id,
            entity_id: cmd.entity_id,
            entity_type,
            public_key: signing_key.public_key,
            generated_at: chrono::Utc::now(),
        };

        Ok(vec![KeyEvent::NatsSigningKeyGenerated(event)])
    }

    fn handle_set_nats_permissions(
        &self,
        cmd: crate::commands::SetNatsPermissionsCommand,
        _projection: &OfflineKeyProjection,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        // Map entity type
        let entity_type = match cmd.entity_type.as_str() {
            "operator" => NatsEntityType::Operator,
            "account" => NatsEntityType::Account,
            "user" => NatsEntityType::User,
            _ => return Err(KeyManagementError::InvalidCommand(format!("Invalid entity type: {}", cmd.entity_type))),
        };

        // Create event
        let event = crate::events::NatsPermissionsSetEvent {
            entity_id: cmd.entity_id,
            entity_type,
            permissions: EventNatsPermissions {
                publish: cmd.publish,
                subscribe: cmd.subscribe,
                allow_responses: cmd.allow_responses,
                max_payload: cmd.max_payload,
            },
            set_at: chrono::Utc::now(),
            set_by: cmd.requestor,
        };

        Ok(vec![KeyEvent::NatsPermissionsSet(event)])
    }

    async fn handle_export_nats_config(
        &self,
        cmd: crate::commands::ExportNatsConfigCommand,
        _nats_port: Option<&dyn NatsKeyPort>,
    ) -> Result<Vec<KeyEvent>, KeyManagementError> {
        // Map export format
        let format = match cmd.format.as_str() {
            "nsc" => crate::events::NatsExportFormat::NscStore,
            "server" => crate::events::NatsExportFormat::ServerConfig,
            "creds" => crate::events::NatsExportFormat::Credentials,
            _ => return Err(KeyManagementError::InvalidCommand(format!("Invalid export format: {}", cmd.format))),
        };

        // Create event
        let event = crate::events::NatsConfigExportedEvent {
            export_id: Uuid::now_v7(),
            operator_id: cmd.operator_id,
            format,
            exported_at: chrono::Utc::now(),
            exported_by: cmd.requestor,
        };

        Ok(vec![KeyEvent::NatsConfigExported(event)])
    }
}

// Remove Entity trait implementation as it's no longer needed
// The aggregate uses AggregateRoot trait instead

impl AggregateRoot for KeyManagementAggregate {
    type Id = uuid::Uuid;

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

/// Errors that can occur in key management operations
#[derive(Debug, thiserror::Error)]
pub enum KeyManagementError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Key not found: {0}")]
    KeyNotFound(Uuid),

    #[error("Certificate not found: {0}")]
    CertificateNotFound(Uuid),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Key generation error: {0}")]
    KeyGenerationError(String),

    #[cfg(feature = "policy")]
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
}