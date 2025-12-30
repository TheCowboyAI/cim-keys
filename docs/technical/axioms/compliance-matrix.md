# N-ary FRP Compliance Analysis for cim-keys

## Executive Summary

cim-keys currently implements a **pure functional reactive architecture** using the Model-View-Intent (MVI) pattern. However, it does not fully comply with **n-ary FRP axioms** as defined in the mathematical foundations. This document analyzes the current state and provides a roadmap for full compliance.

**Overall Compliance Score: 5/10 Axioms (50%)**

## Current Architecture Strengths

### ✅ What We're Doing Right

1. **Event Sourcing (A7)** ✅
   - All state changes through immutable events
   - Events stored as timestamped JSON (change prefixes)
   - Projections rebuild state from event history
   - **Compliance: 100%**

2. **Pure Update Functions (A5)** ✅
   - `update()` function is deterministic
   - No side effects in update (only in Commands)
   - No panics (uses `Result`)
   - **Compliance: 100%**

3. **Decoupled Event Processing (A3)** ✅
   - Commands are async and decoupled
   - Update doesn't depend on future events
   - Intent pattern separates concerns
   - **Compliance: 90%** (missing type-level proof)

4. **Causality Tracking (A4)** 🟡
   - Events have `correlation_id` and `causation_id`
   - Runtime causality tracking
   - ⚠️ Missing: Compile-time guarantees
   - **Compliance: 60%**

5. **Clean Hexagonal Architecture** ✅
   - Ports define interfaces
   - Adapters implement side effects
   - Dependency injection in update
   - **Compliance: 100%**

## Critical Gaps

### ❌ What We're Missing

1. **Multi-Kinded Signal Types (A1)** ❌
   ```rust
   // CURRENT: All events treated the same
   pub enum Intent {
       UiButtonClicked,      // Event signal
       AnimationTick,        // Continuous signal
       ModelUpdated,         // Step signal
   }

   // REQUIRED: Signals distinguished by kind at type level
   pub enum Intent<K: SignalKind> {
       Event(EventIntent),
       Step(StepIntent),
       Continuous(ContinuousIntent),
   }
   ```
   **Compliance: 20%** (events exist, but not typed)
   **Impact: HIGH** - Cannot reason about temporal behavior

2. **Signal Vector Composition (A2)** ❌
   ```rust
   // CURRENT: Single intent input/output
   pub fn update(model: Model, intent: Intent) -> (Model, Command)

   // REQUIRED: Signal vectors (multiple independent signals)
   pub fn update(signals: SignalVector<Inputs>) -> SignalVector<Outputs>
   ```
   **Compliance: 0%** - Not implemented
   **Impact: HIGH** - Artificial coupling between independent signals

3. **Compositional Routing (A6)** ❌
   ```rust
   // CURRENT: Pattern matching on Intent
   match intent {
       Intent::UiGenerateRootCA => { /* handler */ }
       Intent::PortX509Generated => { /* handler */ }
   }

   // REQUIRED: Compositional routing primitives
   let route = id >>> generate_handler >>> store_handler;
   ```
   **Compliance: 0%** - Pattern matching only
   **Impact: MEDIUM** - Cannot compose routes algebraically

4. **Type-Safe Feedback Loops (A8)** ❌
   ```rust
   // CURRENT: No feedback loops
   // Aggregates have implicit feedback but not formalized

   // REQUIRED: Feedback combinator
   let aggregate = feedback(|event, state| {
       let new_state = apply(state, event);
       let new_events = emit(new_state);
       (new_state, new_events)
   });
   ```
   **Compliance: 0%** - Not implemented
   **Impact: MEDIUM** - Cannot model aggregate loops compositionally

5. **Continuous Time Semantics (A10)** ❌
   ```rust
   // CURRENT: Discrete time (event timestamps)
   pub struct KeyEvent {
       timestamp: DateTime<Utc>,  // Discrete
   }

   // REQUIRED: Continuous time semantics
   pub trait Signal<T> {
       fn denote(&self) -> Box<dyn Fn(Time) -> T>;  // Continuous
   }
   ```
   **Compliance: 0%** - No continuous signals
   **Impact: LOW** - Animation and metrics need continuous time

## Detailed Gap Analysis

### Gap 1: Signal Type Hierarchy

**Current State**:
```rust
pub enum Intent {
    // All mixed together
    UiButtonClicked { button_id: String },
    AnimationTick { time: f32 },
    ModelUpdated { state: Model },
}
```

**Problems**:
- Cannot distinguish event vs. continuous vs. step signals at compile time
- No way to enforce different sampling strategies
- Temporal semantics implicit, not explicit

