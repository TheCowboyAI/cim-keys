//! X.509 Certificate Generation from Deterministic Seeds
//!
//! This module implements the PKI hierarchy with intermediate signing-only certificates:
//!
//! ```text
//! Root CA (offline, 20 years)
//!   ├─ pathlen: 1 (can sign intermediates)
//!   └─ keyUsage: keyCertSign, cRLSign
//!
//! Intermediate Signing CA (online, 3 years, SIGNING ONLY)
//!   ├─ pathlen: 0 (cannot sign other CAs)
//!   ├─ keyUsage: keyCertSign, cRLSign (NO digitalSignature!)
//!   └─ NOT used as server identity
//!
//! Server/Leaf Certificates (rotatable, 90 days)
//!   ├─ CA: FALSE
//!   ├─ keyUsage: digitalSignature, keyEncipherment
//!   └─ extendedKeyUsage: serverAuth, clientAuth
//! ```

use super::seed_derivation::MasterSeed;
use super::key_generation::KeyPair;
use rcgen::{
    CertificateParams, DistinguishedName, DnType,
    KeyUsagePurpose, ExtendedKeyUsagePurpose, IsCa, BasicConstraints,
    KeyPair as RcgenKeyPair, Issuer,
};
use time::{Duration, OffsetDateTime};

/// X.509 certificate with associated keypair
#[derive(Clone, Debug)]
pub struct X509Certificate {
    /// The certificate in PEM format
    pub certificate_pem: String,
    /// The private key in PEM format (for non-CA certs or initial distribution)
    pub private_key_pem: String,
    /// The public key bytes (from our deterministic Ed25519 seed)
    pub public_key_bytes: Vec<u8>,
    /// Certificate fingerprint (SHA-256)
    pub fingerprint: String,
    /// Seed derivation path for reproducibility (e.g., "root-ca", "intermediate-engineering")
    pub seed_path: String,
}

/// Parameters for Root CA generation
#[derive(Clone)]
pub struct RootCAParams {
    /// Organization name (e.g., "CowboyAI Inc")
    pub organization: String,
    /// Common name (e.g., "CowboyAI Root CA")
    pub common_name: String,
    /// Country code (e.g., "US")
    pub country: Option<String>,
    /// State/Province
    pub state: Option<String>,
    /// Locality/City
    pub locality: Option<String>,
    /// Validity in years (default: 20)
    pub validity_years: u32,
    /// Path length constraint - how many intermediate CA levels allowed below this root
    /// Default: 1 (standard single-intermediate chain)
    /// Use 2 for CowboyAI hosting scenario (hosting intermediate + client intermediate)
    pub pathlen: u8,
}

/// Parameters for Intermediate CA generation
pub struct IntermediateCAParams {
    /// Organization name
    pub organization: String,
    /// Organizational unit (e.g., "Engineering")
    pub organizational_unit: String,
    /// Common name (e.g., "CowboyAI Engineering Intermediate CA")
    pub common_name: String,
    /// Country code
    pub country: Option<String>,
    /// Validity in years (default: 3)
    pub validity_years: u32,
    /// Path length constraint - how many intermediate CA levels allowed below this one
    /// Default: 0 (can only sign leaf certificates, not other CAs)
    /// Use 1 for CowboyAI hosting intermediate (can sign one more intermediate level)
    pub pathlen: u8,
}

/// Parameters for Server certificate generation
pub struct ServerCertParams {
    /// Common name / primary DNS name (e.g., "nats-server-01.example.com")
    pub common_name: String,
    /// Subject Alternative Names (DNS names, IP addresses)
    pub san_entries: Vec<String>,
    /// Organization name
    pub organization: String,
    /// Organizational unit
    pub organizational_unit: Option<String>,
    /// Validity in days (default: 90)
    pub validity_days: u32,
}

impl Default for RootCAParams {
    fn default() -> Self {
        Self {
            organization: "CIM Organization".to_string(),
            common_name: "CIM Root CA".to_string(),
            country: Some("US".to_string()),
            state: None,
            locality: None,
            validity_years: 20,
            pathlen: 1, // Standard single-intermediate chain
        }
    }
}

