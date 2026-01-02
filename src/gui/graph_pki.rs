//! Graph-First PKI Generation
//!
//! This module implements the core principle: **The organizational graph drives PKI creation**.
//!
//! NOTE: This module uses deprecated `DomainNode` types. Migration pending.
//!
//! ## Flow
//!
//! 1. User builds organizational graph (Organization → Units → People)
//! 2. System analyzes graph structure to determine CA hierarchy
//! 3. System generates PKI in correct order:
//!    - Organization node → Root CA
//!    - OrganizationalUnit nodes → Intermediate CAs
//!    - Person nodes → Leaf certificates
//! 4. Trust relationships follow graph edges
//! 5. Certificates are added as nodes in the graph
//! 6. Visual feedback shows PKI status
//!
//! ## Graph Structure → PKI Mapping
//!
//! ```text
//! Organization "CowboyAI"
//!   └─> Root CA: "CN=CowboyAI Root CA"
//!       │
//!       ├─> OrganizationalUnit "Engineering"
//!       │   └─> Intermediate CA: "CN=CowboyAI Engineering CA"
//!       │       │
//!       │       ├─> Person "Alice Smith" (role: Developer)
//!       │       │   └─> Leaf Cert: "CN=Alice Smith, O=CowboyAI, OU=Engineering"
//!       │       │
//!       │       └─> Person "Bob Jones" (role: Security Admin)
//!       │           └─> Leaf Cert: "CN=Bob Jones, O=CowboyAI, OU=Engineering"
//!       │
//!       └─> OrganizationalUnit "Operations"
//!           └─> Intermediate CA: "CN=CowboyAI Operations CA"
//!               │
//!               └─> Person "Carol White" (role: Operator)
//!                   └─> Leaf Cert: "CN=Carol White, O=CowboyAI, OU=Operations"
//! ```

#![allow(deprecated)] // Bridge module: Uses deprecated DomainNode for certificate creation

use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::{Organization, OrganizationUnit, Person, KeyOwnerRole, Role};
use crate::gui::graph::{OrganizationConcept, ConceptEntity, ConceptRelation, EdgeType};
use crate::gui::domain_node::DomainNode;
use crate::lifting::LiftableDomain;
use iced::{Color, Point};

/// Result of analyzing the organizational graph for PKI generation
#[derive(Debug, Clone)]
pub struct PkiHierarchy {
    /// The organization that will become the root CA
    pub root_organization: Option<(Uuid, Organization)>,

    /// Organizational units that will become intermediate CAs
    /// Map: OrgUnit ID → (OrgUnit, Parent Organization ID)
    pub intermediate_units: HashMap<Uuid, (OrganizationUnit, Uuid)>,

    /// People that will get leaf certificates
    /// Map: Person ID → (Person, Role, Parent OrgUnit ID or Org ID)
    pub leaf_people: HashMap<Uuid, (Person, KeyOwnerRole, Uuid)>,

    /// Hierarchy edges (parent → child relationships)
    pub hierarchy_edges: Vec<(Uuid, Uuid)>,
}

impl PkiHierarchy {
    pub fn new() -> Self {
        Self {
            root_organization: None,
            intermediate_units: HashMap::new(),
            leaf_people: HashMap::new(),
            hierarchy_edges: Vec::new(),
        }
    }
}

/// Derive KeyOwnerRole from a Role entity by examining its name and claims.
///
/// This maps the general Role to the PKI-specific KeyOwnerRole based on naming conventions:
/// - Names containing "root", "authority" → RootAuthority
/// - Names containing "security", "admin" → SecurityAdmin
/// - Names containing "develop", "engineer" → Developer
/// - Names containing "service", "account" → ServiceAccount
/// - Names containing "backup", "recovery" → BackupHolder
/// - Names containing "audit" → Auditor
/// - Default → Developer
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
        // Default to Developer for developer, engineer, or unknown roles
        KeyOwnerRole::Developer
    }
}

