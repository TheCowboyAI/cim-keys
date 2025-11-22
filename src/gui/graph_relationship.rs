//! Graph-First Relationship Management
//!
//! This module implements: **The organizational graph drives relationship analysis**.
//!
//! ## Flow
//!
//! 1. User views the organizational graph
//! 2. System analyzes all relationships and trust connections
//! 3. System can highlight specific relationship types:
//!    - Key delegations
//!    - Trust relationships
//!    - Role assignments
//!    - Access permissions
//!    - Organizational hierarchy
//! 4. User can filter and query relationships
//!
//! ## Relationship Types in Graph
//!
//! ```text
//! Relationship Analysis
//!   │
//!   ├─> Trust Relationships:
//!   │   ├─> Alice trusts Bob (High Trust)
//!   │   └─> Bob trusts Carol (Medium Trust)
//!   │
//!   ├─> Key Delegations:
//!   │   ├─> Alice → Bob (Signing Key, valid until 2025-01-01)
//!   │   └─> Bob → Carol (Backup Access, valid until 2024-12-31)
//!   │
//!   ├─> Role Assignments:
//!   │   ├─> Alice has role "Developer"
//!   │   └─> Bob has role "Security Admin"
//!   │
//!   ├─> Access Permissions:
//!   │   ├─> Alice → Vault A (Full Access)
//!   │   └─> Carol → Vault B (Read Only)
//!   │
//!   └─> Organizational Hierarchy:
//!       ├─> Engineering → Alice (member)
//!       └─> Engineering → Bob (manager)
//! ```

use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::KeyDelegation;
use crate::gui::graph::{OrganizationGraph, EdgeType};

/// Relationship analysis of the organizational graph
#[derive(Debug, Clone)]
pub struct RelationshipAnalysis {
    /// All trust relationships (trustor_id -> (trustee_id, trust_level))
    pub trust_relationships: HashMap<Uuid, Vec<(Uuid, String)>>,

    /// All key delegations (delegator_id -> Vec<delegations>)
    pub key_delegations: HashMap<Uuid, Vec<KeyDelegation>>,

    /// All role assignments (person_id -> Vec<role_id>)
    pub role_assignments: HashMap<Uuid, Vec<Uuid>>,

    /// All access permissions (person_id -> Vec<location_id>)
    pub access_permissions: HashMap<Uuid, Vec<Uuid>>,

    /// Organizational membership (unit_id -> Vec<person_id>)
    pub org_membership: HashMap<Uuid, Vec<Uuid>>,

    /// Organizational management (manager_id -> Vec<unit_id>)
    pub org_management: HashMap<Uuid, Vec<Uuid>>,
}

impl RelationshipAnalysis {
    /// Analyze all relationships in the graph
    pub fn analyze(graph: &OrganizationGraph) -> Self {
        let mut trust_relationships: HashMap<Uuid, Vec<(Uuid, String)>> = HashMap::new();
        let mut key_delegations: HashMap<Uuid, Vec<KeyDelegation>> = HashMap::new();
        let mut role_assignments: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut access_permissions: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut org_membership: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut org_management: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        // Analyze all edges in the graph
        for edge in &graph.edges {
            match &edge.edge_type {
                EdgeType::Trust | EdgeType::Trusts => {
                    trust_relationships
                        .entry(edge.from)
                        .or_insert_with(Vec::new)
                        .push((edge.to, "Trust".to_string()));
                }
                EdgeType::DelegatesKey(delegation) => {
                    key_delegations
                        .entry(edge.from)
                        .or_insert_with(Vec::new)
                        .push(delegation.clone());
                }
                EdgeType::HasRole => {
                    role_assignments
                        .entry(edge.from)
                        .or_insert_with(Vec::new)
                        .push(edge.to);
                }
                EdgeType::HasAccess => {
                    access_permissions
                        .entry(edge.from)
                        .or_insert_with(Vec::new)
                        .push(edge.to);
                }
                EdgeType::MemberOf => {
                    org_membership
                        .entry(edge.to) // unit_id
                        .or_insert_with(Vec::new)
                        .push(edge.from); // person_id
                }
                EdgeType::Manages => {
                    org_management
                        .entry(edge.from) // manager_id
                        .or_insert_with(Vec::new)
                        .push(edge.to); // unit_id
                }
                _ => {}
            }
        }

        RelationshipAnalysis {
            trust_relationships,
            key_delegations,
            role_assignments,
            access_permissions,
            org_membership,
            org_management,
        }
    }

    /// Get all people trusted by a specific person
    pub fn get_trusted_by(&self, person_id: &Uuid) -> Vec<(Uuid, String)> {
        self.trust_relationships
            .get(person_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all key delegations from a specific person
    pub fn get_delegations_from(&self, person_id: &Uuid) -> Vec<KeyDelegation> {
        self.key_delegations
            .get(person_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all roles assigned to a person
    pub fn get_roles_for_person(&self, person_id: &Uuid) -> Vec<Uuid> {
        self.role_assignments
            .get(person_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all people with access to a location
    pub fn get_people_with_access(&self, location_id: &Uuid) -> Vec<Uuid> {
        self.access_permissions
            .iter()
            .filter_map(|(person_id, locations)| {
                if locations.contains(location_id) {
                    Some(*person_id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all members of an organizational unit
    pub fn get_unit_members(&self, unit_id: &Uuid) -> Vec<Uuid> {
        self.org_membership
            .get(unit_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        format!(
            "Relationships: {} trust, {} delegations, {} roles, {} access grants, {} members",
            self.trust_relationships.len(),
            self.key_delegations.len(),
            self.role_assignments.len(),
            self.access_permissions.len(),
            self.org_membership.values().map(|v| v.len()).sum::<usize>()
        )
    }
}
