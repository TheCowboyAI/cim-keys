// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Subject Type - Pub/Sub Pattern Authorization
//!
//! Subject represents NATS messaging patterns for publish/subscribe authorization.
//! This is the second element of the categorical product `Role × Subject` that
//! forms the base capability in the fold: `Capability = fold(Policies, Role × Subject)`.
//!
//! # NATS Subject Patterns
//!
//! NATS uses dot-separated hierarchical subjects with wildcards:
//! - `*` - matches exactly one token
//! - `>` - matches one or more tokens (must be last)
//! - Literal - exact match
//!
//! Example: `organization.unit.entity.operation`
//!
//! # Mathematical Structure
//!
//! Subjects form a partially ordered set under pattern subsumption:
//! - `a ≤ b` iff pattern `b` matches all subjects that `a` matches
//! - `foo.bar` ≤ `foo.*` ≤ `foo.>` ≤ `>`
//!
//! The product `Role × Subject` is a valid categorical product because:
//! - Projections: π₁(r, s) = r, π₂(r, s) = s
//! - Universal property: for any f: X → Role, g: X → Subject,
//!   there exists unique ⟨f,g⟩: X → Role × Subject

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// SUBJECT TOKEN - Individual components of a subject pattern
// ============================================================================

/// A single token in a NATS subject pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectToken {
    /// Exact literal match
    Literal(String),
    /// Single-token wildcard (*)
    Single,
    /// Multi-token wildcard (>) - must be terminal
    Multi,
}

impl SubjectToken {
    /// Check if this token matches another token
    pub fn matches(&self, other: &str) -> bool {
        match self {
            SubjectToken::Literal(s) => s == other,
            SubjectToken::Single => true, // * matches any single token
            SubjectToken::Multi => true,  // > matches any token(s)
        }
    }

    /// Check if this token pattern subsumes another
    pub fn subsumes(&self, other: &SubjectToken) -> bool {
        match (self, other) {
            // Same token subsumes itself
            (a, b) if a == b => true,
            // Multi subsumes everything
            (SubjectToken::Multi, _) => true,
            // Single subsumes literals
            (SubjectToken::Single, SubjectToken::Literal(_)) => true,
            // Literal only subsumes itself
            _ => false,
        }
    }
}

impl fmt::Display for SubjectToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubjectToken::Literal(s) => write!(f, "{}", s),
            SubjectToken::Single => write!(f, "*"),
            SubjectToken::Multi => write!(f, ">"),
        }
    }
}

// ============================================================================
// SUBJECT PATTERN - Complete NATS subject pattern
// ============================================================================

/// A NATS subject pattern for pub/sub authorization
///
/// Subjects follow CIM semantic naming: `organization.unit.entity.operation`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubjectPattern {
    /// Ordered tokens forming the subject
    tokens: Vec<SubjectToken>,
}

impl SubjectPattern {
    /// Create from token list
    pub fn new(tokens: Vec<SubjectToken>) -> Self {
        Self { tokens }
    }

    /// Parse a subject string into a pattern
    pub fn parse(subject: &str) -> Result<Self, SubjectError> {
        if subject.is_empty() {
            return Err(SubjectError::Empty);
        }

        let mut tokens = Vec::new();
        let parts: Vec<&str> = subject.split('.').collect();

        for (i, part) in parts.iter().enumerate() {
            let token = match *part {
                "*" => SubjectToken::Single,
                ">" => {
                    // > must be the last token
                    if i != parts.len() - 1 {
                        return Err(SubjectError::MultiWildcardNotTerminal);
                    }
                    SubjectToken::Multi
                }
                "" => return Err(SubjectError::EmptyToken),
                s => SubjectToken::Literal(s.to_string()),
            };
            tokens.push(token);
        }

        Ok(Self { tokens })
    }

    /// Create a literal subject (no wildcards)
    pub fn literal(subject: &str) -> Result<Self, SubjectError> {
        let pattern = Self::parse(subject)?;
        if pattern.has_wildcards() {
            return Err(SubjectError::UnexpectedWildcard);
        }
        Ok(pattern)
    }

