// Copyright (c) 2025 - Cowboy AI, LLC.

//! Property-Based Tests for LiftableDomain Functor Laws
//!
//! These tests use proptest to verify the functor laws for the LiftableDomain trait.
//!
//! # Mathematical Foundation
//!
//! LiftableDomain forms a faithful functor F: Domain → Graph where:
//! - Objects (entities) map to graph nodes
//! - Identity is preserved: F(id_A) = id_{F(A)}
//! - The functor is "faithful": If F(f) = F(g) then f = g (injective on morphisms)
//!
//! # Laws Tested
//!
//! 1. **Identity Law**: `unlift(lift(entity)) == Some(entity)` (for entity fields that survive roundtrip)
//! 2. **Faithfulness**: Distinct domain objects lift to distinct graph nodes (entity_id differs)
//! 3. **Injection Correctness**: Each type lifts to its expected injection variant

use proptest::prelude::*;

use cim_keys::domain::{
    Organization, OrganizationUnit, OrganizationUnitType, Person, PersonRole,
    RoleType, RoleScope, Permission, Location, LocationMarker, Address,
};
use cim_keys::domain::ids::BootstrapOrgId;
use cim_keys::lifting::{LiftableDomain, Injection};
use cim_domain::EntityId;

// ============================================================================
// Arbitrary Generators
// ============================================================================

/// Generate an arbitrary Organization
fn arb_organization() -> impl Strategy<Value = Organization> {
    (
        prop::string::string_regex("[a-z][a-z0-9_]{2,20}")
            .unwrap()
            .prop_map(|s| if s.is_empty() { "default-org".to_string() } else { s }),
        prop::string::string_regex("[A-Z][a-zA-Z ]{2,30}")
            .unwrap()
            .prop_map(|s| if s.is_empty() { "Default Org".to_string() } else { s }),
        prop::option::of(prop::string::string_regex("[A-Za-z ]{5,50}").unwrap()),
    ).prop_map(|(name, display_name, description)| {
        let mut org = Organization::new(name, display_name);
        if let Some(desc) = description {
            org = org.with_description(desc);
        }
        org
    })
}

/// Generate an arbitrary OrganizationUnit
fn arb_organization_unit() -> impl Strategy<Value = OrganizationUnit> {
    (
        prop::string::string_regex("[a-z][a-z0-9_]{2,20}")
            .unwrap()
            .prop_map(|s| if s.is_empty() { "default-unit".to_string() } else { s }),
        prop::sample::select(vec![
            OrganizationUnitType::Division,
            OrganizationUnitType::Department,
            OrganizationUnitType::Team,
            OrganizationUnitType::Project,
            OrganizationUnitType::Service,
            OrganizationUnitType::Infrastructure,
        ]),
        prop::option::of(prop::string::string_regex("[a-z][a-z0-9_]{2,15}").unwrap()),
    ).prop_map(|(name, unit_type, nats_account)| {
        let mut unit = OrganizationUnit::new(name, unit_type);
        if let Some(acct) = nats_account {
            unit = unit.with_nats_account(acct);
        }
        unit
    })
}

/// Generate an arbitrary Person
fn arb_person() -> impl Strategy<Value = Person> {
    (
        prop::string::string_regex("[A-Z][a-z]+ [A-Z][a-z]+")
            .unwrap()
            .prop_map(|s| if s.is_empty() || !s.contains(' ') { "John Doe".to_string() } else { s }),
        prop::string::string_regex("[a-z]+@[a-z]+\\.[a-z]{2,4}")
            .unwrap()
            .prop_map(|s| if s.is_empty() || !s.contains('@') { "test@example.com".to_string() } else { s }),
        prop::bool::ANY,
    ).prop_map(|(name, email, active)| {
        let org_id = BootstrapOrgId::new();
        let mut person = Person::new(name, email, org_id);
        person.active = active;
        person
    })
}

/// Generate an arbitrary Location
fn arb_location() -> impl Strategy<Value = Location> {
    prop::string::string_regex("[A-Z][a-zA-Z0-9 ]{2,30}")
        .unwrap()
        .prop_map(|name| {
            let name = if name.is_empty() { "Default Location".to_string() } else { name };
            let id = EntityId::<LocationMarker>::new();
            let address = Address::new(
                "123 Main St".to_string(),
                "City".to_string(),
                "State".to_string(),
                "Country".to_string(),
                "12345".to_string(),
            );
            Location::new_physical(id, name, address)
                .expect("Failed to create location")
        })
}

