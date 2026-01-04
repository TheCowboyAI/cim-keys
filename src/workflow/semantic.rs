// Copyright (c) 2025 - Cowboy AI, LLC.

//! Semantic Positioning for Workflow Gaps
//!
//! This module integrates trust chain gaps with conceptual spaces theory,
//! providing semantic positioning for visualization and enhanced prediction.
//!
//! ## Conceptual Space Mapping
//!
//! Each gap is positioned in a 3D semantic space where:
//! - **X-axis (Abstraction)**: Concrete operations → Abstract policies
//! - **Y-axis (Dependency Depth)**: Independent → Highly dependent
//! - **Z-axis (Category)**: Technical (PKI/YubiKey) → Organizational (Domain/Policy)
//!
//! ## Knowledge Level Mapping
//!
//! Gap status maps to knowledge levels:
//! - NotStarted → Unknown
//! - InProgress → KnownUnknown (identified gap)
//! - Implemented → Suspected (partial evidence)
//! - Tested → Suspected (strong evidence)
//! - Verified → Known (proven)

use std::collections::HashMap;
use cim_domain_spaces::{
    Point3, KnowledgeLevel, ConceptId, ConceptualSpaceId,
};
use super::gaps::{TrustChainGap, GapId, GapStatus, GapCategory};

/// Semantic position of a gap in conceptual space
#[derive(Debug, Clone)]
pub struct SemanticPosition {
    /// The gap this position represents
    pub gap_id: GapId,
    /// 3D position in conceptual space
    pub position: Point3<f64>,
    /// Knowledge level based on status
    pub knowledge_level: KnowledgeLevel,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

impl SemanticPosition {
    /// Create a semantic position for a gap
    pub fn from_gap(gap: &TrustChainGap, status: GapStatus) -> Self {
        let position = Self::calculate_position(gap);
        let knowledge_level = Self::status_to_knowledge_level(status);
        let confidence = Self::calculate_confidence(gap, status);

        Self {
            gap_id: gap.id,
            position,
            knowledge_level,
            confidence,
        }
    }

    /// Calculate 3D position based on gap properties
    fn calculate_position(gap: &TrustChainGap) -> Point3<f64> {
        // X-axis: Abstraction level (0.0 = concrete, 1.0 = abstract)
        let abstraction = match gap.category {
            GapCategory::YubiKey => 0.1,  // Very concrete (hardware)
            GapCategory::Pki => 0.3,      // Concrete (cryptographic operations)
            GapCategory::Delegation => 0.5, // Middle (trust relationships)
            GapCategory::Domain => 0.7,   // More abstract (domain concepts)
            GapCategory::Policy => 0.9,   // Most abstract (rules and policies)
        };

        // Y-axis: Dependency depth (0.0 = independent, 1.0 = highly dependent)
        let max_deps = 5.0; // Normalize against expected max dependencies
        let dependency_depth = (gap.dependencies.len() as f64 / max_deps).min(1.0);

        // Z-axis: Priority normalized (0.0 = low priority, 1.0 = high priority)
        let priority_normalized = gap.priority as f64 / 10.0;

        Point3::new(abstraction, dependency_depth, priority_normalized)
    }

    /// Map gap status to knowledge level
    fn status_to_knowledge_level(status: GapStatus) -> KnowledgeLevel {
        match status {
            GapStatus::NotStarted => KnowledgeLevel::Unknown,
            GapStatus::InProgress => KnowledgeLevel::KnownUnknown,
            GapStatus::Implemented => KnowledgeLevel::Suspected,
            GapStatus::Tested => KnowledgeLevel::Suspected,
            GapStatus::Verified => KnowledgeLevel::Known,
        }
    }

    /// Calculate confidence based on evidence
    fn calculate_confidence(gap: &TrustChainGap, status: GapStatus) -> f64 {
        let base_confidence = match status {
            GapStatus::NotStarted => 0.0,
            GapStatus::InProgress => 0.1,
            GapStatus::Implemented => 0.4,
            GapStatus::Tested => 0.7,
            GapStatus::Verified => 0.95,
        };

        // Boost confidence based on evidence
        let evidence_boost = gap.evidence.evidence_score() as f64 * 0.1;

        (base_confidence + evidence_boost).min(1.0)
    }

