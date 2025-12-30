# CIM Keys Architecture

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

## System Overview

```mermaid
graph TB
    subgraph "GUI Layer (Iced 0.13+)"
        V[View]
        I[Intent]
        M[Model]
    end

    subgraph "MVI Architecture"
        UP[Pure Update Function]
        T[Task/Command]
    end

    subgraph "Domain Layer"
        AGG[Aggregate]
        CMD[Commands]
        EVT[Events]
    end

    subgraph "Persistence Layer"
        PROJ[Projection]
        JSON[JSON Files]
        SD[Encrypted SD Card]
    end

    V -->|User Action| I
    I -->|Intent| UP
    M -->|Current State| UP
    UP -->|New Model| M
    UP -->|Task| T
    T -->|Command| CMD
    CMD -->|Processed by| AGG
    AGG -->|Generates| EVT
    EVT -->|Applied to| PROJ
    PROJ -->|Writes to| JSON
    JSON -->|Stored on| SD
    PROJ -->|Updates| M
```

## Domain Model

```mermaid
graph LR
    subgraph "Organization Bounded Context"
        ORG[Organization]
        UNIT[OrganizationUnit]
        PERSON[Person]
        LOC[Location]
    end

    subgraph "PKI Bounded Context"
        ROOT[Root CA]
        INT[Intermediate CA]
        LEAF[Leaf Certificate]
        KEY[CryptoKey]
    end

    subgraph "NATS Bounded Context"
        OP[NatsOperator]
        ACC[NatsAccount]
        USR[NatsUser]
    end

    subgraph "YubiKey Bounded Context"
        YK[YubiKey]
        SLOT[PIV Slot]
    end

    ORG -->|contains| UNIT
    ORG -->|employs| PERSON
    ORG -->|has locations| LOC

    PERSON -->|owns| KEY
    KEY -->|stored in| SLOT
    SLOT -->|belongs to| YK
    YK -->|assigned to| PERSON

    KEY -->|signs| LEAF
    INT -->|signs| LEAF
    ROOT -->|signs| INT

    ORG -->|operates| OP
    UNIT -->|maps to| ACC
    PERSON -->|maps to| USR
```

## Event Flow

```mermaid
sequenceDiagram
    participant U as User
    participant V as View
    participant M as Model
    participant UP as Update
    participant A as Aggregate
    participant P as Projection

    U->>V: Click "Create Person"
    V->>M: Intent::UiCreatePerson
    M->>UP: update(model, intent)
    UP->>UP: Pure transformation
    UP->>M: (new_model, Task::perform)

    Note over M,A: Task executes command
    M->>A: CreatePerson command
    A->>A: Validate against projection
    A->>P: PersonCreated event
    P->>P: Apply event to state
    P->>M: Intent::DomainPersonCreated

    M->>UP: update(model, intent)
    UP->>M: Model with new person
    M->>V: Re-render view
```

## LiftableDomain Pattern

```mermaid
graph TB
    subgraph DOMAIN["Domain Types"]
        O["Organization"]
        OU["OrganizationUnit"]
        P["Person"]
        L["Location"]
    end

    subgraph FUNCTOR["Lifting Functor"]
        LIFT["LiftableDomain::lift"]
        UNLIFT["LiftableDomain::unlift"]
    end

    subgraph GRAPH["Lifted Graph"]
        LN["LiftedNode"]
        LE["LiftedEdge"]
        LG["LiftedGraph"]
    end

    O -->|"lift"| LN
    OU -->|"lift"| LN
    P -->|"lift"| LN
    L -->|"lift"| LN

    LN -->|"unlift"| O
    LN -->|"unlift"| OU
    LN -->|"unlift"| P
    LN -->|"unlift"| L

    LN --> LG
    LE --> LG
```

## MVI Intent Categorization

```mermaid
graph LR
    subgraph INTENTS["Intent Categories"]
        UI["Ui Intents"]
        PORT["Port Intents"]
        DOM["Domain Intents"]
        SYS["System Intents"]
        ERR["Error Intents"]
    end

    subgraph ORIGINS["Origins"]
        USER["User Interaction"]
        FILE["File I/O"]
        AGG["Aggregate Events"]
        TIMER["Timers/Ticks"]
        FAIL["Failures"]
    end

    USER --> UI
    FILE --> PORT
    AGG --> DOM
    TIMER --> SYS
    FAIL --> ERR
```

## Testing Architecture

```mermaid
graph TB
    subgraph TESTS["Test Layers"]
        UNIT["Unit Tests - 341 tests"]
        MVI["MVI Tests - 33 tests"]
        BDD["BDD Tests - 18 tests"]
        PROP["Property Tests - 7 proptest"]
    end

    subgraph SPECS["Specifications"]
        GHERKIN["Gherkin Scenarios - 112 scenarios"]
    end

    subgraph COV["Coverage"]
        LIB["src/ library code"]
        GUI["src/gui/ components"]
        AGG["src/aggregate.rs"]
    end

    UNIT --> LIB
    MVI --> GUI
    BDD --> AGG
    GHERKIN -.->|documents| BDD
    PROP --> MVI
```

