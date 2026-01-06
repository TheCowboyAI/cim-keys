// Copyright (c) 2025 - Cowboy AI, LLC.

//! Subject Alternative Name Extension (RFC 5280 Section 4.2.1.6)
//!
//! Provides type-safe Subject Alternative Name (SAN) entries that comply
//! with RFC 5280. SANs provide additional identities for the certificate
//! subject beyond the Distinguished Name.
//!
//! ## Supported SAN Types
//!
//! - `DnsName` - Fully qualified domain names
//! - `IpAddress` - IPv4 or IPv6 addresses
//! - `Uri` - Uniform Resource Identifiers
//! - `Email` - RFC 822 email addresses (rfc822Name)
//! - `DirectoryName` - X.500 Distinguished Names
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::value_objects::x509::*;
//!
//! let san = SubjectAlternativeName::new()
//!     .with_dns_name("example.com")?
//!     .with_dns_name("*.example.com")?
//!     .with_ip_address("192.168.1.1")?;
//!
//! assert!(san.has_wildcard());
//! assert_eq!(san.dns_names().count(), 2);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

// ============================================================================
// SAN Entry Types
// ============================================================================

/// DNS Name entry in Subject Alternative Name
///
/// Represents a fully qualified domain name or wildcard domain.
/// Per RFC 5280, DNS names must be in the preferred name syntax.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DnsName(String);

impl DnsName {
    /// Create a new DNS name after validation
    pub fn new(name: &str) -> Result<Self, SanError> {
        let name = name.trim().to_lowercase();

        if name.is_empty() {
            return Err(SanError::EmptyValue("DNS name"));
        }

        // Check for valid DNS name characters
        if !Self::is_valid_dns_name(&name) {
            return Err(SanError::InvalidDnsName(name));
        }

        Ok(Self(name))
    }

    /// Check if this is a wildcard DNS name (starts with *.)
    pub fn is_wildcard(&self) -> bool {
        self.0.starts_with("*.")
    }

    /// Get the base domain (without wildcard prefix)
    pub fn base_domain(&self) -> &str {
        if self.is_wildcard() {
            &self.0[2..]
        } else {
            &self.0
        }
    }

    /// Get the full DNS name string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate DNS name format per RFC 1123 / RFC 5280
    fn is_valid_dns_name(name: &str) -> bool {
        // Handle wildcard
        let check_name = if name.starts_with("*.") {
            &name[2..]
        } else {
            name
        };

        if check_name.is_empty() {
            return false;
        }

        // Each label must be 1-63 chars, total max 253
        if check_name.len() > 253 {
            return false;
        }

        let labels: Vec<&str> = check_name.split('.').collect();
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
}

impl fmt::Display for DnsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DNS:{}", self.0)
    }
}

impl DomainConcept for DnsName {}
impl ValueObject for DnsName {}

/// IP Address entry in Subject Alternative Name
///
/// Represents an IPv4 or IPv6 address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SanIpAddress(IpAddr);

impl SanIpAddress {
    /// Create from an IpAddr
    pub fn new(addr: IpAddr) -> Self {
        Self(addr)
    }

    /// Create from an IPv4 address
    pub fn from_ipv4(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self(IpAddr::V4(Ipv4Addr::new(a, b, c, d)))
    }

    /// Create from an IPv6 address
    pub fn from_ipv6(segments: [u16; 8]) -> Self {
        Self(IpAddr::V6(Ipv6Addr::new(
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            segments[4],
            segments[5],
            segments[6],
            segments[7],
        )))
    }

    /// Parse from string
    pub fn parse(s: &str) -> Result<Self, SanError> {
        let addr =
            IpAddr::from_str(s.trim()).map_err(|_| SanError::InvalidIpAddress(s.to_string()))?;
        Ok(Self(addr))
    }

    /// Check if this is an IPv4 address
    pub fn is_ipv4(&self) -> bool {
        self.0.is_ipv4()
    }

    /// Check if this is an IPv6 address
    pub fn is_ipv6(&self) -> bool {
        self.0.is_ipv6()
    }

    /// Get the underlying IpAddr
    pub fn addr(&self) -> &IpAddr {
        &self.0
    }
}

impl fmt::Display for SanIpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IP:{}", self.0)
    }
}

impl DomainConcept for SanIpAddress {}
impl ValueObject for SanIpAddress {}

/// URI entry in Subject Alternative Name
///
/// Represents a Uniform Resource Identifier per RFC 3986.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SanUri(String);

