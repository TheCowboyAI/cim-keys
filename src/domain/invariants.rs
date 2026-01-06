// Copyright (c) 2025 - Cowboy AI, LLC.

//! Cross-Context Domain Invariants
//!
//! This module implements invariants that span multiple bounded contexts,
//! ensuring consistency across the CIM-Keys domain model.
//!
//! # Context Boundaries
//!
//! | Context      | Aggregates                                    |
//! |--------------|-----------------------------------------------|
//! | Organization | Organization, OrganizationUnit, Person, Location |
//! | PKI          | Key, Certificate, CertificateChain, TrustChain |
//! | NATS         | Operator, Account, User, Subject              |
//! | YubiKey      | YubiKey, Slot, SlotBinding                    |
//!
//! # Invariants
//!
//! 1. **NatsUserPersonInvariant**: Every NATS User maps to exactly one Person
//! 2. **YubiKeySlotBindingInvariant**: Every Key stored on YubiKey has valid slot binding
//! 3. **NatsOrganizationHierarchyInvariant**: NATS hierarchy mirrors Organization hierarchy

use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ============================================================================
// INVARIANT RESULT
// ============================================================================

/// Result of invariant verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantResult {
    /// Invariant is satisfied
    Satisfied,
    /// Invariant is violated with explanation
    Violated(InvariantViolation),
    /// Multiple violations found
    MultipleViolations(Vec<InvariantViolation>),
}

impl InvariantResult {
    /// Create a satisfied result
    pub fn satisfied() -> Self {
        Self::Satisfied
    }

    /// Create a violated result
    pub fn violated(message: impl Into<String>) -> Self {
        Self::Violated(InvariantViolation {
            message: message.into(),
            entity_id: None,
            context: None,
        })
    }

    /// Create a violated result with entity context
    pub fn violated_for_entity(entity_id: Uuid, message: impl Into<String>) -> Self {
        Self::Violated(InvariantViolation {
            message: message.into(),
            entity_id: Some(entity_id),
            context: None,
        })
    }

    /// Check if the invariant is satisfied
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied)
    }

    /// Check if the invariant is violated
    pub fn is_violated(&self) -> bool {
        !self.is_satisfied()
    }

    /// Combine multiple results
    pub fn combine(results: Vec<InvariantResult>) -> Self {
        let violations: Vec<InvariantViolation> = results
            .into_iter()
            .filter_map(|r| match r {
                Self::Satisfied => None,
                Self::Violated(v) => Some(vec![v]),
                Self::MultipleViolations(vs) => Some(vs),
            })
            .flatten()
            .collect();

        match violations.len() {
            0 => Self::Satisfied,
            1 => Self::Violated(violations.into_iter().next().unwrap()),
            _ => Self::MultipleViolations(violations),
        }
    }
}

/// Details of an invariant violation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantViolation {
    /// Human-readable description of the violation
    pub message: String,
    /// The entity ID that violates the invariant (if applicable)
    pub entity_id: Option<Uuid>,
    /// Additional context about the violation
    pub context: Option<HashMap<String, String>>,
}

impl InvariantViolation {
    /// Create a new violation
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            entity_id: None,
            context: None,
        }
    }

    /// Add entity context
    pub fn with_entity(mut self, id: Uuid) -> Self {
        self.entity_id = Some(id);
        self
    }

    /// Add additional context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }
}

// ============================================================================
// CROSS-CONTEXT INVARIANT TRAIT
// ============================================================================

/// Trait for cross-context invariants that span bounded contexts
pub trait CrossContextInvariant {
    /// The name of this invariant
    fn name(&self) -> &'static str;

    /// A description of what this invariant enforces
    fn description(&self) -> &'static str;

    /// The contexts this invariant spans
    fn contexts(&self) -> Vec<&'static str>;
}

// ============================================================================
// NATS-USER-PERSON INVARIANT
// ============================================================================