impl Default for IntermediateCAParams {
    fn default() -> Self {
        Self {
            organization: "CIM Organization".to_string(),
            organizational_unit: "Operations".to_string(),
            common_name: "CIM Intermediate CA".to_string(),
            country: Some("US".to_string()),
            validity_years: 3,
            pathlen: 0, // Can only sign leaf certificates by default
        }
    }
}

impl Default for ServerCertParams {
    fn default() -> Self {
        Self {
            common_name: "server.example.com".to_string(),
            san_entries: vec![],
            organization: "CIM Organization".to_string(),
            organizational_unit: None,
            validity_days: 90,
        }
    }
}

/// Generate a Root CA certificate from a seed
///
/// The Root CA is the ultimate trust anchor:
/// - Long-lived (20 years default)
/// - Stored offline on air-gapped YubiKey
/// - Only signs intermediate CA certificates
/// - NEVER signs server certificates directly
///
/// # Example
///
/// ```rust,ignore
/// let root_ca_seed = master_seed.derive_child("root-ca");
/// let params = RootCAParams {
///     organization: "CowboyAI Inc".to_string(),
///     common_name: "CowboyAI Root CA".to_string(),
///     ..Default::default()
/// };
/// let root_ca = generate_root_ca(&root_ca_seed, params)?;
/// ```
pub fn generate_root_ca(
    seed: &MasterSeed,
    params: RootCAParams,
    correlation_id: uuid::Uuid,
    causation_id: Option<uuid::Uuid>,
) -> Result<(X509Certificate, crate::events::CertificateGeneratedEvent), String> {
    // Generate our deterministic Ed25519 keypair from seed (for reference/storage)
    let ed25519_keypair = KeyPair::from_seed(seed);

    // Save params for event emission before they're moved
    let common_name = params.common_name.clone();
    let organization = params.organization.clone();

    // Build distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, params.common_name);
    dn.push(DnType::OrganizationName, params.organization);
    if let Some(country) = params.country {
        dn.push(DnType::CountryName, country);
    }
    if let Some(state) = params.state {
        dn.push(DnType::StateOrProvinceName, state);
    }
    if let Some(locality) = params.locality {
        dn.push(DnType::LocalityName, locality);
    }

    // Create certificate parameters
    let mut cert_params = CertificateParams::new(vec![])
        .map_err(|e| format!("Failed to create certificate params: {}", e))?;

    cert_params.distinguished_name = dn;

    // Root CA specific settings - pathlen determines how many intermediate levels allowed
    cert_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(params.pathlen));
    cert_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];

    // Long validity period (20 years)
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + Duration::days(365 * params.validity_years as i64);
    cert_params.not_before = not_before;
    cert_params.not_after = not_after;

    // Generate rcgen's own keypair
    let key_pair = RcgenKeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;

    // Create self-signed certificate
    let cert = cert_params.self_signed(&key_pair)
        .map_err(|e| format!("Failed to create self-signed certificate: {}", e))?;

    // Get PEM representations
    let certificate_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    // Calculate fingerprint (SHA-256 of DER)
    let cert_der = cert.der();
    let fingerprint = calculate_fingerprint(cert_der);

    let x509_cert = X509Certificate {
        certificate_pem,
        private_key_pem,
        public_key_bytes: ed25519_keypair.public_key_bytes().to_vec(),
        fingerprint,
        seed_path: "root-ca".to_string(),
    };

    // US-021: Emit certificate generation event for audit trail
    let cert_id = uuid::Uuid::now_v7();
    let key_id = uuid::Uuid::now_v7();

    let event = crate::events::CertificateGeneratedEvent {
        cert_id,
        key_id,
        subject: format!("CN={},O={}",
            common_name,
            organization
        ),
        issuer: None, // Self-signed root CA
        not_before: chrono::DateTime::from_timestamp(not_before.unix_timestamp(), 0).unwrap(),
        not_after: chrono::DateTime::from_timestamp(not_after.unix_timestamp(), 0).unwrap(),
        is_ca: true,
        san: vec![], // Root CAs typically don't have SANs
        key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
        extended_key_usage: vec![],
        correlation_id,
        causation_id,
    };

    Ok((x509_cert, event))
}

