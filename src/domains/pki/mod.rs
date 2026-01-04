// Copyright (c) 2025 - Cowboy AI, LLC.

//! PKI Bounded Context
//!
//! This module defines the coproduct for the PKI (Public Key Infrastructure)
//! bounded context, handling certificates and cryptographic keys.
//!
//! ## Entities in this Context
//! - RootCertificate (tier 0)
//! - IntermediateCertificate (tier 1)
//! - LeafCertificate (tier 2)
//! - CryptographicKey (tier 2)
//!
//! ## Published Language
//!
//! The `published` submodule provides reference types for cross-context
//! communication. Other contexts should use these types instead of
//! importing internal PKI types directly.

// Published Language for cross-context communication
pub mod published;

// Anti-Corruption Layer for Organization context
pub mod acl;

use std::fmt;
use uuid::Uuid;

use crate::domain::pki::{Certificate, CryptographicKey};

/// Injection tag for PKI bounded context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PkiInjection {
    RootCertificate,
    IntermediateCertificate,
    LeafCertificate,
    Key,
}

impl PkiInjection {
    /// Display name for this entity type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::RootCertificate => "Root Certificate",
            Self::IntermediateCertificate => "Intermediate Certificate",
            Self::LeafCertificate => "Leaf Certificate",
            Self::Key => "Key",
        }
    }

    /// Layout tier for hierarchical visualization
    pub fn layout_tier(&self) -> u8 {
        match self {
            Self::RootCertificate => 0,
            Self::IntermediateCertificate => 1,
            Self::LeafCertificate | Self::Key => 2,
        }
    }

    /// Check if this is a certificate type
    pub fn is_certificate(&self) -> bool {
        matches!(
            self,
            Self::RootCertificate | Self::IntermediateCertificate | Self::LeafCertificate
        )
    }
}

impl fmt::Display for PkiInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Inner data for PKI context entities
#[derive(Debug, Clone)]
pub enum PkiData {
    RootCertificate(Certificate),
    IntermediateCertificate(Certificate),
    LeafCertificate(Certificate),
    Key(CryptographicKey),
}

/// PKI Entity - Coproduct of PKI-related types
#[derive(Debug, Clone)]
pub struct PkiEntity {
    injection: PkiInjection,
    data: PkiData,
}

impl PkiEntity {
    // ========================================================================
    // Injection Functions
    // ========================================================================

    /// Inject Root Certificate into coproduct
    pub fn inject_root_certificate(cert: Certificate) -> Self {
        Self {
            injection: PkiInjection::RootCertificate,
            data: PkiData::RootCertificate(cert),
        }
    }

    /// Inject Intermediate Certificate into coproduct
    pub fn inject_intermediate_certificate(cert: Certificate) -> Self {
        Self {
            injection: PkiInjection::IntermediateCertificate,
            data: PkiData::IntermediateCertificate(cert),
        }
    }

    /// Inject Leaf Certificate into coproduct
    pub fn inject_leaf_certificate(cert: Certificate) -> Self {
        Self {
            injection: PkiInjection::LeafCertificate,
            data: PkiData::LeafCertificate(cert),
        }
    }

    /// Inject Cryptographic Key into coproduct
    pub fn inject_key(key: CryptographicKey) -> Self {
        Self {
            injection: PkiInjection::Key,
            data: PkiData::Key(key),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the injection tag
    pub fn injection(&self) -> PkiInjection {
        self.injection
    }

    /// Get reference to inner data
    pub fn data(&self) -> &PkiData {
        &self.data
    }

    /// Get entity ID
    pub fn id(&self) -> Uuid {
        match &self.data {
            PkiData::RootCertificate(c) => c.id.as_uuid(),
            PkiData::IntermediateCertificate(c) => c.id.as_uuid(),
            PkiData::LeafCertificate(c) => c.id.as_uuid(),
            PkiData::Key(k) => k.id.as_uuid(),
        }
    }

    /// Get entity name/subject
    pub fn name(&self) -> String {
        match &self.data {
            PkiData::RootCertificate(c) => c.subject.clone(),
            PkiData::IntermediateCertificate(c) => c.subject.clone(),
            PkiData::LeafCertificate(c) => c.subject.clone(),
            PkiData::Key(k) => format!("{:?} ({:?})", k.purpose, k.algorithm),
        }
    }

    // ========================================================================
    // Universal Property (Fold)
    // ========================================================================

    /// Apply a fold to this entity
    pub fn fold<F: FoldPkiEntity>(&self, folder: &F) -> F::Output {
        match &self.data {
            PkiData::RootCertificate(c) => folder.fold_root_certificate(c),
            PkiData::IntermediateCertificate(c) => folder.fold_intermediate_certificate(c),
            PkiData::LeafCertificate(c) => folder.fold_leaf_certificate(c),
            PkiData::Key(k) => folder.fold_key(k),
        }
    }
}

/// Universal property trait for PkiEntity coproduct
pub trait FoldPkiEntity {
    type Output;

    fn fold_root_certificate(&self, cert: &Certificate) -> Self::Output;
    fn fold_intermediate_certificate(&self, cert: &Certificate) -> Self::Output;
    fn fold_leaf_certificate(&self, cert: &Certificate) -> Self::Output;
    fn fold_key(&self, key: &CryptographicKey) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct InjectionFolder;

    impl FoldPkiEntity for InjectionFolder {
        type Output = PkiInjection;

        fn fold_root_certificate(&self, _: &Certificate) -> Self::Output {
            PkiInjection::RootCertificate
        }
        fn fold_intermediate_certificate(&self, _: &Certificate) -> Self::Output {
            PkiInjection::IntermediateCertificate
        }
        fn fold_leaf_certificate(&self, _: &Certificate) -> Self::Output {
            PkiInjection::LeafCertificate
        }
        fn fold_key(&self, _: &CryptographicKey) -> Self::Output {
            PkiInjection::Key
        }
    }
}
