//! Graph-First Key Management
//!
//! This module implements: **The organizational graph drives key-centric views**.
//!
//! NOTE: Uses deprecated `DomainNodeData` for pattern matching. Migration pending.
//!
//! ## Flow
//!
//! 1. User selects a key node in the graph
//! 2. System analyzes key relationships and usage
//! 3. System shows key-centric view:
//!    - Key owner (person/organization)
//!    - Key purpose and algorithm
//!    - Certificates using this key
//!    - Where the key is stored
//!    - Key rotation history
//!    - Delegations involving this key
//! 4. User can perform key-specific operations
//!
//! ## Key-Centric Graph Structure
//!
//! ```text
//! Key "SSH-ed25519-Alice"
//!   ├─> Algorithm: Ed25519
//!   ├─> Purpose: SSH Authentication
//!   ├─> Owner: Alice Smith
//!   │
//!   ├─> Certificates:
//!   │   └─> X.509 Certificate #abc123
//!   │
//!   ├─> Storage:
//!   │   ├─> YubiKey #12345678 (Slot 9a)
//!   │   └─> Backup: Secure Vault A
//!   │
//!   ├─> Delegations:
//!   │   └─> Delegated to: Bob Jones (Signing Operations)
//!   │
//!   └─> Rotation History:
//!       └─> Previous key: #old-key-123 (rotated 2024-01-15)
//! ```

use uuid::Uuid;

use crate::events::{KeyAlgorithm, KeyPurpose};
use crate::gui::graph::{OrganizationConcept, EdgeType};
use crate::gui::domain_node::DomainNodeData;

/// Key-centric analysis of the organizational graph
#[derive(Debug, Clone)]
pub struct KeyAnalysis {
    /// The key being analyzed
    pub key_id: Uuid,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,

    /// Owner of this key (person or organization)
    pub owner_id: Option<Uuid>,
    pub owner_type: Option<String>,

    /// Certificates that use this key
    pub certificates: Vec<Uuid>,

    /// Storage locations for this key
    pub storage_locations: Vec<(Uuid, String)>, // (location_id, location_type)

    /// YubiKeys containing this key (yubikey_id, slot)
    pub yubikey_slots: Vec<(Uuid, String)>,

    /// Delegations involving this key
    pub delegations: Vec<(Uuid, Uuid)>, // (from_person, to_person)

    /// Key rotation chain (previous -> current -> next)
    pub rotation_history: Vec<Uuid>,
}

impl KeyAnalysis {
    /// Analyze a key node in the graph
    pub fn analyze(graph: &OrganizationConcept, key_id: Uuid) -> Option<Self> {
        // Find the key node
        let node = graph.nodes.get(&key_id)?;
        let (algorithm, purpose) = match node.domain_node.data() {
            DomainNodeData::Key(key) => (key.algorithm.clone(), key.purpose),
            _ => return None,
        };

        let mut owner_id = None;
        let mut owner_type = None;
        let mut certificates = Vec::new();
        let mut storage_locations = Vec::new();
        let mut yubikey_slots = Vec::new();
        let mut delegations = Vec::new();
        let mut rotation_history = Vec::new();

        // Analyze edges connected to this key
        for edge in &graph.edges {
            if edge.to == key_id {
                // Incoming edges (pointing TO this key)
                match &edge.edge_type {
                    EdgeType::OwnsKey => {
                        owner_id = Some(edge.from);
                        if let Some(owner_node) = graph.nodes.get(&edge.from) {
                            owner_type = Some(owner_node.domain_node.injection().display_name().to_string());
                        }
                    }
                    EdgeType::CertificateUsesKey => {
                        certificates.push(edge.from);
                    }
                    EdgeType::KeyRotation => {
                        rotation_history.push(edge.from);
                    }
                    _ => {}
                }
            } else if edge.from == key_id {
                // Outgoing edges (pointing FROM this key)
                match &edge.edge_type {
                    EdgeType::StoredAt => {
                        if let Some(location_node) = graph.nodes.get(&edge.to) {
                            storage_locations.push((edge.to, location_node.visualization().primary_text));
                        }
                    }
                    EdgeType::StoredInYubiKeySlot(slot) => {
                        yubikey_slots.push((edge.to, slot.clone()));
                    }
                    _ => {}
                }
            }
        }

        // Find delegations involving this key
        for edge in &graph.edges {
            if let EdgeType::DelegatesKey(_) = edge.edge_type {
                // Check if this delegation involves our key
                // (We'd need to store key_id in KeyDelegation struct for this to work properly)
                delegations.push((edge.from, edge.to));
            }
        }

        Some(KeyAnalysis {
            key_id,
            algorithm,
            purpose,
            owner_id,
            owner_type,
            certificates,
            storage_locations,
            yubikey_slots,
            delegations,
            rotation_history,
        })
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "{:?} ({:?}): {} certs, {} storage locations, {} YubiKey slots",
            self.algorithm,
            self.purpose,
            self.certificates.len(),
            self.storage_locations.len(),
            self.yubikey_slots.len()
        )
    }
}
