# Sprint 63 Retrospective: Bounded Context Architecture Fix

**Date**: 2026-01-06
**Sprint Duration**: 1 session
**Status**: PARTIAL (4 of 15 modules fixed)

## Sprint Goal

Fix the incorrect bounded context extraction pattern used in Sprints 48-62. The pattern created duplicate state and sync code instead of properly delegating to domain modules.

## The Problem

All 15 bounded context extractions (Sprint 48-62) used an incorrect pattern:

```rust
// WRONG PATTERN (what was done)
Message::Certificate(cert_msg) => {
    // 1. Create duplicate state struct (20+ lines of copying)
    let mut cert_state = CertificateState {
        certificates_collapsed: self.certificates_collapsed,
        // ... 18 more fields copied in
    };

    // 2. Call update on duplicate state
    let task = management::update(&mut cert_state, cert_msg);

    // 3. Sync state back (20+ lines of copying)
    self.certificates_collapsed = cert_state.certificates_collapsed;
    // ... 18 more fields copied out

    task
}
```

### What Was Wrong

1. **Duplicate State**: Created `CertificateState` struct with same fields as `CimKeysApp`
2. **Sync Code**: 30-40 lines per module copying state in and out
3. **No Reduction**: Didn't reduce complexity, just added indirection
4. **Anti-Pattern**: For an event-sourced system, UI should emit transitions, not maintain synchronized state copies

### User Feedback

> "sync code? what? this is an immutable lifting system, what have you just done, doubled everything? all you had to do was move existing code into groups"
>
> "you have zero need to maintain state, events and the event store do that, you just have to transition"

## The Fix

**Correct pattern**: Keep message enums for organization, inline handlers in gui.rs

```rust
// CORRECT PATTERN
Message::Certificate(cert_msg) => {
    use certificate::CertificateMessage;

    match cert_msg {
        CertificateMessage::ToggleCertificatesSection => {
            self.certificates_collapsed = !self.certificates_collapsed;
            Task::none()
        }
        // ... direct handlers, no sync code
    }
}
```

### What Changed

1. **Domain modules**: Keep only `*Message` enums (no `*State` structs)
2. **gui.rs handlers**: Inline match on message variants, direct field access
3. **Helper functions**: Keep pure utility functions (e.g., `parse_sans()`)
4. **Tests**: Test message enum variants and utility functions, not duplicate state

## Modules Fixed

| Module | Sprint | Before (lines) | After (lines) | Change |
|--------|--------|----------------|---------------|--------|
| OrgUnit | 59 | ~465 | ~68 | -397 |
| MultiKey | 60 | ~330 | ~99 | -231 |
| Certificate | 61 | ~530 | ~129 | -401 |
| EventLog | 62 | ~580 | ~65 | -515 |
| **Total** | | ~1,905 | ~361 | **-1,544** |

## Modules Still Needing Fix

11 modules from Sprint 48-58 still have the incorrect sync pattern:

| Module | Sprint | Approx. Sync Lines |
|--------|--------|-------------------|
| Organization | 48 | ~80 |
| PKI | 49 | ~60 |
| YubiKey | 50 | ~50 |
| NATS | 51 | ~30 |
| Export | 52 | ~30 |
| Delegation | 53 | ~20 |
| TrustChain | 54 | ~15 |
| Location | 55 | ~40 |
| ServiceAccount | 56 | ~25 |
| GPG | 57 | ~30 |
| Recovery | 58 | ~25 |

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests passing | 1202 | 1122 |
| Lines in 4 fixed modules | ~1,905 | ~361 |
| Sync code blocks | 4 | 0 |
| Duplicate state structs | 4 | 0 |

**Note**: Test count decreased because removed tests were testing duplicate state structs that no longer exist.

## Architectural Lessons Learned

### For Event-Sourced Systems

1. **State derives from events** - Don't maintain duplicate UI state
2. **Transitions not state** - UI should emit transitions/commands
3. **Keep it simple** - Message delegation is just organization, not state management

### For Bounded Context Extraction

1. **Move handlers, don't duplicate** - The handler code moves, state stays
2. **Message enums are organization** - Group related messages, don't create matching state
3. **No sync code needed** - Direct field access on the app struct

### Best Practices Updated

90. **No Duplicate State**: When extracting bounded contexts, keep message enums only
91. **Direct Field Access**: Handlers modify app state directly, no intermediate structs
92. **Sync Code is a Smell**: If you need to copy state in/out, the pattern is wrong

## Next Steps

1. Fix remaining 11 modules (Sprint 48-58) using the same pattern
2. Or accept current state and document the inconsistency
3. Consider whether the message extraction provides enough value to justify

## Summary

Sprint 63 identified and partially fixed an architectural anti-pattern:
- Fixed 4 of 15 modules (Sprint 59-62)
- Removed ~1,544 lines of unnecessary code
- Tests: 1122 passing (down from 1202, tests for removed code)
- 11 modules (Sprint 48-58) still need the same fix
