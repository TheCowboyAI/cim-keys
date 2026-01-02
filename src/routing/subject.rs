// Copyright (c) 2025 - Cowboy AI, LLC.

//! Subject-Based Intent Routing
//!
//! This module implements subject algebra for intent routing per FRP Axiom A6.
//! Instead of massive match statements, intents are routed via subject patterns.
//!
//! ## Subject Hierarchy
//!
//! ```text
//! ui.organization.selected
//! ui.organization.created
//! domain.organization.created
//! port.organization.loaded
//! ```
//!
//! ## Pattern Matching
//!
//! Supports wildcard patterns:
//! - `ui.>` - matches all UI intents
//! - `domain.organization.>` - matches all organization domain intents
//! - `*.created` - matches all created intents
//!
//! ## Category Theory
//!
//! Subject routing forms a Free Monoid where:
//! - Identity: empty pattern (matches nothing)
//! - Composition: pattern concatenation
//! - Monoid laws: associativity and identity

use std::collections::HashMap;
use std::sync::Arc;
use std::fmt;

/// Intent category following CIM subject naming convention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentCategory {
    /// UI-originated intents: `ui.*`
    Ui,
    /// Domain event intents: `domain.*`
    Domain,
    /// Port/adapter intents: `port.*`
    Port,
    /// System-level intents: `system.*`
    System,
    /// Error intents: `error.*`
    Error,
}

impl IntentCategory {
    /// Get the subject prefix for this category
    pub fn as_prefix(&self) -> &'static str {
        match self {
            IntentCategory::Ui => "ui",
            IntentCategory::Domain => "domain",
            IntentCategory::Port => "port",
            IntentCategory::System => "system",
            IntentCategory::Error => "error",
        }
    }

    /// Parse a category from a subject string
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "ui" => Some(IntentCategory::Ui),
            "domain" => Some(IntentCategory::Domain),
            "port" => Some(IntentCategory::Port),
            "system" => Some(IntentCategory::System),
            "error" => Some(IntentCategory::Error),
            _ => None,
        }
    }
}

impl fmt::Display for IntentCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_prefix())
    }
}

/// A subject pattern for matching intents
///
/// Patterns follow NATS subject pattern syntax:
/// - `.` separates tokens
/// - `*` matches single token
/// - `>` matches zero or more tokens (suffix only)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubjectPattern {
    tokens: Vec<PatternToken>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PatternToken {
    Literal(String),
    SingleWildcard,  // *
    MultiWildcard,   // >
}

impl SubjectPattern {
    /// Parse a subject pattern string
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// SubjectPattern::parse("ui.organization.selected")?; // exact match
    /// SubjectPattern::parse("ui.organization.*")?;         // any action
    /// SubjectPattern::parse("ui.>")?;                      // all UI intents
    /// ```
    pub fn parse(pattern: &str) -> Result<Self, SubjectPatternError> {
        if pattern.is_empty() {
            return Err(SubjectPatternError::Empty);
        }

        let tokens: Result<Vec<_>, _> = pattern
            .split('.')
            .map(|token| {
                match token {
                    "*" => Ok(PatternToken::SingleWildcard),
                    ">" => Ok(PatternToken::MultiWildcard),
                    "" => Err(SubjectPatternError::EmptyToken),
                    s => Ok(PatternToken::Literal(s.to_string())),
                }
            })
            .collect();

        let tokens = tokens?;

        // Validate: > can only appear at the end
        for (i, token) in tokens.iter().enumerate() {
            if matches!(token, PatternToken::MultiWildcard) && i != tokens.len() - 1 {
                return Err(SubjectPatternError::MultiWildcardNotAtEnd);
            }
        }

        Ok(SubjectPattern { tokens })
    }

    /// Check if this pattern matches a subject string
    pub fn matches(&self, subject: &str) -> bool {
        let subject_tokens: Vec<&str> = subject.split('.').collect();
        self.matches_tokens(&subject_tokens)
    }

    fn matches_tokens(&self, subject_tokens: &[&str]) -> bool {
        let mut pattern_idx = 0;
        let mut subject_idx = 0;

        while pattern_idx < self.tokens.len() {
            match &self.tokens[pattern_idx] {
                PatternToken::MultiWildcard => {
                    // > matches rest of subject
                    return true;
                }
                PatternToken::SingleWildcard => {
                    // * matches exactly one token
                    if subject_idx >= subject_tokens.len() {
                        return false;
                    }
                    pattern_idx += 1;
                    subject_idx += 1;
                }
                PatternToken::Literal(expected) => {
                    if subject_idx >= subject_tokens.len() {
                        return false;
                    }
                    if subject_tokens[subject_idx] != expected {
                        return false;
                    }
                    pattern_idx += 1;
                    subject_idx += 1;
                }
            }
        }

        // Pattern exhausted - subject must also be exhausted
        subject_idx == subject_tokens.len()
    }

