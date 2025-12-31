<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 11 Retrospective: property_card.rs ViewModel Refactor

**Date**: 2025-12-31
**Sprint Duration**: Single session
**Status**: Completed

## Objective

Refactor property_card.rs to replace all hardcoded UI sizes with ViewModel values, enabling uniform Ctrl+/- scaling across the GUI.

## What Was Done

### Files Modified
1. **src/gui/property_card.rs** - Complete refactor (158 insertions, 154 deletions)
2. **src/gui.rs** - Updated caller to pass `&self.view_model`

### Functions Updated
All view functions now accept `vm: &ViewModel` parameter:
- `view()` - Public entry point
- `view_node()` - Node details rendering
- `view_edge()` - Edge details rendering
- `view_nats_details()` - NATS configuration display
- `view_certificate_details()` - Certificate information
- `view_yubikey_details()` - YubiKey slot details
- `view_policy_details()` - Policy information
- `detail_row()` - Helper function for label/value pairs

### Mappings Applied

| Original Value | ViewModel Field | Usage Context |
|---------------|-----------------|---------------|
| `.size(10)` | `vm.text_tiny` | List items, small details |
| `.size(11)` | `vm.text_tiny` | Certificate fields in iterators |
| `.size(12)` | `vm.text_small` | Detail row values |
| `.size(16)` | `vm.text_medium` | Section headers |
| `.size(18)` | `vm.text_large` | Card headers |
| `.spacing(2)` | `vm.spacing_xs` | Tight vertical lists |
| `.spacing(5)` | `vm.spacing_sm` | Standard element spacing |
| `.spacing(8)` | `vm.spacing_sm` | Column spacing |
| `.spacing(10)` | `vm.spacing_md` | Section spacing |
| `.padding(5)` | `vm.padding_sm` | Container padding |
| `.padding(15)` | `vm.padding_md` | Card padding |
| `.padding(16)` | `vm.padding_lg` | Container padding |
| `.padding(20)` | `vm.padding_xl` | Outer padding |

## What Went Well

### 1. FRP Expert Consultation
Consulting the FRP expert before coding provided clear guidance:
- ViewModel should be passed by reference (`&ViewModel`)
- Views remain pure functions (no mutations)
- Preserved FRP axioms A3, A5, A9

### 2. Closure Variable Capture Pattern
Discovered effective pattern for iterators that need ViewModel values:
```rust
fn view_certificate_details<'a>(&self, domain_node: &'a DomainNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
    let text_tiny = vm.text_tiny;  // Capture before closure
    let spacing_xs = vm.spacing_xs;

    column(cert.key_usage.iter().map(|usage| {
        text(format!("  • {}", usage)).size(text_tiny).into()
    }).collect::<Vec<_>>()).spacing(spacing_xs)
}
```

### 3. Systematic Approach
The task structure in progress.json provided clear checkpoints:
- Add parameter to signature first
- Thread through internal functions
- Replace size/spacing/padding systematically
- Update callers last
- Verify compilation

## Challenges Encountered

### 1. Closure Borrow Issues
Initial attempts to use `vm.text_tiny` directly inside `.map()` closures failed:
```rust
// FAILS: vm borrowed longer than allowed
.map(|usage| text(usage).size(vm.text_tiny))
```

**Solution**: Capture values into local variables before the closure.

### 2. Function Signature Cascade
Changing `detail_row()` to accept `vm` required updating ~40 call sites across 4 different functions. Each call needed the third argument added.

### 3. Lifetime Annotations
The `view_*` functions use explicit lifetimes (`'a`) tied to the input data. The ViewModel parameter uses implicit lifetime (borrows from caller), which worked correctly.

## Lessons Learned

1. **Consult Experts First**: The FRP expert guidance saved time by establishing the correct pattern upfront.

2. **Capture Values for Closures**: When iterating with `.map()` or similar, capture ViewModel fields into local variables outside the closure.

3. **Thread Parameters Systematically**: Start with public function, then internal helpers, then update all call sites.

4. **Test Compilation Frequently**: Running `cargo check` after each major change catches issues early.

## Best Practices Added

**#26**: When passing ViewModel to functions with closures that need its values, capture fields into local variables before the closure (e.g., `let text_tiny = vm.text_tiny;`).

## Metrics

- **Size calls replaced**: 69
- **Spacing calls replaced**: 54
- **Padding calls replaced**: 10
- **Functions updated**: 8
- **Lines changed**: 312 (158 added, 154 removed)
- **Compilation time**: ~5 seconds

## Next Steps (Sprint 12)

Refactor graph.rs buttons and labels:
- 16 .size() calls
- 3 .spacing() calls
- 11 .padding() calls
- Focus on button rendering and canvas text labels