    /// Euclidean distance to another position
    pub fn distance_to(&self, other: &SemanticPosition) -> f64 {
        let dx = self.position.x - other.position.x;
        let dy = self.position.y - other.position.y;
        let dz = self.position.z - other.position.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Semantic similarity (inverse of distance, normalized)
    pub fn similarity_to(&self, other: &SemanticPosition) -> f64 {
        let distance = self.distance_to(other);
        // Max possible distance in unit cube is sqrt(3) ≈ 1.732
        let max_distance = 3.0_f64.sqrt();
        1.0 - (distance / max_distance)
    }
}

/// Semantic space containing all gap positions
#[derive(Debug, Clone)]
pub struct GapConceptualSpace {
    /// Space identifier
    pub space_id: ConceptualSpaceId,
    /// All gap positions
    pub positions: HashMap<GapId, SemanticPosition>,
    /// Concept IDs for each gap (for cim-domain-spaces integration)
    pub concept_ids: HashMap<GapId, ConceptId>,
}

impl GapConceptualSpace {
    /// Create a new conceptual space from gaps and their statuses
    pub fn new(gaps: &[TrustChainGap], statuses: &HashMap<GapId, GapStatus>) -> Self {
        let space_id = ConceptualSpaceId::new();
        let mut positions = HashMap::new();
        let mut concept_ids = HashMap::new();

        for gap in gaps {
            let status = statuses.get(&gap.id).copied().unwrap_or(GapStatus::NotStarted);
            let position = SemanticPosition::from_gap(gap, status);
            positions.insert(gap.id, position);
            concept_ids.insert(gap.id, ConceptId::new());
        }

        Self {
            space_id,
            positions,
            concept_ids,
        }
    }

    /// Update a single gap's position based on new status
    pub fn update_gap(&mut self, gap: &TrustChainGap, status: GapStatus) {
        let position = SemanticPosition::from_gap(gap, status);
        self.positions.insert(gap.id, position);
    }

    /// Get position for a gap
    pub fn get_position(&self, gap_id: GapId) -> Option<&SemanticPosition> {
        self.positions.get(&gap_id)
    }

    /// Find gaps closest to a given gap (semantic neighbors)
    pub fn nearest_neighbors(&self, gap_id: GapId, count: usize) -> Vec<(GapId, f64)> {
        let Some(target) = self.positions.get(&gap_id) else {
            return Vec::new();
        };

        let mut distances: Vec<(GapId, f64)> = self.positions
            .iter()
            .filter(|(id, _)| **id != gap_id)
            .map(|(id, pos)| (*id, target.distance_to(pos)))
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        distances.truncate(count);
        distances
    }

