// Copyright (c) 2025 - Cowboy AI, LLC.

//! URI Value Object (RFC 3986)
//!
//! Provides a validated URI value object that parses and exposes URI components
//! per RFC 3986 specification.
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::value_objects::common::Uri;
//!
//! let uri = Uri::new("https://example.com:8080/path?query=value#fragment")?;
//! assert_eq!(uri.scheme(), "https");
//! assert_eq!(uri.host(), Some("example.com"));
//! assert_eq!(uri.port(), Some(8080));
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

/// URI value object
///
/// Represents a validated Uniform Resource Identifier per RFC 3986.
/// Parses and provides access to individual URI components.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Uri {
    /// The complete URI string
    uri: String,
    /// The scheme (http, https, etc.)
    scheme: String,
    /// The authority (host and optional port)
    authority: Option<String>,
    /// The host name
    host: Option<String>,
    /// The port number
    port: Option<u16>,
    /// The path component
    path: String,
    /// The query string (without ?)
    query: Option<String>,
    /// The fragment (without #)
    fragment: Option<String>,
}

impl Uri {
    /// Create a new URI after validation and parsing
    pub fn new(uri: &str) -> Result<Self, UriError> {
        let uri = uri.trim();

        if uri.is_empty() {
            return Err(UriError::Empty);
        }

        // Must have scheme://
        let scheme_end = uri.find("://").ok_or(UriError::MissingScheme)?;
        let scheme = uri[..scheme_end].to_lowercase();

        if scheme.is_empty() {
            return Err(UriError::EmptyScheme);
        }

        // Validate scheme (must start with letter, then alphanumeric + . - +)
        if !Self::is_valid_scheme(&scheme) {
            return Err(UriError::InvalidScheme(scheme));
        }

        let rest = &uri[scheme_end + 3..];

        // Parse authority (everything before first / or ? or # or end)
        let (authority_str, remainder) = Self::split_authority(rest);

        let (host, port) = if let Some(auth) = &authority_str {
            Self::parse_authority(auth)?
        } else {
            (None, None)
        };

        // Parse path, query, fragment
        let (path, query, fragment) = Self::parse_path_query_fragment(remainder);

        Ok(Self {
            uri: uri.to_string(),
            scheme,
            authority: authority_str,
            host,
            port,
            path,
            query,
            fragment,
        })
    }

    /// Get the complete URI string
    pub fn as_str(&self) -> &str {
        &self.uri
    }

    /// Get the scheme (http, https, ftp, etc.)
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Get the authority (host:port)
    pub fn authority(&self) -> Option<&str> {
        self.authority.as_deref()
    }

    /// Get the host name
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// Get the port number
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Get the path component
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the query string (without leading ?)
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Get the fragment (without leading #)
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    /// Check if this is an HTTPS URI
    pub fn is_https(&self) -> bool {
        self.scheme == "https"
    }

    /// Check if this is an HTTP URI (including HTTPS)
    pub fn is_http(&self) -> bool {
        self.scheme == "http" || self.scheme == "https"
    }

    /// Check if this is a secure scheme (https, wss, etc.)
    pub fn is_secure(&self) -> bool {
        matches!(self.scheme.as_str(), "https" | "wss" | "ftps" | "ldaps")
    }

    /// Get the origin (scheme + host + port)
    pub fn origin(&self) -> Option<String> {
        self.host.as_ref().map(|h| {
            match self.port {
                Some(p) => format!("{}://{}:{}", self.scheme, h, p),
                None => format!("{}://{}", self.scheme, h),
            }
        })
    }

    /// Validate scheme characters
    fn is_valid_scheme(scheme: &str) -> bool {
        if scheme.is_empty() {
            return false;
        }
        let mut chars = scheme.chars();
        // First char must be letter
        if !chars.next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
            return false;
        }
        // Rest can be alphanumeric, +, -, .
        chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    }

