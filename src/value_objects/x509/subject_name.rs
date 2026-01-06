// Copyright (c) 2025 - Cowboy AI, LLC.

//! X.509 Subject/Issuer Name Value Objects (RFC 5280)
//!
//! Provides type-safe Distinguished Name (DN) components that comply with
//! RFC 5280 Section 4.1.2.4 and RFC 4514 for string representation.
//!
//! These value objects replace loose strings in certificate events with
//! properly typed, validated components.

use serde::{Deserialize, Serialize};
use std::fmt;

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

// ============================================================================
// Distinguished Name Components (RFC 5280)
// ============================================================================

/// Common Name (CN) - RFC 5280 Section 4.1.2.4
///
/// The CN attribute identifies the entity (person, server, etc.).
/// Example: "Alice Smith" or "www.example.com"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommonName(String);

impl CommonName {
    /// Create a new CommonName
    ///
    /// # Validation
    /// - Must not be empty
    /// - Must not exceed 64 characters (RFC 5280)
    pub fn new(name: impl Into<String>) -> Result<Self, SubjectNameError> {
        let name = name.into();
        if name.is_empty() {
            return Err(SubjectNameError::EmptyCommonName);
        }
        if name.len() > 64 {
            return Err(SubjectNameError::CommonNameTooLong(name.len()));
        }
        Ok(Self(name))
    }

    /// Create a CommonName without validation (for deserialization)
    pub fn new_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for CommonName {}
impl ValueObject for CommonName {}

/// Organization Name (O) - RFC 5280
///
/// The organization name of the entity.
/// Example: "Cowboy AI, LLC"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationName(String);

impl OrganizationName {
    /// Create a new OrganizationName
    ///
    /// # Validation
    /// - Must not be empty
    /// - Must not exceed 64 characters
    pub fn new(name: impl Into<String>) -> Result<Self, SubjectNameError> {
        let name = name.into();
        if name.is_empty() {
            return Err(SubjectNameError::EmptyOrganizationName);
        }
        if name.len() > 64 {
            return Err(SubjectNameError::OrganizationNameTooLong(name.len()));
        }
        Ok(Self(name))
    }

    pub fn new_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OrganizationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for OrganizationName {}
impl ValueObject for OrganizationName {}

/// Organizational Unit Name (OU) - RFC 5280
///
/// A subdivision within the organization.
/// Example: "Engineering" or "Security"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationalUnitName(String);

impl OrganizationalUnitName {
    pub fn new(name: impl Into<String>) -> Result<Self, SubjectNameError> {
        let name = name.into();
        if name.is_empty() {
            return Err(SubjectNameError::EmptyOrganizationalUnitName);
        }
        if name.len() > 64 {
            return Err(SubjectNameError::OrganizationalUnitNameTooLong(name.len()));
        }
        Ok(Self(name))
    }

    pub fn new_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OrganizationalUnitName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for OrganizationalUnitName {}
impl ValueObject for OrganizationalUnitName {}

/// Country Code (C) - RFC 5280 / ISO 3166-1 alpha-2
///
/// Two-letter country code.
/// Example: "US", "GB", "DE"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CountryCode(String);

impl CountryCode {
    /// Create a new CountryCode
    ///
    /// # Validation
    /// - Must be exactly 2 characters
    /// - Must be uppercase ASCII letters
    pub fn new(code: impl Into<String>) -> Result<Self, SubjectNameError> {
        let code = code.into();
        if code.len() != 2 {
            return Err(SubjectNameError::InvalidCountryCodeLength(code.len()));
        }
        if !code.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(SubjectNameError::InvalidCountryCodeFormat(code));
        }
        Ok(Self(code))
    }

