// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Relations - Pure Domain Layer for Graph Edges
//!
//! This module defines relationships between domain entities without any UI concerns.
//! Colors, positions, and other visualization properties are derived in the UI layer.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Note: KeyDelegation is used in EdgeType in graph.rs, but DelegatesKey here uses inline fields

/// A pure domain relation between two entities.
///
/// Unlike `ConceptRelation` in the GUI layer, this has NO color field.
/// The UI layer derives color from the `relation_type`.
///
/// ## DDD Compliance
///
/// - No `iced::Color` dependency
/// - Serializable for persistence
/// - Type-safe through `RelationType` enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRelation {
    /// Unique ID for this relation
    pub id: Uuid,
    /// Source entity ID
    pub from: Uuid,
    /// Target entity ID
    pub to: Uuid,
    /// Semantic type of the relationship
    pub relation_type: RelationType,
    /// When this relation was established
    pub established_at: DateTime<Utc>,
    /// When this relation expires (if temporal)
    pub expires_at: Option<DateTime<Utc>>,
    /// Metadata about the relation
    pub metadata: RelationMetadata,
}

/// Semantic relationship types (domain layer).
///
/// This mirrors `EdgeType` but is in the domain layer with no UI concerns.
/// The UI layer can map these to colors and styles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    // Organizational hierarchy
    /// Parent-child relationship (Organization → OrganizationalUnit)
    ParentChild,
    /// Manager relationship (Person → OrganizationalUnit)
    ManagesUnit,
    /// Membership (Person → OrganizationalUnit)
    MemberOf,
    /// Person responsible for unit (Person → OrganizationalUnit)
    ResponsibleFor,
    /// General management relationship (Person → Entity)
    Manages,
    /// Resource management (Organization/Unit → Resource)
    ManagesResource,
    /// Managed by relationship (Entity → Organization/Unit)
    ManagedBy,

    // Key relationships
    /// Key ownership (Person → Key)
    OwnsKey,
    /// Key delegation (Person → Person)
    DelegatesKey {
        can_further_delegate: bool,
        key_purposes: Vec<String>,
    },
    /// Storage location (Key → Location)
    StoredAt,
    /// Key rotation chain (OldKey → NewKey)
    KeyRotation,
    /// Certificate uses key (Certificate → Key)
    CertificateUsesKey,
    /// Key stored in YubiKey slot (Key → YubiKey)
    StoredInYubiKeySlot { slot_id: String },

    // Policy relationships
    /// Role assignment (Person → Role) with temporal validity
    HasRole {
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
    },
    /// Separation of duties - roles that cannot be held simultaneously
    IncompatibleWith,
    /// Role contains claim (Role → Claim)
    RoleContainsClaim,
    /// Category contains claim (PolicyCategory → PolicyClaim)
    CategoryContainsClaim,
    /// Separation class contains role (PolicyGroup → PolicyRole)
    ClassContainsRole,
    /// Policy requirement (Role → Policy)
    RoleRequiresPolicy,
    /// Policy governance (Policy → Entity)
    PolicyGovernsEntity,
    /// Organization defines role (Organization → Role)
    DefinesRole,
    /// Organization defines policy (Organization → Policy)
    DefinesPolicy,

    // Trust and Access relationships
    /// Trust relationship (Organization → Organization)
    Trusts,
    /// Certificate authority (Key → Key)
    CertifiedBy,
    /// Access permission (Person → Location/Resource)
    HasAccess,

    // NATS Infrastructure
    /// JWT signing relationship (Operator → Account, Account → User)
    Signs,
    /// Account membership (User → Account)
    BelongsToAccount,
    /// Account mapped to organizational unit (Account → OrganizationalUnit)
    MapsToOrgUnit,
    /// User mapped to person (User → Person)
    MapsToPerson,

    // PKI Trust Chain
    /// Certificate signing relationship (CA cert → signed cert)
    SignedBy,
    /// Certificate certifies a key (Certificate → Key/Person)
    CertifiesKey,
    /// Certificate issued to an entity (Certificate → Person/Service/Organization)
    IssuedTo,

    // YubiKey Hardware
    /// YubiKey ownership (Person → YubiKey)
    OwnsYubiKey,
    /// YubiKey assigned to person (YubiKey → Person)
    AssignedTo,
    /// PIV slot on YubiKey (YubiKey → PivSlot)
    HasSlot,
    /// Key stored in slot (PivSlot → Key)
    StoresKey,
}

impl RelationType {
    /// Get the semantic category for this relation type.
    ///
    /// Used by the UI layer to derive appropriate styling.
    pub fn category(&self) -> RelationCategory {
        match self {
            Self::ParentChild
            | Self::ManagesUnit
            | Self::MemberOf
            | Self::ResponsibleFor
            | Self::Manages
            | Self::ManagesResource
            | Self::ManagedBy => RelationCategory::Organizational,

            Self::OwnsKey
            | Self::DelegatesKey { .. }
            | Self::StoredAt
            | Self::KeyRotation
            | Self::CertificateUsesKey
            | Self::StoredInYubiKeySlot { .. } => RelationCategory::KeyManagement,

            Self::HasRole { .. }
            | Self::IncompatibleWith
            | Self::RoleContainsClaim
            | Self::CategoryContainsClaim
            | Self::ClassContainsRole
            | Self::RoleRequiresPolicy
            | Self::PolicyGovernsEntity
            | Self::DefinesRole
            | Self::DefinesPolicy => RelationCategory::Policy,

            Self::Trusts | Self::CertifiedBy | Self::HasAccess => RelationCategory::Trust,

            Self::Signs
            | Self::BelongsToAccount
            | Self::MapsToOrgUnit
            | Self::MapsToPerson => RelationCategory::Nats,

            Self::SignedBy | Self::CertifiesKey | Self::IssuedTo => RelationCategory::Pki,

            Self::OwnsYubiKey
            | Self::AssignedTo
            | Self::HasSlot
            | Self::StoresKey => RelationCategory::YubiKey,
        }
    }

