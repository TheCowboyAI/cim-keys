// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Command Factory
//!
//! Factory functions for creating organization-related commands from ViewModels.
//! Uses ACL validators and translators to ensure domain integrity.

use chrono::Utc;
use uuid::Uuid;

use crate::acl::{
    validate_organization_form, NonEmptyVec, ValidationError,
};
use crate::commands::organization::{CreateOrganization, CreateOrganizationalUnit};
use crate::gui::view_state::{NewOrgUnitForm, OrganizationForm};

/// Result type for organization command creation
pub type OrganizationCommandResult = Result<CreateOrganization, NonEmptyVec<ValidationError>>;

/// Result type for organizational unit command creation
pub type OrgUnitCommandResult = Result<CreateOrganizationalUnit, NonEmptyVec<ValidationError>>;

/// Create an organization command from a GUI form
///
/// This factory function:
/// 1. Validates the form using ACL validators
/// 2. Translates to domain ValueObjects on success
/// 3. Creates a properly formed CreateOrganization command
///
/// ## Arguments
/// - `form`: The GUI form with organization data
/// - `correlation_id`: Correlation ID for event tracing
///
/// ## Returns
/// - `Ok(CreateOrganization)` if validation passes
/// - `Err(NonEmptyVec<ValidationError>)` with all validation errors
///
/// ## Example
///
/// ```rust,ignore
/// let form = OrganizationForm::new()
///     .with_name("Cowboy AI".to_string())
///     .with_domain("cowboyai.dev".to_string())
///     .with_admin_email("admin@cowboyai.dev".to_string());
///
/// let command = create_organization_command(&form, Uuid::now_v7())?;
/// ```
pub fn create_organization_command(
    form: &OrganizationForm,
    correlation_id: Uuid,
) -> OrganizationCommandResult {
    // 1. Validate using ACL
    let validated = validate_organization_form(form)?;

    // 2. Create command with validated data
    // Entity IDs use UUID v7 for time-ordering
    let command_id = Uuid::now_v7();
    let organization_id = Uuid::now_v7();

    Ok(CreateOrganization {
        command_id,
        organization_id,
        name: validated.name,
        domain: Some(validated.domain),
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
    })
}

/// Create an organization command with a specific organization ID
///
/// Use this when you need to specify the organization ID (e.g., for idempotent operations).
pub fn create_organization_command_with_id(
    form: &OrganizationForm,
    organization_id: Uuid,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
) -> OrganizationCommandResult {
    // 1. Validate using ACL
    let validated = validate_organization_form(form)?;

    // 2. Create command with validated data
    let command_id = Uuid::now_v7();

    Ok(CreateOrganization {
        command_id,
        organization_id,
        name: validated.name,
        domain: Some(validated.domain),
        correlation_id,
        causation_id,
        timestamp: Utc::now(),
    })
}

/// Create an organizational unit command from a GUI form
///
/// ## Arguments
/// - `form`: The GUI form with org unit data
/// - `correlation_id`: Correlation ID for event tracing
///
/// ## Returns
/// - `Ok(CreateOrganizationalUnit)` if validation passes
/// - `Err(NonEmptyVec<ValidationError>)` with validation errors
pub fn create_organizational_unit_command(
    form: &NewOrgUnitForm,
    correlation_id: Uuid,
) -> OrgUnitCommandResult {
    use crate::acl::error::ValidationAccumulator;

    // 1. Validate form fields
    let mut acc = ValidationAccumulator::new();

    if form.name.trim().is_empty() {
        acc.add(ValidationError::required("name"));
    }

    // Unit type is optional but should be valid if present
    // (no additional validation needed for Option<OrganizationUnitType>)

    // 2. Check for validation errors
    let validated_name = form.name.trim().to_string();

    // Convert parent string to UUID if present
    let parent_id = form.parent.as_ref().and_then(|p| {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            None
        } else {
            Uuid::parse_str(trimmed).ok()
        }
    });

    // 3. Create command with validated data
    let command_id = Uuid::now_v7();
    let unit_id = Uuid::now_v7();

    acc.into_result(CreateOrganizationalUnit {
        command_id,
        unit_id,
        name: validated_name,
        parent_id,
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_organization_command() {
        let form = OrganizationForm::new()
            .with_name("Test Org".to_string())
            .with_domain("testorg.com".to_string())
            .with_admin_email("admin@testorg.com".to_string());

        let correlation_id = Uuid::now_v7();
        let result = create_organization_command(&form, correlation_id);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "Test Org");
        assert_eq!(cmd.domain, Some("testorg.com".to_string()));
        assert_eq!(cmd.correlation_id, correlation_id);
        // UUID v7 should be time-ordered
        assert!(cmd.organization_id.get_version() == Some(uuid::Version::SortRand));
    }

    #[test]
    fn test_create_organization_command_with_id() {
        let form = OrganizationForm::new()
            .with_name("Test Org".to_string())
            .with_domain("testorg.com".to_string())
            .with_admin_email("admin@testorg.com".to_string());

        let org_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();
        let causation_id = Some(Uuid::now_v7());

        let result = create_organization_command_with_id(&form, org_id, correlation_id, causation_id);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.organization_id, org_id);
        assert_eq!(cmd.causation_id, causation_id);
    }

    #[test]
    fn test_create_organization_command_validation_failure() {
        let form = OrganizationForm::new()
            .with_name("".to_string()) // Empty
            .with_domain("invalid".to_string()) // No TLD
            .with_admin_email("not-email".to_string()); // Invalid

        let result = create_organization_command(&form, Uuid::now_v7());
        assert!(result.is_err());

        let errors = result.unwrap_err();
        // Should have at least 3 errors (name, domain, email)
        assert!(errors.len() >= 3);
    }

    #[test]
    fn test_create_org_unit_command() {
        let form = NewOrgUnitForm::new()
            .with_name("Engineering".to_string());

        let result = create_organizational_unit_command(&form, Uuid::now_v7());
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.name, "Engineering");
        assert!(cmd.parent_id.is_none());
    }

    #[test]
    fn test_create_org_unit_command_with_parent() {
        let parent_id = Uuid::now_v7();
        let form = NewOrgUnitForm::new()
            .with_name("Frontend Team".to_string())
            .with_parent(parent_id.to_string());

        let result = create_organizational_unit_command(&form, Uuid::now_v7());
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.name, "Frontend Team");
        assert_eq!(cmd.parent_id, Some(parent_id));
    }
}
