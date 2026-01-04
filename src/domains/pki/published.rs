// Copyright (c) 2025 - Cowboy AI, LLC.

//! Published Language for PKI Bounded Context
//!
//! These types form the "Published Language" that other contexts
//! use to reference PKI entities (keys, certificates) WITHOUT
//! creating direct dependencies on internal PKI types.
//!
//! # DDD Pattern: Published Language
//!
//! A Published Language provides a stable API for cross-context
//! communication while allowing internal types to evolve independently.
//!
//! # Usage
//!
//! ```rust,ignore
//! use cim_keys::domains::pki::published::{
//!     KeyReference,
//!     CertificateReference,
//! };
//!
//! // NATS context uses references, not direct PKI types
//! struct NatsUserCredential {
//!     user_id: Uuid,
//!     signing_key: KeyReference,  // NOT CryptographicKey
//! }
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// KEY REFERENCE
// ============================================================================

/// Reference to a CryptographicKey from another bounded context.
///
/// This is the Published Language type that NATS and YubiKey contexts
/// use to reference keys without importing CryptographicKey directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyReference {
    /// Key identifier
    pub id: Uuid,
    /// Algorithm as string (e.g., "Ed25519", "P256")
    pub algorithm: String,
    /// Key fingerprint for verification
    pub fingerprint: String,
    /// Purpose as string (e.g., "Signing", "Encryption")
    pub purpose: String,
}

impl KeyReference {
    /// Create a new key reference.
    pub fn new(
        id: Uuid,
        algorithm: impl Into<String>,
        fingerprint: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        Self {
            id,
            algorithm: algorithm.into(),
            fingerprint: fingerprint.into(),
            purpose: purpose.into(),
        }
    }

    /// Create from just an ID (minimal reference).
    pub fn from_id(id: Uuid) -> Self {
        Self {
            id,
            algorithm: String::new(),
            fingerprint: String::new(),
            purpose: String::new(),
        }
    }
}

// ============================================================================
// CERTIFICATE REFERENCE
// ============================================================================

/// Reference to a Certificate from another bounded context.
///
/// Used by Organization context for trust relationships and
/// NATS context for TLS configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertificateReference {
    /// Certificate identifier
    pub id: Uuid,
    /// Certificate subject (denormalized for display)
    pub subject: String,
    /// Certificate type as string (e.g., "Root", "Intermediate", "Leaf")
    pub cert_type: String,
    /// Expiration date (denormalized for validity checking)
    pub not_after: DateTime<Utc>,
    /// Whether certificate is currently valid
    pub is_valid: bool,
}

impl CertificateReference {
    /// Create a new certificate reference.
    pub fn new(
        id: Uuid,
        subject: impl Into<String>,
        cert_type: impl Into<String>,
        not_after: DateTime<Utc>,
        is_valid: bool,
    ) -> Self {
        Self {
            id,
            subject: subject.into(),
            cert_type: cert_type.into(),
            not_after,
            is_valid,
        }
    }

    /// Create from just an ID (minimal reference).
    pub fn from_id(id: Uuid) -> Self {
        Self {
            id,
            subject: String::new(),
            cert_type: String::new(),
            not_after: Utc::now(),
            is_valid: false,
        }
    }
}

// ============================================================================
// KEY OWNERSHIP REFERENCE
// ============================================================================

/// Reference to key ownership from another bounded context.
///
/// Used by NATS context to determine who owns NATS signing keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyOwnershipReference {
    /// Key being owned
    pub key_id: Uuid,
    /// Person who owns the key (references Organization context)
    pub owner_id: Uuid,
    /// Organization the key belongs to
    pub organization_id: Uuid,
    /// Role that grants this ownership
    pub role: String,
}

impl KeyOwnershipReference {
    /// Create a new key ownership reference.
    pub fn new(
        key_id: Uuid,
        owner_id: Uuid,
        organization_id: Uuid,
        role: impl Into<String>,
    ) -> Self {
        Self {
            key_id,
            owner_id,
            organization_id,
            role: role.into(),
        }
    }
}

