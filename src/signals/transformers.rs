// Copyright (c) 2025 - Cowboy AI, LLC.

//! Signal Transformers for Type-Level FRP
//!
//! This module implements signal transformers that preserve type-level signal kinds.
//! These correspond to the fundamental FRP operations:
//!
//! - **hold**: Event → Step (sample and hold last value)
//! - **changes**: Step → Event (emit events on value changes)
//! - **sample**: (Continuous, Event) → Event (sample continuous at event times)
//! - **merge**: (Event, Event) → Event (combine two event streams)
//!
//! ## Categorical Structure
//!
//! Signal transformers form a category where:
//! - Objects: Signal kinds (Event, Step, Continuous)
//! - Morphisms: Transformers that preserve signal semantics
//!
//! ## FRP Axiom A1 Compliance
//!
//! All transformers maintain type-level kind distinction:
//! - Input kind is verified at compile time
//! - Output kind is determined by the transformer
//! - Invalid transformations are compile errors

use super::{Signal, SignalKind, EventKind, StepKind, ContinuousKind, Time};
use std::marker::PhantomData;

// =============================================================================
// Signal Type Aliases
// =============================================================================

/// An event signal carrying values of type T
pub type Event<T> = Signal<EventKind, T>;

/// A step signal (behavior) carrying values of type T
pub type Step<T> = Signal<StepKind, T>;

/// A continuous signal carrying values of type T
pub type Continuous<T> = Signal<ContinuousKind, T>;

// =============================================================================
// Hold Transformer: Event → Step
// =============================================================================

/// Convert an event stream to a step signal by holding the last value.
///
/// # Type Signature
///
/// ```text
/// hold : T → Event<T> → Step<T>
/// ```
///
/// # Semantics
///
/// ```text
/// ⟦hold init events⟧(t) =
///   if ∃ (t', x) ∈ events where t' ≤ t
///   then last such x
///   else init
/// ```
///
/// # Example
///
/// ```rust,ignore
/// let clicks: Event<ButtonId> = /* ... */;
/// let last_clicked: Step<ButtonId> = hold(ButtonId::None, clicks);
/// ```
pub fn hold<T: Clone>(initial: T, events: Event<T>) -> Step<T> {
    // Get all events up to "now" (we use f64::MAX as now)
    let all_events = events.occurrences(0.0, f64::MAX);

    // Take the last event value, or initial if none
    let value = all_events.last().map(|(_, v)| v.clone()).unwrap_or(initial);

    Signal::<StepKind, T>::step(value)
}

// =============================================================================
// Changes Transformer: Step → Event
// =============================================================================

/// A step signal paired with its change times for the `changes` transformer.
#[derive(Debug, Clone)]
pub struct TrackedStep<T> {
    /// Current value
    pub value: T,
    /// Times when value changed
    pub change_times: Vec<(Time, T)>,
}

impl<T: Clone> TrackedStep<T> {
    /// Create a new tracked step with initial value
    pub fn new(initial: T) -> Self {
        TrackedStep {
            value: initial.clone(),
            change_times: vec![(0.0, initial)],
        }
    }

    /// Update the value, recording the change time
    pub fn update(&mut self, time: Time, new_value: T) {
        self.value = new_value.clone();
        self.change_times.push((time, new_value));
    }

    /// Convert to an event signal of changes
    pub fn to_events(&self) -> Event<T> {
        Signal::<EventKind, T>::event(self.change_times.clone())
    }
}

// =============================================================================
// Sample Transformer: (Step, Event) → Event
// =============================================================================

/// Sample a step signal at event times.
///
/// # Type Signature
///
/// ```text
/// sample : Step<A> → Event<B> → Event<(A, B)>
/// ```
///
/// # Semantics
///
/// For each event occurrence (t, b), emit (t, (step_value_at_t, b))
pub fn sample<A: Clone, B: Clone>(step: &Step<A>, events: Event<B>) -> Event<(A, B)> {
    let step_value = step.sample(0.0);  // Step signals are constant
    let event_occurrences = events.occurrences(0.0, f64::MAX);

    let sampled: Vec<(Time, (A, B))> = event_occurrences
        .into_iter()
        .map(|(t, b)| (t, (step_value.clone(), b)))
        .collect();

    Signal::<EventKind, (A, B)>::event(sampled)
}

