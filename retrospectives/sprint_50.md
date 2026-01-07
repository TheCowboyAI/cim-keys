# Sprint 50 Retrospective: YubiKey Bounded Context Extraction

**Date**: 2026-01-06
**Sprint Duration**: 1 session
**Status**: COMPLETED

## Sprint Goal

Extract the YubiKey bounded context from gui.rs into a dedicated domain module, following the pattern established in Sprints 48-49.

## Context

Sprint 49 successfully extracted the PKI bounded context. Sprint 50 applies the same pattern to YubiKey operations, which represents a hardware-focused domain with lifecycle management.

## What Was Accomplished

### 1. Directory Structure Created

```
src/gui/
├── yubikey/
│   ├── mod.rs               # Module exports (24 lines)
│   └── management.rs        # YubiKey bounded context (720 lines)
```

### 2. YubiKeyMessage Enum

Created domain-specific message enum with ~40 variants organized by sub-domain:

| Sub-domain | Message Count | Purpose |
|------------|---------------|---------|
| Device Detection | 2 | Detect devices, handle results |
| Person Assignment | 3 | Assign YubiKey to person, select for assignment |
| Provisioning | 3 | Provision YubiKey, set passphrase, results |
| Slot Management | 5 | Select slot, generate key for slot, results |
| PIN Management | 7 | Current/new/confirm PIN, change, unlock, reset |
| Management Key Ops | 5 | Current/new/confirm key, change, reset |
| PIV Reset | 2 | Reset PIV, handle result |
| Attestation | 3 | Generate attestation, verify, results |
| Domain Registration | 4 | Register with domain, unregister, results |
| Graph-Based Ops | 3 | Generate from graph, handle results |
| Section Toggles | 3 | Toggle sections and filters |

### 3. YubiKeyState Struct

Created domain state struct with ~25 fields:

```rust
pub struct YubiKeyState {
    // Detection state (2 fields)
    pub detected_yubikeys: Vec<YubiKeyDevice>,
    pub yubikey_detection_status: String,

    // Assignment state (2 fields)
    pub selected_yubikey_for_assignment: Option<String>,
    pub yubikey_assignment_status: String,

    // Provisioning state (2 fields)
    pub yubikey_provisioning_status: String,
    pub yubikey_provisioning_passphrase: String,

    // Slot management (3 fields)
    pub selected_yubikey_slot: PIVSlot,
    pub yubikey_key_generation_status: String,
    pub yubikey_slot_status: String,

    // PIN management (4 fields)
    pub current_pin: String,
    pub new_pin: String,
    pub new_pin_confirm: String,
    pub pin_change_status: String,

    // Management key (4 fields)
    pub current_management_key: String,
    pub new_management_key: String,
    pub new_management_key_confirm: String,
    pub management_key_change_status: String,

    // Other operations (4 fields)
    pub piv_reset_status: String,
    pub attestation_status: String,
    pub domain_registration_status: String,
    pub graph_yubikey_status: String,

    // UI state (3 fields)
    pub yubikey_section_collapsed: bool,
    pub yubikey_slot_section_collapsed: bool,
    pub filter_show_yubikey: bool,
}
```

### 4. Helper Methods

Added utility methods to YubiKeyState:
- `new()` - Creates state with sensible defaults
- `is_pin_valid()` - Validates PIN format (6-8 digits)
- `is_management_key_valid()` - Validates management key (48 hex chars)

### Files Modified

| File | Change |
|------|--------|
| `src/gui/yubikey/mod.rs` | NEW: YubiKey module exports (24 lines) |
| `src/gui/yubikey/management.rs` | NEW: YubiKey bounded context (720 lines) |
| `src/gui.rs` | Added yubikey module, YubiKey variant, delegation (~80 lines added) |

## Design Decisions

### 1. PIN Validation

