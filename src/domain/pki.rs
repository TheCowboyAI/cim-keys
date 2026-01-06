// Copyright (c) 2025 - Cowboy AI, LLC.

//! PKI (Public Key Infrastructure) Bounded Context
//!
//! This module provides the PKI bounded context for cim-keys.
//! It includes certificates, cryptographic keys, and key ownership.
//!
//! ## Domain Types
//!
//! **Certificate Types**:
//! - Root CA - Top of trust hierarchy, stored on YubiKey
//! - Intermediate CA - Per-organizational unit CAs
//! - Leaf Certificates - End-entity certificates for people/services
//!
//! **Key Types**:
//! - Key ownership and delegation
//! - Key permissions (sign, encrypt, certify)
//!
//! ## Bounded Context Separation
//!
//! This context is responsible for:
//! - Certificate hierarchy (root, intermediate, leaf)
//! - Cryptographic key lifecycle
//! - Key ownership tied to people
//! - Key delegation and permissions
//!
//! It does NOT handle:
//! - Organizational structure (see `organization` context)
//! - NATS credentials (see `nats` context)
//! - Hardware storage (see `yubikey` context)
//!
//! ## Certificate Hierarchy
//!
//! ```text
//! Root CA (Organization Level)
//! ├── Intermediate CA (Department/Team)
//! │   ├── Leaf Cert (Person - Authentication)
//! │   ├── Leaf Cert (Person - Signing)
//! │   └── Leaf Cert (Service Account)
//! └── Policy CA (Special Purpose)
//!     ├── Code Signing CA
//!     └── Timestamp Authority
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Re-export phantom-typed IDs for PKI
pub use super::ids::{
    CertificateId,
    CertificateMarker,
    KeyId,
    KeyMarker,
};

// Re-export key ownership types from bootstrap
pub use super::bootstrap::{
    KeyOwnership,
    KeyOwnerRole,
    KeyDelegation,
    KeyPermission,
    OrganizationalPKI,
    PolicyCA,
    PolicyPurpose,
    PolicyConstraint,
    KeyContext,
    AuditRequirement,
};

// Re-export key algorithms and purposes from events
pub use crate::events::{KeyAlgorithm, KeyPurpose};

/// Certificate type within the PKI hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateType {
    /// Root Certificate Authority - top of trust hierarchy
    Root,
    /// Intermediate Certificate Authority - department/team level
    Intermediate,
    /// End-entity certificate for people or services
    Leaf,
    /// Policy-specific CA (code signing, timestamping, etc.)
    Policy,
}

/// Certificate status in lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateStatus {
    /// Certificate is pending generation
    Pending,
    /// Certificate is active and valid
    Active,
    /// Certificate has been revoked
    Revoked,
    /// Certificate has expired
    Expired,
    /// Certificate is suspended (temporary hold)
    Suspended,
}

/// Certificate metadata for visualization and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub id: CertificateId,
    pub cert_type: CertificateType,
    pub subject: String,
    pub issuer_id: Option<CertificateId>,
    pub status: CertificateStatus,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub key_id: Option<KeyId>,
    pub owner_person_id: Option<Uuid>,
    pub san: Vec<String>,
}

impl CertificateInfo {
    /// Create new root CA certificate info
    pub fn new_root(subject: String, validity_days: u32) -> Self {
        let now = Utc::now();
        Self {
            id: CertificateId::new(),
            cert_type: CertificateType::Root,
            subject,
            issuer_id: None, // Self-signed
            status: CertificateStatus::Pending,
            not_before: now,
            not_after: now + chrono::Duration::days(validity_days as i64),
            key_id: None,
            owner_person_id: None,
            san: Vec::new(),
        }
    }

    /// Create intermediate CA certificate info
    pub fn new_intermediate(
        subject: String,
        issuer_id: CertificateId,
        validity_days: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: CertificateId::new(),
            cert_type: CertificateType::Intermediate,
            subject,
            issuer_id: Some(issuer_id),
            status: CertificateStatus::Pending,
            not_before: now,
            not_after: now + chrono::Duration::days(validity_days as i64),
            key_id: None,
            owner_person_id: None,
            san: Vec::new(),
        }
    }

    /// Create leaf certificate info
    pub fn new_leaf(
        subject: String,
        issuer_id: CertificateId,
        owner_person_id: Uuid,
        san: Vec<String>,
        validity_days: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: CertificateId::new(),
            cert_type: CertificateType::Leaf,
            subject,
            issuer_id: Some(issuer_id),
            status: CertificateStatus::Pending,
            not_before: now,
            not_after: now + chrono::Duration::days(validity_days as i64),
            key_id: None,
            owner_person_id: Some(owner_person_id),
            san,
        }
    }

    /// Check if certificate is currently valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        self.status == CertificateStatus::Active
            && now >= self.not_before
            && now <= self.not_after
    }
}

