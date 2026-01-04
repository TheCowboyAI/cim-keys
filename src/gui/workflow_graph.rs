// Copyright (c) 2025 - Cowboy AI, LLC.

//! Workflow Graph Visualization
//!
//! Integrates workflow gaps with the graph visualization system using
//! semantic positioning from conceptual spaces.
//!
//! ## Architecture
//!
//! ```text
//! WorkflowGaps + Statuses
//!         │
//!         ▼
//! GapConceptualSpace (semantic positions)
//!         │
//!         ├──► LiftedGraph (via lift_workflow_graph)
//!         │
//!         ▼
//! OrganizationConcept (positioned by semantic coordinates)
//! ```

use std::collections::HashMap;
use iced::{Color, Point};
#[allow(unused_imports)]
use uuid::Uuid;

use crate::workflow::{
    TrustChainGap, GapId, GapStatus, GapCategory,
    GapConceptualSpace, SemanticDistanceCalculator,
    SphericalProjector,
};
use crate::lifting::{
    lift_workflow_graph, LiftedGraph,
    COLOR_WORKFLOW_GAP_NOT_STARTED, COLOR_WORKFLOW_GAP_IN_PROGRESS,
    COLOR_WORKFLOW_GAP_IMPLEMENTED, COLOR_WORKFLOW_GAP_TESTED, COLOR_WORKFLOW_GAP_VERIFIED,
};
use crate::gui::graph::{OrganizationConcept, ConceptEntity, EdgeType};

/// Configuration for workflow graph layout
#[derive(Debug, Clone)]
pub struct WorkflowGraphConfig {
    /// Scale factor for semantic positions → screen coordinates
    pub scale: f32,
    /// Center point for the graph
    pub center: Point,
    /// Whether to show semantic neighbor edges
    pub show_semantic_neighbors: bool,
    /// Distance threshold for semantic neighbors (0.0-1.0)
    pub semantic_neighbor_threshold: f64,
    /// Whether to show recommended transitions
    pub show_recommended_transitions: bool,
}

impl Default for WorkflowGraphConfig {
    fn default() -> Self {
        Self {
            scale: 200.0,
            center: Point::new(400.0, 300.0),
            show_semantic_neighbors: true,
            semantic_neighbor_threshold: 0.3,
            show_recommended_transitions: true,
        }
    }
}

/// Build an OrganizationConcept from workflow gaps with semantic positioning
pub fn build_workflow_graph(
    gaps: &[TrustChainGap],
    statuses: &HashMap<GapId, GapStatus>,
    config: &WorkflowGraphConfig,
) -> OrganizationConcept {
    // Create conceptual space with semantic positions
    let space = GapConceptualSpace::new(gaps, statuses);

    // Lift gaps to graph nodes
    let lifted = lift_workflow_graph(gaps, statuses);

    // Convert to OrganizationConcept with semantic positioning
    build_from_lifted_with_semantic_positions(
        &lifted,
        &space,
        statuses,
        config,
    )
}

/// Build OrganizationConcept from LiftedGraph using semantic positions for layout
fn build_from_lifted_with_semantic_positions(
    lifted: &LiftedGraph,
    space: &GapConceptualSpace,
    statuses: &HashMap<GapId, GapStatus>,
    config: &WorkflowGraphConfig,
) -> OrganizationConcept {
    let mut graph = OrganizationConcept::new();

    // Add nodes with semantic-based positions
    for node in lifted.nodes() {
        let id = node.id;
        let entity = ConceptEntity::from_lifted_node(node.clone());

        // Calculate position from semantic space
        let position = calculate_semantic_position(id, space, config);

        // Create view with calculated position
        let mut view = entity.create_view(position);

        // Override color based on current status (dynamic)
        if let Some(gap_id) = find_gap_id_for_uuid(id, space) {
            if let Some(status) = statuses.get(&gap_id) {
                view.color = status_to_color(*status);
            }
        }

        graph.nodes.insert(id, entity);
        graph.node_views.insert(id, view);
    }

    // Add dependency edges
    for edge in lifted.edges() {
        graph.add_edge(edge.from_id, edge.to_id, EdgeType::WorkflowDependency);
    }

    // Add semantic neighbor edges if enabled
    if config.show_semantic_neighbors {
        add_semantic_neighbor_edges(&mut graph, space, config);
    }

    // Add recommended transition edges if enabled
    if config.show_recommended_transitions {
        add_recommended_transition_edges(&mut graph, space, statuses);
    }

    // Build edge indices for efficient lookup
    graph.rebuild_adjacency_indices();

    graph
}

