//! Graph-First NATS Generation
//!
//! This module implements: **The organizational graph drives NATS infrastructure creation**.
//!
//! Uses `NatsOperatorSimple`, `NatsAccountSimple`, `NatsUserSimple` types with `LiftableDomain::lift()`.
//!
//! ## Flow
//!
//! 1. User builds organizational graph (Organization → Units → People)
//! 2. System analyzes graph structure to determine NATS hierarchy
//! 3. System generates NATS infrastructure in correct order:
//!    - Organization node → NATS Operator
//!    - OrganizationalUnit nodes → NATS Accounts
//!    - Person nodes → NATS Users
//! 4. Permissions derived from roles in graph
//! 5. NATS entities added as nodes in the graph
//! 6. Visual feedback shows NATS provisioning status
//!
//! ## Graph Structure → NATS Mapping
//!
//! ```text
//! Organization "CowboyAI"
//!   └─> NATS Operator: "CowboyAI"
//!       │
//!       ├─> OrganizationalUnit "Engineering"
//!       │   └─> NATS Account: "Engineering"
//!       │       │
//!       │       ├─> Person "Alice Smith" (role: Developer)
//!       │       │   └─> NATS User: "alice" (permissions: pub/sub)
//!       │       │
//!       │       └─> Person "Bob Jones" (role: Security Admin)
//!       │           └─> NATS User: "bob" (permissions: pub/sub + admin)
//!       │
//!       └─> OrganizationalUnit "Operations"
//!           └─> NATS Account: "Operations"
//!               │
//!               └─> Person "Carol White" (role: Operator)
//!                   └─> NATS User: "carol" (permissions: pub/sub)
//! ```

use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Organization, OrganizationUnit, Person, KeyOwnerRole, Role};
use crate::gui::graph::{OrganizationConcept, ConceptEntity, ConceptRelation, EdgeType};
use crate::lifting::{LiftableDomain, NatsOperatorSimple, NatsAccountSimple, NatsUserSimple};
use iced::{Color, Point};

/// Result of analyzing the organizational graph for NATS generation
#[derive(Debug, Clone)]
pub struct NatsHierarchy {
    /// The organization that will become the NATS Operator
    pub root_organization: Option<(Uuid, Organization)>,

    /// Organizational units that will become NATS Accounts
    /// Map: OrgUnit ID → (OrgUnit, Parent Organization ID)
    pub nats_accounts: HashMap<Uuid, (OrganizationUnit, Uuid)>,

    /// People that will get NATS User credentials
    /// Map: Person ID → (Person, Role, Parent OrgUnit ID or Org ID)
    pub nats_users: HashMap<Uuid, (Person, KeyOwnerRole, Uuid)>,

    /// Hierarchy edges (parent → child relationships)
    pub hierarchy_edges: Vec<(Uuid, Uuid)>,
}

impl NatsHierarchy {
    pub fn new() -> Self {
        Self {
            root_organization: None,
            nats_accounts: HashMap::new(),
            nats_users: HashMap::new(),
            hierarchy_edges: Vec::new(),
        }
    }
}

/// Derive KeyOwnerRole from a Role entity by examining its name.
///
/// Maps the general Role to PKI-specific KeyOwnerRole based on naming conventions.
fn derive_key_owner_role(role: &Role) -> KeyOwnerRole {
    let name_lower = role.name.to_lowercase();

    if name_lower.contains("root") || name_lower.contains("authority") {
        KeyOwnerRole::RootAuthority
    } else if name_lower.contains("security") || name_lower.contains("admin") {
        KeyOwnerRole::SecurityAdmin
    } else if name_lower.contains("service") || name_lower.contains("account") {
        KeyOwnerRole::ServiceAccount
    } else if name_lower.contains("backup") || name_lower.contains("recovery") {
        KeyOwnerRole::BackupHolder
    } else if name_lower.contains("audit") {
        KeyOwnerRole::Auditor
    } else {
        KeyOwnerRole::Developer
    }
}

