# UUID v7 Timestamp Derivation AXIOM

**Date:** 2025-11-22
**Status:** ARCHITECTURAL AXIOM (Non-Negotiable)
**Scope:** All CIM modules and domains

---

## The AXIOM

**UUID v7 contains embedded timestamps. Separate `*_at` timestamp fields are derived/redundant convenience fields.**

This is a foundational principle of the CIM architecture that applies to ALL event-sourced systems.

---

## What is UUID v7?

UUID v7 is a time-ordered UUID format (RFC 9562) that embeds:
- **48-bit timestamp** (millisecond precision)
- **12-bit random sequence** (sub-millisecond ordering)
- **62-bit random data** (uniqueness guarantee)

**Key Property**: The timestamp is embedded in the first 48 bits of the UUID, making it:
- Time-ordered (sortable)
- Extractable (timestamp can be derived from the UUID)
- Immutable (timestamp cannot change after UUID creation)

---

## Implications for Event-Sourced Systems

### 1. Entity IDs Contain Creation Time

Every entity ID created with `Uuid::now_v7()` embeds the creation timestamp:

```rust
pub struct KeyGeneratedEvent {
    pub key_id: Uuid,  // UUID v7 - contains embedded timestamp
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub generated_at: DateTime<Utc>,  // Derived from key_id (UUID v7 timestamp) for convenience
    pub generated_by: String,
    // ...
}
```

**The `generated_at` field is redundant** - it can be derived from `key_id`. We keep it for:
- Convenience (avoids extracting timestamp from UUID)
- Human readability in JSON
- Query optimization (indexed separately)

### 2. Event IDs Contain Event Time

Every event ID contains the event occurrence timestamp:

```rust
pub struct YubiKeyProvisionedEvent {
    pub event_id: Uuid,  // UUID v7 - contains embedded timestamp
    pub yubikey_serial: String,
    pub slots_configured: Vec<YubiKeySlot>,
    pub provisioned_at: DateTime<Utc>,  // Derived from event_id (UUID v7 timestamp) for convenience
    pub provisioned_by: String,
}
```

**The `provisioned_at` field is redundant** - it can be derived from `event_id`.

### 3. Operation Timestamps are NOT Redundant

When an event describes an operation on an existing entity, the `*_at` field is NOT redundant:

```rust
pub struct KeyExportedEvent {
    pub key_id: Uuid,  // References the key (not UUID v7 - key already exists)
    pub format: KeyFormat,
    pub include_private: bool,
    pub exported_at: DateTime<Utc>,  // Actual export timestamp (operation time, not key creation)
    pub exported_by: String,
    pub destination: ExportDestination,
}
```

**The `exported_at` field is NOT redundant** because:
- `key_id` contains the key creation time (when key was generated)
- `exported_at` contains the export operation time (when key was exported)
- These are two different timestamps

---

## Pattern Recognition Rules

### Redundant Timestamp (Derived from UUID v7)

Pattern: `*_at` field describes the same event as the UUID v7 ID creation

```rust
pub key_id: Uuid,  // UUID v7 - created when key generated
pub generated_at: DateTime<Utc>,  // Derived from key_id ✅ REDUNDANT
```

```rust
pub event_id: Uuid,  // UUID v7 - created when event occurred
pub provisioned_at: DateTime<Utc>,  // Derived from event_id ✅ REDUNDANT
```

```rust
pub person_id: Uuid,  // UUID v7 - created when person joined
pub created_at: DateTime<Utc>,  // Derived from person_id ✅ REDUNDANT
```

### Non-Redundant Timestamp (Operation Time)

Pattern: `*_at` field describes a different time than UUID v7 ID creation

```rust
pub key_id: Uuid,  // References existing key (created earlier)
pub exported_at: DateTime<Utc>,  // Export operation time ❌ NOT REDUNDANT
```

```rust
pub key_id: Uuid,  // References existing key
pub activated_at: DateTime<Utc>,  // Activation operation time ❌ NOT REDUNDANT
```

```rust
pub certificate_id: Uuid,  // References existing certificate
pub revoked_at: DateTime<Utc>,  // Revocation operation time ❌ NOT REDUNDANT
```

---

## Extracting Timestamps from UUID v7

While we keep convenience `*_at` fields, you CAN extract timestamps from UUID v7:

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};

