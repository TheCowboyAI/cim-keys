<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 12 Retrospective: graph.rs ViewModel Refactor

**Date**: 2025-12-31
**Sprint Duration**: Single session
**Status**: Completed

## Objective

Refactor the `view_graph` function in graph.rs to replace all hardcoded UI sizes with ViewModel values, enabling uniform Ctrl+/- scaling for graph controls and details.

## What Was Done

### Files Modified
1. **src/gui/graph.rs** - Added ViewModel parameter and replaced hardcoded values
2. **src/gui.rs** - Updated 7 callers to pass `&self.view_model`

### Functions Updated
- `view_graph` - Added `vm: &ViewModel` parameter and explicit lifetime annotations

### Mappings Applied

| Original Value | ViewModel Field | Usage Context |
|---------------|-----------------|---------------|
| `.size(16)` | `vm.text_medium` | +/- zoom buttons |
| `.size(14)` | `vm.text_normal` | Reset button, section headers |
| `.size(12)` | `vm.text_small` | Auto Layout button, role badges |
| `.size(11)` | `vm.text_tiny` | Layout algorithm buttons, "more roles" |
| `.size(6)` | `vm.spacing_xs` | Spacer (pseudo-text for vertical gap) |
| `.padding(6)` | `vm.padding_sm` | Main control buttons |
| `.padding(4)` | `vm.padding_xs` | Algorithm buttons (smaller) |
| `.padding(10)` | `vm.padding_md` | Container outer padding |
| `.spacing(8)` | `vm.spacing_sm` | Controls row spacing |
| `.spacing(10)` | `vm.spacing_md` | Main items column spacing |
| `.spacing(5)` | `vm.spacing_sm` | Details panel spacing |

## What Went Well

### 1. Single Public Function
Graph.rs only exposes one public view function (`view_graph`), making the refactor scope clear and bounded.

### 2. Consistent Patterns from Sprint 11
The patterns established in Sprint 11 (closure variable capture, systematic replacement) applied directly.

### 3. Clean Caller Updates
All 7 callers in gui.rs follow the same pattern - a simple match arm modification.

## Challenges Encountered

### 1. Lifetime Annotations Required
Adding a second reference parameter (`vm: &ViewModel`) alongside the existing `graph: &OrganizationConcept` required explicit lifetime annotations:

```rust
// Before (wouldn't compile with second ref)
pub fn view_graph(graph: &OrganizationConcept, vm: &ViewModel) -> Element<'_, OrganizationIntent>

// After (explicit lifetimes)
pub fn view_graph<'a>(graph: &'a OrganizationConcept, vm: &'a ViewModel) -> Element<'a, OrganizationIntent>
```

The compiler error was clear: "this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `graph` or `vm`".

### 2. Spacer as Text Size
The code used `text("").size(6)` as a visual spacer. This maps to `vm.spacing_xs` semantically (a very small gap), though it's technically a text size. This pattern works but is unusual.

## Lessons Learned

1. **Lifetime Annotations for Multiple References**: When a view function returns `Element<'_>` and takes multiple reference parameters, Rust cannot infer which reference the lifetime comes from. Explicit lifetime parameters resolve this.

2. **Spacer Pattern**: Using empty text with size as a spacer is a workaround. Consider adding a proper `vertical_space()` component in future refactoring.

3. **Consistent Button Sizing**: Main action buttons use `padding_sm` (8px base), while secondary/algorithm buttons use `padding_xs` (4px base). This hierarchy should be maintained.

## Best Practices Added

**#27**: When adding a ViewModel parameter to functions that already take reference parameters and return `Element<'_>`, use explicit lifetime annotations: `fn view<'a>(data: &'a T, vm: &'a ViewModel) -> Element<'a, Msg>`

## Metrics

- **Size calls replaced**: 15
- **Spacing calls replaced**: 3
- **Padding calls replaced**: 11
- **Callers updated**: 7
- **Lines changed**: 80 (43 added, 37 removed)
- **Compilation time**: ~6 seconds

## Next Steps (Sprint 13)

Refactor role_palette.rs:
- 13 .size() calls
- 4 .spacing() calls
- 10 .padding() calls
- Apply same patterns established in Sprints 11-12
