//! Graph Causality - Causal Event Chains for Graph Operations
//!
//! Wraps graph operations in causal chains to track:
//! - Why each node/edge was created (causation)
//! - How operations cascade (correlation)
//! - The complete history of graph transformations
//!
//! ## Key Pattern
//!
//! Graph-first workflows generate cascading events:
//! ```text
//! OrganizationCreated
//!   → OrganizationalUnitCreated (caused by org)
//!   → NatsAccountCreated (caused by unit)
//!   → IntermediateCACertCreated (caused by unit)
//!   → PersonAdded (caused by unit)
//!     → NatsUserCreated (caused by person)
//!     → LeafCertCreated (caused by person)
//! ```

use crate::causality::{CausalChain, CausalEvent};
use crate::gui::graph::{OrganizationGraph, NodeType, EdgeType};
use uuid::Uuid;
use iced::Point;

/// Graph operations that can be tracked causally
#[derive(Clone, Debug, PartialEq)]
pub enum GraphOperation {
    /// Node added to graph
    NodeAdded {
        node_id: Uuid,
        node_type_name: String,
        label: String,
        reason: String,
    },

    /// Edge created between nodes
    EdgeCreated {
        from: Uuid,
        to: Uuid,
        edge_type_name: String,
        reason: String,
    },

    /// Node deleted from graph
    NodeDeleted {
        node_id: Uuid,
        reason: String,
    },

    /// Edge deleted from graph
    EdgeDeleted {
        from: Uuid,
        to: Uuid,
        reason: String,
    },

    /// Node position updated
    PositionUpdated {
        node_id: Uuid,
        old_position: Point,
        new_position: Point,
    },
}

impl GraphOperation {
    /// Get human-readable description of operation
    pub fn description(&self) -> String {
        match self {
            GraphOperation::NodeAdded { node_type_name, label, reason, .. } => {
                format!("Added {} node '{}': {}", node_type_name, label, reason)
            }
            GraphOperation::EdgeCreated { edge_type_name, reason, .. } => {
                format!("Created {} edge: {}", edge_type_name, reason)
            }
            GraphOperation::NodeDeleted { reason, .. } => {
                format!("Deleted node: {}", reason)
            }
            GraphOperation::EdgeDeleted { reason, .. } => {
                format!("Deleted edge: {}", reason)
            }
            GraphOperation::PositionUpdated { node_id, old_position, new_position } => {
                format!("Moved node {}: ({:.0}, {:.0}) → ({:.0}, {:.0})",
                    node_id, old_position.x, old_position.y, new_position.x, new_position.y)
            }
        }
    }

    /// Get affected node IDs
    pub fn affected_nodes(&self) -> Vec<Uuid> {
        match self {
            GraphOperation::NodeAdded { node_id, .. } => vec![*node_id],
            GraphOperation::EdgeCreated { from, to, .. } => vec![*from, *to],
            GraphOperation::NodeDeleted { node_id, .. } => vec![*node_id],
            GraphOperation::EdgeDeleted { from, to, .. } => vec![*from, *to],
            GraphOperation::PositionUpdated { node_id, .. } => vec![*node_id],
        }
    }
}

/// Helper function to get node type name
fn get_node_type_name(node_type: &NodeType) -> &str {
    match node_type {
        NodeType::Person { .. } => "Person",
        NodeType::Organization(_) => "Organization",
        NodeType::OrganizationalUnit(_) => "OrganizationalUnit",
        NodeType::NatsOperator(_) => "NatsOperator",
        NodeType::NatsAccount(_) => "NatsAccount",
        NodeType::NatsUser(_) => "NatsUser",
        NodeType::NatsServiceAccount(_) => "NatsServiceAccount",
        NodeType::RootCertificate { .. } => "RootCertificate",
        NodeType::IntermediateCertificate { .. } => "IntermediateCertificate",
        NodeType::LeafCertificate { .. } => "LeafCertificate",
        NodeType::YubiKey { .. } => "YubiKey",
        NodeType::PivSlot { .. } => "PivSlot",
        NodeType::YubiKeyStatus { .. } => "YubiKeyStatus",
        NodeType::Location(_) => "Location",
        NodeType::Role(_) => "Role",
        NodeType::Policy(_) => "Policy",
    }
}

