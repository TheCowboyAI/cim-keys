// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Graph Module - Pure Domain Layer
//!
//! This module provides the domain layer for organizational graph representation.
//! It contains only domain concepts with no UI dependencies.
//!
//! ## DDD Layer Separation
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  UI Layer (src/gui/graph.rs)                                │
//! │  - OrganizationConcept (visualization state)                │
//! │  - NodeView, EdgeView (UI concerns)                         │
//! │  - User interactions, drag/drop, animations                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Projection Layer (src/gui/graph_projection.rs)             │
//! │  - GraphProjection (materialize from events)                │
//! │  - Caches, indexes for query performance                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Domain Layer (src/domain/graph/)                           │
//! │  - DomainRelation (pure domain edges)                       │
//! │  - RelationType (semantic relationship types)               │
//! │  - NO UI dependencies (Color, Point, etc.)                  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Principles
//!
//! - Domain layer has NO dependencies on iced or GUI types
//! - All UI concerns (color, position, size) are derived in UI layer
//! - Projection layer materializes current state from event history
//! - Domain types are serializable for persistence

mod relations;
mod events;

pub use relations::*;
pub use events::*;
