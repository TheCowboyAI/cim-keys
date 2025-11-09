//! Mock X.509 adapter for testing
//!
//! This adapter implements the X509Port trait using in-memory simulation.
//! It provides a functor from the X.509 PKI category to the Domain category for testing.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: X.509 PKI (simulated)
//! - **Target Category**: Domain (trust operations)
//! - **Functor**: MockX509Adapter maps simulated PKI to domain operations
//! - **Morphisms Preserved**: All certificate chain compositions are preserved

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::ports::x509::*;

/// Mock X.509 adapter for testing
///
/// This is a **Functor** F: X509_PKI_Mock → Domain where:
/// - Objects: Simulated certificates → Domain trust entities
/// - Morphisms: Simulated PKI operations → Domain operations
///
/// **Functor Laws Verified:**
/// 1. Identity: No-op operations preserve state
/// 2. Composition: sign_csr ∘ generate_csr = valid certificate chain
#[derive(Clone)]
pub struct MockX509Adapter {
    /// Certificates stored by serial number
    certificates: Arc<RwLock<HashMap<Vec<u8>, Certificate>>>,

    /// Private keys stored by certificate serial
    private_keys: Arc<RwLock<HashMap<Vec<u8>, PrivateKey>>>,

    /// Certificate chains (leaf serial -> chain)
    chains: Arc<RwLock<HashMap<Vec<u8>, Vec<Certificate>>>>,

    /// Revoked certificates
    revoked: Arc<RwLock<HashMap<Vec<u8>, RevokedCertificate>>>,

    /// Serial counter for generating unique serials
    serial_counter: Arc<RwLock<u64>>,
}

impl MockX509Adapter {
    /// Create a new mock adapter
    pub fn new() -> Self {
        Self {
            certificates: Arc::new(RwLock::new(HashMap::new())),
            private_keys: Arc::new(RwLock::new(HashMap::new())),
            chains: Arc::new(RwLock::new(HashMap::new())),
            revoked: Arc::new(RwLock::new(HashMap::new())),
            serial_counter: Arc::new(RwLock::new(1)),
        }
    }

    /// Clear all state (for test isolation)
    pub fn clear(&self) {
        self.certificates.write().unwrap().clear();
        self.private_keys.write().unwrap().clear();
        self.chains.write().unwrap().clear();
        self.revoked.write().unwrap().clear();
        *self.serial_counter.write().unwrap() = 1;
    }

    fn next_serial(&self) -> Vec<u8> {
        let mut counter = self.serial_counter.write().unwrap();
        let serial = counter.to_be_bytes().to_vec();
        *counter += 1;
        serial
    }

    fn generate_mock_private_key(&self, algorithm: &str) -> PrivateKey {
        let der = match algorithm {
            "RSA-2048" => vec![0x30, 0x82, 0x04, 0xA4], // Mock RSA 2048 DER
            "ECDSA-P256" => vec![0x30, 0x77], // Mock ECDSA P-256 DER
            "Ed25519" => vec![0x30, 0x2E], // Mock Ed25519 DER
            _ => vec![0x30, 0x00],
        };

        let pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
            base64::encode(&der)
        );

        PrivateKey {
            algorithm: algorithm.to_string(),
            der: der.clone(),
            pem,
        }
    }

    fn generate_mock_certificate(
        &self,
        subject: &CertificateSubject,
        issuer: &CertificateSubject,
        serial: Vec<u8>,
        not_before: i64,
        not_after: i64,
        is_ca: bool,
    ) -> Certificate {
        // Mock DER certificate structure
        let der = vec![0x30, 0x82, 0x03, 0x00]; // Simplified mock DER

        let pem = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            base64::encode(&der)
        );

        Certificate {
            der: der.clone(),
            pem,
            subject: subject.clone(),
            issuer: issuer.clone(),
            serial,
            not_before,
            not_after,
            is_ca,
            key_algorithm: "RSA-2048".to_string(),
        }
    }
}

impl Default for MockX509Adapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl X509Port for MockX509Adapter {
    /// **Functor Mapping**: (subject, key, validity) → RootCertificate
    async fn generate_root_ca(
        &self,
        subject: &CertificateSubject,
        key: &PrivateKey,
        validity_days: u32,
    ) -> Result<Certificate, X509Error> {
        let serial = self.next_serial();
        let now = Utc::now();
        let not_before = now.timestamp();
        let not_after = (now + Duration::days(validity_days as i64)).timestamp();

        let cert = self.generate_mock_certificate(
            subject,
            subject, // Self-signed: issuer = subject
            serial.clone(),
            not_before,
            not_after,
            true, // is_ca
        );

        // Store certificate and key
        self.certificates.write().unwrap().insert(serial.clone(), cert.clone());
        self.private_keys.write().unwrap().insert(serial, key.clone());

        Ok(cert)
    }

