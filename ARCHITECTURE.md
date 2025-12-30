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
    subgraph "Domain Types"
        O[Organization]
        OU[OrganizationUnit]
        P[Person]
        L[Location]
    end

    subgraph "Lifting Functor"
        LIFT[LiftableDomain::lift]
        UNLIFT[LiftableDomain::unlift]
    end

    subgraph "Lifted Graph"
        LN[LiftedNode]
        LE[LiftedEdge]
        LG[LiftedGraph]
    end

    O -->|lift()| LN
    OU -->|lift()| LN
    P -->|lift()| LN
    L -->|lift()| LN

    LN -->|unlift()| O
    LN -->|unlift()| OU
    LN -->|unlift()| P
    LN -->|unlift()| L

    LN --> LG
    LE --> LG
```

## MVI Intent Categorization

```mermaid
graph LR
    subgraph "Intent Categories"
        UI[Ui* Intents]
        PORT[Port* Intents]
        DOM[Domain* Intents]
        SYS[System* Intents]
        ERR[Error* Intents]
    end

    subgraph "Origins"
        USER[User Interaction]
        FILE[File I/O]
        AGG[Aggregate Events]
        TIMER[Timers/Ticks]
        FAIL[Failures]
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
    subgraph "Test Layers"
        UNIT[Unit Tests<br>341 tests]
        MVI[MVI Tests<br>33 tests]
        BDD[BDD Tests<br>18 tests]
        PROP[Property Tests<br>7 proptest]
    end

    subgraph "Specifications"
        GHERKIN[Gherkin Scenarios<br>112 scenarios]
    end

    subgraph "Coverage"
        LIB[src/**/*.rs]
        GUI[src/gui/*]
        AGG[src/aggregate.rs]
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
    subgraph "Encrypted SD Card"
        MAN[manifest.json]

        subgraph "domain/"
            ORG_J[organization.json]
            PEOPLE[people/]
            RELS[relationships.json]
        end

        subgraph "keys/"
            KEY_DIR[{key-id}/]
            META[metadata.json]
            PUB[public.pem]
        end

        subgraph "certificates/"
            ROOT_DIR[root-ca/]
            INT_DIR[intermediate-ca/]
            LEAF_DIR[leaf/]
        end

        subgraph "nats/"
            OP_DIR[operator/]
            ACC_DIR[accounts/]
            USR_DIR[users/]
        end

        subgraph "events/"
            DATE[{date}/]
            EVT_LOG[*.jsonl]
        end
    end
```

## FRP Axiom Compliance

| Axiom | Status | Implementation |
|-------|--------|----------------|
| A3: Decoupled | ✅ | `update()` output depends only on input |
| A4: Causality | ⚠️ | Runtime tracking via correlation_id |
| A5: Totality | ✅ | All `with_*` methods are total |
| A7: Event Logs | ✅ | Events stored as timestamped prefixes |
| A9: Composition | ✅ | Associativity verified by proptest |

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
