// Copyright (c) 2025 - Cowboy AI, LLC.

//! Location Command Factory
//!
//! Factory functions for creating location-related commands from ViewModels.
//! Uses ACL validators to ensure domain integrity.

use chrono::Utc;
use uuid::Uuid;

use crate::acl::{NonEmptyVec, ValidationError};
use crate::acl::error::ValidationAccumulator;
use crate::commands::organization::CreateLocation;
use crate::gui::view_state::NewLocationForm;

/// Result type for location command creation
pub type LocationCommandResult = Result<CreateLocation, NonEmptyVec<ValidationError>>;

/// Create a location command from a GUI form
///
/// This factory function:
/// 1. Validates the form fields
/// 2. Creates a properly formed CreateLocation command
///
/// ## Arguments
/// - `form`: The GUI form with location data
/// - `organization_id`: The organization this location belongs to
/// - `correlation_id`: Correlation ID for event tracing
///
/// ## Returns
/// - `Ok(CreateLocation)` if validation passes
/// - `Err(NonEmptyVec<ValidationError>)` with all validation errors
///
/// ## Example
///
/// ```rust,ignore
/// let form = NewLocationForm::new()
///     .with_name("HQ Office".to_string())
///     .with_street("123 Main St".to_string())
///     .with_city("Austin".to_string())
///     .with_region("TX".to_string())
///     .with_country("USA".to_string())
///     .with_postal("78701".to_string());
///
/// let command = create_location_command(&form, Some(org_id), Uuid::now_v7())?;
/// ```
pub fn create_location_command(
    form: &NewLocationForm,
    organization_id: Option<Uuid>,
    correlation_id: Uuid,
) -> LocationCommandResult {
    // 1. Validate form fields using accumulator pattern
    let mut acc = ValidationAccumulator::new();

    // Required: name
    if form.name.trim().is_empty() {
        acc.add(ValidationError::required("name"));
    }

    // Required for physical locations: address fields
    if form.street.trim().is_empty() {
        acc.add(ValidationError::required("street"));
    }

    if form.city.trim().is_empty() {
        acc.add(ValidationError::required("city"));
    }

    if form.region.trim().is_empty() {
        acc.add(ValidationError::required("region"));
    }

    if form.country.trim().is_empty() {
        acc.add(ValidationError::required("country"));
    }

    if form.postal.trim().is_empty() {
        acc.add(ValidationError::required("postal"));
    }

    // 2. Build validated address string
    let address = format!(
        "{}, {}, {} {}, {}",
        form.street.trim(),
        form.city.trim(),
        form.region.trim(),
        form.postal.trim(),
        form.country.trim()
    );

    // 3. Determine location type
    let location_type = form
        .location_type
        .as_ref()
        .map(|lt| format!("{:?}", lt))
        .unwrap_or_else(|| "Physical".to_string());

    // 4. Create command with validated data
    let command_id = Uuid::now_v7();
    let location_id = Uuid::now_v7();

    acc.into_result(CreateLocation {
        command_id,
        location_id,
        name: form.name.trim().to_string(),
        location_type,
        address: Some(address),
        coordinates: None, // Could be geocoded later
        organization_id,
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
    })
}

/// Create a location command with a specific location ID
///
/// Use this when you need to specify the location ID (e.g., for idempotent operations).
pub fn create_location_command_with_id(
    form: &NewLocationForm,
    location_id: Uuid,
    organization_id: Option<Uuid>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
) -> LocationCommandResult {
    // 1. Validate form fields using accumulator pattern
    let mut acc = ValidationAccumulator::new();

    if form.name.trim().is_empty() {
        acc.add(ValidationError::required("name"));
    }

    if form.street.trim().is_empty() {
        acc.add(ValidationError::required("street"));
    }

    if form.city.trim().is_empty() {
        acc.add(ValidationError::required("city"));
    }

    if form.region.trim().is_empty() {
        acc.add(ValidationError::required("region"));
    }

    if form.country.trim().is_empty() {
        acc.add(ValidationError::required("country"));
    }

    if form.postal.trim().is_empty() {
        acc.add(ValidationError::required("postal"));
    }

    // 2. Build validated address string
    let address = format!(
        "{}, {}, {} {}, {}",
        form.street.trim(),
        form.city.trim(),
        form.region.trim(),
        form.postal.trim(),
        form.country.trim()
    );

    // 3. Determine location type
    let location_type = form
        .location_type
        .as_ref()
        .map(|lt| format!("{:?}", lt))
        .unwrap_or_else(|| "Physical".to_string());

    // 4. Create command with validated data
    let command_id = Uuid::now_v7();

    acc.into_result(CreateLocation {
        command_id,
        location_id,
        name: form.name.trim().to_string(),
        location_type,
        address: Some(address),
        coordinates: None,
        organization_id,
        correlation_id,
        causation_id,
        timestamp: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_location_command_success() {
        let form = NewLocationForm::new()
            .with_name("HQ Office".to_string())
            .with_street("123 Main St".to_string())
            .with_city("Austin".to_string())
            .with_region("TX".to_string())
            .with_country("USA".to_string())
            .with_postal("78701".to_string());

        let org_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();
        let result = create_location_command(&form, Some(org_id), correlation_id);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "HQ Office");
        assert_eq!(cmd.organization_id, Some(org_id));
        assert_eq!(cmd.correlation_id, correlation_id);
        assert!(cmd.address.is_some());
        assert!(cmd.address.unwrap().contains("Austin"));
    }

    #[test]
    fn test_create_location_command_missing_required_fields() {
        let form = NewLocationForm::new()
            .with_name("".to_string()) // Empty
            .with_street("".to_string())
            .with_city("".to_string())
            .with_region("".to_string())
            .with_country("".to_string())
            .with_postal("".to_string());

        let result = create_location_command(&form, None, Uuid::now_v7());

        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should have 6 errors (name, street, city, region, country, postal)
        assert_eq!(errors.len(), 6);
    }

    #[test]
    fn test_create_location_command_partial_fields() {
        let form = NewLocationForm::new()
            .with_name("Test Location".to_string())
            .with_street("123 Test St".to_string())
            .with_city("".to_string()) // Missing city
            .with_region("TX".to_string())
            .with_country("USA".to_string())
            .with_postal("".to_string()); // Missing postal

        let result = create_location_command(&form, None, Uuid::now_v7());

        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should have 2 errors (city, postal)
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_create_location_command_with_id() {
        let form = NewLocationForm::new()
            .with_name("Branch Office".to_string())
            .with_street("456 Oak Ave".to_string())
            .with_city("Denver".to_string())
            .with_region("CO".to_string())
            .with_country("USA".to_string())
            .with_postal("80202".to_string());

        let location_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();
        let causation_id = Some(Uuid::now_v7());

        let result = create_location_command_with_id(
            &form,
            location_id,
            Some(org_id),
            correlation_id,
            causation_id,
        );

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.location_id, location_id);
        assert_eq!(cmd.causation_id, causation_id);
    }
}
