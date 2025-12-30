//! Graph-First Certificate Management
//!
//! This module implements: **The organizational graph drives certificate-centric views**.
//!
//! ## Flow
//!
//! 1. User selects a certificate node in the graph
//! 2. System analyzes certificate relationships and usage
//! 3. System shows certificate-centric view:
//!    - Certificate subject and issuer
//!    - Validity period and status
//!    - Key associated with certificate
//!    - Parent certificate (issuer) in trust chain
//!    - Child certificates (issued by this cert)
//!    - Purpose and key usage
//! 4. User can perform certificate-specific operations
//!
//! ## Certificate-Centric Graph Structure
//!
//! ```text
//! Certificate "X.509 Root CA"
//!   ├─> Type: Root CA
//!   ├─> Subject: "CN=CowboyAI Root CA, O=CowboyAI"
//!   ├─> Issuer: Self-signed
//!   ├─> Validity: 2024-01-01 to 2034-01-01
//!   │
//!   ├─> Signing Key:
//!   │   └─> RSA 4096-bit (Key ID: abc123)
//!   │
//!   ├─> Certificates Issued:
//!   │   ├─> Intermediate CA "Engineering"
//!   │   ├─> Intermediate CA "Operations"
//!   │   └─> Intermediate CA "Security"
//!   │
//!   ├─> Key Usage:
//!   │   ├─> Certificate Sign
//!   │   └─> CRL Sign
//!   │
//!   └─> Storage:
//!       ├─> YubiKey #12345678 (Slot 9d)
//!       └─> Backup: Secure Vault A
//! ```

use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::events::KeyAlgorithm;
use crate::gui::graph::{OrganizationConcept, EdgeType};
use crate::gui::domain_node::{DomainNodeData, Injection};

/// Certificate type classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateType {
    RootCA,
    IntermediateCA,
    Leaf,
}

/// Certificate-centric analysis of the organizational graph
#[derive(Debug, Clone)]
pub struct CertificateAnalysis {
    /// The certificate being analyzed
    pub certificate_id: Uuid,
    pub certificate_type: CertificateType,

    /// Certificate details
    pub subject: String,
    pub issuer: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub key_usage: Vec<String>,
    pub san: Vec<String>, // Subject Alternative Names

    /// The key used to sign this certificate
    pub signing_key_id: Option<Uuid>,
    pub signing_key_algorithm: Option<KeyAlgorithm>,

    /// Parent certificate (the CA that issued this cert)
    pub issuer_certificate_id: Option<Uuid>,

    /// Child certificates (certificates signed by this cert)
    pub issued_certificates: Vec<Uuid>,

    /// Subject of the certificate (person, organization, etc.)
    pub subject_entity_id: Option<Uuid>,
    pub subject_entity_type: Option<String>,

    /// Storage locations for this certificate
    pub storage_locations: Vec<(Uuid, String)>,

    /// YubiKeys containing this certificate
    pub yubikey_slots: Vec<(Uuid, String)>,
}

impl CertificateAnalysis {
    /// Analyze a certificate node in the graph
    pub fn analyze(graph: &OrganizationConcept, certificate_id: Uuid) -> Option<Self> {
        // Find the certificate node
        let node = graph.nodes.get(&certificate_id)?;

        // Extract certificate type and details based on node type
        let (certificate_type, subject, issuer, not_before, not_after, key_usage, san) = match node.domain_node.data() {
            DomainNodeData::RootCertificate {
                subject,
                issuer,
                not_before,
                not_after,
                key_usage,
                ..
            } => (
                CertificateType::RootCA,
                subject.clone(),
                issuer.clone(),
                *not_before,
                *not_after,
                key_usage.clone(),
                Vec::new(),
            ),
            DomainNodeData::IntermediateCertificate {
                subject,
                issuer,
                not_before,
                not_after,
                key_usage,
                ..
            } => (
                CertificateType::IntermediateCA,
                subject.clone(),
                issuer.clone(),
                *not_before,
                *not_after,
                key_usage.clone(),
                Vec::new(),
            ),
            DomainNodeData::LeafCertificate {
                subject,
                issuer,
                not_before,
                not_after,
                key_usage,
                san,
                ..
            } => (
                CertificateType::Leaf,
                subject.clone(),
                issuer.clone(),
                *not_before,
                *not_after,
                key_usage.clone(),
                san.clone(),
            ),
            _ => return None,
        };

        let mut signing_key_id = None;
        let mut signing_key_algorithm = None;
        let mut issuer_certificate_id = None;
        let mut issued_certificates = Vec::new();
        let mut subject_entity_id = None;
        let mut subject_entity_type = None;
        let mut storage_locations = Vec::new();
        let mut yubikey_slots = Vec::new();

        // Analyze edges connected to this certificate
        for edge in &graph.edges {
            if edge.to == certificate_id {
                // Incoming edges (pointing TO this certificate)
                match &edge.edge_type {
                    EdgeType::CertificateUsesKey => {
                        // Key used by this certificate
                        signing_key_id = Some(edge.from);
                        if let Some(key_node) = graph.nodes.get(&edge.from) {
                            if let DomainNodeData::Key { algorithm, .. } = key_node.domain_node.data() {
                                signing_key_algorithm = Some(algorithm.clone());
                            }
                        }
                    }
                    EdgeType::Signs | EdgeType::SignedBy => {
                        // Parent certificate that signed this one
                        issuer_certificate_id = Some(edge.from);
                    }
                    EdgeType::IssuedTo => {
                        // Entity (person/org) this certificate was issued to
                        subject_entity_id = Some(edge.from);
                        if let Some(entity_node) = graph.nodes.get(&edge.from) {
                            subject_entity_type = Some(match entity_node.domain_node.injection() {
                                Injection::Person => "Person".to_string(),
                                Injection::Organization => "Organization".to_string(),
                                Injection::OrganizationUnit => "OrganizationalUnit".to_string(),
                                _ => "Unknown".to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            } else if edge.from == certificate_id {
                // Outgoing edges (pointing FROM this certificate)
                match &edge.edge_type {
                    EdgeType::Signs | EdgeType::SignedBy => {
                        // Certificate signed by this one (child certificate)
                        issued_certificates.push(edge.to);
                    }
                    EdgeType::StoredAt => {
                        // Storage location for this certificate
                        if let Some(location_node) = graph.nodes.get(&edge.to) {
                            storage_locations.push((edge.to, location_node.label.clone()));
                        }
                    }
                    EdgeType::StoredInYubiKeySlot(slot) => {
                        // YubiKey slot storing this certificate
                        yubikey_slots.push((edge.to, slot.clone()));
                    }
                    _ => {}
                }
            }
        }

        Some(CertificateAnalysis {
            certificate_id,
            certificate_type,
            subject,
            issuer,
            not_before,
            not_after,
            key_usage,
            san,
            signing_key_id,
            signing_key_algorithm,
            issuer_certificate_id,
            issued_certificates,
            subject_entity_id,
            subject_entity_type,
            storage_locations,
            yubikey_slots,
        })
    }

    /// Check if the certificate is currently valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        now >= self.not_before && now <= self.not_after
    }

    /// Get days until expiration (negative if expired)
    pub fn days_until_expiration(&self) -> i64 {
        let now = Utc::now();
        (self.not_after - now).num_days()
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        let validity_status = if self.is_valid() {
            format!("Valid ({} days remaining)", self.days_until_expiration())
        } else {
            "Expired".to_string()
        };

        format!(
            "{:?}: {} - {} - {} issued certs",
            self.certificate_type,
            self.subject,
            validity_status,
            self.issued_certificates.len()
        )
    }
}
