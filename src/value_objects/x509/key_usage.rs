// Copyright (c) 2025 - Cowboy AI, LLC.

//! X.509 Key Usage Extensions (RFC 5280)
//!
//! Provides type-safe Key Usage and Extended Key Usage extensions that comply
//! with RFC 5280 Section 4.2.1.3 and Section 4.2.1.12.
//!
//! These value objects replace loose Vec<String> in certificate events with
//! properly typed, validated enumerations.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

// ============================================================================
// Key Usage Extension (RFC 5280 Section 4.2.1.3)
// ============================================================================

/// Individual Key Usage bit flags per RFC 5280
///
/// These are the 9 possible key usage bits in an X.509 certificate.
/// Multiple bits can be set simultaneously.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyUsageBit {
    /// Digital signature (bit 0)
    /// Used for entity authentication and data origin authentication.
    DigitalSignature,

    /// Non-repudiation / Content commitment (bit 1)
    /// Used to verify digital signatures for non-repudiation purposes.
    NonRepudiation,

    /// Key encipherment (bit 2)
    /// Used for key transport (e.g., RSA key exchange in TLS).
    KeyEncipherment,

    /// Data encipherment (bit 3)
    /// Used to encrypt user data (rare).
    DataEncipherment,

    /// Key agreement (bit 4)
    /// Used for key agreement protocols (e.g., ECDH).
    KeyAgreement,

    /// Key cert sign (bit 5)
    /// ONLY set for CA certificates that can sign other certificates.
    KeyCertSign,

    /// CRL sign (bit 6)
    /// Used to verify signatures on CRLs.
    CrlSign,

    /// Encipher only (bit 7)
    /// With keyAgreement, means only encryption during key agreement.
    EncipherOnly,

    /// Decipher only (bit 8)
    /// With keyAgreement, means only decryption during key agreement.
    DecipherOnly,
}

impl KeyUsageBit {
    /// Get the bit position in the Key Usage bit string
    pub fn bit_position(&self) -> u8 {
        match self {
            KeyUsageBit::DigitalSignature => 0,
            KeyUsageBit::NonRepudiation => 1,
            KeyUsageBit::KeyEncipherment => 2,
            KeyUsageBit::DataEncipherment => 3,
            KeyUsageBit::KeyAgreement => 4,
            KeyUsageBit::KeyCertSign => 5,
            KeyUsageBit::CrlSign => 6,
            KeyUsageBit::EncipherOnly => 7,
            KeyUsageBit::DecipherOnly => 8,
        }
    }

    /// Get OID string for this key usage
    pub fn oid_name(&self) -> &'static str {
        match self {
            KeyUsageBit::DigitalSignature => "digitalSignature",
            KeyUsageBit::NonRepudiation => "nonRepudiation",
            KeyUsageBit::KeyEncipherment => "keyEncipherment",
            KeyUsageBit::DataEncipherment => "dataEncipherment",
            KeyUsageBit::KeyAgreement => "keyAgreement",
            KeyUsageBit::KeyCertSign => "keyCertSign",
            KeyUsageBit::CrlSign => "cRLSign",
            KeyUsageBit::EncipherOnly => "encipherOnly",
            KeyUsageBit::DecipherOnly => "decipherOnly",
        }
    }

    /// Parse from string name
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "digitalsignature" | "digital_signature" => Some(KeyUsageBit::DigitalSignature),
            "nonrepudiation" | "non_repudiation" | "contentcommitment" => {
                Some(KeyUsageBit::NonRepudiation)
            }
            "keyencipherment" | "key_encipherment" => Some(KeyUsageBit::KeyEncipherment),
            "dataencipherment" | "data_encipherment" => Some(KeyUsageBit::DataEncipherment),
            "keyagreement" | "key_agreement" => Some(KeyUsageBit::KeyAgreement),
            "keycertsign" | "key_cert_sign" | "certsign" => Some(KeyUsageBit::KeyCertSign),
            "crlsign" | "crl_sign" => Some(KeyUsageBit::CrlSign),
            "encipheronly" | "encipher_only" => Some(KeyUsageBit::EncipherOnly),
            "decipheronly" | "decipher_only" => Some(KeyUsageBit::DecipherOnly),
            _ => None,
        }
    }
}

impl fmt::Display for KeyUsageBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.oid_name())
    }
}

impl DomainConcept for KeyUsageBit {}
impl ValueObject for KeyUsageBit {}

