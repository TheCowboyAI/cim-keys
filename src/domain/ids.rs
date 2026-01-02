// Copyright (c) 2025 - Cowboy AI, LLC.

//! Phantom-Typed Entity IDs for Compile-Time Type Safety
//!
//! This module provides type-safe entity identifiers using phantom types.
//! Each bounded context has its own marker type, preventing accidental
//! mixing of IDs from different contexts at compile time.
//!
//! ## Design Notes
//!
//! Some ID types are imported from cim-domain-* crates (PersonId, LocationMarker)
//! while others are defined here for cim-keys specific entities.
//!
//! ## Example
//!
//! ```ignore
//! use cim_keys::domain::ids::{OrganizationId, CertificateId};
//!
//! // Compile error: cannot pass CertificateId where OrganizationId expected
//! fn get_org(id: OrganizationId) { ... }
//! get_org(cert_id); // ERROR!
//! ```

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

// ============================================================================
// PHANTOM-TYPED ENTITY ID
// ============================================================================

/// A type-safe entity identifier using phantom types.
///
/// The phantom type parameter `T` identifies which bounded context this ID
/// belongs to, preventing accidental mixing of IDs at compile time.
///
/// ## UUID v7 Mandate
///
/// Always use `EntityId::new()` which generates UUID v7 (time-ordered).
/// Never use UUID v4 or v5 for new entities.
pub struct EntityId<T> {
    id: Uuid,
    _marker: PhantomData<T>,
}

// Manual Copy implementation - PhantomData<T> is always Copy, Uuid is Copy
impl<T> Copy for EntityId<T> {}

impl<T> EntityId<T> {
    /// Create a new entity ID with UUID v7 (time-ordered).
    #[inline]
    pub fn new() -> Self {
        Self {
            id: Uuid::now_v7(),
            _marker: PhantomData,
        }
    }

    /// Create from an existing UUID (e.g., when loading from storage).
    #[inline]
    pub fn from_uuid(id: Uuid) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Get the underlying UUID.
    #[inline]
    pub fn as_uuid(&self) -> Uuid {
        self.id
    }

    /// Convert to a different entity type (use with caution).
    #[inline]
    pub fn transmute<U>(self) -> EntityId<U> {
        EntityId {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for EntityId<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for EntityId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for EntityId<T> {}

impl<T> Hash for EntityId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> fmt::Debug for EntityId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntityId({})", self.id)
    }
}

impl<T> fmt::Display for EntityId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<T> Serialize for EntityId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for EntityId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = Uuid::deserialize(deserializer)?;
        Ok(Self::from_uuid(id))
    }
}

// ============================================================================
// MARKER TYPES FOR CIM-KEYS SPECIFIC BOUNDED CONTEXTS
// ============================================================================
//
// Note: Some markers are already defined in cim-domain-* crates:
// - PersonMarker, PersonId: from cim-domain-person (policy feature)
// - LocationMarker: from cim-domain-location
//
// We only define markers for cim-keys specific entities here.

// --- Organization Bounded Context ---
// Note: Organization types don't have dedicated IDs in cim-domain-organization yet

/// Marker type for Organization entities (cim-keys bootstrap)
pub struct BootstrapOrgMarker;

/// Marker type for OrganizationUnit entities
pub struct UnitMarker;

/// Marker type for Person entities (cim-keys bootstrap)
pub struct BootstrapPersonMarker;

/// Marker type for Role entities (bootstrap role, not domain role)
pub struct BootstrapRoleMarker;

/// Marker type for Policy entities (bootstrap policy)
pub struct BootstrapPolicyMarker;

// --- PKI Bounded Context ---

/// Marker type for Certificate entities (Root, Intermediate, Leaf)
pub struct CertificateMarker;

/// Marker type for cryptographic Key entities
pub struct KeyMarker;

// --- NATS Bounded Context ---

/// Marker type for NATS Operator entities
pub struct NatsOperatorMarker;

/// Marker type for NATS Account entities
pub struct NatsAccountMarker;

/// Marker type for NATS User entities
pub struct NatsUserMarker;

// --- YubiKey Bounded Context ---

/// Marker type for YubiKey device entities
pub struct YubiKeyMarker;

/// Marker type for PIV Slot entities
pub struct SlotMarker;

// --- Visualization/Export Context ---

/// Marker type for Manifest export entities
pub struct ManifestMarker;

