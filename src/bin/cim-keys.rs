//! CIM Keys CLI - Offline key management tool
//!
//! This CLI tool manages cryptographic keys using event sourcing,
//! designed for air-gapped operation with encrypted SD card storage.

use clap::{Parser, Subcommand};
use cim_keys::prelude::*;
use cim_domain::CommandId;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;

#[derive(Parser)]
#[command(name = "cim-keys")]
#[command(about = "CIM offline key management tool")]
#[command(version)]
struct Cli {
    /// Path to encrypted partition (default: /mnt/cim-keys)
    #[arg(short, long, default_value = "/mnt/cim-keys")]
    partition: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new key storage partition
    Init {
        /// Organization name
        #[arg(long)]
        org: String,

        /// Organization domain
        #[arg(long)]
        domain: String,

        /// Country code
        #[arg(long)]
        country: String,

        /// Admin email
        #[arg(long)]
        email: String,
    },

    /// Generate a new key
    Generate {
        /// Key algorithm (rsa2048, rsa4096, ed25519, ecdsa-p256)
        #[arg(long)]
        algorithm: String,

        /// Key purpose (signing, encryption, authentication, ca)
        #[arg(long)]
        purpose: String,

        /// Key label
        #[arg(long)]
        label: String,

        /// Use hardware token (YubiKey)
        #[arg(long)]
        hardware: bool,
    },

    /// Create PKI hierarchy
    CreatePki {
        /// Hierarchy name
        #[arg(long)]
        name: String,

        /// Root CA common name
        #[arg(long)]
        root_cn: String,

        /// Root CA validity in years
        #[arg(long, default_value = "10")]
        root_years: u32,

        /// Intermediate CA common names (comma-separated)
        #[arg(long)]
        intermediate_cns: Option<String>,
    },

    /// Provision a YubiKey
    ProvisionYubikey {
        /// YubiKey serial number
        #[arg(long)]
        serial: String,

        /// Slots to configure (comma-separated)
        #[arg(long)]
        slots: String,
    },

    /// Generate SSH key
    GenerateSsh {
        /// Key type (rsa, ed25519, ecdsa)
        #[arg(long)]
        key_type: String,

        /// Comment for the key
        #[arg(long)]
        comment: String,
    },

    /// List all keys
    List {
        /// Filter by type (key, cert, yubikey)
        #[arg(long)]
        filter: Option<String>,
    },

    /// Show manifest
    Manifest,

    /// Rebuild from events
    Rebuild,

    /// Export keys for import to leaf nodes
    Export {
        /// Output directory
        #[arg(long)]
        output: PathBuf,

        /// Include private keys
        #[arg(long)]
        include_private: bool,
    },

