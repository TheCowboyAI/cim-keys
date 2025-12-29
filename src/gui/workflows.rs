//! Graph Workflows as Signal Pipelines
//!
//! Demonstrates how to compose graph operations into reactive signal pipelines,
//! enabling declarative graph-first workflows where organizational structure
//! drives PKI and NATS infrastructure generation.
//!
//! ## The Pattern
//!
//! ```text
//! Organization Graph (State)
//!       │
//!       ├──→ PKI Workflow Pipeline
//!       │    ├─→ Generate Root CAs
//!       │    ├─→ Generate Intermediate CAs
//!       │    └─→ Generate Leaf Certificates
//!       │
//!       └──→ NATS Workflow Pipeline
//!            ├─→ Create Operators
//!            ├─→ Create Accounts
//!            └─→ Create Users
//!
//! All changes tracked through CausalChain for complete audit trail.
//! ```
//!
//! ## Signal Composition
//!
//! Workflows are composed using signal transformations:
//!
//! ```rust
//! use cim_keys::gui::workflows::*;
//! use cim_keys::signals::Signal;
//!
//! // Organization graph as signal
//! let org_signal = Signal::step(organization_graph);
//!
//! // Transform to PKI hierarchy
//! let pki_signal = org_signal.map(|graph| build_pki_workflow(&graph));
//!
//! // Transform to NATS infrastructure
//! let nats_signal = org_signal.map(|graph| build_nats_workflow(&graph));
//!
//! // Both pipelines react to same organizational changes
//! ```

use crate::signals::{Signal, StepKind, ContinuousKind};
use crate::causality::{CausalChain, CausalEvent};
use crate::gui::graph::{OrganizationGraph, NodeType};
use crate::gui::graph_causality::GraphOperation;
use uuid::Uuid;
use iced::Point;

/// Workflow step result
///
/// Represents the outcome of a workflow step, including:
/// - The graph operations performed (as causal chain)
/// - Optional metadata about the step
#[derive(Clone, Debug)]
pub struct WorkflowStep {
    /// Operations performed in this step
    pub operations: CausalChain<GraphOperation>,

    /// Human-readable description of step
    pub description: String,

    /// Metadata about workflow execution
    pub metadata: WorkflowMetadata,
}

/// Metadata about workflow execution
#[derive(Clone, Debug)]
pub struct WorkflowMetadata {
    /// Number of operations in chain
    pub total_operations: usize,

    /// Workflow execution time estimate (ms)
    pub estimated_duration_ms: u64,
}