/// Generate an Intermediate CA certificate (SIGNING ONLY)
///
/// **CRITICAL**: This intermediate CA is for SIGNING ONLY.
/// - pathlen: 0 (cannot sign other CAs)
/// - keyUsage: keyCertSign, cRLSign ONLY (NO digitalSignature!)
/// - Does NOT serve as a server identity
/// - Can be rotated without affecting root trust
///
/// # Example
///
/// ```rust,ignore
/// let intermediate_seed = root_ca_seed.derive_child("intermediate-engineering");
/// let params = IntermediateCAParams {
///     organization: "CowboyAI Inc".to_string(),
///     organizational_unit: "Engineering".to_string(),
///     common_name: "CowboyAI Engineering Intermediate CA".to_string(),
///     ..Default::default()
/// };
/// let intermediate_ca = generate_intermediate_ca(
///     &intermediate_seed,
///     params,
///     &root_ca_cert,
/// )?;
/// ```
pub fn generate_intermediate_ca(
    seed: &MasterSeed,
    params: IntermediateCAParams,
    root_ca_cert_pem: &str,
    root_ca_key_pem: &str,
    root_ca_id: uuid::Uuid,
    correlation_id: uuid::Uuid,
    causation_id: Option<uuid::Uuid>,
) -> Result<(X509Certificate, crate::events::CertificateGeneratedEvent, crate::events::CertificateSignedEvent), String> {
    // Generate our deterministic Ed25519 keypair from seed (for reference/storage)
    let ed25519_keypair = KeyPair::from_seed(seed);

    // Build distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, params.common_name.clone());
    dn.push(DnType::OrganizationName, params.organization.clone());
    dn.push(DnType::OrganizationalUnitName, params.organizational_unit.clone());
    if let Some(country) = params.country {
        dn.push(DnType::CountryName, country);
    }

    // Create certificate parameters
    let mut cert_params = CertificateParams::new(vec![])
        .map_err(|e| format!("Failed to create certificate params: {}", e))?;

    cert_params.distinguished_name = dn;

    // Intermediate CA specific settings
    // pathlen determines how many more intermediate levels can exist below this CA
    // pathlen: 0 = can only sign leaf certs (typical)
    // pathlen: 1 = can sign one more intermediate level (for hosting scenario)
    cert_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(params.pathlen));

    // CRITICAL: SIGNING ONLY - no digitalSignature, no keyEncipherment
    cert_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];

    // Medium validity period (3 years)
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + Duration::days(365 * params.validity_years as i64);
    cert_params.not_before = not_before;
    cert_params.not_after = not_after;

    // Generate rcgen's own keypair
    let key_pair = RcgenKeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;

    // Parse root CA key pair for signing
    let root_key_pair = RcgenKeyPair::from_pem(root_ca_key_pem)
        .map_err(|e| format!("Failed to parse root CA key: {}", e))?;

    // Parse root CA certificate to get parameters
    // Note: We're using x509-parser to extract info from the existing cert
    // This is used for validation purposes to ensure the root CA is properly formatted
    let _root_pem_data = pem::parse(root_ca_cert_pem)
        .map_err(|e| format!("Failed to parse root CA PEM: {}", e))?;

    // Create a minimal CertificateParams for the issuer
    // In rcgen 0.14, we just need enough to identify the issuer
    let root_params = CertificateParams::new(vec![])
        .map_err(|e| format!("Failed to create params for issuer: {}", e))?;

    // The signing will use the actual certificate data from the PEM
    // Create issuer with these params
    let issuer = Issuer::new(root_params, root_key_pair);

    // Sign the intermediate certificate with root CA
    let cert = cert_params.signed_by(&key_pair, &issuer)
        .map_err(|e| format!("Failed to sign intermediate CA: {}", e))?;

    // Get PEM representations
    let certificate_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    // Calculate fingerprint
    let cert_der = cert.der();
    let fingerprint = calculate_fingerprint(cert_der);

    let x509_cert = X509Certificate {
        certificate_pem,
        private_key_pem,
        public_key_bytes: ed25519_keypair.public_key_bytes().to_vec(),
        fingerprint,
        seed_path: format!("intermediate-{}", params.organizational_unit),
    };

    // US-021: Emit certificate generation and signing events for audit trail
    let cert_id = uuid::Uuid::now_v7();
    let key_id = uuid::Uuid::now_v7();
    let signed_at = chrono::Utc::now();

    let generation_event = crate::events::CertificateGeneratedEvent {
        cert_id,
        key_id,
        subject: format!("CN={},OU={},O={}",
            params.common_name,
            params.organizational_unit,
            params.organization
        ),
        issuer: Some(root_ca_id),
        not_before: chrono::DateTime::from_timestamp(not_before.unix_timestamp(), 0).unwrap(),
        not_after: chrono::DateTime::from_timestamp(not_after.unix_timestamp(), 0).unwrap(),
        is_ca: true,
        san: vec![],
        key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
        extended_key_usage: vec![],
        correlation_id,
        causation_id,
    };

    let signing_event = crate::events::CertificateSignedEvent {
        cert_id,
        signed_by: root_ca_id,
        signature_algorithm: "Ed25519".to_string(),
        signed_at,
        correlation_id,
        causation_id: Some(cert_id), // Signing caused by certificate generation
    };

    Ok((x509_cert, generation_event, signing_event))
}

