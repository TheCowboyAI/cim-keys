//! Feedback Combinator - Axiom A8
//!
//! Implements causally-sound feedback loops for event-driven aggregates.
//!
//! ## Theory
//!
//! In category theory, a feedback loop is a traced monoidal functor:
//! ```text
//! Tr^X_A,B : C(A ⊗ X, B ⊗ X) → C(A, B)
//! ```
//!
//! In practice:
//! - Input A: New events
//! - Output B: Processed results
//! - State X: Accumulated state (must be Decoupled)
//! - Function: (A, &X) → (B, X) processes event with state
//!
//! ## Causality Guarantee
//!
//! Feedback is safe when:
//! 1. State X is updated AFTER the current event is processed
//! 2. The function (A, &X) → (B, X) uses only the previous state
//! 3. This ensures output at time t depends only on inputs before t

use std::sync::{Arc, Mutex};
use std::marker::PhantomData;

/// Marker trait for types that can safely participate in feedback loops.
///
/// A type is `Decoupled` when it introduces a time delay in the feedback path:
/// - Event aggregates that update state AFTER processing
/// - Accumulators that use previous values
/// - Delayed signals that buffer values
///
/// # Safety
///
/// Only implement this for types where the feedback doesn't create instant
/// circular dependencies. The state should represent "past" values.
///
/// # Example
///
/// ```rust
/// use cim_keys::combinators::feedback::Decoupled;
///
/// #[derive(Clone)]
/// struct EventAccumulator {
///     count: u64,
///     history: Vec<String>,
/// }
///
/// // Safe: state is updated after each event, so it represents the past
/// impl Decoupled for EventAccumulator {}
/// ```
pub trait Decoupled: Clone + Send + Sync + 'static {}

/// A feedback loop that processes inputs with accumulated state.
///
/// # Type Parameters
///
/// - `A`: Input type (events, commands, etc.)
/// - `B`: Output type (results, projections, etc.)
/// - `S`: State type (must be Decoupled)
///
/// # Example
///
/// ```rust
/// use cim_keys::combinators::feedback::{feedback, Decoupled};
///
/// #[derive(Clone)]
/// struct Counter {
///     value: i32,
/// }
///
/// impl Decoupled for Counter {}
///
/// let mut counter_loop = feedback(
///     Counter { value: 0 },
///     |delta: i32, state: &Counter| {
///         let new_value = state.value + delta;
///         let new_state = Counter { value: new_value };
///         (new_value, new_state)
///     }
/// );
///
/// assert_eq!(counter_loop.process(5), 5);
/// assert_eq!(counter_loop.process(3), 8);
/// assert_eq!(counter_loop.process(-2), 6);
/// ```
pub struct FeedbackLoop<A, B, S>
where
    A: Send + Sync,
    B: Send + Sync,
    S: Decoupled,
{
    state: Arc<Mutex<S>>,
    function: Arc<dyn Fn(A, &S) -> (B, S) + Send + Sync>,
    _phantom: PhantomData<(A, B)>,
}

impl<A, B, S> FeedbackLoop<A, B, S>
where
    A: Send + Sync,
    B: Send + Sync,
    S: Decoupled,
{
    /// Create a new feedback loop with initial state.
    pub fn new<F>(initial_state: S, f: F) -> Self
    where
        F: Fn(A, &S) -> (B, S) + Send + Sync + 'static,
    {
        FeedbackLoop {
            state: Arc::new(Mutex::new(initial_state)),
            function: Arc::new(f),
            _phantom: PhantomData,
        }
    }

    /// Process a single input through the feedback loop.
    ///
    /// 1. Locks current state (previous values)
    /// 2. Applies function to get (output, new_state)
    /// 3. Updates state for next iteration
    /// 4. Returns output
    ///
    /// This ensures causality: output depends only on previous state.
    pub fn process(&mut self, input: A) -> B {
        let mut state_guard = self.state.lock().unwrap();
        let (output, new_state) = (self.function)(input, &*state_guard);
        *state_guard = new_state;
        output
    }

    /// Process multiple inputs in sequence.
    ///
    /// Each iteration uses the state from the previous iteration.
    pub fn process_many(&mut self, inputs: Vec<A>) -> Vec<B> {
        inputs.into_iter().map(|input| self.process(input)).collect()
    }

    /// Get a snapshot of the current state.
    ///
    /// This is useful for debugging or checkpointing.
    pub fn current_state(&self) -> S {
        self.state.lock().unwrap().clone()
    }

    /// Reset the state to a new value.
    ///
    /// This is useful for testing or recovery scenarios.
    pub fn reset_state(&mut self, new_state: S) {
        let mut state_guard = self.state.lock().unwrap();
        *state_guard = new_state;
    }

    /// Map the output of this feedback loop through another function.
    ///
    /// This allows composition: feedback(s, f).map(g)
    pub fn map<C, F>(self, f: F) -> FeedbackLoop<A, C, S>
    where
        A: 'static,
        B: 'static,
        C: Send + Sync,
        F: Fn(B) -> C + Send + Sync + 'static,
    {
        let original_function = self.function.clone();
        let mapped_function = move |input: A, state: &S| {
            let (output, new_state) = original_function(input, state);
            (f(output), new_state)
        };

        FeedbackLoop {
            state: self.state.clone(),
            function: Arc::new(mapped_function),
            _phantom: PhantomData,
        }
    }
}

