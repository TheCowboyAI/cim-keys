# CIM-Keys Architectural Refactoring Plan

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

## Executive Summary

This document outlines a comprehensive refactoring of the cim-keys GUI to align with CIM architectural principles, proper Domain-Driven Design, Conceptual Spaces theory, and Functional Reactive Programming best practices.

**Critical Finding**: cim-keys has `cim-domain-*` crates as dependencies but they are **OPTIONAL** and the GUI **redefines domain models inline** instead of using them. This is a massive architectural violation.

---

## Expert Analysis Summary

### 1. DDD Expert Findings (Agent: a996ff8)

| Current Name | Proper Name | Issue |
|-------------|-------------|-------|
| `OrganizationGraph` | `OrganizationConcept` | "Graph" is implementation, not domain language |
| `GraphNode` | `ConceptEntity` | Technical term, not ubiquitous language |
| `NodeType` enum | *(REMOVE)* | Categorical error - Person IS a Person, not a "type of node" |
| `GraphMessage` | `OrganizationIntent` | Conflates UI events with domain operations |

**Validation Checklist (Current: 0/5 passing)**:
- [ ] Uses ubiquitous language from domain
- [ ] Domain expert understands without explanation
- [ ] Is a Concept (has intent/meaning), not just Graph (structural)
- [ ] Implements `LiftableDomain` for composition
- [ ] Domain entities preserved, not flattened to "node types"

### 2. Conceptual Spaces Expert Findings (Agent: a4d2f1b)

| Property | Current | Required |
|----------|---------|----------|
| Position | 2D screen `Point` | `Point3<f64>` on unit sphere |
| Knowledge | `NodeStatus` | `KnowledgeLevel` (Unknown/Suspected/KnownUnknown/Known) |
| Evidence | None | `evidence_cids: Vec<String>` |
| Confidence | None | `confidence: f64` (logarithmic formula) |
| Boundaries | Manual edges | Voronoi tessellation |
| Topology | Static | Evolution state machine |

**Key Insight**: "The graph should be a projection of semantic reality, not a data structure."

### 3. CIM Expert Findings (Agent: ad36a3b)

**Module Boundary Violations**:
- Domain models embedded in GUI (should import from `cim-domain-*`)
- 2,847-line `gui.rs` mixing UI, domain, PKI, NATS concerns
- `NodeType` hardcoded to cim-keys specific types
- Missing `LiftableDomain` trait implementation

**Current Dependency State**:
```toml
cim-domain = { path = "../cim-domain" }
cim-domain-location = { path = "../cim-domain-location" }
cim-domain-person = { path = "../cim-domain-person", optional = true }  # NOT BEING USED!
cim-domain-organization = { path = "../cim-domain-organization", optional = true }  # NOT BEING USED!
```

### 4. Iced/FRP Expert Findings (Agent: a1ec441)

**FRP Compliance: ~40%**

| Property | Current | Target |
|----------|---------|--------|
| Pure Update Function | Partial | Full |
| Immutable Model | No | Yes |
| Command/Effect Separation | Partial | Full |
| Intent Categorization | No | Yes |
| Referential Transparency | No | Yes |
| Composability | No | Yes |

**Critical Violations**:
1. Direct mutation in update function
2. Side effects (port calls) in update logic
3. Mixed Message types without categorization

### 5. TDD Expert Findings (Agent: a999cc2)

**Current Test State**: 48 tests, but:
- No GUI/MVI tests
- No pure function tests
- No property-based tests
- Limited event algebra tests
- No projection tests

### 6. BDD Expert Findings (Agent: abf0f69)

**Current BDD Coverage**: None
- No `.feature` files
- No Gherkin scenarios
- No executable specifications

### 7. Applied Category Theory Expert Findings (Agent: a58ea29)

**Critical Categorical Structures Required**:

| Structure | Purpose | Implementation |
|-----------|---------|----------------|
| **Coproduct** | Replace NodeType enum | `DomainNode` with proper injections (ι_person, ι_org, etc.) |
| **Faithful Functor** | LiftableDomain trait | `lift()`, `unlift()`, `lift_event()` preserving identity/composition |
| **Kan Extension** | Universal graph lifting | `Lan_D G` for domain → graph projection |
| **Monad** | Composition of lifting | unit, bind, join with monad laws |

**Key Traits from ACT Analysis**:
```rust
// 1. DomainEntity - Object in Domain Category
trait DomainEntity { type Id; type Event; fn id(&self) -> Self::Id; }

// 2. LiftableDomain - Faithful Functor to Graph
trait LiftableDomain: DomainEntity {
    type Lifted: GraphElement;
    fn lift(&self) -> Self::Lifted;
    fn unlift(lifted: &Self::Lifted) -> Option<Self>;
}

// 3. DomainNodeFolder - Coproduct Universal Property
trait DomainNodeFolder {
    type Output;
    fn fold_person(&self, p: &Person) -> Self::Output;
    fn fold_organization(&self, o: &Organization) -> Self::Output;
    // ... one method per domain type
}

// 4. GraphProjectionFunctor - Colimit-preserving
trait GraphProjectionFunctor {
    fn project_object<E: LiftableDomain>(&self, e: &E) -> GraphNode;
    fn project_morphism(&self, rel: &Relationship) -> GraphEdge;
}
```

