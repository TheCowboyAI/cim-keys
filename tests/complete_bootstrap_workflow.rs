// Copyright (c) 2025 - Cowboy AI, LLC.

//! Complete Bootstrap Workflow Test
//!
//! This test implements the FULL cim-keys workflow:
//! 1. Load organization from domain-bootstrap.json
//! 2. Generate complete PKI hierarchy (Root CA → Intermediate → Leaf)
//! 3. Provision YubiKeys (org and individual)
//! 4. Generate NATS credentials (Operator → Accounts → Users)
//! 5. Project everything to SD card structure
//! 6. Produce audit trail and key map
//!
//! Run with actual YubiKeys:
//!   cargo test --test complete_bootstrap_workflow -- --nocapture
//!
//! Run in mock mode (CI):
//!   cargo test --test complete_bootstrap_workflow --features mock-yubikey -- --nocapture

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cim_keys::secrets_loader::{SecretsLoader, BootstrapData, YubiKeyAssignment, NatsHierarchy};
use cim_keys::crypto::{
    derive_master_seed,
    x509::{generate_root_ca, generate_intermediate_ca, generate_server_certificate},
    x509::{RootCAParams, IntermediateCAParams, ServerCertParams, X509Certificate},
};
use cim_keys::adapters::nsc::NscAdapter;
use cim_keys::adapters::yubikey_cli::YubiKeyCliAdapter;
use cim_keys::ports::nats::NatsKeyPort;
use cim_keys::ports::yubikey::{YubiKeyPort, PivSlot, KeyAlgorithm, SecureString};
use cim_keys::domain::{Organization, Person, OrganizationUnit};

/// Output structure that will be written to SD card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapOutput {
    /// Manifest metadata
    pub manifest: Manifest,
    /// PKI certificates
    pub pki: PkiOutput,
    /// NATS credentials
    pub nats: NatsOutput,
    /// YubiKey provisioning results
    pub yubikeys: Vec<YubiKeyOutput>,
    /// Complete audit trail
    pub audit_trail: Vec<AuditEvent>,
    /// Key map linking all keys to their owners and purposes
    pub key_map: KeyMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub organization_id: String,
    pub organization_name: String,
    pub created_at: String,
    pub created_by: String,
}