/// Key Usage Extension value object
///
/// Represents the complete Key Usage extension with all set bits.
/// This is a CRITICAL extension per RFC 5280.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyUsage {
    /// Set of key usage bits
    bits: HashSet<KeyUsageBit>,
    /// Whether this extension is critical (should always be true per RFC 5280)
    pub critical: bool,
}

impl KeyUsage {
    /// Create a new empty Key Usage
    pub fn new() -> Self {
        Self {
            bits: HashSet::new(),
            critical: true, // RFC 5280 recommends critical=true
        }
    }

    /// Create Key Usage from a set of bits
    pub fn from_bits(bits: impl IntoIterator<Item = KeyUsageBit>) -> Self {
        Self {
            bits: bits.into_iter().collect(),
            critical: true,
        }
    }

    /// Builder pattern - add a usage bit
    pub fn with_bit(mut self, bit: KeyUsageBit) -> Self {
        self.bits.insert(bit);
        self
    }

    /// Builder pattern - set criticality
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }

    /// Check if a specific bit is set
    pub fn has(&self, bit: KeyUsageBit) -> bool {
        self.bits.contains(&bit)
    }

    /// Get all set bits
    pub fn bits(&self) -> impl Iterator<Item = &KeyUsageBit> {
        self.bits.iter()
    }

    /// Check if this is a CA key usage (has keyCertSign)
    pub fn is_ca(&self) -> bool {
        self.has(KeyUsageBit::KeyCertSign)
    }

    /// Check if this can sign data (digitalSignature or nonRepudiation)
    pub fn can_sign(&self) -> bool {
        self.has(KeyUsageBit::DigitalSignature) || self.has(KeyUsageBit::NonRepudiation)
    }

    /// Check if this supports key exchange
    pub fn supports_key_exchange(&self) -> bool {
        self.has(KeyUsageBit::KeyEncipherment) || self.has(KeyUsageBit::KeyAgreement)
    }

    /// Convert to string list for event serialization
    pub fn to_string_list(&self) -> Vec<String> {
        self.bits.iter().map(|b| b.oid_name().to_string()).collect()
    }

    /// Parse from string list (for backward compatibility with existing events)
    pub fn from_string_list(strings: &[String]) -> Self {
        let bits = strings
            .iter()
            .filter_map(|s| KeyUsageBit::from_str(s))
            .collect();
        Self { bits, critical: true }
    }

    /// Create standard key usage for a CA certificate
    pub fn ca_certificate() -> Self {
        Self::from_bits([
            KeyUsageBit::KeyCertSign,
            KeyUsageBit::CrlSign,
            KeyUsageBit::DigitalSignature,
        ])
    }

    /// Create standard key usage for a TLS server certificate
    pub fn tls_server() -> Self {
        Self::from_bits([
            KeyUsageBit::DigitalSignature,
            KeyUsageBit::KeyEncipherment,
        ])
    }

    /// Create standard key usage for a TLS client certificate
    pub fn tls_client() -> Self {
        Self::from_bits([KeyUsageBit::DigitalSignature])
    }

    /// Create standard key usage for code signing
    pub fn code_signing() -> Self {
        Self::from_bits([KeyUsageBit::DigitalSignature])
    }

    /// Create standard key usage for email protection (S/MIME)
    pub fn email_protection() -> Self {
        Self::from_bits([
            KeyUsageBit::DigitalSignature,
            KeyUsageBit::NonRepudiation,
            KeyUsageBit::KeyEncipherment,
        ])
    }
}

impl Default for KeyUsage {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for KeyUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bits: Vec<String> = self.bits.iter().map(|b| b.to_string()).collect();
        write!(f, "KeyUsage({})", bits.join(", "))
    }
}

impl DomainConcept for KeyUsage {}
impl ValueObject for KeyUsage {}

// ============================================================================
// Extended Key Usage Extension (RFC 5280 Section 4.2.1.12)
// ============================================================================

/// Extended Key Usage purposes per RFC 5280
///
/// These OIDs specify the intended use of the public key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtendedKeyUsagePurpose {
    /// TLS Web Server Authentication (1.3.6.1.5.5.7.3.1)
    ServerAuth,

    /// TLS Web Client Authentication (1.3.6.1.5.5.7.3.2)
    ClientAuth,

    /// Code Signing (1.3.6.1.5.5.7.3.3)
    CodeSigning,

    /// Email Protection / S/MIME (1.3.6.1.5.5.7.3.4)
    EmailProtection,

    /// Time Stamping (1.3.6.1.5.5.7.3.8)
    TimeStamping,

    /// OCSP Signing (1.3.6.1.5.5.7.3.9)
    OcspSigning,

    /// Any Extended Key Usage (2.5.29.37.0)
    AnyExtendedKeyUsage,

    /// Smart Card Logon (Microsoft) (1.3.6.1.4.1.311.20.2.2)
    SmartCardLogon,

    /// Custom OID
    Custom(String),
}

