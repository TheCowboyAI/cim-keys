//! Animation System using Continuous Signals
//!
//! Demonstrates how to replace imperative animation state (delta time, lerp values, etc.)
//! with declarative continuous signals that represent smooth value changes over time.
//!
//! ## The Problem
//!
//! Traditional animation code is imperative and stateful:
//!
//! ```ignore
//! struct AnimationState {
//!     current_position: Point,
//!     target_position: Point,
//!     progress: f32,
//!     duration: f32,
//!     elapsed: f32,
//! }
//!
//! fn update(&mut self, delta_time: f32) {
//!     self.elapsed += delta_time;
//!     self.progress = (self.elapsed / self.duration).min(1.0);
//!     self.current_position = lerp(self.start, self.target, ease_out(self.progress));
//! }
//! ```
//!
//! ## The Solution
//!
//! Use continuous signals to represent animations as time-varying values:
//!
//! ```text
//! Time ──→ ContinuousSignal<Point> ──→ Current Position
//!
//! animation_signal.sample(current_time) → position at that moment
//! ```
//!
//! Benefits:
//! - No mutable state
//! - Composable (sequence, parallel, combine animations)
//! - Seekable (sample at any time)
//! - Testable (deterministic, no side effects)

use crate::signals::{Signal, ContinuousKind};
use iced::Point;
use std::f32::consts::PI;
use uuid::Uuid;
use std::collections::HashMap;

/// Easing functions for natural motion
///
/// All easing functions take progress in [0, 1] and return eased value in [0, 1]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    /// Linear interpolation (constant speed)
    Linear,
    /// Ease in (slow start, accelerating)
    EaseIn,
    /// Ease out (fast start, decelerating)
    EaseOut,
    /// Ease in-out (slow start and end, fast middle)
    EaseInOut,
    /// Bounce effect at the end
    Bounce,
    /// Elastic spring effect
    Elastic,
}

impl EasingFunction {
    /// Apply easing to a progress value [0, 1]
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => t * (2.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingFunction::Bounce => {
                let t = 1.0 - t;
                1.0 - if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
            EasingFunction::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    let t = t - 1.0;
                    -(2.0_f32.powf(10.0 * t) * ((t - s) * (2.0 * PI) / p).sin()) + 1.0
                }
            }
        }
    }
}

/// Animate a single Point over time
///
/// Creates a continuous signal that interpolates between start and end positions
/// using the specified easing function.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::animations::{animate_point, EasingFunction};
/// use iced::Point;
///
/// let start = Point::new(0.0, 0.0);
/// let end = Point::new(100.0, 100.0);
/// let duration = 1.0; // 1 second
///
/// let animation = animate_point(start, end, duration, EasingFunction::EaseOut);
///
/// // Sample at different times
/// let pos_start = animation.sample(0.0);   // Point(0, 0)
/// let pos_mid = animation.sample(0.5);     // Point(50, 50) with easing
/// let pos_end = animation.sample(1.0);     // Point(100, 100)
/// ```
pub fn animate_point(
    start: Point,
    end: Point,
    duration: f32,
    easing: EasingFunction,
) -> Signal<ContinuousKind, Point> {
    Signal::<ContinuousKind, Point>::continuous(Box::new(move |t| {
        let progress = ((t as f32) / duration).clamp(0.0, 1.0);
        let eased = easing.apply(progress);

        Point::new(
            start.x + (end.x - start.x) * eased,
            start.y + (end.y - start.y) * eased,
        )
    }))
}

/// Animate multiple nodes simultaneously
///
/// Returns a signal that interpolates all node positions in parallel.
/// Useful for layout transitions where many nodes move at once.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::animations::{animate_nodes, EasingFunction};
/// use iced::Point;
/// use uuid::Uuid;
/// use std::collections::HashMap;
///
/// let mut start_positions = HashMap::new();
/// let mut end_positions = HashMap::new();
///
/// let node1 = Uuid::new_v4();
/// start_positions.insert(node1, Point::new(0.0, 0.0));
/// end_positions.insert(node1, Point::new(100.0, 0.0));
///
/// let animation = animate_nodes(
///     start_positions,
///     end_positions,
///     1.0,
///     EasingFunction::EaseInOut
/// );
///
/// let positions_at_mid = animation.sample(0.5);
/// ```
pub fn animate_nodes(
    start_positions: HashMap<Uuid, Point>,
    end_positions: HashMap<Uuid, Point>,
    duration: f32,
    easing: EasingFunction,
) -> Signal<ContinuousKind, HashMap<Uuid, Point>> {
    Signal::<ContinuousKind, HashMap<Uuid, Point>>::continuous(Box::new(move |t| {
        let progress = ((t as f32) / duration).clamp(0.0, 1.0);
        let eased = easing.apply(progress);

        let mut current_positions = HashMap::new();

        for (id, start_pos) in &start_positions {
            if let Some(end_pos) = end_positions.get(id) {
                let current = Point::new(
                    start_pos.x + (end_pos.x - start_pos.x) * eased,
                    start_pos.y + (end_pos.y - start_pos.y) * eased,
                );
                current_positions.insert(*id, current);
            } else {
                // Node doesn't have end position, keep at start
                current_positions.insert(*id, *start_pos);
            }
        }

        // Handle nodes that only exist in end positions (newly added)
        for (id, end_pos) in &end_positions {
            if !start_positions.contains_key(id) {
                // Fade in from center or from end position
                current_positions.insert(*id, *end_pos);
            }
        }

        current_positions
    }))
}

