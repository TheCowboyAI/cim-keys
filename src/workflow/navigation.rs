// Copyright (c) 2025 - Cowboy AI, LLC.

//! Navigation System for Trust Chain Gap Fulfillment
//!
//! This module provides navigation from trust chain gaps to the specific
//! domain objects that need to be implemented or modified to fulfill them.
//!
//! ## Navigation Hierarchy
//!
//! ```text
//! Gap (e.g., CertificateChainVerification)
//!   │
//!   ├── Module (e.g., src/value_objects/core.rs)
//!   │     │
//!   │     └── Object (e.g., CertificateChain::verify)
//!   │           │
//!   │           └── Aspect (e.g., signature verification)
//!   │
//!   └── Related Objects (dependencies, tests, docs)
//! ```

use std::path::PathBuf;
use super::gaps::{GapId, RequiredObject, TrustChainGap, ObjectType};

/// Target for navigation within the codebase
#[derive(Debug, Clone)]
pub struct NavigationTarget {
    /// Path to the file
    pub file_path: PathBuf,
    /// Line number (if known)
    pub line_number: Option<u32>,
    /// Column number (if known)
    pub column_number: Option<u32>,
    /// Description of what's at this location
    pub description: String,
    /// The type of target
    pub target_type: TargetType,
}

/// Type of navigation target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    /// Source code implementation
    Implementation,
    /// Test file
    Test,
    /// BDD specification
    BddSpec,
    /// Documentation
    Documentation,
    /// Configuration
    Configuration,
}

impl NavigationTarget {
    pub fn new(file_path: impl Into<PathBuf>, description: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            line_number: None,
            column_number: None,
            description: description.into(),
            target_type: TargetType::Implementation,
        }
    }

    pub fn with_line(mut self, line: u32) -> Self {
        self.line_number = Some(line);
        self
    }

    pub fn with_type(mut self, target_type: TargetType) -> Self {
        self.target_type = target_type;
        self
    }

    /// Format as a clickable location string (file:line:col)
    pub fn location_string(&self) -> String {
        match (self.line_number, self.column_number) {
            (Some(line), Some(col)) => format!("{}:{}:{}", self.file_path.display(), line, col),
            (Some(line), None) => format!("{}:{}", self.file_path.display(), line),
            _ => self.file_path.display().to_string(),
        }
    }
}

/// A path through the codebase to fulfill a gap
#[derive(Debug, Clone)]
pub struct NavigationPath {
    /// The gap this path addresses
    pub gap_id: GapId,
    /// Primary implementation targets
    pub implementation_targets: Vec<NavigationTarget>,
    /// Test targets
    pub test_targets: Vec<NavigationTarget>,
    /// BDD specification targets
    pub bdd_targets: Vec<NavigationTarget>,
    /// Documentation targets
    pub doc_targets: Vec<NavigationTarget>,
    /// Suggested order of work
    pub suggested_order: Vec<String>,
}

impl NavigationPath {
    pub fn new(gap_id: GapId) -> Self {
        Self {
            gap_id,
            implementation_targets: Vec::new(),
            test_targets: Vec::new(),
            bdd_targets: Vec::new(),
            doc_targets: Vec::new(),
            suggested_order: Vec::new(),
        }
    }

    pub fn add_implementation(&mut self, target: NavigationTarget) {
        self.implementation_targets.push(target);
    }

    pub fn add_test(&mut self, target: NavigationTarget) {
        self.test_targets.push(target);
    }

    pub fn add_bdd(&mut self, target: NavigationTarget) {
        self.bdd_targets.push(target);
    }

    pub fn add_doc(&mut self, target: NavigationTarget) {
        self.doc_targets.push(target);
    }

    /// Get all targets in suggested work order
    pub fn all_targets(&self) -> Vec<&NavigationTarget> {
        let mut targets = Vec::new();
        // BDD first (understand the requirement)
        targets.extend(self.bdd_targets.iter());
        // Then implementation
        targets.extend(self.implementation_targets.iter());
        // Then tests
        targets.extend(self.test_targets.iter());
        // Finally documentation
        targets.extend(self.doc_targets.iter());
        targets
    }

    /// Count of unfulfilled targets
    pub fn pending_count(&self) -> usize {
        self.implementation_targets.len() +
        self.test_targets.len() +
        self.bdd_targets.len() +
        self.doc_targets.len()
    }
}

