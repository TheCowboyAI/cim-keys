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
use chrono::{Utc, Duration};
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
        let mut params = CertificateParams::new(san.clone())
            .map_err(|e| KeyError::CertGen(e))?;
        
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, subject);
        params.distinguished_name = dn;

        // Use SystemTime instead of chrono for compatibility with rcgen
        let now = std::time::SystemTime::now();
        let validity_duration = std::time::Duration::from_secs(validity_days as u64 * 24 * 60 * 60);
        
        params.not_before = now.into();
        params.not_after = (now + validity_duration).into();

        // Generate the certificate directly (rcgen handles key generation internally)
        let cert = Certificate::generate(params)
            .map_err(|e| KeyError::CertGen(e))?;

        let cert_der = cert.serialize_der()
            .map_err(|e| KeyError::CertGen(e))?;

        // Parse to get metadata
        let (_, x509_cert) = X509Certificate::from_der(&cert_der)
            .map_err(|e| KeyError::X509(format!("Failed to parse certificate: {:?}", e)))?;

        let cert_id = format!("{:X}", x509_cert.serial);
        let key_id = KeyId::new();

        // Create metadata using chrono for our internal representation
        let chrono_now = Utc::now();
        let metadata = CertificateMetadata {
            subject: subject.to_string(),
            issuer: subject.to_string(), // Self-signed
            serial_number: cert_id.clone(),
            not_before: chrono_now,
            not_after: chrono_now + Duration::days(validity_days as i64),
            san,
            key_usage: vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
            extended_key_usage: vec!["serverAuth".to_string(), "clientAuth".to_string()],
            is_ca: false,
            path_len_constraint: None,
        };

        // Store certificate 
        let entry = TlsCertEntry {
            certificate: cert,
            cert_der,
            metadata: metadata.clone(),
        };

        let mut certs = self.certificates.write().unwrap();
        certs.insert(cert_id.clone(), entry);

        // We don't store the private key separately since rcgen manages it internally
        info!("Generated self-signed certificate with ID {}", cert_id);
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
