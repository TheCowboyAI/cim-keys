//! Causality Validation
//!
//! Validates that causal relationships are properly maintained.

use super::types::{CausalTime, CausalEvent, CausalId, CausalDependency};
use std::collections::{HashMap, HashSet};

/// Errors that can occur when validating causality
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CausalityError {
    /// An event depends on a future event (time travel!)
    FutureDependency {
        event: CausalId,
        dependency: CausalId,
        event_time: u64,
        dep_time: u64,
    },
    /// An event depends on itself (direct circular dependency)
    SelfDependency {
        event: CausalId,
    },
    /// Circular dependency detected
    CircularDependency {
        cycle: Vec<CausalId>,
    },
    /// Referenced dependency does not exist
    MissingDependency {
        event: CausalId,
        dependency: CausalId,
    },
}

impl std::fmt::Display for CausalityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CausalityError::FutureDependency {
                event,
                dependency,
                event_time,
                dep_time,
            } => write!(
                f,
                "Event {:?} at time {} depends on future event {:?} at time {}",
                event, event_time, dependency, dep_time
            ),
            CausalityError::SelfDependency { event } => {
                write!(f, "Event {:?} depends on itself", event)
            }
            CausalityError::CircularDependency { cycle } => {
                write!(f, "Circular dependency detected: {:?}", cycle)
            }
            CausalityError::MissingDependency { event, dependency } => {
                write!(
                    f,
                    "Event {:?} depends on non-existent event {:?}",
                    event, dependency
                )
            }
        }
    }
}

impl std::error::Error for CausalityError {}

/// Validator for causality constraints
pub struct CausalityValidator;

impl CausalityValidator {
    /// Validate a single event's causality
    ///
    /// Checks that all dependencies existed before the event.
    pub fn validate_event<T>(
        event: &CausalEvent<T>,
        known_events: &HashMap<CausalId, CausalTime>,
    ) -> Result<(), CausalityError> {
        // Check for self-dependency
        if event.depends_on(event.id()) {
            return Err(CausalityError::SelfDependency { event: event.id() });
        }

        // Check each dependency
        for &dep_id in event.dependencies() {
            // Check dependency exists
            let dep_time = known_events.get(&dep_id).ok_or_else(|| {
                CausalityError::MissingDependency {
                    event: event.id(),
                    dependency: dep_id,
                }
            })?;

            // Check temporal ordering (dependency must be earlier)
            if *dep_time >= event.time() {
                return Err(CausalityError::FutureDependency {
                    event: event.id(),
                    dependency: dep_id,
                    event_time: event.time().value(),
                    dep_time: dep_time.value(),
                });
            }
        }

        Ok(())
    }

    /// Detect circular dependencies in a graph
    ///
    /// Uses depth-first search to find cycles.
    pub fn detect_cycles(
        dependencies: &[CausalDependency],
    ) -> Result<(), CausalityError> {
        // Build adjacency list
        let mut graph: HashMap<CausalId, Vec<CausalId>> = HashMap::new();
        for dep in dependencies {
            graph
                .entry(dep.dependent)
                .or_insert_with(Vec::new)
                .push(dep.dependency);
        }

        // Track visited nodes
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        // DFS from each node
        for &node in graph.keys() {
            if !visited.contains(&node) {
                if let Some(cycle) = Self::dfs_cycle(
                    node,
                    &graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                ) {
                    return Err(CausalityError::CircularDependency { cycle });
                }
            }
        }

        Ok(())
    }

    /// DFS helper for cycle detection
    fn dfs_cycle(
        node: CausalId,
        graph: &HashMap<CausalId, Vec<CausalId>>,
        visited: &mut HashSet<CausalId>,
        rec_stack: &mut HashSet<CausalId>,
        path: &mut Vec<CausalId>,
    ) -> Option<Vec<CausalId>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if let Some(cycle) =
                        Self::dfs_cycle(neighbor, graph, visited, rec_stack, path)
                    {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&neighbor) {
                    // Found a cycle! Extract it from the path
                    let cycle_start = path.iter().position(|&x| x == neighbor).unwrap();
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
        None
    }

