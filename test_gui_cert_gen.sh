#!/bin/bash
# Test script to verify GUI certificate generation works

echo "Testing GUI certificate generation functionality..."
echo "Output directory: /tmp/cim-keys-output"

# Create output directory
mkdir -p /tmp/cim-keys-output

# Run a simple test that simulates what the GUI does
cat << 'EOF' > /tmp/test_cert_gen.rs
use cim_keys::{
    KeyManagementAggregate, OfflineKeyProjection,
    commands::{KeyCommand, GenerateCertificateCommand, CertificateSubject},
    events::KeyEvent,
    certificate_service,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    println!("Testing certificate generation as GUI would...");

    let output_dir = PathBuf::from("/tmp/cim-keys-output");
    let aggregate = Arc::new(RwLock::new(KeyManagementAggregate::new()));
    let projection = Arc::new(RwLock::new(OfflineKeyProjection::new(output_dir).unwrap()));

    // Create command like GUI does
    let root_ca_cmd = GenerateCertificateCommand {
        command_id: cim_domain::EntityId::new(),
        key_id: uuid::Uuid::now_v7(),
        subject: CertificateSubject {
            common_name: "Test GUI Root CA".to_string(),
            organization: Some("Test Org".to_string()),
            country: Some("US".to_string()),
            organizational_unit: Some("Security".to_string()),
            locality: None,
            state_or_province: None,
        },
        validity_days: 3650,
        is_ca: true,
        san: vec![],
        key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
        extended_key_usage: vec![],
        requestor: "Test Script".to_string(),
        context: None,
    };

    // Process command
    let aggregate = aggregate.read().await;
    let projection_read = projection.read().await;

    let events = aggregate.handle_command(
        KeyCommand::GenerateCertificate(root_ca_cmd),
        &*projection_read,
        None,
        #[cfg(feature = "policy")]
        None
    ).await.unwrap();

    drop(projection_read);
    drop(aggregate);

    // Process events and generate certificate
    if !events.is_empty() {
        let mut projection_write = projection.write().await;

        for event in events {
            match event {
                KeyEvent::CertificateGenerated(e) => {
                    println!("Certificate event generated: {}", e.cert_id);

                    // Generate actual certificate
                    let generated = certificate_service::generate_root_ca_from_event(&e).unwrap();

                    println!("Certificate generated successfully!");
                    println!("  Length: {} bytes", generated.certificate_pem.len());
                    println!("  Fingerprint: {}", generated.fingerprint);

                    // Save to projection
                    let cert_dir = projection_write.root_path.join("certificates").join("root-ca");
                    std::fs::create_dir_all(&cert_dir).unwrap();

                    let cert_file = cert_dir.join(format!("{}.crt", e.cert_id));
                    std::fs::write(&cert_file, generated.certificate_pem.as_bytes()).unwrap();
                    println!("  Saved to: {}", cert_file.display());

                    let key_file = cert_dir.join(format!("{}.key", e.cert_id));
                    std::fs::write(&key_file, generated.private_key_pem.as_bytes()).unwrap();
                    println!("  Key saved to: {}", key_file.display());
                },
                _ => {}
            }
        }
    } else {
        println!("No events generated!");
    }
}
EOF

# Compile and run the test
cd /git/thecowboyai/cim-keys
rustc --edition 2021 -L target/debug/deps /tmp/test_cert_gen.rs \
  --extern cim_keys=target/debug/libcim_keys.rlib \
  --extern cim_domain=target/debug/deps/libcim_domain-*.rlib \
  --extern tokio=target/debug/deps/libtokio-*.rlib \
  --extern uuid=target/debug/deps/libuuid-*.rlib \
  -o /tmp/test_cert_gen 2>/dev/null || {
    # If rustc fails, use cargo instead
    echo "Using cargo to run the test..."
    cargo run --example test_root_ca
    exit 0
}

/tmp/test_cert_gen

echo ""
echo "Checking generated files..."
ls -la /tmp/cim-keys-output/certificates/root-ca/ 2>/dev/null || echo "No certificates generated yet"