/// Animate a scalar value over time
///
/// Generic animation for any numeric value (opacity, scale, rotation, etc.)
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::animations::{animate_value, EasingFunction};
///
/// // Fade opacity from 0.0 to 1.0 over 0.5 seconds
/// let fade_in = animate_value(0.0, 1.0, 0.5, EasingFunction::Linear);
///
/// let opacity = fade_in.sample(0.25); // 0.5
/// ```
pub fn animate_value(
    start: f32,
    end: f32,
    duration: f32,
    easing: EasingFunction,
) -> Signal<ContinuousKind, f32> {
    Signal::<ContinuousKind, f32>::continuous(Box::new(move |t| {
        let progress = ((t as f32) / duration).clamp(0.0, 1.0);
        let eased = easing.apply(progress);
        start + (end - start) * eased
    }))
}

/// Sequence two animations (run one after the other)
///
/// Creates a signal that plays animation_a, then animation_b.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::animations::{animate_value, sequence, EasingFunction};
///
/// let move_right = animate_value(0.0, 100.0, 1.0, EasingFunction::EaseOut);
/// let move_down = animate_value(0.0, 50.0, 0.5, EasingFunction::EaseIn);
///
/// let sequenced = sequence(move_right, move_down, 1.0);
///
/// // 0.0 - 1.0: move_right plays
/// // 1.0 - 1.5: move_down plays
/// let x_at_end = sequenced.sample(1.5); // 50.0
/// ```
pub fn sequence<T: Clone + Send + Sync + 'static>(
    animation_a: Signal<ContinuousKind, T>,
    animation_b: Signal<ContinuousKind, T>,
    duration_a: f32,
) -> Signal<ContinuousKind, T> {
    Signal::<ContinuousKind, T>::continuous(Box::new(move |t| {
        if t < duration_a as f64 {
            animation_a.sample(t)
        } else {
            animation_b.sample(t - duration_a as f64)
        }
    }))
}

/// Delay an animation by a specified time
///
/// Creates a signal that holds at the start value, then plays the animation.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::animations::{animate_value, delay, EasingFunction};
///
/// let fade_in = animate_value(0.0, 1.0, 0.5, EasingFunction::Linear);
/// let delayed_fade = delay(fade_in, 0.0, 1.0); // Wait 1 second before fading
///
/// let opacity_before = delayed_fade.sample(0.5); // 0.0 (still waiting)
/// let opacity_during = delayed_fade.sample(1.25); // 0.5 (halfway through fade)
/// ```
pub fn delay<T: Clone + Send + Sync + 'static>(
    animation: Signal<ContinuousKind, T>,
    start_value: T,
    delay_duration: f32,
) -> Signal<ContinuousKind, T> {
    Signal::<ContinuousKind, T>::continuous(Box::new(move |t| {
        if t < delay_duration as f64 {
            start_value.clone()
        } else {
            animation.sample(t - delay_duration as f64)
        }
    }))
}

/// Spring physics simulation for natural motion
///
/// Creates a continuous signal that simulates spring physics,
/// useful for "springy" UI elements that overshoot and settle.
///
/// Parameters:
/// - `start`: Initial value
/// - `target`: Target value
/// - `stiffness`: How stiff the spring is (higher = faster)
/// - `damping`: Damping coefficient (higher = less oscillation)
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::animations::spring_animation;
///
/// // Spring from 0 to 100 with medium stiffness and light damping
/// let spring = spring_animation(0.0, 100.0, 150.0, 10.0);
///
/// // Will overshoot 100, then settle back
/// let value_at_1s = spring.sample(1.0);
/// ```
pub fn spring_animation(
    start: f32,
    target: f32,
    stiffness: f32,
    damping: f32,
) -> Signal<ContinuousKind, f32> {
    Signal::<ContinuousKind, f32>::continuous(Box::new(move |t| {
        let t = t as f32;
        // Simple spring physics using damped harmonic oscillator
        let displacement = target - start;
        let omega = (stiffness).sqrt();
        let zeta = damping / (2.0 * omega);

        if zeta < 1.0 {
            // Underdamped (oscillates)
            let omega_d = omega * (1.0 - zeta * zeta).sqrt();
            let a = displacement;
            let b = (zeta * omega * displacement) / omega_d;

            let envelope = (-zeta * omega * t).exp();
            let oscillation = a * (omega_d * t).cos() + b * (omega_d * t).sin();

            start + displacement - envelope * oscillation
        } else if zeta == 1.0 {
            // Critically damped (no oscillation, fastest settling)
            let exp_term = (-omega * t).exp();
            start + displacement * (1.0 - exp_term * (1.0 + omega * t))
        } else {
            // Overdamped (slow, no oscillation)
            let r1 = -omega * (zeta + (zeta * zeta - 1.0).sqrt());
            let r2 = -omega * (zeta - (zeta * zeta - 1.0).sqrt());
            let a = displacement / (r1 - r2);

            start + displacement + a * ((r2 * t).exp() - (r1 * t).exp())
        }
    }))
}

