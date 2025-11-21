//! Signal Types for N-ary FRP
//!
//! This module implements the signal type system following n-ary FRP axioms.
//!
//! ## Axiom A1: Multi-Kinded Signals
//!
//! Signals are distinguished by their temporal characteristics at the type level:
//!
//! - **Event Signals**: Discrete occurrences at specific time points
//! - **Step Signals**: Piecewise-constant values that change discretely
//! - **Continuous Signals**: Values defined at all times (smooth functions)
//!
//! ## Usage
//!
//! ```rust
//! use cim_keys::signals::{Signal, EventKind, StepKind, ContinuousKind};
//!
//! // Event signal: Button clicks occur at discrete times
//! let button_clicks: Signal<EventKind, ButtonClick> = Signal::event(vec![
//!     (0.0, ButtonClick { button_id: "generate".into() }),
//!     (1.5, ButtonClick { button_id: "export".into() }),
//! ]);
//!
//! // Step signal: Model state changes discretely
//! let model_state: Signal<StepKind, AppState> = Signal::step(AppState::default());
//!
//! // Continuous signal: Animation time flows smoothly
//! let animation_time: Signal<ContinuousKind, f32> = Signal::continuous(|t| t as f32);
//! ```
//!
//! ## Mathematical Foundations
//!
//! Each signal kind has denotational semantics:
//!
//! - `⟦Event T⟧(t) = [(t', x) | t' ≤ t, x : T]` - List of occurrences up to time t
//! - `⟦Step T⟧(t) = T` - Single value at time t (piecewise constant)
//! - `⟦Continuous T⟧(t) = T` - Value at time t (smooth function)

pub mod kinds;
pub mod vector;
pub mod continuous;

pub use kinds::{SignalKind, EventKind, StepKind, ContinuousKind};
pub use vector::{SignalVector, SignalVec2, SignalVec3, SignalVec4};
pub use continuous::{ContinuousSignal};

use std::marker::PhantomData;

/// Time representation (continuous in semantics, discrete in implementation)
///
/// Following Axiom A10, time is continuous in semantics (f64 representing ℝ)
/// but implementation uses discrete sampling.
pub type Time = f64;

/// A signal parameterized by its kind and value type
///
/// This type enforces Axiom A1 (Multi-Kinded Signals) by distinguishing
/// signals at the type level based on their temporal characteristics.
///
/// # Type Parameters
///
/// - `K`: Signal kind (EventKind, StepKind, or ContinuousKind)
/// - `T`: Value type carried by the signal
///
/// # Examples
///
/// ```rust
/// use cim_keys::signals::{Signal, EventKind, Time};
///
/// // Event signal: Discrete occurrences
/// let events = Signal::<EventKind, String>::event(vec![
///     (0.0, "start".into()),
///     (1.0, "middle".into()),
///     (2.0, "end".into()),
/// ]);
///
/// // Sample occurrences in time range
/// let occurrences = events.occurrences(0.5, 1.5);
/// assert_eq!(occurrences.len(), 1);
/// assert_eq!(occurrences[0].1, "middle");
/// ```
#[derive(Debug, Clone)]
pub struct Signal<K: SignalKind, T> {
    inner: SignalRepr<T>,
    _kind: PhantomData<K>,
}

/// Internal representation of signal data
enum SignalRepr<T> {
    /// Event signal: List of (time, value) occurrences
    Event(Vec<(Time, T)>),

    /// Step signal: Single value (current state)
    Step(T),

    /// Continuous signal: Function from time to value
    Continuous(std::sync::Arc<dyn Fn(Time) -> T + Send + Sync>),
}

impl<T> std::fmt::Debug for SignalRepr<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalRepr::Event(v) => f.debug_tuple("Event").field(v).finish(),
            SignalRepr::Step(v) => f.debug_tuple("Step").field(v).finish(),
            SignalRepr::Continuous(_) => f.debug_tuple("Continuous").field(&"<function>").finish(),
        }
    }
}

impl<T> Clone for SignalRepr<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            SignalRepr::Event(v) => SignalRepr::Event(v.clone()),
            SignalRepr::Step(v) => SignalRepr::Step(v.clone()),
            SignalRepr::Continuous(f) => SignalRepr::Continuous(f.clone()),
        }
    }
}

impl<K: SignalKind, T> Signal<K, T> {
    /// Create an event signal from a list of occurrences
    ///
    /// # Panics
    ///
    /// Panics if K is not EventKind (type system should prevent this)
    pub fn event(occurrences: Vec<(Time, T)>) -> Signal<EventKind, T> {
        Signal {
            inner: SignalRepr::Event(occurrences),
            _kind: PhantomData,
        }
    }

    /// Create a step signal with an initial value
    ///
    /// # Panics
    ///
    /// Panics if K is not StepKind (type system should prevent this)
    pub fn step(value: T) -> Signal<StepKind, T> {
        Signal {
            inner: SignalRepr::Step(value),
            _kind: PhantomData,
        }
    }

    /// Create a continuous signal from a function
    ///
    /// # Panics
    ///
    /// Panics if K is not ContinuousKind (type system should prevent this)
    pub fn continuous<F>(f: F) -> Signal<ContinuousKind, T>
    where
        F: Fn(Time) -> T + Send + Sync + 'static,
    {
        Signal {
            inner: SignalRepr::Continuous(std::sync::Arc::new(f)),
            _kind: PhantomData,
        }
    }
}

