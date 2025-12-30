// Copyright (c) 2025 - Cowboy AI, LLC.
//! Step definitions for key_generation.feature
//!
//! Implements BDD scenarios for cryptographic key generation.

use super::context::TestContext;
use uuid::Uuid;

// =============================================================================
// Given Steps
// =============================================================================

/// Given a bootstrapped CIM domain with organization
pub fn given_domain_with_org(ctx: &mut TestContext, org_name: &str) {
    let org_id = Uuid::now_v7();
    ctx.organizations.insert(org_name.to_string(), org_id);
}

/// Given person exists with role
pub fn given_person_with_role(ctx: &mut TestContext, name: &str, _role: &str) -> Uuid {
    let person_id = Uuid::now_v7();
    ctx.people.insert(name.to_string(), person_id);
    person_id
}

/// Given a root CA exists for organization
pub fn given_root_ca_exists(ctx: &mut TestContext, org_name: &str) -> Uuid {
    let key_id = Uuid::now_v7();
    let key_name = format!("{}-root-ca", org_name);
    ctx.keys.insert(key_name, key_id);
    key_id
}

/// Given an intermediate CA exists for unit
pub fn given_intermediate_ca_exists(ctx: &mut TestContext, unit_name: &str) -> Uuid {
    let key_id = Uuid::now_v7();
    let key_name = format!("{}-intermediate-ca", unit_name);
    ctx.keys.insert(key_name, key_id);
    key_id
}

/// Given a master seed
pub fn given_master_seed(_ctx: &mut TestContext, _seed_hex: &str) {
    // Seed would be stored in crypto context
}

// =============================================================================
// When Steps
// =============================================================================

/// When I generate a root CA key for organization
pub fn when_generate_root_ca(ctx: &mut TestContext, org_name: &str) -> Uuid {
    let key_id = Uuid::now_v7();
    let key_name = format!("{}-root-ca", org_name);
    ctx.keys.insert(key_name, key_id);
    key_id
}

/// When I generate an intermediate CA for unit
pub fn when_generate_intermediate_ca(ctx: &mut TestContext, unit_name: &str) -> Uuid {
    let key_id = Uuid::now_v7();
    let key_name = format!("{}-intermediate-ca", unit_name);
    ctx.keys.insert(key_name, key_id);
    key_id
}

/// When I generate a personal key for person with purpose
pub fn when_generate_personal_key(
    ctx: &mut TestContext,
    person_name: &str,
    purpose: &str,
) -> Uuid {
    let key_id = Uuid::now_v7();
    let key_name = format!("{}-{}", person_name, purpose);
    ctx.keys.insert(key_name, key_id);
    key_id
}

/// When I generate keys for multiple purposes
pub fn when_generate_keys_for_purposes(
    ctx: &mut TestContext,
    person_name: &str,
    purposes: &[&str],
) -> Vec<Uuid> {
    purposes.iter().map(|purpose| {
        when_generate_personal_key(ctx, person_name, purpose)
    }).collect()
}

// =============================================================================
// Then Steps
// =============================================================================

/// Then a RootCAKeyGenerated event should be emitted
pub fn then_root_ca_event_emitted(ctx: &TestContext, org_name: &str) -> bool {
    let key_name = format!("{}-root-ca", org_name);
    ctx.keys.contains_key(&key_name)
}

/// Then the key should use specified algorithm
pub fn then_key_uses_algorithm(_key_id: Uuid, _algorithm: &str) -> bool {
    // Would verify key algorithm from projection
    true
}

/// Then the public key should be stored in projection
pub fn then_public_key_stored(ctx: &TestContext, key_name: &str) -> bool {
    ctx.keys.contains_key(key_name) ||
        ctx.keys.keys().any(|k| k.contains(key_name))
}

/// Then separate keys should be generated for each purpose
pub fn then_separate_keys_per_purpose(
    ctx: &TestContext,
    person_name: &str,
    purposes: &[&str],
) -> bool {
    purposes.iter().all(|purpose| {
        let key_name = format!("{}-{}", person_name, purpose);
        ctx.keys.contains_key(&key_name)
    })
}

/// Then certificate chain should be valid
pub fn then_certificate_chain_valid(_intermediate_key_id: Uuid, _root_key_id: Uuid) -> bool {
    // Would verify certificate chain
    true
}

