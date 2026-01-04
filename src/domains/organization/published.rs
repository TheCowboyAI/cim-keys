// Copyright (c) 2025 - Cowboy AI, LLC.

//! Published Language for Organization Bounded Context
//!
//! These types form the "Published Language" that downstream contexts
//! (PKI, NATS, YubiKey) use to reference Organization domain entities
//! WITHOUT creating direct dependencies on internal Organization types.
//!
//! # DDD Pattern: Published Language
//!
//! A Published Language is a well-documented, versioned set of types that
//! a bounded context publishes for other contexts to consume. This prevents
//! context leakage and provides a stable API for cross-context communication.
//!
//! # Usage
//!
//! ```rust,ignore
//! use cim_keys::domains::organization::published::{
//!     OrganizationReference,
//!     PersonReference,
//!     LocationReference,
//! };
//!
//! // PKI context uses references, not direct Organization types
//! struct KeyOwnership {
//!     key_id: Uuid,
//!     owner: PersonReference,  // NOT Person
//!     organization: OrganizationReference,  // NOT Organization
//! }
//! ```
//!
//! # Context Map
//!
//! ```text
//! Organization Context [Upstream]
//!         │
//!         ▼ publishes
//! ┌─────────────────────────────┐
//! │   Published Language        │
//! │   - OrganizationReference   │
//! │   - PersonReference         │
//! │   - LocationReference       │
//! │   - RoleReference           │
//! └─────────────────────────────┘
//!         │
//!         ▼ consumes via ACL
//! ┌───────────────┬───────────────┬───────────────┐
//! │ PKI Context   │ NATS Context  │ YubiKey Ctx   │
//! │ [Downstream]  │ [Downstream]  │ [Downstream]  │
//! └───────────────┴───────────────┴───────────────┘
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// ORGANIZATION REFERENCE
// ============================================================================

/// Reference to an Organization from another bounded context.
///
/// This is a lightweight reference type that downstream contexts use
/// instead of depending directly on the Organization entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationReference {
    /// Organization identifier
    pub id: Uuid,
    /// Organization name (denormalized for display)
    pub name: String,
    /// Display name (denormalized for UI)
    pub display_name: String,
}

impl OrganizationReference {
    /// Create a new organization reference.
    pub fn new(id: Uuid, name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            display_name: display_name.into(),
        }
    }

    /// Create from just an ID (minimal reference).
    pub fn from_id(id: Uuid) -> Self {
        Self {
            id,
            name: String::new(),
            display_name: String::new(),
        }
    }
}

// ============================================================================
// PERSON REFERENCE
// ============================================================================

/// Reference to a Person from another bounded context.
///
/// This is the Published Language type that PKI, NATS, and YubiKey
/// contexts use to reference people without importing Person directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonReference {
    /// Person identifier
    pub id: Uuid,
    /// Person's display name (denormalized for UI)
    pub display_name: String,
    /// Person's email (denormalized for identification)
    pub email: String,
    /// Whether person is currently active
    pub active: bool,
}

impl PersonReference {
    /// Create a new person reference.
    pub fn new(
        id: Uuid,
        display_name: impl Into<String>,
        email: impl Into<String>,
        active: bool,
    ) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            email: email.into(),
            active,
        }
    }

    /// Create from just an ID (minimal reference).
    pub fn from_id(id: Uuid) -> Self {
        Self {
            id,
            display_name: String::new(),
            email: String::new(),
            active: true,
        }
    }
}

// ============================================================================
// LOCATION REFERENCE
// ============================================================================

/// Reference to a Location from another bounded context.
///
/// Used by YubiKey context to track physical storage locations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationReference {
    /// Location identifier
    pub id: Uuid,
    /// Location name (denormalized for display)
    pub name: String,
    /// Location type as string (avoids importing LocationType)
    pub location_type: String,
}

impl LocationReference {
    /// Create a new location reference.
    pub fn new(id: Uuid, name: impl Into<String>, location_type: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            location_type: location_type.into(),
        }
    }

    /// Create from just an ID (minimal reference).
    pub fn from_id(id: Uuid) -> Self {
        Self {
            id,
            name: String::new(),
            location_type: String::new(),
        }
    }
}

// ============================================================================
// ROLE REFERENCE
// ============================================================================

/// Reference to a Role from another bounded context.
///
/// Used by NATS context for authorization decisions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleReference {
    /// Role identifier
    pub id: Uuid,
    /// Role name (denormalized for display)
    pub name: String,
    /// Role level for authorization (denormalized)
    pub level: u8,
}

