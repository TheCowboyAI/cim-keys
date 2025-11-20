//! Signal Vector Types
//!
//! This module implements Axiom A2: Signal Vector Composition.
//!
//! Signal vectors represent **multiple independent signals** that can be
//! processed simultaneously without artificial ordering dependencies.
//!
//! ## N-ary Signal Functions
//!
//! Instead of functions that take single inputs and produce single outputs:
//! ```text
//! f : Signal<K1, T1> → Signal<K2, T2>
//! ```
//!
//! We have n-ary functions operating on signal vectors:
//! ```text
//! f : (Signal<K1, T1>, Signal<K2, T2>, ...) → (Signal<K3, T3>, Signal<K4, T4>, ...)
//! ```
//!
//! ## Example
//!
//! ```rust
//! use cim_keys::signals::{SignalVec2, Signal, EventKind, StepKind};
//!
//! // Two independent signals as a vector
//! let button_clicks = Signal::<EventKind, ButtonClick>::event(vec![...]);
//! let model_state = Signal::<StepKind, Model>::step(Model::default());
//!
//! let signal_vec = SignalVec2::new(button_clicks, model_state);
//!
//! // Process both simultaneously
//! let (events, state) = signal_vec.split();
//! ```

use super::{Signal, SignalKind};

/// Trait for signal vectors
///
/// Signal vectors represent tuples of independent signals that can be
/// processed without artificial ordering constraints.
pub trait SignalVector {
    /// The tuple type of this signal vector
    ///
    /// For example, SignalVec2<K1, K2, T1, T2> has:
    /// ```text
    /// Tuple = (Signal<K1, T1>, Signal<K2, T2>)
    /// ```
    type Tuple;

    /// Convert from tuple representation
    fn from_tuple(tuple: Self::Tuple) -> Self;

    /// Convert to tuple representation
    fn into_tuple(self) -> Self::Tuple;

    /// Get the arity (number of signals) in this vector
    fn arity() -> usize;
}

/// Signal vector with 2 elements
///
/// Represents a pair of independent signals that can be processed together.
///
/// # Example
///
/// ```rust
/// use cim_keys::signals::{SignalVec2, Signal, EventKind, StepKind};
///
/// let events = Signal::<EventKind, i32>::event(vec![(0.0, 1), (1.0, 2)]);
/// let state = Signal::<StepKind, String>::step("initial".into());
///
/// let vec = SignalVec2::new(events, state);
/// let (e, s) = vec.split();
/// ```
#[derive(Debug, Clone)]
pub struct SignalVec2<K1, K2, T1, T2>
where
    K1: SignalKind,
    K2: SignalKind,
{
    pub first: Signal<K1, T1>,
    pub second: Signal<K2, T2>,
}

impl<K1, K2, T1, T2> SignalVec2<K1, K2, T1, T2>
where
    K1: SignalKind,
    K2: SignalKind,
{
    /// Create a new signal vector from two signals
    pub fn new(first: Signal<K1, T1>, second: Signal<K2, T2>) -> Self {
        Self { first, second }
    }

    /// Split into individual signals
    pub fn split(self) -> (Signal<K1, T1>, Signal<K2, T2>) {
        (self.first, self.second)
    }

    /// Get references to signals without consuming
    pub fn as_ref(&self) -> (&Signal<K1, T1>, &Signal<K2, T2>) {
        (&self.first, &self.second)
    }

    /// Map a function over the first signal
    pub fn map_first<U, F>(self, f: F) -> SignalVec2<K1, K2, U, T2>
    where
        F: Fn(T1) -> U + Send + Sync + 'static,
        T1: 'static,
    {
        SignalVec2 {
            first: self.first.fmap(f),
            second: self.second,
        }
    }

    /// Map a function over the second signal
    pub fn map_second<U, F>(self, f: F) -> SignalVec2<K1, K2, T1, U>
    where
        F: Fn(T2) -> U + Send + Sync + 'static,
        T2: 'static,
    {
        SignalVec2 {
            first: self.first,
            second: self.second.fmap(f),
        }
    }
}

