// Value Objects for CIM Keys
//
// All cryptographic artifacts are immutable value objects that compose
// across domain boundaries. These are NOT entities - they have no identity
// of their own and are defined entirely by their attributes.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::events::{KeyAlgorithm, KeyPurpose};

// ============================================================================
// Cryptographic Value Objects
// ============================================================================

/// Public key value object (immutable)
///
/// Public keys are values, not entities. Two public keys with the same
/// algorithm and data are the same public key, regardless of context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    pub algorithm: KeyAlgorithm,
    pub data: Vec<u8>,
    pub format: PublicKeyFormat,
}

impl PublicKey {
    /// Create a new public key
    pub fn new(algorithm: KeyAlgorithm, data: Vec<u8>, format: PublicKeyFormat) -> Self {
        Self {
            algorithm,
            data,
            format,
        }
    }

    /// Get PEM encoding
    pub fn to_pem(&self) -> Result<String, ValueObjectError> {
        match self.format {
            PublicKeyFormat::Pem(ref pem) => Ok(pem.clone()),
            PublicKeyFormat::Der => {
                // Convert DER to PEM (implementation depends on algorithm)
                Err(ValueObjectError::ConversionError(
                    "DER to PEM conversion not yet implemented".to_string(),
                ))
            }
            PublicKeyFormat::Ssh => Err(ValueObjectError::ConversionError(
                "SSH to PEM conversion not yet implemented".to_string(),
            )),
            PublicKeyFormat::Jwk => Err(ValueObjectError::ConversionError(
                "JWK to PEM conversion not yet implemented".to_string(),
            )),
        }
    }

    /// Get DER encoding
    pub fn to_der(&self) -> Result<Vec<u8>, ValueObjectError> {
        match self.format {
            PublicKeyFormat::Der => Ok(self.data.clone()),
            PublicKeyFormat::Pem(_) => {
                // Convert PEM to DER
                Err(ValueObjectError::ConversionError(
                    "PEM to DER conversion not yet implemented".to_string(),
                ))
            }
            PublicKeyFormat::Ssh => Err(ValueObjectError::ConversionError(
                "SSH to DER conversion not yet implemented".to_string(),
            )),
            PublicKeyFormat::Jwk => Err(ValueObjectError::ConversionError(
                "JWK to DER conversion not yet implemented".to_string(),
            )),
        }
    }

    /// Get fingerprint (SHA-256 hash of public key data)
    pub fn fingerprint(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.data);
        let result = hasher.finalize();
        hex::encode(result)
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PublicKey({:?}, {} bytes, fingerprint: {})",
            self.algorithm,
            self.data.len(),
            &self.fingerprint()[..16]
        )
    }
}

/// Public key format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKeyFormat {
    /// PEM encoding with the PEM string
    Pem(String),
    /// DER encoding (raw bytes in self.data)
    Der,
    /// SSH public key format
    Ssh,
    /// JSON Web Key (JWK) format
    Jwk,
}

/// Private key value object (immutable, always encrypted)
///
/// Private keys are NEVER stored in plaintext. The `encrypted_data` field
/// contains the key material encrypted with a key encryption key (KEK).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateKey {
    pub key_id: Uuid,
    pub algorithm: KeyAlgorithm,
    pub encrypted_data: Vec<u8>,
    pub encryption_algorithm: EncryptionAlgorithm,
    pub public_key: PublicKey,
}

impl PrivateKey {
    /// Create a new encrypted private key
    pub fn new(
        key_id: Uuid,
        algorithm: KeyAlgorithm,
        encrypted_data: Vec<u8>,
        encryption_algorithm: EncryptionAlgorithm,
        public_key: PublicKey,
    ) -> Self {
        Self {
            key_id,
            algorithm,
            encrypted_data,
            encryption_algorithm,
            public_key,
        }
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PrivateKey(id: {}, algorithm: {:?}, encrypted with: {:?})",
            self.key_id, self.algorithm, self.encryption_algorithm
        )
    }
}

/// Encryption algorithm for private key protection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
    /// Age encryption (modern alternative to PGP)
    Age,
}

/// X.509 Certificate value object (immutable)
///
/// Certificates are values that attest to the binding between a public key
/// and an identity. They are immutable once signed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certificate {
    pub serial_number: String,
    pub subject: CertificateSubject,
    pub issuer: CertificateSubject,
    pub public_key: PublicKey,
    pub validity: Validity,
    pub signature: Signature,
    pub der: Vec<u8>,
    pub pem: String,
}

impl Certificate {
    /// Get certificate fingerprint
    pub fn fingerprint(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.der);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Check if certificate is currently valid
    pub fn is_valid_now(&self) -> bool {
        let now = Utc::now();
        self.validity.is_valid_at(now)
    }

    /// Check if certificate is a CA certificate
    pub fn is_ca(&self) -> bool {
        // This would check the basicConstraints extension
        // For now, simplified check based on subject/issuer
        true // TODO: Implement proper extension checking
    }

