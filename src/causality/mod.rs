//! Causality Enforcement for N-ary FRP
//!
//! This module implements Axiom A4: Causality Guarantees.
//!
//! ## Core Principle
//!
//! In functional reactive programming, causality means:
//! - **Events can only depend on past events** (no circular dependencies)
//! - **Clear temporal ordering** (earlier events cause later events)
//! - **Explicit dependencies** (causal relationships are tracked)
//!
//! ## Approach
//!
//! Rather than using complex const generic time indices, we use a practical
//! runtime system that:
//! 1. Assigns monotonic timestamps to all events
//! 2. Tracks causal dependencies explicitly
//! 3. Validates causality constraints when events are created
//! 4. Provides clear error messages for violations
//!
//! ## Example
//!
//! ```rust
//! use cim_keys::causality::{CausalTime, CausalEvent, CausalChain};
//!
//! // Create initial event (no dependencies)
//! let event1 = CausalEvent::new("UserClickedButton", vec![]);
//!
//! // Create dependent event (must happen after event1)
//! let event2 = CausalEvent::caused_by("ValidationStarted", vec![event1.id()]);
//!
//! // Build a causal chain
//! let chain = CausalChain::new()
//!     .add(event1)
//!     .add(event2);
//!
//! // Validate causality
//! assert!(chain.validate().is_ok());
//! ```

pub mod types;
pub mod validation;
pub mod chain;
pub mod helpers;

pub use types::{CausalTime, CausalEvent, CausalId, CausalDependency};
pub use validation::{CausalityValidator, CausalityError};
pub use chain::CausalChain;

/// Check if time t1 happened before time t2
///
/// # Example
///
/// ```rust
/// use cim_keys::causality::{CausalTime, happened_before};
///
/// let t1 = CausalTime::now();
/// std::thread::sleep(std::time::Duration::from_millis(10));
/// let t2 = CausalTime::now();
///
/// assert!(happened_before(t1, t2));
/// assert!(!happened_before(t2, t1));
/// ```
pub fn happened_before(t1: CausalTime, t2: CausalTime) -> bool {
    t1 < t2
}

/// Check if time t1 is concurrent with time t2
///
/// In our system, concurrent means they happened at the exact same
/// monotonic timestamp (rare in practice).
pub fn concurrent(t1: CausalTime, t2: CausalTime) -> bool {
    t1 == t2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happened_before() {
        let t1 = CausalTime::now();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let t2 = CausalTime::now();

        assert!(happened_before(t1, t2));
        assert!(!happened_before(t2, t1));
        assert!(!happened_before(t1, t1));
    }

    #[test]
    fn test_concurrent() {
        let t1 = CausalTime::now();
        let t2 = t1; // Same time

        assert!(concurrent(t1, t2));
    }
}