    /// Get the specificity of this pattern (more specific = higher number)
    /// Used for routing priority
    pub fn specificity(&self) -> usize {
        let mut score = 0;
        for token in &self.tokens {
            match token {
                PatternToken::Literal(_) => score += 10,
                PatternToken::SingleWildcard => score += 1,
                PatternToken::MultiWildcard => score += 0,
            }
        }
        score
    }
}

impl fmt::Display for SubjectPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.tokens.iter().map(|t| match t {
            PatternToken::Literal(s) => s.clone(),
            PatternToken::SingleWildcard => "*".to_string(),
            PatternToken::MultiWildcard => ">".to_string(),
        }).collect();
        write!(f, "{}", parts.join("."))
    }
}

/// Error parsing subject pattern
#[derive(Debug, Clone, PartialEq)]
pub enum SubjectPatternError {
    Empty,
    EmptyToken,
    MultiWildcardNotAtEnd,
}

impl fmt::Display for SubjectPatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubjectPatternError::Empty => write!(f, "Pattern cannot be empty"),
            SubjectPatternError::EmptyToken => write!(f, "Pattern contains empty token"),
            SubjectPatternError::MultiWildcardNotAtEnd => {
                write!(f, "Multi-wildcard (>) must be at end of pattern")
            }
        }
    }
}

impl std::error::Error for SubjectPatternError {}

/// Subject-based intent for compositional routing
///
/// Wraps an intent with its subject string for pattern-based dispatch.
#[derive(Debug, Clone)]
pub struct SubjectIntent<I> {
    /// The subject string (e.g., "ui.organization.selected")
    pub subject: String,
    /// The wrapped intent
    pub intent: I,
}

impl<I> SubjectIntent<I> {
    /// Create a new subject intent
    pub fn new(subject: impl Into<String>, intent: I) -> Self {
        SubjectIntent {
            subject: subject.into(),
            intent,
        }
    }

    /// Get the category from the subject
    pub fn category(&self) -> Option<IntentCategory> {
        self.subject
            .split('.')
            .next()
            .and_then(IntentCategory::from_prefix)
    }
}

/// A type-erased route handler
pub type RouteHandler<M, I> = Arc<dyn Fn(M, I) -> (M, Option<I>) + Send + Sync>;

/// Subject-based router for compositional intent dispatch
///
/// Instead of a massive match statement, handlers are registered by pattern
/// and dispatched based on subject matching.
pub struct SubjectRouter<M, I> {
    routes: Vec<(SubjectPattern, RouteHandler<M, I>)>,
    default_handler: Option<RouteHandler<M, I>>,
}

