# CIM Development Guidelines

## Core CIM Principles - MUST FOLLOW

### 1. EVENT-DRIVEN COMMUNICATION ONLY
**NEVER use direct method calls or synchronous operations**

❌ WRONG (Traditional):
```rust
let user = database.get_user(id);
user.update_name(new_name);
database.save(user);
```

✅ RIGHT (CIM):
```rust
// Emit command event
nats.publish("user.commands.update_name", UpdateNameCommand {
    user_id: id,
    new_name,
    correlation_id: Uuid::new_v4(),
    causation_id: previous_event_id,
});

// Listen for domain event
nats.subscribe("user.events.name_updated", |event: NameUpdatedEvent| {
    // React to the fact that name was updated
});
```

### 2. NO MUTABLE STATE - ONLY EVENT STREAMS

**State is ALWAYS reconstructed from events**

❌ WRONG:
```rust
struct User {
    mut name: String,  // NEVER mutate fields
}
```

✅ RIGHT:
```rust
// Events are immutable facts
struct UserCreatedEvent { name: String, timestamp: DateTime }
struct NameChangedEvent { old_name: String, new_name: String, timestamp: DateTime }

// State is projected from event stream
fn project_user_state(events: Vec<UserEvent>) -> UserProjection {
    events.iter().fold(UserProjection::default(), |state, event| {
        match event {
            UserCreatedEvent(e) => state.with_name(e.name),
            NameChangedEvent(e) => state.with_name(e.new_name),
        }
    })
}
```

### 3. NATS SUBJECT ALGEBRA FOR ALL COMMUNICATION

**Every interaction uses NATS subjects with proper hierarchy**

```
Organization.Unit.Service.Entity.Operation.SubOperation

Examples:
cowboyai.engineering.keys.certificate.generate.root
cowboyai.infrastructure.nats.operator.create
cowboyai.security.audit.key.revoked
```

### 4. CORRELATION AND CAUSATION TRACKING

**EVERY event/command MUST have:**
- `correlation_id`: Links related messages across the entire conversation
- `causation_id`: The event that directly caused this one
- `aggregate_id`: The entity this relates to
- `timestamp`: When this occurred

### 5. GRAPH-BASED RELATIONSHIPS

**Everything is nodes and edges in a directed graph**

❌ WRONG:
```rust
struct Organization {
    users: Vec<User>,  // Embedded collections
}
```

✅ RIGHT:
```rust
// Nodes
struct Organization { id: Uuid }
struct Person { id: Uuid }

// Edges (as events)
struct PersonJoinedOrganizationEvent {
    person_id: Uuid,
    organization_id: Uuid,
    role: String,
    timestamp: DateTime,
}
```

### 6. PROJECTIONS FOR DIFFERENT VIEWS

**Same events, multiple interpretations**

```rust
// Security projection: Who can access what?
SecurityProjection::from_events(events) -> HashMap<PersonId, Vec<Permission>>

// Hierarchy projection: Org structure
OrgChartProjection::from_events(events) -> Graph<Person, ReportsTo>

// Audit projection: What happened when?
AuditLogProjection::from_events(events) -> Vec<AuditEntry>
```

### 7. NO CRUD - ONLY DOMAIN EVENTS

❌ WRONG:
- CreateUser, ReadUser, UpdateUser, DeleteUser

✅ RIGHT:
- PersonJoinedOrganization
- KeyGeneratedForPerson
- TrustEstablishedBetween
- AuthorityDelegatedTo
- KeyRevokedDueToCompromise

### 8. OFFLINE-FIRST WITH EVENT REPLAY

**System must work disconnected and sync later**

```rust
// Local event store
local_events.append(event);

// When connected, sync with NATS
when_online {
    for event in local_events.since(last_sync) {
        nats.publish(event.subject(), event);
    }
}
```

### 9. TIME AS A FIRST-CLASS CONCEPT

**Everything has temporal validity**

```rust
struct KeyOwnership {
    person_id: Uuid,
    key_id: Uuid,
    valid_from: DateTime,
    valid_until: Option<DateTime>,
    // Ownership changes over time via events
}
```

