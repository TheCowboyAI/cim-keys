# Migration Guide: Adopting cim-keys Patterns

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

This guide helps other `cim-*` modules adopt the architectural patterns established in cim-keys.

---

## Overview

cim-keys establishes several core patterns for CIM development:

| Pattern | Purpose | Key Files |
|---------|---------|-----------|
| MVI Architecture | Pure functional GUI | `src/gui.rs`, `src/mvi/` |
| LiftableDomain | Domain→Graph lifting | `src/lifting.rs` |
| Event Sourcing | Immutable state changes | `src/aggregate.rs`, `src/projections.rs` |
| BDD Testing | Executable specifications | `tests/bdd/` |
| FRP Axioms | Compositional guarantees | N_ARY_FRP_AXIOMS.md |

---

## Step 1: Adopt MVI Architecture

### 1.1 Create the Model

```rust
// src/mvi/model.rs
#[derive(Debug, Clone)]
pub struct Model {
    // Immutable state - use with_* methods
    pub tab: Tab,
    pub data: Vec<YourEntity>,
    pub status: Status,
}

impl Model {
    pub fn new() -> Self { /* initial state */ }

    // NEVER mutate - always return new instance
    pub fn with_tab(self, tab: Tab) -> Self {
        Self { tab, ..self }
    }

    pub fn with_data(self, data: Vec<YourEntity>) -> Self {
        Self { data, ..self }
    }
}
```

### 1.2 Define Intents

```rust
// src/mvi/intent.rs

/// Intent naming convention:
/// - Ui* - User interactions (button clicks, selections)
/// - Port* - External I/O (file loaded, network response)
/// - Domain* - Aggregate events (EntityCreated, etc.)
/// - System* - Internal (tick, timer, subscription)
/// - Error* - Failures (validation, network, IO)
#[derive(Debug, Clone)]
pub enum Intent {
    // UI Layer
    UiTabSelected(Tab),
    UiCreateClicked,
    UiItemSelected(Uuid),

    // Port Layer (I/O results)
    PortFileLoaded(Vec<u8>),
    PortNetworkResponse(Response),

    // Domain Layer (aggregate events)
    DomainEntityCreated(Entity),
    DomainEntityUpdated(Entity),

    // System Layer
    SystemTick,

    // Error Layer
    ErrorValidation(String),
    ErrorNetwork(String),
}
```

### 1.3 Implement Pure Update Function

```rust
// src/mvi/update.rs
use iced::Task;

/// CRITICAL: This function must be PURE
/// - Output depends ONLY on inputs (model, intent)
/// - NO side effects (no I/O, no mutation, no randomness)
/// - Side effects go in Task::perform callbacks
pub fn update(model: Model, intent: Intent) -> (Model, Task<Intent>) {
    match intent {
        Intent::UiTabSelected(tab) => {
            (model.with_tab(tab), Task::none())
        }

        Intent::UiCreateClicked => {
            // Clone before moving into closure
            let data = model.data.clone();
            (
                model.with_status(Status::Creating),
                Task::perform(
                    async move { create_entity(data).await },
                    |result| match result {
                        Ok(entity) => Intent::DomainEntityCreated(entity),
                        Err(e) => Intent::ErrorValidation(e.to_string()),
                    }
                )
            )
        }

        Intent::DomainEntityCreated(entity) => {
            let mut data = model.data.clone();
            data.push(entity);
            (model.with_data(data).with_status(Status::Ready), Task::none())
        }

        _ => (model, Task::none())
    }
}
```

---

## Step 2: Implement LiftableDomain

### 2.1 Define Domain Types

```rust
// src/domain.rs
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct YourEntity {
    pub id: Uuid,
    pub name: String,
    // domain-specific fields
}

#[derive(Debug, Clone)]
pub struct AnotherEntity {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    // domain-specific fields
}
```

### 2.2 Define Injection Coproduct