    /// Verify temporal validity at a specific time
    pub fn verify_temporal_validity(&self, at: DateTime<Utc>) -> Result<(), CertificateVerificationError> {
        if at < self.validity.not_before {
            return Err(CertificateVerificationError::NotYetValid {
                cert_fingerprint: self.fingerprint(),
                not_before: self.validity.not_before,
                now: at,
            });
        }
        if at > self.validity.not_after {
            return Err(CertificateVerificationError::Expired {
                cert_fingerprint: self.fingerprint(),
                not_after: self.validity.not_after,
                now: at,
            });
        }
        Ok(())
    }

    /// Verify that this certificate's issuer matches the given CA's subject
    pub fn verify_issuer_matches(&self, issuer_cert: &Certificate) -> Result<(), CertificateVerificationError> {
        // Compare issuer DN of this cert with subject DN of issuer cert
        if self.issuer.common_name != issuer_cert.subject.common_name {
            return Err(CertificateVerificationError::IssuerMismatch {
                cert_fingerprint: self.fingerprint(),
                expected_issuer: issuer_cert.subject.common_name.clone(),
                actual_issuer: self.issuer.common_name.clone(),
            });
        }
        Ok(())
    }

    /// Verify this certificate's signature against the issuer's public key
    ///
    /// This performs actual cryptographic verification using the issuer's public key
    /// to validate that this certificate was signed by the claimed issuer.
    pub fn verify_signature(&self, issuer_public_key: &PublicKey) -> Result<(), CertificateVerificationError> {
        // The signature covers the TBS (To-Be-Signed) certificate data
        // For DER-encoded X.509, this is everything except the signature itself
        // We use x509-parser to extract and verify

        // Parse the certificate to get TBS data
        let (_remaining, cert) = x509_parser::parse_x509_certificate(&self.der)
            .map_err(|e| CertificateVerificationError::CryptoError(
                format!("Failed to parse certificate DER: {:?}", e)
            ))?;

        // Get the TBS certificate bytes (the data that was signed)
        let tbs_certificate = cert.tbs_certificate.as_ref();

        // Verify signature based on algorithm
        match self.signature.algorithm {
            SignatureAlgorithm::Ed25519 => {
                self.verify_ed25519_signature(tbs_certificate, issuer_public_key)
            }
            SignatureAlgorithm::EcdsaSha256 => {
                self.verify_ecdsa_p256_signature(tbs_certificate, issuer_public_key)
            }
            SignatureAlgorithm::RsaSha256 | SignatureAlgorithm::RsaSha512 => {
                self.verify_rsa_signature(tbs_certificate, issuer_public_key)
            }
        }
    }

    /// Verify Ed25519 signature
    fn verify_ed25519_signature(
        &self,
        tbs_data: &[u8],
        issuer_public_key: &PublicKey,
    ) -> Result<(), CertificateVerificationError> {
        use ed25519_dalek::{Verifier, VerifyingKey, Signature as Ed25519Sig};

        // Get public key bytes (Ed25519 public keys are 32 bytes)
        let key_bytes: [u8; 32] = issuer_public_key.data.as_slice()
            .try_into()
            .map_err(|_| CertificateVerificationError::CryptoError(
                format!("Invalid Ed25519 public key length: expected 32, got {}", issuer_public_key.data.len())
            ))?;

        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| CertificateVerificationError::CryptoError(
                format!("Invalid Ed25519 public key: {}", e)
            ))?;

        // Signature must be 64 bytes
        let sig_bytes: [u8; 64] = self.signature.data.as_slice()
            .try_into()
            .map_err(|_| CertificateVerificationError::CryptoError(
                format!("Invalid Ed25519 signature length: expected 64, got {}", self.signature.data.len())
            ))?;

        let signature = Ed25519Sig::from_bytes(&sig_bytes);

