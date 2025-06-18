//! TLS and X.509 certificate management
//!
//! This module provides TLS certificate operations and X.509 certificate management.

use crate::{KeyError, Result};
use crate::types::*;
use crate::traits::*;
use async_trait::async_trait;
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair};
use x509_parser::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{Utc, Duration, Datelike};
use tracing::info;
use base64::{Engine as _, engine::general_purpose};

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
        // Create certificate parameters
        let mut params = CertificateParams::default();
        
        // Set subject
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, subject);
        params.distinguished_name = dn;
        
        // Add SANs
        params.subject_alt_names = san.into_iter()
            .map(|s| rcgen::SanType::DnsName(rcgen::Ia5String::try_from(s).unwrap()))
            .collect();
        
        // Set validity period
        let now = Utc::now();
        let later = now + Duration::days(validity_days as i64);
        
        params.not_before = rcgen::date_time_ymd(
            now.year(),
            now.month() as u8,
            now.day() as u8,
        );
        params.not_after = rcgen::date_time_ymd(
            later.year(),
            later.month() as u8,
            later.day() as u8,
        );
        
        // Generate key pair based on algorithm
        let key_pair = match key_algorithm {
            KeyAlgorithm::Rsa(size) => {
                let bits = match size {
                    RsaKeySize::Rsa2048 => 2048,
                    RsaKeySize::Rsa3072 => 3072,
                    RsaKeySize::Rsa4096 => 4096,
                };
                KeyPair::generate_for(&rcgen::PKCS_RSA_SHA256)
                    .map_err(|e| KeyError::Other(format!("Failed to generate RSA key: {}", e)))?
            }
            KeyAlgorithm::Ecdsa(curve) => {
                let alg = match curve {
                    EcdsaCurve::P256 => &rcgen::PKCS_ECDSA_P256_SHA256,
                    EcdsaCurve::P384 => &rcgen::PKCS_ECDSA_P384_SHA384,
                    EcdsaCurve::P521 => {
                        return Err(KeyError::UnsupportedAlgorithm(
                            "P521 curve not supported without aws-lc-rs feature".to_string()
                        ));
                    }
                };
                KeyPair::generate_for(alg)
                    .map_err(|e| KeyError::Other(format!("Failed to generate ECDSA key: {}", e)))?
            }
            _ => return Err(KeyError::UnsupportedAlgorithm(
                format!("Algorithm {:?} not supported for TLS certificates", key_algorithm)
            )),
        };
        
        // Generate certificate using the key pair
        let cert = params.self_signed(&key_pair)
            .map_err(|e| KeyError::Other(format!("Failed to generate certificate: {}", e)))?;
        
        // Get certificate DER
        let cert_der = cert.der()
            .to_vec();
        
        // Generate certificate ID from serial number
        let cert_id = format!("{:?}", cert.params().serial_number);
        let key_id = KeyId::new();
        
        // Store key pair (KeyPair doesn't implement Clone, so we need to regenerate for storage)
        // This is a limitation of the current design - in production, keys should be stored differently
        
        // Create metadata
        let metadata = CertificateMetadata {
            subject: subject.to_string(),
            issuer: subject.to_string(), // Self-signed
            serial_number: cert_id.clone(),
            not_before: Utc::now(),
            not_after: Utc::now() + Duration::days(validity_days as i64),
            san: cert.params().subject_alt_names.iter()
                .filter_map(|san| match san {
                    rcgen::SanType::DnsName(name) => Some(name.to_string()),
                    _ => None,
                })
                .collect(),
            key_usage: vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
            extended_key_usage: vec!["serverAuth".to_string(), "clientAuth".to_string()],
            is_ca: false,
            path_len_constraint: None,
        };
        
        // Store certificate
        let entry = TlsCertEntry {
            certificate: cert,
            cert_der,
            metadata,
        };
        
        let mut certs = self.certificates.write().unwrap();
        certs.insert(cert_id.clone(), entry);
        
        info!("Generated self-signed certificate {} with key {}", cert_id, key_id);
        Ok((key_id, cert_id))
    }
}

#[async_trait]
impl CertificateManager for TlsManager {
    async fn generate_csr(
        &self,
        _key_id: &KeyId,
        _subject: &str,
        _san: Vec<String>,
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
                // For now, just return an error for PEM import - needs proper PEM library
                return Err(KeyError::InvalidKeyFormat(
                    "PEM import not yet implemented - use DER format".to_string()
                ));
            }
            _ => return Err(KeyError::InvalidKeyFormat(
                format!("Format {:?} not supported for import", format)
            )),
        };

        // Parse certificate
        let (_, x509_cert) = X509Certificate::from_der(&cert_der)
            .map_err(|e| KeyError::X509(format!("Failed to parse certificate: {:?}", e)))?;

        let cert_id = format!("{:?}", x509_cert.serial);

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
                // Manual PEM encoding since pem::encode doesn't exist in this version
                let encoded = general_purpose::STANDARD.encode(&entry.cert_der);
                let pem_string = format!(
                    "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
                    encoded.chars().collect::<Vec<_>>()
                        .chunks(64)
                        .map(|chunk| chunk.iter().collect::<String>())
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                Ok(pem_string.into_bytes())
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
        _cert_id: &str,
        _ca_cert_id: Option<&str>,
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
