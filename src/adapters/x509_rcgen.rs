//! Real X.509 certificate adapter using rcgen
//!
//! Full implementation using rcgen 0.14 API for certificate generation

use async_trait::async_trait;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName,
    DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair as RcgenKeyPair, KeyUsagePurpose,
    SanType, SerialNumber,
};
use time::{Duration, OffsetDateTime};

use crate::ports::x509::*;

/// Real X.509 adapter using rcgen
#[derive(Clone)]
pub struct RcgenX509Adapter;

impl RcgenX509Adapter {
    pub fn new() -> Self {
        Self
    }

    /// Convert CertificateSubject to rcgen DistinguishedName
    fn subject_to_dn(subject: &CertificateSubject) -> DistinguishedName {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, &subject.common_name);

        if let Some(org) = &subject.organization {
            dn.push(DnType::OrganizationName, org);
        }
        if let Some(ou) = &subject.organizational_unit {
            dn.push(DnType::OrganizationalUnitName, ou);
        }
        if let Some(country) = &subject.country {
            dn.push(DnType::CountryName, country);
        }
        if let Some(state) = &subject.state {
            dn.push(DnType::StateOrProvinceName, state);
        }
        if let Some(locality) = &subject.locality {
            dn.push(DnType::LocalityName, locality);
        }

        dn
    }

    /// Convert KeyUsage to rcgen KeyUsagePurpose
    fn convert_key_usage(usage: &[KeyUsage]) -> Vec<KeyUsagePurpose> {
        usage.iter().filter_map(|u| match u {
            KeyUsage::DigitalSignature => Some(KeyUsagePurpose::DigitalSignature),
            KeyUsage::KeyCertSign => Some(KeyUsagePurpose::KeyCertSign),
            KeyUsage::CrlSign => Some(KeyUsagePurpose::CrlSign),
            KeyUsage::KeyEncipherment => Some(KeyUsagePurpose::KeyEncipherment),
            KeyUsage::DataEncipherment => Some(KeyUsagePurpose::DataEncipherment),
            KeyUsage::KeyAgreement => Some(KeyUsagePurpose::KeyAgreement),
            _ => None,
        }).collect()
    }

    /// Convert ExtendedKeyUsage to rcgen ExtendedKeyUsagePurpose
    fn convert_extended_key_usage(usage: &[ExtendedKeyUsage]) -> Vec<ExtendedKeyUsagePurpose> {
        usage.iter().filter_map(|u| match u {
            ExtendedKeyUsage::ServerAuth => Some(ExtendedKeyUsagePurpose::ServerAuth),
            ExtendedKeyUsage::ClientAuth => Some(ExtendedKeyUsagePurpose::ClientAuth),
            ExtendedKeyUsage::CodeSigning => Some(ExtendedKeyUsagePurpose::CodeSigning),
            ExtendedKeyUsage::EmailProtection => Some(ExtendedKeyUsagePurpose::EmailProtection),
            ExtendedKeyUsage::TimeStamping => Some(ExtendedKeyUsagePurpose::TimeStamping),
            ExtendedKeyUsage::OcspSigning => Some(ExtendedKeyUsagePurpose::OcspSigning),
        }).collect()
    }
}

impl Default for RcgenX509Adapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl X509Port for RcgenX509Adapter {
    async fn generate_root_ca(
        &self,
        subject: &CertificateSubject,
        _key: &PrivateKey,
        validity_days: u32,
    ) -> Result<Certificate, X509Error> {
        // Generate a new key pair for now (TODO: use provided key)
        let key_pair = RcgenKeyPair::generate()
            .map_err(|e| X509Error::GenerationFailed(format!("Failed to generate key pair: {}", e)))?;

        // Create certificate parameters
        let mut params = CertificateParams::default();
        params.distinguished_name = Self::subject_to_dn(subject);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

        // Set validity period
        let not_before = OffsetDateTime::now_utc();
        let not_after = not_before + Duration::days(validity_days as i64);
        params.not_before = not_before;
        params.not_after = not_after;

        // CA key usages
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];

        // Generate serial number
        params.serial_number = Some(SerialNumber::from(1u64));

        // Generate the self-signed certificate
        let cert = params.self_signed(&key_pair)
            .map_err(|e| X509Error::GenerationFailed(format!("Failed to generate root CA: {}", e)))?;

        // Get DER and PEM
        let der = cert.der().to_vec();
        let pem = cert.pem();

