// Copyright (c) 2025 - Cowboy AI, LLC.

//! Value Objects for Domain-Driven Design Compliance
//!
//! Value objects are immutable domain primitives that encapsulate validation
//! rules and business invariants at construction time. This ensures that
//! invalid states are not representable in the type system.
//!
//! ## Design Principles
//!
//! 1. **Immutability** - All value objects are immutable after construction
//! 2. **Validation at Construction** - Invalid values cannot be created
//! 3. **Domain Language** - Named according to ubiquitous language
//! 4. **Equality by Value** - Two value objects are equal if their contents are equal
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cim_keys::domain::value_objects::OperatorName;
//!
//! let name = OperatorName::new("CowboyAI")?;
//! println!("Operator: {}", name.as_str());
//! ```

use std::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// DOMAIN ERRORS
// ============================================================================

/// Errors that can occur when creating value objects
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ValueObjectError {
    #[error("operator name cannot be empty")]
    EmptyOperatorName,

    #[error("operator name '{0}' exceeds maximum length of {1} characters")]
    OperatorNameTooLong(String, usize),

    #[error("operator name '{0}' contains invalid characters")]
    InvalidOperatorNameChars(String),

    #[error("account name cannot be empty")]
    EmptyAccountName,

    #[error("account name '{0}' exceeds maximum length of {1} characters")]
    AccountNameTooLong(String, usize),

    #[error("account name '{0}' contains invalid characters")]
    InvalidAccountNameChars(String),

    #[error("user name cannot be empty")]
    EmptyUserName,

    #[error("user name '{0}' exceeds maximum length of {1} characters")]
    UserNameTooLong(String, usize),

    #[error("certificate subject cannot be empty")]
    EmptyCertificateSubject,

    #[error("certificate subject '{0}' is malformed")]
    MalformedCertificateSubject(String),

    #[error("key purpose cannot be empty")]
    EmptyKeyPurpose,

    #[error("fingerprint cannot be empty")]
    EmptyFingerprint,

    #[error("fingerprint '{0}' is malformed (expected hex string)")]
    MalformedFingerprint(String),
}

// ============================================================================
// NATS OPERATOR NAME
// ============================================================================

/// A validated NATS Operator name.
///
/// ## Invariants
///
/// - Not empty
/// - Maximum 256 characters
/// - Only alphanumeric, hyphen, underscore allowed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct OperatorName(String);

impl OperatorName {
    /// Maximum allowed length for operator names
    pub const MAX_LENGTH: usize = 256;

    /// Create a new validated operator name.
    pub fn new(name: impl Into<String>) -> Result<Self, ValueObjectError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ValueObjectError::EmptyOperatorName);
        }

        if name.len() > Self::MAX_LENGTH {
            return Err(ValueObjectError::OperatorNameTooLong(name, Self::MAX_LENGTH));
        }

        // NATS operator names: alphanumeric, hyphen, underscore
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValueObjectError::InvalidOperatorNameChars(name));
        }

        Ok(Self(name))
    }

    /// Get the operator name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OperatorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for OperatorName {
    type Error = ValueObjectError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<OperatorName> for String {
    fn from(name: OperatorName) -> String {
        name.0
    }
}

// ============================================================================
// NATS ACCOUNT NAME
// ============================================================================

/// A validated NATS Account name.
///
/// ## Invariants
///
/// - Not empty
/// - Maximum 256 characters
/// - Only alphanumeric, hyphen, underscore allowed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AccountName(String);

impl AccountName {
    /// Maximum allowed length for account names
    pub const MAX_LENGTH: usize = 256;

    /// Create a new validated account name.
    pub fn new(name: impl Into<String>) -> Result<Self, ValueObjectError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ValueObjectError::EmptyAccountName);
        }

        if name.len() > Self::MAX_LENGTH {
            return Err(ValueObjectError::AccountNameTooLong(name, Self::MAX_LENGTH));
        }

        // NATS account names: alphanumeric, hyphen, underscore
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValueObjectError::InvalidAccountNameChars(name));
        }

        Ok(Self(name))
    }

    /// Get the account name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AccountName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for AccountName {
    type Error = ValueObjectError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<AccountName> for String {
    fn from(name: AccountName) -> String {
        name.0
    }
}

// ============================================================================
// NATS USER NAME
// ============================================================================

/// A validated NATS User name.
///
/// ## Invariants
///
/// - Not empty
/// - Maximum 256 characters
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct UserName(String);