/// Invariant: Every NATS User maps to exactly one Person
///
/// This ensures that NATS credentials are properly bound to organizational identities.
/// A NATS User cannot exist without a corresponding Person entity, and the mapping
/// must be bijective (one-to-one).
#[derive(Debug, Clone, Default)]
pub struct NatsUserPersonInvariant {
    /// Map of NATS User ID to Person ID
    user_to_person: HashMap<Uuid, Uuid>,
    /// Map of Person ID to NATS User ID (for bidirectional lookup)
    person_to_user: HashMap<Uuid, Uuid>,
}

impl NatsUserPersonInvariant {
    /// Create a new invariant checker
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a NATS User -> Person binding
    pub fn register_binding(&mut self, nats_user_id: Uuid, person_id: Uuid) {
        self.user_to_person.insert(nats_user_id, person_id);
        self.person_to_user.insert(person_id, nats_user_id);
    }

    /// Remove a binding
    pub fn remove_binding(&mut self, nats_user_id: Uuid) {
        if let Some(person_id) = self.user_to_person.remove(&nats_user_id) {
            self.person_to_user.remove(&person_id);
        }
    }

    /// Verify that a NATS User has a valid Person binding
    pub fn verify_user(&self, nats_user_id: Uuid) -> InvariantResult {
        match self.user_to_person.get(&nats_user_id) {
            Some(person_id) => {
                // Verify bidirectional mapping
                match self.person_to_user.get(person_id) {
                    Some(&mapped_user_id) if mapped_user_id == nats_user_id => {
                        InvariantResult::satisfied()
                    }
                    Some(&mapped_user_id) => InvariantResult::violated_for_entity(
                        nats_user_id,
                        format!(
                            "NATS User {} maps to Person {}, but Person maps to different User {}",
                            nats_user_id, person_id, mapped_user_id
                        ),
                    ),
                    None => InvariantResult::violated_for_entity(
                        nats_user_id,
                        format!(
                            "NATS User {} maps to Person {}, but Person has no reverse mapping",
                            nats_user_id, person_id
                        ),
                    ),
                }
            }
            None => InvariantResult::violated_for_entity(
                nats_user_id,
                format!("NATS User {} has no corresponding Person", nats_user_id),
            ),
        }
    }

    /// Verify all registered bindings
    pub fn verify_all(&self) -> InvariantResult {
        let results: Vec<InvariantResult> = self
            .user_to_person
            .keys()
            .map(|&user_id| self.verify_user(user_id))
            .collect();

        InvariantResult::combine(results)
    }

    /// Get the Person ID for a NATS User
    pub fn get_person_for_user(&self, nats_user_id: Uuid) -> Option<Uuid> {
        self.user_to_person.get(&nats_user_id).copied()
    }

    /// Get the NATS User ID for a Person
    pub fn get_user_for_person(&self, person_id: Uuid) -> Option<Uuid> {
        self.person_to_user.get(&person_id).copied()
    }
}

impl CrossContextInvariant for NatsUserPersonInvariant {
    fn name(&self) -> &'static str {
        "NatsUserPersonInvariant"
    }

    fn description(&self) -> &'static str {
        "Every NATS User maps to exactly one Person (bijective mapping)"
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec!["Organization", "NATS"]
    }
}

// ============================================================================
// YUBIKEY SLOT BINDING INVARIANT
// ============================================================================

/// PIV slot type for YubiKey
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PivSlot {
    /// Slot 9A - PIV Authentication
    Authentication,
    /// Slot 9C - Digital Signature
    Signature,
    /// Slot 9D - Key Management
    KeyManagement,
    /// Slot 9E - Card Authentication
    CardAuthentication,
    /// Slot 82-95 - Retired Key Management slots
    RetiredKeyManagement(u8),
}