impl SanUri {
    /// Create a new URI after basic validation
    pub fn new(uri: &str) -> Result<Self, SanError> {
        let uri = uri.trim();

        if uri.is_empty() {
            return Err(SanError::EmptyValue("URI"));
        }

        // Basic URI validation - must have scheme://
        if !uri.contains("://") {
            return Err(SanError::InvalidUri(uri.to_string()));
        }

        Ok(Self(uri.to_string()))
    }

    /// Get the URI string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the scheme (http, https, etc.)
    pub fn scheme(&self) -> Option<&str> {
        self.0.split("://").next()
    }
}

impl fmt::Display for SanUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "URI:{}", self.0)
    }
}

impl DomainConcept for SanUri {}
impl ValueObject for SanUri {}

/// Email entry in Subject Alternative Name (rfc822Name)
///
/// Represents an email address per RFC 5321/5322.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SanEmail(String);

impl SanEmail {
    /// Create a new email after validation
    pub fn new(email: &str) -> Result<Self, SanError> {
        let email = email.trim().to_lowercase();

        if email.is_empty() {
            return Err(SanError::EmptyValue("Email"));
        }

        // Basic email validation
        if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
            return Err(SanError::InvalidEmail(email));
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(SanError::InvalidEmail(email));
        }

        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() || domain.is_empty() {
            return Err(SanError::InvalidEmail(email));
        }

        // Domain must have at least one dot
        if !domain.contains('.') {
            return Err(SanError::InvalidEmail(email));
        }

        Ok(Self(email))
    }

    /// Get the email string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the local part (before @)
    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }

    /// Get the domain part (after @)
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
}

impl fmt::Display for SanEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "email:{}", self.0)
    }
}

impl DomainConcept for SanEmail {}
impl ValueObject for SanEmail {}

// ============================================================================
// SAN Entry Enum
// ============================================================================

/// A single Subject Alternative Name entry
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SanEntry {
    /// DNS Name (dNSName)
    DnsName(DnsName),
    /// IP Address (iPAddress)
    IpAddress(SanIpAddress),
    /// URI (uniformResourceIdentifier)
    Uri(SanUri),
    /// Email (rfc822Name)
    Email(SanEmail),
    /// Other/custom entry type with OID
    Other { oid: String, value: String },
}

impl SanEntry {
    /// Get the entry type name
    pub fn type_name(&self) -> &'static str {
        match self {
            SanEntry::DnsName(_) => "DNS",
            SanEntry::IpAddress(_) => "IP",
            SanEntry::Uri(_) => "URI",
            SanEntry::Email(_) => "email",
            SanEntry::Other { .. } => "other",
        }
    }

    /// Get the value as a string
    pub fn value_string(&self) -> String {
        match self {
            SanEntry::DnsName(d) => d.as_str().to_string(),
            SanEntry::IpAddress(ip) => ip.addr().to_string(),
            SanEntry::Uri(u) => u.as_str().to_string(),
            SanEntry::Email(e) => e.as_str().to_string(),
            SanEntry::Other { value, .. } => value.clone(),
        }
    }
}

impl fmt::Display for SanEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SanEntry::DnsName(d) => write!(f, "{}", d),
            SanEntry::IpAddress(ip) => write!(f, "{}", ip),
            SanEntry::Uri(u) => write!(f, "{}", u),
            SanEntry::Email(e) => write!(f, "{}", e),
            SanEntry::Other { oid, value } => write!(f, "{}:{}", oid, value),
        }
    }
}

impl DomainConcept for SanEntry {}
impl ValueObject for SanEntry {}

// ============================================================================
// Subject Alternative Name Extension
// ============================================================================

/// Subject Alternative Name (SAN) Extension value object
///
/// Contains multiple alternative identities for the certificate subject.
/// Per RFC 5280, at least one SAN entry is required if the subject DN is empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectAlternativeName {
    /// Set of SAN entries
    entries: HashSet<SanEntry>,
    /// Whether this extension is critical
    pub critical: bool,
}

impl SubjectAlternativeName {
    /// Create a new empty SAN extension
    pub fn new() -> Self {
        Self {
            entries: HashSet::new(),
            critical: false, // RFC 5280 recommends non-critical unless subject DN is empty
        }
    }

