<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 90 Retrospective: YubiKeyProvisioningState Transition Methods

**Date**: 2026-01-08
**Focus**: Add complete state transition infrastructure to YubiKeyProvisioningState workflow state machine

## Summary

This sprint added proper state transition validation and methods to `YubiKeyProvisioningState`, continuing the established pattern from Sprint 89 (PKIBootstrapState). This is a linear workflow representing the YubiKey initialization and provisioning sequence.

**Sprint Context**: This is Sprint 2 of 4 in the workflow state machine completion plan:
- Sprint 89: PKIBootstrapState ✅
- Sprint 90: YubiKeyProvisioningState (this sprint) ✅
- Sprint 91: ExportWorkflowState
- Sprint 92: Final integration tests and retrospective

## Problem Statement

`YubiKeyProvisioningState` had 9 states and individual `can_*` guard methods but lacked:
- Consolidated `can_transition_to()` validation
- Transition methods returning `Result<State, Error>`
- Error enum for state transition failures
- Terminal state detection methods

**Before:**
- Individual `can_*` guard methods only
- No consolidated transition validation
- No transition methods
- No terminal state detection

**After:**
- `can_transition_to(&self, target: &State) -> bool`
- `is_terminal()` / `is_complete()` methods
- 8 transition methods with validation
- `YubiKeyProvisioningError` enum for error handling

## Completed Work

### 1. Terminal State Detection
```rust
pub fn is_terminal(&self) -> bool {
    matches!(self, YubiKeyProvisioningState::Sealed { .. })
}

pub fn is_complete(&self) -> bool {
    matches!(self, YubiKeyProvisioningState::Sealed { .. })
}
```

### 2. Transition Validation (`can_transition_to`)

Added consolidated transition validation covering 8 valid transitions in a **linear sequence**:

| From State | To State | Notes |
|------------|----------|-------|
| Detected | Authenticated | First authentication |
| Authenticated | PINChanged | Secure the device |
| PINChanged | ManagementKeyRotated | Rotate management key |
| ManagementKeyRotated | SlotsPlanned | Plan slot usage |
| SlotsPlanned | KeysGenerated | Generate keys in slots |
| KeysGenerated | CertificatesImported | Import certificates |
| CertificatesImported | KeysAttested | Attest key provenance |
| KeysAttested | Sealed | Complete (terminal) |

**Key Difference from PKIBootstrapState**: This is a strictly linear workflow with no self-loops or skip paths. Each step must be completed in sequence.

### 3. Transition Methods

Added 8 transition methods that validate state and return `Result<YubiKeyProvisioningState, YubiKeyProvisioningError>`:

- `authenticate(pin_retries_remaining: u8)`
- `change_pin(new_pin_hash: String)`
- `rotate_management_key(algorithm: PivAlgorithm)`
- `plan_slots(slot_plan: HashMap<PivSlot, SlotPlan>)`
- `generate_keys(slot_keys: HashMap<PivSlot, Vec<u8>>)`
- `import_certificates(slot_certs: HashMap<PivSlot, Uuid>)`
- `attest_keys(attestation_chain_verified: bool, attestation_cert_ids: Vec<Uuid>)`
- `seal(sealed_at: DateTime<Utc>, final_config_hash: String)`

### 4. Error Handling

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum YubiKeyProvisioningError {
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
- Initial state guards (`test_yubikey_provisioning_initial_state`)
- Transition validation (`test_yubikey_provisioning_can_transition_to`)
- Full workflow end-to-end (`test_yubikey_provisioning_full_workflow`)
- Invalid transition errors (`test_yubikey_provisioning_invalid_transition`)
- Validation errors (`test_yubikey_provisioning_validation_errors`)
- State descriptions (`test_yubikey_provisioning_state_descriptions`)

## Architecture Decisions

### 1. Linear Workflow Pattern

Unlike PKIBootstrapState which has self-loops and skip paths, YubiKeyProvisioningState is a **strictly linear** workflow:

```
Detected → Authenticated → PINChanged → ManagementKeyRotated
    → SlotsPlanned → KeysGenerated → CertificatesImported
    → KeysAttested → Sealed
```

This reflects the physical reality of YubiKey provisioning: each step must happen in order, and once a key is sealed, the provisioning is complete.

### 2. Data Preservation Pattern

Each transition method preserves data from the previous state:
```rust
pub fn change_pin(&self, new_pin_hash: String) -> Result<...> {
    match self {
        YubiKeyProvisioningState::Authenticated { serial, firmware, .. } => {
            Ok(YubiKeyProvisioningState::PINChanged {
                serial: serial.clone(),      // Preserved
                firmware: firmware.clone(),  // Preserved
                pin_hash: new_pin_hash,      // New
            })
        }
        // ...
    }
}
```

### 3. Validation in Transition Methods

Each method validates its input data before transitioning:
```rust
pub fn plan_slots(&self, slot_plan: HashMap<PivSlot, SlotPlan>) -> Result<...> {
    // Validate slot plan is not empty
    if slot_plan.is_empty() {
        return Err(YubiKeyProvisioningError::ValidationFailed(
            "Slot plan cannot be empty".to_string()
        ));
    }
    // ...
}
```

## Files Modified

| File | Changes |
|------|---------|
| `src/state_machines/workflows.rs` | +280 lines: transition methods, error enum, tests |
| `src/state_machines/mod.rs` | Export YubiKeyProvisioningError (already done in Sprint 89) |

## Test Results

- 6 new tests added
- All 6 tests pass
- Total workflow tests: 12 passing (6 PKI + 6 YubiKey)
- Total library tests: 1101 passing

## What Worked Well

1. **Pattern Reuse**: Following the PKIBootstrapState pattern from Sprint 89 made implementation straightforward
2. **Linear Workflow Simplicity**: No self-loops or skip paths means simpler validation logic
3. **Data Preservation**: Cloning previous state data ensures context is not lost during transitions
4. **Comprehensive Validation**: Both state and data validation in each transition method

## Lessons Learned

1. **Linear vs. Branching**: Not all workflows need self-loops or skip paths - sometimes linear is correct
2. **Data Accumulation**: Each state builds on the previous by preserving and adding fields
3. **Terminal State Semantics**: "Sealed" is the right terminal state name for hardware security

## Comparison: PKIBootstrapState vs YubiKeyProvisioningState

| Aspect | PKIBootstrapState | YubiKeyProvisioningState |
|--------|-------------------|--------------------------|
| Workflow Type | Branching with self-loops | Strictly linear |
| States | 7 | 9 |
| Valid Transitions | 11 | 8 |
| Self-loops | Yes (LeafCertsGenerated, YubiKeysProvisioned) | No |
| Skip Paths | Yes (can skip IntermediateCAPlanned) | No |
| Terminal State | Bootstrapped | Sealed |
| Purpose | PKI hierarchy generation | YubiKey hardware initialization |

## Next Steps

- Sprint 91: Add same infrastructure to `ExportWorkflowState` (7 states)
- Sprint 92: Integration tests and final retrospective

## Sprint Metrics

| Metric | Value |
|--------|-------|
| Lines Added | 280 |
| Methods Added | 11 |
| Error Variants | 3 |
| Tests Added | 6 |
| Workflow Tests Passing | 12 |
| Library Tests Passing | 1101 |
| Valid Transitions | 8 |
| Terminal States | 1 |
