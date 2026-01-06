// Copyright (c) 2025 - Cowboy AI, LLC.

//! Person Name Value Objects
//!
//! Provides structured person name value objects that separate name components
//! for proper handling of international naming conventions.
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::value_objects::common::{PersonName, GivenName, FamilyName};
//!
//! let name = PersonName::new(
//!     GivenName::new("Alice")?,
//!     FamilyName::new("Smith")?,
//! );
//!
//! assert_eq!(name.display_name(), "Alice Smith");
//! assert_eq!(name.sort_name(), "Smith, Alice");
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

// ============================================================================
// Name Component Types
// ============================================================================

/// Given name (first name) value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GivenName(String);

impl GivenName {
    /// Create a new given name
    pub fn new(name: &str) -> Result<Self, PersonNameError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(PersonNameError::EmptyName("Given name"));
        }
        if name.len() > 100 {
            return Err(PersonNameError::NameTooLong("Given name", 100));
        }
        Ok(Self(name.to_string()))
    }

    /// Get the name string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the initial (first character uppercase)
    pub fn initial(&self) -> Option<char> {
        self.0.chars().next().map(|c| c.to_ascii_uppercase())
    }
}

impl fmt::Display for GivenName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for GivenName {}
impl ValueObject for GivenName {}

/// Family name (last name/surname) value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FamilyName(String);

impl FamilyName {
    /// Create a new family name
    pub fn new(name: &str) -> Result<Self, PersonNameError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(PersonNameError::EmptyName("Family name"));
        }
        if name.len() > 100 {
            return Err(PersonNameError::NameTooLong("Family name", 100));
        }
        Ok(Self(name.to_string()))
    }

    /// Get the name string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the initial (first character uppercase)
    pub fn initial(&self) -> Option<char> {
        self.0.chars().next().map(|c| c.to_ascii_uppercase())
    }
}

impl fmt::Display for FamilyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for FamilyName {}
impl ValueObject for FamilyName {}

/// Middle name value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MiddleName(String);

impl MiddleName {
    /// Create a new middle name
    pub fn new(name: &str) -> Result<Self, PersonNameError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(PersonNameError::EmptyName("Middle name"));
        }
        if name.len() > 100 {
            return Err(PersonNameError::NameTooLong("Middle name", 100));
        }
        Ok(Self(name.to_string()))
    }

    /// Get the name string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the initial (first character uppercase)
    pub fn initial(&self) -> Option<char> {
        self.0.chars().next().map(|c| c.to_ascii_uppercase())
    }
}

impl fmt::Display for MiddleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for MiddleName {}
impl ValueObject for MiddleName {}

/// Name prefix (Mr., Mrs., Dr., etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamePrefix(String);

impl NamePrefix {
    /// Create a new name prefix
    pub fn new(prefix: &str) -> Result<Self, PersonNameError> {
        let prefix = prefix.trim();
        if prefix.is_empty() {
            return Err(PersonNameError::EmptyName("Name prefix"));
        }
        if prefix.len() > 20 {
            return Err(PersonNameError::NameTooLong("Name prefix", 20));
        }
        Ok(Self(prefix.to_string()))
    }

    /// Get the prefix string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Common prefixes
    pub fn mr() -> Self {
        Self("Mr.".to_string())
    }
    pub fn mrs() -> Self {
        Self("Mrs.".to_string())
    }
    pub fn ms() -> Self {
        Self("Ms.".to_string())
    }
    pub fn dr() -> Self {
        Self("Dr.".to_string())
    }
    pub fn prof() -> Self {
        Self("Prof.".to_string())
    }
}

impl fmt::Display for NamePrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for NamePrefix {}
impl ValueObject for NamePrefix {}

/// Name suffix (Jr., Sr., III, PhD, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NameSuffix(String);

impl NameSuffix {
    /// Create a new name suffix
    pub fn new(suffix: &str) -> Result<Self, PersonNameError> {
        let suffix = suffix.trim();
        if suffix.is_empty() {
            return Err(PersonNameError::EmptyName("Name suffix"));
        }
        if suffix.len() > 20 {
            return Err(PersonNameError::NameTooLong("Name suffix", 20));
        }
        Ok(Self(suffix.to_string()))
    }