/// Helper function to get edge type name
#[allow(dead_code)]
fn get_edge_type_name(edge_type: &EdgeType) -> &str {
    match edge_type {
        EdgeType::ParentChild => "ParentChild",
        EdgeType::ManagesUnit => "ManagesUnit",
        EdgeType::MemberOf => "MemberOf",
        EdgeType::OwnsKey => "OwnsKey",
        EdgeType::DelegatesKey(_) => "DelegatesKey",
        EdgeType::StoredAt => "StoredAt",
        EdgeType::HasRole => "HasRole",
        EdgeType::RoleRequiresPolicy => "RoleRequiresPolicy",
        EdgeType::PolicyGovernsEntity => "PolicyGovernsEntity",
        EdgeType::Trusts => "Trusts",
        EdgeType::CertifiedBy => "CertifiedBy",
        EdgeType::Signs => "Signs",
        EdgeType::BelongsToAccount => "BelongsToAccount",
        EdgeType::MapsToOrgUnit => "MapsToOrgUnit",
        EdgeType::MapsToPerson => "MapsToPerson",
        EdgeType::SignedBy => "SignedBy",
        EdgeType::CertifiesKey => "CertifiesKey",
        EdgeType::IssuedTo => "IssuedTo",
        EdgeType::OwnsYubiKey => "OwnsYubiKey",
        EdgeType::AssignedTo => "AssignedTo",
        EdgeType::HasSlot => "HasSlot",
        EdgeType::StoresKey => "StoresKey",
        EdgeType::LoadedCertificate => "LoadedCertificate",
        EdgeType::Requires => "Requires",
        EdgeType::Hierarchy => "Hierarchy",
        EdgeType::Trust => "Trust",
    }
}

