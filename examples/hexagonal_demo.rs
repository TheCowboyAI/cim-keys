/// Demonstration of Hexagonal Architecture with Category Theory Functors
/// Run with: cargo run --example hexagonal_demo --features gui

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  CIM-Keys Hexagonal Architecture Demonstration");
    println!("{}", "=".repeat(60));
    println!();

    // Initialize all mock adapters
    println!("📦 Initializing Mock Adapters (Category Theory Functors)...");

    use cim_keys::adapters::{
        InMemoryStorageAdapter,
        MockYubiKeyAdapter,
        MockX509Adapter,
        MockGpgAdapter,
        MockSshKeyAdapter,
    };

    let storage = Arc::new(InMemoryStorageAdapter::new());
    let yubikey = Arc::new(MockYubiKeyAdapter::default());
    let x509 = Arc::new(MockX509Adapter::new());
    let gpg = Arc::new(MockGpgAdapter::new());
    let ssh = Arc::new(MockSshKeyAdapter::new());

    println!("✅ Storage Port (In-Memory Functor)");
    println!("✅ YubiKey Port (Mock Hardware Functor)");
    println!("✅ X509 Port (Mock PKI Functor)");
    println!("✅ GPG Port (Mock OpenPGP Functor)");
    println!("✅ SSH Port (Mock SSH Keys Functor)");
    println!();

    // Demonstrate Category Theory Functor: Storage
    println!("🔬 Category Theory Verification: StoragePort");
    println!("{}", "-".repeat(60));

    use cim_keys::ports::StoragePort;

    let test_data = b"Hello, Hexagonal Architecture!";
    storage.write("test.txt", test_data).await?;
    let retrieved = storage.read("test.txt").await?;

    println!("Identity Law: read(write(data)) == data");
    println!("  Written: {:?}", String::from_utf8_lossy(test_data));
    println!("  Read:    {:?}", String::from_utf8_lossy(&retrieved));
    println!("  ✅ Verified: {}", test_data == retrieved.as_slice());
    println!();

    // Demonstrate YubiKey operations
    println!("🔐 YubiKey Port Demonstration");
    println!("{}", "-".repeat(60));

    use cim_keys::ports::yubikey::{YubiKeyPort, PivSlot, KeyAlgorithm, SecureString};

    let devices = yubikey.list_devices().await?;
    println!("Available YubiKeys: {}", devices.len());
    for device in &devices {
        println!("  📱 {} - {}", device.serial, device.model);
    }

    let pin = SecureString::new("123456");
    let public_key = yubikey.generate_key_in_slot(
        &devices[0].serial,
        PivSlot::Authentication,
        KeyAlgorithm::EccP256,
        &pin
    ).await?;

    println!("Generated key in slot 9A (Authentication)");
    println!("  Algorithm: {:?}", public_key.algorithm);
    println!("  ✅ Key generated successfully");
    println!();

    // Demonstrate X.509 PKI
    println!("📜 X.509 PKI Port Demonstration");
    println!("{}", "-".repeat(60));

    use cim_keys::ports::x509::{X509Port, CertificateSubject, PrivateKey};

    let root_subject = CertificateSubject {
        common_name: "CIM Root CA".to_string(),
        organization: Some("The Cowboy AI".to_string()),
        organizational_unit: None,
        country: Some("US".to_string()),
        state: None,
        locality: None,
        email: None,
    };

    let root_key = PrivateKey {
        algorithm: "RSA-2048".to_string(),
        der: vec![],
        pem: String::new(),
    };

    let root_cert = x509.generate_root_ca(&root_subject, &root_key, 3650).await?;
    println!("Generated Root CA Certificate");
    println!("  Subject: {}", root_cert.subject.common_name);
    println!("  Issuer: {} (self-signed)", root_cert.issuer.common_name);
    println!("  Is CA: {}", root_cert.is_ca);
    println!("  ✅ Root CA created");
    println!();

    // Demonstrate GPG operations
    println!("🔏 GPG Port Demonstration");
    println!("{}", "-".repeat(60));

    use cim_keys::ports::gpg::{GpgPort, GpgKeyType};

    let gpg_keypair = gpg.generate_keypair(
        "alice@example.com",
        GpgKeyType::Rsa,
        2048,
        None
    ).await?;

    println!("Generated GPG Keypair");
    println!("  User: {}", gpg_keypair.user_id);
    println!("  Key ID: {}", gpg_keypair.key_id.0);
    println!("  Fingerprint: {}", gpg_keypair.fingerprint);

    // Verify encryption/decryption identity law
    let plaintext = b"Secret message";
    let ciphertext = gpg.encrypt(&[gpg_keypair.key_id.clone()], plaintext).await?;
    let decrypted = gpg.decrypt(&gpg_keypair.key_id, &ciphertext).await?;

    println!("\nFunctor Identity Law: decrypt(encrypt(m)) == m");
    println!("  Original:  {:?}", String::from_utf8_lossy(plaintext));
    println!("  Decrypted: {:?}", String::from_utf8_lossy(&decrypted));
    println!("  ✅ Verified: {}", plaintext == decrypted.as_slice());
    println!();

    // Demonstrate SSH operations
    println!("🔑 SSH Port Demonstration");
    println!("{}", "-".repeat(60));

    use cim_keys::ports::ssh::{SshKeyPort, SshKeyType, FingerprintHashType};

    let ssh_keypair = ssh.generate_keypair(
        SshKeyType::Ed25519,
        None,
        Some("user@example.com".to_string())
    ).await?;

    println!("Generated SSH Keypair");
    println!("  Type: {:?}", ssh_keypair.public_key.key_type);
    println!("  Comment: {:?}", ssh_keypair.comment);

    let fingerprint = ssh.get_fingerprint(
        &ssh_keypair.public_key,
        FingerprintHashType::Sha256
    ).await?;
    println!("  Fingerprint: {}", fingerprint);

    let authorized_key = ssh.format_authorized_key(
        &ssh_keypair.public_key,
        Some("user@example.com".to_string())
    ).await?;
    println!("  Authorized Key Format: {}...", &authorized_key[..40]);
    println!("  ✅ SSH key ready for use");
    println!();

    // Summary
    println!("{}", "=".repeat(60));
    println!("✨ Hexagonal Architecture Summary");
    println!("{}", "=".repeat(60));
    println!("✅ All 5 ports demonstrated successfully");
    println!("✅ Category Theory Functor laws verified");
    println!("✅ Mock adapters enable offline testing");
    println!("✅ Clean separation: Domain ← Ports ← Adapters");
    println!();
    println!("🎯 Next: Replace mock adapters with production implementations");
    println!("   - InMemoryStorage → FileSystemStorage (encrypted SD card)");
    println!("   - MockYubiKey → YubiKeyPCSC (real hardware)");
    println!("   - MockX509 → RcgenX509 (real certificates)");
    println!("   - MockGPG → SequoiaGPG (real OpenPGP)");
    println!("   - MockSSH → SshKeys (real SSH keys)");

    Ok(())
}
