# Graph Architecture Refactoring Plan

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

**Date:** 2026-01-03
**Status:** ✅ COMPLETED
**Based On:** Expert consultations (ACT, CIM, Graph, Explore agents)
**Completed:** 2026-01-03 (Sprints 31-33)

## Problem Statement

The current cim-keys implementation uses a 29-arm `FoldDomainNode` pattern that:
1. Embeds morphisms as CODE branches instead of DATA
2. Treats entities as containers for values (OOP thinking)
3. Downcasts to concrete types during graph traversal
4. Violates the Kan extension pattern that cim-domain provides

## Correct Architecture (From Expert Consultations)

### The Three Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                    GRAPH LAYER (Abstract)                       │
│  - Nodes are UUIDs + opaque payloads (type-erased)              │
│  - Edges are triples (source_id, relation, target_id)           │
│  - Graph algorithms work HERE (path, cycles, SCC, topo sort)    │
│  - Returns UUIDs, never concrete types                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │  Kan Extension (Lan_K F)
                              │  lift() only when needed
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    DOMAIN LAYER (Concrete)                      │
│  - Person, Organization, Key, Certificate, etc.                 │
│  - Aggregates with state machines                               │
│  - Domain-specific operations (validate, sign, apply_policy)    │
└─────────────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Graph operations return UUIDs** - `reachable_from()`, `shortest_path()`, `find_cycles()` return `Vec<Uuid>`
2. **Lift only at boundaries** - When domain-specific behavior is needed
3. **Morphisms as DATA** - 29 fmap registrations, not 29 code branches
4. **Single fold operation** - Looks up morphism by injection tag
5. **Values are invariants** - Describe constraints, not embedded data

## Sprint Plan

### Sprint 31: Morphism Registry Foundation
**Goal:** Replace 29-arm FoldDomainNode with MorphismRegistry pattern

#### Tasks
- 31.1: Create `src/graph/morphism.rs` with MorphismRegistry type
- 31.2: Define Morphism trait for type-safe transformations
- 31.3: Implement registration of morphisms as DATA entries
- 31.4: Create single `fold()` operation that looks up morphisms by injection tag
- 31.5: Migrate visualization morphisms from FoldDomainNode to registry
- 31.6: Migrate search/query morphisms to registry
- 31.7: Add #[deprecated] to FoldDomainNode trait
- 31.8: Verify compilation and tests pass
- 31.9: Write retrospective

#### Acceptance Criteria
- [x] MorphismRegistry stores morphisms as DATA, not code
- [x] Single fold() operation replaces 29-arm match
- [x] FoldDomainNode marked deprecated (and removed)
- [x] All tests pass (561 tests)

### Sprint 32: Abstract Graph Operations ✅
**Goal:** Graph operations work on UUIDs, never lift during traversal

#### Tasks
- 32.1: ✅ Create `src/graph/abstract_ops.rs` with AbstractGraphOps
- 32.2: ✅ Implement `reachable_from(id) -> HashSet<Uuid>`
- 32.3: ✅ Implement `shortest_path(from, to) -> Option<Vec<Uuid>>`
- 32.4: ✅ Implement `find_roots() -> Vec<Uuid>`
- 32.5: ✅ Implement `find_leaves() -> Vec<Uuid>`
- 32.6: ✅ Implement `neighbors(id) -> Vec<Uuid>`
- 32.7: ✅ Implement `topological_sort()`, `find_cycles()`, `strongly_connected_components()`
- 32.8: ✅ FilteredGraphOps for edge-label-filtered traversal
- 32.9: ✅ Type signature proves no lifting during traversal
- 32.10: ✅ Write retrospective

#### Acceptance Criteria
- [x] Graph operations return UUIDs only
- [x] No lift() calls during traversal (type signature proof)
- [x] Graph algorithms work on abstract structure
- [x] All tests pass (581 tests)

### Sprint 33: Kan Extension Boundary ✅
**Goal:** Clear separation between graph layer and domain layer

#### Tasks
- 33.1: ✅ Define explicit "lift boundary" in architecture
- 33.2: ✅ Create `src/graph/lift.rs` with lifting utilities
- 33.3: ✅ Implement DeferredLift for lazy lifting
- 33.4: ✅ Implement BatchLift for efficient multi-node lifting
- 33.5: ✅ Document the Kan extension pattern in module docs
- 33.6: ✅ Add 10 tests verifying lift boundary behavior
- 33.7: ✅ FoldDomainNode already removed
- 33.8: ✅ Audit lift() calls - all at documented boundaries
- 33.9: ✅ Final verification - 591 tests pass
- 33.10: ✅ Write retrospective

#### Acceptance Criteria
- [x] Clear architectural boundary between graph and domain layers
- [x] Lift happens only when domain semantics needed
- [x] FoldDomainNode completely removed
- [x] Kan extension pattern documented
- [x] All tests pass (591 tests)

## Expert Consultation Summary

### ACT Expert (Categorical Foundations)
- 29-arm fold violates functoriality
- Morphisms should be DATA, not CODE branches
- Kan extension: `Lan_K F` extends domain functors to abstract graph nodes
- Registry pattern: register morphisms as data entries

### CIM Expert (Domain Usage)
- cim-domain already provides `LiftableDomain`, `LiftedGraph`, `LiftedNode`
- `FoldCapability` is pre-computed at lift time
- StateMachines drive all state transitions
- Aggregates = consistency boundaries, Sagas = aggregates of aggregates

### Graph Expert (Abstract Patterns)
- Graph is purely structural - doesn't care what's in nodes
- Edges are triples (source, relation, target)
- Graph algorithms (path, cycles, SCC, topo sort) work on abstracts
- Lift ONLY at boundary when domain semantics needed

### Explore Agent (cim-domain Analysis)
- `LiftedNode` has type erasure via `Arc<dyn Any + Send + Sync>`
- `Injection` enum tag for type dispatch
- `LiftedGraph` supports polymorphic `add<T>()` and `unlift_all<T>()`
- NO pattern matching at fold execution time

## Success Metrics

| Metric | Before | After |
|--------|--------|-------|
| FoldDomainNode arms | 29 | 0 (deleted) |
| Morphisms as CODE | 29 branches | 0 branches |
| Morphisms as DATA | 0 entries | 4+ registries |
| Lift calls during traversal | Many | 0 (type-enforced) |
| Graph ops returning concrete types | Many | 0 |
| Library tests | 550 | 591 |

## Implementation Summary

### Files Created
- `src/graph/mod.rs` - Module with architecture documentation
- `src/graph/morphism.rs` - MorphismRegistry, Morphism, LazyMorphism
- `src/graph/visualization.rs` - VisualizationRegistry
- `src/graph/detail_panel.rs` - DetailPanelRegistry
- `src/graph/abstract_ops.rs` - AbstractGraphOps, FilteredGraphOps
- `src/graph/lift.rs` - DeferredLift, BatchLift, LiftResult, LiftGuard

### Retrospectives
- `retrospectives/sprint_31.md`
- `retrospectives/sprint_32.md`
- `retrospectives/sprint_33.md`
