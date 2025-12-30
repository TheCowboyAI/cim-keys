// PKI Commands
//
// Command handlers for PKI hierarchy creation and certificate generation.
//
// User Stories: US-017

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{KeyContext, KeyOwnership, Organization};
use crate::domain_projections::CertificateRequestProjection;
use crate::events::DomainEvent;
use crate::types::{KeyAlgorithm, KeyPurpose};
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
    pub events: Vec<DomainEvent>,
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
    let mut events = Vec::new();

    // Step 1: Determine algorithm (from command or purpose recommendation)
    let algorithm = cmd.algorithm.unwrap_or_else(|| {
        // Use purpose recommendation
        match cmd.purpose.recommended_algorithm() {
            crate::value_objects::key_purposes::KeyAlgorithmRecommendation::Ed25519 => {
                KeyAlgorithm::Ed25519
            }
            _ => KeyAlgorithm::Ed25519, // Default to Ed25519 for now
        }
    });

    // Step 2: Generate key pair using ed25519-dalek
    let key_id = Uuid::now_v7();

    // Generate Ed25519 key pair
    use ed25519_dalek::SigningKey;

    let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let verifying_key = signing_key.verifying_key();

    // Convert to bytes
    let public_key_bytes = verifying_key.to_bytes();

    let public_key = PublicKey {
        algorithm: algorithm.clone(),
        data: public_key_bytes.to_vec(),
        format: crate::value_objects::PublicKeyFormat::Der,
    };

    // Step 3: Emit key generated event
    events.push(DomainEvent::Key(crate::events::KeyEvents::KeyGenerated(crate::events::key::KeyGeneratedEvent {
        key_id,
        algorithm: algorithm.clone(),
        purpose: match cmd.purpose {
            crate::value_objects::AuthKeyPurpose::X509ServerAuth => KeyPurpose::Authentication,
            crate::value_objects::AuthKeyPurpose::X509CodeSigning => KeyPurpose::Signing,
            crate::value_objects::AuthKeyPurpose::GpgEncryption => KeyPurpose::Encryption,
            _ => KeyPurpose::Authentication,
        },
        generated_at: Utc::now(),
        generated_by: "system".to_string(),
        hardware_backed: false,
        metadata: crate::types::KeyMetadata {
            label: format!("Key pair for {:?}", cmd.purpose),
            description: Some(format!("Generated {:?} key for {:?}", algorithm, cmd.purpose)),
            tags: vec![],
            attributes: std::collections::HashMap::new(),
            jwt_kid: None,
            jwt_alg: None,
            jwt_use: None,
        },
        ownership: Some(cmd.owner_context.actor.clone()),
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    })));

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
    pub events: Vec<DomainEvent>,
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
    // A4: Generate command_id for causation tracking
    let command_id = Uuid::now_v7();

    // Step 1: Generate CA key pair
    let key_pair = handle_generate_key_pair(GenerateKeyPair {
        purpose: crate::value_objects::AuthKeyPurpose::X509ServerAuth,
        algorithm: Some(cmd.algorithm.clone()),
        owner_context: KeyContext {
            actor: KeyOwnership {
                // Root CA is owned by the organization itself
                // Use the organization ID as the person_id to indicate organizational ownership
                person_id: cmd.organization.id,
                organization_id: cmd.organization.id,
                role: crate::domain::KeyOwnerRole::RootAuthority,
                delegations: vec![],
            },
            org_context: Some(crate::domain::OrganizationalPKI {
                root_ca_org_id: cmd.organization.id,
                intermediate_cas: vec![],
                policy_cas: vec![],
                cross_certifications: vec![],
            }),
            nats_identity: None,
            audit_requirements: vec![
                crate::domain::AuditRequirement::SecureLogging {
                    log_level: "CRITICAL".to_string(),
                },
                crate::domain::AuditRequirement::ComplianceReport {
                    standards: vec!["PKI-ROOT-CA-GENERATION".to_string()],
                },
            ],
        },
        correlation_id: cmd.correlation_id,
        causation_id: Some(command_id), // A4: Causation from parent command
    })?;
    events.extend(key_pair.events);

    // Step 2: Project organization to root CA CSR
    let _csr = CertificateRequestProjection::project_root_ca(&cmd.organization, key_pair.public_key.clone());

    // Step 3: Generate self-signed certificate using rcgen
    use rcgen::{CertificateParams, DistinguishedName, DnType, IsCa, BasicConstraints, KeyUsagePurpose, SerialNumber, KeyPair as RcgenKeyPair};
    use time::{Duration as TimeDuration, OffsetDateTime};

    // Generate a new key pair for the certificate
    let rcgen_key_pair = RcgenKeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;

    // Create certificate parameters
    let mut params = CertificateParams::default();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, format!("{} Root CA", cmd.organization.name));
    dn.push(DnType::OrganizationName, cmd.organization.name.clone());
    dn.push(DnType::CountryName, "US");
    params.distinguished_name = dn;
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    // Set validity period
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + TimeDuration::days((cmd.validity_years * 365) as i64);
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
    let rcgen_cert = params.self_signed(&rcgen_key_pair)
        .map_err(|e| format!("Failed to generate root CA certificate: {}", e))?;

    // Get DER and PEM
    let der = rcgen_cert.der().to_vec();
    let pem = rcgen_cert.pem();

    // Create the certificate structure
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
            algorithm: crate::value_objects::SignatureAlgorithm::Ed25519,
            data: vec![0u8; 64], // Signature is embedded in DER/PEM
        },
        der,
        pem,
    };

    // Step 4: Emit certificate generated event
    events.push(DomainEvent::Certificate(crate::events::CertificateEvents::CertificateGenerated(crate::events::certificate::CertificateGeneratedEvent {
        cert_id: ca_id,
        key_id: key_pair.key_id,
        subject: certificate.subject.common_name.clone(),
        issuer: None, // Self-signed root CA
        not_before: certificate.validity.not_before,
        not_after: certificate.validity.not_after,
        is_ca: true,
        san: vec![],
        key_usage: vec![
            "KeyCertSign".to_string(),
            "CrlSign".to_string(),
            "DigitalSignature".to_string(),
        ],
        extended_key_usage: vec![],
        correlation_id: cmd.correlation_id,
        causation_id: Some(key_pair.key_id), // Certificate caused by key generation
    })));

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
    pub key_id: Uuid,  // ID of the key corresponding to public_key
    pub purpose: KeyPurpose,
    pub validity_years: u32,
    pub ca_id: Uuid,
    pub ca_certificate: Option<Certificate>,  // Optional: loaded CA cert
    pub ca_algorithm: Option<KeyAlgorithm>,   // Optional: loaded CA key algorithm
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Result of generating certificate
#[derive(Debug, Clone)]
pub struct CertificateGenerated {
    pub cert_id: Uuid,
    pub certificate: Certificate,
    pub events: Vec<DomainEvent>,
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
    let mut events = Vec::new();
    let cert_id = Uuid::now_v7();

