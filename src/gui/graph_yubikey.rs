//! Graph-First YubiKey Provisioning
//!
//! This module implements the core principle: **The organizational graph drives YubiKey provisioning**.
//!
//! ## Flow
//!
//! 1. User builds organizational graph (Organization → Units → People with roles)
//! 2. System analyzes graph to determine YubiKey assignments
//! 3. System provisions YubiKeys based on person roles:
//!    - Root Authority → Signature slot (9C) for CA signing
//!    - Security Admin → Multiple slots for admin operations
//!    - Developer → Authentication slot (9A) for daily work
//!    - Service Account → Card Auth slot (9E) for automated services
//!    - Backup Holder → Key Management slot (9D) for backup/recovery
//!    - Auditor → Authentication slot (9A) for audit access
//! 4. Keys are generated in appropriate PIV slots
//! 5. Visual feedback shows provisioning status
//!
//! ## Graph Structure → YubiKey Mapping
//!
//! ```text
//! Person "Alice Smith" (role: RootAuthority)
//!   └─> YubiKey Serial: 12345678
//!       └─> PIV Slot 9C (Signature) → Root CA key
//!
//! Person "Bob Jones" (role: Developer)
//!   └─> YubiKey Serial: 87654321
//!       └─> PIV Slot 9A (Authentication) → SSH + TLS keys
//!
//! Person "Carol White" (role: SecurityAdmin)
//!   └─> YubiKey Serial: 11223344
//!       ├─> PIV Slot 9A (Authentication) → Admin access
//!       ├─> PIV Slot 9C (Signature) → Signing operations
//!       └─> PIV Slot 9D (Key Management) → Key escrow
//! ```

use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Person, KeyOwnerRole};
use crate::gui::graph::{OrganizationGraph, NodeType, GraphNode};
use iced::{Color, Point};

/// PIV slot assignments based on role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PIVSlot {
    /// 9A: Authentication (PIV Authentication)
    Authentication,
    /// 9C: Digital Signature (Signing operations)
    DigitalSignature,
    /// 9D: Key Management (Encryption/Decryption)
    KeyManagement,
    /// 9E: Card Authentication (Physical access)
    CardAuthentication,
}

impl PIVSlot {
    pub fn hex(&self) -> &'static str {
        match self {
            PIVSlot::Authentication => "9A",
            PIVSlot::DigitalSignature => "9C",
            PIVSlot::KeyManagement => "9D",
            PIVSlot::CardAuthentication => "9E",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PIVSlot::Authentication => "Authentication (SSH, TLS client auth)",
            PIVSlot::DigitalSignature => "Digital Signature (CA signing, code signing)",
            PIVSlot::KeyManagement => "Key Management (Encryption, key escrow)",
            PIVSlot::CardAuthentication => "Card Authentication (Physical access, service accounts)",
        }
    }
}

/// YubiKey provisioning plan for a person
#[derive(Debug, Clone)]
pub struct YubiKeyProvisionPlan {
    /// Person to provision
    pub person_id: Uuid,
    pub person: Person,
    pub role: KeyOwnerRole,

    /// YubiKey serial number (if detected)
    pub yubikey_serial: Option<String>,

    /// PIV slots to provision
    pub slots: Vec<PIVSlot>,

    /// Whether this YubiKey is already provisioned
    pub already_provisioned: bool,
}

/// Result of analyzing the organizational graph for YubiKey provisioning
#[derive(Debug, Clone)]
pub struct YubiKeyProvisionHierarchy {
    /// People that need YubiKeys, mapped by person ID
    pub provision_plans: HashMap<Uuid, YubiKeyProvisionPlan>,
}

impl YubiKeyProvisionHierarchy {
    pub fn new() -> Self {
        Self {
            provision_plans: HashMap::new(),
        }
    }
}

/// Determine which PIV slots to use based on role
pub fn slots_for_role(role: &KeyOwnerRole) -> Vec<PIVSlot> {
    match role {
        KeyOwnerRole::RootAuthority => {
            // Root CA holder needs signature slot for CA operations
            vec![PIVSlot::DigitalSignature]
        }
        KeyOwnerRole::SecurityAdmin => {
            // Security admins need all slots for comprehensive operations
            vec![
                PIVSlot::Authentication,
                PIVSlot::DigitalSignature,
                PIVSlot::KeyManagement,
            ]
        }
        KeyOwnerRole::Developer => {
            // Developers need authentication for daily work
            vec![PIVSlot::Authentication]
        }
        KeyOwnerRole::ServiceAccount => {
            // Service accounts use card authentication
            vec![PIVSlot::CardAuthentication]
        }
        KeyOwnerRole::BackupHolder => {
            // Backup holders need key management for escrow
            vec![PIVSlot::KeyManagement]
        }
        KeyOwnerRole::Auditor => {
            // Auditors need authentication for read access
            vec![PIVSlot::Authentication]
        }
    }
}

