//! Causal Chain Builder
//!
//! Provides a builder API for constructing validated causal chains of events.

use super::types::{CausalEvent, CausalId, CausalTime};
use super::validation::{CausalityValidator, CausalityError};
use std::collections::HashMap;

/// A validated chain of causally-ordered events
///
/// The chain maintains the invariant that all events respect causality:
/// - Events only depend on earlier events
/// - No circular dependencies
/// - All dependencies are present
#[derive(Debug, Clone)]
pub struct CausalChain<T> {
    events: Vec<CausalEvent<T>>,
    time_map: HashMap<CausalId, CausalTime>,
}

impl<T> CausalChain<T> {
    /// Create a new empty causal chain
    ///
    /// # Example
    ///
    /// ```rust
    /// use cim_keys::causality::CausalChain;
    ///
    /// let chain = CausalChain::<String>::new();
    /// assert_eq!(chain.len(), 0);
    /// ```
    pub fn new() -> Self {
        CausalChain {
            events: Vec::new(),
            time_map: HashMap::new(),
        }
    }

    /// Add an event to the chain
    ///
    /// Validates causality before adding. Returns an error if the event
    /// would violate causality constraints.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cim_keys::causality::{CausalChain, CausalEvent};
    ///
    /// let mut chain = CausalChain::new();
    /// let event = CausalEvent::new("test");
    /// assert!(chain.add(event).is_ok());
    /// ```
    pub fn add(mut self, event: CausalEvent<T>) -> Result<Self, CausalityError> {
        // Validate against existing events
        CausalityValidator::validate_event(&event, &self.time_map)?;

        // Add to chain
        self.time_map.insert(event.id(), event.time());
        self.events.push(event);

        Ok(self)
    }

    /// Try to add an event, returning the chain on success or the event on failure
    ///
    /// This is useful when you want to handle errors without losing the event.
    pub fn try_add(
        mut self,
        event: CausalEvent<T>,
    ) -> Result<Self, (Self, CausalEvent<T>, CausalityError)> {
        match CausalityValidator::validate_event(&event, &self.time_map) {
            Ok(()) => {
                self.time_map.insert(event.id(), event.time());
                self.events.push(event);
                Ok(self)
            }
            Err(e) => Err((self, event, e)),
        }
    }

    /// Validate the entire chain
    ///
    /// Checks both temporal ordering and acyclicity.
    pub fn validate(&self) -> Result<(), CausalityError> {
        CausalityValidator::validate_events(&self.events)
    }

    /// Get the number of events in the chain
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get a reference to all events
    pub fn events(&self) -> &[CausalEvent<T>] {
        &self.events
    }

    /// Get a mutable reference to all events
    ///
    /// # Warning
    ///
    /// Modifying events can violate causality! Use with caution.
    pub fn events_mut(&mut self) -> &mut [CausalEvent<T>] {
        &mut self.events
    }

    /// Get an event by ID
    pub fn get(&self, id: CausalId) -> Option<&CausalEvent<T>> {
        self.events.iter().find(|e| e.id() == id)
    }

    /// Find events that depend on a specific event
    pub fn dependents_of(&self, id: CausalId) -> Vec<&CausalEvent<T>> {
        self.events
            .iter()
            .filter(|e| e.depends_on(id))
            .collect()
    }

    /// Get events in topological order (dependencies before dependents)
    ///
    /// Returns None if there are circular dependencies.
    pub fn topological_order(&self) -> Option<Vec<&CausalEvent<T>>> {
        // Validate first
        if self.validate().is_err() {
            return None;
        }

        // Since we maintain causality invariant, temporal order = topological order
        let mut sorted: Vec<_> = self.events.iter().collect();
        sorted.sort_by_key(|e| e.time());
        Some(sorted)
    }

    /// Consume the chain and return the events
    pub fn into_events(self) -> Vec<CausalEvent<T>> {
        self.events
    }

    /// Create a chain from existing events (validates them)
    ///
    /// # Example
    ///
    /// ```rust
    /// use cim_keys::causality::{CausalChain, CausalEvent};
    ///
    /// let event1 = CausalEvent::new("first");
    /// let event2 = CausalEvent::caused_by("second", vec![event1.id()]);
    ///
    /// let chain = CausalChain::from_events(vec![event1, event2]);
    /// assert!(chain.is_ok());
    /// ```
    pub fn from_events(events: Vec<CausalEvent<T>>) -> Result<Self, CausalityError> {
        // Validate first
        CausalityValidator::validate_events(&events)?;

        // Build time map
        let mut time_map = HashMap::new();
        for event in &events {
            time_map.insert(event.id(), event.time());
        }

        Ok(CausalChain { events, time_map })
    }
}

impl<T> Default for CausalChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_chain() {
        let chain = CausalChain::<String>::new();
        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());
        assert!(chain.validate().is_ok());
    }

    #[test]
    fn test_add_single_event() {
        let chain = CausalChain::new();
        let event = CausalEvent::new("test");

        let chain = chain.add(event).unwrap();
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_add_dependent_events() {
        let event1 = CausalEvent::new("first");
        let event2 = CausalEvent::caused_by("second", vec![event1.id()]);

        let chain = CausalChain::new()
            .add(event1)
            .unwrap()
            .add(event2)
            .unwrap();

        assert_eq!(chain.len(), 2);
        assert!(chain.validate().is_ok());
    }

    #[test]
    fn test_dependents_of() {
        let event1 = CausalEvent::new("first");
        let event1_id = event1.id();
        let event2 = CausalEvent::caused_by("second", vec![event1_id]);
        let event3 = CausalEvent::caused_by("third", vec![event1_id]);

        let chain = CausalChain::new()
            .add(event1)
            .unwrap()
            .add(event2)
            .unwrap()
            .add(event3)
            .unwrap();

        let dependents = chain.dependents_of(event1_id);
        assert_eq!(dependents.len(), 2);
    }

    #[test]
    fn test_topological_order() {
        let event1 = CausalEvent::new("first");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event2 = CausalEvent::caused_by("second", vec![event1.id()]);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event3 = CausalEvent::caused_by("third", vec![event2.id()]);

        let chain = CausalChain::new()
            .add(event1)
            .unwrap()
            .add(event2)
            .unwrap()
            .add(event3)
            .unwrap();

        let ordered = chain.topological_order().unwrap();
        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].data(), &"first");
        assert_eq!(ordered[1].data(), &"second");
        assert_eq!(ordered[2].data(), &"third");
    }

    #[test]
    fn test_from_events_success() {
        let event1 = CausalEvent::new("first");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event2 = CausalEvent::caused_by("second", vec![event1.id()]);

        let events = vec![event1, event2];
        let chain = CausalChain::from_events(events);

        assert!(chain.is_ok());
        assert_eq!(chain.unwrap().len(), 2);
    }

    #[test]
    fn test_try_add_error() {
        let chain = CausalChain::new();
        let phantom_id = CausalId::new();
        let bad_event = CausalEvent::caused_by("bad", vec![phantom_id]);

        let result = chain.try_add(bad_event);
        assert!(result.is_err());

        if let Err((recovered_chain, _event, error)) = result {
            assert_eq!(recovered_chain.len(), 0);
            assert!(matches!(error, CausalityError::MissingDependency { .. }));
        }
    }

    #[test]
    fn test_get_event() {
        let event = CausalEvent::new("test");
        let event_id = event.id();

        let chain = CausalChain::new().add(event).unwrap();

        let retrieved = chain.get(event_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().data(), &"test");
    }
}
