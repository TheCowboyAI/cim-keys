// Copyright (c) 2025 - Cowboy AI, LLC.

//! X.509 Basic Constraints Extension (RFC 5280 Section 4.2.1.9)
//!
//! Provides type-safe basic constraints value object that enforces RFC 5280
//! requirements for CA certificates and path length constraints.
//!
//! ## Basic Constraints
//!
//! The basic constraints extension identifies whether the subject of the
//! certificate is a CA and the maximum depth of valid certification paths
//! that include this certificate.
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::value_objects::x509::BasicConstraints;
//!
//! // Root CA - unlimited path length
//! let root = BasicConstraints::ca();
//!
//! // Intermediate CA - can only issue end-entity certs
//! let intermediate = BasicConstraints::ca_with_path_len(0);
//!
//! // End-entity certificate (not a CA)
//! let leaf = BasicConstraints::end_entity();
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

/// Basic Constraints Extension value object
///
/// Per RFC 5280, this extension MUST appear in all CA certificates.
/// It identifies whether the certificate subject is a CA and constrains
/// the certification path length.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicConstraints {
    /// Whether this is a CA certificate
    is_ca: bool,
    /// Maximum number of intermediate certificates between this CA and end-entity
    /// None means unlimited, Some(0) means can only issue end-entity certs
    path_len_constraint: Option<u32>,
    /// Whether this extension is critical (MUST be critical for CA certs per RFC 5280)
    pub critical: bool,
}

impl BasicConstraints {
    /// Create new basic constraints
    ///
    /// For CA certificates, critical should be true per RFC 5280.
    pub fn new(is_ca: bool, path_len_constraint: Option<u32>) -> Self {
        Self {
            is_ca,
            path_len_constraint,
            // RFC 5280: MUST be critical if is_ca is true
            critical: is_ca,
        }
    }

    /// Create basic constraints for a CA certificate (unlimited path length)
    pub fn ca() -> Self {
        Self::new(true, None)
    }

    /// Create basic constraints for a CA certificate with path length constraint
    ///
    /// - `path_len = 0`: Can only issue end-entity certificates
    /// - `path_len = 1`: Can issue one level of intermediate CAs
    /// - etc.
    pub fn ca_with_path_len(path_len: u32) -> Self {
        Self::new(true, Some(path_len))
    }

    /// Create basic constraints for an end-entity certificate (non-CA)
    pub fn end_entity() -> Self {
        Self::new(false, None)
    }

    /// Check if this is a CA certificate
    pub fn is_ca(&self) -> bool {
        self.is_ca
    }

    /// Check if this is an end-entity (non-CA) certificate
    pub fn is_end_entity(&self) -> bool {
        !self.is_ca
    }

    /// Get the path length constraint (if any)
    pub fn path_len_constraint(&self) -> Option<u32> {
        self.path_len_constraint
    }

    /// Check if path length is constrained
    pub fn has_path_len_constraint(&self) -> bool {
        self.is_ca && self.path_len_constraint.is_some()
    }

    /// Check if this CA can issue other CA certificates
    ///
    /// A CA can issue other CA certs if:
    /// - It is a CA
    /// - path_len_constraint is None (unlimited) or > 0
    pub fn can_issue_ca_certs(&self) -> bool {
        if !self.is_ca {
            return false;
        }
        match self.path_len_constraint {
            None => true,                // Unlimited
            Some(0) => false,            // Can only issue end-entity
            Some(_) => true,             // Can issue at least one level of CA
        }
    }

    /// Calculate the path length constraint for a subordinate CA
    ///
    /// If this CA issues another CA, what should the subordinate's
    /// path_len_constraint be?
    pub fn subordinate_path_len(&self) -> Option<u32> {
        if !self.is_ca {
            return None;
        }
        match self.path_len_constraint {
            None => None,                     // Subordinate also unlimited
            Some(0) => None,                  // Should not issue CA certs
            Some(n) => Some(n - 1),           // Decrement path length
        }
    }

    /// Builder pattern - set criticality
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }

    // ========================================================================
    // Graph Label Generation
    // ========================================================================

    /// Generate labels for graph node based on basic constraints
    pub fn as_labels(&self) -> Vec<String> {
        let mut labels = Vec::new();

        if self.is_ca {
            labels.push("CACertificate".to_string());

            if self.can_issue_ca_certs() {
                labels.push("CanIssueCA".to_string());
            } else {
                labels.push("EndEntityIssuerOnly".to_string());
            }

            match self.path_len_constraint {
                None => labels.push("UnlimitedPathLen".to_string()),
                Some(0) => labels.push("PathLen0".to_string()),
                Some(n) => labels.push(format!("PathLen{}", n)),
            }
        } else {
            labels.push("EndEntityCertificate".to_string());
        }

        labels
    }

    // ========================================================================
    // Convenience Constructors for Common Certificate Types
    // ========================================================================

    /// Root CA - unlimited path length
    pub fn root_ca() -> Self {
        Self::ca()
    }

    /// Intermediate CA that can issue other intermediate CAs
    /// Typically path_len = 1 or more
    pub fn intermediate_ca() -> Self {
        Self::ca_with_path_len(1)
    }

    /// Issuing CA - can only issue end-entity certificates
    /// This is the lowest level CA in a hierarchy
    pub fn issuing_ca() -> Self {
        Self::ca_with_path_len(0)
    }

    /// TLS server certificate (end-entity)
    pub fn tls_server() -> Self {
        Self::end_entity()
    }

    /// TLS client certificate (end-entity)
    pub fn tls_client() -> Self {
        Self::end_entity()
    }

    /// Code signing certificate (end-entity)
    pub fn code_signing() -> Self {
        Self::end_entity()
    }
}

