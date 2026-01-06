// Copyright (c) 2025 - Cowboy AI, LLC.

//! Conceptual Space for CIM-Keys Domain
//!
//! This module implements Gärdenfors' Conceptual Spaces theory for the cim-keys domain.
//! Each domain concept is positioned in an 8-dimensional semantic space with explicit
//! quality dimensions, enabling geometric reasoning about concept similarity and relationships.
//!
//! # Quality Dimensions
//!
//! | Dimension | Symbol | Description |
//! |-----------|--------|-------------|
//! | Authority Level | α | Position in PKI hierarchy (leaf=0, root=1) |
//! | Trust Scope | σ | Breadth of trust (self=0, global=1) |
//! | Temporal Validity | τ | Remaining validity as fraction |
//! | Cryptographic Strength | κ | Algorithm strength normalized |
//! | Revocability | ρ | Speed of revocation propagation |
//! | Hardware Binding | η | Key material protection level |
//! | Delegation Depth | δ | Normalized path length in delegation chain |
//! | Audit Completeness | ω | Evidence coverage for compliance |
//!
//! # Example
//!
//! ```ignore
//! use cim_keys::domain::conceptual_space::{ConceptPosition, prototypes};
//!
//! let org = prototypes::ORGANIZATION;
//! let person = prototypes::PERSON;
//!
//! let similarity = org.similarity(&person);
//! assert!(similarity < 0.7); // Different clusters
//!
//! let nats_op = prototypes::NATS_OPERATOR;
//! let org_nats_sim = org.similarity(&nats_op);
//! assert!(org_nats_sim > 0.9); // Same cluster (authority entities)
//! ```

use serde::{Deserialize, Serialize};

/// Quality dimensions for cim-keys conceptual space.
///
/// Each concept is positioned in an 8-dimensional semantic space where:
/// - Dimensions are normalized to [0.0, 1.0]
/// - Distance defines semantic dissimilarity
/// - Similarity = 1 - normalized_distance
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConceptPosition {
    /// Authority level: 0.0 (leaf) to 1.0 (root)
    ///
    /// Represents position in PKI/organizational hierarchy.
    /// - RootCA: 1.0
    /// - IntermediateCA: 0.5
    /// - LeafCertificate: 0.0
    pub authority: f64,

    /// Trust scope: 0.0 (self) to 1.0 (global)
    ///
    /// Represents breadth of trust relationships.
    /// - Self-signed: 0.0
    /// - Cross-domain: 1.0
    pub trust_scope: f64,

    /// Temporal validity: 0.0 (expired) to 1.0 (perpetual)
    ///
    /// Represents remaining validity as fraction of maximum lifetime.
    pub temporal: f64,

    /// Cryptographic strength: 0.0 (weak) to 1.0 (quantum-safe)
    ///
    /// Normalized algorithm strength:
    /// - RSA-1024: ~0.2
    /// - Ed25519: ~0.8
    /// - Post-quantum: 1.0
    pub crypto_strength: f64,

    /// Revocability: 0.0 (irrevocable) to 1.0 (instant)
    ///
    /// Speed of revocation propagation.
    /// Root CAs are nearly irrevocable (~0.1), leaf certs revoke quickly (~0.9).
    pub revocability: f64,

    /// Hardware binding: 0.0 (software) to 1.0 (HSM)
    ///
    /// Key material protection level.
    /// - Software key: 0.0
    /// - YubiKey: 1.0
    pub hardware_binding: f64,

    /// Delegation depth: 0.0 (direct) to 1.0 (transitive)
    ///
    /// Normalized path length in delegation chain.
    /// Direct ownership: 0.0, deeply delegated: approaching 1.0.
    pub delegation_depth: f64,

    /// Audit completeness: 0.0 (none) to 1.0 (full chain)
    ///
    /// Evidence coverage for compliance.
    pub audit_completeness: f64,
}

impl ConceptPosition {
    /// Create a new concept position with explicit dimension values.
    pub const fn new(
        authority: f64,
        trust_scope: f64,
        temporal: f64,
        crypto_strength: f64,
        revocability: f64,
        hardware_binding: f64,
        delegation_depth: f64,
        audit_completeness: f64,
    ) -> Self {
        Self {
            authority,
            trust_scope,
            temporal,
            crypto_strength,
            revocability,
            hardware_binding,
            delegation_depth,
            audit_completeness,
        }
    }

    /// Calculate Euclidean distance between two concepts in 8D space.
    ///
    /// Distance ranges from 0.0 (identical) to sqrt(8) ≈ 2.83 (maximally different).
    pub fn distance(&self, other: &Self) -> f64 {
        let sum_sq = (self.authority - other.authority).powi(2)
            + (self.trust_scope - other.trust_scope).powi(2)
            + (self.temporal - other.temporal).powi(2)
            + (self.crypto_strength - other.crypto_strength).powi(2)
            + (self.revocability - other.revocability).powi(2)
            + (self.hardware_binding - other.hardware_binding).powi(2)
            + (self.delegation_depth - other.delegation_depth).powi(2)
            + (self.audit_completeness - other.audit_completeness).powi(2);
        sum_sq.sqrt()
    }