Centralized validation for YubiKey PINs:
```rust
pub fn is_pin_valid(&self) -> bool {
    let pin_len = self.new_pin.len();
    pin_len >= 6
        && pin_len <= 8
        && self.new_pin.chars().all(|c| c.is_ascii_digit())
        && self.new_pin == self.new_pin_confirm
}
```

### 2. Management Key Validation

Management keys must be exactly 24 bytes (48 hex chars):
```rust
pub fn is_management_key_valid(&self) -> bool {
    self.new_management_key.len() == 48
        && self.new_management_key.chars().all(|c| c.is_ascii_hexdigit())
        && self.new_management_key == self.new_management_key_confirm
}
```

### 3. Filter Show Default

`filter_show_yubikey` defaults to `true` in `new()` constructor but `false` in `Default` derive. Tests use `new()` for realistic state.

## Tests Added

| Test | Purpose |
|------|---------|
| `test_yubikey_state_default` | Default values |
| `test_yubikey_state_new` | Constructor defaults |
| `test_pin_validation` | PIN format requirements |
| `test_management_key_validation` | Management key requirements |
| `test_toggle_sections` | Section collapse/expand |
| `test_assignment` | YubiKey-to-person assignment |
| `test_detection` | Device detection flow |

## Errors Encountered and Fixed

### 1. YubiKeyDevice Field Names
**Error**: Used `name` and `piv_available` which don't exist on `YubiKeyDevice`
**Fix**: Changed to `model` and `piv_enabled` per actual struct definition

### 2. Default vs New State
**Error**: Test used `default()` but expected `filter_show_yubikey: true`
**Fix**: Changed test to use `new()` which sets proper defaults for UI state

## Metrics

| Metric | Value |
|--------|-------|
| New files created | 2 |
| Lines added | ~800 |
| Tests passing | 1005 (up from 998) |
| Message variants extracted | ~40 |
| State fields extracted | ~25 |

## What Worked Well

1. **Pattern Consistency**: Sprint 48-49 pattern applied seamlessly to YubiKey domain
2. **Validation Helpers**: `is_pin_valid()` and `is_management_key_valid()` encapsulate security requirements
3. **Hardware-Focused Domain**: Clean separation of hardware lifecycle from PKI operations
4. **Test Coverage**: 7 new tests for YubiKey domain validation

## Lessons Learned

1. **Check Struct Fields**: Always verify actual field names (`piv_enabled` not `piv_available`)
2. **Default vs New**: Derive Default gives `false` for bools; use `new()` for UI-appropriate defaults
3. **Domain Size**: YubiKey domain is smaller than PKI (~40 vs ~55 messages) but more hardware-focused

## Best Practices Updated

50. **Field Name Verification**: Check actual struct definitions before using fields in tests
51. **Default vs Constructor**: Use `new()` for realistic UI state, `default()` for minimal state
52. **Hardware Validation**: PIN (6-8 digits) and Management Key (48 hex) have specific format requirements

## Progress Summary

| Sprint | Domain | Messages | State Fields | Tests |
|--------|--------|----------|--------------|-------|
| 48 | Organization | 50+ | 30+ | 991 |
| 49 | PKI | 55+ | 45+ | 998 |
| 50 | YubiKey | 40+ | 25+ | 1005 |
| **Total** | **3 domains** | **145+** | **100+** | **1005** |

## Next Steps (Sprint 51)

1. **Extract NatsMessage**: NATS infrastructure management (~25 messages)
2. **Extract ExportMessage**: Export and projection operations
3. **Continue reducing gui.rs complexity**

## Sprint Summary

Sprint 50 successfully extracted the YubiKey bounded context:
- Created yubikey module with ~40 message variants and ~25 state fields
- Added 7 new tests (total: 1005 passing)
- Implemented PIN and management key validation helpers
- Pattern from Sprints 48-49 proven reusable for hardware-focused domains

Three bounded contexts (Organization + PKI + YubiKey) now have clean separation from the main gui.rs module.