impl Default for BasicConstraints {
    fn default() -> Self {
        Self::end_entity()
    }
}

impl fmt::Display for BasicConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_ca {
            match self.path_len_constraint {
                None => write!(f, "CA:TRUE"),
                Some(n) => write!(f, "CA:TRUE, pathlen:{}", n),
            }
        } else {
            write!(f, "CA:FALSE")
        }
    }
}

impl DomainConcept for BasicConstraints {}
impl ValueObject for BasicConstraints {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ca_basic_constraints() {
        let bc = BasicConstraints::ca();

        assert!(bc.is_ca());
        assert!(!bc.is_end_entity());
        assert!(bc.path_len_constraint().is_none());
        assert!(bc.can_issue_ca_certs());
        assert!(bc.critical); // Must be critical for CA
    }

    #[test]
    fn test_ca_with_path_len_zero() {
        let bc = BasicConstraints::ca_with_path_len(0);

        assert!(bc.is_ca());
        assert_eq!(bc.path_len_constraint(), Some(0));
        assert!(!bc.can_issue_ca_certs()); // path_len=0 means end-entity only
        assert!(bc.has_path_len_constraint());
    }

    #[test]
    fn test_ca_with_path_len_one() {
        let bc = BasicConstraints::ca_with_path_len(1);

        assert!(bc.is_ca());
        assert_eq!(bc.path_len_constraint(), Some(1));
        assert!(bc.can_issue_ca_certs());
        assert_eq!(bc.subordinate_path_len(), Some(0)); // Subordinate gets 0
    }

    #[test]
    fn test_end_entity() {
        let bc = BasicConstraints::end_entity();

        assert!(!bc.is_ca());
        assert!(bc.is_end_entity());
        assert!(bc.path_len_constraint().is_none());
        assert!(!bc.can_issue_ca_certs());
        assert!(!bc.critical); // Not critical for end-entity
    }

    #[test]
    fn test_subordinate_path_len() {
        // Root CA (unlimited)
        let root = BasicConstraints::ca();
        assert_eq!(root.subordinate_path_len(), None);

        // Intermediate with path_len=2
        let intermediate = BasicConstraints::ca_with_path_len(2);
        assert_eq!(intermediate.subordinate_path_len(), Some(1));

        // Issuing CA with path_len=0
        let issuing = BasicConstraints::ca_with_path_len(0);
        assert_eq!(issuing.subordinate_path_len(), None); // Should not issue CAs

        // End-entity
        let ee = BasicConstraints::end_entity();
        assert_eq!(ee.subordinate_path_len(), None);
    }

    #[test]
    fn test_root_ca_convenience() {
        let bc = BasicConstraints::root_ca();

        assert!(bc.is_ca());
        assert!(bc.path_len_constraint().is_none());
    }

    #[test]
    fn test_intermediate_ca_convenience() {
        let bc = BasicConstraints::intermediate_ca();

        assert!(bc.is_ca());
        assert_eq!(bc.path_len_constraint(), Some(1));
    }

    #[test]
    fn test_issuing_ca_convenience() {
        let bc = BasicConstraints::issuing_ca();

        assert!(bc.is_ca());
        assert_eq!(bc.path_len_constraint(), Some(0));
        assert!(!bc.can_issue_ca_certs());
    }

    #[test]
    fn test_labels_ca() {
        let bc = BasicConstraints::ca();
        let labels = bc.as_labels();

        assert!(labels.contains(&"CACertificate".to_string()));
        assert!(labels.contains(&"CanIssueCA".to_string()));
        assert!(labels.contains(&"UnlimitedPathLen".to_string()));
    }

    #[test]
    fn test_labels_issuing_ca() {
        let bc = BasicConstraints::issuing_ca();
        let labels = bc.as_labels();

        assert!(labels.contains(&"CACertificate".to_string()));
        assert!(labels.contains(&"EndEntityIssuerOnly".to_string()));
        assert!(labels.contains(&"PathLen0".to_string()));
    }

    #[test]
    fn test_labels_end_entity() {
        let bc = BasicConstraints::end_entity();
        let labels = bc.as_labels();

        assert!(labels.contains(&"EndEntityCertificate".to_string()));
        assert!(!labels.contains(&"CACertificate".to_string()));
    }

    #[test]
    fn test_display_ca() {
        let bc = BasicConstraints::ca();
        assert_eq!(format!("{}", bc), "CA:TRUE");
    }

    #[test]
    fn test_display_ca_with_path_len() {
        let bc = BasicConstraints::ca_with_path_len(2);
        assert_eq!(format!("{}", bc), "CA:TRUE, pathlen:2");
    }

    #[test]
    fn test_display_end_entity() {
        let bc = BasicConstraints::end_entity();
        assert_eq!(format!("{}", bc), "CA:FALSE");
    }

    #[test]
    fn test_default_is_end_entity() {
        let bc = BasicConstraints::default();
        assert!(bc.is_end_entity());
    }
}
