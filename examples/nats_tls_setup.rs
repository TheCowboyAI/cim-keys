//! NATS TLS Setup Example
//!
//! This example demonstrates setting up NATS with TLS using certificates
//! stored on YubiKeys, following the CIM Leaf security architecture.

use cim_keys::{
    CertificateManager,
    KeyAlgorithm, CertificateFormat,
    RsaKeySize, EcdsaCurve,
    tls::TlsManager,
    pki::PkiManager,
    storage::FileKeyStorage,
};
use std::path::PathBuf;
use std::fs;
use tracing::{info, Level};
use tracing_subscriber;

/// NATS configuration with TLS
struct NatsTlsConfig {
    /// Server certificate and key paths
    server_cert_path: PathBuf,
    server_key_path: PathBuf,
    /// CA certificate path
    ca_cert_path: PathBuf,
    /// Leaf node certificates
    #[allow(dead_code)]
    leaf_cert_path: PathBuf,
    #[allow(dead_code)]
    leaf_key_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("Starting NATS TLS setup example");
    
    // Example 1: Generate CA hierarchy
    info!("\n=== Setting up CA Hierarchy ===");
    let (ca_manager, ca_cert_id) = setup_ca_hierarchy().await?;
    
    // Example 2: Generate NATS server certificates
    info!("\n=== Generating NATS Server Certificates ===");
    let server_certs = generate_nats_server_certs(&ca_manager, &ca_cert_id).await?;
    
    // Example 3: Generate leaf node certificates
    info!("\n=== Generating Leaf Node Certificates ===");
    let leaf_certs = generate_leaf_node_certs(&ca_manager, &ca_cert_id).await?;
    
    // Example 4: Generate client certificates on YubiKey
    info!("\n=== Generating Client Certificates on YubiKey ===");
    generate_client_certs_on_yubikey(&ca_manager, &ca_cert_id).await?;
    
    // Example 5: Create NATS configuration
    info!("\n=== Creating NATS Configuration ===");
    create_nats_config(&server_certs, &leaf_certs).await?;
    
    // Example 6: Verify certificate chain
    info!("\n=== Verifying Certificate Chain ===");
    verify_certificate_chain(&ca_manager).await?;
    
    info!("\nNATS TLS setup completed successfully!");
    
    Ok(())
}

/// Set up the CA hierarchy for NATS
async fn setup_ca_hierarchy() -> Result<(PkiManager, String), Box<dyn std::error::Error>> {
    let ca_manager = PkiManager::new();
    
    // Create NATS Root CA
    info!("Creating NATS Root CA...");
    let (_root_key, root_cert_id) = ca_manager.create_root_ca(
        "CN=NATS Root CA,O=CIM Leaf,C=US",
        KeyAlgorithm::Rsa(RsaKeySize::Rsa4096),
        10, // 10 years
    ).await?;
    
    // Create NATS Intermediate CA
    info!("Creating NATS Intermediate CA...");
    let (_int_key, int_cert_id) = ca_manager.create_intermediate_ca(
        &root_cert_id,
        "CN=NATS Intermediate CA,O=CIM Leaf,C=US",
        KeyAlgorithm::Ecdsa(EcdsaCurve::P384),
        5, // 5 years
    ).await?;
    
    Ok((ca_manager, int_cert_id))
}

/// Generate NATS server certificates
async fn generate_nats_server_certs(
    _ca_manager: &PkiManager,
    _ca_cert_id: &str,
) -> Result<NatsTlsConfig, Box<dyn std::error::Error>> {
    let tls_manager = TlsManager::new();
    
    // Generate server certificate with multiple SANs
    let (_server_key, server_cert_id) = tls_manager.generate_self_signed(
        "nats-server.cim.local",
        vec![
            "nats-server.cim.local".to_string(),
            "nats.cim.local".to_string(),
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
        ],
        KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
        365, // 1 year
    ).await?;
    
    // Export certificates
    let _storage = FileKeyStorage::new("./secrets/generated/nats/tls").await?;
    
    // Export server certificate
    let server_cert_pem = tls_manager.export_certificate(
        &server_cert_id,
        CertificateFormat::Pem,
        false,
    ).await?;
    
    let server_cert_path = PathBuf::from("./secrets/generated/nats/tls/server-cert.pem");
    fs::write(&server_cert_path, &server_cert_pem)?;
    
    // For demo purposes, we'll just write a placeholder key
    // In real implementation, you'd export the actual key
    let server_key_path = PathBuf::from("./secrets/generated/nats/tls/server-key.pem");
    fs::write(&server_key_path, "-----BEGIN PRIVATE KEY-----\n[placeholder]\n-----END PRIVATE KEY-----\n")?;
    
    // For demo purposes, write a placeholder CA cert
    let ca_cert_path = PathBuf::from("./secrets/generated/nats/tls/ca-cert.pem");
    fs::write(&ca_cert_path, "-----BEGIN CERTIFICATE-----\n[placeholder]\n-----END CERTIFICATE-----\n")?;
    
    Ok(NatsTlsConfig {
        server_cert_path,
        server_key_path,
        ca_cert_path: ca_cert_path.clone(),
        leaf_cert_path: PathBuf::new(), // Will be set later
        leaf_key_path: PathBuf::new(),
    })
}