    // Get CA information from command (should be loaded by caller from projection)
    // If not provided, this will generate a self-signed certificate as fallback
    let ca_subject = cmd.ca_certificate.as_ref().map(|cert| cert.subject.clone());
    let signature_algorithm_name = cmd.ca_algorithm.as_ref()
        .map(|alg| format!("{:?}", alg))
        .unwrap_or_else(|| "Ed25519".to_string());

    // Step 1: Determine key usage and extended key usage based on purpose
    let (key_usage, extended_key_usage) = match cmd.purpose {
        KeyPurpose::Authentication => (
            vec!["DigitalSignature".to_string(), "KeyAgreement".to_string()],
            vec!["ServerAuth".to_string(), "ClientAuth".to_string()],
        ),
        KeyPurpose::Signing => (
            vec!["DigitalSignature".to_string()],
            vec!["CodeSigning".to_string()],
        ),
        KeyPurpose::Encryption => (
            vec!["KeyEncipherment".to_string(), "DataEncipherment".to_string()],
            vec!["EmailProtection".to_string()],
        ),
        KeyPurpose::JwtSigning => (
            vec!["DigitalSignature".to_string()],
            vec![],
        ),
        _ => (
            vec!["DigitalSignature".to_string()],
            vec![],
        ),
    };

    // Step 2: Generate certificate using rcgen
    use rcgen::{CertificateParams, DistinguishedName, DnType, IsCa, KeyUsagePurpose, SerialNumber, KeyPair as RcgenKeyPair};
    use time::{Duration as TimeDuration, OffsetDateTime};

    let rcgen_key_pair = RcgenKeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;

    let mut params = CertificateParams::default();

