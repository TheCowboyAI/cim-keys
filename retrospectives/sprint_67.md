<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 67 Retrospective: Unified Event CID Support

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Add cim_domain::cid (Blake3) support to EventEnvelope

## What Was Accomplished

### 1. Domain CID Support for EventEnvelope

Added Blake3-based CID generation using `cim_domain::cid` infrastructure:

**New Fields**:
- `domain_cid: Option<String>` - Blake3 CID for fast content addressing

**New Methods**:
- `with_domain_cid()` - Generate CID using cim_domain::cid
- `has_domain_cid()` - Check if domain CID is present
- `domain_cid_string()` - Get domain CID string
- `message_id()` - Get preferred CID for NATS message ID

### 2. Dual CID Strategy

EventEnvelope now supports two CID types:

| CID Type | Method | Hash | Use Case |
|----------|--------|------|----------|
| IPLD CID | `with_cid()` | SHA2-256 | IPFS compatibility |
| Domain CID | `with_domain_cid()` | Blake3 | NATS deduplication |

### 3. message_id() Convenience Method

```rust
/// Get the preferred CID for NATS message ID
pub fn message_id(&self) -> Option<&str> {
    self.domain_cid.as_deref().or(self.cid.as_deref())
}
```

Prefers Domain CID (faster Blake3) with fallback to IPLD CID.

### 4. Test Results

- **6 new event CID tests** - All passing
- **12 total event tests** - All passing

Tests verify:
- EventEnvelope creation and initial state
- Domain CID generation
- Deterministic CID for identical events
- message_id() preference logic
- EventChainBuilder with CID support

## Key Design Decisions

### 1. Keep Both CID Types

Rather than replace IPLD CID with Domain CID, both are available:
- IPLD CID: Standard format for external systems (IPFS, external storage)
- Domain CID: Fast Blake3 for internal operations (NATS, deduplication)

### 2. Separate Field for Domain CID

```rust
pub struct EventEnvelope {
    // ... existing fields ...
    pub cid: Option<String>,        // IPLD CID (SHA2-256)
    pub domain_cid: Option<String>, // Domain CID (Blake3)
    // ...
}
```

This allows events to have both CIDs if needed for different purposes.

### 3. CID Generation on Inner Event

```rust
pub fn with_domain_cid(mut self) -> Result<Self, String> {
    let cid = generate_cid(&self.event, ContentType::Event)?;
    self.domain_cid = Some(cid.to_string());
    Ok(self)
}
```

CID is generated from `self.event` (inner DomainEvent), not the entire envelope. This ensures:
- Same domain event = same CID
- Envelope metadata (event_id, timestamp) doesn't affect content identity

## Architecture Alignment

### Commands and Events Now Share CID Infrastructure

| Artifact | Module | CID Method |
|----------|--------|------------|
| Commands | `command_factory::cid_support` | `generate_command_cid()` |
| Events | `events::EventEnvelope` | `with_domain_cid()` |

Both use `cim_domain::cid::generate_cid()` with Blake3 hashing.

### NATS Integration Pattern

```rust
// Create event with CID for deduplication
let envelope = EventEnvelope::new(event, correlation_id, None)
    .with_domain_cid()?
    .with_subject("cim.organization.created");

// Use CID as message ID for NATS JetStream
let headers = Headers::new()
    .with_msg_id(envelope.message_id().unwrap_or(""));

jetstream.publish_with_headers(
    envelope.nats_subject.clone(),
    headers,
    payload
).await?;
```

## Metrics

| Metric | Value |
|--------|-------|
| Lines Modified | ~80 |
| New Methods | 4 |
| New Fields | 1 |
| Tests Added | 6 |
| Total Event Tests | 12 |

## What Worked Well

1. **Incremental addition**: New functionality without breaking existing code
2. **Consistent pattern**: Same CID infrastructure as commands
3. **Clear documentation**: Methods document when to use each CID type
4. **Comprehensive tests**: Cover edge cases and deterministic behavior

## Future Work

1. **NATS Integration**: Wire CID into actual NATS JetStream publishing
2. **CID Verification**: Add `verify_domain_cid()` method
3. **Event Replay**: Use CIDs for event deduplication during replay
4. **Aggregate State**: Store CIDs in aggregate state for audit trail

## Conclusion

Sprint 67 unified event CID support with commands by adding Domain CID (Blake3) to EventEnvelope. Events can now be content-addressed using the same fast hashing infrastructure as commands, while maintaining compatibility with standard IPLD CIDs for external systems.

The `message_id()` method provides a convenient single entry point for NATS message deduplication, preferring the faster Domain CID when available.