/// Key information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub id: KeyId,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub owner_person_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub yubikey_serial: Option<String>,
    pub piv_slot: Option<String>,
}

impl KeyInfo {
    /// Create new key info
    pub fn new(algorithm: KeyAlgorithm, purpose: KeyPurpose) -> Self {
        Self {
            id: KeyId::new(),
            algorithm,
            purpose,
            owner_person_id: None,
            created_at: Utc::now(),
            yubikey_serial: None,
            piv_slot: None,
        }
    }

    /// Associate key with a person
    pub fn with_owner(mut self, person_id: Uuid) -> Self {
        self.owner_person_id = Some(person_id);
        self
    }

    /// Associate key with YubiKey storage
    pub fn with_yubikey(mut self, serial: String, slot: String) -> Self {
        self.yubikey_serial = Some(serial);
        self.piv_slot = Some(slot);
        self
    }
}

impl std::fmt::Display for CertificateType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CertificateType::Root => write!(f, "Root CA"),
            CertificateType::Intermediate => write!(f, "Intermediate CA"),
            CertificateType::Leaf => write!(f, "Leaf Certificate"),
            CertificateType::Policy => write!(f, "Policy CA"),
        }
    }
}

impl std::fmt::Display for CertificateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CertificateStatus::Pending => write!(f, "Pending"),
            CertificateStatus::Active => write!(f, "Active"),
            CertificateStatus::Revoked => write!(f, "Revoked"),
            CertificateStatus::Expired => write!(f, "Expired"),
            CertificateStatus::Suspended => write!(f, "Suspended"),
        }
    }
}

// ============================================================================
// GRAPH NODE TYPES (for DomainNode visualization)
// ============================================================================

/// Certificate for graph visualization
///
/// This type is used in `DomainNodeData` for rendering certificates in the graph.
/// It uses phantom-typed `CertificateId` for compile-time safety.
#[derive(Debug, Clone)]
pub struct Certificate {
    pub id: CertificateId,
    pub cert_type: CertificateType,
    pub subject: String,
    pub issuer: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub key_usage: Vec<String>,
    pub san: Vec<String>,
}

impl Certificate {
    /// Create a root certificate
    pub fn root(
        id: CertificateId,
        subject: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    ) -> Self {
        Self {
            id,
            cert_type: CertificateType::Root,
            subject: subject.clone(),
            issuer: subject, // Self-signed
            not_before,
            not_after,
            key_usage,
            san: Vec::new(),
        }
    }

    /// Create an intermediate certificate
    pub fn intermediate(
        id: CertificateId,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    ) -> Self {
        Self {
            id,
            cert_type: CertificateType::Intermediate,
            subject,
            issuer,
            not_before,
            not_after,
            key_usage,
            san: Vec::new(),
        }
    }

    /// Create a leaf certificate
    pub fn leaf(
        id: CertificateId,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
        san: Vec<String>,
    ) -> Self {
        Self {
            id,
            cert_type: CertificateType::Leaf,
            subject,
            issuer,
            not_before,
            not_after,
            key_usage,
            san,
        }
    }

