# N-ary FRP Axioms for cim-keys

## Overview

This document establishes the **mandatory axioms** that cim-keys must follow to properly implement n-ary Functional Reactive Programming (FRP) principles. These axioms ensure mathematical correctness, composability, and alignment with CIM's category-theoretic foundations.

## Status: Current Compliance Assessment

| Axiom | Status | Notes |
|-------|--------|-------|
| **A1: Multi-Kinded Signals** | 🟡 Partial | Events exist, but continuous/step signals not explicit |
| **A2: Signal Vector Composition** | 🔴 Missing | No signal vector types or compositional operators |
| **A3: Decoupled Signal Functions** | 🟢 Good | Intent/Command pattern supports decoupling |
| **A4: Causality Guarantees** | 🟡 Partial | Tracked in events, not enforced by types |
| **A5: Totality and Well-Definedness** | 🟢 Good | Pure functions, no panics in update |
| **A6: Explicit Routing** | 🔴 Missing | Pattern matching, not compositional routing |
| **A7: Change Prefixes as Event Logs** | 🟢 Good | Event sourcing with timestamps |
| **A8: Type-Safe Feedback Loops** | 🔴 Missing | No feedback loop combinators |
| **A9: Semantic Preservation** | 🟡 Partial | Domain events preserved, but not compositionally |
| **A10: Continuous Time Semantics** | 🔴 Missing | No continuous signal support |

## The Ten Axioms

### A1: Multi-Kinded Signal Types (AXIOM)

**Principle**: Signal types must be distinguished by their temporal characteristics at the type level.

**Required Signal Kinds**:

```rust
/// Event Signal: Discrete occurrences at specific time points
/// Examples: KeyGenerated, CertificateSigned, UserClicked
pub trait EventSignal<T> {
    /// Get all occurrences in time range
    fn occurrences(&self, start: Time, end: Time) -> Vec<(Time, T)>;
}

/// Step Signal: Piecewise-constant value that changes discretely
/// Examples: Aggregate state, Projection state, UI model
pub trait StepSignal<T> {
    /// Sample current value at time t
    fn sample(&self, t: Time) -> T;

    /// Get time of last change before t
    fn last_change_before(&self, t: Time) -> Option<Time>;
}

/// Continuous Signal: Value defined at all times
/// Examples: Animation time, system metrics, resource utilization
pub trait ContinuousSignal<T> {
    /// Sample value at any time t
    fn sample(&self, t: Time) -> T;

    /// Derivative (rate of change) at time t
    fn derivative(&self, t: Time) -> T;
}
```

**Current Implementation**:
- ✅ Event signals exist (`KeyEvent` enum)
- ✅ Step signals implicit (Model, Projection)
- ❌ No continuous signals
- ❌ Not distinguished at type level

**Required Changes**:
1. Introduce `Signal<Kind, T>` type with kind parameter
2. Distinguish Intent by signal kind: `Intent::Event`, `Intent::Step`, `Intent::Continuous`
3. Model should be typed as `StepSignal<AppState>`

---

### A2: Signal Vector Composition (AXIOM)

**Principle**: Signal functions operate on **signal vectors** (tuples of independent signals) not single signals.

**Required Types**:

```rust
/// Signal vector descriptor at type level
pub trait SignalVectorDescriptor {
    type Kinds: TupleOfKinds;  // (Event, Step, Continuous, ...)
    type Values: TupleOfValues; // (KeyEvent, Model, f32, ...)
}

/// N-ary signal function: (Input Signals) → (Output Signals)
pub trait SignalFunction<Input: SignalVectorDescriptor, Output: SignalVectorDescriptor> {
    fn apply(&self, input: Input::Values, t: Time) -> Output::Values;
}

/// Composition of signal functions (categorical)
impl<A, B, C> SignalFunction<A, C> for Compose<B, A, C>
where
    A: SignalVectorDescriptor,
    B: SignalVectorDescriptor,
    C: SignalVectorDescriptor,
{
    // f >>> g = g ∘ f
    fn apply(&self, input: A::Values, t: Time) -> C::Values {
        let intermediate = self.first.apply(input, t);
        self.second.apply(intermediate, t)
    }
}
```