/// Calculate screen position from semantic space coordinates
fn calculate_semantic_position(
    uuid: Uuid,
    space: &GapConceptualSpace,
    config: &WorkflowGraphConfig,
) -> Point {
    // Find the gap ID for this UUID
    if let Some(gap_id) = find_gap_id_for_uuid(uuid, space) {
        if let Some(semantic_pos) = space.positions.get(&gap_id) {
            // Project 3D semantic position to 2D using stereographic projection
            let projected = SphericalProjector::project(semantic_pos);

            // Use x, y from projected 3D point for 2D layout
            Point::new(
                config.center.x + (projected.x as f32) * config.scale,
                config.center.y + (projected.y as f32) * config.scale,
            )
        } else {
            // Fallback: center position
            config.center
        }
    } else {
        // Fallback: center position
        config.center
    }
}

/// Find GapId for a given UUID by searching the space
fn find_gap_id_for_uuid(uuid: Uuid, space: &GapConceptualSpace) -> Option<GapId> {
    space.positions.keys()
        .find(|gap_id| gap_id.as_uuid() == uuid)
        .copied()
}

/// Add edges for semantic neighbors (gaps that are close in conceptual space)
fn add_semantic_neighbor_edges(
    graph: &mut OrganizationConcept,
    space: &GapConceptualSpace,
    config: &WorkflowGraphConfig,
) {
    let gap_ids: Vec<GapId> = space.positions.keys().copied().collect();

    for (i, gap_id_a) in gap_ids.iter().enumerate() {
        for gap_id_b in gap_ids.iter().skip(i + 1) {
            // Get positions and calculate distance directly
            if let (Some(pos_a), Some(pos_b)) = (
                space.positions.get(gap_id_a),
                space.positions.get(gap_id_b),
            ) {
                // Use similarity (inverse of distance)
                let similarity = pos_a.similarity_to(pos_b);
                let distance = 1.0 - similarity;

                // Add edge if within threshold (close enough semantically)
                if distance < config.semantic_neighbor_threshold {
                    graph.add_edge(
                        gap_id_a.as_uuid(),
                        gap_id_b.as_uuid(),
                        EdgeType::SemanticNeighbor,
                    );
                }
            }
        }
    }
}

/// Add edges for recommended transitions based on Markov chain predictions
fn add_recommended_transition_edges(
    graph: &mut OrganizationConcept,
    space: &GapConceptualSpace,
    statuses: &HashMap<GapId, GapStatus>,
) {
    let calculator = SemanticDistanceCalculator::new(space.clone());

    // Find gaps that are in progress or have dependencies completed
    let active_gaps: Vec<GapId> = space.positions.keys()
        .filter(|gap_id| {
            matches!(
                statuses.get(gap_id),
                Some(GapStatus::NotStarted) | Some(GapStatus::InProgress)
            )
        })
        .copied()
        .collect();

    // For each active gap, calculate transition probabilities to other gaps
    for from_gap in &active_gaps {
        let transitions = calculator.all_transition_weights(*from_gap);

        // Sort by weight and take top 3 recommended transitions
        let mut sorted_transitions: Vec<_> = transitions.into_iter().collect();
        sorted_transitions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (to_gap, probability) in sorted_transitions.into_iter().take(3) {
            if probability > 0.1 {
                graph.add_edge(
                    from_gap.as_uuid(),
                    to_gap.as_uuid(),
                    EdgeType::RecommendedTransition { probability },
                );
            }
        }
    }
}

/// Convert GapStatus to display color
fn status_to_color(status: GapStatus) -> Color {
    match status {
        GapStatus::NotStarted => COLOR_WORKFLOW_GAP_NOT_STARTED,
        GapStatus::InProgress => COLOR_WORKFLOW_GAP_IN_PROGRESS,
        GapStatus::Implemented => COLOR_WORKFLOW_GAP_IMPLEMENTED,
        GapStatus::Tested => COLOR_WORKFLOW_GAP_TESTED,
        GapStatus::Verified => COLOR_WORKFLOW_GAP_VERIFIED,
    }
}

/// Message type for workflow graph interactions
#[derive(Debug, Clone)]
pub enum WorkflowGraphMessage {
    /// A gap node was selected
    GapSelected(GapId),
    /// A gap node was double-clicked (open details)
    GapActivated(GapId),
    /// Status change requested
    StatusChangeRequested { gap_id: GapId, new_status: GapStatus },
    /// Navigate to gap source file
    NavigateToSource(GapId),
    /// Filter by category
    FilterByCategory(Option<GapCategory>),
    /// Reset view
    ResetView,
}

/// Group gaps by category for hierarchical layout alternative
pub fn group_gaps_by_category(gaps: &[TrustChainGap]) -> HashMap<GapCategory, Vec<&TrustChainGap>> {
    let mut groups: HashMap<GapCategory, Vec<&TrustChainGap>> = HashMap::new();

    for gap in gaps {
        groups.entry(gap.category).or_default().push(gap);
    }

    groups
}

