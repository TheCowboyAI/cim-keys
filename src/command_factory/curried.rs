// Copyright (c) 2025 - Cowboy AI, LLC.

//! Curried Command Factories (True FP)
//!
//! This module provides curried factory functions following functional programming
//! principles. Each function returns a function, enabling:
//!
//! - **Partial Application**: Fix some arguments, pass others later
//! - **Composition**: Chain factories in pipelines
//! - **Point-Free Style**: Build command creators without explicit data
//!
//! ## Currying Pattern
//!
//! Instead of:
//! ```ignore
//! fn create(a: A, b: B, c: C) -> Result<Command, Error>  // OOP
//! ```
//!
//! We use:
//! ```ignore
//! fn create(a: A) -> Box<dyn Fn(B) -> Box<dyn Fn(C) -> Result<Command, Error>>>  // FP
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::command_factory::curried::person;
//!
//! // Partial application - fix correlation_id
//! let with_correlation = person::create(correlation_id);
//!
//! // Fix org_id
//! let for_org = with_correlation(Some(org_id));
//!
//! // Finally apply form
//! let command = for_org(&form)?;
//!
//! // Or compose in one line
//! let command = person::create(correlation_id)(Some(org_id))(&form)?;
//! ```

use chrono::Utc;
use uuid::Uuid;

use crate::acl::{NonEmptyVec, ValidationError};
use crate::acl::error::ValidationAccumulator;
use crate::commands::organization::{
    CreatePerson, CreateOrganization, CreateOrganizationalUnit,
    CreateLocation, CreateServiceAccount,
};
use crate::gui::view_state::{
    NewPersonForm, OrganizationForm, NewOrgUnitForm,
    NewLocationForm, NewServiceAccountForm,
};

// Type aliases for curried function types
pub type PersonResult = Result<CreatePerson, NonEmptyVec<ValidationError>>;
pub type OrganizationResult = Result<CreateOrganization, NonEmptyVec<ValidationError>>;
pub type OrgUnitResult = Result<CreateOrganizationalUnit, NonEmptyVec<ValidationError>>;
pub type LocationResult = Result<CreateLocation, NonEmptyVec<ValidationError>>;
pub type ServiceAccountResult = Result<CreateServiceAccount, NonEmptyVec<ValidationError>>;

// ============================================================================
// PERSON COMMAND FACTORY (CURRIED)
// ============================================================================

pub mod person {
    use super::*;

    /// Type alias for the partially applied person factory
    pub type WithOrg = Box<dyn Fn(&NewPersonForm) -> PersonResult>;
    pub type WithCorrelation = Box<dyn Fn(Option<Uuid>) -> WithOrg>;

    /// Curried person command factory
    ///
    /// ## Type Signature (Haskell-style)
    /// ```text
    /// create :: Uuid -> (Maybe Uuid -> (NewPersonForm -> Result CreatePerson Error))
    /// ```
    ///
    /// ## Usage
    /// ```rust,ignore
    /// let cmd = person::create(correlation_id)(org_id)(&form)?;
    /// ```
    pub fn create(correlation_id: Uuid) -> WithCorrelation {
        Box::new(move |organization_id: Option<Uuid>| -> WithOrg {
            Box::new(move |form: &NewPersonForm| -> PersonResult {
                validate_and_create_person(form, organization_id, correlation_id)
            })
        })
    }

    fn validate_and_create_person(
        form: &NewPersonForm,
        organization_id: Option<Uuid>,
        correlation_id: Uuid,
    ) -> PersonResult {
        let mut acc = ValidationAccumulator::new();

        if form.name.trim().is_empty() {
            acc.add(ValidationError::required("name"));
        }

        if form.email.trim().is_empty() {
            acc.add(ValidationError::required("email"));
        } else if !form.email.contains('@') {
            acc.add(ValidationError::new("email", "Invalid email format"));
        }

        let command_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();

        acc.into_result(CreatePerson {
            command_id,
            person_id,
            name: form.name.trim().to_string(),
            email: form.email.trim().to_string(),
            title: None,
            department: None,
            organization_id,
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        })
    }
}

