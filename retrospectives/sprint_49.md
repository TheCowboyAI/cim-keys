# Sprint 49 Retrospective: PKI Bounded Context Extraction

**Date**: 2026-01-06
**Sprint Duration**: 1 session
**Status**: COMPLETED

## Sprint Goal

Extract the PKI (Public Key Infrastructure) bounded context from gui.rs into a dedicated domain module, following the pattern established in Sprint 48.

## Context

Sprint 48 successfully established the Organization bounded context with the message delegation pattern. Sprint 49 applies the same pattern to PKI operations, which represents a larger and more complex domain.

## What Was Accomplished

### 1. Directory Structure Created

```
src/gui/
├── pki/
│   ├── mod.rs                 # Module exports (26 lines)
│   └── key_generation.rs      # PKI bounded context (605 lines)
```

### 2. PkiMessage Enum

Created domain-specific message enum with 55+ variants organized by sub-domain:

| Sub-domain | Message Count | Purpose |
|------------|---------------|---------|
| Root CA Operations | 3 | Generate, toggle, result |
| Intermediate CA | 4 | Name, unit selection, generate, toggle |
| Server Certificate | 6 | CN, SANs, signing CA, location, generate, toggle |
| Certificate Metadata | 6 | Organization, OU, locality, state, country, validity |
| SSH Keys | 1 | Generate |
| General Key Ops | 5 | Generate all, progress, result, toggles |
| GPG Keys | 9 | User ID, type, length, expiry, generate, list, toggle, results |
| Key Recovery | 8 | Toggle, passphrase, org ID, verify, recover, results |
| Client Cert (mTLS) | 4 | CN, email, generate, result |
| Multi-Purpose Keys | 5 | Toggle, person, purposes, generate, result |
| Root Passphrase | 4 | Changed, confirm, visibility, random |
| Graph-Based PKI | 3 | Certificates loaded, generate from graph, personal keys |

### 3. PkiState Struct

Created domain state struct with 45+ fields matching CimKeysApp:

```rust
pub struct PkiState {
    // Key generation progress (3 fields)
    // Certificate generation (7 fields)
    // Certificate metadata (6 fields)
    // Loaded certificates (1 field)
    // Collapsible sections (6 fields)
    // GPG state (6 fields)
    // Key recovery (6 fields)
    // Client certificate (2 fields)
    // Multi-purpose keys (3 fields)
    // Root passphrase (3 fields)
}
```

### 4. Helper Methods

Added utility methods to PkiState:
- `new()` - Creates state with sensible defaults (365 days validity, 4096-bit keys)
- `is_passphrase_valid()` - Validates passphrase complexity requirements

### Files Modified

| File | Change |
|------|--------|
| `src/gui/pki/mod.rs` | NEW: PKI module exports (26 lines) |
| `src/gui/pki/key_generation.rs` | NEW: PKI bounded context (605 lines) |
| `src/gui.rs` | Added pki module, Pki variant, delegation (~110 lines added) |

## Design Decisions

### 1. Random Passphrase Generation

Implemented local random passphrase generation in the update function:
```rust
PkiMessage::GenerateRandomPassphrase => {
    use rand::Rng;
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%^&*"
        .chars()
        .collect();
    let passphrase: String = (0..20)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect();
    state.root_passphrase = passphrase.clone();
    state.root_passphrase_confirm = passphrase;
    Task::none()
}
```

Note: Excludes ambiguous characters (0, O, l, 1, I) for readability.

### 2. Passphrase Validation

Centralized validation logic in PkiState:
- Minimum 12 characters
- Requires uppercase, lowercase, digit, and special character
- Passwords must match

### 3. Section Toggle Pattern

All collapsible sections use consistent toggle pattern:
```rust
PkiMessage::Toggle*Section => {
    state.*_collapsed = !state.*_collapsed;
    Task::none()
}
```

## Tests Added

| Test | Purpose |
|------|---------|
| `test_pki_state_default` | Default values |
| `test_pki_state_new` | Constructor defaults |
| `test_passphrase_validation` | Complexity requirements |
| `test_toggle_sections` | Section collapse/expand |
| `test_cert_metadata_updates` | Field updates |
| `test_multi_purpose_key_toggle` | Purpose set operations |
| `test_key_generation_progress` | Progress tracking |

## Metrics

| Metric | Value |
|--------|-------|
| New files created | 2 |
| Lines added | ~730 |
| Tests passing | 998 (up from 991) |
| Message variants extracted | 55+ |
| State fields extracted | 45+ |

## What Worked Well

1. **Pattern Replication**: Sprint 48 pattern applied seamlessly to PKI domain
2. **Clean Separation**: PKI concerns now isolated from organization logic
3. **Test Coverage**: 7 new tests for PKI domain validation
4. **Helper Methods**: `is_passphrase_valid()` encapsulates complexity requirements

## Lessons Learned

1. **Defaults Matter**: Using `PkiState::new()` with sensible defaults (365 days, 4096 bits) prevents invalid configurations
2. **Local Operations**: Some operations (random passphrase) can be handled entirely in the domain module
3. **State Size**: PKI domain has more state than Organization (~45 vs ~30 fields)

## Best Practices Updated

46. **Sensible Defaults**: Implement `new()` constructor with secure default values
47. **Validation Methods**: Add validation helpers like `is_passphrase_valid()` to state structs
48. **Ambiguous Char Exclusion**: Random passwords should exclude 0/O/l/1/I for readability
49. **Section Toggle Pattern**: Use consistent `Toggle*Section` → `*_collapsed = !*_collapsed`

## Progress Summary

| Sprint | Domain | Messages | State Fields | Tests |
|--------|--------|----------|--------------|-------|
| 48 | Organization | 50+ | 30+ | 991 |
| 49 | PKI | 55+ | 45+ | 998 |
| **Total** | **2 domains** | **105+** | **75+** | **998** |

## Next Steps (Sprint 50)

1. **Extract YubiKeyMessage**: YubiKey lifecycle management (~40 messages)
2. **Extract NatsMessage**: NATS infrastructure (~25 messages)
3. **Continue reducing gui.rs complexity**

## Sprint Summary

Sprint 49 successfully extracted the PKI bounded context:
- Created pki module with 55+ message variants and 45+ state fields
- Added 7 new tests (total: 998 passing)
- Implemented passphrase validation and random generation helpers
- Pattern from Sprint 48 proven reusable for larger domains

The delegation pattern scales well to complex domains. Two bounded contexts (Organization + PKI) now have clean separation from the main gui.rs module.