    /// Create from a set of entries
    pub fn from_entries(entries: impl IntoIterator<Item = SanEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
            critical: false,
        }
    }

    /// Builder pattern - add a DNS name
    pub fn with_dns_name(mut self, name: &str) -> Result<Self, SanError> {
        let dns = DnsName::new(name)?;
        self.entries.insert(SanEntry::DnsName(dns));
        Ok(self)
    }

    /// Builder pattern - add an IP address (from string)
    pub fn with_ip_address(mut self, addr: &str) -> Result<Self, SanError> {
        let ip = SanIpAddress::parse(addr)?;
        self.entries.insert(SanEntry::IpAddress(ip));
        Ok(self)
    }

    /// Builder pattern - add a URI
    pub fn with_uri(mut self, uri: &str) -> Result<Self, SanError> {
        let san_uri = SanUri::new(uri)?;
        self.entries.insert(SanEntry::Uri(san_uri));
        Ok(self)
    }

    /// Builder pattern - add an email
    pub fn with_email(mut self, email: &str) -> Result<Self, SanError> {
        let san_email = SanEmail::new(email)?;
        self.entries.insert(SanEntry::Email(san_email));
        Ok(self)
    }

    /// Builder pattern - add a raw entry
    pub fn with_entry(mut self, entry: SanEntry) -> Self {
        self.entries.insert(entry);
        self
    }

    /// Builder pattern - set criticality
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }

    /// Get all entries
    pub fn entries(&self) -> impl Iterator<Item = &SanEntry> {
        self.entries.iter()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all DNS name entries
    pub fn dns_names(&self) -> impl Iterator<Item = &DnsName> {
        self.entries.iter().filter_map(|e| {
            if let SanEntry::DnsName(dns) = e {
                Some(dns)
            } else {
                None
            }
        })
    }

    /// Get all IP address entries
    pub fn ip_addresses(&self) -> impl Iterator<Item = &SanIpAddress> {
        self.entries.iter().filter_map(|e| {
            if let SanEntry::IpAddress(ip) = e {
                Some(ip)
            } else {
                None
            }
        })
    }

    /// Get all URI entries
    pub fn uris(&self) -> impl Iterator<Item = &SanUri> {
        self.entries.iter().filter_map(|e| {
            if let SanEntry::Uri(uri) = e {
                Some(uri)
            } else {
                None
            }
        })
    }

    /// Get all email entries
    pub fn emails(&self) -> impl Iterator<Item = &SanEmail> {
        self.entries.iter().filter_map(|e| {
            if let SanEntry::Email(email) = e {
                Some(email)
            } else {
                None
            }
        })
    }

    /// Check if any DNS name is a wildcard
    pub fn has_wildcard(&self) -> bool {
        self.dns_names().any(|d| d.is_wildcard())
    }

    /// Convert to string list for event serialization
    pub fn to_string_list(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.to_string()).collect()
    }

    /// Parse from string list (for backward compatibility)
    ///
    /// Expected formats:
    /// - "DNS:example.com"
    /// - "IP:192.168.1.1"
    /// - "URI:https://example.com"
    /// - "email:user@example.com"
    pub fn from_string_list(strings: &[String]) -> Self {
        let entries = strings
            .iter()
            .filter_map(|s| Self::parse_entry(s))
            .collect();
        Self {
            entries,
            critical: false,
        }
    }

    /// Parse a single SAN entry from string
    fn parse_entry(s: &str) -> Option<SanEntry> {
        let s = s.trim();
        if let Some(dns) = s.strip_prefix("DNS:") {
            DnsName::new(dns).ok().map(SanEntry::DnsName)
        } else if let Some(ip) = s.strip_prefix("IP:") {
            SanIpAddress::parse(ip).ok().map(SanEntry::IpAddress)
        } else if let Some(uri) = s.strip_prefix("URI:") {
            SanUri::new(uri).ok().map(SanEntry::Uri)
        } else if let Some(email) = s.strip_prefix("email:") {
            SanEmail::new(email).ok().map(SanEntry::Email)
        } else {
            // Try to parse as DNS name by default (common case)
            DnsName::new(s).ok().map(SanEntry::DnsName)
        }
    }

    // ========================================================================
    // Convenience Constructors
    // ========================================================================

    /// Create a SAN for a single domain
    pub fn single_domain(domain: &str) -> Result<Self, SanError> {
        Self::new().with_dns_name(domain)
    }

    /// Create a SAN for a domain with wildcard
    pub fn domain_with_wildcard(domain: &str) -> Result<Self, SanError> {
        let wildcard = format!("*.{}", domain);
        Self::new().with_dns_name(domain)?.with_dns_name(&wildcard)
    }

    /// Create a SAN for localhost development
    pub fn localhost() -> Result<Self, SanError> {
        Self::new()
            .with_dns_name("localhost")?
            .with_ip_address("127.0.0.1")?
            .with_ip_address("::1")
    }

    /// Create a SAN for a TLS server with multiple domains
    pub fn tls_server(domains: &[&str]) -> Result<Self, SanError> {
        let mut san = Self::new();
        for domain in domains {
            san = san.with_dns_name(domain)?;
        }
        Ok(san)
    }
}