    /// Calculate normalized similarity (0.0 to 1.0).
    ///
    /// - 1.0: Identical positions
    /// - 0.0: Maximally different (opposite corners of hypercube)
    ///
    /// Thresholds:
    /// - Similarity > 0.90: Synonyms (require disambiguation)
    /// - 0.70 < Similarity <= 0.90: Related concepts
    /// - 0.50 < Similarity <= 0.70: Distinct but connected
    /// - Similarity <= 0.50: Unrelated (different bounded contexts)
    pub fn similarity(&self, other: &Self) -> f64 {
        let max_distance = (8.0_f64).sqrt(); // sqrt(8) for 8 dimensions
        1.0 - (self.distance(other) / max_distance)
    }

    /// Convert to 3D for visualization using stereographic projection.
    ///
    /// Dimension reduction:
    /// - x: (authority + trust_scope) / 2.0
    /// - y: (crypto_strength + hardware_binding) / 2.0
    /// - z: (temporal + audit_completeness) / 2.0
    ///
    /// Returns (x, y, z) in [0.0, 1.0]³.
    pub fn to_3d(&self) -> (f64, f64, f64) {
        let x = (self.authority + self.trust_scope) / 2.0;
        let y = (self.crypto_strength + self.hardware_binding) / 2.0;
        let z = (self.temporal + self.audit_completeness) / 2.0;
        (x, y, z)
    }

    /// Get the dimension values as an array.
    pub fn as_array(&self) -> [f64; 8] {
        [
            self.authority,
            self.trust_scope,
            self.temporal,
            self.crypto_strength,
            self.revocability,
            self.hardware_binding,
            self.delegation_depth,
            self.audit_completeness,
        ]
    }

    /// Create from an array of dimension values.
    pub fn from_array(arr: [f64; 8]) -> Self {
        Self {
            authority: arr[0],
            trust_scope: arr[1],
            temporal: arr[2],
            crypto_strength: arr[3],
            revocability: arr[4],
            hardware_binding: arr[5],
            delegation_depth: arr[6],
            audit_completeness: arr[7],
        }
    }
}

impl Default for ConceptPosition {
    /// Default is the center of the conceptual space.
    fn default() -> Self {
        Self::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5)
    }
}

/// Prototype positions for core domain concepts.
///
/// These represent the canonical semantic positions for each concept type.
pub mod prototypes {
    use super::ConceptPosition;

    // ========================================================================
    // Authority Entities Cluster
    // ========================================================================

    /// Organization: Root authority entity representing a legal/logical entity.
    pub const ORGANIZATION: ConceptPosition = ConceptPosition::new(
        0.9,  // authority: high (root of org hierarchy)
        0.8,  // trust_scope: high (global within domain)
        0.9,  // temporal: long-lived
        0.8,  // crypto_strength: high
        0.3,  // revocability: low (hard to revoke orgs)
        0.5,  // hardware_binding: mixed
        0.0,  // delegation_depth: direct (no delegation)
        0.9,  // audit_completeness: high
    );

    /// OrganizationUnit: Subdivision of Organization with delegated authority.
    pub const ORGANIZATION_UNIT: ConceptPosition = ConceptPosition::new(
        0.5,  // authority: intermediate
        0.5,  // trust_scope: moderate
        0.8,  // temporal: long-lived
        0.8,  // crypto_strength: high
        0.5,  // revocability: moderate
        0.5,  // hardware_binding: mixed
        0.3,  // delegation_depth: some delegation
        0.8,  // audit_completeness: high
    );

    /// NatsOperator: NATS system-level authority (maps to Organization).
    pub const NATS_OPERATOR: ConceptPosition = ConceptPosition::new(
        0.9,  // authority: high (NATS root)
        1.0,  // trust_scope: global (NATS-wide)
        0.9,  // temporal: long-lived
        0.9,  // crypto_strength: high
        0.2,  // revocability: low
        0.9,  // hardware_binding: high
        0.0,  // delegation_depth: direct
        0.9,  // audit_completeness: high
    );

    /// NatsAccount: NATS account-level authority (maps to OrganizationUnit).
    pub const NATS_ACCOUNT: ConceptPosition = ConceptPosition::new(
        0.5,  // authority: intermediate
        0.6,  // trust_scope: moderate
        0.8,  // temporal: long-lived
        0.9,  // crypto_strength: high
        0.5,  // revocability: moderate
        0.7,  // hardware_binding: moderate-high
        0.3,  // delegation_depth: some delegation
        0.8,  // audit_completeness: high
    );

    // ========================================================================
    // Identity Entities Cluster
    // ========================================================================

    /// Person: Human individual with identity and potential key ownership.
    pub const PERSON: ConceptPosition = ConceptPosition::new(
        0.1,  // authority: low (leaf in hierarchy)
        0.2,  // trust_scope: limited
        0.7,  // temporal: medium (employment duration)
        0.8,  // crypto_strength: high
        0.8,  // revocability: high (can be revoked quickly)
        0.7,  // hardware_binding: moderate-high (uses YubiKey)
        0.5,  // delegation_depth: moderate
        0.7,  // audit_completeness: moderate-high
    );

