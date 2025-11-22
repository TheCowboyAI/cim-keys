//! CIM Keys CLI - Offline NATS infrastructure bootstrap tool
//!
//! This CLI generates complete NATS infrastructure (Operator, Accounts, Users)
//! from organizational domain data, designed for air-gapped operation.

use clap::{Parser, Subcommand};
use cim_keys::{
    Organization, Person,
    domain_projections::NatsProjection,
};
use std::fs;
use std::path::PathBuf;
use serde_json;

#[derive(Parser)]
#[command(name = "cim-keys")]
#[command(about = "CIM offline NATS infrastructure bootstrap tool", long_about = None)]
#[command(version)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bootstrap complete NATS infrastructure from domain configuration
    ///
    /// Reads organization and people data, generates all NATS identities
    /// (Operator, Accounts, Users), and writes credentials to output directory.
    Bootstrap {
        /// Path to domain configuration JSON (organization + units)
        #[arg(long, default_value = "secrets/domain-bootstrap.json")]
        domain: PathBuf,

        /// Path to people configuration JSON
        #[arg(long)]
        people: Option<PathBuf>,

        /// Output directory for NATS credentials
        #[arg(short, long, default_value = "./nats-credentials")]
        output: PathBuf,

        /// Format credentials for import to NATS server
        #[arg(long)]
        nats_format: bool,
    },

    /// List available organizations in domain configuration
    List {
        /// Path to domain configuration JSON
        #[arg(long, default_value = "secrets/domain-bootstrap.json")]
        domain: PathBuf,
    },

    /// Validate domain configuration without generating keys
    Validate {
        /// Path to domain configuration JSON
        #[arg(long, default_value = "secrets/domain-bootstrap.json")]
        domain: PathBuf,

        /// Path to people configuration JSON
        #[arg(long)]
        people: Option<PathBuf>,
    },

    /// Show version and build information
    Version,

    /// Validate configuration file
    ValidateConfig {
        /// Path to configuration file (defaults to --config or config.toml)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Create example configuration file
    CreateExampleConfig {
        /// Output path for example config
        #[arg(short, long, default_value = "config.example.toml")]
        output: PathBuf,
    },

    /// Show current configuration
    ShowConfig {
        /// Path to configuration file (defaults to --config or config.toml)
        #[arg(long)]
        path: Option<PathBuf>,
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

    match cli.command {
        Commands::Bootstrap { domain, people, output, nats_format } => {
            bootstrap_command(domain, people, output, nats_format).await?;
        }

        Commands::List { domain } => {
            list_command(domain).await?;
        }

        Commands::Validate { domain, people } => {
            validate_command(domain, people).await?;
        }

        Commands::Version => {
            println!("cim-keys version {}", cim_keys::VERSION);
            println!("Event-sourced NATS infrastructure bootstrap tool");
        }

        Commands::ValidateConfig { path } => {
            validate_config_command(path.or(cli.config)).await?;
        }

        Commands::CreateExampleConfig { output } => {
            create_example_config_command(output).await?;
        }

        Commands::ShowConfig { path } => {
            show_config_command(path.or(cli.config)).await?;
        }
    }

    Ok(())
}

/// Bootstrap complete NATS infrastructure from domain configuration
async fn bootstrap_command(
    domain_path: PathBuf,
    people_path: Option<PathBuf>,
    output_dir: PathBuf,
    nats_format: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 CIM Keys - NATS Infrastructure Bootstrap");
    println!();

    // Load organization from domain configuration
    println!("📖 Loading domain configuration from: {}", domain_path.display());
    let domain_json = fs::read_to_string(&domain_path)?;

    // Parse the domain configuration
    // For now, expect a simple Organization JSON structure
    let organization: Organization = serde_json::from_str(&domain_json)
        .map_err(|e| format!("Failed to parse organization: {}. \n\nThe domain file should contain an Organization JSON object.", e))?;

    println!("   Organization: {}", organization.name);
    println!("   Units: {}", organization.units.len());

    // Load people (from separate file or embedded in domain config)
    let people: Vec<Person> = if let Some(people_path) = people_path {
        println!("📖 Loading people from: {}", people_path.display());
        let people_json = fs::read_to_string(&people_path)?;
        serde_json::from_str(&people_json)?
    } else {
        // Try to extract from metadata or use empty list
        println!("   No people file specified - using empty list");
        vec![]
    };

    println!("   People: {}", people.len());
    println!();

    // Generate complete NATS infrastructure
    println!("🔑 Generating NATS identities...");
    let bootstrap = NatsProjection::bootstrap_organization(&organization, &people);

    println!("   ✓ Operator: {}", organization.name);
    println!("   ✓ Accounts: {}", bootstrap.accounts.len());
    println!("   ✓ Users: {}", bootstrap.users.len());
    println!("   ✓ Total identities: {}", bootstrap.total_identities());
    println!();

    // Create output directory
    fs::create_dir_all(&output_dir)?;
    println!("📁 Writing credentials to: {}", output_dir.display());

    // Write operator JWT
    let operator_path = output_dir.join("operator.jwt");
    fs::write(&operator_path, bootstrap.operator.jwt.token())?;
    println!("   ✓ Operator JWT: {}", operator_path.display());

    // Write operator seed (for backup)
    let operator_seed_path = output_dir.join("operator.nk");
    fs::write(&operator_seed_path, bootstrap.operator.nkey.seed_string())?;
    println!("   ✓ Operator seed: {}", operator_seed_path.display());

    // Create accounts directory
    let accounts_dir = output_dir.join("accounts");
    fs::create_dir_all(&accounts_dir)?;

    // Write account JWTs
    for (_unit_id, (unit, account)) in &bootstrap.accounts {
        let safe_name = unit.name.replace(' ', "_").to_lowercase();
        let account_jwt_path = accounts_dir.join(format!("{}.jwt", safe_name));
        fs::write(&account_jwt_path, account.jwt.token())?;

        let account_seed_path = accounts_dir.join(format!("{}.nk", safe_name));
        fs::write(&account_seed_path, account.nkey.seed_string())?;

        println!("   ✓ Account '{}': {}", unit.name, account_jwt_path.display());
    }

    // Create users directory
    let users_dir = output_dir.join("users");
    fs::create_dir_all(&users_dir)?;

    // Write user credentials
    for (_person_id, (person, user)) in &bootstrap.users {
        if let Some(cred) = &user.credential {
            let safe_name = person.name.replace(' ', "_").to_lowercase();
            let user_creds_path = users_dir.join(format!("{}.creds", safe_name));
            fs::write(&user_creds_path, cred.to_credential_file())?;

            println!("   ✓ User '{}': {}", person.name, user_creds_path.display());
        }
    }

    println!();
    println!("✅ Bootstrap complete!");
    println!();
    println!("📋 Summary:");
    println!("   • {} operator identity", 1);
    println!("   • {} account identities", bootstrap.accounts.len());
    println!("   • {} user identities", bootstrap.users.len());
    println!("   • {} total files written", 2 + (bootstrap.accounts.len() * 2) + bootstrap.users.len());
    println!();
    println!("🔒 Security Notes:");
    println!("   • Store operator.nk and account *.nk files securely (they contain private keys)");
    println!("   • Distribute user *.creds files to respective users via secure channels");
    println!("   • Consider encrypting the entire output directory");
    println!();

    if nats_format {
        println!("📦 Formatting for NATS server import...");
        // TODO: Create NATS server-compatible directory structure
        println!("   (NATS format not yet implemented - use raw credentials)");
    }

    Ok(())
}

/// List organizations in domain configuration
async fn list_command(domain_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Organizations in: {}", domain_path.display());
    println!();

    let domain_json = fs::read_to_string(&domain_path)?;
    let organization: Organization = serde_json::from_str(&domain_json)?;

    println!("Organization: {}", organization.name);
    println!("  ID: {}", organization.id);
    println!("  Display Name: {}", organization.display_name);

    if let Some(desc) = &organization.description {
        println!("  Description: {}", desc);
    }

    println!();
    println!("Organizational Units ({}):", organization.units.len());
    for unit in &organization.units {
        println!("  • {} ({})", unit.name, unit.id);
        println!("    Type: {:?}", unit.unit_type);
    }

    Ok(())
}

/// Validate domain configuration
async fn validate_command(
    domain_path: PathBuf,
    people_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("✓ Validating domain configuration...");
    println!();

    // Load and parse organization
    let domain_json = fs::read_to_string(&domain_path)?;
    let organization: Organization = serde_json::from_str(&domain_json)
        .map_err(|e| format!("Invalid organization JSON: {}", e))?;

    println!("✓ Organization valid: {}", organization.name);
    println!("  • {} units", organization.units.len());

    // Load and validate people if provided
    if let Some(people_path) = people_path {
        let people_json = fs::read_to_string(&people_path)?;
        let people: Vec<Person> = serde_json::from_str(&people_json)
            .map_err(|e| format!("Invalid people JSON: {}", e))?;

        println!("✓ People valid: {} persons", people.len());

        // Check that people reference valid organization
        let mut org_mismatches = 0;
        for person in &people {
            if person.organization_id != organization.id {
                org_mismatches += 1;
            }
        }

        if org_mismatches > 0 {
            println!("⚠  Warning: {} people reference different organization IDs", org_mismatches);
        }
    }

    println!();
    println!("✅ Configuration is valid!");

    Ok(())
}

/// Validate configuration file
async fn validate_config_command(
    config_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use cim_keys::config::Config;

    let path = config_path.unwrap_or_else(|| PathBuf::from("config.toml"));

    println!("🔍 Validating configuration: {}", path.display());
    println!();

    if !path.exists() {
        return Err(format!("Configuration file not found: {}", path.display()).into());
    }

    let config = Config::from_file(&path)?;

    println!("✓ Configuration loaded successfully");
    println!();

    match config.validate() {
        Ok(()) => {
            println!("✅ Configuration is valid!");
            println!();
            println!("📋 Configuration Summary:");
            println!("   • Mode: {:?}", config.mode);
            println!("   • NATS enabled: {}", config.nats.enabled);
            if config.nats.enabled {
                println!("   • NATS URL: {}", cim_keys::config::NATS_URL);
                println!("   • Stream: {}", config.nats.stream_name);
                println!("   • Subject prefix: {}", config.nats.subject_prefix);
            }
            println!("   • Offline events dir: {}", config.storage.offline_events_dir.display());
            println!("   • Keys output dir: {}", config.storage.keys_output_dir.display());
            if config.storage.enable_backup {
                if let Some(backup_dir) = &config.storage.backup_dir {
                    println!("   • Backup dir: {}", backup_dir.display());
                }
            }
        }
        Err(e) => {
            println!("❌ Configuration validation failed:");
            println!("   {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Create example configuration file
async fn create_example_config_command(
    output_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use cim_keys::config::Config;

    println!("📝 Creating example configuration: {}", output_path.display());

    if output_path.exists() {
        return Err(format!("File already exists: {}", output_path.display()).into());
    }

    Config::create_example(&output_path)?;

    println!("✅ Example configuration created!");
    println!();
    println!("📋 Next steps:");
    println!("   1. Copy to config.toml: cp {} config.toml", output_path.display());
    println!("   2. Edit config.toml to match your environment");
    println!("   3. Validate: cim-keys validate-config");
    println!();

    Ok(())
}

/// Show current configuration
async fn show_config_command(
    config_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use cim_keys::config::Config;

    let path = config_path.unwrap_or_else(|| PathBuf::from("config.toml"));

    if !path.exists() {
        return Err(format!("Configuration file not found: {}", path.display()).into());
    }

    let config = Config::from_file(&path)?;

    println!("📋 Configuration from: {}", path.display());
    println!();
    println!("{}", toml::to_string_pretty(&config)?);

    Ok(())
}
