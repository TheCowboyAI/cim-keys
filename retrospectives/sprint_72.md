<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 72 Retrospective: Remove OOP Factories

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Remove OOP-style command factories, keep only curried FP implementation

## What Was Accomplished

### Deleted OOP Factory Files

| File | Lines Removed |
|------|---------------|
| `src/command_factory/person.rs` | ~240 |
| `src/command_factory/organization.rs` | ~230 |
| `src/command_factory/location.rs` | ~280 |
| `src/command_factory/service_account.rs` | ~270 |
| **Total** | **~1,020** |

### Updated mod.rs

Simplified exports to curried module only:

```rust
pub mod cid_support;
pub mod curried;

pub use curried::{
    person, organization, org_unit, location, service_account,
    PersonResult, OrganizationResult, OrgUnitResult, LocationResult, ServiceAccountResult,
};
```

### Updated GUI Handlers

All 5 entity creation handlers updated to use curried factories:

```rust
// Use aliased imports to avoid conflict with GUI submodules
use crate::command_factory::{
    person as person_factory, organization as org_factory,
    org_unit as org_unit_factory, location as location_factory,
    service_account as sa_factory,
    PersonResult, OrganizationResult, OrgUnitResult, LocationResult, ServiceAccountResult,
};

// Explicit type annotation required for Box<dyn Fn> inference
let result: PersonResult = person_factory::create(correlation_id)(org_id)(&form);
```

## Technical Challenges

### 1. Name Conflicts

GUI submodules (`location`, `org_unit`, `service_account`) conflicted with curried factory module names.

**Solution**: Use import aliases (`person_factory`, `org_factory`, etc.)

### 2. Type Inference with Box<dyn Fn>

Rust couldn't infer error types through `Box<dyn Fn>` boundaries.

**Solution**: Explicit type annotations on result:
```rust
let result: PersonResult = person_factory::create(...)(...)(...)
```

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Factory files | 5 OOP + 1 curried | 1 curried only |
| Lines of code | ~1,550 | ~510 |
| Test count | 1085 | 1066 |
| API style | Dual (OOP + FP) | Pure FP |

## Code Reduction

Removed ~1,020 lines of OOP factory code, keeping only the curried FP implementation (~510 lines).

## Test Impact

Test count reduced from 1085 to 1066:
- Removed: ~19 OOP factory tests
- Kept: 7 curried factory tests + 4 mod tests
- All remaining tests pass

## Conclusion

Sprint 72 completed the FP migration by removing redundant OOP factories. The codebase now has a single, consistent curried FP approach for command creation.

Combined with Sprint 71:
- Sprint 71: Added true FP curried factories
- Sprint 72: Removed OOP factories, curried-only API

The command factory module is now 100% functional programming style.
