//! Graph Events for Event Sourcing and Undo/Redo
//!
//! This module defines events that represent state changes in the graph.
//! Events are immutable facts about what happened. Undo is implemented by
//! creating compensating events, NOT by reversing state mutations.

use uuid::Uuid;
use iced::{Point, Color};
use chrono::{DateTime, Utc};
use super::graph::{EdgeType, NodeType};

/// Graph events that can be applied to change graph state
#[derive(Debug, Clone)]
pub enum GraphEvent {
    /// Node was created
    NodeCreated {
        node_id: Uuid,
        node_type: NodeType,
        position: Point,
        color: Color,
        label: String,
        timestamp: DateTime<Utc>,
    },
    /// Node was deleted (compensating event for NodeCreated)
    NodeDeleted {
        node_id: Uuid,
        // Store snapshot for potential redo
        node_type: NodeType,
        position: Point,
        color: Color,
        label: String,
        timestamp: DateTime<Utc>,
    },
    /// Node properties were changed
    NodePropertiesChanged {
        node_id: Uuid,
        // Old values for undo (compensating event with these as new values)
        old_node_type: NodeType,
        old_label: String,
        // New values
        new_node_type: NodeType,
        new_label: String,
        timestamp: DateTime<Utc>,
    },
    /// Node was moved
    NodeMoved {
        node_id: Uuid,
        old_position: Point,
        new_position: Point,
        timestamp: DateTime<Utc>,
    },
    /// Edge was created
    EdgeCreated {
        from: Uuid,
        to: Uuid,
        edge_type: EdgeType,
        color: Color,
        timestamp: DateTime<Utc>,
    },
    /// Edge was deleted (compensating event for EdgeCreated)
    EdgeDeleted {
        from: Uuid,
        to: Uuid,
        edge_type: EdgeType,
        color: Color,
        timestamp: DateTime<Utc>,
    },
}

impl GraphEvent {
    /// Create a compensating event that undoes this event
    pub fn compensate(&self) -> Self {
        match self {
            GraphEvent::NodeCreated { node_id, node_type, position, color, label, .. } => {
                GraphEvent::NodeDeleted {
                    node_id: *node_id,
                    node_type: node_type.clone(),
                    position: *position,
                    color: *color,
                    label: label.clone(),
                    timestamp: Utc::now(),
                }
            }
            GraphEvent::NodeDeleted { node_id, node_type, position, color, label, .. } => {
                GraphEvent::NodeCreated {
                    node_id: *node_id,
                    node_type: node_type.clone(),
                    position: *position,
                    color: *color,
                    label: label.clone(),
                    timestamp: Utc::now(),
                }
            }
            GraphEvent::NodePropertiesChanged {
                node_id,
                old_node_type,
                old_label,
                new_node_type,
                new_label,
                ..
            } => {
                GraphEvent::NodePropertiesChanged {
                    node_id: *node_id,
                    old_node_type: new_node_type.clone(),
                    old_label: new_label.clone(),
                    new_node_type: old_node_type.clone(),
                    new_label: old_label.clone(),
                    timestamp: Utc::now(),
                }
            }
            GraphEvent::NodeMoved {
                node_id,
                old_position,
                new_position,
                ..
            } => {
                GraphEvent::NodeMoved {
                    node_id: *node_id,
                    old_position: *new_position,
                    new_position: *old_position,
                    timestamp: Utc::now(),
                }
            }
            GraphEvent::EdgeCreated { from, to, edge_type, color, .. } => {
                GraphEvent::EdgeDeleted {
                    from: *from,
                    to: *to,
                    edge_type: edge_type.clone(),
                    color: *color,
                    timestamp: Utc::now(),
                }
            }
            GraphEvent::EdgeDeleted { from, to, edge_type, color, .. } => {
                GraphEvent::EdgeCreated {
                    from: *from,
                    to: *to,
                    edge_type: edge_type.clone(),
                    color: *color,
                    timestamp: Utc::now(),
                }
            }
        }
    }

    /// Get a human-readable description of this event
    pub fn description(&self) -> String {
        match self {
            GraphEvent::NodeCreated { label, .. } => format!("Created node '{}'", label),
            GraphEvent::NodeDeleted { label, .. } => format!("Deleted node '{}'", label),
            GraphEvent::NodePropertiesChanged { old_label, new_label, .. } => {
                format!("Changed '{}' to '{}'", old_label, new_label)
            }
            GraphEvent::NodeMoved { node_id, .. } => format!("Moved node {:?}", node_id),
            GraphEvent::EdgeCreated { .. } => "Created edge".to_string(),
            GraphEvent::EdgeDeleted { .. } => "Deleted edge".to_string(),
        }
    }
}

