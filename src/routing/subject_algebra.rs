// Copyright (c) 2025 - Cowboy AI, LLC.

//! Subject Algebra: Formal Monoid and Parser Combinators
//!
//! This module provides a mathematically rigorous subject algebra following
//! Category Theory principles. NATS subjects form a Free Monoid where:
//!
//! - **Identity**: Empty subject `ε`
//! - **Operation**: Concatenation `⊕`
//! - **Laws**: `ε ⊕ a = a`, `a ⊕ ε = a`, `(a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)`
//!
//! ## Subject Grammar (BNF)
//!
//! ```text
//! <subject>    ::= <token> | <token> '.' <subject>
//! <token>      ::= <literal> | '*' | '>'
//! <literal>    ::= [a-zA-Z0-9_-]+
//! <pattern>    ::= <subject>   // '*' and '>' wildcards allowed
//! <publishable>::= <subject>   // No wildcards allowed
//! ```
//!
//! ## Algebraic Operations
//!
//! | Operation | Symbol | Description |
//! |-----------|--------|-------------|
//! | Identity  | `ε`    | Empty subject |
//! | Concat    | `⊕`    | Subject concatenation |
//! | Parse     | `↑`    | String to Subject |
//! | Render    | `↓`    | Subject to String |
//! | Match     | `≼`    | Pattern matching |

use std::fmt;
use std::ops::{Add, BitOr};

/// A formal monoid structure for algebraic types.
///
/// A Monoid (M, ⊕, ε) satisfies:
/// - Left identity: ε ⊕ a = a
/// - Right identity: a ⊕ ε = a
/// - Associativity: (a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)
pub trait Monoid: Sized + Clone + PartialEq {
    /// The identity element (ε)
    fn identity() -> Self;

    /// The binary operation (⊕)
    fn combine(&self, other: &Self) -> Self;

    /// Check if this is the identity element
    fn is_identity(&self) -> bool {
        *self == Self::identity()
    }

    /// Combine multiple elements (fold)
    fn concat_all<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        iter.into_iter().fold(Self::identity(), |acc, x| acc.combine(&x))
    }
}

/// Subject token following NATS subject token rules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    /// Literal string token (alphanumeric, underscore, hyphen)
    Literal(String),
    /// Single-level wildcard (*)
    Single,
    /// Multi-level suffix wildcard (>)
    Suffix,
}

impl Token {
    /// Parse a token from a string
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        if s.is_empty() {
            return Err(ParseError::EmptyToken);
        }
        match s {
            "*" => Ok(Token::Single),
            ">" => Ok(Token::Suffix),
            _ => {
                // Validate literal: alphanumeric, underscore, hyphen only
                if s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                    Ok(Token::Literal(s.to_string()))
                } else {
                    Err(ParseError::InvalidCharacter(s.to_string()))
                }
            }
        }
    }

    /// Check if this is a wildcard token
    pub fn is_wildcard(&self) -> bool {
        matches!(self, Token::Single | Token::Suffix)
    }

    /// Check if this is the suffix wildcard
    pub fn is_suffix(&self) -> bool {
        matches!(self, Token::Suffix)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Literal(s) => write!(f, "{}", s),
            Token::Single => write!(f, "*"),
            Token::Suffix => write!(f, ">"),
        }
    }
}

/// A NATS subject as a free monoid over tokens
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Subject {
    tokens: Vec<Token>,
}

impl Subject {
    /// Create a new empty subject (identity element)
    pub fn empty() -> Self {
        Subject { tokens: Vec::new() }
    }

    /// Create a subject with a single token
    pub fn token(t: Token) -> Self {
        Subject { tokens: vec![t] }
    }

    /// Create a subject from a literal string
    pub fn literal(s: impl Into<String>) -> Result<Self, ParseError> {
        let token = Token::parse(&s.into())?;
        Ok(Subject { tokens: vec![token] })
    }

