// Copyright (c) 2025 - Cowboy AI, LLC.

//! # SD Card Projection
//!
//! Composable projections for exporting domain state to encrypted SD cards.
//!
//! ## Architecture
//!
//! ```text
//! Domain State (KeyManifest)
//!     ↓ via
//! SDCardProjection
//!     ↓ produces
//! SDCardExport (directory structure + files)
//!     ↓ via
//! StoragePort (encryption, write)
//! ```
//!
//! ## Directory Structure on SD Card
//!
//! ```text
//! /mnt/encrypted/cim-keys/
//! ├── manifest.json           # Master index with checksums
//! ├── domain/
//! │   ├── organization.json   # Organization info
//! │   ├── people.json         # All people
//! │   └── locations.json      # Storage locations
//! ├── keys/
//! │   └── {key-id}/
//! │       ├── metadata.json   # Key metadata
//! │       └── public.pem      # Public key (optional)
//! ├── certificates/
//! │   ├── root-ca/
//! │   ├── intermediate-ca/
//! │   └── leaf/
//! ├── nats/
//! │   ├── operator/
//! │   ├── accounts/
//! │   └── users/
//! └── events/
//!     └── {date}/             # Daily event logs
//! ```

use crate::projection::{Projection, ProjectionError};
use crate::projections::KeyManifest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================================
// EXPORT TYPES
// ============================================================================

/// Complete SD card export package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDCardExport {
    /// Export metadata
    pub metadata: ExportMetadata,
    /// Directory structure to create
    pub directories: Vec<PathBuf>,
    /// Files to write (path -> content)
    pub files: Vec<ExportFile>,
    /// Summary statistics
    pub summary: ExportSummary,
}

/// Metadata about the export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub export_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub source: String,
    pub version: String,
    pub checksum: String,
}

/// A file to be written to the SD card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    /// Relative path from export root
    pub path: PathBuf,
    /// File content (JSON serialized)
    pub content: String,
    /// Whether this file contains sensitive data
    pub sensitive: bool,
    /// SHA-256 checksum of content
    pub checksum: String,
}

/// Summary of what's being exported
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportSummary {
    pub organization_name: String,
    pub people_count: usize,
    pub location_count: usize,
    pub key_count: usize,
    pub certificate_count: usize,
    pub nats_operator_count: usize,
    pub nats_account_count: usize,
    pub nats_user_count: usize,
    pub total_files: usize,
    pub total_bytes: usize,
}

// ============================================================================
// PROJECTIONS
// ============================================================================

/// Projection: KeyManifest → SDCardExport
///
/// Transforms the in-memory manifest into a complete export package
/// ready to be written to an SD card.
pub struct ManifestToExportProjection {
    include_public_keys: bool,
    include_certificates: bool,
    include_nats_config: bool,
}

impl Default for ManifestToExportProjection {
    fn default() -> Self {
        Self {
            include_public_keys: true,
            include_certificates: true,
            include_nats_config: true,
        }
    }
}

impl ManifestToExportProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_public_keys(mut self, include: bool) -> Self {
        self.include_public_keys = include;
        self
    }

    pub fn with_certificates(mut self, include: bool) -> Self {
        self.include_certificates = include;
        self
    }

    pub fn with_nats_config(mut self, include: bool) -> Self {
        self.include_nats_config = include;
        self
    }

    /// Calculate SHA-256 checksum of content
    fn calculate_checksum(content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Create an export file with checksum
    fn create_file(path: impl Into<PathBuf>, content: String, sensitive: bool) -> ExportFile {
        let checksum = Self::calculate_checksum(&content);
        ExportFile {
            path: path.into(),
            content,
            sensitive,
            checksum,
        }
    }
}