/// Build causal chain for PKI generation from organization graph
///
/// Given an organization structure, generates the complete PKI hierarchy:
/// 1. Root CA (from organization)
/// 2. Intermediate CAs (from organizational units)
/// 3. Leaf certificates (from people)
pub fn build_pki_from_graph(graph: &OrganizationGraph) -> CausalChain<GraphOperation> {
    let mut chain = CausalChain::new();

    // Step 1: Find organization root nodes
    let org_nodes: Vec<_> = graph.nodes.values()
        .filter(|n| matches!(n.node_type, NodeType::Organization(_)))
        .collect();

    for org_node in org_nodes {
        // Create root CA for organization
        let root_ca_id = Uuid::now_v7();
        let root_ca_label = format!("{} Root CA", org_node.label);

        let root_ca_op = GraphOperation::NodeAdded {
            node_id: root_ca_id,
            node_type_name: "RootCertificate".to_string(),
            label: root_ca_label.clone(),
            reason: format!("Root CA for organization {}", org_node.label),
        };

        let root_ca_event = CausalEvent::new(root_ca_op);
        let root_ca_event_id = root_ca_event.id();
        chain = chain.add(root_ca_event).unwrap();

        // Create edge from org to root CA
        let root_ca_edge_op = GraphOperation::EdgeCreated {
            from: org_node.id,
            to: root_ca_id,
            edge_type_name: "IssuedTo".to_string(),
            reason: "Root CA issued to organization".to_string(),
        };

        let root_ca_edge_event = CausalEvent::caused_by(root_ca_edge_op, vec![root_ca_event_id]);
        let root_ca_edge_event_id = root_ca_edge_event.id();
        chain = chain.add(root_ca_edge_event).unwrap();

        // Step 2: Find organizational units under this org
        let unit_nodes: Vec<_> = graph.edges.iter()
            .filter(|e| e.from == org_node.id && e.edge_type == EdgeType::ParentChild)
            .filter_map(|e| graph.nodes.get(&e.to))
            .filter(|n| matches!(n.node_type, NodeType::OrganizationalUnit(_)))
            .collect();

        for unit_node in unit_nodes {
            // Create intermediate CA for unit
            let intermediate_ca_id = Uuid::now_v7();
            let intermediate_ca_label = format!("{} Intermediate CA", unit_node.label);

            let intermediate_ca_op = GraphOperation::NodeAdded {
                node_id: intermediate_ca_id,
                node_type_name: "IntermediateCertificate".to_string(),
                label: intermediate_ca_label.clone(),
                reason: format!("Intermediate CA for unit {}", unit_node.label),
            };

            let intermediate_ca_event = CausalEvent::caused_by(
                intermediate_ca_op,
                vec![root_ca_edge_event_id],
            );
            let intermediate_ca_event_id = intermediate_ca_event.id();
            chain = chain.add(intermediate_ca_event).unwrap();

            // Create edges
            let unit_edge_event = CausalEvent::caused_by(
                GraphOperation::EdgeCreated {
                    from: intermediate_ca_id,
                    to: unit_node.id,
                    edge_type_name: "IssuedTo".to_string(),
                    reason: "Intermediate CA issued to unit".to_string(),
                },
                vec![intermediate_ca_event_id],
            );
            chain = chain.add(unit_edge_event).unwrap();

            let trust_edge_event = CausalEvent::caused_by(
                GraphOperation::EdgeCreated {
                    from: intermediate_ca_id,
                    to: root_ca_id,
                    edge_type_name: "SignedBy".to_string(),
                    reason: "Intermediate CA signed by root".to_string(),
                },
                vec![intermediate_ca_event_id],
            );
            let trust_edge_event_id = trust_edge_event.id();
            chain = chain.add(trust_edge_event).unwrap();

            // Step 3: Find people under this unit
            let people_nodes: Vec<_> = graph.edges.iter()
                .filter(|e| e.from == unit_node.id && e.edge_type == EdgeType::ParentChild)
                .filter_map(|e| graph.nodes.get(&e.to))
                .filter(|n| matches!(n.node_type, NodeType::Person { .. }))
                .collect();

            for person_node in people_nodes {
                // Create leaf certificate for person
                let leaf_cert_id = Uuid::now_v7();
                let leaf_cert_label = format!("{} Certificate", person_node.label);

                let leaf_cert_op = GraphOperation::NodeAdded {
                    node_id: leaf_cert_id,
                    node_type_name: "LeafCertificate".to_string(),
                    label: leaf_cert_label.clone(),
                    reason: format!("Leaf certificate for {}", person_node.label),
                };

                let leaf_cert_event = CausalEvent::caused_by(
                    leaf_cert_op,
                    vec![trust_edge_event_id],
                );
                let leaf_cert_event_id = leaf_cert_event.id();
                chain = chain.add(leaf_cert_event).unwrap();

                // Create edges
                let person_edge_event = CausalEvent::caused_by(
                    GraphOperation::EdgeCreated {
                        from: leaf_cert_id,
                        to: person_node.id,
                        edge_type_name: "IssuedTo".to_string(),
                        reason: "Leaf certificate issued to person".to_string(),
                    },
                    vec![leaf_cert_event_id],
                );
                chain = chain.add(person_edge_event).unwrap();

                let cert_edge_event = CausalEvent::caused_by(
                    GraphOperation::EdgeCreated {
                        from: leaf_cert_id,
                        to: intermediate_ca_id,
                        edge_type_name: "SignedBy".to_string(),
                        reason: "Leaf signed by intermediate".to_string(),
                    },
                    vec![leaf_cert_event_id],
                );
                chain = chain.add(cert_edge_event).unwrap();
            }
        }
    }

    chain
}

