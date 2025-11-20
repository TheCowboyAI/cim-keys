//! Test YubiKey hardware detection
//!
//! Run with: cargo run --example test_yubikey --features yubikey-support

#[cfg(feature = "yubikey-support")]
fn main() {
    println!("Testing YubiKey hardware access...\n");

    // Try to connect to YubiKey
    match yubikey::YubiKey::open() {
        Ok(mut yk) => {
            println!("✅ YubiKey detected!");

            // Get serial (returns Serial type directly, not Result)
            let serial = yk.serial();
            println!("   Serial: {}", serial);

            // Get version
            let version = yk.version();
            println!("   Firmware: {}.{}.{}", version.major, version.minor, version.patch);

            println!("\n✅ YubiKey hardware access working!");
        }
        Err(e) => {
            println!("❌ Failed to open YubiKey: {}", e);
            println!("\nMake sure:");
            println!("  1. YubiKey is plugged in");
            println!("  2. pcscd daemon is running");
            println!("  3. You have permissions to access the smart card");
        }
    }
}

#[cfg(not(feature = "yubikey-support"))]
fn main() {
    println!("yubikey-support feature not enabled");
    println!("Run with: cargo run --example test_yubikey --features yubikey-support");
}
