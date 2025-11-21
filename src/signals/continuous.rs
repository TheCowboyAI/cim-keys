//! Continuous Signal Operations - Axiom A10
//!
//! This module provides specialized operations for continuous signals,
//! including interpolation, sampling strategies, and animation time.
//!
//! ## Core Principle
//!
//! Continuous signals represent values that are defined at all points in time,
//! following the denotational semantics:
//!
//! ```text
//! ⟦Continuous T⟧ : Time → T
//! ```
//!
//! In practice, continuous signals are implemented as functions from time to values,
//! with various sampling and interpolation strategies.
//!
//! ## Example: Animation Time
//!
//! ```rust
//! use cim_keys::signals::continuous::{ContinuousSignal, linear_time, ease_in_out};
//!
//! // Linear time progression
//! let time = linear_time();
//! assert_eq!(time.sample(0.5), 0.5);
//!
//! // Eased time for smooth animations
//! let eased = ease_in_out(time);
//! // Eased time changes slowly at start/end, faster in middle
//! ```

use super::{Time, Signal, ContinuousKind};
use std::sync::Arc;

/// Trait for continuous signal operations
///
/// Continuous signals are functions from time to values, defined at all time points.
/// This trait provides common operations like sampling, interpolation, and transformation.
///
/// # Laws
///
/// - **Temporal Continuity**: Value changes smoothly (no discrete jumps)
/// - **Composition**: Continuous signals can be composed to form new continuous signals
/// - **Functor**: fmap over continuous signals preserves continuity
pub trait ContinuousSignal<T>: Clone + Send + Sync {
    /// Sample the signal at a specific time point
    fn sample(&self, time: Time) -> T;

    /// Sample at multiple time points
    fn sample_many(&self, times: &[Time]) -> Vec<T> {
        times.iter().map(|&t| self.sample(t)).collect()
    }

    /// Sample at regular intervals in the given range
    fn sample_interval(&self, start: Time, end: Time, count: usize) -> Vec<T> {
        let step = (end - start) / (count - 1).max(1) as f64;
        (0..count)
            .map(|i| {
                let t = start + step * i as f64;
                self.sample(t)
            })
            .collect()
    }