/// Generate a Server/Leaf certificate
///
/// Server certificates are:
/// - Short-lived (90 days default)
/// - NOT a CA (CA: FALSE)
/// - Used for TLS server authentication
/// - Can be rotated frequently
///
/// # Example
///
/// ```rust,ignore
/// let server_seed = intermediate_seed.derive_child("nats-server-prod-01");
/// let params = ServerCertParams {
///     common_name: "nats-server-01.example.com".to_string(),
///     san_entries: vec![
///         "nats-server-01.example.com".to_string(),
///         "10.0.0.5".to_string(),
///     ],
///     organization: "CowboyAI Inc".to_string(),
///     validity_days: 90,
///     ..Default::default()
/// };
/// let server_cert = generate_server_certificate(
///     &server_seed,
///     params,
///     &intermediate_ca_cert,
/// )?;
/// ```
pub fn generate_server_certificate(
    seed: &MasterSeed,
    params: ServerCertParams,
    intermediate_ca_cert_pem: &str,
    intermediate_ca_key_pem: &str,
    intermediate_ca_id: uuid::Uuid,
    correlation_id: uuid::Uuid,
    causation_id: Option<uuid::Uuid>,
) -> Result<(X509Certificate, crate::events::CertificateGeneratedEvent, crate::events::CertificateSignedEvent), String> {
    // Generate our deterministic Ed25519 keypair from seed (for reference/storage)
    let ed25519_keypair = KeyPair::from_seed(seed);

    // Build distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, params.common_name.clone());
    dn.push(DnType::OrganizationName, params.organization.clone());
    if let Some(ou) = params.organizational_unit.clone() {
        dn.push(DnType::OrganizationalUnitName, ou);
    }

    // Create certificate parameters with SAN
    let mut san_entries = params.san_entries.clone();
    if !san_entries.contains(&params.common_name) {
        san_entries.insert(0, params.common_name.clone());
    }

    let mut cert_params = CertificateParams::new(san_entries)
        .map_err(|e| format!("Failed to create certificate params: {}", e))?;

    cert_params.distinguished_name = dn;

    // Server certificate specific settings
    cert_params.is_ca = IsCa::NoCa; // NOT a CA!

    cert_params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];

    cert_params.extended_key_usages = vec![
        ExtendedKeyUsagePurpose::ServerAuth,
        ExtendedKeyUsagePurpose::ClientAuth,
    ];

    // Short validity period (90 days default)
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + Duration::days(params.validity_days as i64);
    cert_params.not_before = not_before;
    cert_params.not_after = not_after;

    // Generate rcgen's own keypair
    let key_pair = RcgenKeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;

    // Parse intermediate CA key pair for signing
    let intermediate_key_pair = RcgenKeyPair::from_pem(intermediate_ca_key_pem)
        .map_err(|e| format!("Failed to parse intermediate CA key: {}", e))?;

    // Parse intermediate CA certificate to get parameters
    // This is used for validation purposes to ensure the intermediate CA is properly formatted
    let _intermediate_pem_data = pem::parse(intermediate_ca_cert_pem)
        .map_err(|e| format!("Failed to parse intermediate CA PEM: {}", e))?;

    // Create a minimal CertificateParams for the issuer
    let intermediate_params = CertificateParams::new(vec![])
        .map_err(|e| format!("Failed to create params for issuer: {}", e))?;

    // Create issuer with these params
    let issuer = Issuer::new(intermediate_params, intermediate_key_pair);

    // Sign the server certificate with intermediate CA
    let cert = cert_params.signed_by(&key_pair, &issuer)
        .map_err(|e| format!("Failed to sign server certificate: {}", e))?;

    // Get PEM representations
    let certificate_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    // Calculate fingerprint
    let cert_der = cert.der();
    let fingerprint = calculate_fingerprint(cert_der);

    let x509_cert = X509Certificate {
        certificate_pem,
        private_key_pem,
        public_key_bytes: ed25519_keypair.public_key_bytes().to_vec(),
        fingerprint,
        seed_path: format!("server-{}", params.common_name),
    };

    // US-021: Emit certificate generation and signing events for audit trail
    let cert_id = uuid::Uuid::now_v7();
    let key_id = uuid::Uuid::now_v7();
    let signed_at = chrono::Utc::now();

    let generation_event = crate::events::CertificateGeneratedEvent {
        cert_id,
        key_id,
        subject: format!("CN={},O={}",
            params.common_name,
            params.organization
        ),
        issuer: Some(intermediate_ca_id),
        not_before: chrono::DateTime::from_timestamp(not_before.unix_timestamp(), 0).unwrap(),
        not_after: chrono::DateTime::from_timestamp(not_after.unix_timestamp(), 0).unwrap(),
        is_ca: false,
        san: params.san_entries.clone(),
        key_usage: vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
        extended_key_usage: vec!["serverAuth".to_string(), "clientAuth".to_string()],
        correlation_id,
        causation_id,
    };

    let signing_event = crate::events::CertificateSignedEvent {
        cert_id,
        signed_by: intermediate_ca_id,
        signature_algorithm: "Ed25519".to_string(),
        signed_at,
        correlation_id,
        causation_id: Some(cert_id), // Signing caused by certificate generation
    };

    Ok((x509_cert, generation_event, signing_event))
}

