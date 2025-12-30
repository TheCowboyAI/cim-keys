//! Graph-First Location Management
//!
//! This module implements: **The organizational graph drives location-centric views**.
//!
//! ## Flow
//!
//! 1. User selects a location node in the graph
//! 2. System analyzes what is stored at this location
//! 3. System shows location-centric view:
//!    - Keys stored at this location
//!    - Certificates stored at this location
//!    - YubiKeys stored at this location
//!    - Access permissions for this location
//!    - Physical/virtual location details
//! 4. User can perform location-specific operations
//!
//! ## Location-Centric Graph Structure
//!
//! ```text
//! Location "Secure Vault A"
//!   ├─> Type: Physical
//!   ├─> Address: "123 Main St, Floor 3, Room 301"
//!   │
//!   ├─> Assets Stored:
//!   │   ├─> Root CA Key (offline backup)
//!   │   ├─> Intermediate CA Keys (encrypted)
//!   │   ├─> YubiKey #12345678 (backup storage)
//!   │   └─> Certificate Archive (PEM bundle)
//!   │
//!   └─> Access Permissions:
//!       ├─> Alice Smith: Full Access
//!       ├─> Bob Jones: Read-Only
//!       └─> Carol White: Emergency Access
//! ```

use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::Location;
use crate::gui::graph::{OrganizationConcept, EdgeType};
use crate::gui::domain_node::{DomainNodeData, Injection};

/// Location-centric analysis of the organizational graph
#[derive(Debug, Clone)]
pub struct LocationAnalysis {
    /// The location being analyzed
    pub location_id: Uuid,
    pub location: Location,

    /// Keys stored at this location (key_id -> description)
    pub stored_keys: HashMap<Uuid, String>,

    /// Certificates stored at this location
    pub stored_certificates: Vec<Uuid>,

    /// YubiKeys stored at this location
    pub stored_yubikeys: Vec<Uuid>,

    /// People with access to this location (person_id -> access_level)
    pub access_permissions: HashMap<Uuid, String>,

    /// Organizational units that manage this location
    pub managing_units: Vec<Uuid>,
}

impl LocationAnalysis {
    /// Analyze a location node in the graph
    pub fn analyze(graph: &OrganizationConcept, location_id: Uuid) -> Option<Self> {
        // Find the location node
        let node = graph.nodes.get(&location_id)?;
        let location = match node.domain_node.data() {
            DomainNodeData::Location(loc) => loc.clone(),
            _ => return None,
        };

        let mut stored_keys = HashMap::new();
        let mut stored_certificates = Vec::new();
        let mut stored_yubikeys = Vec::new();
        let mut access_permissions = HashMap::new();
        let mut managing_units = Vec::new();

        // Analyze edges connected to this location
        for edge in &graph.edges {
            if edge.to == location_id {
                // Incoming edges (things stored AT this location)
                match edge.edge_type {
                    EdgeType::StoredAt => {
                        if let Some(source_node) = graph.nodes.get(&edge.from) {
                            match source_node.domain_node.data() {
                                DomainNodeData::Key { purpose, .. } => {
                                    stored_keys.insert(edge.from, format!("{:?}", purpose));
                                }
                                _ if source_node.domain_node.injection().is_certificate() => {
                                    stored_certificates.push(edge.from);
                                }
                                _ if source_node.domain_node.injection() == Injection::YubiKey => {
                                    stored_yubikeys.push(edge.from);
                                }
                                _ => {}
                            }
                        }
                    }
                    EdgeType::HasAccess => {
                        // Person has access to this location
                        if graph.nodes.contains_key(&edge.from) {
                            access_permissions.insert(edge.from, "Full Access".to_string());
                        }
                    }
                    _ => {}
                }
            } else if edge.from == location_id {
                // Outgoing edges
                match edge.edge_type {
                    EdgeType::ManagedBy => {
                        // Organizational unit managing this location
                        managing_units.push(edge.to);
                    }
                    _ => {}
                }
            }
        }

        Some(LocationAnalysis {
            location_id,
            location,
            stored_keys,
            stored_certificates,
            stored_yubikeys,
            access_permissions,
            managing_units,
        })
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "{}: {} keys, {} certs, {} YubiKeys, {} access grants",
            self.location.name,
            self.stored_keys.len(),
            self.stored_certificates.len(),
            self.stored_yubikeys.len(),
            self.access_permissions.len()
        )
    }

    /// Check if a person has access to this location
    pub fn has_access(&self, person_id: &Uuid) -> bool {
        self.access_permissions.contains_key(person_id)
    }
}
