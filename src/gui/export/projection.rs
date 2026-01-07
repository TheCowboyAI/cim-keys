// Copyright (c) 2025 - Cowboy AI, LLC.

//! Export Message Definitions
//!
//! This module defines the message types for the Export bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **SD Card Export**: Air-gapped encrypted storage
//! 2. **Cypher Export**: Neo4j/graph database export
//! 3. **NSC Export**: NATS credentials export
//! 4. **Graph Export**: JSON graph data export/import
//! 5. **Projection Config**: Target configuration and status

use crate::gui::{ProjectionSection, ProjectionTarget};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_message_variants() {
        let _ = ExportMessage::ExportPathChanged("/tmp".to_string());
        let _ = ExportMessage::ExportPasswordChanged("secret".to_string());
        let _ = ExportMessage::ExportToSDCard;
        let _ = ExportMessage::DomainExported(Ok("/path".to_string()));
        let _ = ExportMessage::SDCardExported(Ok(("/path".to_string(), 10, 1024)));
        let _ = ExportMessage::ExportToCypher;
        let _ = ExportMessage::CypherExported(Ok(("/path".to_string(), 50)));
        let _ = ExportMessage::ExportToNsc;
        let _ = ExportMessage::NscExported(Ok("/path".to_string()));
        let _ = ExportMessage::ExportGraph;
        let _ = ExportMessage::GraphExported(Ok("/path".to_string()));
        let _ = ExportMessage::GraphImported(Ok(None));
        let _ = ExportMessage::ProjectionSectionChanged(ProjectionSection::Overview);
        let _ = ExportMessage::ProjectionSelected(ProjectionTarget::SDCard);
        let _ = ExportMessage::ConnectProjection(ProjectionTarget::SDCard);
        let _ = ExportMessage::DisconnectProjection(ProjectionTarget::SDCard);
        let _ = ExportMessage::SyncProjection(ProjectionTarget::SDCard);
    }
}