    /// NatsUser: NATS user credential (maps to Person within NatsAccount).
    pub const NATS_USER: ConceptPosition = ConceptPosition::new(
        0.1,  // authority: low (leaf)
        0.2,  // trust_scope: limited
        0.6,  // temporal: medium
        0.8,  // crypto_strength: high
        0.8,  // revocability: high
        0.5,  // hardware_binding: moderate
        0.6,  // delegation_depth: moderate
        0.7,  // audit_completeness: moderate-high
    );

    /// Role: Named permission set assignable to Person.
    pub const ROLE: ConceptPosition = ConceptPosition::new(
        0.3,  // authority: low-moderate
        0.3,  // trust_scope: limited
        0.9,  // temporal: long-lived (roles persist)
        0.0,  // crypto_strength: N/A
        0.6,  // revocability: moderate
        0.0,  // hardware_binding: N/A
        0.4,  // delegation_depth: can be delegated
        0.8,  // audit_completeness: high
    );

    // ========================================================================
    // Cryptographic Artifacts Cluster
    // ========================================================================

    /// CryptographicKey: Asymmetric key pair (public + private components).
    pub const CRYPTOGRAPHIC_KEY: ConceptPosition = ConceptPosition::new(
        0.2,  // authority: low (artifact, not entity)
        0.3,  // trust_scope: depends on usage
        0.7,  // temporal: medium (key rotation)
        0.9,  // crypto_strength: high
        0.7,  // revocability: high
        0.7,  // hardware_binding: often on YubiKey
        0.4,  // delegation_depth: can be delegated
        0.8,  // audit_completeness: high
    );

    /// RootCA: Offline HSM-protected root certificate.
    pub const ROOT_CA: ConceptPosition = ConceptPosition::new(
        1.0,  // authority: maximum
        1.0,  // trust_scope: global
        1.0,  // temporal: perpetual (very long)
        1.0,  // crypto_strength: maximum
        0.1,  // revocability: nearly irrevocable
        1.0,  // hardware_binding: HSM required
        0.0,  // delegation_depth: direct (root of chain)
        1.0,  // audit_completeness: maximum
    );

    /// IntermediateCA: Online intermediate for daily operations.
    pub const INTERMEDIATE_CA: ConceptPosition = ConceptPosition::new(
        0.5,  // authority: intermediate
        0.6,  // trust_scope: moderate
        0.8,  // temporal: long but not perpetual
        0.9,  // crypto_strength: high
        0.5,  // revocability: moderate
        0.8,  // hardware_binding: high
        0.3,  // delegation_depth: delegated from root
        0.9,  // audit_completeness: high
    );

    /// LeafCertificate: End-entity certificate for authentication.
    pub const LEAF_CERTIFICATE: ConceptPosition = ConceptPosition::new(
        0.0,  // authority: minimum (leaf)
        0.2,  // trust_scope: limited
        0.5,  // temporal: medium (annual renewal)
        0.8,  // crypto_strength: high
        0.9,  // revocability: high
        0.5,  // hardware_binding: varies
        0.7,  // delegation_depth: end of chain
        0.7,  // audit_completeness: moderate-high
    );

    /// CertificateChain: Ordered sequence from leaf to root certificate.
    pub const CERTIFICATE_CHAIN: ConceptPosition = ConceptPosition::new(
        0.5,  // authority: spans hierarchy
        0.5,  // trust_scope: moderate
        0.6,  // temporal: limited by weakest link
        0.8,  // crypto_strength: high
        0.5,  // revocability: depends on components
        0.6,  // hardware_binding: moderate
        0.5,  // delegation_depth: represents full chain
        0.9,  // audit_completeness: high (verifiable)
    );

    /// TrustChain: Verified chain of TrustLinks with cryptographic proofs.
    pub const TRUST_CHAIN: ConceptPosition = ConceptPosition::new(
        0.4,  // authority: spans hierarchy
        0.5,  // trust_scope: moderate
        0.8,  // temporal: depends on verification
        0.0,  // crypto_strength: N/A (relationship)
        0.4,  // revocability: cascade effects
        0.5,  // hardware_binding: N/A
        0.5,  // delegation_depth: represents path
        0.95, // audit_completeness: very high (verified)
    );

    // ========================================================================
    // Hardware Security Cluster
    // ========================================================================

    /// YubiKey: Hardware security module (HSM) for key storage.
    pub const YUBIKEY: ConceptPosition = ConceptPosition::new(
        0.3,  // authority: low-moderate
        0.3,  // trust_scope: limited
        0.95, // temporal: very long-lived
        0.95, // crypto_strength: very high
        0.7,  // revocability: moderate-high
        1.0,  // hardware_binding: maximum
        0.0,  // delegation_depth: direct
        0.8,  // audit_completeness: high
    );

    /// PivSlot: Specific slot on YubiKey (9A, 9C, 9D, 9E).
    pub const PIV_SLOT: ConceptPosition = ConceptPosition::new(
        0.2,  // authority: low
        0.2,  // trust_scope: limited
        0.9,  // temporal: long-lived
        0.9,  // crypto_strength: high
        0.6,  // revocability: moderate
        1.0,  // hardware_binding: maximum
        0.0,  // delegation_depth: direct
        0.7,  // audit_completeness: moderate-high
    );

