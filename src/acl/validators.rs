// Copyright (c) 2025 - Cowboy AI, LLC.

//! Pure Validation Functions
//!
//! This module contains pure validation functions that validate ViewModels
//! and produce ValidatedForm types. These functions:
//!
//! 1. Are pure (no side effects)
//! 2. Accumulate ALL errors (not just the first)
//! 3. Return ValidatedForm types on success
//!
//! ## Validation Flow
//!
//! ```text
//! ViewModel (String) → validate_*() → Result<ValidatedForm, NonEmptyVec<Error>>
//! ```
//!
//! ## ValidatedForm Types
//!
//! ValidatedForm types are intermediate representations that have passed
//! validation but haven't been translated to domain ValueObjects yet.
//! They contain the original values plus any parsed/validated data.

use super::error::{ValidationAccumulator, ValidationError, ValidationResult};
use crate::domain::KeyOwnerRole;
use crate::gui::view_state::{
    CertificateForm, NewPersonForm, OrganizationForm, PassphraseState,
};

// ============================================================================
// VALIDATED FORM TYPES
// ============================================================================

/// Validated person form data
///
/// This intermediate type proves validation has passed.
/// Created by `validate_person_form()`, consumed by translators.
#[derive(Debug, Clone)]
pub struct ValidatedPersonForm {
    /// Validated non-empty name
    pub name: String,
    /// Validated email address
    pub email: String,
    /// Optional role (None is valid for this field)
    pub role: Option<KeyOwnerRole>,
}

/// Validated organization form data
#[derive(Debug, Clone)]
pub struct ValidatedOrganizationForm {
    /// Validated organization name
    pub name: String,
    /// Validated domain name
    pub domain: String,
    /// Validated admin email
    pub admin_email: String,
}

/// Validated passphrase state
#[derive(Debug, Clone)]
pub struct ValidatedPassphraseState {
    /// Master passphrase (validated non-empty and matching)
    pub master: String,
    /// Root passphrase (validated non-empty and matching)
    pub root: String,
}

/// Validated certificate form data
#[derive(Debug, Clone)]
pub struct ValidatedCertificateForm {
    /// Common name for certificate
    pub common_name: String,
    /// Organization name
    pub organization: String,
    /// Organizational unit (optional)
    pub organizational_unit: Option<String>,
    /// Locality (optional)
    pub locality: Option<String>,
    /// State/Province (optional)
    pub state_province: Option<String>,
    /// Country code (2 letters)
    pub country: Option<String>,
    /// Validity in days
    pub validity_days: u32,
    /// Subject alternative names (parsed)
    pub subject_alt_names: Vec<String>,
}

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

/// Validate a person form
///
/// Checks:
/// - Name is non-empty
/// - Email has valid format (contains @)
/// - Role is valid if present
///
/// Returns ValidatedPersonForm on success, or NonEmptyVec of all errors.
pub fn validate_person_form(form: &NewPersonForm) -> ValidationResult<ValidatedPersonForm> {
    let mut acc = ValidationAccumulator::new();

    // Validate name
    if form.name.trim().is_empty() {
        acc.add(ValidationError::required("name"));
    }

    // Validate email
    if form.email.trim().is_empty() {
        acc.add(ValidationError::required("email"));
    } else if !is_valid_email(&form.email) {
        acc.add(ValidationError::invalid_email("email"));
    }

    // Create validated form
    let validated = ValidatedPersonForm {
        name: form.name.trim().to_string(),
        email: form.email.trim().to_lowercase(),
        role: form.role.clone(),
    };

    acc.into_result(validated)
}

/// Validate an organization form
///
/// Checks:
/// - Name is non-empty
/// - Domain has valid format
/// - Admin email has valid format
pub fn validate_organization_form(
    form: &OrganizationForm,
) -> ValidationResult<ValidatedOrganizationForm> {
    let mut acc = ValidationAccumulator::new();

    // Validate name
    if form.name.trim().is_empty() {
        acc.add(ValidationError::required("name"));
    }

    // Validate domain
    if form.domain.trim().is_empty() {
        acc.add(ValidationError::required("domain"));
    } else if !is_valid_domain(&form.domain) {
        acc.add(ValidationError::invalid_domain("domain"));
    }

    // Validate admin email
    if form.admin_email.trim().is_empty() {
        acc.add(ValidationError::required("admin_email"));
    } else if !is_valid_email(&form.admin_email) {
        acc.add(ValidationError::invalid_email("admin_email"));
    }

    let validated = ValidatedOrganizationForm {
        name: form.name.trim().to_string(),
        domain: form.domain.trim().to_lowercase(),
        admin_email: form.admin_email.trim().to_lowercase(),
    };

    acc.into_result(validated)
}

