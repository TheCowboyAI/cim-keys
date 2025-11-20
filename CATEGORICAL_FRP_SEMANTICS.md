# Categorical Semantics for N-ary FRP in cim-keys

## Overview

This document provides the **categorical foundations** for n-ary FRP in cim-keys, grounding the 10 axioms in rigorous category theory. It bridges abstract category theory with practical Rust implementation.

**Key Insight**: Domain aggregates, events, and projections in cim-keys form a **Process Category** with temporal functors and natural transformations.

## Abstract Process Categories (APCs)

### Definition

An **Abstract Process Category** is a cartesian closed category **C** with finite coproducts and a family of **temporal functors** that model time-varying processes.

**In cim-keys**:
- **Objects**: Domain types (Person, Organization, KeyEvent, Model)
- **Morphisms**: Pure functions between domain types
- **Cartesian closure**: Functions are first-class (Rust closures)
- **Coproducts**: Result<T, E> (sum types)

### Time-Indexed Type Families

Every FRP type `T` corresponds to a family of sets indexed by time:
```
⟦T⟧ : Time → Set
```

**Example in cim-keys**:
```rust
// KeyEvent is a time-indexed family
type KeyEventAt = fn(Time) -> Set<KeyEvent>

// At time t=0, no events yet
⟦KeyEvent⟧(0) = ∅

// At time t=100, events that occurred by t=100
⟦KeyEvent⟧(100) = {KeyGenerated(...), CertificateSigned(...), ...}
```

**Relation to Axiom A1** (Multi-Kinded Signals):
- Event signals: `⟦Event T⟧(t) = [(t', x) | t' ≤ t, x : T]` (finite list)
- Step signals: `⟦Step T⟧(t) = T` (single value)
- Continuous signals: `⟦Continuous T⟧(t) = T` (smooth value)

## Temporal Functors

### Basic Temporal Functor

The fundamental temporal functor `▷'' : W × C × C → C` models process types:

```
A ▷''_W B
```

represents a process with:
- **Continuous part**: type `A` (ongoing state)
- **Terminal event**: type `B` (completion value)
- **Wait set W**: temporal structure (when termination can occur)

**In cim-keys**:
```rust
// Process type in Rust
pub struct Process<A, B> {
    continuous: Signal<Step, A>,      // Ongoing state
    terminal: Signal<Event, B>,        // Completion event
    wait_set: WaitSet,                 // When can it terminate?
}

// Example: Root CA generation process
pub type GenerateRootCAProcess = Process<
    GenerationState,  // A: Current progress
    Certificate       // B: Final certificate
>;
```

### Derived Temporal Functors

The paper derives several functors from `▷''`:

#### 1. Behaviors (□)

Non-terminating time-varying values:
```
□A = A ▷''_∞ 0
```

**In cim-keys**:
```rust
// Behavior: Never terminates, always has value
pub type Behavior<T> = Signal<Continuous, T>;

// Example: Animation time
pub type AnimationTime = Behavior<f32>;
```

**Relation to Axiom A10** (Continuous Time Semantics):
Behaviors model continuous signals that exist at all times.

#### 2. Events (◇)

Values at specific times:
```
◇B = 1 ▷''_W B
```

**In cim-keys**:
```rust
// Event: Occurs at discrete time points
pub type Event<T> = Signal<EventKind, T>;

// Example: Key generation event
pub type KeyGeneratedEvent = Event<KeyGenerated>;
```

**Relation to Axiom A7** (Change Prefixes):
Events are represented as change prefixes: `Time → [(Δt, value)]`

#### 3. Delayed Behaviors (▷'_0)

Processes that start at present time:
```
A ▷'_0 B = A ▷''_[0,∞) B
```

**In cim-keys**:
```rust
// Starts now, may terminate in future
pub struct DelayedBehavior<A, B> {
    start_time: Time,
    process: Process<A, B>,
}

// Example: YubiKey provisioning (starts now, completes when done)
pub type ProvisioningProcess = DelayedBehavior<ProvisioningState, YubiKeyProvisioned>;
```

## Monads and Comonads for Process Composition

### Ideal Monads

An **ideal monad** `(T', μ')` where `T = Id × T'` captures the pattern of pairing current state with future computations.

**Definition**:
```
T'A = A × (future computation)
μ' : T'(T'A) → T'A  (join)
```

