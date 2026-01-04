// Copyright (c) 2025 - Cowboy AI, LLC.

//! Trust Chain Gap Definitions
//!
//! This module defines the 10 trust chain gaps identified in the
//! Domain Ontology Validation Plan, along with their fulfillment
//! requirements and relationships to domain objects.

use std::collections::HashSet;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a trust chain gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GapId(pub u8);

impl GapId {
    pub const CERTIFICATE_CHAIN_VERIFICATION: GapId = GapId(1);
    pub const TRUST_CHAIN_REFERENCE: GapId = GapId(2);
    pub const YUBIKEY_SLOT_BINDING: GapId = GapId(3);
    pub const DELEGATION_REVOCATION_CASCADE: GapId = GapId(4);
    pub const ORPHANED_KEY_DETECTION: GapId = GapId(5);
    pub const CROSS_ORG_TRUST: GapId = GapId(6);
    pub const SERVICE_ACCOUNT_ACCOUNTABILITY: GapId = GapId(7);
    pub const POLICY_EVALUATION_CACHE: GapId = GapId(8);
    pub const KEY_ROTATION_TRUST: GapId = GapId(9);
    pub const BOOTSTRAP_DOMAIN_DUALITY: GapId = GapId(10);

    pub fn all() -> Vec<GapId> {
        vec![
            Self::CERTIFICATE_CHAIN_VERIFICATION,
            Self::TRUST_CHAIN_REFERENCE,
            Self::YUBIKEY_SLOT_BINDING,
            Self::DELEGATION_REVOCATION_CASCADE,
            Self::ORPHANED_KEY_DETECTION,
            Self::CROSS_ORG_TRUST,
            Self::SERVICE_ACCOUNT_ACCOUNTABILITY,
            Self::POLICY_EVALUATION_CACHE,
            Self::KEY_ROTATION_TRUST,
            Self::BOOTSTRAP_DOMAIN_DUALITY,
        ]
    }

    /// Convert to a deterministic UUID for graph representation
    ///
    /// Uses a namespace UUID + gap ID to generate a stable, unique UUID
    /// that will be consistent across runs.
    pub fn as_uuid(&self) -> Uuid {
        // Namespace UUID for workflow gaps (deterministic)
        const WORKFLOW_NAMESPACE: Uuid = Uuid::from_bytes([
            0x77, 0x6f, 0x72, 0x6b, // "work"
            0x66, 0x6c, 0x6f, 0x77, // "flow"
            0x67, 0x61, 0x70, 0x73, // "gaps"
            0x00, 0x00, 0x00, 0x00, // padding
        ]);

        // Create deterministic UUID from namespace + gap id
        let mut bytes = *WORKFLOW_NAMESPACE.as_bytes();
        bytes[15] = self.0; // Put gap ID in last byte
        Uuid::from_bytes(bytes)
    }
}

impl std::fmt::Display for GapId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            1 => write!(f, "Certificate Chain Verification"),
            2 => write!(f, "Trust Chain Reference"),
            3 => write!(f, "YubiKey Slot Binding"),
            4 => write!(f, "Delegation Revocation Cascade"),
            5 => write!(f, "Orphaned Key Detection"),
            6 => write!(f, "Cross-Organization Trust"),
            7 => write!(f, "Service Account Accountability"),
            8 => write!(f, "Policy Evaluation Cache"),
            9 => write!(f, "Key Rotation Trust"),
            10 => write!(f, "Bootstrap/Domain Duality"),
            _ => write!(f, "Unknown Gap {}", self.0),
        }
    }
}

/// Status of a trust chain gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapStatus {
    /// Gap has not been started
    NotStarted,
    /// Gap is currently being worked on
    InProgress,
    /// Gap implementation is complete but not tested
    Implemented,
    /// Gap has been tested
    Tested,
    /// Gap is fully verified and complete
    Verified,
}

impl GapStatus {
    pub fn progress_percentage(&self) -> u8 {
        match self {
            GapStatus::NotStarted => 0,
            GapStatus::InProgress => 25,
            GapStatus::Implemented => 50,
            GapStatus::Tested => 75,
            GapStatus::Verified => 100,
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, GapStatus::Verified)
    }

