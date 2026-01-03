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
//! └─────────────────────────────────────────────────────────────────┘
//!                               │
//!                               │  Kan Extension (Lan_K F)
//!                               │  lift() only when needed
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    DOMAIN LAYER (Concrete)                      │
//! │  - Person, Organization, Key, Certificate, etc.                 │
//! │  - Aggregates with state machines                               │
//! │  - Domain-specific operations (validate, sign, apply_policy)    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Modules
//!
//! - [`morphism`]: MorphismRegistry for categorical folds (replaces 29-arm FoldDomainNode)
//!
//! # Future Modules (Sprint 32-33)
//!
//! - `abstract_ops`: UUID-returning graph operations (reachable_from, shortest_path, etc.)
//! - `lift`: Explicit lift boundary utilities

pub mod morphism;
pub mod visualization;

// Re-export key types for convenience
pub use morphism::{CompleteMorphismRegistry, LazyMorphism, Morphism, MorphismRegistry};
pub use visualization::{VisualizationRegistry, VisualizationRegistryBuilder};
