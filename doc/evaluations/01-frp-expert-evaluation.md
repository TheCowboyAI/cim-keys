# N-ary FRP Axiom Compliance Evaluation for cim-keys

**Evaluator:** FRP Expert
**Date:** 2026-01-02
**Overall FRP Compliance: 55% (5.5/10 axioms)**

---

## Section 1: N-ary FRP Axiom Evaluation

### A1: Multi-Kinded Signal Types

**Status: PARTIAL COMPLIANCE (40%)**

**Evidence:**

File: `/git/thecowboyai/cim-keys/src/mvi/intent.rs` (lines 8-15, 534-629)

The codebase has implemented signal kind classification at the **runtime level**:

```rust
// Lines 562-573
pub fn is_event_signal(&self) -> bool {
    matches!(
        self,
        Intent::UiOrganizationNameChanged(_)
        | Intent::UiOrganizationIdChanged(_)
        // ... other step signals
    ) == false
}
```

**What Works:**
- Intent variants are classified as Event (87%) or Step (13%)
- `SignalKindMarker` enum provides runtime discrimination
- Good documentation of signal semantics

**What Violates:**
- No type-level distinction: `Signal<EventKind, T>` vs `Signal<StepKind, T>` not enforced
- `Intent` is a single enum mixing all temporal kinds
- Continuous signals not implemented (0%)

**Required Fix:**
```rust
// Type-level signal kinds (compile-time enforcement)
pub trait SignalKind { type Semantics; }
pub struct EventKind;
pub struct StepKind;
pub struct ContinuousKind;

// Signals parameterized by kind
pub struct Signal<K: SignalKind, T> {
    _kind: PhantomData<K>,
    value: T,
}

// Separate Intent types per kind
pub enum EventIntent { UiGenerateRootCAClicked, /* ... */ }
pub enum StepIntent { UiOrganizationNameChanged(String), /* ... */ }
```

---

### A2: Signal Vector Composition

**Status: PARTIAL COMPLIANCE (50%)**

**Evidence:**

File: `/git/thecowboyai/cim-keys/src/fold.rs` (lines 144-173)

The fold.rs module provides the combinators:

```rust
// Lines 152-158
pub fn parallel<A, B, C, D, F, G>(f: F, g: G) -> impl Fn((A, C)) -> (B, D)
where
    F: Fn(A) -> B,
    G: Fn(C) -> D,
{
    move |(a, c)| (f(a), g(c))
}
```

**What Works:**
- `parallel`, `fanout`, `first`, `second` combinators exist
- Arrow law tests verify composition
- `FoldCapability` enables n-ary folds

**What Violates:**

File: `/git/thecowboyai/cim-keys/src/mvi/update.rs` (lines 23-31)

```rust
pub fn update(
    model: Model,
    intent: Intent,  // Single signal, NOT a vector!
    storage: Arc<dyn StoragePort>,
    // ...
) -> (Model, Task<Intent>)
```

The update function takes a single `Intent`, not a signal vector.

---

### A3: Decoupled Signal Functions

**Status: COMPLIANT (90%)**

The update function is properly decoupled - output at time t depends only on Model and Intent at time t (not future).

---

### A4: Causality Guarantees

**Status: PARTIAL COMPLIANCE (40%)**

Intents do NOT include correlation/causation IDs. Causality tracked in some domain events but NOT enforced at type level.

---

### A5: Totality and Well-Definedness

**Status: COMPLIANT (85%)**

All Intent variants handled (no panics). Foldable trait returns `R` not `Option<R>`.

---

### A6: Explicit Routing at Reactive Level

**Status: VIOLATION (30%)**

**MAJOR VIOLATION**: The update function is a giant 1000+ line match statement.

File: `/git/thecowboyai/cim-keys/src/mvi/update.rs` (lines 32-1068)

File: `/git/thecowboyai/cim-keys/src/lifting.rs` (lines 640-724) - `apply_properties` uses pattern matching with downcast chain.

---

### A7: Change Prefixes as Event Logs

**Status: COMPLIANT (80%)**

Events are modeled as discrete occurrences with timestamps.

---

### A8: Type-Safe Feedback Loops

**Status: NOT IMPLEMENTED (10%)**

No `feedback` combinator exists.

---

### A9: Semantic Preservation Under Composition

**Status: PARTIAL COMPLIANCE (60%)**

Arrow law tests exist but combinators not used in actual routing.

---

### A10: Continuous Time Semantics

**Status: NOT IMPLEMENTED (5%)**

No continuous signals implemented.

---

## Section 2: OOP Anti-Pattern Identification

### Anti-Pattern 1: Pattern Matching for Routing

**Status: MAJOR VIOLATION**

File: `/git/thecowboyai/cim-keys/src/mvi/update.rs` (lines 32-1068)

1000+ lines of pattern matching. Cannot compose routing.

**Severity: HIGH**

---

### Anti-Pattern 2: Downcast Chains

**Status: MAJOR VIOLATION**

File: `/git/thecowboyai/cim-keys/src/lifting.rs` (lines 640-724)
File: `/git/thecowboyai/cim-keys/src/gui/folds/query/edit_fields.rs` (lines 140-237)

Downcasting defeats type safety.

**Severity: HIGH**

---

## Section 3: Corrective Action Plan

| Priority | Task | Complexity | Files Affected |
|----------|------|------------|----------------|
| **P0** | Replace downcast chain in `apply_properties` with trait-based fold | Medium | `src/lifting.rs:640-724` |
| **P0** | Replace downcast chain in `legacy_extract_via_downcast` | Medium | `src/gui/folds/query/edit_fields.rs:140-237` |
| **P1** | Factor update.rs into arrow-composed routes | High | `src/mvi/update.rs` |
| **P1** | Add correlation_id/causation_id/timestamp to Intent | Low | `src/mvi/intent.rs` |
| **P2** | Implement type-level signal kinds | High | `src/mvi/intent.rs`, new `signals.rs` |
| **P3** | Implement ContinuousKind signals | Medium | New module |
| **P3** | Add feedback combinator with Decoupled proof | High | `src/fold.rs` |

---

## Section 4: Summary Compliance Matrix

| Axiom | Status | Score | Primary Gap |
|-------|--------|-------|-------------|
| A1: Multi-Kinded Signals | Partial | 40% | Runtime only, not type-level |
| A2: Signal Vector Composition | Partial | 50% | Single Intent, not vector |
| A3: Decoupled Functions | Compliant | 90% | No Decoupled marker trait |
| A4: Causality Guarantees | Partial | 40% | No IDs on Intent |
| A5: Totality | Compliant | 85% | Minor .unwrap() usage |
| A6: Explicit Routing | Violation | 30% | Giant match statement |
| A7: Change Prefixes | Compliant | 80% | Not formalized as type |
| A8: Feedback Loops | Not Implemented | 10% | No feedback combinator |
| A9: Semantic Preservation | Partial | 60% | Laws not applied |
| A10: Continuous Time | Not Implemented | 5% | No continuous signals |

**Overall Score: 49/100 (55% weighted)**