**Current Implementation**:
- ❌ Intent is single event, not signal vector
- ❌ Update function takes Intent, not signal vector
- ❌ No compositional operators

**Required Changes**:
1. Define `SignalVector` type for input/output
2. Update function signature: `update(signals: SignalVector<Inputs>) -> SignalVector<Outputs>`
3. Implement composition operators: `>>>`, `***`, `&&&`

---

### A3: Decoupled Signal Functions (AXIOM)

**Principle**: A signal function is **decoupled** if its output at time t depends only on inputs **before** t (strict causality).

**Type-Level Enforcement**:

```rust
/// Marker trait for decoupled signal functions
pub trait Decoupled: SignalFunction<Input, Output> {
    /// Proof: output at t cannot depend on input after t
    type CausalityProof;
}

/// Feedback combinator (only for decoupled functions)
pub fn feedback<SF, State>(sf: SF) -> impl SignalFunction<Input, Output>
where
    SF: SignalFunction<(Input, State), (Output, State)> + Decoupled,
{
    // Well-defined because SF is decoupled
    // State flows backward in time but causality is preserved
    unimplemented!()
}
```

**Current Implementation**:
- ✅ Commands are decoupled (async, return future Intent)
- ✅ Update is pure and decoupled
- ❌ No type-level enforcement
- ❌ No feedback combinators

**Required Changes**:
1. Add `Decoupled` marker trait
2. Enforce decoupling for Commands at type level
3. Implement `feedback` combinator for aggregate loops

---

### A4: Causality Guarantees (AXIOM)

**Principle**: Causality must be **guaranteed by the type system**, not just tracked at runtime.

**Type-Level Causality**:

```rust
/// Time-indexed types for causal ordering
pub struct At<T, const TIME: u64>(T);

/// Causal dependency: B depends on A
pub struct CausalDep<A, B> {
    cause: At<A, TIME_A>,
    effect: At<B, TIME_B>,
    _proof: CausalityProof<TIME_A, TIME_B>,  // TIME_B > TIME_A
}

/// Signal function that preserves causality
pub trait CausalSignalFunction: SignalFunction {
    /// Proof that output time >= input time
    type CausalityInvariant;
}
```

**Current Implementation**:
- ✅ Events have `correlation_id` and `causation_id`
- ✅ Runtime tracking of causality
- ❌ No compile-time guarantees
- ❌ Possible to create acausal dependencies

**Required Changes**:
1. Introduce time-indexed types `At<T, Time>`
2. Causality proof as type parameter
3. Update function must preserve causal ordering

---

### A5: Totality and Well-Definedness (AXIOM)

**Principle**: All signal functions must be **total** (defined for all inputs) and **well-defined** (deterministic, no panics).

**Enforcement**:

```rust
/// Total function: no panics, no undefined behavior
pub trait Total {
    /// This function never panics
    /// This function is deterministic
    /// This function always terminates
    const PROOF: TotalityProof;
}

impl Total for update {
    const PROOF: TotalityProof = TotalityProof {
        no_panic: "All pattern matches exhaustive, no unwrap()",
        deterministic: "No randomness, no external state",
        terminates: "No infinite loops, all recursion bounded",
    };
}
```

**Current Implementation**:
- ✅ Update function is pure and total
- ✅ No panics in update (uses `Result`)
- ✅ Deterministic (no `rand`, no `SystemTime`)
- ❌ Not proven at type level

**Required Changes**:
1. Use `Result<T, E>` for all fallible operations
2. Remove all `.unwrap()` from update function
3. Add totality proofs as documentation

---

### A6: Explicit Routing at Reactive Level (AXIOM)

