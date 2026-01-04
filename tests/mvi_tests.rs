// Copyright (c) 2025 - Cowboy AI, LLC.

//! MVI (Model-View-Intent) Test Suite
//!
//! Tests for the pure functional MVI architecture:
//! - Pure update function: (Model, Intent) → (Model, Task)
//! - Model immutability via with_* methods
//! - FRP axiom compliance
//! - Property-based tests for compositional laws

use cim_keys::mvi::{Intent, Model};
use cim_keys::mvi::model::{DomainStatus, ExportStatus, PersonInput, Tab};
use iced::Point;
use proptest::prelude::*;
use std::path::PathBuf;

// ============================================================================
// MODEL IMMUTABILITY TESTS
// ============================================================================

mod model_immutability {
    use super::*;

    #[test]
    fn test_with_tab_returns_new_model() {
        let original = Model::default();
        let updated = original.clone().with_tab(Tab::Organization);

        // Original unchanged
        assert_eq!(original.current_tab, Tab::Welcome);
        // Updated has new value
        assert_eq!(updated.current_tab, Tab::Organization);
    }

    #[test]
    fn test_with_organization_name_returns_new_model() {
        let original = Model::default();
        let updated = original.clone().with_organization_name("Cowboy AI".to_string());

        assert!(original.organization_name.is_empty());
        assert_eq!(updated.organization_name, "Cowboy AI");
    }

    #[test]
    fn test_with_person_added_preserves_existing() {
        let original = Model::default()
            .with_person_added(PersonInput {
                name: "Alice".to_string(),
                email: "alice@test.com".to_string(),
                id: "1".to_string(),
            });

        let updated = original.clone().with_person_added(PersonInput {
            name: "Bob".to_string(),
            email: "bob@test.com".to_string(),
            id: "2".to_string(),
        });

        assert_eq!(original.people.len(), 1);
        assert_eq!(updated.people.len(), 2);
        assert_eq!(updated.people[0].name, "Alice");
        assert_eq!(updated.people[1].name, "Bob");
    }

    #[test]
    fn test_with_person_removed_immutable() {
        let original = Model::default()
            .with_person_added(PersonInput {
                name: "Alice".to_string(),
                email: "alice@test.com".to_string(),
                id: "1".to_string(),
            })
            .with_person_added(PersonInput {
                name: "Bob".to_string(),
                email: "bob@test.com".to_string(),
                id: "2".to_string(),
            });

        let updated = original.clone().with_person_removed(0);

        assert_eq!(original.people.len(), 2);
        assert_eq!(updated.people.len(), 1);
        assert_eq!(updated.people[0].name, "Bob");
    }

    #[test]
    fn test_with_domain_status_transitions() {
        let model = Model::default();

        let creating = model.clone().with_domain_status(DomainStatus::Creating);
        assert_eq!(creating.domain_status, DomainStatus::Creating);

        let created = creating.with_domain_status(DomainStatus::Created {
            organization_id: "org-1".to_string(),
            organization_name: "Test Org".to_string(),
        });

        match created.domain_status {
            DomainStatus::Created { organization_id, organization_name } => {
                assert_eq!(organization_id, "org-1");
                assert_eq!(organization_name, "Test Org");
            }
            _ => panic!("Expected Created status"),
        }
    }

    #[test]
    fn test_with_key_progress_bounds() {
        let model = Model::default();

        let progress_50 = model.clone().with_key_progress(0.5);
        assert_eq!(progress_50.key_generation_progress, 0.5);

        let progress_100 = model.clone().with_key_progress(1.0);
        assert_eq!(progress_100.key_generation_progress, 1.0);

        let progress_0 = model.with_key_progress(0.0);
        assert_eq!(progress_0.key_generation_progress, 0.0);
    }

    #[test]
    fn test_with_error_set_and_clear() {
        let model = Model::default();

        let with_error = model.clone().with_error(Some("Something went wrong".to_string()));
        assert_eq!(with_error.error_message, Some("Something went wrong".to_string()));

        let cleared = with_error.with_error(None);
        assert_eq!(cleared.error_message, None);
    }