    /// Check if pattern contains any wildcards
    pub fn has_wildcards(&self) -> bool {
        self.tokens
            .iter()
            .any(|t| matches!(t, SubjectToken::Single | SubjectToken::Multi))
    }

    /// Check if this pattern matches a concrete subject
    pub fn matches(&self, subject: &str) -> bool {
        let parts: Vec<&str> = subject.split('.').collect();
        self.matches_parts(&parts)
    }

    fn matches_parts(&self, parts: &[&str]) -> bool {
        let mut token_idx = 0;
        let mut part_idx = 0;

        while token_idx < self.tokens.len() && part_idx < parts.len() {
            match &self.tokens[token_idx] {
                SubjectToken::Literal(s) => {
                    if s != parts[part_idx] {
                        return false;
                    }
                    token_idx += 1;
                    part_idx += 1;
                }
                SubjectToken::Single => {
                    token_idx += 1;
                    part_idx += 1;
                }
                SubjectToken::Multi => {
                    // > matches all remaining tokens
                    return true;
                }
            }
        }

        // Both must be exhausted (unless pattern ends with >)
        token_idx == self.tokens.len() && part_idx == parts.len()
    }

    /// Check if this pattern subsumes another (partial order ≤)
    ///
    /// Pattern A subsumes B if every subject matching B also matches A.
    /// Example: `foo.>` subsumes `foo.bar.*`
    pub fn subsumes(&self, other: &SubjectPattern) -> bool {
        // Handle multi-wildcard cases
        if let Some(SubjectToken::Multi) = self.tokens.last() {
            // If we end with >, check prefix
            if self.tokens.len() == 1 {
                return true; // > subsumes everything
            }
            let prefix_len = self.tokens.len() - 1;
            if other.tokens.len() < prefix_len {
                return false;
            }
            // Check that our prefix matches their prefix
            for i in 0..prefix_len {
                if !self.tokens[i].subsumes(&other.tokens[i]) {
                    return false;
                }
            }
            return true;
        }

        // Same length required for non-> patterns
        if self.tokens.len() != other.tokens.len() {
            return false;
        }

        // Token-by-token subsumption
        self.tokens
            .iter()
            .zip(other.tokens.iter())
            .all(|(a, b)| a.subsumes(b))
    }

    /// Get the number of tokens
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Check if empty (should never be valid)
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Get tokens as slice
    pub fn tokens(&self) -> &[SubjectToken] {
        &self.tokens
    }

    /// Convert to NATS subject string
    pub fn to_nats_string(&self) -> String {
        self.tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl fmt::Display for SubjectPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_nats_string())
    }
}

// ============================================================================
// SUBJECT - Complete authorization subject with pub/sub permissions
// ============================================================================

/// Subject authorization for NATS pub/sub
///
/// This is the second element of `Role × Subject` in the capability model.
/// It specifies what messaging patterns a role can access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    /// Patterns allowed for publishing
    pub publish: Vec<SubjectPattern>,
    /// Patterns allowed for subscribing
    pub subscribe: Vec<SubjectPattern>,
    /// Patterns explicitly denied (takes precedence)
    pub deny_publish: Vec<SubjectPattern>,
    /// Patterns explicitly denied for subscribe
    pub deny_subscribe: Vec<SubjectPattern>,
}

impl Subject {
    /// Create an empty subject (no permissions)
    pub fn new() -> Self {
        Self {
            publish: Vec::new(),
            subscribe: Vec::new(),
            deny_publish: Vec::new(),
            deny_subscribe: Vec::new(),
        }
    }

    /// Create subject with full access (>)
    pub fn full_access() -> Self {
        let all = SubjectPattern::new(vec![SubjectToken::Multi]);
        Self {
            publish: vec![all.clone()],
            subscribe: vec![all],
            deny_publish: Vec::new(),
            deny_subscribe: Vec::new(),
        }
    }

