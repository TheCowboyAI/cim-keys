//! Key Aggregate Commands
//!
//! Commands for the Key aggregate root.
//! Currently re-exports from pki.rs - full migration pending.

// Re-export key-related commands from pki module
pub use super::pki::{
    GenerateKeyPair,
    KeyPairGenerated,
    handle_generate_key_pair,
};

// TODO: Future refactoring
// - Move key generation logic from pki.rs to this module
// - Add key import, export, rotation commands
// - Add SSH/GPG key generation commands
// - Align with KeyEvents from events/key.rs