/// PKI Workflow Pipeline
///
/// Transforms organizational graph into complete PKI hierarchy:
/// 1. Create root CA for each organization
/// 2. Create intermediate CAs for each organizational unit
/// 3. Create leaf certificates for each person
///
/// Returns a workflow step with causal chain of operations.
///
/// **Note**: This is a demonstration of the pattern. In production,
/// you would also update the graph state and create actual certificate nodes.
pub fn build_pki_workflow(graph: &OrganizationGraph) -> WorkflowStep {
    let mut chain = CausalChain::new();

    // Find organization nodes
    let org_nodes: Vec<_> = graph.nodes.values()
        .filter(|n| matches!(n.node_type, NodeType::Organization(_)))
        .collect();

    for org_node in org_nodes {
        // Step 1: Create Root CA
        let root_ca_id = Uuid::now_v7();
        let root_ca_op = GraphOperation::NodeAdded {
            node_id: root_ca_id,
            node_type_name: "RootCertificate".to_string(),
            label: format!("Root CA - {}", org_node.label),
            reason: format!("PKI root for organization {}", org_node.label),
        };

        let root_ca_event = CausalEvent::new(root_ca_op);
        let root_ca_event_id = root_ca_event.id();
        chain = chain.add(root_ca_event).unwrap();

        // Create edge: Organization → Root CA
        let root_ca_edge_op = GraphOperation::EdgeCreated {
            from: org_node.id,
            to: root_ca_id,
            edge_type_name: "ParentChild".to_string(),
            reason: "Organization owns root CA".to_string(),
        };

        let root_ca_edge_event = CausalEvent::caused_by(
            root_ca_edge_op,
            vec![root_ca_event_id],
        );
        let root_ca_edge_event_id = root_ca_edge_event.id();
        chain = chain.add(root_ca_edge_event).unwrap();

        // Step 2: Create Intermediate CAs for organizational units
        let unit_nodes: Vec<_> = graph.nodes.values()
            .filter(|n| matches!(n.node_type, NodeType::OrganizationalUnit(_)))
            .collect();

        for unit_node in &unit_nodes {
            let intermediate_ca_id = Uuid::now_v7();
            let intermediate_ca_op = GraphOperation::NodeAdded {
                node_id: intermediate_ca_id,
                node_type_name: "IntermediateCertificate".to_string(),
                label: format!("Intermediate CA - {}", unit_node.label),
                reason: format!("PKI intermediate for unit {}", unit_node.label),
            };

            let intermediate_ca_event = CausalEvent::caused_by(
                intermediate_ca_op,
                vec![root_ca_edge_event_id],
            );
            let intermediate_ca_event_id = intermediate_ca_event.id();
            chain = chain.add(intermediate_ca_event).unwrap();

            // Create edge: Root CA → Intermediate CA (trust chain)
            let trust_edge_op = GraphOperation::EdgeCreated {
                from: root_ca_id,
                to: intermediate_ca_id,
                edge_type_name: "SignedBy".to_string(),
                reason: "Root CA signs intermediate CA".to_string(),
            };

            let trust_edge_event = CausalEvent::caused_by(
                trust_edge_op,
                vec![intermediate_ca_event_id],
            );
            chain = chain.add(trust_edge_event).unwrap();

            // Step 3: Create Leaf Certificates for people in this unit
            let people_nodes: Vec<_> = graph.nodes.values()
                .filter(|n| matches!(n.node_type, NodeType::Person { .. }))
                .collect();

            for person_node in &people_nodes {
                let leaf_cert_id = Uuid::now_v7();
                let leaf_cert_op = GraphOperation::NodeAdded {
                    node_id: leaf_cert_id,
                    node_type_name: "LeafCertificate".to_string(),
                    label: format!("Cert - {}", person_node.label),
                    reason: format!("Personal certificate for {}", person_node.label),
                };

                let leaf_cert_event = CausalEvent::caused_by(
                    leaf_cert_op,
                    vec![intermediate_ca_event_id],
                );
                let leaf_cert_event_id = leaf_cert_event.id();
                chain = chain.add(leaf_cert_event).unwrap();

                // Create edge: Intermediate CA → Leaf Cert
                let leaf_trust_edge_op = GraphOperation::EdgeCreated {
                    from: intermediate_ca_id,
                    to: leaf_cert_id,
                    edge_type_name: "SignedBy".to_string(),
                    reason: format!("Intermediate CA signs leaf for {}", person_node.label),
                };

                let leaf_trust_edge_event = CausalEvent::caused_by(
                    leaf_trust_edge_op,
                    vec![leaf_cert_event_id],
                );
                chain = chain.add(leaf_trust_edge_event).unwrap();

                // Create edge: Person → Leaf Cert (ownership)
                let ownership_edge_op = GraphOperation::EdgeCreated {
                    from: person_node.id,
                    to: leaf_cert_id,
                    edge_type_name: "IssuedTo".to_string(),
                    reason: format!("{} owns certificate", person_node.label),
                };

                let ownership_edge_event = CausalEvent::caused_by(
                    ownership_edge_op,
                    vec![leaf_cert_event_id],
                );
                chain = chain.add(ownership_edge_event).unwrap();
            }
        }
    }

    let total_operations = chain.len();
    let estimated_duration_ms = (total_operations as u64) * 50; // ~50ms per operation

    WorkflowStep {
        operations: chain,
        description: format!(
            "PKI Workflow: Generated {} operations",
            total_operations
        ),
        metadata: WorkflowMetadata {
            total_operations,
            estimated_duration_ms,
        },
    }
}

