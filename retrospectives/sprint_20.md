<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 20 Retrospective: RGBA Consistency (Not Color Ontology)

**Date**: 2025-12-31
**Sprint Duration**: Single session
**Status**: Completed

## Objective

Ensure consistent color representation in graph.rs for future adapter compatibility.

## Key Insight: Don't Build the Adapter Here

**The color model is RGBA. Period.**

Initial approach (WRONG):
- Add 50+ EdgeType color fields to ColorPalette
- Create edge_type_color() method
- Build a color ontology system within cim-keys-gui

Correct approach (SIMPLE):
- Use RGBA consistently: `Color::from_rgb()` and `Color::from_rgba()`
- Colors in graph.rs canvas drawing are fine as-is
- An adapter can lift RGBA to a Conceptual Space later

## What Was Done

### 1. Reverted Overcomplicated Changes
Removed premature abstractions:
- 50+ EdgeType color fields NOT added to ColorPalette
- Complex edge_type_color() method NOT created
- Color ontology NOT embedded in GUI layer

### 2. Verified RGBA Consistency
Final color format counts in graph.rs:
- `Color::from_rgb()`: 70 occurrences (opaque colors)
- `Color::from_rgba()`: 11 occurrences (transparent colors)
- `Color::from_rgba8()`: 0 occurrences (converted to from_rgba)

### 3. Fixed Minor Issues
- Removed unused `Color` import from property_card.rs
- Converted one `from_rgba8()` call to `from_rgba()` for consistency

## Architectural Principle

```
cim-keys-gui uses RGBA internally
        ↓
    Port/Adapter
        ↓
Conceptual Space (Bicone/HSL model)
```

The translation happens at the adapter layer, NOT inside the GUI. This keeps:
- GUI code simple and focused
- Color values liftable to any conceptual space
- No coupling between GUI and color theory

## Best Practice Added

**#31**: Use consistent RGBA representation in GUI code. Color model translation happens at the adapter layer, not in the implementation.

## Metrics

- **Lines removed**: ~150 (reverted overengineering)
- **from_rgba8 calls converted**: 1
- **Unused imports removed**: 1
- **Compilation time**: ~4 seconds
- **Total RGBA format calls in graph.rs**: 81 (all consistent)

## Next Steps (Sprint 21)

Final integration and SD card testing.
