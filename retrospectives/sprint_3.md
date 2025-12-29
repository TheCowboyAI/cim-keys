# Sprint 3 Retrospective: DomainNode Coproduct

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

**Sprint Duration**: 2025-12-29
**Status**: Phase 1 Complete (Foundation)

---

## Summary

Sprint 3 introduced the categorical coproduct pattern to replace the flat `NodeType` enum. Following Applied Category Theory (ACT) principles, we created `DomainNode` with proper injection functions and a `FoldDomainNode` trait implementing the universal property.

This is a **foundation sprint** - the coproduct infrastructure is in place, but the full migration of 473 NodeType usages will proceed incrementally.

---

## What Went Well

### 1. Categorical Design Implemented
Created a proper coproduct satisfying the universal property:
```rust
// Injection: Person → DomainNode
let node = DomainNode::inject_person(person, role);

// Universal property: fold to any type X
let viz = node.fold(&FoldVisualization);
let injection = node.fold(&FoldInjection);
```

### 2. Separation of Concerns
The `FoldVisualization` cleanly separates rendering from domain:
- Domain nodes know nothing about colors/icons
- Visualization is computed via fold, not embedded in domain

### 3. Gradual Migration Enabled
Added `from_node_type()` conversion function enabling incremental migration:
```rust
let domain_node = DomainNode::from_node_type(&node_type);
```

### 4. All Tests Pass
271 tests pass - no regressions from the new infrastructure.

### 5. Terminology Improved
Per user feedback, renamed `SeparationClassGroup` → `PolicyGroup` for clearer domain language.

---

## What Could Be Improved

### 1. Large Migration Scope
With 473 usages of `NodeType::` across 17 files, full migration was deferred. The coproduct is ready, but adoption will be incremental.

### 2. NodeType Still Exists
Original plan was to remove `NodeType` entirely. Pragmatic decision: keep both temporarily for gradual migration.

### 3. Missing Property Tests
Coproduct laws should be verified with property-based tests:
- `fold(F) ∘ inject_A(a) = F.fold_A(a)` (universal property)
- Injection is injective (no two different inputs map to same output)

---

## Key Decisions Made

1. **Foundation First**: Created the coproduct infrastructure before migration.

2. **Conversion Over Replacement**: Added `from_node_type()` instead of forcing big-bang migration.

3. **FoldVisualization Pattern**: Separated visual concerns from domain using folder abstraction.

4. **Terminology per User Feedback**: Renamed `SeparationClassGroup` → `PolicyGroup` immediately.

---

## Metrics

| Metric | Sprint 2 End | Sprint 3 End |
|--------|--------------|--------------|
| DDD Validation Checks | 2/5 | 3/5 |
| NodeType variants | 25 | 25 (unchanged) |
| DomainNode injections | 0 | 25 |
| FoldDomainNode methods | 0 | 25 |
| Tests passing | 269 | 271 |
| New module (domain_node.rs) | - | ~1,400 LOC |

---

## Technical Details

### New Types Created

| Type | Purpose |
|------|---------|
| `DomainNode` | Coproduct container with injection tag |
| `DomainNodeData` | Inner data for each variant |
| `Injection` | Tag identifying which domain type was injected |
| `FoldDomainNode` | Trait for universal property (25 methods) |
| `VisualizationData` | Output of visualization folder |
| `FoldVisualization` | Folder producing rendering data |

### Files Modified

| File | Change |
|------|--------|
| `src/gui/domain_node.rs` | NEW - Coproduct implementation |
| `src/gui.rs` | Added domain_node module |
| `src/gui/graph.rs` | Renamed PolicyGroup |
| `src/gui/property_card.rs` | Renamed PolicyGroup |
| `src/icons.rs` | Added new icons (KEY, USB, MEMORY, etc.) |

---

## ACT Expert Rationale

From Sprint 0 ACT expert consultation:

> "The current `NodeType` enum violates categorical principles because it loses identity and doesn't preserve injections. We replace it with a proper coproduct that satisfies the universal property."

The key insight is that a coproduct A + B + C has:
- **Injection morphisms**: ι_A: A → Sum, ι_B: B → Sum
- **Universal property**: For any X with morphisms from components, there's a unique morphism from Sum to X

`FoldDomainNode` captures this: implementors define morphisms from each component, and `fold()` is the induced morphism from the coproduct.

---

## Migration Path (Future Work)

### Phase 2: ConceptEntity Migration
Replace `node_type: NodeType` with `domain_node: DomainNode`:
```rust
pub struct ConceptEntity {
    pub id: Uuid,
    pub domain_node: DomainNode,  // Was: node_type: NodeType
    pub position: Point,
    // color and label derived from fold()
}
```

### Phase 3: Rendering Migration
Replace match on NodeType with fold pattern:
```rust
// Old
match &node.node_type {
    NodeType::Person { person, role } => { ... }
    // 25 more arms
}

// New
let viz = node.domain_node.fold(&FoldVisualization);
let color = viz.color;
let label = viz.primary_text;
```

### Phase 4: Remove NodeType
Once all usages migrated, delete `NodeType` enum.

---

## Lessons Learned

1. **Categorical Patterns Work**: The coproduct structure cleanly separates domain from rendering.

2. **Gradual Migration is Pragmatic**: With 473 usages, conversion functions enable incremental adoption.

3. **User Feedback is Valuable**: Immediate terminology improvement (PolicyGroup) shows value of collaboration.

4. **Foundation Before Migration**: Having the coproduct ready makes future migration straightforward.

---

## Next Sprint (Sprint 4)

**Goal**: Implement MVI Intent Layer - Categorize messages by origin

**Key Tasks**:
- Create `src/gui/intent.rs`
- Define `Intent` enum with categories (Ui, Port, Domain, System)
- Migrate `Message` variants to appropriate Intent types
- Update `update` function to route intents

This continues the architectural cleanup before tackling the large NodeType migration.

---

**Retrospective Author**: Claude Code
**Date**: 2025-12-29