/// Validate passphrase state
///
/// Checks:
/// - Master passphrase is non-empty
/// - Master passphrase matches confirmation
/// - Root passphrase is non-empty
/// - Root passphrase matches confirmation
/// - Both meet minimum length requirements
pub fn validate_passphrase_state(
    state: &PassphraseState,
) -> ValidationResult<ValidatedPassphraseState> {
    let mut acc = ValidationAccumulator::new();

    const MIN_PASSPHRASE_LENGTH: usize = 8;

    // Validate master passphrase
    if state.master.is_empty() {
        acc.add(ValidationError::required("master_passphrase"));
    } else {
        if state.master.len() < MIN_PASSPHRASE_LENGTH {
            acc.add(ValidationError::too_short(
                "master_passphrase",
                MIN_PASSPHRASE_LENGTH,
            ));
        }
        if state.master != state.master_confirm {
            acc.add(ValidationError::mismatch(
                "master_passphrase",
                "master_confirm",
            ));
        }
    }

    // Validate root passphrase
    if state.root.is_empty() {
        acc.add(ValidationError::required("root_passphrase"));
    } else {
        if state.root.len() < MIN_PASSPHRASE_LENGTH {
            acc.add(ValidationError::too_short(
                "root_passphrase",
                MIN_PASSPHRASE_LENGTH,
            ));
        }
        if state.root != state.root_confirm {
            acc.add(ValidationError::mismatch("root_passphrase", "root_confirm"));
        }
    }

    let validated = ValidatedPassphraseState {
        master: state.master.clone(),
        root: state.root.clone(),
    };

    acc.into_result(validated)
}

/// Validate a certificate form
///
/// Checks:
/// - Common name or intermediate CA name is non-empty
/// - Country code is 2 letters if present
/// - Validity days is a valid positive number
/// - SANs are parsed correctly
pub fn validate_certificate_form(
    form: &CertificateForm,
) -> ValidationResult<ValidatedCertificateForm> {
    let mut acc = ValidationAccumulator::new();

    // Determine common name (either from intermediate or server cert)
    let common_name = if !form.intermediate_ca_name.trim().is_empty() {
        form.intermediate_ca_name.trim().to_string()
    } else if !form.server_cert_cn.trim().is_empty() {
        form.server_cert_cn.trim().to_string()
    } else {
        acc.add(ValidationError::required("common_name"));
        String::new()
    };

    // Validate country code if present
    let country = if form.country.trim().is_empty() {
        None
    } else {
        let country_trimmed = form.country.trim().to_uppercase();
        if country_trimmed.len() != 2 {
            acc.add(ValidationError::new(
                "country",
                "Country code must be exactly 2 letters (ISO 3166-1 alpha-2)",
            ));
            None
        } else if !country_trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
            acc.add(ValidationError::new(
                "country",
                "Country code must contain only letters",
            ));
            None
        } else {
            Some(country_trimmed)
        }
    };

    // Parse validity days
    let validity_days = if form.validity_days.trim().is_empty() {
        365 // Default to 1 year
    } else {
        match form.validity_days.trim().parse::<u32>() {
            Ok(days) if days > 0 => days,
            Ok(_) => {
                acc.add(ValidationError::new(
                    "validity_days",
                    "Validity must be a positive number",
                ));
                365
            }
            Err(_) => {
                acc.add(ValidationError::new(
                    "validity_days",
                    "Validity must be a valid number",
                ));
                365
            }
        }
    };

    // Parse subject alternative names
    let subject_alt_names = parse_subject_alt_names(&form.server_cert_sans);

    let validated = ValidatedCertificateForm {
        common_name,
        organization: form.organization.trim().to_string(),
        organizational_unit: non_empty_option(&form.organizational_unit),
        locality: non_empty_option(&form.locality),
        state_province: non_empty_option(&form.state_province),
        country,
        validity_days,
        subject_alt_names,
    };

    acc.into_result(validated)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if a string is a valid email format
fn is_valid_email(email: &str) -> bool {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Basic validation: must have exactly one @, with text before and after
    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    // Local part must be non-empty and not start/end with a dot
    if local.is_empty() || local.starts_with('.') || local.ends_with('.') {
        return false;
    }

    // Domain must have at least one dot and be valid
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }

    // Domain parts must be non-empty
    for part in domain.split('.') {
        if part.is_empty() {
            return false;
        }
    }

    true
}