    /// Builder: add publish pattern
    pub fn with_publish(mut self, pattern: SubjectPattern) -> Self {
        self.publish.push(pattern);
        self
    }

    /// Builder: add subscribe pattern
    pub fn with_subscribe(mut self, pattern: SubjectPattern) -> Self {
        self.subscribe.push(pattern);
        self
    }

    /// Builder: add deny publish pattern
    pub fn with_deny_publish(mut self, pattern: SubjectPattern) -> Self {
        self.deny_publish.push(pattern);
        self
    }

    /// Builder: add deny subscribe pattern
    pub fn with_deny_subscribe(mut self, pattern: SubjectPattern) -> Self {
        self.deny_subscribe.push(pattern);
        self
    }

    /// Check if publishing to a subject is allowed
    pub fn can_publish(&self, subject: &str) -> bool {
        // Check deny first
        if self.deny_publish.iter().any(|p| p.matches(subject)) {
            return false;
        }
        // Check allow
        self.publish.iter().any(|p| p.matches(subject))
    }

    /// Check if subscribing to a subject is allowed
    pub fn can_subscribe(&self, subject: &str) -> bool {
        // Check deny first
        if self.deny_subscribe.iter().any(|p| p.matches(subject)) {
            return false;
        }
        // Check allow
        self.subscribe.iter().any(|p| p.matches(subject))
    }

    /// Join two subjects (union of permissions, union of denies)
    pub fn join(mut self, other: Self) -> Self {
        self.publish.extend(other.publish);
        self.subscribe.extend(other.subscribe);
        self.deny_publish.extend(other.deny_publish);
        self.deny_subscribe.extend(other.deny_subscribe);
        self
    }

    /// Check if this subject subsumes another (grants at least same permissions)
    pub fn subsumes(&self, other: &Subject) -> bool {
        // Every publish pattern in other must be covered by some pattern in self
        let publish_covered = other.publish.iter().all(|op| {
            self.publish.iter().any(|sp| sp.subsumes(op))
        });

        // Every subscribe pattern in other must be covered
        let subscribe_covered = other.subscribe.iter().all(|op| {
            self.subscribe.iter().any(|sp| sp.subsumes(op))
        });

        // Our denies must not restrict what other allows
        // (simplified: we shouldn't add new denies)
        let denies_compatible = self.deny_publish.len() <= other.deny_publish.len()
            && self.deny_subscribe.len() <= other.deny_subscribe.len();

        publish_covered && subscribe_covered && denies_compatible
    }
}

impl Default for Subject {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SUBJECT ERROR
// ============================================================================

/// Errors in subject pattern parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectError {
    /// Subject string is empty
    Empty,
    /// Token is empty (consecutive dots)
    EmptyToken,
    /// Multi-wildcard (>) not at end
    MultiWildcardNotTerminal,
    /// Unexpected wildcard in literal context
    UnexpectedWildcard,
}

impl fmt::Display for SubjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubjectError::Empty => write!(f, "Subject cannot be empty"),
            SubjectError::EmptyToken => write!(f, "Subject contains empty token"),
            SubjectError::MultiWildcardNotTerminal => {
                write!(f, "Multi-wildcard (>) must be at end of subject")
            }
            SubjectError::UnexpectedWildcard => {
                write!(f, "Unexpected wildcard in literal subject")
            }
        }
    }
}

impl std::error::Error for SubjectError {}

// ============================================================================
// MESSAGE TYPE - Commands, Queries, Events (CQS/CQRS)
// ============================================================================

/// Message types following CQS/CQRS patterns
///
/// Each type maps to different NATS subject patterns and claim requirements:
/// - Commands: Write operations, require write claims
/// - Queries: Read operations, require read claims
/// - Events: Domain events, require subscribe claims
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// Commands - write operations that change state
    /// Pattern: `org.unit.aggregate.command.{action}`
    /// Claims: Write claims (StreamWrite, KvWrite, etc.)
    Command,

    /// Queries - read operations that return data
    /// Pattern: `org.unit.aggregate.query.{name}`
    /// Claims: Read claims (StreamRead, KvRead, etc.)
    Query,

    /// Events - domain events emitted after state changes
    /// Pattern: `org.unit.aggregate.event.{name}`
    /// Claims: Subscribe claims (StreamRead for consumers)
    Event,
}

