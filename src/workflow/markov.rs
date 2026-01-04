// Copyright (c) 2025 - Cowboy AI, LLC.

//! Markov Chain for Workflow Prediction
//!
//! This module implements a Markov chain that predicts the most likely
//! next step in the trust chain gap fulfillment workflow based on:
//! - Current state (which gaps are fulfilled)
//! - Historical transition patterns
//! - Domain dependencies
//! - Priority weights

use std::collections::HashMap;
use super::gaps::{GapId, TrustChainGap};

/// Transition probability from one gap to another
#[derive(Debug, Clone)]
pub struct TransitionProbability {
    /// Source gap (current state)
    pub from: GapId,
    /// Target gap (next state)
    pub to: GapId,
    /// Probability (0.0 to 1.0)
    pub probability: f64,
    /// Reason for this probability
    pub reason: String,
}

/// Predicted next step with probability and context
#[derive(Debug, Clone)]
pub struct PredictedStep {
    /// The recommended gap to work on
    pub gap_id: GapId,
    /// Probability this is the best next step
    pub probability: f64,
    /// Why this step is recommended
    pub reasoning: Vec<String>,
    /// Expected effort (relative scale 1-10)
    pub expected_effort: u8,
    /// Potential blockers
    pub blockers: Vec<String>,
}

/// Workflow state for Markov chain calculations
#[derive(Debug, Clone, Default)]
pub struct WorkflowState {
    /// Currently fulfilled gaps
    pub fulfilled: Vec<GapId>,
    /// Gap currently being worked on (if any)
    pub in_progress: Option<GapId>,
    /// History of transitions (for learning)
    pub history: Vec<(GapId, GapId)>,
}

impl WorkflowState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fulfilled(fulfilled: Vec<GapId>) -> Self {
        Self {
            fulfilled,
            in_progress: None,
            history: Vec::new(),
        }
    }

    pub fn is_gap_available(&self, gap_id: GapId) -> bool {
        !self.fulfilled.contains(&gap_id) && self.in_progress != Some(gap_id)
    }
}

/// Markov chain for predicting workflow steps
pub struct WorkflowMarkovChain {
    /// Transition matrix: from_gap -> (to_gap -> probability)
    transitions: HashMap<Option<GapId>, HashMap<GapId, f64>>,
    /// All gaps for reference
    gaps: Vec<TrustChainGap>,
    /// Weights for different factors
    weights: PredictionWeights,
}

/// Weights for different prediction factors
#[derive(Debug, Clone)]
pub struct PredictionWeights {
    /// Weight for dependency satisfaction
    pub dependency_weight: f64,
    /// Weight for priority
    pub priority_weight: f64,
    /// Weight for historical patterns
    pub history_weight: f64,
    /// Weight for effort (prefer lower effort)
    pub effort_weight: f64,
    /// Weight for evidence completeness
    pub evidence_weight: f64,
}

impl Default for PredictionWeights {
    fn default() -> Self {
        Self {
            dependency_weight: 0.35,
            priority_weight: 0.25,
            history_weight: 0.15,
            effort_weight: 0.15,
            evidence_weight: 0.10,
        }
    }
}

impl WorkflowMarkovChain {
    /// Create a new Markov chain from gaps
    pub fn new(gaps: Vec<TrustChainGap>) -> Self {
        let mut chain = Self {
            transitions: HashMap::new(),
            gaps,
            weights: PredictionWeights::default(),
        };
        chain.initialize_transitions();
        chain
    }

    /// Create with custom weights
    pub fn with_weights(gaps: Vec<TrustChainGap>, weights: PredictionWeights) -> Self {
        let mut chain = Self {
            transitions: HashMap::new(),
            gaps,
            weights,
        };
        chain.initialize_transitions();
        chain
    }

    /// Initialize transition probabilities based on domain knowledge
    fn initialize_transitions(&mut self) {
        // Initial state (None) -> first gap transitions
        let initial_transitions = self.compute_initial_transitions();
        self.transitions.insert(None, initial_transitions);

        // Transitions from each gap to the next
        for gap in &self.gaps {
            let from_transitions = self.compute_from_transitions(gap.id);
            self.transitions.insert(Some(gap.id), from_transitions);
        }
    }

    /// Compute transitions from initial state
    fn compute_initial_transitions(&self) -> HashMap<GapId, f64> {
        let mut transitions = HashMap::new();
        let mut total_weight = 0.0;

        for gap in &self.gaps {
            // Gaps with no dependencies can be started first
            if gap.dependencies.is_empty() {
                let weight = gap.priority as f64;
                transitions.insert(gap.id, weight);
                total_weight += weight;
            }
        }

        // Normalize to probabilities
        for prob in transitions.values_mut() {
            *prob /= total_weight;
        }

        transitions
    }

