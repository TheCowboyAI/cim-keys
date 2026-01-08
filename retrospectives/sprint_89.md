<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 89 Retrospective: PKIBootstrapState Transition Methods

**Date**: 2026-01-08
**Focus**: Add complete state transition infrastructure to PKIBootstrapState workflow state machine

## Summary

This sprint added proper state transition validation and methods to `PKIBootstrapState`, aligning it with the established pattern used by aggregate state machines (KeyState, CertificateState, etc.).

**Sprint Context**: This is Sprint 1 of 4 in the workflow state machine completion plan:
- Sprint 89: PKIBootstrapState (this sprint) ✅
- Sprint 90: YubiKeyProvisioningState
- Sprint 91: ExportWorkflowState
- Sprint 92: Final integration tests and retrospective

## Problem Statement

The codebase had 15 state machines, but workflow state machines (PKIBootstrapState, YubiKeyProvisioningState, ExportWorkflowState) lacked the `can_transition_to()` method and explicit transition methods that aggregate state machines provided. This created an architectural inconsistency.

**Before:**
- Individual `can_*` guard methods only
- No consolidated transition validation
- No transition methods returning Result<State, Error>
- No terminal state detection

**After:**
- `can_transition_to(&self, target: &State) -> bool`
- `is_terminal()` / `is_complete()` methods
- 8 transition methods with validation
- `PKIBootstrapError` enum for error handling

## Completed Work

### 1. Terminal State Detection
```rust
pub fn is_terminal(&self) -> bool {
    matches!(self, PKIBootstrapState::Bootstrapped { .. })
}

pub fn is_complete(&self) -> bool {
    matches!(self, PKIBootstrapState::Bootstrapped { .. })
}
```

### 2. Transition Validation (`can_transition_to`)

Added consolidated transition validation covering 11 valid transitions:

| From State | To State | Notes |
|------------|----------|-------|
| Uninitialized | RootCAPlanned | Initial planning |
| RootCAPlanned | RootCAGenerated | Offline ceremony |
| RootCAGenerated | IntermediateCAPlanned | Optional planning step |
| RootCAGenerated | IntermediateCAGenerated | Skip planning |
| IntermediateCAPlanned | IntermediateCAGenerated | After planning |
| IntermediateCAGenerated | LeafCertsGenerated | Generate leaves |
| LeafCertsGenerated | LeafCertsGenerated | Add more leaves |
| LeafCertsGenerated | YubiKeysProvisioned | Provision keys |
| YubiKeysProvisioned | YubiKeysProvisioned | Add more keys |
| YubiKeysProvisioned | ExportReady | Prepare manifest |
| ExportReady | Bootstrapped | Complete (terminal) |

### 3. Transition Methods

Added 8 transition methods that validate state and return `Result<PKIBootstrapState, PKIBootstrapError>`:

- `plan_root_ca(subject, validity_years, yubikey_serial)`
- `generate_root_ca(cert_id, key_id, generated_at)`
- `plan_intermediate_ca(subject, validity_years, path_len)`
- `generate_intermediate_ca(intermediate_ca_ids)`
- `generate_leaf_certs(leaf_cert_ids)`
- `provision_yubikeys(yubikey_serials)`
- `prepare_export(manifest_id)`
- `complete_bootstrap(export_location, export_checksum, bootstrapped_at)`

### 4. Error Handling

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PKIBootstrapError {
    #[error("Invalid state transition from {current} on event {event}: {reason}")]
    InvalidTransition { current: String, event: String, reason: String },

    #[error("Terminal state reached: {0}")]
    TerminalState(String),

    #[error("State validation failed: {0}")]
    ValidationFailed(String),
}
```

### 5. Tests

Added 6 unit tests covering:
- Initial state guards (`test_pki_bootstrap_initial_state`)
- Transition validation (`test_pki_bootstrap_can_transition_to`)
- Full workflow end-to-end (`test_pki_bootstrap_full_workflow`)
- Invalid transition errors (`test_pki_bootstrap_invalid_transition`)
- Validation errors (`test_pki_bootstrap_validation_errors`)
- State descriptions (`test_pki_bootstrap_state_descriptions`)

## Architecture Decisions

### 1. Match Pattern Consistency

Used the same `match (self, target)` pattern as aggregate state machines for consistency:
```rust
pub fn can_transition_to(&self, target: &PKIBootstrapState) -> bool {
    match (self, target) {
        (PKIBootstrapState::Uninitialized, PKIBootstrapState::RootCAPlanned { .. }) => true,
        // ...
        _ => false,
    }
}
```

### 2. Self-Loop Transitions

Allowed self-loops for additive states:
- `LeafCertsGenerated → LeafCertsGenerated` (add more leaf certs)
- `YubiKeysProvisioned → YubiKeysProvisioned` (provision more YubiKeys)

### 3. Skip Optional Steps

Allowed skipping optional planning steps:
- `RootCAGenerated → IntermediateCAGenerated` (skip IntermediateCAPlanned)

## Files Modified

| File | Changes |
|------|---------|
| `src/state_machines/workflows.rs` | +280 lines: transition methods, error enum, tests |
| `src/state_machines/mod.rs` | +1 line: export PKIBootstrapError |

## Test Results

- 6 new tests added
- All 6 tests pass
- Total library tests: 1095 passing

## What Worked Well

1. **Pattern Consistency**: Following the established aggregate state machine pattern made implementation straightforward
2. **Comprehensive Validation**: Both transition validation and data validation in methods
3. **Clear Error Messages**: Error variants include context (current state, event, reason)

## Lessons Learned

1. **Self-loops matter**: Some workflow states need to allow adding more items without advancing
2. **Skip paths needed**: Real workflows may skip optional steps (like planning)
3. **Terminal detection simple**: For workflows, usually just one terminal state

## Next Steps

- Sprint 90: Add same infrastructure to `YubiKeyProvisioningState` (9 states)
- Sprint 91: Add same infrastructure to `ExportWorkflowState` (7 states)
- Sprint 92: Integration tests and final retrospective

## Sprint Metrics

| Metric | Value |
|--------|-------|
| Lines Added | 280 |
| Methods Added | 11 |
| Error Variants | 3 |
| Tests Added | 6 |
| Tests Passing | 1095 |
| Valid Transitions | 11 |
| Terminal States | 1 |
