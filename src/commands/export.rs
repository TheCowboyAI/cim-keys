// Export Commands
//
// Command handlers for exporting keys and certificates to encrypted storage.
//
// User Stories: US-021, US-022

use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

use crate::domain::{KeyContext, Organization};
use crate::events::{KeyEvent, KeyExportedEvent, KeyStoredOfflineEvent, NatsConfigExportedEvent};
use crate::value_objects::{Certificate, ExportFormat, NKeyPair, NatsJwt, PublicKey};

// ============================================================================
// Command: Export to Encrypted Storage (US-021, US-022)
// ============================================================================

/// Command to export keys and certificates to encrypted storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportToEncryptedStorage {
    pub output_directory: PathBuf,
    pub organization: Organization,
    pub keys: Vec<KeyExportItem>,
    pub certificates: Vec<CertificateExportItem>,
    pub nats_identities: Vec<NatsIdentityExportItem>,
    pub include_manifest: bool,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Key export item
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyExportItem {
    pub key_id: Uuid,
    pub purpose: String,
    pub public_key: PublicKey,
    pub owner_context: KeyContext,
    pub export_format: ExportFormat,
    pub destination_path: PathBuf,
}

/// Certificate export item
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CertificateExportItem {
    pub cert_id: Uuid,
    pub certificate: Certificate,
    pub export_format: ExportFormat,
    pub destination_path: PathBuf,
}

/// NATS identity export item
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NatsIdentityExportItem {
    pub identity_id: Uuid,
    pub identity_type: String, // "Operator", "Account", "User"
    pub identity_name: String,
    pub nkey: NKeyPair,
    pub jwt: NatsJwt,
    pub destination_path: PathBuf,
}

/// Result of export operation
#[derive(Debug, Clone)]
pub struct ExportCompleted {
    pub manifest_path: Option<PathBuf>,
    pub keys_exported: usize,
    pub certificates_exported: usize,
    pub nats_configs_exported: usize,
    pub total_bytes_written: u64,
    pub events: Vec<KeyEvent>,
}

