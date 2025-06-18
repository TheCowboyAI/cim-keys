//! Basic Usage Example - CIM Keys
//!
//! This example demonstrates the basic usage patterns for CIM's key management system.
//! It showcases SSH key generation, TLS certificate creation, and secure key storage.

use cim_keys::{
    // Core traits
    KeyManager, Signer, CertificateManager, KeyStorage,
    
    // Key types and algorithms
    KeyAlgorithm, KeyUsage, SignatureFormat, KeyLocation,
    RsaKeySize, EcdsaCurve, KeyMetadata,
    CertificateFormat,
    
    // Managers
    ssh::SshKeyManager,
    tls::TlsManager,
    storage::MemoryKeyStorage,
    
    // Types
    KeyId,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 CIM Keys - Basic Usage Example\n");
    
    // Initialize storage backend
    let storage = Arc::new(RwLock::new(MemoryKeyStorage::new()));
    
    // Example 1: SSH Key Management
    println!("=== Example 1: SSH Key Management ===\n");
    
    let ssh_manager = SshKeyManager::new();
    
    // Generate an Ed25519 SSH key
    println!("1. Generating Ed25519 SSH key...");
    let ssh_key_id = ssh_manager.generate_key(
        KeyAlgorithm::Ed25519,
        "alice@cim-example".to_string(),
        KeyUsage {
            sign: true,
            verify: true,
            encrypt: false,
            decrypt: false,
            derive: false,
            authenticate: true,
        },
    ).await?;
    
    println!("   ✅ Generated SSH key: {}", ssh_key_id);
    
    // Export public key in SSH format
    println!("\n2. Exporting SSH public key...");
    let public_key = ssh_manager.export_public_key(&ssh_key_id)?;
    
    println!("   Public key (SSH format):");
    println!("   {}", String::from_utf8_lossy(&public_key));
    
    // Sign data with SSH key
    println!("\n3. Signing data with SSH key...");
    let message = b"Hello, CIM Keys!";
    let signature = ssh_manager.sign(
        &ssh_key_id,
        message,
        SignatureFormat::Raw,
    ).await?;
    
    println!("   ✅ Signature generated ({} bytes)", signature.len());
    
    // Example 2: TLS Certificate Management
    println!("\n=== Example 2: TLS Certificate Management ===\n");
    
    let tls_manager = TlsManager::new();
    
    // Generate self-signed certificate directly
    println!("1. Generating self-signed certificate...");
    let (tls_key_id, cert_id) = tls_manager.generate_self_signed(
        "server.cim.local",
        vec!["server.cim.local".to_string(), "localhost".to_string()],
        KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
        365, // Valid for 1 year
    ).await?;
    
    println!("   ✅ Generated TLS key: {}", tls_key_id);
    println!("   ✅ Generated certificate: {}", cert_id);
    
    // Export certificate
    println!("\n2. Exporting certificate...");
    let cert_pem = tls_manager.export_certificate(
        &cert_id,
        CertificateFormat::Pem,
        false, // Don't include chain
    ).await?;
    
    println!("   Certificate (PEM format):");
    println!("   {}", String::from_utf8_lossy(&cert_pem).lines().take(5).collect::<Vec<_>>().join("\n"));
    println!("   ... (truncated)");
    
    // Get certificate metadata
    println!("\n3. Getting certificate metadata...");
    let cert_metadata = tls_manager.get_certificate_metadata(&cert_id).await?;
    println!("   Subject: {}", cert_metadata.subject);
    println!("   Issuer: {}", cert_metadata.issuer);
    println!("   Valid from: {}", cert_metadata.not_before);
    println!("   Valid until: {}", cert_metadata.not_after);
    
    // Example 3: Key Storage Operations
    println!("\n=== Example 3: Key Storage Operations ===\n");
    
    // Create metadata for a key
    let metadata = KeyMetadata {
        id: KeyId::new(),
        algorithm: KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
        label: "ECDSA Signing Key".to_string(),
        usage: KeyUsage {
            sign: true,
            verify: true,
            encrypt: false,
            decrypt: false,
            derive: false,
            authenticate: false,
        },
        created_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::days(90)),
        description: Some("Key for document signing".to_string()),
        email: Some("alice@example.com".to_string()),
        fingerprint: None,
        hardware_serial: None,
    };
    
    // Store a key (mock data for demo)
    println!("1. Storing ECDSA key...");
    let key_data = b"mock-ecdsa-key-data";
    storage.write().await.store_key(
        &metadata.id, 
        key_data, 
        metadata.clone(),
        KeyLocation::Memory,
    ).await?;
    println!("   ✅ Key stored with ID: {}", metadata.id);
    
    // Check if key exists
    println!("\n2. Checking if key exists...");
    let exists = storage.read().await.key_exists(&metadata.id).await?;
    println!("   Key exists: {}", exists);
    
    // Retrieve key
    println!("\n3. Retrieving key...");
    let (retrieved_data, retrieved_metadata) = storage.read().await.retrieve_key(&metadata.id).await?;
    println!("   ✅ Retrieved key: {} ({} bytes)", retrieved_metadata.label, retrieved_data.len());
    
    // Example 4: Key Rotation
    println!("\n=== Example 4: Key Rotation ===\n");
    
    println!("1. Current SSH key: {}", ssh_key_id);
    
    // Generate new SSH key
    let new_ssh_key_id = ssh_manager.generate_key(
        KeyAlgorithm::Ed25519,
        "alice@cim-example".to_string(),
        KeyUsage {
            sign: true,
            verify: true,
            encrypt: false,
            decrypt: false,
            derive: false,
            authenticate: true,
        },
    ).await?;
    
    println!("2. New SSH key generated: {}", new_ssh_key_id);
    
    // Export new public key
    let new_public_key = ssh_manager.export_public_key(&new_ssh_key_id)?;
    
    println!("3. New public key:");
    println!("   {}", String::from_utf8_lossy(&new_public_key));
    
    // In production, you would:
    // - Update authorized_keys files
    // - Notify key consumers
    // - Schedule old key deletion
    
    println!("\n=== Summary ===");
    println!("✅ SSH key generation and signing");
    println!("✅ TLS certificate management");
    println!("✅ Secure key storage");
    println!("✅ Key metadata and attributes");
    println!("✅ Key rotation patterns");
    
    println!("\n🎉 Basic usage example completed successfully!");
    
    Ok(())
}