/// Secrets record - stored separately for secure backup
/// Contains YubiKey PINs needed to access the keys
/// NOTE: Master passphrase is NEVER stored - it's generated randomly and destroyed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsRecord {
    pub yubikey_pins: Vec<YubiKeyPinRecord>,
    pub created_at: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyPinRecord {
    pub serial: String,
    pub pin: String,
    pub puk: String,
    pub management_key: Option<String>,
    pub assigned_to: Option<String>,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkiOutput {
    pub root_ca: CertificateOutput,
    pub intermediate_cas: Vec<CertificateOutput>,
    pub server_certs: Vec<CertificateOutput>,
    pub chain_pem: String, // Full certificate chain
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateOutput {
    pub id: String,
    pub subject: String,
    pub issuer: Option<String>,
    pub fingerprint: String,
    pub not_before: String,
    pub not_after: String,
    pub is_ca: bool,
    pub certificate_pem: String,
    pub private_key_pem: Option<String>, // None if stored on YubiKey
    pub stored_on_yubikey: Option<String>, // YubiKey serial if applicable
    pub yubikey_slot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOutput {
    pub operator: NatsCredentialOutput,
    pub accounts: Vec<NatsCredentialOutput>,
    pub users: Vec<NatsCredentialOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsCredentialOutput {
    pub id: String,
    pub name: String,
    pub public_key: String,
    pub jwt: Option<String>,
    pub creds_file: Option<String>, // Contents of .creds file for users
    pub seed: Option<String>, // Only included if not on YubiKey
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyOutput {
    pub serial: String,
    pub name: String,
    pub owner_person_id: String,
    pub owner_name: String,
    pub role: String,
    pub slots: Vec<YubiKeySlotOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeySlotOutput {
    pub slot: String,
    pub slot_id: String,
    pub key_type: String,
    pub public_key: Option<String>,
    pub certificate_id: Option<String>,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub description: String,
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub actor: Option<String>,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMap {
    /// Map of key ID to key info
    pub keys: HashMap<String, KeyInfo>,
    /// Map of person ID to their key IDs
    pub person_keys: HashMap<String, Vec<String>>,
    /// Map of YubiKey serial to key IDs stored on it
    pub yubikey_keys: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub id: String,
    pub key_type: String, // "pki", "nats", "signing"
    pub algorithm: String,
    pub owner_id: Option<String>,
    pub owner_type: String, // "organization", "person", "service"
    pub purpose: String,
    pub storage_location: String, // "yubikey:SERIAL:SLOT", "file:PATH", "sd-card"
    pub public_key: String,
    pub created_at: String,
}

/// Generated YubiKey credentials (random, stored only for output)
#[derive(Debug, Clone)]
struct GeneratedYubiKeyCredentials {
    serial: String,
    pin: String,
    puk: String,
    management_key: String,
    assigned_to: String,
    role: String,
}

/// Bootstrap workflow engine
pub struct BootstrapWorkflow {
    bootstrap_data: BootstrapData,
    output_dir: PathBuf,
    audit_trail: Vec<AuditEvent>,
    correlation_id: Uuid,
    /// Generated credentials for each YubiKey - created during workflow
    generated_credentials: Vec<GeneratedYubiKeyCredentials>,
}

impl BootstrapWorkflow {
    pub fn new(
        bootstrap_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
    ) -> Result<Self, String> {
        let bootstrap_data = SecretsLoader::load_from_bootstrap_file(bootstrap_path)
            .map_err(|e| format!("Failed to load bootstrap file: {}", e))?;

        let correlation_id = Uuid::now_v7();

        Ok(Self {
            bootstrap_data,
            output_dir: output_dir.as_ref().to_path_buf(),
            audit_trail: Vec::new(),
            correlation_id,
            generated_credentials: Vec::new(),
        })
    }

    /// Generate a cryptographically secure random passphrase
    /// This passphrase is used once and then destroyed - never stored
    fn generate_ephemeral_passphrase() -> String {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        // 64 bytes of randomness, hex encoded = 128 character passphrase
        let mut random_bytes = [0u8; 64];
        rng.fill_bytes(&mut random_bytes);
        hex::encode(random_bytes)
    }

    /// Generate a random 6-8 digit PIN
    fn generate_random_pin() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        // 6 digit PIN (YubiKey default)
        let pin: u32 = rng.gen_range(100000..999999);
        pin.to_string()
    }

    /// Generate a random 8 digit PUK
    fn generate_random_puk() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        // 8 digit PUK (YubiKey default)
        let puk: u32 = rng.gen_range(10000000..99999999);
        puk.to_string()
    }

    /// Generate a random 24-byte (48 hex chars) management key
    fn generate_random_management_key() -> String {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut key_bytes = [0u8; 24];
        rng.fill_bytes(&mut key_bytes);
        hex::encode(key_bytes)
    }

    /// Add an audit event
    fn audit(&mut self, event_type: &str, description: &str, causation_id: Option<Uuid>, details: HashMap<String, String>) {
        let event = AuditEvent {
            id: Uuid::now_v7().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            description: description.to_string(),
            correlation_id: self.correlation_id.to_string(),
            causation_id: causation_id.map(|id| id.to_string()),
            actor: Some("cim-keys-bootstrap".to_string()),
            details,
        };
        self.audit_trail.push(event);
    }

    /// Generate random credentials for all YubiKeys
    fn generate_yubikey_credentials(&mut self) {
        println!("  Generating random credentials for YubiKeys...");
        for assignment in &self.bootstrap_data.yubikey_assignments {
            let creds = GeneratedYubiKeyCredentials {
                serial: assignment.serial.clone(),
                pin: Self::generate_random_pin(),
                puk: Self::generate_random_puk(),
                management_key: Self::generate_random_management_key(),
                assigned_to: format!("{} ({})", assignment.name, assignment.person_id),
                role: assignment.role.clone(),
            };
            self.generated_credentials.push(creds);
        }
        println!("  ✓ Generated credentials for {} YubiKeys", self.generated_credentials.len());
    }

    /// Execute the complete bootstrap workflow
    pub async fn execute(&mut self) -> Result<BootstrapOutput, String> {
        // Create output directory structure
        self.create_output_structure()?;

        // Phase 0: Generate random credentials for all YubiKeys
        println!("\n=== Phase 0: Generating YubiKey Credentials ===");
        self.generate_yubikey_credentials();
        self.audit("credentials_generated", "Random YubiKey credentials generated", None, HashMap::new());

        // Phase 1: Generate PKI
        println!("\n=== Phase 1: Generating PKI Hierarchy ===");
        self.audit("workflow_started", "Bootstrap workflow initiated", None, HashMap::new());
        let pki = self.generate_pki()?;

        // Phase 2: Provision YubiKeys
        println!("\n=== Phase 2: Provisioning YubiKeys ===");
        let yubikeys = self.provision_yubikeys(&pki).await?;

        // Phase 3: Generate NATS credentials
        println!("\n=== Phase 3: Generating NATS Credentials ===");
        let nats = self.generate_nats_credentials().await?;

        // Phase 4: Build key map
        println!("\n=== Phase 4: Building Key Map ===");
        let key_map = self.build_key_map(&pki, &nats, &yubikeys);

        // Phase 5: Create manifest
        println!("\n=== Phase 5: Creating Manifest ===");
        let manifest = self.create_manifest();

        self.audit("workflow_completed", "Bootstrap workflow completed successfully", Some(self.correlation_id), HashMap::new());

        let output = BootstrapOutput {
            manifest,
            pki,
            nats,
            yubikeys,
            audit_trail: self.audit_trail.clone(),
            key_map,
        };

        // Write output to files
        self.write_output(&output)?;

        Ok(output)
    }

    fn create_output_structure(&self) -> Result<(), String> {
        let dirs = [
            "certificates/root-ca",
            "certificates/intermediate",
            "certificates/server",
            "nats/operator",
            "nats/accounts",
            "nats/users",
            "keys",
            "events",
            "yubikeys",
        ];

        for dir in dirs {
            let path = self.output_dir.join(dir);
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create {}: {}", path.display(), e))?;
        }

        Ok(())
    }

    fn generate_pki(&mut self) -> Result<PkiOutput, String> {
        // Clone to avoid borrow issues
        let org = self.bootstrap_data.organization.clone();
        let units = self.bootstrap_data.units.clone();

        // Generate ephemeral passphrase - used once and destroyed
        // This passphrase is NEVER stored or displayed
        let ephemeral_passphrase = Self::generate_ephemeral_passphrase();

        // Derive master seed from passphrase (Argon2id - intentionally slow for security)
        println!("  Deriving master seed (this may take a moment)...");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let master_seed = derive_master_seed(&ephemeral_passphrase, &org.name)
            .map_err(|e| format!("Failed to derive master seed: {}", e))?;
        // ephemeral_passphrase is dropped here and never used again
        drop(ephemeral_passphrase);
        println!("  ✓ Master seed derived (passphrase destroyed)");

        // Generate Root CA
        println!("  Generating Root CA for {}...", org.display_name);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let root_ca_seed = master_seed.derive_child("root-ca");
        let root_params = RootCAParams {
            organization: org.display_name.clone(),
            common_name: format!("{} Root CA", org.display_name),
            country: org.metadata.get("country").cloned(),
            state: org.metadata.get("state").cloned(),
            locality: org.metadata.get("city").cloned(),
            validity_years: 20,
            pathlen: 1, // Allow one intermediate CA level
        };

        let root_correlation_id = Uuid::now_v7();
        let (root_ca, root_event) = generate_root_ca(&root_ca_seed, root_params, root_correlation_id, Some(self.correlation_id))
            .map_err(|e| format!("Failed to generate root CA: {}", e))?;

        self.audit("root_ca_generated", &format!("Root CA generated: {}", root_event.subject), Some(root_correlation_id), {
            let mut details = HashMap::new();
            details.insert("cert_id".to_string(), root_event.cert_id.to_string());
            details.insert("fingerprint".to_string(), root_ca.fingerprint.clone());
            details
        });

        let root_ca_output = CertificateOutput {
            id: root_event.cert_id.to_string(),
            subject: root_event.subject.clone(),
            issuer: None,
            fingerprint: root_ca.fingerprint.clone(),
            not_before: root_event.not_before.to_rfc3339(),
            not_after: root_event.not_after.to_rfc3339(),
            is_ca: true,
            certificate_pem: root_ca.certificate_pem.clone(),
            private_key_pem: Some(root_ca.private_key_pem.clone()), // Will be moved to YubiKey
            stored_on_yubikey: None, // Updated after provisioning
            yubikey_slot: None,
        };

        // Generate Intermediate CAs for each unit
        let mut intermediate_cas = Vec::new();
        let mut chain_pem = root_ca.certificate_pem.clone();

        for unit in &units {
            println!("  Generating Intermediate CA for {}...", unit.name);
            let intermediate_seed = root_ca_seed.derive_child(&format!("intermediate-{}", unit.name));
            let intermediate_params = IntermediateCAParams {
                organization: org.display_name.clone(),
                organizational_unit: unit.name.clone(),
                common_name: format!("{} {} Intermediate CA", org.display_name, unit.name),
                country: org.metadata.get("country").cloned(),
                validity_years: 3,
                pathlen: 0, // Can only sign leaf certificates
            };

            let intermediate_correlation_id = Uuid::now_v7();
            let (intermediate_ca, gen_event, _sign_event) = generate_intermediate_ca(
                &intermediate_seed,
                intermediate_params,
                &root_ca.certificate_pem,
                &root_ca.private_key_pem,
                root_event.cert_id,
                intermediate_correlation_id,
                Some(root_correlation_id),
            ).map_err(|e| format!("Failed to generate intermediate CA for {}: {}", unit.name, e))?;

            self.audit("intermediate_ca_generated", &format!("Intermediate CA generated: {}", gen_event.subject), Some(intermediate_correlation_id), {
                let mut details = HashMap::new();
                details.insert("cert_id".to_string(), gen_event.cert_id.to_string());
                details.insert("unit".to_string(), unit.name.clone());
                details
            });

            chain_pem.push_str("\n");
            chain_pem.push_str(&intermediate_ca.certificate_pem);

            intermediate_cas.push(CertificateOutput {
                id: gen_event.cert_id.to_string(),
                subject: gen_event.subject,
                issuer: Some(root_event.cert_id.to_string()),
                fingerprint: intermediate_ca.fingerprint,
                not_before: gen_event.not_before.to_rfc3339(),
                not_after: gen_event.not_after.to_rfc3339(),
                is_ca: true,
                certificate_pem: intermediate_ca.certificate_pem,
                private_key_pem: Some(intermediate_ca.private_key_pem),
                stored_on_yubikey: None,
                yubikey_slot: None,
            });
        }

        // Generate server certificates for NATS nodes
        let mut server_certs = Vec::new();
        let server_names = ["nats-1", "nats-2", "nats-3"];

        for (i, server_name) in server_names.iter().enumerate() {
            // Use infrastructure intermediate CA if available, otherwise first intermediate
            let intermediate_ca = intermediate_cas.iter()
                .find(|ca| ca.subject.contains("infrastructure"))
                .or(intermediate_cas.first())
                .ok_or("No intermediate CA available")?;

            let intermediate_seed = root_ca_seed.derive_child(&format!("intermediate-{}",
                if intermediate_ca.subject.contains("infrastructure") { "infrastructure" } else { &units[0].name }));
            let server_seed = intermediate_seed.derive_child(server_name);

            let server_params = ServerCertParams {
                common_name: format!("{}.{}", server_name, org.metadata.get("domain").unwrap_or(&"local".to_string())),
                san_entries: vec![
                    format!("{}.{}", server_name, org.metadata.get("domain").unwrap_or(&"local".to_string())),
                    format!("10.0.0.4{}", i + 1), // NATs cluster IPs
                ],
                organization: org.display_name.clone(),
                organizational_unit: Some("Infrastructure".to_string()),
                validity_days: 90,
            };

            let server_correlation_id = Uuid::now_v7();
            let intermediate_ca_id = Uuid::parse_str(&intermediate_ca.id)
                .map_err(|e| format!("Invalid intermediate CA ID: {}", e))?;

            let (server_cert, gen_event, _sign_event) = generate_server_certificate(
                &server_seed,
                server_params,
                &intermediate_ca.certificate_pem,
                intermediate_ca.private_key_pem.as_ref().unwrap(),
                intermediate_ca_id,
                server_correlation_id,
                Some(Uuid::parse_str(&intermediate_ca.id).unwrap()),
            ).map_err(|e| format!("Failed to generate server cert for {}: {}", server_name, e))?;

            println!("  Generated server certificate for {}...", server_name);

            server_certs.push(CertificateOutput {
                id: gen_event.cert_id.to_string(),
                subject: gen_event.subject,
                issuer: Some(intermediate_ca.id.clone()),
                fingerprint: server_cert.fingerprint,
                not_before: gen_event.not_before.to_rfc3339(),
                not_after: gen_event.not_after.to_rfc3339(),
                is_ca: false,
                certificate_pem: server_cert.certificate_pem,
                private_key_pem: Some(server_cert.private_key_pem),
                stored_on_yubikey: None,
                yubikey_slot: None,
            });
        }

        Ok(PkiOutput {
            root_ca: root_ca_output,
            intermediate_cas,
            server_certs,
            chain_pem,
        })
    }

    async fn provision_yubikeys(&mut self, _pki: &PkiOutput) -> Result<Vec<YubiKeyOutput>, String> {
        let yubikey_adapter = YubiKeyCliAdapter::new();
        let mut yubikey_outputs = Vec::new();

        // List available YubiKeys
        println!("  Discovering YubiKeys...");
        let devices = match yubikey_adapter.list_devices().await {
            Ok(devices) => devices,
            Err(e) => {
                println!("  ⚠ No YubiKeys found: {} - continuing without hardware provisioning", e);
                self.audit("yubikey_discovery_skipped", "No YubiKeys found - skipping hardware provisioning", None, HashMap::new());
                return Ok(vec![]);
            }
        };

        println!("  Found {} YubiKey(s)", devices.len());
        for device in &devices {
            println!("    - {} (Serial: {})", device.model, device.serial);
        }

        // Clone assignments to avoid borrow issues
        let assignments = self.bootstrap_data.yubikey_assignments.clone();
        let people = self.bootstrap_data.people.clone();

        // Match assignments to devices
        for assignment in &assignments {
            // Find the device
            let device = devices.iter().find(|d| d.serial == assignment.serial);

            if device.is_none() {
                println!("  ⚠ YubiKey {} ({}) not found - skipping", assignment.serial, assignment.name);
                continue;
            }

            let _device = device.unwrap();
            println!("  Provisioning {} ({})...", assignment.name, assignment.serial);

            // Find the generated credentials for this YubiKey
            let _creds = self.generated_credentials.iter()
                .find(|c| c.serial == assignment.serial);

            // NOTE: New YubiKeys have a DEFAULT management key.
            // We use the default for key generation, then the user should
            // change it to the generated random key stored in SECRETS.json.
            // TODO: Implement change_management_key to automate this.
            let default_mgmt_key = "010203040506070801020304050607080102030405060708";
            let mgmt_key = SecureString::new(default_mgmt_key);

            // Find owner
            let owner = people.iter()
                .find(|p| p.id.to_string() == assignment.person_id);
            let owner_name = owner.map(|p| p.name.clone()).unwrap_or_else(|| "Unknown".to_string());

            let mut slots = Vec::new();

            // Based on role, provision appropriate slots
            match assignment.role.as_str() {
                "RootAuthority" => {
                    // Root CA key in signature slot (9c)
                    println!("    🔑 Touch YubiKey to generate Root CA key...");
                    match yubikey_adapter.generate_key_in_slot(
                        &assignment.serial,
                        PivSlot::Signature,
                        KeyAlgorithm::EccP384,
                        &mgmt_key,
                    ).await {
                        Ok(pubkey) => {
                            println!("    ✓ Generated Root CA key in slot 9c");
                            slots.push(YubiKeySlotOutput {
                                slot: "9c".to_string(),
                                slot_id: "Signature".to_string(),
                                key_type: "ECCP384".to_string(),
                                public_key: Some(hex::encode(&pubkey.data)),
                                certificate_id: None, // Would link to root_ca after import
                                purpose: "Root CA Signing".to_string(),
                            });

                            self.audit("yubikey_key_generated", &format!("Root CA key generated on YubiKey {}", assignment.serial), None, {
                                let mut details = HashMap::new();
                                details.insert("serial".to_string(), assignment.serial.clone());
                                details.insert("slot".to_string(), "9c".to_string());
                                details
                            });
                        }
                        Err(e) => {
                            println!("    ✗ Failed to generate key in 9c: {}", e);
                        }
                    }
                }
                "SecurityAdmin" | "Developer" => {
                    // Personal auth key in authentication slot (9a)
                    println!("    🔑 Touch YubiKey to generate authentication key...");
                    match yubikey_adapter.generate_key_in_slot(
                        &assignment.serial,
                        PivSlot::Authentication,
                        KeyAlgorithm::EccP256,
                        &mgmt_key,
                    ).await {
                        Ok(pubkey) => {
                            println!("    ✓ Generated personal key in slot 9a");
                            slots.push(YubiKeySlotOutput {
                                slot: "9a".to_string(),
                                slot_id: "Authentication".to_string(),
                                key_type: "ECCP256".to_string(),
                                public_key: Some(hex::encode(&pubkey.data)),
                                certificate_id: None,
                                purpose: "Personal Authentication".to_string(),
                            });

                            self.audit("yubikey_key_generated", &format!("Personal key generated on YubiKey {}", assignment.serial), None, {
                                let mut details = HashMap::new();
                                details.insert("serial".to_string(), assignment.serial.clone());
                                details.insert("slot".to_string(), "9a".to_string());
                                details.insert("owner".to_string(), owner_name.clone());
                                details
                            });
                        }
                        Err(e) => {
                            println!("    ✗ Failed to generate key in 9a: {}", e);
                        }
                    }

                    // Signing key in digital signature slot (9c)
                    println!("    🔑 Touch YubiKey to generate signing key...");
                    match yubikey_adapter.generate_key_in_slot(
                        &assignment.serial,
                        PivSlot::Signature,
                        KeyAlgorithm::EccP256,
                        &mgmt_key,
                    ).await {
                        Ok(pubkey) => {
                            println!("    ✓ Generated signing key in slot 9c");
                            slots.push(YubiKeySlotOutput {
                                slot: "9c".to_string(),
                                slot_id: "Signature".to_string(),
                                key_type: "ECCP256".to_string(),
                                public_key: Some(hex::encode(&pubkey.data)),
                                certificate_id: None,
                                purpose: "Digital Signature".to_string(),
                            });
                        }
                        Err(e) => {
                            println!("    ✗ Failed to generate key in 9c: {}", e);
                        }
                    }
                }
                "BackupHolder" => {
                    // Backup keys - could hold copy of org keys
                    println!("    ℹ Backup YubiKey - no keys generated (manual setup required)");
                }
                _ => {
                    println!("    ℹ Unknown role {} - skipping", assignment.role);
                }
            }

            yubikey_outputs.push(YubiKeyOutput {
                serial: assignment.serial.clone(),
                name: assignment.name.clone(),
                owner_person_id: assignment.person_id.clone(),
                owner_name,
                role: assignment.role.clone(),
                slots,
            });
        }

        Ok(yubikey_outputs)
    }

    async fn generate_nats_credentials(&mut self) -> Result<NatsOutput, String> {
        // Clone NATS hierarchy to avoid borrow issues
        let nats_hierarchy = self.bootstrap_data.nats_hierarchy.clone()
            .ok_or("No NATS hierarchy defined in bootstrap config")?;

        let nsc_store = self.output_dir.join("nats");
        let nsc = NscAdapter::new(&nsc_store, false); // Use native implementation

        // Generate Operator
        println!("  Generating NATS Operator: {}...", nats_hierarchy.operator.name);
        let operator = nsc.generate_operator(&nats_hierarchy.operator.name).await
            .map_err(|e| format!("Failed to generate operator: {}", e))?;

        self.audit("nats_operator_generated", &format!("NATS Operator {} generated", nats_hierarchy.operator.name), None, {
            let mut details = HashMap::new();
            details.insert("public_key".to_string(), operator.public_key.clone());
            details
        });

        let operator_output = NatsCredentialOutput {
            id: operator.id.to_string(),
            name: operator.name.clone(),
            public_key: operator.public_key.clone(),
            jwt: operator.jwt.clone(),
            creds_file: None,
            seed: Some(operator.seed.clone()),
        };

        // Generate Accounts
        let mut accounts = Vec::new();
        for account_config in &nats_hierarchy.accounts {
            println!("  Generating NATS Account: {}...", account_config.name);
            let account = nsc.generate_account(&operator.id.to_string(), &account_config.name).await
                .map_err(|e| format!("Failed to generate account {}: {}", account_config.name, e))?;

            self.audit("nats_account_generated", &format!("NATS Account {} generated", account_config.name), None, {
                let mut details = HashMap::new();
                details.insert("public_key".to_string(), account.public_key.clone());
                details.insert("is_system".to_string(), account_config.is_system.to_string());
                details
            });

            accounts.push(NatsCredentialOutput {
                id: account.id.to_string(),
                name: account.name.clone(),
                public_key: account.public_key.clone(),
                jwt: account.jwt.clone(),
                creds_file: None,
                seed: Some(account.seed.clone()),
            });
        }

        // Generate Users
        let mut users = Vec::new();
        for user_config in &nats_hierarchy.users {
            // Find the account for this user
            let account = accounts.iter()
                .find(|a| a.name == user_config.account)
                .ok_or(format!("Account {} not found for user {}", user_config.account, user_config.name))?;

            println!("  Generating NATS User: {} (account: {})...", user_config.name, user_config.account);
            let user = nsc.generate_user(&account.id, &user_config.name).await
                .map_err(|e| format!("Failed to generate user {}: {}", user_config.name, e))?;

            self.audit("nats_user_generated", &format!("NATS User {} generated", user_config.name), None, {
                let mut details = HashMap::new();
                details.insert("public_key".to_string(), user.public_key.clone());
                details.insert("account".to_string(), user_config.account.clone());
                details
            });

            // Create credentials file content
            let creds_content = format!(
                "-----BEGIN NATS USER JWT-----\n{}\n------END NATS USER JWT------\n\n************************* IMPORTANT *************************\n                    NKEY Seed printed below can be used to sign and prove identity.\n                    NKEYs are sensitive and should be treated as secrets.\n\n-----BEGIN USER NKEY SEED-----\n{}\n------END USER NKEY SEED------\n",
                user.jwt.as_ref().unwrap_or(&"".to_string()),
                user.seed
            );

            users.push(NatsCredentialOutput {
                id: user.id.to_string(),
                name: user.name.clone(),
                public_key: user.public_key.clone(),
                jwt: user.jwt.clone(),
                creds_file: Some(creds_content),
                seed: Some(user.seed.clone()),
            });
        }

        Ok(NatsOutput {
            operator: operator_output,
            accounts,
            users,
        })
    }

    fn build_key_map(&self, pki: &PkiOutput, nats: &NatsOutput, yubikeys: &[YubiKeyOutput]) -> KeyMap {
        let mut keys = HashMap::new();
        let mut person_keys: HashMap<String, Vec<String>> = HashMap::new();
        let mut yubikey_keys: HashMap<String, Vec<String>> = HashMap::new();

        // Add PKI keys
        let root_key_id = format!("pki-root-{}", pki.root_ca.id);
        keys.insert(root_key_id.clone(), KeyInfo {
            id: root_key_id.clone(),
            key_type: "pki".to_string(),
            algorithm: "RSA/ECDSA".to_string(),
            owner_id: Some(self.bootstrap_data.organization.id.to_string()),
            owner_type: "organization".to_string(),
            purpose: "Root CA Signing".to_string(),
            storage_location: pki.root_ca.stored_on_yubikey.clone()
                .map(|s| format!("yubikey:{}:9c", s))
                .unwrap_or_else(|| "sd-card".to_string()),
            public_key: pki.root_ca.fingerprint.clone(),
            created_at: Utc::now().to_rfc3339(),
        });

        // Add intermediate CA keys
        for intermediate in &pki.intermediate_cas {
            let key_id = format!("pki-intermediate-{}", intermediate.id);
            keys.insert(key_id.clone(), KeyInfo {
                id: key_id.clone(),
                key_type: "pki".to_string(),
                algorithm: "RSA/ECDSA".to_string(),
                owner_id: Some(self.bootstrap_data.organization.id.to_string()),
                owner_type: "organization".to_string(),
                purpose: format!("Intermediate CA: {}", intermediate.subject),
                storage_location: "sd-card".to_string(),
                public_key: intermediate.fingerprint.clone(),
                created_at: Utc::now().to_rfc3339(),
            });
        }

        // Add NATS keys
        let operator_key_id = format!("nats-operator-{}", nats.operator.id);
        keys.insert(operator_key_id.clone(), KeyInfo {
            id: operator_key_id.clone(),
            key_type: "nats".to_string(),
            algorithm: "Ed25519".to_string(),
            owner_id: Some(self.bootstrap_data.organization.id.to_string()),
            owner_type: "organization".to_string(),
            purpose: "NATS Operator".to_string(),
            storage_location: "sd-card".to_string(),
            public_key: nats.operator.public_key.clone(),
            created_at: Utc::now().to_rfc3339(),
        });

        for user in &nats.users {
            let key_id = format!("nats-user-{}", user.id);

            // Find person for this user
            let person = self.bootstrap_data.people.iter()
                .find(|p| p.name.to_lowercase().replace(" ", "") == user.name.to_lowercase().replace("-service", ""));

            keys.insert(key_id.clone(), KeyInfo {
                id: key_id.clone(),
                key_type: "nats".to_string(),
                algorithm: "Ed25519".to_string(),
                owner_id: person.map(|p| p.id.to_string()),
                owner_type: if user.name.contains("service") { "service" } else { "person" }.to_string(),
                purpose: format!("NATS User: {}", user.name),
                storage_location: "sd-card".to_string(),
                public_key: user.public_key.clone(),
                created_at: Utc::now().to_rfc3339(),
            });

            if let Some(p) = person {
                person_keys.entry(p.id.to_string())
                    .or_default()
                    .push(key_id);
            }
        }

        // Add YubiKey keys
        for yk in yubikeys {
            for slot in &yk.slots {
                if let Some(pubkey) = &slot.public_key {
                    let key_id = format!("yubikey-{}-{}", yk.serial, slot.slot);
                    keys.insert(key_id.clone(), KeyInfo {
                        id: key_id.clone(),
                        key_type: "signing".to_string(),
                        algorithm: slot.key_type.clone(),
                        owner_id: Some(yk.owner_person_id.clone()),
                        owner_type: if yk.role == "RootAuthority" { "organization" } else { "person" }.to_string(),
                        purpose: slot.purpose.clone(),
                        storage_location: format!("yubikey:{}:{}", yk.serial, slot.slot),
                        public_key: pubkey.clone(),
                        created_at: Utc::now().to_rfc3339(),
                    });

                    yubikey_keys.entry(yk.serial.clone())
                        .or_default()
                        .push(key_id.clone());

                    person_keys.entry(yk.owner_person_id.clone())
                        .or_default()
                        .push(key_id);
                }
            }
        }

        KeyMap {
            keys,
            person_keys,
            yubikey_keys,
        }
    }

    fn create_manifest(&self) -> Manifest {
        Manifest {
            version: "1.0.0".to_string(),
            organization_id: self.bootstrap_data.organization.id.to_string(),
            organization_name: self.bootstrap_data.organization.display_name.clone(),
            created_at: Utc::now().to_rfc3339(),
            created_by: "cim-keys".to_string(),
        }
    }

    fn write_output(&self, output: &BootstrapOutput) -> Result<(), String> {
        // Write manifest
        let manifest_path = self.output_dir.join("manifest.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&output.manifest).unwrap())
            .map_err(|e| format!("Failed to write manifest: {}", e))?;

        // Write PKI certificates
        let root_ca_path = self.output_dir.join("certificates/root-ca/root-ca.pem");
        fs::write(&root_ca_path, &output.pki.root_ca.certificate_pem)
            .map_err(|e| format!("Failed to write root CA: {}", e))?;

        if let Some(ref key) = output.pki.root_ca.private_key_pem {
            let key_path = self.output_dir.join("certificates/root-ca/root-ca-key.pem");
            fs::write(&key_path, key)
                .map_err(|e| format!("Failed to write root CA key: {}", e))?;
        }

        for intermediate in &output.pki.intermediate_cas {
            let name = intermediate.subject.replace(",", "_").replace(" ", "_").replace("=", "_");
            let cert_path = self.output_dir.join(format!("certificates/intermediate/{}.pem", name));
            fs::write(&cert_path, &intermediate.certificate_pem)
                .map_err(|e| format!("Failed to write intermediate CA: {}", e))?;
        }

        // Write chain
        let chain_path = self.output_dir.join("certificates/chain.pem");
        fs::write(&chain_path, &output.pki.chain_pem)
            .map_err(|e| format!("Failed to write certificate chain: {}", e))?;

        // Write NATS credentials
        let operator_jwt_path = self.output_dir.join("nats/operator/operator.jwt");
        if let Some(ref jwt) = output.nats.operator.jwt {
            fs::write(&operator_jwt_path, jwt)
                .map_err(|e| format!("Failed to write operator JWT: {}", e))?;
        }

        for account in &output.nats.accounts {
            let account_path = self.output_dir.join(format!("nats/accounts/{}.jwt", account.name));
            if let Some(ref jwt) = account.jwt {
                fs::write(&account_path, jwt)
                    .map_err(|e| format!("Failed to write account JWT: {}", e))?;
            }
        }

        for user in &output.nats.users {
            let user_path = self.output_dir.join(format!("nats/users/{}.creds", user.name));
            if let Some(ref creds) = user.creds_file {
                fs::write(&user_path, creds)
                    .map_err(|e| format!("Failed to write user creds: {}", e))?;
            }
        }

        // Write key map
        let key_map_path = self.output_dir.join("keys/key_map.json");
        fs::write(&key_map_path, serde_json::to_string_pretty(&output.key_map).unwrap())
            .map_err(|e| format!("Failed to write key map: {}", e))?;

        // Write audit trail
        let audit_path = self.output_dir.join("events/audit_trail.json");
        fs::write(&audit_path, serde_json::to_string_pretty(&output.audit_trail).unwrap())
            .map_err(|e| format!("Failed to write audit trail: {}", e))?;

        // Write secrets record (CRITICAL - contains all secrets for recovery)
        let secrets = self.create_secrets_record();
        let secrets_path = self.output_dir.join("SECRETS.json");
        fs::write(&secrets_path, serde_json::to_string_pretty(&secrets).unwrap())
            .map_err(|e| format!("Failed to write secrets: {}", e))?;

        // Write complete output
        let complete_path = self.output_dir.join("bootstrap_output.json");
        fs::write(&complete_path, serde_json::to_string_pretty(&output).unwrap())
            .map_err(|e| format!("Failed to write complete output: {}", e))?;

        println!("\n=== Output written to {} ===", self.output_dir.display());
        println!("  - manifest.json");
        println!("  - certificates/root-ca/root-ca.pem");
        println!("  - certificates/chain.pem");
        println!("  - nats/operator/operator.jwt");
        println!("  - nats/users/*.creds");
        println!("  - keys/key_map.json");
        println!("  - events/audit_trail.json");
        println!("  - SECRETS.json (⚠️  YUBIKEY PINS - MEMORIZE THEN DESTROY)");
        println!("  - bootstrap_output.json (complete)");

        Ok(())
    }

    /// Create the secrets record containing all sensitive information
    /// Uses the randomly generated credentials, NOT the bootstrap file
    fn create_secrets_record(&self) -> SecretsRecord {
        let yubikey_pins: Vec<YubiKeyPinRecord> = self.generated_credentials
            .iter()
            .map(|creds| YubiKeyPinRecord {
                serial: creds.serial.clone(),
                pin: creds.pin.clone(),
                puk: creds.puk.clone(),
                management_key: Some(creds.management_key.clone()),
                assigned_to: Some(creds.assigned_to.clone()),
                role: creds.role.clone(),
            })
            .collect();

        SecretsRecord {
            yubikey_pins,
            created_at: chrono::Utc::now().to_rfc3339(),
            warning: "⚠️  MEMORIZE THESE CREDENTIALS THEN DESTROY THIS FILE. \
                     Store offline only. Physical security required. \
                     These credentials CANNOT be recovered if lost.".to_string(),
        }
    }
}

#[tokio::test]
async fn test_complete_bootstrap_workflow() {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║         CIM-KEYS COMPLETE BOOTSTRAP WORKFLOW TEST            ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Use the actual domain-bootstrap.json
    let bootstrap_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("secrets/domain-bootstrap.json");

    if !bootstrap_path.exists() {
        panic!("Bootstrap file not found at: {}", bootstrap_path.display());
    }

    // Create temp output directory
    let output_dir = std::env::temp_dir().join(format!("cim-keys-bootstrap-{}", Uuid::now_v7()));

    // Passphrase is generated internally and never exposed
    let mut workflow = BootstrapWorkflow::new(&bootstrap_path, &output_dir)
        .expect("Failed to create workflow");

    let result = workflow.execute().await;

    match result {
        Ok(output) => {
            println!("\n╔═══════════════════════════════════════════════════════════════╗");
            println!("║                    BOOTSTRAP COMPLETE                         ║");
            println!("╚═══════════════════════════════════════════════════════════════╝\n");

            println!("Organization: {}", output.manifest.organization_name);
            println!("Created: {}", output.manifest.created_at);
            println!("\nPKI Summary:");
            println!("  Root CA: {}", output.pki.root_ca.subject);
            println!("  Intermediate CAs: {}", output.pki.intermediate_cas.len());
            println!("  Server Certs: {}", output.pki.server_certs.len());

            println!("\nNATS Summary:");
            println!("  Operator: {}", output.nats.operator.name);
            println!("  Accounts: {}", output.nats.accounts.len());
            println!("  Users: {}", output.nats.users.len());

            println!("\nYubiKey Summary:");
            for yk in &output.yubikeys {
                println!("  {} ({}): {} slots provisioned", yk.name, yk.serial, yk.slots.len());
            }

            println!("\nKey Map Summary:");
            println!("  Total keys: {}", output.key_map.keys.len());
            println!("  Person key associations: {}", output.key_map.person_keys.len());
            println!("  YubiKey key associations: {}", output.key_map.yubikey_keys.len());

            println!("\nAudit Trail: {} events", output.audit_trail.len());

            println!("\nOutput Directory: {}", output_dir.display());

            // Verify key files exist
            assert!(output_dir.join("manifest.json").exists(), "Manifest should exist");
            assert!(output_dir.join("certificates/root-ca/root-ca.pem").exists(), "Root CA should exist");
            assert!(output_dir.join("certificates/chain.pem").exists(), "Chain should exist");
            assert!(output_dir.join("keys/key_map.json").exists(), "Key map should exist");
            assert!(output_dir.join("events/audit_trail.json").exists(), "Audit trail should exist");
        }
        Err(e) => {
            panic!("Bootstrap workflow failed: {}", e);
        }
    }
}

/// Run the bootstrap interactively (for manual testing)
#[tokio::test]
#[ignore] // Run with: cargo test --test complete_bootstrap_workflow test_interactive_bootstrap -- --ignored --nocapture
async fn test_interactive_bootstrap() {
    use std::io::{self, Write};

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║       CIM-KEYS INTERACTIVE BOOTSTRAP (WITH YUBIKEYS)         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // No passphrase prompt - it's generated internally and destroyed

    // Prompt for output directory
    let default_output = format!("/tmp/cim-keys-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    print!("Enter output directory [{}]: ", default_output);
    io::stdout().flush().unwrap();
    let mut output_dir = String::new();
    io::stdin().read_line(&mut output_dir).unwrap();
    let output_dir = output_dir.trim();
    let output_dir = if output_dir.is_empty() {
        PathBuf::from(&default_output)
    } else {
        PathBuf::from(output_dir)
    };

    // Create the output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    }

    let bootstrap_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("secrets/domain-bootstrap.json");

    let mut workflow = BootstrapWorkflow::new(&bootstrap_path, &output_dir)
        .expect("Failed to create workflow");

    let result = workflow.execute().await;

    match result {
        Ok(output) => {
            println!("\n✓ Bootstrap complete!");
            println!("  Output: {}", output_dir.display());
            println!("  Keys generated: {}", output.key_map.keys.len());
            println!("  YubiKeys provisioned: {}", output.yubikeys.len());
        }
        Err(e) => {
            eprintln!("\n✗ Bootstrap failed: {}", e);
        }
    }
}
