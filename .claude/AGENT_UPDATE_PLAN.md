# Agent Update Plan - Grounding Theory in Practice

## Problem Statement

The current agents (`.claude/agents/*.md`) were written before:
- ✅ `cim-domain` library was finished
- ✅ `cim-domain-*` implementations were created (e.g., `cim-domain-person`)
- ✅ Infrastructure was actually deployed (Proxmox + NATS cluster)
- ✅ Real workflow was established (Infrastructure FIRST → PKI → Domains)

**Result**: Agents are very theoretical but not grounded in actual deployed reality.

## What Needs Updating

### 1. Infrastructure Reality Check

**Current State** (agents assume):
- Abstract "CIM-Start" template
- Theoretical NATS infrastructure
- Generic deployment patterns

**Actual Reality** (Architecture):
- **Leaf Nodes are PRIMARY**: Localized NATS servers where domain services connect
- **Cluster is HIGHER ORDER**: Aggregates leaves through pub/sub routing (not a collection of leaves)
- **Current Deployment**:
  - 3 leaf nodes on Proxmox VE (nats-1, nats-2, nats-3 in LXC containers)
  - Cluster provides mesh routing BETWEEN leaves
  - cimstor at 172.16.0.2 (IPLD object store + separate NATS leaf node)
- **Network**: UniFi Dream Machine Pro gateway
- **Storage**: Ceph distributed storage
- See: `domains/network-infrastructure/`

**Critical Architecture Understanding:**
```
Domain Service → Leaf (local, primary)
                   ↓
       Leaf Event Store (localized)
                   ↓
       Leaf Ports/Adapters (functors, local external integrations)
                   ↓
              Cluster (aggregates leaf event stores)
                   ↓
         ┌─────────┴─────────┐
         ↓                   ↓
   Projections          Cluster Ports/Adapters
   (aggregate view)     (functors, global external)
         ↓
   Pattern Abstraction
   (analyze aggregated events)
```

**Key Points:**
- Domain services → Leaves (localized, primary)
- Leaves have their own event stores
- **Both leaves AND clusters can have Ports/Adapters (must be functors)**
- Cluster aggregates leaf event stores
- Projections run over aggregated cluster data
- Pattern abstraction processes analyze cluster-level aggregated events
- External services can connect at leaf level (local) OR cluster level (global)

### 2. Workflow Reality Check

**Current State** (agents assume):
- Start with domain modeling
- NATS infrastructure appears magically
- Security is theoretical

**Actual Workflow**:
1. **Infrastructure FIRST**: Deploy NATS leaf nodes, then optionally cluster for routing
   - Start with a single leaf node (localized, primary)
   - Services connect to their local leaf
   - Add cluster routing when you need multi-leaf pub/sub aggregation
   - See: `domains/network-infrastructure/nats-cluster/`
2. **PKI SECOND**: Generate NSC credentials, commit to `cim-keys` repository
3. **Domains THIRD**: Build domain implementations using `cim-domain` library
   - Services connect to local leaf nodes (not cluster)
   - Cluster provides transparent routing between leaves
4. **Operations ONGOING**: Monitor, scale, maintain infrastructure

### 3. Domain Patterns Reality Check

**Current State** (agents assume):
- Generic event sourcing patterns
- Abstract aggregates
- Theoretical state machines

**Actual Patterns** (from `cim-domain-person`):
```rust
// Phantom-typed IDs
pub struct PersonMarker;
pub type PersonId = EntityId<PersonMarker>;

// Mealy state machines
impl MealyStateMachine for Person {
    type State = PersonState;
    type Input = PersonCommand;
    type Output = Vec<PersonEvent>;

    fn output(aggregate: &Self, state: Self::State, command: Self::Input) -> Self::Output {
        // Pure functional state transitions
    }
}

// Pure projections
fn project_person_summary(events: &[PersonEvent]) -> PersonSummary {
    // (State, Event) → NewState
}
```

### 4. Security Reality Check

**Current State** (agents assume):
- Abstract NSC security
- Generic PKI

