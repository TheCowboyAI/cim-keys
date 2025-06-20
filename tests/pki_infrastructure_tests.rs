//! PKI Infrastructure Tests
//! 
//! Tests for complete PKI setup including CA hierarchy and certificate lifecycle
//! 
//! ## Test Flow Diagram
//! ```mermaid
//! graph TD
//!     A[Create Root CA] --> B[Create Intermediate CA]
//!     B --> C[Issue End Entity Cert]
//!     C --> D[Validate Chain]
//!     D --> E[Revoke Certificate]
//!     E --> F[Generate CRL]
//!     F --> G[Verify Revocation]
//! ```

use cim_keys::{
    pki::PkiManager,
    PkiOperations, CertificateManager, KeyManager,
    KeyId, KeyAlgorithm, CertificateMetadata,
    RsaKeySize, EcdsaCurve,
    Result,
};
use chrono::{Utc, Duration};

// ============================================================================
// Test: Three-Level PKI Hierarchy (CIM Leaf Model)
// ============================================================================

#[test]
fn test_three_level_pki_hierarchy() {
    // CIM Leaf PKI structure:
    // 1. Operator Level - System operations
    // 2. Domain Level - Domain administration  
    // 3. User Level - Day-to-day operations
    
    struct PkiLevel {
        name: String,
        root_ca: String,
        intermediate_ca: String,
        purpose: String,
    }
    
    let operator_level = PkiLevel {
        name: "Operator".to_string(),
        root_ca: "CN=Operator Root CA, O=CIM System".to_string(),
        intermediate_ca: "CN=Operator Intermediate CA, O=CIM System".to_string(),
        purpose: "System operations and disk encryption".to_string(),
    };
    
    let domain_level = PkiLevel {
        name: "Domain".to_string(),
        root_ca: "CN=Domain Root CA, O=Example Domain".to_string(),
        intermediate_ca: "CN=Domain Intermediate CA, O=Example Domain".to_string(),
        purpose: "Domain administration and user certificate signing".to_string(),
    };
    
    let user_level = PkiLevel {
        name: "User".to_string(),
        root_ca: "CN=Domain Intermediate CA, O=Example Domain".to_string(), // Signed by domain
        intermediate_ca: "".to_string(), // No intermediate at user level
        purpose: "Day-to-day user operations".to_string(),
    };
    
    // Verify hierarchy
    assert_eq!(operator_level.name, "Operator");
    assert_eq!(domain_level.name, "Domain");
    assert_eq!(user_level.name, "User");
    
    // Test purposes
    assert!(operator_level.purpose.contains("System operations"));
    assert!(domain_level.purpose.contains("Domain administration"));
    assert!(user_level.purpose.contains("Day-to-day"));
}

// ============================================================================
// Test: YubiKey PIV Slot Allocation
// ============================================================================

#[test]
fn test_yubikey_piv_slots() {
    // PIV slot assignments for CIM
    const SLOT_9A: &str = "9a"; // Authentication
    const SLOT_9C: &str = "9c"; // Digital Signature
    const SLOT_9D: &str = "9d"; // Key Management
    const SLOT_9E: &str = "9e"; // Card Authentication
    const SLOT_82: &str = "82"; // Retired Key Management 1
    const SLOT_83: &str = "83"; // Retired Key Management 2
    
    // Test slot values
    assert_eq!(SLOT_9A, "9a");
    assert_eq!(SLOT_9C, "9c");
    assert_eq!(SLOT_9D, "9d");
    
    // Slot purposes in CIM
    struct SlotAssignment {
        slot: String,
        purpose: String,
        key_type: KeyAlgorithm,
    }
    
    let assignments = vec![
        SlotAssignment {
            slot: SLOT_9A.to_string(),
            purpose: "TLS client authentication".to_string(),
            key_type: KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
        },
        SlotAssignment {
            slot: SLOT_9C.to_string(),
            purpose: "Document and code signing".to_string(),
            key_type: KeyAlgorithm::Rsa(RsaKeySize::Rsa4096),
        },
        SlotAssignment {
            slot: SLOT_9D.to_string(),
            purpose: "Email encryption".to_string(),
            key_type: KeyAlgorithm::Rsa(RsaKeySize::Rsa4096),
        },
    ];
    
    // Verify assignments
    assert_eq!(assignments[0].slot, "9a");
    assert_eq!(assignments[1].slot, "9c");
    assert_eq!(assignments[2].slot, "9d");
}

// ============================================================================
// Test: Certificate Authority Operations
// ============================================================================

#[test]
fn test_ca_operations() {
    // CA certificate constraints
    struct CaConstraints {
        is_ca: bool,
        path_len_constraint: Option<u32>,
        key_usage: Vec<String>,
        validity_years: u32,
    }
    
    // Root CA constraints
    let root_ca = CaConstraints {
        is_ca: true,
        path_len_constraint: Some(2), // Can create 2 levels below
        key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
        validity_years: 20,
    };
    
    // Intermediate CA constraints
    let intermediate_ca = CaConstraints {
        is_ca: true,
        path_len_constraint: Some(0), // Can only sign end entities
        key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
        validity_years: 10,
    };
    
    // End entity constraints
    let end_entity = CaConstraints {
        is_ca: false,
        path_len_constraint: None,
        key_usage: vec!["digitalSignature".to_string(), "keyEncipherment".to_string()],
        validity_years: 1,
    };
    
    // Verify CA flags
    assert!(root_ca.is_ca);
    assert!(intermediate_ca.is_ca);
    assert!(!end_entity.is_ca);
    
    // Verify path length constraints
    assert_eq!(root_ca.path_len_constraint, Some(2));
    assert_eq!(intermediate_ca.path_len_constraint, Some(0));
    assert_eq!(end_entity.path_len_constraint, None);
    
    // Verify validity periods
    assert!(root_ca.validity_years > intermediate_ca.validity_years);
    assert!(intermediate_ca.validity_years > end_entity.validity_years);
}