```rust
// src/lifting.rs
use uuid::Uuid;

/// Injection identifies which type is lifted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Injection {
    YourEntity,
    AnotherEntity,
    Relationship,
}

/// LiftedNode is the coproduct (sum type) in the graph
#[derive(Debug, Clone)]
pub struct LiftedNode {
    pub id: Uuid,
    pub injection: Injection,
    pub data: LiftedData,
}

#[derive(Debug, Clone)]
pub enum LiftedData {
    YourEntity(YourEntity),
    AnotherEntity(AnotherEntity),
    // Add variants for each domain type
}
```

### 2.3 Implement LiftableDomain Trait

```rust
/// Faithful functor from domain category to graph category
pub trait LiftableDomain: Sized {
    /// Lift domain entity into graph node (injection)
    fn lift(&self) -> LiftedNode;

    /// Attempt to unlift from graph node (projection)
    fn unlift(node: &LiftedNode) -> Option<Self>;

    /// Which injection variant this type uses
    fn injection() -> Injection;
}

impl LiftableDomain for YourEntity {
    fn lift(&self) -> LiftedNode {
        LiftedNode {
            id: self.id,
            injection: Injection::YourEntity,
            data: LiftedData::YourEntity(self.clone()),
        }
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        match &node.data {
            LiftedData::YourEntity(e) => Some(e.clone()),
            _ => None,
        }
    }

    fn injection() -> Injection {
        Injection::YourEntity
    }
}
```

### 2.4 Create LiftedGraph

```rust
#[derive(Debug, Clone, Default)]
pub struct LiftedGraph {
    pub nodes: HashMap<Uuid, LiftedNode>,
    pub edges: Vec<LiftedEdge>,
}

impl LiftedGraph {
    /// Add any LiftableDomain entity
    pub fn add<T: LiftableDomain>(&mut self, entity: &T) {
        let node = entity.lift();
        self.nodes.insert(node.id, node);
    }

    /// Get all entities of a specific type
    pub fn unlift_all<T: LiftableDomain>(&self) -> Vec<T> {
        self.nodes.values()
            .filter(|n| n.injection == T::injection())
            .filter_map(T::unlift)
            .collect()
    }
}
```

---

## Step 3: Event Sourcing Pattern

### 3.1 Define Events

```rust
// src/events.rs
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EntityCreated {
    pub entity_id: Uuid,
    pub name: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum DomainEvent {
    EntityCreated(EntityCreated),
    EntityUpdated(EntityUpdated),
    // All state changes as events
}
```

### 3.2 Define Commands

```rust
// src/commands.rs
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateEntity {
    pub command_id: Uuid,
    pub entity_id: Uuid,
    pub name: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}
```

### 3.3 Implement Aggregate

```rust
// src/aggregate.rs

pub struct YourAggregate {
    projection: YourProjection,
}

impl YourAggregate {
    /// Pure function: Command + Projection → Events
    pub fn handle(&self, command: Command) -> Result<Vec<DomainEvent>, AggregateError> {
        match command {
            Command::CreateEntity(cmd) => {
                // Validate against current projection
                if self.projection.entity_exists(&cmd.entity_id) {
                    return Err(AggregateError::AlreadyExists);
                }

                // Generate event (no side effects!)
                Ok(vec![DomainEvent::EntityCreated(EntityCreated {
                    entity_id: cmd.entity_id,
                    name: cmd.name,
                    correlation_id: cmd.correlation_id,
                    causation_id: cmd.causation_id,
                    timestamp: cmd.timestamp,
                })])
            }
        }
    }
}
```

### 3.4 Implement Projection

```rust
// src/projections.rs

pub struct YourProjection {
    entities: HashMap<Uuid, Entity>,
}

impl YourProjection {
    /// Apply event to update projection state
    pub fn apply(&mut self, event: &DomainEvent) {
        match event {
            DomainEvent::EntityCreated(e) => {
                self.entities.insert(e.entity_id, Entity {
                    id: e.entity_id,
                    name: e.name.clone(),
                });
            }
            // Handle all events
        }
    }

    /// Save projection to disk
    pub fn save(&self, path: &Path) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(&self.entities)?;
        fs::write(path.join("entities.json"), json)?;
        Ok(())
    }
}
```

---

## Step 4: BDD Testing Infrastructure