/// Calculate layout positions for hierarchical category-based layout
pub fn hierarchical_category_layout(
    gaps: &[TrustChainGap],
    config: &WorkflowGraphConfig,
) -> HashMap<GapId, Point> {
    let groups = group_gaps_by_category(gaps);
    let mut positions = HashMap::new();

    // Layout categories in columns
    let categories = [
        GapCategory::YubiKey,
        GapCategory::Pki,
        GapCategory::Delegation,
        GapCategory::Domain,
        GapCategory::Policy,
    ];

    let col_width = config.scale * 2.0;
    let row_height = 80.0;

    for (col, category) in categories.iter().enumerate() {
        if let Some(category_gaps) = groups.get(category) {
            for (row, gap) in category_gaps.iter().enumerate() {
                let x = config.center.x - (categories.len() as f32 / 2.0) * col_width
                    + (col as f32) * col_width;
                let y = config.center.y - (category_gaps.len() as f32 / 2.0) * row_height
                    + (row as f32) * row_height;

                positions.insert(gap.id, Point::new(x, y));
            }
        }
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::gaps::TrustChainGap;

    fn get_test_gaps() -> Vec<TrustChainGap> {
        TrustChainGap::all_gaps()
    }

    #[test]
    fn test_build_workflow_graph() {
        let gaps = get_test_gaps();
        let statuses = HashMap::new();
        let config = WorkflowGraphConfig::default();

        let graph = build_workflow_graph(&gaps, &statuses, &config);

        // Should have one node per gap
        assert_eq!(graph.nodes.len(), gaps.len());
        assert_eq!(graph.node_views.len(), gaps.len());
    }

    #[test]
    fn test_semantic_positions() {
        let gaps = get_test_gaps();
        let statuses: HashMap<GapId, GapStatus> = HashMap::new();
        let space = GapConceptualSpace::new(&gaps, &statuses);
        let config = WorkflowGraphConfig::default();

        // Each gap should have a unique position
        let mut positions: Vec<Point> = Vec::new();
        for gap in &gaps {
            let pos = calculate_semantic_position(
                gap.id.as_uuid(),
                &space,
                &config,
            );
            positions.push(pos);
        }

        // Positions should be spread out (not all at center)
        let center_count = positions.iter()
            .filter(|p| (p.x - config.center.x).abs() < 1.0 && (p.y - config.center.y).abs() < 1.0)
            .count();

        // Most positions should NOT be at center
        assert!(center_count < positions.len() / 2);
    }

    #[test]
    fn test_hierarchical_layout() {
        let gaps = get_test_gaps();
        let config = WorkflowGraphConfig::default();

        let positions = hierarchical_category_layout(&gaps, &config);

        // Should have position for each gap
        assert_eq!(positions.len(), gaps.len());

        // All positions should be different
        let unique_positions: std::collections::HashSet<_> = positions.values()
            .map(|p| (p.x as i32, p.y as i32))
            .collect();
        assert_eq!(unique_positions.len(), positions.len());
    }

    #[test]
    fn test_status_colors() {
        assert_eq!(status_to_color(GapStatus::NotStarted), COLOR_WORKFLOW_GAP_NOT_STARTED);
        assert_eq!(status_to_color(GapStatus::InProgress), COLOR_WORKFLOW_GAP_IN_PROGRESS);
        assert_eq!(status_to_color(GapStatus::Implemented), COLOR_WORKFLOW_GAP_IMPLEMENTED);
        assert_eq!(status_to_color(GapStatus::Tested), COLOR_WORKFLOW_GAP_TESTED);
        assert_eq!(status_to_color(GapStatus::Verified), COLOR_WORKFLOW_GAP_VERIFIED);
    }

    #[test]
    fn test_group_by_category() {
        let gaps = get_test_gaps();
        let groups = group_gaps_by_category(&gaps);

        // Should have multiple categories
        assert!(groups.len() > 1);

        // Total gaps in groups should equal input
        let total: usize = groups.values().map(|v| v.len()).sum();
        assert_eq!(total, gaps.len());
    }

    #[test]
    fn test_semantic_neighbor_threshold() {
        let gaps = get_test_gaps();
        let statuses = HashMap::new();

        // With high threshold, should have many neighbor edges
        let high_threshold_config = WorkflowGraphConfig {
            semantic_neighbor_threshold: 0.9,
            ..Default::default()
        };
        let graph_high = build_workflow_graph(&gaps, &statuses, &high_threshold_config);

        // With low threshold, should have fewer neighbor edges
        let low_threshold_config = WorkflowGraphConfig {
            semantic_neighbor_threshold: 0.1,
            ..Default::default()
        };
        let graph_low = build_workflow_graph(&gaps, &statuses, &low_threshold_config);

        // Count semantic neighbor edges
        let count_neighbors = |graph: &OrganizationConcept| {
            graph.edges.iter()
                .filter(|e| matches!(e.edge_type, EdgeType::SemanticNeighbor))
                .count()
        };

        assert!(count_neighbors(&graph_high) >= count_neighbors(&graph_low));
    }
}
