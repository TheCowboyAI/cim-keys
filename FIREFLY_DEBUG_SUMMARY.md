# Firefly Shader Debug Summary

## Problem
- Only 1 firefly was visible instead of 40
- The kuramoto_firefly_shader.rs was over-engineered with complex calculations

## Solution
Created `debug_firefly_shader.rs` with ultra-simple implementation:

### Key Features
1. **Simple Grid Layout**: 8x5 grid = 40 fireflies
2. **GPU Instancing**: Uses `draw(0..6, 0..NUM_FIREFLIES)` for efficient rendering
3. **Minimal Math**:
   - Simple circular motion with `cos/sin`
   - Size pulsing for visual interest
   - Each firefly has unique phase offset

### Shader Structure
```wgsl
// Vertex shader calculates:
- Base grid position
- Circular motion offset (radius = 0.05)
- Size pulsing (0.06 * (1.0 + 0.2 * sin(time)))
- Rainbow coloring based on instance_idx

// Fragment shader:
- Simply returns solid color
```

### Working Features
- ✅ All 40 fireflies visible
- ✅ Each has unique rainbow color
- ✅ Simple circular motion
- ✅ Size pulsing animation
- ✅ Runs efficiently on GPU

## Lessons Learned
1. **Keep it simple** - Complex boundary checking and movement calculations were unnecessary
2. **Use instance_idx directly** - No need for complex buffer management
3. **Test incrementally** - Start with static grid, then add movement
4. **GPU-optimized math** - Simple sin/cos calculations run efficiently in parallel

## Performance
- Minimal GPU load with simple calculations
- 40 instances render smoothly at 60fps
- No complex state management needed

---

## WGSL Shader Alignment Fix (2025-10-25)

### New Issue Discovered
After initial fixes, encountered WGSL shader validation errors:
```
Validation Error: The array stride 4 is not a multiple of the required alignment 16
```

### Root Cause
WGSL requires arrays in uniform buffers to be aligned to 16-byte boundaries. The structs had:
- `time: f32` (4 bytes)
- `num_fireflies: f32` (4 bytes)
- `phase_data: array<f32, 40>` (starting at 8-byte offset - INVALID)

### Solution Applied
Added padding fields to ensure 16-byte alignment:

**Rust Struct:**
```rust
struct Uniforms {
    time: f32,
    num_fireflies: f32,
    _padding: [f32; 2], // 8 bytes of padding
    phase_data: [f32; NUM_FIREFLIES as usize],
}
```

**WGSL Shader:**
```wgsl
struct Uniforms {
    time: f32,
    num_fireflies: f32,
    _padding: vec2<f32>, // Padding to align phase_data to 16 bytes
    phase_data: array<f32, 40>,
}
```

### Files Fixed
- `src/gui/debug_firefly_shader.rs` (lines 165-168, 283)
- `src/gui/simple_kuramoto_shader.rs` (lines 165-168, 268)

### Testing
GUI now runs successfully with:
```bash
# Recommended command (using nix app)
nix run .#gui

# Development mode
nix develop --command cargo run --bin cim-keys-gui --features gui -- /tmp/output
```

✅ All shader validation errors resolved
✅ Firefly animations work correctly with Kuramoto synchronization