### 10. SEMANTIC SUBJECTS NOT TECHNICAL ONES

❌ WRONG:
- database.table.insert
- api.endpoint.post
- service.method.call

✅ RIGHT:
- organization.person.onboarded
- trust.relationship.established
- authority.delegation.granted

## CIM Communication Patterns

### Pattern 1: Command → Event → Projection
```rust
// 1. Someone wants something to happen
PersonWantsToGenerateKeyCommand

// 2. Domain processes command, emits event
KeyGeneratedForPersonEvent

// 3. Multiple projections update
SecurityProjection.apply(event)  // Updates access control
AuditProjection.apply(event)     // Logs the action
GraphProjection.apply(event)     // Updates relationship graph
```

### Pattern 2: Event Choreography (No Orchestrator)
```rust
// Each service listens and reacts independently

// Key service listens for person events
on PersonJoinedOrganizationEvent:
    emit GenerateInitialKeysCommand

// Certificate service listens for key events
on KeyGeneratedEvent where key.type == "CA":
    emit GenerateCertificateCommand

// Trust service listens for certificate events
on CertificateGeneratedEvent:
    emit EstablishTrustCommand
```

### Pattern 3: Temporal Queries
```rust
// "What was the state at time T?"
let events_until_t = events.filter(|e| e.timestamp <= t);
let state_at_t = projection.replay(events_until_t);

// "Who had access during this incident?"
let incident_window = (incident_start, incident_end);
let access_events = events.during(incident_window);
let people_with_access = SecurityProjection.replay(access_events);
```

## CIM-Specific GUI Principles

### 1. GUI as Event Emitter/Observer
```rust
// GUI doesn't call functions, it publishes intentions
gui_button_clicked -> PublishCommand(UserIntendsToGenerateKey)

// GUI subscribes to state changes
subscribe("domain.state.changed") -> update_display()
```

### 2. Graph Visualization is PRIMARY
- Organization is a graph, not a tree
- Show relationships as edges
- Nodes have temporal properties
- Edge types show different relationships

### 3. Event History is Visible
- Show timeline of what happened
- Allow temporal navigation
- Display causation chains

## Implementation Checklist

When building ANY CIM component:

- [ ] Uses NATS subjects for ALL communication
- [ ] Emits events, never mutates state
- [ ] Includes correlation/causation IDs
- [ ] Models relationships as graph edges
- [ ] Provides multiple projections
- [ ] Works offline with event replay
- [ ] Uses domain language, not technical
- [ ] Shows temporal aspects in UI
- [ ] Visualizes as graph when applicable
- [ ] Never uses CRUD operations

## Example: Key Generation in CIM Style

```rust
// 1. User expresses intent (GUI)
let intent = PersonIntendsToGenerateKeyCommand {
    person_id,
    key_purpose: KeyPurpose::Signing,
    correlation_id: Uuid::new_v4(),
    causation_id: gui_click_event_id,
};

// 2. Publish to NATS
nats.publish("cowboyai.security.keys.commands.generate", intent);

// 3. Key aggregate processes command
on_command(intent) -> Vec<Event> {
    // Validate against current projection
    let can_generate = projection.person_has_permission(person_id, "GenerateKeys");

    if can_generate {
        vec![
            KeyGenerationRequestedEvent { ... },
            YubiKeySlotAllocatedEvent { ... },
            KeyMaterialGeneratedEvent { ... },
            KeyOwnershipEstablishedEvent { ... },
        ]
    }
}

// 4. Multiple services react to events
on KeyOwnershipEstablishedEvent:
    - SecurityProjection updates access control graph
    - AuditProjection logs the action with full context
    - NotificationService informs relevant parties
    - GraphProjection adds new edges
    - BackupService schedules key escrow

// 5. GUI updates from projections
gui.subscribe("projection.graph.updated", |graph| {
    render_organization_graph(graph);
});
```

## Remember: CIM is a Living System

- Events flow like blood through vessels
- State emerges from event interactions
- Time is always present
- Everything is connected in a graph
- Offline is normal, online is synchronization
- The system tells its own story through events