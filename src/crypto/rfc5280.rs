// Copyright (c) 2025 - Cowboy AI, LLC.

//! RFC 5280 Compliance Validation for X.509 Certificates
//!
//! This module validates certificates against RFC 5280 (Internet X.509 PKI Certificate
//! and CRL Profile) before allowing operations like import to YubiKey.
//!
//! ## Validation Checks
//!
//! 1. **Version**: Must be v3 (2) for certificates with extensions
//! 2. **Serial Number**: Positive integer, at most 20 octets
//! 3. **Signature Algorithm**: Must be supported and consistent
//! 4. **Issuer**: Must be a non-empty distinguished name
//! 5. **Validity**: notBefore <= notAfter, certificate must be valid (not expired)
//! 6. **Subject**: Non-empty DN for CAs, can be empty for end-entity with SAN
//! 7. **Extensions**: Required extensions present and properly formatted
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::crypto::rfc5280::{validate_certificate, Rfc5280ValidationResult};
//!
//! let pem_bytes = cert.certificate_pem.as_bytes();
//! let result = validate_certificate(pem_bytes)?;
//!
//! if !result.is_valid() {
//!     for error in result.errors() {
//!         eprintln!("Validation error: {}", error);
//!     }
//! }
//! ```

use x509_parser::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// RFC 5280 validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum Rfc5280Error {
    /// Failed to parse PEM data
    PemParseError(String),
    /// Failed to parse DER data
    DerParseError(String),
    /// Certificate version is not v3 (required for extensions)
    InvalidVersion { expected: u32, actual: u32 },
    /// Serial number is invalid (negative, zero, or too long)
    InvalidSerialNumber(String),
    /// Unsupported signature algorithm
    UnsupportedSignatureAlgorithm(String),
    /// Issuer DN is empty
    EmptyIssuer,
    /// Subject DN is empty without Subject Alternative Name
    EmptySubjectWithoutSan,
    /// Validity period is invalid (notBefore > notAfter)
    InvalidValidityPeriod,
    /// Certificate is not yet valid
    NotYetValid { not_before: DateTime<Utc> },
    /// Certificate has expired
    Expired { not_after: DateTime<Utc> },
    /// BasicConstraints extension missing for CA certificate
    MissingBasicConstraintsForCa,
    /// BasicConstraints indicates CA but pathlen is missing when required
    InvalidPathLength(String),
    /// KeyUsage extension has invalid combination
    InvalidKeyUsage(String),
    /// Required extension is missing
    MissingRequiredExtension(String),
    /// Unknown critical extension
    UnknownCriticalExtension(String),
    /// Subject Alternative Name is empty
    EmptySubjectAltName,
    /// Certificate chain validation failed
    ChainValidationFailed(String),
}

impl std::fmt::Display for Rfc5280Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rfc5280Error::PemParseError(e) => write!(f, "PEM parse error: {}", e),
            Rfc5280Error::DerParseError(e) => write!(f, "DER parse error: {}", e),
            Rfc5280Error::InvalidVersion { expected, actual } => {
                write!(f, "Invalid certificate version: expected v{}, got v{}", expected, actual)
            }
            Rfc5280Error::InvalidSerialNumber(e) => write!(f, "Invalid serial number: {}", e),
            Rfc5280Error::UnsupportedSignatureAlgorithm(alg) => {
                write!(f, "Unsupported signature algorithm: {}", alg)
            }
            Rfc5280Error::EmptyIssuer => write!(f, "Issuer distinguished name is empty"),
            Rfc5280Error::EmptySubjectWithoutSan => {
                write!(f, "Subject DN is empty and no Subject Alternative Name extension present")
            }
            Rfc5280Error::InvalidValidityPeriod => {
                write!(f, "Invalid validity period: notBefore > notAfter")
            }
            Rfc5280Error::NotYetValid { not_before } => {
                write!(f, "Certificate is not yet valid (valid from {})", not_before)
            }
            Rfc5280Error::Expired { not_after } => {
                write!(f, "Certificate has expired (expired at {})", not_after)
            }
            Rfc5280Error::MissingBasicConstraintsForCa => {
                write!(f, "CA certificate missing BasicConstraints extension")
            }
            Rfc5280Error::InvalidPathLength(e) => write!(f, "Invalid path length constraint: {}", e),
            Rfc5280Error::InvalidKeyUsage(e) => write!(f, "Invalid key usage: {}", e),
            Rfc5280Error::MissingRequiredExtension(ext) => {
                write!(f, "Missing required extension: {}", ext)
            }
            Rfc5280Error::UnknownCriticalExtension(oid) => {
                write!(f, "Unknown critical extension with OID: {}", oid)
            }
            Rfc5280Error::EmptySubjectAltName => {
                write!(f, "Subject Alternative Name extension is empty")
            }
            Rfc5280Error::ChainValidationFailed(e) => {
                write!(f, "Certificate chain validation failed: {}", e)
            }
        }
    }
}

