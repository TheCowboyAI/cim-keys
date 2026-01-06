// Copyright (c) 2025 - Cowboy AI, LLC.

//! X.509 Certificate Validity Period (RFC 5280 Section 4.1.2.5)
//!
//! Provides type-safe validity period value object that enforces RFC 5280
//! requirements for certificate temporal validity.
//!
//! ## Validity Period
//!
//! Per RFC 5280, a certificate's validity period is the time interval during
//! which the CA warrants that it will maintain information about the status
//! of the certificate.
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::value_objects::x509::CertificateValidity;
//! use chrono::{Duration, Utc};
//!
//! // Create validity for 1 year from now
//! let validity = CertificateCertificateValidity::years_from_now(1);
//!
//! // Check if currently valid
//! assert!(validity.is_valid_now());
//!
//! // Create validity with specific dates
//! let custom = CertificateCertificateValidity::new(
//!     Utc::now(),
//!     Utc::now() + Duration::days(365),
//! )?;
//! ```

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

/// Certificate Validity Period value object
///
/// Represents the time window during which a certificate is valid.
/// Per RFC 5280, the validity period consists of two dates:
/// - `not_before`: The date/time on which the certificate validity begins
/// - `not_after`: The date/time after which the certificate is no longer valid
///
/// This is a more comprehensive version than the basic `Validity` type in core,
/// with additional validation, utility methods, and convenience constructors
/// specifically for X.509 certificates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateValidity {
    /// The date/time from which the certificate is valid (inclusive)
    not_before: DateTime<Utc>,
    /// The date/time until which the certificate is valid (inclusive)
    not_after: DateTime<Utc>,
}

impl CertificateValidity {
    /// Create a new validity period
    ///
    /// Returns an error if `not_before` is after `not_after`.
    pub fn new(not_before: DateTime<Utc>, not_after: DateTime<Utc>) -> Result<Self, ValidityError> {
        if not_before > not_after {
            return Err(ValidityError::InvalidPeriod {
                not_before,
                not_after,
            });
        }
        Ok(Self {
            not_before,
            not_after,
        })
    }

    /// Create validity starting now for a specified duration
    pub fn from_now(duration: Duration) -> Result<Self, ValidityError> {
        let now = Utc::now();
        let not_after = now + duration;
        Self::new(now, not_after)
    }

    /// Create validity for a number of days from now
    pub fn days_from_now(days: i64) -> Result<Self, ValidityError> {
        Self::from_now(Duration::days(days))
    }

    /// Create validity for a number of years from now
    pub fn years_from_now(years: i32) -> Result<Self, ValidityError> {
        Self::from_now(Duration::days(i64::from(years) * 365))
    }

    /// Create validity for a number of years from now with a grace period before
    ///
    /// This is common for certificates where you want them to be valid
    /// slightly before the current time to handle clock skew.
    pub fn years_from_now_with_grace(years: i32, grace_minutes: i64) -> Result<Self, ValidityError> {
        let now = Utc::now();
        let not_before = now - Duration::minutes(grace_minutes);
        let not_after = now + Duration::days(i64::from(years) * 365);
        Self::new(not_before, not_after)
    }

    /// Get the not_before date
    pub fn not_before(&self) -> DateTime<Utc> {
        self.not_before
    }

    /// Get the not_after date
    pub fn not_after(&self) -> DateTime<Utc> {
        self.not_after
    }

    /// Get the validity duration
    pub fn duration(&self) -> Duration {
        self.not_after - self.not_before
    }

    /// Check if a specific time is within the validity period
    pub fn is_valid_at(&self, time: DateTime<Utc>) -> bool {
        time >= self.not_before && time <= self.not_after
    }

    /// Check if the certificate is currently valid
    pub fn is_valid_now(&self) -> bool {
        self.is_valid_at(Utc::now())
    }