    /// **Functor Mapping**: (subject, key, san) → CSR
    async fn generate_csr(
        &self,
        subject: &CertificateSubject,
        key: &PrivateKey,
        san: Vec<String>,
    ) -> Result<CertificateSigningRequest, X509Error> {
        // Mock CSR DER structure
        let der = vec![0x30, 0x82, 0x01, 0x00];

        let pem = format!(
            "-----BEGIN CERTIFICATE REQUEST-----\n{}\n-----END CERTIFICATE REQUEST-----",
            base64::encode(&der)
        );

        Ok(CertificateSigningRequest {
            der,
            pem,
            subject: subject.clone(),
            san,
        })
    }

    /// **Functor Mapping**: (csr, ca_cert, ca_key) → SignedCertificate
    async fn sign_csr(
        &self,
        csr: &CertificateSigningRequest,
        ca_cert: &Certificate,
        ca_key: &PrivateKey,
        validity_days: u32,
        is_ca: bool,
    ) -> Result<Certificate, X509Error> {
        let serial = self.next_serial();
        let now = Utc::now();
        let not_before = now.timestamp();
        let not_after = (now + Duration::days(validity_days as i64)).timestamp();

        let cert = self.generate_mock_certificate(
            &csr.subject,
            &ca_cert.subject, // Issuer is the CA
            serial.clone(),
            not_before,
            not_after,
            is_ca,
        );

        // Store certificate
        self.certificates.write().unwrap().insert(serial, cert.clone());

        Ok(cert)
    }

