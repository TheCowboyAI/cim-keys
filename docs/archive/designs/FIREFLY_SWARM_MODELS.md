# Mathematical Models for Firefly Swarm Behavior

## Overview

This document describes the mathematical models and control formulas used to simulate realistic firefly swarm behavior in the CIM Keys GUI. These models are based on peer-reviewed scientific research and empirical observations of actual firefly populations.

## 1. Kuramoto Model for Synchronization

The Kuramoto model treats each firefly as a phase oscillator that synchronizes with its neighbors through coupling interactions.

### Core Equation

```
dθᵢ/dt = ω + (K/N) ∑ⱼ∈S sin(θⱼ - θᵢ) + η
```

**Variables:**
- `θᵢ`: Phase of firefly i in range [0, 2π)
- `ω`: Natural frequency (phase velocity)
- `K`: Synchronization strength (coupling parameter)
- `N`: Total number of fireflies
- `S`: Set of fireflies within synchronization radius
- `η`: Random noise term for natural variation

**Implementation Notes:**
- When θ exceeds 2π, it resets to zero and triggers a flash
- The coupling term creates the "nudge" that leads to synchronization
- Typical K values: 2.0-3.0 for strong synchronization

### GPU Shader Implementation

```rust
// Kuramoto phase update (CPU side for now)
fn update_kuramoto_phases(&mut self, dt: f32) {
    for i in 0..NUM_FIREFLIES {
        let mut coupling_sum = 0.0;
        for j in 0..NUM_FIREFLIES {
            if i != j {
                let phase_diff = self.phases[j] - self.phases[i];
                coupling_sum += fast_sin(phase_diff);
            }
        }
        let phase_velocity = self.frequencies[i] +
            (COUPLING_STRENGTH / NUM_FIREFLIES as f32) * coupling_sum;
        self.phases[i] += dt * phase_velocity;
    }
}
```

## 2. Elliptic Burster Model for Photinus carolinus

This model captures the burst flashing pattern (multiple rapid flashes followed by silence) observed in P. carolinus fireflies.

### Single Firefly Dynamics

```
u' = u(λ + br - r²) - ωv
v' = v(λ + br - r²) + ωu
λ' = ε(r̄ - r)
```

**Variables:**
- `u, v`: Fast variables for flash dynamics
- `r = √(u² + v²)`: Oscillation amplitude
- `λ`: Slow variable modulating active/quiescent phases
- `b, r̄, ω`: Positive parameters
- `ε`: Small parameter (0 < ε ≪ 1)
- Flash occurs when `u > θ` (threshold)

### Population with Coupling and Noise

```
uⱼ' = uⱼ(λⱼ + brⱼ - rⱼ²) - vⱼ(1 + ωⱼ) + gcⱼ + σᵤξⱼ(t)
vⱼ' = vⱼ(λⱼ + brⱼ - rⱼ²) + uⱼ(1 + ωⱼ) + σᵤζⱼ(t)
λⱼ' = ε(r̄ - rⱼ)(1 + μⱼ) + σₗυⱼ(t)
```

**Additional Parameters:**
- `g`: Coupling strength between fireflies
- `cⱼ`: Number of flashing neighbors within radius R
- `ωⱼ, μⱼ`: Heterogeneity parameters [-0.05, 0.05]
- `σᵤ, σₗ`: Noise strengths
- `ξⱼ(t), ζⱼ(t), υⱼ(t)`: White Gaussian noise

### Synchronization Measure

```
χ = Var(s_tot) / ((1/N)∑ⱼ Var(sⱼ))
```

Where:
- `χ = 0`: Complete asynchrony
- `χ = 1`: Complete synchronization

## 3. Emergent Periodicity Model

Explains how randomly flashing isolated fireflies develop rhythmic periodicity in groups.

### Collective Interburst Interval Distribution

```
Pₙ(Tᵦ) = N (∫[Tᵦ to ∞] b(t)dt)^(N-1) b(Tᵦ)
```

**Variables:**
- `Pₙ(Tᵦ)`: Probability distribution for N fireflies
- `Tᵦ`: Collective interburst interval
- `b(t)`: Single firefly interval distribution
- As N → ∞, distribution converges to δ(Tᵦ - T₀)

### Integrate-and-Fire Model with Coupling

```
dVᵢ(t)/dt = (1/Tsᵢ)εᵢ(t) - (1/Tdᵢ)[1 - εᵢ(t)] + εᵢ(t)∑ᵢⱼ βᵢⱼδᵢⱼ[1 - εⱼ(t)]
```

**Parameters:**
- `Vᵢ(t)`: Internal voltage state
- `εᵢ(t)`: Binary (1=charging, 0=flashing)
- `Tsᵢ`: Charging time
- `Tdᵢ`: Flash duration
- `β`: Coupling strength (0.12-0.18 for groups of 5-15)
- `δᵢⱼ`: Connectivity matrix

## 4. Movement Dynamics

### Lissajous Curves for Organic Movement

Our implementation uses Lissajous curves to create natural figure-8 and circular flight patterns:

```wgsl
// WGSL shader code
let movement_phase = uniforms.time * 0.3 + firefly_id * 0.618033988749895; // Golden ratio
let phase_x = movement_phase * 1.3;
let phase_y = movement_phase * 0.7 + 1.5707963267948966; // +π/2 phase offset

// Create figure-8 and circular patterns
let dx = taylor_sin(phase_x) * 0.15 * (1.0 + 0.3 * taylor_sin(movement_phase * 0.5));
let dy = taylor_cos(phase_y) * 0.12 * (1.0 + 0.3 * taylor_cos(movement_phase * 0.3));
```