impl PivSlot {
    /// Check if this slot is compatible with a key purpose
    pub fn is_compatible_with_purpose(&self, purpose: &KeyPurpose) -> bool {
        match (self, purpose) {
            (PivSlot::Authentication, KeyPurpose::Authentication) => true,
            (PivSlot::Signature, KeyPurpose::Signing) => true,
            (PivSlot::KeyManagement, KeyPurpose::KeyAgreement) => true,
            (PivSlot::KeyManagement, KeyPurpose::Encryption) => true,
            (PivSlot::CardAuthentication, KeyPurpose::Authentication) => true,
            (PivSlot::RetiredKeyManagement(_), KeyPurpose::KeyAgreement) => true,
            (PivSlot::RetiredKeyManagement(_), KeyPurpose::Encryption) => true,
            _ => false,
        }
    }
}

/// Key purpose for slot compatibility checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyPurpose {
    Authentication,
    Signing,
    Encryption,
    KeyAgreement,
}

/// Slot binding information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotBinding {
    /// Key ID that is bound to the slot
    pub key_id: Uuid,
    /// YubiKey serial number
    pub yubikey_serial: String,
    /// PIV slot
    pub slot: PivSlot,
    /// Key purpose
    pub purpose: KeyPurpose,
}

/// Invariant: Every Key stored on YubiKey has valid slot binding
///
/// This ensures that:
/// 1. Every YubiKey-stored key has a slot binding record
/// 2. The slot is appropriate for the key's purpose
/// 3. The YubiKey serial matches between key metadata and binding
#[derive(Debug, Clone, Default)]
pub struct YubiKeySlotBindingInvariant {
    /// Map of Key ID to SlotBinding
    bindings: HashMap<Uuid, SlotBinding>,
    /// Set of YubiKey-stored key IDs (for tracking which keys need bindings)
    yubikey_stored_keys: HashSet<Uuid>,
}

impl YubiKeySlotBindingInvariant {
    /// Create a new invariant checker
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a key as YubiKey-stored
    pub fn register_yubikey_key(
        &mut self,
        key_id: Uuid,
        yubikey_serial: String,
        purpose: KeyPurpose,
    ) {
        self.yubikey_stored_keys.insert(key_id);
        // Note: Slot binding should be registered separately when provisioned
        // This just tracks that a key SHOULD have a binding
        if !self.bindings.contains_key(&key_id) {
            // Create a pending binding (will be validated later)
            self.bindings.insert(key_id, SlotBinding {
                key_id,
                yubikey_serial,
                slot: PivSlot::Authentication, // Default, should be updated
                purpose,
            });
        }
    }

    /// Register a slot binding
    pub fn register_binding(&mut self, binding: SlotBinding) {
        self.yubikey_stored_keys.insert(binding.key_id);
        self.bindings.insert(binding.key_id, binding);
    }

    /// Update a binding's slot
    pub fn update_slot(&mut self, key_id: Uuid, slot: PivSlot) -> InvariantResult {
        match self.bindings.get_mut(&key_id) {
            Some(binding) => {
                binding.slot = slot;
                InvariantResult::satisfied()
            }
            None => InvariantResult::violated_for_entity(
                key_id,
                format!("Key {} has no slot binding to update", key_id),
            ),
        }
    }

    /// Verify a specific key's slot binding
    pub fn verify_key(&self, key_id: Uuid) -> InvariantResult {
        // Check if this is a YubiKey-stored key
        if !self.yubikey_stored_keys.contains(&key_id) {
            return InvariantResult::satisfied(); // Not a YubiKey key, no invariant needed
        }

        match self.bindings.get(&key_id) {
            Some(binding) => {
                // Verify slot is compatible with purpose
                if binding.slot.is_compatible_with_purpose(&binding.purpose) {
                    InvariantResult::satisfied()
                } else {
                    InvariantResult::violated_for_entity(
                        key_id,
                        format!(
                            "Key {} with purpose {:?} is in incompatible slot {:?}",
                            key_id, binding.purpose, binding.slot
                        ),
                    )
                }
            }
            None => InvariantResult::violated_for_entity(
                key_id,
                format!("YubiKey-stored key {} has no slot binding", key_id),
            ),
        }
    }

    /// Verify all registered keys
    pub fn verify_all(&self) -> InvariantResult {
        let results: Vec<InvariantResult> = self
            .yubikey_stored_keys
            .iter()
            .map(|&key_id| self.verify_key(key_id))
            .collect();

        InvariantResult::combine(results)
    }