**In cim-keys (Event Sourcing)**:
```rust
// State paired with future projection
pub struct EventSourced<State, Event> {
    current_state: State,           // Current aggregate state
    future_events: Stream<Event>,   // Future event stream
}

// Monad join: Flatten nested futures
impl<S, E> EventSourced<S, E> {
    fn join(nested: EventSourced<EventSourced<S, E>, E>) -> EventSourced<S, E> {
        EventSourced {
            current_state: nested.current_state.current_state,
            future_events: nested.current_state.future_events
                .chain(nested.future_events),
        }
    }
}
```

**Relation to Axiom A3** (Decoupled Functions):
The monad structure ensures output at time t depends only on state before t.

### Process Joining (Concatenation)

Natural transformation `ϑ'' : A ▷''_W (A ▷_W B) → A ▷''_W B`

Concatenates processes: first process's continuous part, then second process.

**In cim-keys (Command Sequencing)**:
```rust
// Sequential composition of Commands
pub fn then<A, B, C>(
    first: Process<A, B>,
    second: impl FnOnce(B) -> Process<B, C>,
) -> Process<A, C> {
    Process {
        continuous: first.continuous,
        terminal: first.terminal.flat_map(|b| {
            let second_proc = second(b);
            second_proc.terminal
        }),
        wait_set: first.wait_set.union(second.wait_set),
    }
}

// Example: Generate key THEN sign certificate
let workflow = generate_key_process
    .then(|key| sign_certificate_process(key))
    .then(|cert| store_certificate_process(cert));
```

**Relation to Axiom A6** (Compositional Routing):
Process joining is the categorical basis for `>>>` operator.

**Coherence Law** (associativity):
```
(ϑ' ∘ (id × ϑ)) = ϑ' ∘ ϑ''
```

**In Rust**:
```rust
// Test compositional law
#[test]
fn test_process_associativity() {
    let f = process_1;
    let g = process_2;
    let h = process_3;

    let lhs = f.then(|x| g.then(|y| h));
    let rhs = f.then(g).then(h);

    assert_eq!(lhs, rhs);  // Associativity must hold
}
```

### Completely Iterative Monads (Corecursion)

For any morphism `f : C → T'(B + C)`, there exists unique `f^∞ : C → T'B` satisfying:

```
f^∞ = μ' ∘ T'(id_B + f^∞) ∘ f
```

**Interpretation**:
- If `f(z)` terminates with `Left(b)`, then `f^∞(z)` terminates with `b`
- If `f(z)` terminates with `Right(z')`, then `f^∞(z)` is concatenation of `f(z)` with `f^∞(z')`
- If `f(z)` doesn't terminate, neither does `f^∞(z)`

**In cim-keys (Infinite Event Streams)**:
```rust
// Corecursive projection builder
pub fn build_projection<State, Event>(
    initial: State,
    apply: impl Fn(State, Event) -> Either<State, State>,
) -> impl Stream<State> {
    // f^∞: Unfold infinite stream from event log
    unfold(initial, move |state| {
        match event_stream.next() {
            None => None,  // Stream ends
            Some(event) => match apply(state.clone(), event) {
                Left(new_state) => Some(new_state),  // Continue
                Right(new_state) => Some(new_state), // Continue
            }
        }
    })
}

// Example: Build projection from infinite event stream
let projection_stream = build_projection(
    AggregateState::default(),
    |state, event| {
        let new_state = state.apply(event);
        Left(new_state)  // Always continue
    }
);
```

**Relation to Axiom A8** (Type-Safe Feedback Loops):
Corecursion provides the categorical foundation for safe feedback loops.

### Recursive Comonads (Recursion)

For any `f : U'(A × C) → C`, there exists unique `f^* : U'A → C` where `f^*(p)` depends only on proper suffixes of `p`.

**In cim-keys (Historical Aggregation)**:
```rust
// Recursive aggregation over process history
pub fn aggregate_history<A, C>(
    f: impl Fn(&[A], C) -> C,
    history: &[A],
    initial: C,
) -> C {
    // f^*: Recurse over suffixes
    match history {
        [] => initial,
        [x, xs @ ..] => {
            let rest = aggregate_history(f, xs, initial);
            f(&[x], rest)
        }
    }
}

// Example: Calculate total keys generated over time
let total_keys = aggregate_history(
    |event_slice, count| match event_slice[0] {
        KeyEvent::KeyGenerated(_) => count + 1,
        _ => count,
    },
    &event_log,
    0,
);
```

