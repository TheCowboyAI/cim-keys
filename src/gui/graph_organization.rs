//! Graph-First Organization Management
//!
//! This module implements: **The organizational graph drives organization-centric views**.
//!
//! NOTE: This module uses deprecated `Injection` types. Migration pending.
//!
//! ## Flow
//!
//! 1. User selects an organization or organizational unit node in the graph
//! 2. System analyzes organization structure and relationships
//! 3. System shows organization-centric view:
//!    - Organization hierarchy (parent/child units)
//!    - Members of organization/unit
//!    - Roles defined in organization
//!    - Policies governing organization
//!    - Resources managed by organization
//! 4. User can perform organization-specific operations
//!
//! ## Organization-Centric Graph Structure
//!
//! ```text
//! Organization "CowboyAI"
//!   ├─> Type: Root Organization
//!   ├─> Display Name: "CowboyAI Inc."
//!   │
//!   ├─> Organizational Units:
//!   │   ├─> Engineering (Department)
//!   │   │   ├─> Backend Team (Team)
//!   │   │   └─> Frontend Team (Team)
//!   │   ├─> Operations (Department)
//!   │   └─> Security (Department)
//!   │
//!   ├─> Members:
//!   │   ├─> Alice Smith (role: Developer)
//!   │   ├─> Bob Jones (role: Security Admin)
//!   │   └─> Carol White (role: Operator)
//!   │
//!   ├─> Roles Defined:
//!   │   ├─> Developer
//!   │   ├─> Security Admin
//!   │   └─> Operator
//!   │
//!   └─> Policies:
//!       ├─> Key Rotation Policy
//!       └─> Access Control Policy
//! ```

#![allow(deprecated)] // Bridge module: Uses deprecated DomainNode accessors, migration pending

use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Organization, OrganizationUnit, Person, KeyOwnerRole};
use crate::gui::graph::{OrganizationConcept, EdgeType};

/// Organization-centric analysis of the organizational graph
#[derive(Debug, Clone)]
pub struct OrganizationAnalysis {
    /// The organization being analyzed
    pub organization_id: Uuid,
    pub organization: Organization,

    /// Direct child organizational units (unit_id -> unit)
    pub child_units: HashMap<Uuid, OrganizationUnit>,

    /// All members directly in this organization (person_id -> (person, role))
    pub direct_members: HashMap<Uuid, (Person, KeyOwnerRole)>,

    /// All members in child units (person_id -> (person, role, unit_id))
    pub indirect_members: HashMap<Uuid, (Person, KeyOwnerRole, Uuid)>,

    /// Roles defined at this organization level
    pub roles: Vec<Uuid>,

    /// Policies governing this organization
    pub policies: Vec<Uuid>,

    /// Resources managed by this organization (locations, keys, etc.)
    pub managed_resources: Vec<(Uuid, String)>, // (resource_id, resource_type)
}

impl OrganizationAnalysis {
    /// Analyze an organization node in the graph
    ///
    /// Uses accessor methods (partial projections) instead of pattern matching
    /// on DomainNodeData variants.
    pub fn analyze(graph: &OrganizationConcept, organization_id: Uuid) -> Option<Self> {
        // Find the organization node using accessor (partial projection)
        let node = graph.nodes.get(&organization_id)?;
        let organization = node.domain_node.organization()?.clone();

        let mut child_units = HashMap::new();
        let mut direct_members = HashMap::new();
        let mut indirect_members = HashMap::new();
        let mut roles = Vec::new();
        let mut policies = Vec::new();
        let mut managed_resources = Vec::new();

        // Analyze edges connected to this organization
        for edge in &graph.edges {
            if edge.from == organization_id {
                // Outgoing edges (things managed BY this organization)
                match &edge.edge_type {
                    EdgeType::ParentChild | EdgeType::ManagesUnit => {
                        // This organization manages a unit
                        if let Some(child_node) = graph.nodes.get(&edge.to) {
                            if let Some(unit) = child_node.domain_node.organization_unit() {
                                child_units.insert(edge.to, unit.clone());
                            }
                        }
                    }
                    EdgeType::MemberOf => {
                        // Direct member of this organization
                        if let Some(member_node) = graph.nodes.get(&edge.from) {
                            if let Some((person, role)) = member_node.domain_node.person_with_role() {
                                direct_members.insert(edge.from, (person.clone(), *role));
                            }
                        }
                    }
                    EdgeType::ManagesResource => {
                        // Resource managed by this organization
                        if let Some(resource_node) = graph.nodes.get(&edge.to) {
                            let resource_type = resource_node.domain_node.injection().display_name();
                            managed_resources.push((edge.to, resource_type.to_string()));
                        }
                    }
                    EdgeType::DefinesRole => {
                        // Role defined at this organization level
                        roles.push(edge.to);
                    }
                    EdgeType::DefinesPolicy => {
                        // Policy governing this organization
                        policies.push(edge.to);
                    }
                    _ => {}
                }
            }
        }

        // Find indirect members (members of child units)
        for (unit_id, _) in &child_units {
            for edge in &graph.edges {
                if edge.to == *unit_id && edge.edge_type == EdgeType::MemberOf {
                    if let Some(member_node) = graph.nodes.get(&edge.from) {
                        if let Some((person, role)) = member_node.domain_node.person_with_role() {
                            indirect_members.insert(
                                edge.from,
                                (person.clone(), *role, *unit_id),
                            );
                        }
                    }
                }
            }
        }

        Some(OrganizationAnalysis {
            organization_id,
            organization,
            child_units,
            direct_members,
            indirect_members,
            roles,
            policies,
            managed_resources,
        })
    }

