// Copyright (c) 2025 - Cowboy AI, LLC.

//! Email Address Value Object (RFC 5321/5322)
//!
//! Provides a validated email address value object that enforces RFC compliance
//! for email address formatting.
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::value_objects::common::Email;
//!
//! let email = Email::new("user@example.com")?;
//! assert_eq!(email.local_part(), "user");
//! assert_eq!(email.domain(), "example.com");
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

/// Email address value object
///
/// Represents a validated email address per RFC 5321/5322.
/// Email addresses are stored in lowercase for consistent comparison.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email {
    /// The complete email address (lowercase)
    address: String,
    /// The local part (before @)
    local_part: String,
    /// The domain part (after @)
    domain: String,
}

impl Email {
    /// Create a new email address after validation
    pub fn new(email: &str) -> Result<Self, EmailError> {
        let email = email.trim();

        if email.is_empty() {
            return Err(EmailError::Empty);
        }

        // Check for @ symbol
        if !email.contains('@') {
            return Err(EmailError::MissingAtSymbol);
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(EmailError::MultipleAtSymbols);
        }

        let local_part = parts[0];
        let domain = parts[1];

        // Validate local part
        if local_part.is_empty() {
            return Err(EmailError::EmptyLocalPart);
        }
        if local_part.len() > 64 {
            return Err(EmailError::LocalPartTooLong);
        }
        if !Self::is_valid_local_part(local_part) {
            return Err(EmailError::InvalidLocalPart(local_part.to_string()));
        }

        // Validate domain
        if domain.is_empty() {
            return Err(EmailError::EmptyDomain);
        }
        if domain.len() > 253 {
            return Err(EmailError::DomainTooLong);
        }
        if !domain.contains('.') {
            return Err(EmailError::DomainMissingDot);
        }
        if !Self::is_valid_domain(domain) {
            return Err(EmailError::InvalidDomain(domain.to_string()));
        }

        // Store in lowercase for consistent comparison
        let address = email.to_lowercase();
        let local_part = local_part.to_lowercase();
        let domain = domain.to_lowercase();

        Ok(Self {
            address,
            local_part,
            domain,
        })
    }

    /// Get the complete email address
    pub fn as_str(&self) -> &str {
        &self.address
    }

    /// Get the local part (before @)
    pub fn local_part(&self) -> &str {
        &self.local_part
    }

    /// Get the domain part (after @)
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Check if this is a valid local part per RFC 5321
    fn is_valid_local_part(local: &str) -> bool {
        if local.is_empty() {
            return false;
        }

        // Allow alphanumeric and common special chars
        // RFC 5321 allows: ! # $ % & ' * + - / = ? ^ _ ` { | } ~
        let valid_chars = |c: char| {
            c.is_alphanumeric()
                || c == '.'
                || c == '!'
                || c == '#'
                || c == '$'
                || c == '%'
                || c == '&'
                || c == '\''
                || c == '*'
                || c == '+'
                || c == '-'
                || c == '/'
                || c == '='
                || c == '?'
                || c == '^'
                || c == '_'
                || c == '`'
                || c == '{'
                || c == '|'
                || c == '}'
                || c == '~'
        };

        // Cannot start or end with dot
        if local.starts_with('.') || local.ends_with('.') {
            return false;
        }

        // Cannot have consecutive dots
        if local.contains("..") {
            return false;
        }

        local.chars().all(valid_chars)
    }

    /// Check if this is a valid domain per RFC 5321
    fn is_valid_domain(domain: &str) -> bool {
        if domain.is_empty() {
            return false;
        }

        // Each label must be 1-63 chars
        let labels: Vec<&str> = domain.split('.').collect();
        if labels.is_empty() {
            return false;
        }

        for label in labels {
            if label.is_empty() || label.len() > 63 {
                return false;
            }
            // Must start with alphanumeric
            if !label.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                return false;
            }
            // Must end with alphanumeric
            if !label.chars().last().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                return false;
            }
            // Only alphanumeric and hyphen allowed
            if !label.chars().all(|c| c.is_alphanumeric() || c == '-') {
                return false;
            }
        }

        true
    }

    /// Check if this is a common public email provider
    pub fn is_public_provider(&self) -> bool {
        let public_domains = [
            "gmail.com",
            "googlemail.com",
            "yahoo.com",
            "hotmail.com",
            "outlook.com",
            "live.com",
            "msn.com",
            "icloud.com",
            "me.com",
            "aol.com",
            "protonmail.com",
            "proton.me",
            "mail.com",
            "zoho.com",
        ];
        public_domains.contains(&self.domain.as_str())
    }

    /// Check if this appears to be a business email (not public provider)
    pub fn is_business_email(&self) -> bool {
        !self.is_public_provider()
    }

    /// Get the organization domain (for business emails)
    ///
    /// Returns None for public providers, returns domain for business emails.
    pub fn organization_domain(&self) -> Option<&str> {
        if self.is_business_email() {
            Some(&self.domain)
        } else {
            None
        }
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl FromStr for Email {
    type Err = EmailError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Email::new(s)
    }
}

