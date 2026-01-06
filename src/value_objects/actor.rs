// Copyright (c) 2025 - Cowboy AI, LLC.

//! Actor Identity Value Object
//!
//! Represents the identity of an actor who performed an action in the system.
//! Actors can be:
//! - **Person**: A human user identified by their person ID
//! - **ServiceAccount**: An automated service or application
//! - **System**: A built-in system process (e.g., scheduler, migration)
//!
//! This replaces loose `*_by: String` fields in events with type-safe identifiers.

use cim_domain::{DomainConcept, ValueObject};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Actor identity - who performed an action
///
/// ## Migration from String
///
/// Old events used `generated_by: String`, `revoked_by: String`, etc.
/// This type provides:
/// - Type safety for actor references
/// - Queryable actor types
/// - Consistent serialization
///
/// ## Serialization Format
///
/// ```json
/// { "actor_type": "Person", "id": "uuid-here" }
/// { "actor_type": "System", "name": "scheduler" }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "actor_type")]
pub enum ActorId {
    /// A human user identified by person ID
    Person { id: Uuid },

    /// An automated service account
    ServiceAccount { id: Uuid, name: String },

    /// A built-in system process
    System { name: String },

    /// Unknown/legacy actor (for backward compatibility)
    #[serde(alias = "Unknown")]
    Legacy { identifier: String },
}

impl ActorId {
    /// Create an ActorId for a person
    pub fn person(id: Uuid) -> Self {
        ActorId::Person { id }
    }

    /// Create an ActorId for a service account
    pub fn service_account(id: Uuid, name: impl Into<String>) -> Self {
        ActorId::ServiceAccount { id, name: name.into() }
    }

    /// Create an ActorId for a system process
    pub fn system(name: impl Into<String>) -> Self {
        ActorId::System { name: name.into() }
    }

    /// Create a legacy ActorId from a string (for migration)
    pub fn legacy(identifier: impl Into<String>) -> Self {
        ActorId::Legacy { identifier: identifier.into() }
    }

    /// Parse a string into an ActorId, attempting to detect the type
    ///
    /// Heuristics:
    /// - If it's a valid UUID, assume it's a person ID
    /// - If it starts with "system:" or is a known system name, it's System
    /// - If it starts with "service:" it's a ServiceAccount
    /// - Otherwise, it's Legacy
    pub fn parse(s: &str) -> Self {
        // Try UUID first (likely person ID)
        if let Ok(uuid) = Uuid::parse_str(s) {
            return ActorId::Person { id: uuid };
        }

        // Check for system: prefix
        if let Some(name) = s.strip_prefix("system:") {
            return ActorId::System { name: name.to_string() };
        }

        // Check for known system names
        let system_names = ["system", "scheduler", "migration", "auto", "internal"];
        if system_names.iter().any(|&n| s.eq_ignore_ascii_case(n)) {
            return ActorId::System { name: s.to_lowercase() };
        }

        // Check for service: prefix
        if let Some(rest) = s.strip_prefix("service:") {
            // Format: "service:name:uuid" or "service:name"
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            if parts.len() == 2 {
                if let Ok(uuid) = Uuid::parse_str(parts[1]) {
                    return ActorId::ServiceAccount { id: uuid, name: parts[0].to_string() };
                }
            }
            return ActorId::ServiceAccount {
                id: Uuid::now_v7(),
                name: rest.to_string()
            };
        }

        // Default to legacy
        ActorId::Legacy { identifier: s.to_string() }
    }

    /// Convert to a string representation (for legacy compatibility)
    pub fn to_legacy_string(&self) -> String {
        match self {
            ActorId::Person { id } => id.to_string(),
            ActorId::ServiceAccount { id, name } => format!("service:{}:{}", name, id),
            ActorId::System { name } => format!("system:{}", name),
            ActorId::Legacy { identifier } => identifier.clone(),
        }
    }

    /// Check if this is a person actor
    pub fn is_person(&self) -> bool {
        matches!(self, ActorId::Person { .. })
    }

    /// Check if this is a system actor
    pub fn is_system(&self) -> bool {
        matches!(self, ActorId::System { .. })
    }

    /// Check if this is a service account
    pub fn is_service_account(&self) -> bool {
        matches!(self, ActorId::ServiceAccount { .. })
    }

