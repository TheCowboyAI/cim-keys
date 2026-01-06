//! Certificate generation service
//!
//! This service handles the actual cryptographic certificate generation
//! when processing certificate events. Following event-sourcing principles,
//! the aggregate emits events and this service performs the side effects.

use rcgen::{
    CertificateParams, DnType,
    SanType, BasicConstraints, IsCa, KeyUsagePurpose,
    ExtendedKeyUsagePurpose, KeyPair, Issuer,
};
use time::{Duration, OffsetDateTime};
use crate::events::certificate::CertificateGeneratedEvent;

/// Result of certificate generation
pub struct GeneratedCertificate {
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub public_key_pem: String,
    pub fingerprint: String,
}

/// Generate a Root CA certificate from an event (self-signed)
pub fn generate_root_ca_from_event(
    event: &CertificateGeneratedEvent
) -> Result<GeneratedCertificate, String> {
    // Create certificate parameters with no SANs for CA
    let mut params = CertificateParams::new(Vec::new())
        .map_err(|e| format!("Failed to create certificate params: {}", e))?;

    // Set as CA certificate
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    // Set distinguished name from the event subject using typed value objects
    params.distinguished_name.push(DnType::CommonName, event.subject_name.common_name.as_str());

    if let Some(ref org) = event.subject_name.organization {
        params.distinguished_name.push(DnType::OrganizationName, org.as_str());
    }
    if let Some(ref ou) = event.subject_name.organizational_unit {
        params.distinguished_name.push(DnType::OrganizationalUnitName, ou.as_str());
    }
    if let Some(ref country) = event.subject_name.country {
        params.distinguished_name.push(DnType::CountryName, country.as_str());
    }
    if let Some(ref state) = event.subject_name.state {
        params.distinguished_name.push(DnType::StateOrProvinceName, state.as_str());
    }
    if let Some(ref locality) = event.subject_name.locality {
        params.distinguished_name.push(DnType::LocalityName, locality.as_str());
    }

    // Set validity period using time crate
    let day = Duration::days(1);
    let not_before = OffsetDateTime::now_utc().checked_sub(day)
        .ok_or("Failed to calculate not_before date")?;
    let not_after = OffsetDateTime::now_utc().checked_add(Duration::days(3650))
        .ok_or("Failed to calculate not_after date")?;

    params.not_before = not_before;
    params.not_after = not_after;

    // Add Subject Alternative Names if provided
    if let Some(ref san) = event.subject_alt_name {
        for entry in san.entries() {
            match entry {
                crate::value_objects::x509::SanEntry::Email(email) => {
                    if let Ok(e) = email.as_str().try_into() {
                        params.subject_alt_names.push(SanType::Rfc822Name(e));
                    }
                }
                crate::value_objects::x509::SanEntry::IpAddress(ip) => {
                    params.subject_alt_names.push(SanType::IpAddress(*ip.addr()));
                }
                crate::value_objects::x509::SanEntry::DnsName(dns) => {
                    if let Ok(dns_name) = dns.as_str().try_into() {
                        params.subject_alt_names.push(SanType::DnsName(dns_name));
                    }
                }
                crate::value_objects::x509::SanEntry::Uri(uri) => {
                    // URIs are handled differently in rcgen, skip for now
                    let _ = uri;
                }
                crate::value_objects::x509::SanEntry::Other { .. } => {}
            }
        }
    }

    // Set key usage for CA
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);

    // Generate key pair
    let key_pair = KeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;

    // Create self-signed certificate
    let cert = params.self_signed(&key_pair)
        .map_err(|e| format!("Failed to create self-signed certificate: {}", e))?;

    // Get PEM representations
    let cert_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    // Calculate fingerprint (SHA256 of DER-encoded certificate)
    use sha2::{Sha256, Digest};
    let cert_der = cert.der();
    let mut hasher = Sha256::new();
    hasher.update(cert_der);
    let fingerprint = hex::encode(hasher.finalize());

    Ok(GeneratedCertificate {
        certificate_pem: cert_pem,
        private_key_pem,
        public_key_pem: String::new(), // Can be extracted if needed
        fingerprint,
    })
}