// ============================================================================
// Property Tests: Identity Law
// ============================================================================

proptest! {
    /// Property: Organization lift followed by unlift recovers the original entity ID
    ///
    /// This tests the roundtrip property: unlift(lift(org)).entity_id == org.id
    #[test]
    fn prop_organization_lift_unlift_identity(org in arb_organization()) {
        let entity_id = org.entity_id();
        let lifted = org.lift();

        // The lifted node should have the same entity ID
        prop_assert_eq!(lifted.id, entity_id, "Lifted node ID should match entity ID");

        // Unlift should recover the original
        let recovered: Option<Organization> = Organization::unlift(&lifted);
        prop_assert!(recovered.is_some(), "Unlift should succeed for Organization");

        let recovered = recovered.unwrap();
        prop_assert_eq!(recovered.entity_id(), entity_id, "Recovered entity ID should match");
        prop_assert_eq!(recovered.name, org.name, "Recovered name should match");
        prop_assert_eq!(recovered.display_name, org.display_name, "Recovered display_name should match");
    }

    /// Property: OrganizationUnit lift followed by unlift recovers the original entity ID
    #[test]
    fn prop_organization_unit_lift_unlift_identity(unit in arb_organization_unit()) {
        let entity_id = unit.entity_id();
        let lifted = unit.lift();

        prop_assert_eq!(lifted.id, entity_id, "Lifted node ID should match entity ID");

        let recovered: Option<OrganizationUnit> = OrganizationUnit::unlift(&lifted);
        prop_assert!(recovered.is_some(), "Unlift should succeed for OrganizationUnit");

        let recovered = recovered.unwrap();
        prop_assert_eq!(recovered.entity_id(), entity_id, "Recovered entity ID should match");
        prop_assert_eq!(recovered.name, unit.name, "Recovered name should match");
    }

    /// Property: Person lift followed by unlift recovers the original entity ID
    #[test]
    fn prop_person_lift_unlift_identity(person in arb_person()) {
        let entity_id = person.entity_id();
        let lifted = person.lift();

        prop_assert_eq!(lifted.id, entity_id, "Lifted node ID should match entity ID");

        let recovered: Option<Person> = Person::unlift(&lifted);
        prop_assert!(recovered.is_some(), "Unlift should succeed for Person");

        let recovered = recovered.unwrap();
        prop_assert_eq!(recovered.entity_id(), entity_id, "Recovered entity ID should match");
        prop_assert_eq!(recovered.name, person.name, "Recovered name should match");
        prop_assert_eq!(recovered.email, person.email, "Recovered email should match");
        prop_assert_eq!(recovered.active, person.active, "Recovered active flag should match");
    }

    /// Property: Location lift followed by unlift recovers the original entity ID
    #[test]
    fn prop_location_lift_unlift_identity(location in arb_location()) {
        let entity_id = location.entity_id();
        let lifted = location.lift();

        prop_assert_eq!(lifted.id, entity_id, "Lifted node ID should match entity ID");

        let recovered: Option<Location> = Location::unlift(&lifted);
        prop_assert!(recovered.is_some(), "Unlift should succeed for Location");

        let recovered = recovered.unwrap();
        prop_assert_eq!(recovered.entity_id(), entity_id, "Recovered entity ID should match");
    }
}

// ============================================================================
// Property Tests: Faithfulness (Injectivity)
// ============================================================================

proptest! {
    /// Property: Two distinct Organizations lift to nodes with distinct entity IDs
    ///
    /// This is the faithfulness property: distinct domain objects must lift to
    /// distinguishable graph nodes.
    #[test]
    fn prop_distinct_organizations_lift_to_distinct_nodes(
        org1 in arb_organization(),
        org2 in arb_organization(),
    ) {
        // Organizations always have unique IDs (UUID v7 generated on construction)
        prop_assume!(org1.entity_id() != org2.entity_id());

        let lifted1 = org1.lift();
        let lifted2 = org2.lift();

        prop_assert_ne!(lifted1.id, lifted2.id,
            "Distinct organizations should lift to nodes with distinct IDs");
    }

    /// Property: Two distinct Persons lift to nodes with distinct entity IDs
    #[test]
    fn prop_distinct_persons_lift_to_distinct_nodes(
        person1 in arb_person(),
        person2 in arb_person(),
    ) {
        prop_assume!(person1.entity_id() != person2.entity_id());

        let lifted1 = person1.lift();
        let lifted2 = person2.lift();

        prop_assert_ne!(lifted1.id, lifted2.id,
            "Distinct persons should lift to nodes with distinct IDs");
    }

    /// Property: Two distinct Locations lift to nodes with distinct entity IDs
    #[test]
    fn prop_distinct_locations_lift_to_distinct_nodes(
        location1 in arb_location(),
        location2 in arb_location(),
    ) {
        prop_assume!(location1.entity_id() != location2.entity_id());

        let lifted1 = location1.lift();
        let lifted2 = location2.lift();

        prop_assert_ne!(lifted1.id, lifted2.id,
            "Distinct locations should lift to nodes with distinct IDs");
    }
}

