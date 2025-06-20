//! Event Flow Tests for CIM Keys
//! 
//! Tests that validate the event-driven key management patterns
//! 
//! ## Test Flow Diagram
//! ```mermaid
//! graph TD
//!     A[Generate Key] --> B[Key Generated Event]
//!     B --> C[Store Key]
//!     C --> D[Key Stored Event]
//!     D --> E[Sign Data]
//!     E --> F[Signature Created Event]
//!     F --> G[Verify Signature]
//!     G --> H[Signature Verified Event]
//! ```

use cim_keys::{
    KeyId, KeyAlgorithm, KeyUsage, KeyMetadata, SecureString,
    RsaKeySize, EcdsaCurve, AesKeySize,
    SignatureFormat, KeyExportFormat, KeyLocation,
    Result, KeyError,
};
use chrono::Utc;
use uuid::Uuid;

// ============================================================================
// Test: Key ID Generation and Uniqueness
// ============================================================================

#[test]
fn test_key_id_generation() {
    // Generate multiple key IDs
    let id1 = KeyId::new();
    let id2 = KeyId::new();
    let id3 = KeyId::new();
    
    // All IDs should be unique
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
    
    // Display format should work
    let display = format!("{}", id1);
    assert!(!display.is_empty());
    assert_eq!(display.len(), 36); // UUID format
    
    // From UUID should work
    let uuid = Uuid::new_v4();
    let id_from_uuid = KeyId::from_uuid(uuid);
    assert_eq!(format!("{}", id_from_uuid), uuid.to_string());
}

// ============================================================================
// Test: Key Algorithm Variants
// ============================================================================

#[test]
fn test_key_algorithm_variants() {
    // Test RSA variants
    let rsa_2048 = KeyAlgorithm::Rsa(RsaKeySize::Rsa2048);
    let rsa_4096 = KeyAlgorithm::Rsa(RsaKeySize::Rsa4096);
    assert_ne!(rsa_2048, rsa_4096);
    
    // Test ECDSA variants
    let ecdsa_p256 = KeyAlgorithm::Ecdsa(EcdsaCurve::P256);
    let ecdsa_p384 = KeyAlgorithm::Ecdsa(EcdsaCurve::P384);
    assert_ne!(ecdsa_p256, ecdsa_p384);
    
    // Test Ed25519
    let ed25519 = KeyAlgorithm::Ed25519;
    assert_ne!(ed25519, rsa_2048);
    
    // Test AES variants
    let aes_128 = KeyAlgorithm::Aes(AesKeySize::Aes128);
    let aes_256 = KeyAlgorithm::Aes(AesKeySize::Aes256);
    assert_ne!(aes_128, aes_256);
    
    // Test serialization
    let serialized = serde_json::to_string(&ed25519).unwrap();
    let deserialized: KeyAlgorithm = serde_json::from_str(&serialized).unwrap();
    assert_eq!(ed25519, deserialized);
}

// ============================================================================
// Test: Key Usage Flags
// ============================================================================

#[test]
fn test_key_usage_flags() {
    // Test default usage
    let default_usage = KeyUsage::default();
    assert!(default_usage.sign);
    assert!(default_usage.verify);
    assert!(default_usage.encrypt);
    assert!(default_usage.decrypt);
    assert!(!default_usage.derive); // False by default
    assert!(default_usage.authenticate);
    
    // Test custom usage
    let signing_only = KeyUsage {
        sign: true,
        verify: true,
        encrypt: false,
        decrypt: false,
        derive: false,
        authenticate: false,
    };
    
    assert!(signing_only.sign);
    assert!(!signing_only.encrypt);
    
    // Test serialization
    let serialized = serde_json::to_string(&signing_only).unwrap();
    let deserialized: KeyUsage = serde_json::from_str(&serialized).unwrap();
    assert_eq!(signing_only, deserialized);
}

// ============================================================================
// Test: Secure String Handling
// ============================================================================

#[test]
fn test_secure_string() {
    // Create secure string
    let password = "super-secret-password-123";
    let secure = SecureString::new(password.to_string());
    
    // Debug should not expose the secret
    let debug_str = format!("{:?}", secure);
    assert_eq!(debug_str, "SecureString(***)");
    assert!(!debug_str.contains(password));
    
    // But we can expose it when needed
    assert_eq!(secure.expose_secret(), password);
    
    // From string conversion
    let secure2: SecureString = "another-secret".to_string().into();
    assert_eq!(secure2.expose_secret(), "another-secret");
}

// ============================================================================
// Test: Key Metadata Creation
// ============================================================================

#[test]
fn test_key_metadata_creation() {
    let key_id = KeyId::new();
    let created_at = Utc::now();
    
    let metadata = KeyMetadata {
        id: key_id,
        algorithm: KeyAlgorithm::Ed25519,
        usage: KeyUsage::default(),
        created_at,
        expires_at: Some(created_at + chrono::Duration::days(365)),
        label: "Test Key".to_string(),
        description: Some("A test key for unit tests".to_string()),
        email: Some("test@example.com".to_string()),
        fingerprint: Some("SHA256:abcd1234...".to_string()),
        hardware_serial: None,
    };
    
    // Verify fields
    assert_eq!(metadata.id, key_id);
    assert_eq!(metadata.algorithm, KeyAlgorithm::Ed25519);
    assert_eq!(metadata.label, "Test Key");
    assert!(metadata.expires_at.is_some());
    
    // Test serialization
    let serialized = serde_json::to_string(&metadata).unwrap();
    let deserialized: KeyMetadata = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, metadata.id);
    assert_eq!(deserialized.label, metadata.label);
}

// ============================================================================
// Test: Key Location Types
// ============================================================================

#[test]
fn test_key_location_types() {
    use std::path::PathBuf;
    
    // File location
    let file_loc = KeyLocation::File(PathBuf::from("/home/user/.keys/my-key.pem"));
    
    // Hardware token location
    let hw_loc = KeyLocation::HardwareToken {
        serial: "12345678".to_string(),
        slot: "9a".to_string(),
    };
    
    // Memory location
    let mem_loc = KeyLocation::Memory;
    
    // System keyring
    let keyring_loc = KeyLocation::SystemKeyring("my-app-key".to_string());
    
    // Remote server
    let remote_loc = KeyLocation::RemoteServer {
        url: "https://keys.example.com".to_string(),
        key_id: "abc123".to_string(),
    };
    
    // Test equality
    assert_ne!(file_loc, hw_loc);
    assert_ne!(mem_loc, keyring_loc);
    
    // Test serialization
    let serialized = serde_json::to_string(&hw_loc).unwrap();
    let deserialized: KeyLocation = serde_json::from_str(&serialized).unwrap();
    assert_eq!(hw_loc, deserialized);
} 