impl ExtendedKeyUsagePurpose {
    /// Get the OID string for this purpose
    pub fn oid(&self) -> &str {
        match self {
            ExtendedKeyUsagePurpose::ServerAuth => "1.3.6.1.5.5.7.3.1",
            ExtendedKeyUsagePurpose::ClientAuth => "1.3.6.1.5.5.7.3.2",
            ExtendedKeyUsagePurpose::CodeSigning => "1.3.6.1.5.5.7.3.3",
            ExtendedKeyUsagePurpose::EmailProtection => "1.3.6.1.5.5.7.3.4",
            ExtendedKeyUsagePurpose::TimeStamping => "1.3.6.1.5.5.7.3.8",
            ExtendedKeyUsagePurpose::OcspSigning => "1.3.6.1.5.5.7.3.9",
            ExtendedKeyUsagePurpose::AnyExtendedKeyUsage => "2.5.29.37.0",
            ExtendedKeyUsagePurpose::SmartCardLogon => "1.3.6.1.4.1.311.20.2.2",
            ExtendedKeyUsagePurpose::Custom(oid) => oid,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            ExtendedKeyUsagePurpose::ServerAuth => "serverAuth",
            ExtendedKeyUsagePurpose::ClientAuth => "clientAuth",
            ExtendedKeyUsagePurpose::CodeSigning => "codeSigning",
            ExtendedKeyUsagePurpose::EmailProtection => "emailProtection",
            ExtendedKeyUsagePurpose::TimeStamping => "timeStamping",
            ExtendedKeyUsagePurpose::OcspSigning => "OCSPSigning",
            ExtendedKeyUsagePurpose::AnyExtendedKeyUsage => "anyExtendedKeyUsage",
            ExtendedKeyUsagePurpose::SmartCardLogon => "smartCardLogon",
            ExtendedKeyUsagePurpose::Custom(_) => "custom",
        }
    }

    /// Parse from OID string
    pub fn from_oid(oid: &str) -> Self {
        match oid {
            "1.3.6.1.5.5.7.3.1" => ExtendedKeyUsagePurpose::ServerAuth,
            "1.3.6.1.5.5.7.3.2" => ExtendedKeyUsagePurpose::ClientAuth,
            "1.3.6.1.5.5.7.3.3" => ExtendedKeyUsagePurpose::CodeSigning,
            "1.3.6.1.5.5.7.3.4" => ExtendedKeyUsagePurpose::EmailProtection,
            "1.3.6.1.5.5.7.3.8" => ExtendedKeyUsagePurpose::TimeStamping,
            "1.3.6.1.5.5.7.3.9" => ExtendedKeyUsagePurpose::OcspSigning,
            "2.5.29.37.0" => ExtendedKeyUsagePurpose::AnyExtendedKeyUsage,
            "1.3.6.1.4.1.311.20.2.2" => ExtendedKeyUsagePurpose::SmartCardLogon,
            other => ExtendedKeyUsagePurpose::Custom(other.to_string()),
        }
    }

    /// Parse from name string
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "serverauth" | "server_auth" | "tlswebserverauthentication" => {
                Some(ExtendedKeyUsagePurpose::ServerAuth)
            }
            "clientauth" | "client_auth" | "tlswebclientauthentication" => {
                Some(ExtendedKeyUsagePurpose::ClientAuth)
            }
            "codesigning" | "code_signing" => Some(ExtendedKeyUsagePurpose::CodeSigning),
            "emailprotection" | "email_protection" | "smime" => {
                Some(ExtendedKeyUsagePurpose::EmailProtection)
            }
            "timestamping" | "time_stamping" => Some(ExtendedKeyUsagePurpose::TimeStamping),
            "ocspsigning" | "ocsp_signing" => Some(ExtendedKeyUsagePurpose::OcspSigning),
            "anyextendedkeyusage" | "any" => Some(ExtendedKeyUsagePurpose::AnyExtendedKeyUsage),
            "smartcardlogon" | "smart_card_logon" => Some(ExtendedKeyUsagePurpose::SmartCardLogon),
            _ => None,
        }
    }
}

