// Copyright (c) 2025 - Cowboy AI, LLC.

//! Certificate Message Definitions
//!
//! This module defines the message types for the Certificate bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility toggles
//! 2. **Metadata Form**: X.509 certificate fields
//! 3. **Intermediate CA**: Create and select intermediate CAs
//! 4. **Server Certificate**: Generate server certificates with SANs
//! 5. **Client Certificate**: mTLS client certificate generation
//! 6. **Chain View**: Certificate chain visualization

use uuid::Uuid;

use crate::projections::CertificateEntry;

/// Certificate Message
///
/// Organized by sub-domain:
/// - UI State (3 messages)
/// - Metadata Form (6 messages)
/// - Intermediate CA (3 messages)
/// - Server Certificate (4 messages)
/// - Client Certificate (2 messages)
/// - Chain View (1 message)
/// - Loading (1 message)
#[derive(Debug, Clone)]
pub enum CertificateMessage {
    // === UI State ===
    /// Toggle certificates list section visibility
    ToggleCertificatesSection,
    /// Toggle intermediate CA subsection visibility
    ToggleIntermediateCA,
    /// Toggle server certificate subsection visibility
    ToggleServerCert,

    // === Metadata Form ===
    /// X.509 Organization field changed
    OrganizationChanged(String),
    /// X.509 Organizational Unit field changed
    OrganizationalUnitChanged(String),
    /// X.509 Locality field changed
    LocalityChanged(String),
    /// X.509 State/Province field changed
    StateProvinceChanged(String),
    /// X.509 Country field changed
    CountryChanged(String),
    /// Certificate validity period (days) changed
    ValidityDaysChanged(String),

    // === Intermediate CA ===
    /// Intermediate CA name input changed
    IntermediateCANameChanged(String),
    /// Select an existing intermediate CA
    SelectIntermediateCA(String),
    /// Generate a new intermediate CA
    GenerateIntermediateCA,

    // === Server Certificate ===
    /// Server certificate Common Name changed
    ServerCNChanged(String),
    /// Server certificate Subject Alternative Names changed
    ServerSANsChanged(String),
    /// Select storage location for certificate
    SelectLocation(String),
    /// Generate server certificate
    GenerateServerCert,

    // === Client Certificate (mTLS) ===
    /// Generate mTLS client certificate
    GenerateClientCert,
    /// Client certificate generation result
    ClientCertGenerated(Result<String, String>),

    // === Chain View ===
    /// Select certificate for chain visualization
    SelectForChainView(Uuid),

    // === Loading ===
    /// Certificates loaded from manifest
    CertificatesLoaded(Vec<CertificateEntry>),

    // === Sprint 88: Certificate Selection for Import ===
    /// Select a leaf certificate for YubiKey import
    SelectLeafCertForImport(Option<Uuid>),
    /// Validate a certificate for RFC 5280 compliance
    ValidateCertificate(Uuid),
    /// Certificate validation completed
    CertificateValidated {
        cert_id: Uuid,
        result: crate::crypto::rfc5280::Rfc5280ValidationResult,
    },
}

/// Parse SANs from comma-separated string
pub fn parse_sans(sans_input: &str) -> Vec<String> {
    sans_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sans_empty() {
        assert!(parse_sans("").is_empty());
    }

    #[test]
    fn test_parse_sans_single() {
        let sans = parse_sans("localhost");
        assert_eq!(sans, vec!["localhost"]);
    }

    #[test]
    fn test_parse_sans_multiple() {
        let sans = parse_sans("localhost, 127.0.0.1, example.com");
        assert_eq!(sans, vec!["localhost", "127.0.0.1", "example.com"]);
    }

    #[test]
    fn test_parse_sans_trims_whitespace() {
        let sans = parse_sans("  foo  ,  bar  ");
        assert_eq!(sans, vec!["foo", "bar"]);
    }

    #[test]
    fn test_parse_sans_filters_empty() {
        let sans = parse_sans("foo,,bar,");
        assert_eq!(sans, vec!["foo", "bar"]);
    }
}
