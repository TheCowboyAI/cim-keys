//! Causality Type Definitions
//!
//! Core types for tracking causal relationships between events.

use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic causal time
///
/// Represents a point in causal time. Uses a global counter to ensure
/// strict ordering even when system clock is not monotonic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CausalTime(u64);

// Global monotonic counter for causality
static CAUSAL_COUNTER: AtomicU64 = AtomicU64::new(0);

impl CausalTime {
    /// Create a new causal timestamp (now)
    ///
    /// Uses a global monotonic counter to ensure strict ordering.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cim_keys::causality::CausalTime;
    ///
    /// let t1 = CausalTime::now();
    /// let t2 = CausalTime::now();
    /// assert!(t1 < t2);
    /// ```
    pub fn now() -> Self {
        CausalTime(CAUSAL_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Create a causal time from a specific value (for testing)
    ///
    /// # Warning
    ///
    /// Only use this for testing! Production code should use `now()`.
    #[cfg(test)]
    pub fn from_value(value: u64) -> Self {
        CausalTime(value)
    }

    /// Get the raw value
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Get the duration between two causal times
    ///
    /// Returns None if t2 < t1 (time travel)
    pub fn duration_since(&self, earlier: CausalTime) -> Option<u64> {
        if self.0 >= earlier.0 {
            Some(self.0 - earlier.0)
        } else {
            None
        }
    }
}

impl Default for CausalTime {
    fn default() -> Self {
        Self::now()
    }
}

/// Unique identifier for causal events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CausalId(u64);

impl CausalId {
    /// Create a new unique causal ID
    pub fn new() -> Self {
        CausalId(CAUSAL_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Get the raw value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for CausalId {
    fn default() -> Self {
        Self::new()
    }
}

/// Causal dependency relationship
///
/// Represents the fact that one event depends on another event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalDependency {
    /// The event that depends on something
    pub dependent: CausalId,
    /// The event that is depended upon (must have happened earlier)
    pub dependency: CausalId,
}

impl CausalDependency {
    /// Create a new causal dependency
    ///
    /// # Example
    ///
    /// ```rust
    /// use cim_keys::causality::{CausalId, CausalDependency};
    ///
    /// let cause = CausalId::new();
    /// let effect = CausalId::new();
    ///
    /// let dep = CausalDependency::new(effect, cause);
    /// ```
    pub fn new(dependent: CausalId, dependency: CausalId) -> Self {
        CausalDependency {
            dependent,
            dependency,
        }
    }
}

/// A causal event with explicit temporal information
///
/// Tracks both the wall-clock time and causal dependencies.
#[derive(Debug, Clone)]
pub struct CausalEvent<T> {
    /// Unique identifier for this event
    id: CausalId,
    /// When this event occurred (causal time)
    time: CausalTime,
    /// What events this event depends on (causal dependencies)
    dependencies: Vec<CausalId>,
    /// The actual event data
    data: T,
}

impl<T> CausalEvent<T> {
    /// Create a new causal event with no dependencies
    ///
    /// # Example
    ///
    /// ```rust
    /// use cim_keys::causality::CausalEvent;
    ///
    /// let event = CausalEvent::new("UserLoggedIn");
    /// assert!(event.dependencies().is_empty());
    /// ```
    pub fn new(data: T) -> Self {
        CausalEvent {
            id: CausalId::new(),
            time: CausalTime::now(),
            dependencies: Vec::new(),
            data,
        }
    }

    /// Create a new causal event with explicit dependencies
    ///
    /// # Example
    ///
    /// ```rust
    /// use cim_keys::causality::CausalEvent;
    ///
    /// let event1 = CausalEvent::new("ButtonClicked");
    /// let event2 = CausalEvent::caused_by("ValidationStarted", vec![event1.id()]);
    ///
    /// assert_eq!(event2.dependencies().len(), 1);
    /// ```
    pub fn caused_by(data: T, dependencies: Vec<CausalId>) -> Self {
        CausalEvent {
            id: CausalId::new(),
            time: CausalTime::now(),
            dependencies,
            data,
        }
    }

    /// Get the event ID
    pub fn id(&self) -> CausalId {
        self.id
    }

    /// Get the causal time
    pub fn time(&self) -> CausalTime {
        self.time
    }

    /// Get the dependencies
    pub fn dependencies(&self) -> &[CausalId] {
        &self.dependencies
    }

    /// Get the event data
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get mutable reference to event data
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Consume the event and return the data
    pub fn into_data(self) -> T {
        self.data
    }

    /// Check if this event depends on another event
    pub fn depends_on(&self, other: CausalId) -> bool {
        self.dependencies.contains(&other)
    }

    /// Add a dependency to this event
    ///
    /// # Warning
    ///
    /// This should only be used when constructing events. Adding dependencies
    /// after the fact can violate causality if not careful.
    pub fn add_dependency(&mut self, dep: CausalId) {
        if !self.dependencies.contains(&dep) {
            self.dependencies.push(dep);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_time_ordering() {
        let t1 = CausalTime::now();
        let t2 = CausalTime::now();
        let t3 = CausalTime::now();

        assert!(t1 < t2);
        assert!(t2 < t3);
        assert!(t1 < t3);
    }

    #[test]
    fn test_causal_time_duration() {
        let t1 = CausalTime::now();
        let t2 = CausalTime::now();

        let duration = t2.duration_since(t1);
        assert!(duration.is_some());
        assert!(duration.unwrap() > 0);

        // Time travel should return None
        assert!(t1.duration_since(t2).is_none());
    }

    #[test]
    fn test_causal_id_uniqueness() {
        let id1 = CausalId::new();
        let id2 = CausalId::new();
        let id3 = CausalId::new();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_causal_event_no_deps() {
        let event = CausalEvent::new("test");
        assert!(event.dependencies().is_empty());
        assert_eq!(event.data(), &"test");
    }

    #[test]
    fn test_causal_event_with_deps() {
        let event1 = CausalEvent::new("first");
        let event2 = CausalEvent::caused_by("second", vec![event1.id()]);

        assert_eq!(event2.dependencies().len(), 1);
        assert!(event2.depends_on(event1.id()));
    }

    #[test]
    fn test_causal_event_temporal_ordering() {
        let event1 = CausalEvent::new("first");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event2 = CausalEvent::new("second");

        assert!(event1.time() < event2.time());
    }

    #[test]
    fn test_causal_dependency() {
        let cause = CausalId::new();
        let effect = CausalId::new();

        let dep = CausalDependency::new(effect, cause);
        assert_eq!(dep.dependent, effect);
        assert_eq!(dep.dependency, cause);
    }

    #[test]
    fn test_add_dependency() {
        let mut event = CausalEvent::new("test");
        let dep_id = CausalId::new();

        event.add_dependency(dep_id);
        assert!(event.depends_on(dep_id));

        // Adding again should not duplicate
        event.add_dependency(dep_id);
        assert_eq!(event.dependencies().len(), 1);
    }
}