## Projection Storage

```mermaid
graph TB
    subgraph SD["Encrypted SD Card"]
        MAN[manifest.json]

        subgraph DOM["domain/"]
            ORG_J[organization.json]
            PEOPLE["people/"]
            RELS[relationships.json]
        end

        subgraph KEYS["keys/"]
            KEY_DIR["key-id/"]
            META[metadata.json]
            PUB[public.pem]
        end

        subgraph CERTS["certificates/"]
            ROOT_DIR["root-ca/"]
            INT_DIR["intermediate-ca/"]
            LEAF_DIR["leaf/"]
        end

        subgraph NATS["nats/"]
            OP_DIR["operator/"]
            ACC_DIR["accounts/"]
            USR_DIR["users/"]
        end

        subgraph EVT["events/"]
            DATE["date/"]
            EVT_LOG["events.jsonl"]
        end
    end
```

## FRP Axiom Compliance

| Axiom | Status | Implementation |
|-------|--------|----------------|
| A3: Decoupled | ✅ | `update()` output depends only on input |
| A4: Causality | ✅ | UUID v7 + causation tracking (see below) |
| A5: Totality | ✅ | All `with_*` methods are total |
| A7: Event Logs | ✅ | Events stored as timestamped prefixes |
| A9: Composition | ✅ | Associativity verified by proptest |

### A4 Causality Implementation

**Status**: ✅ **COMPLETE** - All 44 instances migrated, 348 tests passing.

**Pattern Applied**:
- **Root events**: `causation_id = Some(event_id)` (self-reference)
- **Derived events**: `causation_id = Some(parent_id)` (reference to parent)

**Helper Functions** (`src/graph_projection.rs`):
```rust
fn create_root_event(&self, aggregate_id: Uuid, payload: EventPayload) -> GraphEvent {
    let event_id = Uuid::now_v7();
    GraphEvent {
        event_id,
        causation_id: Some(event_id), // A4: Self-reference for root
        // ...
    }
}

fn create_derived_event(&self, aggregate_id: Uuid, causation_id: Uuid, payload: EventPayload) -> GraphEvent {
    GraphEvent {
        event_id: Uuid::now_v7(),
        causation_id: Some(causation_id), // A4: Reference to parent
        // ...
    }
}
```

**Causation Module** (`src/causation.rs`):
```rust
use cim_keys::causation::Causation;

// Root event (no prior cause - self-reference)
let causation = Causation::root(event_id);

// Event caused by a command
let causation = Causation::from_command(command_id);

// Event caused by another event
let causation = Causation::from_event(parent_event_id);

// NEVER returns None
assert!(causation.to_option().is_some());
```

## UUID v7 Causality Architecture

**Critical for Security Audit Trail**

All entity and event IDs use UUID v7 (`Uuid::now_v7()`), which provides:

```mermaid
flowchart LR
    subgraph UUID7["UUID v7 Structure - 128 bits"]
        TS["48-bit timestamp"]
        VER["4-bit ver"]
        RAND["12-bit rand"]
        SEQ["64-bit rand+seq"]
    end
    TS --> VER --> RAND --> SEQ
    NOTE["Causality embedded at creation time"]
    UUID7 -.-> NOTE
```

### Causality Guarantees

| Property | Guarantee | Mechanism |
|----------|-----------|-----------|
| **Temporal Ordering** | Events created later have larger UUIDs | Timestamp in bits 0-47 |
| **Audit Trail** | Complete history reconstructable | Sort by UUID = chronological order |
| **No Extra Timestamps** | Timestamp derived from ID itself | `uuid.get_timestamp()` extracts time |
| **Immutable Creation Time** | Cannot be altered post-creation | UUID is the identity |

### Complete Causality Model

**MANDATORY**: Use `cim_domain::MessageFactory` to create message identities with automatic causation tracking.

```rust
use cim_domain::{MessageFactory, MessageIdentity, CausationId};

// MessageIdentity carries the full causality chain
pub struct MessageIdentity {
    pub correlation_id: CorrelationId,  // Shared across entire transaction
    pub causation_id: CausationId,      // Immediate parent's message_id
    pub message_id: Uuid,               // This message's UUID v7
}
```

### MessageFactory Pattern (REQUIRED)

```rust
use cim_domain::MessageFactory;

// ROOT COMMAND: causation = self (I am the root cause)
let root_id = Uuid::now_v7();
let root_identity = MessageFactory::create_root_command(root_id);
// Result: correlation = root_id, causation = root_id, message_id = root_id

// DERIVED COMMAND: causation = parent's message_id
let child_id = Uuid::now_v7();
let child_identity = MessageFactory::command_from_command(child_id, &root_identity);
// Result: correlation = root_id, causation = root_id, message_id = child_id

// EVENT FROM COMMAND: causation = command's message_id
let event_id = Uuid::now_v7();
let event_identity = MessageFactory::command_from_event(event_id, &command_identity);
// Result: correlation preserved, causation = command's message_id
```

### Causation Rules