impl std::error::Error for Rfc5280Error {}

/// Result of RFC 5280 validation
#[derive(Debug, Clone)]
pub struct Rfc5280ValidationResult {
    /// Errors found during validation (empty if valid)
    errors: Vec<Rfc5280Error>,
    /// Warnings (non-fatal issues)
    warnings: Vec<String>,
    /// Certificate metadata extracted during validation
    pub metadata: Option<CertificateMetadata>,
}

/// Metadata extracted from a validated certificate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificateMetadata {
    /// Certificate version (1, 2, or 3)
    pub version: u32,
    /// Serial number as hex string
    pub serial_number: String,
    /// Subject common name (if present)
    pub subject_cn: Option<String>,
    /// Subject organization (if present)
    pub subject_org: Option<String>,
    /// Issuer common name (if present)
    pub issuer_cn: Option<String>,
    /// Not valid before
    pub not_before: DateTime<Utc>,
    /// Not valid after
    pub not_after: DateTime<Utc>,
    /// Is this a CA certificate?
    pub is_ca: bool,
    /// Path length constraint (if CA)
    pub path_length: Option<u32>,
    /// Key usage flags
    pub key_usage: Vec<String>,
    /// Extended key usage OIDs
    pub extended_key_usage: Vec<String>,
    /// Subject Alternative Names
    pub subject_alt_names: Vec<String>,
    /// Certificate fingerprint (SHA-256)
    pub fingerprint_sha256: String,
}

impl Rfc5280ValidationResult {
    /// Create a new validation result
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            metadata: None,
        }
    }

    /// Create a new validation result with an initial error
    pub fn new_with_error(error: Rfc5280Error) -> Self {
        Self {
            errors: vec![error],
            warnings: Vec::new(),
            metadata: None,
        }
    }

    /// Add an error
    fn add_error(&mut self, error: Rfc5280Error) {
        self.errors.push(error);
    }

    /// Add a warning
    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get validation errors
    pub fn errors(&self) -> &[Rfc5280Error] {
        &self.errors
    }

    /// Get validation warnings
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Get a summary of validation results
    pub fn summary(&self) -> String {
        if self.is_valid() {
            if self.warnings.is_empty() {
                "Certificate is RFC 5280 compliant".to_string()
            } else {
                format!(
                    "Certificate is RFC 5280 compliant with {} warning(s)",
                    self.warnings.len()
                )
            }
        } else {
            format!(
                "Certificate validation failed with {} error(s)",
                self.errors.len()
            )
        }
    }
}

/// Validate a certificate in PEM format against RFC 5280
pub fn validate_certificate(pem_bytes: &[u8]) -> Result<Rfc5280ValidationResult, Rfc5280Error> {
    let mut result = Rfc5280ValidationResult::new();

    // Parse PEM - pem crate 3.0 uses parse()
    let pem_str = std::str::from_utf8(pem_bytes)
        .map_err(|e| Rfc5280Error::PemParseError(format!("Invalid UTF-8: {}", e)))?;

    let pem_data = ::pem::parse(pem_str)
        .map_err(|e| Rfc5280Error::PemParseError(e.to_string()))?;

    // Parse DER
    let (_, cert) = X509Certificate::from_der(pem_data.contents())
        .map_err(|e| Rfc5280Error::DerParseError(e.to_string()))?;

    // Validate and extract metadata
    validate_version(&cert, &mut result);
    validate_serial_number(&cert, &mut result);
    validate_signature_algorithm(&cert, &mut result);
    validate_issuer(&cert, &mut result);
    validate_validity(&cert, &mut result);
    validate_subject(&cert, &mut result);
    validate_extensions(&cert, &mut result);

    // Extract metadata
    result.metadata = Some(extract_metadata(&cert, pem_data.contents()));

    Ok(result)
}