    pub fn new_unchecked(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for CountryCode {}
impl ValueObject for CountryCode {}

/// State or Province Name (ST) - RFC 5280
///
/// Example: "Texas", "California"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateName(String);

impl StateName {
    pub fn new(name: impl Into<String>) -> Result<Self, SubjectNameError> {
        let name = name.into();
        if name.is_empty() {
            return Err(SubjectNameError::EmptyStateName);
        }
        if name.len() > 64 {
            return Err(SubjectNameError::StateNameTooLong(name.len()));
        }
        Ok(Self(name))
    }

    pub fn new_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StateName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for StateName {}
impl ValueObject for StateName {}

/// Locality Name (L) - RFC 5280
///
/// City or town name.
/// Example: "Austin", "San Francisco"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalityName(String);

impl LocalityName {
    pub fn new(name: impl Into<String>) -> Result<Self, SubjectNameError> {
        let name = name.into();
        if name.is_empty() {
            return Err(SubjectNameError::EmptyLocalityName);
        }
        if name.len() > 64 {
            return Err(SubjectNameError::LocalityNameTooLong(name.len()));
        }
        Ok(Self(name))
    }

    pub fn new_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LocalityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for LocalityName {}
impl ValueObject for LocalityName {}

/// Email Address - RFC 5280 (emailAddress OID 1.2.840.113549.1.9.1)
///
/// Email address in the Subject DN.
/// Note: RFC 5280 recommends using Subject Alternative Name for email instead.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Create a new EmailAddress
    ///
    /// # Validation
    /// - Must contain exactly one @ symbol
    /// - Must have non-empty local and domain parts
    pub fn new(email: impl Into<String>) -> Result<Self, SubjectNameError> {
        let email = email.into();
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(SubjectNameError::InvalidEmailFormat(email));
        }
        if parts[0].is_empty() || parts[1].is_empty() {
            return Err(SubjectNameError::InvalidEmailFormat(email));
        }
        if email.len() > 255 {
            return Err(SubjectNameError::EmailTooLong(email.len()));
        }
        Ok(Self(email))
    }

    pub fn new_unchecked(email: impl Into<String>) -> Self {
        Self(email.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }

    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for EmailAddress {}
impl ValueObject for EmailAddress {}

// ============================================================================
// Complete Subject/Issuer Name (Distinguished Name)
// ============================================================================

/// Complete X.509 Subject or Issuer Name (Distinguished Name)
///
/// This is the full DN composed of the individual components.
/// Follows RFC 5280 Section 4.1.2.4 and RFC 4514 for string encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectName {
    /// Common Name (CN) - REQUIRED
    pub common_name: CommonName,
    /// Organization (O) - Optional
    pub organization: Option<OrganizationName>,
    /// Organizational Unit (OU) - Optional
    pub organizational_unit: Option<OrganizationalUnitName>,
    /// Country (C) - Optional
    pub country: Option<CountryCode>,
    /// State or Province (ST) - Optional
    pub state: Option<StateName>,
    /// Locality (L) - Optional
    pub locality: Option<LocalityName>,
    /// Email Address - Optional (deprecated, use SAN instead)
    pub email: Option<EmailAddress>,
}

impl SubjectName {
    /// Create a new SubjectName with only CommonName
    pub fn new(common_name: CommonName) -> Self {
        Self {
            common_name,
            organization: None,
            organizational_unit: None,
            country: None,
            state: None,
            locality: None,
            email: None,
        }
    }

    /// Builder pattern - add organization
    pub fn with_organization(mut self, org: OrganizationName) -> Self {
        self.organization = Some(org);
        self
    }

    /// Builder pattern - add organizational unit
    pub fn with_organizational_unit(mut self, ou: OrganizationalUnitName) -> Self {
        self.organizational_unit = Some(ou);
        self
    }

    /// Builder pattern - add country
    pub fn with_country(mut self, country: CountryCode) -> Self {
        self.country = Some(country);
        self
    }

    /// Builder pattern - add state
    pub fn with_state(mut self, state: StateName) -> Self {
        self.state = Some(state);
        self
    }

    /// Builder pattern - add locality
    pub fn with_locality(mut self, locality: LocalityName) -> Self {
        self.locality = Some(locality);
        self
    }

    /// Builder pattern - add email
    pub fn with_email(mut self, email: EmailAddress) -> Self {
        self.email = Some(email);
        self
    }

    /// Convert to RFC 4514 Distinguished Name string
    ///
    /// Example: "CN=Alice Smith,O=Cowboy AI\\, LLC,C=US"
    pub fn to_rfc4514(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("CN={}", escape_rfc4514(self.common_name.as_str())));

        if let Some(ref org) = self.organization {
            parts.push(format!("O={}", escape_rfc4514(org.as_str())));
        }
        if let Some(ref ou) = self.organizational_unit {
            parts.push(format!("OU={}", escape_rfc4514(ou.as_str())));
        }
        if let Some(ref country) = self.country {
            parts.push(format!("C={}", country.as_str()));
        }
        if let Some(ref state) = self.state {
            parts.push(format!("ST={}", escape_rfc4514(state.as_str())));
        }
        if let Some(ref locality) = self.locality {
            parts.push(format!("L={}", escape_rfc4514(locality.as_str())));
        }
        if let Some(ref email) = self.email {
            parts.push(format!("emailAddress={}", escape_rfc4514(email.as_str())));
        }

        parts.join(",")
    }

    /// Parse from RFC 4514 Distinguished Name string
    ///
    /// Handles escaped characters including escaped commas.
    pub fn parse_rfc4514(dn: &str) -> Result<Self, SubjectNameError> {
        let mut common_name = None;
        let mut organization = None;
        let mut organizational_unit = None;
        let mut country = None;
        let mut state = None;
        let mut locality = None;
        let mut email = None;

        // Split by unescaped commas
        let parts = split_dn_parts(dn);

        for part in parts {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                let key = key.trim().to_uppercase();
                let value = unescape_rfc4514(value.trim());

                match key.as_str() {
                    "CN" => common_name = Some(CommonName::new_unchecked(value)),
                    "O" => organization = Some(OrganizationName::new_unchecked(value)),
                    "OU" => organizational_unit = Some(OrganizationalUnitName::new_unchecked(value)),
                    "C" => country = Some(CountryCode::new_unchecked(value)),
                    "ST" | "S" => state = Some(StateName::new_unchecked(value)),
                    "L" => locality = Some(LocalityName::new_unchecked(value)),
                    "EMAILADDRESS" | "EMAIL" => email = Some(EmailAddress::new_unchecked(value)),
                    _ => {} // Ignore unknown attributes
                }
            }
        }

        let common_name = common_name.ok_or(SubjectNameError::MissingCommonName)?;

        Ok(Self {
            common_name,
            organization,
            organizational_unit,
            country,
            state,
            locality,
            email,
        })
    }
}

impl fmt::Display for SubjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_rfc4514())
    }
}