**Principle**: All signal routing must be expressed using **compositional primitives** (identity, composition, fanout) not ad-hoc pattern matching.

**Routing Primitives**:

```rust
/// Identity: SF id = id
pub fn id<A: SignalVectorDescriptor>() -> impl SignalFunction<A, A> {
    |input, _t| input
}

/// Composition: f >>> g = g ∘ f
pub fn compose<A, B, C>(
    f: impl SignalFunction<A, B>,
    g: impl SignalFunction<B, C>,
) -> impl SignalFunction<A, C> {
    |input, t| g.apply(f.apply(input, t), t)
}

/// Parallel: f *** g applies f to first, g to second
pub fn parallel<A1, A2, B1, B2>(
    f: impl SignalFunction<A1, B1>,
    g: impl SignalFunction<A2, B2>,
) -> impl SignalFunction<(A1, A2), (B1, B2)> {
    |(a1, a2), t| (f.apply(a1, t), g.apply(a2, t))
}

/// Fanout: f &&& g sends input to both
pub fn fanout<A, B, C>(
    f: impl SignalFunction<A, B>,
    g: impl SignalFunction<A, C>,
) -> impl SignalFunction<A, (B, C)> {
    |input, t| (f.apply(input.clone(), t), g.apply(input, t))
}
```

**Current Implementation**:
- ❌ Intent routing via pattern matching
- ❌ No compositional routing
- ❌ Ad-hoc command chaining

**Required Changes**:
1. Replace pattern matching with routing DSL
2. Intent handlers as composable signal functions
3. Use `>>>`, `***`, `&&&` for routing

---

### A7: Change Prefixes as Event Logs (AXIOM)

**Principle**: Event and step signals must be represented as **change prefix** functions: `Time → [(Δt, value)]`

**Representation**:

```rust
/// Change prefix: maps time to list of changes
pub struct ChangePrefix<T> {
    changes: Vec<(Time, T)>,  // Sorted by time
}

impl<T> EventSignal<T> for ChangePrefix<T> {
    fn occurrences(&self, start: Time, end: Time) -> Vec<(Time, T)> {
        self.changes
            .iter()
            .filter(|(t, _)| *t >= start && *t < end)
            .cloned()
            .collect()
    }
}

impl<T: Clone> StepSignal<T> for ChangePrefix<T> {
    fn sample(&self, t: Time) -> T {
        self.changes
            .iter()
            .rev()  // Latest first
            .find(|(change_t, _)| *change_t <= t)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("No value before time {}", t))
    }
}
```

**Current Implementation**:
- ✅ Events stored as timestamped JSON
- ✅ Projections rebuilt from events
- ❌ Not formalized as change prefixes
- ❌ No time query operators

**Required Changes**:
1. Formalize events as `ChangePrefix<KeyEvent>`
2. Projections as `StepSignal` derived from change prefix
3. Add time-range query operators

---

### A8: Type-Safe Feedback Loops (AXIOM)

**Principle**: Recursive signal functions (feedback loops) must be **guaranteed well-defined** through type system constraints.

**Safe Feedback**:

```rust
/// Feedback combinator (only for decoupled SFs)
pub fn feedback<Input, Output, State, SF>(sf: SF) -> impl SignalFunction<Input, Output>
where
    SF: SignalFunction<(Input, State), (Output, State)> + Decoupled,
{
    // Well-defined because:
    // 1. SF is decoupled (output at t depends only on input before t)
    // 2. State flows backward but causality preserved
    // 3. Fixed-point exists by denotational semantics

    |input, t| {
        // Compute fixed point of state feedback
        let state_0 = State::default();
        let (output, state) = sf.apply((input, state_0), t);
        // Iterate until convergence (guaranteed by decoupling)
        output
    }
}
```

**Current Implementation**:
- ❌ No feedback loops
- ❌ No recursive signal functions
- ⚠️ Aggregate could be feedback (state + events → new state + events)

