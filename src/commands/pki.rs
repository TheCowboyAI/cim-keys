// PKI Commands
//
// Command handlers for PKI hierarchy creation and certificate generation.
//
// User Stories: US-017

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{KeyContext, KeyOwnership, Organization};
use crate::domain_projections::CertificateRequestProjection;
use crate::events::KeyEvent;
use crate::events::{KeyAlgorithm, KeyPurpose};
use crate::value_objects::{
    Certificate, CertificateSubject, PublicKey, Validity,
};

// ============================================================================
// Command: Generate Key Pair (US-012, US-013)
// ============================================================================

/// Command to generate a cryptographic key pair
#[derive(Debug, Clone)]
pub struct GenerateKeyPair {
    pub purpose: crate::value_objects::AuthKeyPurpose,
    pub algorithm: Option<KeyAlgorithm>,
    pub owner_context: KeyContext,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Result of generating key pair
#[derive(Debug, Clone)]
pub struct KeyPairGenerated {
    pub key_id: Uuid,
    pub public_key: PublicKey,
    // Private key stored securely, not returned
    pub algorithm: KeyAlgorithm,
    pub events: Vec<KeyEvent>,
}

/// Handle GenerateKeyPair command
///
/// Generates key pair based on purpose and algorithm.
///
/// Emits:
/// - KeyGeneratedEvent
///
/// User Story: US-012, US-013
pub fn handle_generate_key_pair(cmd: GenerateKeyPair) -> Result<KeyPairGenerated, String> {
    let events = Vec::new();

    // Step 1: Determine algorithm (from command or purpose recommendation)
    let algorithm = cmd.algorithm.unwrap_or_else(|| {
        // Use purpose recommendation
        // TODO: Update KeyAlgorithmRecommendation to match actual KeyAlgorithm variants
        match cmd.purpose.recommended_algorithm() {
            crate::value_objects::key_purposes::KeyAlgorithmRecommendation::Ed25519 => {
                KeyAlgorithm::Ed25519
            }
            // TODO: Add X25519, EcdsaP256, EcdsaP384, Rsa2048, Rsa4096 variants to KeyAlgorithm
            _ => KeyAlgorithm::Ed25519, // Default to Ed25519 for now
        }
    });

    // Step 2: Generate key pair (stub - needs actual crypto)
    let key_id = Uuid::now_v7();
    let public_key = PublicKey {
        algorithm: algorithm.clone(),
        data: vec![0u8; 32], // TODO: Generate real key
        format: crate::value_objects::PublicKeyFormat::Der,
    };

    // Step 3: Emit key generated event
    // TODO: Create and emit KeyGeneratedEvent

    Ok(KeyPairGenerated {
        key_id,
        public_key,
        algorithm,
        events,
    })
}

// ============================================================================
// Command: Generate Root CA (US-017)
// ============================================================================

/// Command to generate root CA certificate
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerateRootCA {
    pub organization: Organization,
    pub validity_years: u32,
    pub algorithm: KeyAlgorithm,
    pub correlation_id: Uuid,
}

/// Result of generating root CA
#[derive(Debug, Clone)]
pub struct RootCAGenerated {
    pub ca_id: Uuid,
    pub certificate: Certificate,
    pub public_key: PublicKey,
    pub events: Vec<KeyEvent>,
}

/// Handle GenerateRootCA command
///
/// Creates self-signed root CA certificate.
///
/// Emits:
/// - KeyGeneratedEvent (for CA key pair)
/// - CertificateGeneratedEvent (for root CA cert)
/// - PkiHierarchyCreatedEvent
///
/// User Story: US-017
pub fn handle_generate_root_ca(cmd: GenerateRootCA) -> Result<RootCAGenerated, String> {
    let mut events = Vec::new();
    let ca_id = Uuid::now_v7();

    // Step 1: Generate CA key pair
    let key_pair = handle_generate_key_pair(GenerateKeyPair {
        purpose: crate::value_objects::AuthKeyPurpose::X509ServerAuth,
        algorithm: Some(cmd.algorithm),
        owner_context: KeyContext {
            actor: KeyOwnership {
                person_id: Uuid::nil(), // TODO: Root CA owned by organization, not person
                organization_id: cmd.organization.id,
                role: crate::domain::KeyOwnerRole::RootAuthority,
                delegations: vec![],
            },
            org_context: None, // TODO: Add OrganizationalPKI context
            nats_identity: None,
            audit_requirements: vec![], // TODO: Add audit requirements
        },
        correlation_id: cmd.correlation_id,
        causation_id: None,
    })?;
    events.extend(key_pair.events);

    // Step 2: Project organization to root CA CSR
    let _csr = CertificateRequestProjection::project_root_ca(&cmd.organization, key_pair.public_key.clone());

    // Step 3: Generate self-signed certificate
    // TODO: Use rcgen to generate certificate from CSR
    let certificate = Certificate {
        serial_number: "1".to_string(),
        subject: CertificateSubject {
            common_name: format!("{} Root CA", cmd.organization.name),
            organization: Some(cmd.organization.name.clone()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        },
        issuer: CertificateSubject {
            common_name: format!("{} Root CA", cmd.organization.name),
            organization: Some(cmd.organization.name.clone()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        },
        public_key: key_pair.public_key.clone(),
        validity: Validity {
            not_before: Utc::now(),
            not_after: Utc::now() + chrono::Duration::days((cmd.validity_years * 365) as i64),
        },
        signature: crate::value_objects::Signature {
            algorithm: crate::value_objects::SignatureAlgorithm::Ed25519, // TODO: Map from cmd.algorithm
            data: vec![0u8; 64], // TODO: Generate real signature
        },
        der: vec![],  // TODO: Generate real DER
        pem: String::new(), // TODO: Generate real PEM
    };

    // Step 4: Emit certificate generated event
    // TODO: Create and emit CertificateGeneratedEvent

    Ok(RootCAGenerated {
        ca_id,
        certificate,
        public_key: key_pair.public_key,
        events,
    })
}

// ============================================================================
// Command: Generate Certificate (US-017)
// ============================================================================

/// Command to generate certificate signed by CA
#[derive(Debug, Clone)]
pub struct GenerateCertificate {
    pub subject: CertificateSubject,
    pub public_key: PublicKey,
    pub purpose: KeyPurpose,
    pub validity_years: u32,
    pub ca_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Result of generating certificate
#[derive(Debug, Clone)]
pub struct CertificateGenerated {
    pub cert_id: Uuid,
    pub certificate: Certificate,
    pub events: Vec<KeyEvent>,
}

/// Handle GenerateCertificate command
///
/// Generates certificate signed by specified CA.
///
/// Emits:
/// - CertificateGeneratedEvent
/// - CertificateSignedEvent
///
/// User Story: US-017
pub fn handle_generate_certificate(cmd: GenerateCertificate) -> Result<CertificateGenerated, String> {
    let events = Vec::new();
    let cert_id = Uuid::now_v7();

    // TODO: Implement certificate generation
    // 1. Load CA certificate and key
    // 2. Create X.509 extensions based on purpose
    // 3. Sign certificate with CA key
    // 4. Encode as DER and PEM

    Ok(CertificateGenerated {
        cert_id,
        certificate: Certificate {
            serial_number: cert_id.to_string(),
            subject: cmd.subject.clone(),
            issuer: cmd.subject, // TODO: Use actual CA subject
            public_key: cmd.public_key,
            validity: Validity {
                not_before: Utc::now(),
                not_after: Utc::now() + chrono::Duration::days((cmd.validity_years * 365) as i64),
            },
            signature: crate::value_objects::Signature {
                algorithm: crate::value_objects::SignatureAlgorithm::Ed25519, // TODO: Use CA algorithm
                data: vec![],
            },
            der: vec![],
            pem: String::new(),
        },
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_pair_uses_purpose_recommendation() {
        let cmd = GenerateKeyPair {
            purpose: crate::value_objects::AuthKeyPurpose::SshAuthentication,
            algorithm: None, // Let purpose recommend
            owner_context: KeyContext {
                actor: KeyOwnership {
                    person_id: Uuid::now_v7(),
                    organization_id: Uuid::now_v7(),
                    role: crate::domain::KeyOwnerRole::SecurityAdmin,
                    delegations: vec![],
                },
                org_context: None,
                nats_identity: None,
                audit_requirements: vec![],
            },
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_generate_key_pair(cmd).unwrap();

        // SSH should recommend Ed25519
        assert_eq!(result.algorithm, KeyAlgorithm::Ed25519);
    }

    #[test]
    fn test_generate_root_ca_creates_self_signed() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        let cmd = GenerateRootCA {
            organization: org.clone(),
            validity_years: 10,
            algorithm: KeyAlgorithm::Ed25519, // Use Ed25519 instead of EcdsaP384
            correlation_id: Uuid::now_v7(),
        };

        let result = handle_generate_root_ca(cmd).unwrap();

        // Root CA is self-signed (subject == issuer)
        assert_eq!(
            result.certificate.subject.common_name,
            result.certificate.issuer.common_name
        );
        assert!(result.certificate.subject.common_name.contains("Root CA"));
    }
}
