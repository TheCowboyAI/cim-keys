<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 76 Retrospective: Wire Remaining Organizational Entities to CQRS

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Wire CreateOrganizationUnit and CreateServiceAccount GUI handlers to CQRS aggregate

## What Was Accomplished

### Added KeyCommand Variants

Extended the KeyCommand enum to include organizational entity operations:

```rust
// src/commands/mod.rs
pub enum KeyCommand {
    // ... existing variants ...

    // Organizational domain operations
    CreateOrganization(organization::CreateOrganization),
    CreatePerson(organization::CreatePerson),
    CreateLocation(organization::CreateLocation),
    CreateOrganizationalUnit(organization::CreateOrganizationalUnit),  // NEW
    CreateServiceAccount(organization::CreateServiceAccount),           // NEW
}
```

### Implemented Command Handlers

Created async command handlers in `src/commands/organization.rs`:

1. **handle_create_organizational_unit**
   - Validates unit name is non-empty
   - Emits `OrganizationalUnitCreated` event
   - Uses `ActorId::system("organization-cmd")` for created_by

2. **handle_create_service_account**
   - Validates name and purpose are non-empty
   - Emits `ServiceAccountCreated` event
   - Wraps correlation_id in Option (event schema compatibility)

### Added Aggregate Routing

Added match arms in `src/aggregate.rs` for the new KeyCommand variants:

```rust
KeyCommand::CreateOrganizationalUnit(cmd) => {
    crate::commands::organization::handle_create_organizational_unit(cmd).await
}
KeyCommand::CreateServiceAccount(cmd) => {
    crate::commands::organization::handle_create_service_account(cmd).await
}
```

### Updated GUI Handlers

Both organizational entity handlers now use proper CQRS pattern:

1. **CreateOrganizationUnit Handler**
   - Creates `CreateOrganizationalUnit` command with proper parent_id lookup
   - Routes through aggregate via `Task::perform`
   - Extracts unit from `OrganizationalUnitCreated` event
   - Updates both `created_units` and `loaded_units` lists

2. **CreateServiceAccount Handler**
   - Creates `CreateServiceAccount` command with owning unit and responsible person
   - Routes through aggregate via `Task::perform`
   - Extracts account from `ServiceAccountCreated` event
   - Updates `created_service_accounts` list

## CQRS Flow Pattern

```rust
// Before (Sprint 72-73 - curried factory pattern)
let result: OrgUnitResult = org_unit_factory::create(correlation_id)(Some(org_uuid))(&form);
match result {
    Ok(command) => {
        let unit = OrganizationUnit { ... };  // Direct construction
        self.created_units.push(unit);         // Direct mutation
    }
}

// After (Sprint 76 - CQRS through aggregate)
let cmd = CreateOrganizationalUnit { ... };
Task::perform(
    async move {
        let events = aggregate.handle_command(
            KeyCommand::CreateOrganizationalUnit(cmd), ...
        ).await?;

        // Extract from OrganizationalUnitCreated event
        for event in &events {
            if let DomainEvent::Organization(
                OrganizationEvents::OrganizationalUnitCreated(evt)
            ) = event {
                return Ok(OrganizationUnit::from(evt));
            }
        }
    },
    Message::OrganizationUnitCreated
)
```

## Event Flow

```
GUI Form → CreateOrganizationalUnit Command → Aggregate.handle_command()
                                                      ↓
                                             Command Handler validates
                                                      ↓
                                             Emit OrganizationalUnitCreated Event
                                                      ↓
                                             Extract data from event
                                                      ↓
                                             OrganizationUnitCreated Message
                                                      ↓
                                             Update GUI state + add to lists
```

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | +160 |
| Tests passing | 1,080+ |
| Files modified | 3 (commands/mod.rs, commands/organization.rs, aggregate.rs, gui.rs) |
| New command handlers | 2 |

## Issues Encountered and Fixes

1. **correlation_id type mismatch**
   - Issue: `ServiceAccountCreatedEvent.correlation_id` is `Option<Uuid>`, not `Uuid`
   - Fix: Changed to `correlation_id: Some(cmd.correlation_id)`

2. **Uuid dereference error**
   - Issue: `*p.id.as_uuid()` - can't dereference Uuid
   - Fix: Changed to `p.id.as_uuid().clone()`

## What Went Well

1. **Consistent pattern** - Used same Task::perform pattern as Sprint 74-75
2. **Event extraction** - Clean pattern to extract data from emitted domain events
3. **Two-phase update** - Clear form fields immediately, add to lists on success
4. **All tests pass** - No regressions introduced

## Lessons Learned

1. **Event schema awareness** - Some events have Optional correlation_id, check event definition
2. **EntityId API** - Use `.as_uuid().clone()` not `*.as_uuid()` for owned Uuid

## Related Sprints

- Sprint 72: Removed OOP factories (curried-only)
- Sprint 73: Created proper Command trait implementations
- Sprint 74: Wired GUI delegation to CQRS aggregate
- Sprint 75: Wired AddPerson/AddLocation to CQRS aggregate
- Sprint 76: Wired CreateOrganizationUnit/CreateServiceAccount to CQRS aggregate (this sprint)

## CQRS Coverage Progress

| Handler | Before Sprint 74 | After Sprint 76 |
|---------|------------------|-----------------|
| CreateDelegation | Direct GUI state | CQRS ✓ |
| RevokeDelegation | Direct GUI state | CQRS ✓ |
| AddPerson | Factory + Direct | CQRS ✓ |
| AddLocation | Factory + Direct | CQRS ✓ |
| CreateOrganizationUnit | Factory + Direct | CQRS ✓ |
| CreateServiceAccount | Factory + Direct | CQRS ✓ |

## Next Steps

All organizational entity handlers are now wired to CQRS. Potential next:
- PKI key generation handlers (GenerateCertificate, GenerateSshKey)
- YubiKey provisioning handlers
- Continue bounded context refactoring per plan