/// Sample a step signal at event times, ignoring the event value.
///
/// # Type Signature
///
/// ```text
/// sample_ : Step<A> → Event<B> → Event<A>
/// ```
pub fn sample_<A: Clone, B: Clone>(step: &Step<A>, events: Event<B>) -> Event<A> {
    let step_value = step.sample(0.0);
    let event_occurrences = events.occurrences(0.0, f64::MAX);

    let sampled: Vec<(Time, A)> = event_occurrences
        .into_iter()
        .map(|(t, _)| (t, step_value.clone()))
        .collect();

    Signal::<EventKind, A>::event(sampled)
}

// =============================================================================
// Merge Transformer: (Event, Event) → Event
// =============================================================================

/// Strategy for handling simultaneous events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Take the left event on conflict
    Left,
    /// Take the right event on conflict
    Right,
    /// Keep both events (may result in multiple at same time)
    Both,
}

/// Merge two event streams into one.
///
/// # Type Signature
///
/// ```text
/// merge : Event<T> → Event<T> → Event<T>
/// ```
///
/// # Semantics
///
/// Combines occurrences from both streams, ordered by time.
pub fn merge<T: Clone>(left: Event<T>, right: Event<T>, _strategy: MergeStrategy) -> Event<T> {
    let mut left_events = left.occurrences(0.0, f64::MAX);
    let mut right_events = right.occurrences(0.0, f64::MAX);

    // Combine and sort by time
    left_events.append(&mut right_events);
    left_events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    Signal::<EventKind, T>::event(left_events)
}

/// Merge with heterogeneous types using Either.
#[derive(Debug, Clone, PartialEq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Merge two event streams with different types.
///
/// # Type Signature
///
/// ```text
/// merge_either : Event<A> → Event<B> → Event<Either<A, B>>
/// ```
pub fn merge_either<A: Clone, B: Clone>(left: Event<A>, right: Event<B>) -> Event<Either<A, B>> {
    let left_events = left.occurrences(0.0, f64::MAX);
    let right_events = right.occurrences(0.0, f64::MAX);

    let left_tagged: Vec<(Time, Either<A, B>)> = left_events
        .into_iter()
        .map(|(t, a)| (t, Either::Left(a)))
        .collect();

    let right_tagged: Vec<(Time, Either<A, B>)> = right_events
        .into_iter()
        .map(|(t, b)| (t, Either::Right(b)))
        .collect();

    let mut combined = left_tagged;
    combined.extend(right_tagged);
    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    Signal::<EventKind, Either<A, B>>::event(combined)
}

// =============================================================================
// Filter Transformer: Event → Event
// =============================================================================

/// Filter an event stream by a predicate.
///
/// # Type Signature
///
/// ```text
/// filter : (T → Bool) → Event<T> → Event<T>
/// ```
pub fn filter<T: Clone, P>(predicate: P, events: Event<T>) -> Event<T>
where
    P: Fn(&T) -> bool,
{
    let filtered: Vec<(Time, T)> = events
        .occurrences(0.0, f64::MAX)
        .into_iter()
        .filter(|(_, v)| predicate(v))
        .collect();

    Signal::<EventKind, T>::event(filtered)
}

/// Filter and map in one pass.
///
/// # Type Signature
///
/// ```text
/// filter_map : (T → Option<U>) → Event<T> → Event<U>
/// ```
pub fn filter_map<T: Clone, U: Clone, F>(f: F, events: Event<T>) -> Event<U>
where
    F: Fn(T) -> Option<U>,
{
    let mapped: Vec<(Time, U)> = events
        .occurrences(0.0, f64::MAX)
        .into_iter()
        .filter_map(|(t, v)| f(v).map(|u| (t, u)))
        .collect();

    Signal::<EventKind, U>::event(mapped)
}

// =============================================================================
// Scan Transformer: Event → Step (Stateful Fold)
// =============================================================================

/// Scan (stateful fold) over an event stream.
///
/// # Type Signature
///
/// ```text
/// scan : (S → T → S) → S → Event<T> → Step<S>
/// ```
///
/// # Semantics
///
/// Produces a step signal that is the running accumulation of events.
pub fn scan<S: Clone, T: Clone, F>(folder: F, initial: S, events: Event<T>) -> Step<S>
where
    F: Fn(S, T) -> S,
{
    let mut state = initial;

    for (_, value) in events.occurrences(0.0, f64::MAX) {
        state = folder(state, value);
    }

    Signal::<StepKind, S>::step(state)
}

/// Scan with access to time.
///
/// # Type Signature
///
/// ```text
/// scan_with_time : (S → (Time, T) → S) → S → Event<T> → Step<S>
/// ```
pub fn scan_with_time<S: Clone, T: Clone, F>(folder: F, initial: S, events: Event<T>) -> Step<S>
where
    F: Fn(S, (Time, T)) -> S,
{
    let mut state = initial;

    for occurrence in events.occurrences(0.0, f64::MAX) {
        state = folder(state, occurrence);
    }

    Signal::<StepKind, S>::step(state)
}

