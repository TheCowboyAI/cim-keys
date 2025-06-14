//! TLS and X.509 certificate management
//!
//! This module provides TLS certificate operations and X.509 certificate management.

use crate::{KeyError, Result};
use crate::types::*;
use crate::traits::*;
use async_trait::async_trait;
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair};
use x509_parser::prelude::*;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{Utc, Duration};
use tracing::{debug, info};

/// TLS certificate manager
pub struct TlsManager {
    /// Certificate storage
    certificates: Arc<RwLock<HashMap<String, TlsCertEntry>>>,
    /// Key storage
    keys: Arc<RwLock<HashMap<KeyId, KeyPair>>>,
}

/// Internal certificate storage entry
struct TlsCertEntry {
    certificate: Certificate,
    cert_der: Vec<u8>,
    metadata: CertificateMetadata,
}

impl TlsManager {
    /// Create a new TLS manager
    pub fn new() -> Self {
        Self {
            certificates: Arc::new(RwLock::new(HashMap::new())),
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a self-signed certificate
    pub async fn generate_self_signed(
        &self,
        subject: &str,
        san: Vec<String>,
        key_algorithm: KeyAlgorithm,
        validity_days: u32,
    ) -> Result<(KeyId, String)> {
        let mut params = CertificateParams::new(san.clone());
        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(DnType::CommonName, subject);

        // Set validity period
        params.not_before = Utc::now();
        params.not_after = Utc::now() + Duration::days(validity_days as i64);

        // Generate key pair based on algorithm
        let key_pair = match key_algorithm {
            KeyAlgorithm::Rsa(size) => {
                let bits = match size {
                    RsaKeySize::Rsa2048 => 2048,
                    RsaKeySize::Rsa3072 => 3072,
                    RsaKeySize::Rsa4096 => 4096,
                };
                KeyPair::generate(&rcgen::PKCS_RSA_SHA256)
                    .map_err(|e| KeyError::CertGen(e))?
            }
            KeyAlgorithm::Ecdsa(curve) => {
                match curve {
                    EcdsaCurve::P256 => KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256),
                    EcdsaCurve::P384 => KeyPair::generate(&rcgen::PKCS_ECDSA_P384_SHA384),
                    _ => return Err(KeyError::UnsupportedAlgorithm(
                        "P521 not supported by rcgen".to_string()
                    )),
                }.map_err(|e| KeyError::CertGen(e))?
            }
            _ => return Err(KeyError::UnsupportedAlgorithm(
                format!("Algorithm {:?} not supported for TLS certificates", key_algorithm)
            )),
        };

        params.key_pair = Some(key_pair.clone());

        // Generate certificate
        let cert = Certificate::from_params(params)
            .map_err(|e| KeyError::CertGen(e))?;

        let cert_der = cert.serialize_der()
            .map_err(|e| KeyError::CertGen(e))?;

        // Parse to get metadata
        let (_, x509_cert) = X509Certificate::from_der(&cert_der)
            .map_err(|e| KeyError::X509(format!("Failed to parse certificate: {:?}", e)))?;

        let cert_id = format!("{:X}", x509_cert.serial);
        let key_id = KeyId::new();

        // Create metadata
        let metadata = CertificateMetadata {
            subject: subject.to_string(),
            issuer: subject.to_string(), // Self-signed
            serial_number: cert_id.clone(),
            not_before: params.not_before,
            not_after: params.not_after,
            san,
            key_usage: vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
            extended_key_usage: vec!["serverAuth".to_string(), "clientAuth".to_string()],
            is_ca: false,
            path_len_constraint: None,
        };

        // Store certificate and key
        let entry = TlsCertEntry {
            certificate: cert,
            cert_der,
            metadata: metadata.clone(),
        };

        let mut certs = self.certificates.write().unwrap();
        certs.insert(cert_id.clone(), entry);

        let mut keys = self.keys.write().unwrap();
        keys.insert(key_id, key_pair);

        info!("Generated self-signed certificate with ID {}", cert_id);
        Ok((key_id, cert_id))
    }
}

#[async_trait]
impl CertificateManager for TlsManager {
    async fn generate_csr(
        &self,
        key_id: &KeyId,
        subject: &str,
        san: Vec<String>,
    ) -> Result<Vec<u8>> {
        // TODO: Implement CSR generation
        Err(KeyError::Other("CSR generation not yet implemented".to_string()))
    }

    async fn import_certificate(
        &self,
        cert_data: &[u8],
        format: CertificateFormat,
    ) -> Result<String> {
        let cert_der = match format {
            CertificateFormat::Der => cert_data.to_vec(),
            CertificateFormat::Pem => {
                let pem_str = std::str::from_utf8(cert_data)
                    .map_err(|_| KeyError::InvalidKeyFormat("Invalid UTF-8 in PEM".to_string()))?;

                let pem = pem::parse(pem_str)
                    .map_err(|e| KeyError::Pem(e))?;

                if pem.tag() != "CERTIFICATE" {
                    return Err(KeyError::InvalidKeyFormat(
                        format!("Expected CERTIFICATE, got {}", pem.tag())
                    ));
                }

                pem.into_contents()
            }
            _ => return Err(KeyError::InvalidKeyFormat(
                format!("Format {:?} not supported for import", format)
            )),
        };

        // Parse certificate
        let (_, x509_cert) = X509Certificate::from_der(&cert_der)
            .map_err(|e| KeyError::X509(format!("Failed to parse certificate: {:?}", e)))?;

        let cert_id = format!("{:X}", x509_cert.serial);

        // Extract metadata
        let subject = x509_cert.subject.to_string();
        let issuer = x509_cert.issuer.to_string();

        let metadata = CertificateMetadata {
            subject,
            issuer,
            serial_number: cert_id.clone(),
            not_before: Utc::now(), // TODO: Extract from cert
            not_after: Utc::now(), // TODO: Extract from cert
            san: vec![], // TODO: Extract SANs
            key_usage: vec![], // TODO: Extract key usage
            extended_key_usage: vec![], // TODO: Extract EKU
            is_ca: x509_cert.is_ca(),
            path_len_constraint: None, // TODO: Extract
        };

        info!("Imported certificate {}", cert_id);
        Ok(cert_id)
    }

    async fn export_certificate(
        &self,
        cert_id: &str,
        format: CertificateFormat,
        include_chain: bool,
    ) -> Result<Vec<u8>> {
        let certs = self.certificates.read().unwrap();
        let entry = certs.get(cert_id)
            .ok_or_else(|| KeyError::KeyNotFound(format!("Certificate {} not found", cert_id)))?;

        match format {
            CertificateFormat::Der => Ok(entry.cert_der.clone()),
            CertificateFormat::Pem => {
                let pem = pem::Pem::new("CERTIFICATE", entry.cert_der.clone());
                Ok(pem::encode(&pem).into_bytes())
            }
            _ => Err(KeyError::InvalidKeyFormat(
                format!("Format {:?} not supported for export", format)
            )),
        }
    }

    async fn get_certificate_metadata(
        &self,
        cert_id: &str,
    ) -> Result<CertificateMetadata> {
        let certs = self.certificates.read().unwrap();
        certs.get(cert_id)
            .map(|entry| entry.metadata.clone())
            .ok_or_else(|| KeyError::KeyNotFound(format!("Certificate {} not found", cert_id)))
    }

    async fn validate_certificate(
        &self,
        cert_id: &str,
        ca_cert_id: Option<&str>,
    ) -> Result<bool> {
        // TODO: Implement certificate validation
        Ok(true)
    }
}

impl Default for TlsManager {
    fn default() -> Self {
        Self::new()
    }
}