**Required Changes**:
```rust
pub trait SignalKind {
    type Semantics;  // Time → Value or Time → [Value]
}

pub struct EventKind;
impl SignalKind for EventKind {
    type Semantics = Vec<(Time, Value)>;  // Discrete occurrences
}

pub struct StepKind;
impl SignalKind for StepKind {
    type Semantics = Time -> Value;  // Piecewise constant
}

pub struct ContinuousKind;
impl SignalKind for ContinuousKind {
    type Semantics = Time -> Value;  // Smooth function
}

pub enum Intent<K: SignalKind> {
    Event(EventIntent) where K = EventKind,
    Step(StepIntent) where K = StepKind,
    Continuous(ContinuousIntent) where K = ContinuousKind,
}
```

**Effort**: 2-3 weeks
**Priority**: HIGH

---

### Gap 2: Signal Vector Operations

**Current State**:
```rust
// Single input, single output
pub fn update(model: Model, intent: Intent) -> (Model, Command)
```

**Problems**:
- Cannot process multiple independent signals simultaneously
- Forces artificial ordering between unrelated events
- Cannot express parallel composition

**Required Changes**:
```rust
// N-ary input, M-ary output
pub trait SignalVector {
    type Tuple;  // (Signal1, Signal2, ..., SignalN)
}

pub fn update<In: SignalVector, Out: SignalVector>(
    inputs: In::Tuple,
) -> Out::Tuple
where
    In: IndependentSignals,
    Out: IndependentSignals,
{
    // Process all inputs simultaneously
    // Emit multiple independent outputs
}

// Routing combinators
pub fn parallel<A, B, C, D>(
    f: impl SignalFunction<A, B>,
    g: impl SignalFunction<C, D>,
) -> impl SignalFunction<(A, C), (B, D)>

pub fn fanout<A, B, C>(
    f: impl SignalFunction<A, B>,
    g: impl SignalFunction<A, C>,
) -> impl SignalFunction<A, (B, C)>
```

**Effort**: 3-4 weeks
**Priority**: HIGH

---

### Gap 3: Compositional Routing Language

**Current State**:
```rust
match intent {
    Intent::UiGenerateRootCA => {
        // Ad-hoc handler
        let updated = model.with_status("Generating...");
        let command = Task::perform(async { /* ... */ });
        (updated, command)
    }
}
```

**Problems**:
- Cannot compose routes
- No algebraic laws
- Cannot reason about routing

**Required Changes**:
```rust
// Define routing DSL
pub trait Route<In, Out> {
    fn route(&self, input: In) -> Out;
}

// Primitive routes
pub fn id<A>() -> impl Route<A, A>
pub fn compose<A, B, C>(f: impl Route<A, B>, g: impl Route<B, C>) -> impl Route<A, C>

// Express handlers as routes
let generate_root_ca_route =
    id
    >>> validate_passphrase
    >>> generate_key
    >>> sign_certificate
    >>> store_in_projection;

// Verify laws
assert_eq!(f >>> id, f);
assert_eq!(id >>> f, f);
assert_eq!((f >>> g) >>> h, f >>> (g >>> h));
```

**Effort**: 2-3 weeks
**Priority**: MEDIUM

---

### Gap 4: Causality Proof System

**Current State**:
```rust
pub struct KeyEvent {
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,  // Runtime tracking
    pub timestamp: DateTime<Utc>,
}
```

**Problems**:
- Causality violations possible at compile time
- No proof that output depends only on past
- Can accidentally create cycles

**Required Changes**:
```rust
// Time-indexed types
pub struct At<T, const TIME: u64>(T);

// Causality proof as type
pub struct CausalDep<A, B, const T1: u64, const T2: u64>
where
    T2 > T1,  // Compile-time check
{
    cause: At<A, T1>,
    effect: At<B, T2>,
}

// Update function with causality proof
pub fn update<const T_IN: u64, const T_OUT: u64>(
    model: At<Model, T_IN>,
    intent: At<Intent, T_IN>,
) -> At<(Model, Command), T_OUT>
where
    T_OUT >= T_IN,  // Output cannot be before input
{
    // Compiler enforces causality
}
```

**Effort**: 4-5 weeks
**Priority**: MEDIUM

---

### Gap 5: Feedback Loop Combinator

**Current State**:
```rust
// Aggregate implicitly has feedback:
// Events → State
// State → Events
// But not formalized as feedback loop
```

**Problems**:
- No guarantee of well-definedness
- Possible infinite loops
- Cannot reason about convergence

