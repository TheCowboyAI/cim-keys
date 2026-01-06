//! Certificate Aggregate Events
//!
//! Events related to the Certificate aggregate root.
//! Certificates represent X.509 digital certificates in the PKI hierarchy.
//!
//! ## Value Object Migration
//!
//! Events use dual-path fields for backward compatibility:
//! - Old string fields are kept for deserializing existing events
//! - New typed fields use Option<T> for gradual migration
//! - Accessor methods prefer typed fields, fall back to strings

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::value_objects::x509::{
    BasicConstraints, CertificateValidity, ExtendedKeyUsage, KeyUsage,
    SubjectAlternativeName, SubjectName,
};

/// Events for the Certificate aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum CertificateEvents {
    /// A new certificate was generated
    CertificateGenerated(CertificateGeneratedEvent),

    /// A certificate was signed
    CertificateSigned(CertificateSignedEvent),

    /// A certificate was revoked
    CertificateRevoked(CertificateRevokedEvent),

    /// A certificate was renewed
    CertificateRenewed(CertificateRenewedEvent),

    /// A certificate was validated
    CertificateValidated(CertificateValidatedEvent),

    /// A certificate was exported
    CertificateExported(CertificateExportedEvent),

    /// PKI hierarchy was created
    PkiHierarchyCreated(PkiHierarchyCreatedEvent),

    // Lifecycle State Transitions (Phase 11)
    /// Certificate activated
    CertificateActivated(CertificateActivatedEvent),

    /// Certificate suspended
    CertificateSuspended(CertificateSuspendedEvent),

    /// Certificate expired (terminal)
    CertificateExpired(CertificateExpiredEvent),
}

