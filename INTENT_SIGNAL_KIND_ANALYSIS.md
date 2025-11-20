# Intent Signal Kind Analysis

This document classifies each `Intent` variant by its natural signal kind according to n-ary FRP Axiom A1 (Multi-Kinded Signals).

## Signal Kind Definitions

- **EventKind (◇)**: Discrete occurrences at specific time points
  - Button clicks, completions, failures
  - Semantics: `⟦Event T⟧(t) = [(t', x) | t' ≤ t, x : T]`

- **StepKind (□)**: Piecewise-constant values that change discretely
  - Form input values, application state
  - Semantics: `⟦Step T⟧(t) = T` (value at time t)

- **ContinuousKind**: Values defined at all times (smooth functions)
  - Animation time, interpolated positions
  - Semantics: `⟦Continuous T⟧(t) = T` (continuous function)

## Intent Classification

### UI-Originated Intents

| Intent Variant | Signal Kind | Rationale |
|----------------|-------------|-----------|
| `UiTabSelected(Tab)` | EventKind | Discrete click event at specific time |
| `UiCreateDomainClicked` | EventKind | Discrete button click |
| `UiLoadDomainClicked` | EventKind | Discrete button click |
| `UiOrganizationNameChanged(String)` | StepKind | Holds current input value (piecewise constant) |
| `UiOrganizationIdChanged(String)` | StepKind | Holds current input value (piecewise constant) |
| `UiAddPersonClicked` | EventKind | Discrete button click |
| `UiPersonNameChanged` | StepKind | Holds current person name value |
| `UiPersonEmailChanged` | StepKind | Holds current person email value |
| `UiRemovePersonClicked` | EventKind | Discrete button click |
| `UiGenerateRootCAClicked` | EventKind | Discrete button click |
| `UiGenerateIntermediateCAClicked` | EventKind | Discrete button click |
| `UiGenerateServerCertClicked` | EventKind | Discrete button click |
| `UiGenerateSSHKeysClicked` | EventKind | Discrete button click |
| `UiGenerateAllKeysClicked` | EventKind | Discrete button click |
| `UiExportClicked` | EventKind | Discrete button click |
| `UiProvisionYubiKeyClicked` | EventKind | Discrete button click |
| `UiPassphraseChanged(String)` | StepKind | Holds current passphrase value |
| `UiPassphraseConfirmChanged(String)` | StepKind | Holds current confirmation value |
| `UiDeriveMasterSeedClicked` | EventKind | Discrete button click |

### Graph-Originated Intents

| Intent Variant | Signal Kind | Rationale |
|----------------|-------------|-----------|
| `UiGraphCreateNode` | EventKind | Discrete creation event |
| `UiGraphCreateEdgeStarted` | EventKind | Discrete click on source node |
| `UiGraphCreateEdgeCompleted` | EventKind | Discrete click on target node |
| `UiGraphCreateEdgeCancelled` | EventKind | Discrete cancellation |
| `UiGraphNodeClicked` | EventKind | Discrete node selection |
| `UiGraphDeleteNode` | EventKind | Discrete deletion request |
| `UiGraphDeleteEdge` | EventKind | Discrete deletion request |
| `UiGraphEditNodeProperties` | EventKind | Discrete edit request |
| `UiGraphPropertyChanged` | StepKind | Holds current property value |
| `UiGraphPropertiesSaved` | EventKind | Discrete save action |
| `UiGraphPropertiesCancelled` | EventKind | Discrete cancel action |
| `UiGraphAutoLayout` | EventKind | Discrete layout request |

### Domain-Originated Intents

| Intent Variant | Signal Kind | Rationale |
|----------------|-------------|-----------|
| `DomainCreated` | EventKind | Discrete domain creation event |
| `PersonAdded` | EventKind | Discrete person addition event |
| `RootCAGenerated` | EventKind | Discrete completion event |
| `SSHKeyGenerated` | EventKind | Discrete completion event |
| `YubiKeyProvisioned` | EventKind | Discrete completion event |
| `MasterSeedDerived` | EventKind | Discrete completion event |
| `MasterSeedDerivationFailed` | EventKind | Discrete failure event |
| `DomainNodeCreated` | EventKind | Discrete node creation |
| `DomainEdgeCreated` | EventKind | Discrete edge creation |
| `DomainNodeDeleted` | EventKind | Discrete node deletion |
| `DomainNodeUpdated` | EventKind | Discrete update event |
| `DomainOrganizationCreated` | EventKind | Discrete creation event |
| `DomainOrgUnitCreated` | EventKind | Discrete creation event |
| `DomainLocationCreated` | EventKind | Discrete creation event |
| `DomainRoleCreated` | EventKind | Discrete creation event |
| `DomainPolicyCreated` | EventKind | Discrete creation event |
| `DomainPolicyBound` | EventKind | Discrete binding event |

### Port-Originated Intents (Async Responses)

