// Copyright (c) 2025 - Cowboy AI, LLC.

//! Published Language Types for Cross-Context Communication
//!
//! These lightweight reference types enable bounded contexts to communicate
//! without direct dependencies on each other's internal types.
//!
//! ## Design Principles
//!
//! 1. **Lightweight** - Only essential fields, no full entity data
//! 2. **Immutable** - All fields are read-only after construction
//! 3. **Serializable** - Can cross process/network boundaries
//! 4. **Context-Free** - No imports from any specific bounded context
//!
//! ## Usage Pattern
//!
//! Instead of:
//! ```rust,ignore
//! // BAD: Direct context dependency
//! use crate::domain::Person;
//! struct KeyOwnership { owner: Person }
//! ```
//!
//! Use:
//! ```rust,ignore
//! // GOOD: Published language reference
//! use crate::shared_kernel::PersonRef;
//! struct KeyOwnership { owner: PersonRef }
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// ORGANIZATION CONTEXT REFERENCES
// ============================================================================

/// Lightweight reference to an Organization entity.
///
/// Used by downstream contexts (PKI, NATS) to reference organizations
/// without importing the full Organization type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationRef {
    /// Organization identifier
    pub id: Uuid,
    /// Organization name (for display)
    pub name: String,
    /// Organization display name (optional)
    pub display_name: Option<String>,
}

impl OrganizationRef {
    /// Create a new organization reference.
    pub fn new(id: Uuid, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            display_name: None,
        }
    }

    /// Create with display name.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Get the display name, falling back to name.
    pub fn display(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}

/// Lightweight reference to a Person entity.
///
/// Used by downstream contexts to reference people without
/// importing the full Person type with all its relationships.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonRef {
    /// Person identifier
    pub id: Uuid,
    /// Display name
    pub display_name: String,
    /// Email (for identification)
    pub email: String,
}

impl PersonRef {
    /// Create a new person reference.
    pub fn new(id: Uuid, display_name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            email: email.into(),
        }
    }
}

/// Lightweight reference to a Location entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationRef {
    /// Location identifier
    pub id: Uuid,
    /// Location name
    pub name: String,
    /// Location type (physical, virtual, etc.)
    pub location_type: String,
}

impl LocationRef {
    /// Create a new location reference.
    pub fn new(id: Uuid, name: impl Into<String>, location_type: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            location_type: location_type.into(),
        }
    }
}

/// Lightweight reference to an OrganizationUnit entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitRef {
    /// Unit identifier
    pub id: Uuid,
    /// Unit name
    pub name: String,
    /// Parent organization ID
    pub organization_id: Uuid,
}

impl UnitRef {
    /// Create a new unit reference.
    pub fn new(id: Uuid, name: impl Into<String>, organization_id: Uuid) -> Self {
        Self {
            id,
            name: name.into(),
            organization_id,
        }
    }
}

/// Lightweight reference to a Role entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleRef {
    /// Role identifier
    pub id: Uuid,
    /// Role name
    pub name: String,
    /// Role level (for hierarchy)
    pub level: u8,
}

impl RoleRef {
    /// Create a new role reference.
    pub fn new(id: Uuid, name: impl Into<String>, level: u8) -> Self {
        Self {
            id,
            name: name.into(),
            level,
        }
    }
}

// ============================================================================
// PKI CONTEXT REFERENCES
// ============================================================================

/// Lightweight reference to a cryptographic Key.
///
/// Used by downstream contexts to reference keys without
/// importing the full Key type with its cryptographic material.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyRef {
    /// Key identifier
    pub id: Uuid,
    /// Algorithm name (e.g., "Ed25519", "RSA-2048")
    pub algorithm: String,
    /// Key fingerprint (hex string)
    pub fingerprint: String,
    /// Key purpose (e.g., "Authentication", "Signing")
    pub purpose: Option<String>,
}

impl KeyRef {
    /// Create a new key reference.
    pub fn new(
        id: Uuid,
        algorithm: impl Into<String>,
        fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            id,
            algorithm: algorithm.into(),
            fingerprint: fingerprint.into(),
            purpose: None,
        }
    }

    /// Add purpose to key reference.
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }
}

/// Lightweight reference to a Certificate.
///
/// Used by downstream contexts to reference certificates without
/// importing the full Certificate type with its PEM content.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertificateRef {
    /// Certificate identifier
    pub id: Uuid,
    /// Certificate subject (CN)
    pub subject: String,
    /// Issuer certificate ID (None for self-signed)
    pub issuer_id: Option<Uuid>,
    /// Certificate type (Root, Intermediate, Leaf)
    pub cert_type: String,
    /// Expiration timestamp
    pub not_after: DateTime<Utc>,
    /// Fingerprint
    pub fingerprint: String,
}

impl CertificateRef {
    /// Create a new certificate reference.
    pub fn new(
        id: Uuid,
        subject: impl Into<String>,
        cert_type: impl Into<String>,
        not_after: DateTime<Utc>,
        fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            id,
            subject: subject.into(),
            issuer_id: None,
            cert_type: cert_type.into(),
            not_after,
            fingerprint: fingerprint.into(),
        }
    }

    /// Add issuer reference.
    pub fn with_issuer(mut self, issuer_id: Uuid) -> Self {
        self.issuer_id = Some(issuer_id);
        self
    }

    /// Check if certificate is expired.
    pub fn is_expired(&self) -> bool {
        self.not_after < Utc::now()
    }
}

// ============================================================================
// NATS CONTEXT REFERENCES
// ============================================================================