### Boundary Constraints with Soft Reflection

```wgsl
// Soft boundary reflection
let boundary_margin = 0.05;
let boundary_softness = 0.1;

if (base_pos.x < boundary_margin + boundary_softness) {
    base_pos.x = boundary_margin + boundary_softness +
        (boundary_margin + boundary_softness - base_pos.x) * 0.8; // 80% reflection
}
// Similar for other boundaries...

// Hard clamping fallback
base_pos = clamp(base_pos, vec2<f32>(boundary_margin), vec2<f32>(1.0 - boundary_margin));
```

## 5. Light Intensity and Glow Rendering

### SDF-Based Glow Rendering

Instead of expensive exponential falloff, we use Signed Distance Functions:

```wgsl
// SDF circle for efficient glow
fn sdf_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

// Multi-layer glow
let dist = sdf_circle(input.uv, 0.1);
let core = smoothstep(0.1, 0.0, dist);
let inner_glow = smoothstep(0.5, 0.0, dist);
let outer_glow = smoothstep(1.0, 0.0, dist);
let glow = core + inner_glow * 0.6 + outer_glow * 0.3;
```

### Color Based on Synchronization

```wgsl
// Synchronization affects color temperature
let sync_level = input.intensity; // From Kuramoto phase
let color = mix(
    vec3<f32>(0.2, 0.5, 1.0),  // Cool blue (desynchronized)
    vec3<f32>(1.0, 0.9, 0.3),   // Warm yellow (synchronized)
    sync_level
);
```

## 6. Optimization Techniques

### Taylor Series Approximations

Replace expensive trigonometric functions with polynomial approximations:

```rust
// 5th order Taylor series for sine
fn fast_sin(x: f32) -> f32 {
    let x = normalize_angle(x); // Reduce to [-π, π]
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    x - x3 / 6.0 + x5 / 120.0
}

// 6th order Taylor series for cosine
fn taylor_cos(x: f32) -> f32 {
    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    1.0 - x2 * 0.5 + x4 * 0.04166667 - x6 * 0.00138889
}
```

### Bitwise Hash for Randomness

Avoid transcendental functions for random number generation:

```wgsl
// MurmurHash-inspired integer hashing
fn hash(p: vec2<f32>) -> f32 {
    var hx = u32(p.x * 1597.0);
    var hy = u32(p.y * 3797.0);
    hx = ((hx >> 16u) ^ hx) * 0x45d9f3bu;
    hx = ((hx >> 16u) ^ hx) * 0x45d9f3bu;
    hx = (hx >> 16u) ^ hx;
    hy = ((hy >> 16u) ^ hy) * 0x45d9f3bu;
    hy = ((hy >> 16u) ^ hy) * 0x45d9f3bu;
    hy = (hy >> 16u) ^ hy;
    return f32(hx ^ hy) / 4294967296.0;
}
```

## 7. Parameter Ranges from Empirical Observations

Based on scientific literature and field observations:

### Kuramoto Model Parameters
- **Coupling Strength (K)**: 2.0-3.0 for strong synchronization
- **Natural Frequencies**: 0.8-1.2 Hz (slight variation)
- **Synchronization Radius**: 0.3 (normalized units)
- **Number of Fireflies**: 20-50 for good visual effect

### Movement Parameters
- **Flight Speed**: 0.3 units/second base velocity
- **Movement Radius**: 0.12-0.15 normalized units
- **Boundary Margin**: 0.05 (5% of viewport)
- **Reflection Coefficient**: 0.8 (80% velocity retained)

### Flash Parameters
- **Flash Duration**: 0.1-0.2 seconds
- **Interburst Interval**: 3-6 seconds (species-dependent)
- **Synchronization Time**: 10-30 seconds to achieve >80% sync

## 8. Implementation Architecture

### CPU-Side Computation
- Kuramoto phase updates (could move to compute shader)
- Frequency initialization
- Synchronization metrics

### GPU Shader Pipeline
1. **Vertex Shader**: Position fireflies based on ID
2. **Fragment Shader**: Render glow using SDF
3. **Storage Buffer**: Phase intensities for synchronization
4. **Uniform Buffer**: Time, resolution, parameters

### Memory Layout
```rust
struct Uniforms {
    time: f32,
    _padding: f32,  // WGSL alignment requirement
    resolution: [f32; 2],
    num_fireflies: f32,
    coupling_strength: f32,
}
```

## 9. Future Enhancements

### Spatial Coupling
Currently using all-to-all coupling. Could optimize with:
- Spatial binning for neighbor queries
- Distance-based coupling strength
- Vision cone constraints

### Compute Shader Integration
Move Kuramoto calculations to GPU:
- Parallel phase updates
- Neighbor detection on GPU
- Real-time parameter adjustment

### Species Variations
Implement different firefly species patterns:
- *P. carolinus*: Burst flashing (6-8 flashes)
- *P. pyralis*: J-shaped flash pattern
- *Photuris*: Predatory mimicry patterns

## References

1. Kuramoto, Y. (1984). Chemical Oscillations, Waves, and Turbulence.
2. Ermentrout, B. (1991). An adaptive model for synchrony in the firefly Pteroptyx malaccae.
3. Sarfati, R., et al. (2022). Emergent periodicity in the collective synchronous flashing of fireflies.
4. Peleg, O., et al. (2022). The synchronization of firefly flashing in swarms.
5. Various peer-reviewed papers on firefly synchronization and swarm dynamics.

## License

This documentation and the associated implementation are part of the CIM Keys project,
licensed under MIT OR Apache-2.0.