    /// SlotBinding: Relationship binding PivSlot to Person and Key.
    pub const SLOT_BINDING: ConceptPosition = ConceptPosition::new(
        0.2,  // authority: low
        0.2,  // trust_scope: limited
        0.8,  // temporal: medium-long
        0.0,  // crypto_strength: N/A (relationship)
        0.7,  // revocability: moderate-high
        0.9,  // hardware_binding: very high
        0.3,  // delegation_depth: some delegation
        0.8,  // audit_completeness: high
    );

    /// Location: Physical or logical storage location.
    pub const LOCATION: ConceptPosition = ConceptPosition::new(
        0.0,  // authority: none
        0.1,  // trust_scope: very limited
        0.9,  // temporal: long-lived
        0.0,  // crypto_strength: N/A
        0.0,  // revocability: N/A
        1.0,  // hardware_binding: maximum (physical)
        0.0,  // delegation_depth: direct
        0.5,  // audit_completeness: moderate
    );

    // ========================================================================
    // Relationships Cluster
    // ========================================================================

    /// KeyOwnership: Relationship: Person owns CryptographicKey.
    pub const KEY_OWNERSHIP: ConceptPosition = ConceptPosition::new(
        0.2,  // authority: low
        0.3,  // trust_scope: limited
        0.7,  // temporal: medium
        0.0,  // crypto_strength: N/A (relationship)
        0.5,  // revocability: moderate
        0.5,  // hardware_binding: varies
        0.4,  // delegation_depth: can be delegated
        0.9,  // audit_completeness: high
    );

    /// Delegation: Relationship: Person delegates authority to Person.
    pub const DELEGATION: ConceptPosition = ConceptPosition::new(
        0.3,  // authority: low-moderate
        0.4,  // trust_scope: moderate
        0.6,  // temporal: medium
        0.0,  // crypto_strength: N/A (relationship)
        0.7,  // revocability: high
        0.3,  // hardware_binding: low
        0.8,  // delegation_depth: high (transitive)
        0.8,  // audit_completeness: high
    );

    /// TrustLink: Single verified trust relationship between concepts.
    pub const TRUST_LINK: ConceptPosition = ConceptPosition::new(
        0.4,  // authority: moderate
        0.5,  // trust_scope: moderate
        0.8,  // temporal: depends on verification
        0.0,  // crypto_strength: N/A (relationship)
        0.4,  // revocability: cascade effects
        0.5,  // hardware_binding: N/A
        0.5,  // delegation_depth: moderate
        0.95, // audit_completeness: very high
    );

    /// Policy: Named set of constraints on operations.
    pub const POLICY: ConceptPosition = ConceptPosition::new(
        0.4,  // authority: moderate
        0.5,  // trust_scope: moderate
        0.9,  // temporal: long-lived
        0.0,  // crypto_strength: N/A
        0.4,  // revocability: moderate
        0.0,  // hardware_binding: N/A
        0.3,  // delegation_depth: some delegation
        0.9,  // audit_completeness: high
    );
}

/// Attention weights for concepts in different contexts.
///
/// Salience determines which concepts are most relevant in each operational context.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttentionWeights {
    /// Weight during bootstrap/initialization (0.0 to 1.0).
    pub bootstrap: f64,
    /// Weight during normal operations (0.0 to 1.0).
    pub operational: f64,
    /// Weight during audit/compliance checks (0.0 to 1.0).
    pub audit: f64,
    /// Weight during recovery/incident response (0.0 to 1.0).
    pub recovery: f64,
}

impl AttentionWeights {
    /// Create new attention weights.
    pub const fn new(bootstrap: f64, operational: f64, audit: f64, recovery: f64) -> Self {
        Self { bootstrap, operational, audit, recovery }
    }

    /// Calculate average attention weight across all contexts.
    pub fn average(&self) -> f64 {
        (self.bootstrap + self.operational + self.audit + self.recovery) / 4.0
    }
}

/// Attention weights for core concepts.
pub mod attention {
    use super::AttentionWeights;

    pub const ORGANIZATION: AttentionWeights = AttentionWeights::new(0.95, 0.60, 0.70, 0.50);
    pub const ORGANIZATION_UNIT: AttentionWeights = AttentionWeights::new(0.80, 0.75, 0.65, 0.40);
    pub const PERSON: AttentionWeights = AttentionWeights::new(0.90, 0.85, 0.80, 0.70);
    pub const LOCATION: AttentionWeights = AttentionWeights::new(0.70, 0.30, 0.50, 0.85);
    pub const CRYPTOGRAPHIC_KEY: AttentionWeights = AttentionWeights::new(0.75, 0.70, 0.85, 0.90);
    pub const CERTIFICATE: AttentionWeights = AttentionWeights::new(0.70, 0.80, 0.90, 0.75);
    pub const CERTIFICATE_CHAIN: AttentionWeights = AttentionWeights::new(0.50, 0.60, 0.90, 0.60);
    pub const TRUST_CHAIN: AttentionWeights = AttentionWeights::new(0.45, 0.55, 0.95, 0.55);
    pub const NATS_OPERATOR: AttentionWeights = AttentionWeights::new(0.85, 0.70, 0.60, 0.45);
    pub const NATS_ACCOUNT: AttentionWeights = AttentionWeights::new(0.75, 0.80, 0.55, 0.40);
    pub const NATS_USER: AttentionWeights = AttentionWeights::new(0.65, 0.95, 0.50, 0.35);
    pub const YUBIKEY: AttentionWeights = AttentionWeights::new(0.90, 0.50, 0.75, 0.95);
    pub const PIV_SLOT: AttentionWeights = AttentionWeights::new(0.80, 0.45, 0.70, 0.90);
    pub const KEY_OWNERSHIP: AttentionWeights = AttentionWeights::new(0.85, 0.75, 0.90, 0.80);
    pub const DELEGATION: AttentionWeights = AttentionWeights::new(0.60, 0.85, 0.80, 0.85);
    pub const TRUST_LINK: AttentionWeights = AttentionWeights::new(0.40, 0.50, 0.95, 0.60);
    pub const POLICY: AttentionWeights = AttentionWeights::new(0.70, 0.90, 0.85, 0.65);
}

