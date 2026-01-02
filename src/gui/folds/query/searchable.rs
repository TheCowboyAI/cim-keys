// Copyright (c) 2025 - Cowboy AI, LLC.

//! Searchable Text - Query Layer Data Type
//!
//! This module provides the SearchableText type for filtering operations.
//! It contains text fields and keywords for search matching.
//!
//! ## FRP Pipeline Role
//!
//! ```text
//! Model + Query → SearchableText → matches() → bool
//! ```
//!
//! ## Usage
//!
//! SearchableText is produced by LiftedNode::searchable_text() method
//! which provides search data from lifted domain nodes.

// ============================================================================
// OUTPUT TYPE
// ============================================================================

/// Searchable text extracted from a domain node.
///
/// Contains text fields and keywords for search matching.
#[derive(Debug, Clone)]
pub struct SearchableText {
    /// Text fields from the node data (name, email, subject, etc.)
    pub fields: Vec<String>,
    /// Type-specific keywords (e.g., "nats operator", "certificate", "yubikey")
    pub keywords: Vec<String>,
}

impl SearchableText {
    /// Create a new SearchableText with the given fields and keywords
    pub fn new(fields: Vec<String>, keywords: Vec<String>) -> Self {
        Self { fields, keywords }
    }

    /// Check if any field or keyword contains the query (case-insensitive)
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.fields.iter().any(|f| f.to_lowercase().contains(&query_lower)) ||
        self.keywords.iter().any(|k| k.contains(&query_lower))
    }

    /// Create empty searchable text
    pub fn empty() -> Self {
        Self {
            fields: Vec::new(),
            keywords: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_searchable_text_matches() {
        let text = SearchableText {
            fields: vec!["John Doe".to_string(), "john@example.com".to_string()],
            keywords: vec!["person".to_string()],
        };

        assert!(text.matches("john"));
        assert!(text.matches("JOHN")); // case insensitive
        assert!(text.matches("doe"));
        assert!(text.matches("example"));
        assert!(text.matches("person"));
        assert!(!text.matches("alice"));
    }

    #[test]
    fn test_searchable_text_empty() {
        let text = SearchableText::empty();
        assert!(text.fields.is_empty());
        assert!(text.keywords.is_empty());
        assert!(!text.matches("anything"));
    }

    #[test]
    fn test_searchable_text_new() {
        let text = SearchableText::new(
            vec!["Alice".to_string()],
            vec!["user".to_string()],
        );
        assert_eq!(text.fields.len(), 1);
        assert_eq!(text.keywords.len(), 1);
        assert!(text.matches("alice"));
        assert!(text.matches("user"));
    }
}
