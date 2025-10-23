//! CIM Keys GUI - Native/WASM application for offline key generation

#[cfg(feature = "gui")]
use cim_keys::gui;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> iced::Result {
    // Set up logging
    tracing_subscriber::fmt::init();

    // Get output directory from args or use default
    let output_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("./cim-keys-output"));

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    println!("🔐 CIM Keys - Offline Domain Bootstrap");
    println!("📁 Output directory: {}", output_dir);
    println!("⚠️  Ensure this computer is air-gapped!");
    println!();

    // Run the GUI
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(gui::run(output_dir))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Set up panic hook for better error messages in browser
    console_error_panic_hook::set_once();

    // Set up logging to browser console
    wasm_logger::init(wasm_logger::Config::default());

    // For WASM, we use a fixed output location
    let output_dir = String::from("/cim-keys-output");

    // Start the application
    wasm_bindgen_futures::spawn_local(async {
        gui::run(output_dir).await.expect("Failed to run GUI");
    });
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("GUI feature not enabled. Build with --features gui");
    std::process::exit(1);
}