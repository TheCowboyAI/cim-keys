# Sprint 2 Retrospective: Rename Graph → Concept

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

**Sprint Duration**: 2025-12-29
**Status**: Completed

---

## Summary

Sprint 2 renamed all "Graph" terminology to "Concept" terminology per the DDD expert's recommendation. This was a purely mechanical refactoring that touched 26 files with 367 total reference updates.

---

## What Went Well

### 1. Clean Bulk Replacement
Using `sed` with find allowed efficient replacement across all files:
```bash
find src -name "*.rs" -exec sed -i 's/OrganizationGraph/OrganizationConcept/g' {} \;
find src -name "*.rs" -exec sed -i 's/GraphNode/ConceptEntity/g' {} \;
find src -name "*.rs" -exec sed -i 's/GraphEdge/ConceptRelation/g' {} \;
find src -name "*.rs" -exec sed -i 's/GraphMessage/OrganizationIntent/g' {} \;
```

### 2. No Compilation Issues
All renames were pure identifier changes - no structural modifications needed.

### 3. Verified Completeness
Post-rename grep confirmed 0 occurrences of old names:
- OrganizationGraph: 0 (was 146)
- GraphNode: 0 (was 93)
- GraphEdge: 0 (was 21)
- GraphMessage: 0 (was 107)

### 4. All Tests Pass
All 269 tests continue to pass after renaming.

---

## What Could Be Improved

### 1. File Renaming Deferred
Original plan included renaming files:
- `graph.rs` → `concept.rs`
- `graph_*.rs` → `concept_*.rs`

This was deferred to avoid breaking module imports. The type names are more important for domain understanding than file names.

### 2. Documentation Comments
Some doc comments still reference "graph" in lowercase (describing graph theory concepts, which is appropriate). These weren't changed because they're describing implementation details, not domain language.

---

## Key Decisions Made

1. **Type Names Over File Names**: Prioritized renaming type identifiers over file names for stability.

2. **Kept Implementation Terms**: Terms like "graph visualization" in comments were kept since they describe technical implementation, not domain concepts.

3. **Ubiquitous Language Applied**:
   - "Concept" instead of "Graph" - domain experts think in concepts
   - "Entity" instead of "Node" - what exists in the concept space
   - "Relation" instead of "Edge" - how entities relate
   - "Intent" instead of "Message" - what actors intend to do

---

## Metrics

| Metric | Sprint 1 End | Sprint 2 End |
|--------|--------------|--------------|
| DDD Validation Checks | 1/5 | 2/5 |
| Old "Graph" terminology | 367 refs | 0 refs |
| New "Concept" terminology | 0 refs | 367 refs |
| Files modified | - | 26 |
| Tests passing | 269 | 269 |

---

## Technical Details

### Renaming Map
| Old Name | New Name | Count |
|----------|----------|-------|
| OrganizationGraph | OrganizationConcept | 146 |
| GraphNode | ConceptEntity | 93 |
| GraphEdge | ConceptRelation | 21 |
| GraphMessage | OrganizationIntent | 107 |
| **Total** | | **367** |

### Files Modified
```
src/gui.rs
src/gui/graph.rs
src/gui/graph_*.rs (14 files)
src/gui/frp_*.rs (3 files)
src/gui/workflows.rs
src/gui/routing.rs
src/gui/edge_indicator.rs
src/gui/event_emitter.rs
src/gui/gui_frp_integration.rs
src/mvi/intent.rs
src/mvi/update.rs
src/gui/FRP_GUIDE.md
src/gui/FRP_IMPLEMENTATION_SUMMARY.md
```

---

## DDD Expert Rationale

From Sprint 0 DDD expert consultation:

> "OrganizationGraph → OrganizationConcept (Graph is implementation, not domain)"
> "GraphNode → ConceptEntity (Technical term, not ubiquitous language)"
> "GraphMessage → OrganizationIntent (conflates UI events with domain ops)"

The key insight is that domain experts don't think in terms of "graphs" and "nodes" - they think in terms of "concepts", "entities", and "relations". Using domain language in the code makes it easier for domain experts to understand and validate the system.

---

## Lessons Learned

1. **Bulk Refactoring is Safe**: With good test coverage, mechanical renames are low risk.

2. **sed + find is Powerful**: For cross-file renames, shell tools are more efficient than IDE refactoring.

3. **Verify with grep**: Always verify renames completed by checking for 0 occurrences of old names.

4. **Tests are Safety Net**: 269 passing tests gave confidence the rename didn't break anything.

---

## Next Sprint (Sprint 3)

**Goal**: Remove NodeType Enum - Replace with proper domain entity typing

**Key Tasks**:
- Analyze all NodeType usages
- Create ConceptMember enum that wraps EntityIds
- Implement GraphRenderable trait
- Have domain types implement GraphRenderable
- Remove NodeType enum

This is the most significant architectural change - moving from a "type tag" approach to proper polymorphism.

---

**Retrospective Author**: Claude Code
**Date**: 2025-12-29
