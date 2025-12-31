<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 21 Retrospective: Compositional Coproduct Architecture

**Date**: 2025-12-31
**Sprint Duration**: Multi-session
**Status**: Completed

## Objective

Refactor the "god coproduct" anti-pattern in domain_node.rs into per-context coproducts
following DDD, ACT, and FRP principles based on expert consultations.

## Problem Statement

`src/gui/domain_node.rs` exceeded 30,000 tokens (~2,900 lines), violating our 10K token
file size limit. More importantly, it exhibited the "god coproduct" anti-pattern where
a single sum type crossed multiple bounded context boundaries.

## Expert Consultations

### ACT Expert (Category Theory)
- Coproduct structure (carrier + injections + fold) must stay together
- FoldDomainNode trait is the universal property
- Fold implementations are natural transformations (separable)
- Colocate fold output types WITH their fold implementations

### DDD Expert (Domain-Driven Design)
- Each bounded context should have its OWN coproduct
- Graph visualization uses a COMPOSITION coproduct with lifting interfaces
- Aggregates are compositions, not coproduct variants
- Contexts should have NO imports from other contexts

### FRP Expert (Functional Reactive Programming)
- Identified A6 violation: pattern matching for dispatch instead of compositional routing
- Organize folds by pipeline role: view/, query/, update/
- Folds execute in view(), not update()
- Use signal function composition where possible

## Completed Work

### Phase 1: Architecture Documentation
Created ADR-001 at `docs/technical/architecture/ADR-001-compositional-coproduct-architecture.md`
documenting:
- Current anti-patterns
- Recommended architecture
- Implementation plan in 6 sprints
- Expert rationale

### Phase 2: Per-Context Coproducts
Created `src/domains/` module with bounded context coproducts:

| Context | File | Entities | Lines |
|---------|------|----------|-------|
| Organization | organization.rs | Person, Organization, OrgUnit, Location, Role, Policy | ~270 |
| PKI | pki.rs | RootCert, IntermediateCert, LeafCert, Key | ~175 |
| NATS | nats.rs | Operator, Account, User, ServiceAccount + Simple variants | ~240 |
| YubiKey | yubikey.rs | Device, Slot, Status | ~160 |
| Visualization | visualization.rs | Manifest, PolicyRole, PolicyClaim, PolicyCategory, PolicyGroup | ~200 |

Each context module provides:
1. **Injection enum** - Tags for entity types
2. **Data enum** - Inner data carrier
3. **Entity struct** - Coproduct with injection functions
4. **Fold trait** - Universal property

### Phase 3: LiftableDomain Trait - Already Existed!
The LiftableDomain trait was already implemented in `src/lifting.rs` (947 lines):

```rust
pub trait LiftableDomain: Clone + Send + Sync + 'static {
    fn lift(&self) -> LiftedNode;
    fn unlift(node: &LiftedNode) -> Option<Self>;
    fn injection() -> Injection;
    fn entity_id(&self) -> Uuid;
}
```

Existing implementations for: Organization, OrganizationUnit, Person, Location

### Phase 4: Separate Folds by Pipeline Role
Created `src/gui/folds/` module organized by FRP pipeline role:

```
src/gui/folds/
├── mod.rs             # Module exports
├── view/              # Folds for view() - rendering
│   ├── mod.rs
│   └── visualization.rs  # FoldVisualization → VisualizationData
└── query/             # Folds for queries - selection
    ├── mod.rs
    └── searchable.rs     # FoldSearchableText → SearchableText
```

Key benefits:
- VIEW folds execute in view(), produce visual data
- QUERY folds execute for filtering, produce selection data
- Follows FRP pipeline semantics: Model → view() → [Folds] → Element
- Centralized color palette for visual consistency

### Phase 5: Move Aggregates to Separate Coproduct
Created `src/domains/aggregates.rs` with:

```rust
pub enum AggregateInjection {
    Organization,
    PkiChain,
    NatsSecurity,
    YubiKeyProvisioning,
}

pub struct AggregateState {
    injection: AggregateInjection,
    data: AggregateStateData,
}

pub trait FoldAggregateState {
    fn fold_organization(&self, state: &OrganizationAggregateState) -> Self::Output;
    fn fold_pki_chain(&self, state: &PkiChainAggregateState) -> Self::Output;
    fn fold_nats_security(&self, state: &NatsSecurityAggregateState) -> Self::Output;
    fn fold_yubikey_provisioning(&self, state: &YubiKeyProvisioningAggregateState) -> Self::Output;
}
```