/// A new certificate was generated
///
/// This event uses dual-path fields for backward compatibility:
/// - Legacy string fields (`subject`, `san`, etc.) for existing events
/// - Typed value objects (`subject_name`, `subject_alt_name`, etc.) for new events
///
/// Use the accessor methods (e.g., `subject_value_object()`) which prefer
/// typed fields and fall back to parsing legacy strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateGeneratedEvent {
    pub cert_id: Uuid,
    pub key_id: Uuid,

    // ========================================================================
    // Legacy fields (deprecated, kept for backward compatibility)
    // ========================================================================

    /// Legacy: Subject DN as string (use subject_name instead)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use subject_name field instead")]
    pub subject: String,

    /// Legacy: Subject Alternative Names as strings (use subject_alt_name instead)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[deprecated(note = "Use subject_alt_name field instead")]
    pub san: Vec<String>,

    /// Legacy: Key usage as strings (use key_usage_ext instead)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[deprecated(note = "Use key_usage_ext field instead")]
    pub key_usage: Vec<String>,

    /// Legacy: Extended key usage as strings (use extended_key_usage_ext instead)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[deprecated(note = "Use extended_key_usage_ext field instead")]
    pub extended_key_usage: Vec<String>,

    /// Legacy: Validity start (use validity instead)
    #[deprecated(note = "Use validity field instead")]
    pub not_before: DateTime<Utc>,

    /// Legacy: Validity end (use validity instead)
    #[deprecated(note = "Use validity field instead")]
    pub not_after: DateTime<Utc>,

    /// Legacy: CA flag (use basic_constraints instead)
    #[deprecated(note = "Use basic_constraints field instead")]
    pub is_ca: bool,

    // ========================================================================
    // Typed value object fields (preferred)
    // ========================================================================

    /// Typed: Subject Distinguished Name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_name: Option<SubjectName>,

    /// Typed: Subject Alternative Name extension
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_alt_name: Option<SubjectAlternativeName>,

    /// Typed: Key Usage extension
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_usage_ext: Option<KeyUsage>,

    /// Typed: Extended Key Usage extension
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extended_key_usage_ext: Option<ExtendedKeyUsage>,

    /// Typed: Certificate validity period
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validity: Option<CertificateValidity>,

    /// Typed: Basic Constraints extension
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_constraints: Option<BasicConstraints>,

    // ========================================================================
    // Other fields
    // ========================================================================

    /// Issuer certificate ID (None for self-signed root)
    pub issuer: Option<Uuid>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl CertificateGeneratedEvent {
    /// Create a new event using legacy string fields (for backward compatibility)
    ///
    /// Use this when migrating existing code. New code should use `new_typed()`.
    #[allow(clippy::too_many_arguments)]
    pub fn new_legacy(
        cert_id: Uuid,
        key_id: Uuid,
        subject: String,
        issuer: Option<Uuid>,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        is_ca: bool,
        san: Vec<String>,
        key_usage: Vec<String>,
        extended_key_usage: Vec<String>,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
    ) -> Self {
        Self {
            cert_id,
            key_id,
            subject,
            san,
            key_usage,
            extended_key_usage,
            not_before,
            not_after,
            is_ca,
            issuer,
            correlation_id,
            causation_id,
            // New typed fields are None
            subject_name: None,
            subject_alt_name: None,
            key_usage_ext: None,
            extended_key_usage_ext: None,
            validity: None,
            basic_constraints: None,
        }
    }

    /// Create a new event using typed value objects (preferred for new code)
    pub fn new_typed(
        cert_id: Uuid,
        key_id: Uuid,
        subject_name: SubjectName,
        issuer: Option<Uuid>,
        validity: CertificateValidity,
        basic_constraints: BasicConstraints,
        subject_alt_name: Option<SubjectAlternativeName>,
        key_usage: KeyUsage,
        extended_key_usage: Option<ExtendedKeyUsage>,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
    ) -> Self {
        Self {
            cert_id,
            key_id,
            // Legacy fields populated for backward compat serialization
            subject: subject_name.to_rfc4514(),
            san: subject_alt_name.as_ref().map(|s| s.to_string_list()).unwrap_or_default(),
            key_usage: key_usage.to_string_list(),
            extended_key_usage: extended_key_usage.as_ref().map(|e| e.to_string_list()).unwrap_or_default(),
            not_before: validity.not_before(),
            not_after: validity.not_after(),
            is_ca: basic_constraints.is_ca(),
            issuer,
            correlation_id,
            causation_id,
            // Typed fields
            subject_name: Some(subject_name),
            subject_alt_name,
            key_usage_ext: Some(key_usage),
            extended_key_usage_ext: extended_key_usage,
            validity: Some(validity),
            basic_constraints: Some(basic_constraints),
        }
    }

    /// Get SubjectName, preferring typed field, falling back to parsing legacy string
    pub fn subject_value_object(&self) -> Option<SubjectName> {
        if let Some(ref sn) = self.subject_name {
            return Some(sn.clone());
        }
        // Fall back to parsing legacy string
        if !self.subject.is_empty() {
            SubjectName::parse_rfc4514(&self.subject).ok()
        } else {
            None
        }
    }

    /// Get SubjectAlternativeName, preferring typed field
    pub fn san_value_object(&self) -> SubjectAlternativeName {
        if let Some(ref san) = self.subject_alt_name {
            return san.clone();
        }
        // Fall back to parsing legacy strings
        SubjectAlternativeName::from_string_list(&self.san)
    }

    /// Get KeyUsage, preferring typed field
    pub fn key_usage_value_object(&self) -> KeyUsage {
        if let Some(ref ku) = self.key_usage_ext {
            return ku.clone();
        }
        // Fall back to parsing legacy strings
        KeyUsage::from_string_list(&self.key_usage)
    }

    /// Get ExtendedKeyUsage, preferring typed field
    pub fn extended_key_usage_value_object(&self) -> ExtendedKeyUsage {
        if let Some(ref eku) = self.extended_key_usage_ext {
            return eku.clone();
        }
        // Fall back to parsing legacy strings
        ExtendedKeyUsage::from_string_list(&self.extended_key_usage)
    }

    /// Get CertificateValidity, preferring typed field
    pub fn validity_value_object(&self) -> Option<CertificateValidity> {
        if let Some(ref v) = self.validity {
            return Some(v.clone());
        }
        // Fall back to constructing from legacy fields
        CertificateValidity::new(self.not_before, self.not_after).ok()
    }

    /// Get BasicConstraints, preferring typed field
    pub fn basic_constraints_value_object(&self) -> BasicConstraints {
        if let Some(ref bc) = self.basic_constraints {
            return bc.clone();
        }
        // Fall back to constructing from legacy is_ca field
        if self.is_ca {
            BasicConstraints::ca()
        } else {
            BasicConstraints::end_entity()
        }
    }

    /// Check if this event uses typed value objects (new format)
    pub fn uses_typed_fields(&self) -> bool {
        self.subject_name.is_some()
            || self.subject_alt_name.is_some()
            || self.key_usage_ext.is_some()
            || self.extended_key_usage_ext.is_some()
            || self.validity.is_some()
            || self.basic_constraints.is_some()
    }
}

