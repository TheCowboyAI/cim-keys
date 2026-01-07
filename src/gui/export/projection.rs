// Copyright (c) 2025 - Cowboy AI, LLC.

//! Export Projection Bounded Context
//!
//! This module implements the Export domain with:
//! - Message enum for all export operations
//! - State struct for export-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **SD Card Export**: Air-gapped encrypted storage
//! 2. **Cypher Export**: Neo4j/graph database export
//! 3. **NSC Export**: NATS credentials export
//! 4. **Graph Export**: JSON graph data export/import
//! 5. **Projection Config**: Target configuration and status

use std::path::PathBuf;

use iced::Task;

use crate::gui::{ProjectionSection, ProjectionState, ProjectionTarget};

/// Export Message
///
/// Organized by sub-domain:
/// - Path/Password Configuration (2 messages)
/// - SD Card Export (2 messages)
/// - Cypher Export (2 messages)
/// - NSC Export (2 messages)
/// - Graph Export (3 messages)
/// - Projection Configuration (5 messages)
#[derive(Debug, Clone)]
pub enum ExportMessage {
    // === Path/Password Configuration ===
    /// Export path changed
    ExportPathChanged(String),
    /// Export password changed
    ExportPasswordChanged(String),

    // === SD Card Export ===
    /// Export domain to SD card
    ExportToSDCard,
    /// Domain export completed
    DomainExported(Result<String, String>),
    /// SD card export completed (path, files_written, bytes_written)
    SDCardExported(Result<(String, usize, usize), String>),

    // === Cypher Export ===
    /// Export to Cypher format
    ExportToCypher,
    /// Cypher export completed (file_path, query_count)
    CypherExported(Result<(String, usize), String>),

    // === NSC Export ===
    /// Export to NSC format
    ExportToNsc,
    /// NSC export completed
    NscExported(Result<String, String>),

    // === Graph Export ===
    /// Export graph to JSON
    ExportGraph,
    /// Graph export completed
    GraphExported(Result<String, String>),
    /// Graph import completed
    GraphImported(Result<Option<crate::gui::GraphExport>, String>),

    // === Projection Configuration ===
    /// Change projection section view
    ProjectionSectionChanged(ProjectionSection),
    /// Select a projection target
    ProjectionSelected(ProjectionTarget),
    /// Connect to a projection target
    ConnectProjection(ProjectionTarget),
    /// Disconnect from a projection target
    DisconnectProjection(ProjectionTarget),
    /// Sync with a projection target
    SyncProjection(ProjectionTarget),
}

/// Export State
///
/// Contains all state related to export operations.
#[derive(Debug, Clone)]
pub struct ExportState {
    // === Path Configuration ===
    /// Base export path
    pub export_path: PathBuf,
    /// Export password for encryption
    pub export_password: String,

    // === Projection Configuration ===
    /// Current projection section being viewed
    pub projection_section: ProjectionSection,
    /// All projection targets and their status
    pub projections: Vec<ProjectionState>,
    /// Currently selected projection target
    pub selected_projection: Option<ProjectionTarget>,

    // === Export Status ===
    /// Last SD card export path
    pub last_sdcard_export_path: Option<String>,
    /// Last Cypher export path
    pub last_cypher_export_path: Option<String>,
    /// Last NSC export path
    pub last_nsc_export_path: Option<String>,
    /// Last graph export path
    pub last_graph_export_path: Option<String>,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            export_path: PathBuf::new(),
            export_password: String::new(),
            projection_section: ProjectionSection::Overview,
            projections: Vec::new(),
            selected_projection: None,
            last_sdcard_export_path: None,
            last_cypher_export_path: None,
            last_nsc_export_path: None,
            last_graph_export_path: None,
        }
    }
}

impl ExportState {
    /// Create a new ExportState with sensible defaults
    pub fn new(export_path: PathBuf) -> Self {
        Self {
            export_path,
            export_password: String::new(),
            projection_section: ProjectionSection::Overview,
            projections: ProjectionTarget::all()
                .into_iter()
                .map(ProjectionState::new)
                .collect(),
            selected_projection: None,
            last_sdcard_export_path: None,
            last_cypher_export_path: None,
            last_nsc_export_path: None,
            last_graph_export_path: None,
        }
    }

    /// Check if export password is set
    pub fn has_password(&self) -> bool {
        !self.export_password.is_empty()
    }

    /// Check if export path is set
    pub fn has_export_path(&self) -> bool {
        !self.export_path.as_os_str().is_empty()
    }

    /// Check if ready for export (has path and password)
    pub fn is_ready_for_export(&self) -> bool {
        self.has_export_path() && self.has_password()
    }

    /// Get projection state for a target
    pub fn get_projection(&self, target: &ProjectionTarget) -> Option<&ProjectionState> {
        self.projections.iter().find(|p| &p.target == target)
    }