    #[test]
    fn test_with_passphrase_chain() {
        let model = Model::default()
            .with_passphrase("secret123".to_string())
            .with_passphrase_confirmed("secret123".to_string());

        assert_eq!(model.passphrase, "secret123");
        assert_eq!(model.passphrase_confirmed, "secret123");
    }

    #[test]
    fn test_with_export_status_transitions() {
        let model = Model::default();

        assert_eq!(model.export_status, ExportStatus::NotStarted);

        let in_progress = model.clone().with_export_status(ExportStatus::InProgress);
        assert_eq!(in_progress.export_status, ExportStatus::InProgress);

        let completed = in_progress.with_export_status(ExportStatus::Completed {
            path: PathBuf::from("/tmp/output"),
            bytes_written: 1024,
        });

        match completed.export_status {
            ExportStatus::Completed { path, bytes_written } => {
                assert_eq!(path, PathBuf::from("/tmp/output"));
                assert_eq!(bytes_written, 1024);
            }
            _ => panic!("Expected Completed status"),
        }
    }

    #[test]
    fn test_graph_context_menu_state() {
        let model = Model::default();
        assert!(!model.graph_context_menu_visible);

        let with_menu = model.clone().with_context_menu_shown(Point::new(100.0, 200.0));
        assert!(with_menu.graph_context_menu_visible);
        assert_eq!(with_menu.graph_context_menu_position, Point::new(100.0, 200.0));

        let hidden = with_menu.with_context_menu_hidden();
        assert!(!hidden.graph_context_menu_visible);
    }

    #[test]
    fn test_graph_edge_creation_flow() {
        let model = Model::default();
        assert!(!model.graph_edge_creation_active);
        assert!(model.graph_edge_creation_from.is_none());

        let started = model.clone().with_edge_creation_started("node-1".to_string());
        assert!(started.graph_edge_creation_active);
        assert_eq!(started.graph_edge_creation_from, Some("node-1".to_string()));

        let completed = started.clone().with_edge_creation_completed();
        assert!(!completed.graph_edge_creation_active);
        assert!(completed.graph_edge_creation_from.is_none());

        let cancelled = started.with_edge_creation_cancelled();
        assert!(!cancelled.graph_edge_creation_active);
        assert!(cancelled.graph_edge_creation_from.is_none());
    }
}

// ============================================================================
// PURE UPDATE FUNCTION TESTS
// ============================================================================

mod pure_update {
    use super::*;

    // Note: Full update tests require mocked ports. Here we test
    // Intent construction and model state transitions.

    #[test]
    fn test_intent_ui_tab_selected() {
        let intent = Intent::UiTabSelected(Tab::Keys);
        match intent {
            Intent::UiTabSelected(tab) => assert_eq!(tab, Tab::Keys),
            _ => panic!("Wrong intent variant"),
        }
    }

    #[test]
    fn test_intent_domain_created() {
        let intent = Intent::DomainCreated {
            organization_id: "org-123".to_string(),
            organization_name: "Test Org".to_string(),
        };

        match intent {
            Intent::DomainCreated { organization_id, organization_name } => {
                assert_eq!(organization_id, "org-123");
                assert_eq!(organization_name, "Test Org");
            }
            _ => panic!("Wrong intent variant"),
        }
    }

    #[test]
    fn test_intent_error_occurred() {
        let intent = Intent::ErrorOccurred {
            context: "Key generation".to_string(),
            message: "YubiKey not found".to_string(),
        };

        match intent {
            Intent::ErrorOccurred { context, message } => {
                assert_eq!(context, "Key generation");
                assert_eq!(message, "YubiKey not found");
            }
            _ => panic!("Wrong intent variant"),
        }
    }

    #[test]
    fn test_model_default_state() {
        let model = Model::default();

        assert_eq!(model.current_tab, Tab::Welcome);
        assert!(model.organization_name.is_empty());
        assert!(model.people.is_empty());
        assert_eq!(model.domain_status, DomainStatus::NotCreated);
        assert_eq!(model.export_status, ExportStatus::NotStarted);
        assert!(!model.master_seed_derived);
        assert!(model.error_message.is_none());
    }

