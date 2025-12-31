<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# ADR-001: Compositional Coproduct Architecture

**Status**: Accepted
**Date**: 2025-12-31
**Decision Makers**: Domain experts (ACT, DDD, FRP)

## Context

The `cim-keys` GUI module contains a `DomainNode` type that represents all domain entities
in the graph visualization. This type currently implements a "god coproduct" pattern - a
single sum type containing ~25 variants spanning multiple bounded contexts:

- **Organization Context**: Organization, OrganizationUnit, Person, Location
- **PKI Context**: RootCA, IntermediateCA, LeafCertificate, CertificateChain
- **NATS Context**: NatsOperator, NatsAccount, NatsUser, SigningKey
- **YubiKey Context**: YubiKey, PIVSlot
- **Visualization Context**: Category
- **Aggregates**: AggregateOrganization, AggregatePkiChain, etc.

The file `src/gui/domain_node.rs` exceeds 30,000 tokens (~2,900 lines), violating our
10K token file size limit.

## Problem Statement

### Current Anti-Patterns Identified

1. **God Coproduct (DDD Violation)**
   - Single sum type crosses bounded context boundaries
   - Changes to PKI types force recompilation of Organization code
   - No encapsulation of context-specific invariants
   - Violates "each context owns its types" principle

2. **Fold Explosion**
   - Every fold implementation requires 25+ method implementations
   - Adding one entity type requires modifying ALL fold implementations
   - O(n×m) complexity: n entity types × m fold implementations

3. **Pattern Matching for Dispatch (FRP A6 Violation)**
   - Using `match` statements instead of compositional routing
   - Cannot compose signal functions algebraically
   - Violates "explicit routing with compositional primitives"

4. **Mixed Concerns**
   - Aggregates (compositions) mixed with entities (coproduct members)
   - Aggregates should compose entities, not be entities themselves

## Decision

Adopt a **Compositional Coproduct Architecture** based on expert recommendations:

### 1. Per-Context Coproducts (DDD)

Each bounded context defines its OWN entity coproduct:

```rust
// organization/entity.rs
pub enum OrganizationEntity {
    Organization(Organization),
    OrganizationUnit(OrganizationUnit),
    Person(Person),
    Location(Location),
}

// pki/entity.rs
pub enum PkiEntity {
    RootCA(RootCA),
    IntermediateCA(IntermediateCA),
    LeafCertificate(LeafCertificate),
    CertificateChain(CertificateChain),
}

// nats/entity.rs
pub enum NatsEntity {
    Operator(NatsOperator),
    Account(NatsAccount),
    User(NatsUser),
    SigningKey(SigningKey),
}

// yubikey/entity.rs
pub enum YubiKeyEntity {
    Device(YubiKey),
    Slot(PIVSlot),
}
```

### 2. Lifting Functors (ACT)

Each context provides a **functor** from its coproduct to the visualization layer:

```rust
pub trait LiftableDomain: Clone + Send + Sync + 'static {
    /// Lift domain entity into graph representation
    fn lift(&self) -> LiftedNode;

    /// Attempt to recover original type from graph node
    fn unlift(node: &LiftedNode) -> Option<Self>;

    /// Coproduct injection tag for type dispatch
    fn context() -> ContextTag;

    /// Entity identifier
    fn entity_id(&self) -> Uuid;
}
```

This is a **faithful functor** - it preserves structure and enables round-tripping.

### 3. Composition Coproduct

The visualization layer uses a composition coproduct of LIFTED nodes:

```rust
// composition/lifted_node.rs
pub enum LiftedNode {
    Organization(Box<dyn LiftableDomain>),
    Pki(Box<dyn LiftableDomain>),
    Nats(Box<dyn LiftableDomain>),
    YubiKey(Box<dyn LiftableDomain>),
}
```

This coproduct has only 4 variants (one per context), not 25.

### 4. Aggregates as Compositions

Aggregates are NOT coproduct variants. They are **compositions of entities**:

```rust
// aggregates/organization_aggregate.rs
pub struct OrganizationAggregate {
    root: Organization,
    units: Vec<OrganizationUnit>,
    people: Vec<Person>,
    locations: Vec<Location>,
}

impl OrganizationAggregate {
    /// Project aggregate state to lifted nodes for visualization
    pub fn project(&self) -> Vec<LiftedNode> {
        let mut nodes = vec![self.root.lift()];
        nodes.extend(self.units.iter().map(|u| u.lift()));
        nodes.extend(self.people.iter().map(|p| p.lift()));
        nodes.extend(self.locations.iter().map(|l| l.lift()));
        nodes
    }
}
```

### 5. Fold Organization by Pipeline Role (FRP)

Folds are natural transformations organized by WHERE they execute:

```
src/folds/
├── view/           # Execute in view() - pure rendering
│   ├── visualization.rs    # FoldVisualization → visual properties
│   └── detail_panel.rs     # FoldDetailPanel → panel content
│
├── query/          # Execute for queries - pure selection
│   ├── searchable.rs       # FoldSearchableText → search index
│   └── filter.rs           # FoldFilterable → filter predicates
│
└── update/         # Execute in update() - produce commands
    └── intent.rs           # FoldIntent → domain commands
```

### 6. Compositional Routing (FRP A6)

Replace pattern matching with compositional signal functions:

```rust
// Instead of:
match node {
    DomainNode::Organization(o) => handle_org(o),
    DomainNode::Person(p) => handle_person(p),
    // ... 23 more cases
}

// Use compositional routing:
type Router<A, B> = Box<dyn Fn(A) -> B>;

fn route_by_context<A, B>(
    org_handler: Router<OrganizationEntity, B>,
    pki_handler: Router<PkiEntity, B>,
    nats_handler: Router<NatsEntity, B>,
    yubikey_handler: Router<YubiKeyEntity, B>,
) -> Router<LiftedNode, B> {
    Box::new(move |node| match node {
        LiftedNode::Organization(e) => org_handler(e.unlift()),
        LiftedNode::Pki(e) => pki_handler(e.unlift()),
        LiftedNode::Nats(e) => nats_handler(e.unlift()),
        LiftedNode::YubiKey(e) => yubikey_handler(e.unlift()),
    })
}
```

This reduces O(25) pattern cases to O(4) with composed handlers.

## Rationale

### Category Theory (ACT Expert)

> "The coproduct structure (carrier + injections + fold) is the DEFINITION of the type.
> Splitting them breaks the categorical semantics. However, fold IMPLEMENTATIONS are
> natural transformations - those ARE separable."

The FoldDomainNode trait (universal property) stays with the coproduct.
Concrete fold implementations (FoldVisualization, etc.) are natural transformations
that can live separately, colocated with their output types.

### Domain-Driven Design (DDD Expert)

> "Your 'god coproduct' crossing bounded context boundaries IS an anti-pattern.
> Each bounded context should have its OWN coproduct... The graph visualization
> is a SEPARATE context that uses a COMPOSITION coproduct."

Bounded contexts must not share types. The visualization context imports LIFTED
representations via the LiftableDomain trait, never raw domain types.

### Functional Reactive Programming (FRP Expert)

> "A6 - Explicit Routing: Signal routing uses compositional primitives (>>>, ***, &&&)
> not pattern matching. Pattern-matching dispatch is an OOP anti-pattern."

Pattern matching for dispatch violates compositional FRP. Use signal function
composition where possible, falling back to minimal context-level routing.

## Consequences

### Positive

1. **Bounded Context Isolation**: Each context compiles independently
2. **Reduced Fold Explosion**: O(4) context handlers vs O(25) entity handlers
3. **File Size Compliance**: Each context file stays under 10K tokens
4. **Compositional Routing**: Signal functions compose algebraically
5. **Aggregate Clarity**: Aggregates are clearly compositions, not atoms
6. **Testability**: Contexts test in isolation

### Negative

1. **Migration Effort**: Significant refactoring of existing code
2. **Trait Complexity**: LiftableDomain adds indirection
3. **Performance**: Dynamic dispatch for lifted nodes (mitigated by boxing)

### Neutral

1. **File Count**: More files, but each is smaller and focused
2. **Import Structure**: More explicit imports between contexts

## Implementation Plan

### Sprint 1: Context Separation
- Create `src/domains/` directory structure
- Move entity types into per-context modules
- Define per-context coproducts (OrganizationEntity, PkiEntity, etc.)
- Verify bounded context isolation (no cross-imports)

### Sprint 2: Lifting Infrastructure
- Define `LiftableDomain` trait
- Implement `LiftedNode` composition coproduct
- Implement `lift()` for each domain type
- Implement `unlift()` for round-tripping

### Sprint 3: Fold Migration
- Create `src/folds/` directory structure
- Migrate FoldVisualization to folds/view/
- Migrate FoldDetailPanel to folds/view/
- Migrate FoldSearchableText to folds/query/

### Sprint 4: Aggregate Refactoring
- Remove aggregate variants from entity coproducts
- Create aggregate composition types
- Implement `project()` for aggregate visualization
- Update GUI to use aggregate projections

### Sprint 5: Compositional Routing
- Replace entity-level pattern matching with context routing
- Implement signal function composition primitives
- Verify FRP A6 compliance

### Sprint 6: Cleanup and Documentation
- Remove deprecated DomainNode type
- Update all imports
- Write retrospective
- Update CLAUDE.md with new patterns

## References

- [N-ARY FRP AXIOMS](../N_ARY_FRP_AXIOMS.md)
- [Category Theory Foundations](../../.claude/agents/act-expert.md)
- [DDD Patterns](../../.claude/agents/ddd-expert.md)
- [FRP Expert](../../.claude/agents/frp-expert.md)
