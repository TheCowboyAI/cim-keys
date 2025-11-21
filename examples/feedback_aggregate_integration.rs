//! Feedback Loop Integration with Aggregates
//!
//! Demonstrates how to use feedback combinators (Axiom A8) to implement
//! event-sourced aggregates with causally-sound state evolution.
//!
//! ## Key Pattern
//!
//! Traditional aggregate:
//! ```text
//! mut aggregate.handle(command) -> events
//! mut aggregate.apply(event)    -> ()
//! ```
//!
//! Feedback loop aggregate:
//! ```text
//! feedback(initial_state, |command, state| -> (events, new_state))
//! ```
//!
//! Benefits:
//! - No mutable state
//! - Explicit state transitions
//! - Thread-safe by construction
//! - Easier to test and reason about

use cim_keys::combinators::feedback::{feedback, Decoupled};
use cim_keys::causality::{CausalEvent, CausalId};
use std::collections::HashMap;

// ============================================================================
// Example 1: Simple Key Registry
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
enum KeyCommand {
    RegisterKey { key_id: String, public_key: Vec<u8> },
    RevokeKey { key_id: String },
}

#[derive(Clone, Debug, PartialEq)]
enum KeyEventData {
    KeyRegistered { key_id: String, public_key: Vec<u8> },
    KeyRevoked { key_id: String },
}

#[derive(Clone)]
struct KeyRegistry {
    keys: HashMap<String, Vec<u8>>,
    version: u64,
}

impl Decoupled for KeyRegistry {}

fn example_1_simple_registry() {
    println!("=== Example 1: Simple Key Registry ===\n");

    let mut registry = feedback(
        KeyRegistry {
            keys: HashMap::new(),
            version: 0,
        },
        |command: KeyCommand, state: &KeyRegistry| {
            let mut new_keys = state.keys.clone();
            let event = match command {
                KeyCommand::RegisterKey { key_id, public_key } => {
                    new_keys.insert(key_id.clone(), public_key.clone());
                    KeyEventData::KeyRegistered { key_id, public_key }
                }
                KeyCommand::RevokeKey { key_id } => {
                    new_keys.remove(&key_id);
                    KeyEventData::KeyRevoked { key_id }
                }
            };
            let new_state = KeyRegistry {
                keys: new_keys,
                version: state.version + 1,
            };
            (event, new_state)
        }
    );

    // Process commands through feedback loop
    let event1 = registry.process(KeyCommand::RegisterKey {
        key_id: "key-001".to_string(),
        public_key: vec![1, 2, 3],
    });
    println!("Event 1: {:?}", event1);

    let event2 = registry.process(KeyCommand::RegisterKey {
        key_id: "key-002".to_string(),
        public_key: vec![4, 5, 6],
    });
    println!("Event 2: {:?}", event2);

    let event3 = registry.process(KeyCommand::RevokeKey {
        key_id: "key-001".to_string(),
    });
    println!("Event 3: {:?}", event3);

    let final_state = registry.current_state();
    println!("\nFinal state: version={}, keys={}",
        final_state.version,
        final_state.keys.len()
    );
    println!();
}

// ============================================================================
// Example 2: Aggregate with Validation
// ============================================================================

#[derive(Clone, Debug)]
enum ValidationError {
    KeyAlreadyExists,
    KeyNotFound,
    InvalidKeySize,
}

#[derive(Clone, Debug)]
enum CommandResult {
    Success(KeyEventData),
    Error(ValidationError),
}

#[derive(Clone)]
struct ValidatingRegistry {
    keys: HashMap<String, Vec<u8>>,
    version: u64,
}

impl Decoupled for ValidatingRegistry {}

