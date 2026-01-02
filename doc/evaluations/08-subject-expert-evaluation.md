# Subject Expert Evaluation: Routing Pattern Analysis for cim-keys

**Evaluator:** Subject Expert
**Date:** 2026-01-02
**Overall Assessment: SIGNIFICANT VIOLATIONS FOUND**

---

## 1. Current State Analysis

### 1.1 update.rs Routing Analysis

**File:** `/git/thecowboyai/cim-keys/src/mvi/update.rs`

**File Statistics:**
- Total lines: 1,200+ lines in update logic
- Match arms: 50+ distinct intent patterns
- Nesting depth: Up to 4 levels of pattern matching

**Current Routing Pattern (Anti-Pattern):**

```rust
pub fn update(
    model: Model,
    intent: Intent,
    ports: &dyn Ports,
) -> (Model, Command) {
    match intent {
        Intent::UiTabChanged(tab) => { /* ... */ },
        Intent::UiOrganizationSelected(id) => { /* ... */ },
        Intent::UiPersonSelected(id) => { /* ... */ },
        // ... 40+ more arms
        Intent::DomainOrganizationCreated(org) => { /* ... */ },
        Intent::PortYubikeyDetected(info) => { /* ... */ },
        // ... continues
    }
}
```

### 1.2 fold.rs Analysis

The fold.rs file defines arrow combinators but they are NOT used for routing:

```rust
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
pub fn parallel<A, B, C, D, F, G>(f: F, g: G) -> impl Fn((A, C)) -> (B, D)
pub fn fanout<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> (B, C)
```

**Critical Finding:** These combinators exist but are NOT used for intent routing.

### 1.3 Missing Route Module

No dedicated routing module exists at `/git/thecowboyai/cim-keys/src/routing/`

---

## 2. Subject Algebra Violations

### 2.1 Violation: Non-Composable Routing

**Principle Violated:** Subject Algebra requires routing to be a Free Monoid with composition operation.

**Current State:**
```rust
// Anti-pattern: Flat enumeration, no composition
match intent {
    Intent::UiOrganizationSelected(id) => handle_org_selected(model, id),
    Intent::UiPersonSelected(id) => handle_person_selected(model, id),
}
```

**Required Pattern:**
```rust
let routes = Routes::new()
    .route(SubjectPattern::parse("organization.*.selected")?, handle_org_selected)
    .route(SubjectPattern::parse("person.*.selected")?, handle_person_selected)
    .compose();  // Free monoid composition
```

### 2.2 Violation: No Hierarchical Subject Structure

**Current Intent Naming:**
```rust
Intent::UiOrganizationSelected
Intent::UiOrganizationCreated
Intent::DomainOrganizationCreated
Intent::PortOrganizationLoaded
```

**Required Subject Hierarchy:**
```
ui.organization.selected
ui.organization.created
domain.organization.created
port.organization.loaded
```

### 2.3 Violation: No Pattern Matching Algebra

No wildcard support, explicit enumeration only.

---

## 3. Arrow Combinator Violations

### 3.1 Violation: A6 - Pattern Matching Instead of Routing Combinators

**FRP Axiom A6:** Use compositional primitives (>>>, ***, &&&), not pattern matching.

**Current Anti-Pattern:**
```rust
fn update(model: Model, intent: Intent, ports: &dyn Ports) -> (Model, Command) {
    match intent {
        Intent::A => /* handler_a */,
        Intent::B => /* handler_b */,
        // ... 50+ more
    }
}
```

**Required Arrow-Based Routing:**
```rust
fn route_intent() -> Route<Intent, (Model, Command)> {
    let ui_route = ui_router() >>> model_updater();
    let domain_route = domain_router() >>> event_applier();
    let port_route = port_router() >>> port_handler();

    subject_dispatch(&&&(ui_route, domain_route, port_route))
}
```

### 3.2 Violation: No Route Composition

Each intent handler is isolated, no composition chain.

---

## 4. Category Theory Violations

### 4.1 Violation: Routing is Not a Functor

No functorial structure. Routes are ad-hoc match arms.

### 4.2 Violation: No Natural Transformations

No way to transform routes while preserving composition.

---

## 5. Recommended Subject-Algebraic Routing System

### 5.1 Subject-Based Intent Hierarchy

```rust
pub enum IntentCategory {
    Ui,      // ui.*.>
    Domain,  // domain.*.>
    Port,    // port.*.>
    System,  // system.*.>
    Error,   // error.*.>
}

pub struct SubjectIntent {
    subject: Subject,
    payload: IntentPayload,
}

impl SubjectIntent {
    pub fn new(category: IntentCategory, aggregate: &str, action: &str) -> Result<Self, SubjectError> {
        let subject = Subject::parse(&format!(
            "{}.{}.{}",
            category.to_subject_prefix(),
            aggregate,
            action
        ))?;
        Ok(Self { subject, payload: IntentPayload::Empty })
    }
}
```

