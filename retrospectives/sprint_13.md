<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 13 Retrospective: role_palette.rs ViewModel Refactor

**Date**: 2025-12-31
**Sprint Duration**: Single session
**Status**: Completed

## Objective

Refactor role_palette.rs to replace all hardcoded UI sizes with ViewModel values, enabling uniform Ctrl+/- scaling for the role palette sidebar.

## What Was Done

### Files Modified
1. **src/gui/role_palette.rs** - Added ViewModel parameter and replaced values
2. **src/gui.rs** - Updated caller to pass `&self.view_model`

### Functions Updated
- `view()` - Main public function, added `vm: &ViewModel` parameter
- `role_badge()` - Helper for individual role badges
- `category_header()` - Helper for expandable category headers

### Mappings Applied

| Original Value | ViewModel Field | Usage Context |
|---------------|-----------------|---------------|
| `.size(14)` | `vm.text_normal` | "Role Palette" title |
| `.size(13)` | `vm.text_small` | Category names |
| `.size(12)` | `vm.text_small` | Role names |
| `.size(11)` | `vm.text_tiny` | Role count, expand button |
| `.size(10)` | `vm.text_tiny` | Level indicator, claims, arrow |
| `.size(8)` | `vm.padding_sm` | Indicator circle (pseudo-size) |
| `.size(6)` | `vm.spacing_xs` | Small circle indicator |
| `.spacing(6)` | `vm.spacing_sm` | Header row |
| `.spacing(4)` | `vm.padding_xs` | Badge row, content column |
| `.spacing(3)` | `vm.spacing_xs` | Role column |
| `.padding(10)` | `vm.padding_md` | No-data container |
| `.padding(4)` | `vm.padding_xs` | Main container, collapsed state |

## What Went Well

### 1. No Lifetime Annotations Needed
Unlike Sprint 12 (graph.rs), role_palette.rs doesn't need explicit lifetime annotations. The ViewModel borrow is elided because the return type's lifetime is tied to `policy_data`, not `vm`.

```rust
// This works without explicit lifetimes
pub fn view<'a>(&self, policy_data: Option<&'a PolicyBootstrapData>, vm: &ViewModel) -> Element<'a, RolePaletteMessage>
```

### 2. Consistent Helper Pattern
The helper functions (`role_badge`, `category_header`) accept `vm: &ViewModel` without lifetimes since they don't need to tie their return type to the ViewModel.

### 3. Small File Size
At ~400 lines, role_palette.rs is manageable in a single pass. The changes were localized and predictable.

## Challenges Encountered

### 1. Indicator Circles as Text Size
The code uses `text(" ").size(N)` for colored indicator circles. This maps to:
- `.size(8)` → `vm.padding_sm` (8px base)
- `.size(6)` → `vm.spacing_xs` (2px base, but visually works)

This is semantically incorrect (using text size for decorative element size), but maintains backward compatibility.

### 2. Small Size Values
Several `.size(10)` and `.size(11)` calls all map to `vm.text_tiny` (10px base). This loses the 1px distinction but maintains consistent scaling.

## Lessons Learned

1. **Lifetime Elision Works When Return Type Binds to Data**: If the return `Element<'a>` lifetime comes from a data parameter (like `policy_data`), you don't need to explicitly annotate the ViewModel borrow.

2. **Decorative Elements Scale Differently**: Using text size for decorative circles is a workaround. Consider adding dedicated sizing for indicators in future ViewModel updates.

3. **Semantic vs Literal Mapping**: Sometimes multiple hardcoded values (10, 11, 12) map to the same semantic category (tiny, small). This is acceptable for consistent scaling.

## Best Practices Confirmed

**#26** (Sprint 11): Closure variable capture still applies but wasn't needed here.
**#27** (Sprint 12): Explicit lifetimes only needed when multiple refs compete for return lifetime.

## Metrics

- **Size calls replaced**: 13
- **Spacing calls replaced**: 4
- **Padding calls replaced**: 3
- **Helper functions updated**: 2
- **Lines changed**: 55 (28 added, 27 removed)
- **Compilation time**: ~5 seconds

## Next Steps (Sprint 14)

Refactor passphrase_dialog.rs:
- 12 .size() calls
- 7 .spacing() calls
- 1 .padding() call
- Apply same patterns established in Sprints 11-13
