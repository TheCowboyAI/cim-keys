//! Basic usage example for cim-keys
//!
//! This example demonstrates basic key generation, signing, and encryption operations.

use cim_keys::{
    KeyManager, Signer, Encryptor, CertificateManager,
    KeyAlgorithm, KeyUsage, SignatureFormat, EncryptionFormat,
    RsaKeySize, EcdsaCurve, KeyExportFormat,
    ssh::SshKeyManager,
    tls::TlsManager,
    storage::MemoryKeyStorage,
};
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting cim-keys basic usage example");

    // Example 1: SSH Key Management
    info!("\n=== SSH Key Management ===");
    let ssh_manager = SshKeyManager::new();

    // Generate an Ed25519 SSH key
    let ssh_key_id = ssh_manager.generate_key(
        KeyAlgorithm::Ed25519,
        "example@cim-keys".to_string(),
        KeyUsage::Signing,
    ).await?;

    info!("Generated SSH key: {}", ssh_key_id);

    // Export the public key in OpenSSH format
    let public_key = ssh_manager.export_key(
        &ssh_key_id,
        KeyExportFormat::OpenSsh,
        false, // public key only
    ).await?;

    info!("SSH Public Key:\n{}", String::from_utf8_lossy(&public_key));

    // Sign some data
    let message = b"Hello, CIM Keys!";
    let signature = ssh_manager.sign(
        &ssh_key_id,
        message,
        SignatureFormat::Raw,
    ).await?;

    info!("Signature length: {} bytes", signature.len());

    // Verify the signature
    let valid = ssh_manager.verify(
        &ssh_key_id,
        message,
        &signature,
        SignatureFormat::Raw,
    ).await?;

    info!("Signature valid: {}", valid);

    // Example 2: TLS Certificate Management
    info!("\n=== TLS Certificate Management ===");
    let tls_manager = TlsManager::new();

    // Generate a self-signed certificate
    let (tls_key_id, cert_id) = tls_manager.generate_self_signed(
        "example.cim-keys.local",
        vec!["example.cim-keys.local".to_string(), "localhost".to_string()],
        KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
        365, // validity days
    ).await?;

    info!("Generated TLS certificate: {} with key: {}", cert_id, tls_key_id);

    // Export the certificate in PEM format
    let cert_pem = tls_manager.export_certificate(
        &cert_id,
        cim_keys::CertificateFormat::Pem,
        false, // no chain
    ).await?;

    info!("Certificate PEM:\n{}", String::from_utf8_lossy(&cert_pem));

    // Get certificate metadata
    let cert_metadata = tls_manager.get_certificate_metadata(&cert_id).await?;
    info!("Certificate Subject: {}", cert_metadata.subject);
    info!("Certificate Issuer: {}", cert_metadata.issuer);
    info!("Valid from: {} to {}", cert_metadata.not_before, cert_metadata.not_after);

    // Example 3: Key Storage
    info!("\n=== Key Storage ===");
    let storage = MemoryKeyStorage::new();

    // Store a key
    let key_data = b"example-key-data";
    let metadata = cim_keys::KeyMetadata {
        key_id: ssh_key_id,
        algorithm: KeyAlgorithm::Ed25519,
        label: "Example SSH Key".to_string(),
        usage: KeyUsage::Signing,
        created_at: chrono::Utc::now(),
        expires_at: None,
        attributes: std::collections::HashMap::new(),
    };

    storage.store_key(
        &ssh_key_id,
        key_data,
        metadata.clone(),
        cim_keys::KeyLocation::Memory,
    ).await?;

    info!("Stored key in memory storage");

    // Retrieve the key
    let (retrieved_data, retrieved_metadata) = storage.retrieve_key(&ssh_key_id).await?;
    assert_eq!(key_data, &retrieved_data[..]);
    assert_eq!(metadata.label, retrieved_metadata.label);

    info!("Successfully retrieved key from storage");

    // Example 4: Encryption (using RSA for asymmetric encryption)
    info!("\n=== Encryption Example ===");

    // Note: For real encryption, you would use a proper implementation
    // This is just demonstrating the API structure

    info!("Example completed successfully!");

    Ok(())
}