impl Default for SubjectAlternativeName {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SubjectAlternativeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries: Vec<String> = self.entries.iter().map(|e| e.to_string()).collect();
        write!(f, "SAN({})", entries.join(", "))
    }
}

impl DomainConcept for SubjectAlternativeName {}
impl ValueObject for SubjectAlternativeName {}

// ============================================================================
// NodeContributor Implementation for SubjectAlternativeName (Sprint D)
// ============================================================================

impl crate::value_objects::NodeContributor for SubjectAlternativeName {
    /// Generate labels for graph node based on SAN contents
    fn as_labels(&self) -> Vec<crate::value_objects::Label> {
        use crate::value_objects::Label;
        let mut labels = Vec::new();

        if self.has_wildcard() {
            labels.push(Label::new("WildcardCertificate"));
        }
        if self.dns_names().count() > 0 {
            labels.push(Label::new("HasDnsNames"));
        }
        if self.ip_addresses().count() > 0 {
            labels.push(Label::new("HasIpAddresses"));
        }
        if self.emails().count() > 0 {
            labels.push(Label::new("HasEmailAddresses"));
        }
        if self.uris().count() > 0 {
            labels.push(Label::new("HasUris"));
        }
        if self.len() > 10 {
            labels.push(Label::new("ManySans"));
        }

        labels
    }

    /// Generate properties for graph node
    fn as_properties(&self) -> Vec<(crate::value_objects::PropertyKey, crate::value_objects::PropertyValue)> {
        use crate::value_objects::{PropertyKey, PropertyValue};

        let dns_names: Vec<String> = self.dns_names().map(|d| d.as_str().to_string()).collect();
        let ip_addresses: Vec<String> = self.ip_addresses().map(|ip| ip.addr().to_string()).collect();
        let emails: Vec<String> = self.emails().map(|e| e.as_str().to_string()).collect();
        let uris: Vec<String> = self.uris().map(|u| u.as_str().to_string()).collect();

        let mut props = vec![
            (PropertyKey::new("san_count"), PropertyValue::int(self.len() as i64)),
            (PropertyKey::new("has_wildcard"), PropertyValue::bool(self.has_wildcard())),
            (PropertyKey::new("critical"), PropertyValue::bool(self.critical)),
        ];

        if !dns_names.is_empty() {
            props.push((PropertyKey::new("dns_names"), PropertyValue::string_list(dns_names.iter().map(|s| s.as_str()))));
        }
        if !ip_addresses.is_empty() {
            props.push((PropertyKey::new("ip_addresses"), PropertyValue::string_list(ip_addresses.iter().map(|s| s.as_str()))));
        }
        if !emails.is_empty() {
            props.push((PropertyKey::new("san_emails"), PropertyValue::string_list(emails.iter().map(|s| s.as_str()))));
        }
        if !uris.is_empty() {
            props.push((PropertyKey::new("san_uris"), PropertyValue::string_list(uris.iter().map(|s| s.as_str()))));
        }

        props
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur when creating SAN entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SanError {
    /// Empty value provided
    EmptyValue(&'static str),
    /// Invalid DNS name format
    InvalidDnsName(String),
    /// Invalid IP address format
    InvalidIpAddress(String),
    /// Invalid URI format
    InvalidUri(String),
    /// Invalid email format
    InvalidEmail(String),
}

impl fmt::Display for SanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SanError::EmptyValue(field) => write!(f, "{} cannot be empty", field),
            SanError::InvalidDnsName(name) => write!(f, "Invalid DNS name: {}", name),
            SanError::InvalidIpAddress(addr) => write!(f, "Invalid IP address: {}", addr),
            SanError::InvalidUri(uri) => write!(f, "Invalid URI: {}", uri),
            SanError::InvalidEmail(email) => write!(f, "Invalid email: {}", email),
        }
    }
}