    /// Check if certificate is currently valid (not expired)
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        now >= self.not_before && now <= self.not_after
    }

    /// Get days until expiration (negative if expired)
    pub fn days_until_expiry(&self) -> i64 {
        let now = Utc::now();
        (self.not_after - now).num_days()
    }

    // ========================================================================
    // Aggregate Contribution Methods (Sprint D)
    // ========================================================================

    /// Aggregate labels from all composed ValueObjects
    ///
    /// Collects labels from:
    /// - Certificate type (CA, Leaf, etc.)
    /// - Validity status (Valid, Expired, ExpiringSoon)
    /// - Key usage capabilities
    /// - SAN characteristics (Wildcard, etc.)
    pub fn aggregate_labels(&self) -> Vec<crate::value_objects::Label> {
        use crate::value_objects::{Label, NodeContributor};

        let mut labels = Vec::new();

        // Add certificate type label
        match self.cert_type {
            CertificateType::Root => labels.push(Label::new("RootCA")),
            CertificateType::Intermediate => labels.push(Label::new("IntermediateCA")),
            CertificateType::Leaf => labels.push(Label::new("LeafCertificate")),
            CertificateType::Policy => labels.push(Label::new("PolicyCA")),
        }

        // Add validity labels from CertificateValidity value object (via NodeContributor trait)
        if let Ok(validity) = crate::value_objects::CertificateValidity::new(self.not_before, self.not_after) {
            labels.extend(NodeContributor::as_labels(&validity));
        }

        // Add key usage labels from KeyUsage value object (via NodeContributor trait)
        let key_usage = crate::value_objects::KeyUsage::from_string_list(&self.key_usage);
        labels.extend(NodeContributor::as_labels(&key_usage));

        // Add SAN labels from SubjectAlternativeName value object (via NodeContributor trait)
        let san = crate::value_objects::SubjectAlternativeName::from_string_list(&self.san);
        labels.extend(NodeContributor::as_labels(&san));

        labels
    }

    /// Aggregate properties from all composed ValueObjects
    ///
    /// Collects properties from:
    /// - Subject and issuer names
    /// - Validity period details
    /// - Key usage flags
    /// - SAN entries
    pub fn aggregate_properties(&self) -> Vec<(crate::value_objects::PropertyKey, crate::value_objects::PropertyValue)> {
        use crate::value_objects::{PropertyKey, PropertyValue, NodeContributor};

        let mut props = Vec::new();

        // Add core certificate properties
        props.push((PropertyKey::new("certificate_id"), PropertyValue::uuid(self.id.as_uuid())));
        props.push((PropertyKey::new("certificate_type"), PropertyValue::string(format!("{}", self.cert_type))));
        props.push((PropertyKey::new("subject"), PropertyValue::string(&self.subject)));
        props.push((PropertyKey::new("issuer"), PropertyValue::string(&self.issuer)));

        // Add validity properties from CertificateValidity value object
        if let Ok(validity) = crate::value_objects::CertificateValidity::new(self.not_before, self.not_after) {
            props.extend(validity.as_properties());
        }

        // Add key usage properties from KeyUsage value object
        let key_usage = crate::value_objects::KeyUsage::from_string_list(&self.key_usage);
        props.extend(key_usage.as_properties());

        // Add SAN properties from SubjectAlternativeName value object
        let san = crate::value_objects::SubjectAlternativeName::from_string_list(&self.san);
        props.extend(san.as_properties());

        props
    }

    /// Aggregate relationships from all composed ValueObjects
    ///
    /// Note: Most certificate relationships (issuer, key) are at the entity level,
    /// not from ValueObjects. This method collects any ValueObject-sourced relationships.
    pub fn aggregate_relationships(&self) -> Vec<crate::value_objects::ValueRelationship> {
        // Currently, certificate relationships are primarily entity-level (issuer-cert, key-cert)
        // which are handled in the LiftedGraph edge creation, not ValueObject contributions.
        // This method is provided for consistency and future extension.
        Vec::new()
    }
}

/// Implement AggregateContributions trait for Certificate
impl crate::value_objects::AggregateContributions for Certificate {
    fn aggregate_labels(&self) -> Vec<crate::value_objects::Label> {
        Certificate::aggregate_labels(self)
    }

    fn aggregate_properties(&self) -> Vec<(crate::value_objects::PropertyKey, crate::value_objects::PropertyValue)> {
        Certificate::aggregate_properties(self)
    }

    fn aggregate_relationships(&self) -> Vec<crate::value_objects::ValueRelationship> {
        Certificate::aggregate_relationships(self)
    }
}

/// Cryptographic key for graph visualization
///
/// This type is used in `DomainNodeData` for rendering keys in the graph.
/// It uses phantom-typed `KeyId` for compile-time safety.
#[derive(Debug, Clone)]
pub struct CryptographicKey {
    pub id: KeyId,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CryptographicKey {
    /// Create a new cryptographic key
    pub fn new(
        id: KeyId,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self { id, algorithm, purpose, expires_at }
    }

    /// Check if key is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Utc::now() > exp)
            .unwrap_or(false)
    }
}