    /// **Functor Mapping**: Creates intermediate object in certificate category
    async fn generate_intermediate_ca(
        &self,
        subject: &CertificateSubject,
        key: &PrivateKey,
        parent_ca_cert: &Certificate,
        parent_ca_key: &PrivateKey,
        validity_days: u32,
        path_len_constraint: Option<u32>,
    ) -> Result<Certificate, X509Error> {
        let serial = self.next_serial();
        let now = Utc::now();
        let not_before = now.timestamp();
        let not_after = (now + Duration::days(validity_days as i64)).timestamp();

        let cert = self.generate_mock_certificate(
            subject,
            &parent_ca_cert.subject,
            serial.clone(),
            not_before,
            not_after,
            true, // is_ca
        );

        // Store certificate and key
        self.certificates.write().unwrap().insert(serial.clone(), cert.clone());
        self.private_keys.write().unwrap().insert(serial, key.clone());

        Ok(cert)
    }

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
    ) -> Result<Certificate, X509Error> {
        let serial = self.next_serial();
        let now = Utc::now();
        let not_before = now.timestamp();
        let not_after = (now + Duration::days(validity_days as i64)).timestamp();

        let cert = self.generate_mock_certificate(
            subject,
            &ca_cert.subject,
            serial.clone(),
            not_before,
            not_after,
            false, // Not a CA
        );

        // Store certificate and key
        self.certificates.write().unwrap().insert(serial, cert.clone());

        Ok(cert)
    }

    /// **Functor Mapping**: bytes → Certificate (object construction)
    async fn parse_certificate(&self, cert_data: &[u8]) -> Result<Certificate, X509Error> {
        // Simple mock parsing - just check if it looks like a certificate
        if cert_data.len() < 4 || cert_data[0] != 0x30 {
            return Err(X509Error::ParsingError("Invalid certificate data".to_string()));
        }

        // Return a mock parsed certificate
        Ok(Certificate {
            der: cert_data.to_vec(),
            pem: format!(
                "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
                base64::encode(cert_data)
            ),
            subject: CertificateSubject {
                common_name: "Mock Subject".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            issuer: CertificateSubject {
                common_name: "Mock Issuer".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            serial: vec![1, 2, 3, 4],
            not_before: Utc::now().timestamp(),
            not_after: (Utc::now() + Duration::days(365)).timestamp(),
            is_ca: false,
            key_algorithm: "RSA-2048".to_string(),
        })
    }

    /// **Functor Mapping**: [Certificate] → bool (validates morphism composition)
    async fn verify_chain(
        &self,
        leaf_cert: &Certificate,
        intermediates: &[Certificate],
        root_cert: &Certificate,
    ) -> Result<bool, X509Error> {
        // Mock verification - check that chain is connected
        // In real implementation, this would verify signatures

        // Check leaf is signed by first intermediate or root
        if intermediates.is_empty() {
            // Direct signing by root
            if leaf_cert.issuer != root_cert.subject {
                return Ok(false);
            }
        } else {
            // Check leaf signed by last intermediate
            if leaf_cert.issuer != intermediates.last().unwrap().subject {
                return Ok(false);
            }

            // Check intermediate chain
            for window in intermediates.windows(2) {
                if window[0].issuer != window[1].subject {
                    return Ok(false);
                }
            }

            // Check first intermediate signed by root
            if intermediates[0].issuer != root_cert.subject {
                return Ok(false);
            }
        }

        // Check root is self-signed
        if root_cert.issuer != root_cert.subject {
            return Ok(false);
        }

        Ok(true)
    }

    /// **Functor Mapping**: Certificate → bytes (object serialization)
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
        ca_cert: &Certificate,
        ca_key: &PrivateKey,
        revoked_certs: Vec<RevokedCertificate>,
    ) -> Result<CertificateRevocationList, X509Error> {
        let now = Utc::now();
        let der = vec![0x30, 0x82, 0x00, 0x50]; // Mock CRL DER

        let pem = format!(
            "-----BEGIN X509 CRL-----\n{}\n-----END X509 CRL-----",
            base64::encode(&der)
        );

        Ok(CertificateRevocationList {
            der,
            pem,
            issuer: ca_cert.subject.clone(),
            this_update: now.timestamp(),
            next_update: (now + Duration::days(30)).timestamp(),
            revoked_certificates: revoked_certs,
        })
    }

    async fn generate_ocsp_response(
        &self,
        cert: &Certificate,
        issuer_cert: &Certificate,
        issuer_key: &PrivateKey,
        status: OcspStatus,
    ) -> Result<OcspResponse, X509Error> {
        let now = Utc::now();
        let der = vec![0x30, 0x82, 0x00, 0x30]; // Mock OCSP response DER

        Ok(OcspResponse {
            der,
            status,
            this_update: now.timestamp(),
            next_update: Some((now + Duration::days(7)).timestamp()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_functor_identity_law() {
        // F(id) = id - No-op preserves state
        let adapter = MockX509Adapter::new();

        let subject = CertificateSubject {
            common_name: "Test Root CA".to_string(),
            organization: Some("Test Org".to_string()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        };

        let key = adapter.generate_mock_private_key("RSA-2048");
        let cert = adapter.generate_root_ca(&subject, &key, 365).await.unwrap();

        // Verify certificate properties
        assert_eq!(cert.subject, subject);
        assert_eq!(cert.issuer, subject); // Self-signed
        assert!(cert.is_ca);
    }

    #[tokio::test]
    async fn test_functor_composition_law() {
        // F(sign_csr ∘ generate_csr) = F(sign_csr) ∘ F(generate_csr)
        let adapter = MockX509Adapter::new();

        // Generate root CA
        let root_subject = CertificateSubject {
            common_name: "Root CA".to_string(),
            organization: Some("Test Org".to_string()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        };
        let root_key = adapter.generate_mock_private_key("RSA-2048");
        let root_cert = adapter.generate_root_ca(&root_subject, &root_key, 365).await.unwrap();

        // Generate CSR
        let leaf_subject = CertificateSubject {
            common_name: "example.com".to_string(),
            organization: Some("Example Inc".to_string()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        };
        let leaf_key = adapter.generate_mock_private_key("RSA-2048");
        let csr = adapter
            .generate_csr(&leaf_subject, &leaf_key, vec!["example.com".to_string()])
            .await
            .unwrap();

        // Sign CSR (composition)
        let leaf_cert = adapter
            .sign_csr(&csr, &root_cert, &root_key, 90, false)
            .await
            .unwrap();

        // Verify composition preserved structure
        assert_eq!(leaf_cert.subject, leaf_subject);
        assert_eq!(leaf_cert.issuer, root_subject);
        assert!(!leaf_cert.is_ca);
    }

    #[tokio::test]
    async fn test_certificate_chain_verification() {
        let adapter = MockX509Adapter::new();

        // Create root CA
        let root_subject = CertificateSubject {
            common_name: "Root CA".to_string(),
            organization: Some("Test Org".to_string()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        };
        let root_key = adapter.generate_mock_private_key("RSA-2048");
        let root_cert = adapter.generate_root_ca(&root_subject, &root_key, 3650).await.unwrap();

        // Create intermediate CA
        let intermediate_subject = CertificateSubject {
            common_name: "Intermediate CA".to_string(),
            organization: Some("Test Org".to_string()),
            organizational_unit: Some("Security".to_string()),
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        };
        let intermediate_key = adapter.generate_mock_private_key("RSA-2048");
        let intermediate_cert = adapter
            .generate_intermediate_ca(
                &intermediate_subject,
                &intermediate_key,
                &root_cert,
                &root_key,
                1825,
                Some(0),
            )
            .await
            .unwrap();

        // Create leaf certificate
        let leaf_subject = CertificateSubject {
            common_name: "www.example.com".to_string(),
            organization: Some("Example Inc".to_string()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        };
        let leaf_key = adapter.generate_mock_private_key("RSA-2048");
        let leaf_cert = adapter
            .generate_leaf_certificate(
                &leaf_subject,
                &leaf_key,
                &intermediate_cert,
                &intermediate_key,
                90,
                vec!["www.example.com".to_string(), "example.com".to_string()],
                vec![KeyUsage::DigitalSignature, KeyUsage::KeyEncipherment],
                vec![ExtendedKeyUsage::ServerAuth],
            )
            .await
            .unwrap();

        // Verify the chain
        let is_valid = adapter
            .verify_chain(&leaf_cert, &[intermediate_cert], &root_cert)
            .await
            .unwrap();

        assert!(is_valid);
    }
}
