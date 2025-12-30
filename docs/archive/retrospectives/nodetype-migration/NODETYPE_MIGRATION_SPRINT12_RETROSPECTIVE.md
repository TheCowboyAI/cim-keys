<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 12 Retrospective: Migrate GraphEvent and Node Creation to DomainNode

## Goal
Migrate GraphEvent structure and all node creation patterns from NodeType to DomainNode, eliminating NodeType from the event sourcing layer.

## Completed

### Step 1: Analyze GraphEvent Structure
Identified that `GraphEvent` enum used `NodeType` in three variants:
- `NodeCreated { node_type: NodeType, ... }`
- `NodeDeleted { node_type: NodeType, ... }`
- `NodePropertiesChanged { old_node_type: NodeType, new_node_type: NodeType, ... }`

### Step 2: Migrate GraphEvent to DomainNode

**Before:**
```rust
pub enum GraphEvent {
    NodeCreated {
        node_id: Uuid,
        node_type: NodeType,
        position: Point,
        color: Color,
        label: String,
        timestamp: DateTime<Utc>,
    },
    NodeDeleted {
        node_id: Uuid,
        node_type: NodeType,
        ...
    },
    NodePropertiesChanged {
        node_id: Uuid,
        old_node_type: NodeType,
        new_node_type: NodeType,
        ...
    },
}
```

**After:**
```rust
pub enum GraphEvent {
    NodeCreated {
        node_id: Uuid,
        domain_node: DomainNode,  // Categorical coproduct pattern
        position: Point,
        color: Color,
        label: String,
        timestamp: DateTime<Utc>,
    },
    NodeDeleted {
        node_id: Uuid,
        domain_node: DomainNode,
        ...
    },
    NodePropertiesChanged {
        node_id: Uuid,
        old_domain_node: DomainNode,
        new_domain_node: DomainNode,
        ...
    },
}
```

### Step 3: Migrate apply_event Handler

**Before:**
```rust
GraphEvent::NodeCreated { node_id, node_type, position, color, label, .. } => {
    let mut node = ConceptEntity::from_node_type(*node_id, node_type.clone(), *position);
    node.color = *color;
    node.label = label.clone();
    self.nodes.insert(*node_id, node);
}
```

**After:**
```rust
GraphEvent::NodeCreated { node_id, domain_node, position, color, label, .. } => {
    // Use from_domain_node - the preferred constructor
    let mut node = ConceptEntity::from_domain_node(*node_id, domain_node.clone(), *position);
    node.color = *color;
    node.label = label.clone();
    self.nodes.insert(*node_id, node);
}
```

### Step 4: Migrate All Node Creation Patterns

Migrated 16 node creation sites to use `DomainNode::inject_*` methods:

**Pattern 1: OrganizationalNodeType match (6 patterns)**
```rust
// Before
let node_type = NodeType::Organization(org);
GraphEvent::NodeCreated { node_type, ... }

// After
let domain_node = domain_node::DomainNode::inject_organization(org);
GraphEvent::NodeCreated { domain_node, ... }
```

**Pattern 2: CanvasClicked handler (3 patterns)**
```rust
// Before
let (graph_node_type, label, color) = match node_type_str.as_str() {
    "Person" => (NodeType::Person { person, role }, ...)
};

// After
let (domain_node, label, color) = match node_type_str.as_str() {
    "Person" => (DomainNode::inject_person(person, role), ...)
};
```

**Pattern 3: Context Menu CreateNode (6 patterns)**
```rust
// Before
let (graph_node_type, label, color) = match node_type {
    NodeCreationType::Organization => (NodeType::Organization(org), ...)
};

// After
let (domain_node, label, color) = match node_type {
    NodeCreationType::Organization => (DomainNode::inject_organization(org), ...)
};
```

**Pattern 4: NodeDeleted event (1 pattern)**
```rust
// Before
GraphEvent::NodeDeleted { node_type: node.node_type.clone(), ... }

// After
GraphEvent::NodeDeleted { domain_node: node.domain_node.clone(), ... }
```

### Step 5: Migrate PropertyChanged Handlers

Two NodePropertiesChanged event creation sites updated:
- InlineEditSubmit handler
- PropertyCard Save handler

Both now use:
```rust
let old_domain_node = node.domain_node.clone();
// ... mutation logic still uses NodeType (Sprint 13 work) ...
let new_domain_node = DomainNode::from_node_type(&new_node_type);

GraphEvent::NodePropertiesChanged {
    old_domain_node,
    new_domain_node,
    ...
}
```

### Step 6: Migrate Event Extraction Pattern

Pattern that extracted data from NodeCreated event updated to use DomainNode accessors:

```rust
// Before
if let GraphEvent::NodeCreated { node_type, .. } = &event {
    match node_type {
        NodeType::Organization(org) => { ... }
        NodeType::Person { person, .. } => { ... }
    }
}

// After
if let GraphEvent::NodeCreated { domain_node, .. } = &event {
    if let Some(org) = domain_node.organization() { ... }
    else if let Some(person) = domain_node.person() { ... }
}
```

### Step 7: Update Tests

Updated 4 tests in `graph_events.rs` to use DomainNode:
- `test_event_stack_push`
- `test_event_compensation`
- `test_undo_redo`
- `test_max_size`

## Metrics

| Metric | Before Sprint 12 | After Sprint 12 |
|--------|------------------|-----------------|
| NodeType usages in gui.rs | 96 | 96* |
| node_type field usages in gui.rs | N/A | 52 |
| domain_node usages in gui.rs | N/A | 50 |
| GraphEvent uses NodeType | Yes | No |
| Tests passing | 26 | 26 |

*Note: NodeType enum usages remain at 96 because PropertyChanged mutation logic still uses NodeType pattern matching (Sprint 13 work).

### Files Modified

| File | Changes |
|------|---------|
| src/gui/graph_events.rs | Replaced NodeType with DomainNode in 3 event variants, updated compensate() method |
| src/gui/graph.rs | Updated apply_event() to use from_domain_node() |
| src/gui.rs | Migrated 16 node creation patterns to use inject_* methods |

## Architecture Benefits

### 1. Event Sourcing Now Uses Categorical Coproduct
The GraphEvent structure now stores DomainNode directly:
- Events carry the canonical representation
- No NodeType↔DomainNode conversion in event handlers
- apply_event uses `from_domain_node()` directly

### 2. Clean Node Creation Pattern
```rust
// New pattern: Create DomainNode first
let domain_node = DomainNode::inject_organization(org);

GraphEvent::NodeCreated {
    node_id,
    domain_node,  // Canonical representation
    position,
    color,
    label,
    timestamp: Utc::now(),
}
```

### 3. Accessor-Based Event Data Extraction
```rust
// Use DomainNode accessors instead of NodeType matching
if let Some(org) = domain_node.organization() {
    // Work with organization data
}
```

## Code Quality

```
Compilation: ✅ Clean (deprecation warnings expected)
Tests: ✅ 26 passed, 47 ignored, 0 failed
GraphEvent: ✅ Uses DomainNode
Node creation: ✅ Uses inject_* methods
```

## Remaining Work

### PropertyChanged Mutation Logic (Sprint 13)
The mutation logic in PropertyChanged handlers still uses NodeType pattern matching:
```rust
let new_node_type = match &node.node_type {
    graph::NodeType::Organization(org) => {
        let mut updated = org.clone();
        updated.name = new_name.clone();
        graph::NodeType::Organization(updated)
    }
    // ... 30+ other match arms
};
// Convert to DomainNode for event
let new_domain_node = DomainNode::from_node_type(&new_node_type);
```

This requires:
1. Adding mutation methods to DomainNode (e.g., `with_name()`, `with_email()`)
2. Or creating a `MutateDomainNode` pattern
3. This is the largest remaining NodeType usage (31 match arms)

## Future Sprints

### Sprint 13: Migrate PropertyChanged Mutation Logic
- Add setter/builder methods on DomainNode
- Replace NodeType pattern matching in property updates
- Target: Eliminate 31 NodeType usages

### Sprint 14: Clean Up Remaining Patterns
- Migrate remaining 41 read-only matches to folds/accessors
- Document any intentional NodeType usages
- Target: Reduce to < 10 NodeType usages

### Sprint 15: Remove NodeType Enum
- Delete NodeType enum
- Remove `from_node_type()` and `to_node_type()` converters
- Clean up all deprecation warnings
- Target: 0 NodeType usages

## Lessons Learned

1. **GraphEvent is Not Serialized**: Since GraphEvent doesn't implement Serialize/Deserialize, we could safely change its structure without backward compatibility concerns.

2. **Event Sourcing Migration**: Changing the event structure required updating all creation sites, the compensate() method, and the apply_event handler.

3. **Transitional Patterns Work**: Using `DomainNode::from_node_type()` in PropertyChanged handlers provides a clean transition path - events use DomainNode while mutation logic is migrated separately.

4. **Accessor Pattern for Events**: Using `domain_node.organization()` and `domain_node.person()` accessors for event data extraction is cleaner than `match node_type { ... }`.

## Summary

Sprint 12 successfully migrated the GraphEvent structure from NodeType to DomainNode:
- GraphEvent variants now use `domain_node` field instead of `node_type`
- All 16 node creation sites use `DomainNode::inject_*` methods
- apply_event uses `from_domain_node()` constructor
- Tests updated and passing

This is a significant architectural change that establishes DomainNode as the canonical representation in the event sourcing layer. The remaining NodeType usages are concentrated in PropertyChanged mutation logic (31 usages) and other read-only patterns (41 usages), planned for Sprints 13-14.