| Message Type | causation_id Value |
|--------------|-------------------|
| Root command | `self.message_id` (self-reference) |
| Command from command | `parent.message_id` |
| Command from event | `parent.message_id` |
| Event from command | `command.message_id` |
| Query from event | `event.message_id` |

### Anti-Pattern: Manual causation_id

```rust
// ❌ WRONG - manual None breaks audit trail
DomainEvent {
    event_id: Uuid::now_v7(),
    causation_id: None,  // AUDIT TRAIL BROKEN
    // ...
}

// ❌ WRONG - manual Option<Uuid> is error-prone
causation_id: Some(cmd.command_id),  // Easy to forget

// ✅ CORRECT - use MessageFactory
let identity = MessageFactory::command_from_command(event_id, &cmd.identity);
// causation_id automatically set to cmd.identity.message_id
```

### Three-Level Causality

```mermaid
flowchart TB
    subgraph TEMPORAL["1. TEMPORAL - UUID v7"]
        EID["event_id"]
        TS["timestamp embedded"]
        EID --> TS
    end

    subgraph CORRELATION["2. CORRELATION"]
        CID["correlation_id"]
        TXN["transaction grouping"]
        CID --> TXN
    end

    subgraph CAUSATION["3. CAUSATION"]
        CMD["Command A"]
        EVT["Event B"]
        CMD -->|"causes"| EVT
        CMD -.->|"causation_id"| ROOT["what caused A"]
        EVT -.->|"causation_id"| CMD
    end
```

### Audit Trail Reconstruction

```rust
// Full audit trail from any event
fn reconstruct_audit_trail(event_id: Uuid) -> AuditTrail {
    AuditTrail {
        // 1. Temporal: When did this happen?
        timestamp: event_id.get_timestamp(),

        // 2. Correlation: What transaction is this part of?
        transaction_events: events_by_correlation_id(event.correlation_id),

        // 3. Causation: What chain of events led here?
        causality_chain: build_causality_chain(event_id),
    }
}

// Extract timestamp when needed for audit
let created_at = event.event_id.get_timestamp()
    .expect("UUID v7 always has timestamp");
```

### Audit Query Patterns

```rust
// Get all events in chronological order
events.sort_by_key(|e| e.event_id);  // UUID v7 sorts temporally

// Find events in time range
let start_uuid = Uuid::new_v7(Timestamp::from_unix(start_time, 0));
let end_uuid = Uuid::new_v7(Timestamp::from_unix(end_time, 0));
events.filter(|e| e.event_id >= start_uuid && e.event_id <= end_uuid);

// Reconstruct causality chain
fn build_causality_chain(event: &Event, all: &[Event]) -> Vec<&Event> {
    let mut chain = vec![event];
    let mut current = event;
    while let Some(cause_id) = current.causation_id {
        if let Some(cause) = all.iter().find(|e| e.event_id == cause_id) {
            chain.push(cause);
            current = cause;
        } else { break; }
    }
    chain.reverse();  // Oldest first
    chain
}
```

### Why UUID v7 Over Separate Timestamps

| Approach | Problem | UUID v7 Solution |
|----------|---------|------------------|
| Separate `created_at` field | Can be inconsistent with ID | Timestamp IS the ID |
| Clock skew between fields | ID and time can disagree | Single source of truth |
| Storage overhead | Extra 8+ bytes per event | Zero overhead |
| Query complexity | Join on time OR id | Single index suffices |

## Module Dependencies

```mermaid
graph BT
    subgraph "Core"
        TYPES[types]
        EVENTS[events]
        COMMANDS[commands]
    end

    subgraph "Domain"
        DOMAIN[domain]
        AGGREGATE[aggregate]
        PROJECTIONS[projections]
    end

    subgraph "Infrastructure"
        CRYPTO[crypto]
        PORTS[ports]
        ADAPTERS[adapters]
    end

    subgraph "Presentation"
        GUI[gui]
        MVI[mvi]
        LIFTING[lifting]
    end

    EVENTS --> TYPES
    COMMANDS --> TYPES
    AGGREGATE --> EVENTS
    AGGREGATE --> COMMANDS
    PROJECTIONS --> EVENTS

    DOMAIN --> TYPES
    CRYPTO --> TYPES

    GUI --> MVI
    GUI --> DOMAIN
    MVI --> DOMAIN
    LIFTING --> DOMAIN

    ADAPTERS --> PORTS
```

## Key Patterns

### 1. Command-Event Flow
```
CreatePerson → Aggregate.handle() → PersonCreated → Projection.apply()
```

### 2. Immutable Model Updates
```rust
model.with_tab(Tab::Organization)
     .with_person_added(person)
     .with_status(Status::Ready)
```

### 3. Intent Routing
```rust
match intent {
    Intent::UiTabSelected(tab) => // UI handler
    Intent::PortFileLoaded(data) => // Port handler
    Intent::DomainPersonCreated(p) => // Domain handler
}
```

### 4. LiftableDomain Functor
```rust
graph.add(&organization);  // Organization → LiftedNode
let orgs: Vec<Organization> = graph.unlift_all();  // LiftedNode → Organization
```