    /// Transform the signal values
    fn map<U, F>(self, f: F) -> MappedContinuous<T, U, Self, F>
    where
        T: Clone + Send + Sync,
        F: Fn(T) -> U + Send + Sync + Clone,
        U: Clone + Send + Sync,
        Self: Sized,
    {
        MappedContinuous {
            source: self,
            transform: f,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Compose two continuous signals
    fn compose<U, S>(self, other: S) -> ComposedContinuous<T, U, Self, S>
    where
        T: Clone + Send + Sync,
        U: Clone + Send + Sync,
        S: ContinuousSignal<U>,
        Self: Sized,
    {
        ComposedContinuous {
            first: self,
            second: other,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Wrapper to make Signal<ContinuousKind, T> implement ContinuousSignal
impl<T: Clone + Send + Sync> ContinuousSignal<T> for Signal<ContinuousKind, T> {
    fn sample(&self, time: Time) -> T {
        <Signal<ContinuousKind, T>>::sample(self, time)
    }
}

/// A mapped continuous signal
#[derive(Clone)]
pub struct MappedContinuous<T, U, S, F>
where
    S: ContinuousSignal<T>,
    F: Fn(T) -> U + Send + Sync + Clone,
    T: Clone + Send + Sync,
    U: Clone + Send + Sync,
{
    source: S,
    transform: F,
    _phantom: std::marker::PhantomData<(T, U)>,
}

impl<T, U, S, F> ContinuousSignal<U> for MappedContinuous<T, U, S, F>
where
    S: ContinuousSignal<T>,
    F: Fn(T) -> U + Send + Sync + Clone,
    T: Clone + Send + Sync,
    U: Clone + Send + Sync,
{
    fn sample(&self, time: Time) -> U {
        let source_value = self.source.sample(time);
        (self.transform)(source_value)
    }
}

/// A composed continuous signal
#[derive(Clone)]
pub struct ComposedContinuous<T, U, S1, S2>
where
    S1: ContinuousSignal<T>,
    S2: ContinuousSignal<U>,
    T: Clone + Send + Sync,
    U: Clone + Send + Sync,
{
    first: S1,
    second: S2,
    _phantom: std::marker::PhantomData<(T, U)>,
}

impl<T, U, S1, S2> ContinuousSignal<(T, U)> for ComposedContinuous<T, U, S1, S2>
where
    S1: ContinuousSignal<T>,
    S2: ContinuousSignal<U>,
    T: Clone + Send + Sync,
    U: Clone + Send + Sync,
{
    fn sample(&self, time: Time) -> (T, U) {
        (self.first.sample(time), self.second.sample(time))
    }
}

// ============================================================================
// Standard Continuous Signals
// ============================================================================

/// Linear time progression: f(t) = t
///
/// This is the identity signal for time, useful as a base for transformations.
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{linear_time, ContinuousSignal};
///
/// let time = linear_time();
/// assert_eq!(time.sample(0.0), 0.0);
/// assert_eq!(time.sample(1.0), 1.0);
/// assert_eq!(time.sample(2.5), 2.5);
/// ```
pub fn linear_time() -> Signal<ContinuousKind, f64> {
    Signal::<ContinuousKind, f64>::continuous(|t| t)
}

/// Constant signal: f(t) = c for all t
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{constant, ContinuousSignal};
///
/// let always_42 = constant(42);
/// assert_eq!(always_42.sample(0.0), 42);
/// assert_eq!(always_42.sample(100.0), 42);
/// ```
pub fn constant<T: Clone + Send + Sync + 'static>(value: T) -> Signal<ContinuousKind, T> {
    Signal::<ContinuousKind, T>::continuous(move |_| value.clone())
}

/// Periodic signal: f(t) = sin(2π * frequency * t)
///
/// Useful for oscillations and cyclic animations.
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{sine_wave, ContinuousSignal};
///
/// let wave = sine_wave(1.0); // 1 Hz frequency
/// assert!((wave.sample(0.0) - 0.0).abs() < 0.0001);
/// assert!((wave.sample(0.25) - 1.0).abs() < 0.0001); // Peak at quarter period
/// ```
pub fn sine_wave(frequency: f64) -> Signal<ContinuousKind, f64> {
    Signal::<ContinuousKind, f64>::continuous(move |t| (2.0 * std::f64::consts::PI * frequency * t).sin())
}

/// Linear interpolation between two values over a time range
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{lerp, ContinuousSignal};
///
/// let fade = lerp(0.0, 1.0, 0.0, 1.0); // From 0.0 to 1.0 over time 0..1
/// assert_eq!(fade.sample(0.0), 0.0);
/// assert_eq!(fade.sample(0.5), 0.5);
/// assert_eq!(fade.sample(1.0), 1.0);
/// ```
pub fn lerp(start_value: f64, end_value: f64, start_time: Time, end_time: Time) -> Signal<ContinuousKind, f64> {
    Signal::<ContinuousKind, f64>::continuous(move |t| {
        if t <= start_time {
            start_value
        } else if t >= end_time {
            end_value
        } else {
            let progress = (t - start_time) / (end_time - start_time);
            start_value + (end_value - start_value) * progress
        }
    })
}

// ============================================================================
// Easing Functions for Smooth Animations
// ============================================================================

/// Ease-in-out: Smooth acceleration and deceleration
///
/// Uses cubic easing for smooth start and end.
/// Maps linear time [0,1] to eased time [0,1].
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{linear_time, ease_in_out, ContinuousSignal};
///
/// let time = linear_time();
/// let eased = ease_in_out(time);
///
/// // Starts slow, speeds up, then slows down
/// let samples = eased.sample_interval(0.0, 1.0, 5);
/// // Progression is non-linear: slow at 0.0 and 1.0, fast at 0.5
/// ```
pub fn ease_in_out(signal: Signal<ContinuousKind, f64>) -> Signal<ContinuousKind, f64> {
    let signal = Arc::new(signal);
    Signal::<ContinuousKind, f64>::continuous(move |t| {
        let linear = signal.sample(t);
        if linear < 0.5 {
            4.0 * linear * linear * linear
        } else {
            1.0 - (-2.0 * linear + 2.0).powi(3) / 2.0
        }
    })
}

/// Ease-in: Smooth acceleration from rest
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{linear_time, ease_in, ContinuousSignal};
///
/// let eased = ease_in(linear_time());
/// // Starts very slow, accelerates to end
/// ```
pub fn ease_in(signal: Signal<ContinuousKind, f64>) -> Signal<ContinuousKind, f64> {
    let signal = Arc::new(signal);
    Signal::<ContinuousKind, f64>::continuous(move |t| {
        let linear = signal.sample(t);
        linear * linear * linear
    })
}

/// Ease-out: Smooth deceleration to rest
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{linear_time, ease_out, ContinuousSignal};
///
/// let eased = ease_out(linear_time());
/// // Starts fast, decelerates to end
/// ```
pub fn ease_out(signal: Signal<ContinuousKind, f64>) -> Signal<ContinuousKind, f64> {
    let signal = Arc::new(signal);
    Signal::<ContinuousKind, f64>::continuous(move |t| {
        let linear = signal.sample(t);
        1.0 - (1.0 - linear).powi(3)
    })
}

/// Clamp a signal to a range
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{linear_time, clamp, ContinuousSignal};
///
/// let clamped = clamp(linear_time(), 0.0, 1.0);
/// assert_eq!(clamped.sample(-0.5), 0.0); // Clamped to min
/// assert_eq!(clamped.sample(0.5), 0.5);  // Within range
/// assert_eq!(clamped.sample(1.5), 1.0);  // Clamped to max
/// ```
pub fn clamp(signal: Signal<ContinuousKind, f64>, min: f64, max: f64) -> Signal<ContinuousKind, f64> {
    let signal = Arc::new(signal);
    Signal::<ContinuousKind, f64>::continuous(move |t| {
        let value = signal.sample(t);
        value.max(min).min(max)
    })
}

/// Scale a signal by a constant factor
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{linear_time, scale, ContinuousSignal};
///
/// let doubled = scale(linear_time(), 2.0);
/// assert_eq!(doubled.sample(1.0), 2.0);
/// assert_eq!(doubled.sample(2.5), 5.0);
/// ```
pub fn scale(signal: Signal<ContinuousKind, f64>, factor: f64) -> Signal<ContinuousKind, f64> {
    let signal = Arc::new(signal);
    Signal::<ContinuousKind, f64>::continuous(move |t| signal.sample(t) * factor)
}

/// Offset a signal by a constant
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::continuous::{linear_time, offset, ContinuousSignal};
///
/// let shifted = offset(linear_time(), 10.0);
/// assert_eq!(shifted.sample(0.0), 10.0);
/// assert_eq!(shifted.sample(5.0), 15.0);
/// ```
pub fn offset(signal: Signal<ContinuousKind, f64>, delta: f64) -> Signal<ContinuousKind, f64> {
    let signal = Arc::new(signal);
    Signal::<ContinuousKind, f64>::continuous(move |t| signal.sample(t) + delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_time() {
        let time = linear_time();
        assert_eq!(time.sample(0.0), 0.0);
        assert_eq!(time.sample(1.0), 1.0);
        assert_eq!(time.sample(2.5), 2.5);
    }

    #[test]
    fn test_constant() {
        let always_42 = constant(42);
        assert_eq!(always_42.sample(0.0), 42);
        assert_eq!(always_42.sample(100.0), 42);
        assert_eq!(always_42.sample(-50.0), 42);
    }

    #[test]
    fn test_sine_wave() {
        let wave = sine_wave(1.0);
        assert!((wave.sample(0.0) - 0.0).abs() < 0.0001);
        assert!((wave.sample(0.25) - 1.0).abs() < 0.0001); // sin(π/2) = 1
        assert!((wave.sample(0.5) - 0.0).abs() < 0.0001);  // sin(π) = 0
    }

    #[test]
    fn test_lerp() {
        let fade = lerp(0.0, 100.0, 0.0, 1.0);
        assert_eq!(fade.sample(0.0), 0.0);
        assert_eq!(fade.sample(0.5), 50.0);
        assert_eq!(fade.sample(1.0), 100.0);

        // Beyond range
        assert_eq!(fade.sample(-1.0), 0.0);   // Before start
        assert_eq!(fade.sample(2.0), 100.0);  // After end
    }

    #[test]
    fn test_ease_in_out() {
        let time = linear_time();
        let eased = ease_in_out(time);

        // At extremes, should match linear
        assert!((eased.sample(0.0) - 0.0).abs() < 0.0001);
        assert!((eased.sample(1.0) - 1.0).abs() < 0.0001);

        // Middle should be close to linear but not exact
        let middle = eased.sample(0.5);
        assert!((middle - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_clamp() {
        let clamped = clamp(linear_time(), 0.0, 1.0);
        assert_eq!(clamped.sample(-0.5), 0.0);
        assert_eq!(clamped.sample(0.5), 0.5);
        assert_eq!(clamped.sample(1.5), 1.0);
    }

    #[test]
    fn test_scale() {
        let doubled = scale(linear_time(), 2.0);
        assert_eq!(doubled.sample(0.0), 0.0);
        assert_eq!(doubled.sample(1.0), 2.0);
        assert_eq!(doubled.sample(5.0), 10.0);
    }

    #[test]
    fn test_offset() {
        let shifted = offset(linear_time(), 10.0);
        assert_eq!(shifted.sample(0.0), 10.0);
        assert_eq!(shifted.sample(5.0), 15.0);
    }

    #[test]
    fn test_sample_interval() {
        let time = linear_time();
        let samples = time.sample_interval(0.0, 1.0, 5);
        assert_eq!(samples.len(), 5);
        assert_eq!(samples[0], 0.0);
        assert_eq!(samples[4], 1.0);
    }

    #[test]
    fn test_signal_map() {
        let time = linear_time();
        let doubled = time.map(|t| t * 2.0);
        assert_eq!(doubled.sample(1.0), 2.0);
        assert_eq!(doubled.sample(2.5), 5.0);
    }

    #[test]
    fn test_signal_compose() {
        let time1 = linear_time();
        let time2 = scale(linear_time(), 2.0);
        let composed = time1.compose(time2);

        let (t1, t2) = composed.sample(1.0);
        assert_eq!(t1, 1.0);
        assert_eq!(t2, 2.0);
    }
}
