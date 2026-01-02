//! Graph-First Manifest Management
//!
//! This module implements: **The organizational graph drives manifest-centric views**.
//!
//! Uses `LiftedNode` for type-safe access to domain entities.
//!
//! ## Flow
//!
//! 1. User selects a manifest or export node in the graph
//! 2. System analyzes what was exported and where
//! 3. System shows manifest-centric view:
//!    - Export destination and format
//!    - Keys included in export
//!    - Certificates included in export
//!    - NATS credentials included
//!    - Export timestamp and version
//!    - Integrity checksums
//! 4. User can perform manifest-specific operations
//!
//! ## Manifest-Centric Graph Structure
//!
//! ```text
//! Export Manifest "cim-keys-export-2024-01-15"
//!   ├─> Destination: "/mnt/encrypted-sd/cim-keys/"
//!   ├─> Format: Encrypted JSON
//!   ├─> Timestamp: 2024-01-15T14:30:00Z
//!   ├─> Version: 1.0.0
//!   │
//!   ├─> Keys Exported (15):
//!   │   ├─> Root CA Key (RSA 4096)
//!   │   ├─> Intermediate CA Keys (3)
//!   │   ├─> Person Keys (10)
//!   │   └─> NATS Operator Key (Ed25519)
//!   │
//!   ├─> Certificates Exported (20):
//!   │   ├─> Root CA Certificate
//!   │   ├─> Intermediate CA Certificates (3)
//!   │   └─> Leaf Certificates (16)
//!   │
//!   ├─> NATS Credentials (25):
//!   │   ├─> Operator JWT
//!   │   ├─> Account JWTs (3)
//!   │   └─> User Credentials (21)
//!   │
//!   └─> Integrity:
//!       ├─> SHA-256 Checksum: abc123...
//!       ├─> GPG Signature: alice@cowboyai.com
//!       └─> Verification Status: ✓ Valid
//! ```

use std::path::PathBuf;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::gui::graph::{OrganizationConcept, EdgeType};
use crate::lifting::Injection;

/// Export format type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportFormat {
    EncryptedJson,
    PEM,
    PKCS12,
    JWK,
    Unknown(String),
}

/// Manifest-centric analysis of the organizational graph
#[derive(Debug, Clone)]
pub struct ManifestAnalysis {
    /// The manifest being analyzed
    pub manifest_id: Uuid,
    pub manifest_name: String,

    /// Export details
    pub destination: Option<PathBuf>,
    pub format: ExportFormat,
    pub timestamp: DateTime<Utc>,
    pub version: Option<String>,

    /// Keys included in this export
    pub exported_keys: Vec<(Uuid, String)>, // (key_id, key_description)

    /// Certificates included in this export
    pub exported_certificates: Vec<(Uuid, String)>, // (cert_id, cert_subject)

    /// NATS credentials included
    pub exported_nats_operators: Vec<Uuid>,
    pub exported_nats_accounts: Vec<Uuid>,
    pub exported_nats_users: Vec<Uuid>,

    /// YubiKeys referenced (not exported, but documented)
    pub referenced_yubikeys: Vec<(Uuid, String)>, // (yubikey_id, serial_number)

    /// Organization structure exported
    pub exported_organizations: Vec<Uuid>,
    pub exported_units: Vec<Uuid>,
    pub exported_people: Vec<Uuid>,
    pub exported_locations: Vec<Uuid>,

    /// Integrity information
    pub checksum: Option<String>,
    pub signature: Option<String>,
    pub signed_by: Option<Uuid>, // Person who signed the export
}