impl<K1, K2, T1, T2> SignalVector for SignalVec2<K1, K2, T1, T2>
where
    K1: SignalKind,
    K2: SignalKind,
{
    type Tuple = (Signal<K1, T1>, Signal<K2, T2>);

    fn from_tuple(tuple: Self::Tuple) -> Self {
        Self::new(tuple.0, tuple.1)
    }

    fn into_tuple(self) -> Self::Tuple {
        self.split()
    }

    fn arity() -> usize {
        2
    }
}

/// Signal vector with 3 elements
///
/// Represents three independent signals.
#[derive(Debug, Clone)]
pub struct SignalVec3<K1, K2, K3, T1, T2, T3>
where
    K1: SignalKind,
    K2: SignalKind,
    K3: SignalKind,
{
    pub first: Signal<K1, T1>,
    pub second: Signal<K2, T2>,
    pub third: Signal<K3, T3>,
}

impl<K1, K2, K3, T1, T2, T3> SignalVec3<K1, K2, K3, T1, T2, T3>
where
    K1: SignalKind,
    K2: SignalKind,
    K3: SignalKind,
{
    /// Create a new signal vector from three signals
    pub fn new(
        first: Signal<K1, T1>,
        second: Signal<K2, T2>,
        third: Signal<K3, T3>,
    ) -> Self {
        Self { first, second, third }
    }

    /// Split into individual signals
    pub fn split(self) -> (Signal<K1, T1>, Signal<K2, T2>, Signal<K3, T3>) {
        (self.first, self.second, self.third)
    }

    /// Get references to signals
    pub fn as_ref(&self) -> (&Signal<K1, T1>, &Signal<K2, T2>, &Signal<K3, T3>) {
        (&self.first, &self.second, &self.third)
    }
}

impl<K1, K2, K3, T1, T2, T3> SignalVector for SignalVec3<K1, K2, K3, T1, T2, T3>
where
    K1: SignalKind,
    K2: SignalKind,
    K3: SignalKind,
{
    type Tuple = (Signal<K1, T1>, Signal<K2, T2>, Signal<K3, T3>);

    fn from_tuple(tuple: Self::Tuple) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }

    fn into_tuple(self) -> Self::Tuple {
        self.split()
    }

    fn arity() -> usize {
        3
    }
}

/// Signal vector with 4 elements
///
/// Represents four independent signals.
#[derive(Debug, Clone)]
pub struct SignalVec4<K1, K2, K3, K4, T1, T2, T3, T4>
where
    K1: SignalKind,
    K2: SignalKind,
    K3: SignalKind,
    K4: SignalKind,
{
    pub first: Signal<K1, T1>,
    pub second: Signal<K2, T2>,
    pub third: Signal<K3, T3>,
    pub fourth: Signal<K4, T4>,
}

impl<K1, K2, K3, K4, T1, T2, T3, T4> SignalVec4<K1, K2, K3, K4, T1, T2, T3, T4>
where
    K1: SignalKind,
    K2: SignalKind,
    K3: SignalKind,
    K4: SignalKind,
{
    /// Create a new signal vector from four signals
    pub fn new(
        first: Signal<K1, T1>,
        second: Signal<K2, T2>,
        third: Signal<K3, T3>,
        fourth: Signal<K4, T4>,
    ) -> Self {
        Self {
            first,
            second,
            third,
            fourth,
        }
    }

    /// Split into individual signals
    pub fn split(
        self,
    ) -> (
        Signal<K1, T1>,
        Signal<K2, T2>,
        Signal<K3, T3>,
        Signal<K4, T4>,
    ) {
        (self.first, self.second, self.third, self.fourth)
    }

    /// Get references to signals
    pub fn as_ref(
        &self,
    ) -> (
        &Signal<K1, T1>,
        &Signal<K2, T2>,
        &Signal<K3, T3>,
        &Signal<K4, T4>,
    ) {
        (&self.first, &self.second, &self.third, &self.fourth)
    }
}

