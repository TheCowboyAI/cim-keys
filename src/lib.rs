//! # CIM Keys - Cryptographic Key Management and PKI Support
//!
//! This crate provides comprehensive cryptographic key management for the CIM architecture,
//! including YubiKey hardware token support, PKI infrastructure, and various cryptographic
//! operations.
//!
//! ## Features
//!
//! - **YubiKey Support**: Hardware token integration for secure key storage
//! - **GPG Integration**: OpenPGP operations and key management
//! - **SSH Keys**: SSH key generation, management, and authentication
//! - **X.509/TLS**: Certificate generation, validation, and TLS operations
//! - **PKI Infrastructure**: Complete PKI setup with CA, intermediate, and end-entity certificates
//! - **Encryption/Decryption**: Symmetric and asymmetric encryption operations
//! - **Digital Signatures**: Sign and verify data with various algorithms
//!
//! ## Architecture
//!
//! The crate is organized into several modules:
//! - `yubikey`: YubiKey hardware token operations
//! - `gpg`: GPG/OpenPGP functionality
//! - `ssh`: SSH key management
//! - `tls`: TLS/X.509 certificate operations
//! - `pki`: PKI infrastructure and CA operations
//! - `storage`: Secure key storage and retrieval

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod error;
pub mod types;
pub mod traits;

#[cfg(feature = "yubikey-support")]
pub mod yubikey;

#[cfg(feature = "gpg-support")]
pub mod gpg;

pub mod ssh;
pub mod tls;
pub mod pki;
pub mod storage;

// Re-export commonly used types
pub use error::{KeyError, Result};
pub use types::*;
pub use traits::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{KeyError, Result};
    pub use crate::traits::*;
    pub use crate::types::*;

    #[cfg(feature = "yubikey-support")]
    pub use crate::yubikey::YubiKeyManager;

    pub use crate::ssh::SshKeyManager;
    pub use crate::tls::TlsManager;
    pub use crate::pki::PkiManager;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_id_creation() {
        let id1 = KeyId::new();
        let id2 = KeyId::new();
        
        // Each KeyId should be unique
        assert_ne!(id1, id2);
        
        // Display should work
        let display = format!("{id1}");
        assert!(!display.is_empty());
    }

    #[test]
    fn test_key_usage_default() {
        let usage = KeyUsage::default();
        
        // Default should enable most operations
        assert!(usage.sign);
        assert!(usage.verify);
        assert!(usage.encrypt);
        assert!(usage.decrypt);
        assert!(!usage.derive); // Derive is false by default
        assert!(usage.authenticate);
    }

    #[test]
    fn test_secure_string() {
        let secret = SecureString::new("my-secret-password".to_string());
        
        // Debug should not expose the secret
        let debug_str = format!("{:?}", secret);
        assert_eq!(debug_str, "SecureString(***)");
        
        // But we can expose it when needed
        assert_eq!(secret.expose_secret(), "my-secret-password");
    }

    #[test]
    fn test_key_algorithm_variants() {
        // Test that all algorithm variants are accessible
        let _rsa = KeyAlgorithm::Rsa(RsaKeySize::Rsa2048);
        let _ed25519 = KeyAlgorithm::Ed25519;
        let _ecdsa = KeyAlgorithm::Ecdsa(EcdsaCurve::P256);
        let _aes = KeyAlgorithm::Aes(AesKeySize::Aes256);
    }
}
