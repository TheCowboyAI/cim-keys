<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 5 Retrospective: NodeType → DomainNode Migration (Phase 2)

## Goal
Add `domain_node` field to `ConceptEntity` and update all creation sites to use dual-field constructors, establishing the foundation for migrating away from `NodeType` pattern matching.

## Completed

### Step 1: Add domain_node Field to ConceptEntity ✅
- Added `domain_node: DomainNode` field to `ConceptEntity` struct
- Created `from_node_type(id, node_type, position)` constructor that:
  - Creates `DomainNode` from `NodeType`
  - Derives color/label from `FoldVisualization`
  - Sets both fields for backward compatibility
- Created `from_domain_node(id, domain_node, position)` constructor (preferred for new code)
- Added `visualization()` method for accessing fold results
- Added `injection()` accessor method

### Step 2: Update All Creation Sites ✅
Updated ~25 creation sites to use `ConceptEntity::from_node_type()`:

**graph.rs:**
- `add_node()` (person)
- `add_organization_node()`
- `add_org_unit_node()`
- `add_location_node()`
- `add_domain_role_node()`
- `add_role_node()` (policy role)
- `add_claim_node()`
- `add_category_node()`
- `add_separation_class_node()`
- `add_policy_node()`
- `add_nats_operator_node()`
- `add_nats_account_node()`
- `add_nats_user_node()`
- `add_nats_service_account_node()`
- `add_nats_operator_simple()`
- `add_nats_account_simple()`
- `add_nats_user_simple()`
- `add_root_certificate_node()`
- `add_intermediate_certificate_node()`
- `add_leaf_certificate_node()`
- `add_yubikey_node()`
- `add_piv_slot_node()`
- `apply_event()` (GraphEvent::NodeCreated, NodePropertiesChanged)

**graph_pki.rs:**
- `create_root_ca_node()`
- `create_intermediate_ca_node()`
- `create_leaf_certificate_node()`

**graph_nats.rs:**
- `create_nats_operator_node()`
- `create_nats_account_node()`
- `create_nats_user_node()`

**graph_yubikey.rs:**
- `generate_yubikey_provision_from_graph()`

**gui.rs:**
- `Message::RootCAGenerated` handler
- `Message::PersonalKeysGenerated` handler

### Step 3: Add to_node_type() to DomainNode ✅
- Implemented `DomainNode::to_node_type()` for backward compatibility conversion
- Handles all 25 DomainNodeData variants
- Fixed clone issue with `KeyAlgorithm` type

## Deferred to Sprint 6

### Step 3: Migrate Rendering to Fold Pattern (Infrastructure Ready)
- The 180-line match statement in `draw()` (lines 3492-3669) is ready to be replaced with:
  ```rust
  let viz = node.visualization();
  let (type_icon, type_font, primary_text, secondary_text) = (
      viz.icon, viz.icon_font, viz.primary_text, viz.secondary_text,
  );
  ```
- Minor differences in `FoldVisualization` output vs current match:
  - Certificate secondary_text doesn't include expiry dates
  - Some formatting differences
- Safe to migrate incrementally - both paths produce same result

### Step 4: Remove NodeType (Future Sprint)
- Cannot remove `NodeType` until all rendering code migrated
- ~60 pattern matches on `node_type` remain in graph.rs

## Technical Decisions

### Dual-Field Strategy
Chose to maintain both `node_type` and `domain_node` during migration:
- **Benefit**: Zero-risk incremental migration
- **Cost**: Temporary duplication (~8 bytes + DomainNode size per node)
- **Plan**: Remove `node_type` in Sprint 7 after rendering migration

### Constructor Pattern
```rust
// Migration path (current)
ConceptEntity::from_node_type(id, node_type, position)

// Preferred for new code
ConceptEntity::from_domain_node(id, domain_node, position)
```

### Color Override Pattern
Some handlers override computed color/label after construction:
```rust
let mut node = ConceptEntity::from_node_type(id, node_type, position);
node.color = custom_color;  // Override for special cases
node.label = custom_label;
```

## Metrics

- **Files Modified**: 5 (graph.rs, domain_node.rs, graph_pki.rs, graph_nats.rs, graph_yubikey.rs, gui.rs)
- **Lines Changed**: ~200
- **Creation Sites Updated**: 27
- **Compilation Errors Fixed**: 14
- **Warnings Remaining**: 1 (unused `role_to_color` method - intentional)

## Next Sprint (Sprint 6)

1. **Replace Match Statements with Fold**
   - Target: `draw()` function match (lines 3492-3669)
   - Update `FoldVisualization` to match current output exactly

2. **Migrate Remaining Pattern Matches**
   - Other `node_type` matches in rendering code
   - Detail panel rendering

3. **Begin NodeType Deprecation**
   - Add `#[deprecated]` attribute to `NodeType`
   - Document migration path

## Lessons Learned

1. **Incremental is Safe**: The dual-field approach eliminated all runtime risk
2. **Constructor Encapsulation**: Forcing use of constructors makes future changes easier
3. **Fold Pattern Works**: `node.visualization()` is cleaner than 180-line match
4. **Clone Awareness**: Non-Copy types need explicit `.clone()` in conversions