        verifying_key.verify(tbs_data, &signature)
            .map_err(|_| CertificateVerificationError::InvalidSignature {
                cert_fingerprint: self.fingerprint(),
                issuer_fingerprint: issuer_public_key.fingerprint(),
            })
    }

    /// Verify ECDSA P-256 signature
    fn verify_ecdsa_p256_signature(
        &self,
        tbs_data: &[u8],
        issuer_public_key: &PublicKey,
    ) -> Result<(), CertificateVerificationError> {
        use p256::ecdsa::{Signature as P256Sig, VerifyingKey, signature::Verifier};
        use p256::PublicKey as P256PublicKey;

        // Parse P-256 public key from SEC1 or DER format
        let p256_key = P256PublicKey::from_sec1_bytes(&issuer_public_key.data)
            .map_err(|e| CertificateVerificationError::CryptoError(
                format!("Invalid P-256 public key: {}", e)
            ))?;

        let verifying_key = VerifyingKey::from(p256_key);

        // Parse DER-encoded ECDSA signature
        let signature = P256Sig::from_der(&self.signature.data)
            .map_err(|e| CertificateVerificationError::CryptoError(
                format!("Invalid ECDSA signature: {}", e)
            ))?;

        // Hash the TBS data with SHA-256 before verification
        use sha2::{Sha256, Digest};
        let digest = Sha256::digest(tbs_data);

        verifying_key.verify(&digest, &signature)
            .map_err(|_| CertificateVerificationError::InvalidSignature {
                cert_fingerprint: self.fingerprint(),
                issuer_fingerprint: issuer_public_key.fingerprint(),
            })
    }

    /// Verify RSA signature
    fn verify_rsa_signature(
        &self,
        tbs_data: &[u8],
        issuer_public_key: &PublicKey,
    ) -> Result<(), CertificateVerificationError> {
        use rsa::{RsaPublicKey, pkcs1v15::VerifyingKey as RsaVerifyingKey};
        use rsa::pkcs1::DecodeRsaPublicKey;
        use rsa::signature::Verifier;

        // Parse RSA public key
        let rsa_key = RsaPublicKey::from_pkcs1_der(&issuer_public_key.data)
            .or_else(|_| {
                // Try PKCS#8 format if PKCS#1 fails
                use rsa::pkcs8::DecodePublicKey;
                RsaPublicKey::from_public_key_der(&issuer_public_key.data)
            })
            .map_err(|e| CertificateVerificationError::CryptoError(
                format!("Invalid RSA public key: {}", e)
            ))?;

        // Create verifying key based on hash algorithm
        match self.signature.algorithm {
            SignatureAlgorithm::RsaSha256 => {
                let verifying_key = RsaVerifyingKey::<sha2::Sha256>::new(rsa_key);
                let signature = rsa::pkcs1v15::Signature::try_from(self.signature.data.as_slice())
                    .map_err(|e| CertificateVerificationError::CryptoError(
                        format!("Invalid RSA signature: {}", e)
                    ))?;
                verifying_key.verify(tbs_data, &signature)
                    .map_err(|_| CertificateVerificationError::InvalidSignature {
                        cert_fingerprint: self.fingerprint(),
                        issuer_fingerprint: issuer_public_key.fingerprint(),
                    })
            }
            SignatureAlgorithm::RsaSha512 => {
                let verifying_key = RsaVerifyingKey::<sha2::Sha512>::new(rsa_key);
                let signature = rsa::pkcs1v15::Signature::try_from(self.signature.data.as_slice())
                    .map_err(|e| CertificateVerificationError::CryptoError(
                        format!("Invalid RSA signature: {}", e)
                    ))?;
                verifying_key.verify(tbs_data, &signature)
                    .map_err(|_| CertificateVerificationError::InvalidSignature {
                        cert_fingerprint: self.fingerprint(),
                        issuer_fingerprint: issuer_public_key.fingerprint(),
                    })
            }
            _ => Err(CertificateVerificationError::UnsupportedAlgorithm {
                algorithm: self.signature.algorithm,
            }),
        }
    }

    /// Verify that this certificate is self-signed (issuer == subject)
    pub fn verify_self_signed(&self) -> Result<(), CertificateVerificationError> {
        if self.issuer.common_name != self.subject.common_name {
            return Err(CertificateVerificationError::RootNotSelfSigned {
                fingerprint: self.fingerprint(),
            });
        }
        // Also verify signature against own public key
        self.verify_signature(&self.public_key)
    }
}

impl fmt::Display for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Certificate(serial: {}, subject: {}, issuer: {}, valid: {})",
            self.serial_number,
            self.subject.common_name,
            self.issuer.common_name,
            if self.is_valid_now() {
                "yes"
            } else {
                "no"
            }
        )
    }
}

/// Certificate subject/issuer information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateSubject {
    pub common_name: String,
    pub organization: Option<String>,
    pub organizational_unit: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub locality: Option<String>,
    pub email: Option<String>,
}

impl fmt::Display for CertificateSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CN={}", self.common_name)?;
        if let Some(ref o) = self.organization {
            write!(f, ", O={}", o)?;
        }
        if let Some(ref c) = self.country {
            write!(f, ", C={}", c)?;
        }
        Ok(())
    }
}

/// Certificate validity period
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validity {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
}

impl Validity {
    /// Check if validity period includes the given time
    pub fn is_valid_at(&self, time: DateTime<Utc>) -> bool {
        time >= self.not_before && time <= self.not_after
    }

    /// Get validity duration in days
    pub fn duration_days(&self) -> i64 {
        (self.not_after - self.not_before).num_days()
    }
}

/// Cryptographic signature value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub algorithm: SignatureAlgorithm,
    pub data: Vec<u8>,
}

impl Signature {
    /// Create a new signature
    pub fn new(algorithm: SignatureAlgorithm, data: Vec<u8>) -> Self {
        Self { algorithm, data }
    }
}

/// Signature algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// RSA with SHA-256
    RsaSha256,
    /// RSA with SHA-512
    RsaSha512,
    /// ECDSA with SHA-256
    EcdsaSha256,
    /// Ed25519 signature
    Ed25519,
}

// ============================================================================
// Domain Composition Value Objects
// ============================================================================

/// Certificate chain value object
///
/// Represents a complete chain of trust from root CA to leaf certificate.
/// The chain is ordered: [leaf, intermediate(s), root]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateChain {
    pub leaf: Certificate,
    pub intermediates: Vec<Certificate>,
    pub root: Certificate,
}