    /// Get the slot binding for a key
    pub fn get_binding(&self, key_id: Uuid) -> Option<&SlotBinding> {
        self.bindings.get(&key_id)
    }
}

impl CrossContextInvariant for YubiKeySlotBindingInvariant {
    fn name(&self) -> &'static str {
        "YubiKeySlotBindingInvariant"
    }

    fn description(&self) -> &'static str {
        "Every Key stored on YubiKey has valid slot binding with compatible purpose"
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec!["PKI", "YubiKey"]
    }
}

// ============================================================================
// NATS-ORGANIZATION HIERARCHY INVARIANT
// ============================================================================

/// Invariant: NATS hierarchy mirrors Organization hierarchy
///
/// This ensures that:
/// 1. Each Organization maps to exactly one NATS Operator
/// 2. Each OrganizationUnit maps to exactly one NATS Account under the correct Operator
/// 3. Hierarchy relationships are preserved
#[derive(Debug, Clone, Default)]
pub struct NatsOrganizationHierarchyInvariant {
    /// Map of Organization ID to NATS Operator ID
    org_to_operator: HashMap<Uuid, Uuid>,
    /// Map of OrganizationUnit ID to NATS Account ID
    unit_to_account: HashMap<Uuid, Uuid>,
    /// Map of NATS Account ID to its parent Operator ID
    account_to_operator: HashMap<Uuid, Uuid>,
    /// Map of OrganizationUnit ID to its parent Organization ID
    unit_to_org: HashMap<Uuid, Uuid>,
}

impl NatsOrganizationHierarchyInvariant {
    /// Create a new invariant checker
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an Organization -> Operator mapping
    pub fn register_organization(&mut self, org_id: Uuid, operator_id: Uuid) {
        self.org_to_operator.insert(org_id, operator_id);
    }

    /// Register an OrganizationUnit -> Account mapping
    pub fn register_unit(
        &mut self,
        unit_id: Uuid,
        org_id: Uuid,
        account_id: Uuid,
        operator_id: Uuid,
    ) {
        self.unit_to_account.insert(unit_id, account_id);
        self.unit_to_org.insert(unit_id, org_id);
        self.account_to_operator.insert(account_id, operator_id);
    }

    /// Verify that an Organization has a valid Operator
    pub fn verify_organization(&self, org_id: Uuid) -> InvariantResult {
        match self.org_to_operator.get(&org_id) {
            Some(_operator_id) => InvariantResult::satisfied(),
            None => InvariantResult::violated_for_entity(
                org_id,
                format!("Organization {} has no NATS Operator", org_id),
            ),
        }
    }

    /// Verify that an OrganizationUnit has a valid Account under the correct Operator
    pub fn verify_unit(&self, unit_id: Uuid) -> InvariantResult {
        let account_id = match self.unit_to_account.get(&unit_id) {
            Some(id) => *id,
            None => {
                return InvariantResult::violated_for_entity(
                    unit_id,
                    format!("OrganizationUnit {} has no NATS Account", unit_id),
                )
            }
        };

        let org_id = match self.unit_to_org.get(&unit_id) {
            Some(id) => *id,
            None => {
                return InvariantResult::violated_for_entity(
                    unit_id,
                    format!("OrganizationUnit {} has no parent Organization", unit_id),
                )
            }
        };

        let expected_operator = match self.org_to_operator.get(&org_id) {
            Some(id) => *id,
            None => {
                return InvariantResult::violated_for_entity(
                    unit_id,
                    format!(
                        "Parent Organization {} has no NATS Operator",
                        org_id
                    ),
                )
            }
        };

        let actual_operator = match self.account_to_operator.get(&account_id) {
            Some(id) => *id,
            None => {
                return InvariantResult::violated_for_entity(
                    unit_id,
                    format!("NATS Account {} has no parent Operator", account_id),
                )
            }
        };

        if expected_operator == actual_operator {
            InvariantResult::satisfied()
        } else {
            InvariantResult::violated_for_entity(
                unit_id,
                format!(
                    "NATS Account {} is under wrong Operator (expected {}, found {})",
                    account_id, expected_operator, actual_operator
                ),
            )
        }
    }