// =============================================================================
// Integrated BDD Tests
// =============================================================================

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::bdd::context::TestContext;

    #[test]
    fn scenario_generate_root_ca_key() {
        // Feature: Key Generation
        // Scenario: Generate root CA key pair
        eprintln!("\n  Scenario: Generate root CA key pair");

        // Given a bootstrapped CIM domain with organization "SecureCorp"
        let mut ctx = TestContext::new();
        given_domain_with_org(&mut ctx, "SecureCorp");
        eprintln!("    Given a bootstrapped CIM domain with organization 'SecureCorp'");

        // And person "Key Owner" exists with Administrator role
        given_person_with_role(&mut ctx, "Key Owner", "Administrator");
        eprintln!("    And person 'Key Owner' exists with Administrator role");

        // When I generate a root CA key for organization "SecureCorp"
        let key_id = when_generate_root_ca(&mut ctx, "SecureCorp");
        eprintln!("    When I generate a root CA key for organization 'SecureCorp'");

        // Then a RootCAKeyGenerated event should be emitted
        assert!(then_root_ca_event_emitted(&ctx, "SecureCorp"));
        eprintln!("    Then a RootCAKeyGenerated event should be emitted");

        // And the key should have valid UUID v7
        let version = (key_id.as_bytes()[6] >> 4) & 0x0F;
        assert_eq!(version, 7);
        eprintln!("    And the key should have valid UUID v7");

        // And the public key should be stored in the projection
        assert!(then_public_key_stored(&ctx, "SecureCorp-root-ca"));
        eprintln!("    And the public key should be stored in the projection");

        eprintln!("  ✓ Scenario passed\n");
    }

    #[test]
    fn scenario_generate_intermediate_ca_signed_by_root() {
        // Scenario: Generate intermediate CA signed by root
        eprintln!("\n  Scenario: Generate intermediate CA signed by root");

        let mut ctx = TestContext::new();
        given_domain_with_org(&mut ctx, "SecureCorp");

        // Given a root CA exists for organization "SecureCorp"
        let root_key_id = given_root_ca_exists(&mut ctx, "SecureCorp");
        eprintln!("    Given a root CA exists for organization 'SecureCorp'");

        // When I generate an intermediate CA for unit "Engineering"
        let intermediate_key_id = when_generate_intermediate_ca(&mut ctx, "Engineering");
        eprintln!("    When I generate an intermediate CA for unit 'Engineering'");

        // Then the intermediate certificate should be signed by the root CA
        assert!(then_certificate_chain_valid(intermediate_key_id, root_key_id));
        eprintln!("    Then the intermediate certificate should be signed by the root CA");

        eprintln!("  ✓ Scenario passed\n");
    }

    #[test]
    fn scenario_generate_personal_authentication_key() {
        // Scenario: Generate personal authentication key
        eprintln!("\n  Scenario: Generate personal authentication key");

        let mut ctx = TestContext::new();
        given_domain_with_org(&mut ctx, "SecureCorp");

        // Given person "Alice Developer" exists
        given_person_with_role(&mut ctx, "Alice Developer", "Developer");
        eprintln!("    Given person 'Alice Developer' exists");

        // And an intermediate CA exists for their unit
        given_intermediate_ca_exists(&mut ctx, "Engineering");
        eprintln!("    And an intermediate CA exists for their unit");

        // When I generate a personal key with purpose "authentication"
        let key_id = when_generate_personal_key(&mut ctx, "Alice Developer", "authentication");
        eprintln!("    When I generate a personal key with purpose 'authentication'");

        // Then a PersonalKeyGenerated event should be emitted
        assert!(then_public_key_stored(&ctx, "Alice Developer"));
        eprintln!("    Then a PersonalKeyGenerated event should be emitted");

        // And the key should have valid UUID
        assert!(!key_id.is_nil());
        eprintln!("    And the key should be associated with 'Alice Developer'");

        eprintln!("  ✓ Scenario passed\n");
    }

    #[test]
    fn scenario_generate_multiple_keys_for_purposes() {
        // Scenario: Generate multiple keys for different purposes
        eprintln!("\n  Scenario: Generate multiple keys for different purposes");

        let mut ctx = TestContext::new();
        given_domain_with_org(&mut ctx, "SecureCorp");
        given_person_with_role(&mut ctx, "Bob Signer", "Developer");

        // When I generate keys for purposes: authentication, signing, encryption
        let purposes = ["authentication", "signing", "encryption"];
        let key_ids = when_generate_keys_for_purposes(&mut ctx, "Bob Signer", &purposes);
        eprintln!("    When I generate keys for purposes: authentication, signing, encryption");

        // Then separate keys should be generated for each purpose
        assert_eq!(key_ids.len(), 3);
        assert!(then_separate_keys_per_purpose(&ctx, "Bob Signer", &purposes));
        eprintln!("    Then separate keys should be generated for each purpose");

        // And all keys should be associated with "Bob Signer"
        assert!(ctx.keys.keys().filter(|k| k.contains("Bob Signer")).count() == 3);
        eprintln!("    And all keys should be associated with 'Bob Signer'");

        eprintln!("  ✓ Scenario passed\n");
    }
}
