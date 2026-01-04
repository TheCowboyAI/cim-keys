// Copyright (c) 2025 - Cowboy AI, LLC.

//! Progression Tracking for Trust Chain Gap Fulfillment
//!
//! This module tracks the progression of trust chain gaps through their
//! lifecycle stages, emitting events and maintaining state.
//!
//! ## Progression Flow
//!
//! ```text
//! NotStarted ──────────────────────────────────────────────────┐
//!     │                                                        │
//!     ▼                                                        │
//! InProgress ──(tests fail)────────────────────────────────────┤
//!     │                                                        │
//!     ▼                                                        │
//! Implemented ──(tests fail)───────────────────────────────────┤
//!     │                                                        │
//!     ▼                                                        │
//! Tested ──(verification fail)─────────────────────────────────┤
//!     │                                                        │
//!     ▼                                                        │
//! Verified ◄───────────────────────────────────────────────────┘
//!                    (regression detected)
//! ```

use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use super::gaps::{GapId, GapStatus, TrustChainGap};
use super::markov::WorkflowMarkovChain;

/// Event emitted when gap progression changes
#[derive(Debug, Clone)]
pub enum ProgressionEvent {
    /// Gap work has started
    GapStarted {
        gap_id: GapId,
        started_by: Option<String>,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    /// Gap status has advanced
    GapAdvanced {
        gap_id: GapId,
        from_status: GapStatus,
        to_status: GapStatus,
        reason: String,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    /// Gap status has regressed (e.g., test failure)
    GapRegressed {
        gap_id: GapId,
        from_status: GapStatus,
        to_status: GapStatus,
        reason: String,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    /// Required object has been fulfilled
    ObjectFulfilled {
        gap_id: GapId,
        object_name: String,
        fulfilled_by: Option<String>,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    /// Evidence has been added (test, BDD, etc.)
    EvidenceAdded {
        gap_id: GapId,
        evidence_type: EvidenceType,
        count: u32,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    /// Gap has been fully verified
    GapVerified {
        gap_id: GapId,
        verified_by: Option<String>,
        verification_method: String,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    /// Workflow prediction updated
    PredictionUpdated {
        recommended_next: GapId,
        probability: f64,
        reasoning: Vec<String>,
        timestamp: DateTime<Utc>,
    },
}

/// Type of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceType {
    UnitTest,
    PropertyTest,
    BddScenario,
    IntegrationTest,
    Documentation,
}

/// Current state of progression
#[derive(Debug, Clone)]
pub struct ProgressionState {
    /// Status of each gap
    pub gap_statuses: HashMap<GapId, GapStatus>,
    /// Currently active gap (being worked on)
    pub active_gap: Option<GapId>,
    /// History of status changes
    pub history: Vec<ProgressionEvent>,
    /// Overall completion percentage
    pub completion_percentage: f64,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl Default for ProgressionState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressionState {
    pub fn new() -> Self {
        let mut gap_statuses = HashMap::new();

        // Initialize all gaps as NotStarted
        for gap_id in GapId::all() {
            gap_statuses.insert(gap_id, GapStatus::NotStarted);
        }

        Self {
            gap_statuses,
            active_gap: None,
            history: Vec::new(),
            completion_percentage: 0.0,
            last_updated: Utc::now(),
        }
    }

    /// Load state from existing gaps
    pub fn from_gaps(gaps: &[TrustChainGap]) -> Self {
        let mut state = Self::new();

        for gap in gaps {
            state.gap_statuses.insert(gap.id, gap.status);
        }

        state.recalculate_completion();
        state
    }

    /// Get status of a specific gap
    pub fn get_status(&self, gap_id: GapId) -> GapStatus {
        self.gap_statuses.get(&gap_id).copied().unwrap_or(GapStatus::NotStarted)
    }

    /// Check if a gap is completed
    pub fn is_completed(&self, gap_id: GapId) -> bool {
        self.get_status(gap_id) == GapStatus::Verified
    }

    /// Get all completed gaps
    pub fn completed_gaps(&self) -> Vec<GapId> {
        self.gap_statuses
            .iter()
            .filter(|(_, status)| **status == GapStatus::Verified)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get all in-progress gaps
    pub fn in_progress_gaps(&self) -> Vec<GapId> {
        self.gap_statuses
            .iter()
            .filter(|(_, status)| {
                matches!(status,
                    GapStatus::InProgress |
                    GapStatus::Implemented |
                    GapStatus::Tested
                )
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get all not-started gaps
    pub fn not_started_gaps(&self) -> Vec<GapId> {
        self.gap_statuses
            .iter()
            .filter(|(_, status)| **status == GapStatus::NotStarted)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Recalculate overall completion percentage
    fn recalculate_completion(&mut self) {
        let total: f64 = self.gap_statuses.values()
            .map(|s| s.progress_percentage() as f64)
            .sum();

        let max_total = self.gap_statuses.len() as f64 * 100.0;

        self.completion_percentage = if max_total > 0.0 {
            total / max_total * 100.0
        } else {
            0.0
        };

        self.last_updated = Utc::now();
    }
}

/// Tracker for gap progression with event emission
pub struct ProgressionTracker {
    /// Current state
    state: ProgressionState,
    /// Markov chain for predictions
    markov_chain: WorkflowMarkovChain,
    /// Event listeners
    listeners: Vec<Box<dyn Fn(&ProgressionEvent) + Send + Sync>>,
    /// Current correlation ID for related events
    correlation_id: Uuid,
}

impl ProgressionTracker {
    /// Create a new tracker from gaps
    pub fn new(gaps: Vec<TrustChainGap>) -> Self {
        let state = ProgressionState::from_gaps(&gaps);
        let markov_chain = WorkflowMarkovChain::new(gaps);

        Self {
            state,
            markov_chain,
            listeners: Vec::new(),
            correlation_id: Uuid::now_v7(),
        }
    }

    /// Get current state
    pub fn state(&self) -> &ProgressionState {
        &self.state
    }

    /// Add an event listener
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: Fn(&ProgressionEvent) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    /// Emit an event to all listeners
    fn emit(&self, event: &ProgressionEvent) {
        for listener in &self.listeners {
            listener(event);
        }
    }

    /// Start working on a gap
    pub fn start_gap(&mut self, gap_id: GapId, started_by: Option<String>) -> Result<Vec<ProgressionEvent>, ProgressionError> {
        let current_status = self.state.get_status(gap_id);

        if current_status != GapStatus::NotStarted {
            return Err(ProgressionError::AlreadyStarted { gap_id });
        }

        let mut events = Vec::new();

        // Create started event
        let started_event = ProgressionEvent::GapStarted {
            gap_id,
            started_by,
            timestamp: Utc::now(),
            correlation_id: self.correlation_id,
        };
        events.push(started_event.clone());
        self.emit(&started_event);

        // Advance status
        let advanced_event = ProgressionEvent::GapAdvanced {
            gap_id,
            from_status: GapStatus::NotStarted,
            to_status: GapStatus::InProgress,
            reason: "Gap work initiated".to_string(),
            timestamp: Utc::now(),
            correlation_id: self.correlation_id,
            causation_id: Uuid::now_v7(),
        };
        events.push(advanced_event.clone());
        self.emit(&advanced_event);

        // Update state
        self.state.gap_statuses.insert(gap_id, GapStatus::InProgress);
        self.state.active_gap = Some(gap_id);
        self.state.history.extend(events.clone());
        self.state.recalculate_completion();

        // Update Markov chain
        self.markov_chain.observe_transition(
            self.state.completed_gaps().last().copied(),
            gap_id,
        );

        // Emit prediction update
        let prediction_event = self.create_prediction_event();
        events.push(prediction_event.clone());
        self.emit(&prediction_event);

        Ok(events)
    }

    /// Advance a gap to the next status
    pub fn advance_gap(&mut self, gap_id: GapId, reason: String) -> Result<Vec<ProgressionEvent>, ProgressionError> {
        let current_status = self.state.get_status(gap_id);

        if current_status == GapStatus::Verified {
            return Err(ProgressionError::AlreadyVerified { gap_id });
        }

        let next_status = current_status.next_status()
            .expect("Already checked for Verified status");

        let mut events = Vec::new();

        let advanced_event = ProgressionEvent::GapAdvanced {
            gap_id,
            from_status: current_status,
            to_status: next_status,
            reason,
            timestamp: Utc::now(),
            correlation_id: self.correlation_id,
            causation_id: Uuid::now_v7(),
        };
        events.push(advanced_event.clone());
        self.emit(&advanced_event);

        // Update state
        self.state.gap_statuses.insert(gap_id, next_status);
        self.state.history.push(advanced_event);
        self.state.recalculate_completion();

        // If verified, clear active gap and emit verification event
        if next_status == GapStatus::Verified {
            if self.state.active_gap == Some(gap_id) {
                self.state.active_gap = None;
            }

            let verified_event = ProgressionEvent::GapVerified {
                gap_id,
                verified_by: None,
                verification_method: "Manual verification".to_string(),
                timestamp: Utc::now(),
                correlation_id: self.correlation_id,
            };
            events.push(verified_event.clone());
            self.emit(&verified_event);
        }

        // Emit prediction update
        let prediction_event = self.create_prediction_event();
        events.push(prediction_event.clone());
        self.emit(&prediction_event);

        Ok(events)
    }

    /// Regress a gap (e.g., due to test failure)
    pub fn regress_gap(&mut self, gap_id: GapId, to_status: GapStatus, reason: String) -> Result<Vec<ProgressionEvent>, ProgressionError> {
        let current_status = self.state.get_status(gap_id);

        if to_status.progress_percentage() >= current_status.progress_percentage() {
            return Err(ProgressionError::InvalidRegression {
                gap_id,
                from: current_status,
                to: to_status,
            });
        }

        let mut events = Vec::new();

        let regressed_event = ProgressionEvent::GapRegressed {
            gap_id,
            from_status: current_status,
            to_status,
            reason,
            timestamp: Utc::now(),
            correlation_id: self.correlation_id,
            causation_id: Uuid::now_v7(),
        };
        events.push(regressed_event.clone());
        self.emit(&regressed_event);

        // Update state
        self.state.gap_statuses.insert(gap_id, to_status);
        self.state.history.push(regressed_event);
        self.state.recalculate_completion();

        Ok(events)
    }

    /// Record that an object has been fulfilled
    pub fn fulfill_object(&mut self, gap_id: GapId, object_name: String, fulfilled_by: Option<String>) -> ProgressionEvent {
        let event = ProgressionEvent::ObjectFulfilled {
            gap_id,
            object_name,
            fulfilled_by,
            timestamp: Utc::now(),
            correlation_id: self.correlation_id,
        };
        self.emit(&event);
        self.state.history.push(event.clone());
        event
    }

    /// Record that evidence has been added
    pub fn add_evidence(&mut self, gap_id: GapId, evidence_type: EvidenceType, count: u32) -> ProgressionEvent {
        let event = ProgressionEvent::EvidenceAdded {
            gap_id,
            evidence_type,
            count,
            timestamp: Utc::now(),
            correlation_id: self.correlation_id,
        };
        self.emit(&event);
        self.state.history.push(event.clone());
        event
    }

    /// Get the recommended next gap to work on
    pub fn recommend_next(&self) -> Option<super::markov::PredictedStep> {
        let workflow_state = super::markov::WorkflowState::with_fulfilled(
            self.state.completed_gaps()
        );

        self.markov_chain.predict_next_steps(&workflow_state, 1)
            .into_iter()
            .next()
    }

    /// Get top N recommendations
    pub fn recommend_next_n(&self, n: usize) -> Vec<super::markov::PredictedStep> {
        let workflow_state = super::markov::WorkflowState::with_fulfilled(
            self.state.completed_gaps()
        );

        self.markov_chain.predict_next_steps(&workflow_state, n)
    }

    /// Get the optimal path through remaining gaps
    pub fn optimal_path(&self) -> Vec<GapId> {
        let workflow_state = super::markov::WorkflowState::with_fulfilled(
            self.state.completed_gaps()
        );

        self.markov_chain.predict_optimal_path(&workflow_state)
    }

    /// Create a prediction update event
    fn create_prediction_event(&self) -> ProgressionEvent {
        if let Some(prediction) = self.recommend_next() {
            ProgressionEvent::PredictionUpdated {
                recommended_next: prediction.gap_id,
                probability: prediction.probability,
                reasoning: prediction.reasoning,
                timestamp: Utc::now(),
            }
        } else {
            ProgressionEvent::PredictionUpdated {
                recommended_next: GapId::CERTIFICATE_CHAIN_VERIFICATION,
                probability: 0.0,
                reasoning: vec!["No predictions available - all gaps may be complete".to_string()],
                timestamp: Utc::now(),
            }
        }
    }

    /// Get progression summary
    pub fn summary(&self) -> ProgressionSummary {
        ProgressionSummary {
            total_gaps: self.state.gap_statuses.len(),
            completed: self.state.completed_gaps().len(),
            in_progress: self.state.in_progress_gaps().len(),
            not_started: self.state.not_started_gaps().len(),
            completion_percentage: self.state.completion_percentage,
            active_gap: self.state.active_gap,
            event_count: self.state.history.len(),
        }
    }

    /// Start a new correlation context
    pub fn new_correlation(&mut self) {
        self.correlation_id = Uuid::now_v7();
    }
}

/// Summary of progression state
#[derive(Debug, Clone)]
pub struct ProgressionSummary {
    pub total_gaps: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub not_started: usize,
    pub completion_percentage: f64,
    pub active_gap: Option<GapId>,
    pub event_count: usize,
}

/// Errors that can occur during progression
#[derive(Debug, Clone)]
pub enum ProgressionError {
    AlreadyStarted { gap_id: GapId },
    AlreadyVerified { gap_id: GapId },
    InvalidRegression { gap_id: GapId, from: GapStatus, to: GapStatus },
    DependencyNotMet { gap_id: GapId, dependency: GapId },
}

impl std::fmt::Display for ProgressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyStarted { gap_id } =>
                write!(f, "Gap {} has already been started", gap_id),
            Self::AlreadyVerified { gap_id } =>
                write!(f, "Gap {} is already verified", gap_id),
            Self::InvalidRegression { gap_id, from, to } =>
                write!(f, "Invalid regression for {}: {:?} -> {:?}", gap_id, from, to),
            Self::DependencyNotMet { gap_id, dependency } =>
                write!(f, "Gap {} depends on {} which is not complete", gap_id, dependency),
        }
    }
}

impl std::error::Error for ProgressionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progression_state_creation() {
        let state = ProgressionState::new();
        assert_eq!(state.gap_statuses.len(), GapId::all().len());
        assert_eq!(state.completion_percentage, 0.0);
    }

    #[test]
    fn test_tracker_creation() {
        let gaps = TrustChainGap::all_gaps();
        let tracker = ProgressionTracker::new(gaps);

        let summary = tracker.summary();
        assert_eq!(summary.total_gaps, 10);
        // Certificate Chain Verification was completed in Sprint 41
        assert_eq!(summary.completed, 1);
    }

    #[test]
    fn test_start_gap() {
        let gaps = TrustChainGap::all_gaps();
        let mut tracker = ProgressionTracker::new(gaps);

        // Use TRUST_CHAIN_REFERENCE which is still NotStarted
        let events = tracker.start_gap(GapId::TRUST_CHAIN_REFERENCE, Some("Test".to_string()));
        assert!(events.is_ok());

        let summary = tracker.summary();
        assert_eq!(summary.in_progress, 1);
        assert_eq!(summary.active_gap, Some(GapId::TRUST_CHAIN_REFERENCE));
    }

    #[test]
    fn test_advance_gap() {
        let gaps = TrustChainGap::all_gaps();
        let mut tracker = ProgressionTracker::new(gaps);

        // Use TRUST_CHAIN_REFERENCE which is still NotStarted
        tracker.start_gap(GapId::TRUST_CHAIN_REFERENCE, None).unwrap();
        let events = tracker.advance_gap(
            GapId::TRUST_CHAIN_REFERENCE,
            "Implementation complete".to_string(),
        );

        assert!(events.is_ok());
        let status = tracker.state().get_status(GapId::TRUST_CHAIN_REFERENCE);
        assert_eq!(status, GapStatus::Implemented);
    }

    #[test]
    fn test_recommend_next() {
        let gaps = TrustChainGap::all_gaps();
        let tracker = ProgressionTracker::new(gaps);

        let recommendation = tracker.recommend_next();
        assert!(recommendation.is_some());
    }

    #[test]
    fn test_optimal_path() {
        let gaps = TrustChainGap::all_gaps();
        let tracker = ProgressionTracker::new(gaps);

        let path = tracker.optimal_path();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_event_listener() {
        let gaps = TrustChainGap::all_gaps();
        let mut tracker = ProgressionTracker::new(gaps);

        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let event_count = Arc::new(AtomicUsize::new(0));
        let event_count_clone = event_count.clone();

        tracker.add_listener(move |_event| {
            event_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Use TRUST_CHAIN_REFERENCE which is still NotStarted
        tracker.start_gap(GapId::TRUST_CHAIN_REFERENCE, None).unwrap();

        // Should have received GapStarted, GapAdvanced, and PredictionUpdated events
        assert!(event_count.load(Ordering::SeqCst) >= 3);
    }

    #[test]
    fn test_regress_gap() {
        let gaps = TrustChainGap::all_gaps();
        let mut tracker = ProgressionTracker::new(gaps);

        // Use TRUST_CHAIN_REFERENCE which is still NotStarted
        tracker.start_gap(GapId::TRUST_CHAIN_REFERENCE, None).unwrap();
        tracker.advance_gap(GapId::TRUST_CHAIN_REFERENCE, "Done".to_string()).unwrap();

        let result = tracker.regress_gap(
            GapId::TRUST_CHAIN_REFERENCE,
            GapStatus::InProgress,
            "Tests failed".to_string(),
        );

        assert!(result.is_ok());
        let status = tracker.state().get_status(GapId::TRUST_CHAIN_REFERENCE);
        assert_eq!(status, GapStatus::InProgress);
    }
}