// ============================================================================
// Property Tests: Injection Correctness
// ============================================================================

proptest! {
    /// Property: Organization always lifts to the Organization injection variant
    #[test]
    fn prop_organization_injection_correct(org in arb_organization()) {
        let lifted = org.lift();
        prop_assert_eq!(lifted.injection, Injection::Organization,
            "Organization should lift to Organization injection");
        prop_assert_eq!(Organization::injection(), Injection::Organization);
    }

    /// Property: OrganizationUnit always lifts to the OrganizationUnit injection variant
    #[test]
    fn prop_organization_unit_injection_correct(unit in arb_organization_unit()) {
        let lifted = unit.lift();
        prop_assert_eq!(lifted.injection, Injection::OrganizationUnit,
            "OrganizationUnit should lift to OrganizationUnit injection");
        prop_assert_eq!(OrganizationUnit::injection(), Injection::OrganizationUnit);
    }

    /// Property: Person always lifts to the Person injection variant
    #[test]
    fn prop_person_injection_correct(person in arb_person()) {
        let lifted = person.lift();
        prop_assert_eq!(lifted.injection, Injection::Person,
            "Person should lift to Person injection");
        prop_assert_eq!(Person::injection(), Injection::Person);
    }

    /// Property: Location always lifts to the Location injection variant
    #[test]
    fn prop_location_injection_correct(location in arb_location()) {
        let lifted = location.lift();
        prop_assert_eq!(lifted.injection, Injection::Location,
            "Location should lift to Location injection");
        prop_assert_eq!(Location::injection(), Injection::Location);
    }
}

// ============================================================================
// Property Tests: Type Discrimination (No False Unlift)
// ============================================================================

proptest! {
    /// Property: Unlift with wrong type returns None
    ///
    /// Organization lifted cannot be unlifted as Person, OrganizationUnit, or Location
    #[test]
    fn prop_organization_does_not_unlift_as_other_types(org in arb_organization()) {
        let lifted = org.lift();

        let as_person: Option<Person> = Person::unlift(&lifted);
        prop_assert!(as_person.is_none(), "Organization should not unlift as Person");

        let as_unit: Option<OrganizationUnit> = OrganizationUnit::unlift(&lifted);
        prop_assert!(as_unit.is_none(), "Organization should not unlift as OrganizationUnit");

        let as_location: Option<Location> = Location::unlift(&lifted);
        prop_assert!(as_location.is_none(), "Organization should not unlift as Location");
    }

    /// Property: Person lifted cannot be unlifted as Organization or Location
    #[test]
    fn prop_person_does_not_unlift_as_other_types(person in arb_person()) {
        let lifted = person.lift();

        let as_org: Option<Organization> = Organization::unlift(&lifted);
        prop_assert!(as_org.is_none(), "Person should not unlift as Organization");

        let as_unit: Option<OrganizationUnit> = OrganizationUnit::unlift(&lifted);
        prop_assert!(as_unit.is_none(), "Person should not unlift as OrganizationUnit");

        let as_location: Option<Location> = Location::unlift(&lifted);
        prop_assert!(as_location.is_none(), "Person should not unlift as Location");
    }
}

// ============================================================================
// Property Tests: Idempotence
// ============================================================================

proptest! {
    /// Property: Lifting is deterministic - same entity always produces same node structure
    #[test]
    fn prop_lift_is_deterministic(org in arb_organization()) {
        let lifted1 = org.lift();
        let lifted2 = org.lift();

        prop_assert_eq!(lifted1.id, lifted2.id, "Lift should be deterministic for ID");
        prop_assert_eq!(lifted1.injection, lifted2.injection, "Lift should be deterministic for injection");
        prop_assert_eq!(lifted1.label, lifted2.label, "Lift should be deterministic for label");
    }

    /// Property: Double lift produces same entity ID (through unlift+lift)
    #[test]
    fn prop_double_roundtrip_preserves_id(person in arb_person()) {
        let original_id = person.entity_id();

        // First roundtrip
        let lifted1 = person.lift();
        let recovered1 = Person::unlift(&lifted1).unwrap();

        // Second roundtrip
        let lifted2 = recovered1.lift();
        let recovered2 = Person::unlift(&lifted2).unwrap();

        prop_assert_eq!(original_id, recovered1.entity_id(), "First roundtrip preserves ID");
        prop_assert_eq!(original_id, recovered2.entity_id(), "Second roundtrip preserves ID");
    }
}

