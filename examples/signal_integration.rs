//! N-ary FRP Signal Integration Examples
//!
//! This module demonstrates how to use signals in real cim-keys workflows,
//! following n-ary FRP axioms for compositional reactive programming.
//!
//! ## Examples Included
//!
//! 1. **Key Generation Workflow** - Event signal composition for PKI operations
//! 2. **Form Input Behavior** - Step signal for organization name input
//! 3. **Signal Vector Operations** - N-ary update function with (Model, Intent)
//! 4. **Event Filtering** - Selecting events by time range
//! 5. **Signal Transformation** - Functor operations (fmap)
//!
//! ## Running Examples
//!
//! ```bash
//! cargo run --example signal_integration
//! ```

use cim_keys::mvi::{Intent, Model, EventIntent, StepValue, ModelSignal, UpdateInputs};
use cim_keys::signals::{Signal, SignalVec2, SignalVector, EventKind, StepKind, Time};

/// Example 1: Key Generation Workflow
///
/// Demonstrates event signals for discrete PKI operations:
/// 1. User clicks "Generate Root CA"
/// 2. System validates prerequisites
/// 3. Port generates certificate
/// 4. Domain records generation event
///
/// All events are discrete occurrences at specific times.
fn example_1_key_generation_workflow() {
    println!("\n=== Example 1: Key Generation Workflow ===\n");

    // Create event signal with discrete PKI operations
    let pki_workflow = EventIntent::event(vec![
        (0.0, Intent::UiGenerateRootCAClicked),
        (1.5, Intent::PortX509RootCAGenerated {
            certificate_pem: "-----BEGIN CERTIFICATE-----\n...".into(),
            private_key_pem: "-----BEGIN PRIVATE KEY-----\n...".into(),
            fingerprint: "SHA256:abc123...".into(),
        }),
        (2.0, Intent::RootCAGenerated {
            certificate_id: "root-ca-001".into(),
            subject: "CN=My Organization Root CA".into(),
        }),
    ]);

    // Query events in specific time ranges
    println!("Events in first 1 second:");
    for (time, intent) in pki_workflow.occurrences(0.0, 1.0) {
        println!("  t={:.1}s: {:?}", time, intent);
    }

    println!("\nEvents in 1-2 second range:");
    for (time, _intent) in pki_workflow.occurrences(1.0, 2.0) {
        println!("  t={:.1}s: Port generated certificate", time);
    }

    println!("\nTotal events: {}", pki_workflow.count(0.0, 3.0));
}

/// Example 2: Form Input Behavior (Step Signal)
///
/// Demonstrates step signals for piecewise-constant form inputs:
/// - Organization name holds value until user types
/// - No artificial "change events" - just the current value
/// - Semantics: ⟦Step T⟧(t) = value at most recent change ≤ t
fn example_2_form_input_behavior() {
    println!("\n=== Example 2: Form Input Behavior (Step Signal) ===\n");

    // Step signal holds current organization name
    let org_name = StepValue::<String>::step("Acme Corporation".into());

    println!("Organization name at t=0.0: {}", org_name.sample(0.0));
    println!("Organization name at t=10.0: {}", org_name.sample(10.0));
    println!("  (Same value - step signal holds until changed)");

    // Update creates new signal (immutable)
    let updated_org_name = org_name.with_value("New Acme Corp".into());

    println!("\nAfter update:");
    println!("  Old signal: {}", org_name.sample(0.0));
    println!("  New signal: {}", updated_org_name.sample(0.0));
    println!("  (Immutable - old signal unchanged)");
}

/// Example 3: Signal Vector Operations (N-ary Update Function)
///
/// Demonstrates signal vectors for n-ary functions:
/// - Update function operates on (Model, Intent) pair
/// - Input: SignalVec2<StepKind, EventKind, Model, Intent>
/// - Output: SignalVec2<StepKind, EventKind, Model, Intent>
///
/// This enables compositional reasoning about the update function.
fn example_3_signal_vector_operations() {
    println!("\n=== Example 3: Signal Vector Operations ===\n");

    // Create model signal (step - holds application state)
    let model = ModelSignal::step(Model::default());

    // Create intent signal (event - discrete user action)
    let intent = EventIntent::event(vec![
        (0.0, Intent::UiGenerateRootCAClicked),
    ]);

    // Combine into signal vector
    let inputs = UpdateInputs::new(model.clone(), intent.clone());

    println!("Signal vector arity: {}", UpdateInputs::arity());
    println!("  (2 signals: Model (Step) + Intent (Event))");

    // Split vector to access individual signals
    let (m, i) = inputs.split();

    println!("\nModel signal at t=0.0:");
    println!("  Tab: {:?}", m.sample(0.0).current_tab);

    println!("\nIntent signal occurrences:");
    for (time, intent) in i.occurrences(0.0, 1.0) {
        println!("  t={:.1}s: {:?}", time, intent);
    }
}

