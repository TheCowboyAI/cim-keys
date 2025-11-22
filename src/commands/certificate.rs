//! Certificate Aggregate Commands
//!
//! Commands for the Certificate aggregate root.
//! Currently re-exports from pki.rs - full migration pending.

// Re-export certificate-related commands from pki module
pub use super::pki::{
    GenerateCertificate,
    CertificateGenerated,
    handle_generate_certificate,
    GenerateRootCA,
    RootCAGenerated,
    handle_generate_root_ca,
};

// TODO: Future refactoring
// - Move certificate generation logic from pki.rs to this module
// - Separate root CA generation into its own command
// - Add certificate renewal, revocation commands
// - Align with CertificateEvents from events/certificate.rs
