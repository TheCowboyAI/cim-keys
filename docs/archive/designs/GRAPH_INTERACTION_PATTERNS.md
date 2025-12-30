# Graph Interaction Patterns

## Overview

The cim-keys GUI provides multiple graph-based visualizations for exploring CQRS event-sourced systems. Each view reveals different aspects of the system's behavior and structure.

## Graph Views

### 1. Organization View (Default)
**Purpose**: Visualize organizational structure, people, locations, and relationships

**Node Types**:
- **Organizations** (Purple) - Root entities
- **Organizational Units** (Light Purple) - Departments, teams
- **People** (Blue) - Individuals with roles
- **Locations** (Green) - Physical or logical storage
- **Roles** (Orange) - Permission sets
- **Policies** (Red) - Access control rules

**Edge Types**:
- **Reports To** - Hierarchical relationships
- **Works At** - Location assignments
- **Has Role** - Role assignments
- **Governs** - Policy applications

**Interactions**:
- **Right-click empty space** → Create new entity
- **Right-click node** → Edit properties, delete
- **Click-drag between nodes** → Create relationship
- **Click node** → View details in property card
- **Double-click** → Start inline editing

---

### 2. NATS Infrastructure View
**Purpose**: Visualize NATS messaging hierarchy (operators, accounts, users)

**Node Types**:
- **NATS Operator** (Orange) - Root authority
- **NATS Account** (Light Orange) - Isolated messaging space
- **NATS User** (Gold) - Individual credentials
- **Service Accounts** (Brown) - Automated access

**Edge Types**:
- **Contains** - Hierarchical ownership
- **Delegates** - Permission delegation

**Interactions**:
- **Right-click** → Create operator/account/user
- **View property card** → See JWT claims, permissions
- **Export** → Download NATS configuration files

---

### 3. PKI Trust Chain View
**Purpose**: Visualize certificate authority hierarchy and trust relationships

**Node Types**:
- **Root CA** (Dark Red) - Root certificate authority
- **Intermediate CA** (Orange-Red) - Intermediate certificates
- **Leaf Certificates** (Yellow) - End-entity certificates

**Edge Types**:
- **Signs** - Certificate signing relationships
- **Trusts** - Trust chain links

**Interactions**:
- **Click certificate** → View X.509 details
- **Trace trust path** → Follow edges to root
- **Verify chain** → Check validity dates

---

### 4. YubiKey Details View
**Purpose**: Visualize hardware security key provisioning and slot allocation

**Node Types**:
- **YubiKey** (Brown) - Physical hardware token
- **PIV Slots** (Light Brown) - Card slots (9A, 9C, 9D, 9E)
- **Person** (Blue) - Key owner
- **Status Indicators** (various) - Provisioning state

**Edge Types**:
- **Assigned To** - Ownership
- **Contains** - Slot membership
- **Stores** - Key storage

**Interactions**:
- **View slot details** → See key type, usage
- **Check provisioning** → Green = provisioned, gray = empty
- **Plan allocation** → Drag-drop keys to slots

---

### 5. Timeline View (NEW)
**Purpose**: Temporal visualization of all events in chronological order

**Node Types**:
- **Timeline Events** - Color-coded by aggregate type
  - Blue: Person events
  - Cyan: Key events
  - Sky Blue: Certificate events
  - Brown: YubiKey events
  - Orange: NATS events

**Edge Types**:
- **Causation** (Green) - Event A caused Event B
- **Temporal** (Light Gray) - Time sequence

**Interactions**:
- **Scroll vertically** → Navigate through time
- **Click event** → See full event details (correlation_id, causation_id, payload)
- **Follow causation** → Trace cause-effect chains
- **Filter by time** → Show events in date range
- **Search** → Find events by type or content

**Use Cases**:
- Audit trail analysis
- Debugging event sequences
- Compliance reporting
- System behavior analysis

---

### 6. Aggregates View (NEW)
**Purpose**: State machine diagrams for domain aggregates

**Aggregate Types**:
- Person, Organization, Location, Role, Policy
- Key, Certificate, YubiKey
- NatsOperator, NatsAccount, NatsUser

