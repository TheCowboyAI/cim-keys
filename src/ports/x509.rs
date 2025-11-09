//! X.509 certificate port
//!
//! This defines the interface for X.509/TLS certificate operations.
//! This is a **Functor** mapping from the X.509 PKI category to the Domain category.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: X.509 PKI (certificates, trust chains, CAs)
//! - **Target Category**: Domain (key management and trust operations)
//! - **Functor**: X509Port maps PKI operations to domain operations
//! - **Morphisms Preserved**: sign_csr ∘ generate_csr maintains certificate hierarchy

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Port for X.509 certificate operations
///
/// This is a **Functor** F: X509_PKI → Domain where:
/// - Objects: Certificates and keys → Domain trust entities
/// - Morphisms: PKI operations (sign, verify, chain) → Domain trust relationships
///
/// **Functor Laws:**
/// 1. Identity: F(id) = id - No-op on cert maps to no domain change
/// 2. Composition: F(sign_csr ∘ generate_csr) = F(sign_csr) ∘ F(generate_csr)
#[async_trait]
pub trait X509Port: Send + Sync {
    /// Generate self-signed root CA certificate
    ///
    /// **Functor Mapping**: (subject, key, validity) → RootCertificate
    /// This is an initial object in the certificate category
    async fn generate_root_ca(
        &self,
        subject: &CertificateSubject,
        key: &PrivateKey,
        validity_days: u32,
    ) -> Result<Certificate, X509Error>;

    /// Generate certificate signing request (CSR)
    ///
    /// **Functor Mapping**: (subject, key, san) → CSR
    async fn generate_csr(
        &self,
        subject: &CertificateSubject,
        key: &PrivateKey,
        san: Vec<String>,
    ) -> Result<CertificateSigningRequest, X509Error>;

    /// Sign CSR with CA
    ///
    /// **Functor Mapping**: (csr, ca_cert, ca_key) → SignedCertificate
    /// Composes with generate_csr to form certificate chain
    async fn sign_csr(
        &self,
        csr: &CertificateSigningRequest,
        ca_cert: &Certificate,
        ca_key: &PrivateKey,
        validity_days: u32,
        is_ca: bool,
    ) -> Result<Certificate, X509Error>;

    /// Generate intermediate CA certificate
    ///
    /// **Functor Mapping**: Creates intermediate object in certificate category
    async fn generate_intermediate_ca(
        &self,
        subject: &CertificateSubject,
        key: &PrivateKey,
        parent_ca_cert: &Certificate,
        parent_ca_key: &PrivateKey,
        validity_days: u32,
        path_len_constraint: Option<u32>,
    ) -> Result<Certificate, X509Error>;

    /// Generate leaf certificate
    ///
    /// **Functor Mapping**: Creates terminal object in certificate category
    async fn generate_leaf_certificate(
        &self,
        subject: &CertificateSubject,
        key: &PrivateKey,
        ca_cert: &Certificate,
        ca_key: &PrivateKey,
        validity_days: u32,
        san: Vec<String>,
        key_usage: Vec<KeyUsage>,
        extended_key_usage: Vec<ExtendedKeyUsage>,
    ) -> Result<Certificate, X509Error>;

    /// Parse X.509 certificate
    ///
    /// **Functor Mapping**: bytes → Certificate (object construction)
    async fn parse_certificate(&self, cert_data: &[u8]) -> Result<Certificate, X509Error>;

    /// Verify certificate chain
    ///
    /// **Functor Mapping**: [Certificate] → bool (validates morphism composition)
    async fn verify_chain(
        &self,
        leaf_cert: &Certificate,
        intermediates: &[Certificate],
        root_cert: &Certificate,
    ) -> Result<bool, X509Error>;

    /// Export certificate (PEM/DER)
    ///
    /// **Functor Mapping**: Certificate → bytes (object serialization)
    async fn export_certificate(
        &self,
        cert: &Certificate,
        format: CertificateFormat,
    ) -> Result<Vec<u8>, X509Error>;

    /// Generate certificate revocation list (CRL)
    async fn generate_crl(
        &self,
        ca_cert: &Certificate,
        ca_key: &PrivateKey,
        revoked_certs: Vec<RevokedCertificate>,
    ) -> Result<CertificateRevocationList, X509Error>;

