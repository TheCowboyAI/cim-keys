/// MVI Architecture Demonstration
///
/// This example shows how to integrate the MVI (Model-View-Intent) architecture
/// with Iced's Application trait and hexagonal ports.
///
/// Run with: cargo run --example mvi_demo --features gui

use std::sync::Arc;
use iced::{application, Task, Element, Theme};

// Import MVI components
use cim_keys::mvi::{Intent, Model, update, view};

// Import ports
use cim_keys::ports::{StoragePort, X509Port, SshKeyPort, YubiKeyPort};

// Import mock adapters
use cim_keys::adapters::{
    InMemoryStorageAdapter,
    MockX509Adapter,
    MockSshKeyAdapter,
    MockYubiKeyAdapter,
};

/// Application integrating MVI architecture with hexagonal ports
struct MviDemoApp {
    /// Pure immutable state
    model: Model,

    /// Hexagonal ports (dependency injected)
    storage: Arc<dyn StoragePort>,
    x509: Arc<dyn X509Port>,
    ssh: Arc<dyn SshKeyPort>,
    yubikey: Arc<dyn YubiKeyPort>,
}

impl MviDemoApp {
    fn new() -> (Self, Task<Intent>) {
        // Initialize mock adapters (production would use real adapters)
        let storage = Arc::new(InMemoryStorageAdapter::new());
        let x509 = Arc::new(MockX509Adapter::new());
        let ssh = Arc::new(MockSshKeyAdapter::new());
        let yubikey = Arc::new(MockYubiKeyAdapter::default());

        let app = Self {
            model: Model::new("/tmp/cim-keys-mvi-demo".into()),
            storage,
            x509,
            ssh,
            yubikey,
        };

        (app, Task::none())
    }

    fn update(&mut self, intent: Intent) -> Task<Intent> {
        // Call the pure update function with model and ports
        let (updated_model, command) = update(
            self.model.clone(),
            intent,
            self.storage.clone(),
            self.x509.clone(),
            self.ssh.clone(),
            self.yubikey.clone(),
        );

        // Update our model
        self.model = updated_model;

        // Return the command (async operations)
        command
    }

    fn view(&self) -> Element<'_, Intent> {
        // Call the pure view function
        view(&self.model)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn main() -> iced::Result {
    println!("🚀 Starting CIM Keys MVI Demo");
    println!("================================================");
    println!();
    println!("This demo showcases the MVI architecture:");
    println!("  • Intent - Unified event source abstraction");
    println!("  • Model - Pure immutable state");
    println!("  • Update - Pure state transition function");
    println!("  • View - Pure rendering function");
    println!("  • Ports - Hexagonal architecture integration");
    println!();
    println!("Try these workflows:");
    println!("  1. Go to Organization tab → Add people");
    println!("  2. Go to Keys tab → Generate Root CA");
    println!("  3. Generate SSH keys for all users");
    println!("  4. Provision YubiKeys");
    println!("  5. Export to SD card");
    println!();
    println!("All operations flow through the Intent enum!");
    println!("================================================");

    application("CIM Keys - MVI Demo", MviDemoApp::update, MviDemoApp::view)
        .theme(MviDemoApp::theme)
        .run_with(MviDemoApp::new)
}