/// Find the primary KeyOwnerRole for a person by traversing HasRole edges.
fn find_person_key_role(person_id: Uuid, graph: &OrganizationConcept) -> KeyOwnerRole {
    let role_ids: Vec<Uuid> = graph.edges
        .iter()
        .filter(|edge| edge.from == person_id && matches!(edge.edge_type, EdgeType::HasRole { .. }))
        .map(|edge| edge.to)
        .collect();

    let mut best_role = KeyOwnerRole::Developer;
    for role_id in role_ids {
        if let Some(node) = graph.nodes.get(&role_id) {
            if let Some(role) = Role::unlift(&node.lifted_node) {
                let derived = derive_key_owner_role(&role);
                best_role = match (&best_role, &derived) {
                    (_, KeyOwnerRole::RootAuthority) => KeyOwnerRole::RootAuthority,
                    (KeyOwnerRole::RootAuthority, _) => KeyOwnerRole::RootAuthority,
                    (_, KeyOwnerRole::SecurityAdmin) => KeyOwnerRole::SecurityAdmin,
                    (KeyOwnerRole::SecurityAdmin, _) => KeyOwnerRole::SecurityAdmin,
                    _ => derived,
                };
            }
        }
    }
    best_role
}

/// Analyze the organizational graph to determine NATS hierarchy
///
/// Uses LiftableDomain::unlift() to recover domain types from lifted nodes.
/// Traverses HasRole edges to determine KeyOwnerRole for each person.
pub fn analyze_graph_for_nats(graph: &OrganizationConcept) -> NatsHierarchy {
    let mut hierarchy = NatsHierarchy::new();

    // Step 1: Find the root organization using LiftableDomain::unlift
    for (id, node) in &graph.nodes {
        if let Some(org) = Organization::unlift(&node.lifted_node) {
            hierarchy.root_organization = Some((*id, org));
            break; // Assume single root organization
        }
    }

    // Step 2: Find all organizational units and their parent organization
    if let Some((org_id, _)) = &hierarchy.root_organization {
        for (unit_id, node) in &graph.nodes {
            if let Some(unit) = OrganizationUnit::unlift(&node.lifted_node) {
                // Find parent edge (Organization → OrganizationalUnit)
                let parent_id = graph.edges_to(*unit_id)
                    .iter()
                    .find(|edge| {
                        edge.edge_type == EdgeType::ParentChild || edge.edge_type == EdgeType::ManagesUnit
                    })
                    .map(|edge| edge.from)
                    .unwrap_or(*org_id);

                hierarchy.nats_accounts.insert(*unit_id, (unit, parent_id));
                hierarchy.hierarchy_edges.push((parent_id, *unit_id));
            }
        }
    }

    // Step 3: Find all people and their parent organizational unit
    // Traverse HasRole edges to determine KeyOwnerRole
    for (person_id, node) in &graph.nodes {
        if let Some(person) = Person::unlift(&node.lifted_node) {
            let parent_id = graph.edges_to(*person_id)
                .iter()
                .find(|edge| edge.edge_type == EdgeType::MemberOf)
                .map(|edge| edge.from)
                .or_else(|| hierarchy.root_organization.as_ref().map(|(id, _)| *id));

            if let Some(parent) = parent_id {
                let role = find_person_key_role(*person_id, graph);
                hierarchy.nats_users.insert(*person_id, (person, role, parent));
                hierarchy.hierarchy_edges.push((parent, *person_id));
            }
        }
    }

    hierarchy
}

/// NATS entity generation order (for dependency resolution)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NatsGenerationOrder {
    Operator(Uuid),      // Organization → NATS Operator
    Account(Uuid),       // OrgUnit → NATS Account
    User(Uuid),          // Person → NATS User
}

/// Determine the order in which NATS entities should be generated
/// (Operator must be generated before Accounts, which must be generated before Users)
pub fn determine_generation_order(hierarchy: &NatsHierarchy) -> Vec<NatsGenerationOrder> {
    let mut order = Vec::new();

    // 1. NATS Operator first
    if let Some((org_id, _)) = &hierarchy.root_organization {
        order.push(NatsGenerationOrder::Operator(*org_id));
    }

    // 2. NATS Accounts (one per organizational unit)
    for (unit_id, _) in &hierarchy.nats_accounts {
        order.push(NatsGenerationOrder::Account(*unit_id));
    }

    // 3. NATS Users last
    for (person_id, _) in &hierarchy.nats_users {
        order.push(NatsGenerationOrder::User(*person_id));
    }

    order
}