/// Semantic similarity thresholds for concept relationships.
pub mod thresholds {
    /// Concepts with similarity > 0.90 are synonyms (require disambiguation).
    pub const SYNONYM_THRESHOLD: f64 = 0.90;
    /// Concepts with 0.70 < similarity <= 0.90 are related (often used together).
    pub const RELATED_THRESHOLD: f64 = 0.70;
    /// Concepts with 0.50 < similarity <= 0.70 are distinct but connected.
    pub const DISTINCT_THRESHOLD: f64 = 0.50;
    // Concepts with similarity <= 0.50 are unrelated (different bounded contexts).
}

// ============================================================================
// KNOWLEDGE LEVEL TRACKING
// ============================================================================

/// Knowledge level for domain concepts (Bloom's Taxonomy adapted for PKI domain).
///
/// This represents the depth of understanding and evidence supporting a concept.
/// The ubiquitous language is a PROJECTION derived from concepts at each knowledge level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KnowledgeLevel {
    /// Concept exists but details unknown (events observed, not understood).
    Awareness = 1,

    /// Basic understanding of concept (can describe its role).
    Understanding = 2,

    /// Can work with concept practically (can use in workflows).
    Application = 3,

    /// Deep understanding, can analyze (can debug and troubleshoot).
    Analysis = 4,

    /// Can synthesize new concepts (can extend and compose).
    Synthesis = 5,

    /// Can evaluate and improve concepts (domain expert level).
    Evaluation = 6,
}

impl KnowledgeLevel {
    /// Convert to normalized score (0.0 to 1.0).
    pub fn as_score(&self) -> f64 {
        (*self as u8 as f64) / 6.0
    }

    /// Get description suitable for ubiquitous language projection.
    pub fn description(&self) -> &'static str {
        match self {
            KnowledgeLevel::Awareness => "Aware of concept existence",
            KnowledgeLevel::Understanding => "Basic understanding",
            KnowledgeLevel::Application => "Practical application",
            KnowledgeLevel::Analysis => "Deep analysis capability",
            KnowledgeLevel::Synthesis => "Can synthesize new concepts",
            KnowledgeLevel::Evaluation => "Domain expert evaluation",
        }
    }
}

impl Default for KnowledgeLevel {
    fn default() -> Self {
        KnowledgeLevel::Awareness
    }
}

impl std::fmt::Display for KnowledgeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

// ============================================================================
// EVIDENCE SCORE (Projection Input)
// ============================================================================

/// Evidence supporting knowledge claims about a concept.
///
/// This is the INPUT to the ubiquitous language projection - the raw evidence
/// from tests, scenarios, and production usage that determines concept confidence.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EvidenceScore {
    /// Number of unit tests validating this concept.
    pub test_coverage: u32,

    /// Number of BDD scenarios (Gherkin specifications).
    pub bdd_scenarios: u32,

    /// Number of property-based tests (proptest).
    pub property_tests: u32,

    /// Documentation completeness (0.0 to 1.0).
    pub documentation_score: f64,

    /// Production usage count (events observed).
    pub usage_count: u64,

    /// Last verification timestamp (for temporal decay).
    pub last_verified: Option<chrono::DateTime<chrono::Utc>>,
}

impl EvidenceScore {
    /// Create a new evidence score with all metrics.
    pub fn new(
        test_coverage: u32,
        bdd_scenarios: u32,
        property_tests: u32,
        documentation_score: f64,
        usage_count: u64,
    ) -> Self {
        Self {
            test_coverage,
            bdd_scenarios,
            property_tests,
            documentation_score,
            usage_count,
            last_verified: Some(chrono::Utc::now()),
        }
    }

    /// Calculate weighted total score (0.0 to 1.0).
    ///
    /// Weights:
    /// - Unit tests: 30%
    /// - BDD scenarios: 25%
    /// - Property tests: 20%
    /// - Documentation: 15%
    /// - Production usage: 10%
    pub fn total_score(&self) -> f64 {
        const TEST_WEIGHT: f64 = 0.30;
        const BDD_WEIGHT: f64 = 0.25;
        const PROP_WEIGHT: f64 = 0.20;
        const DOC_WEIGHT: f64 = 0.15;
        const USAGE_WEIGHT: f64 = 0.10;

        // Normalize: 100 tests = full score, 20 BDD = full, 10 property = full
        let test_score = (self.test_coverage as f64 / 100.0).min(1.0);
        let bdd_score = (self.bdd_scenarios as f64 / 20.0).min(1.0);
        let prop_score = (self.property_tests as f64 / 10.0).min(1.0);
        let usage_score = (self.usage_count as f64 / 1000.0).min(1.0);

        TEST_WEIGHT * test_score
            + BDD_WEIGHT * bdd_score
            + PROP_WEIGHT * prop_score
            + DOC_WEIGHT * self.documentation_score.min(1.0)
            + USAGE_WEIGHT * usage_score
    }