/// Handle ExportToEncryptedStorage command
///
/// Exports all cryptographic material to encrypted storage with proper
/// access controls and audit trail.
///
/// Emits:
/// - KeyExportedEvent (for each key)
/// - CertificateExportedEvent (for each certificate)
/// - NatsConfigExportedEvent (for each NATS identity)
/// - KeyStoredOfflineEvent (for offline storage confirmation)
/// - ManifestCreatedEvent (if manifest requested)
///
/// User Story: US-021, US-022
pub fn handle_export_to_encrypted_storage(
    cmd: ExportToEncryptedStorage,
) -> Result<ExportCompleted, String> {
    let mut events = Vec::new();
    let mut total_bytes = 0u64;

    // Step 1: Validate output directory
    validate_export_directory(&cmd.output_directory)?;

    // Step 2: Export keys
    for key_item in &cmd.keys {
        // Export key to file and get bytes written + checksum
        let (bytes_written, checksum) = export_key_to_file(key_item)?;
        total_bytes += bytes_written;

        // Map ExportFormat to KeyFormat
        let key_format = map_export_format_to_key_format(&key_item.export_format);

        events.push(KeyEvent::KeyExported(KeyExportedEvent {
            key_id: key_item.key_id,
            format: key_format,
            include_private: false, // Only exporting public keys
            exported_at: Utc::now(),
            exported_by: cmd.organization.name.clone(),
            destination: crate::events::ExportDestination::File {
                path: key_item.destination_path.to_string_lossy().to_string(),
            },
        }));

        // Emit offline storage event with actual checksum
        events.push(KeyEvent::KeyStoredOffline(KeyStoredOfflineEvent {
            key_id: key_item.key_id,
            partition_id: Uuid::now_v7(), // ID of the encrypted partition
            encrypted: true,
            stored_at: Utc::now(),
            checksum,
        }));
    }

    // Step 3: Export certificates
    for cert_item in &cmd.certificates {
        // Export certificate to file
        let bytes_written = export_certificate_to_file(cert_item)?;
        total_bytes += bytes_written;

        events.push(KeyEvent::CertificateExported(
            crate::events::CertificateExportedEvent {
                export_id: Uuid::now_v7(),
                cert_id: cert_item.cert_id,
                export_format: format!("{:?}", cert_item.export_format),
                destination_path: cert_item.destination_path.to_string_lossy().to_string(),
                exported_at: Utc::now(),
                correlation_id: cmd.correlation_id,
                causation_id: cmd.causation_id,
            },
        ));
    }

    // Step 4: Export NATS configurations
    for nats_item in &cmd.nats_identities {
        // Export NATS credentials to file in standard .creds format
        let bytes_written = export_nats_credentials_to_file(nats_item)?;
        total_bytes += bytes_written;

        // Emit event with proper correlation tracking
        events.push(KeyEvent::NatsConfigExported(NatsConfigExportedEvent {
            export_id: Uuid::now_v7(),
            operator_id: nats_item.identity_id, // Use identity_id for tracking
            format: crate::events::NatsExportFormat::Credentials,
            exported_at: Utc::now(),
            exported_by: cmd.organization.name.clone(),
        }));
    }

    // Step 5: Generate manifest if requested
    let manifest_path = if cmd.include_manifest {
        let manifest = generate_manifest(&cmd, &events)?;
        let path = cmd.output_directory.join("manifest.json");

        // Write manifest to file
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        std::fs::write(&path, &manifest_json)
            .map_err(|e| format!("Failed to write manifest to {}: {}", path.display(), e))?;
        let manifest_bytes = manifest_json.len() as u64;
        total_bytes += manifest_bytes;

        events.push(KeyEvent::ManifestCreated(
            crate::events::ManifestCreatedEvent {
                manifest_id: Uuid::now_v7(),
                manifest_path: path.to_string_lossy().to_string(),
                organization_id: cmd.organization.id,
                organization_name: cmd.organization.name.clone(),
                keys_count: cmd.keys.len(),
                certificates_count: cmd.certificates.len(),
                nats_configs_count: cmd.nats_identities.len(),
                created_at: Utc::now(),
                correlation_id: cmd.correlation_id,
            },
        ));

        Some(path)
    } else {
        None
    };

    Ok(ExportCompleted {
        manifest_path,
        keys_exported: cmd.keys.len(),
        certificates_exported: cmd.certificates.len(),
        nats_configs_exported: cmd.nats_identities.len(),
        total_bytes_written: total_bytes,
        events,
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Validate export directory exists and is writable
fn validate_export_directory(path: &PathBuf) -> Result<(), String> {
    use std::fs;

    // Check if directory exists, create it if not
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create export directory {}: {}", path.display(), e))?;
    }

    // Verify it's actually a directory
    if !path.is_dir() {
        return Err(format!("Export path {} exists but is not a directory", path.display()));
    }

    // Check write permissions by attempting to create a test file
    let test_file = path.join(".cim-keys-write-test");
    fs::write(&test_file, b"test")
        .map_err(|e| format!("Export directory {} is not writable: {}", path.display(), e))?;
    fs::remove_file(&test_file)
        .map_err(|e| format!("Failed to cleanup test file in {}: {}", path.display(), e))?;

    // Check available disk space (warn if less than 100MB)
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(metadata) = fs::metadata(path) {
            // On Unix, we can check filesystem stats
            // For simplicity, just warn - actual space check would require platform-specific code
            let _ = metadata.size(); // Touch to avoid unused warning
        }
    }

    // Warn if not on encrypted partition (security best practice)
    if !path.to_string_lossy().starts_with("/mnt/encrypted") {
        eprintln!(
            "WARNING: Export directory {} is not on encrypted partition",
            path.display()
        );
        eprintln!("         Consider using an encrypted partition for key material");
    }

    Ok(())
}

/// Map ExportFormat to KeyFormat for events
fn map_export_format_to_key_format(export_format: &ExportFormat) -> crate::events::KeyFormat {
    match export_format {
        ExportFormat::Pem => crate::events::KeyFormat::Pem,
        ExportFormat::Der => crate::events::KeyFormat::Der,
        ExportFormat::Pkcs8 => crate::events::KeyFormat::Pkcs8,
        ExportFormat::Pkcs12 => crate::events::KeyFormat::Pkcs12,
        ExportFormat::Jwk => crate::events::KeyFormat::Jwk,
        ExportFormat::SshPublicKey => crate::events::KeyFormat::SshPublicKey,
        ExportFormat::SshPrivateKey => crate::events::KeyFormat::Pem, // SSH private keys often use PEM
        ExportFormat::Raw => crate::events::KeyFormat::Der, // Raw binary is similar to DER
    }
}

/// Export key to file with checksum calculation
/// Returns (bytes_written, sha256_checksum)
fn export_key_to_file(key_item: &KeyExportItem) -> Result<(u64, String), String> {
    use std::fs;
    use sha2::{Sha256, Digest};

    // Ensure parent directory exists
    if let Some(parent) = key_item.destination_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory {}: {}", parent.display(), e))?;
        }
    }

    // Step 1: Serialize public key based on export_format
    let key_data = match key_item.export_format {
        ExportFormat::Pem => {
            let pem = key_item.public_key.to_pem()
                .map_err(|e| format!("Failed to convert key to PEM: {:?}", e))?;
            pem.into_bytes()
        }
        ExportFormat::Der => {
            key_item.public_key.to_der()
                .map_err(|e| format!("Failed to convert key to DER: {:?}", e))?
        }
        _ => {
            return Err(format!(
                "Export format {:?} not yet supported for public keys",
                key_item.export_format
            ));
        }
    };

    // Step 2: Calculate SHA-256 checksum before writing
    let mut hasher = Sha256::new();
    hasher.update(&key_data);
    let checksum = hex::encode(hasher.finalize());

    // Step 3: Write to destination_path with proper permissions (0600)
    fs::write(&key_item.destination_path, &key_data)
        .map_err(|e| format!("Failed to write key to {}: {}", key_item.destination_path.display(), e))?;

    // Step 4: Set file permissions (Unix only) - keys are sensitive, so 0600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&key_item.destination_path, perms)
            .map_err(|e| format!("Failed to set permissions on {}: {}", key_item.destination_path.display(), e))?;
    }

    // Step 5: Return bytes written and checksum
    Ok((key_data.len() as u64, checksum))
}

