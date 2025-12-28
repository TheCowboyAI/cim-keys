//! Signal Kind Type System
//!
//! This module implements Axiom A1: Multi-Kinded Signals.
//!
//! Signals are distinguished by their temporal characteristics at the type level:
//!
//! - **EventKind**: Discrete occurrences at specific time points
//! - **StepKind**: Piecewise-constant values that change discretely
//! - **ContinuousKind**: Values defined at all times (smooth functions)
//!
//! ## Denotational Semantics
//!
//! Each kind has a mathematical semantics:
//!
//! ```text
//! ⟦Event T⟧(t) = [(t', x) | t' ≤ t, x : T]
//!     Interpretation: List of occurrences up to time t
//!
//! ⟦Step T⟧(t) = T
//!     Interpretation: Single value at time t (constant between changes)
//!
//! ⟦Continuous T⟧(t) = T
//!     Interpretation: Smooth function of time
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::signals::{SignalKind, EventKind, StepKind, ContinuousKind};
//!
//! fn process_event<K: SignalKind>(signal: Signal<K, Data>) {
//!     // Type-safe: Kind is known at compile time
//! }
//! ```

use super::Time;
use std::fmt;

/// Trait for signal kinds
///
/// This trait is sealed - only EventKind, StepKind, and ContinuousKind
/// can implement it. This ensures type safety and prevents invalid signal kinds.
pub trait SignalKind: private::Sealed {
    /// The denotational semantics of this signal kind
    ///
    /// - EventKind: Vec<(Time, T)> - list of occurrences
    /// - StepKind: T - single value (piecewise constant)
    /// - ContinuousKind: fn(Time) -> T - smooth function
    type Semantics<T>;

    /// Human-readable name for this signal kind
    fn kind_name() -> &'static str;

    /// Whether this kind represents discrete or continuous time
    fn is_discrete() -> bool;
}

// Sealed trait pattern to prevent external implementations
mod private {
    pub trait Sealed {}
}

/// Event signal kind: Discrete occurrences at specific time points
///
/// # Semantics
///
/// An event signal ◇T represents a sequence of occurrences:
/// ```text
/// ⟦Event T⟧(t) = [(t₁, x₁), (t₂, x₂), ...] where tᵢ ≤ t
/// ```
///
/// # Examples
///
/// - Button clicks
/// - Key presses
/// - Domain events (KeyGenerated, CertificateSigned)
/// - NATS messages received
///
/// # Categorical Interpretation
///
/// Event signals correspond to the ◇ (diamond) functor in the paper:
/// ```text
/// ◇B = 1 ▷''_W B
/// ```
/// A process with no continuous part, only a terminal event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventKind;

impl private::Sealed for EventKind {}

impl SignalKind for EventKind {
    type Semantics<T> = Vec<(Time, T)>;

    fn kind_name() -> &'static str {
        "Event"
    }

    fn is_discrete() -> bool {
        true
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Event")
    }
}

/// Step signal kind: Piecewise-constant values that change discretely
///
/// # Semantics
///
/// A step signal represents a value that changes only at discrete time points
/// but remains defined between changes:
/// ```text
/// ⟦Step T⟧(t) = x where x is the value at the most recent change ≤ t
/// ```
///
/// # Examples
///
/// - Application state (Model in MVI)
/// - Aggregate state (projection of events)
/// - Configuration values
/// - UI state
///
/// # Categorical Interpretation
///
/// Step signals are derived from event signals via the `hold` operator:
/// ```text
/// hold : ◇T → □T
/// ```
/// Takes an event stream and produces a behavior that holds the last value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepKind;

impl private::Sealed for StepKind {}

impl SignalKind for StepKind {
    type Semantics<T> = T;

    fn kind_name() -> &'static str {
        "Step"
    }

    fn is_discrete() -> bool {
        true
    }
}

impl fmt::Display for StepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Step")
    }
}

/// Continuous signal kind: Values defined at all times (smooth functions)
///
/// # Semantics
///
/// A continuous signal □T represents a time-varying value defined at all times:
/// ```text
/// ⟦Continuous T⟧ : Time → T
/// ```
///
/// # Examples
///
/// - Animation time
/// - Mouse position (interpolated)
/// - System metrics (CPU, memory)
/// - Analog sensor readings
///
/// # Categorical Interpretation
///
/// Continuous signals correspond to behaviors (□ functor):
/// ```text
/// □A = A ▷''_∞ 0
/// ```
/// A process with continuous part A that never terminates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContinuousKind;

impl private::Sealed for ContinuousKind {}

impl SignalKind for ContinuousKind {
    type Semantics<T> = std::sync::Arc<dyn Fn(Time) -> T + Send + Sync>;

    fn kind_name() -> &'static str {
        "Continuous"
    }

    fn is_discrete() -> bool {
        false
    }
}

impl fmt::Display for ContinuousKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Continuous")
    }
}

/// Type-level predicate: Is this kind an event?
pub trait IsEvent: SignalKind {}
impl IsEvent for EventKind {}

/// Type-level predicate: Is this kind discrete (Event or Step)?
pub trait IsDiscrete: SignalKind {}
impl IsDiscrete for EventKind {}
impl IsDiscrete for StepKind {}

/// Type-level predicate: Is this kind continuous?
pub trait IsContinuous: SignalKind {}
impl IsContinuous for ContinuousKind {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_kind_names() {
        assert_eq!(EventKind::kind_name(), "Event");
        assert_eq!(StepKind::kind_name(), "Step");
        assert_eq!(ContinuousKind::kind_name(), "Continuous");
    }

    #[test]
    fn test_signal_kind_discrete() {
        assert!(EventKind::is_discrete());
        assert!(StepKind::is_discrete());
        assert!(!ContinuousKind::is_discrete());
    }

    #[test]
    fn test_signal_kind_display() {
        assert_eq!(format!("{}", EventKind), "Event");
        assert_eq!(format!("{}", StepKind), "Step");
        assert_eq!(format!("{}", ContinuousKind), "Continuous");
    }

    #[test]
    fn test_signal_kind_equality() {
        // Can only compare same types
        assert_eq!(EventKind, EventKind);
        assert_eq!(StepKind, StepKind);
        assert_eq!(ContinuousKind, ContinuousKind);

        // Different types cannot be compared (would be compile error)
        // assert_ne!(EventKind, StepKind);  // ERROR: mismatched types
    }

    // Type-level tests (compile-time verification)
    #[allow(dead_code)]
    fn type_level_tests() {
        // EventKind implements IsEvent and IsDiscrete
        fn require_event<K: IsEvent>() {}
        fn require_discrete<K: IsDiscrete>() {}
        fn require_continuous<K: IsContinuous>() {}

        require_event::<EventKind>();
        require_discrete::<EventKind>();
        require_discrete::<StepKind>();
        require_continuous::<ContinuousKind>();

        // These would fail to compile:
        // require_event::<StepKind>();  // StepKind is not an event
        // require_continuous::<EventKind>();  // EventKind is not continuous
    }
}
