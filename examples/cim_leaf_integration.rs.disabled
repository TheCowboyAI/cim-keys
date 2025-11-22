//! CIM Leaf Integration Example
//!
//! This example demonstrates how cim-keys integrates with the CIM Leaf
//! three-level PKI infrastructure using YubiKeys.

use cim_keys::{
    KeyManager, Signer, Encryptor, CertificateManager, PkiOperations,
    KeyAlgorithm, KeyUsage, SignatureFormat, EncryptionFormat,
    RsaKeySize, EcdsaCurve, KeyExportFormat, CertificateFormat,
    yubikey::{YubiKeyManager, PivSlot, TouchPolicy, PinPolicy},
    pki::PkiManager,
    ssh::SshKeyManager,
    tls::TlsManager,
    storage::FileKeyStorage,
};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber;

/// CIM Leaf PKI hierarchy levels
#[derive(Debug, Clone)]
enum PkiLevel {
    /// Operator level - system operations and disk encryption
    Operator,
    /// Domain level - domain administration and root CA
    Domain,
    /// User level - day-to-day operations
    User,
}

/// Configuration for a CIM Leaf YubiKey
struct LeafYubiKeyConfig {
    serial: String,
    level: PkiLevel,
    pin: String,
    puk: String,
    management_key: Vec<u8>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("Starting CIM Leaf YubiKey integration example");
    
    // Example 1: Initialize PKI Infrastructure
    info!("\n=== Setting up CIM Leaf PKI Infrastructure ===");
    let pki_manager = setup_pki_infrastructure().await?;
    
    // Example 2: Configure Operator YubiKey
    info!("\n=== Configuring Operator YubiKey ===");
    let operator_config = LeafYubiKeyConfig {
        serial: "15000433".to_string(),
        level: PkiLevel::Operator,
        pin: "123456".to_string(), // In production, load from secure storage
        puk: "12345678".to_string(),
        management_key: hex::decode("010203040506070801020304050607080102030405060708")?,
    };
    let operator_yubikey = setup_yubikey(&operator_config, &pki_manager).await?;
    
    // Example 3: Configure Domain YubiKey
    info!("\n=== Configuring Domain YubiKey ===");
    let domain_config = LeafYubiKeyConfig {
        serial: "15000434".to_string(),
        level: PkiLevel::Domain,
        pin: "123456".to_string(),
        puk: "12345678".to_string(),
        management_key: hex::decode("010203040506070801020304050607080102030405060708")?,
    };
    let domain_yubikey = setup_yubikey(&domain_config, &pki_manager).await?;
    
    // Example 4: Configure User YubiKey
    info!("\n=== Configuring User YubiKey ===");
    let user_config = LeafYubiKeyConfig {
        serial: "11059708".to_string(),
        level: PkiLevel::User,
        pin: "123456".to_string(),
        puk: "12345678".to_string(),
        management_key: hex::decode("010203040506070801020304050607080102030405060708")?,
    };
    let user_yubikey = setup_yubikey(&user_config, &pki_manager).await?;
    
    // Example 5: NATS Certificate Generation
    info!("\n=== Generating NATS Certificates ===");
    generate_nats_certificates(&domain_yubikey).await?;
    
    // Example 6: Container Certificate Management
    info!("\n=== Container Certificate Management ===");
    manage_container_certificates(&domain_yubikey).await?;
    
    // Example 7: SSH Key Setup with FIDO2
    info!("\n=== SSH Key Setup with FIDO2 ===");
    setup_ssh_keys(&user_yubikey).await?;
    
    // Example 8: OATH TOTP Configuration
    info!("\n=== OATH TOTP Configuration ===");
    configure_oath_totp(&user_yubikey).await?;
    
    info!("\nCIM Leaf integration example completed successfully!");
    
    Ok(())
}

/// Set up the three-level PKI infrastructure
async fn setup_pki_infrastructure() -> Result<PkiManager, Box<dyn std::error::Error>> {
    let pki_manager = PkiManager::new();
    
    // Create Operator Root CA
    info!("Creating Operator Root CA...");
    let (operator_root_key, operator_root_cert) = pki_manager.create_root_ca(
        "CN=CIM Operator Root CA,O=Cowboy AI LLC,C=US",
        KeyAlgorithm::Rsa(RsaKeySize::Rsa4096),
        10, // 10 years validity
    ).await?;
    
    // Create Operator Intermediate CA
    info!("Creating Operator Intermediate CA...");
    let (operator_int_key, operator_int_cert) = pki_manager.create_intermediate_ca(
        &operator_root_cert,
        "CN=CIM Operator Intermediate CA,O=Cowboy AI LLC,C=US",
        KeyAlgorithm::Rsa(RsaKeySize::Rsa4096),
        5, // 5 years validity
    ).await?;
    
    // Create Domain Root CA
    info!("Creating Domain Root CA...");
    let (domain_root_key, domain_root_cert) = pki_manager.create_root_ca(
        "CN=CIM Domain Root CA,O=Cowboy AI LLC,C=US",
        KeyAlgorithm::Rsa(RsaKeySize::Rsa4096),
        10,
    ).await?;
    
    // Create Domain Intermediate CA
    info!("Creating Domain Intermediate CA...");
    let (domain_int_key, domain_int_cert) = pki_manager.create_intermediate_ca(
        &domain_root_cert,
        "CN=CIM Domain Intermediate CA,O=Cowboy AI LLC,C=US",
        KeyAlgorithm::Rsa(RsaKeySize::Rsa4096),
        5,
    ).await?;
    
    // Add trusted roots
    pki_manager.add_trusted_root(
        operator_root_cert.clone(),
        operator_root_cert.metadata,
    ).await?;
    
    pki_manager.add_trusted_root(
        domain_root_cert.clone(),
        domain_root_cert.metadata,
    ).await?;
    
    info!("PKI infrastructure setup complete");
    Ok(pki_manager)
}

