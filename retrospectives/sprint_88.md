<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 88 Retrospective: Certificate Selection UI, RFC 5280 Compliance & State Machine

**Date**: 2026-01-08
**Focus**: Certificate selection for YubiKey import with RFC 5280 validation and proper state machine architecture

## Summary

This sprint enhanced the YubiKey certificate import workflow by implementing:
1. RFC 5280 compliance validation for X.509 certificates
2. Certificate selection UI when multiple leaf certificates are available
3. Automatic validation on certificate generation
4. **CertificateImportState state machine** for proper DDD aggregate behavior
5. **CertificateImport events** for event sourcing
6. **Command handlers** (aggregate behavior) for import workflow

## State Machine Ecosystem Context

CIM-Keys uses a comprehensive state machine architecture. `CertificateImportState` is the 15th state machine added to the system:

### Workflow State Machines (Cross-Aggregate Sagas)
| State Machine | States | Purpose |
|---------------|--------|---------|
| `PKIBootstrapState` | 7 | Root CA → Intermediate → Leaf certificate workflow |
| `YubiKeyProvisioningState` | 6 | YubiKey initialization and slot provisioning |
| `ExportWorkflowState` | 5 | Export to encrypted storage |

### Aggregate Lifecycle State Machines

**Phase 1: CRITICAL Security & Identity**
| State Machine | States | Lifecycle |
|---------------|--------|-----------|
| `KeyState` | 8 | Generated → Active → Rotated/Revoked → Archived |
| `CertificateState` | 8 | Requested → Issued → Active → Expired/Revoked |
| `PolicyState` | 5 | Draft → Active → Deprecated → Archived |

**Phase 2: Core Domain**
| State Machine | States | Lifecycle |
|---------------|--------|-----------|
| `PersonState` | 5 | Invited → Active → Suspended → Departed |
| `OrganizationState` | 4 | Planned → Active → Archived |
| `LocationState` | 4 | Proposed → Active → Decommissioned |
| `RelationshipState` | 6 | Proposed → Active → Severed |

**Phase 3: Infrastructure & Export**
| State Machine | States | Lifecycle |
|---------------|--------|-----------|
| `ManifestState` | 6 | Initializing → Complete → Verified |
| `NatsOperatorState` | 5 | Created → Active → Revoked |
| `NatsAccountState` | 5 | Created → Active → Suspended |
| `NatsUserState` | 5 | Created → Active → Revoked |
| `YubiKeyState` | 6 | Detected → Provisioned → Active → Retired |

**Phase 4: Import Workflows (NEW - Sprint 88)**
| State Machine | States | Lifecycle |
|---------------|--------|-----------|
| `CertificateImportState` | 10 | NoCertificateSelected → Validated → Importing → Imported |

### Pattern Consistency

All state machines follow the established patterns:
- States are enums with associated data
- Transitions validated via `can_*()` methods
- Events trigger state changes
- Terminal states prevent further modifications
- All states implement `Serialize`/`Deserialize` for event sourcing

## Completed Work

### 1. RFC 5280 Compliance Validation (`src/crypto/rfc5280.rs`)

Created a comprehensive RFC 5280 validation module that checks:

- **Version**: Must be v3 (2) for certificates with extensions
- **Serial Number**: Positive integer, at most 20 octets, non-zero
- **Signature Algorithm**: Supported algorithms (RSA/SHA*, ECDSA/SHA*, Ed25519/Ed448)
- **Issuer**: Non-empty distinguished name
- **Validity**: notBefore <= notAfter, not expired, not yet valid
- **Subject**: Non-empty DN for CAs, can be empty with SAN for end-entity
- **Extensions**: BasicConstraints, KeyUsage, SubjectAltName validation

Key structures:
- `Rfc5280Error` - Enumeration of validation error types
- `Rfc5280ValidationResult` - Errors, warnings, and certificate metadata
- `CertificateMetadata` - Extracted certificate information for UI display (with Serialize/Deserialize)

### 2. Certificate Selection UI (`src/gui.rs:13107-13206`)

Enhanced the YubiKey import section with:

- **Pick List**: When multiple leaf certificates exist, shows a dropdown with:
  - Validation status indicator (✓/✗)
  - Subject CN or identifier
  - Fingerprint prefix for disambiguation

- **Validation Status Display**: Shows "RFC 5280 ✓" or "RFC 5280 ✗ (N)" with colored indicator