    /// Find gaps by category region
    pub fn gaps_in_category(&self, category: GapCategory) -> Vec<GapId> {
        let (min_x, max_x) = match category {
            GapCategory::YubiKey => (0.0, 0.2),
            GapCategory::Pki => (0.2, 0.4),
            GapCategory::Delegation => (0.4, 0.6),
            GapCategory::Domain => (0.6, 0.8),
            GapCategory::Policy => (0.8, 1.0),
        };

        self.positions
            .iter()
            .filter(|(_, pos)| pos.position.x >= min_x && pos.position.x < max_x)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Calculate centroid of all gap positions
    pub fn centroid(&self) -> Point3<f64> {
        if self.positions.is_empty() {
            return Point3::new(0.5, 0.5, 0.5);
        }

        let sum = self.positions.values().fold(
            (0.0, 0.0, 0.0),
            |(sx, sy, sz), pos| {
                (sx + pos.position.x, sy + pos.position.y, sz + pos.position.z)
            },
        );

        let n = self.positions.len() as f64;
        Point3::new(sum.0 / n, sum.1 / n, sum.2 / n)
    }

    /// Get knowledge distribution across the space
    pub fn knowledge_distribution(&self) -> HashMap<KnowledgeLevel, usize> {
        let mut distribution = HashMap::new();
        for pos in self.positions.values() {
            *distribution.entry(pos.knowledge_level).or_insert(0) += 1;
        }
        distribution
    }

    /// Average confidence across all gaps
    pub fn average_confidence(&self) -> f64 {
        if self.positions.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.positions.values().map(|p| p.confidence).sum();
        sum / self.positions.len() as f64
    }

    /// Get gaps ordered by confidence (ascending - least confident first)
    pub fn gaps_by_confidence(&self) -> Vec<(GapId, f64)> {
        let mut gaps: Vec<_> = self.positions
            .iter()
            .map(|(id, pos)| (*id, pos.confidence))
            .collect();
        gaps.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        gaps
    }
}

/// Semantic distance calculator for prediction enhancement
pub struct SemanticDistanceCalculator {
    space: GapConceptualSpace,
}

impl SemanticDistanceCalculator {
    /// Create a new calculator from a conceptual space
    pub fn new(space: GapConceptualSpace) -> Self {
        Self { space }
    }

    /// Calculate semantic weight for transitioning from one gap to another
    /// Used to enhance Markov chain predictions
    pub fn transition_weight(&self, from: GapId, to: GapId) -> f64 {
        let Some(from_pos) = self.space.get_position(from) else {
            return 0.5; // Default weight if position unknown
        };
        let Some(to_pos) = self.space.get_position(to) else {
            return 0.5;
        };

        // Closer gaps (semantically similar) get higher transition weight
        let similarity = from_pos.similarity_to(to_pos);

        // Prefer transitions toward higher abstraction (completing foundations first)
        let abstraction_bonus = if to_pos.position.x > from_pos.position.x {
            0.1 // Small bonus for moving toward abstraction
        } else {
            0.0
        };

        // Prefer transitions that increase knowledge level
        let knowledge_bonus = match (from_pos.knowledge_level, to_pos.knowledge_level) {
            (KnowledgeLevel::Known, _) => 0.0, // Already known, no bonus
            (_, KnowledgeLevel::Unknown) => 0.15, // Good to start new work
            _ => 0.05,
        };

        (similarity + abstraction_bonus + knowledge_bonus).min(1.0)
    }

    /// Get all transition weights from a given gap
    pub fn all_transition_weights(&self, from: GapId) -> HashMap<GapId, f64> {
        self.space.positions.keys()
            .filter(|id| **id != from)
            .map(|to| (*to, self.transition_weight(from, *to)))
            .collect()
    }