    /// Check if the certificate has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.not_after
    }

    /// Check if the certificate is not yet valid
    pub fn is_not_yet_valid(&self) -> bool {
        Utc::now() < self.not_before
    }

    /// Get time remaining until expiration (None if already expired)
    pub fn time_remaining(&self) -> Option<Duration> {
        let now = Utc::now();
        if now > self.not_after {
            None
        } else {
            Some(self.not_after - now)
        }
    }

    /// Get days remaining until expiration (None if already expired)
    pub fn days_remaining(&self) -> Option<i64> {
        self.time_remaining().map(|d| d.num_days())
    }

    /// Check if certificate is expiring soon (within threshold)
    pub fn is_expiring_soon(&self, threshold: Duration) -> bool {
        if let Some(remaining) = self.time_remaining() {
            remaining <= threshold
        } else {
            true // Already expired
        }
    }

    /// Check if certificate expires within N days
    pub fn expires_within_days(&self, days: i64) -> bool {
        self.is_expiring_soon(Duration::days(days))
    }

    // ========================================================================
    // Graph Label Generation
    // ========================================================================

    /// Generate labels for graph node based on validity status
    ///
    /// Returns labels like "Valid", "Expired", "ExpiringSoon", "NotYetValid"
    pub fn as_labels(&self) -> Vec<String> {
        let mut labels = Vec::new();

        if self.is_expired() {
            labels.push("Expired".to_string());
        } else if self.is_not_yet_valid() {
            labels.push("NotYetValid".to_string());
        } else {
            labels.push("Valid".to_string());

            // Add warning labels
            if self.expires_within_days(30) {
                labels.push("ExpiringSoon".to_string());
            } else if self.expires_within_days(90) {
                labels.push("ExpiringWithin90Days".to_string());
            }
        }

        labels
    }

    // ========================================================================
    // Convenience Constructors for Common Certificate Types
    // ========================================================================

    /// Standard validity for a Root CA certificate (10-20 years)
    pub fn root_ca() -> Result<Self, ValidityError> {
        Self::years_from_now_with_grace(20, 5)
    }

    /// Standard validity for an Intermediate CA certificate (5-10 years)
    pub fn intermediate_ca() -> Result<Self, ValidityError> {
        Self::years_from_now_with_grace(10, 5)
    }

    /// Standard validity for a TLS server certificate (1-2 years)
    /// Note: Per CA/Browser Forum, max is 398 days for public TLS
    pub fn tls_server() -> Result<Self, ValidityError> {
        Self::days_from_now(398)
    }

    /// Standard validity for a TLS client certificate (1 year)
    pub fn tls_client() -> Result<Self, ValidityError> {
        Self::years_from_now_with_grace(1, 5)
    }

    /// Standard validity for a code signing certificate (3 years)
    pub fn code_signing() -> Result<Self, ValidityError> {
        Self::years_from_now_with_grace(3, 5)
    }

    /// Short-lived certificate for development/testing (90 days)
    pub fn development() -> Result<Self, ValidityError> {
        Self::days_from_now(90)
    }
}

impl fmt::Display for CertificateValidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Valid from {} to {}",
            self.not_before.format("%Y-%m-%d %H:%M:%S UTC"),
            self.not_after.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

impl DomainConcept for CertificateValidity {}
impl ValueObject for CertificateValidity {}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur when creating validity periods
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidityError {
    /// not_before is after not_after
    InvalidPeriod {
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
    },
}

impl fmt::Display for ValidityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidityError::InvalidPeriod {
                not_before,
                not_after,
            } => {
                write!(
                    f,
                    "Invalid validity period: not_before ({}) is after not_after ({})",
                    not_before, not_after
                )
            }
        }
    }
}

impl std::error::Error for ValidityError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validity_new() {
        let now = Utc::now();
        let future = now + Duration::days(365);
        let validity = CertificateValidity::new(now, future).unwrap();