### 4.1 Create TestContext

```rust
// tests/bdd/mod.rs
use std::collections::HashMap;
use uuid::Uuid;

pub struct TestContext {
    pub aggregate: Option<YourAggregate>,
    pub projection: Option<YourProjection>,
    pub captured_events: Vec<DomainEvent>,
    pub entities: HashMap<String, Uuid>,
    pub last_error: Option<String>,
    pub correlation_id: Uuid,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            aggregate: None,
            projection: None,
            captured_events: Vec::new(),
            entities: HashMap::new(),
            last_error: None,
            correlation_id: Uuid::now_v7(),
        }
    }

    pub fn has_event_of_type(&self, event_type: &str) -> bool {
        self.captured_events.iter()
            .any(|e| format!("{:?}", e).contains(event_type))
    }
}
```

### 4.2 Create Step Definitions

```rust
// tests/bdd/your_steps.rs

// Given steps - setup
pub fn given_clean_environment() -> TestContext {
    TestContext::new()
}

pub fn given_entity_exists(ctx: &mut TestContext, name: &str) -> Uuid {
    let id = Uuid::now_v7();
    ctx.entities.insert(name.to_string(), id);
    id
}

// When steps - actions
pub async fn when_create_entity(
    ctx: &mut TestContext,
    name: &str
) -> Result<Vec<DomainEvent>, String> {
    let command = CreateEntity {
        command_id: Uuid::now_v7(),
        entity_id: Uuid::now_v7(),
        name: name.to_string(),
        correlation_id: ctx.correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
    };

    let result = ctx.aggregate.as_ref()
        .ok_or("Aggregate not initialized")?
        .handle(Command::CreateEntity(command));

    match result {
        Ok(events) => {
            ctx.captured_events.extend(events.clone());
            Ok(events)
        }
        Err(e) => {
            ctx.last_error = Some(e.to_string());
            Err(e.to_string())
        }
    }
}

// Then steps - assertions
pub fn then_entity_created_event_emitted(ctx: &TestContext) -> bool {
    ctx.has_event_of_type("EntityCreated")
}
```

### 4.3 Create Feature Files

```gherkin
# doc/qa/features/your_feature.feature
Feature: Entity Management
  As a system administrator
  I want to manage entities
  So that I can track domain objects

  Background:
    Given a clean environment

  @entity @happy-path
  Scenario: Create a new entity
    Given no entities exist
    When I create an entity named "Test Entity"
    Then an EntityCreated event should be emitted
    And the entity should have name "Test Entity"

  @entity @validation
  Scenario: Reject duplicate entity
    Given an entity named "Existing" exists
    When I try to create an entity named "Existing"
    Then an error should be returned
    And no EntityCreated event should be emitted
```

### 4.4 Create BDD Test Entry Point

```rust
// tests/bdd_tests.rs
mod bdd;

use bdd::*;

#[tokio::test]
async fn scenario_create_entity() {
    // Given a clean environment
    let mut ctx = given_clean_environment();

    // When I create an entity
    let result = when_create_entity(&mut ctx, "Test Entity").await;

    // Then EntityCreated event should be emitted
    assert!(result.is_ok());
    assert!(then_entity_created_event_emitted(&ctx));
}
```

---

## Step 5: FRP Axiom Compliance

### 5.1 Verify Decoupling (A3)

```rust
#[test]
fn update_output_depends_only_on_input() {
    let model1 = Model::new();
    let model2 = Model::new();
    let intent = Intent::UiTabSelected(Tab::Settings);

    let (result1, _) = update(model1.clone(), intent.clone());
    let (result2, _) = update(model2.clone(), intent.clone());

    // Same inputs → Same outputs
    assert_eq!(result1.tab, result2.tab);
}
```

### 5.2 Verify Totality (A5)

```rust
#[test]
fn update_handles_all_intents() {
    let model = Model::new();

    // Test every Intent variant
    let intents = vec![
        Intent::UiTabSelected(Tab::Home),
        Intent::UiCreateClicked,
        Intent::DomainEntityCreated(Entity::default()),
        Intent::ErrorValidation("test".to_string()),
        // ALL variants must be tested
    ];

    for intent in intents {
        let (new_model, _) = update(model.clone(), intent);
        // Should never panic
        assert!(new_model.tab == model.tab || new_model.tab != model.tab);
    }
}
```