    /// Get total member count (direct + indirect)
    pub fn total_member_count(&self) -> usize {
        self.direct_members.len() + self.indirect_members.len()
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "{}: {} units, {} members, {} roles, {} policies",
            self.organization.name,
            self.child_units.len(),
            self.total_member_count(),
            self.roles.len(),
            self.policies.len()
        )
    }
}

/// Organizational Unit-centric analysis of the organizational graph
#[derive(Debug, Clone)]
pub struct OrganizationalUnitAnalysis {
    /// The organizational unit being analyzed
    pub unit_id: Uuid,
    pub unit: OrganizationUnit,

    /// Parent organization or unit
    pub parent_id: Option<Uuid>,
    pub parent_type: Option<String>,

    /// Child units (for hierarchical structures)
    pub child_units: HashMap<Uuid, OrganizationUnit>,

    /// Members of this unit (person_id -> (person, role))
    pub members: HashMap<Uuid, (Person, KeyOwnerRole)>,

    /// Responsible person for this unit
    pub responsible_person: Option<(Uuid, Person)>,

    /// Resources managed by this unit
    pub managed_resources: Vec<(Uuid, String)>,
}

impl OrganizationalUnitAnalysis {
    /// Analyze an organizational unit node in the graph
    ///
    /// Uses accessor methods (partial projections) instead of pattern matching
    /// on DomainNodeData variants.
    pub fn analyze(graph: &OrganizationConcept, unit_id: Uuid) -> Option<Self> {
        // Find the unit node using accessor (partial projection)
        let node = graph.nodes.get(&unit_id)?;
        let unit = node.domain_node.organization_unit()?.clone();

        let mut parent_id = None;
        let mut parent_type = None;
        let mut child_units = HashMap::new();
        let mut members = HashMap::new();
        let mut responsible_person = None;
        let mut managed_resources = Vec::new();

        // Analyze edges connected to this unit
        for edge in &graph.edges {
            if edge.to == unit_id {
                // Incoming edges (pointing TO this unit)
                match &edge.edge_type {
                    EdgeType::ParentChild | EdgeType::ManagesUnit => {
                        // Find parent
                        parent_id = Some(edge.from);
                        if let Some(parent_node) = graph.nodes.get(&edge.from) {
                            parent_type = Some(parent_node.domain_node.injection().display_name().to_string());
                        }
                    }
                    EdgeType::MemberOf => {
                        // Member of this unit
                        if let Some(member_node) = graph.nodes.get(&edge.from) {
                            if let Some((person, role)) = member_node.domain_node.person_with_role() {
                                members.insert(edge.from, (person.clone(), *role));
                            }
                        }
                    }
                    EdgeType::ResponsibleFor => {
                        // Person responsible for this unit
                        if let Some(person_node) = graph.nodes.get(&edge.from) {
                            if let Some(person) = person_node.domain_node.person() {
                                responsible_person = Some((edge.from, person.clone()));
                            }
                        }
                    }
                    _ => {}
                }
            } else if edge.from == unit_id {
                // Outgoing edges (pointing FROM this unit)
                match &edge.edge_type {
                    EdgeType::ParentChild | EdgeType::ManagesUnit => {
                        // Child unit
                        if let Some(child_node) = graph.nodes.get(&edge.to) {
                            if let Some(child) = child_node.domain_node.organization_unit() {
                                child_units.insert(edge.to, child.clone());
                            }
                        }
                    }
                    EdgeType::ManagesResource => {
                        // Resource managed by this unit
                        if let Some(resource_node) = graph.nodes.get(&edge.to) {
                            let resource_type = resource_node.domain_node.injection().display_name();
                            managed_resources.push((edge.to, resource_type.to_string()));
                        }
                    }
                    _ => {}
                }
            }
        }

        Some(OrganizationalUnitAnalysis {
            unit_id,
            unit,
            parent_id,
            parent_type,
            child_units,
            members,
            responsible_person,
            managed_resources,
        })
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "{} ({}): {} members, {} child units, {} resources",
            self.unit.name,
            format!("{:?}", self.unit.unit_type),
            self.members.len(),
            self.child_units.len(),
            self.managed_resources.len()
        )
    }
}