/// Generate NATS hierarchy from organizational graph
/// Returns list of (NATS node, position, parent NATS entity ID) to add to graph
pub fn generate_nats_from_graph(
    graph: &OrganizationConcept,
) -> Result<Vec<(ConceptEntity, Point, Option<Uuid>)>, String> {
    // Step 1: Analyze graph structure
    let hierarchy = analyze_graph_for_nats(graph);

    if hierarchy.root_organization.is_none() {
        return Err("No organization found in graph. Create an organization node first.".to_string());
    }

    // Step 2: Determine generation order
    let generation_order = determine_generation_order(&hierarchy);

    // Step 3: Generate NATS entities in order
    let mut nats_nodes = Vec::new();
    let mut nats_id_map: HashMap<Uuid, Uuid> = HashMap::new(); // Entity ID → NATS Node ID

    for order_item in generation_order {
        match order_item {
            NatsGenerationOrder::Operator(org_id) => {
                if let Some((_, org)) = &hierarchy.root_organization {
                    let nats_id = Uuid::now_v7();
                    let (nats_node, position) = create_nats_operator_node(nats_id, org)?;

                    nats_id_map.insert(org_id, nats_id);
                    nats_nodes.push((nats_node, position, None)); // Operator has no parent

                    tracing::info!("Generated NATS Operator for organization: {}", org.name);
                }
            }

            NatsGenerationOrder::Account(unit_id) => {
                if let Some((unit, parent_id)) = hierarchy.nats_accounts.get(&unit_id) {
                    // Copy the parent NATS ID to avoid borrow checker issues
                    let parent_nats_id = nats_id_map.get(parent_id).copied();

                    if parent_nats_id.is_none() {
                        return Err(format!(
                            "Parent NATS entity not found for unit: {}. Generate parent first.",
                            unit.name
                        ));
                    }

                    let parent_nats_id = parent_nats_id.unwrap();
                    let nats_id = Uuid::now_v7();
                    let (nats_node, position) = create_nats_account_node(nats_id, unit, &parent_nats_id)?;

                    nats_id_map.insert(unit_id, nats_id);
                    nats_nodes.push((nats_node, position, Some(parent_nats_id)));

                    tracing::info!("Generated NATS Account for unit: {}", unit.name);
                }
            }

            NatsGenerationOrder::User(person_id) => {
                if let Some((person, role, parent_id)) = hierarchy.nats_users.get(&person_id) {
                    // Copy the parent NATS ID to avoid borrow checker issues
                    let parent_nats_id = nats_id_map.get(parent_id).copied();

                    if parent_nats_id.is_none() {
                        return Err(format!(
                            "Parent NATS entity not found for person: {}. Generate parent first.",
                            person.name
                        ));
                    }

                    let parent_nats_id = parent_nats_id.unwrap();
                    let nats_id = Uuid::now_v7();
                    let (nats_node, position) = create_nats_user_node(nats_id, person, role, &parent_nats_id)?;

                    nats_id_map.insert(person_id, nats_id);
                    nats_nodes.push((nats_node, position, Some(parent_nats_id)));

                    tracing::info!("Generated NATS User for person: {}", person.name);
                }
            }
        }
    }

    Ok(nats_nodes)
}

/// Create NATS Operator node from Organization
fn create_nats_operator_node(nats_id: Uuid, org: &Organization) -> Result<(ConceptEntity, Point), String> {
    // Use simplified visualization type instead of full NatsIdentityProjection
    let operator = NatsOperatorSimple::new(
        nats_id,
        org.name.clone(),
        Some(org.id.as_uuid()),
    );

    let position = Point::new(400.0, 100.0); // Top center
    let entity = ConceptEntity::from_lifted_node(operator.lift());
    Ok((entity, position))
}

