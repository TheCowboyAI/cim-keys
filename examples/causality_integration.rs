//! Causality Integration Example
//!
//! Demonstrates how to integrate the causality enforcement system with
//! cim-keys domain events to track causal relationships in key management
//! workflows.

use cim_keys::causality::{CausalEvent, CausalChain, CausalityValidator};

// ============================================================================
// Domain Events (Simplified for Example)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum KeyManagementEvent {
    OrganizationCreated { name: String },
    PersonAdded { name: String, org: String },
    RootCAGenerated { org: String },
    IntermediateCAGenerated { org: String, signed_by: String },
    PersonKeyGenerated { person: String, signed_by: String },
    KeyExported { key_id: String, location: String },
}

// ============================================================================
// Example 1: Simple Causal Chain
// ============================================================================

fn example_1_simple_chain() {
    println!("=== Example 1: Simple Causal Chain ===\n");

    // Create events with explicit causal dependencies
    let event1 = CausalEvent::new(KeyManagementEvent::OrganizationCreated {
        name: "CowboyAI".to_string(),
    });
    println!("Event 1: Organization created (no dependencies)");

    std::thread::sleep(std::time::Duration::from_millis(1));
    let event2 = CausalEvent::caused_by(
        KeyManagementEvent::PersonAdded {
            name: "Alice".to_string(),
            org: "CowboyAI".to_string(),
        },
        vec![event1.id()], // Depends on organization being created first
    );
    println!("Event 2: Person added (depends on event 1)");

    std::thread::sleep(std::time::Duration::from_millis(1));
    let event3 = CausalEvent::caused_by(
        KeyManagementEvent::RootCAGenerated {
            org: "CowboyAI".to_string(),
        },
        vec![event1.id()], // Depends on organization existing
    );
    println!("Event 3: Root CA generated (depends on event 1)");

    // Build a validated causal chain
    let chain = CausalChain::new()
        .add(event1)
        .unwrap()
        .add(event2)
        .unwrap()
        .add(event3)
        .unwrap();

    println!("\nChain created with {} events", chain.len());
    println!("Validation: {:?}\n", chain.validate());
}

// ============================================================================
// Example 2: PKI Hierarchy Causality
// ============================================================================

fn example_2_pki_hierarchy() {
    println!("=== Example 2: PKI Hierarchy with Causality ===\n");

    // Organization must exist first
    let org_event = CausalEvent::new(KeyManagementEvent::OrganizationCreated {
        name: "Acme Corp".to_string(),
    });
    println!("Step 1: Organization created");

    std::thread::sleep(std::time::Duration::from_millis(1));

    // Root CA depends on organization
    let root_ca_event = CausalEvent::caused_by(
        KeyManagementEvent::RootCAGenerated {
            org: "Acme Corp".to_string(),
        },
        vec![org_event.id()],
    );
    println!("Step 2: Root CA generated (depends on organization)");

    std::thread::sleep(std::time::Duration::from_millis(1));

    // Intermediate CA depends on root CA
    let intermediate_event = CausalEvent::caused_by(
        KeyManagementEvent::IntermediateCAGenerated {
            org: "Acme Corp".to_string(),
            signed_by: "RootCA".to_string(),
        },
        vec![root_ca_event.id()],
    );
    println!("Step 3: Intermediate CA generated (depends on root CA)");

    std::thread::sleep(std::time::Duration::from_millis(1));

    // Person must be added before generating their key
    let person_event = CausalEvent::caused_by(
        KeyManagementEvent::PersonAdded {
            name: "Bob".to_string(),
            org: "Acme Corp".to_string(),
        },
        vec![org_event.id()],
    );
    println!("Step 4: Person added (depends on organization)");

    std::thread::sleep(std::time::Duration::from_millis(1));

    // Person's key depends on both person existing and intermediate CA
    let person_key_event = CausalEvent::caused_by(
        KeyManagementEvent::PersonKeyGenerated {
            person: "Bob".to_string(),
            signed_by: "IntermediateCA".to_string(),
        },
        vec![person_event.id(), intermediate_event.id()],
    );
    println!("Step 5: Person key generated (depends on person AND intermediate CA)");

    // Build the chain
    let chain = CausalChain::new()
        .add(org_event)
        .unwrap()
        .add(root_ca_event)
        .unwrap()
        .add(intermediate_event)
        .unwrap()
        .add(person_event)
        .unwrap()
        .add(person_key_event)
        .unwrap();

    println!("\nPKI hierarchy chain: {} events", chain.len());

    // Get topological order (respects dependencies)
    if let Some(ordered) = chain.topological_order() {
        println!("\nTopological order:");
        for (i, event) in ordered.iter().enumerate() {
            println!("  {}. {:?}", i + 1, event.data());
        }
    }
    println!();
}

// ============================================================================
// Example 3: Detecting Causality Violations
// ============================================================================

