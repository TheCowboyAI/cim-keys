<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 7 Retrospective: Migrate Layout Functions to DomainNode Pattern

## Goal
Migrate the remaining `node_type` pattern matches in layout functions to use the DomainNode categorical coproduct pattern, establishing clean accessor methods on the Injection enum and DomainNode struct.

## Completed

### Step 1: Analyze Remaining node_type Usages
Identified ~15 remaining `node_type` usages in graph.rs:
- Layout functions (`layout_hierarchical`, `layout_yubikey_grouped`, `layout_nats_hierarchical`)
- Detail panel rendering (~200 lines - deferred to Sprint 8)
- Message handlers (gui.rs) for entity updates

### Step 2: Add layout_tier() to Injection
Added hierarchical tier method to the Injection enum:

```rust
impl Injection {
    /// Get the layout tier for hierarchical positioning
    pub fn layout_tier(&self) -> u8 {
        match self {
            // Tier 0: Root entities
            Self::Organization => 0,
            Self::NatsOperator | Self::NatsOperatorSimple => 0,
            Self::RootCertificate => 0,
            Self::YubiKey => 0,
            Self::YubiKeyStatus => 0,
            Self::PolicyGroup => 0,

            // Tier 1: Intermediate entities
            Self::OrganizationUnit => 1,
            Self::Role => 1,
            Self::Policy => 1,
            Self::NatsAccount | Self::NatsAccountSimple => 1,
            Self::IntermediateCertificate => 1,
            Self::PivSlot => 1,
            Self::PolicyRole => 1,
            Self::PolicyCategory => 1,

            // Tier 2: Leaf entities
            Self::Person => 2,
            Self::Location => 2,
            Self::NatsUser | Self::NatsUserSimple | Self::NatsServiceAccount => 2,
            Self::LeafCertificate => 2,
            Self::Key => 2,
            Self::Manifest => 2,
            Self::PolicyClaim => 2,
        }
    }
}
```

Also added helper methods:
- `is_nats()` - Check if NATS infrastructure node
- `is_certificate()` - Check if PKI certificate node
- `is_yubikey()` - Check if YubiKey-related node
- `is_policy()` - Check if policy-related node

### Step 3: Add Accessor Methods to DomainNode
Added layout-specific accessors to DomainNode:

```rust
impl DomainNode {
    /// Get YubiKey serial number if this is a YubiKey or PivSlot node
    pub fn yubikey_serial(&self) -> Option<&str>

    /// Get NATS account name if this is a NatsAccountSimple node
    pub fn nats_account_name(&self) -> Option<&str>

    /// Get the parent account name if this is a NATS user node
    pub fn nats_user_account_name(&self) -> Option<&str>

    /// Get the name/identifier for NATS simple nodes
    pub fn nats_name(&self) -> Option<&str>
}
```

### Step 4: Migrate layout_hierarchical()
**Before (30 lines):**
```rust
for (id, node) in &self.nodes {
    match &node.node_type {
        NodeType::Organization(_) => tier_0.push(*id),
        NodeType::OrganizationalUnit(_) => tier_1.push(*id),
        NodeType::Person { .. } => tier_2.push(*id),
        // ... 25 more variants
    }
}
```

**After (6 lines):**
```rust
for (id, node) in &self.nodes {
    match node.injection().layout_tier() {
        0 => tier_0.push(*id),
        1 => tier_1.push(*id),
        _ => tier_2.push(*id),
    }
}
```

**Reduction: 30 lines → 6 lines (80% reduction)**

### Step 5: Migrate layout_yubikey_grouped()
**Before (12 lines):**
```rust
for (id, node) in &self.nodes {
    match &node.node_type {
        NodeType::YubiKey { serial, .. } => {
            yubikeys.push((*id, serial.clone()));
        }
        NodeType::PivSlot { yubikey_serial, .. } => {
            piv_slots.push((*id, yubikey_serial.clone()));
        }
        _ => {}
    }
}
```