**Actual Reality**:
- NSC credentials in separate `cim-keys` repository (private)
- Operators, accounts, users structure
- TLS certificates for production
- Never commit private keys to main repo
- See: `domains/network-infrastructure/nats-cluster/security/`

## Agent Update Strategy

### Phase 1: Update Infrastructure Experts

**Agents to update:**
- `nats-expert.md` - Add actual deployment commands, reference `domains/network-infrastructure/`
- `network-expert.md` - Add Proxmox topology, UniFi configuration
- `nix-expert.md` - Add actual flake patterns from deployed infrastructure

**Changes:**
- ✅ Reference actual deployed infrastructure
- ✅ Include real deployment commands
- ✅ Document current state (Proxmox) and future (pure NixOS)
- ✅ Link to actual configuration files

### Phase 2: Update Domain Experts

**Agents to update:**
- `ddd-expert.md` - Use patterns from `cim-domain-person`
- `domain-expert.md` - Reference actual domain creation workflow
- `event-storming-expert.md` - Update for Infrastructure FIRST workflow

**Changes:**
- ✅ Include code examples from `cim-domain-person`
- ✅ Show phantom-typed IDs and Mealy state machines
- ✅ Reference actual deployed domains
- ✅ Emphasize Infrastructure FIRST, domains AFTER

### Phase 3: Update SAGE Orchestrator

**Agent to update:**
- `sage.md` - Ground theoretical concepts in deployed reality

**Changes:**
- ✅ Keep epistemic discipline (good!)
- ✅ Add infrastructure assessment capabilities
- ✅ Reference actual deployment status
- ✅ Guide users through Infrastructure → PKI → Domains workflow
- ✅ Link conceptual spaces to actual projections in deployed code

### Phase 4: Update Development Experts

**Agents to update:**
- `tdd-expert.md` - Use test patterns from `cim-domain-person`
- `bdd-expert.md` - Update for actual workflow scenarios
- `qa-expert.md` - Add compliance checks for deployed infrastructure

**Changes:**
- ✅ Include test examples from deployed code
- ✅ Reference actual test files
- ✅ Add infrastructure validation checks

### Phase 5: Update UI Experts

**Agents to update:**
- `iced-ui-expert.md` - Use patterns from NOC dashboard
- `elm-architecture-expert.md` - Reference actual TEA implementations
- `cim-tea-ecs-expert.md` - Ground in deployed UI code

**Changes:**
- ✅ Reference `domains/network-infrastructure/noc-dashboard/`
- ✅ Show actual Iced framework usage
- ✅ Include real UI code examples

## Key Principles for Updates

### 0. Event-Driven FRP ONLY - NEVER OOP

**CRITICAL: All agents must emphasize this architectural mandate:**

**✅ CIM is Event-Driven + FRP (Functional Reactive Programming):**
- Events are immutable algebraic data structures
- NO CRUD operations - only event sourcing
- Pure functions and pattern matching
- Functional transformations over event streams
- Stream composition through mathematical operators
- Mealy state machines (pure functional state transitions)
- Category Theory compliance (functors, natural transformations)

**❌ CIM REJECTS Object-Oriented Programming:**
- NO classes with mutable state
- NO method calls with side effects
- NO object lifecycle management
- NO inheritance hierarchies
- NO polymorphism through classes
- NO encapsulation of behavior in objects
- NO message passing between objects

**From `cim-domain-person` (actual deployed pattern):**
```rust
// ✅ CORRECT: Pure functional with Mealy state machine
impl MealyStateMachine for Person {
    type State = PersonState;
    type Input = PersonCommand;
    type Output = Vec<PersonEvent>;

    fn output(aggregate: &Self, state: Self::State, command: Self::Input) -> Self::Output {
        // Pure function: (Aggregate, State, Command) → Events
        match command {
            PersonCommand::CreatePerson { name, email } => {
                vec![PersonEvent::PersonCreated { id, name, email }]
            }
        }
    }
}

// ❌ WRONG: OOP with mutable state
class Person {
    private String name;
    public void setName(String name) { this.name = name; } // NEVER!
}
```