/// Find the primary KeyOwnerRole for a person by traversing HasRole edges.
///
/// Follows edges from Person to Role nodes and derives KeyOwnerRole.
/// Returns the highest-privilege role if multiple roles exist.
fn find_person_key_role(
    person_id: Uuid,
    graph: &OrganizationConcept,
) -> KeyOwnerRole {
    // Find outgoing HasRole edges from this person
    let role_ids: Vec<Uuid> = graph.edges
        .iter()
        .filter(|edge| edge.from == person_id && matches!(edge.edge_type, EdgeType::HasRole { .. }))
        .map(|edge| edge.to)
        .collect();

    // Get roles and derive KeyOwnerRole, return highest privilege
    let mut best_role = KeyOwnerRole::Developer;

    for role_id in role_ids {
        if let Some(node) = graph.nodes.get(&role_id) {
            if let Some(role) = Role::unlift(&node.lifted_node) {
                let derived = derive_key_owner_role(&role);
                // RootAuthority > SecurityAdmin > BackupHolder > Auditor > ServiceAccount > Developer
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

/// Analyze the organizational graph to determine PKI hierarchy
///
/// Uses LiftableDomain::unlift() to recover domain types from lifted nodes.
/// Traverses HasRole edges to determine KeyOwnerRole for each person.
pub fn analyze_graph_for_pki(graph: &OrganizationConcept) -> PkiHierarchy {
    let mut hierarchy = PkiHierarchy::new();

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
                // Find parent edge (Organization → OrganizationalUnit) using O(1) lookup
                let parent_id = graph.edges_to(*unit_id)
                    .iter()
                    .find(|edge| {
                        edge.edge_type == EdgeType::ParentChild || edge.edge_type == EdgeType::ManagesUnit
                    })
                    .map(|edge| edge.from)
                    .unwrap_or(*org_id); // Default to root org if no parent found

                hierarchy.intermediate_units.insert(*unit_id, (unit, parent_id));
                hierarchy.hierarchy_edges.push((parent_id, *unit_id));
            }
        }
    }

    // Step 3: Find all people and their parent organizational unit
    // Traverse HasRole edges to determine KeyOwnerRole
    for (person_id, node) in &graph.nodes {
        if let Some(person) = Person::unlift(&node.lifted_node) {
            // Find parent edge (OrganizationalUnit → Person or Organization → Person)
            let parent_id = graph.edges_to(*person_id)
                .iter()
                .find(|edge| edge.edge_type == EdgeType::MemberOf)
                .map(|edge| edge.from)
                .or_else(|| hierarchy.root_organization.as_ref().map(|(id, _)| *id));

            if let Some(parent) = parent_id {
                // Traverse graph to find role via HasRole edges
                let role = find_person_key_role(*person_id, graph);
                hierarchy.leaf_people.insert(*person_id, (person, role, parent));
                hierarchy.hierarchy_edges.push((parent, *person_id));
            }
        }
    }

    hierarchy
}

/// Certificate generation order (for dependency resolution)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CertificateOrder {
    Root(Uuid),           // Organization → Root CA
    Intermediate(Uuid),   // OrgUnit → Intermediate CA
    Leaf(Uuid),          // Person → Leaf certificate
}

/// Determine the order in which certificates should be generated
/// (Root CA must be generated before Intermediate CAs, which must be generated before Leaf certs)
pub fn determine_generation_order(hierarchy: &PkiHierarchy) -> Vec<CertificateOrder> {
    let mut order = Vec::new();

    // 1. Root CA first
    if let Some((org_id, _)) = &hierarchy.root_organization {
        order.push(CertificateOrder::Root(*org_id));
    }

    // 2. Intermediate CAs (in breadth-first order)
    // For now, simple approach: all intermediate CAs at once
    // TODO: Implement proper topological sort for nested org units
    for (unit_id, _) in &hierarchy.intermediate_units {
        order.push(CertificateOrder::Intermediate(*unit_id));
    }

    // 3. Leaf certificates last
    for (person_id, _) in &hierarchy.leaf_people {
        order.push(CertificateOrder::Leaf(*person_id));
    }

    order
}