fn example_2_validating_aggregate() {
    println!("=== Example 2: Aggregate with Validation ===\n");

    let mut registry = feedback(
        ValidatingRegistry {
            keys: HashMap::new(),
            version: 0,
        },
        |command: KeyCommand, state: &ValidatingRegistry| {
            let result = match command {
                KeyCommand::RegisterKey { key_id, public_key } => {
                    // Validation rules
                    if state.keys.contains_key(&key_id) {
                        CommandResult::Error(ValidationError::KeyAlreadyExists)
                    } else if public_key.len() < 32 {
                        CommandResult::Error(ValidationError::InvalidKeySize)
                    } else {
                        CommandResult::Success(KeyEventData::KeyRegistered {
                            key_id,
                            public_key,
                        })
                    }
                }
                KeyCommand::RevokeKey { key_id } => {
                    if !state.keys.contains_key(&key_id) {
                        CommandResult::Error(ValidationError::KeyNotFound)
                    } else {
                        CommandResult::Success(KeyEventData::KeyRevoked { key_id })
                    }
                }
            };

            // Only update state on success
            let new_state = match &result {
                CommandResult::Success(event) => {
                    let mut new_keys = state.keys.clone();
                    match event {
                        KeyEventData::KeyRegistered { key_id, public_key } => {
                            new_keys.insert(key_id.clone(), public_key.clone());
                        }
                        KeyEventData::KeyRevoked { key_id } => {
                            new_keys.remove(key_id);
                        }
                    }
                    ValidatingRegistry {
                        keys: new_keys,
                        version: state.version + 1,
                    }
                }
                CommandResult::Error(_) => state.clone(),
            };

            (result, new_state)
        }
    );

    // Valid command
    let result1 = registry.process(KeyCommand::RegisterKey {
        key_id: "key-001".to_string(),
        public_key: vec![0u8; 32],  // Valid size
    });
    println!("Result 1: {:?}", result1);

    // Duplicate key - should fail
    let result2 = registry.process(KeyCommand::RegisterKey {
        key_id: "key-001".to_string(),
        public_key: vec![0u8; 32],
    });
    println!("Result 2: {:?}", result2);

    // Invalid size - should fail
    let result3 = registry.process(KeyCommand::RegisterKey {
        key_id: "key-002".to_string(),
        public_key: vec![1, 2, 3],  // Too small
    });
    println!("Result 3: {:?}", result3);

    // Revoke non-existent - should fail
    let result4 = registry.process(KeyCommand::RevokeKey {
        key_id: "key-999".to_string(),
    });
    println!("Result 4: {:?}", result4);

    println!("\nFinal version: {}", registry.current_state().version);
    println!();
}

// ============================================================================
// Example 3: Integration with Causality System
// ============================================================================

#[derive(Clone)]
struct CausalRegistry {
    keys: HashMap<String, Vec<u8>>,
    event_history: Vec<CausalId>,
    version: u64,
}

impl Decoupled for CausalRegistry {}

fn example_3_causal_feedback() {
    println!("=== Example 3: Feedback + Causality Integration ===\n");

    let mut registry = feedback(
        CausalRegistry {
            keys: HashMap::new(),
            event_history: vec![],
            version: 0,
        },
        |command: KeyCommand, state: &CausalRegistry| {
            let event_data = match command {
                KeyCommand::RegisterKey { key_id, public_key } => {
                    KeyEventData::KeyRegistered { key_id, public_key }
                }
                KeyCommand::RevokeKey { key_id } => {
                    KeyEventData::KeyRevoked { key_id }
                }
            };

            // Wrap in causal event with dependencies on previous events
            let causal_event = if state.event_history.is_empty() {
                CausalEvent::new(event_data.clone())
            } else {
                CausalEvent::caused_by(event_data.clone(), state.event_history.clone())
            };

            // Update state
            let mut new_keys = state.keys.clone();
            match &event_data {
                KeyEventData::KeyRegistered { key_id, public_key } => {
                    new_keys.insert(key_id.clone(), public_key.clone());
                }
                KeyEventData::KeyRevoked { key_id } => {
                    new_keys.remove(key_id);
                }
            }

            let mut new_history = state.event_history.clone();
            new_history.push(causal_event.id());

            let new_state = CausalRegistry {
                keys: new_keys,
                event_history: new_history,
                version: state.version + 1,
            };

            (causal_event, new_state)
        }
    );

    // Process commands - each depends on previous
    let event1 = registry.process(KeyCommand::RegisterKey {
        key_id: "root-ca".to_string(),
        public_key: vec![0u8; 32],
    });
    println!("Event 1: id={:?}, deps={}", event1.id(), event1.dependencies().len());

    std::thread::sleep(std::time::Duration::from_millis(1));

    let event2 = registry.process(KeyCommand::RegisterKey {
        key_id: "intermediate-ca".to_string(),
        public_key: vec![1u8; 32],
    });
    println!("Event 2: id={:?}, deps={}", event2.id(), event2.dependencies().len());

    std::thread::sleep(std::time::Duration::from_millis(1));

    let event3 = registry.process(KeyCommand::RevokeKey {
        key_id: "root-ca".to_string(),
    });
    println!("Event 3: id={:?}, deps={}", event3.id(), event3.dependencies().len());

    let final_state = registry.current_state();
    println!("\nFinal state:");
    println!("  Version: {}", final_state.version);
    println!("  Keys: {}", final_state.keys.len());
    println!("  Event history: {}", final_state.event_history.len());
    println!();
}