**Relation to Axiom A5** (Totality):
Recursion on well-founded orders guarantees termination.

## Concrete Process Categories (CPCs)

### Functor Category Construction

A **Concrete Process Category** is a functor category `B^I` where:
- **I**: Temporal index category (totally ordered time)
- **B**: Base category (Set for Rust types)

**Objects in I**: Pairs `(t, t_o)` where:
- `t`: Time of inhabitant
- `t_o`: Observation time
- Constraint: `t ≤ t_o` (can only observe past)

**In cim-keys (Eventual Consistency)**:
```rust
// Observation of value at time t, observed at time t_o
pub struct Observation<T> {
    value: T,
    value_time: Time,       // t: When value existed
    observation_time: Time, // t_o: When we learned about it
}

impl<T> Observation<T> {
    fn new(value: T, value_time: Time, observation_time: Time) -> Result<Self, Error> {
        if value_time > observation_time {
            Err(Error::CausalityViolation)  // Cannot observe future!
        } else {
            Ok(Observation { value, value_time, observation_time })
        }
    }
}

// Example: NATS event with delivery time
pub struct NatsEvent<T> {
    payload: T,
    event_time: Time,     // When event occurred
    received_time: Time,  // When we received it (t_o)
}
```

**Relation to Axiom A4** (Causality Guarantees):
The constraint `t ≤ t_o` enforces causality at type level.

### R-CPCs (Well-Founded Time)

For recursion and corecursion support, require:
- For all `t ∈ T`, the order `≥` on `{t' | t' ≤ t}` is well-founded

**Interpretation**: No infinite past (prevents ω-supertasks)

**In cim-keys**:
```rust
// Well-founded time: Every descending chain terminates
pub trait WellFoundedTime {
    /// Predecessor function (guaranteed to terminate)
    fn pred(&self) -> Option<Time>;

    /// Verify no infinite regression
    fn check_well_founded(&self) -> bool {
        let mut current = Some(*self);
        let mut visited = HashSet::new();

        while let Some(t) = current {
            if !visited.insert(t) {
                return false;  // Cycle detected!
            }
            current = t.pred();
        }

        true  // Terminated, well-founded
    }
}

// Example: Natural number time (well-founded)
impl WellFoundedTime for u64 {
    fn pred(&self) -> Option<Time> {
        if *self == 0 {
            None  // Base case
        } else {
            Some(*self - 1)
        }
    }
}
```

**Relation to Axiom A5** (Totality):
Well-founded time ensures all recursive lookups terminate.

## Mapping to DDD and cim-keys

### Domain Aggregates as Objects

```rust
// Aggregate Root = Object in category C
pub trait AggregateRoot {
    type Id: EntityId;
    type State: Clone;
    type Event: DomainEvent;

    // Morphism: Event → State → State
    fn apply(&self, state: Self::State, event: Self::Event) -> Self::State;
}

// Example: KeyManagementAggregate
impl AggregateRoot for KeyManagementAggregate {
    type Id = Uuid;
    type State = KeyManagementState;
    type Event = KeyEvent;

    fn apply(&self, mut state: Self::State, event: Self::Event) -> Self::State {
        match event {
            KeyEvent::KeyGenerated(e) => {
                state.keys.insert(e.key_id, e);
                state
            }
            // ... other events
        }
    }
}
```

**Categorical Interpretation**:
- `apply` is a morphism in **C**
- Preserves aggregate invariants (morphism laws)

### Events as Values in Temporal Functors

```rust
// Domain Event = Value in ◇B
pub trait DomainEvent {
    fn event_time(&self) -> Time;
    fn aggregate_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
}

// Event Signal: ◇(KeyEvent)
pub type KeyEventSignal = Signal<EventKind, KeyEvent>;
```

**Categorical Interpretation**:
- Each event is an inhabitant of `◇B` at a specific time
- Event streams are morphisms `1 → □(◇B)`

### Event Handlers as Natural Transformations