    /// Derive knowledge level from evidence score.
    ///
    /// This is part of the PROJECTION: evidence → knowledge level.
    pub fn derive_knowledge_level(&self) -> KnowledgeLevel {
        let score = self.total_score();
        if score >= 0.90 {
            KnowledgeLevel::Evaluation
        } else if score >= 0.75 {
            KnowledgeLevel::Synthesis
        } else if score >= 0.60 {
            KnowledgeLevel::Analysis
        } else if score >= 0.45 {
            KnowledgeLevel::Application
        } else if score >= 0.25 {
            KnowledgeLevel::Understanding
        } else {
            KnowledgeLevel::Awareness
        }
    }

    /// Check if evidence is stale (older than 30 days).
    pub fn is_stale(&self) -> bool {
        match self.last_verified {
            Some(t) => {
                let age = chrono::Utc::now() - t;
                age.num_days() > 30
            }
            None => true,
        }
    }
}

// ============================================================================
// CONCEPT KNOWLEDGE (Composed Type for Projection)
// ============================================================================

/// Complete knowledge representation for a concept.
///
/// This combines position in conceptual space, evidence, and derived knowledge level.
/// The ubiquitous language projection uses this to determine term definitions,
/// relationships, and prohibited aliases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptKnowledge {
    /// Canonical term in ubiquitous language.
    pub term: String,

    /// Position in 8D conceptual space.
    pub position: ConceptPosition,

    /// Evidence supporting this concept's understanding.
    pub evidence: EvidenceScore,

    /// Bounded context this concept belongs to.
    pub context: String,

    /// Prohibited aliases (terms that must NOT be used).
    pub prohibited_aliases: Vec<String>,

    /// Related terms (similarity > RELATED_THRESHOLD).
    pub related_terms: Vec<String>,
}

impl ConceptKnowledge {
    /// Create new concept knowledge.
    pub fn new(
        term: impl Into<String>,
        position: ConceptPosition,
        context: impl Into<String>,
    ) -> Self {
        Self {
            term: term.into(),
            position,
            evidence: EvidenceScore::default(),
            context: context.into(),
            prohibited_aliases: Vec::new(),
            related_terms: Vec::new(),
        }
    }

    /// Get derived knowledge level from evidence.
    pub fn knowledge_level(&self) -> KnowledgeLevel {
        self.evidence.derive_knowledge_level()
    }

    /// Add a prohibited alias (anti-vocabulary).
    pub fn with_prohibited_alias(mut self, alias: impl Into<String>) -> Self {
        self.prohibited_aliases.push(alias.into());
        self
    }

    /// Add multiple prohibited aliases.
    pub fn with_prohibited_aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.prohibited_aliases.extend(aliases.into_iter().map(|s| s.into()));
        self
    }

    /// Add a related term.
    pub fn with_related_term(mut self, term: impl Into<String>) -> Self {
        self.related_terms.push(term.into());
        self
    }

    /// Calculate similarity to another concept.
    pub fn similarity_to(&self, other: &Self) -> f64 {
        self.position.similarity(&other.position)
    }

    /// Check if this concept is a synonym of another (similarity > SYNONYM_THRESHOLD).
    pub fn is_synonym_of(&self, other: &Self) -> bool {
        self.similarity_to(other) > thresholds::SYNONYM_THRESHOLD
    }

    /// Check if this concept is related to another (similarity > RELATED_THRESHOLD).
    pub fn is_related_to(&self, other: &Self) -> bool {
        self.similarity_to(other) > thresholds::RELATED_THRESHOLD
    }
}

// ============================================================================
// PROHIBITED ALIASES (Anti-Vocabulary for Ubiquitous Language)
// ============================================================================

/// Prohibited aliases for ubiquitous language enforcement.
///
/// These are terms that MUST NOT be used in code, documentation, or conversation.
/// The canonical term should be used instead.
pub mod prohibited_aliases {
    /// Organization: These terms are prohibited, use "Organization".
    pub const ORGANIZATION: &[&str] = &["Company", "Firm", "Enterprise", "Corp", "Business"];

    /// Person: These terms are prohibited, use "Person".
    pub const PERSON: &[&str] = &["User", "Member", "Employee", "Individual", "Human"];

    /// CryptographicKey: These terms are prohibited, use "CryptographicKey" or "Key".
    pub const KEY: &[&str] = &["KeyPair", "CryptoKey", "PrivateKey", "SecretKey"];

    /// Certificate: These terms are prohibited, use "Certificate".
    pub const CERTIFICATE: &[&str] = &["Cert", "X509", "PKICert", "X509Cert"];

