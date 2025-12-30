# N-ary FRP Getting Started Guide

**Complete guide to using the n-ary Functional Reactive Programming system in cim-keys**

## Table of Contents

1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [Quick Start](#quick-start)
4. [Working with Signals](#working-with-signals)
5. [Routing and Composition](#routing-and-composition)
6. [Causality and Event Ordering](#causality-and-event-ordering)
7. [Feedback Loops](#feedback-loops)
8. [Continuous Time and Animation](#continuous-time-and-animation)
9. [Common Patterns](#common-patterns)
10. [Examples](#examples)

---

## Introduction

The n-ary FRP system in cim-keys implements a type-safe, compositional approach to functional reactive programming based on abstract process categories. This guide will help you understand and use the system effectively.

### What is N-ary FRP?

N-ary FRP extends traditional FRP with:
- **Multi-kinded signals**: Events, Steps, and Continuous values
- **Type-safe composition**: Signals can be combined safely
- **Causal ordering**: Time flows forward, no paradoxes
- **Feedback loops**: State evolution without mutation
- **Routing combinators**: Arrow-based data flow

### Why Use It?

- ✅ **No mutable state**: All operations pure
- ✅ **Type safety**: Compiler catches temporal errors
- ✅ **Composable**: Build complex behaviors from simple parts
- ✅ **Testable**: Pure functions are easy to test
- ✅ **Thread-safe**: No data races by construction

---

## Core Concepts

### 1. Signal Kinds

Signals are distinguished by their temporal characteristics:

```rust
use cim_keys::signals::{Signal, EventKind, StepKind, ContinuousKind};

// Event: Discrete occurrences at specific times
let clicks: Signal<EventKind, ButtonClick> =
    Signal::event(vec![(0.0, click1), (1.5, click2)]);

// Step: Piecewise constant (changes discretely)
let model: Signal<StepKind, AppState> =
    Signal::step(AppState::default());

// Continuous: Smooth function of time
let animation: Signal<ContinuousKind, f64> =
    Signal::continuous(|t| t * 2.0);
```

### 2. Signal Vectors

Combine multiple signals for n-ary operations:

```rust
use cim_keys::signals::{SignalVec2, SignalVec3};

// Pair of signals
let pair = SignalVec2::new(signal_a, signal_b);

// Triple of signals
let triple = SignalVec3::new(signal_x, signal_y, signal_z);
```

### 3. Time Representation

```rust
use cim_keys::signals::Time;

// Time is f64 representing continuous semantics
let now: Time = 0.0;
let later: Time = 1.5;
```

---

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cim-keys = { version = "0.8.0", features = ["gui"] }
```

### Your First Signal

```rust
use cim_keys::signals::{Signal, EventKind};

fn main() {
    // Create an event signal
    let events = Signal::<EventKind, String>::event(vec![
        (0.0, "start".to_string()),
        (1.0, "middle".to_string()),
        (2.0, "end".to_string()),
    ]);

    // Query occurrences
    let middle_events = events.occurrences(0.5, 1.5);
    println!("Events between 0.5 and 1.5: {:?}", middle_events);
}
```

Run the examples:

```bash
# See all available examples
ls examples/*signal*.rs examples/*routing*.rs examples/*feedback*.rs

# Run an example
cargo run --example signal_integration
cargo run --example animation_time_continuous
```

---

## Working with Signals

### Event Signals

Use for discrete occurrences:

```rust
use cim_keys::signals::{Signal, EventKind};

// Create event signal
let key_events = Signal::<EventKind, KeyEvent>::event(vec![
    (0.0, KeyEvent::Generated { key_id: 1 }),
    (1.0, KeyEvent::Exported { key_id: 1 }),
    (2.0, KeyEvent::Revoked { key_id: 1 }),
]);

// Query by time range
let recent = key_events.occurrences(0.5, 2.5);

// Count occurrences
let count = key_events.count(0.0, 3.0);

// Get all up to time
let history = key_events.occurrences_until(1.5);
```

### Step Signals

Use for state that changes discretely:

```rust
use cim_keys::signals::{Signal, StepKind};

#[derive(Clone)]
struct AppState {
    counter: i32,
    status: String,
}

// Create step signal
let state = Signal::<StepKind, AppState>::step(AppState {
    counter: 0,
    status: "idle".to_string(),
});

// Sample at any time (same value until updated)
let current = state.sample(0.5);

// Create new state (immutable)
let new_state = state.with_value(AppState {
    counter: 1,
    status: "active".to_string(),
});
```

### Continuous Signals

Use for smooth time-varying values:

```rust
use cim_keys::signals::{Signal, ContinuousKind};
use cim_keys::signals::continuous::{linear_time, sine_wave};

// Linear time
let time = linear_time();
assert_eq!(time.sample(1.5), 1.5);

// Sine wave
let wave = sine_wave(1.0); // 1 Hz
let value = wave.sample(0.25); // Peak

// Custom function
let custom = Signal::<ContinuousKind, f64>::continuous(|t| {
    t * t + 2.0 * t + 1.0
});
```

### Transforming Signals

```rust
// Map values (functor)
let doubled = events.fmap(|x| x * 2);

// For continuous signals
use cim_keys::signals::continuous::ContinuousSignal;

let scaled = time.map(|t| t * 10.0);
let composed = signal1.compose(signal2);
```

---

## Routing and Composition

### Basic Routes

```rust
use cim_keys::routing::{Route, id, compose};

// Identity route
let identity = id::<String>();
assert_eq!(identity.run("hello".to_string()), "hello");

// Create route
let double = Route::new(|x: i32| x * 2);
assert_eq!(double.run(5), 10);

// Compose routes
let add_one = Route::new(|x: i32| x + 1);
let times_two = Route::new(|x: i32| x * 2);
let pipeline = compose(add_one, times_two);
assert_eq!(pipeline.run(3), 8); // (3 + 1) * 2
```

### Route Builder (Fluent API)

```rust
use cim_keys::routing::RouteBuilder;

// Build complex pipeline
let result = RouteBuilder::new(|x: i32| x + 1)
    .then(|x| x * 2)
    .then(|x| x.to_string())
    .run_with(5);

assert_eq!(result, "12");
```

### Parallel and Fanout

```rust
use cim_keys::routing::{parallel, fanout};

// Process two inputs in parallel
let route1 = Route::new(|x: i32| x + 1);
let route2 = Route::new(|y: String| y.len());
let parallel_route = parallel(route1, route2);

let (result1, result2) = parallel_route.run((5, "hello".to_string()));
assert_eq!(result1, 6);
assert_eq!(result2, 5);

// Fanout: Send one input to multiple routes
let inc = Route::new(|x: i32| x + 1);
let dbl = Route::new(|x: i32| x * 2);
let fanned = fanout(inc, dbl);

let (inc_result, dbl_result) = fanned.run(5);
assert_eq!(inc_result, 6);
assert_eq!(dbl_result, 10);
```

### Integration with MVI

```rust
use cim_keys::routing::Route;
use cim_keys::mvi::Intent;

// Route intents to handlers
let intent_router = Route::new(|intent: Intent| {
    match intent {
        Intent::UiButtonClicked { .. } => handle_click(),
        Intent::DomainEventReceived { .. } => handle_event(),
        _ => ()
    }
});
```

---

## Causality and Event Ordering

### Creating Causal Events

```rust
use cim_keys::causality::{CausalEvent, CausalChain};

// Root event (no dependencies)
let event1 = CausalEvent::new("Organization created");

// Dependent event
let event2 = CausalEvent::caused_by(
    "Person added",
    vec![event1.id()], // Depends on event1
);

// Build validated chain
let chain = CausalChain::new()
    .add(event1)?
    .add(event2)?;

// Validate causality
chain.validate()?;
```

### Working with Domain Events

```rust
use cim_keys::events::KeyEvent;
use cim_keys::causality::helpers::{wrap_event, wrap_dependent_event};

// Wrap domain event
let root = wrap_event(KeyEvent::KeyGenerated { /* ... */ });

// Create dependent event
let dependent = wrap_dependent_event(
    KeyEvent::KeyExported { /* ... */ },
    vec![root.id()],
);
```

### Querying Dependencies

```rust
// Get topological order
if let Some(ordered) = chain.topological_order() {
    for event in ordered {
        println!("Event: {:?}", event.data());
    }
}

// Find dependents
let dependents = chain.dependents_of(event_id);
```

---

## Feedback Loops

### Basic Feedback

```rust
use cim_keys::combinators::feedback::{feedback, Decoupled};

#[derive(Clone)]
struct Counter {
    value: i32,
}

impl Decoupled for Counter {}

// Create feedback loop
let mut counter = feedback(
    Counter { value: 0 },
    |delta: i32, state: &Counter| {
        let new_value = state.value + delta;
        (new_value, Counter { value: new_value })
    }
);

// Process inputs
assert_eq!(counter.process(5), 5);
assert_eq!(counter.process(3), 8);
assert_eq!(counter.process(-2), 6);
```

### Aggregate as Feedback

```rust
use cim_keys::combinators::feedback::{feedback, Decoupled};

#[derive(Clone)]
struct Aggregate {
    version: u64,
    data: Vec<String>,
}

impl Decoupled for Aggregate {}

let mut aggregate = feedback(
    Aggregate { version: 0, data: vec![] },
    |event: String, state: &Aggregate| {
        let mut new_data = state.data.clone();
        new_data.push(event);
        let new_state = Aggregate {
            version: state.version + 1,
            data: new_data,
        };
        let summary = format!("v{}: {} events", new_state.version, new_state.data.len());
        (summary, new_state)
    }
);

let result = aggregate.process("Event1".to_string());
```

### Composing Feedback

```rust
// Map output
let doubled = counter.map(|x| x * 2);

// Access state
let current = counter.current_state();

// Reset state
counter.reset_state(Counter { value: 0 });
```

---

## Continuous Time and Animation

### Basic Animation

```rust
use cim_keys::signals::continuous::{
    linear_time, lerp, ease_in_out, ContinuousSignal
};

// Fade in over 2 seconds
let fade = lerp(0.0, 1.0, 0.0, 2.0);

for t in [0.0, 0.5, 1.0, 1.5, 2.0] {
    let opacity = fade.sample(t);
    println!("t={:.1}s: opacity={:.2}", t, opacity);
}
```

### Easing Functions

```rust
use cim_keys::signals::continuous::{linear_time, ease_in, ease_out, ease_in_out};

let time = linear_time();

// Smooth acceleration
let eased_in = ease_in(time.clone());

// Smooth deceleration
let eased_out = ease_out(time.clone());

// Smooth start and end
let eased_both = ease_in_out(time);
```

### Signal Transformations

```rust
use cim_keys::signals::continuous::{scale, offset, clamp};

// Double animation speed
let faster = scale(linear_time(), 2.0);

// Delay by 1 second
let delayed = offset(linear_time(), -1.0);

// Clamp to valid range
let clamped = clamp(linear_time(), 0.0, 1.0);
```

### Periodic Animations

```rust
use cim_keys::signals::continuous::sine_wave;

// Bounce animation (2 Hz)
let bounce = sine_wave(2.0).map(|sin_val| {
    let center = 200.0;
    let amplitude = 100.0;
    center + sin_val * amplitude
});

let y_position = bounce.sample(0.5);
```

---

## Common Patterns

### Pattern 1: Event Processing Pipeline

```rust
use cim_keys::routing::RouteBuilder;

let pipeline = RouteBuilder::new(|event: RawEvent| {
    // Parse
    parse_event(event)
})
.then(|parsed| {
    // Validate
    validate_event(parsed)
})
.then(|validated| {
    // Transform
    transform_event(validated)
})
.then(|transformed| {
    // Persist
    persist_event(transformed)
});
```

### Pattern 2: Aggregate with Feedback

```rust
#[derive(Clone)]
struct KeyRegistry {
    keys: HashMap<String, PublicKey>,
    version: u64,
}

impl Decoupled for KeyRegistry {}

let mut registry = feedback(
    KeyRegistry::default(),
    |command: KeyCommand, state: &KeyRegistry| {
        match command {
            KeyCommand::Register { id, key } => {
                let mut new_keys = state.keys.clone();
                new_keys.insert(id, key);
                (
                    Ok(()),
                    KeyRegistry {
                        keys: new_keys,
                        version: state.version + 1,
                    }
                )
            }
            // ... other commands
        }
    }
);
```

### Pattern 3: Multi-Stage Timeline

```rust
use cim_keys::signals::continuous::{lerp, constant, ease_in, ease_out};

fn multi_stage_animation(t: f64) -> f64 {
    if t < 1.0 {
        // Stage 1: Fade in (0-1s)
        ease_in(lerp(0.0, 1.0, 0.0, 1.0)).sample(t)
    } else if t < 3.0 {
        // Stage 2: Hold (1-3s)
        1.0
    } else {
        // Stage 3: Fade out (3-4s)
        ease_out(lerp(1.0, 0.0, 3.0, 4.0)).sample(t)
    }
}
```

### Pattern 4: Causal Event Chain

```rust
use cim_keys::causality::CausalChain;

let mut chain = CausalChain::new();

// Build chain with validation
let org_event = CausalEvent::new(create_organization());
chain = chain.add(org_event.clone())?;

let person_event = CausalEvent::caused_by(
    add_person(),
    vec![org_event.id()],
);
chain = chain.add(person_event.clone())?;

let key_event = CausalEvent::caused_by(
    generate_key(),
    vec![person_event.id()],
);
chain = chain.add(key_event)?;

// Validate entire chain
chain.validate()?;
```

---

## Examples

### Complete Example: Animated Key Generation

```rust
use cim_keys::signals::continuous::{linear_time, lerp, ease_in_out, ContinuousSignal};
use cim_keys::combinators::feedback::{feedback, Decoupled};

#[derive(Clone)]
struct GenerationProgress {
    stage: String,
    percent: f64,
}

impl Decoupled for GenerationProgress {}

fn main() {
    // Animation timeline
    let progress_animation = ease_in_out(lerp(0.0, 100.0, 0.0, 2.0));

    // State machine with feedback
    let mut generator = feedback(
        GenerationProgress {
            stage: "idle".to_string(),
            percent: 0.0,
        },
        |command: GenerateCommand, state: &GenerationProgress| {
            match command {
                GenerateCommand::Start => (
                    "Starting...".to_string(),
                    GenerationProgress {
                        stage: "generating".to_string(),
                        percent: 0.0,
                    }
                ),
                GenerateCommand::Progress(t) => (
                    format!("{:.0}%", progress_animation.sample(t)),
                    GenerationProgress {
                        stage: "generating".to_string(),
                        percent: progress_animation.sample(t),
                    }
                ),
                // ... more stages
            }
        }
    );

    // Simulate generation
    println!("{}", generator.process(GenerateCommand::Start));
    for t in [0.0, 0.5, 1.0, 1.5, 2.0] {
        println!("{}", generator.process(GenerateCommand::Progress(t)));
    }
}
```

### See More Examples

Run these examples to see the system in action:

```bash
# Signals
cargo run --example signal_integration

# Routing
cargo run --example routing_dsl_integration
cargo run --example mvi_routing_pattern

# Causality
cargo run --example causality_integration

# Feedback
cargo run --example feedback_aggregate_integration

# Continuous Time
cargo run --example animation_time_continuous
```

---

## Next Steps

1. **Read the Axioms**: See `N_ARY_FRP_AXIOMS.md` for theoretical foundation
2. **Explore Examples**: Run all 6 examples to see patterns in action
3. **Check API Docs**: Run `cargo doc --open` for detailed API documentation
4. **Integration Guide**: See `N_ARY_FRP_INTEGRATION_GUIDE.md` for MVI/GUI integration

## Getting Help

- **Examples**: `examples/` directory has 6 comprehensive examples
- **API Docs**: Run `cargo doc --open`
- **Theory**: Read `N_ARY_FRP_AXIOMS.md`
- **Progress**: See `N_ARY_FRP_PROGRESS.md` for implementation details

---

**Happy Reactive Programming! 🚀**