        assert_eq!(validity.not_before(), now);
        assert_eq!(validity.not_after(), future);
    }

    #[test]
    fn test_validity_invalid_period() {
        let now = Utc::now();
        let past = now - Duration::days(1);
        let result = CertificateValidity::new(now, past);

        assert!(result.is_err());
    }

    #[test]
    fn test_validity_days_from_now() {
        let validity = CertificateValidity::days_from_now(30).unwrap();

        assert!(validity.is_valid_now());
        assert!(!validity.is_expired());
        assert!(!validity.is_not_yet_valid());

        let days = validity.duration().num_days();
        assert!(days >= 29 && days <= 30); // Allow for timing variance
    }

    #[test]
    fn test_validity_years_from_now() {
        let validity = CertificateValidity::years_from_now(1).unwrap();

        assert!(validity.is_valid_now());
        let days = validity.duration().num_days();
        assert!(days >= 364 && days <= 366);
    }

    #[test]
    fn test_validity_is_valid_at() {
        let now = Utc::now();
        let validity = CertificateValidity::new(now, now + Duration::days(30)).unwrap();

        // Within range
        assert!(validity.is_valid_at(now + Duration::days(15)));

        // Before range
        assert!(!validity.is_valid_at(now - Duration::days(1)));

        // After range
        assert!(!validity.is_valid_at(now + Duration::days(31)));
    }

    #[test]
    fn test_validity_expiration_checks() {
        // Create already expired validity
        let past = Utc::now() - Duration::days(10);
        let expired = CertificateValidity::new(past - Duration::days(30), past).unwrap();

        assert!(expired.is_expired());
        assert!(!expired.is_valid_now());
        assert!(expired.time_remaining().is_none());
    }

    #[test]
    fn test_validity_not_yet_valid() {
        let future = Utc::now() + Duration::days(10);
        let not_yet = CertificateValidity::new(future, future + Duration::days(30)).unwrap();

        assert!(not_yet.is_not_yet_valid());
        assert!(!not_yet.is_valid_now());
        assert!(!not_yet.is_expired());
    }

    #[test]
    fn test_validity_expiring_soon() {
        let now = Utc::now();
        // Expires in 15 days
        let validity = CertificateValidity::new(now - Duration::days(15), now + Duration::days(15)).unwrap();

        assert!(validity.expires_within_days(30));
        assert!(validity.expires_within_days(16));
        assert!(!validity.expires_within_days(10));
    }

    #[test]
    fn test_validity_days_remaining() {
        let validity = CertificateValidity::days_from_now(100).unwrap();
        let remaining = validity.days_remaining().unwrap();

        assert!(remaining >= 99 && remaining <= 100);
    }

    #[test]
    fn test_validity_labels_valid() {
        let validity = CertificateValidity::years_from_now(1).unwrap();
        let labels = validity.as_labels();

        assert!(labels.contains(&"Valid".to_string()));
        assert!(!labels.contains(&"Expired".to_string()));
    }

    #[test]
    fn test_validity_labels_expiring_soon() {
        let now = Utc::now();
        let validity = CertificateValidity::new(now - Duration::days(300), now + Duration::days(20)).unwrap();
        let labels = validity.as_labels();

        assert!(labels.contains(&"Valid".to_string()));
        assert!(labels.contains(&"ExpiringSoon".to_string()));
    }

    #[test]
    fn test_validity_labels_expired() {
        let past = Utc::now() - Duration::days(10);
        let expired = CertificateValidity::new(past - Duration::days(30), past).unwrap();
        let labels = expired.as_labels();

        assert!(labels.contains(&"Expired".to_string()));
        assert!(!labels.contains(&"Valid".to_string()));
    }

    #[test]
    fn test_validity_root_ca() {
        let validity = CertificateValidity::root_ca().unwrap();
        let years = validity.duration().num_days() / 365;

        assert!(years >= 19 && years <= 21); // ~20 years
    }

    #[test]
    fn test_validity_tls_server() {
        let validity = CertificateValidity::tls_server().unwrap();
        let days = validity.duration().num_days();

        // Should be close to 398 days (CA/Browser Forum max)
        assert!(days >= 397 && days <= 399);
    }

    #[test]
    fn test_validity_display() {
        let validity = CertificateValidity::days_from_now(30).unwrap();
        let display = format!("{}", validity);

        assert!(display.contains("Valid from"));
        assert!(display.contains("to"));
        assert!(display.contains("UTC"));
    }

    #[test]
    fn test_validity_with_grace() {
        let validity = CertificateValidity::years_from_now_with_grace(1, 5).unwrap();

        // Should be valid now (has 5 minute grace in the past)
        assert!(validity.is_valid_now());

        // not_before should be in the past
        assert!(validity.not_before() < Utc::now());
    }
}
