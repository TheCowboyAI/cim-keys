//! Animation Time with Continuous Signals
//!
//! Demonstrates using continuous signals (Axiom A10) for smooth animations
//! and time-based transformations.
//!
//! ## Key Pattern
//!
//! Continuous signals represent values defined at all time points:
//! ```text
//! ⟦Continuous T⟧ : Time → T
//! ```
//!
//! This is ideal for:
//! - Animation timelines
//! - Smooth transitions
//! - Easing functions
//! - Periodic oscillations

use cim_keys::signals::continuous::{
    ContinuousSignal, linear_time, constant, sine_wave, lerp,
    ease_in_out, ease_in, ease_out, clamp, scale, offset
};

// ============================================================================
// Example 1: Basic Time Progression
// ============================================================================

fn example_1_linear_time() {
    println!("=== Example 1: Linear Time Progression ===\n");

    let time = linear_time();

    println!("Sampling linear time:");
    for t in [0.0, 0.5, 1.0, 1.5, 2.0] {
        let value = time.sample(t);
        println!("  time({:.1}) = {:.1}", t, value);
    }

    // Use sample_interval for batch sampling
    let samples = time.sample_interval(0.0, 2.0, 5);
    println!("\nInterval sampling (0..2, 5 points): {:?}", samples);
    println!();
}

// ============================================================================
// Example 2: Constant and Periodic Signals
// ============================================================================

fn example_2_constants_and_waves() {
    println!("=== Example 2: Constants and Periodic Signals ===\n");

    // Constant signal
    let always_42 = constant(42);
    println!("Constant signal:");
    println!("  t=0.0  -> {}", always_42.sample(0.0));
    println!("  t=100.0 -> {}", always_42.sample(100.0));

    // Sine wave (1 Hz)
    let wave = sine_wave(1.0);
    println!("\nSine wave (1 Hz):");
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        println!("  sin(2π * {:.2}) = {:.3}", t, wave.sample(t));
    }
    println!();
}

// ============================================================================
// Example 3: Linear Interpolation for Fade Effects
// ============================================================================

fn example_3_fade_transitions() {
    println!("=== Example 3: Fade Transitions with Lerp ===\n");

    // Fade from 0.0 to 1.0 over 2 seconds
    let fade_in = lerp(0.0, 1.0, 0.0, 2.0);

    println!("Fade in (0.0 → 1.0 over 2s):");
    for t in [0.0, 0.5, 1.0, 1.5, 2.0] {
        let opacity = fade_in.sample(t);
        println!("  t={:.1}s -> opacity={:.2}", t, opacity);
    }

    // Fade color channels
    let red_fade = lerp(0.0, 255.0, 0.0, 1.0);
    let blue_fade = lerp(255.0, 0.0, 0.0, 1.0);

    println!("\nColor transition (red → blue):");
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let r = red_fade.sample(t) as u8;
        let b = blue_fade.sample(t) as u8;
        println!("  t={:.2}s -> RGB({}, 0, {})", t, r, b);
    }
    println!();
}

// ============================================================================
// Example 4: Easing Functions for Smooth Animations
// ============================================================================

fn example_4_easing_functions() {
    println!("=== Example 4: Easing Functions ===\n");

    let time = linear_time();

    // Compare linear, ease-in, ease-out, ease-in-out
    let linear = time.clone();
    let eased_in = ease_in(time.clone());
    let eased_out = ease_out(time.clone());
    let eased_in_out = ease_in_out(time);

    println!("Animation progress comparison:");
    println!("Time  | Linear | Ease-In | Ease-Out | In-Out");
    println!("------|--------|---------|----------|--------");

    for t in [0.0, 0.2, 0.4, 0.6, 0.8, 1.0] {
        let lin = linear.sample(t);
        let ein = eased_in.sample(t);
        let eout = eased_out.sample(t);
        let einout = eased_in_out.sample(t);
        println!(
            "{:.1}   | {:.3}  | {:.3}   | {:.3}    | {:.3}",
            t, lin, ein, eout, einout
        );
    }

    println!("\nObservations:");
    println!("- Ease-in: Starts slow, accelerates (good for object appearing)");
    println!("- Ease-out: Starts fast, decelerates (good for object settling)");
    println!("- Ease-in-out: Smooth start and end (most natural looking)");
    println!();
}

// ============================================================================
// Example 5: Combining Continuous Signals
// ============================================================================