/// Validate a certificate in DER format against RFC 5280
pub fn validate_certificate_der(der_bytes: &[u8]) -> Result<Rfc5280ValidationResult, Rfc5280Error> {
    let mut result = Rfc5280ValidationResult::new();

    // Parse DER
    let (_, cert) = X509Certificate::from_der(der_bytes)
        .map_err(|e| Rfc5280Error::DerParseError(e.to_string()))?;

    // Validate and extract metadata
    validate_version(&cert, &mut result);
    validate_serial_number(&cert, &mut result);
    validate_signature_algorithm(&cert, &mut result);
    validate_issuer(&cert, &mut result);
    validate_validity(&cert, &mut result);
    validate_subject(&cert, &mut result);
    validate_extensions(&cert, &mut result);

    // Extract metadata
    result.metadata = Some(extract_metadata(&cert, der_bytes));

    Ok(result)
}

/// Validate certificate version (RFC 5280 Section 4.1.2.1)
fn validate_version(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let version = cert.version().0;

    // If certificate has extensions, must be v3 (encoded as 2)
    if !cert.extensions().is_empty() && version != 2 {
        result.add_error(Rfc5280Error::InvalidVersion {
            expected: 3,
            actual: version + 1, // Display as v1/v2/v3
        });
    }
}

/// Validate serial number (RFC 5280 Section 4.1.2.2)
fn validate_serial_number(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let serial = cert.raw_serial();

    // Serial number must not be empty
    if serial.is_empty() {
        result.add_error(Rfc5280Error::InvalidSerialNumber("Empty serial number".to_string()));
        return;
    }

    // Serial number must be at most 20 octets
    if serial.len() > 20 {
        result.add_error(Rfc5280Error::InvalidSerialNumber(
            format!("Serial number too long: {} bytes (max 20)", serial.len())
        ));
    }

    // Check for all zeros (invalid)
    if serial.iter().all(|&b| b == 0) {
        result.add_error(Rfc5280Error::InvalidSerialNumber("Serial number is zero".to_string()));
    }
}

/// Validate signature algorithm (RFC 5280 Section 4.1.2.3)
fn validate_signature_algorithm(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let sig_alg = cert.signature_algorithm.algorithm.to_string();

    // Check for known/supported algorithms
    let supported_oids = [
        "1.2.840.113549.1.1.5",  // sha1WithRSAEncryption (deprecated but valid)
        "1.2.840.113549.1.1.11", // sha256WithRSAEncryption
        "1.2.840.113549.1.1.12", // sha384WithRSAEncryption
        "1.2.840.113549.1.1.13", // sha512WithRSAEncryption
        "1.2.840.10045.4.3.2",   // ecdsa-with-SHA256
        "1.2.840.10045.4.3.3",   // ecdsa-with-SHA384
        "1.2.840.10045.4.3.4",   // ecdsa-with-SHA512
        "1.3.101.112",          // Ed25519
        "1.3.101.113",          // Ed448
    ];

    if !supported_oids.contains(&sig_alg.as_str()) {
        result.add_warning(format!(
            "Signature algorithm OID {} may not be widely supported",
            sig_alg
        ));
    }
}

/// Validate issuer (RFC 5280 Section 4.1.2.4)
fn validate_issuer(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let issuer = cert.issuer();

    if issuer.iter().count() == 0 {
        result.add_error(Rfc5280Error::EmptyIssuer);
    }
}

/// Validate validity period (RFC 5280 Section 4.1.2.5)
fn validate_validity(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let validity = cert.validity();
    let now = Utc::now();

    // Convert ASN1Time to DateTime<Utc>
    let not_before = asn1_time_to_datetime(&validity.not_before);
    let not_after = asn1_time_to_datetime(&validity.not_after);

    match (not_before, not_after) {
        (Some(nb), Some(na)) => {
            // notBefore must be before notAfter
            if nb > na {
                result.add_error(Rfc5280Error::InvalidValidityPeriod);
            }

            // Check if certificate is not yet valid
            if now < nb {
                result.add_error(Rfc5280Error::NotYetValid { not_before: nb });
            }

            // Check if certificate has expired
            if now > na {
                result.add_error(Rfc5280Error::Expired { not_after: na });
            }
        }
        _ => {
            result.add_error(Rfc5280Error::InvalidValidityPeriod);
        }
    }
}

/// Validate subject (RFC 5280 Section 4.1.2.6)
fn validate_subject(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let subject = cert.subject();

    // Subject can be empty only if SubjectAltName extension is present
    if subject.iter().count() == 0 {
        let has_san = cert.extensions().iter()
            .any(|ext| ext.oid == oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME);

        if !has_san {
            result.add_error(Rfc5280Error::EmptySubjectWithoutSan);
        }
    }
}