// Allow cloning feedback loops (shares state via Arc)
impl<A, B, S> Clone for FeedbackLoop<A, B, S>
where
    A: Send + Sync,
    B: Send + Sync,
    S: Decoupled,
{
    fn clone(&self) -> Self {
        FeedbackLoop {
            state: Arc::clone(&self.state),
            function: Arc::clone(&self.function),
            _phantom: PhantomData,
        }
    }
}

/// Create a feedback loop with initial state and processing function.
///
/// # Arguments
///
/// - `initial_state`: The starting state (must be Decoupled)
/// - `f`: Function that takes (input, &state) and returns (output, new_state)
///
/// # Returns
///
/// A `FeedbackLoop` that can process inputs sequentially with state accumulation.
///
/// # Example: Event Aggregate
///
/// ```rust
/// use cim_keys::combinators::feedback::{feedback, Decoupled};
///
/// #[derive(Clone)]
/// struct Aggregate {
///     version: u64,
///     data: Vec<String>,
/// }
///
/// impl Decoupled for Aggregate {}
///
/// let mut aggregate = feedback(
///     Aggregate { version: 0, data: vec![] },
///     |event: String, state: &Aggregate| {
///         let mut new_data = state.data.clone();
///         new_data.push(event.clone());
///         let new_state = Aggregate {
///             version: state.version + 1,
///             data: new_data,
///         };
///         let summary = format!("v{}: {} events", new_state.version, new_state.data.len());
///         (summary, new_state)
///     }
/// );
///
/// assert_eq!(aggregate.process("Event1".to_string()), "v1: 1 events");
/// assert_eq!(aggregate.process("Event2".to_string()), "v2: 2 events");
/// ```
pub fn feedback<A, B, S, F>(initial_state: S, f: F) -> FeedbackLoop<A, B, S>
where
    A: Send + Sync,
    B: Send + Sync,
    S: Decoupled,
    F: Fn(A, &S) -> (B, S) + Send + Sync + 'static,
{
    FeedbackLoop::new(initial_state, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Counter {
        value: i32,
    }

    impl Decoupled for Counter {}

    #[derive(Clone)]
    struct Accumulator {
        sum: i32,
        count: usize,
    }

    impl Decoupled for Accumulator {}

    #[test]
    fn test_simple_counter() {
        let mut counter = feedback(
            Counter { value: 0 },
            |delta: i32, state: &Counter| {
                let new_value = state.value + delta;
                (new_value, Counter { value: new_value })
            }
        );

        assert_eq!(counter.process(5), 5);
        assert_eq!(counter.process(3), 8);
        assert_eq!(counter.process(-2), 6);
    }

    #[test]
    fn test_accumulator() {
        let mut acc = feedback(
            Accumulator { sum: 0, count: 0 },
            |value: i32, state: &Accumulator| {
                let new_sum = state.sum + value;
                let new_count = state.count + 1;
                let average = new_sum as f64 / new_count as f64;
                let new_state = Accumulator { sum: new_sum, count: new_count };
                (average, new_state)
            }
        );

        assert_eq!(acc.process(10), 10.0);
        assert_eq!(acc.process(20), 15.0);  // (10 + 20) / 2
        assert_eq!(acc.process(30), 20.0);  // (10 + 20 + 30) / 3
    }

    #[test]
    fn test_process_many() {
        let mut counter = feedback(
            Counter { value: 0 },
            |delta: i32, state: &Counter| {
                let new_value = state.value + delta;
                (new_value, Counter { value: new_value })
            }
        );

        let results = counter.process_many(vec![1, 2, 3, 4, 5]);
        assert_eq!(results, vec![1, 3, 6, 10, 15]);
    }

    #[test]
    fn test_current_state() {
        let mut counter = feedback(
            Counter { value: 0 },
            |delta: i32, state: &Counter| {
                let new_value = state.value + delta;
                (new_value, Counter { value: new_value })
            }
        );

        counter.process(5);
        assert_eq!(counter.current_state().value, 5);

        counter.process(3);
        assert_eq!(counter.current_state().value, 8);
    }

    #[test]
    fn test_reset_state() {
        let mut counter = feedback(
            Counter { value: 0 },
            |delta: i32, state: &Counter| {
                let new_value = state.value + delta;
                (new_value, Counter { value: new_value })
            }
        );

        counter.process(5);
        assert_eq!(counter.current_state().value, 5);

        counter.reset_state(Counter { value: 0 });
        assert_eq!(counter.process(3), 3);
    }

    #[test]
    fn test_map_combinator() {
        let counter = feedback(
            Counter { value: 0 },
            |delta: i32, state: &Counter| {
                let new_value = state.value + delta;
                (new_value, Counter { value: new_value })
            }
        );

        let mut doubled = counter.map(|x| x * 2);

        assert_eq!(doubled.process(5), 10);   // 5 * 2
        assert_eq!(doubled.process(3), 16);   // 8 * 2
    }

    #[derive(Clone)]
    struct EventLog {
        events: Vec<String>,
    }

    impl Decoupled for EventLog {}

    #[test]
    fn test_event_aggregate() {
        let mut log = feedback(
            EventLog { events: vec![] },
            |event: String, state: &EventLog| {
                let mut new_events = state.events.clone();
                new_events.push(event);
                let count = new_events.len();
                let new_state = EventLog { events: new_events };
                (count, new_state)
            }
        );

        assert_eq!(log.process("Event1".to_string()), 1);
        assert_eq!(log.process("Event2".to_string()), 2);
        assert_eq!(log.process("Event3".to_string()), 3);

        let state = log.current_state();
        assert_eq!(state.events.len(), 3);
        assert_eq!(state.events[0], "Event1");
    }

    #[test]
    fn test_clone_shares_state() {
        let mut counter1 = feedback(
            Counter { value: 0 },
            |delta: i32, state: &Counter| {
                let new_value = state.value + delta;
                (new_value, Counter { value: new_value })
            }
        );

        counter1.process(5);

        let mut counter2 = counter1.clone();
        assert_eq!(counter2.process(3), 8);  // Uses shared state: 5 + 3 = 8
        assert_eq!(counter1.current_state().value, 8);  // Both see same state
    }

    #[derive(Clone)]
    struct StateMachine {
        state: String,
        transitions: usize,
    }

    impl Decoupled for StateMachine {}

    #[test]
    fn test_state_machine() {
        let mut sm = feedback(
            StateMachine {
                state: "Init".to_string(),
                transitions: 0,
            },
            |event: &str, state: &StateMachine| {
                let new_state_name = match (state.state.as_str(), event) {
                    ("Init", "start") => "Running",
                    ("Running", "pause") => "Paused",
                    ("Paused", "resume") => "Running",
                    ("Running", "stop") => "Stopped",
                    _ => state.state.as_str(),
                };
                let new_state = StateMachine {
                    state: new_state_name.to_string(),
                    transitions: state.transitions + 1,
                };
                (new_state.state.clone(), new_state)
            }
        );

        assert_eq!(sm.process("start"), "Running");
        assert_eq!(sm.process("pause"), "Paused");
        assert_eq!(sm.process("resume"), "Running");
        assert_eq!(sm.process("stop"), "Stopped");
        assert_eq!(sm.current_state().transitions, 4);
    }
}