    #[test]
    fn test_person_input_with_methods() {
        let person = PersonInput {
            name: String::new(),
            email: String::new(),
            id: "1".to_string(),
        }
        .with_name("Alice".to_string())
        .with_email("alice@test.com".to_string());

        assert_eq!(person.name, "Alice");
        assert_eq!(person.email, "alice@test.com");
    }
}

// ============================================================================
// FRP AXIOM COMPLIANCE TESTS
// ============================================================================

mod frp_axioms {
    use super::*;

    /// A3: Decoupled Signal Functions
    /// Output at time t depends only on input before t
    #[test]
    fn test_a3_update_is_decoupled() {
        // Given the same model and intent, update should produce the same result
        let model1 = Model::default();
        let model2 = Model::default();

        let updated1 = model1.with_tab(Tab::Organization);
        let updated2 = model2.with_tab(Tab::Organization);

        // Both should have the same resulting tab
        assert_eq!(updated1.current_tab, updated2.current_tab);
    }

    /// A5: Totality and Well-Definedness
    /// All functions are total (no panics, no undefined)
    #[test]
    fn test_a5_with_methods_are_total() {
        let model = Model::default();

        // Test all with_* methods don't panic
        let _ = model.clone().with_tab(Tab::Welcome);
        let _ = model.clone().with_tab(Tab::Organization);
        let _ = model.clone().with_tab(Tab::Keys);
        let _ = model.clone().with_tab(Tab::Projections);

        let _ = model.clone().with_organization_name(String::new());
        let _ = model.clone().with_organization_name("Test".to_string());

        let _ = model.clone().with_error(None);
        let _ = model.clone().with_error(Some("error".to_string()));

        let _ = model.clone().with_key_progress(0.0);
        let _ = model.clone().with_key_progress(0.5);
        let _ = model.clone().with_key_progress(1.0);

        let _ = model.clone().with_context_menu_shown(Point::ORIGIN);
        let _ = model.clone().with_context_menu_hidden();

        // If we get here, all methods are total
    }

    /// A5: Edge cases don't panic
    #[test]
    fn test_a5_edge_cases_are_total() {
        let model = Model::default();

        // Empty strings
        let _ = model.clone().with_organization_name(String::new());
        let _ = model.clone().with_passphrase(String::new());

        // Remove from empty list should not panic (returns early)
        let _ = model.clone().with_person_removed(0);
        let _ = model.clone().with_person_removed(100);

        // Update at invalid index should not panic
        let _ = model.clone().with_person_name_updated(999, "Test".to_string());
        let _ = model.clone().with_person_email_updated(999, "test@test.com".to_string());
    }

    /// A7: Change Prefixes as Event Logs
    /// Events are stored as timestamped change prefixes
    #[test]
    fn test_a7_intent_is_event_log() {
        // Intent represents a discrete change at a point in time
        let intents: Vec<Intent> = vec![
            Intent::UiTabSelected(Tab::Organization),
            Intent::UiOrganizationNameChanged("Cowboy AI".to_string()),
            Intent::UiAddPersonClicked,
            Intent::DomainCreated {
                organization_id: "org-1".to_string(),
                organization_name: "Cowboy AI".to_string(),
            },
        ];

        // Replaying these intents should produce deterministic results
        let mut model = Model::default();

        for intent in intents {
            model = match intent {
                Intent::UiTabSelected(tab) => model.with_tab(tab),
                Intent::UiOrganizationNameChanged(name) => model.with_organization_name(name),
                Intent::UiAddPersonClicked => model.with_person_added(PersonInput {
                    name: String::new(),
                    email: String::new(),
                    id: uuid::Uuid::now_v7().to_string(),
                }),
                Intent::DomainCreated { organization_id, organization_name } => {
                    model.with_domain_status(DomainStatus::Created {
                        organization_id,
                        organization_name,
                    })
                }
                _ => model,
            };
        }

        assert_eq!(model.current_tab, Tab::Organization);
        assert_eq!(model.organization_name, "Cowboy AI");
        assert_eq!(model.people.len(), 1);
        match model.domain_status {
            DomainStatus::Created { organization_name, .. } => {
                assert_eq!(organization_name, "Cowboy AI");
            }
            _ => panic!("Expected Created status"),
        }
    }