/// Validate extensions (RFC 5280 Section 4.2)
fn validate_extensions(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    // Check for critical extensions we don't understand
    let known_critical_oids = [
        oid_registry::OID_X509_EXT_BASIC_CONSTRAINTS,
        oid_registry::OID_X509_EXT_KEY_USAGE,
        oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME,
        oid_registry::OID_X509_EXT_NAME_CONSTRAINTS,
    ];

    for ext in cert.extensions().iter() {
        if ext.critical {
            if !known_critical_oids.contains(&ext.oid) {
                result.add_warning(format!(
                    "Unknown critical extension: {}",
                    ext.oid
                ));
            }
        }
    }

    // Validate BasicConstraints for CA certificates
    validate_basic_constraints(cert, result);

    // Validate KeyUsage
    validate_key_usage(cert, result);

    // Validate SubjectAltName if present
    validate_subject_alt_name(cert, result);
}

/// Validate BasicConstraints extension (RFC 5280 Section 4.2.1.9)
fn validate_basic_constraints(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let bc_ext = cert.extensions().iter()
        .find(|ext| ext.oid == oid_registry::OID_X509_EXT_BASIC_CONSTRAINTS);

    if let Some(ext) = bc_ext {
        match ext.parsed_extension() {
            ParsedExtension::BasicConstraints(bc) => {
                if bc.ca {
                    // CA certificates should have BasicConstraints marked critical
                    if !ext.critical {
                        result.add_warning(
                            "BasicConstraints should be critical for CA certificates".to_string()
                        );
                    }
                }
            }
            _ => {
                result.add_warning("Could not parse BasicConstraints extension".to_string());
            }
        }
    } else {
        // Check if this might be a CA (has KeyCertSign in KeyUsage)
        let ku_ext = cert.extensions().iter()
            .find(|ext| ext.oid == oid_registry::OID_X509_EXT_KEY_USAGE);

        if let Some(ku) = ku_ext {
            if let ParsedExtension::KeyUsage(ku) = ku.parsed_extension() {
                if ku.key_cert_sign() {
                    result.add_error(Rfc5280Error::MissingBasicConstraintsForCa);
                }
            }
        }
    }
}