    /// Split authority from the rest of the URI
    fn split_authority(s: &str) -> (Option<String>, &str) {
        if s.is_empty() {
            return (None, s);
        }

        // Find the end of authority (first /, ?, #, or end)
        let end = s
            .find('/')
            .or_else(|| s.find('?'))
            .or_else(|| s.find('#'))
            .unwrap_or(s.len());

        if end == 0 {
            (None, s)
        } else {
            (Some(s[..end].to_string()), &s[end..])
        }
    }

    /// Parse authority into host and optional port
    fn parse_authority(auth: &str) -> Result<(Option<String>, Option<u16>), UriError> {
        if auth.is_empty() {
            return Ok((None, None));
        }

        // Handle IPv6 addresses [host]:port
        if auth.starts_with('[') {
            let bracket_end = auth.find(']').ok_or(UriError::InvalidHost(auth.to_string()))?;
            let host = auth[1..bracket_end].to_string();

            let port = if auth.len() > bracket_end + 1 && auth[bracket_end + 1..].starts_with(':') {
                let port_str = &auth[bracket_end + 2..];
                Some(
                    port_str
                        .parse::<u16>()
                        .map_err(|_| UriError::InvalidPort(port_str.to_string()))?,
                )
            } else {
                None
            };

            return Ok((Some(host), port));
        }

        // Regular host:port
        if let Some(colon_pos) = auth.rfind(':') {
            // Check if this looks like a port (all digits after colon)
            let potential_port = &auth[colon_pos + 1..];
            if !potential_port.is_empty() && potential_port.chars().all(|c| c.is_ascii_digit()) {
                let host = auth[..colon_pos].to_string();
                let port = potential_port
                    .parse::<u16>()
                    .map_err(|_| UriError::InvalidPort(potential_port.to_string()))?;
                return Ok((Some(host), Some(port)));
            }
        }

        // Just host, no port
        Ok((Some(auth.to_string()), None))
    }

    /// Parse path, query, and fragment
    fn parse_path_query_fragment(s: &str) -> (String, Option<String>, Option<String>) {
        let (path_query, fragment) = if let Some(hash_pos) = s.find('#') {
            (&s[..hash_pos], Some(s[hash_pos + 1..].to_string()))
        } else {
            (s, None)
        };

        let (path, query) = if let Some(q_pos) = path_query.find('?') {
            (
                path_query[..q_pos].to_string(),
                Some(path_query[q_pos + 1..].to_string()),
            )
        } else {
            (path_query.to_string(), None)
        };

        (path, query, fragment)
    }

    // ========================================================================
    // Convenience Constructors
    // ========================================================================

    /// Create an HTTPS URI
    pub fn https(host: &str, path: &str) -> Result<Self, UriError> {
        let uri = if path.is_empty() || path == "/" {
            format!("https://{}", host)
        } else if path.starts_with('/') {
            format!("https://{}{}", host, path)
        } else {
            format!("https://{}/{}", host, path)
        };
        Self::new(&uri)
    }

    /// Create an HTTP URI
    pub fn http(host: &str, path: &str) -> Result<Self, UriError> {
        let uri = if path.is_empty() || path == "/" {
            format!("http://{}", host)
        } else if path.starts_with('/') {
            format!("http://{}{}", host, path)
        } else {
            format!("http://{}/{}", host, path)
        };
        Self::new(&uri)
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl FromStr for Uri {
    type Err = UriError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uri::new(s)
    }
}

impl DomainConcept for Uri {}
impl ValueObject for Uri {}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur when creating a URI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriError {
    /// URI string is empty
    Empty,
    /// Missing scheme (no ://)
    MissingScheme,
    /// Scheme is empty
    EmptyScheme,
    /// Invalid scheme format
    InvalidScheme(String),
    /// Invalid host format
    InvalidHost(String),
    /// Invalid port number
    InvalidPort(String),
}

impl fmt::Display for UriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UriError::Empty => write!(f, "URI cannot be empty"),
            UriError::MissingScheme => write!(f, "URI must include scheme (e.g., https://)"),
            UriError::EmptyScheme => write!(f, "URI scheme cannot be empty"),
            UriError::InvalidScheme(s) => write!(f, "Invalid URI scheme: {}", s),
            UriError::InvalidHost(s) => write!(f, "Invalid URI host: {}", s),
            UriError::InvalidPort(s) => write!(f, "Invalid URI port: {}", s),
        }
    }
}