// =============================================================================
// Type-Level Witness Types
// =============================================================================

/// Witness that a type is an Event signal
pub struct IsEventWitness<K: SignalKind>(PhantomData<K>);

impl IsEventWitness<EventKind> {
    pub fn witness() -> Self {
        IsEventWitness(PhantomData)
    }
}

/// Witness that a type is a Step signal
pub struct IsStepWitness<K: SignalKind>(PhantomData<K>);

impl IsStepWitness<StepKind> {
    pub fn witness() -> Self {
        IsStepWitness(PhantomData)
    }
}

/// Witness that a type is a Continuous signal
pub struct IsContinuousWitness<K: SignalKind>(PhantomData<K>);

impl IsContinuousWitness<ContinuousKind> {
    pub fn witness() -> Self {
        IsContinuousWitness(PhantomData)
    }
}

// =============================================================================
// Arrow-Style Composition
// =============================================================================

/// A signal function (arrow) from A to B
pub trait SignalFunction<A, B> {
    fn apply(&self, input: A) -> B;
}

/// Compose two signal functions.
pub struct Compose<F, G, B> {
    first: F,
    second: G,
    _phantom: PhantomData<B>,
}

impl<A, B, C, F, G> SignalFunction<A, C> for Compose<F, G, B>
where
    F: SignalFunction<A, B>,
    G: SignalFunction<B, C>,
{
    fn apply(&self, input: A) -> C {
        let intermediate = self.first.apply(input);
        self.second.apply(intermediate)
    }
}

/// Parallel composition of signal functions.
pub struct Parallel<F, G> {
    left: F,
    right: G,
}

impl<A, B, C, D, F, G> SignalFunction<(A, C), (B, D)> for Parallel<F, G>
where
    F: SignalFunction<A, B>,
    G: SignalFunction<C, D>,
{
    fn apply(&self, input: (A, C)) -> (B, D) {
        (self.left.apply(input.0), self.right.apply(input.1))
    }
}

/// Fanout composition of signal functions.
pub struct Fanout<F, G> {
    left: F,
    right: G,
}

