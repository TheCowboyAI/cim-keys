// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Subject Naming Algebra
//!
//! This module provides type-safe NATS subject construction following
//! the semantic naming pattern: `organization.unit.entity.operation`
//!
//! ## Subject Algebra Properties
//!
//! NATS subjects form a free monoid under concatenation with identity "".
//! The algebra supports:
//!
//! - **Concatenation**: `a.b` (associative: `(a.b).c = a.(b.c)`)
//! - **Identity**: Empty subject (left/right identity for concatenation)
//! - **Wildcards**: `*` (single token), `>` (multi-token suffix)
//!
//! ## Semantic Subject Pattern
//!
//! ```text
//! organization.unit.entity.operation
//!      │         │      │       │
//!      │         │      │       └─ Command/Event/Query verb
//!      │         │      └─────── Domain entity type
//!      │         └─────────── Organizational unit or context
//!      └───────────────── Organization or domain root
//! ```
//!
//! ## Examples
//!
//! ```ignore
//! // Type-safe subject construction
//! let subject = Subject::new("cowboyai")
//!     .unit("security")
//!     .entity("keys")
//!     .operation("certificate.generate.root");
//!
//! // Yields: "cowboyai.security.keys.certificate.generate.root"
//!
//! // Wildcard subscription
//! let subscription = Subject::new("cowboyai")
//!     .unit("security")
//!     .wildcard_suffix();
//!
//! // Yields: "cowboyai.security.>"
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A NATS subject following the semantic naming algebra.
///
/// Subjects are immutable; all operations return new instances.
/// This enforces functional purity and enables safe sharing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subject {
    /// The subject tokens (stored without dots)
    tokens: Vec<SubjectToken>,
}

/// A single token in a NATS subject.
///
/// Tokens can be:
/// - Literal values (alphanumeric strings)
/// - Single-level wildcard (*)
/// - Multi-level suffix wildcard (>)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectToken {
    /// A literal token (must be non-empty, no dots or spaces)
    Literal(String),
    /// Single-level wildcard (*) - matches exactly one token
    SingleWildcard,
    /// Multi-level suffix wildcard (>) - matches one or more tokens
    /// Must be the last token if present
    SuffixWildcard,
}

impl SubjectToken {
    /// Create a new literal token, validating the input.
    pub fn literal(value: impl Into<String>) -> Result<Self, SubjectError> {
        let value = value.into();
        if value.is_empty() {
            return Err(SubjectError::EmptyToken);
        }
        if value.contains('.') {
            return Err(SubjectError::DotInToken(value));
        }
        if value.contains(' ') {
            return Err(SubjectError::SpaceInToken(value));
        }
        if value == "*" {
            return Ok(Self::SingleWildcard);
        }
        if value == ">" {
            return Ok(Self::SuffixWildcard);
        }
        Ok(Self::Literal(value))
    }

    /// Check if this is a wildcard token
    pub fn is_wildcard(&self) -> bool {
        matches!(self, Self::SingleWildcard | Self::SuffixWildcard)
    }

    /// Check if this is the suffix wildcard
    pub fn is_suffix(&self) -> bool {
        matches!(self, Self::SuffixWildcard)
    }
}

impl fmt::Display for SubjectToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(s) => write!(f, "{}", s),
            Self::SingleWildcard => write!(f, "*"),
            Self::SuffixWildcard => write!(f, ">"),
        }
    }
}

impl Subject {
    /// Create a new subject with an organization root.
    pub fn new(organization: impl Into<String>) -> Self {
        let org = organization.into();
        Self {
            tokens: vec![SubjectToken::Literal(org)],
        }
    }

    /// Create an empty subject (identity element).
    pub fn empty() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Parse a subject from a dot-delimited string.
    pub fn parse(subject: &str) -> Result<Self, SubjectError> {
        if subject.is_empty() {
            return Ok(Self::empty());
        }

        let parts: Vec<&str> = subject.split('.').collect();
        let mut tokens = Vec::with_capacity(parts.len());

        for (i, part) in parts.iter().enumerate() {
            let token = SubjectToken::literal(*part)?;

            // Suffix wildcard must be last
            if token.is_suffix() && i != parts.len() - 1 {
                return Err(SubjectError::SuffixNotLast);
            }

            tokens.push(token);
        }

        Ok(Self { tokens })
    }

    /// Add a unit (department/context) to the subject.
    pub fn unit(self, unit: impl Into<String>) -> Self {
        self.append_token(unit.into())
    }

    /// Add an entity type to the subject.
    pub fn entity(self, entity: impl Into<String>) -> Self {
        self.append_token(entity.into())
    }