impl<M, I> SubjectRouter<M, I>
where
    M: Clone + Send + 'static,
    I: Clone + Send + 'static,
{
    /// Create a new empty router
    pub fn new() -> Self {
        SubjectRouter {
            routes: Vec::new(),
            default_handler: None,
        }
    }

    /// Register a handler for a subject pattern
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// router.route("ui.organization.>", handle_ui_organization);
    /// router.route("domain.>", handle_domain_events);
    /// ```
    pub fn route<F>(mut self, pattern: &str, handler: F) -> Self
    where
        F: Fn(M, I) -> (M, Option<I>) + Send + Sync + 'static,
    {
        if let Ok(pat) = SubjectPattern::parse(pattern) {
            self.routes.push((pat, Arc::new(handler)));
        }
        self
    }

    /// Set a default handler for unmatched subjects
    pub fn default<F>(mut self, handler: F) -> Self
    where
        F: Fn(M, I) -> (M, Option<I>) + Send + Sync + 'static,
    {
        self.default_handler = Some(Arc::new(handler));
        self
    }

    /// Sort routes by specificity (most specific first)
    pub fn build(mut self) -> Self {
        self.routes.sort_by(|a, b| {
            b.0.specificity().cmp(&a.0.specificity())
        });
        self
    }

    /// Dispatch an intent to the appropriate handler
    pub fn dispatch(&self, model: M, intent: &SubjectIntent<I>) -> (M, Option<I>) {
        // Find first matching route (sorted by specificity)
        for (pattern, handler) in &self.routes {
            if pattern.matches(&intent.subject) {
                return handler(model, intent.intent.clone());
            }
        }

        // Fall back to default handler
        if let Some(handler) = &self.default_handler {
            return handler(model, intent.intent.clone());
        }

        // No match - return model unchanged
        (model, None)
    }

    /// Get the number of registered routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

impl<M, I> Default for SubjectRouter<M, I>
where
    M: Clone + Send + 'static,
    I: Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Compose multiple routers hierarchically
///
/// Creates a router that dispatches to sub-routers based on category.
pub struct HierarchicalRouter<M, I> {
    category_routers: HashMap<IntentCategory, SubjectRouter<M, I>>,
    root_router: SubjectRouter<M, I>,
}

impl<M, I> HierarchicalRouter<M, I>
where
    M: Clone + Send + 'static,
    I: Clone + Send + 'static,
{
    /// Create a new hierarchical router
    pub fn new() -> Self {
        HierarchicalRouter {
            category_routers: HashMap::new(),
            root_router: SubjectRouter::new(),
        }
    }

    /// Register a router for a specific category
    pub fn category(mut self, category: IntentCategory, router: SubjectRouter<M, I>) -> Self {
        self.category_routers.insert(category, router);
        self
    }

    /// Set the root router for cross-category patterns
    pub fn root(mut self, router: SubjectRouter<M, I>) -> Self {
        self.root_router = router;
        self
    }

    /// Dispatch an intent
    pub fn dispatch(&self, model: M, intent: &SubjectIntent<I>) -> (M, Option<I>) {
        // Try category-specific router first
        if let Some(category) = intent.category() {
            if let Some(router) = self.category_routers.get(&category) {
                return router.dispatch(model, intent);
            }
        }

        // Fall back to root router
        self.root_router.dispatch(model, intent)
    }
}

impl<M, I> Default for HierarchicalRouter<M, I>
where
    M: Clone + Send + 'static,
    I: Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_pattern_exact() {
        let pattern = SubjectPattern::parse("ui.organization.selected").unwrap();
        assert!(pattern.matches("ui.organization.selected"));
        assert!(!pattern.matches("ui.organization.created"));
        assert!(!pattern.matches("ui.organization"));
    }

    #[test]
    fn test_subject_pattern_single_wildcard() {
        let pattern = SubjectPattern::parse("ui.organization.*").unwrap();
        assert!(pattern.matches("ui.organization.selected"));
        assert!(pattern.matches("ui.organization.created"));
        assert!(!pattern.matches("ui.organization"));
        assert!(!pattern.matches("ui.organization.foo.bar"));
    }

    #[test]
    fn test_subject_pattern_multi_wildcard() {
        let pattern = SubjectPattern::parse("ui.>").unwrap();
        assert!(pattern.matches("ui.organization.selected"));
        assert!(pattern.matches("ui.person"));
        assert!(pattern.matches("ui.anything.at.all"));
        assert!(!pattern.matches("domain.organization"));
    }

    #[test]
    fn test_subject_pattern_specificity() {
        let exact = SubjectPattern::parse("ui.organization.selected").unwrap();
        let single = SubjectPattern::parse("ui.organization.*").unwrap();
        let multi = SubjectPattern::parse("ui.>").unwrap();

        assert!(exact.specificity() > single.specificity());
        assert!(single.specificity() > multi.specificity());
    }

    #[test]
    fn test_intent_category() {
        assert_eq!(IntentCategory::Ui.as_prefix(), "ui");
        assert_eq!(IntentCategory::from_prefix("domain"), Some(IntentCategory::Domain));
        assert_eq!(IntentCategory::from_prefix("unknown"), None);
    }

    #[test]
    fn test_subject_intent() {
        let intent = SubjectIntent::new("ui.organization.selected", 42);
        assert_eq!(intent.category(), Some(IntentCategory::Ui));
        assert_eq!(intent.intent, 42);
    }

    #[test]
    fn test_subject_router_dispatch() {
        let router: SubjectRouter<i32, i32> = SubjectRouter::new()
            .route("ui.organization.>", |m, i| (m + 1, Some(i)))
            .route("ui.person.>", |m, i| (m + 10, Some(i)))
            .default(|m, _i| (m, None))
            .build();

        let org_intent = SubjectIntent::new("ui.organization.selected", 5);
        let (model, result) = router.dispatch(0, &org_intent);
        assert_eq!(model, 1);
        assert_eq!(result, Some(5));

        let person_intent = SubjectIntent::new("ui.person.added", 10);
        let (model, result) = router.dispatch(0, &person_intent);
        assert_eq!(model, 10);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_hierarchical_router() {
        let ui_router: SubjectRouter<i32, i32> = SubjectRouter::new()
            .route("ui.>", |m, i| (m + i, None))
            .build();

        let domain_router: SubjectRouter<i32, i32> = SubjectRouter::new()
            .route("domain.>", |m, i| (m * i, None))
            .build();

        let router = HierarchicalRouter::new()
            .category(IntentCategory::Ui, ui_router)
            .category(IntentCategory::Domain, domain_router);

        let ui_intent = SubjectIntent::new("ui.click", 5);
        let (model, _) = router.dispatch(10, &ui_intent);
        assert_eq!(model, 15);

        let domain_intent = SubjectIntent::new("domain.event", 3);
        let (model, _) = router.dispatch(10, &domain_intent);
        assert_eq!(model, 30);
    }

    #[test]
    fn test_pattern_validation() {
        assert!(SubjectPattern::parse("").is_err());
        assert!(SubjectPattern::parse("ui..action").is_err());
        assert!(SubjectPattern::parse("ui.>.more").is_err());
    }
}
