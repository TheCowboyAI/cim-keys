# Contributing to cim-keys

**Version:** 1.0.0
**Date:** 2025-01-21
**Status:** Active

---

## 🔴 CRITICAL: FRP-FIRST ARCHITECTURE

This project uses **Functional Reactive Programming (FRP)** with **N-ary signal composition**. **Object-Oriented Programming (OOP) patterns are strictly forbidden.**

Before contributing, you **MUST**:
1. Read this entire document
2. Understand the [10 N-ary FRP Axioms](./N_ARY_FRP_AXIOMS.md)
3. Review the [FRP Violation Report](./FRP_VIOLATION_REPORT.md) to see what NOT to do
4. Study the [Graph-First UI Architecture](./docs/GRAPH_FIRST_UI_ARCHITECTURE.md)

**If you write OOP code, your PR will be immediately rejected.**

---

## Table of Contents

1. [Pre-Commit Checklist](#pre-commit-checklist)
2. [FRP Compliance Requirements](#frp-compliance-requirements)
3. [OOP Anti-Pattern Detection](#oop-anti-pattern-detection)
4. [Correct FRP Patterns](#correct-frp-patterns)
5. [Domain Boundary Rules](#domain-boundary-rules)
6. [Testing Requirements](#testing-requirements)
7. [Code Review Process](#code-review-process)
8. [Documentation Requirements](#documentation-requirements)

---

## Pre-Commit Checklist

**Before committing ANY code, verify ALL items:**

### ✅ FRP Compliance

- [ ] All functions are pure (no `&mut self`, consume `self` → return new)
- [ ] No pattern matching for routing (use compositional operators)
- [ ] No functions stored in data structures
- [ ] No `.await` in domain logic (use Commands/Tasks)
- [ ] No mutable field assignments
- [ ] N-ary properties used (HashMap<String, Value>, not individual fields)

### ✅ Domain Boundaries

- [ ] Email/Phone are in **Location** domain (not Person)
- [ ] Roles are in **Organization** domain (not Person fields)
- [ ] Relationships are **graph edges** (not embedded fields)
- [ ] Importing from cim-domain-* (not recreating aggregates)

### ✅ Testing

- [ ] All functions have unit tests
- [ ] Pure functions tested (no mocking needed)
- [ ] Property tests for compositional laws (if using operators)
- [ ] Tests pass: `cargo test --all-features`

### ✅ Documentation

- [ ] Module-level doc comments explain FRP patterns used
- [ ] Axiom compliance marked in comments (e.g., `// FRP Axiom A3`)
- [ ] Public functions documented
- [ ] Examples provided for complex patterns

### ✅ Build

- [ ] Code compiles: `cargo build --all-features`
- [ ] No new warnings introduced
- [ ] Clippy passes: `cargo clippy --all-features`
- [ ] Format applied: `cargo fmt`

---

## FRP Compliance Requirements

### Mandatory: All 10 N-ary FRP Axioms

Your code **MUST** satisfy these axioms. Violations will be rejected.

#### A1: Multi-Kinded Signal Types (REQUIRED)

**Rule:** Distinguish Event/Step/Continuous signals at type level.

**How to Comply:**
```rust
// ✅ CORRECT: Signal kinds distinguished
pub enum Message {
    Event(EventMessage),      // Discrete events
    Step(StepMessage),        // Piecewise constant
    Continuous(ContinuousMessage), // Smooth
}
```

**Violation Example:**
```rust
// ❌ WRONG: All signals treated the same
pub enum Message {
    Click,
    MouseMove,
    // No distinction between discrete and continuous
}
```

---

#### A2: Signal Vector Composition (REQUIRED)

**Rule:** Operate on signal vectors (n-ary), not single signals.

**How to Comply:**
```rust
// ✅ CORRECT: N-ary properties (HashMap)
pub struct DomainObject {
    pub properties: HashMap<String, Value>,  // N-ary!
}

// Operations on multiple properties simultaneously
fn update_properties(
    self,
    updates: HashMap<String, Value>
) -> Self
```

**Violation Example:**
```rust
// ❌ WRONG: Individual fields (not n-ary)
pub struct Person {
    pub name: String,      // Individual field
    pub email: String,     // Individual field
    pub phone: String,     // Individual field
}
```

---

#### A3: Decoupled Signal Functions (REQUIRED)

**Rule:** Output at time t depends ONLY on input before t (no future dependencies).

**How to Comply:**
```rust
// ✅ CORRECT: Pure function, consumes self, returns new
pub fn apply_event(self, event: &Event) -> Self {
    Self {
        state: event.apply(self.state),
        version: self.version + 1,
        ..self
    }
}

// Mark compliance in comments
// FRP Axiom A3: Decoupled - output depends only on prior input
```

**Violation Example:**
```rust
// ❌ WRONG: Mutable update (not decoupled)
pub fn apply_event(&mut self, event: &Event) {
    self.state = event.apply(&self.state);
    self.version += 1;
}
```

---

#### A4: Causality Guarantees (REQUIRED)

**Rule:** Events have correlation/causation IDs. Ordering preserved.

**How to Comply:**
```rust
// ✅ CORRECT: Causality tracking
pub struct Event {
    pub id: Uuid,
    pub correlation_id: Uuid,      // Links related events
    pub causation_id: Option<Uuid>, // What caused this
    pub timestamp: DateTime<Utc>,
}

// Use UUID v7 for time-ordering
let id = Uuid::now_v7();
```

**Violation Example:**
```rust
// ❌ WRONG: No causality tracking
pub struct Event {
    pub data: String,
    // Missing correlation_id, causation_id
}
```

---

#### A5: Totality and Well-Definedness (REQUIRED)

**Rule:** All functions total (no panics), deterministic.

**How to Comply:**
```rust
// ✅ CORRECT: Total function, handles all cases
pub fn parse_value(s: &str) -> Value {
    if let Ok(num) = s.parse::<f64>() {
        Value::Number(num)
    } else if s == "true" {
        Value::Bool(true)
    } else if s == "false" {
        Value::Bool(false)
    } else {
        Value::String(s.to_string())  // Fallback: always succeeds
    }
}
```

**Violation Example:**
```rust
// ❌ WRONG: Can panic
pub fn parse_value(s: &str) -> Value {
    Value::Number(s.parse().unwrap())  // PANIC if not a number!
}
```

---

#### A6: Explicit Routing at Reactive Level (REQUIRED)

**Rule:** Use compositional operators (`>>>`, `***`, `&&&`), not pattern matching.

**How to Comply:**
```rust
// ✅ CORRECT: Compositional routing (future requirement)
let pipeline = validate >>> transform >>> persist;

// Current acceptable alternative (prototype):
// Simple message discrimination is okay
match message {
    Message::Event(e) => handle_event(self, e),
    Message::Step(s) => handle_step(self, s),
}
```

**Violation Example:**
```rust
// ❌ WRONG: Complex nested pattern matching for routing
match (message, self.state, self.mode) {
    (Msg::A, State::X, Mode::Y) => /* ... */,
    (Msg::B, State::X, Mode::Z) => /* ... */,
    // ... 100+ lines of routing logic
}
```

**Note:** Compositional operators are planned for Phase 5. Current pattern matching is acceptable if simple and not nested.

---

#### A7: Change Prefixes as Event Logs (REQUIRED)

**Rule:** Events stored as timestamped change prefixes. State derived from fold.

**How to Comply:**
```rust
// ✅ CORRECT: Events as ordered log
pub struct EventLog {
    events: Vec<Event>,  // Ordered by timestamp
}

// State derived from fold
pub fn project(events: &[Event]) -> State {
    events.iter().fold(State::default(), |state, event| {
        state.apply_event(event)
    })
}

// FRP Axiom A7: Change prefixes as event logs
```

**Violation Example:**
```rust
// ❌ WRONG: Direct state mutation (CRUD)
pub fn update(&mut self, name: String) {
    self.name = name;  // No event log!
}
```

---

#### A8: Type-Safe Feedback Loops (REQUIRED)

**Rule:** Feedback only for decoupled functions with proof.

**How to Comply:**
```rust
// ✅ CORRECT: Feedback loop (graph projection)
pub fn apply_events(self, events: &[Event]) -> Self {
    events.iter().fold(self, |graph, event| {
        graph.apply_event(event)  // Pure, decoupled
    })
}

// FRP Axiom A8: Type-safe feedback loop
```

**Violation Example:**
```rust
// ❌ WRONG: Feedback with side effects
pub fn update(&mut self, events: &[Event]) {
    for event in events {
        self.save_to_db(event);  // Side effect in loop!
        self.apply(event);
    }
}
```

---

#### A9: Semantic Preservation Under Composition (REQUIRED)

**Rule:** Compositional laws must hold. Composition preserves semantics.

**How to Comply:**
```rust
// ✅ CORRECT: Compositional law verification
#[test]
fn test_composition_associativity() {
    let f = |x| x + 1;
    let g = |x| x * 2;
    let h = |x| x - 3;

    // (f >>> g) >>> h == f >>> (g >>> h)
    assert_eq!(
        compose(compose(f, g), h)(5),
        compose(f, compose(g, h))(5)
    );
}

// FRP Axiom A9: Semantic preservation
```

**Violation Example:**
```rust
// ❌ WRONG: Composition breaks semantics
fn transform(&mut self, x: i32) -> i32 {
    self.counter += 1;  // Side effect breaks composition!
    x * 2
}
```

---

#### A10: Continuous Time Semantics (REQUIRED)

**Rule:** Time is continuous in semantics (discrete in implementation okay).

**How to Comply:**
```rust
// ✅ CORRECT: Continuous time semantics
pub trait Signal<T> {
    fn sample(&self, t: f64) -> T;  // t is continuous
}

// Implementation can use discrete samples
pub struct DiscreteSignal {
    samples: Vec<(f64, Value)>,  // (time, value) pairs
}

// FRP Axiom A10: Continuous time semantics
```

**Note:** For event-driven systems, discrete time (UUID v7 timestamps) is acceptable. Continuous time is needed for animations.

---

## OOP Anti-Pattern Detection

**These patterns are STRICTLY FORBIDDEN. Code review will reject them immediately.**

### ❌ Anti-Pattern 1: Mutable Methods (`&mut self`)

**NEVER use `&mut self`. ALWAYS consume `self` and return new instance.**

```rust
// ❌ WRONG
impl Aggregate {
    pub fn update(&mut self, value: String) {
        self.field = value;
    }
}

// ✅ CORRECT
impl Aggregate {
    pub fn update(self, value: String) -> Self {
        Self {
            field: value,
            version: self.version + 1,
            ..self
        }
    }
}
```

**Detection:** Search for `&mut self` in domain code. If found, reject PR.

---

### ❌ Anti-Pattern 2: Pattern Matching for Routing

**NEVER use complex pattern matching for routing. Use compositional operators.**

```rust
// ❌ WRONG: Complex routing via pattern matching
match (command, state, mode) {
    (Cmd::A, State::X, Mode::Y) => route_a_x_y(),
    (Cmd::A, State::X, Mode::Z) => route_a_x_z(),
    // ... 50 more branches
}

// ✅ CORRECT: Compositional routing (planned for Phase 5)
let router = when(is_mode_y) >>> route_a_x
           . when(is_mode_z) >>> route_a_z;

// ✅ ACCEPTABLE (current): Simple message discrimination
match message {
    Message::Click(id) => handle_click(self, id),
    Message::Input(text) => handle_input(self, text),
}
```

**Detection:** Look for nested pattern matching with 10+ branches. Flag for review.

---

### ❌ Anti-Pattern 3: Stored Callbacks

**NEVER store functions in data structures. Store data only.**

```rust
// ❌ WRONG
pub struct Widget {
    on_click: Box<dyn Fn()>,  // Stored callback!
}

// ✅ CORRECT
pub struct Widget {
    id: Uuid,
}

pub enum Intent {
    WidgetClicked(Uuid),  // Data, not function
}
```

**Detection:** Search for `Box<dyn Fn`, `fn()`, `Fn`, `FnMut` in struct fields. Reject if found.

---

### ❌ Anti-Pattern 4: Direct `.await` in Domain Logic

**NEVER use `.await` in domain update functions. Use Commands/Tasks.**

```rust
// ❌ WRONG
pub async fn update(self, id: Uuid) -> Self {
    let data = fetch_from_db(id).await;  // Side effect!
    Self { data, ..self }
}

// ✅ CORRECT
pub fn update(self, id: Uuid) -> (Self, Command) {
    let command = Command::FetchData(id);
    (self, command)  // Return command, execute elsewhere
}
```

**Detection:** Search for `.await` in `update`, `apply`, `transition` functions. Reject if found.

---

### ❌ Anti-Pattern 5: Mutable Field Assignments

**NEVER assign to fields. Use struct update syntax.**

```rust
// ❌ WRONG
pub fn update(mut self, name: String) -> Self {
    self.name = name;  // Direct assignment!
    self
}

// ✅ CORRECT
pub fn update(self, name: String) -> Self {
    Self {
        name,
        version: self.version + 1,
        ..self
    }
}
```

**Detection:** Search for `self.field =` in function bodies. Reject if found.

---

### ❌ Anti-Pattern 6: Individual Struct Fields

**NEVER use individual fields for domain properties. Use HashMap (n-ary).**

```rust
// ❌ WRONG
pub struct Person {
    pub name: String,
    pub email: String,
    pub phone: String,
}

// ✅ CORRECT
pub struct DomainObject {
    pub id: Uuid,
    pub aggregate_type: String,
    pub properties: HashMap<String, Value>,  // N-ary!
}

let person = DomainObject {
    properties: hashmap!{
        "name" => json!("Alice"),
        "email" => json!("alice@example.com"),
    },
    // ...
};
```

**Detection:** Check if domain structs have >3 individual fields. Flag for review. Require HashMap for properties.

---

## Correct FRP Patterns

### ✅ Pattern 1: Pure Event Application

**Template:**
```rust
pub fn apply_event(self, event: &Event) -> DomainResult<Self> {
    match event {
        Event::Created(e) => self.apply_created(e),
        Event::Updated(e) => self.apply_updated(e),
    }
}

fn apply_created(self, event: &Created) -> DomainResult<Self> {
    Ok(Self {
        id: event.id,
        properties: event.properties.clone(),
        version: 0,
    })
}

fn apply_updated(self, event: &Updated) -> DomainResult<Self> {
    Ok(Self {
        properties: event.new_properties.clone(),
        version: self.version + 1,
        ..self
    })
}
```

**Key Points:**
- Consume `self` by value
- Return `Self` (new instance)
- Use struct update syntax `..self`
- No side effects

---

### ✅ Pattern 2: N-ary Properties

**Template:**
```rust
pub struct DomainObject {
    pub id: Uuid,
    pub aggregate_type: DomainAggregateType,
    pub properties: HashMap<String, serde_json::Value>,  // N-ary!
    pub version: u64,
}

impl DomainObject {
    pub fn with_property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.properties.insert(key.into(), value);
        Self {
            version: self.version + 1,
            ..self
        }
    }
}
```

**Key Points:**
- Properties as HashMap (not individual fields)
- Works for ANY aggregate type
- No hardcoded property names
- Generic operations

---

### ✅ Pattern 3: Generic Rendering

**Template:**
```rust
pub fn render_node(&self, object: &DomainObject) -> Element {
    let label = self.get_node_label(object);
    let icon = self.get_node_icon(object);
    let color = self.get_node_color(object);

    // Render using generic properties
    Container::new(...)
}

fn get_node_label(&self, object: &DomainObject) -> String {
    // Try common property names (no domain-specific code!)
    if let Some(name) = object.properties.get("legal_name") {
        return name.as_str().unwrap_or("Unnamed").to_string();
    }
    if let Some(name) = object.properties.get("name") {
        return name.as_str().unwrap_or("Unnamed").to_string();
    }
    // Fallback
    format!("{} ({})", object.aggregate_type, &object.id.to_string()[..8])
}
```

**Key Points:**
- NO `if aggregate_type == "Person"` branches
- Property-based logic (check property names, not types)
- Works for ANY aggregate type
- Fallback to generic representation

---

### ✅ Pattern 4: Domain Boundaries as Edges

**Template:**
```rust
// Person aggregate (from cim-domain-person)
let person = DomainObject {
    id: person_id,
    aggregate_type: DomainAggregateType::Person,
    properties: hashmap!{
        "legal_name" => json!("Alice Smith"),
    },
};

// Email as Location aggregate (from cim-domain-location)
let email_location = DomainObject {
    id: location_id,
    aggregate_type: DomainAggregateType::Location,
    properties: hashmap!{
        "address" => json!("alice@example.com"),
        "location_type" => json!("email"),
    },
};

// Relationship as graph edge (NOT embedded!)
let contact_edge = DomainRelationship {
    source_id: person_id,
    target_id: location_id,
    relationship_type: RelationshipType::Custom("has_contact"),
};
```

**Key Points:**
- Email is Location domain (not Person field!)
- Relationship as edge, not embedded Vec
- Each aggregate independent
- Graph structure explicit

---

## Domain Boundary Rules

**CRITICAL: These boundaries are MANDATORY. Violations will be rejected.**

### Person Domain (cim-domain-person)

**Contains:**
- ✅ Legal name
- ✅ Birth/death dates
- ✅ Lifecycle state

**Does NOT Contain:**
- ❌ Email (use Location + edge)
- ❌ Phone (use Location + edge)
- ❌ Address (use Location + edge)
- ❌ Roles (use Organization + edge)
- ❌ Employment (use Organization + edge)

---

### Organization Domain (cim-domain-organization)

**Contains:**
- ✅ Organization name
- ✅ Structure policies

**Does NOT Contain:**
- ❌ Embedded departments (separate aggregates + edges)
- ❌ Embedded members (Person aggregates + edges)
- ❌ Contact info (Location + edges)

---

### Location Domain (cim-domain-location)

**Contains:**
- ✅ Physical locations
- ✅ Email addresses
- ✅ Phone numbers
- ✅ Location type (physical/email/phone/etc.)

**Does NOT Contain:**
- ❌ Owner information (use edges to Person/Organization)

---

### Key Domain (cim-keys - THIS REPOSITORY)

**Contains:**
- ✅ Key algorithm
- ✅ Key purpose
- ✅ Key state

**Does NOT Contain:**
- ❌ Owner fields (use edges to Person)
- ❌ Storage location fields (use edges to Location)

---

## Testing Requirements

### Unit Tests (REQUIRED)

**Every function MUST have unit tests.**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_update() {
        let obj1 = DomainObject::new("Test");
        let obj2 = obj1.clone().with_property("key", json!("value"));

        // Original unchanged (immutability)
        assert_eq!(obj1.properties.len(), 0);

        // New has property
        assert_eq!(obj2.properties.get("key"), Some(&json!("value")));

        // Version incremented
        assert_eq!(obj2.version, obj1.version + 1);
    }
}
```

---

### Property Tests (REQUIRED for Compositional Laws)

**Use `proptest` to verify compositional laws.**

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn composition_associativity(x in 0..100) {
            let f = |n| n + 1;
            let g = |n| n * 2;
            let h = |n| n - 3;

            // (f >>> g) >>> h == f >>> (g >>> h)
            prop_assert_eq!(
                compose(compose(f, g), h)(x),
                compose(f, compose(g, h))(x)
            );
        }
    }
}
```

---

### Integration Tests (RECOMMENDED)

**Test with real event sourcing.**

```rust
#[test]
fn test_event_sourcing_roundtrip() {
    let events = vec![
        Event::PersonCreated { id: person_id, legal_name: "Alice".into() },
        Event::LocationCreated { id: email_id, address: "alice@example.com".into() },
        Event::RelationshipEstablished { source_id: person_id, target_id: email_id, ... },
    ];

    let graph = events.iter().fold(DomainGraph::new(), |g, e| g.apply_event(e));

    // Verify projection
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}
```

---

## Code Review Process

### Automated Checks (CI)

All PRs MUST pass:

1. **Compilation**: `cargo build --all-features`
2. **Tests**: `cargo test --all-features`
3. **Clippy**: `cargo clippy --all-features -- -D warnings`
4. **Format**: `cargo fmt -- --check`

### Manual Review Checklist

Reviewers MUST verify:

- [ ] No OOP anti-patterns (check all 6)
- [ ] FRP axioms satisfied (mark which ones in code)
- [ ] Domain boundaries respected
- [ ] Tests comprehensive
- [ ] Documentation clear
- [ ] Examples provided

### FRP Expert Review (Complex Changes)

For architectural changes, invoke FRP expert agent:

```bash
# Request FRP expert validation
# See .claude/agents/frp-expert.md
```

---

## Documentation Requirements

### Module-Level Comments (REQUIRED)

```rust
//! Graph-First UI Module
//!
//! FRP-compliant graph visualization using cim-graph DomainObjects.
//!
//! **FRP Axioms Satisfied:**
//! - A3: Decoupled Signal Functions (pure updates)
//! - A5: Totality (all functions total)
//! - A7: Change Prefixes (event logs)
//!
//! **Key Design:**
//! - Generic rendering (works with ANY DomainObject)
//! - N-ary properties (HashMap, not struct fields)
```

---

### Function Comments (REQUIRED for Public API)

```rust
/// Update domain object with new property (pure function).
///
/// **FRP Axiom A3:** Consumes self, returns new instance (decoupled).
///
/// # Examples
///
/// ```
/// let obj = DomainObject::new("Person")
///     .with_property("name", json!("Alice"));
/// ```
pub fn with_property(mut self, key: impl Into<String>, value: Value) -> Self {
    // ...
}
```

---

### Architecture Decision Records (RECOMMENDED)

For major changes, document decisions:

```markdown
# ADR 001: Use HashMap for N-ary Properties

**Date:** 2025-01-21
**Status:** Accepted

## Context
Need generic property storage that works for ANY aggregate type.

## Decision
Use `HashMap<String, serde_json::Value>` for properties.

## Consequences
- ✅ Generic (works for all types)
- ✅ Flexible (add properties without code changes)
- ⚠️ Less type safety (runtime checks needed)
```

---

## Getting Help

### Resources

1. **N-ary FRP Axioms**: [N_ARY_FRP_AXIOMS.md](./N_ARY_FRP_AXIOMS.md)
2. **Violation Examples**: [FRP_VIOLATION_REPORT.md](./FRP_VIOLATION_REPORT.md)
3. **Architecture Guide**: [docs/GRAPH_FIRST_UI_ARCHITECTURE.md](./docs/GRAPH_FIRST_UI_ARCHITECTURE.md)
4. **Prototype Example**: [examples/graph_ui_prototype.rs](./examples/graph_ui_prototype.rs)

### Expert Agents

Invoke specialized agents for guidance:

- **FRP Expert**: `.claude/agents/frp-expert.md` - FRP compliance validation
- **DDD Expert**: `.claude/agents/ddd-expert.md` - Domain boundary guidance
- **CIM Expert**: `.claude/agents/cim-expert.md` - Architecture guidance

### Questions?

1. Check existing issues/discussions
2. Read the architecture docs
3. Study the prototype example
4. Ask in project discussions

---

## Contribution Workflow

1. **Read this document** (you're doing it!)
2. **Study the prototype** (`examples/graph_ui_prototype.rs`)
3. **Fork and branch** (`git checkout -b feature/your-feature`)
4. **Write tests first** (TDD)
5. **Implement FRP-compliant code** (use checklist)
6. **Run tests** (`cargo test --all-features`)
7. **Format code** (`cargo fmt`)
8. **Check clippy** (`cargo clippy --all-features`)
9. **Commit** (use conventional commits: `feat:`, `fix:`, etc.)
10. **Push and create PR**
11. **Address review feedback**

---

## Final Note

**This is not optional.** FRP compliance is the foundation of this project. OOP patterns will corrupt the architecture and make the codebase unmaintainable.

When in doubt:
1. Check the checklist
2. Study the prototype
3. Ask the FRP expert agent

Thank you for contributing to a mathematically sound, compositional, maintainable codebase!

---

**Version History:**
- v1.0.0 (2025-01-21): Initial version based on Phase 1-3 learnings