    /// Add an operation to the subject.
    ///
    /// Operations can be dot-delimited (e.g., "certificate.generate.root")
    /// and will be split into individual tokens.
    pub fn operation(self, operation: impl Into<String>) -> Self {
        let op = operation.into();
        let parts: Vec<&str> = op.split('.').collect();
        let mut result = self;
        for part in parts {
            result = result.append_token(part.to_string());
        }
        result
    }

    /// Append a single-level wildcard (*).
    pub fn wildcard(self) -> Self {
        Self {
            tokens: {
                let mut t = self.tokens;
                t.push(SubjectToken::SingleWildcard);
                t
            },
        }
    }

    /// Append a multi-level suffix wildcard (>).
    ///
    /// This should be the last token added.
    pub fn wildcard_suffix(self) -> Self {
        Self {
            tokens: {
                let mut t = self.tokens;
                t.push(SubjectToken::SuffixWildcard);
                t
            },
        }
    }

    /// Concatenate two subjects (monoid operation).
    pub fn concat(self, other: Self) -> Self {
        Self {
            tokens: {
                let mut t = self.tokens;
                t.extend(other.tokens);
                t
            },
        }
    }

    /// Get the subject as a dot-delimited string.
    pub fn as_str(&self) -> String {
        self.tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Get the number of tokens in the subject.
    pub fn depth(&self) -> usize {
        self.tokens.len()
    }

    /// Check if this is a subscribe pattern (contains wildcards).
    pub fn is_pattern(&self) -> bool {
        self.tokens.iter().any(|t| t.is_wildcard())
    }

    /// Check if this is a publish subject (no wildcards).
    pub fn is_publishable(&self) -> bool {
        !self.is_pattern()
    }

    /// Check if this subject matches a pattern.
    ///
    /// The pattern may contain wildcards, self should be a literal subject.
    pub fn matches(&self, pattern: &Subject) -> bool {
        if self.is_pattern() {
            return false; // Can only match patterns against literal subjects
        }

        let mut self_idx = 0;
        let mut pattern_idx = 0;

        while pattern_idx < pattern.tokens.len() {
            match &pattern.tokens[pattern_idx] {
                SubjectToken::SuffixWildcard => {
                    // > matches everything remaining
                    return self_idx < self.tokens.len();
                }
                SubjectToken::SingleWildcard => {
                    // * matches exactly one token
                    if self_idx >= self.tokens.len() {
                        return false;
                    }
                    self_idx += 1;
                    pattern_idx += 1;
                }
                SubjectToken::Literal(pat) => {
                    if self_idx >= self.tokens.len() {
                        return false;
                    }
                    match &self.tokens[self_idx] {
                        SubjectToken::Literal(s) if s == pat => {
                            self_idx += 1;
                            pattern_idx += 1;
                        }
                        _ => return false,
                    }
                }
            }
        }

        // Both should be fully consumed for a match
        self_idx == self.tokens.len()
    }

    fn append_token(self, token: String) -> Self {
        Self {
            tokens: {
                let mut t = self.tokens;
                t.push(SubjectToken::Literal(token));
                t
            },
        }
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Builder for constructing domain-specific subjects.
///
/// This provides a fluent API for building subjects that map
/// to CIM domain concepts.
#[derive(Debug, Clone)]
pub struct SubjectBuilder {
    organization: String,
    unit: Option<String>,
    entity: Option<String>,
    operation: Option<String>,
}

impl SubjectBuilder {
    /// Create a new subject builder for an organization.
    pub fn new(organization: impl Into<String>) -> Self {
        Self {
            organization: organization.into(),
            unit: None,
            entity: None,
            operation: None,
        }
    }

    /// Set the organizational unit.
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the entity type.
    pub fn entity(mut self, entity: impl Into<String>) -> Self {
        self.entity = Some(entity.into());
        self
    }

    /// Set the operation.
    pub fn operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Build the subject.
    pub fn build(self) -> Subject {
        let mut subject = Subject::new(self.organization);

        if let Some(unit) = self.unit {
            subject = subject.unit(unit);
        }
        if let Some(entity) = self.entity {
            subject = subject.entity(entity);
        }
        if let Some(operation) = self.operation {
            subject = subject.operation(operation);
        }

        subject
    }
}

// ============================================================================
// DOMAIN-SPECIFIC SUBJECT FACTORIES
// ============================================================================

/// Create subjects for organizational structure events.
pub mod organization {
    use super::*;

    /// Subject for organization events.
    pub fn events(org_name: &str) -> Subject {
        Subject::new(org_name).unit("events").entity("organization")
    }

    /// Subject for person events in an organization.
    pub fn person_events(org_name: &str) -> Subject {
        Subject::new(org_name).unit("events").entity("person")
    }

    /// Subject for unit events in an organization.
    pub fn unit_events(org_name: &str) -> Subject {
        Subject::new(org_name).unit("events").entity("unit")
    }

    /// Subscribe to all organization events.
    pub fn all_events(org_name: &str) -> Subject {
        Subject::new(org_name).unit("events").wildcard_suffix()
    }
}

/// Create subjects for key management events.
pub mod keys {
    use super::*;

    /// Subject for certificate generation events.
    pub fn certificate_generate(org_name: &str, cert_type: &str) -> Subject {
        Subject::new(org_name)
            .unit("security")
            .entity("keys")
            .operation(format!("certificate.generate.{}", cert_type))
    }

    /// Subject for key revocation events.
    pub fn key_revoked(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("security")
            .entity("keys")
            .operation("revoked")
    }

    /// Subject for key rotation events.
    pub fn key_rotated(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("security")
            .entity("keys")
            .operation("rotated")
    }

    /// Subscribe to all key events.
    pub fn all_events(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("security")
            .entity("keys")
            .wildcard_suffix()
    }
}

/// Create subjects for NATS infrastructure events.
pub mod infrastructure {
    use super::*;

    /// Subject for NATS operator events.
    pub fn operator_events(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("infrastructure")
            .entity("nats")
            .operation("operator")
    }

    /// Subject for NATS account events.
    pub fn account_events(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("infrastructure")
            .entity("nats")
            .operation("account")
    }

    /// Subject for NATS user events.
    pub fn user_events(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("infrastructure")
            .entity("nats")
            .operation("user")
    }

    /// Subscribe to all infrastructure events.
    pub fn all_events(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("infrastructure")
            .wildcard_suffix()
    }
}

/// Create subjects for audit events.
pub mod audit {
    use super::*;

    /// Subject for security audit events.
    pub fn security(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("security")
            .entity("audit")
    }

    /// Subject for specific audit event types.
    pub fn event(org_name: &str, event_type: &str) -> Subject {
        Subject::new(org_name)
            .unit("security")
            .entity("audit")
            .operation(event_type)
    }

    /// Subscribe to all audit events.
    pub fn all_events(org_name: &str) -> Subject {
        Subject::new(org_name)
            .unit("security")
            .entity("audit")
            .wildcard_suffix()
    }
}

/// Create subjects for request/reply services.
pub mod services {
    use super::*;

    /// Subject for a service endpoint.
    pub fn endpoint(org_name: &str, service: &str, method: &str) -> Subject {
        Subject::new(org_name)
            .unit("services")
            .entity(service)
            .operation(method)
    }

    /// Subject for service health checks.
    pub fn health(org_name: &str, service: &str) -> Subject {
        Subject::new(org_name)
            .unit("services")
            .entity(service)
            .operation("health")
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Errors that can occur during subject construction.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SubjectError {
    #[error("Subject token cannot be empty")]
    EmptyToken,

    #[error("Subject token cannot contain dots: {0}")]
    DotInToken(String),

    #[error("Subject token cannot contain spaces: {0}")]
    SpaceInToken(String),

    #[error("Suffix wildcard (>) must be the last token")]
    SuffixNotLast,

    #[error("Invalid subject format: {0}")]
    InvalidFormat(String),
}

// ============================================================================
// PERMISSION HELPERS
// ============================================================================

/// Create publish permission for a subject.
pub fn publish_permission(subject: &Subject) -> String {
    subject.as_str()
}

/// Create subscribe permission for a subject pattern.
pub fn subscribe_permission(pattern: &Subject) -> String {
    pattern.as_str()
}

/// Create permission set for a user based on their role.
#[derive(Debug, Clone)]
pub struct PermissionSet {
    pub publish: Vec<Subject>,
    pub subscribe: Vec<Subject>,
}

impl PermissionSet {
    pub fn new() -> Self {
        Self {
            publish: Vec::new(),
            subscribe: Vec::new(),
        }
    }

    pub fn with_publish(mut self, subject: Subject) -> Self {
        self.publish.push(subject);
        self
    }

    pub fn with_subscribe(mut self, pattern: Subject) -> Self {
        self.subscribe.push(pattern);
        self
    }

    /// Convert to string vectors for NATS permissions.
    pub fn to_strings(&self) -> (Vec<String>, Vec<String>) {
        (
            self.publish.iter().map(|s| s.as_str()).collect(),
            self.subscribe.iter().map(|s| s.as_str()).collect(),
        )
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Create permissions for a standard user.
pub fn user_permissions(org_name: &str, _user_id: Uuid) -> PermissionSet {
    PermissionSet::new()
        .with_subscribe(organization::all_events(org_name))
        .with_publish(audit::security(org_name))
}

/// Create permissions for an administrator.
pub fn admin_permissions(org_name: &str) -> PermissionSet {
    PermissionSet::new()
        .with_subscribe(Subject::new(org_name).wildcard_suffix())
        .with_publish(Subject::new(org_name).wildcard_suffix())
}

/// Create permissions for a service account.
pub fn service_permissions(org_name: &str, service_name: &str) -> PermissionSet {
    PermissionSet::new()
        .with_subscribe(services::endpoint(org_name, service_name, "*"))
        .with_publish(services::endpoint(org_name, service_name, "*"))
        .with_publish(audit::event(org_name, service_name))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_construction() {
        let subject = Subject::new("cowboyai")
            .unit("security")
            .entity("keys")
            .operation("certificate.generate.root");

        assert_eq!(
            subject.as_str(),
            "cowboyai.security.keys.certificate.generate.root"
        );
    }

    #[test]
    fn test_subject_parse() {
        let subject = Subject::parse("cowboyai.security.keys").unwrap();
        assert_eq!(subject.depth(), 3);
        assert_eq!(subject.as_str(), "cowboyai.security.keys");
    }

    #[test]
    fn test_wildcard_subjects() {
        let pattern = Subject::new("cowboyai").unit("security").wildcard_suffix();

        assert_eq!(pattern.as_str(), "cowboyai.security.>");
        assert!(pattern.is_pattern());
        assert!(!pattern.is_publishable());
    }

    #[test]
    fn test_subject_matching() {
        let subject = Subject::parse("cowboyai.security.keys.rotated").unwrap();
        let pattern = Subject::parse("cowboyai.security.>").unwrap();

        assert!(subject.matches(&pattern));

        let specific_pattern = Subject::parse("cowboyai.security.keys.*").unwrap();
        assert!(subject.matches(&specific_pattern));

        let wrong_pattern = Subject::parse("cowboyai.infrastructure.>").unwrap();
        assert!(!subject.matches(&wrong_pattern));
    }

    #[test]
    fn test_subject_builder() {
        let subject = SubjectBuilder::new("cowboyai")
            .unit("events")
            .entity("person")
            .operation("created")
            .build();

        assert_eq!(subject.as_str(), "cowboyai.events.person.created");
    }

    #[test]
    fn test_domain_subject_factories() {
        let cert_subject = keys::certificate_generate("cowboyai", "root");
        assert_eq!(
            cert_subject.as_str(),
            "cowboyai.security.keys.certificate.generate.root"
        );

        let audit_subject = audit::event("cowboyai", "key.revoked");
        assert_eq!(audit_subject.as_str(), "cowboyai.security.audit.key.revoked");
    }

    #[test]
    fn test_subject_concat() {
        let a = Subject::new("cowboyai");
        let b = Subject::parse("security.keys").unwrap();
        let combined = a.concat(b);

        assert_eq!(combined.as_str(), "cowboyai.security.keys");
    }

    #[test]
    fn test_empty_subject_identity() {
        let subject = Subject::new("cowboyai");
        let empty = Subject::empty();

        // Empty is left identity
        let result = empty.clone().concat(subject.clone());
        assert_eq!(result.as_str(), subject.as_str());

        // Empty is right identity
        let result2 = subject.concat(empty);
        assert_eq!(result2.as_str(), "cowboyai");
    }

    #[test]
    fn test_permission_set() {
        let perms = admin_permissions("cowboyai");
        let (pub_perms, sub_perms) = perms.to_strings();

        assert!(pub_perms.contains(&"cowboyai.>".to_string()));
        assert!(sub_perms.contains(&"cowboyai.>".to_string()));
    }

    #[test]
    fn test_subject_token_validation() {
        assert!(SubjectToken::literal("valid").is_ok());
        assert!(SubjectToken::literal("with.dot").is_err());
        assert!(SubjectToken::literal("with space").is_err());
        assert!(SubjectToken::literal("").is_err());

        // Wildcards are parsed correctly
        let star = SubjectToken::literal("*").unwrap();
        assert!(star.is_wildcard());

        let gt = SubjectToken::literal(">").unwrap();
        assert!(gt.is_suffix());
    }

    #[test]
    fn test_suffix_wildcard_must_be_last() {
        let result = Subject::parse("cowboyai.>.security");
        assert!(result.is_err());
    }
}