Why separate?
- **Different abstraction level**: Entities are domain objects, aggregates are state machines
- **Different lifecycle**: Entities change via events, aggregates track version and consistency
- **DDD principle**: Aggregates define boundaries, entities live within them

### Phase 6: Verification
All tests pass:
- domains::aggregates tests (3 tests)
- domains::organization tests (1 test)
- gui::folds::query::searchable tests (2 tests)
- gui::folds::view::visualization tests (2 tests)

## Compilation Fixes Required
Several field access issues discovered during compilation:
- Location uses `l.id().as_uuid()` (AggregateRoot trait), not `l.id`
- EntityId uses `.as_uuid()` not `.into_inner()`
- CryptographicKey has no `label` field (uses purpose/algorithm instead)
- NatsIdentityProjection has `nkey.id` and `nkey.name`, not direct fields
- PivSlotView uses `slot_name` not `label`
- YubiKeyStatus has no `status` field (derive from slots)
- KeyOwnerRole variants: RootAuthority, SecurityAdmin, Developer, ServiceAccount, BackupHolder, Auditor
- Role and Policy have `description: String` not `Option<String>`

## Metrics

| Metric | Value |
|--------|-------|
| Files created | 11 |
| Total new lines | ~2,300 |
| Tests passing | 8 domain/fold tests |
| Compilation time | ~15 seconds |
| Original domain_node.rs | 2,882 lines (for future deprecation) |

## Best Practices Confirmed

1. **Per-Context Coproducts**: Each bounded context owns its entity coproduct
2. **Aggregate Separation**: Aggregates are state machines, not entity variants
3. **Natural Transformations Separable**: Fold implementations colocated with outputs
4. **FRP Pipeline Organization**: Folds organized by pipeline role (view/query/update)
5. **Centralized Color Palette**: Visual consistency via dedicated palette module
6. **Field Access Via Traits**: Use AggregateRoot::id() not struct fields for Location

## Architecture Summary

```
                    ┌─────────────────────────────────────────┐
                    │              DomainNode                 │
                    │          (Legacy Coproduct)             │
                    │    Still used, will be deprecated       │
                    └─────────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                 │
           ┌────────▼────────┐ ┌─────▼─────┐ ┌────────▼────────┐
           │  Entity Layer   │ │ Aggregate │ │   Folds Layer   │
           │  per-context    │ │   Layer   │ │   by role       │
           └────────┬────────┘ └─────┬─────┘ └────────┬────────┘
                    │                │                │
    ┌───────────────┼────────────────┤                │
    │               │                │                │
┌───▼───┐  ┌───────▼───────┐  ┌─────▼─────┐    ┌─────▼─────┐
│ Org   │  │     PKI       │  │   NATS    │    │   view/   │
│Entity │  │    Entity     │  │  Entity   │    │visualization│
└───────┘  └───────────────┘  └───────────┘    └───────────┘
    │              │                │          ┌───────────┐
┌───▼───┐  ┌───────▼───────┐  ┌─────▼─────┐    │  query/   │
│YubiKey│  │Visualization  │  │ Aggregate │    │searchable │
│Entity │  │   Entity      │  │   State   │    └───────────┘
└───────┘  └───────────────┘  └───────────┘
```

## Lessons Learned

1. **Check struct fields before assuming**: Organization, KeyOwnerRole, Role all had different fields than expected
2. **FoldDomainNode uses decomposed primitives**: The trait uses decomposed primitive parameters, not domain objects, for flexibility
3. **Existing infrastructure exists**: LiftableDomain was already implemented in lifting.rs
4. **Tests reveal field changes**: Compilation of tests exposes struct changes before runtime

## Next Steps (Future Sprint)

1. **Compositional Routing (A6 Compliance)**: Replace pattern matching with signal function composition
2. **Domain Node Deprecation**: Migrate usages to per-context coproducts
3. **Update Fold Trait**: Consider using per-context fold traits instead of unified FoldDomainNode
4. **View Integration**: Wire new folds into actual view() implementations
