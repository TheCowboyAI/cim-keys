<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 77 Retrospective: State Machine Driven PKI Bootstrap

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Wire PKI handlers to PKIBootstrapState state machine for deterministic workflow enforcement

## What Was Accomplished

### Added PKI State Machine to GUI Model

Extended the `CimKeysApp` struct to track the PKI bootstrap workflow state:

```rust
// src/gui.rs - Model
pub struct CimKeysApp {
    // ... existing fields ...

    // PKI Bootstrap State Machine (Sprint 77)
    pki_state: crate::state_machines::workflows::PKIBootstrapState,
}
```

### Added New Message Variants

Created three new Message variants for state machine transitions:

```rust
// PKI State Machine Transitions (Sprint 77)
PkiPlanRootCA {
    subject: crate::state_machines::workflows::CertificateSubject,
    validity_years: u32,
    yubikey_serial: String,
},
PkiExecuteRootCAGeneration,
PkiRootCAGenerationComplete {
    root_ca_cert_id: Uuid,
    root_ca_key_id: Uuid,
},
```

### Implemented State Machine Handlers

Added handlers with explicit state guards:

1. **PkiPlanRootCA Handler**
   - Validates `can_plan_root_ca()` guard
   - Transitions: `Uninitialized` → `RootCAPlanned`
   - Captures subject, validity_years, yubikey_serial

2. **PkiExecuteRootCAGeneration Handler**
   - Validates `can_generate_root_ca()` guard
   - Extracts planning data from `RootCAPlanned` state
   - Shows passphrase dialog for seed derivation
   - Logs state machine execution

3. **PkiRootCAGenerationComplete Handler**
   - Transitions: `RootCAPlanned` → `RootCAGenerated`
   - Captures cert_id, key_id, generated_at timestamp

### Integrated State Machine with Existing Handler

Modified `Message::RootCAGenerated` to also update the state machine:

```rust
// Sprint 77: Update PKI state machine if in RootCAPlanned state
if matches!(self.pki_state, PKIBootstrapState::RootCAPlanned { .. }) {
    self.pki_state = PKIBootstrapState::RootCAGenerated {
        root_ca_cert_id: cert_id_uuid,
        root_ca_key_id: key_id,
        generated_at: chrono::Utc::now(),
    };
    tracing::info!(
        "PKI state machine transitioned to RootCAGenerated (cert: {}, key: {})",
        cert_id_uuid, key_id
    );
}
```

## State Machine Flow

```
┌─────────────────┐
│  Uninitialized  │ ← Initial state
└────────┬────────┘
         │ can_plan_root_ca() guard
         ▼
┌─────────────────┐
│  RootCAPlanned  │ ← PkiPlanRootCA message
│  - subject      │
│  - validity_yrs │
│  - yubikey_sn   │
└────────┬────────┘
         │ can_generate_root_ca() guard
         ▼
   PassphraseDialog
         │
         ▼
   Async Generation
         │
         ▼
┌─────────────────┐
│ RootCAGenerated │ ← RootCAGenerated message
│  - cert_id      │
│  - key_id       │
│  - generated_at │
└─────────────────┘
```

## Event Flow Comparison

```rust
// Before Sprint 77 (passphrase dialog directly triggers generation)
PassphraseDialogMessage::Submit
    → async crypto::x509::generate_root_ca()
    → Message::RootCAGenerated(cert)
    → GUI updates (no state tracking)

// After Sprint 77 (state machine governs workflow)
Message::PkiPlanRootCA { subject, ... }
    → validates can_plan_root_ca()
    → PKIBootstrapState::RootCAPlanned

Message::PkiExecuteRootCAGeneration
    → validates can_generate_root_ca()
    → extracts subject from state
    → shows passphrase dialog

PassphraseDialogMessage::Submit (RootCA purpose)
    → async crypto::x509::generate_root_ca()
    → Message::RootCAGenerated(cert)

Message::RootCAGenerated handler
    → checks if in RootCAPlanned state
    → transitions to RootCAGenerated
    → logs state transition
```

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~90 |
| Tests passing | 1,072 |
| Files modified | 1 (gui.rs) |
| New message handlers | 3 |
| State machine guards used | 3 |

## What Went Well

1. **Clean state guards** - Using `can_plan_root_ca()`, `can_generate_root_ca()` ensures only valid transitions
2. **Backward compatible** - Existing passphrase-triggered flow still works
3. **Explicit state tracking** - `pki_state` field now visible in GUI model
4. **Logging** - State transitions logged with tracing::info for debugging
5. **All tests pass** - No regressions introduced

## Lessons Learned

1. **State machine guards prevent invalid states** - The guards enforce: plan before generate
2. **Two trigger paths** - Both old (passphrase direct) and new (state machine) paths work
3. **Personal keys need IntermediateCA** - State machine enforces proper PKI hierarchy

## Technical Notes

### PersonalKeys State Machine Integration

The `PKIBootstrapState` requires `IntermediateCAGenerated` before `LeafCertsGenerated` (personal keys):

```rust
pub fn can_generate_leaf_cert(&self) -> bool {
    matches!(
        self,
        PKIBootstrapState::IntermediateCAGenerated { .. }
            | PKIBootstrapState::LeafCertsGenerated { .. }
    )
}
```

Current PersonalKeys handler generates a placeholder self-signed cert (noted as TODO). Full state machine integration for personal keys requires IntermediateCA support first.

## Related Sprints

- Sprint 74: Wired GUI delegation to CQRS aggregate
- Sprint 75: Wired AddPerson/AddLocation to CQRS aggregate
- Sprint 76: Wired CreateOrganizationUnit/CreateServiceAccount to CQRS aggregate
- Sprint 77: Wired PKI handlers to PKIBootstrapState state machine (this sprint)

## Next Steps

1. **IntermediateCA state machine** - Wire intermediate CA generation through state machine
2. **LeafCerts/PersonalKeys** - Wire personal keys through state machine (requires IntermediateCA)
3. **YubiKey provisioning** - Wire YubiKeyProvisioningState state machine
4. **Export workflow** - Wire export preparation through state machine
5. **State persistence** - Persist PKI state to projection for recovery