impl RoleReference {
    /// Create a new role reference.
    pub fn new(id: Uuid, name: impl Into<String>, level: u8) -> Self {
        Self {
            id,
            name: name.into(),
            level,
        }
    }

    /// Create from just an ID (minimal reference).
    pub fn from_id(id: Uuid) -> Self {
        Self {
            id,
            name: String::new(),
            level: 0,
        }
    }
}

// ============================================================================
// ORGANIZATIONAL UNIT REFERENCE
// ============================================================================

/// Reference to an OrganizationUnit from another bounded context.
///
/// Used by NATS context for account mapping.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationUnitReference {
    /// Unit identifier
    pub id: Uuid,
    /// Unit name (denormalized for display)
    pub name: String,
    /// Unit type as string (avoids importing OrganizationUnitType)
    pub unit_type: String,
    /// Parent organization ID
    pub organization_id: Uuid,
}

impl OrganizationUnitReference {
    /// Create a new unit reference.
    pub fn new(
        id: Uuid,
        name: impl Into<String>,
        unit_type: impl Into<String>,
        organization_id: Uuid,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            unit_type: unit_type.into(),
            organization_id,
        }
    }

    /// Create from just an ID (minimal reference).
    pub fn from_id(id: Uuid, organization_id: Uuid) -> Self {
        Self {
            id,
            name: String::new(),
            unit_type: String::new(),
            organization_id,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_reference_new() {
        let id = Uuid::now_v7();
        let org_ref = OrganizationReference::new(id, "CowboyAI", "Cowboy AI, LLC");

        assert_eq!(org_ref.id, id);
        assert_eq!(org_ref.name, "CowboyAI");
        assert_eq!(org_ref.display_name, "Cowboy AI, LLC");
    }

    #[test]
    fn test_organization_reference_from_id() {
        let id = Uuid::now_v7();
        let org_ref = OrganizationReference::from_id(id);

        assert_eq!(org_ref.id, id);
        assert!(org_ref.name.is_empty());
    }

    #[test]
    fn test_person_reference_new() {
        let id = Uuid::now_v7();
        let person_ref = PersonReference::new(id, "John Doe", "john@example.com", true);

        assert_eq!(person_ref.id, id);
        assert_eq!(person_ref.display_name, "John Doe");
        assert_eq!(person_ref.email, "john@example.com");
        assert!(person_ref.active);
    }

    #[test]
    fn test_person_reference_from_id() {
        let id = Uuid::now_v7();
        let person_ref = PersonReference::from_id(id);

        assert_eq!(person_ref.id, id);
        assert!(person_ref.active);
    }

    #[test]
    fn test_location_reference_new() {
        let id = Uuid::now_v7();
        let loc_ref = LocationReference::new(id, "Headquarters", "PhysicalBuilding");

        assert_eq!(loc_ref.id, id);
        assert_eq!(loc_ref.name, "Headquarters");
        assert_eq!(loc_ref.location_type, "PhysicalBuilding");
    }

    #[test]
    fn test_role_reference_new() {
        let id = Uuid::now_v7();
        let role_ref = RoleReference::new(id, "Administrator", 10);

        assert_eq!(role_ref.id, id);
        assert_eq!(role_ref.name, "Administrator");
        assert_eq!(role_ref.level, 10);
    }

    #[test]
    fn test_organization_unit_reference_new() {
        let id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        let unit_ref = OrganizationUnitReference::new(id, "Engineering", "Department", org_id);

        assert_eq!(unit_ref.id, id);
        assert_eq!(unit_ref.name, "Engineering");
        assert_eq!(unit_ref.unit_type, "Department");
        assert_eq!(unit_ref.organization_id, org_id);
    }

    #[test]
    fn test_references_implement_hash_eq() {
        use std::collections::HashSet;

        let id = Uuid::now_v7();
        let ref1 = PersonReference::from_id(id);
        let ref2 = PersonReference::from_id(id);

        assert_eq!(ref1, ref2);

        let mut set = HashSet::new();
        set.insert(ref1);
        assert!(set.contains(&ref2));
    }

    #[test]
    fn test_references_serialize_deserialize() {
        let id = Uuid::now_v7();
        let org_ref = OrganizationReference::new(id, "TestOrg", "Test Organization");

        let json = serde_json::to_string(&org_ref).unwrap();
        let deserialized: OrganizationReference = serde_json::from_str(&json).unwrap();

        assert_eq!(org_ref, deserialized);
    }
}
