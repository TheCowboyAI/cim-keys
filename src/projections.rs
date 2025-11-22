//! Projections for key management state
//!
//! Projections write immutable JSON files to an encrypted partition.
//! This is designed for offline key management where the SD card IS the state.

use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::events::{
    KeyEvent, KeyAlgorithm, KeyPurpose, KeyMetadata,
};

// Import state machines for lifecycle tracking
use crate::state_machines::{
    KeyState, CertificateState, PersonState,
    LocationState, YubiKeyState,
};

/// Offline key storage projection
///
/// This projection writes all state as JSON files to an encrypted partition.
/// The partition structure is:
/// ```
/// /mnt/keys/
/// ├── manifest.json           # Master index of all keys
/// ├── events/                 # Event log (append-only)
/// │   └── {timestamp}_{event_id}.json
/// ├── keys/                   # Key material
/// │   └── {key_id}/
/// │       ├── metadata.json
/// │       ├── public.pem
/// │       └── private.pem (encrypted)
/// ├── certificates/           # X.509 certificates
/// │   └── {cert_id}/
/// │       ├── metadata.json
/// │       ├── cert.pem
/// │       └── chain.pem
/// ├── yubikeys/              # YubiKey configurations
/// │   └── {serial}/
/// │       └── config.json
/// └── pki/                   # PKI hierarchies
///     └── {hierarchy_name}/
///         ├── hierarchy.json
///         ├── root-ca/
///         └── intermediate-ca/
/// ```
pub struct OfflineKeyProjection {
    /// Root path to the encrypted partition
    pub root_path: PathBuf,

    /// Current manifest state
    manifest: KeyManifest,
}

/// Master manifest of all keys and certificates
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyManifest {
    /// Manifest version
    pub version: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Organization info
    pub organization: OrganizationInfo,

    /// All people in the organization
    pub people: Vec<PersonEntry>,

    /// All locations in the organization
    pub locations: Vec<LocationEntry>,

    /// All keys indexed by ID
    pub keys: Vec<KeyEntry>,

    /// All certificates indexed by ID
    pub certificates: Vec<CertificateEntry>,

    /// PKI hierarchies
    pub pki_hierarchies: Vec<PkiHierarchyEntry>,

    /// YubiKey serials
    pub yubikeys: Vec<YubiKeyEntry>,

    /// NATS operators
    pub nats_operators: Vec<NatsOperatorEntry>,

    /// NATS accounts
    pub nats_accounts: Vec<NatsAccountEntry>,

    /// NATS users
    pub nats_users: Vec<NatsUserEntry>,

    /// Event count for consistency checking
    pub event_count: u64,

    /// Checksum of all content
    pub checksum: String,
}

/// Entry for a key in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    pub key_id: Uuid,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub hardware_backed: bool,
    pub yubikey_serial: Option<String>,
    pub yubikey_slot: Option<String>,
    pub revoked: bool,
    pub file_path: String,
    /// Lifecycle state machine for this key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<KeyState>,
}

/// Entry for a certificate in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateEntry {
    pub cert_id: Uuid,
    pub key_id: Uuid,
    pub subject: String,
    pub issuer: Option<String>,
    pub serial_number: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub is_ca: bool,
    pub file_path: String,
    /// Lifecycle state machine for this certificate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<CertificateState>,
}

/// Entry for PKI hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkiHierarchyEntry {
    pub hierarchy_name: String,
    pub root_ca_id: Uuid,
    pub intermediate_ca_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub directory_path: String,
}

/// Entry for YubiKey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyEntry {
    pub serial: String,
    pub provisioned_at: DateTime<Utc>,
    pub slots_used: Vec<String>,
    pub config_path: String,
    /// Lifecycle state machine for this YubiKey
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<YubiKeyState>,
}

/// Organization information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrganizationInfo {
    pub name: String,
    pub domain: String,
    pub country: String,
    pub admin_email: String,
}

/// Entry for a person in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonEntry {
    pub person_id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub organization_id: Uuid,
    pub created_at: DateTime<Utc>,
    /// Lifecycle state machine for this person
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<PersonState>,
}

/// Entry for a location in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationEntry {
    pub location_id: Uuid,
    pub name: String,
    pub location_type: String,
    pub organization_id: Uuid,
    pub created_at: DateTime<Utc>,
    // Address details
    pub street: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    /// Lifecycle state machine for this location
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<LocationState>,
}

/// Entry for a NATS operator in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorEntry {
    pub operator_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,  // Derived from operator_id (UUID v7 timestamp)
    pub created_by: String,
}

/// Entry for a NATS account in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountEntry {
    pub account_id: Uuid,
    pub operator_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub is_system: bool,
    pub organization_unit_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,  // Derived from account_id (UUID v7 timestamp)
    pub created_by: String,
}

/// Entry for a NATS user in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserEntry {
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub person_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,  // Derived from user_id (UUID v7 timestamp)
    pub created_by: String,
}

impl OfflineKeyProjection {
    /// Create a new projection targeting an encrypted partition
    pub fn new<P: AsRef<Path>>(root_path: P) -> Result<Self, ProjectionError> {
        let root_path = root_path.as_ref().to_path_buf();

        // Ensure directory structure exists
        Self::ensure_directory_structure(&root_path)?;

        // Load or create manifest
        let manifest = Self::load_or_create_manifest(&root_path)?;

        Ok(Self {
            root_path,
            manifest,
        })
    }

    /// Ensure the directory structure exists on the partition
    fn ensure_directory_structure(root: &Path) -> Result<(), ProjectionError> {
        let dirs = [
            root.join("events"),
            root.join("keys"),
            root.join("certificates"),
            root.join("yubikeys"),
            root.join("pki"),
            root.join("nats"),
        ];

        for dir in &dirs {
            fs::create_dir_all(dir).map_err(|e| {
                ProjectionError::IoError(format!("Failed to create directory {:?}: {}", dir, e))
            })?;
        }

        Ok(())
    }

    /// Load existing manifest or create a new one
    fn load_or_create_manifest(root: &Path) -> Result<KeyManifest, ProjectionError> {
        let manifest_path = root.join("manifest.json");

        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)
                .map_err(|e| ProjectionError::IoError(format!("Failed to read manifest: {}", e)))?;

