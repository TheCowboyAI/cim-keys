// Copyright (c) 2025 - Cowboy AI, LLC.

//! Service Account Command Factory
//!
//! Factory functions for creating service account commands from ViewModels.
//! Enforces the accountability requirement: every service account must have
//! a responsible person.

use chrono::Utc;
use uuid::Uuid;

use crate::acl::{NonEmptyVec, ValidationError};
use crate::acl::error::ValidationAccumulator;
use crate::commands::organization::CreateServiceAccount;
use crate::gui::view_state::NewServiceAccountForm;

/// Result type for service account command creation
pub type ServiceAccountCommandResult = Result<CreateServiceAccount, NonEmptyVec<ValidationError>>;

/// Create a service account command from a GUI form
///
/// This factory function:
/// 1. Validates the form fields (name, purpose required)
/// 2. Enforces accountability requirement (owning_unit, responsible_person required)
/// 3. Creates a properly formed CreateServiceAccount command
///
/// ## Arguments
/// - `form`: The GUI form with service account data
/// - `correlation_id`: Correlation ID for event tracing
///
/// ## Returns
/// - `Ok(CreateServiceAccount)` if validation passes
/// - `Err(NonEmptyVec<ValidationError>)` with all validation errors
///
/// ## Example
///
/// ```rust,ignore
/// let form = NewServiceAccountForm::new()
///     .with_name("backup-agent".to_string())
///     .with_purpose("Automated backup operations".to_string())
///     .with_owning_unit(unit_id)
///     .with_responsible_person(person_id);
///
/// let command = create_service_account_command(&form, Uuid::now_v7())?;
/// ```
pub fn create_service_account_command(
    form: &NewServiceAccountForm,
    correlation_id: Uuid,
) -> ServiceAccountCommandResult {
    // 1. Validate form fields using accumulator pattern
    let mut acc = ValidationAccumulator::new();

    // Required: name
    if form.name.trim().is_empty() {
        acc.add(ValidationError::required("name"));
    }

    // Required: purpose
    if form.purpose.trim().is_empty() {
        acc.add(ValidationError::required("purpose"));
    }

    // Required: owning unit (accountability)
    let owning_unit_id = match form.owning_unit {
        Some(id) => id,
        None => {
            acc.add(ValidationError::new(
                "owning_unit",
                "Owning organizational unit is required",
            ));
            Uuid::nil() // Placeholder, won't be used if validation fails
        }
    };

    // Required: responsible person (accountability)
    let responsible_person_id = match form.responsible_person {
        Some(id) => id,
        None => {
            acc.add(ValidationError::new(
                "responsible_person",
                "A responsible person is required for service accounts",
            ));
            Uuid::nil() // Placeholder, won't be used if validation fails
        }
    };

    // 2. Create command with validated data
    let command_id = Uuid::now_v7();
    let service_account_id = Uuid::now_v7();

    acc.into_result(CreateServiceAccount {
        command_id,
        service_account_id,
        name: form.name.trim().to_string(),
        purpose: form.purpose.trim().to_string(),
        owning_unit_id,
        responsible_person_id,
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
    })
}

/// Create a service account command with a specific ID
///
/// Use this when you need to specify the service account ID (e.g., for idempotent operations).
pub fn create_service_account_command_with_id(
    form: &NewServiceAccountForm,
    service_account_id: Uuid,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
) -> ServiceAccountCommandResult {
    // 1. Validate form fields using accumulator pattern
    let mut acc = ValidationAccumulator::new();

    if form.name.trim().is_empty() {
        acc.add(ValidationError::required("name"));
    }

    if form.purpose.trim().is_empty() {
        acc.add(ValidationError::required("purpose"));
    }

    let owning_unit_id = match form.owning_unit {
        Some(id) => id,
        None => {
            acc.add(ValidationError::new(
                "owning_unit",
                "Owning organizational unit is required",
            ));
            Uuid::nil()
        }
    };

    let responsible_person_id = match form.responsible_person {
        Some(id) => id,
        None => {
            acc.add(ValidationError::new(
                "responsible_person",
                "A responsible person is required for service accounts",
            ));
            Uuid::nil()
        }
    };

    // 2. Create command with validated data
    let command_id = Uuid::now_v7();

    acc.into_result(CreateServiceAccount {
        command_id,
        service_account_id,
        name: form.name.trim().to_string(),
        purpose: form.purpose.trim().to_string(),
        owning_unit_id,
        responsible_person_id,
        correlation_id,
        causation_id,
        timestamp: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_service_account_command_success() {
        let unit_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();

        let form = NewServiceAccountForm::new()
            .with_name("backup-agent".to_string())
            .with_purpose("Automated backup operations".to_string())
            .with_owning_unit(unit_id)
            .with_responsible_person(person_id);

        let correlation_id = Uuid::now_v7();
        let result = create_service_account_command(&form, correlation_id);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "backup-agent");
        assert_eq!(cmd.purpose, "Automated backup operations");
        assert_eq!(cmd.owning_unit_id, unit_id);
        assert_eq!(cmd.responsible_person_id, person_id);
        assert_eq!(cmd.correlation_id, correlation_id);
    }

    #[test]
    fn test_create_service_account_command_missing_required_fields() {
        let form = NewServiceAccountForm::new()
            .with_name("".to_string())
            .with_purpose("".to_string());
        // owning_unit and responsible_person are None by default

        let result = create_service_account_command(&form, Uuid::now_v7());

        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should have 4 errors (name, purpose, owning_unit, responsible_person)
        assert_eq!(errors.len(), 4);
    }

    #[test]
    fn test_create_service_account_command_missing_accountability() {
        // Has name and purpose, but missing accountability fields
        let form = NewServiceAccountForm::new()
            .with_name("test-service".to_string())
            .with_purpose("Test purpose".to_string());

        let result = create_service_account_command(&form, Uuid::now_v7());

        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should have 2 errors (owning_unit, responsible_person)
        assert_eq!(errors.len(), 2);

        // Check that the errors are about accountability
        let error_fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
        assert!(error_fields.contains(&"owning_unit"));
        assert!(error_fields.contains(&"responsible_person"));
    }

    #[test]
    fn test_create_service_account_command_with_id() {
        let sa_id = Uuid::now_v7();
        let unit_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();
        let causation_id = Some(Uuid::now_v7());

        let form = NewServiceAccountForm::new()
            .with_name("deploy-agent".to_string())
            .with_purpose("Deployment automation".to_string())
            .with_owning_unit(unit_id)
            .with_responsible_person(person_id);

        let correlation_id = Uuid::now_v7();
        let result = create_service_account_command_with_id(
            &form,
            sa_id,
            correlation_id,
            causation_id,
        );

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.service_account_id, sa_id);
        assert_eq!(cmd.causation_id, causation_id);
    }

    #[test]
    fn test_create_service_account_command_trims_whitespace() {
        let unit_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();

        let form = NewServiceAccountForm::new()
            .with_name("  monitoring-agent  ".to_string())
            .with_purpose("  System monitoring  ".to_string())
            .with_owning_unit(unit_id)
            .with_responsible_person(person_id);

        let result = create_service_account_command(&form, Uuid::now_v7());

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "monitoring-agent");
        assert_eq!(cmd.purpose, "System monitoring");
    }
}
