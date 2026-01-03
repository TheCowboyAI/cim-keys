// Copyright (c) 2025 - Cowboy AI, LLC.

//! Graph Module - Categorical Operations on Domain Graph
//!
//! This module provides the abstract graph layer for cim-keys, implementing
//! the Kan extension pattern where graph operations work on UUIDs and
//! relationships, lifting to domain semantics only at boundaries.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    GRAPH LAYER (Abstract)                       │
//! │  - Nodes are UUIDs + opaque payloads (type-erased)              │
//! │  - Edges are triples (source_id, relation, target_id)           │
//! │  - Graph algorithms work HERE (path, cycles, SCC, topo sort)    │
//! │  - Returns UUIDs, never concrete types                          │
//! │                                                                 │
//! │  Modules: abstract_ops (AbstractGraphOps)                       │
//! └─────────────────────────────────────────────────────────────────┘
//!                               │
//!                               │  ═══════════════════════════════
//!                               │  ║     LIFT BOUNDARY           ║
//!                               │  ║  (DeferredLift, BatchLift)  ║
//!                               │  ═══════════════════════════════
//!                               │
//!                               │  Modules: lift, morphism
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    DOMAIN LAYER (Concrete)                      │
//! │  - Person, Organization, Key, Certificate, etc.                 │
//! │  - Aggregates with state machines                               │
//! │  - Domain-specific operations (validate, sign, apply_policy)    │
//! │                                                                 │
//! │  Modules: visualization, detail_panel (morphism registries)     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Modules
//!
//! - [`abstract_ops`]: UUID-returning graph algorithms (reachable, path, SCC)
//! - [`lift`]: Explicit lift boundary utilities (DeferredLift, BatchLift)
//! - [`morphism`]: MorphismRegistry for categorical folds
//! - [`visualization`]: Themed visualization morphisms
//! - [`detail_panel`]: Detail panel data morphisms
//!
//! # Usage Pattern
//!
//! ```rust,ignore
//! use cim_keys::graph::{AbstractGraphOps, DeferredLift, VisualizationRegistry};
//!
//! // 1. Graph operations return UUIDs only
//! let ops = AbstractGraphOps::from_graph(&graph);
//! let reachable: HashSet<Uuid> = ops.reachable_from(start_id);
//!
//! // 2. At the boundary, lift to domain semantics
//! let deferred = graph.defer_lift(node_id);
//! let lifted = deferred.lift()?;
//!
//! // 3. Morphism registries handle domain-specific transformations
//! let registry = VisualizationRegistry::new(&theme);
//! let vis_data = registry.fold(&lifted)?;
//! ```

pub mod morphism;
pub mod visualization;
pub mod detail_panel;
pub mod abstract_ops;
pub mod lift;

// Re-export key types for convenience
pub use morphism::{CompleteMorphismRegistry, LazyMorphism, Morphism, MorphismRegistry};
pub use visualization::{VisualizationRegistry, VisualizationRegistryBuilder};
pub use detail_panel::DetailPanelRegistry;
pub use abstract_ops::{AbstractGraphOps, FilteredGraphOps};
pub use lift::{DeferredLift, BatchLift, LiftResult, LiftFromGraph, LiftBoundary};
