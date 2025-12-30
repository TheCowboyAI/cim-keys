<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 14 Retrospective: Eliminate graph::NodeType from gui.rs

## Goal
Migrate remaining read-only patterns and reduce NodeType usages in gui.rs to < 15.

## Result
**EXCEEDED GOAL**: Achieved **0 `graph::NodeType` usages** in gui.rs!

## Completed

### Step 1: Analyze Remaining NodeType Usages

Starting position: 38 NodeType usages in gui.rs

Categorized into:
1. **Type-checking patterns** (4 patterns) - using NodeType in `matches!()` macros
2. **Unused import** (1) - `use crate::gui::graph::NodeType;`
3. **Certificate node creation** (2 patterns) - using `graph::NodeType::RootCertificate` etc.
4. **Data extraction** (2 patterns) - using `match &node.node_type { ... }`

### Step 2: Migrate Type-Checking Patterns to Injection

**Pattern 1: Certificate count**
```rust
// Before
.filter(|(_, node)| matches!(
    node.node_type,
    graph::NodeType::RootCertificate{..} |
    graph::NodeType::IntermediateCertificate{..} |
    graph::NodeType::LeafCertificate{..}
))

// After
.filter(|(_, node)| node.domain_node.injection().is_certificate())
```

**Pattern 2: NATS entity count**
```rust
// Before
.filter(|(_, node)| matches!(
    node.node_type,
    graph::NodeType::NatsOperator(_) |
    graph::NodeType::NatsAccount(_) |
    graph::NodeType::NatsUser(_)
))

// After
.filter(|(_, node)| node.domain_node.injection().is_nats())
```

**Pattern 3: YubiKeyStatus count**
```rust
// Before
.filter(|(_, node)| matches!(
    node.node_type,
    graph::NodeType::YubiKeyStatus{..}
))

// After
use crate::gui::domain_node::Injection;
.filter(|(_, node)| node.domain_node.injection() == Injection::YubiKeyStatus)
```

**Pattern 4: NATS account/user count in export readiness**
```rust
// Before
match &node.node_type {
    graph::NodeType::NatsAccount(_) | graph::NodeType::NatsAccountSimple { .. } => { ... }
    graph::NodeType::NatsUser(_) | graph::NodeType::NatsUserSimple { .. } => { ... }
}

// After
use crate::gui::domain_node::Injection;
match node.domain_node.injection() {
    Injection::NatsAccount | Injection::NatsAccountSimple => { ... }
    Injection::NatsUser | Injection::NatsUserSimple => { ... }
}
```

### Step 3: Remove Unused Import

Removed unused import in SelectNodeType handler:
```rust
// Before
use crate::gui::graph::NodeType;  // NOT USED - match is on OrganizationalNodeType

// After: Import removed
```

### Step 4: Migrate Certificate Node Creation

**RootCertificate creation**
```rust
// Before
let node_type = graph::NodeType::RootCertificate { ... };
let mut root_ca_node = graph::ConceptEntity::from_node_type(cert_id, node_type, position);

// After
let domain_node = domain_node::DomainNode::inject_root_certificate(
    cert_id, subject.clone(), subject.clone(), // Self-signed
    chrono::Utc::now(),
    chrono::Utc::now() + chrono::Duration::days(365 * 20),
    vec!["keyCertSign".to_string(), "cRLSign".to_string()],
);
let mut root_ca_node = graph::ConceptEntity::from_domain_node(cert_id, domain_node, position);
```

**LeafCertificate creation** - same pattern.

### Step 5: Migrate Data Extraction Patterns

**GenerateRootCA handler**
```rust
// Before
match &node.node_type {
    graph::NodeType::Person { person, .. } => { ... }
    graph::NodeType::Organization(org) => { ... }
    _ => { ... }
}

// After
if let Some(person) = node.domain_node.person() {
    // Show passphrase dialog for Root CA generation
    self.status_message = format!("Enter passphrase to generate Root CA for {}", person.name);
} else if let Some(org) = node.domain_node.organization() {
    // Organization generates root CA (top-level authority)
    self.status_message = format!("Enter passphrase to generate Root CA for organization '{}'", org.display_name);
} else {
    self.status_message = "Root CA can only be generated for Organizations or Persons".to_string();
}
```

**GenerateIntermediateCA handler** - same pattern with `organization()` and `organization_unit()` accessors.

### Step 6: New DomainNode Accessor

Added `organization_unit()` accessor to support the GenerateIntermediateCA pattern:
```rust
/// Get OrganizationUnit reference if this is an OrganizationUnit node
pub fn organization_unit(&self) -> Option<&OrganizationUnit> {
    match &self.data {
        DomainNodeData::OrganizationUnit(unit) => Some(unit),
        _ => None,
    }
}
```