**Categorical Laws to Verify**:
1. **Functor Laws**: `F(id) = id`, `F(g ∘ f) = F(g) ∘ F(f)`
2. **Coproduct**: `[f,g] ∘ ι_A = f` (universal property)
3. **Monad Laws**: Left/Right identity, Associativity

**Key Insight**: "The graph is a categorically correct projection of domain structure. Use proper coproducts, Kan extensions, and faithful functors to preserve all domain semantics."

---

## Refactoring Sprints

### Sprint 0: Foundation & Analysis (Pre-work)
**Duration**: 1 day
**Goal**: Establish baseline, create tracking infrastructure

#### Tasks:
1. [ ] Create `progress.json` tracking file
2. [ ] Document current state metrics (LOC, test count, coupling)
3. [ ] Set up sprint retrospective template
4. [ ] Verify all cim-domain-* crates are accessible
5. [ ] Create feature branch for refactoring

#### Deliverables:
- `progress.json` initialized
- Baseline metrics documented
- Sprint 0 retrospective written

---

### Sprint 1: Extract Domain Layer
**Duration**: 3-5 days
**Goal**: Remove inline domain models, use cim-domain-* and cim-graph crates

#### Available Crates (to use):
- `cim-domain` - Core domain primitives
- `cim-domain-person` - Person aggregate
- `cim-domain-organization` - Organization aggregate
- `cim-domain-location` - Location aggregate
- `cim-domain-policy` - Policy/Role/Claims
- `cim-domain-agent` - Agent (automated actors) aggregate
- `cim-domain-relationship` - **ADD TO CARGO.TOML** - Relationship modeling
- `cim-graph` - Graph composition utilities

#### Tasks:
1. [ ] Add `cim-domain-relationship` to Cargo.toml
2. [ ] Enable `cim-domain-person` feature by default
3. [ ] Enable `cim-domain-organization` feature by default
4. [ ] Enable `cim-domain-policy` feature by default
5. [ ] Enable `cim-domain-agent` feature by default
6. [ ] Remove inline `Person` struct from `src/domain.rs` and `src/domain_stubs.rs`
7. [ ] Remove inline `Organization` struct from `src/domain.rs` and `src/domain_stubs.rs`
8. [ ] Remove inline `OrganizationUnit` struct from both files
9. [ ] Replace with imports from cim-domain-* crates
10. [ ] Use cim-graph for graph composition patterns
11. [ ] Use cim-domain-relationship for entity relationships
12. [ ] Update all references throughout codebase
13. [ ] Add mapping functions where API differs
14. [ ] Verify compilation
15. [ ] Run existing tests

#### Acceptance Criteria:
- Zero inline domain model definitions in `src/domain.rs`, `src/domain_stubs.rs`, `src/gui/`
- All domain types imported from `cim-domain-*`
- Graph operations use `cim-graph` patterns
- All existing tests pass
- Compilation without warnings

#### Risks:
- API differences between inline and cim-domain-* types
- Missing traits or implementations

---

### Sprint 2: Rename Graph → Concept
**Duration**: 2-3 days
**Goal**: Align naming with DDD ubiquitous language

#### Tasks:
1. [ ] Rename `OrganizationGraph` → `OrganizationConcept`
2. [ ] Rename `GraphNode` → `ConceptEntity`
3. [ ] Rename `GraphEdge` → `ConceptRelation`
4. [ ] Rename `GraphMessage` → `OrganizationIntent`
5. [ ] Rename `graph.rs` → `concept.rs`
6. [ ] Rename `src/gui/graph/` → `src/gui/concept/`
7. [ ] Update all imports and references
8. [ ] Update documentation
9. [ ] Verify compilation
10. [ ] Run tests

#### Acceptance Criteria:
- No "Graph" terminology in domain-facing code
- All references updated
- Documentation reflects new names

---

### Sprint 3: Replace NodeType with Categorical Coproduct
**Duration**: 3-5 days
**Goal**: Use proper coproduct with injections and universal property (per ACT expert)

#### Categorical Foundation:
The current `NodeType` enum violates categorical principles because it loses identity and doesn't preserve injections. We replace it with a proper coproduct that satisfies the universal property.