impl std::error::Error for UriError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_uri() {
        let uri = Uri::new("https://example.com").unwrap();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host(), Some("example.com"));
        assert_eq!(uri.port(), None);
        assert_eq!(uri.path(), "");
    }

    #[test]
    fn test_uri_with_port() {
        let uri = Uri::new("https://example.com:8080").unwrap();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host(), Some("example.com"));
        assert_eq!(uri.port(), Some(8080));
    }

    #[test]
    fn test_uri_with_path() {
        let uri = Uri::new("https://example.com/path/to/resource").unwrap();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host(), Some("example.com"));
        assert_eq!(uri.path(), "/path/to/resource");
    }

    #[test]
    fn test_uri_with_query() {
        let uri = Uri::new("https://example.com/path?query=value&foo=bar").unwrap();
        assert_eq!(uri.path(), "/path");
        assert_eq!(uri.query(), Some("query=value&foo=bar"));
    }

    #[test]
    fn test_uri_with_fragment() {
        let uri = Uri::new("https://example.com/path#section").unwrap();
        assert_eq!(uri.path(), "/path");
        assert_eq!(uri.fragment(), Some("section"));
    }

    #[test]
    fn test_full_uri() {
        let uri =
            Uri::new("https://example.com:8080/path/to/resource?query=value#fragment").unwrap();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host(), Some("example.com"));
        assert_eq!(uri.port(), Some(8080));
        assert_eq!(uri.path(), "/path/to/resource");
        assert_eq!(uri.query(), Some("query=value"));
        assert_eq!(uri.fragment(), Some("fragment"));
    }

    #[test]
    fn test_invalid_uris() {
        assert!(Uri::new("").is_err());
        assert!(Uri::new("no-scheme").is_err());
        assert!(Uri::new("://missing-scheme").is_err());
    }

    #[test]
    fn test_uri_is_secure() {
        assert!(Uri::new("https://example.com").unwrap().is_secure());
        assert!(Uri::new("wss://example.com").unwrap().is_secure());
        assert!(!Uri::new("http://example.com").unwrap().is_secure());
        assert!(!Uri::new("ws://example.com").unwrap().is_secure());
    }

    #[test]
    fn test_uri_origin() {
        let uri = Uri::new("https://example.com:8080/path").unwrap();
        assert_eq!(uri.origin(), Some("https://example.com:8080".to_string()));

        let uri2 = Uri::new("https://example.com/path").unwrap();
        assert_eq!(uri2.origin(), Some("https://example.com".to_string()));
    }

    #[test]
    fn test_uri_display() {
        let uri = Uri::new("https://example.com/path").unwrap();
        assert_eq!(format!("{}", uri), "https://example.com/path");
    }

    #[test]
    fn test_uri_from_str() {
        let uri: Uri = "https://example.com".parse().unwrap();
        assert_eq!(uri.scheme(), "https");
    }

    #[test]
    fn test_https_convenience() {
        let uri = Uri::https("example.com", "/api/v1").unwrap();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host(), Some("example.com"));
        assert_eq!(uri.path(), "/api/v1");
    }

    #[test]
    fn test_http_convenience() {
        let uri = Uri::http("localhost:8080", "/health").unwrap();
        assert_eq!(uri.scheme(), "http");
        assert_eq!(uri.host(), Some("localhost"));
        assert_eq!(uri.port(), Some(8080));
        assert_eq!(uri.path(), "/health");
    }

    #[test]
    fn test_various_schemes() {
        assert!(Uri::new("ftp://ftp.example.com/file").is_ok());
        assert!(Uri::new("ldap://ldap.example.com").is_ok());
        assert!(Uri::new("ssh://git@github.com/repo").is_ok());
        assert!(Uri::new("file:///path/to/file").is_ok());
    }

    #[test]
    fn test_ipv6_host() {
        let uri = Uri::new("https://[::1]:8080/path").unwrap();
        assert_eq!(uri.host(), Some("::1"));
        assert_eq!(uri.port(), Some(8080));
    }
}
