# Sprint 45 Retrospective: Projections Write New Format (Sprint F Completion)

**Date**: 2026-01-06
**Sprint Duration**: 1 session
**Status**: COMPLETED

## Sprint Goal

Update command handlers and projections to write typed ActorId fields (new format) instead of relying solely on legacy string fields. This completes Sprint F (Full Rollout) of the ValueObject migration plan.

## Context

Sprint 44 completed the dual-path field migration for all events. Sprint 45 focuses on updating the code that *creates* events to use the new typed constructors and fields.

## Deliverables

### 1. PKI Command Handler (`src/commands/pki.rs`)

Updated `handle_generate_key_pair()`:
- Changed from `KeyGeneratedEvent::new_legacy()` to `KeyGeneratedEvent::new_typed()`
- Uses `ActorId::system("pki-keygen")` instead of `"system".to_string()`

### 2. Organization Command Handler (`src/commands/organization.rs`)

Updated `handle_create_person()`:
- Changed from direct struct initialization to `PersonCreatedEvent::new_typed()`
- Uses `ActorId::system("organization-cmd")` instead of `Some("system".to_string())`

### 3. Export Command Handler (`src/commands/export.rs`)

Updated `handle_export_to_encrypted_storage()`:
- Added `ActorId` import
- Sets `exported_by_actor: Some(ActorId::system("export-cmd"))` in KeyExportedEvent
- Keeps legacy `exported_by` field populated for backward compatibility

### 4. Saga Command Handler (`src/domain/nats/saga_command_handler.rs`)

Updated `execute_next_step()`:
- Added `ActorId` import
- Sets `generated_by_actor: Some(ActorId::legacy(&person_email))` in KeyGeneratedEvent
- Converts email string to typed ActorId while preserving original value

## ActorId Usage Patterns

| Context | ActorId Type | Example |
|---------|--------------|---------|
| System operations | `ActorId::system()` | `ActorId::system("pki-keygen")` |
| User-provided strings | `ActorId::legacy()` | `ActorId::legacy(&person_email)` |
| Known person UUID | `ActorId::person()` | `ActorId::person(user_id)` |
| Service accounts | `ActorId::service_account()` | `ActorId::service_account(id, "backup")` |

## Migration Pattern

For struct initializations (not using `new_typed()`):
```rust
#[allow(deprecated)]
let event = SomeEvent {
    // ... other fields ...
    actor_by: "legacy-string".to_string(),      // Legacy field (backward compat)
    actor_by_actor: Some(ActorId::system("service-name")),  // Typed field (preferred)
    // ... other fields ...
};
```

## What Worked Well

1. **Clear ActorId API**: Constructors like `system()`, `person()`, `legacy()` make intent explicit
2. **Backward Compatibility**: Legacy fields still populated for older consumers
3. **Incremental Adoption**: Can update handlers one at a time without breaking changes
4. **Test Coverage**: All 985 library tests pass after changes

## Files Modified

| File | Change |
|------|--------|
| `src/commands/pki.rs` | Use `new_typed()` with `ActorId::system()` |
| `src/commands/organization.rs` | Use `new_typed()` with `ActorId::system()` |
| `src/commands/export.rs` | Add typed `exported_by_actor` field |
| `src/domain/nats/saga_command_handler.rs` | Add typed `generated_by_actor` field |

## Metrics

| Metric | Value |
|--------|-------|
| Files Modified | 4 |
| Lines Added | ~25 |
| Tests Passing | 985 |
| Command Handlers Updated | 4 |

## Test Files Note

Test files (`tests/key_events.rs`, `src/event_store.rs`, etc.) intentionally keep using legacy fields with `#[allow(deprecated)]` to:
1. Test backward compatibility of deserialization
2. Verify legacy fields still work correctly
3. Ensure dual-path pattern functions as designed

## Remaining Work

Sprint F is now complete. Future enhancements could include:
1. **Add more new_typed() constructors**: Some events only have struct init, not constructors
2. **Update projection readers**: Prefer `*_value_object()` accessor methods
3. **Remove deprecated fields**: In a future major version, remove legacy string fields

## Lessons Learned

1. **ActorId::legacy() for Unknown Strings**: When you have a string but don't know its semantic type, use `legacy()` to preserve it in the typed system
2. **Keep Legacy Fields Populated**: Even when setting typed field, populate legacy field for consumers not yet updated
3. **Separate Test Strategy**: Tests should use legacy fields to verify backward compatibility

## Best Practices Updated

34. **ActorId Selection**: Use `system()` for automated processes, `person()` for users, `legacy()` for unknown strings
35. **Dual-Write Strategy**: When using typed ActorId, still populate legacy field for backward compatibility
36. **Test Files Keep Legacy**: Test files should continue using legacy patterns to verify backward compatibility