    /// Generate OCSP response
    async fn generate_ocsp_response(
        &self,
        cert: &Certificate,
        issuer_cert: &Certificate,
        issuer_key: &PrivateKey,
        status: OcspStatus,
    ) -> Result<OcspResponse, X509Error>;
}

/// Certificate subject (Distinguished Name)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CertificateSubject {
    /// Common Name (CN)
    pub common_name: String,

    /// Organization (O)
    pub organization: Option<String>,

    /// Organizational Unit (OU)
    pub organizational_unit: Option<String>,

    /// Country (C)
    pub country: Option<String>,

    /// State/Province (ST)
    pub state: Option<String>,

    /// Locality (L)
    pub locality: Option<String>,

    /// Email
    pub email: Option<String>,
}

/// X.509 certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Certificate in DER format
    pub der: Vec<u8>,

    /// Certificate in PEM format
    pub pem: String,

    /// Subject
    pub subject: CertificateSubject,

    /// Issuer
    pub issuer: CertificateSubject,

    /// Serial number
    pub serial: Vec<u8>,

    /// Valid from (Unix timestamp)
    pub not_before: i64,

    /// Valid until (Unix timestamp)
    pub not_after: i64,

    /// Is CA certificate
    pub is_ca: bool,

    /// Public key algorithm
    pub key_algorithm: String,
}

/// Certificate Signing Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSigningRequest {
    /// CSR in DER format
    pub der: Vec<u8>,

    /// CSR in PEM format
    pub pem: String,

    /// Subject
    pub subject: CertificateSubject,

    /// Subject Alternative Names
    pub san: Vec<String>,
}

/// Private key (abstract)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateKey {
    /// Key algorithm
    pub algorithm: String,

    /// Key in DER format
    pub der: Vec<u8>,

    /// Key in PEM format
    pub pem: String,
}

/// Key usage extensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyUsage {
    DigitalSignature,
    NonRepudiation,
    KeyEncipherment,
    DataEncipherment,
    KeyAgreement,
    KeyCertSign,
    CrlSign,
    EncipherOnly,
    DecipherOnly,
}

/// Extended key usage extensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExtendedKeyUsage {
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OcspSigning,
}

/// Certificate format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CertificateFormat {
    Pem,
    Der,
}

/// Revoked certificate entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokedCertificate {
    /// Certificate serial number
    pub serial: Vec<u8>,

    /// Revocation time (Unix timestamp)
    pub revocation_time: i64,

    /// Revocation reason
    pub reason: RevocationReason,
}

/// Revocation reason codes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RevocationReason {
    Unspecified,
    KeyCompromise,
    CaCompromise,
    AffiliationChanged,
    Superseded,
    CessationOfOperation,
    CertificateHold,
    RemoveFromCrl,
    PrivilegeWithdrawn,
    AaCompromise,
}

/// Certificate Revocation List
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRevocationList {
    /// CRL in DER format
    pub der: Vec<u8>,

    /// CRL in PEM format
    pub pem: String,

    /// Issuer
    pub issuer: CertificateSubject,

    /// This update time (Unix timestamp)
    pub this_update: i64,

    /// Next update time (Unix timestamp)
    pub next_update: i64,

    /// Revoked certificates
    pub revoked_certificates: Vec<RevokedCertificate>,
}

/// OCSP status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OcspStatus {
    Good,
    Revoked,
    Unknown,
}

/// OCSP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcspResponse {
    /// Response in DER format
    pub der: Vec<u8>,

    /// Certificate status
    pub status: OcspStatus,

    /// This update time (Unix timestamp)
    pub this_update: i64,

    /// Next update time (Unix timestamp)
    pub next_update: Option<i64>,
}

/// X.509 operation errors
#[derive(Debug, Error)]
pub enum X509Error {
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),

    #[error("Invalid CSR: {0}")]
    InvalidCsr(String),

    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("Certificate generation failed: {0}")]
    GenerationFailed(String),

    #[error("Certificate signing failed: {0}")]
    SigningFailed(String),

    #[error("Certificate verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid certificate chain")]
    InvalidChain,

    #[error("Certificate expired")]
    Expired,

    #[error("Certificate not yet valid")]
    NotYetValid,

    #[error("Invalid subject: {0}")]
    InvalidSubject(String),

    #[error("Invalid serial number: {0}")]
    InvalidSerial(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("X.509 operation error: {0}")]
    OperationError(String),
}