impl MessageType {
    /// Get the subject token for this message type
    pub fn token(&self) -> &'static str {
        match self {
            MessageType::Command => "command",
            MessageType::Query => "query",
            MessageType::Event => "event",
        }
    }

    /// Parse from subject token
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "command" | "cmd" => Some(MessageType::Command),
            "query" | "qry" => Some(MessageType::Query),
            "event" | "evt" => Some(MessageType::Event),
            _ => None,
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token())
    }
}

// ============================================================================
// TYPED SUBJECT - Subject with message type semantics
// ============================================================================

/// A subject pattern with message type semantics
///
/// This captures the CQS/CQRS nature of the subject for claim validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedSubject {
    /// The underlying pattern
    pub pattern: SubjectPattern,
    /// The message type (determines claim requirements)
    pub message_type: MessageType,
}

impl TypedSubject {
    /// Create a new typed subject
    pub fn new(pattern: SubjectPattern, message_type: MessageType) -> Self {
        Self { pattern, message_type }
    }

    /// Create a command subject
    pub fn command(pattern: SubjectPattern) -> Self {
        Self::new(pattern, MessageType::Command)
    }

    /// Create a query subject
    pub fn query(pattern: SubjectPattern) -> Self {
        Self::new(pattern, MessageType::Query)
    }

    /// Create an event subject
    pub fn event(pattern: SubjectPattern) -> Self {
        Self::new(pattern, MessageType::Event)
    }

    /// Try to infer message type from subject pattern
    pub fn from_pattern_inferred(pattern: SubjectPattern) -> Self {
        let pattern_str = pattern.to_nats_string();
        let message_type = if pattern_str.contains(".command.") || pattern_str.contains(".cmd.") {
            MessageType::Command
        } else if pattern_str.contains(".query.") || pattern_str.contains(".qry.") {
            MessageType::Query
        } else if pattern_str.contains(".event.") || pattern_str.contains(".evt.") {
            MessageType::Event
        } else {
            // Default to Event for pub/sub patterns
            MessageType::Event
        };
        Self::new(pattern, message_type)
    }
}

impl fmt::Display for TypedSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.pattern, self.message_type)
    }
}

// ============================================================================
// CIM STANDARD SUBJECTS - Semantic naming patterns
// ============================================================================

/// Standard CIM subject namespace patterns
///
/// CIM uses semantic naming: `organization.unit.aggregate.{command|query|event}.action`
pub struct CimSubjects;

impl CimSubjects {
    // ========================================================================
    // TYPED SUBJECTS - Command/Query/Event patterns
    // ========================================================================