impl fmt::Display for ExtendedKeyUsagePurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl DomainConcept for ExtendedKeyUsagePurpose {}
impl ValueObject for ExtendedKeyUsagePurpose {}

/// Extended Key Usage Extension value object
///
/// Specifies the purposes for which the public key may be used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtendedKeyUsage {
    /// Set of extended key usage purposes
    purposes: HashSet<ExtendedKeyUsagePurpose>,
    /// Whether this extension is critical
    pub critical: bool,
}

impl ExtendedKeyUsage {
    /// Create a new empty Extended Key Usage
    pub fn new() -> Self {
        Self {
            purposes: HashSet::new(),
            critical: false, // RFC 5280 recommends critical=false
        }
    }

    /// Create from a set of purposes
    pub fn from_purposes(purposes: impl IntoIterator<Item = ExtendedKeyUsagePurpose>) -> Self {
        Self {
            purposes: purposes.into_iter().collect(),
            critical: false,
        }
    }

    /// Builder pattern - add a purpose
    pub fn with_purpose(mut self, purpose: ExtendedKeyUsagePurpose) -> Self {
        self.purposes.insert(purpose);
        self
    }

    /// Builder pattern - set criticality
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }

    /// Check if a specific purpose is set
    pub fn has(&self, purpose: &ExtendedKeyUsagePurpose) -> bool {
        self.purposes.contains(purpose)
    }

    /// Get all purposes
    pub fn purposes(&self) -> impl Iterator<Item = &ExtendedKeyUsagePurpose> {
        self.purposes.iter()
    }

    /// Check if this includes server auth
    pub fn allows_server_auth(&self) -> bool {
        self.has(&ExtendedKeyUsagePurpose::ServerAuth)
            || self.has(&ExtendedKeyUsagePurpose::AnyExtendedKeyUsage)
    }

    /// Check if this includes client auth
    pub fn allows_client_auth(&self) -> bool {
        self.has(&ExtendedKeyUsagePurpose::ClientAuth)
            || self.has(&ExtendedKeyUsagePurpose::AnyExtendedKeyUsage)
    }

    /// Convert to string list for event serialization
    pub fn to_string_list(&self) -> Vec<String> {
        self.purposes.iter().map(|p| p.name().to_string()).collect()
    }

    /// Parse from string list (for backward compatibility)
    pub fn from_string_list(strings: &[String]) -> Self {
        let purposes = strings
            .iter()
            .filter_map(|s| ExtendedKeyUsagePurpose::from_name(s))
            .collect();
        Self {
            purposes,
            critical: false,
        }
    }

    /// Create standard EKU for a TLS server certificate
    pub fn tls_server() -> Self {
        Self::from_purposes([ExtendedKeyUsagePurpose::ServerAuth])
    }

    /// Create standard EKU for a TLS client certificate
    pub fn tls_client() -> Self {
        Self::from_purposes([ExtendedKeyUsagePurpose::ClientAuth])
    }

    /// Create standard EKU for a TLS server+client certificate
    pub fn tls_server_client() -> Self {
        Self::from_purposes([
            ExtendedKeyUsagePurpose::ServerAuth,
            ExtendedKeyUsagePurpose::ClientAuth,
        ])
    }

    /// Create standard EKU for code signing
    pub fn code_signing() -> Self {
        Self::from_purposes([ExtendedKeyUsagePurpose::CodeSigning])
    }

    /// Create standard EKU for email protection
    pub fn email_protection() -> Self {
        Self::from_purposes([ExtendedKeyUsagePurpose::EmailProtection])
    }
}

impl Default for ExtendedKeyUsage {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ExtendedKeyUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let purposes: Vec<String> = self.purposes.iter().map(|p| p.to_string()).collect();
        write!(f, "ExtendedKeyUsage({})", purposes.join(", "))
    }
}

impl DomainConcept for ExtendedKeyUsage {}
impl ValueObject for ExtendedKeyUsage {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_usage_bit_positions() {
        assert_eq!(KeyUsageBit::DigitalSignature.bit_position(), 0);
        assert_eq!(KeyUsageBit::KeyCertSign.bit_position(), 5);
        assert_eq!(KeyUsageBit::DecipherOnly.bit_position(), 8);
    }