- **Import Gate**: Prevents importing certificates with validation errors

### 3. CertificateImportState State Machine (`src/state_machines/certificate_import.rs`)

Created a comprehensive state machine with 10 states:

```
NoCertificateSelected
       │
       ▼ CertificateSelectedForImport
CertificateSelected { cert_id, slot }
       │
       ▼ CertificateValidationStarted
Validating { cert_id }
       │
       ├──▶ ValidationFailed { errors }
       │
       ▼ CertificateValidationSucceeded { metadata }
Validated { cert_id, metadata }
       │
       ▼ PinRequested
AwaitingPin { cert_id, slot }
       │
       ├──▶ PinFailed { attempts_remaining }
       │
       ▼ PinVerified
Importing { cert_id, slot }
       │
       ├──▶ ImportFailed { reason }
       │
       ▼ CertificateImportSucceeded
Imported { cert_id, slot, imported_at }
```

Key features:
- State query methods (`can_select_certificate()`, `is_validated()`, etc.)
- State transition validation (`can_transition_to()`)
- Data accessors for cert_id, yubikey_serial, slot, metadata
- PIN attempt tracking with retry limits

### 4. CertificateImport Events (`src/events/certificate_import.rs`)

Created 13 event types following the event sourcing pattern:

| Event | Purpose |
|-------|---------|
| `CertificateSelectedForImport` | Certificate chosen for import |
| `CertificateDeselected` | Selection cancelled |
| `CertificateValidationStarted` | RFC 5280 validation initiated |
| `CertificateValidationSucceeded` | Validation passed with metadata |
| `CertificateValidationFailed` | Validation failed with errors |
| `PinEntryRequested` | PIN prompt shown to user |
| `PinVerified` | PIN accepted by YubiKey |
| `PinEntryFailed` | Wrong PIN entered |
| `CertificateImportStarted` | Import operation begun |
| `CertificateImportSucceeded` | Certificate stored on YubiKey |
| `CertificateImportFailed` | Import operation failed |
| `CertificateImportAborted` | User cancelled workflow |
| `WorkflowReset` | Workflow cleared for next import |

All events include:
- `correlation_id` - Links related events
- `causation_id` - What triggered this event
- Timestamps for temporal ordering
- Domain context (cert_id, yubikey_serial, slot)

### 5. Command Handlers (`src/commands/certificate_import.rs`)

Created aggregate behavior as command handlers:

| Command | Handler | Events Produced |
|---------|---------|-----------------|
| `SelectCertificateForImport` | `handle_select_certificate()` | `CertificateSelectedForImport` |
| `ValidateCertificate` | `handle_validate_certificate()` | `ValidationSucceeded/Failed` |
| (internal) | `handle_request_pin()` | `PinEntryRequested` |
| (internal) | `handle_pin_result()` | `PinVerified/PinEntryFailed` |
| `ImportCertificate` | `handle_import_result()` | `ImportSucceeded/Failed` |
| `AbortImport` | `handle_abort()` | `CertificateImportAborted` |
| `ResetWorkflow` | `handle_reset()` | `WorkflowReset` |

Each handler:
- Validates state allows the operation
- Creates appropriate event(s)
- Returns new state after transition
- Returns errors for invalid operations

### 6. Integration Points

- Added `CertificateImport` variant to `DomainEvent` enum
- Updated all match statements in:
  - `src/projection/jetstream.rs` (3 matches)
  - `src/adapters/nats_client.rs` (1 match)
  - `src/domain/nats/publisher.rs` (1 match + detailed event routing)
  - `src/domain/nats/replay.rs` (2 matches)
  - `src/events/mod.rs` (2 matches)
  - `src/gui.rs` (1 match)

- Added `certificate_import_states` HashMap to `CimKeysApp` for tracking state per YubiKey

## Architecture Decisions

### 1. Behaviors = State Transitions

Following the user's guidance that "state transitions are the functionality an Aggregate follows", the state machine encodes:
- **What states exist** (the workflow lifecycle)
- **What transitions are valid** (the business rules)
- **What data each state carries** (the context needed for operations)

### 2. Commands → Events Pattern

Rather than direct state mutation, the aggregate pattern:
1. Command arrives (user intent)
2. Handler validates against current state
3. Events are produced (domain facts)
4. New state derived from events
5. UI observes state changes

### 3. Metadata Serialization

`CertificateMetadata` now derives `Serialize` and `Deserialize` to support:
- State machine serialization (event sourcing)
- Event payloads
- Potential persistence to manifest

