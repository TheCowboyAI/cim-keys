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
}

/// Entry for a location in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationEntry {
    pub location_id: Uuid,
    pub name: String,
    pub location_type: String,
    pub organization_id: Uuid,
    pub created_at: DateTime<Utc>,
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
            KeyEvent::CertificateGenerated(e) => self.project_certificate_generated(e)?,
            KeyEvent::YubiKeyProvisioned(e) => self.project_yubikey_provisioned(e)?,
            KeyEvent::PkiHierarchyCreated(e) => self.project_pki_hierarchy_created(e)?,
            KeyEvent::KeyStoredOffline(e) => self.project_key_stored_offline(e)?,
            KeyEvent::KeyRevoked(e) => self.project_key_revoked(e)?,
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

        // Add to manifest
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
        });

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

        // Add to manifest
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
        });

        Ok(())
    }

    /// Project YubiKey provisioning
    fn project_yubikey_provisioned(&mut self, event: &crate::events::YubiKeyProvisionedEvent) -> Result<(), ProjectionError> {
        let yubikey_dir = self.root_path.join("yubikeys").join(&event.yubikey_serial);
        fs::create_dir_all(&yubikey_dir)
            .map_err(|e| ProjectionError::IoError(format!("Failed to create YubiKey directory: {}", e)))?;

        let config_path = yubikey_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&event)
            .map_err(|e| ProjectionError::SerializationError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, config_json)
            .map_err(|e| ProjectionError::IoError(format!("Failed to write config: {}", e)))?;

        self.manifest.yubikeys.push(YubiKeyEntry {
            serial: event.yubikey_serial.clone(),
            provisioned_at: event.provisioned_at,
            slots_used: event.slots_configured.iter().map(|s| s.slot_id.clone()).collect(),
            config_path: format!("yubikeys/{}/config.json", event.yubikey_serial),
        });

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
        // Mark key as revoked in manifest
        if let Some(key) = self.manifest.keys.iter_mut().find(|k| k.key_id == event.key_id) {
            key.revoked = true;
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
    pub fn add_location(&mut self, location_id: Uuid, name: String, location_type: String, organization_id: Uuid) -> Result<(), ProjectionError> {
        let location_entry = LocationEntry {
            location_id,
            name,
            location_type,
            organization_id,
            created_at: Utc::now(),
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
}