//! YubiKey Demo Example - CIM Keys
//!
//! This example demonstrates YubiKey hardware token integration with CIM's key management system.
//! It showcases key listing, generation, and signing operations using a physical YubiKey.

use cim_keys::{
    // Core traits
    HardwareTokenManager,
    
    // YubiKey support
    yubikey::YubiKeyManager,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 CIM Keys - YubiKey Demo\n");
    
    // Initialize YubiKey manager
    println!("=== Initializing YubiKey Manager ===\n");
    let yubikey_manager = match YubiKeyManager::new() {
        Ok(manager) => {
            println!("✅ YubiKey manager initialized successfully");
            manager
        }
        Err(e) => {
            println!("❌ Failed to initialize YubiKey manager: {}", e);
            println!("\nMake sure:");
            println!("  - A YubiKey is connected");
            println!("  - PC/SC daemon (pcscd) is running");
            println!("  - You have the necessary permissions");
            return Err(e.into());
        }
    };
    
    // Find connected YubiKeys
    println!("\n=== Finding YubiKeys ===\n");
    let serials = match yubikey_manager.find_yubikeys() {
        Ok(serials) => {
            if serials.is_empty() {
                println!("❌ No YubiKeys detected");
                println!("\nPlease connect a YubiKey and try again.");
                return Ok(());
            }
            
            println!("✅ Found {} YubiKey(s):", serials.len());
            for (idx, serial) in serials.iter().enumerate() {
                println!("  #{}: Serial {}", idx + 1, serial);
            }
            serials
        }
        Err(e) => {
            println!("❌ Failed to find YubiKeys: {}", e);
            return Err(e.into());
        }
    };
    
    // Connect to the first YubiKey
    let serial = &serials[0];
    println!("\n=== Connecting to YubiKey ===\n");
    println!("Connecting to YubiKey with serial: {}", serial);
    
    match yubikey_manager.connect(*serial) {
        Ok(()) => println!("✅ Successfully connected to YubiKey"),
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            return Err(e.into());
        }
    }
    
    // Get YubiKey info
    match yubikey_manager.get_info(&serial.to_string()) {
        Ok(info) => {
            println!("\nYubiKey Information:");
            println!("  Type: {}", info.token_type);
            println!("  Serial: {}", info.serial_number);
            println!("  Firmware: {}", info.firmware_version);
            println!("  Available Slots: {:?}", info.available_slots);
            println!("  Supported Algorithms: {:?}", info.supported_algorithms);
            if let Some(pin_retries) = info.pin_retries {
                println!("  PIN Retries: {}", pin_retries);
            }
        }
        Err(e) => {
            println!("⚠️  Failed to get YubiKey info: {}", e);
        }
    }
    
    // List hardware tokens (demonstrates the HardwareTokenManager trait)
    println!("\n=== Hardware Tokens ===\n");
    match yubikey_manager.list_tokens().await {
        Ok(tokens) => {
            for token in tokens {
                println!("Token: {}", token.token_type);
                println!("  Serial: {}", token.serial_number);
                println!("  Firmware: {}", token.firmware_version);
                println!("  Available Slots: {:?}", token.available_slots);
                println!("  Supported Algorithms: {:?}", token.supported_algorithms);
                if let Some(pin_retries) = token.pin_retries {
                    println!("  PIN Retries: {}", pin_retries);
                }
                println!();
            }
        }
        Err(e) => {
            println!("⚠️  Failed to list tokens: {}", e);
        }
    }
    
    // Demonstrate PIN authentication (optional)
    println!("\n=== PIN Authentication (Demo) ===\n");
    println!("Note: This would normally prompt for your PIN.");
    println!("For security, we're not actually doing PIN auth in this demo.");
    
    // In a real application, you would:
    // let pin = SecureString::new(prompt_for_pin());
    // yubikey_manager.authenticate_token(&serial.to_string(), pin).await?;
    
    println!("\n=== Demo Complete ===");
    println!("\n🎉 YubiKey demo completed!");
    println!("\nFor production use, consider:");
    println!("  - Implementing proper PIN management");
    println!("  - Using management keys for administrative operations");
    println!("  - Setting up key attestation for trust verification");
    println!("  - Implementing key backup strategies");
    
    Ok(())
} 