### 5.2 Arrow-Based Route Composition

```rust
pub trait RouteArrow<A, B> {
    fn apply(&self, input: A) -> B;
}

pub trait ArrowExt<A, B>: RouteArrow<A, B> + Sized {
    fn then<C, G>(self, next: G) -> Compose<Self, G>
    where G: RouteArrow<B, C>;

    fn both<C, D, G>(self, other: G) -> Parallel<Self, G>
    where G: RouteArrow<C, D>;

    fn split<C, G>(self, other: G) -> Fanout<Self, G>
    where A: Clone, G: RouteArrow<A, C>;
}
```

### 5.3 Subject-Dispatch Router

```rust
pub struct SubjectRouter<M, C> {
    routes: Vec<(SubjectPattern, RouteHandler<M, C>)>,
    default: Option<RouteHandler<M, C>>,
}

impl<M, C> SubjectRouter<M, C> {
    pub fn route<F>(mut self, pattern: SubjectPattern, handler: F) -> Self
    where F: Fn(M, IntentPayload) -> (M, C) + Send + Sync + 'static;

    pub fn dispatch(&self, model: M, intent: &SubjectIntent) -> Option<(M, C)>;
}
```

### 5.4 Refactored Update Function

```rust
fn build_router() -> SubjectRouter<Model, Command> {
    let ui_router = SubjectRouter::new()
        .route(SubjectPattern::parse("ui.organization.>").unwrap(), handle_ui_organization)
        .route(SubjectPattern::parse("ui.person.>").unwrap(), handle_ui_person)
        .route(SubjectPattern::parse("ui.tab.>").unwrap(), handle_ui_tab);

    let domain_router = SubjectRouter::new()
        .route(SubjectPattern::parse("domain.organization.>").unwrap(), handle_domain_organization)
        .route(SubjectPattern::parse("domain.person.>").unwrap(), handle_domain_person);

    compose_routers(vec![
        (SubjectPattern::parse("ui.>").unwrap(), ui_router),
        (SubjectPattern::parse("domain.>").unwrap(), domain_router),
    ])
}

pub fn update(model: Model, intent: SubjectIntent, ports: &dyn Ports) -> (Model, Command) {
    static ROUTER: Lazy<SubjectRouter<Model, Command>> = Lazy::new(build_router);
    ROUTER.dispatch(model, &intent).unwrap_or_else(|| (model, Command::None))
}
```

---

## 6. Corrective Action Plan

### Sprint N: Subject-Algebraic Routing Refactor (8 days)

#### Phase 1: Foundation (2 days)

| Task | Description |
|------|-------------|
| 1.1 | Create `src/routing/` module |
| 1.2 | Implement `SubjectIntent` with `cim_domain::Subject` |
| 1.3 | Implement arrow combinators |
| 1.4 | Implement `SubjectRouter` |

#### Phase 2: Migration (3 days)

| Task | Description |
|------|-------------|
| 2.1 | Map existing `Intent` enum to `SubjectIntent` |
| 2.2 | Extract UI handlers from monolithic match |
| 2.3 | Extract Domain handlers |
| 2.4 | Extract Port handlers |
| 2.5 | Extract Error handlers |

#### Phase 3: Composition (2 days)

| Task | Description |
|------|-------------|
| 3.1 | Build hierarchical router tree |
| 3.2 | Replace monolithic `update()` |
| 3.3 | Add route composition tests |

#### Phase 4: Verification (1 day)

| Task | Description |
|------|-------------|
| 4.1 | Property tests for arrow laws |
| 4.2 | Pattern matching algebra tests |
| 4.3 | Integration tests |

### Success Criteria

1. **No match statement > 20 arms** in any single function
2. **Arrow composition** used for all route chains
3. **Subject patterns** for all intent routing
4. **Functor laws** verified by property tests
5. **Route composition** tested for associativity

---

## 7. Summary of Violations

| Violation | Severity | Impact | Sprint Priority |
|-----------|----------|--------|-----------------|
| A6: Pattern matching routing | Critical | Non-compositional | P0 |
| No subject hierarchy | High | No wildcards | P0 |
| Missing arrow combinators | High | No composition | P0 |
| No route functor | Medium | No transformation | P1 |
| Monolithic update() | High | Maintenance burden | P0 |

---

## 8. Conclusion

The cim-keys routing system fundamentally violates CIM Subject Algebra principles. The current 1000+ line match statement is the antithesis of compositional routing.

**Recommended Sprint Duration:** 8 days