impl std::error::Error for SanError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_name_valid() {
        assert!(DnsName::new("example.com").is_ok());
        assert!(DnsName::new("sub.example.com").is_ok());
        assert!(DnsName::new("my-domain.org").is_ok());
        assert!(DnsName::new("*.example.com").is_ok());
    }

    #[test]
    fn test_dns_name_invalid() {
        assert!(DnsName::new("").is_err());
        assert!(DnsName::new("-invalid.com").is_err());
        assert!(DnsName::new("invalid-.com").is_err());
        assert!(DnsName::new("invalid..com").is_err());
    }

    #[test]
    fn test_dns_name_wildcard() {
        let wildcard = DnsName::new("*.example.com").unwrap();
        assert!(wildcard.is_wildcard());
        assert_eq!(wildcard.base_domain(), "example.com");

        let regular = DnsName::new("example.com").unwrap();
        assert!(!regular.is_wildcard());
        assert_eq!(regular.base_domain(), "example.com");
    }

    #[test]
    fn test_ip_address_v4() {
        let ip = SanIpAddress::parse("192.168.1.1").unwrap();
        assert!(ip.is_ipv4());
        assert!(!ip.is_ipv6());
    }

    #[test]
    fn test_ip_address_v6() {
        let ip = SanIpAddress::parse("::1").unwrap();
        assert!(ip.is_ipv6());
        assert!(!ip.is_ipv4());
    }

    #[test]
    fn test_ip_address_invalid() {
        assert!(SanIpAddress::parse("invalid").is_err());
        assert!(SanIpAddress::parse("999.999.999.999").is_err());
    }

    #[test]
    fn test_uri_valid() {
        assert!(SanUri::new("https://example.com").is_ok());
        assert!(SanUri::new("http://localhost:8080/path").is_ok());
        assert!(SanUri::new("ldap://directory.example.com").is_ok());
    }

    #[test]
    fn test_uri_invalid() {
        assert!(SanUri::new("").is_err());
        assert!(SanUri::new("not-a-uri").is_err());
    }

    #[test]
    fn test_email_valid() {
        assert!(SanEmail::new("user@example.com").is_ok());
        assert!(SanEmail::new("admin@sub.example.org").is_ok());
    }

    #[test]
    fn test_email_invalid() {
        assert!(SanEmail::new("").is_err());
        assert!(SanEmail::new("noatsign").is_err());
        assert!(SanEmail::new("@nodomain").is_err());
        assert!(SanEmail::new("nolocal@").is_err());
        assert!(SanEmail::new("user@nodot").is_err());
    }

    #[test]
    fn test_san_builder() {
        let san = SubjectAlternativeName::new()
            .with_dns_name("example.com")
            .unwrap()
            .with_dns_name("*.example.com")
            .unwrap()
            .with_ip_address("192.168.1.1")
            .unwrap();

        assert_eq!(san.len(), 3);
        assert!(san.has_wildcard());
        assert_eq!(san.dns_names().count(), 2);
        assert_eq!(san.ip_addresses().count(), 1);
    }

    #[test]
    fn test_san_localhost() {
        let san = SubjectAlternativeName::localhost().unwrap();
        assert!(san.dns_names().any(|d| d.as_str() == "localhost"));
        assert!(san.ip_addresses().any(|ip| ip.addr().to_string() == "127.0.0.1"));
        assert!(san.ip_addresses().any(|ip| ip.addr().to_string() == "::1"));
    }

    #[test]
    fn test_san_to_string_list() {
        let san = SubjectAlternativeName::new()
            .with_dns_name("example.com")
            .unwrap()
            .with_ip_address("10.0.0.1")
            .unwrap();

        let strings = san.to_string_list();
        assert!(strings.iter().any(|s| s.contains("example.com")));
        assert!(strings.iter().any(|s| s.contains("10.0.0.1")));
    }

    #[test]
    fn test_san_from_string_list() {
        let strings = vec![
            "DNS:example.com".to_string(),
            "IP:10.0.0.1".to_string(),
            "email:user@example.com".to_string(),
        ];
        let san = SubjectAlternativeName::from_string_list(&strings);

        assert_eq!(san.len(), 3);
        assert_eq!(san.dns_names().count(), 1);
        assert_eq!(san.ip_addresses().count(), 1);
        assert_eq!(san.emails().count(), 1);
    }

    #[test]
    fn test_san_domain_with_wildcard() {
        let san = SubjectAlternativeName::domain_with_wildcard("example.com").unwrap();
        assert_eq!(san.dns_names().count(), 2);
        assert!(san.has_wildcard());
        assert!(san.dns_names().any(|d| d.as_str() == "example.com"));
        assert!(san.dns_names().any(|d| d.as_str() == "*.example.com"));
    }

    #[test]
    fn test_san_tls_server() {
        let san = SubjectAlternativeName::tls_server(&["example.com", "www.example.com"]).unwrap();
        assert_eq!(san.dns_names().count(), 2);
    }
}