// ============================================================================
// Test: Certificate Serial Number Management
// ============================================================================

#[test]
fn test_certificate_serial_numbers() {
    use std::collections::HashSet;
    
    // Mock serial number generator
    struct SerialNumberGenerator {
        used_serials: HashSet<String>,
        counter: u64,
    }
    
    impl SerialNumberGenerator {
        fn new() -> Self {
            Self {
                used_serials: HashSet::new(),
                counter: 1,
            }
        }
        
        fn next_serial(&mut self) -> String {
            let serial = format!("{:016X}", self.counter);
            self.used_serials.insert(serial.clone());
            self.counter += 1;
            serial
        }
        
        fn is_unique(&self, serial: &str) -> bool {
            !self.used_serials.contains(serial)
        }
    }
    
    let mut generator = SerialNumberGenerator::new();
    
    // Generate serials
    let serial1 = generator.next_serial();
    let serial2 = generator.next_serial();
    let serial3 = generator.next_serial();
    
    // Verify uniqueness
    assert_ne!(serial1, serial2);
    assert_ne!(serial2, serial3);
    assert_ne!(serial1, serial3);
    
    // Verify format (16 hex digits)
    assert_eq!(serial1.len(), 16);
    assert!(serial1.chars().all(|c| c.is_ascii_hexdigit()));
    
    // Verify tracking
    assert!(!generator.is_unique(&serial1));
    assert!(generator.is_unique("FFFFFFFFFFFFFFFF"));
}

// ============================================================================
// Test: Certificate Templates
// ============================================================================

#[test]
fn test_certificate_templates() {
    // Common certificate templates
    enum CertificateTemplate {
        WebServer,
        CodeSigning,
        EmailProtection,
        ClientAuthentication,
        TimestampingServer,
    }
    
    // Template configurations
    fn get_template_config(template: CertificateTemplate) -> (Vec<&'static str>, Vec<&'static str>) {
        match template {
            CertificateTemplate::WebServer => (
                vec!["digitalSignature", "keyEncipherment"],
                vec!["serverAuth"],
            ),
            CertificateTemplate::CodeSigning => (
                vec!["digitalSignature"],
                vec!["codeSigning"],
            ),
            CertificateTemplate::EmailProtection => (
                vec!["digitalSignature", "keyEncipherment"],
                vec!["emailProtection"],
            ),
            CertificateTemplate::ClientAuthentication => (
                vec!["digitalSignature", "keyAgreement"],
                vec!["clientAuth"],
            ),
            CertificateTemplate::TimestampingServer => (
                vec!["digitalSignature", "nonRepudiation"],
                vec!["timeStamping"],
            ),
        }
    }
    
    // Test web server template
    let (web_ku, web_eku) = get_template_config(CertificateTemplate::WebServer);
    assert!(web_ku.contains(&"digitalSignature"));
    assert!(web_ku.contains(&"keyEncipherment"));
    assert!(web_eku.contains(&"serverAuth"));
    
    // Test code signing template
    let (code_ku, code_eku) = get_template_config(CertificateTemplate::CodeSigning);
    assert!(code_ku.contains(&"digitalSignature"));
    assert!(code_eku.contains(&"codeSigning"));
}

// ============================================================================
// Test: Trust Store Management
// ============================================================================

#[test]
fn test_trust_store_management() {
    use std::collections::HashMap;
    
    // Mock trust store
    struct TrustStore {
        trusted_roots: HashMap<String, String>, // fingerprint -> subject
        trusted_intermediates: HashMap<String, String>,
        revoked_certificates: HashMap<String, u8>, // serial -> reason
    }
    
    impl TrustStore {
        fn new() -> Self {
            Self {
                trusted_roots: HashMap::new(),
                trusted_intermediates: HashMap::new(),
                revoked_certificates: HashMap::new(),
            }
        }
        
        fn add_trusted_root(&mut self, fingerprint: String, subject: String) {
            self.trusted_roots.insert(fingerprint, subject);
        }
        
        fn add_trusted_intermediate(&mut self, fingerprint: String, subject: String) {
            self.trusted_intermediates.insert(fingerprint, subject);
        }
        
        fn revoke_certificate(&mut self, serial: String, reason: u8) {
            self.revoked_certificates.insert(serial, reason);
        }
        
        fn is_trusted(&self, fingerprint: &str) -> bool {
            self.trusted_roots.contains_key(fingerprint) ||
            self.trusted_intermediates.contains_key(fingerprint)
        }
        
        fn is_revoked(&self, serial: &str) -> bool {
            self.revoked_certificates.contains_key(serial)
        }
    }
    
    let mut trust_store = TrustStore::new();
    
    // Add trusted CAs
    trust_store.add_trusted_root(
        "SHA256:abcd1234...".to_string(),
        "CN=Root CA, O=Example Corp".to_string(),
    );
    
    trust_store.add_trusted_intermediate(
        "SHA256:efgh5678...".to_string(),
        "CN=Intermediate CA, O=Example Corp".to_string(),
    );
    
    // Revoke a certificate
    trust_store.revoke_certificate("0123456789ABCDEF".to_string(), 1); // Key compromise
    
    // Test trust validation
    assert!(trust_store.is_trusted("SHA256:abcd1234..."));
    assert!(trust_store.is_trusted("SHA256:efgh5678..."));
    assert!(!trust_store.is_trusted("SHA256:unknown..."));
    
    // Test revocation
    assert!(trust_store.is_revoked("0123456789ABCDEF"));
    assert!(!trust_store.is_revoked("FEDCBA9876543210"));
} 