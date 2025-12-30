// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Bounded Context Module
//!
//! This module organizes the NATS bounded context with proper separation:
//! - **Entities**: NATS hierarchy types (Operator, Account, User)
//! - **Subjects**: Type-safe NATS subject naming algebra
//!
//! ## Subject Algebra
//!
//! NATS subjects follow semantic naming: `organization.unit.entity.operation`
//!
//! ```ignore
//! use cim_keys::domain::nats::subjects;
//!
//! let subject = subjects::keys::certificate_generate("cowboyai", "root");
//! // => "cowboyai.security.keys.certificate.generate.root"
//! ```

pub mod entities;
pub mod subjects;

// Re-export entity types for backward compatibility
pub use entities::*;

// Re-export subject algebra at module level
pub use subjects::{
    Subject,
    SubjectBuilder,
    SubjectToken,
    SubjectError,
    PermissionSet,
    // Factory modules
    organization as org_subjects,
    keys as key_subjects,
    infrastructure as infra_subjects,
    audit as audit_subjects,
    services as service_subjects,
};
