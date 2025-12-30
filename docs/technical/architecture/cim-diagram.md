# CIM Keys Architecture & Integration Points

## Current Architecture (Partial Integration)

```mermaid
graph TB
    subgraph "cim-keys Module"
        subgraph "Domain Layer"
            CMD[Commands]
            EVT[Events]
            AGG[Aggregates]
            PROJ[Projections]
        end

        subgraph "Ports & Adapters"
            NPORT[NATS Port<br/>✅ Defined]
            SPORT[Storage Port<br/>✅ Defined]
            YPORT[YubiKey Port<br/>✅ Defined]

            NADPT[NATS Adapter<br/>❌ Stub Only]
            SADPT[Storage Adapter<br/>✅ JSON Files]
            YADPT[YubiKey Adapter<br/>✅ Implemented]
        end

        subgraph "Graph UI Layer"
            GUI[Graph GUI<br/>75% Complete]
            DO[DomainObject]
            DR[DomainRelationship]
            GE[GraphEvent]
        end
    end

    subgraph "Missing Integrations"
        NATS[NATS Cluster<br/>❌ Not Connected]
        IPLD[IPLD Store<br/>❌ Not Implemented]
        JS[JetStream<br/>❌ Not Used]
    end

    subgraph "Other CIM Modules"
        PERSON[cim-domain-person<br/>⚠️ Import Only]
        ORG[cim-domain-organization<br/>⚠️ Import Only]
        LOC[cim-domain-location<br/>⚠️ Import Only]
    end

    CMD --> AGG
    AGG --> EVT
    EVT --> PROJ
    PROJ --> SADPT

    GUI --> DO
    GUI --> DR
    GUI --> GE

    NPORT -.-> NADPT
    SPORT --> SADPT
    YPORT --> YADPT

    NADPT -.-> NATS
    EVT -.-> IPLD
    EVT -.-> JS

    PERSON -.-> DO
    ORG -.-> DO
    LOC -.-> DO

    style NATS fill:#ff6666
    style IPLD fill:#ff6666
    style JS fill:#ff6666
    style NADPT fill:#ffcc66
    style PERSON fill:#ffcc66
    style ORG fill:#ffcc66
    style LOC fill:#ffcc66
```

## Required CIM Integration Architecture

```mermaid
graph TB
    subgraph "CIM Ecosystem"
        subgraph "cim-keys Leaf Node"
            LEAF[Leaf Service<br/>Daemon]

            subgraph "Event Processing"
                EPUB[Event Publisher]
                ESUB[Event Subscriber]
                SAGA[Saga Coordinator]
            end

            subgraph "Domain Core"
                CMDS[Commands]
                EVTS[Events with CIDs]
                AGGS[Aggregates]
                PROJS[Projections]
            end

            subgraph "Infrastructure"
                NCLIENT[NATS Client]
                IPLDSTORE[IPLD Store]
                JETSTREAM[JetStream]
            end
        end

        subgraph "NATS Infrastructure"
            NCLUSTER[NATS Cluster]
            SUBJECTS[Subject Hierarchy<br/>org.unit.entity.op]
            STREAMS[Event Streams]
        end

        subgraph "Other Leaf Nodes"
            PERSON_LEAF[cim-domain-person]
            ORG_LEAF[cim-domain-organization]
            POLICY_LEAF[cim-domain-policy]
        end
    end

    LEAF --> NCLIENT
    NCLIENT <--> NCLUSTER

    CMDS --> AGGS
    AGGS --> EVTS
    EVTS --> EPUB
    EPUB --> SUBJECTS

    SUBJECTS --> STREAMS
    STREAMS --> ESUB
    ESUB --> SAGA
    SAGA --> CMDS

    EVTS --> IPLDSTORE
    IPLDSTORE --> JETSTREAM

    EVTS --> PROJS
    PROJS --> JETSTREAM

    PERSON_LEAF <--> NCLUSTER
    ORG_LEAF <--> NCLUSTER
    POLICY_LEAF <--> NCLUSTER

    style LEAF fill:#66ff66
    style NCLIENT fill:#66ff66
    style IPLDSTORE fill:#66ff66
    style JETSTREAM fill:#66ff66
```

## Event Flow with Proper CIM Integration

