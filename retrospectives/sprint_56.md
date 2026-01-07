# Sprint 56 Retrospective: Service Account Bounded Context Extraction

**Date**: 2026-01-06
**Sprint Duration**: 1 session
**Status**: COMPLETED

## Sprint Goal

Extract the Service Account bounded context from gui.rs into a dedicated domain module. Service Account handles automated system accounts that require human accountability - each must have a responsible person for security and lifecycle management.

## Context

Sprint 55 extracted Location. Sprint 56 extracts Service Account as a proper bounded context for managing automated systems that operate with their own credentials but under human supervision.

## What Was Accomplished

### 1. Directory Structure Created

```
src/gui/
├── service_account/
│   ├── mod.rs                 # Module exports (22 lines)
│   └── management.rs          # Service Account bounded context (~400 lines)
```

### 2. ServiceAccountMessage Enum

Created domain-specific message enum with 11 variants organized by sub-domain:

| Sub-domain | Message Count | Purpose |
|------------|---------------|---------|
| UI State | 1 | Section visibility toggle |
| Form Input | 4 | Name, purpose, owning unit, responsible person |
| Lifecycle | 4 | Create, created result, deactivate, remove |
| Key Generation | 2 | Generate key, key generated result |

### 3. ServiceAccountState Struct

Created domain state struct with 6 fields:

```rust
pub struct ServiceAccountState {
    // UI State
    pub section_collapsed: bool,

    // Form Input
    pub new_name: String,
    pub new_purpose: String,
    pub new_owning_unit: Option<Uuid>,
    pub new_responsible_person: Option<Uuid>,

    // Loaded Data
    pub created_service_accounts: Vec<ServiceAccount>,
}
```

### 4. Accountability Model

Service Account enforces accountability:
- Every service account MUST have an owning organizational unit
- Every service account MUST have a responsible person
- Form validation requires both before creation can proceed

### 5. Helper Methods

Added utility methods to ServiceAccountState:

- `new()` - Creates state with sensible defaults (section collapsed)
- `is_form_valid()` - Validates required fields (name, unit, person)
- `validation_error()` - Get human-readable validation error
- `clear_form()` - Reset form after successful creation
- `service_account_count()` - Count of all service accounts
- `active_count()` - Count of active (not deactivated) accounts
- `find_service_account()` - Find by ID (immutable)
- `find_service_account_mut()` - Find by ID (mutable)
- `deactivate()` - Deactivate account by ID
- `remove()` - Remove account by ID

### Files Modified

| File | Change |
|------|--------|
| `src/gui/service_account/mod.rs` | NEW: Service Account module exports (22 lines) |
| `src/gui/service_account/management.rs` | NEW: Service Account bounded context (~400 lines) |
| `src/gui.rs` | Added service_account module, ServiceAccount Message variant, handler |

## Design Decisions

### 1. Accountability Enforcement

Every service account requires both owning unit AND responsible person:
```rust
pub fn is_form_valid(&self) -> bool {
    !self.new_name.is_empty()
        && self.new_owning_unit.is_some()
        && self.new_responsible_person.is_some()
}
```

### 2. Soft Delete Pattern

Service accounts support deactivation (soft delete) before removal:
- `Deactivate(Uuid)` - Mark inactive but preserve record
- `Remove(Uuid)` - Complete removal from list

### 3. Delegated Complex Operations

Create and KeyGeneration require validation and workflow access:
```rust
Create => {
    // Actual creation requires validation and projection access
    // Delegated to main update function
    Task::none()
}
```

## Tests Added

| Test | Purpose |
|------|---------|
| `test_service_account_state_default` | Default state values |
| `test_service_account_state_new` | Constructor defaults (section collapsed) |
| `test_toggle_section` | Section visibility toggle |
| `test_name_changed` | Name field update |
| `test_purpose_changed` | Purpose field update |
| `test_owning_unit_selected` | Owning unit selection |
| `test_responsible_person_selected` | Responsible person selection |
| `test_is_form_valid` | Form validation with required fields |
| `test_validation_error_name_required` | Name validation error |
| `test_validation_error_owning_unit_required` | Unit validation error |
| `test_validation_error_responsible_person_required` | Person validation error |
| `test_clear_form` | Form reset |
| `test_service_account_count` | Account counting |
| `test_active_count` | Active account filtering |
| `test_find_service_account` | Find by ID |
| `test_deactivate` | Deactivate functionality |
| `test_remove` | Remove functionality |

## Metrics

| Metric | Value |
|--------|-------|
| New files created | 2 |
| Lines added | ~420 |
| Tests passing | 1077 (up from 1068) |
| Message variants extracted | 11 |
| State fields extracted | 6 |
| Service Account-specific tests | 17 |

## What Worked Well

1. **Accountability Model**: Clear enforcement of responsible person requirement
2. **Soft Delete Pattern**: Deactivate before remove provides audit trail
3. **Comprehensive Helpers**: Multiple ways to find and manage accounts
4. **Active Counting**: Easy to see active vs total accounts

## Lessons Learned

1. **ServiceAccount Fields**: Remember to check domain struct for `active` field
2. **Default vs New**: Default leaves section_collapsed false, new() sets true
3. **Mutable Helpers**: Provide both immutable and mutable find methods

## Best Practices Updated

69. **Accountability Enforcement**: Service accounts need human ownership (unit + person)
70. **Soft Delete Pattern**: Deactivate before remove for audit trails
71. **Section Collapse State**: Default behavior may differ from explicit new()

## Progress Summary

| Sprint | Type | Module | Messages | State Fields | Tests |
|--------|------|--------|----------|--------------|-------|
| 48 | Domain | Organization | 50+ | 30+ | 991 |
| 49 | Domain | PKI | 55+ | 45+ | 998 |
| 50 | Domain | YubiKey | 40+ | 25+ | 1005 |
| 51 | Domain | NATS | 20+ | 14+ | 1014 |
| 52 | Port | Export | 15+ | 9+ | 1024 |
| 53 | Domain | Delegation | 9 | 6 | 1035 |
| 54 | Domain | TrustChain | 5 | 3 | 1053 |
| 55 | Domain | Location | 10 | 9 | 1068 |
| 56 | Domain | ServiceAccount | 11 | 6 | 1077 |
| **Total** | **8 domains, 1 port** | | **215+** | **147+** | **1077** |

## Next Steps (Sprint 57+)

1. **GPG Keys domain**: Generate, list, manage GPG keys
2. **Organization Unit domain**: Create, manage organizational units
3. **Key Recovery domain**: Seed verification and key recovery
4. **Review gui.rs size**: Measure cumulative reduction

## Sprint Summary

Sprint 56 successfully extracted the Service Account bounded context:
- Created service_account module with 11 message variants and 6 state fields
- Enforced accountability model (owning unit + responsible person required)
- Implemented soft delete pattern (deactivate before remove)
- Added 17 new tests (total: 1077 passing)

Eight bounded contexts (Organization + PKI + YubiKey + NATS + Delegation + TrustChain + Location + ServiceAccount) plus one Port (Export) now have clean separation from the main gui.rs module.