**Required Changes**:
1. Model aggregate as feedback loop
2. Implement `feedback` combinator
3. Prove decoupling for aggregate update

---

### A9: Semantic Preservation Under Composition (AXIOM)

**Principle**: Composition of signal functions must preserve semantics. `(f >>> g)(x) = g(f(x))` must hold denotationally.

**Compositional Laws**:

```rust
// Identity laws
f >>> id = f
id >>> f = f

// Associativity
(f >>> g) >>> h = f >>> (g >>> h)

// Parallel laws
id *** id = id
(f >>> g) *** (h >>> k) = (f *** h) >>> (g *** k)

// Fanout laws
(f &&& g) >>> first h = (f >>> h) &&& g
```

**Current Implementation**:
- ❌ No compositional operators
- ❌ Laws not encoded

**Required Changes**:
1. Implement routing operators
2. Prove laws with property tests
3. Document categorical structure

---

### A10: Continuous Time Semantics (AXIOM)

**Principle**: Time must be **continuous** in semantics, even if implementation is discrete sampling.

**Denotational Semantics**:

```rust
/// Continuous time (semantics)
pub type Time = f64;  // R (real numbers)

/// Discrete sampling (implementation)
pub type SamplingRate = Duration;

/// Signal as function of continuous time
pub trait Signal<T> {
    /// Denotational semantics: continuous function
    fn denote(&self) -> Box<dyn Fn(Time) -> T>;
}

/// Sampling is an approximation
pub fn sample<T, S: Signal<T>>(signal: S, rate: SamplingRate) -> Vec<(Time, T)> {
    // Discrete approximation of continuous signal
    let denoted = signal.denote();
    (0..)
        .map(|i| {
            let t = i as f64 * rate.as_secs_f64();
            (t, denoted(t))
        })
        .take(1000)
        .collect()
}
```

**Current Implementation**:
- ❌ Time is discrete (event timestamps)
- ❌ No continuous signals
- ⚠️ Animation time could be continuous

**Required Changes**:
1. Model time as `f64` in semantics
2. Discrete events are delta functions `δ(t - t₀)`
3. Interpolation for continuous queries

---

## Implementation Roadmap

### Phase 1: Type-Level Signal Kinds (Priority: HIGH)

```rust
// 1. Define signal kind types
pub enum SignalKind {
    Event,
    Step,
    Continuous,
}

// 2. Parameterize Intent by kind
pub enum Intent<K: SignalKind> {
    // Event signals
    UiButtonClicked { button_id: String } where K = Event,

    // Step signals (state changes)
    ModelUpdated { new_state: Model } where K = Step,

    // Continuous signals
    AnimationTick { time: f32 } where K = Continuous,
}

// 3. Update function respects signal kinds
pub fn update<In: SignalVector, Out: SignalVector>(
    signals: In,
) -> Out
where
    In: CausallyOrdered,
    Out: CausallyOrdered,
{
    // ...
}
```

### Phase 2: Compositional Routing (Priority: HIGH)

```rust
// 1. Define routing primitives
pub fn route_intent(intent: Intent) -> Route {
    match intent {
        Intent::UiGenerateRootCA =>
            id >>> generate_ca_handler >>> store_cert_handler,

        Intent::UiGenerateSSHKeys =>
            id >>> fanout(
                generate_ssh_handler,
                update_progress_handler,
            ) >>> merge,
    }
}

// 2. Replace pattern matching with routing DSL
```

### Phase 3: Causality Enforcement (Priority: MEDIUM)

```rust
// 1. Add time indices to events
pub struct KeyEvent<const T: Time> {
    event_type: KeyEventType,
    timestamp: T,  // Compile-time constant
}

// 2. Causality proof in type signatures
pub fn handle_event<const T1: Time, const T2: Time>(
    cause: KeyEvent<T1>,
) -> KeyEvent<T2>
where
    T2 > T1,  // Compile-time check
{
    // ...
}
```

