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
    Certificate, CertificateParams, DistinguishedName, DnType,
    KeyUsagePurpose, ExtendedKeyUsagePurpose, IsCa, BasicConstraints,
    CertificateSigningRequest, KeyPair as RcgenKeyPair,
};
use time::{Duration, OffsetDateTime};

/// X.509 certificate with associated keypair
#[derive(Clone)]
pub struct X509Certificate {
    /// The certificate in PEM format
    pub certificate_pem: String,
    /// The private key in PEM format (for non-CA certs or initial distribution)
    pub private_key_pem: String,
    /// The public key bytes
    pub public_key_bytes: Vec<u8>,
    /// Certificate fingerprint (SHA-256)
    pub fingerprint: String,
}

/// Parameters for Root CA generation
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
) -> Result<X509Certificate, String> {
    // Generate keypair deterministically from seed
    let keypair = KeyPair::from_seed(seed);

    // Convert Ed25519 keypair to rcgen KeyPair
    let rcgen_keypair = ed25519_to_rcgen_keypair(&keypair)?;

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
    cert_params.key_pair = Some(rcgen_keypair);

    // Root CA specific settings
    cert_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(1)); // pathlen: 1
    cert_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];

    // Long validity period (20 years)
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + Duration::days(365 * params.validity_years as i64);
    cert_params.not_before = not_before;
    cert_params.not_after = not_after;

    // Generate self-signed certificate
    let cert = Certificate::generate_self_signed(cert_params)
        .map_err(|e| format!("Failed to generate root CA: {}", e))?;

    let certificate_pem = cert.serialize_pem()
        .map_err(|e| format!("Failed to serialize certificate: {}", e))?;
    let private_key_pem = cert.serialize_private_key_pem();

    // Calculate fingerprint (SHA-256 of DER)
    let cert_der = cert.serialize_der()
        .map_err(|e| format!("Failed to serialize DER: {}", e))?;
    let fingerprint = calculate_fingerprint(&cert_der);

    Ok(X509Certificate {
        certificate_pem,
        private_key_pem,
        public_key_bytes: keypair.public_key_bytes().to_vec(),
        fingerprint,
    })
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
) -> Result<X509Certificate, String> {
    // Generate keypair deterministically from seed
    let keypair = KeyPair::from_seed(seed);
    let rcgen_keypair = ed25519_to_rcgen_keypair(&keypair)?;

    // Build distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, params.common_name);
    dn.push(DnType::OrganizationName, params.organization.clone());
    dn.push(DnType::OrganizationalUnitName, params.organizational_unit);
    if let Some(country) = params.country {
        dn.push(DnType::CountryName, country);
    }

    // Create certificate parameters
    let mut cert_params = CertificateParams::new(vec![])
        .map_err(|e| format!("Failed to create certificate params: {}", e))?;

    cert_params.distinguished_name = dn;
    cert_params.key_pair = Some(rcgen_keypair);

    // Intermediate CA specific settings
    // CRITICAL: pathlen 0 means this CA cannot sign other CAs!
    cert_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0)); // pathlen: 0

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

    // Parse root CA for signing
    let root_ca = parse_certificate_and_key(root_ca_cert_pem, root_ca_key_pem)?;

    // Generate certificate signed by root CA
    let cert = Certificate::generate(cert_params, &root_ca)
        .map_err(|e| format!("Failed to generate intermediate CA: {}", e))?;

    let certificate_pem = cert.serialize_pem_with_signer(&root_ca)
        .map_err(|e| format!("Failed to sign intermediate CA: {}", e))?;
    let private_key_pem = cert.serialize_private_key_pem();

    // Calculate fingerprint
    let cert_der = cert.serialize_der_with_signer(&root_ca)
        .map_err(|e| format!("Failed to serialize DER: {}", e))?;
    let fingerprint = calculate_fingerprint(&cert_der);

    Ok(X509Certificate {
        certificate_pem,
        private_key_pem,
        public_key_bytes: keypair.public_key_bytes().to_vec(),
        fingerprint,
    })
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
) -> Result<X509Certificate, String> {
    // Generate keypair deterministically from seed
    let keypair = KeyPair::from_seed(seed);
    let rcgen_keypair = ed25519_to_rcgen_keypair(&keypair)?;

    // Build distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, params.common_name.clone());
    dn.push(DnType::OrganizationName, params.organization);
    if let Some(ou) = params.organizational_unit {
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
    cert_params.key_pair = Some(rcgen_keypair);

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

    // Parse intermediate CA for signing
    let intermediate_ca = parse_certificate_and_key(intermediate_ca_cert_pem, intermediate_ca_key_pem)?;

    // Generate certificate signed by intermediate CA
    let cert = Certificate::generate(cert_params, &intermediate_ca)
        .map_err(|e| format!("Failed to generate server certificate: {}", e))?;

    let certificate_pem = cert.serialize_pem_with_signer(&intermediate_ca)
        .map_err(|e| format!("Failed to sign server certificate: {}", e))?;
    let private_key_pem = cert.serialize_private_key_pem();

    // Calculate fingerprint
    let cert_der = cert.serialize_der_with_signer(&intermediate_ca)
        .map_err(|e| format!("Failed to serialize DER: {}", e))?;
    let fingerprint = calculate_fingerprint(&cert_der);

    Ok(X509Certificate {
        certificate_pem,
        private_key_pem,
        public_key_bytes: keypair.public_key_bytes().to_vec(),
        fingerprint,
    })
}

/// Convert Ed25519 KeyPair to rcgen KeyPair
fn ed25519_to_rcgen_keypair(keypair: &KeyPair) -> Result<RcgenKeyPair, String> {
    // rcgen expects PKCS#8 format for Ed25519
    // Ed25519 private key is 32 bytes, public key is 32 bytes
    let private_key_bytes = keypair.private_key_bytes();

    // Create PKCS#8 formatted key
    RcgenKeyPair::from_der(&private_key_bytes)
        .or_else(|_| {
            // Try alternative format
            RcgenKeyPair::from_pkcs8_pem(&format!(
                "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &private_key_bytes)
            ))
        })
        .map_err(|e| format!("Failed to convert Ed25519 key to rcgen format: {}", e))
}

/// Parse certificate and key from PEM strings
fn parse_certificate_and_key(cert_pem: &str, key_pem: &str) -> Result<Certificate, String> {
    let key_pair = RcgenKeyPair::from_pem(key_pem)
        .map_err(|e| format!("Failed to parse key: {}", e))?;

    let params = CertificateParams::from_ca_cert_pem(cert_pem)
        .map_err(|e| format!("Failed to parse certificate: {}", e))?;

    // Create a new certificate with the key pair
    let mut params_with_key = params?;
    params_with_key.key_pair = Some(key_pair);

    Certificate::generate_self_signed(params_with_key)
        .map_err(|e| format!("Failed to create certificate from params: {}", e))
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

        let root_ca = generate_root_ca(&root_ca_seed, params).unwrap();

        assert!(root_ca.certificate_pem.contains("BEGIN CERTIFICATE"));
        assert!(root_ca.private_key_pem.contains("BEGIN PRIVATE KEY"));
        assert!(!root_ca.fingerprint.is_empty());
    }

    #[test]
    fn test_deterministic_root_ca() {
        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");

        let params = RootCAParams::default();

        let root_ca_1 = generate_root_ca(&root_ca_seed, params.clone()).unwrap();
        let root_ca_2 = generate_root_ca(&root_ca_seed, params).unwrap();

        // Same seed should produce same public key
        assert_eq!(root_ca_1.public_key_bytes, root_ca_2.public_key_bytes);
    }

    #[test]
    fn test_intermediate_ca_signed_by_root() {
        let master_seed = derive_master_seed("test passphrase", "test-org").unwrap();
        let root_ca_seed = master_seed.derive_child("root-ca");
        let intermediate_seed = root_ca_seed.derive_child("intermediate-test");

        // Generate root CA
        let root_params = RootCAParams::default();
        let root_ca = generate_root_ca(&root_ca_seed, root_params).unwrap();

        // Generate intermediate CA signed by root
        let intermediate_params = IntermediateCAParams {
            organizational_unit: "Test Unit".to_string(),
            ..Default::default()
        };

        let intermediate_ca = generate_intermediate_ca(
            &intermediate_seed,
            intermediate_params,
            &root_ca.certificate_pem,
            &root_ca.private_key_pem,
        ).unwrap();

        assert!(intermediate_ca.certificate_pem.contains("BEGIN CERTIFICATE"));
        assert!(!intermediate_ca.fingerprint.is_empty());
    }
}