    /// Compute transitions from a specific gap
    fn compute_from_transitions(&self, from: GapId) -> HashMap<GapId, f64> {
        let mut transitions = HashMap::new();
        let mut total_weight = 0.0;

        for gap in &self.gaps {
            if gap.id == from {
                continue;
            }

            // Base weight from priority
            let mut weight = gap.priority as f64 / 10.0;

            // Bonus for natural dependency chains
            if gap.dependencies.contains(&from) {
                weight *= 2.0; // Strong preference for dependent gaps
            }

            // Bonus for same category
            let from_gap = self.gaps.iter().find(|g| g.id == from);
            if let Some(from_gap) = from_gap {
                if from_gap.category == gap.category {
                    weight *= 1.3; // Prefer staying in same category
                }
            }

            if weight > 0.0 {
                transitions.insert(gap.id, weight);
                total_weight += weight;
            }
        }

        // Normalize
        if total_weight > 0.0 {
            for prob in transitions.values_mut() {
                *prob /= total_weight;
            }
        }

        transitions
    }

    /// Predict the best next steps given current state
    pub fn predict_next_steps(&self, state: &WorkflowState, count: usize) -> Vec<PredictedStep> {
        let mut candidates: Vec<(GapId, f64, Vec<String>)> = Vec::new();

        // Get base probabilities from Markov chain
        let base_probs = if let Some(current) = state.in_progress {
            self.transitions.get(&Some(current)).cloned().unwrap_or_default()
        } else if let Some(last) = state.fulfilled.last() {
            self.transitions.get(&Some(*last)).cloned().unwrap_or_default()
        } else {
            self.transitions.get(&None).cloned().unwrap_or_default()
        };

        // Score each unfulfilled gap
        for gap in &self.gaps {
            if !state.is_gap_available(gap.id) {
                continue;
            }

            let mut score = base_probs.get(&gap.id).copied().unwrap_or(0.1);
            let mut reasons = Vec::new();

            // Factor 1: Dependencies satisfied
            let deps_satisfied = gap.dependencies.iter()
                .all(|dep| state.fulfilled.contains(dep));

            if deps_satisfied {
                score += self.weights.dependency_weight;
                reasons.push("All dependencies satisfied".to_string());
            } else {
                score *= 0.1; // Heavy penalty for unmet dependencies
                let unmet: Vec<_> = gap.dependencies.iter()
                    .filter(|dep| !state.fulfilled.contains(dep))
                    .map(|dep| dep.to_string())
                    .collect();
                reasons.push(format!("Blocked by: {}", unmet.join(", ")));
            }

            // Factor 2: Priority
            let priority_score = gap.priority as f64 / 10.0 * self.weights.priority_weight;
            score += priority_score;
            if gap.priority >= 8 {
                reasons.push(format!("High priority ({})", gap.priority));
            }

            // Factor 3: Evidence momentum (partial work already done)
            if gap.evidence.total_tests() > 0 {
                let evidence_bonus = gap.evidence.evidence_score() as f64 * self.weights.evidence_weight;
                score += evidence_bonus;
                reasons.push(format!("Partial evidence exists ({} tests)", gap.evidence.total_tests()));
            }

            // Factor 4: Natural workflow progression
            if let Some(last_fulfilled) = state.fulfilled.last() {
                if gap.dependencies.contains(last_fulfilled) {
                    score += 0.2;
                    reasons.push("Natural next step in dependency chain".to_string());
                }
            }

            candidates.push((gap.id, score, reasons));
        }

        // Sort by score descending
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N and normalize probabilities
        let total_score: f64 = candidates.iter().take(count).map(|(_, s, _)| s).sum();

        candidates.into_iter()
            .take(count)
            .map(|(gap_id, score, reasons)| {
                let gap = self.gaps.iter().find(|g| g.id == gap_id).unwrap();
                let probability = if total_score > 0.0 { score / total_score } else { 0.0 };

                PredictedStep {
                    gap_id,
                    probability,
                    reasoning: reasons,
                    expected_effort: self.estimate_effort(gap),
                    blockers: self.identify_blockers(gap, state),
                }
            })
            .collect()
    }

    /// Estimate effort for a gap (1-10 scale)
    fn estimate_effort(&self, gap: &TrustChainGap) -> u8 {
        let base_effort = gap.required_objects.len() as u8;
        let complexity_factor = match gap.category {
            super::gaps::GapCategory::Pki => 2,
            super::gaps::GapCategory::Delegation => 2,
            super::gaps::GapCategory::YubiKey => 3,
            super::gaps::GapCategory::Policy => 1,
            super::gaps::GapCategory::Domain => 1,
        };

        let evidence_reduction = if gap.evidence.total_tests() > 0 {
            (gap.evidence.evidence_score() * 2.0f32) as u8
        } else {
            0
        };

        (base_effort * complexity_factor).saturating_sub(evidence_reduction).clamp(1, 10)
    }