/// Check if a string is a valid domain format
fn is_valid_domain(domain: &str) -> bool {
    let trimmed = domain.trim().to_lowercase();
    if trimmed.is_empty() {
        return false;
    }

    // Must have at least one dot
    if !trimmed.contains('.') {
        return false;
    }

    // Each part must be valid
    for part in trimmed.split('.') {
        if part.is_empty() {
            return false;
        }
        // Must not start or end with hyphen
        if part.starts_with('-') || part.ends_with('-') {
            return false;
        }
        // Must contain only alphanumeric or hyphen
        if !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
    }

    true
}

/// Convert an empty string to None
fn non_empty_option(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse subject alternative names from a comma or newline-separated string
fn parse_subject_alt_names(sans: &str) -> Vec<String> {
    sans.split(|c| c == ',' || c == '\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.org"));
        assert!(is_valid_email("a@b.co"));

        assert!(!is_valid_email(""));
        assert!(!is_valid_email("no-at-sign"));
        assert!(!is_valid_email("@no-local.com"));
        assert!(!is_valid_email("no-domain@"));
        assert!(!is_valid_email("no@tld"));
        assert!(!is_valid_email("@"));
        assert!(!is_valid_email("double@@at.com"));
    }

    #[test]
    fn test_domain_validation() {
        assert!(is_valid_domain("example.com"));
        assert!(is_valid_domain("sub.domain.org"));
        assert!(is_valid_domain("my-domain.co.uk"));

        assert!(!is_valid_domain(""));
        assert!(!is_valid_domain("no-dot"));
        assert!(!is_valid_domain(".starts-with-dot.com"));
        assert!(!is_valid_domain("-starts-with-hyphen.com"));
        assert!(!is_valid_domain("ends-with-hyphen-.com"));
    }

    #[test]
    fn test_validate_person_form_success() {
        let form = NewPersonForm::new()
            .with_name("John Doe".to_string())
            .with_email("john@example.com".to_string())
            .with_role(KeyOwnerRole::Developer);

        let result = validate_person_form(&form);
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert_eq!(validated.name, "John Doe");
        assert_eq!(validated.email, "john@example.com");
        assert_eq!(validated.role, Some(KeyOwnerRole::Developer));
    }

    #[test]
    fn test_validate_person_form_trims_whitespace() {
        let form = NewPersonForm::new()
            .with_name("  John Doe  ".to_string())
            .with_email("  JOHN@EXAMPLE.COM  ".to_string());

        let result = validate_person_form(&form);
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert_eq!(validated.name, "John Doe");
        assert_eq!(validated.email, "john@example.com");
    }

    #[test]
    fn test_validate_person_form_accumulates_errors() {
        let form = NewPersonForm::new()
            .with_name("".to_string())
            .with_email("invalid-email".to_string());

        let result = validate_person_form(&form);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);

        let error_fields: Vec<_> = errors.iter().map(|e| e.field.as_str()).collect();
        assert!(error_fields.contains(&"name"));
        assert!(error_fields.contains(&"email"));
    }

    #[test]
    fn test_validate_organization_form() {
        let form = OrganizationForm::new()
            .with_name("Cowboy AI".to_string())
            .with_domain("cowboyai.dev".to_string())
            .with_admin_email("admin@cowboyai.dev".to_string());

        let result = validate_organization_form(&form);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_passphrase_state() {
        let valid = PassphraseState::new()
            .with_master("SecurePassword123".to_string())
            .with_master_confirm("SecurePassword123".to_string())
            .with_root("RootPassword456".to_string())
            .with_root_confirm("RootPassword456".to_string());

        assert!(validate_passphrase_state(&valid).is_ok());

        // Too short
        let short = PassphraseState::new()
            .with_master("short".to_string())
            .with_master_confirm("short".to_string())
            .with_root("short".to_string())
            .with_root_confirm("short".to_string());

        let result = validate_passphrase_state(&short);
        assert!(result.is_err());
        // Should have at least 2 errors (master too short, root too short)
        assert!(result.unwrap_err().len() >= 2);
    }

    #[test]
    fn test_validate_certificate_form() {
        let form = CertificateForm::new()
            .with_intermediate_ca_name("My CA".to_string())
            .with_organization("Test Org".to_string())
            .with_country("US".to_string())
            .with_validity_days("365".to_string());

        let result = validate_certificate_form(&form);
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert_eq!(validated.common_name, "My CA");
        assert_eq!(validated.country, Some("US".to_string()));
        assert_eq!(validated.validity_days, 365);
    }

    #[test]
    fn test_parse_subject_alt_names() {
        let sans = "example.com, *.example.com\napi.example.com";
        let parsed = parse_subject_alt_names(sans);

        assert_eq!(parsed.len(), 3);
        assert!(parsed.contains(&"example.com".to_string()));
        assert!(parsed.contains(&"*.example.com".to_string()));
        assert!(parsed.contains(&"api.example.com".to_string()));
    }
}