    /// Command subject: `org.unit.aggregate.command.action`
    pub fn command(org: &str, unit: &str, aggregate: &str, action: &str) -> TypedSubject {
        let pattern = SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Literal(unit.to_string()),
            SubjectToken::Literal(aggregate.to_string()),
            SubjectToken::Literal("command".to_string()),
            SubjectToken::Literal(action.to_string()),
        ]);
        TypedSubject::command(pattern)
    }

    /// All commands for an aggregate: `org.unit.aggregate.command.>`
    pub fn commands(org: &str, unit: &str, aggregate: &str) -> TypedSubject {
        let pattern = SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Literal(unit.to_string()),
            SubjectToken::Literal(aggregate.to_string()),
            SubjectToken::Literal("command".to_string()),
            SubjectToken::Multi,
        ]);
        TypedSubject::command(pattern)
    }

    /// Query subject: `org.unit.aggregate.query.name`
    pub fn query(org: &str, unit: &str, aggregate: &str, name: &str) -> TypedSubject {
        let pattern = SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Literal(unit.to_string()),
            SubjectToken::Literal(aggregate.to_string()),
            SubjectToken::Literal("query".to_string()),
            SubjectToken::Literal(name.to_string()),
        ]);
        TypedSubject::query(pattern)
    }

    /// All queries for an aggregate: `org.unit.aggregate.query.>`
    pub fn queries(org: &str, unit: &str, aggregate: &str) -> TypedSubject {
        let pattern = SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Literal(unit.to_string()),
            SubjectToken::Literal(aggregate.to_string()),
            SubjectToken::Literal("query".to_string()),
            SubjectToken::Multi,
        ]);
        TypedSubject::query(pattern)
    }

    /// Event subject: `org.unit.aggregate.event.name`
    pub fn event(org: &str, unit: &str, aggregate: &str, name: &str) -> TypedSubject {
        let pattern = SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Literal(unit.to_string()),
            SubjectToken::Literal(aggregate.to_string()),
            SubjectToken::Literal("event".to_string()),
            SubjectToken::Literal(name.to_string()),
        ]);
        TypedSubject::event(pattern)
    }

    /// All events for an aggregate: `org.unit.aggregate.event.>`
    pub fn events(org: &str, unit: &str, aggregate: &str) -> TypedSubject {
        let pattern = SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Literal(unit.to_string()),
            SubjectToken::Literal(aggregate.to_string()),
            SubjectToken::Literal("event".to_string()),
            SubjectToken::Multi,
        ]);
        TypedSubject::event(pattern)
    }

    // ========================================================================
    // UNTYPED PATTERNS - For backward compatibility
    // ========================================================================

    /// Organization-level all messages
    pub fn organization(org: &str) -> SubjectPattern {
        SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Multi,
        ])
    }

    /// Unit-level all messages within organization
    pub fn organization_unit(org: &str, unit: &str) -> SubjectPattern {
        SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Literal(unit.to_string()),
            SubjectToken::Multi,
        ])
    }

    /// Aggregate-level all messages: `org.unit.aggregate.>`
    pub fn aggregate(org: &str, unit: &str, aggregate: &str) -> SubjectPattern {
        SubjectPattern::new(vec![
            SubjectToken::Literal(org.to_string()),
            SubjectToken::Literal(unit.to_string()),
            SubjectToken::Literal(aggregate.to_string()),
            SubjectToken::Multi,
        ])
    }

    // ========================================================================
    // SYSTEM SUBJECTS
    // ========================================================================

    /// PKI namespace: `$CIM.pki.>`
    pub fn pki_all() -> SubjectPattern {
        SubjectPattern::parse("$CIM.pki.>").unwrap()
    }

    /// PKI commands: `$CIM.pki.command.>`
    pub fn pki_commands() -> TypedSubject {
        TypedSubject::command(SubjectPattern::parse("$CIM.pki.command.>").unwrap())
    }

    /// PKI queries: `$CIM.pki.query.>`
    pub fn pki_queries() -> TypedSubject {
        TypedSubject::query(SubjectPattern::parse("$CIM.pki.query.>").unwrap())
    }

    /// PKI events: `$CIM.pki.event.>`
    pub fn pki_events() -> TypedSubject {
        TypedSubject::event(SubjectPattern::parse("$CIM.pki.event.>").unwrap())
    }

    /// Key commands: `$CIM.pki.key.command.>`
    pub fn key_commands() -> TypedSubject {
        TypedSubject::command(SubjectPattern::parse("$CIM.pki.key.command.>").unwrap())
    }

    /// Key events: `$CIM.pki.key.event.>`
    pub fn key_events() -> TypedSubject {
        TypedSubject::event(SubjectPattern::parse("$CIM.pki.key.event.>").unwrap())
    }

    /// Certificate commands: `$CIM.pki.certificate.command.>`
    pub fn certificate_commands() -> TypedSubject {
        TypedSubject::command(SubjectPattern::parse("$CIM.pki.certificate.command.>").unwrap())
    }

    /// Certificate events: `$CIM.pki.certificate.event.>`
    pub fn certificate_events() -> TypedSubject {
        TypedSubject::event(SubjectPattern::parse("$CIM.pki.certificate.event.>").unwrap())
    }

    /// NATS admin namespace: `$SYS.>`
    pub fn nats_system() -> SubjectPattern {
        SubjectPattern::parse("$SYS.>").unwrap()
    }

    /// JetStream admin: `$JS.API.>`
    pub fn jetstream_api() -> SubjectPattern {
        SubjectPattern::parse("$JS.API.>").unwrap()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_pattern_parse() {
        let pattern = SubjectPattern::parse("foo.bar.baz").unwrap();
        assert_eq!(pattern.len(), 3);
        assert!(!pattern.has_wildcards());
    }

    #[test]
    fn test_subject_pattern_wildcard_single() {
        let pattern = SubjectPattern::parse("foo.*.baz").unwrap();
        assert!(pattern.has_wildcards());
        assert!(pattern.matches("foo.bar.baz"));
        assert!(pattern.matches("foo.qux.baz"));
        assert!(!pattern.matches("foo.bar.qux"));
    }

    #[test]
    fn test_subject_pattern_wildcard_multi() {
        let pattern = SubjectPattern::parse("foo.>").unwrap();
        assert!(pattern.has_wildcards());
        assert!(pattern.matches("foo.bar"));
        assert!(pattern.matches("foo.bar.baz"));
        assert!(pattern.matches("foo.bar.baz.qux"));
        assert!(!pattern.matches("bar.foo"));
    }

    #[test]
    fn test_multi_wildcard_must_be_terminal() {
        let result = SubjectPattern::parse("foo.>.bar");
        assert!(matches!(result, Err(SubjectError::MultiWildcardNotTerminal)));
    }

    #[test]
    fn test_subject_pattern_subsumption() {
        let broad = SubjectPattern::parse("foo.>").unwrap();
        let narrow = SubjectPattern::parse("foo.bar.*").unwrap();
        let specific = SubjectPattern::parse("foo.bar.baz").unwrap();

        assert!(broad.subsumes(&narrow));
        assert!(broad.subsumes(&specific));
        assert!(!narrow.subsumes(&broad));
        assert!(narrow.subsumes(&specific));
    }

    #[test]
    fn test_subject_can_publish() {
        let subject = Subject::new()
            .with_publish(SubjectPattern::parse("foo.>").unwrap())
            .with_deny_publish(SubjectPattern::parse("foo.secret.*").unwrap());

        assert!(subject.can_publish("foo.bar.baz"));
        assert!(!subject.can_publish("foo.secret.key"));
        assert!(!subject.can_publish("bar.baz"));
    }

    #[test]
    fn test_subject_join() {
        let s1 = Subject::new()
            .with_publish(SubjectPattern::parse("foo.>").unwrap());
        let s2 = Subject::new()
            .with_subscribe(SubjectPattern::parse("bar.>").unwrap());

        let joined = s1.join(s2);
        assert!(joined.can_publish("foo.test"));
        assert!(joined.can_subscribe("bar.test"));
    }

    #[test]
    fn test_cim_subjects() {
        let org_pattern = CimSubjects::organization("cowboyai");
        assert!(org_pattern.matches("cowboyai.security.keys.generate"));

        // Using typed command subject
        let cmd_subject = CimSubjects::command("cowboyai", "security", "keys", "generate");
        assert_eq!(cmd_subject.message_type, MessageType::Command);
        assert!(cmd_subject.pattern.matches("cowboyai.security.keys.command.generate"));

        // Using aggregate pattern
        let agg_pattern = CimSubjects::aggregate("cowboyai", "security", "keys");
        assert!(agg_pattern.matches("cowboyai.security.keys.command.generate"));
        assert!(agg_pattern.matches("cowboyai.security.keys.query.by_id"));
        assert!(agg_pattern.matches("cowboyai.security.keys.event.generated"));
    }

    #[test]
    fn test_subject_full_access() {
        let full = Subject::full_access();
        assert!(full.can_publish("anything.here"));
        assert!(full.can_subscribe("any.subject.at.all"));
    }
}
