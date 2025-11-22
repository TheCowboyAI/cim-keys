//! Certificate Aggregate Events
//!
//! Events related to the Certificate aggregate root.
//! Certificates represent X.509 digital certificates in the PKI hierarchy.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
}

/// A new certificate was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateGeneratedEvent {
    pub cert_id: Uuid,
    pub key_id: Uuid,
    pub subject: String,
    pub issuer: Option<Uuid>, // None for self-signed
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub is_ca: bool,
    pub san: Vec<String>,
    pub key_usage: Vec<String>,
    pub extended_key_usage: Vec<String>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
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
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub organization_id: Uuid,
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
        }
    }
}
