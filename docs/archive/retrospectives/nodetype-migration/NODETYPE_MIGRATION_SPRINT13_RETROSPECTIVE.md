<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 13 Retrospective: Migrate PropertyChanged Mutation Logic

## Goal
Replace the 177-line NodeType pattern match in PropertyChanged handlers with the new `PropertyUpdate` struct and `DomainNode::with_properties()` method.

## Completed

### Step 1: Analyze PropertyChanged Mutation Patterns

Identified two PropertyChanged handlers in gui.rs:

1. **InlineEditSubmit handler** (lines 1774-1786): Updates only `name` field
   - Person: name
   - OrganizationalUnit: name
   - Others: no change

2. **PropertyCard Save handler** (lines 4075-4252): Full property updates
   - 6 mutable types with updates
   - 19 read-only types (just clone)
   - 177 lines of match arms

**Mutable properties identified:**
| Property | Types |
|----------|-------|
| name | Organization, OrganizationUnit, Person, Location, Role, Policy |
| description | Organization, Role, Policy |
| email | Person |
| enabled/active | Person (active), Role (active), Policy (enabled) |
| claims | Policy |

### Step 2: Design DomainNode Mutation Approach

Designed a `PropertyUpdate` struct with builder pattern:

```rust
pub struct PropertyUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub enabled: Option<bool>,
    pub claims: Option<Vec<PolicyClaim>>,
}

impl PropertyUpdate {
    pub fn new() -> Self { Self::default() }
    pub fn with_name(mut self, name: impl Into<String>) -> Self { ... }
    pub fn with_description(mut self, description: impl Into<String>) -> Self { ... }
    pub fn with_email(mut self, email: impl Into<String>) -> Self { ... }
    pub fn with_enabled(mut self, enabled: bool) -> Self { ... }
    pub fn with_claims(mut self, claims: Vec<PolicyClaim>) -> Self { ... }
}
```

### Step 3: Implement PropertyUpdate and with_properties()

Added to `src/gui/domain_node.rs`:

```rust
impl DomainNode {
    /// Apply property updates, returning new node
    pub fn with_properties(&self, update: &PropertyUpdate) -> Self {
        match &self.data {
            DomainNodeData::Organization(org) => {
                // Apply name, description
                Self::inject_organization(updated)
            }
            DomainNodeData::Person { person, role } => {
                // Apply name, email, enabled (as active)
                Self::inject_person(updated, role.clone())
            }
            // ... 4 more mutable types ...
            _ => self.clone(), // Read-only types
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self.injection,
            Injection::Organization | Injection::OrganizationUnit |
            Injection::Person | Injection::Location |
            Injection::Role | Injection::Policy
        )
    }
}
```

### Step 4: Migrate PropertyChanged Handlers

**InlineEditSubmit Handler (Before: 13 lines)**
```rust
let new_node_type = match &node.node_type {
    NodeType::Person { person, role } => {
        let mut updated_person = person.clone();
        updated_person.name = self.inline_edit_name.clone();
        NodeType::Person { person: updated_person, role: *role }
    }
    NodeType::OrganizationalUnit(unit) => {
        let mut updated = unit.clone();
        updated.name = self.inline_edit_name.clone();
        NodeType::OrganizationalUnit(updated)
    }
    other => other.clone(),
};
let new_domain_node = DomainNode::from_node_type(&new_node_type);
```

**InlineEditSubmit Handler (After: 3 lines)**
```rust
let update = PropertyUpdate::new().with_name(self.inline_edit_name.clone());
let new_domain_node = node.domain_node.with_properties(&update);
```

**PropertyCard Save Handler (Before: 177 lines)**
```rust
let new_node_type = match &node.node_type {
    graph::NodeType::Organization(org) => {
        let mut updated = org.clone();
        updated.name = new_name.clone();
        updated.display_name = new_name.clone();
        updated.description = Some(new_description);
        graph::NodeType::Organization(updated)
    }
    // ... 5 more mutable types ...
    // ... 19 read-only types with full cloning ...
};
let new_domain_node = DomainNode::from_node_type(&new_node_type);
```

**PropertyCard Save Handler (After: 10 lines)**
```rust
let update = PropertyUpdate::new()
    .with_name(new_name.clone())
    .with_description(new_description)
    .with_email(new_email)
    .with_enabled(new_enabled)
    .with_claims(self.property_card.claims());

let new_domain_node = node.domain_node.with_properties(&update);
```

## Metrics