/// Keyframe animation
///
/// Creates a piecewise animation from a series of keyframes.
/// Each keyframe specifies a time and value.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::animations::{keyframe_animation, EasingFunction};
///
/// let keyframes = vec![
///     (0.0, 0.0),    // Start at 0
///     (0.5, 100.0),  // Jump to 100 at t=0.5
///     (1.0, 50.0),   // Back to 50 at t=1.0
/// ];
///
/// let animation = keyframe_animation(keyframes, EasingFunction::EaseInOut);
///
/// let value_at_0_25 = animation.sample(0.25); // Halfway between 0 and 100
/// ```
pub fn keyframe_animation(
    keyframes: Vec<(f32, f32)>,
    easing: EasingFunction,
) -> Signal<ContinuousKind, f32> {
    Signal::<ContinuousKind, f32>::continuous(Box::new(move |t| {
        let t = t as f32;
        if keyframes.is_empty() {
            return 0.0;
        }

        if keyframes.len() == 1 {
            return keyframes[0].1;
        }

        // Find the two keyframes to interpolate between
        let mut start_frame = &keyframes[0];
        let mut end_frame = &keyframes[keyframes.len() - 1];

        for i in 0..keyframes.len() - 1 {
            if t >= keyframes[i].0 && t <= keyframes[i + 1].0 {
                start_frame = &keyframes[i];
                end_frame = &keyframes[i + 1];
                break;
            }
        }

        // Handle edge cases
        if t <= start_frame.0 {
            return start_frame.1;
        }
        if t >= end_frame.0 {
            return end_frame.1;
        }

        // Interpolate between keyframes
        let duration = end_frame.0 - start_frame.0;
        let progress = (t - start_frame.0) / duration;
        let eased = easing.apply(progress);

        start_frame.1 + (end_frame.1 - start_frame.1) * eased
    }))
}

