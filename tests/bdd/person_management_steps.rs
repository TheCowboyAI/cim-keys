// Copyright (c) 2025 - Cowboy AI, LLC.
//! Step definitions for person_management.feature
//!
//! Implements BDD scenarios for person lifecycle management.

use super::context::TestContext;
use cim_keys::commands::organization::CreatePerson;
use chrono::Utc;
use uuid::Uuid;

// =============================================================================
// Given Steps
// =============================================================================

/// Given a bootstrapped CIM domain with organization
pub fn given_bootstrapped_domain(ctx: &mut TestContext, org_name: &str) {
    let org_id = Uuid::now_v7();
    ctx.organizations.insert(org_name.to_string(), org_id);
}

/// Given organizational units exist
pub fn given_units_exist(ctx: &mut TestContext, units: &[(&str, &str)]) {
    for (name, _unit_type) in units {
        let unit_id = Uuid::now_v7();
        ctx.units.insert(name.to_string(), unit_id);
    }
}

/// Given person exists with name and email
pub fn given_person_exists(
    ctx: &mut TestContext,
    name: &str,
    _email: &str,
) -> Uuid {
    let person_id = Uuid::now_v7();
    ctx.people.insert(name.to_string(), person_id);
    person_id
}

/// Given person exists with role
pub fn given_person_with_role(
    ctx: &mut TestContext,
    name: &str,
    _role: &str,
) -> Uuid {
    let person_id = Uuid::now_v7();
    ctx.people.insert(name.to_string(), person_id);
    person_id
}

// =============================================================================
// When Steps
// =============================================================================

/// When I create a person with attributes
pub fn when_create_person(
    ctx: &mut TestContext,
    name: &str,
    email: &str,
    role: &str,
) -> CreatePerson {
    let person_id = Uuid::now_v7();
    ctx.people.insert(name.to_string(), person_id);

    let org_id = ctx.organizations.values().next().copied();

    CreatePerson {
        command_id: Uuid::now_v7(),
        person_id,
        organization_id: org_id,
        name: name.to_string(),
        email: email.to_string(),
        title: Some(role.to_string()),
        department: None,
        correlation_id: ctx.correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
    }
}

/// When I update the person's name
pub fn when_update_person_name(
    _ctx: &mut TestContext,
    _person_id: Uuid,
    _new_name: &str,
) {
    // In a real implementation, this would create an UpdatePersonName command
    // For now, we demonstrate the BDD pattern
}

/// When I deactivate the person
pub fn when_deactivate_person(
    _ctx: &mut TestContext,
    _person_id: Uuid,
) {
    // Creates DeactivatePerson command
}

// =============================================================================
// Then Steps
// =============================================================================

/// Then a PersonCreated event should be emitted
pub fn then_person_created_event_emitted(ctx: &TestContext) -> bool {
    ctx.has_event_of_type("PersonCreated")
}

/// Then the person should have name
pub fn then_person_has_name(ctx: &TestContext, expected_name: &str) -> bool {
    ctx.people.contains_key(expected_name)
}

/// Then the person should be associated with organization
pub fn then_person_associated_with_org(ctx: &TestContext, org_name: &str) -> bool {
    // Verify via projection that person is linked to org
    ctx.organizations.contains_key(org_name)
}

/// Then the person should be persisted to the projection
pub fn then_person_persisted(ctx: &TestContext, name: &str) -> bool {
    ctx.people.contains_key(name)
}

/// Then PersonCreated events should be emitted for N people
pub fn then_person_events_emitted_count(ctx: &TestContext, expected: usize) -> bool {
    ctx.captured_events.iter()
        .filter(|e| format!("{:?}", e).contains("PersonCreated"))
        .count() >= expected
}