```rust
// Natural transformation: F ⇒ G
pub trait EventHandler<F, G> {
    fn handle(&self, input: F) -> G;

    // Naturality condition: commutes with time evolution
    fn naturality_law(&self, f: impl Fn(F) -> F) -> bool;
}

// Example: Key generation handler
pub struct GenerateKeyHandler;

impl EventHandler<GenerateKeyCommand, KeyGeneratedEvent> for GenerateKeyHandler {
    fn handle(&self, cmd: GenerateKeyCommand) -> KeyGeneratedEvent {
        // Generate key...
        KeyGeneratedEvent { /* ... */ }
    }

    fn naturality_law(&self, f: impl Fn(GenerateKeyCommand) -> GenerateKeyCommand) -> bool {
        // η_X ∘ F(f) = G(f) ∘ η_Y
        // Must commute with morphisms
        true
    }
}
```

**Categorical Interpretation**:
- Handler is natural transformation between functors
- Naturality ensures it commutes with time evolution

### Sagas as Process Types

```rust
// Saga = Process type A ▷_W B
pub struct Saga<State, Result> {
    initial_state: State,
    steps: Vec<SagaStep<State>>,
    compensation: Vec<CompensatingAction<State>>,
    result: PhantomData<Result>,
}

// Example: PKI hierarchy generation saga
pub type PKIHierarchySaga = Saga<
    PKIState,            // A: Intermediate state
    PKIHierarchyResult   // B: Final result
>;

impl PKIHierarchySaga {
    // Process joining: Compose saga steps
    pub fn then(self, next: impl FnOnce(PKIState) -> Saga<PKIState, PKIHierarchyResult>)
        -> Saga<PKIState, PKIHierarchyResult>
    {
        // ϑ'': Concatenate process continuous parts
        unimplemented!()
    }
}
```

**Categorical Interpretation**:
- Saga is inhabitant of `A ▷_W B`
- Composition via `ϑ''` (process joining)

### Projections as Corecursive Functions

```rust
// Projection = f^∞ : EventStream → ReadModel
pub struct ProjectionBuilder<Event, ReadModel> {
    apply: Box<dyn Fn(ReadModel, Event) -> Either<ReadModel, ReadModel>>,
}

impl<E, R> ProjectionBuilder<E, R> {
    // Build projection corecursively
    pub fn build(&self, initial: R, events: impl Stream<E>) -> impl Stream<R> {
        // f^∞: Corecursive unfold
        events.scan(initial, |state, event| {
            match (self.apply)(state.clone(), event) {
                Left(new_state) => {
                    *state = new_state.clone();
                    Some(new_state)
                }
                Right(new_state) => {
                    *state = new_state.clone();
                    Some(new_state)
                }
            }
        })
    }
}
```

**Categorical Interpretation**:
- Projection builder is corecursive morphism `f^∞`
- Processes infinite event streams productively

## Axiom-to-Category Mapping

| Axiom | Categorical Concept | Implementation |
|-------|---------------------|----------------|
| **A1: Multi-Kinded Signals** | Temporal functors (□, ◇, ▷) | `Signal<Kind, T>` |
| **A2: Signal Vectors** | Product types in **C** | `SignalVector<(T1, T2, ...)>` |
| **A3: Decoupled Functions** | Causal morphisms (no future deps) | `Decoupled` marker trait |
| **A4: Causality Guarantees** | CPC observation constraint (t ≤ t_o) | `At<T, TIME>` |
| **A5: Totality** | Well-founded recursion (R-CPCs) | `Total` marker trait |
| **A6: Compositional Routing** | Process joining (ϑ''), Arrow category | `>>>`, `***`, `&&&` operators |
| **A7: Change Prefixes** | Discrete event representation ◇B | `ChangePrefix<T>` |
| **A8: Feedback Loops** | Completely iterative monads (f^∞) | `feedback` combinator |
| **A9: Semantic Preservation** | Functoriality, naturality | Property tests for laws |
| **A10: Continuous Time** | Dense time in CPCs, behaviors □A | `Time = f64` (continuous) |

## Implementation Checklist

### Phase 1: Temporal Functors

- [ ] Define temporal functor types
  ```rust
  pub enum TemporalFunctor {
      Behavior(Behavior),    // □A
      Event(Event),          // ◇B
      Process(Process),      // A ▷_W B
  }
  ```

- [ ] Implement functor operations (fmap)
  ```rust
  impl<A, B> Functor<A, B> for TemporalFunctor {
      fn fmap<F: Fn(A) -> B>(&self, f: F) -> TemporalFunctor;
  }
  ```