| Intent Variant | Signal Kind | Rationale |
|----------------|-------------|-----------|
| `PortStorageWriteCompleted` | EventKind | Discrete completion event |
| `PortStorageWriteFailed` | EventKind | Discrete failure event |
| `PortX509RootCAGenerated` | EventKind | Discrete completion event |
| `PortX509IntermediateCAGenerated` | EventKind | Discrete completion event |
| `PortX509ServerCertGenerated` | EventKind | Discrete completion event |
| `PortX509GenerationFailed` | EventKind | Discrete failure event |
| `PortSSHKeypairGenerated` | EventKind | Discrete completion event |
| `PortSSHGenerationFailed` | EventKind | Discrete failure event |
| `PortYubiKeyDevicesListed` | EventKind | Discrete completion event |
| `PortYubiKeyKeyGenerated` | EventKind | Discrete completion event |
| `PortYubiKeyOperationFailed` | EventKind | Discrete failure event |

### System-Originated Intents

| Intent Variant | Signal Kind | Rationale |
|----------------|-------------|-----------|
| `SystemFileSelected` | EventKind | Discrete file selection |
| `SystemFilePickerCancelled` | EventKind | Discrete cancellation |
| `SystemErrorOccurred` | EventKind | Discrete error occurrence |
| `SystemClipboardUpdated` | EventKind | Discrete clipboard change |

### Error Intents

| Intent Variant | Signal Kind | Rationale |
|----------------|-------------|-----------|
| `ErrorOccurred` | EventKind | Discrete error occurrence |
| `ErrorDismissed` | EventKind | Discrete dismissal action |
| `NoOp` | EventKind | Discrete no-operation marker |

## Summary Statistics

- **EventKind**: 62 variants (87%)
- **StepKind**: 9 variants (13%)
- **ContinuousKind**: 0 variants (0%)

## Key Insights

### 1. Most Intents are Events
The vast majority of intents (87%) are discrete events representing user actions, domain occurrences, or async completions.

### 2. Step Signals for Input Values
Form input values (`*Changed` variants) are naturally step signals because they hold the current value of an input field between changes.

### 3. No Continuous Signals (Yet)
cim-keys doesn't currently use continuous signals. Potential future uses:
- Animation time for UI transitions
- Mouse position during graph node dragging
- Progress indicators (0.0 to 1.0)

### 4. Model is a Step Signal
The `Model` type itself is a step signal - it holds the current application state and changes only when events occur.

## Implementation Strategy

### Phase 1: Add Signal Kind Method (Week 2)

```rust
impl Intent {
    /// Get the signal kind for this intent variant
    pub fn signal_kind(&self) -> SignalKind {
        match self {
            // Step signals (form inputs)
            Intent::UiOrganizationNameChanged(_)
            | Intent::UiOrganizationIdChanged(_)
            | Intent::UiPersonNameChanged { .. }
            | Intent::UiPersonEmailChanged { .. }
            | Intent::UiPassphraseChanged(_)
            | Intent::UiPassphraseConfirmChanged(_)
            | Intent::UiGraphPropertyChanged { .. } => SignalKind::Step,

            // All others are event signals
            _ => SignalKind::Event,
        }
    }
}
```

### Phase 2: Type Aliases (Week 2)

```rust
use crate::signals::{Signal, EventKind, StepKind};

/// Event signal carrying an intent
pub type EventSignal = Signal<EventKind, Intent>;

/// Step signal carrying a value
pub type StepSignal<T> = Signal<StepKind, T>;

/// Model is a step signal (holds current state)
pub type ModelSignal = Signal<StepKind, Model>;
```

### Phase 3: Signal Vectors for Update (Week 3-4)

```rust
use crate::signals::{SignalVec2, EventKind, StepKind};

/// Update function signature with signal vectors
pub fn update(
    inputs: SignalVec2<StepKind, EventKind, Model, Intent>,
    // ... ports
) -> SignalVec2<StepKind, EventKind, Model, Intent> {
    let (model, intent) = inputs.split();
    // ... process
    SignalVec2::new(updated_model, result_intent)
}
```

### Phase 4: Routing DSL (Phase 2 of roadmap)

```rust
let workflow: Route<EventIntent, ModelUpdate> =
    validate_passphrase
    >>> generate_key
    >>> sign_certificate
    >>> store_projection;
```

## Alignment with N-ary FRP Axioms

### A1: Multi-Kinded Signals ✅
- All intents classified by kind
- Type-level distinction via `signal_kind()` method
- Future: Separate EventIntent and StepIntent types

### A2: Signal Vector Composition 🔄
- Prepared for signal vector operations
- Update function can use SignalVec2<StepKind, EventKind, Model, Intent>
- Enables n-ary signal functions

### A7: Change Prefixes as Event Logs ✅
- All EventKind intents are immutable events
- Timestamped (implicitly by MVI framework)
- Can be logged and replayed

## Next Steps

1. Add `signal_kind()` method to Intent
2. Create type aliases for common signal patterns
3. Update documentation with signal kind examples
4. Write tests verifying signal kind classification
5. Prepare for Week 3: Signal vector integration in update function

---

**Status**: Analysis complete, ready for implementation
**Date**: 2025-01-20
**Phase**: Week 2 - Parameterize Intent by Kind