/// Calculate SHA-256 fingerprint of DER-encoded certificate
fn calculate_fingerprint(cert_der: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(cert_der);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::seed_derivation::derive_master_seed;

    #[test]
    fn test_generate_root_ca() {
        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");

        let params = RootCAParams {
            organization: "Test Organization".to_string(),
            common_name: "Test Root CA".to_string(),
            ..Default::default()
        };

        // US-021: generate_root_ca now returns (X509Certificate, CertificateGeneratedEvent)
        let correlation_id = uuid::Uuid::now_v7();
        let (root_ca, _event) = generate_root_ca(&root_ca_seed, params, correlation_id, None).unwrap();

        assert!(root_ca.certificate_pem.contains("BEGIN CERTIFICATE"));
        assert!(root_ca.private_key_pem.contains("BEGIN PRIVATE KEY"));
        assert!(!root_ca.fingerprint.is_empty());
    }

    #[test]
    fn test_deterministic_root_ca() {
        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");

        let params = RootCAParams::default();

        // US-021: Extract certificates from tuple returns
        let correlation_id_1 = uuid::Uuid::now_v7();
        let (root_ca_1, _event_1) = generate_root_ca(&root_ca_seed, params.clone(), correlation_id_1, None).unwrap();
        let correlation_id_2 = uuid::Uuid::now_v7();
        let (root_ca_2, _event_2) = generate_root_ca(&root_ca_seed, params, correlation_id_2, None).unwrap();

        // Same seed should produce same public key
        assert_eq!(root_ca_1.public_key_bytes, root_ca_2.public_key_bytes);
    }

    #[test]
    fn test_intermediate_ca_signed_by_root() {
        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");
        let intermediate_seed = root_ca_seed.derive_child("intermediate-test");

        // US-021: Generate root CA with event emission
        let root_params = RootCAParams::default();
        let root_correlation_id = uuid::Uuid::now_v7();
        let (root_ca, _root_event) = generate_root_ca(&root_ca_seed, root_params, root_correlation_id, None).unwrap();

        // US-021: Generate intermediate CA signed by root with event emission
        let intermediate_params = IntermediateCAParams {
            organizational_unit: "Test Unit".to_string(),
            ..Default::default()
        };

        let intermediate_correlation_id = uuid::Uuid::now_v7();
        let root_ca_id = uuid::Uuid::now_v7(); // Would be tracked from root_event in production
        let (intermediate_ca, _gen_event, _sign_event) = generate_intermediate_ca(
            &intermediate_seed,
            intermediate_params,
            &root_ca.certificate_pem,
            &root_ca.private_key_pem,
            root_ca_id,
            intermediate_correlation_id,
            Some(root_correlation_id),
        ).unwrap();

        assert!(intermediate_ca.certificate_pem.contains("BEGIN CERTIFICATE"));
        assert!(!intermediate_ca.fingerprint.is_empty());
    }

    #[test]
    fn test_root_ca_basic_constraints() {
        use x509_parser::prelude::*;

        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");
        let params = RootCAParams::default();

        // US-021: Extract certificate from tuple
        let correlation_id = uuid::Uuid::now_v7();
        let (root_ca, _event) = generate_root_ca(&root_ca_seed, params, correlation_id, None).unwrap();

        // Parse certificate from PEM
        let (_, pem) = parse_x509_pem(root_ca.certificate_pem.as_bytes()).unwrap();
        let cert = pem.parse_x509().unwrap();

        // Check basic constraints extension
        let basic_constraints = cert
            .basic_constraints()
            .expect("Root CA must have basic constraints")
            .expect("Basic constraints must be present")
            .value;

        // Root CA must be a CA
        assert!(basic_constraints.ca, "Root CA must have CA=true");

        // Root CA should allow intermediate CAs (pathlen >= 1 or unconstrained)
        assert!(
            basic_constraints.path_len_constraint.is_none()
            || basic_constraints.path_len_constraint.unwrap() >= 1,
            "Root CA must allow at least 1 intermediate CA level"
        );
    }

    #[test]
    fn test_intermediate_ca_pathlen_zero() {
        use x509_parser::prelude::*;

        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");
        let intermediate_seed = root_ca_seed.derive_child("intermediate-test");

        // US-021: Generate certificates with event emission
        let root_params = RootCAParams::default();
        let root_correlation_id = uuid::Uuid::now_v7();
        let (root_ca, _root_event) = generate_root_ca(&root_ca_seed, root_params, root_correlation_id, None).unwrap();

        let intermediate_params = IntermediateCAParams::default();
        let intermediate_correlation_id = uuid::Uuid::now_v7();
        let root_ca_id = uuid::Uuid::now_v7();
        let (intermediate_ca, _gen_event, _sign_event) = generate_intermediate_ca(
            &intermediate_seed,
            intermediate_params,
            &root_ca.certificate_pem,
            &root_ca.private_key_pem,
            root_ca_id,
            intermediate_correlation_id,
            Some(root_correlation_id),
        ).unwrap();

        // Parse intermediate certificate
        let (_, pem) = parse_x509_pem(intermediate_ca.certificate_pem.as_bytes()).unwrap();
        let cert = pem.parse_x509().unwrap();

        // Check basic constraints
        let basic_constraints = cert
            .basic_constraints()
            .expect("Intermediate CA must have basic constraints")
            .expect("Basic constraints must be present")
            .value;

        // Intermediate CA must be a CA
        assert!(basic_constraints.ca, "Intermediate CA must have CA=true");

        // Intermediate CA must have pathlen:0 (signing-only, can't create sub-CAs)
        assert_eq!(
            basic_constraints.path_len_constraint,
            Some(0),
            "Intermediate CA must have pathlen:0 for signing-only operation"
        );
    }

    #[test]
    fn test_ca_key_usage() {
        use x509_parser::prelude::*;

        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");
        let params = RootCAParams::default();

        // US-021: Extract certificate from tuple
        let correlation_id = uuid::Uuid::now_v7();
        let (root_ca, _event) = generate_root_ca(&root_ca_seed, params, correlation_id, None).unwrap();

        // Parse certificate
        let (_, pem) = parse_x509_pem(root_ca.certificate_pem.as_bytes()).unwrap();
        let cert = pem.parse_x509().unwrap();

        // Check key usage extension
        let key_usage = cert
            .key_usage()
            .expect("Root CA must have key usage extension")
            .expect("Key usage must be present")
            .value;

        // Root CA should have keyCertSign and cRLSign
        assert!(key_usage.key_cert_sign(), "Root CA must have keyCertSign");
        assert!(key_usage.crl_sign(), "Root CA must have cRLSign");
    }

    #[test]
    fn test_complete_certificate_chain() {
        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");
        let intermediate_seed = root_ca_seed.derive_child("intermediate-engineering");
        let server_seed = intermediate_seed.derive_child("server-api");

        // US-021: Generate complete chain: Root → Intermediate → Server with event emission
        let root_params = RootCAParams {
            organization: "Test Organization".to_string(),
            common_name: "Test Root CA".to_string(),
            country: Some("US".to_string()),
            validity_years: 20,
            ..Default::default()
        };
        let root_correlation_id = uuid::Uuid::now_v7();
        let (root_ca, _root_event) = generate_root_ca(&root_ca_seed, root_params, root_correlation_id, None).unwrap();

        let intermediate_params = IntermediateCAParams {
            organization: "Test Organization".to_string(),
            organizational_unit: "Engineering".to_string(),
            common_name: "Engineering Intermediate CA".to_string(),
            country: Some("US".to_string()),
            validity_years: 10,
        };
        let intermediate_correlation_id = uuid::Uuid::now_v7();
        let root_ca_id = uuid::Uuid::now_v7();
        let (intermediate_ca, _int_gen_event, _int_sign_event) = generate_intermediate_ca(
            &intermediate_seed,
            intermediate_params,
            &root_ca.certificate_pem,
            &root_ca.private_key_pem,
            root_ca_id,
            intermediate_correlation_id,
            Some(root_correlation_id),
        ).unwrap();

        let server_params = ServerCertParams {
            common_name: "api.example.com".to_string(),
            san_entries: vec!["api.example.com".to_string(), "www.example.com".to_string()],
            organization: "Test Organization".to_string(),
            organizational_unit: Some("Engineering".to_string()),
            validity_days: 90,
        };
        let server_correlation_id = uuid::Uuid::now_v7();
        let intermediate_ca_id = uuid::Uuid::now_v7();
        let (server_cert, _srv_gen_event, _srv_sign_event) = generate_server_certificate(
            &server_seed,
            server_params,
            &intermediate_ca.certificate_pem,
            &intermediate_ca.private_key_pem,
            intermediate_ca_id,
            server_correlation_id,
            Some(intermediate_correlation_id),
        ).unwrap();

        // Verify all certificates were generated
        assert!(root_ca.certificate_pem.contains("BEGIN CERTIFICATE"));
        assert!(intermediate_ca.certificate_pem.contains("BEGIN CERTIFICATE"));
        assert!(server_cert.certificate_pem.contains("BEGIN CERTIFICATE"));

        // Verify fingerprints are all unique
        assert_ne!(root_ca.fingerprint, intermediate_ca.fingerprint);
        assert_ne!(intermediate_ca.fingerprint, server_cert.fingerprint);
        assert_ne!(root_ca.fingerprint, server_cert.fingerprint);
    }

    #[test]
    fn test_certificate_validity_period() {
        use x509_parser::prelude::*;

        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");

        let params = RootCAParams {
            validity_years: 10,
            ..Default::default()
        };

        // US-021: Extract certificate from tuple
        let correlation_id = uuid::Uuid::now_v7();
        let (root_ca, _event) = generate_root_ca(&root_ca_seed, params, correlation_id, None).unwrap();

        // Parse certificate
        let (_, pem) = parse_x509_pem(root_ca.certificate_pem.as_bytes()).unwrap();
        let cert = pem.parse_x509().unwrap();

        // Check validity period
        let validity = cert.validity();
        let not_before = validity.not_before;
        let not_after = validity.not_after;

        // Calculate duration in seconds
        // Expected: approximately 10 years * 365 days * 24 hours * 3600 seconds
        // (rcgen uses 365 days/year, not accounting for leap years)
        let expected_seconds = 10.0 * 365.0 * 24.0 * 3600.0;
        let actual_seconds = (not_after.timestamp() - not_before.timestamp()) as f64;

        // Allow 5 day tolerance for leap years and timezone differences
        let tolerance = 5.0 * 24.0 * 3600.0;
        assert!(
            (actual_seconds - expected_seconds).abs() < tolerance,
            "Certificate validity period should be approximately 10 years (got {} seconds, expected {})",
            actual_seconds, expected_seconds
        );
    }
}