    #[test]
    fn test_key_usage_bit_from_str() {
        assert_eq!(
            KeyUsageBit::from_str("digitalSignature"),
            Some(KeyUsageBit::DigitalSignature)
        );
        assert_eq!(
            KeyUsageBit::from_str("keyCertSign"),
            Some(KeyUsageBit::KeyCertSign)
        );
        assert!(KeyUsageBit::from_str("unknown").is_none());
    }

    #[test]
    fn test_key_usage_ca() {
        let ku = KeyUsage::ca_certificate();
        assert!(ku.is_ca());
        assert!(ku.has(KeyUsageBit::KeyCertSign));
        assert!(ku.has(KeyUsageBit::CrlSign));
        assert!(ku.has(KeyUsageBit::DigitalSignature));
    }

    #[test]
    fn test_key_usage_tls_server() {
        let ku = KeyUsage::tls_server();
        assert!(!ku.is_ca());
        assert!(ku.has(KeyUsageBit::DigitalSignature));
        assert!(ku.has(KeyUsageBit::KeyEncipherment));
        assert!(ku.supports_key_exchange());
    }

    #[test]
    fn test_key_usage_to_string_list() {
        let ku = KeyUsage::from_bits([KeyUsageBit::DigitalSignature, KeyUsageBit::KeyCertSign]);
        let strings = ku.to_string_list();
        assert!(strings.contains(&"digitalSignature".to_string()));
        assert!(strings.contains(&"keyCertSign".to_string()));
    }

    #[test]
    fn test_key_usage_from_string_list() {
        let strings = vec!["digitalSignature".to_string(), "keyEncipherment".to_string()];
        let ku = KeyUsage::from_string_list(&strings);
        assert!(ku.has(KeyUsageBit::DigitalSignature));
        assert!(ku.has(KeyUsageBit::KeyEncipherment));
    }

    #[test]
    fn test_eku_purpose_oids() {
        assert_eq!(ExtendedKeyUsagePurpose::ServerAuth.oid(), "1.3.6.1.5.5.7.3.1");
        assert_eq!(ExtendedKeyUsagePurpose::ClientAuth.oid(), "1.3.6.1.5.5.7.3.2");
        assert_eq!(ExtendedKeyUsagePurpose::CodeSigning.oid(), "1.3.6.1.5.5.7.3.3");
    }

    #[test]
    fn test_eku_from_oid() {
        assert_eq!(
            ExtendedKeyUsagePurpose::from_oid("1.3.6.1.5.5.7.3.1"),
            ExtendedKeyUsagePurpose::ServerAuth
        );
        assert!(matches!(
            ExtendedKeyUsagePurpose::from_oid("1.2.3.4.5"),
            ExtendedKeyUsagePurpose::Custom(_)
        ));
    }

    #[test]
    fn test_eku_from_name() {
        assert_eq!(
            ExtendedKeyUsagePurpose::from_name("serverAuth"),
            Some(ExtendedKeyUsagePurpose::ServerAuth)
        );
        assert_eq!(
            ExtendedKeyUsagePurpose::from_name("codeSigning"),
            Some(ExtendedKeyUsagePurpose::CodeSigning)
        );
        assert!(ExtendedKeyUsagePurpose::from_name("unknown").is_none());
    }

    #[test]
    fn test_eku_tls_server() {
        let eku = ExtendedKeyUsage::tls_server();
        assert!(eku.allows_server_auth());
        assert!(!eku.allows_client_auth());
    }

    #[test]
    fn test_eku_tls_server_client() {
        let eku = ExtendedKeyUsage::tls_server_client();
        assert!(eku.allows_server_auth());
        assert!(eku.allows_client_auth());
    }

    #[test]
    fn test_eku_to_string_list() {
        let eku = ExtendedKeyUsage::from_purposes([
            ExtendedKeyUsagePurpose::ServerAuth,
            ExtendedKeyUsagePurpose::ClientAuth,
        ]);
        let strings = eku.to_string_list();
        assert!(strings.contains(&"serverAuth".to_string()));
        assert!(strings.contains(&"clientAuth".to_string()));
    }

    #[test]
    fn test_eku_from_string_list() {
        let strings = vec!["serverAuth".to_string(), "codeSigning".to_string()];
        let eku = ExtendedKeyUsage::from_string_list(&strings);
        assert!(eku.has(&ExtendedKeyUsagePurpose::ServerAuth));
        assert!(eku.has(&ExtendedKeyUsagePurpose::CodeSigning));
    }
}