/// Configure a YubiKey according to CIM Leaf specifications
async fn setup_yubikey(
    config: &LeafYubiKeyConfig,
    pki_manager: &PkiManager,
) -> Result<YubiKeyManager, Box<dyn std::error::Error>> {
    let mut yubikey = YubiKeyManager::connect_by_serial(&config.serial)?;
    
    // Set PIN and PUK
    yubikey.change_pin("123456", &config.pin)?;
    yubikey.change_puk("12345678", &config.puk)?;
    yubikey.set_management_key(&config.management_key)?;
    
    // Configure PIV slots based on level
    match config.level {
        PkiLevel::Operator => {
            info!("Configuring Operator YubiKey PIV slots...");
            
            // Slot 9A: Authentication
            let auth_key = yubikey.generate_key(
                PivSlot::Authentication,
                KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
                PinPolicy::Once,
                TouchPolicy::Never,
            ).await?;
            
            // Generate certificate
            let auth_cert = pki_manager.issue_certificate(
                &operator_int_key,
                &auth_key.public_key_der()?,
                365, // 1 year
                false,
                None,
            ).await?;
            
            yubikey.import_certificate(
                PivSlot::Authentication,
                &auth_cert,
                CertificateFormat::Der,
            ).await?;
            
            // Slot 9C: Digital Signature (touch required)
            let sign_key = yubikey.generate_key(
                PivSlot::Signature,
                KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
                PinPolicy::Always,
                TouchPolicy::Always,
            ).await?;
            
            // Slot 9D: Key Management
            let encrypt_key = yubikey.generate_key(
                PivSlot::KeyManagement,
                KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
                PinPolicy::Once,
                TouchPolicy::Never,
            ).await?;
        }
        PkiLevel::Domain => {
            info!("Configuring Domain YubiKey PIV slots...");
            // Similar configuration for domain level
        }
        PkiLevel::User => {
            info!("Configuring User YubiKey PIV slots...");
            // Similar configuration for user level
        }
    }
    
    // Configure OpenPGP applet
    info!("Configuring OpenPGP applet...");
    yubikey.configure_openpgp(
        &config.pin,
        "12345678", // Admin PIN
        TouchPolicy::Always, // Signature requires touch
        TouchPolicy::Cached, // Encryption caches touch
        TouchPolicy::Cached, // Authentication caches touch
    ).await?;
    
    // Configure FIDO2 for SSH
    info!("Configuring FIDO2 for SSH...");
    yubikey.configure_fido2(
        Some(&config.pin),
        true, // Require user presence
    ).await?;
    
    Ok(yubikey)
}

/// Generate NATS certificates for secure messaging
async fn generate_nats_certificates(
    yubikey: &YubiKeyManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let tls_manager = TlsManager::new();
    
    // Generate NATS server certificate
    info!("Generating NATS server certificate...");
    let (server_key, server_cert) = tls_manager.generate_self_signed(
        "nats-server.cim.local",
        vec![
            "nats-server.cim.local".to_string(),
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ],
        KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
        365,
    ).await?;
    
    // Generate NATS leaf node certificate
    info!("Generating NATS leaf node certificate...");
    let (leaf_key, leaf_cert) = tls_manager.generate_self_signed(
        "leaf-node.cim.local",
        vec!["leaf-node.cim.local".to_string()],
        KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
        365,
    ).await?;
    
    // Store certificates
    let storage = FileKeyStorage::new("./secrets/generated/nats").await?;
    storage.store_key(
        &server_key,
        &server_cert.to_der()?,
        server_cert.metadata,
        cim_keys::KeyLocation::File(PathBuf::from("server.crt")),
    ).await?;
    
    info!("NATS certificates generated and stored");
    Ok(())
}

/// Manage certificates for containers
async fn manage_container_certificates(
    yubikey: &YubiKeyManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // This would integrate with the container management system
    // to issue certificates for each container group
    
    info!("Container certificate management configured");
    Ok(())
}

/// Set up SSH keys with FIDO2
async fn setup_ssh_keys(
    yubikey: &YubiKeyManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let ssh_manager = SshKeyManager::new();
    
    // Generate FIDO2 SSH key
    info!("Generating FIDO2 SSH key...");
    let ssh_key = yubikey.generate_fido2_ssh_key(
        "steele@thecowboy.ai",
        true, // Resident key
    ).await?;
    
    // Export public key
    let public_key = ssh_manager.export_key(
        &ssh_key,
        KeyExportFormat::OpenSsh,
        false,
    ).await?;
    
    info!("SSH public key:\n{}", String::from_utf8_lossy(&public_key));
    
    // Store in authorized_keys format
    let storage = FileKeyStorage::new("./secrets/generated/ssh").await?;
    storage.store_key(
        &ssh_key,
        &public_key,
        ssh_key.metadata,
        cim_keys::KeyLocation::File(PathBuf::from("id_ed25519_sk.pub")),
    ).await?;
    
    Ok(())
}

/// Configure OATH TOTP
async fn configure_oath_totp(
    yubikey: &YubiKeyManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Configuring OATH TOTP...");
    
    // Add TOTP credential
    yubikey.add_oath_credential(
        "example:steele@thecowboy.ai",
        "JBSWY3DPEHPK3PXP", // Example secret
        cim_keys::OathType::Totp,
        cim_keys::OathAlgorithm::Sha1,
        6, // digits
        30, // period
    ).await?;
    
    info!("OATH TOTP configured");
    Ok(())
} 