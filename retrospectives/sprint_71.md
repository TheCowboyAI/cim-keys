<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 71 Retrospective: True FP Curried Command Factories

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Refactor command factories from OOP to true Functional Programming with currying

## Problem Statement

User feedback on Sprint 68-70 work:

> "this is an OOP implementation of Factory unless you are using currying."
> "we want a curried approach in fp/frp"

The existing factories were OOP-style:

```rust
// OOP Factory (Sprint 68-70)
fn create_person_command(
    form: &NewPersonForm,
    organization_id: Option<Uuid>,
    correlation_id: Uuid,
) -> Result<CreatePerson, NonEmptyVec<ValidationError>>
```

This is **not** true FP because:
1. All arguments passed at once
2. No partial application possible
3. No function composition
4. Cannot reuse partially applied factories

## Solution: True FP Currying

### Haskell-Style Type Signature

```text
create :: Uuid -> (Maybe Uuid -> (NewPersonForm -> Result CreatePerson Error))
```

In Rust, this becomes:

```rust
// True FP Curried Factory (Sprint 71)
pub fn create(correlation_id: Uuid) -> WithCorrelation {
    Box::new(move |organization_id: Option<Uuid>| -> WithOrg {
        Box::new(move |form: &NewPersonForm| -> PersonResult {
            validate_and_create_person(form, organization_id, correlation_id)
        })
    })
}
```

### Rust Implementation Challenge

Rust error E0562 prevents nested `impl Trait`:

```rust
// This DOES NOT compile:
fn create(id: Uuid) -> impl Fn(Option<Uuid>) -> impl Fn(&Form) -> Result
//                                              ^^^^ ERROR: impl Trait not allowed
```

**Solution**: Use `Box<dyn Fn>` with type aliases for readability:

```rust
pub type WithOrg = Box<dyn Fn(&NewPersonForm) -> PersonResult>;
pub type WithCorrelation = Box<dyn Fn(Option<Uuid>) -> WithOrg>;

pub fn create(correlation_id: Uuid) -> WithCorrelation { ... }
```

## What Was Implemented

### New Module: `src/command_factory/curried.rs`

| Factory | Type Signature (Haskell-style) |
|---------|-------------------------------|
| `person::create` | `Uuid → Maybe Uuid → NewPersonForm → Result CreatePerson` |
| `organization::create` | `Uuid → OrganizationForm → Result CreateOrganization` |
| `org_unit::create` | `Uuid → NewOrgUnitForm → Result CreateOrganizationalUnit` |
| `location::create` | `Uuid → Maybe Uuid → NewLocationForm → Result CreateLocation` |
| `service_account::create` | `Uuid → NewServiceAccountForm → Result CreateServiceAccount` |

### Usage Patterns

**1. Full Curried Application (one line)**
```rust
let cmd = person::create(correlation_id)(org_id)(&form)?;
```

**2. Partial Application (reusable factory)**
```rust
// Fix correlation_id once
let with_correlation = person::create(correlation_id);

// Fix org_id once
let for_org = with_correlation(Some(org_id));

// Apply to multiple forms with same context
let cmd1 = for_org(&form1)?;
let cmd2 = for_org(&form2)?;
let cmd3 = for_org(&form3)?;
```

**3. Batch Creation (functional style)**
```rust
let create_person_for_org = person::create(correlation_id)(org_id);

let commands: Vec<_> = people
    .iter()
    .map(|(name, email)| {
        let form = NewPersonForm::new()
            .with_name(name.to_string())
            .with_email(email.to_string());
        create_person_for_org(&form)
    })
    .collect::<Result<Vec<_>, _>>()?;
```

## OOP vs FP Comparison

| Aspect | OOP Factory (Before) | FP Curried (After) |
|--------|---------------------|-------------------|
| Argument passing | All at once | One at a time |
| Partial application | Not possible | Native |
| Reusability | Must pass all args every time | Cache intermediate functions |
| Composition | Manual wiring | Point-free pipelines |
| Type safety | Same | Same (Result types) |

## Test Results

7 new curried factory tests added:
- `test_person_curried_full_application` - Full curried call
- `test_person_curried_partial_application` - Reusable factory
- `test_person_curried_validation_errors` - Error accumulation
- `test_organization_curried` - Organization factory
- `test_location_curried` - Location factory
- `test_service_account_curried` - Service account factory
- `test_batch_creation_with_partial_application` - Batch use case

All 1085 tests pass (1078 existing + 7 new).

## Metrics

| Metric | Value |
|--------|-------|
| New File | `src/command_factory/curried.rs` |
| Lines Added | ~510 |
| Curried Factories | 5 |
| Tests Added | 7 |
| Type Aliases Added | 8 |

## What Worked Well

1. **`Box<dyn Fn>` pattern**: Solves Rust's nested `impl Trait` limitation
2. **Type aliases**: Make curried signatures readable
3. **Same validation**: Curried factories reuse existing validation logic
4. **Both APIs available**: OOP factories remain for simple cases, curried for FP composition

## Lessons Learned

1. **Rust currying requires boxing**: Can't use `impl Trait` in nested closures
2. **Type aliases are essential**: Without them, signatures become unreadable
3. **FP in Rust has overhead**: `Box<dyn Fn>` has heap allocation + vtable lookup
4. **Documentation matters**: Haskell-style signatures help readers understand currying

## API Design Decision

Both APIs are now available:

```rust
// OOP-style (simple, direct)
use crate::command_factory::create_person_command;
let cmd = create_person_command(&form, org_id, correlation_id)?;

// FP curried style (composable, reusable)
use crate::command_factory::curried::person;
let cmd = person::create(correlation_id)(org_id)(&form)?;
```

This gives users choice based on their use case:
- **Direct calls**: Use OOP factory
- **Batch processing**: Use curried factory with partial application
- **Pipeline composition**: Use curried factory

## FRP Axiom Compliance

The curried factories satisfy FRP axioms:

- **A3 (Decoupled)**: Each curried function stage is independent
- **A5 (Totality)**: All functions return Result, no panics
- **A9 (Composition)**: Curried functions compose naturally

## Future Considerations

1. **Macro for currying**: Could generate curried versions automatically
2. **Generic currying trait**: Abstract the pattern for any multi-arg function
3. **Benchmark overhead**: Measure `Box<dyn Fn>` vs direct call performance

## Conclusion

Sprint 71 addressed the fundamental architectural feedback about OOP vs FP. The command factory module now provides true FP curried factories with partial application, enabling:

- Reusable partially-applied factories
- Point-free style composition
- Batch processing with shared context
- Idiomatic functional programming patterns

The Rust implementation uses `Box<dyn Fn>` to work around language limitations while preserving the functional semantics.
