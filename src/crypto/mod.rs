//! Cryptographic primitives for CIM-Keys
//!
//! This module provides the cryptographic operations for deterministic
//! key generation from a single master passphrase.
//!
//! ## Architecture
//!
//! ```text
//! Passphrase
//!   ↓ Argon2id (memory-hard KDF)
//! Master Seed (256-bit)
//!   ↓ HKDF-SHA256 (hierarchical derivation)
//! ├─ Root CA Seed
//! ├─ Intermediate CA Seeds (one per OU)
//! ├─ User Key Seeds (one per person)
//! └─ NATS Credential Seeds
//! ```
//!
//! ## Security Properties
//!
//! - **Deterministic**: Same passphrase always produces same keys
//! - **Memory-Hard**: Argon2id resists GPU/ASIC attacks
//! - **Hierarchical**: HKDF provides proper key separation
//! - **Offline**: All operations air-gapped

pub mod seed_derivation;
pub mod key_generation;
pub mod passphrase;
pub mod x509;
pub mod rfc5280;

pub use seed_derivation::{MasterSeed, derive_master_seed, derive_child_seed};
pub use key_generation::{KeyPair, generate_keypair_from_seed};
pub use passphrase::{PassphraseStrength, validate_passphrase};
pub use x509::{
    X509Certificate, RootCAParams, IntermediateCAParams, ServerCertParams,
    generate_root_ca, generate_intermediate_ca, generate_server_certificate,
};
pub use rfc5280::{
    validate_certificate, validate_certificate_der,
    Rfc5280ValidationResult, Rfc5280Error, CertificateMetadata,
};