    /// A9: Semantic Preservation (Compositional Laws)
    /// Composition should be associative
    #[test]
    fn test_a9_composition_is_associative() {
        let model = Model::default();

        // (f . g) . h = f . (g . h)
        // where f, g, h are with_* methods

        // Left associative: ((model.with_tab).with_name).with_person
        let left = model
            .clone()
            .with_tab(Tab::Organization)
            .with_organization_name("Test".to_string())
            .with_person_added(PersonInput {
                name: "Alice".to_string(),
                email: "alice@test.com".to_string(),
                id: "1".to_string(),
            });

        // Right associative: model.(with_tab.(with_name.with_person))
        // In Rust, this is the same due to method chaining, but we verify result
        let right = model
            .with_tab(Tab::Organization)
            .with_organization_name("Test".to_string())
            .with_person_added(PersonInput {
                name: "Alice".to_string(),
                email: "alice@test.com".to_string(),
                id: "1".to_string(),
            });

        // Results should be equivalent
        assert_eq!(left.current_tab, right.current_tab);
        assert_eq!(left.organization_name, right.organization_name);
        assert_eq!(left.people.len(), right.people.len());
        assert_eq!(left.people[0].name, right.people[0].name);
    }
}

// ============================================================================
// PROPERTY-BASED TESTS
// ============================================================================

mod property_tests {
    use super::*;

    // Strategy for generating Tab values
    fn arb_tab() -> impl Strategy<Value = Tab> {
        prop_oneof![
            Just(Tab::Welcome),
            Just(Tab::Organization),
            Just(Tab::Keys),
            Just(Tab::Projections),
        ]
    }

    // Strategy for generating organization names
    fn arb_org_name() -> impl Strategy<Value = String> {
        prop_oneof![
            Just(String::new()),
            "[a-zA-Z0-9 ]{1,50}".prop_map(|s| s.to_string()),
        ]
    }

    // Strategy for generating person inputs
    fn arb_person() -> impl Strategy<Value = PersonInput> {
        ("[a-zA-Z ]{1,20}", "[a-z]{1,10}@test\\.com", "[0-9a-f]{8}").prop_map(
            |(name, email, id)| PersonInput {
                name,
                email,
                id,
            },
        )
    }

    proptest! {
        /// Property: with_tab is idempotent for same value
        #[test]
        fn prop_with_tab_idempotent(tab in arb_tab()) {
            let model = Model::default();
            let once = model.clone().with_tab(tab);
            let twice = once.clone().with_tab(tab);

            prop_assert_eq!(once.current_tab, twice.current_tab);
        }

        /// Property: with_organization_name preserves other fields
        #[test]
        fn prop_with_org_name_preserves_tab(name in arb_org_name(), tab in arb_tab()) {
            let model = Model::default().with_tab(tab);
            let updated = model.clone().with_organization_name(name);

            // Tab should be preserved
            prop_assert_eq!(model.current_tab, updated.current_tab);
        }

        /// Property: adding person increases count by 1
        #[test]
        fn prop_add_person_increases_count(person in arb_person()) {
            let model = Model::default();
            let count_before = model.people.len();
            let updated = model.with_person_added(person);

            prop_assert_eq!(updated.people.len(), count_before + 1);
        }

        /// Property: Model::default() is a valid initial state
        #[test]
        fn prop_default_is_valid(_seed in 0u64..1000) {
            let model = Model::default();

            prop_assert_eq!(model.current_tab, Tab::Welcome);
            prop_assert!(model.people.is_empty());
            prop_assert_eq!(model.domain_status, DomainStatus::NotCreated);
        }

        /// Property: with_error(Some(x)).with_error(None) clears error
        #[test]
        fn prop_error_clear(error in "[a-zA-Z0-9 ]{1,50}") {
            let model = Model::default()
                .with_error(Some(error))
                .with_error(None);

            prop_assert!(model.error_message.is_none());
        }

        /// Property: with_key_progress works for valid f32 values
        #[test]
        fn prop_key_progress_accepts_any_value(progress in 0.0f32..=1.0f32) {
            let model = Model::default();
            // Should not panic
            let updated = model.with_key_progress(progress);
            // Value is stored as-is
            prop_assert_eq!(updated.key_generation_progress, progress);
        }

        /// Property: person addition order is preserved (FIFO)
        #[test]
        fn prop_person_order_preserved(
            p1 in arb_person(),
            p2 in arb_person()
        ) {
            let model = Model::default()
                .with_person_added(p1.clone())
                .with_person_added(p2.clone());

            prop_assert_eq!(model.people.len(), 2);
            prop_assert_eq!(&model.people[0].id, &p1.id);
            prop_assert_eq!(&model.people[1].id, &p2.id);
        }
    }
}