### Phase 2: Process Joining

- [ ] Implement ϑ'' (process concatenation)
  ```rust
  pub fn join_processes<A, B, C>(
      first: Process<A, B>,
      second: impl FnOnce(B) -> Process<B, C>,
  ) -> Process<A, C>
  ```

- [ ] Verify coherence law (associativity)
  ```rust
  #[test]
  fn test_process_joining_associativity() { /* ... */ }
  ```

### Phase 3: Monadic Structure

- [ ] Define ideal monad for event sourcing
  ```rust
  pub struct EventSourcedMonad<T> {
      current: T,
      future: Stream<Event>,
  }

  impl<T> EventSourcedMonad<T> {
      fn join(nested: EventSourcedMonad<EventSourcedMonad<T>>) -> EventSourcedMonad<T>;
  }
  ```

- [ ] Prove monad laws
  ```rust
  #[test]
  fn test_monad_left_identity() { /* ... */ }

  #[test]
  fn test_monad_right_identity() { /* ... */ }

  #[test]
  fn test_monad_associativity() { /* ... */ }
  ```

### Phase 4: Corecursion

- [ ] Implement f^∞ combinator
  ```rust
  pub fn corecursive<C, B>(
      f: impl Fn(C) -> Either<B, C>,
  ) -> impl Fn(C) -> Stream<B>
  ```

- [ ] Apply to projection builders
  ```rust
  let projection = corecursive(|state| apply_event(state, next_event));
  ```

### Phase 5: Well-Founded Time

- [ ] Define well-founded time trait
  ```rust
  pub trait WellFoundedTime {
      fn pred(&self) -> Option<Time>;
      fn check_well_founded(&self) -> bool;
  }
  ```

- [ ] Enforce in recursive operations
  ```rust
  pub fn recursive_lookup<T: WellFoundedTime>(
      time: T,
      lookup: impl Fn(T) -> Option<Value>,
  ) -> Option<Value>
  ```

## Testing Categorical Properties

### Functoriality

```rust
#[test]
fn test_functor_identity() {
    // fmap id = id
    let signal: Signal<Event, i32> = Signal::new(42);
    assert_eq!(signal.fmap(|x| x), signal);
}

#[test]
fn test_functor_composition() {
    // fmap (g ∘ f) = fmap g ∘ fmap f
    let f = |x: i32| x + 1;
    let g = |x: i32| x * 2;

    let signal: Signal<Event, i32> = Signal::new(10);

    let lhs = signal.clone().fmap(|x| g(f(x)));
    let rhs = signal.fmap(f).fmap(g);

    assert_eq!(lhs, rhs);
}
```

### Naturality

```rust
#[test]
fn test_natural_transformation_square() {
    // η_X ∘ F(f) = G(f) ∘ η_Y
    let f: fn(i32) -> i32 = |x| x + 1;
    let eta: fn(Signal<Event, i32>) -> Signal<Step, i32> = /* ... */;

    let signal: Signal<Event, i32> = Signal::new(10);

    let lhs = eta(signal.fmap(f));
    let rhs = signal.fmap(f).then(eta);

    assert_eq!(lhs, rhs);
}
```

### Monad Laws

```rust
#[test]
fn test_monad_laws() {
    // Left identity: return a >>= f ≡ f a
    // Right identity: m >>= return ≡ m
    // Associativity: (m >>= f) >>= g ≡ m >>= (\x -> f x >>= g)
    // ...
}
```

## Conclusion

The categorical semantics provide:

1. **Rigorous Foundation**: N-ary FRP axioms grounded in category theory
2. **Compositional Structure**: Arrow category with process joining
3. **Temporal Reasoning**: Time-indexed types with causality guarantees
4. **Safe Recursion**: Well-founded time prevents infinite regression
5. **Infinite Streams**: Corecursion for productive event processing
6. **Formal Verification**: Laws testable with property-based tests

By implementing these categorical structures, cim-keys achieves:
- **Mathematical Correctness**: Behavior proven from categorical axioms
- **Composability**: Build complex systems from simple categorical operations
- **Type Safety**: Causality and termination guaranteed by types
- **Testability**: Verify categorical laws automatically

**Next Steps**: Implement temporal functors and process joining in Phase 1, then build up to full categorical semantics incrementally.