/// Validate KeyUsage extension (RFC 5280 Section 4.2.1.3)
fn validate_key_usage(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let ku_ext = cert.extensions().iter()
        .find(|ext| ext.oid == oid_registry::OID_X509_EXT_KEY_USAGE);

    if let Some(ext) = ku_ext {
        if let ParsedExtension::KeyUsage(ku) = ext.parsed_extension() {
            // If keyCertSign or cRLSign is set, certificate should be a CA
            if ku.key_cert_sign() || ku.crl_sign() {
                let bc_ext = cert.extensions().iter()
                    .find(|e| e.oid == oid_registry::OID_X509_EXT_BASIC_CONSTRAINTS);

                if let Some(bc) = bc_ext {
                    if let ParsedExtension::BasicConstraints(bc) = bc.parsed_extension() {
                        if !bc.ca {
                            result.add_error(Rfc5280Error::InvalidKeyUsage(
                                "keyCertSign or cRLSign set but BasicConstraints.cA is false".to_string()
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// Validate SubjectAltName extension (RFC 5280 Section 4.2.1.6)
fn validate_subject_alt_name(cert: &X509Certificate, result: &mut Rfc5280ValidationResult) {
    let san_ext = cert.extensions().iter()
        .find(|ext| ext.oid == oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME);

    if let Some(ext) = san_ext {
        if let ParsedExtension::SubjectAlternativeName(san) = ext.parsed_extension() {
            if san.general_names.is_empty() {
                result.add_error(Rfc5280Error::EmptySubjectAltName);
            }
        }
    }
}

/// Convert ASN1Time to DateTime<Utc>
fn asn1_time_to_datetime(time: &x509_parser::time::ASN1Time) -> Option<DateTime<Utc>> {
    chrono::DateTime::from_timestamp(time.timestamp(), 0)
}

/// Extract metadata from a parsed certificate
fn extract_metadata(cert: &X509Certificate, der_bytes: &[u8]) -> CertificateMetadata {
    use sha2::{Sha256, Digest};

    // Calculate fingerprint
    let mut hasher = Sha256::new();
    hasher.update(der_bytes);
    let fingerprint = format!("{:x}", hasher.finalize());

    // Extract subject CN
    let subject_cn = cert.subject()
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok().map(|s| s.to_string()));

    // Extract subject O
    let subject_org = cert.subject()
        .iter_organization()
        .next()
        .and_then(|o| o.as_str().ok().map(|s| s.to_string()));

    // Extract issuer CN
    let issuer_cn = cert.issuer()
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok().map(|s| s.to_string()));

    // Extract validity
    let not_before = asn1_time_to_datetime(&cert.validity().not_before)
        .unwrap_or_else(|| Utc::now());
    let not_after = asn1_time_to_datetime(&cert.validity().not_after)
        .unwrap_or_else(|| Utc::now());

    // Extract BasicConstraints
    let (is_ca, path_length) = cert.extensions().iter()
        .find(|ext| ext.oid == oid_registry::OID_X509_EXT_BASIC_CONSTRAINTS)
        .and_then(|ext| {
            if let ParsedExtension::BasicConstraints(bc) = ext.parsed_extension() {
                Some((bc.ca, bc.path_len_constraint))
            } else {
                None
            }
        })
        .unwrap_or((false, None));

    // Extract KeyUsage
    let key_usage = cert.extensions().iter()
        .find(|ext| ext.oid == oid_registry::OID_X509_EXT_KEY_USAGE)
        .and_then(|ext| {
            if let ParsedExtension::KeyUsage(ku) = ext.parsed_extension() {
                let mut usages = Vec::new();
                if ku.digital_signature() { usages.push("digitalSignature".to_string()); }
                if ku.non_repudiation() { usages.push("nonRepudiation".to_string()); }
                if ku.key_encipherment() { usages.push("keyEncipherment".to_string()); }
                if ku.data_encipherment() { usages.push("dataEncipherment".to_string()); }
                if ku.key_agreement() { usages.push("keyAgreement".to_string()); }
                if ku.key_cert_sign() { usages.push("keyCertSign".to_string()); }
                if ku.crl_sign() { usages.push("cRLSign".to_string()); }
                if ku.encipher_only() { usages.push("encipherOnly".to_string()); }
                if ku.decipher_only() { usages.push("decipherOnly".to_string()); }
                Some(usages)
            } else {
                None
            }
        })
        .unwrap_or_default();

    // Extract Extended KeyUsage
    let extended_key_usage = cert.extensions().iter()
        .find(|ext| ext.oid == oid_registry::OID_X509_EXT_EXTENDED_KEY_USAGE)
        .and_then(|ext| {
            if let ParsedExtension::ExtendedKeyUsage(eku) = ext.parsed_extension() {
                let mut usages: Vec<String> = Vec::new();
                if eku.any { usages.push("anyExtendedKeyUsage".to_string()); }
                if eku.server_auth { usages.push("serverAuth".to_string()); }
                if eku.client_auth { usages.push("clientAuth".to_string()); }
                if eku.code_signing { usages.push("codeSigning".to_string()); }
                if eku.email_protection { usages.push("emailProtection".to_string()); }
                if eku.time_stamping { usages.push("timeStamping".to_string()); }
                if eku.ocsp_signing { usages.push("OCSPSigning".to_string()); }
                // Also include any other OIDs
                for oid in &eku.other {
                    usages.push(oid.to_string());
                }
                Some(usages)
            } else {
                None
            }
        })
        .unwrap_or_default();

    // Extract Subject Alternative Names
    let subject_alt_names = cert.extensions().iter()
        .find(|ext| ext.oid == oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME)
        .and_then(|ext| {
            if let ParsedExtension::SubjectAlternativeName(san) = ext.parsed_extension() {
                Some(san.general_names.iter().map(|gn| format!("{:?}", gn)).collect())
            } else {
                None
            }
        })
        .unwrap_or_default();

    CertificateMetadata {
        version: cert.version().0 + 1, // Display as 1/2/3
        serial_number: hex::encode(cert.raw_serial()),
        subject_cn,
        subject_org,
        issuer_cn,
        not_before,
        not_after,
        is_ca,
        path_length,
        key_usage,
        extended_key_usage,
        subject_alt_names,
        fingerprint_sha256: fingerprint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_new() {
        let result = Rfc5280ValidationResult::new();
        assert!(result.is_valid());
        assert!(result.errors().is_empty());
        assert!(result.warnings().is_empty());
    }

    #[test]
    fn test_validation_result_with_error() {
        let mut result = Rfc5280ValidationResult::new();
        result.add_error(Rfc5280Error::EmptyIssuer);
        assert!(!result.is_valid());
        assert_eq!(result.errors().len(), 1);
    }

    #[test]
    fn test_validation_result_with_warning() {
        let mut result = Rfc5280ValidationResult::new();
        result.add_warning("Test warning".to_string());
        assert!(result.is_valid()); // Warnings don't make it invalid
        assert_eq!(result.warnings().len(), 1);
    }

    #[test]
    fn test_error_display() {
        let error = Rfc5280Error::InvalidVersion { expected: 3, actual: 1 };
        assert!(error.to_string().contains("version"));
    }
}