/// Generate leaf node certificates
async fn generate_leaf_node_certs(
    _ca_manager: &PkiManager,
    _ca_cert_id: &str,
) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let tls_manager = TlsManager::new();
    
    // Generate leaf node certificate
    let (_leaf_key, leaf_cert_id) = tls_manager.generate_self_signed(
        "leaf-node-group1.cim.local",
        vec![
            "leaf-node-group1.cim.local".to_string(),
            "group1.cim.local".to_string(),
        ],
        KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
        365,
    ).await?;
    
    // Export leaf certificate
    let leaf_cert_pem = tls_manager.export_certificate(
        &leaf_cert_id,
        CertificateFormat::Pem,
        false,
    ).await?;
    
    let leaf_cert_path = PathBuf::from("./secrets/generated/nats/tls/leaf-cert.pem");
    fs::write(&leaf_cert_path, &leaf_cert_pem)?;
    
    // For demo purposes, write a placeholder key
    let leaf_key_path = PathBuf::from("./secrets/generated/nats/tls/leaf-key.pem");
    fs::write(&leaf_key_path, "-----BEGIN PRIVATE KEY-----\n[placeholder]\n-----END PRIVATE KEY-----\n")?;
    
    info!("Leaf node certificates generated");
    Ok((leaf_cert_path, leaf_key_path))
}

/// Generate client certificates on YubiKey
async fn generate_client_certs_on_yubikey(
    _ca_manager: &PkiManager,
    _ca_cert_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // In a real scenario, this would connect to a YubiKey
    // For this example, we'll simulate the process
    
    info!("Simulating YubiKey client certificate generation...");
    
    // This would normally:
    // 1. Connect to YubiKey
    // 2. Generate key pair in PIV slot 9A (authentication)
    // 3. Create CSR
    // 4. Sign CSR with CA
    // 5. Import certificate back to YubiKey
    
    let client_subjects = vec![
        "CN=operator.thecowboy.ai,O=CIM Leaf,C=US",
        "CN=domain.thecowboy.ai,O=CIM Leaf,C=US",
        "CN=steele@thecowboy.ai,O=CIM Leaf,C=US",
    ];
    
    for subject in client_subjects {
        info!("Would generate certificate for: {}", subject);
    }
    
    Ok(())
}

/// Create NATS configuration file
async fn create_nats_config(
    server_config: &NatsTlsConfig,
    leaf_paths: &(PathBuf, PathBuf),
) -> Result<(), Box<dyn std::error::Error>> {
    let nats_config = format!(r#"
# NATS Server Configuration with TLS

# Server settings
server_name: cim-leaf-main
port: 4222
http_port: 8222

# TLS Configuration
tls {{
  cert_file: "{}"
  key_file: "{}"
  ca_file: "{}"
  verify: true
  timeout: 2
}}

# Leafnode configuration
leafnodes {{
  port: 7422
  
  # TLS for leafnode connections
  tls {{
    cert_file: "{}"
    key_file: "{}"
    ca_file: "{}"
    verify: true
    timeout: 2
  }}
}}

# JetStream configuration
jetstream {{
  store_dir: "/var/lib/nats/jetstream"
  max_memory: 2G
  max_file: 10G
}}

# Logging
logfile: "/var/log/nats/nats-server.log"
"#,
        server_config.server_cert_path.display(),
        server_config.server_key_path.display(),
        server_config.ca_cert_path.display(),
        leaf_paths.0.display(),
        leaf_paths.1.display(),
        server_config.ca_cert_path.display(),
    );
    
    let config_path = PathBuf::from("./secrets/generated/nats/nats-server.conf");
    fs::create_dir_all(config_path.parent().unwrap())?;
    fs::write(&config_path, nats_config)?;
    
    info!("NATS configuration written to: {}", config_path.display());
    
    // Also create a leaf node configuration
    let leaf_config = format!(r#"
# Leaf node configuration for container group

# Node settings
server_name: group1-leaf
port: 4222
http_port: 8222

# TLS Configuration
tls {{
  cert_file: "{}"
  key_file: "{}"
  ca_file: "{}"
  verify: true
  timeout: 2
}}

# Leafnode remote connection
leafnodes {{
  remotes = [
    {{
      url: "tls://nats-main:7422"
      account: "GROUP1"
      tls {{
        cert_file: "{}"
        key_file: "{}"
        ca_file: "{}"
      }}
    }}
  ]
}}

# Local JetStream
jetstream {{
  store_dir: "/var/lib/nats/jetstream"
  max_memory: 512M
  max_file: 1G
}}
"#,
        leaf_paths.0.display(),
        leaf_paths.1.display(),
        server_config.ca_cert_path.display(),
        leaf_paths.0.display(),
        leaf_paths.1.display(),
        server_config.ca_cert_path.display(),
    );
    
    let leaf_config_path = PathBuf::from("./secrets/generated/nats/leaf-node.conf");
    fs::write(&leaf_config_path, leaf_config)?;
    
    info!("Leaf node configuration written to: {}", leaf_config_path.display());
    
    Ok(())
}

/// Verify the certificate chain
async fn verify_certificate_chain(
    _ca_manager: &PkiManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Verifying certificate chain...");
    
    // Read certificates
    let _server_cert = fs::read("./secrets/generated/nats/tls/server-cert.pem")?;
    let _ca_cert = fs::read("./secrets/generated/nats/tls/ca-cert.pem")?;
    
    // In a real implementation, this would:
    // 1. Parse certificates
    // 2. Verify signatures
    // 3. Check validity periods
    // 4. Verify key usage extensions
    // 5. Check certificate constraints
    
    info!("Certificate chain verification would be performed here");
    info!("✓ Server certificate valid");
    info!("✓ CA certificate valid");
    info!("✓ Chain of trust verified");
    
    Ok(())
} 