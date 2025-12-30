# GUI Enhancements Architecture
## State Machine Visualization, Event History Viewer, and Lifecycle Tracking

**Date**: 2025-01-22
**Status**: Architecture Design
**Goal**: Production-ready GUI with complete visibility into system state

---

## Overview

This document defines the architecture for three critical GUI features:
1. **State Machine Visualization** - Visual representation of entity lifecycles
2. **Event History Viewer** - Temporal event browser with filtering
3. **Complete Lifecycle Tracking** - State transitions for ALL entities

---

## Feature 1: State Machine Visualization

### Objectives
- Visualize state graphs for each entity type
- Show current state, valid transitions, terminal states
- Interactive: click states to see transition requirements
- Real-time: update as events occur

### Entity State Machines to Visualize

#### 1. **Key Lifecycle**
```
Generated → InUse ⟷ Rotated → Revoked (terminal)
```

#### 2. **Certificate Lifecycle**
```
Requested → Issued → Active ⟷ Suspended → Revoked/Expired (terminal)
```

#### 3. **NATS Operator Lifecycle**
```
Created → KeysGenerated → Active ⟷ Suspended → Revoked (terminal)
```

#### 4. **NATS Account Lifecycle**
```
Created → Active ⟷ Suspended ⟷ Reactivated → Deleted (terminal)
```

#### 5. **NATS User Lifecycle**
```
Created → Active ⟷ Suspended ⟷ Reactivated → Deleted (terminal)
```

#### 6. **Person Lifecycle** (NEW)
```
Created → Active ⟷ Suspended → Archived (terminal)
```

#### 7. **Location Lifecycle** (NEW)
```
Created → Active → Decommissioned (terminal)
```

#### 8. **Policy Lifecycle** (NEW)
```
Draft → Active ⟷ Suspended → Revoked (terminal)
```

#### 9. **Relationship Lifecycle** (NEW)
```
Established → Active → Dissolved (terminal)
```

#### 10. **YubiKey Lifecycle** (NEW)
```
Detected → Provisioned → Active ⟷ Locked → Retired (terminal)
```

#### 11. **Manifest Lifecycle**
```
Created → Updated → Sealed (terminal)
```

### Visualization Components

#### State Graph Widget (Iced Canvas)
```rust
pub struct StateGraphWidget<S: StateMachine> {
    state_machine: S,
    current_state: S::State,
    layout: GraphLayout,
    highlighted_transitions: Vec<Transition>,
}

impl<S: StateMachine> Widget for StateGraphWidget<S> {
    fn draw(&self, renderer: &mut Renderer) {
        // Draw nodes (states) as circles
        // Draw edges (transitions) as arrows
        // Highlight current state
        // Show valid transitions from current state
    }
}
```

#### Graph Layout Algorithm
- **Nodes**: States as colored circles
  - Green: Current state
  - Blue: Valid next states
  - Gray: Inactive states
  - Red: Terminal states
- **Edges**: Transitions as arrows
  - Solid: Available from current state
  - Dotted: Not available
- **Labels**: State names and transition conditions

### UI Integration
```
┌─────────────────────────────────────────┐
│ Entity: NATS Operator (op-123)         │
│ Current State: Active                   │
├─────────────────────────────────────────┤
│                                         │
│     ┌─────────┐                        │
│     │ Created │                        │
│     └────┬────┘                        │
│          ↓                             │
│    ┌──────────────┐                   │
│    │KeysGenerated │                   │
│    └──────┬───────┘                   │
│           ↓                            │
│      ┌────────┐  ←→  ┌──────────┐    │
│      │ Active │ ←───→ │Suspended │    │
│      └───┬────┘       └──────────┘    │
│          ↓                             │
│     ┌─────────┐                       │
│     │ Revoked │  (terminal)           │
│     └─────────┘                       │
└─────────────────────────────────────────┘
```

---

## Feature 2: Event History Viewer

### Objectives
- Chronological event timeline
- Filter by entity type, date range, event type
- Show correlation chains (related events)
- Show causation chains (what caused what)
- Export event history

### Data Model

#### Event Entry
```rust
pub struct EventHistoryEntry {
    pub event_id: Uuid,
    pub event_type: String,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub data: serde_json::Value,
}
```

#### Event Query
```rust
pub struct EventQuery {
    pub entity_type: Option<EntityType>,
    pub entity_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub correlation_id: Option<Uuid>,
    pub limit: usize,
    pub offset: usize,
}
```

### Visualization Components

