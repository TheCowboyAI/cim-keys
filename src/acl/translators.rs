// Copyright (c) 2025 - Cowboy AI, LLC.

//! Pure Translation Functions (ViewModel → ValueObject)
//!
//! This module contains pure translation functions that convert ValidatedForms
//! into domain ValueObjects. These functions:
//!
//! 1. Are pure (no side effects)
//! 2. Cannot fail (validation already passed)
//! 3. Return immutable ValueObjects
//!
//! ## Translation Flow
//!
//! ```text
//! ValidatedForm → translate_*() → ValueObject
//! ```
//!
//! ## ValueObject Properties
//!
//! ValueObjects returned by these functions:
//! - Are immutable after creation
//! - Have CID-based identity (content-addressable)
//! - Can be persisted to NATS JetStream
//! - Follow `cim_domain::formal_domain::ValueObject` trait

use super::validators::{
    ValidatedCertificateForm, ValidatedOrganizationForm, ValidatedPersonForm,
};
use crate::value_objects::{
    CertificateValidity, CommonName, CountryCode, LocalityName, OrganizationName,
    OrganizationalUnitName, StateName, SubjectName,
};
use chrono::{Duration, Utc};

// ============================================================================
// PERSON TRANSLATIONS
// ============================================================================

/// Translate a validated person form to a PersonName value object
///
/// PersonName is a simple wrapper that enforces non-empty invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonName(String);

impl PersonName {
    /// Create a new PersonName (infallible - already validated)
    pub fn new(name: String) -> Self {
        Self(name)
    }

    /// Get the name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for PersonName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Translate validated person form to PersonName
pub fn translate_person_name(validated: &ValidatedPersonForm) -> PersonName {
    PersonName::new(validated.name.clone())
}

/// Translate a validated email to an EmailAddress value object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Create a new EmailAddress (infallible - already validated)
    pub fn new(email: String) -> Self {
        Self(email)
    }

    /// Get the email as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the local part (before @)
    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }

    /// Get the domain part (after @)
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }

    /// Consume and return the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Translate validated person form to EmailAddress
pub fn translate_email_address(validated: &ValidatedPersonForm) -> EmailAddress {
    EmailAddress::new(validated.email.clone())
}

// ============================================================================
// ORGANIZATION TRANSLATIONS
// ============================================================================

/// Translate validated organization form to OrganizationName
pub fn translate_organization_name(validated: &ValidatedOrganizationForm) -> OrganizationName {
    // OrganizationName from value_objects/x509.rs
    OrganizationName::new_unchecked(&validated.name)
}

/// DomainName value object for organization domains
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainName(String);

impl DomainName {
    /// Create a new DomainName (infallible - already validated)
    pub fn new(domain: String) -> Self {
        Self(domain)
    }

    /// Get the domain as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the top-level domain
    pub fn tld(&self) -> &str {
        self.0.rsplit('.').next().unwrap_or("")
    }