    pub fn next_status(&self) -> Option<GapStatus> {
        match self {
            GapStatus::NotStarted => Some(GapStatus::InProgress),
            GapStatus::InProgress => Some(GapStatus::Implemented),
            GapStatus::Implemented => Some(GapStatus::Tested),
            GapStatus::Tested => Some(GapStatus::Verified),
            GapStatus::Verified => None,
        }
    }
}

/// Category of trust chain gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GapCategory {
    /// PKI and certificate-related gaps
    Pki,
    /// Delegation and authorization gaps
    Delegation,
    /// YubiKey and hardware security gaps
    YubiKey,
    /// Policy and evaluation gaps
    Policy,
    /// Domain model gaps
    Domain,
}

/// A domain object required to fulfill a gap
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredObject {
    /// Type of the object (aggregate, entity, value object)
    pub object_type: ObjectType,
    /// Name of the object
    pub name: String,
    /// Module path where the object is defined
    pub module_path: String,
    /// Specific aspect of the object that needs work
    pub aspect: String,
    /// Whether this object's aspect is fulfilled
    pub fulfilled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Aggregate,
    Entity,
    ValueObject,
    Service,
    Port,
    Adapter,
}

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Aggregate => write!(f, "Aggregate"),
            ObjectType::Entity => write!(f, "Entity"),
            ObjectType::ValueObject => write!(f, "Value Object"),
            ObjectType::Service => write!(f, "Service"),
            ObjectType::Port => write!(f, "Port"),
            ObjectType::Adapter => write!(f, "Adapter"),
        }
    }
}

/// A trust chain gap with its requirements and fulfillment state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustChainGap {
    /// Unique identifier
    pub id: GapId,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Category of the gap
    pub category: GapCategory,
    /// Current status
    pub status: GapStatus,
    /// Priority (1-10, higher is more important)
    pub priority: u8,
    /// Required objects to fulfill this gap
    pub required_objects: Vec<RequiredObject>,
    /// Gaps that must be completed before this one
    pub dependencies: Vec<GapId>,
    /// Sprint where this gap was addressed (if any)
    pub addressed_in_sprint: Option<u32>,
    /// When the gap was last updated
    pub last_updated: DateTime<Utc>,
    /// Evidence of fulfillment (test counts, etc.)
    pub evidence: GapEvidence,
}

/// Evidence that a gap has been properly addressed
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GapEvidence {
    /// Number of unit tests
    pub unit_tests: u32,
    /// Number of property tests
    pub property_tests: u32,
    /// Number of BDD scenarios
    pub bdd_scenarios: u32,
    /// Documentation completeness (0.0-1.0)
    pub documentation_score: f32,
    /// Whether cryptographic verification is implemented
    pub crypto_verified: bool,
}

impl GapEvidence {
    pub fn total_tests(&self) -> u32 {
        self.unit_tests + self.property_tests + self.bdd_scenarios
    }

    pub fn evidence_score(&self) -> f32 {
        let test_score = (self.total_tests() as f32 / 50.0).min(1.0);
        let crypto_score = if self.crypto_verified { 0.2 } else { 0.0 };

        test_score * 0.6 + self.documentation_score * 0.2 + crypto_score
    }
}

impl TrustChainGap {
    /// Create all 10 trust chain gaps with their initial state
    pub fn all_gaps() -> Vec<TrustChainGap> {
        vec![
            TrustChainGap::certificate_chain_verification(),
            TrustChainGap::trust_chain_reference(),
            TrustChainGap::yubikey_slot_binding(),
            TrustChainGap::delegation_revocation_cascade(),
            TrustChainGap::orphaned_key_detection(),
            TrustChainGap::cross_org_trust(),
            TrustChainGap::service_account_accountability(),
            TrustChainGap::policy_evaluation_cache(),
            TrustChainGap::key_rotation_trust(),
            TrustChainGap::bootstrap_domain_duality(),
        ]
    }