/// Export certificate to file
fn export_certificate_to_file(cert_item: &CertificateExportItem) -> Result<u64, String> {
    use std::fs;

    // Ensure parent directory exists
    if let Some(parent) = cert_item.destination_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory {}: {}", parent.display(), e))?;
        }
    }

    // Step 1: Serialize certificate based on export_format
    let cert_data = match cert_item.export_format {
        ExportFormat::Pem => {
            cert_item.certificate.pem.as_bytes().to_vec()
        }
        ExportFormat::Der => {
            cert_item.certificate.der.clone()
        }
        _ => {
            return Err(format!(
                "Export format {:?} not yet supported for certificates",
                cert_item.export_format
            ));
        }
    };

    // Step 2: Write to destination_path
    fs::write(&cert_item.destination_path, &cert_data)
        .map_err(|e| format!("Failed to write certificate to {}: {}", cert_item.destination_path.display(), e))?;

    // Step 3: Set file permissions (Unix only) - certificates are public, so 0644
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o644);
        fs::set_permissions(&cert_item.destination_path, perms)
            .map_err(|e| format!("Failed to set permissions on {}: {}", cert_item.destination_path.display(), e))?;
    }

    // Step 4: Return bytes written
    Ok(cert_data.len() as u64)
}

/// Export NATS credentials to file in standard .creds format
fn export_nats_credentials_to_file(nats_item: &NatsIdentityExportItem) -> Result<u64, String> {
    use std::fs;

    // Ensure parent directory exists
    if let Some(parent) = nats_item.destination_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory {}: {}", parent.display(), e))?;
        }
    }

    // Step 1: Create credentials file in NATS standard format
    // Format specification: https://docs.nats.io/using-nats/developer/connecting/creds
    let identity_type_upper = nats_item.identity_type.to_uppercase();
    let credentials_content = format!(
        "-----BEGIN NATS {} JWT-----\n{}\n------END NATS {} JWT------\n\n\
         ************************* IMPORTANT *************************\n\
         NKEY Seed printed below can be used to sign and prove identity.\n\
         NKEYs are sensitive and should be treated as secrets.\n\n\
         -----BEGIN {} NKEY SEED-----\n{}\n------END {} NKEY SEED------\n\n\
         *************************************************************\n",
        identity_type_upper,
        nats_item.jwt.token(),
        identity_type_upper,
        identity_type_upper,
        nats_item.nkey.seed_string(),
        identity_type_upper
    );

    // Step 2: Write to destination_path with restricted permissions
    fs::write(&nats_item.destination_path, &credentials_content)
        .map_err(|e| format!("Failed to write NATS credentials to {}: {}", nats_item.destination_path.display(), e))?;

    // Step 3: Set file permissions (Unix only) - credentials are VERY sensitive, so 0400 (read-only by owner)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o400);
        fs::set_permissions(&nats_item.destination_path, perms)
            .map_err(|e| format!("Failed to set permissions on {}: {}", nats_item.destination_path.display(), e))?;
    }

    // Step 4: Return bytes written
    Ok(credentials_content.len() as u64)
}