            serde_json::from_str(&content)
                .map_err(|e| ProjectionError::ParseError(format!("Invalid manifest JSON: {}", e)))
        } else {
            let manifest = KeyManifest {
                version: "1.0.0".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                organization: OrganizationInfo::default(),
                people: Vec::new(),
                locations: Vec::new(),
                keys: Vec::new(),
                certificates: Vec::new(),
                pki_hierarchies: Vec::new(),
                yubikeys: Vec::new(),
                nats_operators: Vec::new(),
                nats_accounts: Vec::new(),
                nats_users: Vec::new(),
                event_count: 0,
                checksum: String::new(),
            };

            Ok(manifest)
        }
    }

    /// Apply an event and update the projection files
    pub fn apply(&mut self, event: &KeyEvent) -> Result<(), ProjectionError> {
        // First, append event to the event log
        self.append_event(event)?;

        // Then update the specific projections
        match event {
            KeyEvent::KeyGenerated(e) => self.project_key_generated(e)?,
            KeyEvent::KeyImported(e) => self.project_key_imported(e)?,
            KeyEvent::KeyExported(e) => self.project_key_exported(e)?,
            KeyEvent::KeyStoredOffline(e) => self.project_key_stored_offline(e)?,
            KeyEvent::KeyRevoked(e) => self.project_key_revoked(e)?,
            KeyEvent::KeyRotationInitiated(e) => self.project_key_rotation_initiated(e)?,
            KeyEvent::KeyRotationCompleted(e) => self.project_key_rotation_completed(e)?,
            KeyEvent::CertificateGenerated(e) => self.project_certificate_generated(e)?,
            KeyEvent::CertificateSigned(e) => self.project_certificate_signed(e)?,
            KeyEvent::YubiKeyDetected(e) => self.project_yubikey_detected(e)?,
            KeyEvent::YubiKeyProvisioned(e) => self.project_yubikey_provisioned(e)?,
            KeyEvent::PkiHierarchyCreated(e) => self.project_pki_hierarchy_created(e)?,
            KeyEvent::PersonCreated(e) => self.project_person_created(e)?,
            KeyEvent::LocationCreated(e) => self.project_location_created(e)?,
            KeyEvent::OrganizationCreated(e) => self.project_organization_created(e)?,
            KeyEvent::NatsOperatorCreated(e) => self.project_nats_operator_created(e)?,
            KeyEvent::NatsAccountCreated(e) => self.project_nats_account_created(e)?,
            KeyEvent::NatsUserCreated(e) => self.project_nats_user_created(e)?,
            // NATS Operator State Transitions
            KeyEvent::NatsOperatorSuspended(e) => self.project_nats_operator_suspended(e)?,
            KeyEvent::NatsOperatorReactivated(e) => self.project_nats_operator_reactivated(e)?,
            KeyEvent::NatsOperatorRevoked(e) => self.project_nats_operator_revoked(e)?,
            // NATS Account State Transitions
            KeyEvent::NatsAccountActivated(e) => self.project_nats_account_activated(e)?,
            KeyEvent::NatsAccountSuspended(e) => self.project_nats_account_suspended(e)?,
            KeyEvent::NatsAccountReactivated(e) => self.project_nats_account_reactivated(e)?,
            KeyEvent::NatsAccountDeleted(e) => self.project_nats_account_deleted(e)?,
            // NATS User State Transitions
            KeyEvent::NatsUserActivated(e) => self.project_nats_user_activated(e)?,
            KeyEvent::NatsUserSuspended(e) => self.project_nats_user_suspended(e)?,
            KeyEvent::NatsUserReactivated(e) => self.project_nats_user_reactivated(e)?,
            KeyEvent::NatsUserDeleted(e) => self.project_nats_user_deleted(e)?,
            // Certificate Lifecycle State Transitions (Phase 11)
            KeyEvent::CertificateActivated(e) => self.project_certificate_activated(e)?,
            KeyEvent::CertificateSuspended(e) => self.project_certificate_suspended(e)?,
            KeyEvent::CertificateRevoked(e) => self.project_certificate_revoked(e)?,
            KeyEvent::CertificateExpired(e) => self.project_certificate_expired(e)?,
            KeyEvent::CertificateRenewed(e) => self.project_certificate_renewed(e)?,
            // Person Lifecycle State Transitions (Phase 12)
            KeyEvent::PersonActivated(e) => self.project_person_activated(e)?,
            KeyEvent::PersonSuspended(e) => self.project_person_suspended(e)?,
            KeyEvent::PersonReactivated(e) => self.project_person_reactivated(e)?,
            KeyEvent::PersonArchived(e) => self.project_person_archived(e)?,
            // Location Lifecycle State Transitions (Phase 12)
            KeyEvent::LocationActivated(e) => self.project_location_activated(e)?,
            KeyEvent::LocationSuspended(e) => self.project_location_suspended(e)?,
            KeyEvent::LocationReactivated(e) => self.project_location_reactivated(e)?,
            KeyEvent::LocationDecommissioned(e) => self.project_location_decommissioned(e)?,
            KeyEvent::NatsSigningKeyGenerated(e) => self.project_nats_signing_key_generated(e)?,
            KeyEvent::NatsPermissionsSet(e) => self.project_nats_permissions_set(e)?,
            KeyEvent::NatsConfigExported(e) => self.project_nats_config_exported(e)?,
            KeyEvent::NKeyGenerated(e) => self.project_nkey_generated(e)?,
            KeyEvent::JwtClaimsCreated(e) => self.project_jwt_claims_created(e)?,
            KeyEvent::JwtSigned(e) => self.project_jwt_signed(e)?,
            KeyEvent::ServiceAccountCreated(e) => self.project_service_account_created(e)?,
            KeyEvent::AgentCreated(e) => self.project_agent_created(e)?,
            _ => {} // Handle other events as needed
        }

        // Update manifest
        self.manifest.updated_at = Utc::now();
        self.manifest.event_count += 1;
        self.save_manifest()?;

        Ok(())
    }

    /// Check if a key exists in the projection
    pub fn key_exists(&self, key_id: &Uuid) -> bool {
        self.manifest.keys.iter().any(|k| k.key_id == *key_id)
    }

    /// Append event to the event log
    fn append_event(&self, event: &KeyEvent) -> Result<(), ProjectionError> {
        let event_id = Uuid::now_v7();
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let filename = format!("{}_{}.json", timestamp, event_id);
        let event_path = self.root_path.join("events").join(filename);

        let event_json = serde_json::to_string_pretty(event)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize event: {}", e)))?;

        fs::write(&event_path, event_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write event: {}", e)))?;

        Ok(())
    }

    /// Project a key generation event
    fn project_key_generated(&mut self, event: &crate::events::KeyGeneratedEvent) -> Result<(), ProjectionError> {
        // Create key directory
        let key_dir = self.root_path.join("keys").join(event.key_id.to_string());
        fs::create_dir_all(&key_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create key directory: {}", e)))?;

        // Write metadata
        let metadata_path = key_dir.join("metadata.json");
        let metadata = KeyMetadataFile {
            key_id: event.key_id,
            algorithm: event.algorithm.clone(),
            purpose: event.purpose.clone(),
            generated_at: event.generated_at,
            generated_by: event.generated_by.clone(),
            hardware_backed: event.hardware_backed,
            metadata: event.metadata.clone(),
        };

        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&metadata_path, metadata_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write metadata: {}", e)))?;

        // Add to manifest with initial state
        self.manifest.keys.push(KeyEntry {
            key_id: event.key_id,
            algorithm: event.algorithm.clone(),
            purpose: event.purpose.clone(),
            label: event.metadata.label.clone(),
            created_at: event.generated_at,
            hardware_backed: event.hardware_backed,
            yubikey_serial: None,
            yubikey_slot: None,
            revoked: false,
            file_path: format!("keys/{}", event.key_id),
            // Initialize state machine
            state: Some(KeyState::Generated {
                algorithm: event.algorithm.clone(),
                generated_at: event.generated_at,
                generated_by: Uuid::now_v7(), // TODO: Get from event ownership
            }),
        });

        Ok(())
    }

    /// Project a key import event (initialize with Imported state)
    fn project_key_imported(&mut self, event: &crate::events_legacy::KeyImportedEvent) -> Result<(), ProjectionError> {
        // Create key directory
        let key_dir = self.root_path.join("keys").join(event.key_id.to_string());
        fs::create_dir_all(&key_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create key directory: {}", e)))?;

        // Write import metadata
        let metadata_path = key_dir.join("metadata.json");
        let import_info = serde_json::json!({
            "key_id": event.key_id,
            "source": event.source,
            "format": event.format,
            "imported_at": event.imported_at,
            "imported_by": event.imported_by,
            "metadata": event.metadata,
        });

        let metadata_json = serde_json::to_string_pretty(&import_info)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize import metadata: {}", e)))?;

        fs::write(&metadata_path, metadata_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write import metadata: {}", e)))?;

        // Write import source marker
        let import_marker_path = key_dir.join("IMPORTED.json");
        let import_marker = serde_json::json!({
            "source": event.source,
            "format": event.format,
            "imported_at": event.imported_at,
        });

        fs::write(&import_marker_path, serde_json::to_string_pretty(&import_marker).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write import marker: {}", e)))?;

        // Add to manifest with Imported state
        self.manifest.keys.push(KeyEntry {
            key_id: event.key_id,
            algorithm: crate::events::KeyAlgorithm::Ed25519,  // TODO: Get from event.metadata
            purpose: crate::events::KeyPurpose::Signing,      // TODO: Get from event.metadata
            label: event.metadata.label.clone(),
            created_at: event.imported_at,
            hardware_backed: false,  // Imported keys are typically not hardware-backed
            yubikey_serial: None,
            yubikey_slot: None,
            revoked: false,
            file_path: format!("keys/{}", event.key_id),
            // Initialize state machine to Imported
            state: Some(KeyState::Imported {
                source: match &event.source {
                    crate::events_legacy::ImportSource::File { path } => {
                        crate::state_machines::key::ImportSource::File { path: path.clone() }
                    },
                    crate::events_legacy::ImportSource::YubiKey { serial } => {
                        crate::state_machines::key::ImportSource::YubiKey {
                            serial: serial.clone(),
                            slot: "Unknown".to_string(),  // Not available in event
                        }
                    },
                    crate::events_legacy::ImportSource::Hsm { identifier } => {
                        crate::state_machines::key::ImportSource::ExternalPKI {
                            authority: identifier.clone(),
                        }
                    },
                    crate::events_legacy::ImportSource::Memory => {
                        crate::state_machines::key::ImportSource::File {
                            path: "<memory>".to_string(),
                        }
                    },
                },
                imported_at: event.imported_at,
                imported_by: Uuid::now_v7(),  // TODO: Parse imported_by string to UUID
            }),
        });

        Ok(())
    }

    /// Project a key export event (record export operation, no state change)
    fn project_key_exported(&mut self, event: &crate::events_legacy::KeyExportedEvent) -> Result<(), ProjectionError> {
        // Find the key directory
        let key_dir = self.root_path.join("keys").join(event.key_id.to_string());

        if !key_dir.exists() {
            return Err(ProjectionError::IoError(format!(
                "Key directory not found for export: {}",
                event.key_id
            )));
        }

        // Write export metadata
        let exports_dir = key_dir.join("exports");
        fs::create_dir_all(&exports_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create exports directory: {}", e)))?;

        // Create export record with timestamp in filename
        let export_filename = format!("export_{}.json", event.exported_at.timestamp_millis());
        let export_path = exports_dir.join(export_filename);

        let export_record = serde_json::json!({
            "key_id": event.key_id,
            "format": event.format,
            "include_private": event.include_private,
            "exported_at": event.exported_at,
            "exported_by": event.exported_by,
            "destination": event.destination,
        });

        fs::write(&export_path, serde_json::to_string_pretty(&export_record).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write export record: {}", e)))?;

        // Note: We don't change the key's state - export is an operation, not a state transition
        // The key remains in its current state (Generated, Imported, Active, etc.)
        // Export records are tracked in the filesystem (exports/ directory)
        // We could add a "last_exported_at" field to KeyEntry in the future if needed

        Ok(())
    }

    /// Project a person creation event (initialize with Created state)
    fn project_person_created(&mut self, event: &crate::events_legacy::PersonCreatedEvent) -> Result<(), ProjectionError> {
        // Create person directory
        let person_dir = self.root_path.join("people").join(event.person_id.to_string());
        fs::create_dir_all(&person_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create person directory: {}", e)))?;

        // Write person metadata
        let metadata_path = person_dir.join("metadata.json");
        let person_info = serde_json::json!({
            "person_id": event.person_id,
            "name": event.name,
            "email": event.email,
            "title": event.title,
            "department": event.department,
            "organization_id": event.organization_id,
            "created_at": event.created_at,
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&person_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write person metadata: {}", e)))?;

        // Add to manifest with Created state
        self.manifest.people.push(PersonEntry {
            person_id: event.person_id,
            name: event.name.clone(),
            email: event.email.clone(),
            role: event.title.clone().unwrap_or_else(|| "Member".to_string()),
            organization_id: event.organization_id.unwrap_or_else(Uuid::now_v7),
            created_at: event.created_at,
            // Initialize state machine to Created
            state: Some(PersonState::Created {
                created_at: event.created_at,  // Derived from person_id (UUID v7 timestamp)
                created_by: Uuid::now_v7(),    // TODO: Get from event
            }),
        });

        Ok(())
    }

    /// Project a location creation event (initialize with Active state)
    fn project_location_created(&mut self, event: &crate::events_legacy::LocationCreatedEvent) -> Result<(), ProjectionError> {
        // Create location directory
        let location_dir = self.root_path.join("locations").join(event.location_id.to_string());
        fs::create_dir_all(&location_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create location directory: {}", e)))?;

        // Write location metadata
        let metadata_path = location_dir.join("metadata.json");
        let location_info = serde_json::json!({
            "location_id": event.location_id,
            "name": event.name,
            "location_type": event.location_type,
            "address": event.address,
            "coordinates": event.coordinates,
            "organization_id": event.organization_id,
            "created_at": event.created_at,
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&location_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write location metadata: {}", e)))?;

        // Add to manifest with Active state
        self.manifest.locations.push(LocationEntry {
            location_id: event.location_id,
            name: event.name.clone(),
            location_type: event.location_type.clone(),
            organization_id: event.organization_id.unwrap_or_else(Uuid::now_v7),
            created_at: event.created_at,
            street: event.address.clone(),
            city: None,
            region: None,
            country: None,
            postal_code: None,
            // Initialize state machine to Active (locations are immediately active when created)
            state: Some(LocationState::Active {
                activated_at: event.created_at,  // Derived from location_id (UUID v7 timestamp)
                access_grants: Vec::new(),
                assets_stored: 0,
                last_accessed: None,
            }),
        });

        Ok(())
    }

    /// Project an organization creation event (initialize organization info)
    fn project_organization_created(&mut self, event: &crate::events_legacy::OrganizationCreatedEvent) -> Result<(), ProjectionError> {
        // Create organization directory
        let org_dir = self.root_path.join("organization");
        fs::create_dir_all(&org_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create organization directory: {}", e)))?;

        // Write organization metadata
        let metadata_path = org_dir.join("metadata.json");
        let org_info = serde_json::json!({
            "organization_id": event.organization_id,
            "name": event.name,
            "domain": event.domain,
            "created_at": event.created_at,
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&org_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write organization metadata: {}", e)))?;

        // Update manifest organization info
        self.manifest.organization = OrganizationInfo {
            name: event.name.clone(),
            domain: event.domain.clone().unwrap_or_else(|| "example.com".to_string()),
            country: "US".to_string(),  // TODO: Add to event
            admin_email: "admin@example.com".to_string(),  // TODO: Add to event
        };

        Ok(())
    }

    /// Project a NATS operator creation event (initialize operator entry)
    fn project_nats_operator_created(&mut self, event: &crate::events_legacy::NatsOperatorCreatedEvent) -> Result<(), ProjectionError> {
        // Create NATS operator directory
        let operator_dir = self.root_path
            .join("nats")
            .join("operators")
            .join(event.operator_id.to_string());
        fs::create_dir_all(&operator_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create operator directory: {}", e)))?;

        // Write operator metadata
        let metadata_path = operator_dir.join("metadata.json");
        let operator_info = serde_json::json!({
            "operator_id": event.operator_id,
            "name": event.name,
            "public_key": event.public_key,
            "organization_id": event.organization_id,
            "created_at": event.created_at,
            "created_by": event.created_by,
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&operator_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write operator metadata: {}", e)))?;

        // Add to manifest
        self.manifest.nats_operators.push(NatsOperatorEntry {
            operator_id: event.operator_id,
            name: event.name.clone(),
            public_key: event.public_key.clone(),
            organization_id: event.organization_id,
            created_at: event.created_at,  // Derived from operator_id (UUID v7 timestamp)
            created_by: event.created_by.clone(),
        });

        Ok(())
    }

    /// Project a NATS account creation event (initialize account entry)
    fn project_nats_account_created(&mut self, event: &crate::events_legacy::NatsAccountCreatedEvent) -> Result<(), ProjectionError> {
        // Create NATS account directory
        let account_dir = self.root_path
            .join("nats")
            .join("accounts")
            .join(event.account_id.to_string());
        fs::create_dir_all(&account_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create account directory: {}", e)))?;

        // Write account metadata
        let metadata_path = account_dir.join("metadata.json");
        let account_info = serde_json::json!({
            "account_id": event.account_id,
            "operator_id": event.operator_id,
            "name": event.name,
            "public_key": event.public_key,
            "is_system": event.is_system,
            "organization_unit_id": event.organization_unit_id,
            "created_at": event.created_at,
            "created_by": event.created_by,
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&account_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write account metadata: {}", e)))?;

        // Add to manifest
        self.manifest.nats_accounts.push(NatsAccountEntry {
            account_id: event.account_id,
            operator_id: event.operator_id,
            name: event.name.clone(),
            public_key: event.public_key.clone(),
            is_system: event.is_system,
            organization_unit_id: event.organization_unit_id,
            created_at: event.created_at,  // Derived from account_id (UUID v7 timestamp)
            created_by: event.created_by.clone(),
        });

        Ok(())
    }

    /// Project a NATS user creation event (initialize user entry)
    fn project_nats_user_created(&mut self, event: &crate::events_legacy::NatsUserCreatedEvent) -> Result<(), ProjectionError> {
        // Create NATS user directory
        let user_dir = self.root_path
            .join("nats")
            .join("users")
            .join(event.user_id.to_string());
        fs::create_dir_all(&user_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create user directory: {}", e)))?;

        // Write user metadata
        let metadata_path = user_dir.join("metadata.json");
        let user_info = serde_json::json!({
            "user_id": event.user_id,
            "account_id": event.account_id,
            "name": event.name,
            "public_key": event.public_key,
            "person_id": event.person_id,
            "created_at": event.created_at,
            "created_by": event.created_by,
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&user_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write user metadata: {}", e)))?;

        // Add to manifest
        self.manifest.nats_users.push(NatsUserEntry {
            user_id: event.user_id,
            account_id: event.account_id,
            name: event.name.clone(),
            public_key: event.public_key.clone(),
            person_id: event.person_id,
            created_at: event.created_at,  // Derived from user_id (UUID v7 timestamp)
            created_by: event.created_by.clone(),
        });

        Ok(())
    }

    // ========================================================================
    // NATS Operator State Transition Handlers (Phase 10)
    // ========================================================================

    /// Project NATS operator suspended event
    fn project_nats_operator_suspended(&mut self, event: &crate::events_legacy::NatsOperatorSuspendedEvent) -> Result<(), ProjectionError> {
        let operator_dir = self.root_path
            .join("nats")
            .join("operators")
            .join(event.operator_id.to_string());

        // Update state in metadata
        let state_path = operator_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Suspended",
            "reason": event.reason,
            "suspended_at": event.suspended_at,
            "suspended_by": event.suspended_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project NATS operator reactivated event
    fn project_nats_operator_reactivated(&mut self, event: &crate::events_legacy::NatsOperatorReactivatedEvent) -> Result<(), ProjectionError> {
        let operator_dir = self.root_path
            .join("nats")
            .join("operators")
            .join(event.operator_id.to_string());

        let state_path = operator_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Active",
            "reactivated_at": event.reactivated_at,
            "reactivated_by": event.reactivated_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project NATS operator revoked event (terminal state)
    fn project_nats_operator_revoked(&mut self, event: &crate::events_legacy::NatsOperatorRevokedEvent) -> Result<(), ProjectionError> {
        let operator_dir = self.root_path
            .join("nats")
            .join("operators")
            .join(event.operator_id.to_string());

        let state_path = operator_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Revoked",
            "reason": event.reason,
            "revoked_at": event.revoked_at,
            "revoked_by": event.revoked_by,
            "correlation_id": event.correlation_id,
            "terminal": true,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    // ========================================================================
    // NATS Account State Transition Handlers (Phase 10)
    // ========================================================================

    /// Project NATS account activated event
    fn project_nats_account_activated(&mut self, event: &crate::events_legacy::NatsAccountActivatedEvent) -> Result<(), ProjectionError> {
        let account_dir = self.root_path
            .join("nats")
            .join("accounts")
            .join(event.account_id.to_string());

        let state_path = account_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Active",
            "permissions": event.permissions,
            "activated_at": event.activated_at,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project NATS account suspended event
    fn project_nats_account_suspended(&mut self, event: &crate::events_legacy::NatsAccountSuspendedEvent) -> Result<(), ProjectionError> {
        let account_dir = self.root_path
            .join("nats")
            .join("accounts")
            .join(event.account_id.to_string());

        let state_path = account_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Suspended",
            "reason": event.reason,
            "suspended_at": event.suspended_at,
            "suspended_by": event.suspended_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project NATS account reactivated event
    fn project_nats_account_reactivated(&mut self, event: &crate::events_legacy::NatsAccountReactivatedEvent) -> Result<(), ProjectionError> {
        let account_dir = self.root_path
            .join("nats")
            .join("accounts")
            .join(event.account_id.to_string());

        let state_path = account_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Reactivated",
            "permissions": event.permissions,
            "reactivated_at": event.reactivated_at,
            "reactivated_by": event.reactivated_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project NATS account deleted event (terminal state)
    fn project_nats_account_deleted(&mut self, event: &crate::events_legacy::NatsAccountDeletedEvent) -> Result<(), ProjectionError> {
        let account_dir = self.root_path
            .join("nats")
            .join("accounts")
            .join(event.account_id.to_string());

        let state_path = account_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Deleted",
            "reason": event.reason,
            "deleted_at": event.deleted_at,
            "deleted_by": event.deleted_by,
            "correlation_id": event.correlation_id,
            "terminal": true,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    // ========================================================================
    // NATS User State Transition Handlers (Phase 10)
    // ========================================================================

    /// Project NATS user activated event
    fn project_nats_user_activated(&mut self, event: &crate::events_legacy::NatsUserActivatedEvent) -> Result<(), ProjectionError> {
        let user_dir = self.root_path
            .join("nats")
            .join("users")
            .join(event.user_id.to_string());

        let state_path = user_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Active",
            "permissions": event.permissions,
            "activated_at": event.activated_at,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project NATS user suspended event
    fn project_nats_user_suspended(&mut self, event: &crate::events_legacy::NatsUserSuspendedEvent) -> Result<(), ProjectionError> {
        let user_dir = self.root_path
            .join("nats")
            .join("users")
            .join(event.user_id.to_string());

        let state_path = user_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Suspended",
            "reason": event.reason,
            "suspended_at": event.suspended_at,
            "suspended_by": event.suspended_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project NATS user reactivated event
    fn project_nats_user_reactivated(&mut self, event: &crate::events_legacy::NatsUserReactivatedEvent) -> Result<(), ProjectionError> {
        let user_dir = self.root_path
            .join("nats")
            .join("users")
            .join(event.user_id.to_string());

        let state_path = user_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Reactivated",
            "permissions": event.permissions,
            "reactivated_at": event.reactivated_at,
            "reactivated_by": event.reactivated_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project NATS user deleted event (terminal state)
    fn project_nats_user_deleted(&mut self, event: &crate::events_legacy::NatsUserDeletedEvent) -> Result<(), ProjectionError> {
        let user_dir = self.root_path
            .join("nats")
            .join("users")
            .join(event.user_id.to_string());

        let state_path = user_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Deleted",
            "reason": event.reason,
            "deleted_at": event.deleted_at,
            "deleted_by": event.deleted_by,
            "correlation_id": event.correlation_id,
            "terminal": true,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project a NATS signing key generation event (operational)
    fn project_nats_signing_key_generated(&mut self, event: &crate::events_legacy::NatsSigningKeyGeneratedEvent) -> Result<(), ProjectionError> {
        // Determine entity directory based on type
        let entity_dir = match event.entity_type {
            crate::events_legacy::NatsEntityType::Operator => {
                self.root_path.join("nats").join("operators").join(event.entity_id.to_string())
            }
            crate::events_legacy::NatsEntityType::Account => {
                self.root_path.join("nats").join("accounts").join(event.entity_id.to_string())
            }
            crate::events_legacy::NatsEntityType::User => {
                self.root_path.join("nats").join("users").join(event.entity_id.to_string())
            }
        };

        // Create signing_keys subdirectory
        let signing_keys_dir = entity_dir.join("signing_keys");
        fs::create_dir_all(&signing_keys_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create signing keys directory: {}", e)))?;

        // Write signing key metadata
        let key_file = signing_keys_dir.join(format!("{}.json", event.key_id));
        let key_info = serde_json::json!({
            "key_id": event.key_id,
            "public_key": event.public_key,
            "generated_at": event.generated_at,  // Actual generation time (operation timestamp)
        });

        fs::write(&key_file, serde_json::to_string_pretty(&key_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write signing key metadata: {}", e)))?;

        Ok(())
    }

    /// Project a NATS permissions set event (operational)
    fn project_nats_permissions_set(&mut self, event: &crate::events_legacy::NatsPermissionsSetEvent) -> Result<(), ProjectionError> {
        // Determine entity directory based on type
        let entity_dir = match event.entity_type {
            crate::events_legacy::NatsEntityType::Operator => {
                self.root_path.join("nats").join("operators").join(event.entity_id.to_string())
            }
            crate::events_legacy::NatsEntityType::Account => {
                self.root_path.join("nats").join("accounts").join(event.entity_id.to_string())
            }
            crate::events_legacy::NatsEntityType::User => {
                self.root_path.join("nats").join("users").join(event.entity_id.to_string())
            }
        };

        // Write permissions file (overwrite previous)
        let permissions_file = entity_dir.join("permissions.json");
        let permissions_info = serde_json::json!({
            "publish": event.permissions.publish,
            "subscribe": event.permissions.subscribe,
            "allow_responses": event.permissions.allow_responses,
            "max_payload": event.permissions.max_payload,
            "set_at": event.set_at,  // Actual set time (operation timestamp)
            "set_by": event.set_by,
        });

        fs::write(&permissions_file, serde_json::to_string_pretty(&permissions_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write permissions file: {}", e)))?;

        Ok(())
    }

    /// Project a NATS config export event (operational)
    fn project_nats_config_exported(&mut self, event: &crate::events_legacy::NatsConfigExportedEvent) -> Result<(), ProjectionError> {
        // Create exports directory under operator
        let exports_dir = self.root_path
            .join("nats")
            .join("operators")
            .join(event.operator_id.to_string())
            .join("exports");
        fs::create_dir_all(&exports_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create exports directory: {}", e)))?;

        // Write export record
        let export_file = exports_dir.join(format!("export_{}.json", event.export_id));
        let export_info = serde_json::json!({
            "export_id": event.export_id,
            "format": event.format,
            "exported_at": event.exported_at,  // Actual export time (operation timestamp)
            "exported_by": event.exported_by,
        });

        fs::write(&export_file, serde_json::to_string_pretty(&export_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write export record: {}", e)))?;

        Ok(())
    }

    /// Project an NKey generation event (operational)
    fn project_nkey_generated(&mut self, event: &crate::events_legacy::NKeyGeneratedEvent) -> Result<(), ProjectionError> {
        // Create nkeys directory
        let nkeys_dir = self.root_path.join("nats").join("nkeys");
        fs::create_dir_all(&nkeys_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create nkeys directory: {}", e)))?;

        // Write NKey metadata (public key only, seed is sensitive)
        let nkey_file = nkeys_dir.join(format!("{}.json", event.nkey_id));
        let nkey_info = serde_json::json!({
            "nkey_id": event.nkey_id,
            "key_type": event.key_type,
            "public_key": event.public_key,
            "purpose": event.purpose,
            "expires_at": event.expires_at,
            "generated_at": event.generated_at,  // Derived from nkey_id (UUID v7 timestamp)
            "correlation_id": event.correlation_id,
            "causation_id": event.causation_id,
        });

        fs::write(&nkey_file, serde_json::to_string_pretty(&nkey_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write NKey metadata: {}", e)))?;

        Ok(())
    }

    /// Project a JWT claims creation event (operational)
    fn project_jwt_claims_created(&mut self, event: &crate::events_legacy::JwtClaimsCreatedEvent) -> Result<(), ProjectionError> {
        // Create jwt_claims directory
        let claims_dir = self.root_path.join("nats").join("jwt_claims");
        fs::create_dir_all(&claims_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create jwt_claims directory: {}", e)))?;

        // Write JWT claims metadata
        let claims_file = claims_dir.join(format!("{}.json", event.claims_id));
        let claims_info = serde_json::json!({
            "claims_id": event.claims_id,
            "issuer": event.issuer,
            "subject": event.subject,
            "audience": event.audience,
            "permissions": event.permissions,
            "not_before": event.not_before,
            "expires_at": event.expires_at,
            "created_at": event.created_at,  // Derived from claims_id (UUID v7 timestamp)
            "correlation_id": event.correlation_id,
            "causation_id": event.causation_id,
        });

        fs::write(&claims_file, serde_json::to_string_pretty(&claims_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write JWT claims: {}", e)))?;

        Ok(())
    }

    /// Project a JWT signed event (operational)
    fn project_jwt_signed(&mut self, event: &crate::events_legacy::JwtSignedEvent) -> Result<(), ProjectionError> {
        // Create jwt_tokens directory
        let tokens_dir = self.root_path.join("nats").join("jwt_tokens");
        fs::create_dir_all(&tokens_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create jwt_tokens directory: {}", e)))?;

        // Write JWT token metadata (NOT the actual token - that's sensitive)
        let token_file = tokens_dir.join(format!("{}.json", event.jwt_id));
        let token_info = serde_json::json!({
            "jwt_id": event.jwt_id,
            "signer_public_key": event.signer_public_key,
            "signature_algorithm": event.signature_algorithm,
            "signature_verification_data": event.signature_verification_data,
            "signed_at": event.signed_at,  // Derived from jwt_id (UUID v7 timestamp)
            "correlation_id": event.correlation_id,
            "causation_id": event.causation_id,
        });

        fs::write(&token_file, serde_json::to_string_pretty(&token_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write JWT token metadata: {}", e)))?;

        Ok(())
    }

    /// Project a service account creation event (initialize service account entry)
    fn project_service_account_created(&mut self, event: &crate::events_legacy::ServiceAccountCreatedEvent) -> Result<(), ProjectionError> {
        // Create service account directory under NATS users
        let sa_dir = self.root_path
            .join("nats")
            .join("service_accounts")
            .join(event.service_account_id.to_string());
        fs::create_dir_all(&sa_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create service account directory: {}", e)))?;

        // Write service account metadata
        let metadata_path = sa_dir.join("metadata.json");
        let sa_info = serde_json::json!({
            "service_account_id": event.service_account_id,
            "name": event.name,
            "purpose": event.purpose,
            "owning_unit_id": event.owning_unit_id,
            "responsible_person_id": event.responsible_person_id,
            "created_at": event.created_at,  // Derived from service_account_id (UUID v7 timestamp)
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&sa_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write service account metadata: {}", e)))?;

        Ok(())
    }

    /// Project an agent creation event (initialize agent entry)
    fn project_agent_created(&mut self, event: &crate::events_legacy::AgentCreatedEvent) -> Result<(), ProjectionError> {
        // Create agent directory under NATS
        let agent_dir = self.root_path
            .join("nats")
            .join("agents")
            .join(event.agent_id.to_string());
        fs::create_dir_all(&agent_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create agent directory: {}", e)))?;

        // Write agent metadata
        let metadata_path = agent_dir.join("metadata.json");
        let agent_info = serde_json::json!({
            "agent_id": event.agent_id,
            "name": event.name,
            "agent_type": event.agent_type,
            "responsible_person_id": event.responsible_person_id,
            "organization_id": event.organization_id,
            "created_at": event.created_at,  // Derived from agent_id (UUID v7 timestamp)
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&agent_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write agent metadata: {}", e)))?;

        Ok(())
    }

    /// Project a key rotation initiated event (Active → RotationPending)
    fn project_key_rotation_initiated(&mut self, event: &crate::events_legacy::KeyRotationInitiatedEvent) -> Result<(), ProjectionError> {
        // Find the old key entry and transition to RotationPending
        if let Some(key_entry) = self.manifest.keys.iter_mut().find(|k| k.key_id == event.old_key_id) {
            if let Some(current_state) = &key_entry.state {
                // Validate transition from Active state
                if !matches!(current_state, KeyState::Active { .. }) {
                    return Err(ProjectionError::InvalidStateTransition(format!(
                        "Cannot initiate rotation from state: {}",
                        current_state.description()
                    )));
                }

                // Transition to RotationPending
                key_entry.state = Some(KeyState::RotationPending {
                    new_key_id: event.new_key_id,
                    initiated_at: event.initiated_at,
                    initiated_by: Uuid::now_v7(),  // TODO: Parse from event.initiated_by
                });

                // Write rotation marker
                let key_dir = self.root_path.join("keys").join(event.old_key_id.to_string());
                let rotation_path = key_dir.join("ROTATION_PENDING.json");
                let rotation_info = serde_json::json!({
                    "rotation_id": event.rotation_id,
                    "old_key_id": event.old_key_id,
                    "new_key_id": event.new_key_id,
                    "initiated_at": event.initiated_at,
                    "reason": event.rotation_reason,
                });

                fs::write(&rotation_path, serde_json::to_string_pretty(&rotation_info).unwrap())
                    .map_err(|e| ProjectionError::IoError(format!("Failed to write rotation marker: {}", e)))?;
            }
        }

        Ok(())
    }

    /// Project a key rotation completed event (RotationPending → Rotated)
    fn project_key_rotation_completed(&mut self, event: &crate::events_legacy::KeyRotationCompletedEvent) -> Result<(), ProjectionError> {
        // Find the old key entry and transition to Rotated
        if let Some(key_entry) = self.manifest.keys.iter_mut().find(|k| k.key_id == event.old_key_id) {
            if let Some(current_state) = &key_entry.state {
                // Validate transition from RotationPending state
                if !matches!(current_state, KeyState::RotationPending { .. }) {
                    return Err(ProjectionError::InvalidStateTransition(format!(
                        "Cannot complete rotation from state: {}",
                        current_state.description()
                    )));
                }

                // Transition to Rotated
                key_entry.state = Some(KeyState::Rotated {
                    new_key_id: event.new_key_id,
                    rotated_at: event.completed_at,
                    rotated_by: Uuid::now_v7(),  // TODO: Get from event
                });

                // Update rotation marker
                let key_dir = self.root_path.join("keys").join(event.old_key_id.to_string());
                let rotation_path = key_dir.join("ROTATED.json");
                let rotation_info = serde_json::json!({
                    "rotation_id": event.rotation_id,
                    "old_key_id": event.old_key_id,
                    "new_key_id": event.new_key_id,
                    "completed_at": event.completed_at,
                });

                fs::write(&rotation_path, serde_json::to_string_pretty(&rotation_info).unwrap())
                    .map_err(|e| ProjectionError::IoError(format!("Failed to write rotation completion: {}", e)))?;

                // Remove pending marker if it exists
                let pending_path = key_dir.join("ROTATION_PENDING.json");
                let _ = fs::remove_file(&pending_path);  // Ignore error if file doesn't exist
            }
        }

        Ok(())
    }

    /// Project a certificate generation event
    fn project_certificate_generated(&mut self, event: &crate::events::CertificateGeneratedEvent) -> Result<(), ProjectionError> {
        // Create certificate directory
        let cert_dir = self.root_path.join("certificates").join(event.cert_id.to_string());
        fs::create_dir_all(&cert_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create certificate directory: {}", e)))?;

        // Write metadata
        let metadata_path = cert_dir.join("metadata.json");
        let metadata = CertificateMetadataFile {
            cert_id: event.cert_id,
            key_id: event.key_id,
            subject: event.subject.clone(),
            issuer: event.issuer.map(|id| id.to_string()),
            serial_number: Uuid::now_v7().to_string(), // Generate serial
            not_before: event.not_before,
            not_after: event.not_after,
            is_ca: event.is_ca,
            san: event.san.clone(),
            key_usage: event.key_usage.clone(),
            extended_key_usage: event.extended_key_usage.clone(),
        };

        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&metadata_path, metadata_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write metadata: {}", e)))?;

        // Add to manifest with initial state
        self.manifest.certificates.push(CertificateEntry {
            cert_id: event.cert_id,
            key_id: event.key_id,
            subject: event.subject.clone(),
            issuer: event.issuer.map(|id| id.to_string()),
            serial_number: metadata.serial_number.clone(),
            not_before: event.not_before,
            not_after: event.not_after,
            is_ca: event.is_ca,
            file_path: format!("certificates/{}", event.cert_id),
            // Initialize state machine
            state: Some(CertificateState::Pending {
                csr_id: None,
                pending_since: event.not_before,
                requested_by: Uuid::now_v7(), // TODO: Get from event
            }),
        });

        Ok(())
    }

    /// Project certificate signing (transition from Pending to Active)
    fn project_certificate_signed(&mut self, event: &crate::events::CertificateSignedEvent) -> Result<(), ProjectionError> {
        // Find the certificate and transition its state
        if let Some(cert) = self.manifest.certificates.iter_mut().find(|c| c.cert_id == event.cert_id) {
            // Transition state from Pending to Active
            if let Some(current_state) = &cert.state {
                match current_state {
                    CertificateState::Pending { .. } => {
                        // Certificate is now signed and active
                        // Use not_before and not_after from the cert entry
                        cert.state = Some(CertificateState::Active {
                            not_before: cert.not_before,
                            not_after: cert.not_after,
                            usage_count: 0,
                            last_used: None,
                        });
                    }
                    // Already active or in other states - no transition
                    _ => {}
                }
            } else {
                // No state exists, initialize to Active
                cert.state = Some(CertificateState::Active {
                    not_before: cert.not_before,
                    not_after: cert.not_after,
                    usage_count: 0,
                    last_used: None,
                });
            }

            // Write signing info to certificate directory
            let cert_dir = self.root_path.join("certificates").join(event.cert_id.to_string());
            let signature_path = cert_dir.join("SIGNATURE.json");

            let signature_json = serde_json::to_string_pretty(&event)
                .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize signature: {}", e)))?;

            fs::write(&signature_path, signature_json)
                .map_err(|e| ProjectionError::IoError(format!("Failed to write signature: {}", e)))?;
        }

        Ok(())
    }

    // ========================================================================
    // Certificate Lifecycle State Transition Handlers (Phase 11)
    // ========================================================================

    /// Project certificate activated event
    fn project_certificate_activated(&mut self, event: &crate::events_legacy::CertificateActivatedEvent) -> Result<(), ProjectionError> {
        let cert_dir = self.root_path
            .join("certificates")
            .join(event.cert_id.to_string());

        let state_path = cert_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Active",
            "activated_at": event.activated_at,
            "activated_by": event.activated_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project certificate suspended event
    fn project_certificate_suspended(&mut self, event: &crate::events_legacy::CertificateSuspendedEvent) -> Result<(), ProjectionError> {
        let cert_dir = self.root_path
            .join("certificates")
            .join(event.cert_id.to_string());

        let state_path = cert_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Suspended",
            "reason": event.reason,
            "suspended_at": event.suspended_at,
            "suspended_by": event.suspended_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project certificate revoked event (terminal state)
    fn project_certificate_revoked(&mut self, event: &crate::events_legacy::CertificateRevokedEvent) -> Result<(), ProjectionError> {
        let cert_dir = self.root_path
            .join("certificates")
            .join(event.cert_id.to_string());

        let state_path = cert_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Revoked",
            "reason": event.reason,
            "revoked_at": event.revoked_at,
            "revoked_by": event.revoked_by,
            "correlation_id": event.correlation_id,
            "terminal": true,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project certificate expired event (terminal state)
    fn project_certificate_expired(&mut self, event: &crate::events_legacy::CertificateExpiredEvent) -> Result<(), ProjectionError> {
        let cert_dir = self.root_path
            .join("certificates")
            .join(event.cert_id.to_string());

        let state_path = cert_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Expired",
            "expired_at": event.expired_at,
            "not_after": event.not_after,
            "correlation_id": event.correlation_id,
            "terminal": true,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write state file: {}", e)))?;

        Ok(())
    }

    /// Project certificate renewed event
    fn project_certificate_renewed(&mut self, event: &crate::events_legacy::CertificateRenewedEvent) -> Result<(), ProjectionError> {
        // Update old certificate to show it's been renewed
        let old_cert_dir = self.root_path
            .join("certificates")
            .join(event.old_cert_id.to_string());

        let old_state_path = old_cert_dir.join("state.json");
        let old_state_info = serde_json::json!({
            "state": "Renewed",
            "renewed_at": event.renewed_at,
            "renewed_by": event.renewed_by,
            "new_cert_id": event.new_cert_id,
            "correlation_id": event.correlation_id,
        });
        fs::write(&old_state_path, serde_json::to_string_pretty(&old_state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write old cert state: {}", e)))?;

        // Mark new certificate as active (renewal creates new cert)
        let new_cert_dir = self.root_path
            .join("certificates")
            .join(event.new_cert_id.to_string());

        fs::create_dir_all(&new_cert_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create new cert directory: {}", e)))?;

        let new_state_path = new_cert_dir.join("state.json");
        let new_state_info = serde_json::json!({
            "state": "Active",
            "renewed_from": event.old_cert_id,
            "renewed_at": event.renewed_at,
            "new_not_after": event.new_not_after,
            "correlation_id": event.correlation_id,
        });
        fs::write(&new_state_path, serde_json::to_string_pretty(&new_state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write new cert state: {}", e)))?;

        Ok(())
    }

    // ========================================================================
    // Person Lifecycle State Transition Handlers (Phase 12)
    // ========================================================================

    fn project_person_activated(&mut self, event: &crate::events_legacy::PersonActivatedEvent) -> Result<(), ProjectionError> {
        let person_dir = self.root_path
            .join("people")
            .join(event.person_id.to_string());

        let state_path = person_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Active",
            "activated_at": event.activated_at,
            "activated_by": event.activated_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write person state: {}", e)))?;
        Ok(())
    }

    fn project_person_suspended(&mut self, event: &crate::events_legacy::PersonSuspendedEvent) -> Result<(), ProjectionError> {
        let person_dir = self.root_path
            .join("people")
            .join(event.person_id.to_string());

        let state_path = person_dir.join("state.json");
        let mut state_info = serde_json::json!({
            "state": "Suspended",
            "reason": event.reason,
            "suspended_at": event.suspended_at,
            "suspended_by": event.suspended_by,
            "correlation_id": event.correlation_id,
        });
        if let Some(expected_return) = event.expected_return {
            state_info["expected_return"] = serde_json::json!(expected_return);
        }
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write person state: {}", e)))?;
        Ok(())
    }

    fn project_person_reactivated(&mut self, event: &crate::events_legacy::PersonReactivatedEvent) -> Result<(), ProjectionError> {
        let person_dir = self.root_path
            .join("people")
            .join(event.person_id.to_string());

        let state_path = person_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Active",
            "reactivated_at": event.reactivated_at,
            "reactivated_by": event.reactivated_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write person state: {}", e)))?;
        Ok(())
    }

    fn project_person_archived(&mut self, event: &crate::events_legacy::PersonArchivedEvent) -> Result<(), ProjectionError> {
        let person_dir = self.root_path
            .join("people")
            .join(event.person_id.to_string());

        let state_path = person_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Archived",
            "reason": event.reason,
            "archived_at": event.archived_at,
            "archived_by": event.archived_by,
            "correlation_id": event.correlation_id,
            "terminal": true,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write person state: {}", e)))?;
        Ok(())
    }

    // ========================================================================
    // Location Lifecycle State Transition Handlers (Phase 12)
    // ========================================================================

    fn project_location_activated(&mut self, event: &crate::events_legacy::LocationActivatedEvent) -> Result<(), ProjectionError> {
        let location_dir = self.root_path
            .join("locations")
            .join(event.location_id.to_string());

        let state_path = location_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Active",
            "activated_at": event.activated_at,
            "activated_by": event.activated_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write location state: {}", e)))?;
        Ok(())
    }

    fn project_location_suspended(&mut self, event: &crate::events_legacy::LocationSuspendedEvent) -> Result<(), ProjectionError> {
        let location_dir = self.root_path
            .join("locations")
            .join(event.location_id.to_string());

        let state_path = location_dir.join("state.json");
        let mut state_info = serde_json::json!({
            "state": "Suspended",
            "reason": event.reason,
            "suspended_at": event.suspended_at,
            "suspended_by": event.suspended_by,
            "correlation_id": event.correlation_id,
        });
        if let Some(expected_restoration) = event.expected_restoration {
            state_info["expected_restoration"] = serde_json::json!(expected_restoration);
        }
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write location state: {}", e)))?;
        Ok(())
    }

    fn project_location_reactivated(&mut self, event: &crate::events_legacy::LocationReactivatedEvent) -> Result<(), ProjectionError> {
        let location_dir = self.root_path
            .join("locations")
            .join(event.location_id.to_string());

        let state_path = location_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Active",
            "reactivated_at": event.reactivated_at,
            "reactivated_by": event.reactivated_by,
            "correlation_id": event.correlation_id,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write location state: {}", e)))?;
        Ok(())
    }

    fn project_location_decommissioned(&mut self, event: &crate::events_legacy::LocationDecommissionedEvent) -> Result<(), ProjectionError> {
        let location_dir = self.root_path
            .join("locations")
            .join(event.location_id.to_string());

        let state_path = location_dir.join("state.json");
        let state_info = serde_json::json!({
            "state": "Decommissioned",
            "reason": event.reason,
            "decommissioned_at": event.decommissioned_at,
            "decommissioned_by": event.decommissioned_by,
            "correlation_id": event.correlation_id,
            "terminal": true,
        });
        fs::write(&state_path, serde_json::to_string_pretty(&state_info).unwrap())
            .map_err(|e| ProjectionError::IoError(format!("Failed to write location state: {}", e)))?;
        Ok(())
    }

    /// Project YubiKey detection (initialize with Detected state)
    fn project_yubikey_detected(&mut self, event: &crate::events::YubiKeyDetectedEvent) -> Result<(), ProjectionError> {
        // Check if YubiKey already exists in manifest
        if self.manifest.yubikeys.iter().any(|y| y.serial == event.yubikey_serial) {
            // Already exists, don't create duplicate
            return Ok(());
        }

        // Create YubiKey directory
        let yubikey_dir = self.root_path.join("yubikeys").join(&event.yubikey_serial);
        fs::create_dir_all(&yubikey_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create YubiKey directory: {}", e)))?;

        // Write detection info
        let detection_path = yubikey_dir.join("DETECTED.json");
        let detection_json = serde_json::to_string_pretty(&event)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize detection: {}", e)))?;

        fs::write(&detection_path, detection_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write detection: {}", e)))?;

        // Add to manifest with Detected state
        self.manifest.yubikeys.push(YubiKeyEntry {
            serial: event.yubikey_serial.clone(),
            provisioned_at: event.detected_at, // Use detected_at as placeholder
            slots_used: Vec::new(), // No slots configured yet
            config_path: format!("yubikeys/{}/DETECTED.json", event.yubikey_serial),
            // Initialize state machine to Detected
            state: Some(YubiKeyState::Detected {
                serial: event.yubikey_serial.clone(),
                firmware: event.firmware_version.clone(),
                detected_at: event.detected_at,
                detected_by: Uuid::now_v7(), // TODO: Get from event
            }),
        });

        Ok(())
    }

    /// Project YubiKey provisioning (transition from Detected to Provisioned)
    fn project_yubikey_provisioned(&mut self, event: &crate::events::YubiKeyProvisionedEvent) -> Result<(), ProjectionError> {
        let yubikey_dir = self.root_path.join("yubikeys").join(&event.yubikey_serial);
        fs::create_dir_all(&yubikey_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create YubiKey directory: {}", e)))?;

        let config_path = yubikey_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&event)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, config_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write config: {}", e)))?;

        // Check if YubiKey entry already exists (from YubiKeyDetected)
        if let Some(yubikey) = self.manifest.yubikeys.iter_mut().find(|y| y.serial == event.yubikey_serial) {
            // Update existing entry and transition state
            yubikey.provisioned_at = event.provisioned_at;
            yubikey.slots_used = event.slots_configured.iter().map(|s| s.slot_id.clone()).collect();
            yubikey.config_path = format!("yubikeys/{}/config.json", event.yubikey_serial);

            // Transition state from Detected to Provisioned
            if let Some(current_state) = &yubikey.state {
                match current_state {
                    YubiKeyState::Detected { .. } => {
                        yubikey.state = Some(YubiKeyState::Provisioned {
                            provisioned_at: event.provisioned_at,
                            provisioned_by: Uuid::now_v7(), // TODO: Get from event
                            slots: std::collections::HashMap::new(), // TODO: Map from event.slots_configured
                            pin_changed: true, // Assume changed during provisioning
                            puk_changed: true, // Assume changed during provisioning
                        });
                    }
                    // Already provisioned or in other states - update fields but don't transition
                    _ => {}
                }
            }
        } else {
            // No existing entry (backward compatibility) - create new entry
            self.manifest.yubikeys.push(YubiKeyEntry {
                serial: event.yubikey_serial.clone(),
                provisioned_at: event.provisioned_at,
                slots_used: event.slots_configured.iter().map(|s| s.slot_id.clone()).collect(),
                config_path: format!("yubikeys/{}/config.json", event.yubikey_serial),
                // Initialize state machine to Provisioned
                state: Some(YubiKeyState::Provisioned {
                    provisioned_at: event.provisioned_at,
                    provisioned_by: Uuid::now_v7(), // TODO: Get from event
                    slots: std::collections::HashMap::new(), // TODO: Map from event.slots_configured
                    pin_changed: true, // Assume changed during provisioning
                    puk_changed: true, // Assume changed during provisioning
                }),
            });
        }

        Ok(())
    }

    /// Project PKI hierarchy creation
    fn project_pki_hierarchy_created(&mut self, event: &crate::events::PkiHierarchyCreatedEvent) -> Result<(), ProjectionError> {
        let pki_dir = self.root_path.join("pki").join(&event.hierarchy_name);
        fs::create_dir_all(&pki_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create PKI directory: {}", e)))?;

        let hierarchy_path = pki_dir.join("hierarchy.json");
        let hierarchy_json = serde_json::to_string_pretty(&event)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize hierarchy: {}", e)))?;

        fs::write(&hierarchy_path, hierarchy_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write hierarchy: {}", e)))?;

        self.manifest.pki_hierarchies.push(PkiHierarchyEntry {
            hierarchy_name: event.hierarchy_name.clone(),
            root_ca_id: event.root_ca_id,
            intermediate_ca_ids: event.intermediate_ca_ids.clone(),
            created_at: event.created_at,
            directory_path: format!("pki/{}", event.hierarchy_name),
        });

        Ok(())
    }

    /// Project key stored offline
    fn project_key_stored_offline(&mut self, event: &crate::events::KeyStoredOfflineEvent) -> Result<(), ProjectionError> {
        // Update the key entry to note it's stored offline
        if let Some(key) = self.manifest.keys.iter_mut().find(|k| k.key_id == event.key_id) {
            // Update file_path to indicate offline storage location
            key.file_path = format!("keys/{}/offline", event.key_id);

            // Transition state from Generated/Imported to Active
            if let Some(current_state) = &key.state {
                // Key is now stored and ready for use - transition to Active
                match current_state {
                    KeyState::Generated { .. } | KeyState::Imported { .. } => {
                        key.state = Some(KeyState::Active {
                            activated_at: event.stored_at,
                            usage_count: 0,
                            last_used: None,
                        });
                    }
                    // Already active or in other states - no transition needed
                    _ => {}
                }
            }

            // Log the offline storage event
            let key_dir = self.root_path.join("keys").join(event.key_id.to_string());
            let offline_marker = key_dir.join("OFFLINE_STORAGE.json");

            let offline_info = serde_json::to_string_pretty(&event)
                .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize offline storage info: {}", e)))?;

            fs::write(&offline_marker, offline_info)
                .map_err(|e| ProjectionError::IoError(format!("Failed to write offline storage marker: {}", e)))?;
        }
        Ok(())
    }

    /// Project key revocation
    fn project_key_revoked(&mut self, event: &crate::events::KeyRevokedEvent) -> Result<(), ProjectionError> {
        // Mark key as revoked and transition state
        if let Some(key) = self.manifest.keys.iter_mut().find(|k| k.key_id == event.key_id) {
            key.revoked = true;

            // Convert event revocation reason to state machine revocation reason
            use crate::events_legacy::RevocationReason as EventReason;
            use crate::state_machines::key::RevocationReason as StateReason;

            let state_reason = match &event.reason {
                EventReason::KeyCompromise => StateReason::Compromised,
                EventReason::CaCompromise => StateReason::Administrative {
                    reason: "CA compromised".to_string(),
                },
                EventReason::AffiliationChanged => StateReason::EmployeeTermination,
                EventReason::Superseded => StateReason::Superseded,
                EventReason::CessationOfOperation => StateReason::CessationOfOperation,
                EventReason::Unspecified => StateReason::Administrative {
                    reason: "Unspecified".to_string(),
                },
            };

            // Transition state machine to Revoked
            if let Some(current_state) = &key.state {
                // Validate that we can revoke from current state
                let revoked_state = KeyState::Revoked {
                    reason: state_reason.clone(),
                    revoked_at: event.revoked_at,
                    revoked_by: Uuid::now_v7(), // TODO: Parse from event.revoked_by string
                };

                if current_state.can_transition_to(&revoked_state) {
                    key.state = Some(revoked_state);
                } else {
                    return Err(ProjectionError::InvalidStateTransition(
                        format!("Cannot revoke key {} from state: {}", event.key_id, current_state.description())
                    ));
                }
            } else {
                // No state exists, initialize directly to Revoked
                key.state = Some(KeyState::Revoked {
                    reason: state_reason,
                    revoked_at: event.revoked_at,
                    revoked_by: Uuid::now_v7(),
                });
            }
        }

        // Write revocation info to key directory
        let key_dir = self.root_path.join("keys").join(event.key_id.to_string());
        let revocation_path = key_dir.join("REVOKED.json");

        let revocation_json = serde_json::to_string_pretty(&event)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize revocation: {}", e)))?;

        fs::write(&revocation_path, revocation_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write revocation: {}", e)))?;

        Ok(())
    }

    /// Save the manifest to disk
    /// Update organization information
    pub fn set_organization(&mut self, name: String, domain: String, country: String, admin_email: String) -> Result<(), ProjectionError> {
        self.manifest.organization = OrganizationInfo {
            name,
            domain,
            country,
            admin_email,
        };
        self.manifest.updated_at = Utc::now();
        self.save_manifest()?;
        Ok(())
    }

    /// Get organization information
    pub fn get_organization(&self) -> &OrganizationInfo {
        &self.manifest.organization
    }

    /// Add a person to the organization
    pub fn add_person(&mut self, person_id: Uuid, name: String, email: String, role: String, organization_id: Uuid) -> Result<(), ProjectionError> {
        let person_entry = PersonEntry {
            person_id,
            name,
            email,
            role,
            organization_id,
            created_at: Utc::now(),
            // Initialize state machine
            state: Some(PersonState::Created {
                created_at: Utc::now(),
                created_by: Uuid::now_v7(), // System created
            }),
        };

        self.manifest.people.push(person_entry);
        self.manifest.updated_at = Utc::now();
        self.save_manifest()?;
        Ok(())
    }

    /// Get all people in the organization
    pub fn get_people(&self) -> &[PersonEntry] {
        &self.manifest.people
    }

    /// Add a location to the organization
    pub fn add_location(
        &mut self,
        location_id: Uuid,
        name: String,
        location_type: String,
        organization_id: Uuid,
        street: Option<String>,
        city: Option<String>,
        region: Option<String>,
        country: Option<String>,
        postal_code: Option<String>,
    ) -> Result<(), ProjectionError> {
        let location_entry = LocationEntry {
            location_id,
            name,
            location_type,
            organization_id,
            created_at: Utc::now(),
            street,
            city,
            region,
            country,
            postal_code,
            // Initialize state machine
            state: Some(LocationState::Active {
                activated_at: Utc::now(),
                access_grants: Vec::new(),
                assets_stored: 0,
                last_accessed: None,
            }),
        };

        self.manifest.locations.push(location_entry);
        self.manifest.updated_at = Utc::now();
        self.save_manifest()?;
        Ok(())
    }

    /// Get all locations in the organization
    pub fn get_locations(&self) -> &[LocationEntry] {
        &self.manifest.locations
    }

    /// Get all certificates
    pub fn get_certificates(&self) -> &[CertificateEntry] {
        &self.manifest.certificates
    }

    /// Get all keys
    pub fn get_keys(&self) -> &[KeyEntry] {
        &self.manifest.keys
    }

    /// Get all YubiKeys
    pub fn get_yubikeys(&self) -> &[YubiKeyEntry] {
        &self.manifest.yubikeys
    }

    /// Remove a location from the organization
    pub fn remove_location(&mut self, location_id: Uuid) -> Result<(), ProjectionError> {
        let initial_len = self.manifest.locations.len();
        self.manifest.locations.retain(|loc| loc.location_id != location_id);

        if self.manifest.locations.len() == initial_len {
            return Err(ProjectionError::NotFound(format!("Location {} not found", location_id)));
        }

        self.manifest.updated_at = Utc::now();
        self.save_manifest()?;
        Ok(())
    }

    pub fn save_manifest(&self) -> Result<(), ProjectionError> {
        let manifest_path = self.root_path.join("manifest.json");

        let manifest_json = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize manifest: {}", e)))?;

        fs::write(&manifest_path, manifest_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write manifest: {}", e)))?;

        Ok(())
    }

    /// Rebuild projection from event log
    pub fn rebuild_from_events(&mut self) -> Result<(), ProjectionError> {
        let events_dir = self.root_path.join("events");

        // Read all event files in order
        let mut event_files: Vec<_> = fs::read_dir(&events_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to read events directory: {}", e)))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .collect();

        // Sort by filename (timestamp)
        event_files.sort_by_key(|entry| entry.file_name());

        // Reset manifest
        self.manifest = KeyManifest {
            version: "1.0.0".to_string(),
            created_at: self.manifest.created_at,
            updated_at: Utc::now(),
            organization: self.manifest.organization.clone(),
            people: Vec::new(),
            locations: Vec::new(),
            keys: Vec::new(),
            certificates: Vec::new(),
            pki_hierarchies: Vec::new(),
            yubikeys: Vec::new(),
            nats_operators: Vec::new(),
            nats_accounts: Vec::new(),
            nats_users: Vec::new(),
            event_count: 0,
            checksum: String::new(),
        };

        // Replay all events
        for entry in event_files {
            let content = fs::read_to_string(entry.path())
                .map_err(|e| ProjectionError::IoError(format!("Failed to read event file: {}", e)))?;

            let event: KeyEvent = serde_json::from_str(&content)
                .map_err(|e| ProjectionError::ParseError(format!("Invalid event JSON: {}", e)))?;

            self.apply(&event)?;
        }

        Ok(())
    }
}

// Supporting types for file storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyMetadataFile {
    key_id: Uuid,
    algorithm: KeyAlgorithm,
    purpose: KeyPurpose,
    generated_at: DateTime<Utc>,
    generated_by: String,
    hardware_backed: bool,
    metadata: KeyMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CertificateMetadataFile {
    cert_id: Uuid,
    key_id: Uuid,
    subject: String,
    issuer: Option<String>,
    serial_number: String,
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
    is_ca: bool,
    san: Vec<String>,
    key_usage: Vec<String>,
    extended_key_usage: Vec<String>,
}

/// Errors that can occur in projections
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
}