    /// Gap 1: Certificate Chain Verification (COMPLETED in Sprint 41)
    fn certificate_chain_verification() -> Self {
        TrustChainGap {
            id: GapId::CERTIFICATE_CHAIN_VERIFICATION,
            name: "Certificate Chain Cryptographic Verification".to_string(),
            description: "CertificateChain::verify() must perform actual cryptographic \
                signature verification, not just return Ok(())".to_string(),
            category: GapCategory::Pki,
            status: GapStatus::Verified, // Completed in Sprint 41
            priority: 10,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::ValueObject,
                    name: "CertificateChain".to_string(),
                    module_path: "src/value_objects/core.rs".to_string(),
                    aspect: "verify() method".to_string(),
                    fulfilled: true,
                },
                RequiredObject {
                    object_type: ObjectType::ValueObject,
                    name: "Certificate".to_string(),
                    module_path: "src/value_objects/core.rs".to_string(),
                    aspect: "verify_signature() method".to_string(),
                    fulfilled: true,
                },
                RequiredObject {
                    object_type: ObjectType::ValueObject,
                    name: "TrustPath".to_string(),
                    module_path: "src/value_objects/core.rs".to_string(),
                    aspect: "result type".to_string(),
                    fulfilled: true,
                },
            ],
            dependencies: vec![],
            addressed_in_sprint: Some(41),
            last_updated: Utc::now(),
            evidence: GapEvidence {
                unit_tests: 16,
                property_tests: 15,
                bdd_scenarios: 40,
                documentation_score: 0.9,
                crypto_verified: true,
            },
        }
    }

    /// Gap 2: Trust Chain Reference Verification
    fn trust_chain_reference() -> Self {
        TrustChainGap {
            id: GapId::TRUST_CHAIN_REFERENCE,
            name: "Trust Chain Reference Cryptographic Proofs".to_string(),
            description: "TrustChainReference.is_valid is a simple boolean flag. \
                Need cryptographic verification with proofs.".to_string(),
            category: GapCategory::Pki,
            status: GapStatus::NotStarted,
            priority: 9,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::ValueObject,
                    name: "TrustChainReference".to_string(),
                    module_path: "src/domain/bootstrap.rs".to_string(),
                    aspect: "verify() method with PkiContext".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::ValueObject,
                    name: "VerifiedTrustChain".to_string(),
                    module_path: "src/domain/trust.rs".to_string(),
                    aspect: "new type with verification proof".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::Service,
                    name: "PkiContext".to_string(),
                    module_path: "src/domain/pki.rs".to_string(),
                    aspect: "context for verification".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![GapId::CERTIFICATE_CHAIN_VERIFICATION],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence::default(),
        }
    }

    /// Gap 3: YubiKey Slot Binding Validation
    fn yubikey_slot_binding() -> Self {
        TrustChainGap {
            id: GapId::YUBIKEY_SLOT_BINDING,
            name: "YubiKey Slot Binding Constraints".to_string(),
            description: "Keys stored on YubiKey need validated slot bindings \
                with proper purpose/slot compatibility checks.".to_string(),
            category: GapCategory::YubiKey,
            status: GapStatus::NotStarted,
            priority: 8,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::ValueObject,
                    name: "SlotBinding".to_string(),
                    module_path: "src/domain/yubikey.rs".to_string(),
                    aspect: "slot compatibility validation".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::Aggregate,
                    name: "YubiKeyAggregate".to_string(),
                    module_path: "src/domain/yubikey.rs".to_string(),
                    aspect: "slot allocation invariants".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence::default(),
        }
    }

    /// Gap 4: Delegation Revocation Cascade
    fn delegation_revocation_cascade() -> Self {
        TrustChainGap {
            id: GapId::DELEGATION_REVOCATION_CASCADE,
            name: "Delegation Revocation Cascade".to_string(),
            description: "Delegation revocation must cascade to all dependent \
                delegations (transitive closure).".to_string(),
            category: GapCategory::Delegation,
            status: GapStatus::NotStarted,
            priority: 9,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::Aggregate,
                    name: "DelegationAggregate".to_string(),
                    module_path: "src/domain/delegation.rs".to_string(),
                    aspect: "revoke() with cascade".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::Entity,
                    name: "Delegation".to_string(),
                    module_path: "src/domain/bootstrap.rs".to_string(),
                    aspect: "derives_from tracking".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence {
                unit_tests: 0,
                property_tests: 0,
                bdd_scenarios: 30, // Created in Sprint 41
                documentation_score: 0.5,
                crypto_verified: false,
            },
        }
    }

    /// Gap 5: Orphaned Key Detection
    fn orphaned_key_detection() -> Self {
        TrustChainGap {
            id: GapId::ORPHANED_KEY_DETECTION,
            name: "Orphaned Key Detection".to_string(),
            description: "Detect keys that have no valid ownership chain \
                (owner deleted, organization dissolved, etc.).".to_string(),
            category: GapCategory::Pki,
            status: GapStatus::NotStarted,
            priority: 7,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::Service,
                    name: "OrphanKeyDetector".to_string(),
                    module_path: "src/domain/pki.rs".to_string(),
                    aspect: "detect_orphans() method".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::Aggregate,
                    name: "KeyAggregate".to_string(),
                    module_path: "src/domain/pki.rs".to_string(),
                    aspect: "ownership_valid() check".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![GapId::DELEGATION_REVOCATION_CASCADE],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence::default(),
        }
    }

    /// Gap 6: Cross-Organization Trust Enforcement
    fn cross_org_trust() -> Self {
        TrustChainGap {
            id: GapId::CROSS_ORG_TRUST,
            name: "Cross-Organization Trust Enforcement".to_string(),
            description: "Trust relationships between organizations must be \
                explicitly established and verified.".to_string(),
            category: GapCategory::Domain,
            status: GapStatus::NotStarted,
            priority: 6,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::Entity,
                    name: "OrganizationTrust".to_string(),
                    module_path: "src/domain/organization.rs".to_string(),
                    aspect: "inter-org trust relationship".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::Service,
                    name: "TrustEnforcer".to_string(),
                    module_path: "src/domain/trust.rs".to_string(),
                    aspect: "cross-org validation".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![GapId::TRUST_CHAIN_REFERENCE],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence::default(),
        }
    }

    /// Gap 7: Service Account Accountability
    fn service_account_accountability() -> Self {
        TrustChainGap {
            id: GapId::SERVICE_ACCOUNT_ACCOUNTABILITY,
            name: "Service Account Accountability".to_string(),
            description: "Service accounts must have clear ownership and \
                audit trails linking actions to responsible persons.".to_string(),
            category: GapCategory::Domain,
            status: GapStatus::NotStarted,
            priority: 7,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::Entity,
                    name: "ServiceAccount".to_string(),
                    module_path: "src/domain/bootstrap.rs".to_string(),
                    aspect: "responsible_person tracking".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::Service,
                    name: "AuditService".to_string(),
                    module_path: "src/domain/audit.rs".to_string(),
                    aspect: "action attribution".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence::default(),
        }
    }

    /// Gap 8: Policy Evaluation Cache
    fn policy_evaluation_cache() -> Self {
        TrustChainGap {
            id: GapId::POLICY_EVALUATION_CACHE,
            name: "Policy Evaluation Cache".to_string(),
            description: "Policy evaluations should be cached with proper \
                invalidation on policy or context changes.".to_string(),
            category: GapCategory::Policy,
            status: GapStatus::NotStarted,
            priority: 5,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::Service,
                    name: "PolicyCache".to_string(),
                    module_path: "src/policy/cache.rs".to_string(),
                    aspect: "caching with invalidation".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::Service,
                    name: "PolicyEngine".to_string(),
                    module_path: "src/policy/policy_engine.rs".to_string(),
                    aspect: "cache integration".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence::default(),
        }
    }

    /// Gap 9: Key Rotation Trust Gap
    fn key_rotation_trust() -> Self {
        TrustChainGap {
            id: GapId::KEY_ROTATION_TRUST,
            name: "Key Rotation Trust Gap".to_string(),
            description: "During key rotation, there's a window where both old \
                and new keys are valid. Need explicit trust transition.".to_string(),
            category: GapCategory::Pki,
            status: GapStatus::NotStarted,
            priority: 8,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::Aggregate,
                    name: "KeyRotationSaga".to_string(),
                    module_path: "src/domain/pki.rs".to_string(),
                    aspect: "rotation state machine".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::ValueObject,
                    name: "KeyRotationWindow".to_string(),
                    module_path: "src/domain/pki.rs".to_string(),
                    aspect: "overlap period management".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![GapId::CERTIFICATE_CHAIN_VERIFICATION],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence::default(),
        }
    }

    /// Gap 10: Bootstrap/Domain Type Duality
    fn bootstrap_domain_duality() -> Self {
        TrustChainGap {
            id: GapId::BOOTSTRAP_DOMAIN_DUALITY,
            name: "Bootstrap/Domain Type Duality".to_string(),
            description: "Bootstrap types (e.g., BootstrapPerson) and domain types \
                (e.g., Person) can diverge, causing state inconsistency.".to_string(),
            category: GapCategory::Domain,
            status: GapStatus::NotStarted,
            priority: 6,
            required_objects: vec![
                RequiredObject {
                    object_type: ObjectType::Service,
                    name: "TypeReconciler".to_string(),
                    module_path: "src/domain/reconciliation.rs".to_string(),
                    aspect: "bootstrap-domain sync".to_string(),
                    fulfilled: false,
                },
                RequiredObject {
                    object_type: ObjectType::ValueObject,
                    name: "ReconciliationReport".to_string(),
                    module_path: "src/domain/reconciliation.rs".to_string(),
                    aspect: "divergence detection".to_string(),
                    fulfilled: false,
                },
            ],
            dependencies: vec![],
            addressed_in_sprint: None,
            last_updated: Utc::now(),
            evidence: GapEvidence::default(),
        }
    }

    /// Check if this gap can be started (all dependencies fulfilled)
    pub fn can_start(&self, fulfilled_gaps: &HashSet<GapId>) -> bool {
        self.dependencies.iter().all(|dep| fulfilled_gaps.contains(dep))
    }

    /// Get overall progress percentage
    pub fn progress_percentage(&self) -> u8 {
        let status_progress = self.status.progress_percentage();
        let object_progress = if self.required_objects.is_empty() {
            100
        } else {
            let fulfilled = self.required_objects.iter().filter(|o| o.fulfilled).count();
            ((fulfilled as f32 / self.required_objects.len() as f32) * 100.0) as u8
        };

        // Weighted average: status 60%, objects 40%
        ((status_progress as u16 * 60 + object_progress as u16 * 40) / 100) as u8
    }

    /// Get unfulfilled required objects
    pub fn unfulfilled_objects(&self) -> Vec<&RequiredObject> {
        self.required_objects.iter().filter(|o| !o.fulfilled).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_id_display() {
        assert_eq!(
            GapId::CERTIFICATE_CHAIN_VERIFICATION.to_string(),
            "Certificate Chain Verification"
        );
    }

    #[test]
    fn test_gap_status_progression() {
        assert_eq!(GapStatus::NotStarted.next_status(), Some(GapStatus::InProgress));
        assert_eq!(GapStatus::InProgress.next_status(), Some(GapStatus::Implemented));
        assert_eq!(GapStatus::Implemented.next_status(), Some(GapStatus::Tested));
        assert_eq!(GapStatus::Tested.next_status(), Some(GapStatus::Verified));
        assert_eq!(GapStatus::Verified.next_status(), None);
    }

    #[test]
    fn test_all_gaps_created() {
        let gaps = TrustChainGap::all_gaps();
        assert_eq!(gaps.len(), 10);
    }

    #[test]
    fn test_gap_dependencies() {
        let gaps = TrustChainGap::all_gaps();

        // Certificate chain has no dependencies
        let cert_chain = gaps.iter().find(|g| g.id == GapId::CERTIFICATE_CHAIN_VERIFICATION).unwrap();
        assert!(cert_chain.dependencies.is_empty());

        // Trust chain reference depends on certificate chain
        let trust_ref = gaps.iter().find(|g| g.id == GapId::TRUST_CHAIN_REFERENCE).unwrap();
        assert!(trust_ref.dependencies.contains(&GapId::CERTIFICATE_CHAIN_VERIFICATION));
    }

    #[test]
    fn test_can_start_with_dependencies() {
        let gap = TrustChainGap::trust_chain_reference();

        // Cannot start without dependencies
        let empty_set = HashSet::new();
        assert!(!gap.can_start(&empty_set));

        // Can start with dependency fulfilled
        let mut fulfilled = HashSet::new();
        fulfilled.insert(GapId::CERTIFICATE_CHAIN_VERIFICATION);
        assert!(gap.can_start(&fulfilled));
    }

    #[test]
    fn test_progress_percentage() {
        let gap = TrustChainGap::certificate_chain_verification();
        assert_eq!(gap.status, GapStatus::Verified);
        assert_eq!(gap.progress_percentage(), 100);
    }

    #[test]
    fn test_evidence_score() {
        let evidence = GapEvidence {
            unit_tests: 16,
            property_tests: 15,
            bdd_scenarios: 40,
            documentation_score: 0.9,
            crypto_verified: true,
        };

        let score = evidence.evidence_score();
        assert!(score > 0.8); // Should be high with good evidence
    }
}