**Node Types**:
- **States** (colored by aggregate) - Lifecycle states
- **Transitions** (arrows) - Commands that trigger state changes

**Interactions**:
- **Select aggregate** → Choose from dropdown
- **View state machine** → See all possible states
- **Click state** → See description, is_terminal flag
- **Follow transition** → See command that triggers change

**Use Cases**:
- Understanding aggregate lifecycle
- Planning state transitions
- Documenting business logic
- Validating state machine completeness

---

### 7. Command History View (NEW)
**Purpose**: CQRS write audit trail - visualize all commands executed

**Node Types**:
- **Commands** (color-coded by type)
  - Blue: CreatePerson
  - Cyan: GenerateKey
  - Sky Blue: GenerateCertificate
  - Brown: ProvisionYubiKey
  - Red: Failed commands

**Display Format**:
- ✓ = Success
- ✗ = Failure
- Timestamp (HH:MM:SS)
- Command type

**Edge Types**:
- **Causation** (Gray, semi-transparent) - Command chains

**Interactions**:
- **Click command** → View details (description, executor, error messages)
- **Trace chain** → Follow causation from root to leaf
- **Filter** → Show only failures, or by type
- **Statistics** → View success/failure counts

**Use Cases**:
- Write operation audit
- Failure analysis
- Command pattern verification
- Performance bottleneck identification

---

### 8. Causality Chains View (NEW)
**Purpose**: Distributed tracing - visualize complete workflows by correlation_id

**Node Types**:
- **Events** (color-coded by aggregate type)
- Positioned by **causal depth** (level in workflow)

**Display Format**:
- Event type
- Timestamp (HH:MM:SS.mmm)
- Latency (+XXXms from previous)

**Edge Types**:
- **Causation** (Green) - Event A directly caused Event B

**Workflow Grouping**:
- Each workflow (correlation_id) displayed vertically
- Events laid out left-to-right by causal depth
- Multiple workflows stacked with spacing

**Interactions**:
- **Click event** → View full details (correlation_id, causation_id, description)
- **Follow chain** → Trace complete workflow
- **Analyze timing** → Identify slow steps (high +Xms values)
- **Statistics** → View workflow count, average depth, total duration

**Use Cases**:
- Distributed tracing
- Workflow analysis
- Performance profiling
- Causation debugging

---

## Common Interaction Patterns

### Context Menu (Right-Click)
All views support context menus for:
- **Empty space** → Create new entity appropriate to view
- **Node** → Edit, delete, view details
- **Edge** → Change type, delete

### Property Card (Right Panel)
Displays detailed information when selecting a node:
- **Entity metadata** (ID, timestamps, correlation_id)
- **Editable fields** (name, email, description)
- **Relationships** (incoming/outgoing edges)
- **Actions** (save, cancel, delete)

### Drag-and-Drop
- **Click-drag from node to node** → Create relationship
- **Drop on empty space** → Cancel
- **Drop on target node** → Confirm relationship creation

### Search (Ctrl+F)
Global search across all node types:
- Matches on name, email, description, type
- Highlights matching nodes
- Clears previous highlights on new search

### View Switching
Toolbar buttons at top:
- 🏢 Organization
- 📡 NATS Infrastructure
- 🔐 PKI Trust Chain
- 🔑 YubiKey Details
- ⏰ Timeline (Event Sourcing)
- 🔄 Aggregates (State Machines)
- 📝 Command History (CQRS Writes)
- 🔗 Causality Chains (Distributed Tracing)

---

## CQRS Event Sourcing Workflow

### Write Side (Commands)
1. User performs action in Organization view (create person, assign key)
2. GUI emits **Command** with correlation_id and causation_id
3. Aggregate validates and processes command
4. Aggregate emits **Events**
5. Events logged to Command History view

### Read Side (Projections)
1. Events applied to projections
2. Projections update graph visualizations
3. Timeline view shows new events
4. Causality Chains view shows workflow progression
5. Organization view reflects new state

