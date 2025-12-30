//! Graph-First Person Management
//!
//! This module implements: **The organizational graph drives person-centric views**.
//!
//! ## Flow
//!
//! 1. User selects a person node in the graph
//! 2. System analyzes person's relationships and roles
//! 3. System shows person-centric view:
//!    - Roles assigned to this person
//!    - Keys owned by this person
//!    - Delegations from/to this person
//!    - YubiKeys assigned to this person
//!    - Certificates issued to this person
//!    - NATS users for this person
//! 4. User can perform person-specific operations from this view
//!
//! ## Person-Centric Graph Structure
//!
//! ```text
//! Person "Alice Smith"
//!   ├─> Role: "Developer"
//!   ├─> Role: "Security Admin"
//!   │
//!   ├─> Keys Owned:
//!   │   ├─> SSH Key (ed25519)
//!   │   ├─> GPG Key (RSA 4096)
//!   │   └─> X.509 Certificate Key
//!   │
//!   ├─> Delegations:
//!   │   ├─> Delegated from: "Bob Jones" (Signing Key)
//!   │   └─> Delegated to: "Carol White" (Backup Access)
//!   │
//!   ├─> YubiKey: Serial #12345678
//!   │   ├─> Slot 9a: Authentication
//!   │   ├─> Slot 9c: Digital Signature
//!   │   └─> Slot 9d: Key Management
//!   │
//!   └─> NATS User: "alice@engineering"
//!       └─> Account: "Engineering"
//! ```

use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Person, KeyOwnerRole, KeyDelegation};
use crate::gui::graph::{OrganizationConcept, EdgeType};
use crate::gui::domain_node::DomainNodeData;

/// Person-centric analysis of the organizational graph
#[derive(Debug, Clone)]
pub struct PersonAnalysis {
    /// The person being analyzed
    pub person_id: Uuid,
    pub person: Person,
    pub role: KeyOwnerRole,

    /// Roles assigned to this person
    pub roles: Vec<Uuid>,

    /// Keys owned by this person (key_id -> node_id in graph)
    pub owned_keys: HashMap<Uuid, String>,

    /// Delegations where this person is the delegator
    pub delegations_from: Vec<KeyDelegation>,

    /// Delegations where this person is the delegate
    pub delegations_to: Vec<KeyDelegation>,

    /// YubiKeys assigned to this person
    pub yubikeys: Vec<Uuid>,

    /// Certificates issued to this person
    pub certificates: Vec<Uuid>,

    /// NATS users for this person
    pub nats_users: Vec<Uuid>,
}

impl PersonAnalysis {
    /// Analyze a person node in the graph
    pub fn analyze(graph: &OrganizationConcept, person_id: Uuid) -> Option<Self> {
        // Find the person node
        let node = graph.nodes.get(&person_id)?;
        let (person, role) = match node.domain_node.data() {
            DomainNodeData::Person { person, role } => (person.clone(), role.clone()),
            _ => return None,
        };

        let mut roles = Vec::new();
        let mut owned_keys = HashMap::new();
        let mut delegations_from = Vec::new();
        let mut delegations_to = Vec::new();
        let mut yubikeys = Vec::new();
        let mut certificates = Vec::new();
        let mut nats_users = Vec::new();

        // Analyze edges connected to this person
        for edge in &graph.edges {
            if edge.from == person_id {
                // Outgoing edges
                match edge.edge_type {
                    EdgeType::HasRole { .. } => {
                        roles.push(edge.to);
                    }
                    EdgeType::OwnsKey => {
                        if let Some(target_node) = graph.nodes.get(&edge.to) {
                            owned_keys.insert(edge.to, target_node.visualization().primary_text);
                        }
                    }
                    EdgeType::DelegatesKey(ref delegation) => {
                        delegations_from.push(delegation.clone());
                    }
                    EdgeType::OwnsYubiKey => {
                        yubikeys.push(edge.to);
                    }
                    _ => {}
                }
            } else if edge.to == person_id {
                // Incoming edges
                match edge.edge_type {
                    EdgeType::DelegatesKey(ref delegation) => {
                        delegations_to.push(delegation.clone());
                    }
                    EdgeType::IssuedTo => {
                        certificates.push(edge.from);
                    }
                    EdgeType::MapsToPerson => {
                        nats_users.push(edge.from);
                    }
                    _ => {}
                }
            }
        }

        Some(PersonAnalysis {
            person_id,
            person,
            role,
            roles,
            owned_keys,
            delegations_from,
            delegations_to,
            yubikeys,
            certificates,
            nats_users,
        })
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "{}: {} roles, {} keys, {} delegations, {} YubiKeys, {} certificates",
            self.person.name,
            self.roles.len(),
            self.owned_keys.len(),
            self.delegations_from.len() + self.delegations_to.len(),
            self.yubikeys.len(),
            self.certificates.len()
        )
    }
}