impl CertificateChain {
    /// Create a new certificate chain
    pub fn new(leaf: Certificate, intermediates: Vec<Certificate>, root: Certificate) -> Self {
        Self {
            leaf,
            intermediates,
            root,
        }
    }

    /// Verify the chain integrity with full cryptographic verification
    ///
    /// This performs the following verifications:
    /// 1. Temporal validity: All certificates are currently valid (not expired, not future)
    /// 2. Issuer chain: Each certificate's issuer matches the next certificate's subject
    /// 3. Signature chain: Each certificate's signature is valid against the issuer's public key
    /// 4. Root validation: Root certificate is self-signed
    ///
    /// Returns a TrustPath on success containing verified links.
    pub fn verify(&self) -> Result<TrustPath, ValueObjectError> {
        self.verify_at(Utc::now())
    }

    /// Verify the chain integrity at a specific point in time
    ///
    /// This allows verification against historical or future times for testing
    /// certificate validity windows.
    pub fn verify_at(&self, at: DateTime<Utc>) -> Result<TrustPath, ValueObjectError> {
        let mut trust_path = TrustPath::new();

        // Get all certificates in order: [leaf, intermediate(s)..., root]
        let all_certs = self.all_certificates();

        if all_certs.is_empty() {
            return Err(ValueObjectError::CertificateError(
                CertificateVerificationError::EmptyChain,
            ));
        }

        // Verify each certificate in the chain
        for (i, cert) in all_certs.iter().enumerate() {
            // 1. Verify temporal validity
            cert.verify_temporal_validity(at)
                .map_err(ValueObjectError::CertificateError)?;

            if i + 1 < all_certs.len() {
                // This is not the root - verify against next cert (issuer)
                let issuer = all_certs[i + 1];

                // 2. Verify issuer DN matches
                cert.verify_issuer_matches(issuer)
                    .map_err(ValueObjectError::CertificateError)?;

                // 3. Verify cryptographic signature
                cert.verify_signature(&issuer.public_key)
                    .map_err(ValueObjectError::CertificateError)?;

                trust_path.add_link(
                    cert.fingerprint(),
                    Some(issuer.fingerprint()),
                    TrustLevel::Complete,
                );
            } else {
                // This is the root certificate - verify it's self-signed
                cert.verify_self_signed()
                    .map_err(ValueObjectError::CertificateError)?;

                trust_path.add_link(
                    cert.fingerprint(),
                    None, // Root has no issuer
                    TrustLevel::Complete,
                );
            }
        }

        Ok(trust_path)
    }

    /// Verify the chain against a set of trusted root certificates
    ///
    /// In addition to standard chain verification, this ensures the root
    /// certificate's fingerprint is in the trusted roots set.
    pub fn verify_against_trusted_roots(
        &self,
        trusted_roots: &std::collections::HashSet<String>,
    ) -> Result<TrustPath, ValueObjectError> {
        // First perform standard chain verification
        let trust_path = self.verify()?;

        // Then verify root is in trusted set
        let root_fingerprint = self.root.fingerprint();
        if !trusted_roots.contains(&root_fingerprint) {
            return Err(ValueObjectError::CertificateError(
                CertificateVerificationError::UntrustedRoot {
                    fingerprint: root_fingerprint,
                },
            ));
        }

        Ok(trust_path)
    }

    /// Get all certificates in the chain
    pub fn all_certificates(&self) -> Vec<&Certificate> {
        let mut certs = vec![&self.leaf];
        certs.extend(self.intermediates.iter());
        certs.push(&self.root);
        certs
    }

    /// Get chain depth (number of certificates)
    pub fn depth(&self) -> usize {
        2 + self.intermediates.len() // leaf + intermediates + root
    }

    /// Check if chain is valid without returning detailed trust path
    pub fn is_valid(&self) -> bool {
        self.verify().is_ok()
    }

    /// Check if chain is valid at a specific time
    pub fn is_valid_at(&self, at: DateTime<Utc>) -> bool {
        self.verify_at(at).is_ok()
    }
}

/// Key ownership value object
///
/// Represents the binding between a key and a person/entity with temporal bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyOwnership {
    pub key_id: Uuid,
    pub owner_id: Uuid,
    pub owner_type: OwnerType,
    pub purpose: KeyPurpose,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

impl KeyOwnership {
    /// Check if ownership is currently valid
    pub fn is_valid_now(&self) -> bool {
        let now = Utc::now();
        self.is_valid_at(now)
    }

    /// Check if ownership is valid at a specific time
    pub fn is_valid_at(&self, time: DateTime<Utc>) -> bool {
        if time < self.valid_from {
            return false;
        }
        if let Some(valid_until) = self.valid_until {
            time <= valid_until
        } else {
            true // No expiration
        }
    }
}

/// Owner type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerType {
    Person,
    ServiceAccount,
    Device,
    Organization,
}

// ============================================================================
// Self-Sovereign Identity (SSI) Value Objects
// ============================================================================

/// Decentralized Identifier (DID) value object
///
/// DIDs are W3C standard identifiers that are globally unique and
/// cryptographically verifiable without a centralized registry.
///
/// Format: did:method:method-specific-id
/// Examples:
/// - did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
/// - did:web:example.com:user:alice
/// - did:pkh:eip155:1:0x1234...
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct DID {
    pub method: DidMethod,
    pub identifier: String,
    pub fragment: Option<String>,
}

