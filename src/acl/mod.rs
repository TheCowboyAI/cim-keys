// Copyright (c) 2025 - Cowboy AI, LLC.

//! Anti-Corruption Layer (ACL) Module
//!
//! This module provides pure translation functions between the presentation layer
//! (ViewModels) and the domain layer (ValueObjects, Commands).
//!
//! ## DDD Anti-Corruption Layer Pattern
//!
//! The ACL protects the domain model from presentation concerns:
//!
//! ```text
//! ViewModel (String) → validate() → ValidatedForm → translate() → ValueObject
//! ```
//!
//! ## Key Properties
//!
//! 1. **Pure Functions**: All validators and translators are pure (no side effects)
//! 2. **Error Accumulation**: Validation collects ALL errors, not just the first
//! 3. **Immutable Transformation**: ViewModels → ValueObjects, never mutated
//! 4. **Type-Safe Boundaries**: Clear separation between layers
//!
//! ## Module Structure
//!
//! - `error`: Validation error types and NonEmptyVec accumulator
//! - `validators`: Pure validation functions for each ViewModel
//! - `translators`: ViewModel → ValueObject translation functions
//!
//! ## Example
//!
//! ```rust,ignore
//! use crate::acl::{validate_person_form, translate_person};
//! use crate::gui::view_state::NewPersonForm;
//!
//! let form = NewPersonForm::new()
//!     .with_name("John Doe".to_string())
//!     .with_email("john@example.com".to_string());
//!
//! // Validate returns Result<ValidatedPersonForm, NonEmptyVec<ValidationError>>
//! match validate_person_form(&form) {
//!     Ok(validated) => {
//!         // Translate to domain ValueObjects
//!         let person_name = translate_person_name(&validated);
//!         let email = translate_email(&validated);
//!     }
//!     Err(errors) => {
//!         // All errors accumulated for user feedback
//!         for error in errors.iter() {
//!             eprintln!("{}: {}", error.field, error.message);
//!         }
//!     }
//! }
//! ```

pub mod error;
pub mod translators;
pub mod validators;

// Re-export primary types
pub use error::{NonEmptyVec, ValidationError};
pub use validators::{
    ValidatedCertificateForm, ValidatedOrganizationForm, ValidatedPassphraseState,
    ValidatedPersonForm,
};

// Re-export validation functions
pub use validators::{
    validate_certificate_form, validate_organization_form, validate_passphrase_state,
    validate_person_form,
};

// Re-export translation functions
pub use translators::{
    translate_certificate_validity, translate_common_name, translate_email_address,
    translate_organization_name, translate_person_name, translate_subject_name,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::KeyOwnerRole;
    use crate::gui::view_state::{NewPersonForm, OrganizationForm, PassphraseState};

    #[test]
    fn test_full_person_workflow() {
        // 1. Create ViewModel with user input
        let form = NewPersonForm::new()
            .with_name("Alice Smith".to_string())
            .with_email("alice@example.com".to_string())
            .with_role(KeyOwnerRole::Developer);

        // 2. Validate (pure function)
        let validated = validate_person_form(&form).expect("should validate");

        // 3. Translate to ValueObjects (pure function)
        let person_name = translate_person_name(&validated);
        let email = translate_email_address(&validated);

        assert_eq!(person_name.as_str(), "Alice Smith");
        assert_eq!(email.as_str(), "alice@example.com");
    }

    #[test]
    fn test_validation_accumulates_all_errors() {
        // Create ViewModel with multiple invalid fields
        let form = NewPersonForm::new()
            .with_name("".to_string()) // Empty name - error
            .with_email("not-an-email".to_string()); // Invalid email - error

        // Validate should return ALL errors
        let result = validate_person_form(&form);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        // Should have at least 2 errors (name and email)
        assert!(errors.len() >= 2, "Expected at least 2 errors, got {}", errors.len());
    }

    #[test]
    fn test_organization_workflow() {
        let form = OrganizationForm::new()
            .with_name("Cowboy AI".to_string())
            .with_domain("cowboyai.dev".to_string())
            .with_admin_email("admin@cowboyai.dev".to_string());

        let validated = validate_organization_form(&form).expect("should validate");
        let org_name = translate_organization_name(&validated);

        assert_eq!(org_name.as_str(), "Cowboy AI");
    }

    #[test]
    fn test_passphrase_validation() {
        // Valid passphrases that match
        let valid = PassphraseState::new()
            .with_master("SecurePass123!".to_string())
            .with_master_confirm("SecurePass123!".to_string())
            .with_root("RootPass456!".to_string())
            .with_root_confirm("RootPass456!".to_string());

        assert!(validate_passphrase_state(&valid).is_ok());

        // Invalid: passphrases don't match
        let mismatch = PassphraseState::new()
            .with_master("password1".to_string())
            .with_master_confirm("password2".to_string())
            .with_root("root1".to_string())
            .with_root_confirm("root2".to_string());

        let result = validate_passphrase_state(&mismatch);
        assert!(result.is_err());

        // Should report both mismatches
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
    }
}
