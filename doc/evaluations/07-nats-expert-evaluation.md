# NATS Messaging Pattern Evaluation for cim-keys

**Evaluator:** NATS Expert
**Date:** 2026-01-02
**Overall NATS Compliance: 40%**

---

## 1. Current NATS Architecture Analysis

### Port/Adapter Structure Found

```
src/
├── ports/
│   └── nats.rs          # NatsPort trait definition
├── adapters/
│   └── nsc.rs           # NSC adapter for NATS security
└── nats_operations.rs   # NATS operation commands
```

### NatsPort Trait Definition

```rust
pub trait NatsPort: Send + Sync {
    async fn create_operator(&self, name: &str) -> Result<String, KeyError>;
    async fn create_account(&self, operator: &str, account: &str) -> Result<String, KeyError>;
    async fn create_user(&self, account: &str, user: &str) -> Result<String, KeyError>;
    async fn generate_credentials(&self, account: &str, user: &str) -> Result<String, KeyError>;
    async fn set_permissions(&self, account: &str, user: &str, pub_subjects: &[String], sub_subjects: &[String]) -> Result<(), KeyError>;
}
```

---

## 2. Critical Findings

### Finding 1: No Real NATS Client Integration

**Severity: HIGH**

The repository has NO actual async-nats client dependency. The NatsPort is exclusively for NSC (NATS Security CLI) operations, not for actual message publishing/subscribing.

### Finding 2: No JetStream Configuration

**Severity: HIGH**

JetStream is completely absent:
- No stream definitions
- No consumer configurations
- No persistence layer for events
- No exactly-once delivery guarantees

### Finding 3: Subject Pattern Violations

**Severity: MEDIUM**

**Expected CIM Subject Pattern:**
```
<organization>.<unit>.<domain>.<entity>.<operation>
cowboyai.security.keys.operator.create
```

**Current Implementation:** No subjects defined - operations are CLI commands, not NATS messages.

### Finding 4: Missing NATS Headers

**Severity: HIGH**

Required CIM headers:
- `CIM-Correlation-Id`
- `CIM-Causation-Id`
- `CIM-Event-Type`
- `CIM-Timestamp`
- `CIM-Source`

**Current State:** Headers not implemented anywhere.

### Finding 5: Event Envelope Not Propagated to NATS

Events have correlation/causation IDs in the struct but are NOT propagated to NATS headers because there is no NATS publishing implementation.

---

## 3. Anti-Patterns Identified

### Anti-Pattern 1: CLI-Based NSC Instead of Programmatic NATS

```rust
// Current: Shelling out to CLI
let output = Command::new(&self.nsc_path)
    .args(["add", "operator", name])
    .output()
    .await?;
```

**Problem:**
- Subprocess spawning is not WASM-compatible
- No retry logic or circuit breakers
- Error handling limited to exit codes

### Anti-Pattern 2: Missing Event Publishing Pipeline

Events are generated but never published to NATS.

### Anti-Pattern 3: No Consumer Acknowledgment Strategy

No acknowledgment patterns found.

---

## 4. Required Corrections

### Correction 1: Add async-nats Dependency

```toml
[dependencies]
async-nats = "0.35"

[features]
nats = ["async-nats"]
```

### Correction 2: Implement JetStream Event Publishing

```rust
pub trait JetStreamPort: Send + Sync {
    async fn publish_event<E: CimEvent>(&self, event: E) -> Result<(), KeyError>;
    async fn subscribe(&self, subject: &str) -> Result<EventStream, KeyError>;
}
```

### Correction 3: Define CIM Header Specification

