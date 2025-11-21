# CQRS Graph-Based Write Model Design

## Current State Analysis

### ✅ What We Have (Read Model - Queries)

**Graph Infrastructure:**
- `OrganizationGraph` with multiple node types (Person, Org, Location, NATS, PKI, YubiKey)
- Event-sourced graph with undo/redo (EventStack)
- Drag-and-drop node positioning
- Edge creation with visual indicator
- Node/edge selection and filtering

**Existing Graph Views:**
1. **Organization** - Org structure, people, relationships
2. **NatsInfrastructure** - Operator/Account/User hierarchy
3. **PkiTrustChain** - Certificate trust chain (Root → Intermediate → Leaf)
4. **YubiKeyDetails** - PIV slot provisioning status

**Graph Events (Immutable):**
- NodeCreated, NodeDeleted, NodePropertiesChanged, NodeMoved
- EdgeCreated, EdgeDeleted, EdgeTypeChanged
- Compensating events for undo

### ❌ What's Missing (Write Model - Commands)

**Missing Graph Views:**
1. **Timeline** - Temporal event visualization (event sourcing audit trail)
2. **Aggregates** - State machine diagrams for each aggregate type
3. **CommandHistory** - CQRS command audit trail
4. **Causality** - Correlation/causation chains

**Missing Command Creation UI:**
1. **Intuitive node creation** - Right-click canvas, dropdown selection
2. **Relationship drag-to-create** - Drag from node to create edges
3. **Inline property editing** - Click node to edit properties
4. **Visual state transitions** - Click to trigger aggregate transitions
5. **Command palette** - Keyboard-driven command creation

**Missing Write Operations:**
- Location creation through graph (not just forms)
- Key relationship modeling (delegation, trust)
- Policy attachment to entities
- Role assignment through drag-and-drop
- Aggregate command triggering from graph

## Design: Complete CQRS Implementation

### Architecture Pattern

```
┌─────────────────────────────────────────────┐
│           User Intent (Graph UI)            │
│  Click, Drag, Type, Right-click            │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│              Commands (Write)               │
│  CreateLocation, AddDelegation,             │
│  AssignRole, ProvisionYubiKey               │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│           Aggregates (Business Logic)       │
│  Person, Location, Key, Certificate         │
│  Validate → Generate Events                 │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│         Events (Immutable Facts)            │
│  LocationCreated, KeyDelegated,             │
│  RoleAssigned, YubiKeyProvisioned           │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│        Projections (Read Model)             │
│  Organization Graph, NATS Graph,            │
│  PKI Graph, Timeline, StateMachines         │
└─────────────────────────────────────────────┘
```

### New Graph Views to Implement

#### 1. Timeline Graph
**Purpose**: Visualize event sourcing audit trail over time
**Nodes**: Events (one per domain event)
**Edges**: Causation chains (causation_id → event_id)
**Layout**: Vertical timeline (newest at top)
**Interactions**:
- Click event to see full details
- Hover to see correlation chain
- Filter by aggregate type
- Search by entity ID

```rust
pub enum GraphView {
    Organization,
    NatsInfrastructure,
    PkiTrustChain,
    YubiKeyDetails,
    Timeline,           // NEW
    Aggregates,         // NEW
    CommandHistory,     // NEW
    CausalityChains,    // NEW
}
```

#### 2. Aggregate State Machine Graph
**Purpose**: Visualize state machines for each aggregate type
**Nodes**: States (PersonCreated, KeyGenerated, etc.)
**Edges**: Transitions (commands that move between states)
**Layout**: Hierarchical (initial state at top)
**Interactions**:
- Click transition to see command details
- Hover state to see current count
- Select entity to highlight its path

#### 3. Command History Graph
**Purpose**: CQRS command audit trail
**Nodes**: Commands executed
**Edges**: Command dependencies (which commands triggered others)
**Layout**: Timeline (chronological)
**Interactions**:
- Click command to see parameters
- Click to re-execute (if idempotent)
- Filter by command type
- Search by user/entity

