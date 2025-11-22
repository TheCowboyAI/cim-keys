//! Example: NATS Integration with IPLD/CID Support
//!
//! Demonstrates:
//! - Connecting to NATS server
//! - Publishing events with content addressing
//! - Offline queue management
//! - JetStream persistence
//!
//! Usage:
//! ```bash
//! # Start local NATS server with JetStream
//! nats-server -js
//!
//! # Run this example
//! cargo run --example nats_integration --features nats-client,ipld
//! ```

use cim_keys::{
    adapters::NatsClientAdapter,
    config::NatsConfig,
    events::*,
};
use std::path::PathBuf;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 CIM-Keys NATS Integration Example\n");

    // Create NATS configuration
    let mut config = NatsConfig::default();
    config.enabled = true;
    config.url = "nats://localhost:4222".to_string();
    config.enable_jetstream = true;
    config.enable_ipld = true;

    println!("📋 Configuration:");
    println!("  NATS URL: {}", config.url);
    println!("  JetStream: {}", config.enable_jetstream);
    println!("  IPLD/CID: {}", config.enable_ipld);
    println!("  Subject Prefix: {}\n", config.subject_prefix);

    // Create adapter with offline queue
    let queue_path = PathBuf::from("./nats-queue.json");
    let adapter = NatsClientAdapter::new(config, queue_path.clone());

    // Try to connect to NATS
    println!("🔌 Connecting to NATS...");
    match adapter.connect().await {
        Ok(_) => println!("✅ Connected to NATS server\n"),
        Err(e) => {
            println!("⚠️  Could not connect to NATS: {}", e);
            println!("📦 Operating in offline mode - events will be queued\n");
        }
    }

    // Create some example events
    println!("📝 Creating example events...\n");

    // 1. Organization created
    let org_event = KeyEvent::OrganizationCreated(OrganizationCreatedEvent {
        organization_id: Uuid::now_v7(),
        name: "Acme Corp".to_string(),
        domain: Some("acme.com".to_string()),
        created_at: chrono::Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    });

    println!("1️⃣  Publishing: OrganizationCreated");
    adapter.publish_event(&org_event).await?;

    // 2. Person created
    let person_event = KeyEvent::PersonCreated(PersonCreatedEvent {
        person_id: Uuid::now_v7(),
        name: "Alice Smith".to_string(),
        email: "alice@acme.com".to_string(),
        title: Some("Security Engineer".to_string()),
        department: Some("InfoSec".to_string()),
        organization_id: Some(Uuid::now_v7()),
        created_at: chrono::Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    });

    println!("2️⃣  Publishing: PersonCreated");
    adapter.publish_event(&person_event).await?;

    // 3. Key generated
    let key_event = KeyEvent::KeyGenerated(KeyGeneratedEvent {
        key_id: Uuid::now_v7(),
        algorithm: KeyAlgorithm::Ed25519,
        purpose: KeyPurpose::Signing,
        generated_at: chrono::Utc::now(),
        generated_by: "alice@acme.com".to_string(),
        hardware_backed: false,
        metadata: KeyMetadata {
            label: "Alice's Signing Key".to_string(),
            description: Some("Primary signing key for code commits".to_string()),
            tags: vec!["signing".to_string(), "ed25519".to_string()],
            attributes: std::collections::HashMap::new(),
            jwt_kid: None,
            jwt_alg: None,
            jwt_use: None,
        },
        owner: None,
    });

    println!("3️⃣  Publishing: KeyGenerated");
    adapter.publish_event(&key_event).await?;

    // 4. NATS operator created
    let nats_event = KeyEvent::NatsOperatorCreated(NatsOperatorCreatedEvent {
        operator_id: Uuid::now_v7(),
        name: "acme-operator".to_string(),
        public_key: "OABC123XYZ...".to_string(),
        created_at: chrono::Utc::now(),
        created_by: "alice@acme.com".to_string(),
        organization_id: Some(Uuid::now_v7()),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    });

    println!("4️⃣  Publishing: NatsOperatorCreated\n");
    adapter.publish_event(&nats_event).await?;

    // Show queue status
    let queue_size = adapter.queue_size().await;
    println!("📊 Status:");
    println!("  Connected: {}", adapter.is_connected().await);
    println!("  Queue Size: {}", queue_size);

    if queue_size > 0 {
        println!("\n💾 Events are queued locally at: {}", queue_path.display());
        println!("   They will be published when NATS is available");

        // Demonstrate queue persistence
        println!("\n📦 Attempting to flush queue...");
        let flushed = adapter.flush_queue().await?;
        println!("   Flushed {} events", flushed);
        println!("   {} events remain in queue", adapter.queue_size().await);
    }

    // Demonstrate IPLD/CID verification
    #[cfg(feature = "ipld")]
    {
        println!("\n🔍 IPLD/CID Verification:");
        use cim_keys::ipld_support::ContentAddressedEvent;

        let ca_event = ContentAddressedEvent::new(org_event.clone())?;
        println!("  Event CID: {}", ca_event.cid);
        println!("  Verified: {}", ca_event.verify()?);

        // Demonstrate deterministic CID
        let ca_event2 = ContentAddressedEvent::new(org_event)?;
        println!("  Deterministic: {}", ca_event.cid == ca_event2.cid);
    }

    println!("\n✅ Example complete!");
    println!("\n💡 Subject Pattern Examples:");
    println!("   cim.keys.organization.organizationcreated");
    println!("   cim.keys.person.personcreated");
    println!("   cim.keys.key.keygenerated");
    println!("   cim.keys.nats.operator.natsoperatorcreated");

    Ok(())
}