/// Generate PKI hierarchy from organizational graph
/// Returns list of (certificate node, position, parent certificate ID) to add to graph
pub fn generate_pki_from_graph(
    graph: &OrganizationConcept,
    _root_passphrase: &str,
) -> Result<Vec<(ConceptEntity, Point, Option<Uuid>)>, String> {
    // Step 1: Analyze graph structure
    let hierarchy = analyze_graph_for_pki(graph);

    if hierarchy.root_organization.is_none() {
        return Err("No organization found in graph. Create an organization node first.".to_string());
    }

    // Step 2: Determine generation order
    let generation_order = determine_generation_order(&hierarchy);

    // Step 3: Generate certificates in order
    let mut certificate_nodes = Vec::new();
    let mut cert_id_map: HashMap<Uuid, Uuid> = HashMap::new(); // Entity ID → Certificate ID

    for order_item in generation_order {
        match order_item {
            CertificateOrder::Root(org_id) => {
                if let Some((_, org)) = &hierarchy.root_organization {
                    let cert_id = Uuid::now_v7();
                    let (cert_node, position) = create_root_ca_node(cert_id, org)?;

                    cert_id_map.insert(org_id, cert_id);
                    certificate_nodes.push((cert_node, position, None)); // Root has no parent

                    tracing::info!("Generated Root CA for organization: {}", org.name);
                }
            }

            CertificateOrder::Intermediate(unit_id) => {
                if let Some((unit, parent_id)) = hierarchy.intermediate_units.get(&unit_id) {
                    // Copy the parent cert ID to avoid borrow checker issues
                    let parent_cert_id = cert_id_map.get(parent_id).copied();

                    if parent_cert_id.is_none() {
                        return Err(format!(
                            "Parent certificate not found for unit: {}. Generate parent first.",
                            unit.name
                        ));
                    }

                    let parent_cert_id = parent_cert_id.unwrap();
                    let cert_id = Uuid::now_v7();
                    let (cert_node, position) = create_intermediate_ca_node(cert_id, unit, &parent_cert_id)?;

                    cert_id_map.insert(unit_id, cert_id);
                    certificate_nodes.push((cert_node, position, Some(parent_cert_id)));

                    tracing::info!("Generated Intermediate CA for unit: {}", unit.name);
                }
            }

            CertificateOrder::Leaf(person_id) => {
                if let Some((person, role, parent_id)) = hierarchy.leaf_people.get(&person_id) {
                    // Copy the parent cert ID to avoid borrow checker issues
                    let parent_cert_id = cert_id_map.get(parent_id).copied();

                    if parent_cert_id.is_none() {
                        return Err(format!(
                            "Parent certificate not found for person: {}. Generate parent first.",
                            person.name
                        ));
                    }

                    let parent_cert_id = parent_cert_id.unwrap();
                    let cert_id = Uuid::now_v7();
                    let (cert_node, position) = create_leaf_certificate_node(cert_id, person, role, &parent_cert_id)?;

                    cert_id_map.insert(person_id, cert_id);
                    certificate_nodes.push((cert_node, position, Some(parent_cert_id)));

                    tracing::info!("Generated leaf certificate for person: {}", person.name);
                }
            }
        }
    }

    Ok(certificate_nodes)
}

/// Create Root CA certificate node from Organization
fn create_root_ca_node(cert_id: Uuid, org: &Organization) -> Result<(ConceptEntity, Point), String> {
    let now = Utc::now();
    let valid_until = now + chrono::Duration::days(3650); // 10 years

    let subject = format!("CN={} Root CA, O={}", org.name, org.name);
    let issuer = subject.clone(); // Self-signed

    let domain_node = DomainNode::inject_root_certificate_uuid(
        cert_id,
        subject,
        issuer,
        now,
        valid_until,
        vec![
            "Certificate Sign".to_string(),
            "CRL Sign".to_string(),
        ],
    );
    let position = Point::new(400.0, 100.0); // Top center
    let entity = ConceptEntity::from_domain_node(cert_id, domain_node);
    Ok((entity, position))
}