/// Loop an animation infinitely
///
/// Creates a signal that repeats the animation continuously.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::animations::{animate_value, loop_animation, EasingFunction};
///
/// let pulse = animate_value(0.5, 1.0, 1.0, EasingFunction::EaseInOut);
/// let looping_pulse = loop_animation(pulse, 1.0);
///
/// // Will oscillate between 0.5 and 1.0 forever
/// let opacity_at_5s = looping_pulse.sample(5.0);
/// ```
pub fn loop_animation<T: Clone + Send + Sync + 'static>(
    animation: Signal<ContinuousKind, T>,
    duration: f32,
) -> Signal<ContinuousKind, T> {
    Signal::<ContinuousKind, T>::continuous(Box::new(move |t| {
        let t_mod = t % (duration as f64);
        animation.sample(t_mod)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_linear() {
        let easing = EasingFunction::Linear;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_in() {
        let easing = EasingFunction::EaseIn;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.5) < 0.5); // Slow start
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_out() {
        let easing = EasingFunction::EaseOut;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.5) > 0.5); // Fast start
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_animate_point() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(100.0, 100.0);
        let animation = animate_point(start, end, 1.0, EasingFunction::Linear);

        let pos_start = animation.sample(0.0);
        assert_eq!(pos_start.x, 0.0);
        assert_eq!(pos_start.y, 0.0);

        let pos_mid = animation.sample(0.5);
        assert!((pos_mid.x - 50.0).abs() < 0.01);
        assert!((pos_mid.y - 50.0).abs() < 0.01);

        let pos_end = animation.sample(1.0);
        assert_eq!(pos_end.x, 100.0);
        assert_eq!(pos_end.y, 100.0);
    }

    #[test]
    fn test_animate_nodes() {
        let mut start_positions = HashMap::new();
        let mut end_positions = HashMap::new();

        let node1 = Uuid::now_v7();
        let node2 = Uuid::now_v7();

        start_positions.insert(node1, Point::new(0.0, 0.0));
        start_positions.insert(node2, Point::new(0.0, 100.0));

        end_positions.insert(node1, Point::new(100.0, 0.0));
        end_positions.insert(node2, Point::new(100.0, 100.0));

        let animation = animate_nodes(
            start_positions,
            end_positions,
            1.0,
            EasingFunction::Linear,
        );

        let positions = animation.sample(0.5);

        let pos1 = positions.get(&node1).unwrap();
        assert!((pos1.x - 50.0).abs() < 0.01);
        assert_eq!(pos1.y, 0.0);

        let pos2 = positions.get(&node2).unwrap();
        assert!((pos2.x - 50.0).abs() < 0.01);
        assert_eq!(pos2.y, 100.0);
    }

    #[test]
    fn test_animate_value() {
        let animation = animate_value(0.0, 100.0, 1.0, EasingFunction::Linear);

        assert_eq!(animation.sample(0.0), 0.0);
        assert!((animation.sample(0.5) - 50.0).abs() < 0.01);
        assert_eq!(animation.sample(1.0), 100.0);
        assert_eq!(animation.sample(2.0), 100.0); // Clamped at end
    }

    #[test]
    fn test_sequence() {
        let anim1 = animate_value(0.0, 50.0, 1.0, EasingFunction::Linear);
        let anim2 = animate_value(50.0, 100.0, 1.0, EasingFunction::Linear);

        let sequenced = sequence(anim1, anim2, 1.0);

        assert_eq!(sequenced.sample(0.0), 0.0);
        assert!((sequenced.sample(0.5) - 25.0).abs() < 0.01);
        assert_eq!(sequenced.sample(1.0), 50.0);
        assert!((sequenced.sample(1.5) - 75.0).abs() < 0.01);
        assert_eq!(sequenced.sample(2.0), 100.0);
    }

    #[test]
    fn test_delay() {
        let animation = animate_value(0.0, 100.0, 1.0, EasingFunction::Linear);
        let delayed = delay(animation, 0.0, 1.0);

        assert_eq!(delayed.sample(0.0), 0.0);
        assert_eq!(delayed.sample(0.5), 0.0); // Still delayed
        assert_eq!(delayed.sample(1.0), 0.0); // Just starting
        assert!((delayed.sample(1.5) - 50.0).abs() < 0.01);
        assert_eq!(delayed.sample(2.0), 100.0);
    }

    #[test]
    fn test_spring_animation() {
        let spring = spring_animation(0.0, 100.0, 150.0, 10.0);

        let start = spring.sample(0.0);
        assert_eq!(start, 0.0);

        let mid = spring.sample(1.0);
        // Should be approaching target
        assert!(mid > 50.0);

        let later = spring.sample(5.0);
        // Should be very close to target after settling
        assert!((later - 100.0).abs() < 5.0);
    }

    #[test]
    fn test_keyframe_animation() {
        let keyframes = vec![
            (0.0, 0.0),
            (0.5, 100.0),
            (1.0, 50.0),
        ];

        let animation = keyframe_animation(keyframes, EasingFunction::Linear);

        assert_eq!(animation.sample(0.0), 0.0);
        assert!((animation.sample(0.25) - 50.0).abs() < 0.01);
        assert_eq!(animation.sample(0.5), 100.0);
        assert!((animation.sample(0.75) - 75.0).abs() < 0.01);
        assert_eq!(animation.sample(1.0), 50.0);
    }

    #[test]
    fn test_loop_animation() {
        let animation = animate_value(0.0, 100.0, 1.0, EasingFunction::Linear);
        let looped = loop_animation(animation, 1.0);

        assert_eq!(looped.sample(0.0), 0.0);
        assert!((looped.sample(0.5) - 50.0).abs() < 0.01);
        assert_eq!(looped.sample(1.0), 0.0); // Loops back
        assert!((looped.sample(1.5) - 50.0).abs() < 0.01);
        assert_eq!(looped.sample(2.0), 0.0); // Loops back again
    }

    #[test]
    fn test_easing_bounds() {
        let easings = vec![
            EasingFunction::Linear,
            EasingFunction::EaseIn,
            EasingFunction::EaseOut,
            EasingFunction::EaseInOut,
            EasingFunction::Bounce,
            EasingFunction::Elastic,
        ];

        for easing in easings {
            // All easings should start at 0 and end at 1
            assert_eq!(easing.apply(0.0), 0.0);
            assert!((easing.apply(1.0) - 1.0).abs() < 0.01);

            // Middle values should be in reasonable range
            let mid = easing.apply(0.5);
            assert!(mid >= -0.5 && mid <= 1.5); // Some overshoot is ok (bounce, elastic)
        }
    }
}