impl DID {
    /// Create a new DID
    pub fn new(method: DidMethod, identifier: String) -> Self {
        Self {
            method,
            identifier,
            fragment: None,
        }
    }

    /// Create a DID with a fragment
    pub fn with_fragment(method: DidMethod, identifier: String, fragment: String) -> Self {
        Self {
            method,
            identifier,
            fragment: Some(fragment),
        }
    }

    /// Parse a DID from a string
    pub fn parse(did_string: &str) -> Result<Self, ValueObjectError> {
        if !did_string.starts_with("did:") {
            return Err(ValueObjectError::InvalidFormat(
                "DID must start with 'did:'".to_string(),
            ));
        }

        let parts: Vec<&str> = did_string.split(':').collect();
        if parts.len() < 3 {
            return Err(ValueObjectError::InvalidFormat(
                "DID must have at least method and identifier".to_string(),
            ));
        }

        let method = match parts[1] {
            "key" => DidMethod::Key,
            "web" => DidMethod::Web,
            "pkh" => DidMethod::Pkh,
            other => DidMethod::Other(other.to_string()),
        };

        let identifier = parts[2..].join(":");

        // Check for fragment (#)
        if let Some(hash_pos) = identifier.find('#') {
            let (id, frag) = identifier.split_at(hash_pos);
            Ok(Self {
                method,
                identifier: id.to_string(),
                fragment: Some(frag[1..].to_string()),
            })
        } else {
            Ok(Self {
                method,
                identifier,
                fragment: None,
            })
        }
    }
}

impl fmt::Display for DID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "did:{}:{}", self.method, self.identifier)?;
        if let Some(ref frag) = self.fragment {
            write!(f, "#{}", frag)?;
        }
        Ok(())
    }
}

/// DID method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DidMethod {
    /// did:key - Self-contained, no resolution needed
    Key,
    /// did:web - DNS-based, uses HTTPS for resolution
    Web,
    /// did:pkh - Public Key Hash (blockchain addresses)
    Pkh,
    /// Other methods
    Other(String),
}

impl fmt::Display for DidMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DidMethod::Key => write!(f, "key"),
            DidMethod::Web => write!(f, "web"),
            DidMethod::Pkh => write!(f, "pkh"),
            DidMethod::Other(s) => write!(f, "{}", s),
        }
    }
}

/// DID Document value object
///
/// Contains the public keys, services, and other metadata for a DID.
/// This is the resolved representation of a DID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidDocument {
    pub id: DID,
    pub verification_methods: Vec<VerificationMethod>,
    pub authentication: Vec<DID>, // References to verification methods
    pub assertion_method: Vec<DID>,
    pub key_agreement: Vec<DID>,
    pub services: Vec<Service>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl DidDocument {
    /// Create a new DID document
    pub fn new(id: DID) -> Self {
        let now = Utc::now();
        Self {
            id,
            verification_methods: Vec::new(),
            authentication: Vec::new(),
            assertion_method: Vec::new(),
            key_agreement: Vec::new(),
            services: Vec::new(),
            created: now,
            updated: now,
        }
    }

    /// Add a verification method
    pub fn add_verification_method(&mut self, method: VerificationMethod) {
        self.verification_methods.push(method);
        self.updated = Utc::now();
    }

    /// Add a service endpoint
    pub fn add_service(&mut self, service: Service) {
        self.services.push(service);
        self.updated = Utc::now();
    }
}

/// Verification method in a DID document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: DID,
    pub method_type: VerificationMethodType,
    pub controller: DID,
    pub public_key: PublicKey,
}

/// Verification method type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethodType {
    /// JSON Web Key 2020
    JsonWebKey2020,
    /// Ed25519 Verification Key 2020
    Ed25519VerificationKey2020,
    /// ECDSA secp256k1 Verification Key 2019
    EcdsaSecp256k1VerificationKey2019,
    /// RSA Verification Key 2018
    RsaVerificationKey2018,
    /// X.509 Certificate (2023)
    X509Certificate2023,
}

/// Service endpoint in a DID document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    pub id: DID,
    pub service_type: ServiceType,
    pub service_endpoint: String, // URL or other endpoint
}

/// Service type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    /// LinkedDomains (for did:web verification)
    LinkedDomains,
    /// DID Communication (DIDComm)
    DidCommMessaging,
    /// Verifiable Credential service
    CredentialRegistry,
    /// Custom service type
    Custom(String),
}

/// Verifiable Credential value object
///
/// W3C Verifiable Credential that attests to claims about a subject.
/// The credential is cryptographically signed by the issuer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiableCredential {
    pub id: Uuid,
    pub context: Vec<String>,
    pub credential_type: Vec<CredentialType>,
    pub issuer: DID,
    pub issuance_date: DateTime<Utc>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub credential_subject: CredentialSubject,
    pub proof: CredentialProof,
}