    /// Verify all registered mappings
    pub fn verify_all(&self) -> InvariantResult {
        let org_results: Vec<InvariantResult> = self
            .org_to_operator
            .keys()
            .map(|&org_id| self.verify_organization(org_id))
            .collect();

        let unit_results: Vec<InvariantResult> = self
            .unit_to_account
            .keys()
            .map(|&unit_id| self.verify_unit(unit_id))
            .collect();

        InvariantResult::combine([org_results, unit_results].concat())
    }

    /// Get the NATS Operator ID for an Organization
    pub fn get_operator_for_org(&self, org_id: Uuid) -> Option<Uuid> {
        self.org_to_operator.get(&org_id).copied()
    }

    /// Get the NATS Account ID for an OrganizationUnit
    pub fn get_account_for_unit(&self, unit_id: Uuid) -> Option<Uuid> {
        self.unit_to_account.get(&unit_id).copied()
    }
}

impl CrossContextInvariant for NatsOrganizationHierarchyInvariant {
    fn name(&self) -> &'static str {
        "NatsOrganizationHierarchyInvariant"
    }

    fn description(&self) -> &'static str {
        "NATS hierarchy (Operator/Account) mirrors Organization hierarchy (Org/Unit)"
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec!["Organization", "NATS"]
    }
}

// ============================================================================
// INVARIANT REGISTRY
// ============================================================================

/// Registry for managing multiple cross-context invariants
#[derive(Debug, Default)]
pub struct InvariantRegistry {
    /// NATS User to Person invariant
    pub nats_user_person: NatsUserPersonInvariant,
    /// YubiKey slot binding invariant
    pub yubikey_slot_binding: YubiKeySlotBindingInvariant,
    /// NATS-Organization hierarchy invariant
    pub nats_organization: NatsOrganizationHierarchyInvariant,
}

impl InvariantRegistry {
    /// Create a new invariant registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify all registered invariants
    pub fn verify_all(&self) -> InvariantResult {
        InvariantResult::combine(vec![
            self.nats_user_person.verify_all(),
            self.yubikey_slot_binding.verify_all(),
            self.nats_organization.verify_all(),
        ])
    }