    /// Check if this is a legacy actor
    pub fn is_legacy(&self) -> bool {
        matches!(self, ActorId::Legacy { .. })
    }

    /// Get the UUID if this actor has one
    pub fn uuid(&self) -> Option<Uuid> {
        match self {
            ActorId::Person { id } => Some(*id),
            ActorId::ServiceAccount { id, .. } => Some(*id),
            ActorId::System { .. } => None,
            ActorId::Legacy { identifier } => Uuid::parse_str(identifier).ok(),
        }
    }

    /// Get a display name for the actor
    pub fn display_name(&self) -> String {
        match self {
            ActorId::Person { id } => format!("Person({})", &id.to_string()[..8]),
            ActorId::ServiceAccount { name, .. } => format!("Service({})", name),
            ActorId::System { name } => format!("System({})", name),
            ActorId::Legacy { identifier } => identifier.clone(),
        }
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for ActorId {
    fn default() -> Self {
        ActorId::System { name: "system".to_string() }
    }
}

// Convert from legacy string
impl From<String> for ActorId {
    fn from(s: String) -> Self {
        ActorId::parse(&s)
    }
}

impl From<&str> for ActorId {
    fn from(s: &str) -> Self {
        ActorId::parse(s)
    }
}

// Convert from person UUID
impl From<Uuid> for ActorId {
    fn from(id: Uuid) -> Self {
        ActorId::Person { id }
    }
}

// DDD marker traits
impl DomainConcept for ActorId {}
impl ValueObject for ActorId {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_actor() {
        let id = Uuid::now_v7();
        let actor = ActorId::person(id);

        assert!(actor.is_person());
        assert!(!actor.is_system());
        assert_eq!(actor.uuid(), Some(id));
    }

    #[test]
    fn test_system_actor() {
        let actor = ActorId::system("scheduler");

        assert!(actor.is_system());
        assert!(!actor.is_person());
        assert_eq!(actor.uuid(), None);
    }

    #[test]
    fn test_service_account_actor() {
        let id = Uuid::now_v7();
        let actor = ActorId::service_account(id, "backup-service");

        assert!(actor.is_service_account());
        assert_eq!(actor.uuid(), Some(id));
    }

    #[test]
    fn test_parse_uuid_string() {
        let id = Uuid::now_v7();
        let actor = ActorId::parse(&id.to_string());

        assert!(actor.is_person());
        assert_eq!(actor.uuid(), Some(id));
    }

    #[test]
    fn test_parse_system_prefix() {
        let actor = ActorId::parse("system:scheduler");

        assert!(actor.is_system());
        match actor {
            ActorId::System { name } => assert_eq!(name, "scheduler"),
            _ => panic!("Expected System actor"),
        }
    }

    #[test]
    fn test_parse_known_system_name() {
        let actor = ActorId::parse("scheduler");
        assert!(actor.is_system());

        let actor = ActorId::parse("SYSTEM");
        assert!(actor.is_system());
    }

    #[test]
    fn test_parse_legacy() {
        let actor = ActorId::parse("john.doe@example.com");
        assert!(actor.is_legacy());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let actors = vec![
            ActorId::person(Uuid::now_v7()),
            ActorId::system("scheduler"),
            ActorId::service_account(Uuid::now_v7(), "backup"),
            ActorId::legacy("admin@example.com"),
        ];

        for actor in actors {
            let json = serde_json::to_string(&actor).unwrap();
            let deserialized: ActorId = serde_json::from_str(&json).unwrap();
            assert_eq!(actor, deserialized);
        }
    }

    #[test]
    fn test_to_legacy_string() {
        let id = Uuid::now_v7();

        let person = ActorId::person(id);
        assert_eq!(person.to_legacy_string(), id.to_string());

        let system = ActorId::system("scheduler");
        assert_eq!(system.to_legacy_string(), "system:scheduler");
    }

    #[test]
    fn test_display() {
        let actor = ActorId::system("scheduler");
        assert_eq!(format!("{}", actor), "System(scheduler)");
    }

    #[test]
    fn test_from_uuid() {
        let id = Uuid::now_v7();
        let actor: ActorId = id.into();
        assert!(actor.is_person());
    }

    #[test]
    fn test_from_string() {
        let actor: ActorId = "system".into();
        assert!(actor.is_system());
    }
}
