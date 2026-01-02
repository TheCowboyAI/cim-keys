# CIM Architectural Compliance Evaluation: cim-keys

**Evaluator:** CIM Expert
**Date:** 2026-01-02
**Overall CIM Compliance: 65%**

---

## 1. Event Sourcing Analysis

### 1.1 Events Structure Review

```
src/events/
├── mod.rs
├── aggregate_events.rs
├── certificate_events.rs
├── domain_events.rs
├── key_events.rs
├── location_events.rs
├── manifest_events.rs
├── nats_events.rs
├── organization_events.rs
├── person_events.rs
├── pki_events.rs
└── yubikey_events.rs
```

### 1.2 Event Envelope Structure

```rust
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub event: DomainEvent,
}
```

**COMPLIANCE: PARTIAL**

| Requirement | Status |
|-------------|--------|
| `event_id` present | PASS |
| `correlation_id` present | PASS |
| `causation_id` present | PASS |
| `timestamp` present | PASS |
| Content-addressed (CID) | **FAIL** |
| Immutability guarantee | **PARTIAL** |

### 1.3 Aggregate Analysis

**COMPLIANCE: STRONG**

The aggregate follows the pure function pattern:
- `Command + Projection -> Vec<Event>`
- No side effects in command handlers
- Events are the only output

### 1.4 Projection Analysis

**COMPLIANCE: STRONG**

- State derived exclusively from event application
- `rebuild_from_events` enables event replay

**Issue:** Uses `&mut self` which allows mutation outside apply method.

---

## 2. NATS Integration Analysis

### 2.1 Subject Pattern Evaluation

**Expected Pattern:**
```
<organization>.<unit>.<domain>.<entity>.<operation>
```

**COMPLIANCE: PARTIAL**

| Aspect | Status |
|--------|--------|
| Semantic naming | PASS |
| 5-part structure | PARTIAL |
| Event vs Command distinction | **FAIL** |
| Wildcard support | PASS |

---

## 3. Offline-First Architecture Analysis

**COMPLIANCE: STRONG**

- Events persisted to `events/` directory as JSON
- Projections written to domain directories
- No network dependency for core operations

**Issue:** Events not content-addressed (should use CID-based naming).

---

## 4. Graph-Based Organization Analysis

**COMPLIANCE: PARTIAL**

| Aspect | Status |
|--------|--------|
| People as nodes | PASS |
| Relationships as edges | PASS |
| Temporal validity | PASS |
| Graph-first modeling | **PARTIAL** |

**Anti-Pattern Found:**

```rust
// ANTI-PATTERN: Hierarchical ownership
pub struct Organization {
    pub units: Vec<OrganizationUnit>,  // Parent owns children
}
```

---

## 5. Anti-Pattern Summary

| ID | Location | Anti-Pattern | CIM Requirement |
|----|----------|--------------|-----------------|
| AP-1 | `src/events/*.rs` | Events not content-addressed | IPLD CID for all events |
| AP-2 | `src/aggregate.rs` | Transient state in handlers | Pure event generation only |
| AP-3 | `src/projections.rs` | Mutable `&mut self` apply | Pure `apply_event_pure(self)` |
| AP-4 | `src/nats/` | No JetStream headers | Correlation/causation in headers |
| AP-5 | `src/domain.rs` | Hierarchical ownership | Graph-edge relationships |

---

## 6. Corrective Action Plan

### Sprint 11: Event Envelope Hardening (3 days)

1. [ ] Add `#[non_exhaustive]` to `EventEnvelope` struct
2. [ ] Implement IPLD serialization for events
3. [ ] Generate CID for each event on creation
4. [ ] Store events by CID, not sequential number

### Sprint 12: Pure Projection Pattern (2 days)

1. [ ] Convert `apply(&mut self, event)` to `apply_event_pure(self, event) -> Self`
2. [ ] Remove all `&mut` references in projection methods
3. [ ] Implement `with_*` builder pattern

### Sprint 13: NATS JetStream Integration (4 days)

1. [ ] Replace basic NATS publish with JetStream
2. [ ] Add correlation/causation headers
3. [ ] Implement exactly-once delivery via message ID

### Sprint 14: Graph-First Domain Model (5 days)

1. [ ] Remove `Vec<OrganizationUnit>` from `Organization`
2. [ ] Create `OrganizationUnitEdge` relationship type
3. [ ] Update all queries to use graph traversal

### Sprint 15: IPLD Content Addressing (5 days)

1. [ ] Add `cid` crate dependency
2. [ ] Implement `ToCid` trait for `EventEnvelope`
3. [ ] Store events in IPLD format on disk

---

## 7. Compliance Score Card

| Category | Current | Target | Gap |
|----------|---------|--------|-----|
| Event Sourcing | 75% | 100% | 25% |
| NATS Integration | 50% | 100% | 50% |
| Offline-First | 90% | 100% | 10% |
| Graph-Based | 60% | 100% | 40% |
| Content Addressing | 0% | 100% | 100% |
| **Overall** | **65%** | **100%** | **35%** |

---

## 8. Conclusion

The cim-keys repository demonstrates solid understanding of event sourcing principles. Primary gaps:

1. **Missing content addressing** - Events not CID-based
2. **Hierarchical remnants** - Some ownership patterns remain
3. **Incomplete NATS headers** - Correlation/causation not in messages
4. **Mutable projections** - `&mut self` pattern

**Priority Recommendation**: Begin with Sprint 11 (Event Hardening).