fn example_3_causality_violations() {
    println!("=== Example 3: Detecting Causality Violations ===\n");

    // Try to create a person key before the person exists
    let phantom_person_id = cim_keys::causality::CausalId::new();

    let bad_event = CausalEvent::caused_by(
        KeyManagementEvent::PersonKeyGenerated {
            person: "NonExistent".to_string(),
            signed_by: "SomeCA".to_string(),
        },
        vec![phantom_person_id], // References non-existent event!
    );

    let chain = CausalChain::new();
    let result = chain.try_add(bad_event);

    match result {
        Ok(_) => println!("✅ Event added (unexpected!)"),
        Err((chain, _event, error)) => {
            println!("❌ Causality violation detected!");
            println!("   Error: {}", error);
            println!("   Chain remains valid with {} events", chain.len());
        }
    }
    println!();
}

// ============================================================================
// Example 4: Querying Causal Dependencies
// ============================================================================

fn example_4_dependency_queries() {
    println!("=== Example 4: Querying Causal Dependencies ===\n");

    let org_event = CausalEvent::new(KeyManagementEvent::OrganizationCreated {
        name: "TechCorp".to_string(),
    });
    let org_id = org_event.id();

    std::thread::sleep(std::time::Duration::from_millis(1));

    let person1 = CausalEvent::caused_by(
        KeyManagementEvent::PersonAdded {
            name: "Alice".to_string(),
            org: "TechCorp".to_string(),
        },
        vec![org_id],
    );

    std::thread::sleep(std::time::Duration::from_millis(1));

    let person2 = CausalEvent::caused_by(
        KeyManagementEvent::PersonAdded {
            name: "Bob".to_string(),
            org: "TechCorp".to_string(),
        },
        vec![org_id],
    );

    std::thread::sleep(std::time::Duration::from_millis(1));

    let root_ca = CausalEvent::caused_by(
        KeyManagementEvent::RootCAGenerated {
            org: "TechCorp".to_string(),
        },
        vec![org_id],
    );

    let chain = CausalChain::new()
        .add(org_event)
        .unwrap()
        .add(person1)
        .unwrap()
        .add(person2)
        .unwrap()
        .add(root_ca)
        .unwrap();

    // Find all events that depend on the organization
    let dependents = chain.dependents_of(org_id);
    println!("Events depending on organization creation: {}", dependents.len());
    for event in dependents {
        println!("  - {:?}", event.data());
    }
    println!();
}

// ============================================================================
// Example 5: Workflow State Machine with Causality
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum WorkflowState {
    Initial,
    OrganizationReady,
    PKIInitialized,
    PeopleAdded,
    Complete,
}

fn example_5_workflow_state_machine() {
    println!("=== Example 5: Workflow State Machine with Causality ===\n");

    let mut state = WorkflowState::Initial;
    let mut chain = CausalChain::new();

    // Step 1: Create organization
    let org_event = CausalEvent::new(KeyManagementEvent::OrganizationCreated {
        name: "StartupCo".to_string(),
    });
    chain = chain.add(org_event.clone()).unwrap();
    state = WorkflowState::OrganizationReady;
    println!("State: {:?} -> Organization created", state);

    std::thread::sleep(std::time::Duration::from_millis(1));

    // Step 2: Initialize PKI
    let pki_event = CausalEvent::caused_by(
        KeyManagementEvent::RootCAGenerated {
            org: "StartupCo".to_string(),
        },
        vec![org_event.id()],
    );
    chain = chain.add(pki_event).unwrap();
    state = WorkflowState::PKIInitialized;
    println!("State: {:?} -> PKI initialized", state);

    std::thread::sleep(std::time::Duration::from_millis(1));

    // Step 3: Add people
    let person_event = CausalEvent::caused_by(
        KeyManagementEvent::PersonAdded {
            name: "Charlie".to_string(),
            org: "StartupCo".to_string(),
        },
        vec![org_event.id()],
    );
    chain = chain.add(person_event).unwrap();
    state = WorkflowState::PeopleAdded;
    println!("State: {:?} -> People added", state);

    // Validate entire workflow
    match chain.validate() {
        Ok(()) => {
            state = WorkflowState::Complete;
            println!("\n✅ Workflow complete! All causality constraints satisfied.");
            println!("   Final state: {:?}", state);
            println!("   Total events: {}", chain.len());
        }
        Err(e) => {
            println!("\n❌ Workflow validation failed: {}", e);
        }
    }
    println!();
}

// ============================================================================
// Main Example Runner
// ============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Causality Integration Examples for cim-keys             ║");
    println!("║  Demonstrating causal event tracking and validation      ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    example_1_simple_chain();
    example_2_pki_hierarchy();
    example_3_causality_violations();
    example_4_dependency_queries();
    example_5_workflow_state_machine();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Key Insights:                                            ║");
    println!("║  - Causal dependencies ensure correct event ordering      ║");
    println!("║  - PKI hierarchies naturally map to causal chains         ║");
    println!("║  - Violations are detected before system corruption       ║");
    println!("║  - Dependency queries enable audit and compliance         ║");
    println!("║  - State machines benefit from causality validation       ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
}