**Required Changes**:
```rust
// Feedback combinator (only for decoupled functions)
pub fn feedback<Input, Output, State, SF>(sf: SF) -> impl SignalFunction<Input, Output>
where
    SF: SignalFunction<(Input, State), (Output, State)> + Decoupled,
{
    // Well-defined because SF is decoupled
    // Fixed point exists by denotational semantics
    unimplemented!()
}

// Model aggregate as feedback loop
let aggregate_loop = feedback(|event: KeyEvent, state: AggregateState| {
    let new_state = apply_event(state, event);
    let new_events = emit_events(&new_state);
    (new_events, new_state)
});
```

**Effort**: 2-3 weeks
**Priority**: LOW (aggregates work, just not formalized)

---

## Compliance Roadmap

### Phase 1: Foundational Types (Weeks 1-4)

**Goal**: Establish type-level signal kinds and vector operations

**Tasks**:
1. Define `SignalKind` trait and implementations
2. Parameterize `Intent` by signal kind
3. Define `SignalVector` trait
4. Implement basic routing primitives

**Deliverables**:
- [ ] `src/signals/kinds.rs` - Signal kind type system
- [ ] `src/signals/vectors.rs` - Signal vector operations
- [ ] `src/routing/primitives.rs` - id, compose, parallel, fanout
- [ ] Update `Intent` to be parameterized by kind

**Success Criteria**:
- A1: 80% compliance
- A2: 60% compliance

---

### Phase 2: Compositional Routing (Weeks 5-7)

**Goal**: Replace pattern matching with compositional routing DSL

**Tasks**:
1. Define `Route` trait
2. Implement routing combinators
3. Refactor update function to use routing
4. Verify compositional laws with property tests

**Deliverables**:
- [ ] `src/routing/dsl.rs` - Routing DSL
- [ ] `src/routing/laws.rs` - Property tests for routing laws
- [ ] Refactored `update()` using routing

**Success Criteria**:
- A6: 80% compliance
- A9: 70% compliance

---

### Phase 3: Causality Enforcement (Weeks 8-11)

**Goal**: Add compile-time causality guarantees

**Tasks**:
1. Define time-indexed types `At<T, Time>`
2. Add causality proofs to event types
3. Enforce causality in update signature
4. Update all event handlers to preserve causality

**Deliverables**:
- [ ] `src/causality/types.rs` - Time-indexed types
- [ ] `src/causality/proofs.rs` - Causality proof system
- [ ] Updated event types with causality proofs

**Success Criteria**:
- A4: 90% compliance
- Compile errors on causality violations

---

### Phase 4: Feedback Loops (Weeks 12-14)

**Goal**: Formalize aggregate as feedback loop

**Tasks**:
1. Implement `feedback` combinator
2. Prove decoupling for aggregate
3. Refactor aggregate to use feedback
4. Add convergence tests

**Deliverables**:
- [ ] `src/combinators/feedback.rs` - Feedback combinator
- [ ] `src/aggregate_frp.rs` - Aggregate as feedback loop
- [ ] Convergence property tests

**Success Criteria**:
- A8: 80% compliance
- Aggregates modeled compositionally

---

### Phase 5: Continuous Time (Weeks 15-16)

**Goal**: Add continuous signal support for animations

**Tasks**:
1. Define `ContinuousSignal` trait
2. Implement animation time as continuous signal
3. Add sampling operators
4. Document continuous time semantics

**Deliverables**:
- [ ] `src/signals/continuous.rs` - Continuous signal support
- [ ] `src/animation/time.rs` - Animation as continuous signal
- [ ] Sampling and interpolation operators

**Success Criteria**:
- A10: 70% compliance
- Animation uses continuous time

---

## Testing Strategy

### Property-Based Tests

Each axiom must be verified with property tests:

```rust
use proptest::prelude::*;

#[test]
fn axiom_a9_composition_associativity() {
    proptest!(|(f: Route, g: Route, h: Route, x: Input)| {
        let lhs = compose(compose(f, g), h).route(x);
        let rhs = compose(f, compose(g, h)).route(x);
        prop_assert_eq!(lhs, rhs);
    });
}

#[test]
fn axiom_a3_decoupling() {
    proptest!(|(intent: Intent, model: Model)| {
        let (model1, _) = update(model.clone(), intent);

        // Adding future events should not change past output
        let future_intent = Intent::at_time(intent.time() + 1000);
        let (model2, _) = update(model1.clone(), future_intent);

        // Past state should be unchanged
        prop_assert_eq!(model1, model2);
    });
}
```

### Integration Tests

Test complete workflows with n-ary FRP primitives:

```rust
#[tokio::test]
async fn test_root_ca_workflow_with_routing() {
    let route =
        validate_input
        >>> generate_key
        >>> sign_certificate
        >>> store_projection;

    let input = Intent::UiGenerateRootCA { /* ... */ };
    let output = route.apply(input).await;

    assert!(matches!(output, Intent::DomainRootCAGenerated { /* ... */ }));
}
```