/// Lightweight reference to a NATS Operator.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperatorRef {
    /// Operator identifier
    pub id: Uuid,
    /// Operator name
    pub name: String,
    /// Public key (nkey)
    pub public_key: String,
}

impl OperatorRef {
    /// Create a new operator reference.
    pub fn new(id: Uuid, name: impl Into<String>, public_key: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            public_key: public_key.into(),
        }
    }
}

/// Lightweight reference to a NATS Account.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountRef {
    /// Account identifier
    pub id: Uuid,
    /// Account name
    pub name: String,
    /// Parent operator ID
    pub operator_id: Uuid,
    /// Public key (nkey)
    pub public_key: String,
}

impl AccountRef {
    /// Create a new account reference.
    pub fn new(
        id: Uuid,
        name: impl Into<String>,
        operator_id: Uuid,
        public_key: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            operator_id,
            public_key: public_key.into(),
        }
    }
}

/// Lightweight reference to a NATS User.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserRef {
    /// User identifier
    pub id: Uuid,
    /// User name
    pub name: String,
    /// Parent account ID
    pub account_id: Uuid,
    /// Associated person ID (cross-context reference)
    pub person_id: Option<Uuid>,
}

impl UserRef {
    /// Create a new user reference.
    pub fn new(id: Uuid, name: impl Into<String>, account_id: Uuid) -> Self {
        Self {
            id,
            name: name.into(),
            account_id,
            person_id: None,
        }
    }

    /// Associate with a person.
    pub fn for_person(mut self, person_id: Uuid) -> Self {
        self.person_id = Some(person_id);
        self
    }
}

// ============================================================================
// YUBIKEY CONTEXT REFERENCES
// ============================================================================

/// Lightweight reference to a YubiKey device.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceRef {
    /// Device identifier
    pub id: Uuid,
    /// Serial number
    pub serial: String,
    /// Owner person ID (cross-context reference)
    pub owner_id: Option<Uuid>,
}

impl DeviceRef {
    /// Create a new device reference.
    pub fn new(id: Uuid, serial: impl Into<String>) -> Self {
        Self {
            id,
            serial: serial.into(),
            owner_id: None,
        }
    }

    /// Associate with an owner.
    pub fn owned_by(mut self, owner_id: Uuid) -> Self {
        self.owner_id = Some(owner_id);
        self
    }
}

/// Lightweight reference to a PIV slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SlotRef {
    /// Slot identifier
    pub id: Uuid,
    /// Slot name (e.g., "9A", "9C")
    pub slot: String,
    /// Parent device ID
    pub device_id: Uuid,
    /// Key ID in this slot
    pub key_id: Option<Uuid>,
}

impl SlotRef {
    /// Create a new slot reference.
    pub fn new(id: Uuid, slot: impl Into<String>, device_id: Uuid) -> Self {
        Self {
            id,
            slot: slot.into(),
            device_id,
            key_id: None,
        }
    }

    /// Set key in slot.
    pub fn with_key(mut self, key_id: Uuid) -> Self {
        self.key_id = Some(key_id);
        self
    }

    /// Check if slot has a key.
    pub fn has_key(&self) -> bool {
        self.key_id.is_some()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_ref() {
        let org = OrganizationRef::new(Uuid::now_v7(), "cowboyai")
            .with_display_name("Cowboy AI, LLC");

        assert_eq!(org.display(), "Cowboy AI, LLC");
    }

    #[test]
    fn test_person_ref() {
        let person = PersonRef::new(
            Uuid::now_v7(),
            "John Doe",
            "john@example.com",
        );

        assert_eq!(person.display_name, "John Doe");
        assert_eq!(person.email, "john@example.com");
    }

    #[test]
    fn test_key_ref() {
        let key = KeyRef::new(
            Uuid::now_v7(),
            "Ed25519",
            "SHA256:abc123",
        ).with_purpose("Authentication");

        assert_eq!(key.algorithm, "Ed25519");
        assert_eq!(key.purpose, Some("Authentication".to_string()));
    }

    #[test]
    fn test_certificate_ref_expiry() {
        let expired = CertificateRef::new(
            Uuid::now_v7(),
            "CN=Expired",
            "Leaf",
            Utc::now() - chrono::Duration::days(1),
            "abc123",
        );
        assert!(expired.is_expired());

        let valid = CertificateRef::new(
            Uuid::now_v7(),
            "CN=Valid",
            "Leaf",
            Utc::now() + chrono::Duration::days(365),
            "def456",
        );
        assert!(!valid.is_expired());
    }

    #[test]
    fn test_user_ref_person_association() {
        let account_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();

        let user = UserRef::new(Uuid::now_v7(), "jdoe", account_id)
            .for_person(person_id);

        assert_eq!(user.person_id, Some(person_id));
    }

    #[test]
    fn test_device_ref_ownership() {
        let owner_id = Uuid::now_v7();

        let device = DeviceRef::new(Uuid::now_v7(), "12345678")
            .owned_by(owner_id);

        assert_eq!(device.owner_id, Some(owner_id));
    }

    #[test]
    fn test_slot_ref_key() {
        let device_id = Uuid::now_v7();
        let key_id = Uuid::now_v7();

        let slot = SlotRef::new(Uuid::now_v7(), "9A", device_id);
        assert!(!slot.has_key());

        let slot_with_key = slot.with_key(key_id);
        assert!(slot_with_key.has_key());
    }

    #[test]
    fn test_serialization() {
        let person = PersonRef::new(Uuid::now_v7(), "Test User", "test@example.com");
        let json = serde_json::to_string(&person).unwrap();
        let deserialized: PersonRef = serde_json::from_str(&json).unwrap();
        assert_eq!(person, deserialized);
    }
}