### Observability
- **Timeline View**: See all events in temporal order
- **Command History**: See all write operations and their results
- **Causality Chains**: See complete workflows grouped by correlation_id
- **Aggregates View**: See current state in lifecycle

---

## Best Practices

### For Event Sourcing Analysis
1. Start with **Timeline View** to see overall activity
2. Switch to **Causality Chains** to trace specific workflows
3. Use **Command History** to analyze write patterns
4. Check **Aggregates View** to understand state machines

### For System Modeling
1. Start with **Organization View** to define structure
2. Use **NATS Infrastructure** to model messaging
3. Use **PKI Trust Chain** to plan certificate hierarchy
4. Use **YubiKey Details** to allocate hardware

### For Debugging
1. **Timeline**: Find the event that indicates the problem
2. **Command History**: Check if command failed (red ✗)
3. **Causality Chains**: Trace the workflow to find where it broke
4. **Property Card**: Check event details (correlation_id, error messages)

### For Compliance
1. **Timeline**: Complete audit trail with timestamps
2. **Command History**: Who executed what commands
3. **Causality Chains**: Trace accountability through workflows
4. **Export**: Generate reports from projection data

---

## Keyboard Shortcuts

- **Ctrl+F** - Search
- **Delete** - Delete selected node
- **Escape** - Clear selection / cancel editing
- **Enter** - Save inline edit
- **Tab** - Cycle through views
- **Ctrl+Z** - Undo (command-based)
- **Ctrl+Y** - Redo

---

## Performance Considerations

### Large Graphs (1000+ nodes)
- Use **Search** to find specific nodes
- Use **Filters** to show subset of nodes
- Switch to specific views (don't render everything at once)

### Event History (10,000+ events)
- **Timeline View** uses pagination (renders window)
- **Causality Chains** groups by workflow (not all events at once)
- Use date range filters to limit scope

### Real-time Updates
- Views auto-refresh when events arrive
- Incremental updates (only new nodes/edges)
- Background projection rebuilding

---

## Troubleshooting

### "No nodes visible"
- Check view type - some views show specific entity types only
- Check if domain has been loaded
- Try switching views and back

### "Edges not connecting"
- Ensure both nodes exist in current view
- Check edge type compatibility
- Try refreshing the graph

### "Property card not updating"
- Click node again to refresh
- Check if projection sync is enabled
- Verify events are being applied

### "Command failed" (red ✗ in Command History)
- Click command to see error message
- Check validation requirements
- Verify prerequisites exist (e.g., organization before person)

---

## Architecture Notes

### Graph Rendering
- Built with **Iced 0.13+** framework
- Supports WASM (browser) and native (desktop)
- Declarative reactive updates (TEA pattern)

### Data Flow
```
User Intent
  → Command (with correlation_id, causation_id)
  → Aggregate (validation + business logic)
  → Events (immutable facts)
  → Projections (materialized views)
  → Graph Updates (UI refresh)
```

### Event Metadata
Every event carries:
- `event_id`: Unique identifier
- `correlation_id`: Groups related events (workflow)
- `causation_id`: References causing event (tracing)
- `timestamp`: When it occurred
- `aggregate_id`: Which entity changed
- `aggregate_type`: Type of entity

This metadata enables:
- Distributed tracing (correlation_id)
- Causality analysis (causation_id)
- Audit trails (timestamp + event details)
- Projection rebuilding (replay events)

---

## Future Enhancements

- **Real-time collaboration**: Multiple users editing same graph
- **3D visualization**: Depth for temporal dimension
- **Graph diff**: Compare states at different timestamps
- **Query builder**: Complex event searches
- **Export formats**: PDF, SVG, Mermaid diagrams
- **Animation**: Replay workflows in real-time

---

## References

- [CQRS_GRAPH_DESIGN.md](./CQRS_GRAPH_DESIGN.md) - Technical design document
- [N_ARY_FRP_AXIOMS.md](../N_ARY_FRP_AXIOMS.md) - FRP architecture principles
- [USER_STORY_COVERAGE.md](../USER_STORY_COVERAGE.md) - Feature coverage tracking
- Integration tests: `tests/graph_view_integration.rs`