impl<K1, K2, K3, K4, T1, T2, T3, T4> SignalVector
    for SignalVec4<K1, K2, K3, K4, T1, T2, T3, T4>
where
    K1: SignalKind,
    K2: SignalKind,
    K3: SignalKind,
    K4: SignalKind,
{
    type Tuple = (
        Signal<K1, T1>,
        Signal<K2, T2>,
        Signal<K3, T3>,
        Signal<K4, T4>,
    );

    fn from_tuple(tuple: Self::Tuple) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2, tuple.3)
    }

    fn into_tuple(self) -> Self::Tuple {
        self.split()
    }

    fn arity() -> usize {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::{EventKind, StepKind};

    #[test]
    fn test_signal_vec2_creation() {
        let events = Signal::<EventKind, i32>::event(vec![(0.0, 1), (1.0, 2)]);
        let state = Signal::<StepKind, String>::step("test".into());

        let vec = SignalVec2::new(events, state);
        assert_eq!(SignalVec2::<EventKind, StepKind, i32, String>::arity(), 2);

        let (e, s) = vec.split();
        assert_eq!(e.count(0.0, 2.0), 2);
        assert_eq!(s.sample(0.0), "test");
    }

    #[test]
    fn test_signal_vec2_map() {
        let events = Signal::<EventKind, i32>::event(vec![(0.0, 1), (1.0, 2)]);
        let state = Signal::<StepKind, i32>::step(10);

        let vec = SignalVec2::new(events, state);

        // Map over first signal
        let vec2 = vec.map_first(|x| x * 2);
        let (e, _) = vec2.split();
        let occurrences = e.occurrences(0.0, 2.0);
        assert_eq!(occurrences[0].1, 2);
        assert_eq!(occurrences[1].1, 4);
    }

    #[test]
    fn test_signal_vec3() {
        let e1 = Signal::<EventKind, i32>::event(vec![(0.0, 1)]);
        let e2 = Signal::<EventKind, i32>::event(vec![(1.0, 2)]);
        let s = Signal::<StepKind, String>::step("test".into());

        let vec = SignalVec3::new(e1, e2, s);
        assert_eq!(
            SignalVec3::<EventKind, EventKind, StepKind, i32, i32, String>::arity(),
            3
        );

        let (events1, events2, state) = vec.split();
        assert_eq!(events1.count(0.0, 1.0), 1);
        assert_eq!(events2.count(0.0, 2.0), 1);
        assert_eq!(state.sample(0.0), "test");
    }

    #[test]
    fn test_signal_vec4() {
        let e1 = Signal::<EventKind, ()>::event(vec![(0.0, ())]);
        let e2 = Signal::<EventKind, ()>::event(vec![(1.0, ())]);
        let e3 = Signal::<EventKind, ()>::event(vec![(2.0, ())]);
        let s = Signal::<StepKind, i32>::step(42);

        let _vec = SignalVec4::new(e1, e2, e3, s);
        assert_eq!(
            SignalVec4::<EventKind, EventKind, EventKind, StepKind, (), (), (), i32>::arity(),
            4
        );
    }

    #[test]
    fn test_signal_vector_trait() {
        let e = Signal::<EventKind, i32>::event(vec![]);
        let s = Signal::<StepKind, String>::step("".into());

        let vec = SignalVec2::new(e.clone(), s.clone());
        let tuple = vec.into_tuple();
        let vec2 = SignalVec2::from_tuple(tuple);

        let (e2, s2) = vec2.split();
        assert_eq!(e2.count(0.0, 10.0), e.count(0.0, 10.0));
        assert_eq!(s2.sample(0.0), s.sample(0.0));
    }
}