    /// Get mutable projection state for a target
    pub fn get_projection_mut(&mut self, target: &ProjectionTarget) -> Option<&mut ProjectionState> {
        self.projections.iter_mut().find(|p| &p.target == target)
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update export state based on message
///
/// This function handles export domain messages. Note that some messages
/// require access to the full application state or external services
/// and will be delegated back to the main update function.
pub fn update(state: &mut ExportState, message: ExportMessage) -> Task<Message> {
    use ExportMessage::*;

    match message {
        // === Path/Password Configuration ===
        ExportPathChanged(path) => {
            state.export_path = PathBuf::from(path);
            Task::none()
        }

        ExportPasswordChanged(password) => {
            state.export_password = password;
            Task::none()
        }

        // === Projection Configuration ===
        ProjectionSectionChanged(section) => {
            state.projection_section = section;
            Task::none()
        }

        ProjectionSelected(target) => {
            state.selected_projection = Some(target);
            Task::none()
        }

        // === Export Operations (delegated to main) ===
        ExportToSDCard => {
            // Requires access to projection and domain data - delegated to main
            Task::none()
        }

        DomainExported(result) => {
            if let Ok(path) = &result {
                state.last_sdcard_export_path = Some(path.clone());
            }
            Task::none()
        }

        SDCardExported(result) => {
            if let Ok((path, _, _)) = &result {
                state.last_sdcard_export_path = Some(path.clone());
            }
            Task::none()
        }

        ExportToCypher => {
            // Requires access to org_graph - delegated to main
            Task::none()
        }

        CypherExported(result) => {
            if let Ok((path, _)) = &result {
                state.last_cypher_export_path = Some(path.clone());
            }
            Task::none()
        }

        ExportToNsc => {
            // Requires access to org_graph and nats data - delegated to main
            Task::none()
        }

        NscExported(result) => {
            if let Ok(path) = &result {
                state.last_nsc_export_path = Some(path.clone());
            }
            Task::none()
        }

        ExportGraph => {
            // Requires access to org_graph - delegated to main
            Task::none()
        }

        GraphExported(result) => {
            if let Ok(path) = &result {
                state.last_graph_export_path = Some(path.clone());
            }
            Task::none()
        }

        GraphImported(_result) => {
            // Graph import updates org_graph in main - delegated
            Task::none()
        }

        ConnectProjection(_target) => {
            // Connection logic is in main app - delegated
            Task::none()
        }

        DisconnectProjection(_target) => {
            // Disconnection logic is in main app - delegated
            Task::none()
        }

        SyncProjection(_target) => {
            // Sync logic is in main app - delegated
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_state_default() {
        let state = ExportState::default();
        assert!(state.export_path.as_os_str().is_empty());
        assert!(state.export_password.is_empty());
        assert!(state.projections.is_empty());
        assert!(state.selected_projection.is_none());
    }

    #[test]
    fn test_export_state_new() {
        let state = ExportState::new(PathBuf::from("/tmp/export"));
        assert_eq!(state.export_path, PathBuf::from("/tmp/export"));
        assert!(state.export_password.is_empty());
        assert!(!state.projections.is_empty()); // Has default projections
    }

    #[test]
    fn test_has_password() {
        let mut state = ExportState::default();
        assert!(!state.has_password());

        state.export_password = "secret".to_string();
        assert!(state.has_password());
    }

    #[test]
    fn test_has_export_path() {
        let mut state = ExportState::default();
        assert!(!state.has_export_path());

        state.export_path = PathBuf::from("/tmp/export");
        assert!(state.has_export_path());
    }

    #[test]
    fn test_is_ready_for_export() {
        let mut state = ExportState::default();
        assert!(!state.is_ready_for_export());

        state.export_path = PathBuf::from("/tmp/export");
        assert!(!state.is_ready_for_export());

        state.export_password = "secret".to_string();
        assert!(state.is_ready_for_export());
    }

    #[test]
    fn test_path_changed() {
        let mut state = ExportState::default();

        let _ = update(&mut state, ExportMessage::ExportPathChanged("/new/path".to_string()));
        assert_eq!(state.export_path, PathBuf::from("/new/path"));
    }

    #[test]
    fn test_password_changed() {
        let mut state = ExportState::default();

        let _ = update(&mut state, ExportMessage::ExportPasswordChanged("secret".to_string()));
        assert_eq!(state.export_password, "secret");
    }

    #[test]
    fn test_projection_section_changed() {
        let mut state = ExportState::default();

        let _ = update(&mut state, ExportMessage::ProjectionSectionChanged(ProjectionSection::Outgoing));
        assert!(matches!(state.projection_section, ProjectionSection::Outgoing));
    }

    #[test]
    fn test_projection_selected() {
        let mut state = ExportState::default();

        let _ = update(&mut state, ExportMessage::ProjectionSelected(ProjectionTarget::SDCard));
        assert_eq!(state.selected_projection, Some(ProjectionTarget::SDCard));
    }

    #[test]
    fn test_export_result_tracking() {
        let mut state = ExportState::default();

        // Test SD card export result
        let _ = update(
            &mut state,
            ExportMessage::SDCardExported(Ok(("/mnt/sdcard".to_string(), 10, 1024))),
        );
        assert_eq!(state.last_sdcard_export_path, Some("/mnt/sdcard".to_string()));

        // Test Cypher export result
        let _ = update(
            &mut state,
            ExportMessage::CypherExported(Ok(("/tmp/export.cypher".to_string(), 50))),
        );
        assert_eq!(state.last_cypher_export_path, Some("/tmp/export.cypher".to_string()));

        // Test NSC export result
        let _ = update(
            &mut state,
            ExportMessage::NscExported(Ok("/tmp/nsc".to_string())),
        );
        assert_eq!(state.last_nsc_export_path, Some("/tmp/nsc".to_string()));

        // Test graph export result
        let _ = update(
            &mut state,
            ExportMessage::GraphExported(Ok("/tmp/graph.json".to_string())),
        );
        assert_eq!(state.last_graph_export_path, Some("/tmp/graph.json".to_string()));
    }
}