#### Tasks:
1. [ ] Analyze all `NodeType` usages (21 variants currently)
2. [ ] Create `DomainNode` coproduct with injection functions
3. [ ] Implement `DomainNodeFolder` trait for universal property
4. [ ] Create injection functions: `inject_person()`, `inject_organization()`, etc.
5. [ ] Implement `fold()` method satisfying `[f,g] ∘ ι_A = f`
6. [ ] Create `VisualizationFolder` for rendering (separates concerns)
7. [ ] Update rendering code to use `fold()` pattern
8. [ ] Remove `NodeType` enum
9. [ ] Add property tests for coproduct laws
10. [ ] Verify compilation
11. [ ] Run tests

#### New Structure (per ACT Expert):
```rust
/// Coproduct of domain types - preserves injections and universal property
pub struct DomainNode {
    inner: Box<dyn DomainEntity>,
    injection: Injection,
    type_id: TypeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Injection {
    Person, Organization, OrganizationUnit, Location,
    Policy, Agent, Relationship, Key, Certificate,
    NatsOperator, NatsAccount, NatsUser, YubiKey,
}

impl DomainNode {
    /// Injection ι_Person: Person → DomainNode
    pub fn inject_person(person: Person) -> Self { /* ... */ }

    /// Universal property: fold with cases
    pub fn fold<X, F: DomainNodeFolder<Output = X>>(&self, cases: F) -> X { /* ... */ }
}

/// Universal property: For any X with morphisms from each component
pub trait DomainNodeFolder {
    type Output;
    fn fold_person(&self, p: &Person) -> Self::Output;
    fn fold_organization(&self, o: &Organization) -> Self::Output;
    fn fold_location(&self, l: &Location) -> Self::Output;
    // ... one method per domain type
}

/// Visualization as a separate functor (separation of concerns)
pub struct VisualizationFolder;
impl DomainNodeFolder for VisualizationFolder {
    type Output = VisualizationData;
    fn fold_person(&self, p: &Person) -> VisualizationData { /* ... */ }
    // ...
}
```

---

### Sprint 4: Implement MVI Intent Layer
**Duration**: 3-5 days
**Goal**: Categorize messages by origin per FRP principles

#### Tasks:
1. [ ] Create `src/gui/intent.rs`
2. [ ] Define `Intent` enum with categories:
   - `Ui(UiIntent)` - UI-originated
   - `Port(PortIntent)` - External events
   - `Domain(DomainIntent)` - Domain events
   - `System(SystemIntent)` - Ticks, resize
3. [ ] Migrate `Message` variants to appropriate Intent types
4. [ ] Update `update` function signature
5. [ ] Route intents to specialized handlers
6. [ ] Verify compilation
7. [ ] Run tests

#### New Structure:
```rust
pub enum Intent {
    Ui(UiIntent),
    Port(PortIntent),
    Domain(DomainIntent),
    System(SystemIntent),
}

pub enum UiIntent {
    ViewChanged(View),
    NodeSelected(NodeId),
    FormSubmitted,
    // ...
}
```

---

### Sprint 5: Pure Update Functions
**Duration**: 3-5 days
**Goal**: Achieve referential transparency in update logic

#### Tasks:
1. [ ] Extract `update_pure` function that returns `(Model, Command)`
2. [ ] Remove direct mutations - use `with_*` methods
3. [ ] Remove port calls from update - return Commands instead
4. [ ] Implement `Command` enum for side effects
5. [ ] Create `Model` with immutable `with_*` transformation methods
6. [ ] Update Iced Application to call pure function
7. [ ] Verify no side effects in update logic
8. [ ] Add property tests for update purity
9. [ ] Run tests

#### New Pattern:
```rust
pub fn update_pure(model: Model, intent: Intent) -> (Model, Command<Intent>) {
    match intent {
        Intent::Ui(ui) => update_ui(model, ui),
        Intent::Port(port) => update_port(model, port),
        // ...
    }
}

impl Model {
    pub fn with_view(self, view: View) -> Self {
        Self { current_view: view, ..self }
    }
}
```

---

### Sprint 6: Conceptual Spaces Integration
**Duration**: 5-7 days
**Goal**: Align with cim-domain-spaces Concept model

#### Tasks:
1. [ ] Add `cim-domain-spaces` dependency
2. [ ] Replace 2D `Point` with `Point3<f64>` semantic positions
3. [ ] Implement stereographic projection for 2D rendering
4. [ ] Add `KnowledgeLevel` to entities
5. [ ] Add `confidence` scoring
6. [ ] Add `evidence_cids` tracking
7. [ ] Implement Fibonacci sphere layout
8. [ ] Add Voronoi cell visualization (optional)
9. [ ] Create visual encoding for knowledge levels
10. [ ] Run tests

#### Visual Encoding:
| Knowledge Level | Visual |
|----------------|--------|
| Unknown | Dashed outline, 30% opacity |
| Suspected | Amber, 50% opacity |
| KnownUnknown | Purple, solid with `?` |
| Known | Green, fully solid |