impl UserName {
    /// Maximum allowed length for user names
    pub const MAX_LENGTH: usize = 256;

    /// Create a new validated user name.
    pub fn new(name: impl Into<String>) -> Result<Self, ValueObjectError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ValueObjectError::EmptyUserName);
        }

        if name.len() > Self::MAX_LENGTH {
            return Err(ValueObjectError::UserNameTooLong(name, Self::MAX_LENGTH));
        }

        Ok(Self(name))
    }

    /// Get the user name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for UserName {
    type Error = ValueObjectError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<UserName> for String {
    fn from(name: UserName) -> String {
        name.0
    }
}

// ============================================================================
// CERTIFICATE SUBJECT
// ============================================================================

/// A validated X.509 Certificate Subject (Distinguished Name).
///
/// ## Invariants
///
/// - Not empty
/// - Contains at least a Common Name (CN) component
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CertificateSubject(String);

impl CertificateSubject {
    /// Create a new validated certificate subject.
    pub fn new(subject: impl Into<String>) -> Result<Self, ValueObjectError> {
        let subject = subject.into();

        if subject.is_empty() {
            return Err(ValueObjectError::EmptyCertificateSubject);
        }

        // Basic validation: should contain CN= or be a simple common name
        // In production, this would parse the full DN syntax
        if !subject.contains("CN=") && !subject.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.') {
            return Err(ValueObjectError::MalformedCertificateSubject(subject));
        }

        Ok(Self(subject))
    }

    /// Create a certificate subject from just a common name.
    pub fn from_common_name(cn: impl Into<String>) -> Result<Self, ValueObjectError> {
        let cn = cn.into();
        if cn.is_empty() {
            return Err(ValueObjectError::EmptyCertificateSubject);
        }
        Self::new(format!("CN={}", cn))
    }

    /// Get the certificate subject as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract the Common Name from the subject.
    pub fn common_name(&self) -> Option<&str> {
        if let Some(start) = self.0.find("CN=") {
            let rest = &self.0[start + 3..];
            if let Some(end) = rest.find(',') {
                Some(&rest[..end])
            } else {
                Some(rest)
            }
        } else if !self.0.contains('=') {
            // Simple name without DN format
            Some(&self.0)
        } else {
            None
        }
    }
}

impl fmt::Display for CertificateSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for CertificateSubject {
    type Error = ValueObjectError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<CertificateSubject> for String {
    fn from(subject: CertificateSubject) -> String {
        subject.0
    }
}

// ============================================================================
// KEY PURPOSE
// ============================================================================

/// A validated key purpose description.
///
/// ## Invariants
///
/// - Not empty
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct KeyPurpose(String);

impl KeyPurpose {
    /// Create a new validated key purpose.
    pub fn new(purpose: impl Into<String>) -> Result<Self, ValueObjectError> {
        let purpose = purpose.into();

        if purpose.is_empty() {
            return Err(ValueObjectError::EmptyKeyPurpose);
        }

        Ok(Self(purpose))
    }

    /// Get the key purpose as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Standard key purposes
    pub fn authentication() -> Self {
        Self("Authentication".to_string())
    }

    pub fn signing() -> Self {
        Self("Digital Signature".to_string())
    }

    pub fn encryption() -> Self {
        Self("Key Management".to_string())
    }

    pub fn card_authentication() -> Self {
        Self("Card Authentication".to_string())
    }
}

impl fmt::Display for KeyPurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for KeyPurpose {
    type Error = ValueObjectError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<KeyPurpose> for String {
    fn from(purpose: KeyPurpose) -> String {
        purpose.0
    }
}

// ============================================================================
// FINGERPRINT
// ============================================================================

/// A validated cryptographic fingerprint (hash).
///
/// ## Invariants
///
/// - Not empty
/// - Valid hexadecimal string (optionally with colons)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Fingerprint(String);

impl Fingerprint {
    /// Create a new validated fingerprint.
    pub fn new(fingerprint: impl Into<String>) -> Result<Self, ValueObjectError> {
        let fingerprint = fingerprint.into();

        if fingerprint.is_empty() {
            return Err(ValueObjectError::EmptyFingerprint);
        }

        // Remove colons for validation
        let hex_only: String = fingerprint.chars().filter(|c| *c != ':').collect();

        // Should be valid hexadecimal
        if !hex_only.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValueObjectError::MalformedFingerprint(fingerprint));
        }

        Ok(Self(fingerprint))
    }

