//! Feedback Combinators for N-ary FRP
//!
//! This module implements Axiom A8: Feedback Loops with causality guarantees.
//!
//! ## Core Principle
//!
//! Feedback loops allow signals to depend on their own past values in a
//! causally sound way. The key insight is that feedback must be **decoupled**
//! through a time delay or state accumulator to prevent infinite loops.
//!
//! ## Decoupled Feedback
//!
//! For feedback to be valid:
//! 1. The feedback path must introduce a time delay (event → next event)
//! 2. Or use an accumulator that updates after evaluation
//! 3. This ensures causality: output at time t can only depend on inputs before t
//!
//! ## Example: Event-Driven Aggregate
//!
//! ```rust
//! use cim_keys::combinators::feedback::{feedback, Decoupled};
//! use cim_keys::signals::{Signal, EventKind};
//!
//! // State that introduces time delay (previous event → current state)
//! #[derive(Clone)]
//! struct AggregateState {
//!     version: u64,
//!     data: String,
//! }
//!
//! // Mark as decoupled because state updates AFTER event processing
//! impl Decoupled for AggregateState {}
//!
//! // Create feedback loop: events + state → new events + updated state
//! let process_event = feedback(
//!     AggregateState { version: 0, data: String::new() },
//!     |event: String, state: &AggregateState| {
//!         // Process event with current state
//!         let new_state = AggregateState {
//!             version: state.version + 1,
//!             data: format!("{} + {}", state.data, event),
//!         };
//!         let output = format!("Processed: {} at v{}", event, new_state.version);
//!         (output, new_state)
//!     }
//! );
//! ```

pub mod feedback;

pub use feedback::{Decoupled, feedback, FeedbackLoop};