fn example_5_signal_composition() {
    println!("=== Example 5: Signal Composition ===\n");

    // Scale time by 2 (make animation twice as fast)
    let double_speed = scale(linear_time(), 2.0);
    println!("Double speed animation:");
    for t in [0.0, 0.5, 1.0] {
        println!("  real_time={:.1}s -> animation_time={:.1}s", t, double_speed.sample(t));
    }

    // Offset time (delay animation start)
    let delayed = offset(linear_time(), -1.0); // Start 1s later
    println!("\nDelayed animation (start at t=1):");
    for t in [0.0, 0.5, 1.0, 1.5, 2.0] {
        let anim_time = delayed.sample(t).max(0.0); // Clamp to non-negative
        println!("  real_time={:.1}s -> animation_time={:.1}s", t, anim_time);
    }

    // Clamp to valid range [0, 1]
    let clamped = clamp(linear_time(), 0.0, 1.0);
    println!("\nClamped progress (0..1):");
    for t in [-0.5, 0.0, 0.5, 1.0, 1.5] {
        println!("  time={:.1} -> progress={:.1}", t, clamped.sample(t));
    }
    println!();
}

// ============================================================================
// Example 6: Bounce Animation with Sine Wave
// ============================================================================

fn example_6_bounce_animation() {
    println!("=== Example 6: Bounce Animation ===\n");

    // Vertical position oscillating with sine wave
    let frequency = 2.0; // 2 cycles per second
    let amplitude = 100.0; // pixels
    let base_position = 200.0; // center

    let wave = sine_wave(frequency);
    let bouncing = wave.map(move |sin_value| {
        base_position + sin_value * amplitude
    });

    println!("Ball vertical position (pixels):");
    println!("Time(s) | Position(px) | State");
    println!("--------|--------------|----------");

    for t in [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0] {
        let y = bouncing.sample(t);
        let state = if y > base_position + amplitude * 0.8 {
            "Top"
        } else if y < base_position - amplitude * 0.8 {
            "Bottom"
        } else {
            "Middle"
        };
        println!("{:.3}   | {:.1}       | {}", t, y, state);
    }
    println!();
}

// ============================================================================
// Example 7: Multi-Stage Animation Timeline
// ============================================================================

fn example_7_timeline_stages() {
    println!("=== Example 7: Multi-Stage Animation Timeline ===\n");

    // Stage 1: Fade in (0-1s)
    let stage1 = ease_in(lerp(0.0, 1.0, 0.0, 1.0));

    // Stage 2: Stay visible (1-3s)
    let stage2 = constant(1.0);

    // Stage 3: Fade out (3-4s)
    let stage3 = ease_out(lerp(1.0, 0.0, 3.0, 4.0));

    println!("Animation timeline (fade in -> hold -> fade out):");
    println!("Time(s) | Opacity | Stage");
    println!("--------|---------|----------");

    for t in [0.0, 0.5, 1.0, 2.0, 3.0, 3.5, 4.0] {
        let opacity = if t < 1.0 {
            stage1.sample(t)
        } else if t < 3.0 {
            stage2.sample(t)
        } else {
            stage3.sample(t)
        };

        let stage = if t < 1.0 {
            "Fade In"
        } else if t < 3.0 {
            "Hold"
        } else {
            "Fade Out"
        };

        println!("{:.1}     | {:.3}   | {}", t, opacity, stage);
    }
    println!();
}

// ============================================================================
// Example 8: Parametric Animation (Circle Path)
// ============================================================================

fn example_8_parametric_path() {
    println!("=== Example 8: Parametric Circle Animation ===\n");

    // Object moves in circle using parametric equations
    let frequency = 0.5; // Half rotation per second
    let radius = 100.0;
    let center_x = 200.0;
    let center_y = 200.0;

    let angle_signal = sine_wave(frequency).map(|sin_val| {
        // Convert to angle: sin gives us vertical component
        // but we want full rotation
        sin_val
    });

    println!("Circular motion path:");
    println!("Time(s) | X(px)  | Y(px)  | Quadrant");
    println!("--------|--------|--------|----------");

    for t in [0.0, 0.5, 1.0, 1.5, 2.0] {
        use std::f64::consts::PI;
        let angle = 2.0 * PI * frequency * t;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();

        let quadrant = match (angle.cos() > 0.0, angle.sin() > 0.0) {
            (true, true) => "I (↗)",
            (false, true) => "II (↖)",
            (false, false) => "III (↙)",
            (true, false) => "IV (↘)",
        };

        println!("{:.1}     | {:.1} | {:.1} | {}", t, x, y, quadrant);
    }
    println!();
}

// ============================================================================
// Main Example Runner
// ============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Continuous Signals for Animation Time                   ║");
    println!("║  Demonstrating Axiom A10: Continuous Time Semantics      ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    example_1_linear_time();
    example_2_constants_and_waves();
    example_3_fade_transitions();
    example_4_easing_functions();
    example_5_signal_composition();
    example_6_bounce_animation();
    example_7_timeline_stages();
    example_8_parametric_path();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Key Insights:                                            ║");
    println!("║  - Continuous signals model smooth time progression       ║");
    println!("║  - Easing functions create natural-looking motion         ║");
    println!("║  - Composition enables complex animation timelines        ║");
    println!("║  - Parametric equations support arbitrary paths           ║");
    println!("║  - All operations are pure functions (no mutable state)   ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
}
