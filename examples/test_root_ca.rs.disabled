//! Test Root CA generation
//!
//! Run with: cargo run --example test_root_ca

use cim_keys::{
    certificate_service::generate_root_ca_from_event,
    events::CertificateGeneratedEvent,
};
use chrono::{Utc, Duration};
use uuid::Uuid;

fn main() {
    println!("Testing Root CA generation...");

    // Create a test event
    let event = CertificateGeneratedEvent {
        cert_id: Uuid::now_v7(),
        key_id: Uuid::now_v7(),
        subject: "CN=Test Root CA, O=Test Organization, C=US".to_string(),
        issuer: None, // Self-signed
        not_before: Utc::now(),
        not_after: Utc::now() + Duration::days(3650), // 10 years
        is_ca: true,
        san: vec!["ca.example.com".to_string()],
        key_usage: vec![
            "keyCertSign".to_string(),
            "cRLSign".to_string(),
            "digitalSignature".to_string(),
        ],
        extended_key_usage: vec![],
    };

    // Generate the certificate
    match generate_root_ca_from_event(&event) {
        Ok(cert) => {
            println!("✅ Root CA generated successfully!");
            println!("Certificate length: {} bytes", cert.certificate_pem.len());
            println!("Private key length: {} bytes", cert.private_key_pem.len());
            println!("Fingerprint: {}", cert.fingerprint);

            // Print first few lines of certificate
            let lines: Vec<&str> = cert.certificate_pem.lines().take(5).collect();
            println!("\nCertificate (first 5 lines):");
            for line in lines {
                println!("  {}", line);
            }

            println!("\n📝 Full certificate:\n{}", cert.certificate_pem);
        }
        Err(e) => {
            eprintln!("❌ Failed to generate Root CA: {}", e);
        }
    }
}