    /// Get the suffix string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Common suffixes
    pub fn jr() -> Self {
        Self("Jr.".to_string())
    }
    pub fn sr() -> Self {
        Self("Sr.".to_string())
    }
    pub fn ii() -> Self {
        Self("II".to_string())
    }
    pub fn iii() -> Self {
        Self("III".to_string())
    }
    pub fn phd() -> Self {
        Self("PhD".to_string())
    }
    pub fn md() -> Self {
        Self("MD".to_string())
    }
    pub fn esq() -> Self {
        Self("Esq.".to_string())
    }
}

impl fmt::Display for NameSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DomainConcept for NameSuffix {}
impl ValueObject for NameSuffix {}

// ============================================================================
// Full Person Name
// ============================================================================

/// Complete person name value object
///
/// Combines name components into a structured representation that supports
/// various display and sorting formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonName {
    /// Name prefix (Mr., Dr., etc.)
    pub prefix: Option<NamePrefix>,
    /// Given name (first name)
    pub given_name: GivenName,
    /// Middle name(s)
    pub middle_name: Option<MiddleName>,
    /// Family name (last name)
    pub family_name: FamilyName,
    /// Name suffix (Jr., PhD, etc.)
    pub suffix: Option<NameSuffix>,
    /// Preferred display name (nickname)
    pub preferred_name: Option<String>,
}

impl PersonName {
    /// Create a new person name with required fields
    pub fn new(given_name: GivenName, family_name: FamilyName) -> Self {
        Self {
            prefix: None,
            given_name,
            middle_name: None,
            family_name,
            suffix: None,
            preferred_name: None,
        }
    }

    /// Builder pattern - add prefix
    pub fn with_prefix(mut self, prefix: NamePrefix) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// Builder pattern - add middle name
    pub fn with_middle_name(mut self, middle: MiddleName) -> Self {
        self.middle_name = Some(middle);
        self
    }

    /// Builder pattern - add suffix
    pub fn with_suffix(mut self, suffix: NameSuffix) -> Self {
        self.suffix = Some(suffix);
        self
    }

    /// Builder pattern - add preferred name
    pub fn with_preferred_name(mut self, name: &str) -> Self {
        self.preferred_name = Some(name.to_string());
        self
    }

    /// Get display name (what to show in UI)
    ///
    /// Uses preferred name if set, otherwise "Given Family"
    pub fn display_name(&self) -> String {
        if let Some(ref preferred) = self.preferred_name {
            return preferred.clone();
        }
        format!("{} {}", self.given_name, self.family_name)
    }

    /// Get full formal name with all components
    ///
    /// Format: "Prefix Given Middle Family, Suffix"
    pub fn full_name(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref prefix) = self.prefix {
            parts.push(prefix.to_string());
        }
        parts.push(self.given_name.to_string());
        if let Some(ref middle) = self.middle_name {
            parts.push(middle.to_string());
        }
        parts.push(self.family_name.to_string());

        let base = parts.join(" ");

        if let Some(ref suffix) = self.suffix {
            format!("{}, {}", base, suffix)
        } else {
            base
        }
    }

    /// Get sort name (for alphabetical sorting)
    ///
    /// Format: "Family, Given"
    pub fn sort_name(&self) -> String {
        format!("{}, {}", self.family_name, self.given_name)
    }

    /// Get initials
    ///
    /// Format: "GF" or "GMF" if middle name exists
    pub fn initials(&self) -> String {
        let mut initials = String::new();
        if let Some(c) = self.given_name.initial() {
            initials.push(c);
        }
        if let Some(ref middle) = self.middle_name {
            if let Some(c) = middle.initial() {
                initials.push(c);
            }
        }
        if let Some(c) = self.family_name.initial() {
            initials.push(c);
        }
        initials
    }

    /// Get short name (Given + Family initial)
    ///
    /// Format: "Given F."
    pub fn short_name(&self) -> String {
        if let Some(initial) = self.family_name.initial() {
            format!("{} {}.", self.given_name, initial)
        } else {
            self.given_name.to_string()
        }
    }

    // ========================================================================
    // Convenience Constructors
    // ========================================================================

    /// Create from simple string "Given Family"
    pub fn from_display(name: &str) -> Result<Self, PersonNameError> {
        let name = name.trim();
        let parts: Vec<&str> = name.split_whitespace().collect();

        if parts.len() < 2 {
            return Err(PersonNameError::InsufficientNameParts);
        }

        let given = GivenName::new(parts[0])?;
        let family = FamilyName::new(parts[parts.len() - 1])?;

        let middle = if parts.len() > 2 {
            // Everything between first and last is middle name
            let middle_parts: Vec<&str> = parts[1..parts.len() - 1].to_vec();
            Some(MiddleName::new(&middle_parts.join(" "))?)
        } else {
            None
        };

        Ok(Self {
            prefix: None,
            given_name: given,
            middle_name: middle,
            family_name: family,
            suffix: None,
            preferred_name: None,
        })
    }
}