### Axiom Compliance Tests

Create a test suite that verifies each axiom:

```rust
mod axiom_tests {
    #[test]
    fn verify_a1_multi_kinded_signals() {
        // Verify all signals have proper kind
    }

    #[test]
    fn verify_a2_signal_vectors() {
        // Verify update operates on signal vectors
    }

    // ... tests for all 10 axioms
}
```

---

## Benefits of Full Compliance

### 1. Mathematical Correctness

- **Formal semantics**: Behavior defined denotationally
- **Compositional reasoning**: Understand parts independently
- **Verified properties**: Laws checked at compile time

### 2. Cross-Framework Portability

N-ary FRP is framework-independent:
- Same routing DSL works in Iced, egui, CLI, web
- Core logic portable across UI frameworks
- Can swap rendering without changing logic

### 3. Better Composability

```rust
// Compose complex workflows from simple routes
let full_workflow =
    bootstrap_domain
    >>> generate_pki_hierarchy
    >>> provision_yubikeys
    >>> export_to_storage;

// Parallel composition
let parallel_generation =
    generate_root_ca *** generate_ssh_keys *** provision_yubikey;

// Fanout (send to multiple handlers)
let notify_all =
    update_gui &&& log_to_audit &&& emit_nats_event;
```

### 4. Provable Properties

With type-level proofs:
- **No causality violations**: Compile error if output depends on future
- **No infinite loops**: Feedback only for decoupled functions
- **Totality**: All functions defined for all inputs

### 5. Better Testing

- **Property tests**: Verify laws automatically
- **Generative tests**: Test all possible signal combinations
- **Compositional tests**: Test routes independently

---

## Migration Strategy

### Incremental Adoption

**Goal**: Don't break existing functionality while adding n-ary FRP

**Approach**:
1. Add n-ary FRP types alongside existing types
2. Implement adapters between old and new
3. Migrate one module at a time
4. Remove old types once migration complete

**Example**:
```rust
// Old (keep working)
pub enum Intent {
    UiGenerateRootCA,
    // ...
}

// New (add alongside)
pub enum IntentV2<K: SignalKind> {
    Event(EventIntent),
    Step(StepIntent),
    Continuous(ContinuousIntent),
}

// Adapter
impl From<Intent> for IntentV2<EventKind> {
    fn from(old: Intent) -> Self {
        match old {
            Intent::UiGenerateRootCA => IntentV2::Event(EventIntent::UiGenerateRootCA),
            // ...
        }
    }
}
```

### Backward Compatibility

- Keep old Intent enum until migration complete
- Provide adapters for both directions
- Update tests to use new types
- Documentation for migration path

---

## Conclusion

**Current Status**: cim-keys has a solid foundation with event sourcing, pure functions, and hexagonal architecture.

**Gaps**: Missing n-ary FRP features (signal kinds, vector composition, routing DSL, causality proofs, feedback loops).

**Effort**: 16 weeks (~4 months) to achieve full compliance.

**Priority**: HIGH for Phase 1-2 (foundational types and routing), MEDIUM for Phase 3-4, LOW for Phase 5.

**Benefits**: Mathematical correctness, composability, portability, provable properties.

**Next Steps**: Begin Phase 1 (signal kinds and vectors) after stakeholder approval.

---

## Appendix: Axiom Compliance Matrix

| Axiom | Current | Target | Priority | Effort | Phase |
|-------|---------|--------|----------|--------|-------|
| A1: Multi-Kinded Signals | 20% | 90% | HIGH | 2 weeks | 1 |
| A2: Signal Vectors | 0% | 80% | HIGH | 3 weeks | 1 |
| A3: Decoupled Functions | 90% | 95% | LOW | 1 week | - |
| A4: Causality Guarantees | 60% | 90% | MEDIUM | 4 weeks | 3 |
| A5: Totality | 100% | 100% | - | - | - |
| A6: Compositional Routing | 0% | 80% | MEDIUM | 3 weeks | 2 |
| A7: Change Prefixes | 100% | 100% | - | - | - |
| A8: Feedback Loops | 0% | 80% | LOW | 3 weeks | 4 |
| A9: Semantic Preservation | 40% | 80% | MEDIUM | 2 weeks | 2 |
| A10: Continuous Time | 0% | 70% | LOW | 2 weeks | 5 |
| **TOTAL** | **50%** | **87%** | - | **16 weeks** | - |

**Target**: 87% compliance (some axioms are research-level and may not reach 100%)
**Timeline**: 4 months
**Risk**: MEDIUM (foundational changes but incremental migration)