impl Projection<KeyManifest, SDCardExport, ProjectionError> for ManifestToExportProjection {
    fn project(&self, manifest: KeyManifest) -> Result<SDCardExport, ProjectionError> {
        let export_id = Uuid::now_v7();
        let created_at = Utc::now();
        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut total_bytes = 0usize;

        // Create directory structure
        directories.push(PathBuf::from("domain"));
        directories.push(PathBuf::from("keys"));
        directories.push(PathBuf::from("certificates"));
        directories.push(PathBuf::from("certificates/root-ca"));
        directories.push(PathBuf::from("certificates/intermediate-ca"));
        directories.push(PathBuf::from("certificates/leaf"));
        directories.push(PathBuf::from("nats"));
        directories.push(PathBuf::from("nats/operator"));
        directories.push(PathBuf::from("nats/accounts"));
        directories.push(PathBuf::from("nats/users"));
        directories.push(PathBuf::from("events"));

        // Export organization info
        let org_content = serde_json::to_string_pretty(&manifest.organization)
            .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
        total_bytes += org_content.len();
        files.push(Self::create_file("domain/organization.json", org_content, false));

        // Export people
        let people_content = serde_json::to_string_pretty(&manifest.people)
            .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
        total_bytes += people_content.len();
        files.push(Self::create_file("domain/people.json", people_content, false));

        // Export locations
        let locations_content = serde_json::to_string_pretty(&manifest.locations)
            .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
        total_bytes += locations_content.len();
        files.push(Self::create_file("domain/locations.json", locations_content, false));

        // Export keys
        for key in &manifest.keys {
            let key_dir = PathBuf::from(format!("keys/{}", key.key_id));
            directories.push(key_dir.clone());

            let key_content = serde_json::to_string_pretty(&key)
                .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
            total_bytes += key_content.len();
            files.push(Self::create_file(
                key_dir.join("metadata.json"),
                key_content,
                true, // Key metadata is sensitive
            ));
        }

        // Export certificates
        if self.include_certificates {
            for cert in &manifest.certificates {
                let cert_dir = if cert.is_ca {
                    if cert.issuer.is_none() {
                        PathBuf::from("certificates/root-ca")
                    } else {
                        PathBuf::from("certificates/intermediate-ca")
                    }
                } else {
                    PathBuf::from("certificates/leaf")
                };

                let cert_content = serde_json::to_string_pretty(&cert)
                    .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
                total_bytes += cert_content.len();
                files.push(Self::create_file(
                    cert_dir.join(format!("{}.json", cert.cert_id)),
                    cert_content,
                    false,
                ));
            }
        }

        // Export NATS config
        if self.include_nats_config {
            // Operators
            for operator in &manifest.nats_operators {
                let op_content = serde_json::to_string_pretty(&operator)
                    .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
                total_bytes += op_content.len();
                files.push(Self::create_file(
                    format!("nats/operator/{}.json", operator.operator_id),
                    op_content,
                    true, // NATS credentials are sensitive
                ));
            }

            // Accounts
            for account in &manifest.nats_accounts {
                let acc_content = serde_json::to_string_pretty(&account)
                    .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
                total_bytes += acc_content.len();
                files.push(Self::create_file(
                    format!("nats/accounts/{}.json", account.account_id),
                    acc_content,
                    true,
                ));
            }

            // Users
            for user in &manifest.nats_users {
                let user_content = serde_json::to_string_pretty(&user)
                    .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
                total_bytes += user_content.len();
                files.push(Self::create_file(
                    format!("nats/users/{}.json", user.user_id),
                    user_content,
                    true,
                ));
            }
        }

        // Build summary
        let summary = ExportSummary {
            organization_name: manifest.organization.name.clone(),
            people_count: manifest.people.len(),
            location_count: manifest.locations.len(),
            key_count: manifest.keys.len(),
            certificate_count: manifest.certificates.len(),
            nats_operator_count: manifest.nats_operators.len(),
            nats_account_count: manifest.nats_accounts.len(),
            nats_user_count: manifest.nats_users.len(),
            total_files: files.len(),
            total_bytes,
        };

        // Create manifest file with all checksums
        let manifest_export = ManifestExport {
            version: manifest.version.clone(),
            export_id,
            created_at,
            organization: manifest.organization.name.clone(),
            file_checksums: files.iter()
                .map(|f| (f.path.display().to_string(), f.checksum.clone()))
                .collect(),
            summary: summary.clone(),
        };

        let manifest_content = serde_json::to_string_pretty(&manifest_export)
            .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;
        let manifest_checksum = Self::calculate_checksum(&manifest_content);
        files.insert(0, Self::create_file("manifest.json", manifest_content, false));

        // Build metadata
        let metadata = ExportMetadata {
            export_id,
            created_at,
            source: "cim-keys".to_string(),
            version: manifest.version,
            checksum: manifest_checksum,
        };

        Ok(SDCardExport {
            metadata,
            directories,
            files,
            summary,
        })
    }

    fn name(&self) -> &'static str {
        "ManifestToExport"
    }
}

/// Manifest file structure for the export
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestExport {
    version: String,
    export_id: Uuid,
    created_at: DateTime<Utc>,
    organization: String,
    file_checksums: HashMap<String, String>,
    summary: ExportSummary,
}

// ============================================================================
// WRITE PROJECTION
// ============================================================================

/// Projection: SDCardExport → WriteResult
///
/// Actually writes the export to the filesystem.
/// This is the only projection that performs I/O.
pub struct ExportToFilesystemProjection {
    base_path: PathBuf,
    create_directories: bool,
}

impl ExportToFilesystemProjection {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            create_directories: true,
        }
    }

    pub fn without_directory_creation(mut self) -> Self {
        self.create_directories = false;
        self
    }
}

/// Result of writing to filesystem
#[derive(Debug, Clone)]
pub struct WriteResult {
    pub base_path: PathBuf,
    pub files_written: usize,
    pub bytes_written: usize,
    pub errors: Vec<String>,
}

