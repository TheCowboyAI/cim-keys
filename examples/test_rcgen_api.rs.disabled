//! Quick test of rcgen API
use rcgen::{CertificateParams, Certificate};

fn main() {
    // Test the basic API
    let subject_alt_names = vec!["example.com".to_string()];

    // CertificateParams::new() returns Result
    let params = CertificateParams::new(subject_alt_names);

    // Create the cert from params
    let cert = Certificate::from_params(params).unwrap();

    // Get PEM
    let cert_pem = cert.serialize_pem().unwrap();
    println!("Certificate generated: {} bytes", cert_pem.len());
}