---

### Sprint 7: LiftableDomain as Faithful Functor + Monad
**Duration**: 3-5 days
**Goal**: Enable composition with proper categorical structure (per ACT expert)

#### Categorical Foundation:
`LiftableDomain` is a **faithful functor** from Domain categories to Graph category, with **monad structure** for composition (unit, bind, join).

#### Tasks:
1. [ ] Define `LiftableDomain` trait as faithful functor
2. [ ] Implement functor laws: `lift(id) = id`, `lift(g ∘ f) = lift(g) ∘ lift(f)`
3. [ ] Implement `lift_event()` for morphism mapping
4. [ ] Add `unlift()` for partial inverse (faithfulness)
5. [ ] Create `LiftedEntity<T>` monad wrapper
6. [ ] Implement monad operations: `unit()`, `bind()`, `join()`
7. [ ] Add property tests for monad laws (left/right identity, associativity)
8. [ ] Implement for Person, Organization, Location, Policy, Agent, Relationship
9. [ ] Create `GraphProjectionFunctor` for colimit-preserving projection
10. [ ] Test composition with Kan extension pattern
11. [ ] Document categorical laws and composition pattern

#### New Structure (per ACT Expert):
```rust
/// Faithful functor from Domain to Graph
pub trait LiftableDomain: DomainEntity + Sized {
    type Lifted: GraphElement;

    fn lift(&self) -> Self::Lifted;
    fn unlift(lifted: &Self::Lifted) -> Option<Self>;
    fn lift_event(event: Self::Event) -> GraphEvent;

    // Functor law verification
    fn verify_identity_preservation(&self) -> bool;
    fn verify_composition_preservation(f: Self::Event, g: Self::Event) -> bool;
}

/// Monad for composing liftings
pub struct LiftedEntity<T: LiftableDomain> {
    lifted: T::Lifted,
    original: Option<T>,
}

impl<T: LiftableDomain> LiftedEntity<T> {
    pub fn unit(entity: T) -> Self { /* monad unit */ }
    pub fn bind<B, F>(self, f: F) -> LiftedEntity<B> where B: LiftableDomain, F: FnOnce(T) -> LiftedEntity<B>;
    pub fn join(nested: LiftedEntity<LiftedEntity<T>>) -> LiftedEntity<T>;
}
```

---

### Sprint 8: Test Infrastructure
**Duration**: 3-5 days
**Goal**: Comprehensive test coverage

#### Tasks:
1. [ ] Create `tests/unit/gui/` directory structure
2. [ ] Add property-based testing with `proptest`
3. [ ] Write pure update function tests
4. [ ] Write model immutability tests
5. [ ] Write event sourcing property tests
6. [ ] Write FRP axiom compliance tests
7. [ ] Achieve >80% coverage on `src/gui/`

---

### Sprint 9: BDD Specifications
**Duration**: 3-5 days
**Goal**: Executable specifications

#### Tasks:
1. [ ] Create `doc/qa/features/` directory
2. [ ] Write `domain_bootstrap.feature`
3. [ ] Write `person_management.feature`
4. [ ] Write `key_generation.feature`
5. [ ] Write `yubikey_provisioning.feature`
6. [ ] Write `nats_security_bootstrap.feature`
7. [ ] Write `export_manifest.feature`
8. [ ] Implement step definitions in `tests/bdd/`
9. [ ] Verify all scenarios pass

---

### Sprint 10: Final Integration & Documentation
**Duration**: 2-3 days
**Goal**: Polish and document

#### Tasks:
1. [ ] Final code review
2. [ ] Update CLAUDE.md with new patterns
3. [ ] Update README.md
4. [ ] Create architecture diagram
5. [ ] Write migration guide for other cim-* modules
6. [ ] Final retrospective

---

## Success Metrics

| Metric | Before | After |
|--------|--------|-------|
| DDD Validation Checks | 0/5 | 5/5 |
| FRP Compliance | 40% | 95% |
| Inline Domain Models | 5+ | 0 |
| Test Coverage (GUI) | ~10% | >80% |
| BDD Scenarios | 0 | 18+ |
| NodeType Variants | 15+ | 0 (removed) |
| Lines in gui.rs | 2,847 | <500 |

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| cim-domain-* API incompatibility | Medium | High | Create adapter layer |
| Breaking existing functionality | Medium | High | Comprehensive tests before refactor |
| Scope creep | High | Medium | Strict sprint boundaries |
| Performance regression | Low | Medium | Benchmark before/after |
| Team unfamiliarity with patterns | Medium | Medium | Document patterns, pair programming |

---

## Approval

- [ ] User reviewed and approved Sprint 0 tasks
- [ ] User confirmed understanding of multi-sprint timeline
- [ ] User agreed to retrospective process

**Date**: _______________
**Approved By**: _______________