```rust
pub const CIM_CORRELATION_ID: &str = "CIM-Correlation-Id";
pub const CIM_CAUSATION_ID: &str = "CIM-Causation-Id";
pub const CIM_EVENT_TYPE: &str = "CIM-Event-Type";
pub const CIM_TIMESTAMP: &str = "CIM-Timestamp";
pub const CIM_SOURCE: &str = "CIM-Source";
pub const NATS_MSG_ID: &str = "Nats-Msg-Id";  // For deduplication

pub fn build_headers<E: CimEvent>(event: &E) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CIM_CORRELATION_ID, event.correlation_id().to_string());
    if let Some(causation) = event.causation_id() {
        headers.insert(CIM_CAUSATION_ID, causation.to_string());
    }
    headers.insert(CIM_EVENT_TYPE, event.event_type());
    headers.insert(CIM_TIMESTAMP, event.timestamp().to_rfc3339());
    headers.insert(CIM_SOURCE, "cim-keys");
    headers.insert(NATS_MSG_ID, event.message_id().to_string());
    headers
}
```

### Correction 4: Define Subject Hierarchy

```rust
pub mod subjects {
    pub const KEY_GENERATED: &str = "cowboyai.security.keys.key.generated";
    pub const KEY_REVOKED: &str = "cowboyai.security.keys.key.revoked";
    pub const CERTIFICATE_ISSUED: &str = "cowboyai.security.keys.certificate.issued";
    pub const OPERATOR_CREATED: &str = "cowboyai.security.keys.operator.created";
    pub const ACCOUNT_CREATED: &str = "cowboyai.security.keys.account.created";
    pub const USER_CREATED: &str = "cowboyai.security.keys.user.created";
}
```

### Correction 5: JetStream Stream Configuration

```rust
pub fn keys_events_stream_config() -> Config {
    Config {
        name: "KEYS_EVENTS".to_string(),
        subjects: vec!["cowboyai.security.keys.>".to_string()],
        retention: RetentionPolicy::Limits,
        max_age: std::time::Duration::from_secs(90 * 24 * 60 * 60), // 90 days
        storage: StorageType::File,
        duplicate_window: std::time::Duration::from_secs(120),
        ..Default::default()
    }
}
```

---

## 5. Sprint Planning - Corrective Actions

### Sprint N+1: NATS Foundation (5 story points)

| Task | Priority | Effort |
|------|----------|--------|
| Add async-nats dependency with feature flag | HIGH | 1 |
| Create CIM header specification module | HIGH | 1 |
| Define subject hierarchy constants | HIGH | 1 |
| Create JetStreamPort trait | HIGH | 2 |

### Sprint N+2: JetStream Integration (8 story points)

| Task | Priority | Effort |
|------|----------|--------|
| Implement JetStreamAdapter | HIGH | 3 |
| Configure KEYS_EVENTS stream | HIGH | 2 |
| Add event publishing to aggregate | HIGH | 2 |
| Write integration tests with real NATS | MEDIUM | 1 |

### Sprint N+3: Event Consumption (5 story points)

| Task | Priority | Effort |
|------|----------|--------|
| Create durable consumer configuration | HIGH | 2 |
| Implement event replay for projections | HIGH | 2 |
| Add circuit breaker for NATS connection | MEDIUM | 1 |

### Sprint N+4: NSC Migration (3 story points)

| Task | Priority | Effort |
|------|----------|--------|
| Replace CLI shelling with nkeys crate | MEDIUM | 2 |
| WASM-compatible key generation | LOW | 1 |

---

## 6. Compliance Checklist

| Requirement | Current | Target | Gap |
|-------------|---------|--------|-----|
| Subject Pattern | Not implemented | `org.unit.domain.entity.op` | CRITICAL |
| JetStream Streams | Missing | KEYS_EVENTS stream | CRITICAL |
| CIM Headers | Missing | All 7 headers | CRITICAL |
| Correlation Tracking | In payload only | Payload + headers | HIGH |
| Exactly-Once | None | Msg-Id deduplication | HIGH |

---

## 7. Conclusion

The cim-keys repository has solid foundations with hexagonal architecture and event sourcing patterns, but **NATS integration is fundamentally incomplete**. The NatsPort exists only for NSC security operations, not for actual message publishing.

**Estimated Total Effort:** 21 story points across 4 sprints
