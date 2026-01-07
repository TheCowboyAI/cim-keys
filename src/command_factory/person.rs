// Copyright (c) 2025 - Cowboy AI, LLC.

//! Person Command Factory
//!
//! Factory functions for creating person-related commands from ViewModels.
//! Uses ACL validators and translators to ensure domain integrity.

use chrono::Utc;
use uuid::Uuid;

use crate::acl::{validate_person_form, NonEmptyVec, ValidationError};
use crate::commands::organization::CreatePerson;
use crate::gui::view_state::NewPersonForm;

/// Result type for person command creation
pub type PersonCommandResult = Result<CreatePerson, NonEmptyVec<ValidationError>>;

/// Create a person command from a GUI form
///
/// This factory function:
/// 1. Validates the form using ACL validators
/// 2. Translates to domain ValueObjects on success
/// 3. Creates a properly formed CreatePerson command
///
/// ## Arguments
/// - `form`: The GUI form with person data
/// - `organization_id`: Optional organization this person belongs to
/// - `correlation_id`: Correlation ID for event tracing
///
/// ## Returns
/// - `Ok(CreatePerson)` if validation passes
/// - `Err(NonEmptyVec<ValidationError>)` with all validation errors
///
/// ## Example
///
/// ```rust,ignore
/// let form = NewPersonForm::new()
///     .with_name("Alice Smith".to_string())
///     .with_email("alice@example.com".to_string())
///     .with_role(KeyOwnerRole::Developer);
///
/// let command = create_person_command(&form, Some(org_id), Uuid::now_v7())?;
/// ```
pub fn create_person_command(
    form: &NewPersonForm,
    organization_id: Option<Uuid>,
    correlation_id: Uuid,
) -> PersonCommandResult {
    // 1. Validate using ACL
    let validated = validate_person_form(form)?;

    // 2. Create command with validated data
    // Entity IDs use UUID v7 for time-ordering
    let command_id = Uuid::now_v7();
    let person_id = Uuid::now_v7();

    // Map role to department/title if present
    let (title, department) = match &validated.role {
        Some(role) => {
            let title = Some(format!("{:?}", role));
            let department = None; // Could be derived from role if needed
            (title, department)
        }
        None => (None, None),
    };

    Ok(CreatePerson {
        command_id,
        person_id,
        name: validated.name,
        email: validated.email,
        title,
        department,
        organization_id,
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
    })
}

/// Create a person command with a specific person ID
///
/// Use this when you need to specify the person ID (e.g., for idempotent operations).
pub fn create_person_command_with_id(
    form: &NewPersonForm,
    person_id: Uuid,
    organization_id: Option<Uuid>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
) -> PersonCommandResult {
    // 1. Validate using ACL
    let validated = validate_person_form(form)?;

    // 2. Create command with validated data
    let command_id = Uuid::now_v7();

    let (title, department) = match &validated.role {
        Some(role) => (Some(format!("{:?}", role)), None),
        None => (None, None),
    };

    Ok(CreatePerson {
        command_id,
        person_id,
        name: validated.name,
        email: validated.email,
        title,
        department,
        organization_id,
        correlation_id,
        causation_id,
        timestamp: Utc::now(),
    })
}

/// Batch create multiple person commands from a list of forms
///
/// Returns a tuple of (successful commands, validation errors per form index).
/// This allows partial success when some forms are valid and others are not.
pub fn create_person_commands_batch(
    forms: &[NewPersonForm],
    organization_id: Option<Uuid>,
    correlation_id: Uuid,
) -> (Vec<CreatePerson>, Vec<(usize, NonEmptyVec<ValidationError>)>) {
    let mut commands = Vec::new();
    let mut errors = Vec::new();

    for (index, form) in forms.iter().enumerate() {
        match create_person_command(form, organization_id, correlation_id) {
            Ok(cmd) => commands.push(cmd),
            Err(validation_errors) => errors.push((index, validation_errors)),
        }
    }

    (commands, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::KeyOwnerRole;

    #[test]
    fn test_create_person_command() {
        let form = NewPersonForm::new()
            .with_name("Alice Smith".to_string())
            .with_email("alice@example.com".to_string())
            .with_role(KeyOwnerRole::Developer);

        let org_id = Some(Uuid::now_v7());
        let correlation_id = Uuid::now_v7();

        let result = create_person_command(&form, org_id, correlation_id);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "Alice Smith");
        assert_eq!(cmd.email, "alice@example.com");
        assert_eq!(cmd.organization_id, org_id);
        assert_eq!(cmd.correlation_id, correlation_id);
        assert!(cmd.title.is_some());
    }

    #[test]
    fn test_create_person_command_without_role() {
        let form = NewPersonForm::new()
            .with_name("Bob Jones".to_string())
            .with_email("bob@example.com".to_string());

        let result = create_person_command(&form, None, Uuid::now_v7());

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.title.is_none());
        assert!(cmd.organization_id.is_none());
    }

    #[test]
    fn test_create_person_command_validation_failure() {
        let form = NewPersonForm::new()
            .with_name("".to_string()) // Empty
            .with_email("not-email".to_string()); // Invalid

        let result = create_person_command(&form, None, Uuid::now_v7());
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_create_person_command_with_id() {
        let form = NewPersonForm::new()
            .with_name("Charlie".to_string())
            .with_email("charlie@example.com".to_string());

        let person_id = Uuid::now_v7();
        let causation_id = Some(Uuid::now_v7());

        let result = create_person_command_with_id(
            &form,
            person_id,
            None,
            Uuid::now_v7(),
            causation_id,
        );

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.person_id, person_id);
        assert_eq!(cmd.causation_id, causation_id);
    }

    #[test]
    fn test_batch_create_person_commands() {
        let forms = vec![
            NewPersonForm::new()
                .with_name("Alice".to_string())
                .with_email("alice@example.com".to_string()),
            NewPersonForm::new()
                .with_name("".to_string()) // Invalid - will fail
                .with_email("invalid".to_string()),
            NewPersonForm::new()
                .with_name("Charlie".to_string())
                .with_email("charlie@example.com".to_string()),
        ];

        let (commands, errors) = create_person_commands_batch(&forms, None, Uuid::now_v7());

        // Should have 2 successful commands (Alice and Charlie)
        assert_eq!(commands.len(), 2);
        // Should have 1 error (Bob at index 1)
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, 1); // Index of failed form
    }
}
