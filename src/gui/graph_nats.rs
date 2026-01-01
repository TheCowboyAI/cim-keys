//! Graph-First NATS Generation
//!
//! This module implements: **The organizational graph drives NATS infrastructure creation**.
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
use chrono::Utc;

use crate::domain::{Organization, OrganizationUnit, Person, KeyOwnerRole};
use crate::gui::graph::{OrganizationConcept, ConceptEntity, ConceptRelation, EdgeType};
use crate::gui::domain_node::DomainNode;
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

/// Analyze the organizational graph to determine NATS hierarchy
///
/// Uses accessor methods (partial projections) instead of pattern matching
/// on DomainNodeData variants.
pub fn analyze_graph_for_nats(graph: &OrganizationConcept) -> NatsHierarchy {
    let mut hierarchy = NatsHierarchy::new();

    // Step 1: Find the root organization using accessor
    for (id, node) in &graph.nodes {
        if let Some(org) = node.domain_node.organization() {
            hierarchy.root_organization = Some((*id, org.clone()));
            break; // Assume single root organization
        }
    }

    // Step 2: Find all organizational units and their parent organization
    if let Some((org_id, _)) = &hierarchy.root_organization {
        for (unit_id, node) in &graph.nodes {
            if let Some(unit) = node.domain_node.organization_unit() {
                // Find parent edge (Organization → OrganizationalUnit) using O(1) lookup
                let parent_id = graph.edges_to(*unit_id)
                    .iter()
                    .find(|edge| {
                        edge.edge_type == EdgeType::ParentChild || edge.edge_type == EdgeType::ManagesUnit
                    })
                    .map(|edge| edge.from)
                    .unwrap_or(*org_id); // Default to root org if no parent found

                hierarchy.nats_accounts.insert(*unit_id, (unit.clone(), parent_id));
                hierarchy.hierarchy_edges.push((parent_id, *unit_id));
            }
        }
    }

    // Step 3: Find all people and their parent organizational unit
    for (person_id, node) in &graph.nodes {
        if let Some((person, role)) = node.domain_node.person_with_role() {
            // Find parent edge (OrganizationalUnit → Person or Organization → Person) using O(1) lookup
            let parent_id = graph.edges_to(*person_id)
                .iter()
                .find(|edge| edge.edge_type == EdgeType::MemberOf)
                .map(|edge| edge.from)
                .or_else(|| hierarchy.root_organization.as_ref().map(|(id, _)| *id));

            if let Some(parent) = parent_id {
                hierarchy.nats_users.insert(*person_id, (person.clone(), *role, parent));
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
    // Create a simplified identity projection for visualization
    // TODO: Generate actual NKeys and JWTs using NatsProjection when NSC integration is complete
    use crate::value_objects::{NKeyPair, NatsJwt, NKeyType, NKeySeed, NKeyPublic};
    use crate::domain_projections::NatsIdentityProjection;

    let nkey = NKeyPair {
        id: Uuid::now_v7(),
        key_type: NKeyType::Operator,
        seed: NKeySeed::new(
            NKeyType::Operator,
            format!("SO_PLACEHOLDER_{}", nats_id), // Placeholder seed
            Utc::now(),
        ),
        public_key: NKeyPublic::new(
            NKeyType::Operator,
            format!("O_PLACEHOLDER_{}", nats_id), // Placeholder public key
        ),
        name: Some(org.name.clone()),
        expires_at: None,
    };

    let jwt = NatsJwt::new(
        NKeyType::Operator,
        format!("OPERATOR_JWT_{}", nats_id), // Placeholder
        nkey.public_key.clone(),
        nkey.public_key.clone(), // Self-signed
        Utc::now(),
        None,
    );

    let identity = NatsIdentityProjection {
        nkey,
        jwt,
        credential: None,
        events: Vec::new(), // US-021: GUI placeholder (events collected during projection)
    };

    let domain_node = DomainNode::inject_nats_operator(identity);
    let position = Point::new(400.0, 100.0); // Top center
    let entity = ConceptEntity::from_domain_node(nats_id, domain_node);
    Ok((entity, position))
}

/// Create NATS Account node from OrganizationalUnit
fn create_nats_account_node(
    nats_id: Uuid,
    unit: &OrganizationUnit,
    _parent_nats_id: &Uuid,
) -> Result<(ConceptEntity, Point), String> {
    use crate::value_objects::{NKeyPair, NatsJwt, NKeyType, NKeySeed, NKeyPublic};
    use crate::domain_projections::NatsIdentityProjection;

    let nkey = NKeyPair {
        id: Uuid::now_v7(),
        key_type: NKeyType::Account,
        seed: NKeySeed::new(
            NKeyType::Account,
            format!("SA_PLACEHOLDER_{}", nats_id),
            Utc::now(),
        ),
        public_key: NKeyPublic::new(
            NKeyType::Account,
            format!("A_PLACEHOLDER_{}", nats_id),
        ),
        name: Some(unit.name.clone()),
        expires_at: None,
    };

    let jwt = NatsJwt::new(
        NKeyType::Account,
        format!("ACCOUNT_JWT_{}", nats_id),
        NKeyPublic::new(
            NKeyType::Operator,
            "O_OPERATOR_PLACEHOLDER".to_string(),
        ),
        nkey.public_key.clone(),
        Utc::now(),
        None,
    );

    let identity = NatsIdentityProjection {
        nkey,
        jwt,
        credential: None,
        events: Vec::new(), // US-021: GUI placeholder (events collected during projection)
    };

    let domain_node = DomainNode::inject_nats_account(identity);
    let position = Point::new(400.0, 250.0); // Middle
    let entity = ConceptEntity::from_domain_node(nats_id, domain_node);
    Ok((entity, position))
}

/// Create NATS User node from Person
fn create_nats_user_node(
    nats_id: Uuid,
    person: &Person,
    _role: &KeyOwnerRole,
    _parent_nats_id: &Uuid,
) -> Result<(ConceptEntity, Point), String> {
    use crate::value_objects::{NKeyPair, NatsJwt, NatsCredential, NKeyType, NKeySeed, NKeyPublic};
    use crate::domain_projections::NatsIdentityProjection;

    let username = person.name.split_whitespace().next().unwrap_or(&person.name).to_lowercase();

    let nkey = NKeyPair {
        id: Uuid::now_v7(),
        key_type: NKeyType::User,
        seed: NKeySeed::new(
            NKeyType::User,
            format!("SU_PLACEHOLDER_{}", nats_id),
            Utc::now(),
        ),
        public_key: NKeyPublic::new(
            NKeyType::User,
            format!("U_PLACEHOLDER_{}", nats_id),
        ),
        name: Some(username.clone()),
        expires_at: None,
    };

    let jwt = NatsJwt::new(
        NKeyType::User,
        format!("USER_JWT_{}", nats_id),
        NKeyPublic::new(
            NKeyType::Account,
            "A_ACCOUNT_PLACEHOLDER".to_string(),
        ),
        nkey.public_key.clone(),
        Utc::now(),
        None,
    );

    let credential = NatsCredential {
        id: Uuid::now_v7(),
        jwt: jwt.clone(),
        seed: nkey.seed.clone(),
        name: Some(username.clone()),
    };

    let identity = NatsIdentityProjection {
        nkey,
        jwt,
        credential: Some(credential),
        events: Vec::new(), // US-021: GUI placeholder (events collected during projection)
    };

    let domain_node = DomainNode::inject_nats_user(identity);
    let position = Point::new(400.0, 400.0); // Bottom
    let entity = ConceptEntity::from_domain_node(nats_id, domain_node);
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
    use std::collections::HashMap;
    use chrono::Utc;

    #[test]
    fn test_analyze_simple_org_for_nats() {
        let mut graph = OrganizationConcept::new();

        // Create organization
        let org = Organization {
            id: Uuid::now_v7(),
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

        let org_id = Uuid::now_v7();
        let unit_id = Uuid::now_v7();
        let _person_id = Uuid::now_v7();

        hierarchy.root_organization = Some((org_id, Organization {
            id: org_id,
            name: "TestOrg".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: Vec::new(),
            metadata: HashMap::new(),
        }));

        hierarchy.nats_accounts.insert(unit_id, (
            OrganizationUnit {
                id: unit_id,
                name: "Engineering".to_string(),
                unit_type: crate::domain::OrganizationUnitType::Department,
                parent_unit_id: None,
                responsible_person_id: None,
            nats_account_name: None,
            },
            org_id,
        ));

        let order = determine_generation_order(&hierarchy);

        assert_eq!(order.len(), 2);
        assert_eq!(order[0], NatsGenerationOrder::Operator(org_id));
        assert_eq!(order[1], NatsGenerationOrder::Account(unit_id));
    }
}
