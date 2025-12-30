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
