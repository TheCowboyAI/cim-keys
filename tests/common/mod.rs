//! Common test utilities and fixtures for cim-keys tests
//!
//! Note: This module has been simplified to support the updated aggregate-based event architecture.
//! Full test utilities will be restored after integration tests are stabilized.

// Re-export for backward compatibility

/// Temporary directory utilities
pub mod temp {
    use tempfile::TempDir;
    use std::path::PathBuf;

    /// Create a temporary test environment
    pub fn create_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Create expected directory structure
        std::fs::create_dir_all(&path.join("keys")).ok();
        std::fs::create_dir_all(&path.join("certificates")).ok();
        std::fs::create_dir_all(&path.join("events")).ok();
        std::fs::create_dir_all(&path.join("nats")).ok();

        (temp_dir, path)
    }
}