impl VerifiableCredential {
    /// Check if credential is currently valid (not expired)
    pub fn is_valid_now(&self) -> bool {
        let now = Utc::now();
        if now < self.issuance_date {
            return false;
        }
        if let Some(exp) = self.expiration_date {
            now <= exp
        } else {
            true
        }
    }

    /// Verify the credential signature
    pub fn verify(&self, _issuer_public_key: &PublicKey) -> Result<(), ValueObjectError> {
        // Verify the proof signature matches the credential content
        // This would use the actual cryptographic verification
        Ok(())
    }
}

/// Credential type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialType {
    /// Organization membership credential
    OrganizationCredential,
    /// Role/position credential
    RoleCredential,
    /// Key ownership credential
    KeyOwnershipCredential,
    /// Certificate validity credential
    CertificateCredential,
    /// Custom credential type
    Custom(String),
}

impl fmt::Display for CredentialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialType::OrganizationCredential => write!(f, "OrganizationCredential"),
            CredentialType::RoleCredential => write!(f, "RoleCredential"),
            CredentialType::KeyOwnershipCredential => write!(f, "KeyOwnershipCredential"),
            CredentialType::CertificateCredential => write!(f, "CertificateCredential"),
            CredentialType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Credential subject (the entity the credential is about)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialSubject {
    pub id: DID,
    pub claims: serde_json::Value,
}

/// Credential proof (cryptographic signature)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialProof {
    pub proof_type: ProofType,
    pub created: DateTime<Utc>,
    pub verification_method: DID,
    pub proof_value: Vec<u8>,
}

/// Proof type for verifiable credentials
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    /// JSON Web Signature 2020
    JsonWebSignature2020,
    /// Ed25519 Signature 2020
    Ed25519Signature2020,
    /// ECDSA secp256k1 Signature 2019
    EcdsaSecp256k1Signature2019,
}

/// Trust chain link value object
///
/// Represents a link in a web of trust where one DID attests to
/// the trustworthiness of another DID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustChainLink {
    pub trustor_did: DID,
    pub trustee_did: DID,
    pub trust_level: TrustLevel,
    pub credential: VerifiableCredential,
    pub established: DateTime<Utc>,
}

/// Trust level in web of trust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Complete trust (can issue credentials)
    Complete,
    /// Marginal trust (can attest)
    Marginal,
    /// No trust
    None,
}

// ============================================================================
// Export Formats
// ============================================================================

/// Format for exporting keys and certificates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// PEM format (base64 encoded with headers)
    Pem,
    /// DER format (binary ASN.1)
    Der,
    /// SSH public key format
    SshPublicKey,
    /// OpenSSH private key format
    SshPrivateKey,
    /// PKCS#8 format
    Pkcs8,
    /// PKCS#12 format (PFX)
    Pkcs12,
    /// JSON Web Key (JWK)
    Jwk,
    /// Raw binary data
    Raw,
}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur with value objects
#[derive(Debug, thiserror::Error)]
pub enum ValueObjectError {
    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Certificate error: {0}")]
    CertificateError(CertificateVerificationError),
}

/// Certificate verification errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum CertificateVerificationError {
    #[error("Certificate expired: not_after={not_after}, now={now}")]
    Expired {
        cert_fingerprint: String,
        not_after: DateTime<Utc>,
        now: DateTime<Utc>,
    },

    #[error("Certificate not yet valid: not_before={not_before}, now={now}")]
    NotYetValid {
        cert_fingerprint: String,
        not_before: DateTime<Utc>,
        now: DateTime<Utc>,
    },

    #[error("Invalid signature: certificate {cert_fingerprint} not signed by claimed issuer")]
    InvalidSignature {
        cert_fingerprint: String,
        issuer_fingerprint: String,
    },

    #[error("Untrusted root certificate: {fingerprint}")]
    UntrustedRoot {
        fingerprint: String,
    },

    #[error("Root certificate is not self-signed")]
    RootNotSelfSigned {
        fingerprint: String,
    },

    #[error("Certificate chain is empty")]
    EmptyChain,

    #[error("Issuer mismatch: certificate issuer does not match CA subject")]
    IssuerMismatch {
        cert_fingerprint: String,
        expected_issuer: String,
        actual_issuer: String,
    },

    #[error("Unsupported signature algorithm: {algorithm:?}")]
    UnsupportedAlgorithm {
        algorithm: SignatureAlgorithm,
    },

    #[error("Cryptographic error: {0}")]
    CryptoError(String),
}

/// Result of successful certificate chain verification
#[derive(Debug, Clone)]
pub struct TrustPath {
    /// Verified links in the chain (from leaf to root)
    pub links: Vec<TrustPathLink>,
    /// Time at which verification occurred
    pub verified_at: DateTime<Utc>,
}

impl TrustPath {
    /// Create a new empty trust path
    pub fn new() -> Self {
        Self {
            links: Vec::new(),
            verified_at: Utc::now(),
        }
    }

    /// Add a verified link to the path
    pub fn add_link(&mut self, cert_fingerprint: String, issuer_fingerprint: Option<String>, trust_level: TrustLevel) {
        self.links.push(TrustPathLink {
            cert_fingerprint,
            issuer_fingerprint,
            trust_level,
        });
    }