    /// Get a human-readable label for this relation type
    pub fn label(&self) -> &'static str {
        match self {
            Self::ParentChild => "contains",
            Self::ManagesUnit => "manages",
            Self::MemberOf => "member of",
            Self::ResponsibleFor => "responsible for",
            Self::Manages => "manages",
            Self::ManagesResource => "manages resource",
            Self::ManagedBy => "managed by",
            Self::OwnsKey => "owns key",
            Self::DelegatesKey { .. } => "delegates key",
            Self::StoredAt => "stored at",
            Self::KeyRotation => "rotated to",
            Self::CertificateUsesKey => "uses key",
            Self::StoredInYubiKeySlot { .. } => "stored in slot",
            Self::HasRole { .. } => "has role",
            Self::IncompatibleWith => "incompatible with",
            Self::RoleContainsClaim => "contains claim",
            Self::CategoryContainsClaim => "contains claim",
            Self::ClassContainsRole => "contains role",
            Self::RoleRequiresPolicy => "requires policy",
            Self::PolicyGovernsEntity => "governs",
            Self::DefinesRole => "defines role",
            Self::DefinesPolicy => "defines policy",
            Self::Trusts => "trusts",
            Self::CertifiedBy => "certified by",
            Self::HasAccess => "has access",
            Self::Signs => "signs",
            Self::BelongsToAccount => "belongs to",
            Self::MapsToOrgUnit => "maps to unit",
            Self::MapsToPerson => "maps to person",
            Self::SignedBy => "signed by",
            Self::CertifiesKey => "certifies",
            Self::IssuedTo => "issued to",
            Self::OwnsYubiKey => "owns YubiKey",
            Self::AssignedTo => "assigned to",
            Self::HasSlot => "has slot",
            Self::StoresKey => "stores key",
        }
    }

    /// Check if this relation type is bidirectional
    pub fn is_bidirectional(&self) -> bool {
        matches!(self, Self::IncompatibleWith | Self::Trusts)
    }

    /// Check if this relation type is temporal (has validity period)
    pub fn is_temporal(&self) -> bool {
        matches!(self, Self::HasRole { .. } | Self::DelegatesKey { .. })
    }
}

/// Semantic categories for relation types.
///
/// Used by the UI layer to derive consistent styling across related types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationCategory {
    /// Organizational structure relationships
    Organizational,
    /// Key management relationships
    KeyManagement,
    /// Policy and role relationships
    Policy,
    /// Trust and access relationships
    Trust,
    /// NATS infrastructure relationships
    Nats,
    /// PKI certificate relationships
    Pki,
    /// YubiKey hardware relationships
    YubiKey,
}

/// Metadata about a relation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationMetadata {
    /// Who created this relation
    pub created_by: Option<Uuid>,
    /// Description or notes about the relation
    pub description: Option<String>,
    /// Additional key-value pairs
    pub attributes: std::collections::HashMap<String, String>,
}

impl DomainRelation {
    /// Create a new domain relation
    pub fn new(from: Uuid, to: Uuid, relation_type: RelationType) -> Self {
        Self {
            id: Uuid::now_v7(),
            from,
            to,
            relation_type,
            established_at: Utc::now(),
            expires_at: None,
            metadata: RelationMetadata::default(),
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: RelationMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if the relation is currently valid (not expired)
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() < expires,
            None => true,
        }
    }

    /// Get the relation category for styling
    pub fn category(&self) -> RelationCategory {
        self.relation_type.category()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_relation_creation() {
        let from = Uuid::now_v7();
        let to = Uuid::now_v7();
        let relation = DomainRelation::new(from, to, RelationType::ParentChild);

        assert_eq!(relation.from, from);
        assert_eq!(relation.to, to);
        assert!(relation.is_valid());
    }

    #[test]
    fn test_relation_category() {
        assert_eq!(
            RelationType::ParentChild.category(),
            RelationCategory::Organizational
        );
        assert_eq!(
            RelationType::OwnsKey.category(),
            RelationCategory::KeyManagement
        );
        assert_eq!(
            RelationType::HasRole {
                valid_from: Utc::now(),
                valid_until: None
            }
            .category(),
            RelationCategory::Policy
        );
        assert_eq!(RelationType::Signs.category(), RelationCategory::Nats);
        assert_eq!(RelationType::SignedBy.category(), RelationCategory::Pki);
        assert_eq!(RelationType::HasSlot.category(), RelationCategory::YubiKey);
    }

    #[test]
    fn test_relation_labels() {
        assert_eq!(RelationType::ParentChild.label(), "contains");
        assert_eq!(RelationType::MemberOf.label(), "member of");
        assert_eq!(RelationType::Signs.label(), "signs");
    }

    #[test]
    fn test_expired_relation() {
        let from = Uuid::now_v7();
        let to = Uuid::now_v7();
        let expired = DomainRelation::new(from, to, RelationType::Trusts)
            .with_expiration(Utc::now() - chrono::Duration::hours(1));

        assert!(!expired.is_valid());
    }
}