    /// Identify blockers for a gap
    fn identify_blockers(&self, gap: &TrustChainGap, state: &WorkflowState) -> Vec<String> {
        let mut blockers = Vec::new();

        // Check dependencies
        for dep in &gap.dependencies {
            if !state.fulfilled.contains(dep) {
                blockers.push(format!("Requires {} to be completed first", dep));
            }
        }

        // Check for unfulfilled objects
        let unfulfilled = gap.unfulfilled_objects();
        if !unfulfilled.is_empty() {
            blockers.push(format!(
                "{} required objects need implementation",
                unfulfilled.len()
            ));
        }

        blockers
    }

    /// Update the chain based on observed transition
    pub fn observe_transition(&mut self, from: Option<GapId>, to: GapId) {
        let transitions = self.transitions.entry(from).or_insert_with(HashMap::new);

        // Increase probability for observed transition
        let current = transitions.entry(to).or_insert(0.1);
        *current *= 1.1; // 10% boost

        // Renormalize
        let total: f64 = transitions.values().sum();
        for prob in transitions.values_mut() {
            *prob /= total;
        }
    }

    /// Get the most likely path through all remaining gaps
    pub fn predict_optimal_path(&self, state: &WorkflowState) -> Vec<GapId> {
        let mut path = Vec::new();
        let mut current_state = state.clone();

        while path.len() < 10 {
            let predictions = self.predict_next_steps(&current_state, 1);
            if predictions.is_empty() {
                break;
            }

            let next = predictions[0].gap_id;
            path.push(next);
            current_state.fulfilled.push(next);
        }

        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markov_chain_creation() {
        let gaps = TrustChainGap::all_gaps();
        let chain = WorkflowMarkovChain::new(gaps);

        // Should have initial transitions
        assert!(chain.transitions.contains_key(&None));
    }

    #[test]
    fn test_predict_first_step() {
        let gaps = TrustChainGap::all_gaps();
        let chain = WorkflowMarkovChain::new(gaps);
        let state = WorkflowState::new();

        let predictions = chain.predict_next_steps(&state, 3);
        assert!(!predictions.is_empty());

        // First prediction should have highest probability
        assert!(predictions[0].probability >= predictions[1].probability);
    }

    #[test]
    fn test_predictions_respect_dependencies() {
        let gaps = TrustChainGap::all_gaps();
        let chain = WorkflowMarkovChain::new(gaps);
        let state = WorkflowState::new();

        let predictions = chain.predict_next_steps(&state, 10);

        // Trust chain reference should have lower probability than cert chain
        // because it depends on cert chain
        let cert_pred = predictions.iter().find(|p| p.gap_id == GapId::CERTIFICATE_CHAIN_VERIFICATION);
        let trust_pred = predictions.iter().find(|p| p.gap_id == GapId::TRUST_CHAIN_REFERENCE);

        if let (Some(cert), Some(trust)) = (cert_pred, trust_pred) {
            // Trust chain should have blockers since cert chain not done
            assert!(!trust.blockers.is_empty());
        }
    }

    #[test]
    fn test_predict_after_first_gap() {
        let gaps = TrustChainGap::all_gaps();
        let chain = WorkflowMarkovChain::new(gaps);
        let state = WorkflowState::with_fulfilled(vec![GapId::CERTIFICATE_CHAIN_VERIFICATION]);

        let predictions = chain.predict_next_steps(&state, 3);

        // Trust chain reference should now be more likely
        let trust_pred = predictions.iter().find(|p| p.gap_id == GapId::TRUST_CHAIN_REFERENCE);
        assert!(trust_pred.is_some());

        // No dependency blockers (certificate chain is fulfilled)
        // But there may be object implementation blockers
        let has_dependency_blockers = trust_pred.unwrap().blockers.iter()
            .any(|b| b.contains("to be completed first"));
        assert!(!has_dependency_blockers, "Should have no dependency blockers");
    }

    #[test]
    fn test_optimal_path() {
        let gaps = TrustChainGap::all_gaps();
        let chain = WorkflowMarkovChain::new(gaps);
        let state = WorkflowState::new();

        let path = chain.predict_optimal_path(&state);

        // Should produce a valid ordering
        assert!(!path.is_empty());

        // Certificate chain should come before trust chain reference
        let cert_pos = path.iter().position(|g| *g == GapId::CERTIFICATE_CHAIN_VERIFICATION);
        let trust_pos = path.iter().position(|g| *g == GapId::TRUST_CHAIN_REFERENCE);

        if let (Some(cp), Some(tp)) = (cert_pos, trust_pos) {
            assert!(cp < tp, "Cert chain should come before trust chain ref");
        }
    }
}