/// Marker type for PolicyRole visualization entities
pub struct PolicyRoleMarker;

/// Marker type for PolicyClaim visualization entities
pub struct ClaimMarker;

/// Marker type for PolicyCategory grouping entities
pub struct PolicyCategoryMarker;

/// Marker type for PolicyGroup (SeparationClass) entities
pub struct PolicyGroupMarker;

// ============================================================================
// TYPE ALIASES FOR CONVENIENCE
// ============================================================================

// --- Organization Bounded Context ---

/// Type-safe Organization ID (bootstrap)
pub type BootstrapOrgId = EntityId<BootstrapOrgMarker>;

/// Type-safe OrganizationUnit ID
pub type UnitId = EntityId<UnitMarker>;

/// Type-safe Person ID (bootstrap)
pub type BootstrapPersonId = EntityId<BootstrapPersonMarker>;

/// Type-safe Role ID (bootstrap)
pub type BootstrapRoleId = EntityId<BootstrapRoleMarker>;

/// Type-safe Policy ID (bootstrap)
pub type BootstrapPolicyId = EntityId<BootstrapPolicyMarker>;

// --- PKI Bounded Context ---

/// Type-safe Certificate ID (for Root, Intermediate, and Leaf certificates)
pub type CertificateId = EntityId<CertificateMarker>;

/// Type-safe cryptographic Key ID
pub type KeyId = EntityId<KeyMarker>;

// --- NATS Bounded Context ---

/// Type-safe NATS Operator ID
pub type NatsOperatorId = EntityId<NatsOperatorMarker>;

/// Type-safe NATS Account ID
pub type NatsAccountId = EntityId<NatsAccountMarker>;

/// Type-safe NATS User ID
pub type NatsUserId = EntityId<NatsUserMarker>;

// --- YubiKey Bounded Context ---

/// Type-safe YubiKey device ID
pub type YubiKeyDeviceId = EntityId<YubiKeyMarker>;

/// Type-safe PIV Slot ID
pub type SlotId = EntityId<SlotMarker>;

// --- Visualization/Export Context ---

/// Type-safe Manifest ID
pub type ManifestId = EntityId<ManifestMarker>;

/// Type-safe PolicyRole ID
pub type PolicyRoleId = EntityId<PolicyRoleMarker>;

/// Type-safe PolicyClaim ID
pub type ClaimId = EntityId<ClaimMarker>;

/// Type-safe PolicyCategory ID
pub type PolicyCategoryId = EntityId<PolicyCategoryMarker>;

/// Type-safe PolicyGroup ID (for SeparationClass groupings)
pub type PolicyGroupId = EntityId<PolicyGroupMarker>;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_creation() {
        let org_id = BootstrapOrgId::new();
        let cert_id = CertificateId::new();

        // Different types should have different UUIDs
        assert_ne!(org_id.as_uuid(), cert_id.as_uuid());
    }

    #[test]
    fn test_entity_id_from_uuid() {
        let uuid = Uuid::now_v7();
        let org_id = BootstrapOrgId::from_uuid(uuid);
        assert_eq!(org_id.as_uuid(), uuid);
    }

    #[test]
    fn test_entity_id_equality() {
        let uuid = Uuid::now_v7();
        let id1 = BootstrapOrgId::from_uuid(uuid);
        let id2 = BootstrapOrgId::from_uuid(uuid);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_entity_id_hash() {
        use std::collections::HashSet;

        let mut set: HashSet<BootstrapOrgId> = HashSet::new();
        let id1 = BootstrapOrgId::new();
        let id2 = BootstrapOrgId::new();

        set.insert(id1);
        set.insert(id2);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }

    #[test]
    fn test_entity_id_serialization() {
        let id = BootstrapOrgId::new();
        let json = serde_json::to_string(&id).unwrap();

        // Should serialize as just the UUID string
        assert!(json.contains(&id.as_uuid().to_string()));

        // Should deserialize back
        let deserialized: BootstrapOrgId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_entity_id_display() {
        let uuid = Uuid::now_v7();
        let id = BootstrapOrgId::from_uuid(uuid);
        assert_eq!(format!("{}", id), format!("{}", uuid));
    }

    #[test]
    fn test_transmute_between_types() {
        let org_id = BootstrapOrgId::new();
        let transmuted: CertificateId = org_id.transmute();

        // UUIDs should be the same
        assert_eq!(org_id.as_uuid(), transmuted.as_uuid());
    }
}