    /// YubiKey: These terms are prohibited, use "YubiKey".
    pub const YUBIKEY: &[&str] = &["Token", "HSM", "SmartCard", "SecurityKey", "FIDO"];

    /// Location: These terms are prohibited, use "Location".
    pub const LOCATION: &[&str] = &["Site", "Place", "Venue", "Address"];

    /// NatsOperator: These terms are prohibited, use "NatsOperator".
    pub const NATS_OPERATOR: &[&str] = &["Operator", "SystemOperator", "NATSOperator"];

    /// NatsAccount: These terms are prohibited, use "NatsAccount".
    pub const NATS_ACCOUNT: &[&str] = &["Account", "Namespace", "Tenant", "NATSAccount"];

    /// NatsUser: These terms are prohibited, use "NatsUser".
    pub const NATS_USER: &[&str] = &["NATSUser", "Subscriber", "Client"];

    /// Delegation: These terms are prohibited, use "Delegation".
    pub const DELEGATION: &[&str] = &["Permission", "Grant", "Authorization", "Allowance"];

    /// Check if a term is prohibited and return the canonical replacement.
    pub fn check_term(term: &str) -> Option<&'static str> {
        let term_lower = term.to_lowercase();

        for alias in ORGANIZATION {
            if alias.to_lowercase() == term_lower {
                return Some("Organization");
            }
        }
        for alias in PERSON {
            if alias.to_lowercase() == term_lower {
                return Some("Person");
            }
        }
        for alias in KEY {
            if alias.to_lowercase() == term_lower {
                return Some("CryptographicKey");
            }
        }
        for alias in CERTIFICATE {
            if alias.to_lowercase() == term_lower {
                return Some("Certificate");
            }
        }
        for alias in YUBIKEY {
            if alias.to_lowercase() == term_lower {
                return Some("YubiKey");
            }
        }
        for alias in LOCATION {
            if alias.to_lowercase() == term_lower {
                return Some("Location");
            }
        }
        for alias in NATS_OPERATOR {
            if alias.to_lowercase() == term_lower {
                return Some("NatsOperator");
            }
        }
        for alias in NATS_ACCOUNT {
            if alias.to_lowercase() == term_lower {
                return Some("NatsAccount");
            }
        }
        for alias in NATS_USER {
            if alias.to_lowercase() == term_lower {
                return Some("NatsUser");
            }
        }
        for alias in DELEGATION {
            if alias.to_lowercase() == term_lower {
                return Some("Delegation");
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_identical() {
        let pos = ConceptPosition::default();
        assert!((pos.distance(&pos) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_distance_maximum() {
        let min = ConceptPosition::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let max = ConceptPosition::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        let expected = (8.0_f64).sqrt();
        assert!((min.distance(&max) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_similarity_identical() {
        let pos = prototypes::ORGANIZATION;
        assert!((pos.similarity(&pos) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_similarity_maximum_difference() {
        let min = ConceptPosition::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let max = ConceptPosition::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        assert!((min.similarity(&max) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_organization_nats_operator_similarity() {
        // These should be highly similar (same cluster)
        let sim = prototypes::ORGANIZATION.similarity(&prototypes::NATS_OPERATOR);
        assert!(sim > thresholds::RELATED_THRESHOLD, "Organization and NatsOperator should be related");
    }

    #[test]
    fn test_person_nats_user_similarity() {
        // These should be highly similar (maps to each other)
        let sim = prototypes::PERSON.similarity(&prototypes::NATS_USER);
        assert!(sim > thresholds::RELATED_THRESHOLD, "Person and NatsUser should be related");
    }

    #[test]
    fn test_yubikey_location_similarity() {
        // Both in hardware cluster
        let sim = prototypes::YUBIKEY.similarity(&prototypes::LOCATION);
        assert!(sim > thresholds::DISTINCT_THRESHOLD, "YubiKey and Location should be connected");
    }

    #[test]
    fn test_root_ca_leaf_cert_dissimilarity() {
        // Different ends of hierarchy
        let sim = prototypes::ROOT_CA.similarity(&prototypes::LEAF_CERTIFICATE);
        assert!(sim < thresholds::RELATED_THRESHOLD, "RootCA and LeafCertificate should be less related");
    }

    #[test]
    fn test_to_3d_bounds() {
        let pos = prototypes::ORGANIZATION;
        let (x, y, z) = pos.to_3d();
        assert!(x >= 0.0 && x <= 1.0);
        assert!(y >= 0.0 && y <= 1.0);
        assert!(z >= 0.0 && z <= 1.0);
    }

    #[test]
    fn test_attention_weights_average() {
        let avg = attention::PERSON.average();
        assert!((avg - 0.8125).abs() < 1e-10); // (0.90 + 0.85 + 0.80 + 0.70) / 4
    }

    #[test]
    fn test_array_round_trip() {
        let original = prototypes::YUBIKEY;
        let arr = original.as_array();
        let restored = ConceptPosition::from_array(arr);
        assert!((original.distance(&restored) - 0.0).abs() < 1e-10);
    }

    // ========================================================================
    // Knowledge Level Tests
    // ========================================================================

    #[test]
    fn test_knowledge_level_ordering() {
        assert!(KnowledgeLevel::Awareness < KnowledgeLevel::Understanding);
        assert!(KnowledgeLevel::Understanding < KnowledgeLevel::Application);
        assert!(KnowledgeLevel::Application < KnowledgeLevel::Analysis);
        assert!(KnowledgeLevel::Analysis < KnowledgeLevel::Synthesis);
        assert!(KnowledgeLevel::Synthesis < KnowledgeLevel::Evaluation);
    }

    #[test]
    fn test_knowledge_level_score() {
        assert!((KnowledgeLevel::Awareness.as_score() - 1.0/6.0).abs() < 1e-10);
        assert!((KnowledgeLevel::Evaluation.as_score() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_knowledge_level_display() {
        assert_eq!(format!("{}", KnowledgeLevel::Awareness), "Aware of concept existence");
        assert_eq!(format!("{}", KnowledgeLevel::Evaluation), "Domain expert evaluation");
    }

    // ========================================================================
    // Evidence Score Tests
    // ========================================================================

    #[test]
    fn test_evidence_score_default() {
        let score = EvidenceScore::default();
        assert_eq!(score.test_coverage, 0);
        assert_eq!(score.bdd_scenarios, 0);
        assert!((score.total_score() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_evidence_score_full() {
        let score = EvidenceScore::new(100, 20, 10, 1.0, 1000);
        assert!((score.total_score() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_evidence_score_partial() {
        let score = EvidenceScore::new(50, 10, 5, 0.5, 500);
        // 0.30 * 0.5 + 0.25 * 0.5 + 0.20 * 0.5 + 0.15 * 0.5 + 0.10 * 0.5 = 0.5
        assert!((score.total_score() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_evidence_derives_knowledge_level() {
        // No evidence -> Awareness
        let none = EvidenceScore::default();
        assert_eq!(none.derive_knowledge_level(), KnowledgeLevel::Awareness);

        // Full evidence -> Evaluation
        let full = EvidenceScore::new(100, 20, 10, 1.0, 1000);
        assert_eq!(full.derive_knowledge_level(), KnowledgeLevel::Evaluation);

        // Half evidence (0.5) -> Application (0.45-0.60 range)
        let half = EvidenceScore::new(50, 10, 5, 0.5, 500);
        assert_eq!(half.derive_knowledge_level(), KnowledgeLevel::Application);

        // Higher evidence (>0.6) -> Analysis
        let higher = EvidenceScore::new(70, 15, 7, 0.7, 700);
        assert_eq!(higher.derive_knowledge_level(), KnowledgeLevel::Analysis);
    }

    #[test]
    fn test_evidence_staleness() {
        let fresh = EvidenceScore::new(10, 5, 2, 0.5, 100);
        assert!(!fresh.is_stale());

        let no_timestamp = EvidenceScore::default();
        assert!(no_timestamp.is_stale());
    }

    // ========================================================================
    // Concept Knowledge Tests
    // ========================================================================

    #[test]
    fn test_concept_knowledge_creation() {
        let ck = ConceptKnowledge::new("Organization", prototypes::ORGANIZATION, "Organization");
        assert_eq!(ck.term, "Organization");
        assert_eq!(ck.context, "Organization");
        assert_eq!(ck.knowledge_level(), KnowledgeLevel::Awareness);
    }

    #[test]
    fn test_concept_knowledge_with_aliases() {
        let ck = ConceptKnowledge::new("Person", prototypes::PERSON, "Organization")
            .with_prohibited_aliases(["User", "Member", "Employee"]);
        assert_eq!(ck.prohibited_aliases.len(), 3);
        assert!(ck.prohibited_aliases.contains(&"User".to_string()));
    }

    #[test]
    fn test_concept_knowledge_similarity() {
        let org = ConceptKnowledge::new("Organization", prototypes::ORGANIZATION, "Organization");
        let nats_op = ConceptKnowledge::new("NatsOperator", prototypes::NATS_OPERATOR, "NATS");

        assert!(org.is_related_to(&nats_op));
        assert!(!org.is_synonym_of(&nats_op));
    }

    // ========================================================================
    // Prohibited Aliases Tests
    // ========================================================================

    #[test]
    fn test_prohibited_alias_check() {
        assert_eq!(prohibited_aliases::check_term("Company"), Some("Organization"));
        assert_eq!(prohibited_aliases::check_term("User"), Some("Person"));
        assert_eq!(prohibited_aliases::check_term("KeyPair"), Some("CryptographicKey"));
        assert_eq!(prohibited_aliases::check_term("Cert"), Some("Certificate"));
        assert_eq!(prohibited_aliases::check_term("Token"), Some("YubiKey"));
    }

    #[test]
    fn test_prohibited_alias_case_insensitive() {
        assert_eq!(prohibited_aliases::check_term("company"), Some("Organization"));
        assert_eq!(prohibited_aliases::check_term("COMPANY"), Some("Organization"));
        assert_eq!(prohibited_aliases::check_term("Company"), Some("Organization"));
    }

    #[test]
    fn test_prohibited_alias_canonical_allowed() {
        // Canonical terms should not be flagged
        assert_eq!(prohibited_aliases::check_term("Organization"), None);
        assert_eq!(prohibited_aliases::check_term("Person"), None);
        assert_eq!(prohibited_aliases::check_term("YubiKey"), None);
    }
}