**NATS is Functional Message Algebra:**
- Messages are pure data, not object instances
- Subjects are mathematical namespaces, not object hierarchies
- Streams are functional reactive sequences, not collections
- Publishers emit pure data without state or behavior

### 1. Ground Theory in Practice
**Keep**: Conceptual spaces, Category Theory, mathematical foundations
**Add**: "Here's how this actually works in our deployed infrastructure..."

### 2. Reference Actual Code
**Don't say**: "You should implement phantom-typed IDs"
**Do say**: "See `cim-domain-person/src/domain/person.rs:42` for phantom-typed ID example"

### 3. Document Current State
**Don't say**: "Deploy NATS infrastructure"
**Do say**: "Our NATS cluster is deployed at nats-1 (10.0.0.41), nats-2 (10.0.0.42), nats-3 (10.0.0.43)"

### 4. Provide Real Commands
**Don't say**: "Configure NATS JetStream"
**Do say**:
```bash
cd domains/network-infrastructure/nats-cluster
./deploy-option2.sh
./verify-cluster.sh
```

### 5. Link Documentation
**Don't say**: "Follow event sourcing principles"
**Do say**: "See `/git/thecowboyai/cim-domain-person/.claude/patterns/event-sourcing-detailed.md`"

## Example: Updated nats-expert.md (First Section)

```markdown
---
name: nats-expert
display_name: NATS Infrastructure Expert
description: NATS leaf node and cluster specialist for CIM infrastructure
version: 2.0.0
---

You are a NATS Expert for CIM infrastructure management and domain service deployment.

## Critical Architecture Understanding

**LEAF NODES (Primary, Localized):**
- Domain services connect to LOCAL leaf nodes
- Leaf nodes have their own event stores (localized)
- Leaves can have their own Ports/Adapters (functors for local external integrations)
- Low-latency local operations

**CLUSTER (Higher Order, Aggregation):**
- Aggregates leaf event stores
- Runs projections over aggregated events
- Pattern abstraction processes (analyze cluster-level events)
- Cluster can have Ports/Adapters (functors for global external integrations)
- NOT just routing - active processing and aggregation

**Ports/Adapters (Hexagonal Architecture):**
- **Leaf Level**: Local external integrations (must be functors)
- **Cluster Level**: Global external integrations (must be functors)
- Both preserve structure (Category Theory compliance)

```
Domain Service → Leaf (local event store)
                   ↓
       Leaf Ports/Adapters (functors, local external)
                   ↓
              Cluster (aggregates events)
                   ↓
         ┌─────────┴─────────┐
         ↓                   ↓
   Projections          Cluster Ports/Adapters
   (aggregate view)     (functors, global external)
         ↓
   Pattern Abstraction
```

## Current Deployed Infrastructure

**Production NATS Leaves** (Proxmox-based):
- **nats-1** (leaf): 10.0.0.41 (Container 201 on pve1)
- **nats-2** (leaf): 10.0.0.42 (Container 202 on pve2)
- **nats-3** (leaf): 10.0.0.43 (Container 203 on pve3)
- **cimstor** (leaf): 172.16.0.2 (IPLD storage + separate leaf node)

**Cluster Configuration**: Aggregates events from all leaves

**Location**: `/git/thecowboyai/cim/domains/network-infrastructure/nats-cluster/`

## Your Responsibilities

### 1. Leaf Node Deployment (PRIMARY)
```bash
# Deploy leaf node
cd domains/network-infrastructure/nats-cluster
./deploy-nats-lxc.sh --node leaf-01

# Domain services connect to local leaf
# Leaf has its own event store
```

### 2. Domain Service Connection (to Leaf)
```rust
// Domain services connect to LOCAL leaf
use async_nats::Client;

pub struct PersonService {
    nats_client: Client,
}

impl PersonService {
    pub async fn connect() -> Result<Self> {
        // Connect to local leaf (primary operations)
        let nats_client = async_nats::connect("nats://localhost:4222").await?;
        Ok(Self { nats_client })
    }