fn extract_timestamp(uuid: &Uuid) -> DateTime<Utc> {
    let bytes = uuid.as_bytes();

    // First 48 bits are timestamp (milliseconds since Unix epoch)
    let timestamp_ms =
        (bytes[0] as u64) << 40 |
        (bytes[1] as u64) << 32 |
        (bytes[2] as u64) << 24 |
        (bytes[3] as u64) << 16 |
        (bytes[4] as u64) << 8 |
        (bytes[5] as u64);

    // Convert milliseconds to DateTime
    let secs = (timestamp_ms / 1000) as i64;
    let nsecs = ((timestamp_ms % 1000) * 1_000_000) as u32;

    Utc.timestamp_opt(secs, nsecs).unwrap()
}
```

**However**, in practice we don't do this because:
- Convenience fields already exist in events
- JSON serialization is human-readable
- Database queries can index the separate field
- Code clarity over byte manipulation

---

## Design Rationale

### Why Keep Redundant Fields?

1. **Human Readability**: JSON event logs are more readable with explicit timestamps
2. **Query Performance**: Database indexes on timestamp columns are more efficient
3. **API Ergonomics**: Clients don't need UUID parsing logic
4. **Backward Compatibility**: Existing event schemas remain valid
5. **Convention over Configuration**: Explicit is better than implicit

### Why Document This as REDUNDANT?

1. **Architectural Awareness**: Developers must understand the single source of truth
2. **Prevent Inconsistency**: Never update `*_at` without updating UUID (or vice versa)
3. **Event Sourcing Correctness**: The UUID is the immutable timestamp, `*_at` is derived
4. **Testing Clarity**: Tests should generate UUIDs first, derive timestamps second

---

## Coding Standards

### ✅ CORRECT: Generate UUID first, derive timestamp

```rust
let key_id = Uuid::now_v7();
let generated_at = extract_timestamp(&key_id); // Or use Utc::now() immediately after

KeyGeneratedEvent {
    key_id,
    algorithm: KeyAlgorithm::Ed25519,
    generated_at,
    // ...
}
```

### ❌ INCORRECT: Generate timestamp separately

```rust
let key_id = Uuid::now_v7();
let generated_at = Utc::now();  // ❌ Timestamp skew! These are different times!

KeyGeneratedEvent {
    key_id,
    algorithm: KeyAlgorithm::Ed25519,
    generated_at,
    // ...
}
```

**Why this is wrong**: `key_id` and `generated_at` will have different timestamps (milliseconds apart), violating the derivation principle.

### ✅ CORRECT: Use same timestamp for both

```rust
let now = Utc::now();
let key_id = Uuid::now_v7();  // Uses current time (approximately `now`)

// Better: Extract exact timestamp from UUID
let generated_at = extract_timestamp(&key_id);

// Or: Use the captured timestamp (close enough for human purposes)
let generated_at = now;

KeyGeneratedEvent {
    key_id,
    algorithm: KeyAlgorithm::Ed25519,
    generated_at,
    // ...
}
```

---

## Testing Implications

### ✅ CORRECT: Test with UUID v7 extraction

```rust
#[test]
fn test_key_generated_event_timestamp_consistency() {
    let event = KeyGeneratedEvent {
        key_id: Uuid::now_v7(),
        algorithm: KeyAlgorithm::Ed25519,
        generated_at: Utc::now(),
        // ...
    };

    let extracted_timestamp = extract_timestamp(&event.key_id);

    // Timestamps should be within milliseconds of each other
    let diff = (event.generated_at.timestamp_millis() - extracted_timestamp.timestamp_millis()).abs();
    assert!(diff < 10, "Timestamp skew too large: {} ms", diff);
}
```

### ✅ CORRECT: Mock time for deterministic tests

```rust
#[test]
fn test_key_lifecycle_with_fixed_time() {
    // Use a fixed timestamp for testing
    let fixed_time = Utc.with_ymd_and_hms(2025, 11, 22, 12, 0, 0).unwrap();

    // In real code, we'd need a time provider abstraction
    // For now, accept that UUIDs will have current time
    let key_id = Uuid::now_v7();

    // But we can use fixed time for derived fields
    let event = KeyGeneratedEvent {
        key_id,
        algorithm: KeyAlgorithm::Ed25519,
        generated_at: fixed_time,
        // ...
    };

    // Test logic here
}
```

---

## Documentation Standard

### Event Structure Documentation

ALL event structures MUST document UUID v7 timestamp relationships:

```rust
/// Key was imported from external source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyImportedEvent {
    pub key_id: Uuid,  // UUID v7 - contains embedded timestamp
    pub source: ImportSource,
    pub format: KeyFormat,
    pub imported_at: DateTime<Utc>,  // Derived from key_id (UUID v7 timestamp) for convenience
    pub imported_by: String,
    pub metadata: KeyMetadata,
}
```

**Required comments:**
- `// UUID v7 - contains embedded timestamp` on ID fields
- `// Derived from X_id (UUID v7 timestamp) for convenience` on redundant timestamp fields
- `// Actual operation timestamp (not entity creation time)` on non-redundant timestamp fields

---

## Conclusion

**UUID v7 timestamp derivation is an AXIOM of the CIM architecture.**

- All entity IDs use `Uuid::now_v7()`
- Separate `*_at` fields for entity creation are derived/redundant
- Operation timestamps (export, activate, revoke) are NOT redundant
- Document all UUID v7 relationships in code comments
- The UUID is the single source of truth for creation time

This pattern ensures:
- Time-ordered event streams
- Immutable temporal semantics
- Human-readable event logs
- Architectural consistency across all CIM modules

**When in doubt**: The UUID contains the truth, the `*_at` field is for convenience.

---

**References:**
- RFC 9562: UUID Version 7 (Time-Ordered)
- `docs/STATE_MACHINE_PHASE4_RETROSPECTIVE.md` - Phase 4 integration
- `docs/STATE_MACHINE_PHASE3_RETROSPECTIVE.md` - All state machines
- `src/events_legacy.rs` - Event definitions with UUID v7 comments
- `src/state_machines/` - State machine definitions with timestamp comments