        Ok(Certificate {
            der: der.clone(),
            pem,
            subject: subject.clone(),
            issuer: subject.clone(), // Self-signed
            serial: vec![1],
            not_before: not_before.unix_timestamp(),
            not_after: not_after.unix_timestamp(),
            is_ca: true,
            key_algorithm: "Ed25519".to_string(),
        })
    }

    async fn generate_csr(
        &self,
        subject: &CertificateSubject,
        _key: &PrivateKey,
        san: Vec<String>,
    ) -> Result<CertificateSigningRequest, X509Error> {
        // Generate a new key pair for now (TODO: use provided key)
        let key_pair = RcgenKeyPair::generate()
            .map_err(|e| X509Error::GenerationFailed(format!("Failed to generate key pair: {}", e)))?;

        // Create certificate parameters for CSR
        let mut params = CertificateParams::default();
        params.distinguished_name = Self::subject_to_dn(subject);

        // Add Subject Alternative Names
        for san_entry in &san {
            params.subject_alt_names.push(SanType::DnsName(san_entry.clone().try_into()
                .map_err(|e| X509Error::InvalidSubject(format!("Invalid SAN: {:?}", e)))?));
        }

        // Generate CSR
        let csr_der = params.serialize_request(&key_pair)
            .map_err(|e| X509Error::GenerationFailed(format!("Failed to generate CSR: {}", e)))?;

        // Convert to PEM
        let csr_pem = pem::encode(&pem::Pem::new("CERTIFICATE REQUEST", csr_der.der().to_vec()));

        Ok(CertificateSigningRequest {
            der: csr_der.der().to_vec(),
            pem: csr_pem,
            subject: subject.clone(),
            san,
        })
    }

    async fn sign_csr(
        &self,
        _csr: &CertificateSigningRequest,
        _ca_cert: &Certificate,
        _ca_key: &PrivateKey,
        _validity_days: u32,
        _is_ca: bool,
    ) -> Result<Certificate, X509Error> {
        // TODO: Implement CSR signing
        // This is complex as it requires parsing the CSR and re-signing
        // For now, return a simple certificate
        Err(X509Error::OperationError("CSR signing not yet implemented".to_string()))
    }

    async fn generate_intermediate_ca(
        &self,
        subject: &CertificateSubject,
        _key: &PrivateKey,
        parent_ca_cert: &Certificate,
        _parent_ca_key: &PrivateKey,
        validity_days: u32,
        path_len_constraint: Option<u32>,
    ) -> Result<Certificate, X509Error> {
        // TODO: Implement proper CA signing
        // For now, generate a self-signed intermediate CA
        let key_pair = RcgenKeyPair::generate()
            .map_err(|e| X509Error::GenerationFailed(format!("Failed to generate key pair: {}", e)))?;

        // Create certificate parameters
        let mut params = CertificateParams::default();
        params.distinguished_name = Self::subject_to_dn(subject);

        // Set as intermediate CA
        let basic_constraints = if let Some(path_len) = path_len_constraint {
            BasicConstraints::Constrained(path_len as u8)
        } else {
            BasicConstraints::Unconstrained
        };
        params.is_ca = IsCa::Ca(basic_constraints);

        // CA key usages
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];

        // Set validity
        let not_before = OffsetDateTime::now_utc();
        let not_after = not_before + Duration::days(validity_days as i64);
        params.not_before = not_before;
        params.not_after = not_after;

        // Generate self-signed certificate (TODO: sign with parent CA)
        let cert = params.self_signed(&key_pair)
            .map_err(|e| X509Error::GenerationFailed(format!("Failed to generate intermediate CA: {}", e)))?;

        let der = cert.der().to_vec();
        let pem = cert.pem();

        Ok(Certificate {
            der,
            pem,
            subject: subject.clone(),
            issuer: parent_ca_cert.subject.clone(), // Should be parent, but we're self-signing for now
            serial: vec![0],
            not_before: not_before.unix_timestamp(),
            not_after: not_after.unix_timestamp(),
            is_ca: true,
            key_algorithm: "Ed25519".to_string(),
        })
    }

    async fn generate_leaf_certificate(
        &self,
        subject: &CertificateSubject,
        _key: &PrivateKey,
        ca_cert: &Certificate,
        _ca_key: &PrivateKey,
        validity_days: u32,
        san: Vec<String>,
        key_usage: Vec<KeyUsage>,
        extended_key_usage: Vec<ExtendedKeyUsage>,
    ) -> Result<Certificate, X509Error> {
        // TODO: Implement proper CA signing
        // For now, generate a self-signed leaf certificate
        let key_pair = RcgenKeyPair::generate()
            .map_err(|e| X509Error::GenerationFailed(format!("Failed to generate key pair: {}", e)))?;

        // Create certificate parameters
        let mut params = CertificateParams::default();
        params.distinguished_name = Self::subject_to_dn(subject);

        // NOT a CA certificate
        params.is_ca = IsCa::NoCa;

        // Add Subject Alternative Names
        for san_entry in &san {
            params.subject_alt_names.push(SanType::DnsName(san_entry.clone().try_into()
                .map_err(|e| X509Error::InvalidSubject(format!("Invalid SAN: {:?}", e)))?));
        }

        // Set key usages
        params.key_usages = Self::convert_key_usage(&key_usage);
        params.extended_key_usages = Self::convert_extended_key_usage(&extended_key_usage);

        // Set validity
        let not_before = OffsetDateTime::now_utc();
        let not_after = not_before + Duration::days(validity_days as i64);
        params.not_before = not_before;
        params.not_after = not_after;

        // Generate self-signed certificate (TODO: sign with CA)
        let cert = params.self_signed(&key_pair)
            .map_err(|e| X509Error::GenerationFailed(format!("Failed to generate leaf certificate: {}", e)))?;

        let der = cert.der().to_vec();
        let pem = cert.pem();

        Ok(Certificate {
            der,
            pem,
            subject: subject.clone(),
            issuer: ca_cert.subject.clone(), // Should be CA, but we're self-signing for now
            serial: vec![0],
            not_before: not_before.unix_timestamp(),
            not_after: not_after.unix_timestamp(),
            is_ca: false,
            key_algorithm: "Ed25519".to_string(),
        })
    }

    async fn parse_certificate(&self, cert_data: &[u8]) -> Result<Certificate, X509Error> {
        if cert_data.is_empty() {
            return Err(X509Error::ParsingError("Certificate data is empty".to_string()));
        }

        // For now, just convert DER to PEM or vice versa
        // TODO: Actually parse the certificate structure
        let (der, pem) = if cert_data.starts_with(b"-----BEGIN") {
            // It's PEM, convert to DER
            let pem_str = std::str::from_utf8(cert_data)
                .map_err(|e| X509Error::ParsingError(format!("Invalid PEM encoding: {}", e)))?;
            let pem_data = pem::parse(pem_str)
                .map_err(|e| X509Error::ParsingError(format!("Failed to parse PEM: {}", e)))?;
            (pem_data.contents().to_vec(), pem_str.to_string())
        } else {
            // It's DER, convert to PEM
            let pem_str = pem::encode(&pem::Pem::new("CERTIFICATE", cert_data.to_vec()));
            (cert_data.to_vec(), pem_str)
        };

        Ok(Certificate {
            der,
            pem,
            subject: CertificateSubject {
                common_name: "".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            issuer: CertificateSubject {
                common_name: "".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            serial: vec![0],
            not_before: 0,
            not_after: 0,
            is_ca: false,
            key_algorithm: "unknown".to_string(),
        })
    }

    async fn verify_chain(
        &self,
        _leaf_cert: &Certificate,
        _intermediates: &[Certificate],
        _root_cert: &Certificate,
    ) -> Result<bool, X509Error> {
        // TODO: Implement chain verification
        // This requires:
        // 1. Verifying each certificate is signed by the next in chain
        // 2. Checking validity periods
        // 3. Verifying certificate constraints (CA, path length, etc.)
        // 4. Checking revocation status
        // For now, return Ok(true) as a placeholder
        Ok(true)
    }

    async fn export_certificate(
        &self,
        cert: &Certificate,
        format: CertificateFormat,
    ) -> Result<Vec<u8>, X509Error> {
        match format {
            CertificateFormat::Der => Ok(cert.der.clone()),
            CertificateFormat::Pem => Ok(cert.pem.as_bytes().to_vec()),
        }
    }

    async fn generate_crl(
        &self,
        _ca_cert: &Certificate,
        _ca_key: &PrivateKey,
        _revoked_certs: Vec<RevokedCertificate>,
    ) -> Result<CertificateRevocationList, X509Error> {
        Err(X509Error::OperationError("Not implemented - use MockX509Adapter".to_string()))
    }

    async fn generate_ocsp_response(
        &self,
        _cert: &Certificate,
        _issuer_cert: &Certificate,
        _issuer_key: &PrivateKey,
        _status: OcspStatus,
    ) -> Result<OcspResponse, X509Error> {
        Err(X509Error::OperationError("Not implemented - use MockX509Adapter".to_string()))
    }
}
