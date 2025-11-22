//! Manifest Aggregate Commands
//!
//! Commands for the Manifest aggregate root.
//! Currently re-exports from export.rs - full migration pending.

// Re-export manifest/export-related commands from export module
pub use super::export::{
    ExportToEncryptedStorage,
    ExportCompleted,
    handle_export_to_encrypted_storage,
    KeyExportItem,
    CertificateExportItem,
    NatsIdentityExportItem,
};

// TODO: Future refactoring
// - Add CreateManifest command
// - Add UpdateManifest command
// - Add ExportJWKS command
// - Add ApplyProjection command
// - Align with ManifestEvents from events/manifest.rs