    /// Parse a subject from a dot-delimited string
    ///
    /// # Grammar
    ///
    /// ```text
    /// subject ::= ε | token ('.' token)*
    /// ```
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        if s.is_empty() {
            return Ok(Self::empty());
        }

        let parts: Vec<&str> = s.split('.').collect();
        let mut tokens = Vec::with_capacity(parts.len());

        for (i, part) in parts.iter().enumerate() {
            let token = Token::parse(part)?;

            // Suffix wildcard must be last
            if token.is_suffix() && i != parts.len() - 1 {
                return Err(ParseError::SuffixNotLast);
            }

            tokens.push(token);
        }

        Ok(Subject { tokens })
    }

    /// Render the subject as a dot-delimited string
    pub fn render(&self) -> String {
        self.tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Get the number of tokens (depth)
    pub fn depth(&self) -> usize {
        self.tokens.len()
    }

    /// Check if this subject contains wildcards (pattern)
    pub fn is_pattern(&self) -> bool {
        self.tokens.iter().any(|t| t.is_wildcard())
    }

    /// Check if this subject can be published (no wildcards)
    pub fn is_publishable(&self) -> bool {
        !self.is_pattern()
    }

    /// Get the tokens
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Append a literal token using fluent API
    pub fn append(self, s: impl Into<String>) -> Result<Self, ParseError> {
        let token = Token::parse(&s.into())?;
        let mut tokens = self.tokens;
        tokens.push(token);
        Ok(Subject { tokens })
    }

    /// Append a single-level wildcard
    pub fn wildcard(self) -> Self {
        let mut tokens = self.tokens;
        tokens.push(Token::Single);
        Subject { tokens }
    }

    /// Append a suffix wildcard
    pub fn suffix(self) -> Self {
        let mut tokens = self.tokens;
        tokens.push(Token::Suffix);
        Subject { tokens }
    }

    /// Check if this subject matches a pattern
    ///
    /// The pattern may contain wildcards, self should be publishable.
    pub fn matches(&self, pattern: &Subject) -> bool {
        if self.is_pattern() {
            return false; // Can only match publishable subjects against patterns
        }

        let mut self_idx = 0;
        let mut pattern_idx = 0;

        while pattern_idx < pattern.tokens.len() {
            match &pattern.tokens[pattern_idx] {
                Token::Suffix => {
                    // > matches everything remaining (at least one token)
                    return self_idx < self.tokens.len();
                }
                Token::Single => {
                    // * matches exactly one token
                    if self_idx >= self.tokens.len() {
                        return false;
                    }
                    self_idx += 1;
                    pattern_idx += 1;
                }
                Token::Literal(pat) => {
                    if self_idx >= self.tokens.len() {
                        return false;
                    }
                    match &self.tokens[self_idx] {
                        Token::Literal(s) if s == pat => {
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

    /// Calculate specificity score (higher = more specific)
    ///
    /// Used for route priority ordering.
    pub fn specificity(&self) -> usize {
        self.tokens.iter().fold(0, |acc, t| {
            acc + match t {
                Token::Literal(_) => 10,
                Token::Single => 1,
                Token::Suffix => 0,
            }
        })
    }
}

impl Monoid for Subject {
    /// Identity element: empty subject
    fn identity() -> Self {
        Subject::empty()
    }

    /// Binary operation: concatenation
    fn combine(&self, other: &Self) -> Self {
        let mut tokens = self.tokens.clone();
        tokens.extend(other.tokens.iter().cloned());
        Subject { tokens }
    }
}

// Operator overloading for Subject algebra
impl Add for Subject {
    type Output = Subject;

    /// Subject concatenation: `a + b`
    fn add(self, rhs: Subject) -> Subject {
        self.combine(&rhs)
    }
}

impl Add<&Subject> for Subject {
    type Output = Subject;

    fn add(self, rhs: &Subject) -> Subject {
        self.combine(rhs)
    }
}

impl BitOr for Subject {
    type Output = Vec<Subject>;

    /// Subject alternative: `a | b` (returns both as alternatives)
    fn bitor(self, rhs: Subject) -> Vec<Subject> {
        vec![self, rhs]
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// Parse error for subject algebra
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("Empty token in subject")]
    EmptyToken,

    #[error("Invalid character in token: {0}")]
    InvalidCharacter(String),

    #[error("Suffix wildcard (>) must be the last token")]
    SuffixNotLast,
}

/// Subject builder with fluent DSL
#[derive(Debug, Clone, Default)]
pub struct SubjectBuilder {
    subject: Subject,
}

impl SubjectBuilder {
    /// Create a new subject builder
    pub fn new() -> Self {
        SubjectBuilder { subject: Subject::empty() }
    }

    /// Start with an organization prefix
    pub fn org(org: impl Into<String>) -> Result<Self, ParseError> {
        Ok(SubjectBuilder {
            subject: Subject::literal(org)?,
        })
    }

    /// Add a unit/context token
    pub fn unit(self, unit: impl Into<String>) -> Result<Self, ParseError> {
        Ok(SubjectBuilder {
            subject: self.subject.append(unit)?,
        })
    }

    /// Add an entity token
    pub fn entity(self, entity: impl Into<String>) -> Result<Self, ParseError> {
        Ok(SubjectBuilder {
            subject: self.subject.append(entity)?,
        })
    }

    /// Add an operation token
    pub fn operation(self, op: impl Into<String>) -> Result<Self, ParseError> {
        let op_str = op.into();
        // Split operation on dots
        let parts: Vec<&str> = op_str.split('.').collect();
        let mut subject = self.subject;
        for part in parts {
            subject = subject.append(part)?;
        }
        Ok(SubjectBuilder { subject })
    }

    /// Add a single wildcard
    pub fn any(self) -> Self {
        SubjectBuilder {
            subject: self.subject.wildcard(),
        }
    }

    /// Add a suffix wildcard (match all remaining)
    pub fn all(self) -> Self {
        SubjectBuilder {
            subject: self.subject.suffix(),
        }
    }

    /// Build the subject
    pub fn build(self) -> Subject {
        self.subject
    }
}

/// Convenience functions for common subject patterns
pub mod patterns {
    use super::*;

    /// Create an organization subject
    pub fn org(name: &str) -> Result<Subject, ParseError> {
        Subject::literal(name)
    }

    /// Create a subscribe pattern for all events under an org
    pub fn org_events(name: &str) -> Result<Subject, ParseError> {
        SubjectBuilder::org(name)?
            .unit("events")?
            .all()
            .build()
            .pipe(Ok)
    }

    /// Create a service endpoint subject
    pub fn service(org: &str, svc: &str, method: &str) -> Result<Subject, ParseError> {
        SubjectBuilder::org(org)?
            .unit("services")?
            .entity(svc)?
            .operation(method)?
            .build()
            .pipe(Ok)
    }

    /// Create a key management subject
    pub fn keys(org: &str, operation: &str) -> Result<Subject, ParseError> {
        SubjectBuilder::org(org)?
            .unit("security")?
            .entity("keys")?
            .operation(operation)?
            .build()
            .pipe(Ok)
    }

    /// Create an audit subject
    pub fn audit(org: &str, event_type: &str) -> Result<Subject, ParseError> {
        SubjectBuilder::org(org)?
            .unit("security")?
            .entity("audit")?
            .operation(event_type)?
            .build()
            .pipe(Ok)
    }
}

// Helper trait for fluent pipe syntax
trait Pipe: Sized {
    fn pipe<T, F: FnOnce(Self) -> T>(self, f: F) -> T {
        f(self)
    }
}

impl<X> Pipe for X {}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Monoid Law Tests
    // =========================================================================

    #[test]
    fn test_monoid_left_identity() {
        // ε ⊕ a = a
        let a = Subject::parse("cowboyai.security.keys").unwrap();
        let result = Subject::identity().combine(&a);
        assert_eq!(result, a);
    }

    #[test]
    fn test_monoid_right_identity() {
        // a ⊕ ε = a
        let a = Subject::parse("cowboyai.security.keys").unwrap();
        let result = a.combine(&Subject::identity());
        assert_eq!(result, a);
    }

    #[test]
    fn test_monoid_associativity() {
        // (a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)
        let a = Subject::parse("cowboyai").unwrap();
        let b = Subject::parse("security").unwrap();
        let c = Subject::parse("keys").unwrap();

        let left = a.clone().combine(&b).combine(&c);
        let right = a.combine(&b.combine(&c));
        assert_eq!(left, right);
    }

    #[test]
    fn test_monoid_concat_all() {
        let subjects = vec![
            Subject::parse("cowboyai").unwrap(),
            Subject::parse("security").unwrap(),
            Subject::parse("keys").unwrap(),
        ];

        let result = Subject::concat_all(subjects);
        assert_eq!(result.render(), "cowboyai.security.keys");
    }

    // =========================================================================
    // Subject Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_empty() {
        let subject = Subject::parse("").unwrap();
        assert!(subject.is_identity());
        assert_eq!(subject.depth(), 0);
    }

    #[test]
    fn test_parse_simple() {
        let subject = Subject::parse("cowboyai.security.keys").unwrap();
        assert_eq!(subject.depth(), 3);
        assert_eq!(subject.render(), "cowboyai.security.keys");
    }

    #[test]
    fn test_parse_with_wildcards() {
        let pattern = Subject::parse("cowboyai.security.*").unwrap();
        assert!(pattern.is_pattern());
        assert!(!pattern.is_publishable());

        let suffix = Subject::parse("cowboyai.security.>").unwrap();
        assert!(suffix.is_pattern());
    }

    #[test]
    fn test_parse_error_suffix_not_last() {
        let result = Subject::parse("cowboyai.>.keys");
        assert!(matches!(result, Err(ParseError::SuffixNotLast)));
    }

    #[test]
    fn test_parse_error_empty_token() {
        let result = Subject::parse("cowboyai..keys");
        assert!(matches!(result, Err(ParseError::EmptyToken)));
    }

    // =========================================================================
    // Subject Matching Tests
    // =========================================================================

    #[test]
    fn test_match_exact() {
        let subject = Subject::parse("cowboyai.security.keys").unwrap();
        let pattern = Subject::parse("cowboyai.security.keys").unwrap();
        assert!(subject.matches(&pattern));
    }

    #[test]
    fn test_match_single_wildcard() {
        let subject = Subject::parse("cowboyai.security.keys").unwrap();
        let pattern = Subject::parse("cowboyai.*.keys").unwrap();
        assert!(subject.matches(&pattern));

        let pattern2 = Subject::parse("cowboyai.security.*").unwrap();
        assert!(subject.matches(&pattern2));
    }

    #[test]
    fn test_match_suffix_wildcard() {
        let subject = Subject::parse("cowboyai.security.keys.generated").unwrap();
        let pattern = Subject::parse("cowboyai.security.>").unwrap();
        assert!(subject.matches(&pattern));

        // Should not match if nothing follows
        let short = Subject::parse("cowboyai").unwrap();
        let pattern2 = Subject::parse("cowboyai.>").unwrap();
        assert!(!short.matches(&pattern2));
    }

    #[test]
    fn test_match_no_match() {
        let subject = Subject::parse("cowboyai.security.keys").unwrap();
        let pattern = Subject::parse("cowboyai.infrastructure.>").unwrap();
        assert!(!subject.matches(&pattern));
    }

    // =========================================================================
    // Operator Tests
    // =========================================================================

    #[test]
    fn test_add_operator() {
        let a = Subject::parse("cowboyai").unwrap();
        let b = Subject::parse("security.keys").unwrap();
        let result = a + b;
        assert_eq!(result.render(), "cowboyai.security.keys");
    }

    #[test]
    fn test_bitor_operator() {
        let a = Subject::parse("cowboyai.security").unwrap();
        let b = Subject::parse("cowboyai.infrastructure").unwrap();
        let alternatives = a | b;
        assert_eq!(alternatives.len(), 2);
    }

    // =========================================================================
    // Builder Tests
    // =========================================================================

    #[test]
    fn test_builder_fluent() {
        let subject = SubjectBuilder::org("cowboyai")
            .unwrap()
            .unit("security")
            .unwrap()
            .entity("keys")
            .unwrap()
            .operation("certificate.generate.root")
            .unwrap()
            .build();

        assert_eq!(
            subject.render(),
            "cowboyai.security.keys.certificate.generate.root"
        );
    }

    #[test]
    fn test_builder_with_wildcards() {
        let pattern = SubjectBuilder::org("cowboyai")
            .unwrap()
            .unit("events")
            .unwrap()
            .any()
            .all()
            .build();

        // any() adds *, all() adds >
        assert_eq!(pattern.render(), "cowboyai.events.*.>");
        assert!(pattern.is_pattern());
    }

    // =========================================================================
    // Specificity Tests
    // =========================================================================

    #[test]
    fn test_specificity_ordering() {
        let exact = Subject::parse("cowboyai.security.keys").unwrap();
        let single = Subject::parse("cowboyai.security.*").unwrap();
        let suffix = Subject::parse("cowboyai.>").unwrap();

        assert!(exact.specificity() > single.specificity());
        assert!(single.specificity() > suffix.specificity());
    }

    // =========================================================================
    // Pattern Functions Tests
    // =========================================================================

    #[test]
    fn test_patterns_service() {
        let subject = patterns::service("cowboyai", "auth", "login").unwrap();
        assert_eq!(subject.render(), "cowboyai.services.auth.login");
    }

    #[test]
    fn test_patterns_keys() {
        let subject = patterns::keys("cowboyai", "certificate.generate.root").unwrap();
        assert_eq!(
            subject.render(),
            "cowboyai.security.keys.certificate.generate.root"
        );
    }

    #[test]
    fn test_patterns_audit() {
        let subject = patterns::audit("cowboyai", "key.revoked").unwrap();
        assert_eq!(subject.render(), "cowboyai.security.audit.key.revoked");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Generate valid token strings
    fn arb_token() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_-]{0,10}".prop_map(|s| s)
    }

    // Generate valid subjects (1-4 tokens)
    fn arb_subject() -> impl Strategy<Value = Subject> {
        proptest::collection::vec(arb_token(), 1..=4)
            .prop_map(|tokens| {
                let s = tokens.join(".");
                Subject::parse(&s).unwrap()
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_monoid_left_identity(a in arb_subject()) {
            let result = Subject::identity().combine(&a);
            prop_assert_eq!(result, a);
        }

        #[test]
        fn prop_monoid_right_identity(a in arb_subject()) {
            let result = a.clone().combine(&Subject::identity());
            prop_assert_eq!(result, a);
        }

        #[test]
        fn prop_monoid_associativity(
            a in arb_subject(),
            b in arb_subject(),
            c in arb_subject()
        ) {
            let left = a.clone().combine(&b).combine(&c);
            let right = a.combine(&b.combine(&c));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn prop_parse_render_roundtrip(a in arb_subject()) {
            let rendered = a.render();
            let parsed = Subject::parse(&rendered).unwrap();
            prop_assert_eq!(a, parsed);
        }

        #[test]
        fn prop_specificity_literal_beats_wildcard(a in arb_subject()) {
            let with_wildcard = a.clone().wildcard();
            // If original has same depth, literal should be more specific
            if a.depth() == with_wildcard.depth() - 1 {
                prop_assert!(a.specificity() < with_wildcard.specificity() + 9);
            }
        }
    }
}