/// Create Intermediate CA certificate node from OrganizationalUnit
fn create_intermediate_ca_node(
    cert_id: Uuid,
    unit: &OrganizationUnit,
    parent_cert_id: &Uuid,
) -> Result<(ConceptEntity, Point), String> {
    let now = Utc::now();
    let valid_until = now + chrono::Duration::days(1825); // 5 years

    let subject = format!("CN={} CA, OU={}", unit.name, unit.name);
    let issuer = format!("Parent CA {}", parent_cert_id); // Simplified

    let domain_node = DomainNode::inject_intermediate_certificate_uuid(
        cert_id,
        subject,
        issuer,
        now,
        valid_until,
        vec![
            "Certificate Sign".to_string(),
            "CRL Sign".to_string(),
        ],
    );
    let position = Point::new(400.0, 250.0); // Middle
    let entity = ConceptEntity::from_domain_node(cert_id, domain_node);
    Ok((entity, position))
}

/// Create Leaf certificate node from Person
fn create_leaf_certificate_node(
    cert_id: Uuid,
    person: &Person,
    role: &KeyOwnerRole,
    parent_cert_id: &Uuid,
) -> Result<(ConceptEntity, Point), String> {
    let now = Utc::now();
    let valid_until = now + chrono::Duration::days(365); // 1 year

    let subject = format!("CN={}, emailAddress={}", person.name, person.email);
    let issuer = format!("Parent CA {}", parent_cert_id); // Simplified

    // Determine key usage based on role
    let key_usage = match role {
        KeyOwnerRole::RootAuthority => vec!["Certificate Sign".to_string(), "CRL Sign".to_string()],
        KeyOwnerRole::SecurityAdmin => vec!["Digital Signature".to_string(), "Key Encipherment".to_string(), "Certificate Sign".to_string()],
        KeyOwnerRole::Developer => vec!["Digital Signature".to_string(), "Key Encipherment".to_string()],
        KeyOwnerRole::ServiceAccount => vec!["Digital Signature".to_string()],
        KeyOwnerRole::BackupHolder => vec!["Digital Signature".to_string(), "Key Encipherment".to_string()],
        KeyOwnerRole::Auditor => vec!["Digital Signature".to_string()],
    };

    let domain_node = DomainNode::inject_leaf_certificate_uuid(
        cert_id,
        subject,
        issuer,
        now,
        valid_until,
        key_usage,
        vec![person.email.clone()], // Subject Alternative Name
    );
    let position = Point::new(400.0, 400.0); // Bottom
    let entity = ConceptEntity::from_domain_node(cert_id, domain_node);
    Ok((entity, position))
}

/// Add PKI nodes and edges to the organizational graph
pub fn add_pki_to_graph(
    graph: &mut OrganizationConcept,
    certificate_nodes: Vec<(ConceptEntity, Point, Option<Uuid>)>,
) {
    for (cert_node, position, parent_cert_id) in certificate_nodes {
        let cert_id = cert_node.id;

        // Create view for the node
        let view = cert_node.create_view(position);

        // Add certificate node and view
        graph.nodes.insert(cert_id, cert_node);
        graph.node_views.insert(cert_id, view);

        // Add "signs" edge from parent certificate
        if let Some(parent_id) = parent_cert_id {
            graph.edges.push(ConceptRelation {
                from: parent_id,
                to: cert_id,
                edge_type: EdgeType::Signs, // Certificate signing relationship
                color: Color::from_rgb(0.8, 0.2, 0.2), // Red for trust chain
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
    fn test_analyze_simple_org_structure() {
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

        let hierarchy = analyze_graph_for_pki(&graph);

        assert!(hierarchy.root_organization.is_some());
        assert_eq!(hierarchy.root_organization.unwrap().1.name, "TestOrg");
    }

    #[test]
    fn test_determine_generation_order() {
        let mut hierarchy = PkiHierarchy::new();

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

        hierarchy.intermediate_units.insert(unit_id, (
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
        assert_eq!(order[0], CertificateOrder::Root(org_id));
        assert_eq!(order[1], CertificateOrder::Intermediate(unit_id));
    }
}