/// Analyze the organizational graph to determine YubiKey provisioning needs
pub fn analyze_graph_for_yubikey(graph: &OrganizationGraph) -> YubiKeyProvisionHierarchy {
    let mut hierarchy = YubiKeyProvisionHierarchy::new();

    // Find all people in the graph and determine their YubiKey needs
    for (person_id, node) in &graph.nodes {
        if let NodeType::Person { person, role } = &node.node_type {
            let slots = slots_for_role(role);

            let plan = YubiKeyProvisionPlan {
                person_id: *person_id,
                person: person.clone(),
                role: *role,
                yubikey_serial: None, // Will be filled in during detection
                slots,
                already_provisioned: false,
            };

            hierarchy.provision_plans.insert(*person_id, plan);
        }
    }

    hierarchy
}

/// Generate YubiKey provisioning commands (placeholder for actual provisioning)
pub fn generate_yubikey_provision_from_graph(
    graph: &OrganizationGraph,
) -> Result<Vec<(GraphNode, Uuid)>, String> {
    // Step 1: Analyze graph structure
    let hierarchy = analyze_graph_for_yubikey(graph);

    if hierarchy.provision_plans.is_empty() {
        return Err("No people found in graph. Add people to the Organization tab first.".to_string());
    }

    // Step 2: Generate provision nodes (visual indicators)
    let mut provision_nodes = Vec::new();

    for (person_id, plan) in hierarchy.provision_plans {
        let provision_node_id = Uuid::now_v7();

        // Create a visual node showing YubiKey status
        let status_label = if plan.already_provisioned {
            format!("✓ {} YubiKey ({})", plan.person.name, plan.role)
        } else {
            format!("⏳ {} YubiKey ({} slots needed)", plan.person.name, plan.slots.len())
        };

        let color = if plan.already_provisioned {
            Color::from_rgb(0.2, 0.8, 0.2) // Green for provisioned
        } else {
            Color::from_rgb(0.9, 0.6, 0.2) // Orange for pending
        };

        let provision_node = GraphNode {
            id: provision_node_id,
            node_type: NodeType::YubiKeyStatus {
                person_id,
                yubikey_serial: plan.yubikey_serial.clone(),
                slots_provisioned: if plan.already_provisioned { plan.slots.clone() } else { vec![] },
                slots_needed: plan.slots.clone(),
            },
            position: Point::new(400.0, 500.0), // Below person nodes
            color,
            label: status_label,
        };

        provision_nodes.push((provision_node, person_id));

        tracing::info!(
            "YubiKey provision plan for {}: {} slots ({})",
            plan.person.name,
            plan.slots.len(),
            plan.role
        );
    }

    Ok(provision_nodes)
}

/// Add YubiKey provision status nodes to the organizational graph
pub fn add_yubikey_status_to_graph(
    graph: &mut OrganizationGraph,
    provision_nodes: Vec<(GraphNode, Uuid)>,
) {
    use crate::gui::graph::{GraphEdge, EdgeType};

    for (provision_node, person_id) in provision_nodes {
        let provision_node_id = provision_node.id;

        // Add provision status node
        graph.nodes.insert(provision_node_id, provision_node);

        // Add "requires" edge from person to YubiKey status
        graph.edges.push(GraphEdge {
            from: person_id,
            to: provision_node_id,
            edge_type: EdgeType::Requires, // Person requires YubiKey
            color: Color::from_rgb(0.6, 0.4, 0.8), // Purple for requirement
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, Person};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_slots_for_role() {
        // Root authority gets signature slot
        let slots = slots_for_role(&KeyOwnerRole::RootAuthority);
        assert_eq!(slots, vec![PIVSlot::DigitalSignature]);

        // Security admin gets all slots
        let slots = slots_for_role(&KeyOwnerRole::SecurityAdmin);
        assert_eq!(slots.len(), 3);
        assert!(slots.contains(&PIVSlot::Authentication));
        assert!(slots.contains(&PIVSlot::DigitalSignature));
        assert!(slots.contains(&PIVSlot::KeyManagement));

        // Developer gets authentication
        let slots = slots_for_role(&KeyOwnerRole::Developer);
        assert_eq!(slots, vec![PIVSlot::Authentication]);
    }

    #[test]
    fn test_analyze_graph_for_yubikey() {
        let mut graph = OrganizationGraph::new();

        // Create organization
        let org_id = Uuid::now_v7();
        let org = Organization {
            id: org_id,
            name: "TestOrg".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };
        graph.add_organization_node(org);

        // Add a developer
        let person = Person {
            id: Uuid::now_v7(),
            name: "Alice Developer".to_string(),
            email: "alice@test.com".to_string(),
            roles: vec![],
            organization_id: org_id,
            unit_ids: vec![],
            created_at: Utc::now(),
            active: true,
            nats_permissions: None,
        };
        graph.add_node(person.clone(), KeyOwnerRole::Developer);

        let hierarchy = analyze_graph_for_yubikey(&graph);

        assert_eq!(hierarchy.provision_plans.len(), 1);
        let plan = hierarchy.provision_plans.values().next().unwrap();
        assert_eq!(plan.person.name, "Alice Developer");
        assert_eq!(plan.slots, vec![PIVSlot::Authentication]);
    }
}