```mermaid
sequenceDiagram
    participant UI as Graph UI
    participant CMD as Command Handler
    participant AGG as Aggregate
    participant EVT as Event Generator
    participant CID as CID Generator
    participant PUB as NATS Publisher
    participant JS as JetStream
    participant IPLD as IPLD Store
    participant SUB as Other Modules

    UI->>CMD: User Action
    CMD->>AGG: Process Command
    AGG->>EVT: Generate Event
    EVT->>CID: Generate Content ID
    CID->>EVT: Event with CID

    par Publish to NATS
        EVT->>PUB: Publish Event
        PUB->>JS: Persist to Stream
        JS->>SUB: Notify Subscribers
    and Store in IPLD
        EVT->>IPLD: Store Payload
        IPLD-->>EVT: Return CID
    end

    SUB->>SUB: Process Event
    SUB->>PUB: Publish Response

    Note over JS,IPLD: Events are both streamed<br/>and content-addressed
```

## NATS Subject Hierarchy for cim-keys

```mermaid
graph LR
    subgraph "Subject Structure"
        ROOT[cim.keys.*]

        ROOT --> OP[Operations]
        ROOT --> EVT[Events]
        ROOT --> CMD[Commands]
        ROOT --> QUERY[Queries]

        OP --> OPGEN[cim.keys.op.generate]
        OP --> OPSIGN[cim.keys.op.sign]
        OP --> OPREV[cim.keys.op.revoke]

        EVT --> EVTGEN[cim.keys.event.generated]
        EVT --> EVTSIGN[cim.keys.event.signed]
        EVT --> EVTREV[cim.keys.event.revoked]

        CMD --> CMDGEN[cim.keys.cmd.generate]
        CMD --> CMDPROV[cim.keys.cmd.provision]

        QUERY --> QLIST[cim.keys.query.list]
        QUERY --> QGET[cim.keys.query.get]
    end
```

## Integration Priority Matrix

```mermaid
quadrantChart
    title Integration Priority vs Complexity
    x-axis Low Complexity --> High Complexity
    y-axis Low Priority --> High Priority
    quadrant-1 Quick Wins
    quadrant-2 Strategic
    quadrant-3 Low Priority
    quadrant-4 Complex but Critical

    "NATS Client": [0.8, 0.9]
    "Event Publishing": [0.3, 0.9]
    "JetStream": [0.5, 0.8]
    "IPLD/CID": [0.7, 0.7]
    "Event Subscription": [0.4, 0.8]
    "Saga Implementation": [0.9, 0.6]
    "Leaf Node Service": [0.6, 0.7]
    "Multi-Module Choreography": [0.8, 0.5]
    "FRP Completion": [0.5, 0.6]
    "Remove Deprecated": [0.2, 0.5]
```

## Module Dependency Graph

```mermaid
graph TD
    CK[cim-keys]
    CD[cim-domain<br/>✅ Core]
    CDP[cim-domain-person<br/>✅ Used]
    CDO[cim-domain-organization<br/>✅ Used]
    CDL[cim-domain-location<br/>✅ Used]
    CDPO[cim-domain-policy<br/>✅ Optional]
    CDA[cim-domain-agent<br/>✅ Optional]
    CG[cim-graph<br/>✅ Used]

    CK --> CD
    CK --> CDP
    CK --> CDO
    CK --> CDL
    CK -.-> CDPO
    CK -.-> CDA
    CK --> CG

    subgraph "Missing Dependencies"
        NATS_RS[async-nats<br/>❌ Not Used]
        CID_RS[cid<br/>❌ Not Used]
        IPLD_RS[ipld<br/>❌ Not Used]
    end

    CK -.-> NATS_RS
    CK -.-> CID_RS
    CK -.-> IPLD_RS

    style NATS_RS fill:#ff6666
    style CID_RS fill:#ff6666
    style IPLD_RS fill:#ff6666
```

## Key Integration Points Summary

| Component | Status | Required Action | Priority |
|-----------|--------|-----------------|----------|
| NATS Client | ❌ Stub Only | Implement real client | CRITICAL |
| Event Publishing | ❌ Local Only | Publish to NATS subjects | CRITICAL |
| Event Subscription | ❌ None | Subscribe to external events | HIGH |
| IPLD/CID | ❌ Not Implemented | Add content addressing | HIGH |
| JetStream | ❌ Not Used | Persist events to streams | HIGH |
| Leaf Node Service | ⚠️ Example Only | Create daemon service | MEDIUM |
| Saga Coordinator | ❌ None | Implement workflows | MEDIUM |
| Graph UI | ✅ 75% Complete | Finish migration | MEDIUM |
| FRP Axioms | ⚠️ 50% Complete | Implement missing axioms | LOW |

## Legend
- ✅ Implemented and working
- ⚠️ Partially implemented
- ❌ Not implemented / Critical gap
- Solid lines: Active connections
- Dotted lines: Missing/planned connections