/// NATS Workflow Pipeline
///
/// Transforms organizational graph into NATS infrastructure:
/// 1. Create NATS Operator for each organization
/// 2. Create NATS Accounts for each organizational unit
/// 3. Create NATS Users for each person
///
/// Returns a workflow step with causal chain of operations.
pub fn build_nats_workflow(graph: &OrganizationGraph) -> WorkflowStep {
    let mut chain = CausalChain::new();

    // Find organization nodes
    let org_nodes: Vec<_> = graph.nodes.values()
        .filter(|n| matches!(n.node_type, NodeType::Organization(_)))
        .collect();

    for org_node in org_nodes {
        // Step 1: Create NATS Operator
        let operator_id = Uuid::now_v7();
        let operator_op = GraphOperation::NodeAdded {
            node_id: operator_id,
            node_type_name: "NatsOperator".to_string(),
            label: format!("NATS Operator - {}", org_node.label),
            reason: format!("NATS root for organization {}", org_node.label),
        };

        let operator_event = CausalEvent::new(operator_op);
        let operator_event_id = operator_event.id();
        chain = chain.add(operator_event).unwrap();

        // Create edge: Organization → Operator
        let operator_edge_op = GraphOperation::EdgeCreated {
            from: org_node.id,
            to: operator_id,
            edge_type_name: "ParentChild".to_string(),
            reason: "Organization owns NATS operator".to_string(),
        };

        let operator_edge_event = CausalEvent::caused_by(
            operator_edge_op,
            vec![operator_event_id],
        );
        let operator_edge_event_id = operator_edge_event.id();
        chain = chain.add(operator_edge_event).unwrap();

        // Step 2: Create NATS Accounts for organizational units
        let unit_nodes: Vec<_> = graph.nodes.values()
            .filter(|n| matches!(n.node_type, NodeType::OrganizationalUnit(_)))
            .collect();

        for unit_node in &unit_nodes {
            let account_id = Uuid::now_v7();
            let account_op = GraphOperation::NodeAdded {
                node_id: account_id,
                node_type_name: "NatsAccount".to_string(),
                label: format!("NATS Account - {}", unit_node.label),
                reason: format!("NATS account for unit {}", unit_node.label),
            };

            let account_event = CausalEvent::caused_by(
                account_op,
                vec![operator_edge_event_id],
            );
            let account_event_id = account_event.id();
            chain = chain.add(account_event).unwrap();

            // Create edge: Operator → Account
            let account_edge_op = GraphOperation::EdgeCreated {
                from: operator_id,
                to: account_id,
                edge_type_name: "Signs".to_string(),
                reason: "Operator manages account".to_string(),
            };

            let account_edge_event = CausalEvent::caused_by(
                account_edge_op,
                vec![account_event_id],
            );
            chain = chain.add(account_edge_event).unwrap();

            // Step 3: Create NATS Users for people in this unit
            let people_nodes: Vec<_> = graph.nodes.values()
                .filter(|n| matches!(n.node_type, NodeType::Person { .. }))
                .collect();

            for person_node in &people_nodes {
                let user_id = Uuid::now_v7();
                let user_op = GraphOperation::NodeAdded {
                    node_id: user_id,
                    node_type_name: "NatsUser".to_string(),
                    label: format!("NATS User - {}", person_node.label),
                    reason: format!("NATS user for {}", person_node.label),
                };

                let user_event = CausalEvent::caused_by(
                    user_op,
                    vec![account_event_id],
                );
                let user_event_id = user_event.id();
                chain = chain.add(user_event).unwrap();

                // Create edge: Account → User
                let user_edge_op = GraphOperation::EdgeCreated {
                    from: account_id,
                    to: user_id,
                    edge_type_name: "Signs".to_string(),
                    reason: format!("Account manages user {}", person_node.label),
                };

                let user_edge_event = CausalEvent::caused_by(
                    user_edge_op,
                    vec![user_event_id],
                );
                chain = chain.add(user_edge_event).unwrap();

                // Create edge: Person → User (mapping)
                let mapping_edge_op = GraphOperation::EdgeCreated {
                    from: person_node.id,
                    to: user_id,
                    edge_type_name: "MapsToPerson".to_string(),
                    reason: format!("{} owns NATS user", person_node.label),
                };

                let mapping_edge_event = CausalEvent::caused_by(
                    mapping_edge_op,
                    vec![user_event_id],
                );
                chain = chain.add(mapping_edge_event).unwrap();
            }
        }
    }

    let total_operations = chain.len();
    let estimated_duration_ms = (total_operations as u64) * 50; // ~50ms per operation

    WorkflowStep {
        operations: chain,
        description: format!(
            "NATS Workflow: Generated {} operations",
            total_operations
        ),
        metadata: WorkflowMetadata {
            total_operations,
            estimated_duration_ms,
        },
    }
}

/// Combined Infrastructure Workflow
///
/// Runs both PKI and NATS workflows in sequence, showing how
/// organizational structure drives complete infrastructure generation.
///
/// This demonstrates signal composition where multiple workflows
/// are chained together.
pub fn build_complete_infrastructure_workflow(
    graph: &OrganizationGraph,
) -> Vec<WorkflowStep> {
    vec![
        build_pki_workflow(graph),
        build_nats_workflow(graph),
    ]
}

/// Create a signal pipeline for PKI workflow
///
/// Demonstrates how to wrap workflow in a signal for reactive updates.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::workflows::create_pki_pipeline;
/// use cim_keys::gui::graph::OrganizationGraph;
///
/// let org_graph = OrganizationGraph::new();
/// let pki_signal = create_pki_pipeline(org_graph);
///
/// // Sample at different times
/// let step = pki_signal.sample(0.0);
/// println!("{}", step.description);
/// ```
pub fn create_pki_pipeline(
    graph: OrganizationGraph,
) -> Signal<StepKind, WorkflowStep> {
    let workflow_step = build_pki_workflow(&graph);
    Signal::<StepKind, WorkflowStep>::step(workflow_step)
}

/// Create a signal pipeline for NATS workflow
pub fn create_nats_pipeline(
    graph: OrganizationGraph,
) -> Signal<StepKind, WorkflowStep> {
    let workflow_step = build_nats_workflow(&graph);
    Signal::<StepKind, WorkflowStep>::step(workflow_step)
}