impl DomainConcept for SubjectName {}
impl ValueObject for SubjectName {}

// ============================================================================
// NodeContributor Implementation for SubjectName (Sprint D)
// ============================================================================

impl crate::value_objects::NodeContributor for SubjectName {
    /// Generate labels for graph node based on subject name
    fn as_labels(&self) -> Vec<crate::value_objects::Label> {
        use crate::value_objects::Label;
        let mut labels = Vec::new();

        // Add country label if present
        if let Some(ref country) = self.country {
            labels.push(Label::new(format!("Country:{}", country.as_str())));
        }

        // Add organization label if present
        if self.organization.is_some() {
            labels.push(Label::new("HasOrganization"));
        }

        // Add email label if present in DN
        if self.email.is_some() {
            labels.push(Label::new("HasDnEmail"));
        }

        labels
    }

    /// Generate properties for graph node
    fn as_properties(&self) -> Vec<(crate::value_objects::PropertyKey, crate::value_objects::PropertyValue)> {
        use crate::value_objects::{PropertyKey, PropertyValue};

        let mut props = vec![
            (PropertyKey::new("subject_cn"), PropertyValue::string(self.common_name.as_str())),
            (PropertyKey::new("subject_dn"), PropertyValue::string(self.to_rfc4514())),
        ];

        if let Some(ref org) = self.organization {
            props.push((PropertyKey::new("subject_o"), PropertyValue::string(org.as_str())));
        }
        if let Some(ref ou) = self.organizational_unit {
            props.push((PropertyKey::new("subject_ou"), PropertyValue::string(ou.as_str())));
        }
        if let Some(ref country) = self.country {
            props.push((PropertyKey::new("subject_c"), PropertyValue::string(country.as_str())));
        }
        if let Some(ref state) = self.state {
            props.push((PropertyKey::new("subject_st"), PropertyValue::string(state.as_str())));
        }
        if let Some(ref locality) = self.locality {
            props.push((PropertyKey::new("subject_l"), PropertyValue::string(locality.as_str())));
        }
        if let Some(ref email) = self.email {
            props.push((PropertyKey::new("subject_email"), PropertyValue::string(email.as_str())));
        }

        props
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Split DN string by unescaped commas
///
/// Handles escaped commas (\,) properly.
fn split_dn_parts(dn: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = dn.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            // Escape sequence - include both backslash and next char
            current.push(c);
            if let Some(&next) = chars.peek() {
                current.push(next);
                chars.next();
            }
        } else if c == ',' {
            // Unescaped comma - split here
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }

    // Don't forget the last part
    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Escape special characters per RFC 4514
fn escape_rfc4514(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ',' | '+' | '"' | '\\' | '<' | '>' | ';' => {
                result.push('\\');
                result.push(c);
            }
            '#' if result.is_empty() => {
                result.push('\\');
                result.push(c);
            }
            ' ' if result.is_empty() => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    // Handle trailing space
    if result.ends_with(' ') {
        result.pop();
        result.push_str("\\ ");
    }
    result
}

/// Unescape RFC 4514 escaped characters
fn unescape_rfc4514(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                result.push(next);
                chars.next();
            }
        } else {
            result.push(c);
        }
    }
    result
}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur with Subject Name value objects
#[derive(Debug, Clone, thiserror::Error)]
pub enum SubjectNameError {
    #[error("Common Name (CN) is required but missing")]
    MissingCommonName,