// ============================================================================
// ORGANIZATION COMMAND FACTORY (CURRIED)
// ============================================================================

pub mod organization {
    use super::*;
    use crate::acl::validate_organization_form;

    /// Type alias for the partially applied organization factory
    pub type WithCorrelation = Box<dyn Fn(&OrganizationForm) -> OrganizationResult>;

    /// Curried organization command factory
    ///
    /// ## Type Signature
    /// ```text
    /// create :: Uuid -> (OrganizationForm -> Result CreateOrganization Error)
    /// ```
    pub fn create(correlation_id: Uuid) -> WithCorrelation {
        Box::new(move |form: &OrganizationForm| -> OrganizationResult {
            let validated = validate_organization_form(form)?;

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
        })
    }
}

// ============================================================================
// ORG UNIT COMMAND FACTORY (CURRIED)
// ============================================================================

pub mod org_unit {
    use super::*;

    /// Type alias for the partially applied org unit factory
    pub type WithCorrelation = Box<dyn Fn(&NewOrgUnitForm) -> OrgUnitResult>;

    /// Curried org unit command factory
    ///
    /// ## Type Signature
    /// ```text
    /// create :: Uuid -> (NewOrgUnitForm -> Result CreateOrganizationalUnit Error)
    /// ```
    pub fn create(correlation_id: Uuid) -> WithCorrelation {
        Box::new(move |form: &NewOrgUnitForm| -> OrgUnitResult {
            let mut acc = ValidationAccumulator::new();

            if form.name.trim().is_empty() {
                acc.add(ValidationError::required("name"));
            }

            let parent_id = form.parent.as_ref().and_then(|p| {
                let trimmed = p.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Uuid::parse_str(trimmed).ok()
                }
            });

            let command_id = Uuid::now_v7();
            let unit_id = Uuid::now_v7();

            acc.into_result(CreateOrganizationalUnit {
                command_id,
                unit_id,
                name: form.name.trim().to_string(),
                parent_id,
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            })
        })
    }
}

// ============================================================================
// LOCATION COMMAND FACTORY (CURRIED)
// ============================================================================

pub mod location {
    use super::*;

    /// Type aliases for the partially applied location factory
    pub type WithOrg = Box<dyn Fn(&NewLocationForm) -> LocationResult>;
    pub type WithCorrelation = Box<dyn Fn(Option<Uuid>) -> WithOrg>;

    /// Curried location command factory
    ///
    /// ## Type Signature
    /// ```text
    /// create :: Uuid -> (Maybe Uuid -> (NewLocationForm -> Result CreateLocation Error))
    /// ```
    pub fn create(correlation_id: Uuid) -> WithCorrelation {
        Box::new(move |organization_id: Option<Uuid>| -> WithOrg {
            Box::new(move |form: &NewLocationForm| -> LocationResult {
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

                let address = format!(
                    "{}, {}, {} {}, {}",
                    form.street.trim(),
                    form.city.trim(),
                    form.region.trim(),
                    form.postal.trim(),
                    form.country.trim()
                );

                let location_type = form
                    .location_type
                    .as_ref()
                    .map(|lt| format!("{:?}", lt))
                    .unwrap_or_else(|| "Physical".to_string());

                let command_id = Uuid::now_v7();
                let location_id = Uuid::now_v7();

                acc.into_result(CreateLocation {
                    command_id,
                    location_id,
                    name: form.name.trim().to_string(),
                    location_type,
                    address: Some(address),
                    coordinates: None,
                    organization_id,
                    correlation_id,
                    causation_id: None,
                    timestamp: Utc::now(),
                })
            })
        })
    }
}

// ============================================================================
// SERVICE ACCOUNT COMMAND FACTORY (CURRIED)
// ============================================================================

pub mod service_account {
    use super::*;