impl ManifestAnalysis {
    /// Analyze a manifest node in the graph
    pub fn analyze(graph: &OrganizationConcept, manifest_id: Uuid) -> Option<Self> {
        // Find the manifest node
        let node = graph.nodes.get(&manifest_id)?;

        // Extract manifest details from node type
        // Note: Current graph doesn't have a Manifest node type yet
        // This is a placeholder implementation
        let manifest_name = node.visualization().primary_text;
        let timestamp = Utc::now(); // TODO: Extract from node metadata
        let version = None; // TODO: Extract from node metadata
        let destination = None; // TODO: Extract from node metadata
        let format = ExportFormat::EncryptedJson; // TODO: Extract from node metadata

        let mut exported_keys = Vec::new();
        let mut exported_certificates = Vec::new();
        let mut exported_nats_operators = Vec::new();
        let mut exported_nats_accounts = Vec::new();
        let mut exported_nats_users = Vec::new();
        let mut referenced_yubikeys = Vec::new();
        let mut exported_organizations = Vec::new();
        let mut exported_units = Vec::new();
        let mut exported_people = Vec::new();
        let mut exported_locations = Vec::new();
        let mut checksum = None;
        let signature = None;
        let mut signed_by = None;

        // Analyze edges connected to this manifest
        for edge in &graph.edges {
            if edge.from == manifest_id {
                // Outgoing edges (things exported BY this manifest)
                match &edge.edge_type {
                    EdgeType::ExportedTo => {
                        // Track what was exported using lifted_node accessors
                        if let Some(exported_node) = graph.nodes.get(&edge.to) {
                            // Use lifted_node for type-safe access
                            if let Some(key) = exported_node.lifted_node.key() {
                                exported_keys.push((key.id.as_uuid(), exported_node.visualization().primary_text));
                            } else if let Some(cert) = exported_node.lifted_node.certificate() {
                                exported_certificates.push((edge.to, cert.subject.clone()));
                            } else if let Some(yk) = exported_node.lifted_node.yubikey() {
                                referenced_yubikeys.push((edge.to, yk.serial.clone()));
                            } else {
                                // Use injection type for simple identification
                                match exported_node.lifted_node.injection() {
                                    Injection::NatsOperator | Injection::NatsOperatorSimple => {
                                        exported_nats_operators.push(edge.to);
                                    }
                                    Injection::NatsAccount | Injection::NatsAccountSimple => {
                                        exported_nats_accounts.push(edge.to);
                                    }
                                    Injection::NatsUser | Injection::NatsUserSimple | Injection::NatsServiceAccount => {
                                        exported_nats_users.push(edge.to);
                                    }
                                    Injection::Organization => {
                                        exported_organizations.push(edge.to);
                                    }
                                    Injection::OrganizationUnit => {
                                        exported_units.push(edge.to);
                                    }
                                    Injection::Person => {
                                        exported_people.push(edge.to);
                                    }
                                    Injection::Location => {
                                        exported_locations.push(edge.to);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    EdgeType::SignedByPerson => {
                        // Person who signed this export
                        signed_by = Some(edge.to);
                    }
                    _ => {}
                }
            }
        }

        // Extract checksum and signature from manifest node metadata
        if let Some(manifest_node) = graph.nodes.get(&manifest_id) {
            if let Some(manifest) = manifest_node.lifted_node.manifest() {
                checksum = manifest.checksum.clone();
            }
        }

        Some(ManifestAnalysis {
            manifest_id,
            manifest_name,
            destination,
            format,
            timestamp,
            version,
            exported_keys,
            exported_certificates,
            exported_nats_operators,
            exported_nats_accounts,
            exported_nats_users,
            referenced_yubikeys,
            exported_organizations,
            exported_units,
            exported_people,
            exported_locations,
            checksum,
            signature,
            signed_by,
        })
    }

    /// Get total count of exported items
    pub fn total_exported_items(&self) -> usize {
        self.exported_keys.len()
            + self.exported_certificates.len()
            + self.exported_nats_operators.len()
            + self.exported_nats_accounts.len()
            + self.exported_nats_users.len()
            + self.exported_organizations.len()
            + self.exported_units.len()
            + self.exported_people.len()
            + self.exported_locations.len()
    }

    /// Get total count of NATS credentials
    pub fn total_nats_credentials(&self) -> usize {
        self.exported_nats_operators.len()
            + self.exported_nats_accounts.len()
            + self.exported_nats_users.len()
    }

    /// Check if manifest has valid signature
    pub fn is_signed(&self) -> bool {
        self.signature.is_some() && self.signed_by.is_some()
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        let signature_status = if self.is_signed() {
            "Signed"
        } else {
            "Unsigned"
        };

        format!(
            "{}: {} items exported ({} keys, {} certs, {} NATS creds) - {}",
            self.manifest_name,
            self.total_exported_items(),
            self.exported_keys.len(),
            self.exported_certificates.len(),
            self.total_nats_credentials(),
            signature_status
        )
    }
}