/// Generate an intermediate CA certificate (signed by root CA)
pub fn generate_intermediate_ca_from_event(
    event: &CertificateGeneratedEvent,
    issuer_params: CertificateParams,
    issuer_key_pair: &KeyPair,
) -> Result<GeneratedCertificate, String> {
    // Create certificate parameters
    let mut params = CertificateParams::new(Vec::new())
        .map_err(|e| format!("Failed to create certificate params: {}", e))?;

    // Set as CA certificate with path length constraint
    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));

    // Set distinguished name from the event subject using typed value objects
    params.distinguished_name.push(DnType::CommonName, event.subject_name.common_name.as_str());

    if let Some(ref org) = event.subject_name.organization {
        params.distinguished_name.push(DnType::OrganizationName, org.as_str());
    }
    if let Some(ref ou) = event.subject_name.organizational_unit {
        params.distinguished_name.push(DnType::OrganizationalUnitName, ou.as_str());
    }
    if let Some(ref country) = event.subject_name.country {
        params.distinguished_name.push(DnType::CountryName, country.as_str());
    }

    // Set validity period
    let day = Duration::days(1);
    let not_before = OffsetDateTime::now_utc().checked_sub(day)
        .ok_or("Failed to calculate not_before date")?;
    let not_after = OffsetDateTime::now_utc().checked_add(Duration::days(1825)) // 5 years
        .ok_or("Failed to calculate not_after date")?;

    params.not_before = not_before;
    params.not_after = not_after;

    // Set key usage
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);

    // Use authority key identifier
    params.use_authority_key_identifier_extension = true;

    // Generate key pair for this certificate
    let key_pair = KeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;

    // Create issuer from the root CA
    let issuer = Issuer::new(issuer_params, issuer_key_pair);

    // Sign the certificate with the issuer
    let cert = params.signed_by(&key_pair, &issuer)
        .map_err(|e| format!("Failed to sign certificate: {}", e))?;

    // Get PEM representations
    let cert_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    // Calculate fingerprint
    use sha2::{Sha256, Digest};
    let cert_der = cert.der();
    let mut hasher = Sha256::new();
    hasher.update(cert_der);
    let fingerprint = hex::encode(hasher.finalize());

    Ok(GeneratedCertificate {
        certificate_pem: cert_pem,
        private_key_pem,
        public_key_pem: String::new(),
        fingerprint,
    })
}

/// Generate a leaf certificate (end-entity, signed by CA)
pub fn generate_leaf_certificate_from_event(
    event: &CertificateGeneratedEvent,
    issuer_params: CertificateParams,
    issuer_key_pair: &KeyPair,
) -> Result<GeneratedCertificate, String> {
    // Create certificate parameters with SANs - use common name as default SAN
    let common_name = event.subject_name.common_name.as_str().to_string();
    let mut params = CertificateParams::new(vec![common_name])
        .map_err(|e| format!("Failed to create certificate params: {}", e))?;

    // Not a CA certificate
    params.is_ca = IsCa::NoCa;

    // Set distinguished name from the event subject using typed value objects
    params.distinguished_name.push(DnType::CommonName, event.subject_name.common_name.as_str());

    if let Some(ref org) = event.subject_name.organization {
        params.distinguished_name.push(DnType::OrganizationName, org.as_str());
    }
    if let Some(ref ou) = event.subject_name.organizational_unit {
        params.distinguished_name.push(DnType::OrganizationalUnitName, ou.as_str());
    }
    if let Some(ref country) = event.subject_name.country {
        params.distinguished_name.push(DnType::CountryName, country.as_str());
    }

    // Set validity period (shorter for leaf certs)
    let day = Duration::days(1);
    let not_before = OffsetDateTime::now_utc().checked_sub(day)
        .ok_or("Failed to calculate not_before date")?;
    let not_after = OffsetDateTime::now_utc().checked_add(Duration::days(365)) // 1 year
        .ok_or("Failed to calculate not_after date")?;

    params.not_before = not_before;
    params.not_after = not_after;

    // Add extended key usage for leaf certificates
    if let Some(ref eku) = event.extended_key_usage {
        for purpose_str in eku.to_string_list() {
            match purpose_str.as_str() {
                "TLS Web Server Authentication" | "serverAuth" => params.extended_key_usages.push(ExtendedKeyUsagePurpose::ServerAuth),
                "TLS Web Client Authentication" | "clientAuth" => params.extended_key_usages.push(ExtendedKeyUsagePurpose::ClientAuth),
                "Code Signing" | "codeSigning" => params.extended_key_usages.push(ExtendedKeyUsagePurpose::CodeSigning),
                "Email Protection" | "emailProtection" => params.extended_key_usages.push(ExtendedKeyUsagePurpose::EmailProtection),
                _ => {}
            }
        }
    }

    // Add key usage
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyEncipherment);

    // Use authority key identifier
    params.use_authority_key_identifier_extension = true;

    // Generate key pair for this certificate
    let key_pair = KeyPair::generate()
        .map_err(|e| format!("Failed to generate key pair: {}", e))?;

    // Create issuer from the CA
    let issuer = Issuer::new(issuer_params, issuer_key_pair);

    // Sign the certificate with the issuer
    let cert = params.signed_by(&key_pair, &issuer)
        .map_err(|e| format!("Failed to sign certificate: {}", e))?;

    // Get PEM representations
    let cert_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    // Calculate fingerprint
    use sha2::{Sha256, Digest};
    let cert_der = cert.der();
    let mut hasher = Sha256::new();
    hasher.update(cert_der);
    let fingerprint = hex::encode(hasher.finalize());

    Ok(GeneratedCertificate {
        certificate_pem: cert_pem,
        private_key_pem,
        public_key_pem: String::new(),
        fingerprint,
    })
}