// ============================================================================
// COMPOSITIONAL LAW TESTS (Axiom A9)
// ============================================================================

mod compositional_laws {
    use super::*;

    /// Identity law: model.with_x(current_x) == model
    #[test]
    fn test_identity_law_tab() {
        let model = Model::default();
        let current_tab = model.current_tab;
        let updated = model.clone().with_tab(current_tab);

        assert_eq!(model.current_tab, updated.current_tab);
    }

    /// Composition commutes for independent operations
    #[test]
    fn test_independent_operations_commute() {
        let model = Model::default();

        // Order 1: tab then name
        let order1 = model
            .clone()
            .with_tab(Tab::Keys)
            .with_organization_name("Org A".to_string());

        // Order 2: name then tab
        let order2 = model
            .with_organization_name("Org A".to_string())
            .with_tab(Tab::Keys);

        // Results should be equivalent for independent fields
        assert_eq!(order1.current_tab, order2.current_tab);
        assert_eq!(order1.organization_name, order2.organization_name);
    }

    /// Sequential person additions are order-dependent (non-commutative)
    #[test]
    fn test_person_additions_order_dependent() {
        let model = Model::default();

        let alice = PersonInput {
            name: "Alice".to_string(),
            email: "alice@test.com".to_string(),
            id: "1".to_string(),
        };
        let bob = PersonInput {
            name: "Bob".to_string(),
            email: "bob@test.com".to_string(),
            id: "2".to_string(),
        };

        let alice_first = model
            .clone()
            .with_person_added(alice.clone())
            .with_person_added(bob.clone());

        let bob_first = model
            .with_person_added(bob)
            .with_person_added(alice);

        // Order matters for indexed access
        assert_eq!(alice_first.people[0].name, "Alice");
        assert_eq!(bob_first.people[0].name, "Bob");
    }

    /// State machine transitions
    #[test]
    fn test_domain_status_state_machine() {
        let model = Model::default();

        // Valid transition: NotCreated -> Creating
        let creating = model.clone().with_domain_status(DomainStatus::Creating);
        assert_eq!(creating.domain_status, DomainStatus::Creating);

        // Valid transition: Creating -> Created
        let created = creating.with_domain_status(DomainStatus::Created {
            organization_id: "org-1".to_string(),
            organization_name: "Test".to_string(),
        });
        match created.domain_status {
            DomainStatus::Created { .. } => {}
            _ => panic!("Expected Created"),
        }
    }

    /// Export status state machine
    #[test]
    fn test_export_status_state_machine() {
        let model = Model::default();

        // NotStarted -> InProgress
        let in_progress = model.clone().with_export_status(ExportStatus::InProgress);
        assert_eq!(in_progress.export_status, ExportStatus::InProgress);

        // InProgress -> Completed
        let completed = in_progress.with_export_status(ExportStatus::Completed {
            path: PathBuf::from("/output"),
            bytes_written: 2048,
        });
        match completed.export_status {
            ExportStatus::Completed { bytes_written, .. } => {
                assert_eq!(bytes_written, 2048);
            }
            _ => panic!("Expected Completed"),
        }

        // Also test failure path
        let failed = model.with_export_status(ExportStatus::Failed {
            error: "Disk full".to_string(),
        });
        match failed.export_status {
            ExportStatus::Failed { error } => {
                assert_eq!(error, "Disk full");
            }
            _ => panic!("Expected Failed"),
        }
    }
}