    /// Get the fingerprint as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the fingerprint as raw hex bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let hex_only: String = self.0.chars().filter(|c| *c != ':').collect();
        hex::decode(&hex_only).unwrap_or_default()
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Fingerprint {
    type Error = ValueObjectError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Fingerprint> for String {
    fn from(fp: Fingerprint) -> String {
        fp.0
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod operator_name_tests {
        use super::*;

        #[test]
        fn test_valid_operator_name() {
            let name = OperatorName::new("CowboyAI").unwrap();
            assert_eq!(name.as_str(), "CowboyAI");
        }

        #[test]
        fn test_operator_name_with_hyphen() {
            let name = OperatorName::new("cowboy-ai").unwrap();
            assert_eq!(name.as_str(), "cowboy-ai");
        }

        #[test]
        fn test_operator_name_with_underscore() {
            let name = OperatorName::new("cowboy_ai").unwrap();
            assert_eq!(name.as_str(), "cowboy_ai");
        }

        #[test]
        fn test_empty_operator_name_fails() {
            let result = OperatorName::new("");
            assert!(matches!(result, Err(ValueObjectError::EmptyOperatorName)));
        }

        #[test]
        fn test_operator_name_with_invalid_chars() {
            let result = OperatorName::new("cowboy@ai");
            assert!(matches!(result, Err(ValueObjectError::InvalidOperatorNameChars(_))));
        }

        #[test]
        fn test_operator_name_too_long() {
            let long_name = "a".repeat(300);
            let result = OperatorName::new(long_name);
            assert!(matches!(result, Err(ValueObjectError::OperatorNameTooLong(_, 256))));
        }

        #[test]
        fn test_operator_name_serde() {
            let name = OperatorName::new("TestOperator").unwrap();
            let json = serde_json::to_string(&name).unwrap();
            assert_eq!(json, "\"TestOperator\"");

            let deserialized: OperatorName = serde_json::from_str(&json).unwrap();
            assert_eq!(name, deserialized);
        }
    }

    mod account_name_tests {
        use super::*;

        #[test]
        fn test_valid_account_name() {
            let name = AccountName::new("infrastructure").unwrap();
            assert_eq!(name.as_str(), "infrastructure");
        }

        #[test]
        fn test_empty_account_name_fails() {
            let result = AccountName::new("");
            assert!(matches!(result, Err(ValueObjectError::EmptyAccountName)));
        }
    }

    mod certificate_subject_tests {
        use super::*;

        #[test]
        fn test_dn_format() {
            let subject = CertificateSubject::new("CN=Root CA,O=CowboyAI").unwrap();
            assert_eq!(subject.common_name(), Some("Root CA"));
        }

        #[test]
        fn test_from_common_name() {
            let subject = CertificateSubject::from_common_name("My Root CA").unwrap();
            assert_eq!(subject.as_str(), "CN=My Root CA");
            assert_eq!(subject.common_name(), Some("My Root CA"));
        }

        #[test]
        fn test_simple_name() {
            let subject = CertificateSubject::new("My-Server.example.com").unwrap();
            assert_eq!(subject.common_name(), Some("My-Server.example.com"));
        }
    }

    mod fingerprint_tests {
        use super::*;

        #[test]
        fn test_hex_fingerprint() {
            let fp = Fingerprint::new("deadbeef").unwrap();
            assert_eq!(fp.to_bytes(), vec![0xde, 0xad, 0xbe, 0xef]);
        }

        #[test]
        fn test_colon_separated_fingerprint() {
            let fp = Fingerprint::new("de:ad:be:ef").unwrap();
            assert_eq!(fp.to_bytes(), vec![0xde, 0xad, 0xbe, 0xef]);
        }

        #[test]
        fn test_invalid_fingerprint() {
            let result = Fingerprint::new("not-hex");
            assert!(matches!(result, Err(ValueObjectError::MalformedFingerprint(_))));
        }
    }

    mod key_purpose_tests {
        use super::*;

        #[test]
        fn test_standard_purposes() {
            assert_eq!(KeyPurpose::authentication().as_str(), "Authentication");
            assert_eq!(KeyPurpose::signing().as_str(), "Digital Signature");
            assert_eq!(KeyPurpose::encryption().as_str(), "Key Management");
        }

        #[test]
        fn test_custom_purpose() {
            let purpose = KeyPurpose::new("Code Signing").unwrap();
            assert_eq!(purpose.as_str(), "Code Signing");
        }
    }
}