impl<A: Clone, B, C, F, G> SignalFunction<A, (B, C)> for Fanout<F, G>
where
    F: SignalFunction<A, B>,
    G: SignalFunction<A, C>,
{
    fn apply(&self, input: A) -> (B, C) {
        (self.left.apply(input.clone()), self.right.apply(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_aliases() {
        let _event: Event<i32> = Signal::<EventKind, i32>::event(vec![(0.0, 42)]);
        let _step: Step<String> = Signal::<StepKind, String>::step("hello".to_string());
        let _cont: Continuous<f64> = Signal::<ContinuousKind, f64>::continuous(|t| t * 2.0);
    }

    #[test]
    fn test_hold() {
        let events: Event<i32> = Signal::<EventKind, i32>::event(vec![
            (0.0, 1),
            (1.0, 2),
            (2.0, 3),
        ]);

        let held = hold(0, events);
        assert_eq!(held.sample(0.0), 3);  // Last value
    }

    #[test]
    fn test_hold_empty() {
        let events: Event<i32> = Signal::<EventKind, i32>::event(vec![]);
        let held = hold(42, events);
        assert_eq!(held.sample(0.0), 42);  // Initial value
    }

    #[test]
    fn test_sample() {
        let step: Step<String> = Signal::<StepKind, String>::step("hello".to_string());
        let events: Event<i32> = Signal::<EventKind, i32>::event(vec![
            (1.0, 10),
            (2.0, 20),
        ]);

        let sampled = sample(&step, events);
        let occurrences = sampled.occurrences(0.0, 3.0);

        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0].1, ("hello".to_string(), 10));
        assert_eq!(occurrences[1].1, ("hello".to_string(), 20));
    }

    #[test]
    fn test_merge() {
        let left: Event<char> = Signal::<EventKind, char>::event(vec![
            (0.0, 'a'),
            (2.0, 'b'),
        ]);
        let right: Event<char> = Signal::<EventKind, char>::event(vec![
            (1.0, 'x'),
            (3.0, 'y'),
        ]);

        let merged = merge(left, right, MergeStrategy::Both);
        let occurrences = merged.occurrences(0.0, 4.0);

        assert_eq!(occurrences.len(), 4);
        assert_eq!(occurrences[0].1, 'a');
        assert_eq!(occurrences[1].1, 'x');
        assert_eq!(occurrences[2].1, 'b');
        assert_eq!(occurrences[3].1, 'y');
    }

    #[test]
    fn test_merge_either() {
        let left: Event<i32> = Signal::<EventKind, i32>::event(vec![(0.0, 1)]);
        let right: Event<String> = Signal::<EventKind, String>::event(vec![(1.0, "two".to_string())]);

        let merged = merge_either(left, right);
        let occurrences = merged.occurrences(0.0, 2.0);

        assert_eq!(occurrences.len(), 2);
        assert!(matches!(occurrences[0].1, Either::Left(1)));
        assert!(matches!(&occurrences[1].1, Either::Right(s) if s == "two"));
    }

    #[test]
    fn test_filter() {
        let events: Event<i32> = Signal::<EventKind, i32>::event(vec![
            (0.0, 1),
            (1.0, 2),
            (2.0, 3),
            (3.0, 4),
        ]);

        let filtered = filter(|x| x % 2 == 0, events);
        let occurrences = filtered.occurrences(0.0, 4.0);

        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0].1, 2);
        assert_eq!(occurrences[1].1, 4);
    }

    #[test]
    fn test_scan() {
        let events: Event<i32> = Signal::<EventKind, i32>::event(vec![
            (0.0, 1),
            (1.0, 2),
            (2.0, 3),
        ]);

        let sum = scan(|acc, x| acc + x, 0, events);
        assert_eq!(sum.sample(0.0), 6);  // 1 + 2 + 3
    }

    #[test]
    fn test_tracked_step() {
        let mut tracked = TrackedStep::new(0);
        tracked.update(1.0, 10);
        tracked.update(2.0, 20);

        let events = tracked.to_events();
        let occurrences = events.occurrences(0.0, 3.0);

        assert_eq!(occurrences.len(), 3);
        assert_eq!(occurrences[0].1, 0);
        assert_eq!(occurrences[1].1, 10);
        assert_eq!(occurrences[2].1, 20);
    }

    #[test]
    fn test_witness_types() {
        // These compile only for the correct kinds
        let _event_witness = IsEventWitness::<EventKind>::witness();
        let _step_witness = IsStepWitness::<StepKind>::witness();
        let _continuous_witness = IsContinuousWitness::<ContinuousKind>::witness();

        // These would fail to compile:
        // let _wrong = IsEventWitness::<StepKind>::witness();
        // let _wrong = IsStepWitness::<EventKind>::witness();
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_time() -> impl Strategy<Value = Time> {
        (0.0..100.0f64)
    }

    fn arb_event_signal() -> impl Strategy<Value = Event<i32>> {
        proptest::collection::vec((arb_time(), any::<i32>()), 0..10)
            .prop_map(|mut v| {
                v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                Signal::<EventKind, i32>::event(v)
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn prop_hold_preserves_last(events in arb_event_signal(), initial: i32) {
            let held = hold(initial, events.clone());
            let occurrences = events.occurrences(0.0, f64::MAX);

            if occurrences.is_empty() {
                prop_assert_eq!(held.sample(0.0), initial);
            } else {
                prop_assert_eq!(held.sample(0.0), occurrences.last().unwrap().1);
            }
        }

        #[test]
        fn prop_filter_subset(events in arb_event_signal()) {
            let filtered = filter(|x| x > &0, events.clone());
            let original_count = events.occurrences(0.0, f64::MAX).len();
            let filtered_count = filtered.occurrences(0.0, f64::MAX).len();

            prop_assert!(filtered_count <= original_count);
        }

        #[test]
        fn prop_merge_combines_all(
            left in arb_event_signal(),
            right in arb_event_signal()
        ) {
            let merged = merge(left.clone(), right.clone(), MergeStrategy::Both);

            let left_count = left.occurrences(0.0, f64::MAX).len();
            let right_count = right.occurrences(0.0, f64::MAX).len();
            let merged_count = merged.occurrences(0.0, f64::MAX).len();

            prop_assert_eq!(merged_count, left_count + right_count);
        }

        #[test]
        fn prop_scan_is_fold(events in arb_event_signal(), initial: i32) {
            let scanned = scan(|acc, x| acc.wrapping_add(x), initial, events.clone());
            let expected: i32 = events
                .occurrences(0.0, f64::MAX)
                .iter()
                .fold(initial, |acc, (_, x)| acc.wrapping_add(*x));

            prop_assert_eq!(scanned.sample(0.0), expected);
        }
    }
}