### Phase 4: Feedback Loops (Priority: LOW)

```rust
// 1. Implement feedback combinator
pub fn aggregate_loop<State, Event>(
    initial: State,
) -> impl SignalFunction<Event, (State, Vec<Event>)>
where
    State: Decoupled,
{
    feedback(|event, state| {
        let new_state = apply_event(state, event);
        let new_events = generate_events(&new_state);
        (new_state, new_events)
    })
}
```

## Compliance Checklist

Use this checklist to verify compliance with n-ary FRP axioms:

- [ ] **A1**: All signals typed with kind (Event/Step/Continuous)
- [ ] **A2**: Signal functions operate on signal vectors
- [ ] **A3**: Update function is decoupled (no future dependencies)
- [ ] **A4**: Causality guaranteed at type level
- [ ] **A5**: All functions total (no panics, Result types)
- [ ] **A6**: Routing uses compositional primitives (>>>, ***, &&&)
- [ ] **A7**: Events stored as change prefixes
- [ ] **A8**: Feedback loops use safe combinator
- [ ] **A9**: Compositional laws hold (property tests)
- [ ] **A10**: Continuous time semantics documented

## Testing Requirements

Each axiom must be verified through tests:

```rust
#[test]
fn axiom_a3_decoupling() {
    // Test: Output at time t does not depend on input after t
    let model = Model::default();
    let intent_t1 = Intent::at_time(10.0);
    let intent_t2 = Intent::at_time(20.0);

    let (model1, _) = update(model.clone(), intent_t1);
    let (model2, _) = update(model.clone(), intent_t2);

    // Changing future event should not affect past output
    assert_eq!(model1, model2);
}

#[test]
fn axiom_a9_composition_associativity() {
    // Test: (f >>> g) >>> h = f >>> (g >>> h)
    let f = route_1;
    let g = route_2;
    let h = route_3;

    let lhs = compose(compose(f, g), h);
    let rhs = compose(f, compose(g, h));

    prop_assert_eq!(lhs, rhs);
}
```

## References

- **N-ary FRP Paper**: "Safe and Efficient Functional Reactive Programming" (original paper)
- **cim-domain**: Event sourcing patterns in CIM
- **MVI_IMPLEMENTATION_GUIDE.md**: Current MVI architecture
- **HEXAGONAL_ARCHITECTURE.md**: Ports and adapters

## Appendix: Mathematical Foundations

### Category Theory Basis

N-ary FRP is grounded in **Arrows** (generalized monads):

```
class Arrow arr where
    arr :: (a -> b) -> arr a b           -- lift pure function
    (>>>) :: arr a b -> arr b c -> arr a c  -- composition
    first :: arr a b -> arr (a, c) (b, c)   -- parallel

    -- Laws:
    -- arr id >>> f = f
    -- f >>> arr id = f
    -- (f >>> g) >>> h = f >>> (g >>> h)
```

Signal functions form an Arrow category:

```rust
impl Arrow for SignalFunction {
    fn arr<F: Fn(A) -> B>(f: F) -> Self { /* lift */ }
    fn compose(self, g: Self) -> Self { /* >>> */ }
    fn first(self) -> Self { /* *** */ }
}
```

### Denotational Semantics

Signals denote functions from time to values:

```
⟦Signal T⟧ = Time → T

⟦Event T⟧ = Time → [T]  -- list of occurrences

⟦Step T⟧ = Time → T  -- piecewise constant
```

Signal functions denote continuous functions:

```
⟦SF A B⟧ = (Time → A) → (Time → B)
```

### Operational Semantics (Sampling)

Implementation uses discrete sampling:

```
sample : (Time → T) → SamplingRate → [(Time, T)]
```

But semantics remain continuous (sampling theorem).

---

**This document is MANDATORY for all cim-keys development. Violations of these axioms must be corrected immediately.**