/// Animate workflow execution
///
/// Converts a workflow step into a continuous signal that can drive
/// smooth animations of graph changes.
///
/// The signal value represents progress from 0.0 (start) to 1.0 (complete).
pub fn animate_workflow_progress(duration_ms: u64) -> Signal<ContinuousKind, f32> {
    Signal::<ContinuousKind, f32>::continuous(Box::new(move |t| {
        let progress: f32 = ((t as f32) * 1000.0) / duration_ms as f32;
        progress.min(1.0).max(0.0)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Person, Organization, KeyOwnerRole};

    fn test_person(name: &str) -> Person {
        Person {
            id: Uuid::now_v7(),
            name: name.to_string(),
            email: format!("{}@example.com", name.to_lowercase()),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            active: true,
        }
    }

    fn test_org(name: &str) -> Organization {
        Organization {
            id: Uuid::now_v7(),
            name: name.to_string(),
            display_name: name.to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_pki_workflow_empty_graph() {
        let graph = OrganizationGraph::new();
        let step = build_pki_workflow(&graph);

        assert_eq!(step.metadata.total_operations, 0);
    }

    #[test]
    fn test_pki_workflow_with_org() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        graph.add_organization_node(org);

        let step = build_pki_workflow(&graph);

        // Should create: Root CA + 1 edge
        assert!(step.metadata.total_operations >= 2);
        assert!(step.operations.len() > 0);
    }

    #[test]
    fn test_nats_workflow_empty_graph() {
        let graph = OrganizationGraph::new();
        let step = build_nats_workflow(&graph);

        assert_eq!(step.metadata.total_operations, 0);
    }

    #[test]
    fn test_nats_workflow_with_org() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        graph.add_organization_node(org);

        let step = build_nats_workflow(&graph);

        // Should create: NATS Operator + 1 edge
        assert!(step.metadata.total_operations >= 2);
        assert!(step.operations.len() > 0);
    }

    #[test]
    fn test_complete_infrastructure_workflow() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        graph.add_organization_node(org);

        let steps = build_complete_infrastructure_workflow(&graph);

        assert_eq!(steps.len(), 2); // PKI + NATS

        // Verify PKI step
        assert!(steps[0].description.contains("PKI"));
        assert!(steps[0].metadata.total_operations > 0);

        // Verify NATS step
        assert!(steps[1].description.contains("NATS"));
        assert!(steps[1].metadata.total_operations > 0);
    }

    #[test]
    fn test_pki_pipeline_signal() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        graph.add_organization_node(org);

        let signal = create_pki_pipeline(graph);

        // Sample the signal
        let step = signal.sample(0.0);
        assert!(step.description.contains("PKI"));
        assert!(step.metadata.total_operations > 0);
    }

    #[test]
    fn test_nats_pipeline_signal() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        graph.add_organization_node(org);

        let signal = create_nats_pipeline(graph);

        // Sample the signal
        let step = signal.sample(0.0);
        assert!(step.description.contains("NATS"));
        assert!(step.metadata.total_operations > 0);
    }

    #[test]
    fn test_animate_workflow_progress() {
        let progress_signal = animate_workflow_progress(1000); // 1 second

        // Progress at start
        let start = progress_signal.sample(0.0);
        assert_eq!(start, 0.0);

        // Progress at midpoint
        let mid = progress_signal.sample(0.5);
        assert!((mid - 0.5).abs() < 0.01);

        // Progress at end
        let end = progress_signal.sample(1.0);
        assert_eq!(end, 1.0);

        // Progress beyond end (should clamp to 1.0)
        let beyond = progress_signal.sample(2.0);
        assert_eq!(beyond, 1.0);
    }

    #[test]
    fn test_workflow_metadata() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        let person = test_person("Alice");

        graph.add_organization_node(org);
        graph.add_node(person, KeyOwnerRole::Developer);

        let pki_step = build_pki_workflow(&graph);

        // Verify metadata accuracy
        assert_eq!(
            pki_step.metadata.total_operations,
            pki_step.operations.len()
        );

        // Duration should be reasonable
        assert!(pki_step.metadata.estimated_duration_ms > 0);
        assert!(pki_step.metadata.estimated_duration_ms < 100000); // < 100 seconds
    }

    #[test]
    fn test_workflow_causality() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        graph.add_organization_node(org);

        let step = build_pki_workflow(&graph);

        // Verify all operations have causal IDs
        for (i, event) in step.operations.events().iter().enumerate() {
            // All events should have valid IDs
            assert!(event.id().value() > 0);

            // Operations after the first should have dependencies
            if i > 0 {
                assert!(!event.dependencies().is_empty());
            }
        }
    }
}