    /// Curried service account command factory
    ///
    /// ## Type Signature
    /// ```text
    /// create :: Uuid -> NewServiceAccountForm -> Result CreateServiceAccount Error
    /// ```
    pub fn create(correlation_id: Uuid)
        -> impl Fn(&NewServiceAccountForm)
        -> Result<CreateServiceAccount, NonEmptyVec<ValidationError>>
    {
        move |form| {
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
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::KeyOwnerRole;

    #[test]
    fn test_person_curried_full_application() {
        let form = NewPersonForm::new()
            .with_name("Alice".to_string())
            .with_email("alice@example.com".to_string());

        let correlation_id = Uuid::now_v7();
        let org_id = Some(Uuid::now_v7());

        // Full curried application
        let result = person::create(correlation_id)(org_id)(&form);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "Alice");
        assert_eq!(cmd.correlation_id, correlation_id);
    }

    #[test]
    fn test_person_curried_partial_application() {
        let correlation_id = Uuid::now_v7();
        let org_id = Some(Uuid::now_v7());

        // Partial application - fix correlation_id
        let with_correlation = person::create(correlation_id);

        // Partial application - fix org_id
        let for_org = with_correlation(org_id);

        // Now we have a reusable function for this org
        let form1 = NewPersonForm::new()
            .with_name("Bob".to_string())
            .with_email("bob@example.com".to_string());

        let form2 = NewPersonForm::new()
            .with_name("Carol".to_string())
            .with_email("carol@example.com".to_string());

        // Apply to multiple forms with same context
        let cmd1 = for_org(&form1).unwrap();
        let cmd2 = for_org(&form2).unwrap();

        assert_eq!(cmd1.name, "Bob");
        assert_eq!(cmd2.name, "Carol");
        assert_eq!(cmd1.correlation_id, cmd2.correlation_id);
        assert_eq!(cmd1.organization_id, cmd2.organization_id);
    }

    #[test]
    fn test_person_curried_validation_errors() {
        let form = NewPersonForm::new()
            .with_name("".to_string())
            .with_email("invalid".to_string());

        let result = person::create(Uuid::now_v7())(None)(&form);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_organization_curried() {
        let form = OrganizationForm::new()
            .with_name("Cowboy AI".to_string())
            .with_domain("cowboyai.dev".to_string())
            .with_admin_email("admin@cowboyai.dev".to_string());

        let correlation_id = Uuid::now_v7();

        let result = organization::create(correlation_id)(&form);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "Cowboy AI");
    }

    #[test]
    fn test_location_curried() {
        let form = NewLocationForm::new()
            .with_name("HQ".to_string())
            .with_street("123 Main St".to_string())
            .with_city("Austin".to_string())
            .with_region("TX".to_string())
            .with_country("USA".to_string())
            .with_postal("78701".to_string());

        let correlation_id = Uuid::now_v7();
        let org_id = Some(Uuid::now_v7());

        let result = location::create(correlation_id)(org_id)(&form);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "HQ");
    }

    #[test]
    fn test_service_account_curried() {
        let form = NewServiceAccountForm::new()
            .with_name("backup-agent".to_string())
            .with_purpose("Automated backups".to_string())
            .with_owning_unit(Uuid::now_v7())
            .with_responsible_person(Uuid::now_v7());

        let result = service_account::create(Uuid::now_v7())(&form);

        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_creation_with_partial_application() {
        let correlation_id = Uuid::now_v7();
        let org_id = Some(Uuid::now_v7());

        // Create a reusable person factory for this batch
        let create_person_for_org = person::create(correlation_id)(org_id);

        let people = vec![
            ("Alice", "alice@example.com"),
            ("Bob", "bob@example.com"),
            ("Carol", "carol@example.com"),
        ];

        let commands: Vec<_> = people
            .iter()
            .map(|(name, email)| {
                let form = NewPersonForm::new()
                    .with_name(name.to_string())
                    .with_email(email.to_string());
                create_person_for_org(&form)
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(commands.len(), 3);
        // All share same correlation_id and org_id
        assert!(commands.iter().all(|c| c.correlation_id == correlation_id));
        assert!(commands.iter().all(|c| c.organization_id == org_id));
    }
}
