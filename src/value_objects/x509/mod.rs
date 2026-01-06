// Copyright (c) 2025 - Cowboy AI, LLC.

//! X.509 Certificate Value Objects
//!
//! RFC 5280 compliant value objects for X.509 certificate components.
//! These typed value objects replace loose strings in certificate events
//! with properly validated, RFC-compliant types.
//!
//! ## Modules
//!
//! - `subject_name` - Distinguished Name (DN) components
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cim_keys::value_objects::x509::*;
//!
//! let cn = CommonName::new("Alice Smith")?;
//! let org = OrganizationName::new("Cowboy AI, LLC")?;
//! let country = CountryCode::new("US")?;
//!
//! let subject = SubjectName::new(cn)
//!     .with_organization(org)
//!     .with_country(country);
//!
//! // Convert to RFC 4514 string
//! let dn_string = subject.to_rfc4514();
//! // "CN=Alice Smith,O=Cowboy AI\\, LLC,C=US"
//! ```

pub mod subject_name;

// Re-export all types
pub use subject_name::{
    CommonName,
    CountryCode,
    EmailAddress,
    LocalityName,
    OrganizationalUnitName,
    OrganizationName,
    StateName,
    SubjectName,
    SubjectNameError,
};
