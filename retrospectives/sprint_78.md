<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 78 Retrospective: Complete PKI Chain State Machine

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Wire complete PKI chain through state machines: IntermediateCA → LeafCerts → YubiKey provisioning

## What Was Accomplished

### Extended PKI Bootstrap State Machine

Added handlers for the full PKI chain progression:

```
Uninitialized
    ↓ (Sprint 77)
RootCAPlanned → RootCAGenerated
    ↓ (Sprint 78)
IntermediateCAPlanned → IntermediateCAGenerated
    ↓
LeafCertsGenerated
    ↓
YubiKeysProvisioned
    ↓
ExportReady → Bootstrapped
```

### Added IntermediateCA Message Variants

```rust
// IntermediateCA State Machine Transitions (Sprint 78)
PkiPlanIntermediateCA {
    subject: CertificateSubject,
    validity_years: u32,
    path_len: Option<u32>,
},
PkiExecuteIntermediateCAGeneration,
PkiIntermediateCAGenerationComplete {
    intermediate_ca_id: Uuid,
},
```

### Added LeafCert Message Variants

```rust
// LeafCert State Machine Transitions (Sprint 78)
PkiPlanLeafCert {
    subject: CertificateSubject,
    validity_years: u32,
    person_id: Option<Uuid>,
},
PkiExecuteLeafCertGeneration,
PkiLeafCertGenerationComplete {
    leaf_cert_id: Uuid,
},
```

### Added YubiKey Provisioning State Machine

Added per-YubiKey state tracking with HashMap:

```rust
// In CimKeysApp model
yubikey_states: HashMap<String, YubiKeyProvisioningState>,
```

Added YubiKey state machine messages:

```rust
YubiKeyDetected { serial: String, firmware_version: String },
YubiKeyAuthenticated { serial: String, pin_retries: u8 },
YubiKeyPINChanged { serial: String },
YubiKeyProvisioningComplete { serial: String },
```

### State Machine Guards

All handlers validate state machine guards before transitions:

| Handler | Guard | Error if Invalid |
|---------|-------|------------------|
| PkiPlanIntermediateCA | `can_plan_intermediate_ca()` | "Generate Root CA first" |
| PkiExecuteIntermediateCAGeneration | `can_generate_intermediate_ca()` | "Plan Intermediate CA first" |
| PkiPlanLeafCert | `can_generate_leaf_cert()` | "Generate Intermediate CA first" |
| YubiKeyDetected | `can_provision_yubikey()` | "Generate leaf certificates first" |
| YubiKeyAuthenticated | `state.can_authenticate()` | "Detect YubiKey first" |
| YubiKeyPINChanged | `state.can_change_pin()` | "Authenticate first" |

### Multi-Certificate Tracking

IntermediateCA and LeafCerts track multiple certificates:

```rust
Message::PkiIntermediateCAGenerationComplete { intermediate_ca_id } => {
    let intermediate_ca_ids = match &self.pki_state {
        PKIBootstrapState::IntermediateCAGenerated { intermediate_ca_ids } => {
            let mut ids = intermediate_ca_ids.clone();
            ids.push(intermediate_ca_id);
            ids
        }
        _ => vec![intermediate_ca_id],
    };
    self.pki_state = PKIBootstrapState::IntermediateCAGenerated { intermediate_ca_ids };
}
```

## State Machine Flow Diagram

```
                    PKI Bootstrap State Machine
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  Uninitialized                                                  │
│       │                                                         │
│       ▼ can_plan_root_ca()                                      │
│  RootCAPlanned ─────────────────► RootCAGenerated               │
│                can_generate_root_ca()      │                    │
│                                            ▼ can_plan_intermediate_ca()
│                               IntermediateCAPlanned             │
│                                            │                    │
│                                            ▼ can_generate_intermediate_ca()
│                               IntermediateCAGenerated           │
│                                            │                    │
│                                            ▼ can_generate_leaf_cert()
│                               LeafCertsGenerated                │
│                                            │                    │
│                                            ▼ can_provision_yubikey()
│  ┌─────────────────────────────────────────┴──────────────────┐ │
│  │           YubiKey Provisioning State Machine               │ │
│  │  Detected → Authenticated → PINChanged → ... → Sealed      │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                            │                    │
│                                            ▼                    │
│                               YubiKeysProvisioned               │
│                                            │                    │
│                                            ▼ can_prepare_export()
│                               ExportReady                       │
│                                            │                    │
│                                            ▼ can_export()       │
│                               Bootstrapped                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~280 |
| Tests passing | 1,072 |
| Files modified | 1 (gui.rs) |
| New Message variants | 10 |
| New state machine handlers | 10 |
| State machine guards used | 7 |

## What Went Well

1. **Consistent pattern** - All handlers follow same guard → transition → log pattern
2. **Multi-cert tracking** - Vec accumulation pattern works for multiple CAs/certs
3. **Two-level state machines** - PKI state machine gates YubiKey provisioning
4. **Per-device tracking** - HashMap<serial, YubiKeyProvisioningState> for multiple YubiKeys
5. **All tests pass** - No regressions

## Lessons Learned

1. **State machine guards enforce order** - Can't skip steps in PKI chain
2. **Optional planning states** - LeafCerts go directly from IntermediateCAGenerated (no LeafCertPlanned state)
3. **Nested state machines** - YubiKey has its own per-device state machine within the PKI flow

## Technical Notes

### State Machine Dependencies

The PKI state machine enforces this dependency chain:

```
Root CA → Intermediate CA → Leaf Certs → YubiKey → Export
```

Each step requires the previous step to be complete before it can proceed.

### YubiKey State Machine Independence

Each YubiKey has its own independent provisioning state:

```rust
yubikey_states: HashMap<String, YubiKeyProvisioningState>
```

This allows provisioning multiple YubiKeys in parallel, each at different stages.

## Related Sprints

- Sprint 74: Wired GUI delegation to CQRS aggregate
- Sprint 75: Wired AddPerson/AddLocation to CQRS aggregate
- Sprint 76: Wired CreateOrganizationUnit/CreateServiceAccount to CQRS aggregate
- Sprint 77: Wired PKI handlers to PKIBootstrapState state machine (RootCA)
- Sprint 78: Wired full PKI chain (IntermediateCA, LeafCerts, YubiKey) (this sprint)

## Next Steps

1. **Export workflow** - Add ExportReady and Bootstrapped state transitions
2. **Actual crypto implementation** - Wire IntermediateCA passphrase handler to `generate_intermediate_ca()`
3. **YubiKey full workflow** - Implement remaining states (ManagementKeyRotated, SlotPlanned, etc.)
4. **State persistence** - Save PKI and YubiKey states to projection for recovery
5. **UI integration** - Update views to show current state machine status