## Metrics

| Metric | Before Sprint 14 | After Sprint 14 | Change |
|--------|------------------|-----------------|--------|
| graph::NodeType in gui.rs | 14 | 0 | **-100%** |
| node.node_type field access | 8 | 4 | -50% |
| Deprecation warnings | 627 | 599 | -28 |
| Tests passing | 26 | 26 | ✅ |

### Remaining node.node_type Usages (4)

All 4 are PropertyCard integration:
- Lines 3428, 3435: `property_card.set_node(*id, node.node_type.clone())` - node selection
- Line 3996: Same pattern when creating a node
- Line 4470: Export debug string `format!("{:?}", node.node_type)`

These require migrating PropertyCard to use DomainNode (84 NodeType usages in property_card.rs).

### Files Modified

| File | Changes |
|------|---------|
| src/gui/domain_node.rs | +8 lines (added organization_unit() accessor) |
| src/gui.rs | Migrated 9 patterns, removed 1 import |

## Architecture Benefits

### 1. Injection Enum for Type Checks
The `Injection` enum provides clean type discrimination:
```rust
// Before: Pattern match on deprecated enum
matches!(node.node_type, graph::NodeType::NatsAccount(_) | ...)

// After: Use injection helpers
node.domain_node.injection().is_nats()
node.domain_node.injection() == Injection::YubiKeyStatus
```

### 2. DomainNode Accessors for Data Extraction
Clean accessors eliminate pattern matching:
```rust
// Before: Destructure deprecated enum
match &node.node_type {
    graph::NodeType::Person { person, .. } => { ... }
}

// After: Use typed accessor
if let Some(person) = node.domain_node.person() { ... }
```

### 3. from_domain_node() for Node Creation
Direct DomainNode injection eliminates NodeType construction:
```rust
// Before: Create NodeType then convert
let node_type = graph::NodeType::RootCertificate { ... };
ConceptEntity::from_node_type(id, node_type, position)

// After: Inject directly into DomainNode
let domain_node = DomainNode::inject_root_certificate(...);
ConceptEntity::from_domain_node(id, domain_node, position)
```

## Code Quality

```
Compilation: ✅ Clean (deprecation warnings expected, reduced by 28)
Tests: ✅ 26 passed, 48 ignored, 0 failed
graph::NodeType in gui.rs: ✅ 0 (100% elimination!)
OrganizationalNodeType: 22 (different enum, UI-only, not deprecated)
PropertyCard integration: 4 (requires separate Sprint 15)
```

## Future Sprints

### Sprint 15: Migrate PropertyCard to DomainNode
- 84 NodeType usages in property_card.rs
- Change `set_node(id, NodeType)` to `set_node(id, &DomainNode)`
- Migrate all internal pattern matches to use DomainNode accessors/folds

### Sprint 16: Remove NodeType Enum
- After PropertyCard migration
- Delete `graph::NodeType` enum
- Remove `from_node_type()` and `to_node_type()` converters
- Clean up all deprecation warnings

## Lessons Learned

1. **Injection Enum Covers Type Checks**: The `is_certificate()`, `is_nats()`, `is_yubikey()` helpers on Injection handle most type-checking needs cleanly.

2. **Accessors Beat Pattern Matching**: `node.domain_node.person()` is cleaner than `match &node.node_type { NodeType::Person { person, .. } => ... }`.

3. **Unused Imports Accumulate**: The SelectNodeType handler had an unused `NodeType` import that was never actually used in the match arms.

4. **PropertyCard is a Module Boundary**: The remaining 4 usages are all at the PropertyCard interface boundary. Migrating PropertyCard will eliminate them all at once.

5. **OrganizationalNodeType is Distinct**: This UI enum for selecting node types to create is completely separate from the deprecated `graph::NodeType` data enum.

## Summary

Sprint 14 **exceeded its goal** by achieving 0 `graph::NodeType` usages in gui.rs:

- **Type-checking patterns**: Migrated to `Injection` enum helpers
- **Certificate creation**: Migrated to `inject_root_certificate`/`inject_leaf_certificate`
- **Data extraction**: Migrated to DomainNode accessors (`person()`, `organization()`, `organization_unit()`)
- **Unused import**: Removed

The remaining 4 `node.node_type` usages are PropertyCard integration points, requiring PropertyCard migration (Sprint 15) before the NodeType enum can be removed (Sprint 16).

Total progress across sprints:
- Sprint 11: 121 → 96 (-25 usages)
- Sprint 12: GraphEvent migrated to DomainNode
- Sprint 13: 96 → 38 (-58 usages, 60% reduction)
- Sprint 14: 38 → 0 graph::NodeType (**100% elimination in gui.rs!**)