/// Navigator for finding and organizing targets for gap fulfillment
pub struct ObjectNavigator {
    /// Base path of the codebase
    base_path: PathBuf,
    /// All gaps for reference
    gaps: Vec<TrustChainGap>,
}

impl ObjectNavigator {
    pub fn new(base_path: impl Into<PathBuf>, gaps: Vec<TrustChainGap>) -> Self {
        Self {
            base_path: base_path.into(),
            gaps,
        }
    }

    /// Get navigation path for a specific gap
    pub fn navigate_to_gap(&self, gap_id: GapId) -> Option<NavigationPath> {
        let gap = self.gaps.iter().find(|g| g.id == gap_id)?;
        let mut path = NavigationPath::new(gap_id);

        // Add implementation targets from required objects
        for obj in &gap.required_objects {
            if !obj.fulfilled {
                let target = self.object_to_target(obj);
                path.add_implementation(target);
            }
        }

        // Add standard test locations
        path.add_test(self.test_target_for_gap(gap));

        // Add BDD specification
        path.add_bdd(self.bdd_target_for_gap(gap));

        // Add documentation
        path.add_doc(self.doc_target_for_gap(gap));

        // Set suggested order
        path.suggested_order = vec![
            "1. Review BDD specification to understand requirements".to_string(),
            "2. Implement the required objects".to_string(),
            "3. Write unit tests".to_string(),
            "4. Run property-based tests".to_string(),
            "5. Update documentation".to_string(),
        ];

        Some(path)
    }

    /// Convert a RequiredObject to a NavigationTarget
    fn object_to_target(&self, obj: &RequiredObject) -> NavigationTarget {
        let file_path = self.base_path.join(&obj.module_path);

        NavigationTarget {
            file_path,
            line_number: None, // Line numbers resolved dynamically
            column_number: None,
            description: format!(
                "{:?}: {} - {}",
                obj.object_type, obj.name, obj.aspect
            ),
            target_type: TargetType::Implementation,
        }
    }

    /// Get the test file target for a gap
    fn test_target_for_gap(&self, gap: &TrustChainGap) -> NavigationTarget {
        let test_file = match gap.id {
            GapId::CERTIFICATE_CHAIN_VERIFICATION =>
                "tests/certificate_chain_property_tests.rs",
            GapId::TRUST_CHAIN_REFERENCE |
            GapId::KEY_ROTATION_TRUST =>
                "tests/trust_chain_tests.rs",
            GapId::DELEGATION_REVOCATION_CASCADE |
            GapId::CROSS_ORG_TRUST =>
                "tests/delegation_tests.rs",
            GapId::YUBIKEY_SLOT_BINDING =>
                "tests/yubikey_tests.rs",
            GapId::ORPHANED_KEY_DETECTION |
            GapId::SERVICE_ACCOUNT_ACCOUNTABILITY =>
                "tests/key_management_tests.rs",
            GapId::POLICY_EVALUATION_CACHE =>
                "tests/policy_tests.rs",
            GapId::BOOTSTRAP_DOMAIN_DUALITY =>
                "tests/bootstrap_tests.rs",
            _ => "tests/integration_tests.rs",
        };

        NavigationTarget::new(
            self.base_path.join(test_file),
            format!("Property tests for {}", gap.name),
        ).with_type(TargetType::Test)
    }

    /// Get the BDD specification target for a gap
    fn bdd_target_for_gap(&self, gap: &TrustChainGap) -> NavigationTarget {
        let feature_file = match gap.category {
            super::gaps::GapCategory::Pki => "certificate_chain.feature",
            super::gaps::GapCategory::Delegation => "delegation_cascade.feature",
            super::gaps::GapCategory::YubiKey => "yubikey_binding.feature",
            super::gaps::GapCategory::Policy => "policy_evaluation.feature",
            super::gaps::GapCategory::Domain => "domain_bootstrap.feature",
        };

        NavigationTarget::new(
            self.base_path.join("doc/qa/features/trust_chain").join(feature_file),
            format!("BDD scenarios for {}", gap.name),
        ).with_type(TargetType::BddSpec)
    }