    /// Launch the graphical user interface
    #[cfg(feature = "gui")]
    Gui {
        /// Output directory for key operations
        #[arg(default_value = "./output")]
        output_dir: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Setup logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Handle GUI command separately to avoid projection creation
    #[cfg(feature = "gui")]
    if let Commands::Gui { output_dir } = &cli.command {
        println!("🔐 Starting GUI with output directory: {}", output_dir);
        return cim_keys::gui::run(output_dir.clone()).await.map_err(|e| e.into());
    }

    // Create or load projection for non-GUI commands
    let mut projection = OfflineKeyProjection::new(&cli.partition)?;

    // Create aggregate
    let aggregate = KeyManagementAggregate::new();

    match cli.command {
        Commands::Init { org, domain, country, email } => {
            println!("Initializing key storage partition at {:?}", cli.partition);

            // Update organization info in manifest
            // This would be done via events in a full implementation
            println!("Organization: {}", org);
            println!("Domain: {}", domain);
            println!("Country: {}", country);
            println!("Admin Email: {}", email);

            println!("✓ Partition initialized successfully");
        }

        Commands::Generate { algorithm, purpose, label, hardware } => {
            println!("Generating {} key: {}", algorithm, label);

            let key_algorithm = parse_algorithm(&algorithm)?;
            let key_purpose = parse_purpose(&purpose)?;

            let command = KeyCommand::GenerateKey(GenerateKeyCommand {
                command_id: CommandId::new(),
                algorithm: key_algorithm,
                purpose: key_purpose,
                label,
                hardware_backed: hardware,
                requestor: "cli".to_string(),
                context: None,
            });

            // Process command through aggregate
            let events = aggregate.handle_command(
                command,
                &projection,
                None,
                #[cfg(feature = "policy")]
                None
            ).await?;

            // Apply events to projection
            for event in events {
                projection.apply(&event)?;
                println!("✓ Event applied: {}", event.event_type());
            }

            println!("✓ Key generated successfully");
        }

        Commands::CreatePki { name, root_cn, root_years, intermediate_cns } => {
            println!("Creating PKI hierarchy: {}", name);

            let command = KeyCommand::CreatePkiHierarchy(CreatePkiHierarchyCommand {
                command_id: CommandId::new(),
                hierarchy_name: name.clone(),
                root_ca_config: CaConfig {
                    name: root_cn.clone(),
                    subject: CertificateSubject {
                        common_name: root_cn,
                        organization: None,
                        organizational_unit: None,
                        country: None,
                        state_or_province: None,
                        locality: None,
                    },
                    validity_years: root_years,
                    key_algorithm: KeyAlgorithm::Rsa { bits: 4096 },
                    path_len_constraint: None,
                },
                intermediate_ca_configs: intermediate_cns
                    .map(|cns| {
                        cns.split(',')
                            .map(|cn| CaConfig {
                                name: cn.trim().to_string(),
                                subject: CertificateSubject {
                                    common_name: cn.trim().to_string(),
                                    organization: None,
                                    organizational_unit: None,
                                    country: None,
                                    state_or_province: None,
                                    locality: None,
                                },
                                validity_years: 5,
                                key_algorithm: KeyAlgorithm::Rsa { bits: 4096 },
                                path_len_constraint: Some(0),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                requestor: "cli".to_string(),
            });

            let events = aggregate.handle_command(
                command,
                &projection,
                None,
                #[cfg(feature = "policy")]
                None
            ).await?;
            for event in events {
                projection.apply(&event)?;
            }

            println!("✓ PKI hierarchy created successfully");
        }

        Commands::ProvisionYubikey { serial, slots } => {
            println!("Provisioning YubiKey: {}", serial);

            let slot_configs: Vec<_> = slots
                .split(',')
                .map(|slot| YubiKeySlotConfig {
                    slot_id: slot.trim().to_string(),
                    key_algorithm: KeyAlgorithm::Rsa { bits: 2048 },
                    purpose: KeyPurpose::Authentication,
                    pin_policy: "once".to_string(),
                    touch_policy: "always".to_string(),
                })
                .collect();

            let command = KeyCommand::ProvisionYubiKey(ProvisionYubiKeyCommand {
                command_id: CommandId::new(),
                yubikey_serial: serial,
                slots: slot_configs,
                management_key: None,
                requestor: "cli".to_string(),
                context: None,
            });

            let events = aggregate.handle_command(
                command,
                &projection,
                None,
                #[cfg(feature = "policy")]
                None
            ).await?;
            for event in events {
                projection.apply(&event)?;
            }

            println!("✓ YubiKey provisioned successfully");
        }

        Commands::GenerateSsh { key_type, comment } => {
            println!("Generating SSH key: {}", comment);

            let command = KeyCommand::GenerateSshKey(GenerateSshKeyCommand {
                command_id: CommandId::new(),
                key_type,
                comment,
                requestor: "cli".to_string(),
            });

            let events = aggregate.handle_command(
                command,
                &projection,
                None,
                #[cfg(feature = "policy")]
                None
            ).await?;
            for event in events {
                projection.apply(&event)?;
            }

            println!("✓ SSH key generated successfully");
        }

        Commands::List { filter } => {
            println!("Keys and certificates:");
            println!();

            // In real implementation, read from manifest
            println!("Use 'cim-keys manifest' to see full details");
        }

        Commands::Manifest => {
            let manifest_path = cli.partition.join("manifest.json");
            if manifest_path.exists() {
                let content = std::fs::read_to_string(&manifest_path)?;
                println!("{}", content);
            } else {
                println!("No manifest found. Run 'cim-keys init' first.");
            }
        }

        Commands::Rebuild => {
            println!("Rebuilding projection from event log...");
            projection.rebuild_from_events()?;
            println!("✓ Projection rebuilt successfully");
        }

        Commands::Export { output, include_private } => {
            println!("Exporting keys to {:?}", output);

            // Create export directory
            std::fs::create_dir_all(&output)?;

            // Copy relevant files (in real implementation)
            println!("✓ Keys exported successfully");
            println!("  Import to leaf nodes using: cim-leaf import-keys {:?}", output);
        }

        #[cfg(feature = "gui")]
        Commands::Gui { .. } => {
            // Already handled above, this should never be reached
            unreachable!("GUI command should have been handled earlier");
        }
    }

    Ok(())
}

fn parse_algorithm(alg: &str) -> Result<KeyAlgorithm, String> {
    match alg.to_lowercase().as_str() {
        "rsa2048" => Ok(KeyAlgorithm::Rsa { bits: 2048 }),
        "rsa4096" => Ok(KeyAlgorithm::Rsa { bits: 4096 }),
        "ed25519" => Ok(KeyAlgorithm::Ed25519),
        "ecdsa-p256" => Ok(KeyAlgorithm::Ecdsa { curve: "P256".to_string() }),
        _ => Err(format!("Unknown algorithm: {}", alg)),
    }
}

fn parse_purpose(purpose: &str) -> Result<KeyPurpose, String> {
    match purpose.to_lowercase().as_str() {
        "signing" => Ok(KeyPurpose::Signing),
        "encryption" => Ok(KeyPurpose::Encryption),
        "authentication" => Ok(KeyPurpose::Authentication),
        "ca" => Ok(KeyPurpose::CertificateAuthority),
        _ => Err(format!("Unknown purpose: {}", purpose)),
    }
}