/// Create NATS Account node from OrganizationalUnit
fn create_nats_account_node(
    nats_id: Uuid,
    unit: &OrganizationUnit,
    _parent_nats_id: &Uuid,
) -> Result<(ConceptEntity, Point), String> {
    // Use simplified visualization type instead of full NatsIdentityProjection
    let account = NatsAccountSimple::new(
        nats_id,
        unit.name.clone(),
        Some(unit.id.as_uuid()),
        false, // Not a system account
    );

    let position = Point::new(400.0, 250.0); // Middle
    let entity = ConceptEntity::from_lifted_node(account.lift());
    Ok((entity, position))
}

/// Create NATS User node from Person
fn create_nats_user_node(
    nats_id: Uuid,
    person: &Person,
    _role: &KeyOwnerRole,
    _parent_nats_id: &Uuid,
) -> Result<(ConceptEntity, Point), String> {
    let username = person.name.split_whitespace().next().unwrap_or(&person.name).to_lowercase();

    // Use simplified visualization type instead of full NatsIdentityProjection
    let user = NatsUserSimple::new(
        nats_id,
        username,
        Some(person.id.as_uuid()),
        "default".to_string(), // Account name (simplified)
    );

    let position = Point::new(400.0, 400.0); // Bottom
    let entity = ConceptEntity::from_lifted_node(user.lift());
    Ok((entity, position))
}

/// Add NATS nodes and edges to the organizational graph
pub fn add_nats_to_graph(
    graph: &mut OrganizationConcept,
    nats_nodes: Vec<(ConceptEntity, Point, Option<Uuid>)>,
) {
    for (nats_node, position, parent_nats_id) in nats_nodes {
        let nats_id = nats_node.id;

        // Create view for the node
        let view = nats_node.create_view(position);

        // Add NATS node and view
        graph.nodes.insert(nats_id, nats_node);
        graph.node_views.insert(nats_id, view);

        // Add "manages" edge from parent NATS entity
        if let Some(parent_id) = parent_nats_id {
            graph.edges.push(ConceptRelation {
                from: parent_id,
                to: nats_id,
                edge_type: EdgeType::ManagesUnit, // NATS hierarchy relationship
                color: Color::from_rgb(0.4, 0.2, 0.8), // Purple for NATS
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, OrganizationUnit};
    use crate::domain::ids::{BootstrapOrgId, UnitId};
    use std::collections::HashMap;

    #[test]
    fn test_analyze_simple_org_for_nats() {
        let mut graph = OrganizationConcept::new();

        // Create organization
        let org = Organization {
            id: BootstrapOrgId::new(),
            name: "TestOrg".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: Vec::new(),
            metadata: HashMap::new(),
        };
        graph.add_organization_node(org.clone());

        let hierarchy = analyze_graph_for_nats(&graph);

        assert!(hierarchy.root_organization.is_some());
        assert_eq!(hierarchy.root_organization.unwrap().1.name, "TestOrg");
    }

    #[test]
    fn test_determine_nats_generation_order() {
        let mut hierarchy = NatsHierarchy::new();

        let org_id = BootstrapOrgId::new();
        let unit_id = UnitId::new();

        let org_uuid = org_id.as_uuid();
        let unit_uuid = unit_id.as_uuid();

        hierarchy.root_organization = Some((org_uuid, Organization {
            id: org_id,
            name: "TestOrg".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: Vec::new(),
            metadata: HashMap::new(),
        }));

        hierarchy.nats_accounts.insert(unit_uuid, (
            OrganizationUnit {
                id: unit_id,
                name: "Engineering".to_string(),
                unit_type: crate::domain::OrganizationUnitType::Department,
                parent_unit_id: None,
                responsible_person_id: None,
            nats_account_name: None,
            },
            org_uuid,
        ));

        let order = determine_generation_order(&hierarchy);

        assert_eq!(order.len(), 2);
        assert_eq!(order[0], NatsGenerationOrder::Operator(org_uuid));
        assert_eq!(order[1], NatsGenerationOrder::Account(unit_uuid));
    }
}