// ============================================================================
// TRUST CHAIN REFERENCE
// ============================================================================

/// Reference to a certificate trust chain from another bounded context.
///
/// Used by NATS context for TLS verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustChainReference {
    /// Root CA certificate
    pub root_cert_id: Uuid,
    /// Intermediate CA certificates (in order from root to leaf)
    pub intermediate_cert_ids: Vec<Uuid>,
    /// Whether the chain is complete and valid
    pub is_valid: bool,
}

impl TrustChainReference {
    /// Create a new trust chain reference.
    pub fn new(root_cert_id: Uuid, intermediate_cert_ids: Vec<Uuid>, is_valid: bool) -> Self {
        Self {
            root_cert_id,
            intermediate_cert_ids,
            is_valid,
        }
    }

    /// Create an empty/invalid trust chain.
    pub fn empty() -> Self {
        Self {
            root_cert_id: Uuid::nil(),
            intermediate_cert_ids: Vec::new(),
            is_valid: false,
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
    fn test_key_reference_new() {
        let id = Uuid::now_v7();
        let key_ref = KeyReference::new(id, "Ed25519", "SHA256:abc123", "Signing");

        assert_eq!(key_ref.id, id);
        assert_eq!(key_ref.algorithm, "Ed25519");
        assert_eq!(key_ref.fingerprint, "SHA256:abc123");
        assert_eq!(key_ref.purpose, "Signing");
    }

    #[test]
    fn test_key_reference_from_id() {
        let id = Uuid::now_v7();
        let key_ref = KeyReference::from_id(id);

        assert_eq!(key_ref.id, id);
        assert!(key_ref.algorithm.is_empty());
    }

    #[test]
    fn test_certificate_reference_new() {
        let id = Uuid::now_v7();
        let not_after = Utc::now() + chrono::Duration::days(365);
        let cert_ref = CertificateReference::new(id, "CN=Root CA", "Root", not_after, true);

        assert_eq!(cert_ref.id, id);
        assert_eq!(cert_ref.subject, "CN=Root CA");
        assert_eq!(cert_ref.cert_type, "Root");
        assert!(cert_ref.is_valid);
    }

    #[test]
    fn test_certificate_reference_from_id() {
        let id = Uuid::now_v7();
        let cert_ref = CertificateReference::from_id(id);

        assert_eq!(cert_ref.id, id);
        assert!(!cert_ref.is_valid);
    }

    #[test]
    fn test_key_ownership_reference_new() {
        let key_id = Uuid::now_v7();
        let owner_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        let ownership = KeyOwnershipReference::new(key_id, owner_id, org_id, "SecurityAdmin");

        assert_eq!(ownership.key_id, key_id);
        assert_eq!(ownership.owner_id, owner_id);
        assert_eq!(ownership.organization_id, org_id);
        assert_eq!(ownership.role, "SecurityAdmin");
    }

    #[test]
    fn test_trust_chain_reference_new() {
        let root_id = Uuid::now_v7();
        let intermediate_ids = vec![Uuid::now_v7(), Uuid::now_v7()];
        let chain = TrustChainReference::new(root_id, intermediate_ids.clone(), true);

        assert_eq!(chain.root_cert_id, root_id);
        assert_eq!(chain.intermediate_cert_ids.len(), 2);
        assert!(chain.is_valid);
    }

    #[test]
    fn test_trust_chain_reference_empty() {
        let chain = TrustChainReference::empty();

        assert!(chain.root_cert_id.is_nil());
        assert!(chain.intermediate_cert_ids.is_empty());
        assert!(!chain.is_valid);
    }

    #[test]
    fn test_references_serialize_deserialize() {
        let id = Uuid::now_v7();
        let key_ref = KeyReference::new(id, "P256", "fingerprint", "Encryption");

        let json = serde_json::to_string(&key_ref).unwrap();
        let deserialized: KeyReference = serde_json::from_str(&json).unwrap();

        assert_eq!(key_ref, deserialized);
    }
}