    #[error("Common Name (CN) cannot be empty")]
    EmptyCommonName,

    #[error("Common Name (CN) too long: {0} characters (max 64)")]
    CommonNameTooLong(usize),

    #[error("Organization Name (O) cannot be empty")]
    EmptyOrganizationName,

    #[error("Organization Name (O) too long: {0} characters (max 64)")]
    OrganizationNameTooLong(usize),

    #[error("Organizational Unit Name (OU) cannot be empty")]
    EmptyOrganizationalUnitName,

    #[error("Organizational Unit Name (OU) too long: {0} characters (max 64)")]
    OrganizationalUnitNameTooLong(usize),

    #[error("Country Code (C) must be exactly 2 characters, got {0}")]
    InvalidCountryCodeLength(usize),

    #[error("Country Code (C) must be uppercase ASCII letters: {0}")]
    InvalidCountryCodeFormat(String),

    #[error("State Name (ST) cannot be empty")]
    EmptyStateName,

    #[error("State Name (ST) too long: {0} characters (max 64)")]
    StateNameTooLong(usize),

    #[error("Locality Name (L) cannot be empty")]
    EmptyLocalityName,

    #[error("Locality Name (L) too long: {0} characters (max 64)")]
    LocalityNameTooLong(usize),

    #[error("Invalid email format: {0}")]
    InvalidEmailFormat(String),

    #[error("Email too long: {0} characters (max 255)")]
    EmailTooLong(usize),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_name_creation() {
        let cn = CommonName::new("Alice Smith").unwrap();
        assert_eq!(cn.as_str(), "Alice Smith");
    }

    #[test]
    fn test_common_name_empty_rejected() {
        assert!(CommonName::new("").is_err());
    }

    #[test]
    fn test_common_name_too_long_rejected() {
        let long_name = "a".repeat(65);
        assert!(CommonName::new(long_name).is_err());
    }

    #[test]
    fn test_country_code_valid() {
        let code = CountryCode::new("US").unwrap();
        assert_eq!(code.as_str(), "US");
    }

    #[test]
    fn test_country_code_invalid_length() {
        assert!(CountryCode::new("USA").is_err());
        assert!(CountryCode::new("U").is_err());
    }

    #[test]
    fn test_country_code_invalid_format() {
        assert!(CountryCode::new("us").is_err()); // lowercase
        assert!(CountryCode::new("U1").is_err()); // digit
    }

    #[test]
    fn test_email_valid() {
        let email = EmailAddress::new("alice@example.com").unwrap();
        assert_eq!(email.local_part(), "alice");
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn test_email_invalid() {
        assert!(EmailAddress::new("invalid").is_err());
        assert!(EmailAddress::new("@example.com").is_err());
        assert!(EmailAddress::new("alice@").is_err());
    }

    #[test]
    fn test_subject_name_to_rfc4514() {
        let cn = CommonName::new_unchecked("Alice Smith");
        let org = OrganizationName::new_unchecked("Cowboy AI, LLC");
        let country = CountryCode::new_unchecked("US");

        let subject = SubjectName::new(cn)
            .with_organization(org)
            .with_country(country);

        let dn = subject.to_rfc4514();
        assert!(dn.contains("CN=Alice Smith"));
        assert!(dn.contains("O=Cowboy AI\\, LLC")); // Comma escaped
        assert!(dn.contains("C=US"));
    }

    #[test]
    fn test_subject_name_parse_rfc4514() {
        let dn = "CN=Alice Smith,O=Cowboy AI\\, LLC,C=US";
        let subject = SubjectName::parse_rfc4514(dn).unwrap();

        assert_eq!(subject.common_name.as_str(), "Alice Smith");
        assert_eq!(subject.organization.as_ref().unwrap().as_str(), "Cowboy AI, LLC");
        assert_eq!(subject.country.as_ref().unwrap().as_str(), "US");
    }

    #[test]
    fn test_subject_name_parse_missing_cn() {
        let dn = "O=Cowboy AI";
        assert!(SubjectName::parse_rfc4514(dn).is_err());
    }

    #[test]
    fn test_subject_name_display() {
        let cn = CommonName::new_unchecked("test.example.com");
        let subject = SubjectName::new(cn);
        assert_eq!(format!("{}", subject), "CN=test.example.com");
    }
}