/// A certificate was signed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSignedEvent {
    pub cert_id: Uuid,
    pub signed_by: Uuid, // CA cert ID
    pub signature_algorithm: String,
    pub signed_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A certificate was revoked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRevokedEvent {
    pub cert_id: Uuid,
    pub reason: String,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: String,
    pub crl_distribution_point: Option<String>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A certificate was renewed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRenewedEvent {
    pub old_cert_id: Uuid,
    pub new_cert_id: Uuid,
    pub renewed_at: DateTime<Utc>,
    pub renewed_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A certificate was validated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidatedEvent {
    pub cert_id: Uuid,
    pub validation_method: String,
    pub validation_result: bool,
    pub validation_errors: Vec<String>,
    pub validated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A certificate was exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateExportedEvent {
    pub export_id: Uuid,
    pub cert_id: Uuid,
    pub export_format: String,
    pub destination_path: String,
    pub exported_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// PKI hierarchy was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkiHierarchyCreatedEvent {
    pub root_ca_id: Uuid,
    pub intermediate_cas: Vec<Uuid>,
    pub created_by: String,
    pub organization_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Certificate Lifecycle State Transitions (Phase 11)
// ============================================================================

/// Certificate activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateActivatedEvent {
    pub cert_id: Uuid,
    pub activated_at: DateTime<Utc>,
    pub activated_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Certificate suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSuspendedEvent {
    pub cert_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Certificate expired (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateExpiredEvent {
    pub cert_id: Uuid,
    pub expired_at: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for CertificateEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            CertificateEvents::CertificateGenerated(e) => e.cert_id,
            CertificateEvents::CertificateSigned(e) => e.cert_id,
            CertificateEvents::CertificateRevoked(e) => e.cert_id,
            CertificateEvents::CertificateRenewed(e) => e.old_cert_id,
            CertificateEvents::CertificateValidated(e) => e.cert_id,
            CertificateEvents::CertificateExported(e) => e.cert_id,
            CertificateEvents::PkiHierarchyCreated(e) => e.root_ca_id,
            CertificateEvents::CertificateActivated(e) => e.cert_id,
            CertificateEvents::CertificateSuspended(e) => e.cert_id,
            CertificateEvents::CertificateExpired(e) => e.cert_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            CertificateEvents::CertificateGenerated(_) => "CertificateGenerated",
            CertificateEvents::CertificateSigned(_) => "CertificateSigned",
            CertificateEvents::CertificateRevoked(_) => "CertificateRevoked",
            CertificateEvents::CertificateRenewed(_) => "CertificateRenewed",
            CertificateEvents::CertificateValidated(_) => "CertificateValidated",
            CertificateEvents::CertificateExported(_) => "CertificateExported",
            CertificateEvents::PkiHierarchyCreated(_) => "PkiHierarchyCreated",
            CertificateEvents::CertificateActivated(_) => "CertificateActivated",
            CertificateEvents::CertificateSuspended(_) => "CertificateSuspended",
            CertificateEvents::CertificateExpired(_) => "CertificateExpired",
        }
    }
}
