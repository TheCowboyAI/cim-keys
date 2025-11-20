//! Integration Examples: Routing DSL with cim-keys Workflows
//!
//! This example demonstrates how to use the routing DSL to build
//! compositional workflows for key generation, NATS identity management,
//! and other cim-keys operations.

use cim_keys::routing::{Route, RouteBuilder};

// ============================================================================
// Example 1: Key Generation Pipeline
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct KeyRequest {
    organization: String,
    key_type: String,
    strength: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct ValidatedRequest {
    organization: String,
    key_type: String,
    strength: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct KeyMaterial {
    public_key: Vec<u8>,
    key_id: String,
}

#[derive(Debug, Clone, PartialEq)]
struct StoredKey {
    key_id: String,
    location: String,
}

fn validate_key_request(req: KeyRequest) -> ValidatedRequest {
    // In real implementation, would validate organization exists, key type is supported, etc.
    ValidatedRequest {
        organization: req.organization,
        key_type: req.key_type,
        strength: req.strength,
    }
}

fn generate_key_material(req: ValidatedRequest) -> KeyMaterial {
    // Simulate key generation
    KeyMaterial {
        public_key: vec![1, 2, 3, 4], // Mock key bytes
        key_id: format!("{}_{}_key", req.organization, req.key_type),
    }
}

fn store_key_material(material: KeyMaterial) -> StoredKey {
    // Simulate storing to encrypted partition
    StoredKey {
        key_id: material.key_id,
        location: "/mnt/encrypted/keys/".to_string(),
    }
}

fn example_1_key_generation_pipeline() {
    println!("=== Example 1: Key Generation Pipeline ===\n");

    // Build a composable key generation workflow using the routing DSL
    let key_gen_workflow = RouteBuilder::new()
        .then(validate_key_request)
        .then(generate_key_material)
        .then(store_key_material)
        .build();

    let request = KeyRequest {
        organization: "CowboyAI".to_string(),
        key_type: "root_ca".to_string(),
        strength: 4096,
    };

    let result = key_gen_workflow.run(request);
    println!("Generated and stored key: {:?}", result);
    println!();
}

// ============================================================================
// Example 2: NATS Identity Workflow with Fanout
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct NatsIdentityRequest {
    person_name: String,
    role: String,
}

#[derive(Debug, Clone, PartialEq)]
struct OperatorCredential {
    nkey: String,
}

#[derive(Debug, Clone, PartialEq)]
struct UserCredential {
    jwt: String,
}

fn create_operator_credential(req: NatsIdentityRequest) -> OperatorCredential {
    OperatorCredential {
        nkey: format!("OPERATOR_{}", req.person_name.to_uppercase()),
    }
}

fn create_user_credential(req: NatsIdentityRequest) -> UserCredential {
    UserCredential {
        jwt: format!("JWT_USER_{}_{}", req.person_name, req.role),
    }
}

fn example_2_nats_identity_fanout() {
    println!("=== Example 2: NATS Identity Workflow with Fanout ===\n");

    // Use split() to create both operator and user credentials in parallel
    let nats_identity_workflow = RouteBuilder::new()
        .split(
            create_operator_credential,
            create_user_credential
        )
        .build();

    let request = NatsIdentityRequest {
        person_name: "Alice".to_string(),
        role: "admin".to_string(),
    };

    let (operator, user) = nats_identity_workflow.run(request);
    println!("Operator credential: {:?}", operator);
    println!("User credential: {:?}", user);
    println!();
}

// ============================================================================
// Example 3: Multi-Stage Validation and Processing
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct RawInput {
    data: String,
}

#[derive(Debug, Clone, PartialEq)]
struct SanitizedInput {
    data: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ValidatedInput {
    data: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ProcessedOutput {
    data: String,
    hash: String,
}

fn sanitize_input(raw: RawInput) -> SanitizedInput {
    SanitizedInput {
        data: raw.data.trim().to_lowercase(),
    }
}

fn validate_input(sanitized: SanitizedInput) -> ValidatedInput {
    // In real implementation, would check for malicious input
    ValidatedInput {
        data: sanitized.data,
    }
}

fn process_validated_input(validated: ValidatedInput) -> ProcessedOutput {
    ProcessedOutput {
        data: validated.data.clone(),
        hash: format!("HASH_{}", validated.data.len()),
    }
}

fn example_3_multi_stage_processing() {
    println!("=== Example 3: Multi-Stage Validation and Processing ===\n");

    let processing_pipeline = RouteBuilder::new()
        .then(sanitize_input)
        .then(validate_input)
        .then(process_validated_input)
        .build();

    let raw = RawInput {
        data: "  Hello World  ".to_string(),
    };

    let result = processing_pipeline.run(raw);
    println!("Processed: {:?}", result);
    println!();
}

// ============================================================================
// Example 4: Composing Existing Routes
// ============================================================================

fn example_4_route_composition() {
    println!("=== Example 4: Composing Existing Routes ===\n");

    // Create individual routes
    let double = Route::new(|x: i32| x * 2);
    let add_ten = Route::new(|x: i32| x + 10);
    let to_string = Route::new(|x: i32| format!("Result: {}", x));

    // Compose them using the builder
    let workflow = RouteBuilder::from_route(double)
        .compose(add_ten)
        .compose(to_string)
        .build();

    let result = workflow.run(5);
    println!("{}", result); // "Result: 20" (5*2 + 10)
    println!();
}

// ============================================================================
// Example 5: Complex Branching Workflow
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct DataPacket {
    id: u32,
    payload: String,
}

#[derive(Debug, Clone, PartialEq)]
struct DiskReceipt {
    id: u32,
    disk_path: String,
}

#[derive(Debug, Clone, PartialEq)]
struct NatsReceipt {
    id: u32,
    subject: String,
}

fn save_to_disk(packet: DataPacket) -> DiskReceipt {
    DiskReceipt {
        id: packet.id,
        disk_path: format!("/data/{}.json", packet.id),
    }
}

fn publish_to_nats(packet: DataPacket) -> NatsReceipt {
    NatsReceipt {
        id: packet.id,
        subject: format!("events.packet.{}", packet.id),
    }
}

fn merge_receipts((disk, nats): (DiskReceipt, NatsReceipt)) -> String {
    format!(
        "Packet {} saved to {} and published to {}",
        disk.id, disk.disk_path, nats.subject
    )
}

fn example_5_branching_workflow() {
    println!("=== Example 5: Complex Branching Workflow ===\n");

    // Process data and save to both disk and NATS, then merge results
    let workflow = RouteBuilder::new()
        .split(save_to_disk, publish_to_nats)
        .then(merge_receipts)
        .build();

    let packet = DataPacket {
        id: 42,
        payload: "Important data".to_string(),
    };

    let result = workflow.run(packet);
    println!("{}", result);
    println!();
}

// ============================================================================
// Example 6: Builder with run_with() Convenience
// ============================================================================

fn example_6_run_with_convenience() {
    println!("=== Example 6: run_with() Convenience Method ===\n");

    // Build and run in one expression
    let result = RouteBuilder::new()
        .then(|x: i32| x * 2)
        .then(|x: i32| x + 1)
        .then(|x: i32| format!("Answer: {}", x))
        .run_with(20);

    println!("{}", result); // "Answer: 41"
    println!();
}

// ============================================================================
// Main Example Runner
// ============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Routing DSL Integration Examples for cim-keys           ║");
    println!("║  Demonstrating compositional workflows with RouteBuilder ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    example_1_key_generation_pipeline();
    example_2_nats_identity_fanout();
    example_3_multi_stage_processing();
    example_4_route_composition();
    example_5_branching_workflow();
    example_6_run_with_convenience();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  All routing DSL examples completed successfully!        ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
}