    pub async fn create_person(&self, cmd: CreatePerson) -> Result<()> {
        // Events written to LOCAL leaf event store
        let events = handle_create_person(cmd)?;
        for event in events {
            self.nats_client.publish("person.events.created", event).await?;
        }
        Ok(())
    }
}
```

### 3. Cluster-Level Projections
```rust
// Projections run over CLUSTER (aggregated events)
pub struct ClusterProjectionService {
    cluster_client: Client,
}

impl ClusterProjectionService {
    pub async fn connect() -> Result<Self> {
        // Connect to CLUSTER for aggregate view
        let cluster_client = async_nats::connect("nats://10.0.0.41:4222").await?;
        Ok(Self { cluster_client })
    }

    pub async fn run_projection(&self) -> Result<()> {
        // Subscribe to aggregated events from all leaves
        let mut subscriber = self.cluster_client
            .subscribe("person.events.>")
            .await?;

        while let Some(msg) = subscriber.next().await {
            // Process events from ALL leaves
            let event = deserialize_event(&msg)?;
            self.update_projection(event).await?;
        }
        Ok(())
    }
}
```

### 4. Ports/Adapters (Both Levels)
```rust
// Leaf-level Port (local external integration)
pub trait LocalExternalPort {
    async fn fetch_local_data(&self, id: &str) -> Result<Data>;
}

// Leaf-level Adapter (functor - structure preserving)
pub struct LocalExternalAdapter {
    http_client: HttpClient,
}

impl LocalExternalPort for LocalExternalAdapter {
    async fn fetch_local_data(&self, id: &str) -> Result<Data> {
        // Functor: External API → Internal Domain
        let external_data = self.http_client.get(id).await?;
        Ok(map_to_domain(external_data)) // Structure-preserving
    }
}

// Cluster-level Port (global external integration)
pub trait GlobalExternalPort {
    async fn publish_aggregate_analytics(&self, data: Analytics) -> Result<()>;
}

// Cluster-level Adapter (functor - structure preserving)
pub struct GlobalExternalAdapter {
    cluster_client: Client,
}

impl GlobalExternalPort for GlobalExternalAdapter {
    async fn publish_aggregate_analytics(&self, analytics: Analytics) -> Result<()> {
        // Functor: Internal Projection → External Service
        let external_format = map_to_external(analytics); // Structure-preserving
        self.cluster_client.publish("analytics.global", external_format).await
    }
}
```

Reference patterns from: `/git/thecowboyai/cim-domain-person/`
```

## Implementation Order

1. **Start with nats-expert** (most critical for infrastructure)
2. **Update network-expert** (understand topology)
3. **Update nix-expert** (deployment patterns)
4. **Update sage** (orchestration with real infrastructure)
5. **Update domain experts** (use cim-domain-person patterns)
6. **Update development experts** (testing, BDD, QA)
7. **Update UI experts** (NOC dashboard patterns)

## Success Criteria

Updated agents should:
- ✅ **ENFORCE FRP ONLY**: Event-Driven + Functional Reactive, NEVER OOP
- ✅ **REJECT CRUD**: All state changes through immutable events
- ✅ Reference actual deployed infrastructure
- ✅ Include real commands and code examples
- ✅ Link to actual files in repositories
- ✅ Follow Infrastructure FIRST → PKI → Domains workflow
- ✅ Use patterns from `cim-domain` and `cim-domain-person`
- ✅ Show Mealy state machines and pure projections
- ✅ Emphasize Category Theory compliance (functors, natural transformations)
- ✅ Balance theory (conceptual spaces, CT) with practice (deployed code)
- ✅ Be immediately useful for both infrastructure ops AND domain development

## Next Steps

1. Review this plan
2. Start with nats-expert.md update
3. Test with actual infrastructure questions
4. Iterate based on usefulness
5. Roll out to other agents
6. Document patterns that emerge

This transforms agents from theoretical guides to practical assistants grounded in deployed reality.