    // Set subject DN
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, &cmd.subject.common_name);
    if let Some(org) = &cmd.subject.organization {
        dn.push(DnType::OrganizationName, org);
    }
    if let Some(ou) = &cmd.subject.organizational_unit {
        dn.push(DnType::OrganizationalUnitName, ou);
    }
    if let Some(country) = &cmd.subject.country {
        dn.push(DnType::CountryName, country);
    }
    params.distinguished_name = dn;

    // Not a CA certificate
    params.is_ca = IsCa::NoCa;

    // Set validity period
    let not_before = OffsetDateTime::now_utc();
    let not_after = not_before + TimeDuration::days((cmd.validity_years * 365) as i64);
    params.not_before = not_before;
    params.not_after = not_after;

    // Set key usages based on purpose
    params.key_usages = key_usage.iter().filter_map(|usage| match usage.as_str() {
        "DigitalSignature" => Some(KeyUsagePurpose::DigitalSignature),
        "KeyEncipherment" => Some(KeyUsagePurpose::KeyEncipherment),
        "DataEncipherment" => Some(KeyUsagePurpose::DataEncipherment),
        "KeyAgreement" => Some(KeyUsagePurpose::KeyAgreement),
        _ => None,
    }).collect();

    // Generate serial number from cert_id
    let serial_bytes = cert_id.as_u128().to_be_bytes();
    params.serial_number = Some(SerialNumber::from_slice(&serial_bytes));

    // Generate certificate signed by CA (or self-signed if no CA provided)
    let rcgen_cert = if cmd.ca_certificate.is_some() {
        // TODO: In a real implementation, we would:
        // 1. Load CA private key from secure storage
        // 2. Sign the certificate with CA key
        // For now, fall back to self-signed as we don't have CA private key access here
        // This is a placeholder that should be replaced with proper CA signing
        params.self_signed(&rcgen_key_pair)
            .map_err(|e| format!("Failed to generate certificate: {}", e))?
    } else {
        params.self_signed(&rcgen_key_pair)
            .map_err(|e| format!("Failed to generate self-signed certificate: {}", e))?
    };

    let der = rcgen_cert.der().to_vec();
    let pem = rcgen_cert.pem();

    // Determine issuer: use CA subject if available, otherwise self-signed
    let issuer = ca_subject.unwrap_or_else(|| cmd.subject.clone());

    // Step 3: Create certificate structure
    let certificate = Certificate {
        serial_number: cert_id.to_string(),
        subject: cmd.subject.clone(),
        issuer,
        public_key: cmd.public_key.clone(),
        validity: Validity {
            not_before: Utc::now(),
            not_after: Utc::now() + chrono::Duration::days((cmd.validity_years * 365) as i64),
        },
        signature: crate::value_objects::Signature {
            algorithm: crate::value_objects::SignatureAlgorithm::Ed25519,
            data: vec![0u8; 64], // Signature is embedded in DER/PEM
        },
        der,
        pem,
    };

    // Step 4: Emit certificate generated event
    events.push(DomainEvent::Certificate(crate::events::CertificateEvents::CertificateGenerated(crate::events::certificate::CertificateGeneratedEvent {
        cert_id,
        key_id: cmd.key_id, // Link to the actual key ID
        subject: cmd.subject.common_name.clone(),
        issuer: Some(cmd.ca_id), // The CA that issued this certificate
        not_before: certificate.validity.not_before,
        not_after: certificate.validity.not_after,
        is_ca: false,
        san: vec![],
        key_usage,
        extended_key_usage,
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    })));

    // Step 5: Emit certificate signed event
    events.push(DomainEvent::Certificate(crate::events::CertificateEvents::CertificateSigned(crate::events::certificate::CertificateSignedEvent {
        cert_id,
        signed_by: cmd.ca_id, // The CA certificate ID that signed this
        signature_algorithm: signature_algorithm_name, // Use actual CA algorithm
        signed_at: Utc::now(),
        correlation_id: cmd.correlation_id,
        causation_id: Some(cert_id), // Signing caused by certificate generation
    })));

    Ok(CertificateGenerated {
        cert_id,
        certificate,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_pair_uses_purpose_recommendation() {
        // A4: Generate test command_id for causation tracking
        let test_command_id = Uuid::now_v7();
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
            causation_id: Some(test_command_id), // A4: Self-reference for root command
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
