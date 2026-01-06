// Copyright (c) 2025 - Cowboy AI, LLC.

//! Common Value Objects
//!
//! General-purpose value objects used across multiple domains.
//! These provide validated, type-safe representations of common data types.
//!
//! ## Modules
//!
//! - `email` - RFC 5321/5322 compliant email addresses
//! - `uri` - RFC 3986 compliant URIs
//! - `person_name` - Structured person name components
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::value_objects::common::*;
//!
//! let email = Email::new("user@example.com")?;
//! let uri = Uri::new("https://api.example.com/v1")?;
//! let name = PersonName::from_display("Alice Smith")?;
//! ```

pub mod email;
pub mod person_name;
pub mod uri;

// Re-export all types
pub use email::{Email, EmailError};
pub use person_name::{
    FamilyName, GivenName, MiddleName, NamePrefix, NameSuffix, PersonName, PersonNameError,
};
pub use uri::{Uri, UriError};