impl Projection<SDCardExport, WriteResult, ProjectionError> for ExportToFilesystemProjection {
    fn project(&self, export: SDCardExport) -> Result<WriteResult, ProjectionError> {
        use std::fs;

        let mut files_written = 0;
        let mut bytes_written = 0;
        let mut errors = Vec::new();

        // Create directories
        if self.create_directories {
            for dir in &export.directories {
                let full_path = self.base_path.join(dir);
                if let Err(e) = fs::create_dir_all(&full_path) {
                    errors.push(format!("Failed to create {}: {}", full_path.display(), e));
                }
            }
        }

        // Write files
        for file in &export.files {
            let full_path = self.base_path.join(&file.path);

            // Ensure parent directory exists
            if let Some(parent) = full_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    errors.push(format!("Failed to create parent dir for {}: {}", full_path.display(), e));
                    continue;
                }
            }

            match fs::write(&full_path, &file.content) {
                Ok(_) => {
                    files_written += 1;
                    bytes_written += file.content.len();
                }
                Err(e) => {
                    errors.push(format!("Failed to write {}: {}", full_path.display(), e));
                }
            }
        }

        if !errors.is_empty() && files_written == 0 {
            return Err(ProjectionError::IoError(errors.join("; ")));
        }

        Ok(WriteResult {
            base_path: self.base_path.clone(),
            files_written,
            bytes_written,
            errors,
        })
    }

    fn name(&self) -> &'static str {
        "ExportToFilesystem"
    }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/// Create a manifest-to-export projection with default settings
pub fn manifest_to_export() -> ManifestToExportProjection {
    ManifestToExportProjection::new()
}

/// Create a complete SD card export pipeline
pub fn sdcard_export_pipeline(base_path: impl Into<PathBuf>) -> impl Projection<KeyManifest, WriteResult, ProjectionError> {
    manifest_to_export().then(ExportToFilesystemProjection::new(base_path))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projections::{OrganizationInfo, PersonEntry};

    fn sample_manifest() -> KeyManifest {
        KeyManifest {
            version: "1.0.0".to_string(),
            updated_at: Utc::now(),
            organization: OrganizationInfo {
                name: "Test Org".to_string(),
                domain: "test.org".to_string(),
                country: "US".to_string(),
                admin_email: "admin@test.org".to_string(),
            },
            people: vec![],
            locations: vec![],
            keys: vec![],
            certificates: vec![],
            pki_hierarchies: vec![],
            yubikeys: vec![],
            nats_operators: vec![],
            nats_accounts: vec![],
            nats_users: vec![],
            event_count: 0,
            checksum: String::new(),
        }
    }

    #[test]
    fn test_manifest_to_export_creates_directories() {
        let manifest = sample_manifest();
        let projection = manifest_to_export();

        let export = projection.project(manifest).unwrap();

        assert!(export.directories.contains(&PathBuf::from("domain")));
        assert!(export.directories.contains(&PathBuf::from("keys")));
        assert!(export.directories.contains(&PathBuf::from("certificates")));
        assert!(export.directories.contains(&PathBuf::from("nats")));
    }

    #[test]
    fn test_manifest_to_export_creates_manifest_file() {
        let manifest = sample_manifest();
        let projection = manifest_to_export();

        let export = projection.project(manifest).unwrap();

        let manifest_file = export.files.iter()
            .find(|f| f.path == PathBuf::from("manifest.json"));
        assert!(manifest_file.is_some());
    }

    #[test]
    fn test_manifest_to_export_includes_organization() {
        let manifest = sample_manifest();
        let projection = manifest_to_export();

        let export = projection.project(manifest).unwrap();

        let org_file = export.files.iter()
            .find(|f| f.path == PathBuf::from("domain/organization.json"));
        assert!(org_file.is_some());
        assert!(org_file.unwrap().content.contains("Test Org"));
    }

    #[test]
    fn test_export_summary_counts() {
        let mut manifest = sample_manifest();
        let org_id = Uuid::now_v7();
        manifest.people = vec![
            PersonEntry {
                person_id: Uuid::now_v7(),
                name: "Alice".to_string(),
                email: "alice@test.org".to_string(),
                role: "Admin".to_string(),
                organization_id: org_id,
                state: None,
            },
        ];

        let projection = manifest_to_export();
        let export = projection.project(manifest).unwrap();

        assert_eq!(export.summary.people_count, 1);
        assert_eq!(export.summary.organization_name, "Test Org");
    }

    #[test]
    fn test_checksum_calculation() {
        let content = "test content";
        let checksum = ManifestToExportProjection::calculate_checksum(content);
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_composed_pipeline() {
        let manifest = sample_manifest();
        let temp_dir = std::env::temp_dir().join(format!("cim-keys-test-{}", Uuid::now_v7()));

        let pipeline = sdcard_export_pipeline(&temp_dir);
        let result = pipeline.project(manifest);

        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert!(write_result.files_written > 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