### 4. NATS Subject Patterns

Certificate import events use hierarchical subjects:
```
keys.events.certificate.import.selected
keys.events.certificate.import.validation-started
keys.events.certificate.import.validation-succeeded
keys.events.certificate.import.pin-requested
keys.events.certificate.import.succeeded
```

### 5. State Machine Interactions

`CertificateImportState` integrates with existing state machines:

```
┌─────────────────────────────────────────────────────────────────┐
│                    CertificateImportState                       │
│  (NoCertificateSelected → Validated → Importing → Imported)     │
└───────────────────────────┬─────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│ CertificateState │ │ YubiKeyState  │   │   KeyState    │
│  (must be in   │   │  (must be    │   │  (associated  │
│  Active state) │   │  Active)     │   │  key lifecycle│
└───────────────┘   └───────────────┘   └───────────────┘
```

**Cross-State Machine Invariants:**
- Certificate must exist in `CertificateState::Active` before import
- YubiKey must be in `YubiKeyState::Active` to receive certificate
- On successful import, `YubiKeyState.slots` map updated with cert reference
- Import failure doesn't change `CertificateState` (certificates are immutable)

## Files Created/Modified

| File | Changes |
|------|---------|
| `src/crypto/rfc5280.rs` | **NEW** - RFC 5280 validation module (656 lines) |
| `src/crypto/mod.rs` | Added rfc5280 module exports |
| `src/state_machines/certificate_import.rs` | **NEW** - State machine (608 lines) |
| `src/state_machines/mod.rs` | Added certificate_import module and exports |
| `src/events/certificate_import.rs` | **NEW** - Import events (470 lines) |
| `src/events/mod.rs` | Added CertificateImport to DomainEvent enum |
| `src/commands/certificate_import.rs` | **NEW** - Command handlers (560 lines) |
| `src/commands/mod.rs` | Added certificate_import module |
| `src/projection/jetstream.rs` | Added CertificateImport pattern to matches |
| `src/adapters/nats_client.rs` | Added CertificateImport pattern |
| `src/domain/nats/publisher.rs` | Added detailed CertificateImport event routing |
| `src/domain/nats/replay.rs` | Added CertificateImport patterns |
| `src/gui.rs` | Certificate selection UI, state tracking, event display |
| `src/gui/certificate/management.rs` | Added 3 new CertificateMessage variants |

## Test Results

- 4 RFC 5280 unit tests
- 6 state machine tests
- 4 command handler tests
- 4 event tests
- All **1089** library tests pass
- Build compiles successfully with `--features gui`

## What Worked Well

1. **Modular Validation**: Separating RFC 5280 validation into its own module made it easy to test and reuse
2. **State Machine Pattern**: Clear workflow states make behavior predictable and testable
3. **Event Sourcing**: All state changes captured as immutable facts
4. **Command Handlers**: Pure functions that validate and produce events
5. **Comprehensive State Queries**: Methods like `can_validate()` make UI logic cleaner

## Lessons Learned

1. **State Machine First**: Start with the state diagram before implementing handlers
2. **Match Exhaustiveness**: Adding new enum variants requires updating all match statements
3. **Metadata Serialization**: Include `Serialize/Deserialize` early if data will be in events
4. **PivSlot Naming**: Use `PivSlot::Signature` not `DigitalSignature`
5. **Private Fields**: Use accessor methods (`errors()`) not direct field access (`errors`)

## Next Steps

Potential enhancements for future sprints:
- Full UI integration using state machine to drive workflow display
- Event persistence to manifest for audit trail
- Certificate details view showing full validation results
- PIN dialog integration with state machine transitions
- NATS publication of import events
- Certificate chain validation (issuer chain verification)

## Dependencies

- `x509-parser` - X.509 certificate parsing
- `pem` - PEM format handling
- `sha2` - SHA-256 fingerprint calculation
- `chrono` - DateTime handling for validity checks
- `thiserror` - Error type derivation

## Sprint Metrics

| Metric | Value |
|--------|-------|
| Files Created | 4 |
| Files Modified | 10 |
| Lines Added | ~2,300 |
| Tests Added | 18 |
| Tests Passing | 1089 |
| New State Machine States | 10 |
| New Event Types | 13 |
| New Command Handlers | 7 |
| **Total State Machines in System** | **15** |
| **Total Aggregate States** | **~75** |