/// Example 4: Event Filtering
///
/// Demonstrates temporal queries on event signals:
/// - Filter events by time range
/// - Count events in period
/// - Get all occurrences up to time t
fn example_4_event_filtering() {
    println!("\n=== Example 4: Event Filtering ===\n");

    // Create event signal with multiple user actions
    let user_actions = EventIntent::event(vec![
        (0.0, Intent::UiCreateDomainClicked),
        (0.5, Intent::UiAddPersonClicked),
        (1.0, Intent::UiAddPersonClicked),
        (1.5, Intent::UiAddPersonClicked),
        (2.0, Intent::UiGenerateRootCAClicked),
        (2.5, Intent::UiGenerateSSHKeysClicked),
    ]);

    println!("Events in first second (0.0-1.0):");
    let first_second = user_actions.occurrences(0.0, 1.0);
    println!("  Count: {}", first_second.len());

    println!("\nEvents in second second (1.0-2.0):");
    let second_second = user_actions.occurrences(1.0, 2.0);
    println!("  Count: {}", second_second.len());

    println!("\nAll events up to t=2.0:");
    let until_two = user_actions.occurrences_until(2.0);
    println!("  Count: {}", until_two.len());

    println!("\nTotal events: {}", user_actions.count(0.0, 3.0));
}

/// Example 5: Signal Transformation (Functor)
///
/// Demonstrates functor operations on signals:
/// - fmap applies function to all values
/// - Preserves signal kind
/// - Follows functor laws:
///   - fmap id = id
///   - fmap (g ∘ f) = fmap g ∘ fmap f
fn example_5_signal_transformation() {
    println!("\n=== Example 5: Signal Transformation (Functor) ===\n");

    // Event signal with integer values
    let numbers = Signal::<EventKind, i32>::event(vec![
        (0.0, 1),
        (1.0, 2),
        (2.0, 3),
    ]);

    println!("Original signal:");
    for (time, value) in numbers.occurrences(0.0, 3.0) {
        println!("  t={:.1}s: {}", time, value);
    }

    // Transform: double each value (fmap consumes, so clone first)
    let doubled = numbers.clone().fmap(|x| x * 2);

    println!("\nDoubled signal (fmap):");
    for (time, value) in doubled.occurrences(0.0, 3.0) {
        println!("  t={:.1}s: {}", time, value);
    }

    // Step signal transformation
    let count = StepValue::<usize>::step(42);

    println!("\nStep signal:");
    println!("  Original: {}", count.sample(0.0));

    // Clone before fmap (fmap consumes self)
    let incremented = count.clone().fmap(|x| x + 1);
    println!("  Incremented: {}", incremented.sample(0.0));
}

/// Example 6: Intent Classification
///
/// Demonstrates signal kind classification on intents:
/// - Event intents (87%): Discrete occurrences
/// - Step intents (13%): Piecewise-constant values
fn example_6_intent_classification() {
    println!("\n=== Example 6: Intent Classification ===\n");

    // Event intents (discrete occurrences)
    let event_intents = vec![
        Intent::UiGenerateRootCAClicked,
        Intent::UiAddPersonClicked,
        Intent::DomainCreated {
            organization_id: "org1".into(),
            organization_name: "Acme".into(),
        },
    ];

    println!("Event intents:");
    for intent in &event_intents {
        println!("  {:?}", intent);
        println!("    is_event: {}", intent.is_event_signal());
        println!("    is_step: {}", intent.is_step_signal());
    }

    // Step intents (piecewise-constant values)
    let step_intents = vec![
        Intent::UiOrganizationNameChanged("Acme Corp".into()),
        Intent::UiPassphraseChanged("secret123".into()),
        Intent::UiPersonNameChanged {
            index: 0,
            name: "John Doe".into(),
        },
    ];

    println!("\nStep intents:");
    for intent in &step_intents {
        println!("  UiOrganizationNameChanged/UiPassphraseChanged/UiPersonNameChanged");
        println!("    is_event: {}", intent.is_event_signal());
        println!("    is_step: {}", intent.is_step_signal());
        break; // Just show first one
    }
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  N-ary FRP Signal Integration Examples for cim-keys      ║");
    println!("║  Following Axioms A1 (Multi-Kinded Signals) & A2 (Vectors)║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    example_1_key_generation_workflow();
    example_2_form_input_behavior();
    example_3_signal_vector_operations();
    example_4_event_filtering();
    example_5_signal_transformation();
    example_6_intent_classification();

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  All examples completed successfully!                      ║");
    println!("║                                                             ║");
    println!("║  Key Takeaways:                                             ║");
    println!("║  1. Event signals: Discrete occurrences (button clicks)    ║");
    println!("║  2. Step signals: Piecewise-constant values (form inputs)  ║");
    println!("║  3. Signal vectors: N-ary functions (Model, Intent) pair   ║");
    println!("║  4. Functor operations: Transform values while preserving  ║");
    println!("║     signal kind                                             ║");
    println!("║  5. Type safety: Cannot mix events and behaviors without   ║");
    println!("║     explicit conversion                                     ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
}