/// Event stack for undo/redo
#[derive(Debug, Clone)]
pub struct EventStack {
    /// Events that have been applied (undo stack)
    applied: Vec<GraphEvent>,
    /// Events that have been undone (redo stack)
    undone: Vec<GraphEvent>,
    /// Maximum number of events to keep
    max_size: usize,
}

impl EventStack {
    /// Create a new event stack with a maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            applied: Vec::new(),
            undone: Vec::new(),
            max_size,
        }
    }

    /// Apply a new event (clears redo stack)
    pub fn push(&mut self, event: GraphEvent) {
        self.applied.push(event);
        self.undone.clear();

        // Trim if exceeds max size
        if self.applied.len() > self.max_size {
            self.applied.remove(0);
        }
    }

    /// Undo the last event by creating a compensating event
    pub fn undo(&mut self) -> Option<GraphEvent> {
        if let Some(event) = self.applied.pop() {
            let compensating = event.compensate();
            self.undone.push(event);
            Some(compensating)
        } else {
            None
        }
    }

    /// Redo the last undone event
    pub fn redo(&mut self) -> Option<GraphEvent> {
        if let Some(event) = self.undone.pop() {
            let compensating = event.compensate();
            self.applied.push(event);
            Some(compensating)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.applied.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// Get a description of the last undoable event
    pub fn undo_description(&self) -> Option<String> {
        self.applied.last().map(|e| e.description())
    }

    /// Get a description of the last redoable event
    pub fn redo_description(&self) -> Option<String> {
        self.undone.last().map(|e| e.description())
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.applied.clear();
        self.undone.clear();
    }
}

impl Default for EventStack {
    fn default() -> Self {
        Self::new(100) // Default to 100 undo levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{KeyOwnerRole, Person};

    #[test]
    fn test_event_stack_push() {
        let mut stack = EventStack::new(10);
        let person = Person {
            id: Uuid::now_v7(),
            name: "Test Person".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            created_at: Utc::now(),
            active: true,
        };

        let event = GraphEvent::NodeCreated {
            node_id: person.id,
            node_type: NodeType::Person {
                person,
                role: KeyOwnerRole::Developer,
            },
            position: Point::new(100.0, 200.0),
            color: Color::from_rgb(0.5, 0.5, 0.5),
            label: "Test".to_string(),
            timestamp: Utc::now(),
        };

        stack.push(event);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_event_compensation() {
        let person = Person {
            id: Uuid::now_v7(),
            name: "Test Person".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            created_at: Utc::now(),
            active: true,
        };

        let created = GraphEvent::NodeCreated {
            node_id: person.id,
            node_type: NodeType::Person {
                person: person.clone(),
                role: KeyOwnerRole::Developer,
            },
            position: Point::new(100.0, 200.0),
            color: Color::from_rgb(0.5, 0.5, 0.5),
            label: "Test".to_string(),
            timestamp: Utc::now(),
        };

        let compensating = created.compensate();
        matches!(compensating, GraphEvent::NodeDeleted { .. });
    }

    #[test]
    fn test_undo_redo() {
        let mut stack = EventStack::new(10);
        let person = Person {
            id: Uuid::now_v7(),
            name: "Test Person".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            created_at: Utc::now(),
            active: true,
        };

        let event = GraphEvent::NodeCreated {
            node_id: person.id,
            node_type: NodeType::Person {
                person,
                role: KeyOwnerRole::Developer,
            },
            position: Point::new(100.0, 200.0),
            color: Color::from_rgb(0.5, 0.5, 0.5),
            label: "Test".to_string(),
            timestamp: Utc::now(),
        };

        stack.push(event);
        assert!(stack.can_undo());

        let undo_event = stack.undo();
        assert!(undo_event.is_some());
        assert!(stack.can_redo());
        assert!(!stack.can_undo());

        let redo_event = stack.redo();
        assert!(redo_event.is_some());
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_max_size() {
        let mut stack = EventStack::new(3);
        let person = Person {
            id: Uuid::now_v7(),
            name: "Test Person".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            created_at: Utc::now(),
            active: true,
        };

        for i in 0..5 {
            let event = GraphEvent::NodeCreated {
                node_id: Uuid::now_v7(),
                node_type: NodeType::Person {
                    person: person.clone(),
                    role: KeyOwnerRole::Developer,
                },
                position: Point::new(i as f32 * 100.0, 200.0),
                color: Color::from_rgb(0.5, 0.5, 0.5),
                label: format!("Test {}", i),
                timestamp: Utc::now(),
            };
            stack.push(event);
        }

        // Should only keep the last 3 events
        assert_eq!(stack.applied.len(), 3);
    }
}