    /// Recommend next gap based purely on semantic positioning
    pub fn semantic_recommendation(&self, current: Option<GapId>) -> Option<GapId> {
        // Get gaps with lowest confidence (most need work)
        let gaps_by_confidence = self.space.gaps_by_confidence();

        if let Some(current_id) = current {
            // Find closest gap among low-confidence ones
            let weights = self.all_transition_weights(current_id);
            let low_confidence_gaps: Vec<_> = gaps_by_confidence
                .into_iter()
                .filter(|(_, conf)| *conf < 0.5)
                .take(5)
                .collect();

            low_confidence_gaps
                .into_iter()
                .max_by(|a, b| {
                    let weight_a = weights.get(&a.0).unwrap_or(&0.0);
                    let weight_b = weights.get(&b.0).unwrap_or(&0.0);
                    weight_a.partial_cmp(weight_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(id, _)| id)
        } else {
            // No current gap - recommend lowest confidence, lowest dependency gap
            gaps_by_confidence
                .into_iter()
                .find(|(id, _)| {
                    self.space.get_position(*id)
                        .map(|p| p.position.y < 0.3) // Low dependency
                        .unwrap_or(false)
                })
                .map(|(id, _)| id)
        }
    }
}

/// Spherical projection for 3D visualization
/// Projects points onto unit sphere surface
pub struct SphericalProjector;

impl SphericalProjector {
    /// Project a semantic position onto the unit sphere
    pub fn project(position: &SemanticPosition) -> Point3<f64> {
        // Convert cube coordinates to spherical
        // Map [0,1]³ to angles and project to sphere
        let theta = position.position.x * std::f64::consts::PI; // 0 to π
        let phi = position.position.y * 2.0 * std::f64::consts::PI; // 0 to 2π
        let r = 0.5 + position.position.z * 0.5; // Radius varies with priority

        Point3::new(
            r * theta.sin() * phi.cos(),
            r * theta.sin() * phi.sin(),
            r * theta.cos(),
        )
    }

    /// Project all positions in a space
    pub fn project_space(space: &GapConceptualSpace) -> HashMap<GapId, Point3<f64>> {
        space.positions
            .iter()
            .map(|(id, pos)| (*id, Self::project(pos)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to get a gap by ID from all gaps
    fn get_gap(id: GapId) -> TrustChainGap {
        TrustChainGap::all_gaps()
            .into_iter()
            .find(|g| g.id == id)
            .expect("Gap not found")
    }

    /// Helper to get a gap by category
    fn get_gap_by_category(category: GapCategory) -> TrustChainGap {
        TrustChainGap::all_gaps()
            .into_iter()
            .find(|g| g.category == category)
            .expect("Gap not found for category")
    }

    #[test]
    fn test_semantic_position_creation() {
        let gap = get_gap(GapId::CERTIFICATE_CHAIN_VERIFICATION);
        let position = SemanticPosition::from_gap(&gap, GapStatus::NotStarted);

        assert_eq!(position.gap_id, gap.id);
        assert_eq!(position.knowledge_level, KnowledgeLevel::Unknown);
        assert!(position.confidence >= 0.0 && position.confidence <= 1.0);
    }

    #[test]
    fn test_knowledge_level_mapping() {
        assert_eq!(
            SemanticPosition::status_to_knowledge_level(GapStatus::NotStarted),
            KnowledgeLevel::Unknown
        );
        assert_eq!(
            SemanticPosition::status_to_knowledge_level(GapStatus::InProgress),
            KnowledgeLevel::KnownUnknown
        );
        assert_eq!(
            SemanticPosition::status_to_knowledge_level(GapStatus::Implemented),
            KnowledgeLevel::Suspected
        );
        assert_eq!(
            SemanticPosition::status_to_knowledge_level(GapStatus::Verified),
            KnowledgeLevel::Known
        );
    }

    #[test]
    fn test_position_calculation() {
        // PKI gap should be more concrete (lower X)
        let pki_gap = get_gap_by_category(GapCategory::Pki);
        let pki_pos = SemanticPosition::from_gap(&pki_gap, GapStatus::NotStarted);

        // Policy gap should be more abstract (higher X)
        let policy_gap = get_gap_by_category(GapCategory::Policy);
        let policy_pos = SemanticPosition::from_gap(&policy_gap, GapStatus::NotStarted);

        assert!(pki_pos.position.x < policy_pos.position.x);
    }

    #[test]
    fn test_distance_calculation() {
        let gap1 = get_gap(GapId::CERTIFICATE_CHAIN_VERIFICATION);
        let gap2 = get_gap(GapId::TRUST_CHAIN_REFERENCE);

        let pos1 = SemanticPosition::from_gap(&gap1, GapStatus::NotStarted);
        let pos2 = SemanticPosition::from_gap(&gap2, GapStatus::NotStarted);

        let distance = pos1.distance_to(&pos2);
        assert!(distance >= 0.0);

        // Same gap should have zero distance
        assert!((pos1.distance_to(&pos1) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_similarity_calculation() {
        let gap = get_gap(GapId::CERTIFICATE_CHAIN_VERIFICATION);
        let pos = SemanticPosition::from_gap(&gap, GapStatus::NotStarted);

        // Same position should have similarity 1.0
        let similarity = pos.similarity_to(&pos);
        assert!((similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_conceptual_space_creation() {
        let gaps = TrustChainGap::all_gaps();
        let statuses: HashMap<GapId, GapStatus> = HashMap::new();
        let space = GapConceptualSpace::new(&gaps, &statuses);

        assert_eq!(space.positions.len(), gaps.len());
        assert_eq!(space.concept_ids.len(), gaps.len());
    }

    #[test]
    fn test_nearest_neighbors() {
        let gaps = TrustChainGap::all_gaps();
        let statuses: HashMap<GapId, GapStatus> = HashMap::new();
        let space = GapConceptualSpace::new(&gaps, &statuses);

        let neighbors = space.nearest_neighbors(GapId::CERTIFICATE_CHAIN_VERIFICATION, 3);
        assert_eq!(neighbors.len(), 3);

        // Distances should be sorted ascending
        for i in 1..neighbors.len() {
            assert!(neighbors[i].1 >= neighbors[i - 1].1);
        }
    }

    #[test]
    fn test_knowledge_distribution() {
        let gaps = TrustChainGap::all_gaps();
        let statuses: HashMap<GapId, GapStatus> = HashMap::new();
        let space = GapConceptualSpace::new(&gaps, &statuses);

        let distribution = space.knowledge_distribution();
        // All should be Unknown since no statuses provided
        assert_eq!(distribution.get(&KnowledgeLevel::Unknown), Some(&gaps.len()));
    }

    #[test]
    fn test_semantic_distance_calculator() {
        let gaps = TrustChainGap::all_gaps();
        let statuses: HashMap<GapId, GapStatus> = HashMap::new();
        let space = GapConceptualSpace::new(&gaps, &statuses);
        let calculator = SemanticDistanceCalculator::new(space);

        let weight = calculator.transition_weight(
            GapId::CERTIFICATE_CHAIN_VERIFICATION,
            GapId::TRUST_CHAIN_REFERENCE,
        );
        assert!(weight >= 0.0 && weight <= 1.0);
    }

    #[test]
    fn test_spherical_projection() {
        let gap = get_gap(GapId::CERTIFICATE_CHAIN_VERIFICATION);
        let position = SemanticPosition::from_gap(&gap, GapStatus::NotStarted);
        let projected = SphericalProjector::project(&position);

        // Point should be on or inside unit sphere (radius ≤ 1)
        let radius = (projected.x.powi(2) + projected.y.powi(2) + projected.z.powi(2)).sqrt();
        assert!(radius <= 1.1); // Allow small tolerance
    }

    #[test]
    fn test_semantic_recommendation() {
        let gaps = TrustChainGap::all_gaps();
        let statuses: HashMap<GapId, GapStatus> = HashMap::new();
        let space = GapConceptualSpace::new(&gaps, &statuses);
        let calculator = SemanticDistanceCalculator::new(space);

        // Should recommend something when starting fresh
        let recommendation = calculator.semantic_recommendation(None);
        assert!(recommendation.is_some());

        // Should also recommend when we have a current gap
        let recommendation = calculator.semantic_recommendation(Some(GapId::CERTIFICATE_CHAIN_VERIFICATION));
        assert!(recommendation.is_some());
    }

    #[test]
    fn test_confidence_increases_with_status() {
        let gap = get_gap(GapId::CERTIFICATE_CHAIN_VERIFICATION);

        let not_started = SemanticPosition::from_gap(&gap, GapStatus::NotStarted);
        let in_progress = SemanticPosition::from_gap(&gap, GapStatus::InProgress);
        let implemented = SemanticPosition::from_gap(&gap, GapStatus::Implemented);
        let tested = SemanticPosition::from_gap(&gap, GapStatus::Tested);
        let verified = SemanticPosition::from_gap(&gap, GapStatus::Verified);

        assert!(not_started.confidence < in_progress.confidence);
        assert!(in_progress.confidence < implemented.confidence);
        assert!(implemented.confidence < tested.confidence);
        assert!(tested.confidence < verified.confidence);
    }
}