// ============================================================================
// Property Tests: Layout Tier Consistency
// ============================================================================

proptest! {
    /// Property: Injection layout tier is consistent with domain type
    #[test]
    fn prop_layout_tier_consistent_for_organization(org in arb_organization()) {
        let lifted = org.lift();
        let tier = lifted.injection.layout_tier();
        prop_assert_eq!(tier, 0, "Organization should be in tier 0 (root level)");
    }

    /// Property: Person should be in tier 2 (leaf level)
    #[test]
    fn prop_layout_tier_consistent_for_person(person in arb_person()) {
        let lifted = person.lift();
        let tier = lifted.injection.layout_tier();
        prop_assert_eq!(tier, 2, "Person should be in tier 2 (leaf level)");
    }

    /// Property: OrganizationUnit should be in tier 1 (intermediate level)
    #[test]
    fn prop_layout_tier_consistent_for_unit(unit in arb_organization_unit()) {
        let lifted = unit.lift();
        let tier = lifted.injection.layout_tier();
        prop_assert_eq!(tier, 1, "OrganizationUnit should be in tier 1 (intermediate level)");
    }
}

// ============================================================================
// Unit Tests: Functor Laws Verification
// ============================================================================

#[test]
fn test_organization_functor_identity() {
    let org = Organization::new("cowboyai", "CowboyAI LLC")
        .with_description("A technology company");

    let lifted = org.lift();
    let recovered = Organization::unlift(&lifted).expect("Should unlift");

    assert_eq!(recovered.entity_id(), org.entity_id());
    assert_eq!(recovered.name, org.name);
    assert_eq!(recovered.display_name, org.display_name);
    assert_eq!(recovered.description, org.description);
}

#[test]
fn test_person_functor_identity() {
    let org_id = BootstrapOrgId::new();
    let person = Person::new("John Doe", "john@example.com", org_id)
        .with_role(PersonRole {
            role_type: RoleType::Developer,
            scope: RoleScope::Organization,
            permissions: vec![Permission::CreateKeys],
        });

    let lifted = person.lift();
    let recovered = Person::unlift(&lifted).expect("Should unlift");

    assert_eq!(recovered.entity_id(), person.entity_id());
    assert_eq!(recovered.name, person.name);
    assert_eq!(recovered.email, person.email);
    assert_eq!(recovered.organization_id, person.organization_id);
}

#[test]
fn test_cross_type_unlift_fails() {
    let org = Organization::new("test", "Test Org");
    let lifted = org.lift();

    // These should all return None
    assert!(Person::unlift(&lifted).is_none());
    assert!(OrganizationUnit::unlift(&lifted).is_none());
    assert!(Location::unlift(&lifted).is_none());
}

#[test]
fn test_faithfulness_distinct_entities() {
    let org1 = Organization::new("org1", "Org One");
    let org2 = Organization::new("org2", "Org Two");

    let lifted1 = org1.lift();
    let lifted2 = org2.lift();

    // Faithful functor: distinct inputs → distinct outputs
    assert_ne!(lifted1.id, lifted2.id);
}

#[test]
fn test_injection_type_correctness() {
    let org = Organization::new("test", "Test");
    let unit = OrganizationUnit::new("unit", OrganizationUnitType::Team);
    let org_id = BootstrapOrgId::new();
    let person = Person::new("Test", "test@test.com", org_id);
    let location = Location::new_physical(
        EntityId::<LocationMarker>::new(),
        "Office".to_string(),
        Address::new(
            "456 Office St".to_string(),
            "City".to_string(),
            "State".to_string(),
            "Country".to_string(),
            "12345".to_string(),
        ),
    ).expect("Failed to create location");

    assert_eq!(org.lift().injection, Injection::Organization);
    assert_eq!(unit.lift().injection, Injection::OrganizationUnit);
    assert_eq!(person.lift().injection, Injection::Person);
    assert_eq!(location.lift().injection, Injection::Location);
}