impl DomainConcept for Email {}
impl ValueObject for Email {}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur when creating an email address
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailError {
    /// Email string is empty
    Empty,
    /// Missing @ symbol
    MissingAtSymbol,
    /// Multiple @ symbols found
    MultipleAtSymbols,
    /// Local part is empty
    EmptyLocalPart,
    /// Local part exceeds 64 characters
    LocalPartTooLong,
    /// Local part contains invalid characters
    InvalidLocalPart(String),
    /// Domain is empty
    EmptyDomain,
    /// Domain exceeds 253 characters
    DomainTooLong,
    /// Domain has no dots
    DomainMissingDot,
    /// Domain contains invalid characters
    InvalidDomain(String),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::Empty => write!(f, "Email address cannot be empty"),
            EmailError::MissingAtSymbol => write!(f, "Email address must contain @"),
            EmailError::MultipleAtSymbols => write!(f, "Email address cannot contain multiple @"),
            EmailError::EmptyLocalPart => write!(f, "Email local part cannot be empty"),
            EmailError::LocalPartTooLong => {
                write!(f, "Email local part cannot exceed 64 characters")
            }
            EmailError::InvalidLocalPart(s) => {
                write!(f, "Email local part contains invalid characters: {}", s)
            }
            EmailError::EmptyDomain => write!(f, "Email domain cannot be empty"),
            EmailError::DomainTooLong => write!(f, "Email domain cannot exceed 253 characters"),
            EmailError::DomainMissingDot => write!(f, "Email domain must contain at least one dot"),
            EmailError::InvalidDomain(s) => {
                write!(f, "Email domain contains invalid characters: {}", s)
            }
        }
    }
}

impl std::error::Error for EmailError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(Email::new("user@example.com").is_ok());
        assert!(Email::new("first.last@example.com").is_ok());
        assert!(Email::new("user+tag@example.com").is_ok());
        assert!(Email::new("user@subdomain.example.com").is_ok());
        assert!(Email::new("USER@EXAMPLE.COM").is_ok()); // Should normalize to lowercase
    }

    #[test]
    fn test_invalid_emails() {
        assert!(Email::new("").is_err());
        assert!(Email::new("noatsign").is_err());
        assert!(Email::new("@nodomain").is_err());
        assert!(Email::new("nolocal@").is_err());
        assert!(Email::new("user@nodot").is_err());
        assert!(Email::new("user@@example.com").is_err());
        assert!(Email::new(".user@example.com").is_err());
        assert!(Email::new("user.@example.com").is_err());
        assert!(Email::new("user..name@example.com").is_err());
    }

    #[test]
    fn test_email_parts() {
        let email = Email::new("User@Example.COM").unwrap();
        assert_eq!(email.local_part(), "user");
        assert_eq!(email.domain(), "example.com");
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_email_display() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(format!("{}", email), "user@example.com");
    }

    #[test]
    fn test_email_from_str() {
        let email: Email = "user@example.com".parse().unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_public_provider() {
        let gmail = Email::new("user@gmail.com").unwrap();
        assert!(gmail.is_public_provider());
        assert!(!gmail.is_business_email());
        assert!(gmail.organization_domain().is_none());
    }

    #[test]
    fn test_business_email() {
        let business = Email::new("user@company.com").unwrap();
        assert!(!business.is_public_provider());
        assert!(business.is_business_email());
        assert_eq!(business.organization_domain(), Some("company.com"));
    }

    #[test]
    fn test_email_equality() {
        let email1 = Email::new("User@Example.COM").unwrap();
        let email2 = Email::new("user@example.com").unwrap();
        assert_eq!(email1, email2); // Should be equal after normalization
    }

    #[test]
    fn test_special_chars_in_local() {
        assert!(Email::new("user+tag@example.com").is_ok());
        assert!(Email::new("user.name@example.com").is_ok());
        assert!(Email::new("user_name@example.com").is_ok());
        assert!(Email::new("user-name@example.com").is_ok());
    }
}
