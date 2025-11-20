// SSI (Self-Sovereign Identity) Projections
//
// Projects domain compositions (Organization × Person × Certificate)
// into DID documents and Verifiable Credentials.
//
// This creates the DID-PKI bridge:
//   Organization Root CA ↔ Organization DID
//   Person Certificate ↔ Person DID

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::domain::{Organization, Person};
use crate::value_objects::{
    Certificate, CredentialProof, CredentialSubject, CredentialType, DID, DidDocument,
    DidMethod, ProofType, PublicKey, Service, ServiceType, VerifiableCredential,
    VerificationMethod, VerificationMethodType,
};

// ============================================================================
// Projection: Domain + Certificate → DID Document
// ============================================================================

/// Projection functor: Domain → DID Document
pub struct DidDocumentProjection;

impl DidDocumentProjection {
    /// Project organization with root CA to DID document
    ///
    /// Creates a did:web DID that references the organization's PKI root
    ///
    /// Emits:
    /// - DidDocumentProjectedEvent
    /// - DidPublishedEvent (when published to did:web location)
    pub fn project_organization_did(
        organization: &Organization,
        root_ca_cert: &Certificate,
        domain: &str,
    ) -> DidDocument {
        // Create did:web DID for organization
        let did = DID::new(
            DidMethod::Web,
            format!("{}:organization:{}", domain, organization.name.to_lowercase()),
        );

        let mut doc = DidDocument::new(did.clone());

        // Add verification method referencing the root CA public key
        let vm_id = DID::new(
            DidMethod::Web,
            format!(
                "{}:organization:{}#root-ca",
                domain,
                organization.name.to_lowercase()
            ),
        );

        doc.add_verification_method(VerificationMethod {
            id: vm_id.clone(),
            method_type: VerificationMethodType::X509Certificate2023,
            controller: did.clone(),
            public_key: root_ca_cert.public_key.clone(),
        });

        // This key can be used for assertion (signing credentials)
        doc.assertion_method.push(vm_id.clone());

        // Add service endpoint for DID resolution
        doc.add_service(Service {
            id: DID::new(
                DidMethod::Web,
                format!(
                    "{}:organization:{}#did-configuration",
                    domain,
                    organization.name.to_lowercase()
                ),
            ),
            service_type: ServiceType::LinkedDomains,
            service_endpoint: format!("https://{}", domain),
        });

        doc
    }

    /// Project person with certificate to DID document
    ///
    /// Creates a did:key DID (self-contained) that references person's X.509 cert
    ///
    /// Emits:
    /// - PersonDidProjectedEvent
    /// - DidDocumentCreatedEvent
    #[allow(unused_variables)]
    pub fn project_person_did(
        person: &Person,
        person_cert: &Certificate,
        organization_did: &DID,
    ) -> DidDocument {
        // Create did:key DID from person's public key
        let did = DID::new(
            DidMethod::Key,
            Self::encode_public_key_as_did_key(&person_cert.public_key),
        );

        let mut doc = DidDocument::new(did.clone());

        // Add verification method
        let vm_id = DID::with_fragment(
            DidMethod::Key,
            Self::encode_public_key_as_did_key(&person_cert.public_key),
            "x509-cert".to_string(),
        );

        doc.add_verification_method(VerificationMethod {
            id: vm_id.clone(),
            method_type: VerificationMethodType::X509Certificate2023,
            controller: did.clone(),
            public_key: person_cert.public_key.clone(),
        });

        // Person can authenticate and sign with this key
        doc.authentication.push(vm_id.clone());
        doc.assertion_method.push(vm_id);

        // Add service linking to organization
        doc.add_service(Service {
            id: DID::with_fragment(
                DidMethod::Key,
                Self::encode_public_key_as_did_key(&person_cert.public_key),
                "organization".to_string(),
            ),
            service_type: ServiceType::Custom("OrganizationMembership".to_string()),
            service_endpoint: organization_did.to_string(),
        });

        doc
    }

    /// Project certificate chain to DID document with full verification chain
    #[allow(unused_variables)]
    pub fn project_certificate_chain_to_did(
        person: &Person,
        leaf_cert: &Certificate,
        intermediate_cert: &Certificate,
        root_cert: &Certificate,
        organization_did: &DID,
    ) -> DidDocument {
        let did = DID::new(
            DidMethod::Key,
            Self::encode_public_key_as_did_key(&leaf_cert.public_key),
        );

        let mut doc = DidDocument::new(did.clone());

        // Add leaf certificate verification method
        let leaf_vm_id = DID::with_fragment(
            DidMethod::Key,
            Self::encode_public_key_as_did_key(&leaf_cert.public_key),
            "leaf-cert".to_string(),
        );

        doc.add_verification_method(VerificationMethod {
            id: leaf_vm_id.clone(),
            method_type: VerificationMethodType::X509Certificate2023,
            controller: did.clone(),
            public_key: leaf_cert.public_key.clone(),
        });

        // Add intermediate CA reference
        let intermediate_vm_id = DID::with_fragment(
            DidMethod::Key,
            Self::encode_public_key_as_did_key(&intermediate_cert.public_key),
            "intermediate-ca".to_string(),
        );

        doc.add_verification_method(VerificationMethod {
            id: intermediate_vm_id,
            method_type: VerificationMethodType::X509Certificate2023,
            controller: organization_did.clone(),
            public_key: intermediate_cert.public_key.clone(),
        });

        // Add root CA reference
        let root_vm_id = DID::with_fragment(
            DidMethod::Key,
            Self::encode_public_key_as_did_key(&root_cert.public_key),
            "root-ca".to_string(),
        );

        doc.add_verification_method(VerificationMethod {
            id: root_vm_id,
            method_type: VerificationMethodType::X509Certificate2023,
            controller: organization_did.clone(),
            public_key: root_cert.public_key.clone(),
        });

        doc.authentication.push(leaf_vm_id);

        doc
    }