impl fmt::Display for PersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl DomainConcept for PersonName {}
impl ValueObject for PersonName {}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur when creating person names
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonNameError {
    /// Name component is empty
    EmptyName(&'static str),
    /// Name component is too long
    NameTooLong(&'static str, usize),
    /// Not enough parts to parse name
    InsufficientNameParts,
}

impl fmt::Display for PersonNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonNameError::EmptyName(field) => write!(f, "{} cannot be empty", field),
            PersonNameError::NameTooLong(field, max) => {
                write!(f, "{} cannot exceed {} characters", field, max)
            }
            PersonNameError::InsufficientNameParts => {
                write!(f, "Name must have at least given name and family name")
            }
        }
    }
}

impl std::error::Error for PersonNameError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_name() {
        let name = PersonName::new(
            GivenName::new("Alice").unwrap(),
            FamilyName::new("Smith").unwrap(),
        );

        assert_eq!(name.display_name(), "Alice Smith");
        assert_eq!(name.sort_name(), "Smith, Alice");
        assert_eq!(name.initials(), "AS");
    }

    #[test]
    fn test_full_name() {
        let name = PersonName::new(
            GivenName::new("John").unwrap(),
            FamilyName::new("Doe").unwrap(),
        )
        .with_prefix(NamePrefix::dr())
        .with_middle_name(MiddleName::new("William").unwrap())
        .with_suffix(NameSuffix::jr());

        assert_eq!(name.full_name(), "Dr. John William Doe, Jr.");
        assert_eq!(name.initials(), "JWD");
    }

    #[test]
    fn test_preferred_name() {
        let name = PersonName::new(
            GivenName::new("William").unwrap(),
            FamilyName::new("Smith").unwrap(),
        )
        .with_preferred_name("Bill");

        assert_eq!(name.display_name(), "Bill");
    }

    #[test]
    fn test_short_name() {
        let name = PersonName::new(
            GivenName::new("Alice").unwrap(),
            FamilyName::new("Smith").unwrap(),
        );

        assert_eq!(name.short_name(), "Alice S.");
    }

    #[test]
    fn test_from_display() {
        let name = PersonName::from_display("Alice Marie Smith").unwrap();

        assert_eq!(name.given_name.as_str(), "Alice");
        assert_eq!(name.middle_name.as_ref().unwrap().as_str(), "Marie");
        assert_eq!(name.family_name.as_str(), "Smith");
    }

    #[test]
    fn test_from_display_simple() {
        let name = PersonName::from_display("Alice Smith").unwrap();

        assert_eq!(name.given_name.as_str(), "Alice");
        assert!(name.middle_name.is_none());
        assert_eq!(name.family_name.as_str(), "Smith");
    }

    #[test]
    fn test_from_display_insufficient() {
        assert!(PersonName::from_display("Alice").is_err());
    }

    #[test]
    fn test_empty_name_error() {
        assert!(GivenName::new("").is_err());
        assert!(FamilyName::new("").is_err());
        assert!(GivenName::new("   ").is_err());
    }

    #[test]
    fn test_name_prefixes() {
        assert_eq!(NamePrefix::dr().as_str(), "Dr.");
        assert_eq!(NamePrefix::mr().as_str(), "Mr.");
        assert_eq!(NamePrefix::prof().as_str(), "Prof.");
    }

    #[test]
    fn test_name_suffixes() {
        assert_eq!(NameSuffix::jr().as_str(), "Jr.");
        assert_eq!(NameSuffix::phd().as_str(), "PhD");
        assert_eq!(NameSuffix::iii().as_str(), "III");
    }

    #[test]
    fn test_display_trait() {
        let name = PersonName::new(
            GivenName::new("Alice").unwrap(),
            FamilyName::new("Smith").unwrap(),
        );

        assert_eq!(format!("{}", name), "Alice Smith");
    }
}