// ============================================================================
// Example 4: Multi-Step Workflow with Feedback
// ============================================================================

#[derive(Clone, Debug)]
enum WorkflowState {
    Idle,
    GeneratingKey,
    KeyGenerated,
    RegisteringKey,
    Complete,
}

#[derive(Clone)]
struct WorkflowAggregate {
    state: WorkflowState,
    current_key_id: Option<String>,
    steps_completed: Vec<String>,
}

impl Decoupled for WorkflowAggregate {}

#[derive(Clone, Debug)]
enum WorkflowCommand {
    StartWorkflow,
    GenerateKey(String),
    RegisterKey,
    Complete,
}

#[derive(Clone, Debug)]
struct WorkflowEvent {
    step: String,
    new_state: WorkflowState,
}

fn example_4_workflow_state_machine() {
    println!("=== Example 4: Workflow State Machine with Feedback ===\n");

    let mut workflow = feedback(
        WorkflowAggregate {
            state: WorkflowState::Idle,
            current_key_id: None,
            steps_completed: vec![],
        },
        |command: WorkflowCommand, state: &WorkflowAggregate| {
            let (new_state_enum, step_name, key_id) = match (&state.state, command) {
                (WorkflowState::Idle, WorkflowCommand::StartWorkflow) => {
                    (WorkflowState::GeneratingKey, "Started workflow", None)
                }
                (WorkflowState::GeneratingKey, WorkflowCommand::GenerateKey(id)) => {
                    (WorkflowState::KeyGenerated, "Generated key", Some(id))
                }
                (WorkflowState::KeyGenerated, WorkflowCommand::RegisterKey) => {
                    (WorkflowState::RegisteringKey, "Registered key", state.current_key_id.clone())
                }
                (WorkflowState::RegisteringKey, WorkflowCommand::Complete) => {
                    (WorkflowState::Complete, "Completed workflow", state.current_key_id.clone())
                }
                _ => {
                    // Invalid transition - stay in same state
                    (state.state.clone(), "Invalid transition", state.current_key_id.clone())
                }
            };

            let mut new_steps = state.steps_completed.clone();
            new_steps.push(step_name.to_string());

            let new_state = WorkflowAggregate {
                state: new_state_enum.clone(),
                current_key_id: key_id,
                steps_completed: new_steps,
            };

            let event = WorkflowEvent {
                step: step_name.to_string(),
                new_state: new_state_enum,
            };

            (event, new_state)
        }
    );

    // Execute workflow
    let events = vec![
        WorkflowCommand::StartWorkflow,
        WorkflowCommand::GenerateKey("key-123".to_string()),
        WorkflowCommand::RegisterKey,
        WorkflowCommand::Complete,
    ];

    for (i, cmd) in events.into_iter().enumerate() {
        let event = workflow.process(cmd);
        println!("Step {}: {:?}", i + 1, event);
    }

    let final_state = workflow.current_state();
    println!("\nWorkflow completed {} steps", final_state.steps_completed.len());
    println!();
}

// ============================================================================
// Example 5: Composing Feedback Loops
// ============================================================================

#[derive(Clone)]
struct Counter {
    count: usize,
}

impl Decoupled for Counter {}

fn example_5_composed_feedback() {
    println!("=== Example 5: Composing Feedback Loops ===\n");

    // First feedback loop: count events
    let counter = feedback(
        Counter { count: 0 },
        |_event: String, state: &Counter| {
            let new_state = Counter { count: state.count + 1 };
            (new_state.count, new_state)
        }
    );

    // Compose with map to create derived view
    let mut with_label = counter.map(|count| {
        format!("Event count: {}", count)
    });

    let result1 = with_label.process("Event 1".to_string());
    println!("{}", result1);

    let result2 = with_label.process("Event 2".to_string());
    println!("{}", result2);

    let result3 = with_label.process("Event 3".to_string());
    println!("{}", result3);

    println!();
}

// ============================================================================
// Main Example Runner
// ============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Feedback Loop Integration with Aggregates               ║");
    println!("║  Demonstrating Axiom A8: Causally-sound state evolution  ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    example_1_simple_registry();
    example_2_validating_aggregate();
    example_3_causal_feedback();
    example_4_workflow_state_machine();
    example_5_composed_feedback();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Key Insights:                                            ║");
    println!("║  - Feedback loops eliminate mutable aggregate state       ║");
    println!("║  - State transitions are explicit and testable            ║");
    println!("║  - Thread-safe by construction (Arc + Mutex)              ║");
    println!("║  - Composable with map and other combinators              ║");
    println!("║  - Integrates seamlessly with causality system            ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
}