    /// Validate an entire collection of events
    ///
    /// Checks both temporal ordering and acyclicity.
    pub fn validate_events<T>(events: &[CausalEvent<T>]) -> Result<(), CausalityError> {
        // Build time map
        let mut time_map = HashMap::new();
        for event in events {
            time_map.insert(event.id(), event.time());
        }

        // Validate each event
        for event in events {
            Self::validate_event(event, &time_map)?;
        }

        // Build dependency list and check for cycles
        let mut dependencies = Vec::new();
        for event in events {
            for &dep in event.dependencies() {
                dependencies.push(CausalDependency::new(event.id(), dep));
            }
        }

        Self::detect_cycles(&dependencies)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causality::types::CausalEvent;

    #[test]
    fn test_validate_simple_causality() {
        let event1 = CausalEvent::new("first");
        let event2 = CausalEvent::caused_by("second", vec![event1.id()]);

        let mut time_map = HashMap::new();
        time_map.insert(event1.id(), event1.time());
        time_map.insert(event2.id(), event2.time());

        assert!(CausalityValidator::validate_event(&event1, &time_map).is_ok());
        assert!(CausalityValidator::validate_event(&event2, &time_map).is_ok());
    }

    #[test]
    fn test_self_dependency_error() {
        let mut event = CausalEvent::new("test");
        event.add_dependency(event.id());

        let mut time_map = HashMap::new();
        time_map.insert(event.id(), event.time());

        let result = CausalityValidator::validate_event(&event, &time_map);
        assert!(matches!(result, Err(CausalityError::SelfDependency { .. })));
    }

    #[test]
    fn test_missing_dependency_error() {
        let phantom_id = CausalId::new();
        let event = CausalEvent::caused_by("test", vec![phantom_id]);

        let mut time_map = HashMap::new();
        time_map.insert(event.id(), event.time());
        // Intentionally don't add phantom_id to time_map

        let result = CausalityValidator::validate_event(&event, &time_map);
        assert!(matches!(
            result,
            Err(CausalityError::MissingDependency { .. })
        ));
    }

    #[test]
    fn test_detect_no_cycles() {
        let id1 = CausalId::new();
        let id2 = CausalId::new();
        let id3 = CausalId::new();

        let deps = vec![
            CausalDependency::new(id2, id1), // 2 depends on 1
            CausalDependency::new(id3, id2), // 3 depends on 2
        ];

        assert!(CausalityValidator::detect_cycles(&deps).is_ok());
    }

    #[test]
    fn test_detect_simple_cycle() {
        let id1 = CausalId::new();
        let id2 = CausalId::new();

        let deps = vec![
            CausalDependency::new(id1, id2), // 1 depends on 2
            CausalDependency::new(id2, id1), // 2 depends on 1 (cycle!)
        ];

        let result = CausalityValidator::detect_cycles(&deps);
        assert!(matches!(
            result,
            Err(CausalityError::CircularDependency { .. })
        ));
    }

    #[test]
    fn test_validate_events_success() {
        let event1 = CausalEvent::new("first");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event2 = CausalEvent::caused_by("second", vec![event1.id()]);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event3 = CausalEvent::caused_by("third", vec![event2.id()]);

        let events = vec![event1, event2, event3];
        assert!(CausalityValidator::validate_events(&events).is_ok());
    }

    #[test]
    fn test_validate_events_with_multiple_deps() {
        let event1 = CausalEvent::new("first");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event2 = CausalEvent::new("second");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event3 = CausalEvent::caused_by("third", vec![event1.id(), event2.id()]);

        let events = vec![event1, event2, event3];
        assert!(CausalityValidator::validate_events(&events).is_ok());
    }
}