#### Timeline Widget
```
┌─────────────────────────────────────────────────────┐
│ Event History                        [Filter] [⚙️]  │
├─────────────────────────────────────────────────────┤
│ 2025-01-22 10:00:00  NatsOperatorCreated           │
│ ├─ Operator: cowboyai-root                        │
│ ├─ Correlation: abc-123                           │
│ └─ Created by: alice@cowboyai.com                 │
│                                                     │
│ 2025-01-22 10:05:00  NatsSigningKeyGenerated       │
│ ├─ Operator: cowboyai-root                        │
│ ├─ Correlation: abc-123  (same chain)             │
│ └─ Causation: NatsOperatorCreated                 │
│                                                     │
│ 2025-01-22 10:10:00  NatsOperatorSuspended         │
│ ├─ Operator: cowboyai-root                        │
│ ├─ Reason: Security audit                         │
│ └─ Suspended by: bob@cowboyai.com                 │
└─────────────────────────────────────────────────────┘
```

#### Filter Panel
```rust
pub struct EventFilter {
    pub entity_types: Vec<EntityType>,
    pub event_types: Vec<String>,
    pub date_range: (Option<DateTime<Utc>>, Option<DateTime<Utc>>),
    pub search_text: String,
}
```

### UI Components
- **Timeline View**: Scrollable chronological list
- **Correlation View**: Group by correlation_id
- **Causation Graph**: Show cause-effect relationships
- **Detail Panel**: Expand event to see full JSON
- **Export Button**: Save filtered events to JSON/CSV

---

## Feature 3: Complete Lifecycle Tracking

### Current State Analysis

#### ✅ **Entities with State Transition Events**
1. **Key**: KeyRevoked
2. **NATS Operator**: Suspended, Reactivated, Revoked
3. **NATS Account**: Activated, Suspended, Reactivated, Deleted
4. **NATS User**: Activated, Suspended, Reactivated, Deleted

#### ❌ **Entities Missing State Transition Events**

##### 1. Certificate Lifecycle
**Missing Events**:
- `CertificateActivated` - Certificate becomes active
- `CertificateSuspended` - Temporary suspension
- `CertificateRevoked` - Permanent revocation
- `CertificateExpired` - Natural expiry
- `CertificateRenewed` - Renewal before expiry

**State Machine**:
```
Requested → Issued → Active ⟷ Suspended → Revoked/Expired (terminal)
                              ↓
                           Renewed → Active
```

##### 2. Person Lifecycle
**Missing Events**:
- `PersonActivated` - Person becomes active in system
- `PersonSuspended` - Temporary suspension (e.g., on leave)
- `PersonReactivated` - Return from suspension
- `PersonArchived` - Permanently archived (e.g., left organization)

**State Machine**:
```
Created → Active ⟷ Suspended → Archived (terminal)
```

##### 3. Location Lifecycle
**Missing Events**:
- `LocationActivated` - Location becomes operational
- `LocationSuspended` - Temporary unavailability
- `LocationReactivated` - Location restored
- `LocationDecommissioned` - Permanent shutdown

**State Machine**:
```
Created → Active ⟷ Suspended → Decommissioned (terminal)
```

##### 4. Organization/Unit Lifecycle
**Missing Events**:
- `OrganizationActivated`
- `OrganizationSuspended`
- `OrganizationDissolved`

**State Machine**:
```
Created → Active ⟷ Suspended → Dissolved (terminal)
```

##### 5. Policy Lifecycle
**Missing Events**:
- `PolicyActivated` - Policy becomes enforceable
- `PolicyAmended` - Policy updated
- `PolicySuspended` - Temporary suspension
- `PolicyRevoked` - Permanent revocation

**State Machine**:
```
Draft → Active ⟷ Suspended → Revoked (terminal)
         ↓
      Amended → Active
```

##### 6. Relationship Lifecycle
**Missing Events**:
- `RelationshipSuspended` - Temporary pause
- `RelationshipReactivated` - Resume relationship
- `RelationshipDissolved` - Permanent end

**State Machine**:
```
Established → Active ⟷ Suspended → Dissolved (terminal)
```

##### 7. YubiKey Lifecycle
**Missing Events**:
- `YubiKeyActivated` - YubiKey ready for use
- `YubiKeyLocked` - PIN locked
- `YubiKeyUnlocked` - PIN unlocked
- `YubiKeyLost` - Report as lost
- `YubiKeyRetired` - Permanent retirement

**State Machine**:
```
Detected → Provisioned → Active ⟷ Locked → Retired (terminal)
                          ↓
                        Lost → Retired
```

