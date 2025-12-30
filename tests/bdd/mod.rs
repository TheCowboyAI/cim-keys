// Copyright (c) 2025 - Cowboy AI, LLC.
//! BDD-style test infrastructure for cim-keys
//!
//! This module provides step definition implementations that correspond
//! to the Gherkin scenarios in doc/qa/features/*.feature
//!
//! The tests are organized to mirror the feature files:
//! - domain_bootstrap_steps: domain_bootstrap.feature
//! - person_management_steps: person_management.feature
//! - key_generation_steps: key_generation.feature
//! - yubikey_provisioning_steps: yubikey_provisioning.feature
//! - nats_security_steps: nats_security_bootstrap.feature
//! - export_manifest_steps: export_manifest.feature

pub mod domain_bootstrap_steps;
pub mod person_management_steps;
pub mod key_generation_steps;
pub mod export_manifest_steps;

/// Test context shared across step definitions
pub mod context {
    use cim_keys::{
        aggregate::KeyManagementAggregate,
        projections::OfflineKeyProjection,
        events::DomainEvent,
    };
    use tempfile::TempDir;
    use uuid::Uuid;
    use std::collections::HashMap;

    /// BDD test context that maintains state across Given/When/Then steps
    #[derive(Default)]
    pub struct TestContext {
        /// Temporary directory for test artifacts
        pub temp_dir: Option<TempDir>,
        /// The aggregate under test
        pub aggregate: Option<KeyManagementAggregate>,
        /// The projection for state verification
        pub projection: Option<OfflineKeyProjection>,
        /// Events captured during test execution
        pub captured_events: Vec<DomainEvent>,
        /// Named entities for reference in steps
        pub organizations: HashMap<String, Uuid>,
        pub people: HashMap<String, Uuid>,
        pub units: HashMap<String, Uuid>,
        pub keys: HashMap<String, Uuid>,
        /// Last error encountered (for error verification)
        pub last_error: Option<String>,
        /// Correlation ID for current scenario
        pub correlation_id: Uuid,
    }

    impl TestContext {
        pub fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let output_path = temp_dir.path().to_path_buf();

            let aggregate = KeyManagementAggregate::new(Uuid::now_v7());
            let projection = OfflineKeyProjection::new(output_path)
                .expect("Failed to create projection");

            Self {
                temp_dir: Some(temp_dir),
                aggregate: Some(aggregate),
                projection: Some(projection),
                captured_events: Vec::new(),
                organizations: HashMap::new(),
                people: HashMap::new(),
                units: HashMap::new(),
                keys: HashMap::new(),
                last_error: None,
                correlation_id: Uuid::now_v7(),
            }
        }

        pub fn aggregate(&self) -> &KeyManagementAggregate {
            self.aggregate.as_ref().expect("Aggregate not initialized")
        }

        pub fn projection(&self) -> &OfflineKeyProjection {
            self.projection.as_ref().expect("Projection not initialized")
        }

        pub fn projection_mut(&mut self) -> &mut OfflineKeyProjection {
            self.projection.as_mut().expect("Projection not initialized")
        }

        pub fn capture_events(&mut self, events: Vec<DomainEvent>) {
            self.captured_events.extend(events);
        }

        pub fn has_event_of_type(&self, event_type: &str) -> bool {
            self.captured_events.iter().any(|e| {
                format!("{:?}", e).contains(event_type)
            })
        }

        pub fn event_count(&self) -> usize {
            self.captured_events.len()
        }
    }
}

/// BDD assertion macros for readable test assertions
#[macro_export]
macro_rules! then_assert {
    ($condition:expr, $message:expr) => {
        assert!($condition, "THEN assertion failed: {}", $message);
    };
}

#[macro_export]
macro_rules! given {
    ($description:expr, $setup:expr) => {{
        eprintln!("  Given {}", $description);
        $setup
    }};
}

#[macro_export]
macro_rules! when {
    ($description:expr, $action:expr) => {{
        eprintln!("  When {}", $description);
        $action
    }};
}

#[macro_export]
macro_rules! then {
    ($description:expr, $assertion:expr) => {{
        eprintln!("  Then {}", $description);
        $assertion
    }};
}
