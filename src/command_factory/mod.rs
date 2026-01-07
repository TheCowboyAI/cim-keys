// Copyright (c) 2025 - Cowboy AI, LLC.

//! Command Factory Module
//!
//! This module provides factory functions for creating domain commands from
//! GUI ViewModels. It integrates with the ACL (Anti-Corruption Layer) to:
//!
//! 1. Validate ViewModels using pure validation functions
//! 2. Translate to domain ValueObjects on successful validation
//! 3. Create properly formed Commands with correlation/causation tracking
//!
//! ## Architecture Flow
//!
//! ```text
//! ViewModel (GUI) → validate() → ValidatedForm → translate() → ValueObjects → create_command() → Command
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::command_factory::{create_organization_command, create_person_command};
//! use crate::gui::view_state::{OrganizationForm, NewPersonForm};
//!
//! // Create organization command from GUI form
//! let form = OrganizationForm::new()
//!     .with_name("Cowboy AI".to_string())
//!     .with_domain("cowboyai.dev".to_string())
//!     .with_admin_email("admin@cowboyai.dev".to_string());
//!
//! match create_organization_command(&form, correlation_id) {
//!     Ok(command) => {
//!         // Command is ready to be sent to aggregate
//!         aggregate.handle(command)?;
//!     }
//!     Err(errors) => {
//!         // Return validation errors to GUI for user feedback
//!         for error in errors.iter() {
//!             show_error(error.field, error.message);
//!         }
//!     }
//! }
//! ```
//!
//! ## Key Properties
//!
//! - **Pure Functions**: All factory functions are pure (no side effects)
//! - **Validation First**: Commands only created after validation passes
//! - **Error Accumulation**: All validation errors returned, not just first
//! - **Correlation Tracking**: Commands include correlation/causation IDs
//! - **UUID v7**: Entity IDs use time-ordered UUIDs

pub mod cid_support;
pub mod organization;
pub mod person;

// Re-export factory functions
pub use organization::{
    create_organization_command, create_organizational_unit_command, OrganizationCommandResult,
    OrgUnitCommandResult,
};
pub use person::{create_person_command, PersonCommandResult};

// Re-export error types from ACL
pub use crate::acl::{NonEmptyVec, ValidationError};

// Re-export CID support types
pub use cid_support::{generate_command_cid, commands_equal, CommandWithCid, ContentAddressable};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::view_state::{NewPersonForm, OrganizationForm};
    use crate::domain::KeyOwnerRole;
    use uuid::Uuid;

    #[test]
    fn test_organization_command_success() {
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
    }

    #[test]
    fn test_organization_command_validation_failure() {
        let form = OrganizationForm::new()
            .with_name("".to_string()) // Empty name - invalid
            .with_domain("not-a-domain".to_string()) // No TLD - invalid
            .with_admin_email("not-an-email".to_string()); // Invalid email

        let correlation_id = Uuid::now_v7();
        let result = create_organization_command(&form, correlation_id);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should have multiple validation errors
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_person_command_success() {
        let form = NewPersonForm::new()
            .with_name("Alice Smith".to_string())
            .with_email("alice@example.com".to_string())
            .with_role(KeyOwnerRole::Developer);

        let correlation_id = Uuid::now_v7();
        let organization_id = Some(Uuid::now_v7());
        let result = create_person_command(&form, organization_id, correlation_id);

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.name, "Alice Smith");
        assert_eq!(cmd.email, "alice@example.com");
        assert_eq!(cmd.correlation_id, correlation_id);
    }

    #[test]
    fn test_person_command_validation_failure() {
        let form = NewPersonForm::new()
            .with_name("".to_string()) // Empty - invalid
            .with_email("bad-email".to_string()); // No @ - invalid

        let correlation_id = Uuid::now_v7();
        let result = create_person_command(&form, None, correlation_id);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
    }
}