##### 8. Manifest Lifecycle
**Missing Events**:
- `ManifestValidated` - Integrity verified
- `ManifestSealed` - Finalized, no more changes
- `ManifestInvalidated` - Marked invalid

**State Machine**:
```
Created → Updated → Validated → Sealed (terminal)
                       ↓
                  Invalidated (terminal)
```

---

## Implementation Plan

### Phase 11: Certificate Lifecycle (5 events)
- CertificateActivated
- CertificateSuspended
- CertificateRevoked
- CertificateExpired
- CertificateRenewed

### Phase 12: Person/Location Lifecycle (8 events)
- PersonActivated, PersonSuspended, PersonReactivated, PersonArchived
- LocationActivated, LocationSuspended, LocationReactivated, LocationDecommissioned

### Phase 13: Organization/Policy Lifecycle (7 events)
- OrganizationActivated, OrganizationSuspended, OrganizationDissolved
- PolicyActivated, PolicyAmended, PolicySuspended, PolicyRevoked

### Phase 14: Relationship/YubiKey Lifecycle (8 events)
- RelationshipSuspended, RelationshipReactivated, RelationshipDissolved
- YubiKeyActivated, YubiKeyLocked, YubiKeyUnlocked, YubiKeyLost, YubiKeyRetired

### Phase 15: Manifest Lifecycle (3 events)
- ManifestValidated, ManifestSealed, ManifestInvalidated

### Phase 16: State Machine Visualization UI
- Canvas-based graph rendering
- Interactive state exploration
- Real-time updates

### Phase 17: Event History Viewer UI
- Timeline component
- Filter/search functionality
- Correlation/causation views
- Export capabilities

---

## Data Structures

### StateMachine Trait
```rust
pub trait StateMachine {
    type State: PartialEq + Clone;
    type Event;

    fn current_state(&self) -> &Self::State;
    fn valid_transitions(&self) -> Vec<(Self::State, String)>; // (next_state, condition)
    fn is_terminal(&self, state: &Self::State) -> bool;
    fn transition(&mut self, event: Self::Event) -> Result<Self::State, StateError>;
}
```

### GraphLayout
```rust
pub struct GraphLayout {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub struct Node {
    pub id: String,
    pub position: Point,
    pub radius: f32,
    pub color: Color,
    pub label: String,
}

pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub style: EdgeStyle, // Solid, Dotted, Dashed
}
```

---

## Success Criteria

### State Machine Visualization
- ✅ All 14 entity state machines visualized
- ✅ Current state highlighted
- ✅ Valid transitions shown
- ✅ Terminal states marked
- ✅ Interactive exploration

### Event History Viewer
- ✅ All events displayed chronologically
- ✅ Filter by entity type, date, event type
- ✅ Correlation chain navigation
- ✅ Causation chain visualization
- ✅ Export to JSON/CSV

### Lifecycle Tracking
- ✅ All 14 entities have complete state machines
- ✅ All state transitions have events
- ✅ All events have projection handlers
- ✅ State transitions validated at type level
- ✅ Audit trail for all transitions

---

## Technical Notes

### Iced Canvas for Visualization
- Use `iced::widget::canvas` for graph rendering
- Implement `canvas::Program` trait for interactivity
- Cache layout calculations
- Use geometric algorithms for node positioning (force-directed layout)

### Event Storage
- Events already stored in JSON files by date
- Event history loads from filesystem projections
- Index by entity_id, event_type, correlation_id for fast querying

### Performance Considerations
- Lazy loading for event history (pagination)
- Virtual scrolling for long timelines
- Cached state machine graphs
- Incremental updates on new events

---

## Estimated Work

**Phase 11-15** (Lifecycle Events): ~31 new events + 31 projection handlers = 62 implementations
**Phase 16** (Visualization): ~1000 lines of Iced canvas code
**Phase 17** (History Viewer): ~800 lines of UI components

**Total**: ~1800 lines of production code + documentation

**Timeline**:
- Phases 11-15: 2-3 hours (event/handler implementation)
- Phase 16: 1-2 hours (visualization UI)
- Phase 17: 1-2 hours (history viewer UI)
- **Total**: 4-7 hours of implementation

---

## Next Steps

1. Implement Phase 11 (Certificate lifecycle)
2. Implement Phase 12 (Person/Location lifecycle)
3. Continue through Phase 15
4. Build visualization widget
5. Build event history viewer
6. Integration testing
7. Documentation