/// Build causal chain for NATS infrastructure from organization graph
///
/// Given an organization structure, generates NATS hierarchy:
/// 1. NATS Operator (from organization)
/// 2. NATS Accounts (from organizational units)
/// 3. NATS Users (from people)
pub fn build_nats_from_graph(graph: &OrganizationGraph) -> CausalChain<GraphOperation> {
    let mut chain = CausalChain::new();

    // Find organization root nodes
    let org_nodes: Vec<_> = graph.nodes.values()
        .filter(|n| matches!(n.node_type, NodeType::Organization(_)))
        .collect();

    for org_node in org_nodes {
        // Create NATS operator for organization
        let operator_id = Uuid::now_v7();
        let operator_label = format!("{} NATS Operator", org_node.label);

        let operator_op = GraphOperation::NodeAdded {
            node_id: operator_id,
            node_type_name: "NatsOperator".to_string(),
            label: operator_label.clone(),
            reason: format!("NATS operator for organization {}", org_node.label),
        };

        let operator_event = CausalEvent::new(operator_op);
        let operator_event_id = operator_event.id();
        chain = chain.add(operator_event).unwrap();

        let operator_edge_event = CausalEvent::caused_by(
            GraphOperation::EdgeCreated {
                from: operator_id,
                to: org_node.id,
                edge_type_name: "IssuedTo".to_string(),
                reason: "NATS operator issued to organization".to_string(),
            },
            vec![operator_event_id],
        );
        let operator_edge_event_id = operator_edge_event.id();
        chain = chain.add(operator_edge_event).unwrap();

        // Find organizational units
        let unit_nodes: Vec<_> = graph.edges.iter()
            .filter(|e| e.from == org_node.id && e.edge_type == EdgeType::ParentChild)
            .filter_map(|e| graph.nodes.get(&e.to))
            .filter(|n| matches!(n.node_type, NodeType::OrganizationalUnit(_)))
            .collect();

        for unit_node in unit_nodes {
            // Create NATS account for unit
            let account_id = Uuid::now_v7();
            let account_label = format!("{} NATS Account", unit_node.label);

            let account_op = GraphOperation::NodeAdded {
                node_id: account_id,
                node_type_name: "NatsAccount".to_string(),
                label: account_label.clone(),
                reason: format!("NATS account for unit {}", unit_node.label),
            };

            let account_event = CausalEvent::caused_by(
                account_op,
                vec![operator_edge_event_id],
            );
            let account_event_id = account_event.id();
            chain = chain.add(account_event).unwrap();

            // Create edges
            let unit_account_edge = CausalEvent::caused_by(
                GraphOperation::EdgeCreated {
                    from: account_id,
                    to: unit_node.id,
                    edge_type_name: "MapsToOrgUnit".to_string(),
                    reason: "NATS account maps to organizational unit".to_string(),
                },
                vec![account_event_id],
            );
            chain = chain.add(unit_account_edge).unwrap();

            let operator_account_edge = CausalEvent::caused_by(
                GraphOperation::EdgeCreated {
                    from: operator_id,
                    to: account_id,
                    edge_type_name: "Signs".to_string(),
                    reason: "Operator signs account JWT".to_string(),
                },
                vec![account_event_id],
            );
            let operator_account_edge_id = operator_account_edge.id();
            chain = chain.add(operator_account_edge).unwrap();

            // Find people under this unit
            let people_nodes: Vec<_> = graph.edges.iter()
                .filter(|e| e.from == unit_node.id && e.edge_type == EdgeType::ParentChild)
                .filter_map(|e| graph.nodes.get(&e.to))
                .filter(|n| matches!(n.node_type, NodeType::Person { .. }))
                .collect();

            for person_node in people_nodes {
                // Create NATS user for person
                let user_id = Uuid::now_v7();
                let user_label = format!("{} NATS User", person_node.label);

                let user_op = GraphOperation::NodeAdded {
                    node_id: user_id,
                    node_type_name: "NatsUser".to_string(),
                    label: user_label.clone(),
                    reason: format!("NATS user for {}", person_node.label),
                };

                let user_event = CausalEvent::caused_by(
                    user_op,
                    vec![operator_account_edge_id],
                );
                let user_event_id = user_event.id();
                chain = chain.add(user_event).unwrap();

                // Create edges
                let person_user_edge = CausalEvent::caused_by(
                    GraphOperation::EdgeCreated {
                        from: user_id,
                        to: person_node.id,
                        edge_type_name: "MapsToPerson".to_string(),
                        reason: "NATS user maps to person".to_string(),
                    },
                    vec![user_event_id],
                );
                chain = chain.add(person_user_edge).unwrap();

                let account_user_edge = CausalEvent::caused_by(
                    GraphOperation::EdgeCreated {
                        from: account_id,
                        to: user_id,
                        edge_type_name: "Signs".to_string(),
                        reason: "Account signs user JWT".to_string(),
                    },
                    vec![user_event_id],
                );
                chain = chain.add(account_user_edge).unwrap();

                let belongs_edge = CausalEvent::caused_by(
                    GraphOperation::EdgeCreated {
                        from: user_id,
                        to: account_id,
                        edge_type_name: "BelongsToAccount".to_string(),
                        reason: "User belongs to account".to_string(),
                    },
                    vec![user_event_id],
                );
                chain = chain.add(belongs_edge).unwrap();
            }
        }
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, OrganizationUnit, OrganizationUnitType, Person, KeyOwnerRole};

    fn create_test_graph() -> OrganizationGraph {
        let mut graph = OrganizationGraph::new();

        // Create org
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Acme Corp".to_string(),
            display_name: "Acme Corporation".to_string(),
            description: Some("Test organization".to_string()),
            parent_id: None,
            units: vec![],
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        let org_id = org.id;
        graph.add_organization_node(org);

        // Create unit
        let unit = OrganizationUnit {
            id: Uuid::now_v7(),
            name: "Engineering".to_string(),
            unit_type: OrganizationUnitType::Department,
            parent_unit_id: None,
            responsible_person_id: None,
        };
        let unit_id = unit.id;
        graph.add_org_unit_node(unit);

        // Create person
        let person = Person {
            id: Uuid::now_v7(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            roles: vec![],
            organization_id: org_id,
            unit_ids: vec![unit_id],
            created_at: chrono::Utc::now(),
            active: true,
        };
        let person_id = person.id;
        graph.add_node(person, KeyOwnerRole::RootAuthority);

        // Add hierarchy edges
        graph.add_edge(org_id, unit_id, EdgeType::ParentChild);
        graph.add_edge(unit_id, person_id, EdgeType::ParentChild);

        graph
    }

    #[test]
    fn test_pki_causal_chain() {
        let graph = create_test_graph();

        // Generate PKI chain
        let chain = build_pki_from_graph(&graph);

        // Should have:
        // 1 root CA + 1 edge = 2
        // 1 intermediate CA + 2 edges (owns + trusts) = 3
        // 1 leaf cert + 2 edges (owns + certified) = 3
        // Total: 8 operations
        assert!(chain.events().len() >= 8);

        // Verify causality
        let events = chain.events();
        for (i, event) in events.iter().enumerate() {
            if i > 0 {
                // Each event after the first should have dependencies
                assert!(!event.dependencies().is_empty());
            }
        }
    }

    #[test]
    fn test_nats_causal_chain() {
        let graph = create_test_graph();

        // Generate NATS chain
        let chain = build_nats_from_graph(&graph);

        // Should have operator, account, user + edges
        assert!(chain.events().len() >= 6);
    }

    #[test]
    fn test_operation_description() {
        let op = GraphOperation::NodeAdded {
            node_id: Uuid::now_v7(),
            node_type_name: "Person".to_string(),
            label: "Dave".to_string(),
            reason: "New hire".to_string(),
        };

        let desc = op.description();
        assert!(desc.contains("Dave"));
        assert!(desc.contains("New hire"));
    }

    #[test]
    fn test_affected_nodes() {
        let node_id = Uuid::now_v7();
        let op = GraphOperation::NodeAdded {
            node_id,
            node_type_name: "Person".to_string(),
            label: "Eve".to_string(),
            reason: "Test".to_string(),
        };

        let affected = op.affected_nodes();
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0], node_id);
    }
}