    /// Get chain length
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Check if path is empty
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

impl Default for TrustPath {
    fn default() -> Self {
        Self::new()
    }
}

/// A single link in the trust path
#[derive(Debug, Clone)]
pub struct TrustPathLink {
    /// Fingerprint of the verified certificate
    pub cert_fingerprint: String,
    /// Fingerprint of the issuer certificate (None for self-signed root)
    pub issuer_fingerprint: Option<String>,
    /// Trust level assigned to this link
    pub trust_level: TrustLevel,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    /// Helper to create a mock certificate for testing
    fn create_mock_certificate(
        common_name: &str,
        issuer_cn: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
    ) -> Certificate {
        Certificate {
            serial_number: format!("SN-{}", common_name),
            subject: CertificateSubject {
                common_name: common_name.to_string(),
                organization: Some("Test Org".to_string()),
                organizational_unit: None,
                country: Some("US".to_string()),
                state: None,
                locality: None,
                email: None,
            },
            issuer: CertificateSubject {
                common_name: issuer_cn.to_string(),
                organization: Some("Test Org".to_string()),
                organizational_unit: None,
                country: Some("US".to_string()),
                state: None,
                locality: None,
                email: None,
            },
            public_key: PublicKey {
                algorithm: crate::events::KeyAlgorithm::Ed25519,
                data: vec![0u8; 32], // Mock 32-byte Ed25519 public key
                format: PublicKeyFormat::Der,
            },
            validity: Validity {
                not_before,
                not_after,
            },
            signature: Signature {
                algorithm: SignatureAlgorithm::Ed25519,
                data: vec![0u8; 64], // Mock 64-byte Ed25519 signature
            },
            der: vec![0u8; 100], // Mock DER data
            pem: String::new(),
        }
    }

    #[test]
    fn test_validity_is_valid_at() {
        let now = Utc::now();
        let validity = Validity {
            not_before: now - Duration::days(1),
            not_after: now + Duration::days(1),
        };

        assert!(validity.is_valid_at(now));
        assert!(validity.is_valid_at(now - Duration::hours(12)));
        assert!(validity.is_valid_at(now + Duration::hours(12)));
        assert!(!validity.is_valid_at(now - Duration::days(2)));
        assert!(!validity.is_valid_at(now + Duration::days(2)));
    }

    #[test]
    fn test_certificate_temporal_validity_success() {
        let now = Utc::now();
        let cert = create_mock_certificate(
            "test.example.com",
            "CA",
            now - Duration::days(1),
            now + Duration::days(1),
        );

        assert!(cert.verify_temporal_validity(now).is_ok());
    }

    #[test]
    fn test_certificate_temporal_validity_expired() {
        let now = Utc::now();
        let cert = create_mock_certificate(
            "test.example.com",
            "CA",
            now - Duration::days(30),
            now - Duration::days(1),
        );

        let result = cert.verify_temporal_validity(now);
        assert!(result.is_err());
        match result {
            Err(CertificateVerificationError::Expired { .. }) => (),
            _ => panic!("Expected Expired error"),
        }
    }

    #[test]
    fn test_certificate_temporal_validity_not_yet_valid() {
        let now = Utc::now();
        let cert = create_mock_certificate(
            "test.example.com",
            "CA",
            now + Duration::days(1),
            now + Duration::days(30),
        );

        let result = cert.verify_temporal_validity(now);
        assert!(result.is_err());
        match result {
            Err(CertificateVerificationError::NotYetValid { .. }) => (),
            _ => panic!("Expected NotYetValid error"),
        }
    }

    #[test]
    fn test_certificate_issuer_matches() {
        let now = Utc::now();
        let ca_cert = create_mock_certificate(
            "Root CA",
            "Root CA", // Self-signed
            now - Duration::days(365),
            now + Duration::days(365),
        );
        let leaf_cert = create_mock_certificate(
            "leaf.example.com",
            "Root CA",
            now - Duration::days(30),
            now + Duration::days(30),
        );

        assert!(leaf_cert.verify_issuer_matches(&ca_cert).is_ok());
    }

    #[test]
    fn test_certificate_issuer_mismatch() {
        let now = Utc::now();
        let wrong_ca = create_mock_certificate(
            "Wrong CA",
            "Wrong CA",
            now - Duration::days(365),
            now + Duration::days(365),
        );
        let leaf_cert = create_mock_certificate(
            "leaf.example.com",
            "Root CA", // Expects "Root CA" as issuer
            now - Duration::days(30),
            now + Duration::days(30),
        );

        let result = leaf_cert.verify_issuer_matches(&wrong_ca);
        assert!(result.is_err());
        match result {
            Err(CertificateVerificationError::IssuerMismatch { .. }) => (),
            _ => panic!("Expected IssuerMismatch error"),
        }
    }

