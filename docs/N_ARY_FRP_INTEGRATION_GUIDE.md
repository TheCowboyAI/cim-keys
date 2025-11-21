# N-ary FRP Integration Guide

**How to integrate the n-ary FRP system with cim-keys MVI architecture, GUI, and domain events**

## Table of Contents

1. [Overview](#overview)
2. [MVI Integration](#mvi-integration)
3. [GUI Integration](#gui-integration)
4. [Domain Event Integration](#domain-event-integration)
5. [NATS Integration](#nats-integration)
6. [Complete Workflow Example](#complete-workflow-example)
7. [Best Practices](#best-practices)

---

## Overview

The n-ary FRP system integrates with cim-keys through several layers:

```
┌─────────────────────────────────────────────┐
│  GUI (Iced)                                  │
│  - Emits Intents                            │
│  - Renders Model                            │
└──────────────┬──────────────────────────────┘
               │ Intents
               ▼
┌─────────────────────────────────────────────┐
│  MVI Layer                                   │
│  - Routes Intents → Commands                │
│  - Updates Model (pure functions)           │
└──────────────┬──────────────────────────────┘
               │ Commands
               ▼
┌─────────────────────────────────────────────┐
│  FRP Layer (N-ary)                          │
│  - Signals: Event streams                   │
│  - Routes: Intent → Command pipelines       │
│  - Feedback: Aggregate state evolution      │
│  - Causality: Event ordering                │
└──────────────┬──────────────────────────────┘
               │ Events
               ▼
┌─────────────────────────────────────────────┐
│  Domain Layer                                │
│  - Aggregates                               │
│  - Domain Events                            │
│  - Projections                              │
└──────────────┬──────────────────────────────┘
               │ Publish
               ▼
┌─────────────────────────────────────────────┐
│  NATS (Event Stream)                        │
└─────────────────────────────────────────────┘
```

---

## MVI Integration

### Intent Classification with Signals

```rust
use cim_keys::mvi::Intent;
use cim_keys::signals::{Signal, EventKind};

// Intents are naturally event signals (discrete occurrences)
fn collect_intents(intents: Vec<(f64, Intent)>) -> Signal<EventKind, Intent> {
    Signal::event(intents)
}

// Query intents by time range
fn get_recent_intents(
    signal: &Signal<EventKind, Intent>,
    start: f64,
    end: f64
) -> Vec<Intent> {
    signal.occurrences(start, end)
        .into_iter()
        .map(|(_, intent)| intent)
        .collect()
}
```

### Routing Intents to Handlers

```rust
use cim_keys::routing::{Route, RouteBuilder};
use cim_keys::mvi::{Intent, IntentClass};

// Build intent routing pipeline
fn create_intent_router() -> Route<Intent, ModelUpdate> {
    RouteBuilder::new(|intent: Intent| {
        // Classify intent
        classify_intent(&intent)
    })
    .then(|class| {
        // Route to appropriate handler
        match class {
            IntentClass::Ui => handle_ui_intent(),
            IntentClass::Domain => handle_domain_intent(),
            IntentClass::Port => handle_port_intent(),
            IntentClass::System => handle_system_intent(),
            IntentClass::Error => handle_error(),
        }
    })
    .build()
}

// Use in update function
fn update(model: Model, intent: Intent) -> (Model, Command) {
    let router = create_intent_router();
    let update = router.run(intent);
    apply_update(model, update)
}
```

### Model as Step Signal

```rust
use cim_keys::signals::{Signal, StepKind};

// Model is a step signal (changes discretely)
fn create_model_signal(initial: Model) -> Signal<StepKind, Model> {
    Signal::step(initial)
}

// Update creates new signal
fn update_model_signal(
    signal: Signal<StepKind, Model>,
    new_model: Model
) -> Signal<StepKind, Model> {
    signal.with_value(new_model)
}

// Sample current model
fn current_model(signal: &Signal<StepKind, Model>) -> Model {
    signal.sample(0.0) // Time doesn't matter for step signals
}
```

---

## GUI Integration

### Animation with Continuous Signals

```rust
use cim_keys::signals::continuous::{linear_time, ease_in_out, ContinuousSignal};
use iced::{Application, Command, Element};

struct App {
    animation_start: Option<f64>,
    current_time: f64,
}

impl App {
    fn update_animation(&mut self, delta_time: f64) {
        self.current_time += delta_time;

        if let Some(start) = self.animation_start {
            let elapsed = self.current_time - start;

            // Use continuous signal for smooth animation
            let progress = ease_in_out(linear_time()).sample(elapsed / 2.0);

            // Apply animation (e.g., fade, move, scale)
            self.apply_animation_progress(progress.min(1.0));

            // Stop after 2 seconds
            if elapsed >= 2.0 {
                self.animation_start = None;
            }
        }
    }

    fn apply_animation_progress(&mut self, progress: f64) {
        // Update visual properties based on progress
        // progress goes from 0.0 to 1.0 with ease-in-out curve
    }
}
```

### Event Stream as Signal

```rust
use cim_keys::signals::{Signal, EventKind};
use iced::Event;

// Convert Iced events to signal
fn collect_gui_events(events: Vec<Event>) -> Signal<EventKind, Event> {
    let timed_events: Vec<(f64, Event)> = events
        .into_iter()
        .enumerate()
        .map(|(i, event)| (i as f64, event))
        .collect();

    Signal::event(timed_events)
}
```

### Feedback for GUI State

```rust
use cim_keys::combinators::feedback::{feedback, Decoupled};

#[derive(Clone)]
struct GuiState {
    selected_index: Option<usize>,
    scroll_position: f32,
    search_query: String,
}

impl Decoupled for GuiState {}

enum GuiAction {
    Select(usize),
    Scroll(f32),
    Search(String),
    Clear,
}

fn create_gui_state_machine(initial: GuiState) -> FeedbackLoop<GuiAction, GuiState, GuiState> {
    feedback(initial, |action, state| {
        let new_state = match action {
            GuiAction::Select(idx) => GuiState {
                selected_index: Some(idx),
                ..state.clone()
            },
            GuiAction::Scroll(pos) => GuiState {
                scroll_position: pos,
                ..state.clone()
            },
            GuiAction::Search(query) => GuiState {
                search_query: query,
                ..state.clone()
            },
            GuiAction::Clear => GuiState {
                selected_index: None,
                scroll_position: 0.0,
                search_query: String::new(),
            },
        };

        (new_state.clone(), new_state)
    })
}
```

---

## Domain Event Integration

### Wrapping Domain Events with Causality

```rust
use cim_keys::events::KeyEvent;
use cim_keys::causality::{CausalEvent, CausalChain};
use cim_keys::causality::helpers::{wrap_event, wrap_dependent_event};

// Wrap domain events in causal events
fn process_key_generation_workflow() -> Result<CausalChain<KeyEvent>, CausalityError> {
    let mut chain = CausalChain::new();

    // Step 1: Generate root CA (no dependencies)
    let root_ca_event = wrap_event(KeyEvent::KeyGenerated {
        key_id: generate_id(),
        key_type: KeyType::RootCA,
        // ...
    });
    chain = chain.add(root_ca_event.clone())?;

    // Step 2: Generate intermediate CA (depends on root CA)
    std::thread::sleep(std::time::Duration::from_millis(1));
    let intermediate_event = wrap_dependent_event(
        KeyEvent::KeyGenerated {
            key_id: generate_id(),
            key_type: KeyType::IntermediateCA,
            // ...
        },
        vec![root_ca_event.id()], // Explicit dependency
    );
    chain = chain.add(intermediate_event.clone())?;

    // Step 3: Generate leaf certificate (depends on intermediate)
    std::thread::sleep(std::time::Duration::from_millis(1));
    let leaf_event = wrap_dependent_event(
        KeyEvent::CertificateIssued {
            cert_id: generate_id(),
            // ...
        },
        vec![intermediate_event.id()],
    );
    chain = chain.add(leaf_event)?;

    // Validate causality
    chain.validate()?;

    Ok(chain)
}
```

### Aggregate as Feedback Loop

```rust
use cim_keys::combinators::feedback::{feedback, Decoupled};
use cim_keys::aggregate::KeyManagementAggregate;
use cim_keys::events::KeyEvent;
use cim_keys::commands::KeyCommand;

// Make aggregate state decoupled
#[derive(Clone)]
struct AggregateState {
    version: u64,
    keys: HashMap<Uuid, KeyData>,
    certificates: HashMap<Uuid, Certificate>,
}

impl Decoupled for AggregateState {}

// Create aggregate as feedback loop
fn create_aggregate_feedback(
    initial_state: AggregateState
) -> FeedbackLoop<KeyCommand, Result<Vec<KeyEvent>, DomainError>, AggregateState> {
    feedback(initial_state, |command, state| {
        // Process command with current state
        match process_command(command, state) {
            Ok(events) => {
                // Apply events to create new state
                let new_state = apply_events(state, &events);
                (Ok(events), new_state)
            }
            Err(error) => {
                // Return error, keep state unchanged
                (Err(error), state.clone())
            }
        }
    })
}

fn process_command(
    command: KeyCommand,
    state: &AggregateState
) -> Result<Vec<KeyEvent>, DomainError> {
    // Validate command against current state
    // Generate events
    // Return events (don't modify state here!)
    todo!()
}

fn apply_events(
    state: &AggregateState,
    events: &[KeyEvent]
) -> AggregateState {
    // Fold events into new state
    events.iter().fold(state.clone(), |acc, event| {
        // Apply each event to state
        apply_single_event(acc, event)
    })
}
```

---

## NATS Integration

### Publishing Events to NATS

```rust
use cim_keys::signals::{Signal, EventKind};
use cim_keys::ports::NatsEventPublisher;

// Collect events as signal
let events: Signal<EventKind, KeyEvent> = Signal::event(vec![
    (0.0, event1),
    (1.0, event2),
    (2.0, event3),
]);

// Publish to NATS
async fn publish_events(
    publisher: &NatsEventPublisher,
    events: &Signal<EventKind, KeyEvent>,
    start: f64,
    end: f64,
) -> Result<(), PublishError> {
    for (time, event) in events.occurrences(start, end) {
        publisher.publish(&event).await?;
    }
    Ok(())
}
```

### Routing NATS Subjects

```rust
use cim_keys::routing::Route;

// Route events to NATS subjects
fn create_nats_router() -> Route<KeyEvent, String> {
    Route::new(|event: KeyEvent| {
        match event {
            KeyEvent::KeyGenerated { key_type, .. } => {
                format!("cim.keys.generated.{}", key_type)
            }
            KeyEvent::KeyRevoked { .. } => {
                "cim.keys.revoked".to_string()
            }
            KeyEvent::CertificateIssued { .. } => {
                "cim.keys.certificates.issued".to_string()
            }
            // ... other events
        }
    })
}

// Use router
let subject = create_nats_router().run(event);
publisher.publish_to_subject(&event, &subject).await?;
```

---

## Complete Workflow Example

Here's a complete example integrating all FRP components:

```rust
use cim_keys::signals::{Signal, EventKind, StepKind};
use cim_keys::routing::RouteBuilder;
use cim_keys::causality::CausalChain;
use cim_keys::combinators::feedback::{feedback, Decoupled};
use cim_keys::mvi::Intent;

// === 1. GUI Layer ===

#[derive(Clone)]
struct AppModel {
    keys: Vec<KeyInfo>,
    status: String,
}

enum AppIntent {
    GenerateKey,
    KeyGenerated(KeyInfo),
    Error(String),
}

// === 2. Intent Signal ===

fn collect_intents(intents: Vec<AppIntent>) -> Signal<EventKind, AppIntent> {
    let timed: Vec<(f64, AppIntent)> = intents
        .into_iter()
        .enumerate()
        .map(|(i, intent)| (i as f64, intent))
        .collect();
    Signal::event(timed)
}

// === 3. Intent Router ===

fn create_router() -> Route<AppIntent, KeyCommand> {
    RouteBuilder::new(|intent: AppIntent| {
        match intent {
            AppIntent::GenerateKey => KeyCommand::Generate {
                algorithm: Algorithm::Ed25519,
            },
            _ => KeyCommand::NoOp,
        }
    }).build()
}

// === 4. Aggregate with Feedback ===

#[derive(Clone)]
struct AggregateState {
    keys: Vec<KeyInfo>,
    version: u64,
}

impl Decoupled for AggregateState {}

fn create_aggregate() -> FeedbackLoop<KeyCommand, Vec<KeyEvent>, AggregateState> {
    feedback(
        AggregateState { keys: vec![], version: 0 },
        |command, state| {
            match command {
                KeyCommand::Generate { algorithm } => {
                    let key = generate_key(algorithm);
                    let event = KeyEvent::KeyGenerated { /* ... */ };

                    let mut new_keys = state.keys.clone();
                    new_keys.push(key.into());

                    (
                        vec![event],
                        AggregateState {
                            keys: new_keys,
                            version: state.version + 1,
                        }
                    )
                }
                _ => (vec![], state.clone())
            }
        }
    )
}

// === 5. Causality Tracking ===

fn build_causal_chain(events: Vec<KeyEvent>) -> Result<CausalChain<KeyEvent>, CausalityError> {
    let mut chain = CausalChain::new();

    for event in events {
        let causal_event = if chain.len() == 0 {
            CausalEvent::new(event)
        } else {
            let dependencies: Vec<_> = chain.events()
                .iter()
                .map(|e| e.id())
                .collect();
            CausalEvent::caused_by(event, dependencies)
        };

        std::thread::sleep(std::time::Duration::from_millis(1));
        chain = chain.add(causal_event)?;
    }

    Ok(chain)
}

// === 6. Complete Workflow ===

async fn complete_workflow() -> Result<(), Box<dyn Error>> {
    // 1. Collect user intents
    let intents = collect_intents(vec![
        AppIntent::GenerateKey,
    ]);

    // 2. Route intents to commands
    let router = create_router();
    let commands: Vec<_> = intents.occurrences(0.0, 10.0)
        .into_iter()
        .map(|(_, intent)| router.clone().run(intent))
        .collect();

    // 3. Process commands through aggregate
    let mut aggregate = create_aggregate();
    let all_events: Vec<_> = commands.into_iter()
        .flat_map(|cmd| aggregate.process(cmd))
        .collect();

    // 4. Build causal chain
    let chain = build_causal_chain(all_events.clone())?;
    chain.validate()?;

    // 5. Publish to NATS
    let publisher = NatsEventPublisher::new().await?;
    for event in all_events {
        publisher.publish(&event).await?;
    }

    // 6. Get topological order for replay
    if let Some(ordered) = chain.topological_order() {
        println!("Event order:");
        for (i, event) in ordered.iter().enumerate() {
            println!("  {}. {:?}", i + 1, event.data());
        }
    }

    Ok(())
}
```

---

## Best Practices

### 1. Intent Classification

Always classify intents by source:

```rust
// ✅ GOOD: Clear intent classification
enum Intent {
    UiButtonClicked { id: String },           // From GUI
    DomainEventReceived { event: KeyEvent },  // From domain
    PortNatsConnected,                        // From port
    SystemTimerTicked,                        // From system
    ErrorOccurred { error: String },          // Error handling
}

// ❌ BAD: Mixed concerns
enum Intent {
    Click,
    Event(KeyEvent),
    Tick,
}
```

### 2. Use Signals for Temporal Reasoning

```rust
// ✅ GOOD: Query events in time range
let recent_errors = error_signal.occurrences(now - 60.0, now);

// ❌ BAD: Manual filtering
let recent_errors: Vec<_> = all_errors.iter()
    .filter(|e| e.timestamp > now - 60.0)
    .collect();
```

### 3. Explicit Causality

```rust
// ✅ GOOD: Explicit dependencies
let event2 = CausalEvent::caused_by(data, vec![event1.id()]);

// ❌ BAD: Implicit ordering
events.push(event1);
events.push(event2); // Might be reordered!
```

### 4. Feedback for State

```rust
// ✅ GOOD: Feedback loop
let mut state_machine = feedback(initial, process_fn);

// ❌ BAD: Mutable state
let mut state = initial;
state.apply(event); // Mutation!
```

### 5. Route Complex Logic

```rust
// ✅ GOOD: Composable routes
let pipeline = RouteBuilder::new(parse)
    .then(validate)
    .then(transform)
    .then(persist)
    .build();

// ❌ BAD: Giant match statement
match intent {
    Intent::A => { /* hundreds of lines */ },
    Intent::B => { /* hundreds of lines */ },
    // ...
}
```

---

## Troubleshooting

### Common Issues

**Issue**: "Type inference errors with signals"
```rust
// ❌ This may fail
Signal::continuous(|t| t * 2.0)

// ✅ Be explicit
Signal::<ContinuousKind, f64>::continuous(|t| t * 2.0)
```

**Issue**: "Move closure errors with feedback"
```rust
// ❌ Signal moved into closure
feedback(state, move |x, s| signal.sample(x))

// ✅ Wrap in Arc
let signal_arc = Arc::new(signal);
feedback(state, move |x, s| signal_arc.sample(x))
```

**Issue**: "Causality validation fails"
```rust
// ❌ Events too close in time
let e1 = CausalEvent::new(data1);
let e2 = CausalEvent::caused_by(data2, vec![e1.id()]);

// ✅ Add delay
let e1 = CausalEvent::new(data1);
std::thread::sleep(Duration::from_millis(1));
let e2 = CausalEvent::caused_by(data2, vec![e1.id()]);
```

---

## Next Steps

1. **Read Getting Started**: See `N_ARY_FRP_GETTING_STARTED.md`
2. **Run Examples**: All 6 examples demonstrate integration patterns
3. **Study MVI Guide**: See `MVI_IMPLEMENTATION_GUIDE.md` for MVI patterns
4. **Check API Docs**: Run `cargo doc --open`

---

**Happy Integrating! 🚀**
