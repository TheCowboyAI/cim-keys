//! CIM Keys GUI - Native/WASM application for offline key generation

#[cfg(feature = "gui")]
use cim_keys::{gui, config::{Config, NATS_URL}};

#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Parser)]
#[command(name = "cim-keys-gui")]
#[command(about = "CIM Keys graphical interface for offline key generation")]
#[command(version)]
struct Cli {
    /// Output directory for generated keys
    #[arg(default_value = "./cim-keys-output")]
    output_dir: String,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> iced::Result {
    let cli = Cli::parse();

    // Set up logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Load configuration if provided
    let config = if let Some(config_path) = cli.config {
        match Config::from_file(&config_path) {
            Ok(cfg) => {
                tracing::info!("Loaded configuration from: {}", config_path.display());

                // Validate configuration
                if let Err(e) = cfg.validate() {
                    eprintln!("❌ Configuration validation failed: {}", e);
                    std::process::exit(2);
                }

                tracing::info!("Configuration valid - Mode: {:?}", cfg.mode);
                Some(cfg)
            }
            Err(e) => {
                eprintln!("❌ Failed to load configuration: {}", e);
                eprintln!("   Try: cim-keys create-example-config");
                std::process::exit(2);
            }
        }
    } else {
        // Try to load default config.toml if it exists
        let default_path = std::path::PathBuf::from("config.toml");
        if default_path.exists() {
            match Config::from_file(&default_path) {
                Ok(cfg) => {
                    tracing::info!("Loaded default configuration from config.toml");
                    cfg.validate().ok(); // Warn but don't fail on validation
                    Some(cfg)
                }
                Err(_) => None
            }
        } else {
            tracing::warn!("No configuration file found - using defaults");
            None
        }
    };

    // Use config storage path if available, otherwise use CLI arg
    let output_dir = if let Some(ref cfg) = config {
        cfg.storage.keys_output_dir.to_string_lossy().to_string()
    } else {
        cli.output_dir.clone()
    };

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    println!("🔐 [CIM Keys] - Offline Domain Bootstrap");
    println!("📁 [Output] Directory: {}", output_dir);

    if let Some(ref cfg) = config {
        println!("⚙️  [Mode] {:?}", cfg.mode);
        if cfg.nats.enabled {
            println!("📡 [NATS] Enabled - events will be published to {}", NATS_URL);
        } else {
            println!("📴 [NATS] Disabled - offline mode");
        }
    }

    println!("⚠️  [WARNING] Ensure this computer is air-gapped for secure key generation!");
    println!();

    // Run the GUI (iced manages its own async runtime internally)
    gui::run(output_dir, config)
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Set up panic hook for better error messages in browser
    console_error_panic_hook::set_once();

    // Set up logging to browser console
    wasm_logger::init(wasm_logger::Config::default());

    // For WASM, we use a fixed output location
    let output_dir = String::from("/cim-keys-output");

    // Start the application (WASM doesn't use config files)
    wasm_bindgen_futures::spawn_local(async {
        gui::run(output_dir, None).await.expect("Failed to run GUI");
    });
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("GUI feature not enabled. Build with --features gui");
    std::process::exit(1);
}