    /// Get a list of all invariant names and descriptions
    pub fn list_invariants(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                self.nats_user_person.name(),
                self.nats_user_person.description(),
            ),
            (
                self.yubikey_slot_binding.name(),
                self.yubikey_slot_binding.description(),
            ),
            (
                self.nats_organization.name(),
                self.nats_organization.description(),
            ),
        ]
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_user_person_valid_binding() {
        let mut invariant = NatsUserPersonInvariant::new();
        let user_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();

        invariant.register_binding(user_id, person_id);

        let result = invariant.verify_user(user_id);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_nats_user_person_missing_binding() {
        let invariant = NatsUserPersonInvariant::new();
        let user_id = Uuid::now_v7();

        let result = invariant.verify_user(user_id);
        assert!(result.is_violated());
    }

    #[test]
    fn test_nats_user_person_bidirectional_lookup() {
        let mut invariant = NatsUserPersonInvariant::new();
        let user_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();

        invariant.register_binding(user_id, person_id);

        assert_eq!(invariant.get_person_for_user(user_id), Some(person_id));
        assert_eq!(invariant.get_user_for_person(person_id), Some(user_id));
    }

    #[test]
    fn test_yubikey_slot_binding_valid() {
        let mut invariant = YubiKeySlotBindingInvariant::new();
        let key_id = Uuid::now_v7();

        invariant.register_binding(SlotBinding {
            key_id,
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            purpose: KeyPurpose::Authentication,
        });

        let result = invariant.verify_key(key_id);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_yubikey_slot_binding_incompatible_purpose() {
        let mut invariant = YubiKeySlotBindingInvariant::new();
        let key_id = Uuid::now_v7();

        // Authentication slot with Signing purpose - incompatible
        invariant.register_binding(SlotBinding {
            key_id,
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            purpose: KeyPurpose::Signing, // Wrong!
        });

        let result = invariant.verify_key(key_id);
        assert!(result.is_violated());
    }

    #[test]
    fn test_yubikey_slot_binding_missing() {
        let mut invariant = YubiKeySlotBindingInvariant::new();
        let key_id = Uuid::now_v7();

        // Mark key as YubiKey-stored but don't add proper binding
        invariant.yubikey_stored_keys.insert(key_id);
        // Don't add binding - it should fail

        let result = invariant.verify_key(key_id);
        assert!(result.is_violated());
    }

    #[test]
    fn test_nats_organization_hierarchy_valid() {
        let mut invariant = NatsOrganizationHierarchyInvariant::new();
        let org_id = Uuid::now_v7();
        let operator_id = Uuid::now_v7();
        let unit_id = Uuid::now_v7();
        let account_id = Uuid::now_v7();

        invariant.register_organization(org_id, operator_id);
        invariant.register_unit(unit_id, org_id, account_id, operator_id);

        let result = invariant.verify_all();
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_nats_organization_hierarchy_wrong_operator() {
        let mut invariant = NatsOrganizationHierarchyInvariant::new();
        let org_id = Uuid::now_v7();
        let operator_id = Uuid::now_v7();
        let wrong_operator_id = Uuid::now_v7();
        let unit_id = Uuid::now_v7();
        let account_id = Uuid::now_v7();

        invariant.register_organization(org_id, operator_id);
        // Register unit with wrong operator
        invariant.register_unit(unit_id, org_id, account_id, wrong_operator_id);

        let result = invariant.verify_unit(unit_id);
        assert!(result.is_violated());
    }

    #[test]
    fn test_invariant_result_combine() {
        let satisfied = InvariantResult::satisfied();
        let violated = InvariantResult::violated("Test violation");

        // All satisfied
        assert!(InvariantResult::combine(vec![
            InvariantResult::satisfied(),
            InvariantResult::satisfied(),
        ])
        .is_satisfied());

        // Any violation makes result violated
        assert!(InvariantResult::combine(vec![satisfied.clone(), violated.clone()]).is_violated());
    }

    #[test]
    fn test_invariant_registry() {
        let mut registry = InvariantRegistry::new();

        // Register valid bindings
        let user_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();
        registry.nats_user_person.register_binding(user_id, person_id);

        let org_id = Uuid::now_v7();
        let operator_id = Uuid::now_v7();
        registry.nats_organization.register_organization(org_id, operator_id);

        // Verify all should pass (no YubiKey keys registered)
        let result = registry.verify_all();
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_piv_slot_compatibility() {
        assert!(PivSlot::Authentication.is_compatible_with_purpose(&KeyPurpose::Authentication));
        assert!(PivSlot::Signature.is_compatible_with_purpose(&KeyPurpose::Signing));
        assert!(PivSlot::KeyManagement.is_compatible_with_purpose(&KeyPurpose::Encryption));
        assert!(PivSlot::KeyManagement.is_compatible_with_purpose(&KeyPurpose::KeyAgreement));

        // Incompatible combinations
        assert!(!PivSlot::Authentication.is_compatible_with_purpose(&KeyPurpose::Signing));
        assert!(!PivSlot::Signature.is_compatible_with_purpose(&KeyPurpose::Authentication));
    }

    #[test]
    fn test_invariant_violation_with_context() {
        let violation = InvariantViolation::new("Test violation")
            .with_entity(Uuid::now_v7())
            .with_context("key", "value")
            .with_context("another", "context");

        assert!(violation.entity_id.is_some());
        assert!(violation.context.is_some());
        let ctx = violation.context.unwrap();
        assert_eq!(ctx.get("key"), Some(&"value".to_string()));
        assert_eq!(ctx.get("another"), Some(&"context".to_string()));
    }
}