    /// Get the documentation target for a gap
    fn doc_target_for_gap(&self, gap: &TrustChainGap) -> NavigationTarget {
        NavigationTarget::new(
            self.base_path.join("doc/domain-ontology/trust-chain-gaps.md"),
            format!("Documentation for {}", gap.name),
        ).with_type(TargetType::Documentation)
    }

    /// Find all unfulfilled objects across all gaps
    pub fn find_all_unfulfilled(&self) -> Vec<(GapId, &RequiredObject)> {
        let mut unfulfilled = Vec::new();

        for gap in &self.gaps {
            for obj in &gap.required_objects {
                if !obj.fulfilled {
                    unfulfilled.push((gap.id, obj));
                }
            }
        }

        unfulfilled
    }

    /// Get objects by type
    pub fn find_by_type(&self, object_type: ObjectType) -> Vec<(GapId, &RequiredObject)> {
        let mut matches = Vec::new();

        for gap in &self.gaps {
            for obj in &gap.required_objects {
                if obj.object_type == object_type {
                    matches.push((gap.id, obj));
                }
            }
        }

        matches
    }

    /// Find objects in a specific module
    pub fn find_in_module(&self, module_path: &str) -> Vec<(GapId, &RequiredObject)> {
        let mut matches = Vec::new();

        for gap in &self.gaps {
            for obj in &gap.required_objects {
                if obj.module_path == module_path {
                    matches.push((gap.id, obj));
                }
            }
        }

        matches
    }

    /// Get navigation summary for display
    pub fn navigation_summary(&self) -> NavigationSummary {
        let mut total_objects = 0;
        let mut fulfilled_objects = 0;
        let mut by_category = std::collections::HashMap::new();

        for gap in &self.gaps {
            let category_stats = by_category
                .entry(gap.category)
                .or_insert((0, 0));

            for obj in &gap.required_objects {
                total_objects += 1;
                category_stats.0 += 1;

                if obj.fulfilled {
                    fulfilled_objects += 1;
                    category_stats.1 += 1;
                }
            }
        }

        NavigationSummary {
            total_objects,
            fulfilled_objects,
            completion_percentage: if total_objects > 0 {
                (fulfilled_objects as f64 / total_objects as f64) * 100.0
            } else {
                0.0
            },
            by_category,
        }
    }
}

/// Summary of navigation state
#[derive(Debug)]
pub struct NavigationSummary {
    pub total_objects: usize,
    pub fulfilled_objects: usize,
    pub completion_percentage: f64,
    pub by_category: std::collections::HashMap<super::gaps::GapCategory, (usize, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_navigation_target_creation() {
        let target = NavigationTarget::new(
            "src/value_objects/core.rs",
            "Certificate chain verification",
        ).with_line(319);

        assert_eq!(target.file_path, Path::new("src/value_objects/core.rs"));
        assert_eq!(target.line_number, Some(319));
        assert_eq!(target.location_string(), "src/value_objects/core.rs:319");
    }

    #[test]
    fn test_navigation_path() {
        let mut path = NavigationPath::new(GapId::CERTIFICATE_CHAIN_VERIFICATION);

        path.add_implementation(NavigationTarget::new(
            "src/value_objects/core.rs",
            "Implementation",
        ));
        path.add_test(NavigationTarget::new(
            "tests/cert_tests.rs",
            "Tests",
        ).with_type(TargetType::Test));

        assert_eq!(path.pending_count(), 2);
        assert_eq!(path.all_targets().len(), 2);
    }

    #[test]
    fn test_navigator_creation() {
        let gaps = TrustChainGap::all_gaps();
        let navigator = ObjectNavigator::new("/project", gaps);

        let path = navigator.navigate_to_gap(GapId::CERTIFICATE_CHAIN_VERIFICATION);
        assert!(path.is_some());
    }

    #[test]
    fn test_find_unfulfilled() {
        let gaps = TrustChainGap::all_gaps();
        let navigator = ObjectNavigator::new("/project", gaps);

        let unfulfilled = navigator.find_all_unfulfilled();
        assert!(!unfulfilled.is_empty());
    }

    #[test]
    fn test_navigation_summary() {
        let gaps = TrustChainGap::all_gaps();
        let navigator = ObjectNavigator::new("/project", gaps);

        let summary = navigator.navigation_summary();
        assert!(summary.total_objects > 0);
        assert!(summary.completion_percentage >= 0.0);
        assert!(summary.completion_percentage <= 100.0);
    }
}