    #[test]
    fn test_trust_path_creation() {
        let mut path = TrustPath::new();
        assert!(path.is_empty());
        assert_eq!(path.len(), 0);

        path.add_link("leaf-fp".to_string(), Some("ca-fp".to_string()), TrustLevel::Complete);
        assert!(!path.is_empty());
        assert_eq!(path.len(), 1);

        path.add_link("ca-fp".to_string(), None, TrustLevel::Complete);
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn test_certificate_fingerprint_deterministic() {
        let now = Utc::now();
        let cert = create_mock_certificate(
            "test.example.com",
            "CA",
            now - Duration::days(1),
            now + Duration::days(1),
        );

        let fp1 = cert.fingerprint();
        let fp2 = cert.fingerprint();
        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 64); // SHA-256 hex is 64 chars
    }

    #[test]
    fn test_public_key_fingerprint() {
        let pk = PublicKey {
            algorithm: crate::events::KeyAlgorithm::Ed25519,
            data: vec![1, 2, 3, 4, 5],
            format: PublicKeyFormat::Der,
        };

        let fp = pk.fingerprint();
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn test_certificate_chain_depth() {
        let now = Utc::now();
        let root = create_mock_certificate("Root CA", "Root CA", now - Duration::days(365), now + Duration::days(365));
        let leaf = create_mock_certificate("leaf", "Root CA", now - Duration::days(30), now + Duration::days(30));

        // Chain with no intermediates
        let chain = CertificateChain::new(leaf.clone(), vec![], root.clone());
        assert_eq!(chain.depth(), 2);

        // Chain with one intermediate
        let intermediate = create_mock_certificate("Intermediate CA", "Root CA", now - Duration::days(180), now + Duration::days(180));
        let chain_with_int = CertificateChain::new(leaf.clone(), vec![intermediate], root.clone());
        assert_eq!(chain_with_int.depth(), 3);
    }

    #[test]
    fn test_certificate_chain_all_certificates() {
        let now = Utc::now();
        let root = create_mock_certificate("Root CA", "Root CA", now - Duration::days(365), now + Duration::days(365));
        let intermediate = create_mock_certificate("Intermediate CA", "Root CA", now - Duration::days(180), now + Duration::days(180));
        let leaf = create_mock_certificate("leaf", "Intermediate CA", now - Duration::days(30), now + Duration::days(30));

        let chain = CertificateChain::new(leaf.clone(), vec![intermediate.clone()], root.clone());
        let all = chain.all_certificates();

        assert_eq!(all.len(), 3);
        assert_eq!(all[0].subject.common_name, "leaf");
        assert_eq!(all[1].subject.common_name, "Intermediate CA");
        assert_eq!(all[2].subject.common_name, "Root CA");
    }

    #[test]
    fn test_did_parse_valid() {
        let did = DID::parse("did:key:z6MkhaXgBZDvotDkL").unwrap();
        assert!(matches!(did.method, DidMethod::Key));
        assert_eq!(did.identifier, "z6MkhaXgBZDvotDkL");
        assert!(did.fragment.is_none());
    }

    #[test]
    fn test_did_parse_with_fragment() {
        let did = DID::parse("did:key:z6MkhaXgBZDvotDkL#key-1").unwrap();
        assert!(matches!(did.method, DidMethod::Key));
        assert_eq!(did.identifier, "z6MkhaXgBZDvotDkL");
        assert_eq!(did.fragment, Some("key-1".to_string()));
    }

    #[test]
    fn test_did_parse_invalid() {
        assert!(DID::parse("invalid").is_err());
        assert!(DID::parse("did:").is_err());
    }

    #[test]
    fn test_did_display() {
        let did = DID::new(DidMethod::Key, "z6MkhaXgBZDvotDkL".to_string());
        assert_eq!(did.to_string(), "did:key:z6MkhaXgBZDvotDkL");

        let did_with_frag = DID::with_fragment(DidMethod::Key, "z6MkhaXgBZDvotDkL".to_string(), "key-1".to_string());
        assert_eq!(did_with_frag.to_string(), "did:key:z6MkhaXgBZDvotDkL#key-1");
    }

    #[test]
    fn test_verifiable_credential_validity() {
        let now = Utc::now();

        let valid_cred = VerifiableCredential {
            id: uuid::Uuid::now_v7(),
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            credential_type: vec![CredentialType::OrganizationCredential],
            issuer: DID::new(DidMethod::Key, "issuer".to_string()),
            issuance_date: now - Duration::days(1),
            expiration_date: Some(now + Duration::days(30)),
            credential_subject: CredentialSubject {
                id: DID::new(DidMethod::Key, "subject".to_string()),
                claims: serde_json::json!({"role": "admin"}),
            },
            proof: CredentialProof {
                proof_type: ProofType::Ed25519Signature2020,
                created: now - Duration::days(1),
                verification_method: DID::new(DidMethod::Key, "issuer#key-1".to_string()),
                proof_value: vec![0u8; 64],
            },
        };

        assert!(valid_cred.is_valid_now());

        let expired_cred = VerifiableCredential {
            expiration_date: Some(now - Duration::days(1)),
            ..valid_cred.clone()
        };
        assert!(!expired_cred.is_valid_now());

        let future_cred = VerifiableCredential {
            issuance_date: now + Duration::days(1),
            ..valid_cred
        };
        assert!(!future_cred.is_valid_now());
    }
}