    /// Consume and return the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for DomainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Translate validated organization form to DomainName
pub fn translate_domain_name(validated: &ValidatedOrganizationForm) -> DomainName {
    DomainName::new(validated.domain.clone())
}

// ============================================================================
// CERTIFICATE TRANSLATIONS
// ============================================================================

/// Translate validated certificate form to CommonName
pub fn translate_common_name(validated: &ValidatedCertificateForm) -> CommonName {
    CommonName::new_unchecked(&validated.common_name)
}

/// Translate validated certificate form to SubjectName
///
/// Composes multiple fields into a complete X.509 subject name.
pub fn translate_subject_name(validated: &ValidatedCertificateForm) -> SubjectName {
    // Start with required common name
    let common_name = CommonName::new_unchecked(&validated.common_name);
    let mut subject = SubjectName::new(common_name);

    // Organization if present
    if !validated.organization.is_empty() {
        subject = subject.with_organization(OrganizationName::new_unchecked(&validated.organization));
    }

    // Optional fields
    if let Some(ref ou) = validated.organizational_unit {
        subject = subject.with_organizational_unit(OrganizationalUnitName::new_unchecked(ou));
    }

    if let Some(ref locality) = validated.locality {
        subject = subject.with_locality(LocalityName::new_unchecked(locality));
    }

    if let Some(ref state) = validated.state_province {
        subject = subject.with_state(StateName::new_unchecked(state));
    }

    if let Some(ref country) = validated.country {
        subject = subject.with_country(CountryCode::new_unchecked(country));
    }

    subject
}

/// Translate validated certificate form to CertificateValidity
///
/// Creates a validity period from now until the specified number of days.
pub fn translate_certificate_validity(validated: &ValidatedCertificateForm) -> CertificateValidity {
    let not_before = Utc::now();
    let not_after = not_before + Duration::days(validated.validity_days as i64);

    CertificateValidity::new(not_before, not_after).expect("validity calculation should not fail")
}

// ============================================================================
// COMPOSITE TRANSLATIONS
// ============================================================================

/// All person-related value objects from a validated form
pub struct PersonValueObjects {
    pub name: PersonName,
    pub email: EmailAddress,
}

/// Translate a validated person form to all its value objects
pub fn translate_person(validated: &ValidatedPersonForm) -> PersonValueObjects {
    PersonValueObjects {
        name: translate_person_name(validated),
        email: translate_email_address(validated),
    }
}

/// All organization-related value objects from a validated form
pub struct OrganizationValueObjects {
    pub name: OrganizationName,
    pub domain: DomainName,
    pub admin_email: EmailAddress,
}

/// Translate a validated organization form to all its value objects
pub fn translate_organization(validated: &ValidatedOrganizationForm) -> OrganizationValueObjects {
    OrganizationValueObjects {
        name: translate_organization_name(validated),
        domain: translate_domain_name(validated),
        admin_email: EmailAddress::new(validated.admin_email.clone()),
    }
}

/// All certificate-related value objects from a validated form
pub struct CertificateValueObjects {
    pub subject_name: SubjectName,
    pub validity: CertificateValidity,
    pub subject_alt_names: Vec<String>,
}

/// Translate a validated certificate form to all its value objects
pub fn translate_certificate(validated: &ValidatedCertificateForm) -> CertificateValueObjects {
    CertificateValueObjects {
        subject_name: translate_subject_name(validated),
        validity: translate_certificate_validity(validated),
        subject_alt_names: validated.subject_alt_names.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_validated_person() -> ValidatedPersonForm {
        ValidatedPersonForm {
            name: "Alice Smith".to_string(),
            email: "alice@example.com".to_string(),
            role: None,
        }
    }

    fn make_validated_organization() -> ValidatedOrganizationForm {
        ValidatedOrganizationForm {
            name: "Cowboy AI".to_string(),
            domain: "cowboyai.dev".to_string(),
            admin_email: "admin@cowboyai.dev".to_string(),
        }
    }

    fn make_validated_certificate() -> ValidatedCertificateForm {
        ValidatedCertificateForm {
            common_name: "Test CA".to_string(),
            organization: "Test Org".to_string(),
            organizational_unit: Some("Engineering".to_string()),
            locality: Some("Austin".to_string()),
            state_province: Some("TX".to_string()),
            country: Some("US".to_string()),
            validity_days: 365,
            subject_alt_names: vec!["example.com".to_string()],
        }
    }

    #[test]
    fn test_translate_person_name() {
        let validated = make_validated_person();
        let name = translate_person_name(&validated);

        assert_eq!(name.as_str(), "Alice Smith");
        assert_eq!(name.to_string(), "Alice Smith");
    }

    #[test]
    fn test_translate_email_address() {
        let validated = make_validated_person();
        let email = translate_email_address(&validated);

        assert_eq!(email.as_str(), "alice@example.com");
        assert_eq!(email.local_part(), "alice");
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn test_translate_organization() {
        let validated = make_validated_organization();
        let org = translate_organization(&validated);

        assert_eq!(org.name.as_str(), "Cowboy AI");
        assert_eq!(org.domain.as_str(), "cowboyai.dev");
        assert_eq!(org.domain.tld(), "dev");
    }

    #[test]
    fn test_translate_subject_name() {
        let validated = make_validated_certificate();
        let subject = translate_subject_name(&validated);

        // SubjectName should contain the common name (access via public field)
        assert_eq!(subject.common_name.as_str(), "Test CA");
    }

    #[test]
    fn test_translate_certificate_validity() {
        let validated = make_validated_certificate();
        let validity = translate_certificate_validity(&validated);

        // Should be valid for approximately 365 days
        let duration = validity.not_after() - validity.not_before();
        assert_eq!(duration.num_days(), 365);
    }

    #[test]
    fn test_translate_full_person() {
        let validated = make_validated_person();
        let value_objects = translate_person(&validated);

        assert_eq!(value_objects.name.as_str(), "Alice Smith");
        assert_eq!(value_objects.email.as_str(), "alice@example.com");
    }

    #[test]
    fn test_translate_full_certificate() {
        let validated = make_validated_certificate();
        let value_objects = translate_certificate(&validated);

        assert_eq!(value_objects.subject_name.common_name.as_str(), "Test CA");
        assert_eq!(value_objects.subject_alt_names.len(), 1);
    }
}