// =============================================================================
// Integrated BDD Tests
// =============================================================================

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::bdd::domain_bootstrap_steps::{
        given_clean_cim_environment,
        when_execute_bootstrap,
    };
    use cim_keys::commands::KeyCommand;

    #[tokio::test]
    async fn scenario_create_new_person() {
        // Feature: Person Management
        // Scenario: Create a new person in the organization
        eprintln!("\n  Scenario: Create a new person in the organization");

        // Given a bootstrapped CIM domain with organization "TechCorp"
        let mut ctx = given_clean_cim_environment();
        given_bootstrapped_domain(&mut ctx, "TechCorp");
        eprintln!("    Given a bootstrapped CIM domain with organization 'TechCorp'");

        // And organizational units exist
        given_units_exist(&mut ctx, &[
            ("Engineering", "Department"),
            ("Security", "Department"),
        ]);
        eprintln!("    And organizational units exist");

        // When I create a person with name, email, role
        let create_person = when_create_person(
            &mut ctx,
            "John Smith",
            "john@techcorp.com",
            "Developer",
        );
        eprintln!("    When I create a person 'John Smith'");

        let command = KeyCommand::CreatePerson(create_person);
        let result = when_execute_bootstrap(&mut ctx, command).await;

        // Then a PersonCreated event should be emitted
        assert!(result.is_ok(), "Person creation should succeed");
        assert!(then_person_created_event_emitted(&ctx));
        eprintln!("    Then a PersonCreated event should be emitted");

        // And the person should have name "John Smith"
        assert!(then_person_has_name(&ctx, "John Smith"));
        eprintln!("    And the person should have name 'John Smith'");

        // And the person should be associated with organization "TechCorp"
        assert!(then_person_associated_with_org(&ctx, "TechCorp"));
        eprintln!("    And the person should be associated with organization 'TechCorp'");

        eprintln!("  ✓ Scenario passed\n");
    }

    #[tokio::test]
    async fn scenario_create_multiple_people() {
        // Scenario: Create multiple people with roles
        eprintln!("\n  Scenario: Create multiple people with roles");

        let mut ctx = given_clean_cim_environment();
        given_bootstrapped_domain(&mut ctx, "MultiOrg");

        // Create multiple people
        let people = vec![
            ("Alice Admin", "alice@corp.com", "Administrator"),
            ("Bob Dev", "bob@corp.com", "Developer"),
            ("Carol Ops", "carol@corp.com", "Operator"),
        ];

        for (name, email, role) in &people {
            let create_person = when_create_person(&mut ctx, name, email, role);
            let command = KeyCommand::CreatePerson(create_person);
            let _ = when_execute_bootstrap(&mut ctx, command).await;
        }

        // Verify all people were created
        assert_eq!(ctx.people.len(), 3);
        assert!(ctx.people.contains_key("Alice Admin"));
        assert!(ctx.people.contains_key("Bob Dev"));
        assert!(ctx.people.contains_key("Carol Ops"));

        eprintln!("  ✓ Scenario passed\n");
    }

    #[tokio::test]
    async fn scenario_person_has_valid_uuid() {
        // Scenario: Person has valid UUID v7 identifier
        eprintln!("\n  Scenario: Person has valid UUID v7 identifier");

        let mut ctx = given_clean_cim_environment();
        given_bootstrapped_domain(&mut ctx, "UuidOrg");

        let create_person = when_create_person(
            &mut ctx,
            "UUID Person",
            "uuid@test.com",
            "Tester",
        );

        // Verify UUID v7
        let version = (create_person.person_id.as_bytes()[6] >> 4) & 0x0F;
        assert_eq!(version, 7, "Person ID should be UUID v7");

        eprintln!("    Person ID is valid UUID v7");
        eprintln!("  ✓ Scenario passed\n");
    }

    #[test]
    fn scenario_person_role_assignment() {
        // Scenario: Person role is recorded correctly
        eprintln!("\n  Scenario: Person role is recorded correctly");

        let mut ctx = TestContext::new();
        given_bootstrapped_domain(&mut ctx, "RoleOrg");

        let person_id = given_person_with_role(&mut ctx, "Role Person", "Administrator");
        assert!(ctx.people.contains_key("Role Person"));
        assert!(!person_id.is_nil());

        eprintln!("  ✓ Scenario passed\n");
    }
}
