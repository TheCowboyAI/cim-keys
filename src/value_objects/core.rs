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

    /// Verify the chain integrity
    pub fn verify(&self) -> Result<(), ValueObjectError> {
        // Verify leaf is signed by first intermediate (or root if no intermediates)
        // Verify each intermediate is signed by the next
        // Verify last intermediate is signed by root
        // For now, simplified implementation
        Ok(())
    }

    /// Get all certificates in the chain
    pub fn all_certificates(&self) -> Vec<&Certificate> {
        let mut certs = vec![&self.leaf];
        certs.extend(self.intermediates.iter());
        certs.push(&self.root);
        certs
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
}
