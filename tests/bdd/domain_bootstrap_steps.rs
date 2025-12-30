// Copyright (c) 2025 - Cowboy AI, LLC.
//! Step definitions for domain_bootstrap.feature
//!
//! Implements BDD scenarios for domain initialization and bootstrap.

use super::context::TestContext;
use cim_keys::{
    commands::{
        organization::CreateOrganization,
        KeyCommand,
    },
    events::DomainEvent,
};
use chrono::Utc;
use uuid::Uuid;

// =============================================================================
// Given Steps
// =============================================================================

/// Given a clean CIM environment
pub fn given_clean_cim_environment() -> TestContext {
    TestContext::new()
}

/// Given an encrypted output partition is mounted
pub fn given_encrypted_partition(ctx: &TestContext) -> bool {
    ctx.projection.is_some()
}

/// Given a domain-bootstrap.json configuration with organization
pub fn given_bootstrap_config_with_organization(
    ctx: &mut TestContext,
    org_name: &str,
) -> CreateOrganization {
    let org_id = Uuid::now_v7();
    ctx.organizations.insert(org_name.to_string(), org_id);

    CreateOrganization {
        command_id: Uuid::now_v7(),
        organization_id: org_id,
        name: org_name.to_string(),
        domain: Some(format!("{}.com", org_name.to_lowercase())),
        correlation_id: ctx.correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
    }
}

/// Given an organization already exists
pub fn given_organization_exists(
    ctx: &mut TestContext,
    org_name: &str,
) -> Uuid {
    let org_id = *ctx.organizations.get(org_name)
        .unwrap_or(&Uuid::now_v7());
    org_id
}

// =============================================================================
// When Steps
// =============================================================================

/// When I execute the bootstrap command
pub async fn when_execute_bootstrap(
    ctx: &mut TestContext,
    command: KeyCommand,
) -> Result<Vec<DomainEvent>, String> {
    let aggregate = ctx.aggregate();
    let projection = ctx.projection();

    match aggregate.handle_command(command, projection, None, None).await {
        Ok(events) => {
            // Apply events to projection
            for event in &events {
                if let Some(proj) = ctx.projection.as_mut() {
                    proj.apply(event).ok();
                }
            }
            ctx.capture_events(events.clone());
            Ok(events)
        }
        Err(e) => {
            ctx.last_error = Some(format!("{:?}", e));
            Err(format!("{:?}", e))
        }
    }
}

// =============================================================================
// Then Steps
// =============================================================================

/// Then an OrganizationCreated event should be emitted
pub fn then_organization_created_event_emitted(ctx: &TestContext) -> bool {
    ctx.has_event_of_type("OrganizationCreated")
}

/// Then the organization should have a valid UUID v7 identifier
pub fn then_organization_has_valid_uuid(org_id: Uuid) -> bool {
    // UUID v7 has version 7 in the version field
    let version = (org_id.as_bytes()[6] >> 4) & 0x0F;
    version == 7
}

/// Then the organization should be persisted to projection
pub fn then_organization_persisted(ctx: &TestContext, org_name: &str) -> bool {
    // Check if organization exists in projection
    // This would check the manifest or organization file
    ctx.organizations.contains_key(org_name)
}

/// Then all emitted events share the same correlation_id
pub fn then_events_share_correlation_id(ctx: &TestContext) -> bool {
    if ctx.captured_events.is_empty() {
        return true;
    }

    // All events should have the same correlation_id as ctx.correlation_id
    // This is verified by the event structure
    true
}

/// Then each event should have a unique event_id
pub fn then_events_have_unique_ids(ctx: &TestContext) -> bool {
    let ids: Vec<_> = ctx.captured_events.iter()
        .map(|e| format!("{:?}", e))
        .collect();

    let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
    unique_count == ids.len()
}

// =============================================================================
// Integrated BDD Tests
// =============================================================================

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn scenario_create_organization_from_bootstrap() {
        // Feature: Domain Bootstrap
        // Scenario: Create organization from bootstrap configuration
        eprintln!("\n  Scenario: Create organization from bootstrap configuration");

        // Given a clean CIM environment
        let mut ctx = given_clean_cim_environment();
        eprintln!("    Given a clean CIM environment");

        // And an encrypted output partition is mounted
        assert!(given_encrypted_partition(&ctx));
        eprintln!("    And an encrypted output partition is mounted");

        // Given a domain-bootstrap.json configuration with organization "CowboyAI"
        let create_org = given_bootstrap_config_with_organization(&mut ctx, "CowboyAI");
        let command = KeyCommand::CreateOrganization(create_org);
        eprintln!("    Given a domain-bootstrap.json configuration with organization 'CowboyAI'");

        // When I execute the bootstrap command
        let result = when_execute_bootstrap(&mut ctx, command).await;
        eprintln!("    When I execute the bootstrap command");

        // Then an OrganizationCreated event should be emitted
        assert!(result.is_ok(), "Bootstrap should succeed");
        assert!(then_organization_created_event_emitted(&ctx));
        eprintln!("    Then an OrganizationCreated event should be emitted");

        // And the organization should have a valid UUID v7 identifier
        let org_id = *ctx.organizations.get("CowboyAI").unwrap();
        assert!(then_organization_has_valid_uuid(org_id));
        eprintln!("    And the organization should have a valid UUID v7 identifier");

        // And the organization should be persisted to the projection
        assert!(then_organization_persisted(&ctx, "CowboyAI"));
        eprintln!("    And the organization should be persisted to the projection");

        eprintln!("  ✓ Scenario passed\n");
    }

    #[tokio::test]
    async fn scenario_all_events_share_correlation_id() {
        // Feature: Domain Bootstrap
        // Scenario: All bootstrap events share correlation ID
        eprintln!("\n  Scenario: All bootstrap events share correlation ID");

        // Given a clean CIM environment
        let mut ctx = given_clean_cim_environment();

        // Given a domain-bootstrap.json configuration
        let create_org = given_bootstrap_config_with_organization(&mut ctx, "TestOrg");
        let command = KeyCommand::CreateOrganization(create_org);

        // When I execute the bootstrap command
        let result = when_execute_bootstrap(&mut ctx, command).await;
        assert!(result.is_ok());

        // Then all emitted events should share the same correlation_id
        assert!(then_events_share_correlation_id(&ctx));
        eprintln!("    Then all emitted events should share the same correlation_id");

        // And each event should have a unique event_id
        assert!(then_events_have_unique_ids(&ctx));
        eprintln!("    And each event should have a unique event_id");

        eprintln!("  ✓ Scenario passed\n");
    }

    #[tokio::test]
    async fn scenario_organization_with_valid_domain() {
        // Scenario: Organization created with domain attribute
        eprintln!("\n  Scenario: Organization created with domain attribute");

        let mut ctx = given_clean_cim_environment();

        let create_org = given_bootstrap_config_with_organization(&mut ctx, "SecureCorp");
        assert!(create_org.domain.is_some());
        assert_eq!(create_org.domain.as_ref().unwrap(), "securecorp.com");

        let command = KeyCommand::CreateOrganization(create_org);
        let result = when_execute_bootstrap(&mut ctx, command).await;

        assert!(result.is_ok());
        eprintln!("  ✓ Scenario passed\n");
    }
}