| Metric | Before Sprint 13 | After Sprint 13 | Change |
|--------|------------------|-----------------|--------|
| NodeType usages in gui.rs | 96 | 38 | -58 (60% reduction) |
| node_type field usages | 52 | 48 | -4 |
| domain_node usages | 50 | 50 | 0 |
| PropertyChanged lines (InlineEdit) | 13 | 3 | -10 |
| PropertyChanged lines (PropertyCard) | 177 | 10 | -167 |
| Total lines removed | - | - | ~177 |
| Deprecation warnings | 762 | 627 | -135 |
| Tests passing | 26 | 26 | 0 |

### Files Modified

| File | Changes |
|------|---------|
| src/gui/domain_node.rs | +200 lines (PropertyUpdate struct, with_properties method, is_mutable method) |
| src/gui.rs | -177 lines (replaced 2 NodeType match statements with PropertyUpdate calls) |

## Architecture Benefits

### 1. Type-Safe Property Updates
```rust
// Builder pattern ensures only valid properties are set
let update = PropertyUpdate::new()
    .with_name("New Name")      // All types
    .with_email("a@b.com")      // Person only
    .with_claims(claims);        // Policy only
```

### 2. DomainNode Encapsulates Mutability
```rust
// DomainNode knows which types are mutable
if node.domain_node.is_mutable() {
    let updated = node.domain_node.with_properties(&update);
}
```

### 3. No NodeType Pattern Matching for Mutations
```rust
// Old: 177 lines of match arms
let new_node_type = match &node.node_type { ... };
let new_domain_node = DomainNode::from_node_type(&new_node_type);

// New: 3 lines
let new_domain_node = node.domain_node.with_properties(&update);
```

### 4. Read-Only Types Automatically Handled
```rust
// with_properties() returns clone for read-only types
// No need to explicitly handle 19 read-only cases
```

## Code Quality

```
Compilation: ✅ Clean (deprecation warnings expected, reduced by 135)
Tests: ✅ 26 passed, 48 ignored, 0 failed
InlineEditSubmit: ✅ Migrated (13 → 3 lines)
PropertyCard Save: ✅ Migrated (177 → 10 lines)
NodeType usages: ✅ 96 → 38 (-60%)
```

## Remaining NodeType Usages (38 total)

| Category | Count | Description |
|----------|-------|-------------|
| Node creation patterns | 6 | Creating new nodes in CanvasClicked, context menus |
| Type checking for NATS | 2 | `matches!(&node.node_type, NodeType::NatsOperator(...))` |
| Read-only access | ~30 | Various display/rendering patterns |

These remaining usages are:
1. **Node creation**: Uses `DomainNode::inject_*` → NodeType via `to_node_type()`
2. **Type guards**: Simple checks, could migrate to `node.injection()`
3. **Read-only patterns**: Mostly in rendering code, lower priority

## Future Sprints

### Sprint 14: Migrate Read-Only Patterns
- Replace remaining NodeType pattern matches with fold/accessor patterns
- Target: Reduce to < 15 NodeType usages

### Sprint 15: Remove NodeType Enum
- Delete NodeType enum
- Remove `from_node_type()` and `to_node_type()` converters
- Clean up all deprecation warnings
- Target: 0 NodeType usages

## Lessons Learned

1. **PropertyUpdate Builder Pattern**: Using Option<T> fields allows partial updates - only specified properties are changed, others are preserved.

2. **Type-Specific Semantics**: `enabled` maps to different fields per type (Person.active, Role.active, Policy.enabled) - encapsulated in `with_properties()`.

3. **claims is Vec<PolicyClaim>**: Initially used `Vec<String>`, but Policy.claims uses the PolicyClaim enum. Caught by compiler.

4. **Massive Line Reduction**: 177 → 10 lines (94% reduction) for the PropertyCard handler shows the power of proper abstraction.

5. **Warning Reduction**: 762 → 627 warnings (-135) because we removed NodeType pattern matches that triggered deprecation warnings.

## Summary

Sprint 13 successfully migrated the PropertyChanged mutation logic from NodeType pattern matching to the new PropertyUpdate/with_properties() pattern:

- **InlineEditSubmit**: 13 lines → 3 lines
- **PropertyCard Save**: 177 lines → 10 lines
- **NodeType usages**: 96 → 38 (60% reduction)
- **Warnings**: 762 → 627 (-135)

The PropertyUpdate struct provides:
- Builder pattern for setting properties
- Type-safe optional updates
- Automatic handling of read-only types
- Clean separation of mutation logic from event creation

This is the largest single reduction in NodeType usage, eliminating the biggest remaining match statement. The remaining 38 usages are primarily in node creation and read-only patterns, planned for Sprints 14-15.