    // Helper: Encode public key as did:key identifier
    fn encode_public_key_as_did_key(public_key: &PublicKey) -> String {
        // In real implementation, this would use multibase/multicodec encoding
        // For now, simplified version
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::URL_SAFE_NO_PAD.encode(&public_key.data)
    }
}

// ============================================================================
// Projection: Domain → Verifiable Credential
// ============================================================================

/// Projection functor: Domain → Verifiable Credential
pub struct VerifiableCredentialProjection;

impl VerifiableCredentialProjection {
    /// Project organization membership to verifiable credential
    ///
    /// Emits:
    /// - CredentialProjectedEvent
    /// - CredentialSignedEvent
    /// - CredentialIssuedEvent
    pub fn project_organization_membership(
        organization: &Organization,
        person: &Person,
        organization_did: &DID,
        person_did: &DID,
        issuer_private_key: &[u8], // For signing
    ) -> VerifiableCredential {
        let now = Utc::now();

        let credential_subject = CredentialSubject {
            id: person_did.clone(),
            claims: json!({
                "name": person.name,
                "email": person.email,
                "organization": organization.name,
                "memberSince": now,
                "roles": ["member"],
            }),
        };

        // Sign the credential (simplified - real implementation would use proper signing)
        let proof = CredentialProof {
            proof_type: ProofType::Ed25519Signature2020,
            created: now,
            verification_method: DID::with_fragment(
                organization_did.method.clone(),
                organization_did.identifier.clone(),
                "root-ca".to_string(),
            ),
            proof_value: issuer_private_key[..64].to_vec(), // Simplified
        };

        VerifiableCredential {
            id: Uuid::now_v7(),
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://cim.thecowboy.ai/credentials/v1".to_string(),
            ],
            credential_type: vec![CredentialType::OrganizationCredential],
            issuer: organization_did.clone(),
            issuance_date: now,
            expiration_date: None, // Permanent membership
            credential_subject,
            proof,
        }
    }

    /// Project role assignment to verifiable credential
    pub fn project_role_credential(
        organization: &Organization,
        person: &Person,
        role: &str,
        organization_did: &DID,
        person_did: &DID,
        issuer_private_key: &[u8],
    ) -> VerifiableCredential {
        let now = Utc::now();

        let credential_subject = CredentialSubject {
            id: person_did.clone(),
            claims: json!({
                "name": person.name,
                "email": person.email,
                "organization": organization.name,
                "role": role,
                "grantedAt": now,
            }),
        };

        let proof = CredentialProof {
            proof_type: ProofType::Ed25519Signature2020,
            created: now,
            verification_method: DID::with_fragment(
                organization_did.method.clone(),
                organization_did.identifier.clone(),
                "root-ca".to_string(),
            ),
            proof_value: issuer_private_key[..64].to_vec(),
        };

        VerifiableCredential {
            id: Uuid::now_v7(),
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://cim.thecowboy.ai/credentials/v1".to_string(),
            ],
            credential_type: vec![CredentialType::RoleCredential],
            issuer: organization_did.clone(),
            issuance_date: now,
            expiration_date: None,
            credential_subject,
            proof,
        }
    }

    /// Project certificate ownership to verifiable credential
    ///
    /// This creates a credential that attests to the person's ownership
    /// of a specific X.509 certificate
    pub fn project_certificate_credential(
        organization_did: &DID,
        person_did: &DID,
        certificate: &Certificate,
        issuer_private_key: &[u8],
    ) -> VerifiableCredential {
        let now = Utc::now();

        let credential_subject = CredentialSubject {
            id: person_did.clone(),
            claims: json!({
                "certificateSerialNumber": certificate.serial_number,
                "certificateFingerprint": certificate.fingerprint(),
                "certificateSubject": format!("{}", certificate.subject),
                "validFrom": certificate.validity.not_before,
                "validUntil": certificate.validity.not_after,
            }),
        };

        let proof = CredentialProof {
            proof_type: ProofType::Ed25519Signature2020,
            created: now,
            verification_method: DID::with_fragment(
                organization_did.method.clone(),
                organization_did.identifier.clone(),
                "root-ca".to_string(),
            ),
            proof_value: issuer_private_key[..64].to_vec(),
        };

        VerifiableCredential {
            id: Uuid::now_v7(),
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://cim.thecowboy.ai/credentials/v1".to_string(),
            ],
            credential_type: vec![CredentialType::CertificateCredential],
            issuer: organization_did.clone(),
            issuance_date: now,
            expiration_date: Some(certificate.validity.not_after),
            credential_subject,
            proof,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_key_encoding() {
        use crate::events::KeyAlgorithm;
        use crate::value_objects::PublicKeyFormat;

        let public_key = PublicKey {
            algorithm: KeyAlgorithm::Ed25519,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8],
            format: PublicKeyFormat::Der,
        };

        let encoded = DidDocumentProjection::encode_public_key_as_did_key(&public_key);
        assert!(!encoded.is_empty());
    }
}