### 5.3 Verify Composition (A9)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn update_composition_associative(
        tab1 in any::<u8>().prop_map(|n| Tab::from(n % 3)),
        tab2 in any::<u8>().prop_map(|n| Tab::from(n % 3)),
    ) {
        let model = Model::new();

        // (f >>> g) >>> h = f >>> (g >>> h)
        let (m1, _) = update(model.clone(), Intent::UiTabSelected(tab1));
        let (m2, _) = update(m1, Intent::UiTabSelected(tab2));

        let (m3, _) = update(model.clone(), Intent::UiTabSelected(tab1));
        let (m4, _) = update(m3, Intent::UiTabSelected(tab2));

        prop_assert_eq!(m2.tab, m4.tab);
    }
}
```

---

## Migration Checklist

### Phase 1: Core Architecture
- [ ] Create `src/mvi/model.rs` with immutable Model
- [ ] Create `src/mvi/intent.rs` with categorized Intents
- [ ] Create `src/mvi/update.rs` with pure update function
- [ ] Verify no side effects in update function

### Phase 2: Domain Lifting
- [ ] Define Injection coproduct for domain types
- [ ] Implement LiftableDomain trait for each entity
- [ ] Create LiftedGraph with add/unlift operations
- [ ] Verify functor laws (identity, composition)

### Phase 3: Event Sourcing
- [ ] Define all events with correlation/causation IDs
- [ ] Define commands with timestamps
- [ ] Implement aggregate with pure handle function
- [ ] Implement projection with apply/save methods

### Phase 4: Testing
- [ ] Create TestContext for BDD tests
- [ ] Create step definitions (Given/When/Then)
- [ ] Write Gherkin feature files
- [ ] Implement property-based tests for FRP axioms

### Phase 5: Documentation
- [ ] Document MVI intent categories
- [ ] Document LiftableDomain pattern
- [ ] Create architecture diagrams
- [ ] Add test coverage summary

---

## Common Pitfalls

### 1. Mutating Model in Update

```rust
// WRONG - mutates model
fn update(mut model: Model, intent: Intent) -> (Model, Task<Intent>) {
    model.tab = Tab::Settings;  // Mutation!
    (model, Task::none())
}

// CORRECT - returns new instance
fn update(model: Model, intent: Intent) -> (Model, Task<Intent>) {
    (model.with_tab(Tab::Settings), Task::none())
}
```

### 2. Side Effects in Update

```rust
// WRONG - side effect in update
fn update(model: Model, intent: Intent) -> (Model, Task<Intent>) {
    fs::write("state.json", &model);  // Side effect!
    (model, Task::none())
}

// CORRECT - side effect in Task
fn update(model: Model, intent: Intent) -> (Model, Task<Intent>) {
    let data = model.data.clone();
    (model, Task::perform(
        async move { fs::write("state.json", &data) },
        |_| Intent::SystemSaved
    ))
}
```

### 3. Wrong Unlift Type

```rust
// WRONG - type mismatch
let entities: Vec<WrongType> = graph.unlift_all::<WrongType>();

// CORRECT - use matching type
let entities: Vec<YourEntity> = graph.unlift_all::<YourEntity>();
```

---

## Reference Implementation

See `cim-keys` for complete working examples:

| Component | Location |
|-----------|----------|
| MVI Model | `src/mvi/model.rs` |
| MVI Intent | `src/mvi/intent.rs` |
| MVI Update | `src/mvi/update.rs` |
| LiftableDomain | `src/lifting.rs` |
| Events | `src/events.rs` |
| Commands | `src/commands.rs` |
| Aggregate | `src/aggregate.rs` |
| Projection | `src/projections.rs` |
| BDD Tests | `tests/bdd/` |
| Feature Files | `doc/qa/features/` |
| MVI Tests | `tests/mvi_tests.rs` |