**After (12 lines):**
```rust
for (id, node) in &self.nodes {
    let injection = node.injection();
    if injection == Injection::YubiKey {
        if let Some(serial) = node.domain_node.yubikey_serial() {
            yubikeys.push((*id, serial.to_string()));
        }
    } else if injection == Injection::PivSlot {
        if let Some(serial) = node.domain_node.yubikey_serial() {
            piv_slots.push((*id, serial.to_string()));
        }
    }
}
```

**Reduction: Same line count, but cleaner separation of concerns**

### Step 6: Migrate layout_nats_hierarchical()
**Before (23 lines):**
```rust
for (id, node) in &self.nodes {
    match &node.node_type {
        NodeType::NatsOperator(_) | NodeType::NatsOperatorSimple { .. } => { ... }
        NodeType::NatsAccount(_) => { ... }
        NodeType::NatsAccountSimple { name, .. } => { ... }
        NodeType::NatsUser(_) => { ... }
        NodeType::NatsUserSimple { account_name, .. } => { ... }
        NodeType::NatsServiceAccount(_) => { ... }
        _ => {}
    }
}
```

**After (20 lines):**
```rust
for (id, node) in &self.nodes {
    let injection = node.injection();
    match injection {
        Injection::NatsOperator | Injection::NatsOperatorSimple => { ... }
        Injection::NatsAccount | Injection::NatsAccountSimple => {
            let name = node.domain_node.nats_account_name()
                .unwrap_or_else(|| node.label.clone());
            ...
        }
        Injection::NatsUser | Injection::NatsUserSimple => {
            let account_name = node.domain_node.nats_user_account_name()
                .unwrap_or_default();
            ...
        }
        Injection::NatsServiceAccount => { ... }
        _ => {}
    }
}
```

**Reduction: 23 lines → 20 lines (13% reduction), cleaner pattern**

## Metrics

| Metric | Value |
|--------|-------|
| Lines removed from layout_hierarchical | 24 |
| Accessor methods added to DomainNode | 4 |
| Helper methods added to Injection | 5 (layout_tier, is_nats, is_certificate, is_yubikey, is_policy) |
| Layout functions migrated | 3 |
| Remaining NodeType deprecation warnings | 32 |

## Architectural Benefits

### 1. Centralized Tier Logic
- Layout tier logic now lives in `Injection::layout_tier()`
- Adding new node types: update one place, all layouts work
- Tier assignments are documented in the method docstring

### 2. Type-Safe Field Access
- Accessor methods on DomainNode provide type-safe field extraction
- No more pattern matching to extract single fields
- Clear API for layout-specific data needs

### 3. Gradual Migration Path
- Old code still works with deprecation warnings
- New code uses clean injection() and accessor patterns
- Both patterns coexist during transition

## Deferred to Sprint 8

### Detail Panel Rendering (~200 lines)
The largest remaining `node_type` usage is in detail panel rendering (lines ~4175+).
This requires extracting many fields for display and will need:
- Additional accessor methods or a `DetailData` fold
- Careful migration to preserve all displayed information

### Message Handlers in gui.rs
Several message handlers update nodes by pattern matching:
- `Message::PersonUpdated`
- `Message::OrganizationalUnitUpdated`
- etc.

These may need accessor methods for field updates, or refactoring to use the fold pattern.

## Code Quality

```
Compilation: ✅ Clean (deprecation warnings expected)
Tests: ✅ 26 passed, 47 ignored, 0 failed
NodeType deprecated: ✅ Yes (since Sprint 6)
Layout functions migrated: ✅ 3/3
```

## Lessons Learned

1. **Accessor Methods Scale Well**: Simple accessor methods reduce match statement complexity
2. **Injection Enum as Discriminant**: The Injection enum provides clean type discrimination
3. **Fallback Patterns**: Some accessors need fallback to node.label for projection-based nodes
4. **Incremental Migration Works**: Each layout function migrated independently

## Next Sprint (Sprint 8)

1. **Migrate Detail Panel Rendering**
   - Add `DetailData` fold or more accessor methods
   - Target ~200 lines of detail panel code

2. **Migrate Message Handlers**
   - Update handlers in gui.rs to use DomainNode pattern

3. **Consider Removing NodeType**
   - Assess if remaining usages justify keeping the deprecated enum
   - Plan final removal strategy
