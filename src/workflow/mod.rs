// Copyright (c) 2025 - Cowboy AI, LLC.

//! Workflow Guidance System
//!
//! This module implements predictive workflow guidance based on:
//! - Domain Concepts (entities, aggregates, value objects)
//! - Current State (what has been fulfilled)
//! - Markov Chain transitions (most likely next step)
//!
//! ## Architecture
//!
//! The workflow system uses a Markov chain to model the probability of
//! transitioning from one gap state to another, enabling intelligent
//! suggestions for the next most productive action.
//!
//! ```text
//!                    ┌─────────────────┐
//!                    │ Current State   │
//!                    │ (Fulfilled Gaps)│
//!                    └────────┬────────┘
//!                             │
//!                    ┌────────▼────────┐
//!                    │ Markov Chain    │
//!                    │ Transition      │
//!                    │ Probabilities   │
//!                    └────────┬────────┘
//!                             │
//!         ┌───────────────────┼───────────────────┐
//!         │                   │                   │
//!    ┌────▼────┐        ┌────▼────┐        ┌────▼────┐
//!    │ Gap A   │        │ Gap B   │        │ Gap C   │
//!    │ P=0.65  │        │ P=0.25  │        │ P=0.10  │
//!    └─────────┘        └─────────┘        └─────────┘
//! ```

pub mod gaps;
pub mod markov;
pub mod navigation;
pub mod progression;
pub mod semantic;

pub use gaps::{TrustChainGap, GapId, GapStatus, GapCategory, RequiredObject};
pub use markov::{WorkflowMarkovChain, TransitionProbability, PredictedStep};
pub use navigation::{ObjectNavigator, NavigationTarget, NavigationPath};
pub use progression::{ProgressionTracker, ProgressionState, ProgressionEvent};
pub use semantic::{SemanticPosition, GapConceptualSpace, SemanticDistanceCalculator, SphericalProjector};