#### 4. Causality Chain Graph
**Purpose**: Distributed tracing (correlation_id chains)
**Nodes**: Events in a single workflow
**Edges**: Causation links
**Layout**: Horizontal flow (cause → effect)
**Interactions**:
- Enter correlation_id to visualize chain
- Export to distributed tracing format
- Measure time between events

### Enhanced Command Creation UI

#### Context Menu (Right-click Canvas)
```
Right-click empty space:
  ├─ Create Person
  ├─ Create Location
  ├─ Create Policy
  ├─ Create Role
  └─ Import from...

Right-click node:
  ├─ Edit Properties
  ├─ Assign Role
  ├─ Add Relationship
  ├─ Generate Keys
  ├─ Provision YubiKey
  ├─ Delete
  └─ View Details
```

#### Drag-to-Create Relationships
```
Drag from Person → Person:  KeyDelegation
Drag from Person → Location: KeyStorage
Drag from Policy → Person:  PolicyApplies
Drag from Role → Person:    RoleAssignment
```

#### Inline Editing
- Double-click node label → edit name
- Click node → property panel appears
- Tab through fields for quick editing
- Enter to commit, Esc to cancel

#### Visual State Transitions
- Aggregate nodes show current state
- Available transitions shown as buttons
- Click transition → command dialog
- Confirm → command executed → event emitted → projection updates

### Implementation Plan

**Phase 1: Timeline Graph** (2-3 hours)
- [x] Add GraphView::Timeline enum variant
- [ ] Implement timeline layout algorithm (vertical)
- [ ] Render events as nodes with timestamp labels
- [ ] Draw causation edges
- [ ] Add event detail popup
- [ ] Wire up to event store

**Phase 2: Enhanced Command UI** (3-4 hours)
- [ ] Extend context menu for all entity types
- [ ] Implement drag-to-create edge logic
- [ ] Add relationship type selector dialog
- [ ] Emit proper commands (not just graph events)
- [ ] Connect to aggregate handlers

**Phase 3: Aggregate State Machines** (2-3 hours)
- [ ] Add GraphView::Aggregates
- [ ] Define state machine schemas for each aggregate
- [ ] Render states and transitions
- [ ] Highlight current states from projections
- [ ] Add transition trigger buttons

**Phase 4: Command History** (2 hours)
- [ ] Add GraphView::CommandHistory
- [ ] Capture all commands in audit log
- [ ] Render as timeline graph
- [ ] Add command replay functionality

**Phase 5: Causality Chains** (2 hours)
- [ ] Add GraphView::CausalityChains
- [ ] Query events by correlation_id
- [ ] Render cause-effect chains
- [ ] Add timing analysis

**Phase 6: Integration & Testing** (2-3 hours)
- [ ] Add tests for command creation flows
- [ ] Add tests for each graph view
- [ ] Document graph interaction patterns
- [ ] Update USER_STORY_COVERAGE.md

## Total Estimated Effort: 13-17 hours

## Success Criteria

1. **Complete CQRS Pattern**:
   - ✅ Commands trigger aggregate logic
   - ✅ Aggregates emit events
   - ✅ Projections rebuild from events
   - ✅ Multiple read models (graphs)

2. **Intuitive Creation**:
   - ✅ Right-click to create any entity
   - ✅ Drag to create relationships
   - ✅ Inline editing for all properties
   - ✅ Visual feedback for all actions

3. **Complete Observability**:
   - ✅ Timeline shows all events
   - ✅ State machines show current states
   - ✅ Command history shows write operations
   - ✅ Causality chains show workflows

4. **Test Coverage**:
   - ✅ Integration tests for each graph view
   - ✅ Tests for command creation flows
   - ✅ Tests for projection rebuilding
   - ✅ Tests for undo/redo with commands