impl<T: Clone> Signal<EventKind, T> {
    /// Get all occurrences in the given time range [start, end)
    ///
    /// This operation has denotational semantics:
    /// `occurrences(start, end) = [(t, x) | start ≤ t < end]`
    pub fn occurrences(&self, start: Time, end: Time) -> Vec<(Time, T)> {
        match &self.inner {
            SignalRepr::Event(occurrences) => {
                occurrences
                    .iter()
                    .filter(|(t, _)| *t >= start && *t < end)
                    .cloned()
                    .collect()
            }
            _ => panic!("Called occurrences() on non-event signal"),
        }
    }

    /// Get all occurrences up to (and including) the given time
    pub fn occurrences_until(&self, time: Time) -> Vec<(Time, T)> {
        self.occurrences(0.0, time + 0.0001) // Include time itself
    }

    /// Count occurrences in time range
    pub fn count(&self, start: Time, end: Time) -> usize {
        self.occurrences(start, end).len()
    }
}

impl<T: Clone> Signal<StepKind, T> {
    /// Sample the signal at the given time
    ///
    /// For step signals, this returns the current value
    /// (piecewise constant between events)
    pub fn sample(&self, _time: Time) -> T {
        match &self.inner {
            SignalRepr::Step(value) => value.clone(),
            _ => panic!("Called sample() on non-step signal"),
        }
    }

    /// Update the step signal with a new value
    ///
    /// Returns a new signal with the updated value (immutable)
    pub fn with_value(&self, new_value: T) -> Self {
        Signal {
            inner: SignalRepr::Step(new_value),
            _kind: PhantomData,
        }
    }
}

impl<T: Clone> Signal<ContinuousKind, T> {
    /// Sample the continuous signal at the given time
    pub fn sample(&self, time: Time) -> T {
        match &self.inner {
            SignalRepr::Continuous(f) => f(time),
            _ => panic!("Called sample() on non-continuous signal"),
        }
    }

    /// Sample at multiple time points
    pub fn sample_many(&self, times: &[Time]) -> Vec<T> {
        times.iter().map(|&t| self.sample(t)).collect()
    }
}

// Functor instance for Signal (fmap)
impl<K: SignalKind, T> Signal<K, T> {
    /// Map a function over the signal values
    ///
    /// This implements the Functor fmap operation:
    /// `fmap f signal = signal with f applied to all values`
    ///
    /// Functor laws:
    /// - `fmap id = id`
    /// - `fmap (g ∘ f) = fmap g ∘ fmap f`
    pub fn fmap<U, F>(self, f: F) -> Signal<K, U>
    where
        F: Fn(T) -> U + Send + Sync + 'static,
        T: 'static,
    {
        let new_inner = match self.inner {
            SignalRepr::Event(occurrences) => {
                SignalRepr::Event(
                    occurrences
                        .into_iter()
                        .map(|(t, x)| (t, f(x)))
                        .collect()
                )
            }
            SignalRepr::Step(value) => {
                SignalRepr::Step(f(value))
            }
            SignalRepr::Continuous(g) => {
                SignalRepr::Continuous(std::sync::Arc::new(move |t| f(g(t))))
            }
        };

        Signal {
            inner: new_inner,
            _kind: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_signal_creation() {
        let signal = Signal::<EventKind, i32>::event(vec![
            (0.0, 1),
            (1.0, 2),
            (2.0, 3),
        ]);

        let all = signal.occurrences(0.0, 3.0);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].1, 1);
        assert_eq!(all[1].1, 2);
        assert_eq!(all[2].1, 3);
    }

    #[test]
    fn test_event_signal_time_range() {
        let signal = Signal::<EventKind, String>::event(vec![
            (0.0, "a".into()),
            (1.0, "b".into()),
            (2.0, "c".into()),
            (3.0, "d".into()),
        ]);

        let middle = signal.occurrences(1.0, 3.0);
        assert_eq!(middle.len(), 2);
        assert_eq!(middle[0].1, "b");
        assert_eq!(middle[1].1, "c");
    }

    #[test]
    fn test_step_signal() {
        let signal = Signal::<StepKind, i32>::step(42);

        assert_eq!(signal.sample(0.0), 42);
        assert_eq!(signal.sample(100.0), 42);

        let updated = signal.with_value(99);
        assert_eq!(updated.sample(0.0), 99);
    }

    #[test]
    fn test_continuous_signal() {
        let signal = Signal::<ContinuousKind, f64>::continuous(|t| t * 2.0);

        assert_eq!(signal.sample(0.0), 0.0);
        assert_eq!(signal.sample(1.0), 2.0);
        assert_eq!(signal.sample(2.5), 5.0);
    }

    #[test]
    fn test_functor_fmap() {
        // Event signal
        let events = Signal::<EventKind, i32>::event(vec![
            (0.0, 1),
            (1.0, 2),
            (2.0, 3),
        ]);

        let doubled = events.fmap(|x| x * 2);
        let result = doubled.occurrences(0.0, 3.0);
        assert_eq!(result[0].1, 2);
        assert_eq!(result[1].1, 4);
        assert_eq!(result[2].1, 6);

        // Step signal
        let step = Signal::<StepKind, i32>::step(10);
        let incremented = step.fmap(|x| x + 1);
        assert_eq!(incremented.sample(0.0), 11);
    }

    #[test]
    fn test_functor_identity_law() {
        let signal = Signal::<StepKind, i32>::step(42);
        let identity_mapped = signal.clone().fmap(|x| x);

        assert_eq!(signal.sample(0.0), identity_mapped.sample(0.0));
    }

    #[test]
    fn test_event_count() {
        let signal = Signal::<EventKind, ()>::event(vec![
            (0.0, ()),
            (0.5, ()),
            (1.0, ()),
            (2.0, ()),
        ]);

        assert_eq!(signal.count(0.0, 1.0), 2);
        assert_eq!(signal.count(0.5, 2.0), 2);
        assert_eq!(signal.count(0.0, 3.0), 4);
    }
}