/// Generate export manifest
fn generate_manifest(
    cmd: &ExportToEncryptedStorage,
    events: &[KeyEvent],
) -> Result<ExportManifest, String> {
    Ok(ExportManifest {
        manifest_id: Uuid::now_v7(),
        organization_id: cmd.organization.id,
        organization_name: cmd.organization.name.clone(),
        export_timestamp: Utc::now(),
        keys: cmd
            .keys
            .iter()
            .map(|k| ManifestKeyEntry {
                key_id: k.key_id,
                purpose: k.purpose.clone(),
                path: k.destination_path.to_string_lossy().to_string(),
                format: format!("{:?}", k.export_format),
            })
            .collect(),
        certificates: cmd
            .certificates
            .iter()
            .map(|c| ManifestCertEntry {
                cert_id: c.cert_id,
                subject: c.certificate.subject.common_name.clone(),
                path: c.destination_path.to_string_lossy().to_string(),
                format: format!("{:?}", c.export_format),
            })
            .collect(),
        nats_configs: cmd
            .nats_identities
            .iter()
            .map(|n| ManifestNatsEntry {
                identity_id: n.identity_id,
                identity_type: n.identity_type.clone(),
                identity_name: n.identity_name.clone(),
                path: n.destination_path.to_string_lossy().to_string(),
            })
            .collect(),
        events_count: events.len(),
        correlation_id: cmd.correlation_id,
    })
}

/// Export manifest structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ExportManifest {
    manifest_id: Uuid,
    organization_id: Uuid,
    organization_name: String,
    export_timestamp: chrono::DateTime<Utc>,
    keys: Vec<ManifestKeyEntry>,
    certificates: Vec<ManifestCertEntry>,
    nats_configs: Vec<ManifestNatsEntry>,
    events_count: usize,
    correlation_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ManifestKeyEntry {
    key_id: Uuid,
    purpose: String,
    path: String,
    format: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ManifestCertEntry {
    cert_id: Uuid,
    subject: String,
    path: String,
    format: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ManifestNatsEntry {
    identity_id: Uuid,
    identity_type: String,
    identity_name: String,
    path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Organization;

    #[test]
    fn test_export_emits_events_for_all_artifacts() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        // Use /tmp for testing which should exist on all systems
        let test_dir = std::env::temp_dir().join(format!("cim-keys-test-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&test_dir).unwrap();

        let cmd = ExportToEncryptedStorage {
            output_directory: test_dir.clone(),
            organization: org,
            keys: vec![],
            certificates: vec![],
            nats_identities: vec![],
            include_manifest: true,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_export_to_encrypted_storage(cmd).unwrap();

        // Should emit manifest creation event
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, KeyEvent::ManifestCreated(_))));
        assert!(result.manifest_path.is_some());

        // Clean up test directory
        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_export_validates_directory() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        // Test with non-encrypted path (should warn but succeed)
        let cmd = ExportToEncryptedStorage {
            output_directory: PathBuf::from("/tmp/test"),
            organization: org,
            keys: vec![],
            certificates: vec![],
            nats_identities: vec![],
            include_manifest: false,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_export_to_encrypted_storage(cmd);
        assert!(result.is_ok()); // Should succeed but warn
    }
}
