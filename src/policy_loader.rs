// Copyright (c) 2025 - Cowboy AI, LLC.
//! Policy Bootstrap Loader
//!
//! Loads policy configuration from `policy-bootstrap.json` including:
//! - Standard role definitions
//! - Role assignments to people
//! - C-Level executive assignments
//! - Separation of duties rules
//! - Claim category mappings

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::policy::SeparationClass;

/// Errors that can occur during policy loading
#[derive(Debug, thiserror::Error)]
pub enum PolicyLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Complete policy bootstrap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBootstrapData {
    /// Schema identifier
    #[serde(rename = "$schema", default)]
    pub schema: String,

    /// Schema version
    #[serde(default)]
    pub version: String,

    /// Organization reference
    pub organization: OrganizationRef,

    /// C-Level executive assignments
    pub c_level_assignments: CLevelAssignments,

    /// People with role assignments
    #[serde(default)]
    pub people: Vec<PolicyPersonEntry>,

    /// Standard role definitions
    #[serde(default)]
    pub standard_roles: Vec<StandardRoleEntry>,

    /// Role assignments linking people to roles
    #[serde(default)]
    pub role_assignments: Vec<RoleAssignmentEntry>,

    /// Separation of duties rules
    #[serde(default)]
    pub separation_of_duties_rules: Vec<SeparationRule>,

    /// Claim categories with their claims
    #[serde(default)]
    pub claim_categories: HashMap<String, Vec<String>>,

    /// Metadata about the bootstrap file
    #[serde(default)]
    pub metadata: PolicyMetadata,
}

/// Organization reference in policy bootstrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationRef {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub domain: Option<String>,
}

/// C-Level executive assignments
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CLevelAssignments {
    #[serde(rename = "CEO")]
    pub ceo: Option<PersonRef>,
    #[serde(rename = "COO")]
    pub coo: Option<PersonRef>,
    #[serde(rename = "CFO")]
    pub cfo: Option<PersonRef>,
    #[serde(rename = "CLO")]
    pub clo: Option<PersonRef>,
    #[serde(rename = "CSO")]
    pub cso: Option<PersonRef>,
}

/// Person reference in C-Level assignments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonRef {
    pub person_id: Uuid,
    pub person_name: String,
    pub email: String,
}

/// Person entry in policy bootstrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyPersonEntry {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub assigned_roles: Vec<String>,
    #[serde(default)]
    pub security_clearance: Option<String>,
    #[serde(default = "default_true")]
    pub active: bool,
}

fn default_true() -> bool {
    true
}

/// Standard role definition from bootstrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardRoleEntry {
    pub name: String,
    pub purpose: String,
    #[serde(default)]
    pub level: u8,
    #[serde(default)]
    pub separation_class: String,
    #[serde(default)]
    pub claims: Vec<String>,
    #[serde(default)]
    pub incompatible_with: Vec<String>,
}

impl StandardRoleEntry {
    /// Get the separation class as an enum
    pub fn separation_class_enum(&self) -> SeparationClass {
        match self.separation_class.as_str() {
            "Operational" => SeparationClass::Operational,
            "Administrative" => SeparationClass::Administrative,
            "Audit" => SeparationClass::Audit,
            "Emergency" => SeparationClass::Emergency,
            "Financial" => SeparationClass::Financial,
            "Personnel" => SeparationClass::Personnel,
            _ => SeparationClass::Operational,
        }
    }

    /// Get the color for this role based on separation class
    pub fn color(&self) -> (f32, f32, f32) {
        match self.separation_class_enum() {
            SeparationClass::Operational => (0.3, 0.6, 0.9),    // Blue
            SeparationClass::Administrative => (0.6, 0.4, 0.8), // Purple
            SeparationClass::Audit => (0.2, 0.7, 0.5),          // Teal
            SeparationClass::Emergency => (0.9, 0.3, 0.2),      // Red
            SeparationClass::Financial => (0.9, 0.7, 0.2),      // Gold
            SeparationClass::Personnel => (0.8, 0.4, 0.6),      // Rose
        }
    }

    /// Get the node size based on level (0-5)
    pub fn node_size(&self) -> f32 {
        match self.level {
            0 => 20.0,
            1 => 24.0,
            2 => 28.0,
            3 => 32.0,
            4 => 36.0,
            5 => 40.0,
            _ => 24.0,
        }
    }
}

/// Role assignment entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignmentEntry {
    pub person_id: Uuid,
    #[serde(default)]
    pub person_name: Option<String>,
    pub roles: Vec<String>,
    #[serde(default)]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub granted_by: Option<String>,
    #[serde(default)]
    pub context: Option<AssignmentContext>,
}

/// Context for role assignment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssignmentContext {
    #[serde(default)]
    pub organization_id: Option<Uuid>,
    #[serde(default)]
    pub unit_id: Option<Uuid>,
    #[serde(default)]
    pub environments: Vec<String>,
}

/// Separation of duties rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeparationRule {
    pub name: String,
    pub role_a: String,
    pub conflicts_with: Vec<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Metadata about the policy bootstrap
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyMetadata {
    #[serde(default)]
    pub total_claims: usize,
    #[serde(default)]
    pub total_roles: usize,
    #[serde(default)]
    pub total_people: usize,
    #[serde(default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Policy loader for reading policy-bootstrap.json
pub struct PolicyLoader;

impl PolicyLoader {
    /// Load policy bootstrap data from a file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<PolicyBootstrapData, PolicyLoaderError> {
        let content = fs::read_to_string(path)?;
        let data: PolicyBootstrapData = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Load policy bootstrap data from a string
    pub fn load_from_str(content: &str) -> Result<PolicyBootstrapData, PolicyLoaderError> {
        let data: PolicyBootstrapData = serde_json::from_str(content)?;
        Ok(data)
    }

    /// Get the default policy bootstrap path
    pub fn default_path() -> std::path::PathBuf {
        std::path::PathBuf::from("secrets/policy-bootstrap.json")
    }
}

impl PolicyBootstrapData {
    /// Get role assignments for a specific person by ID
    pub fn get_roles_for_person(&self, person_id: Uuid) -> Vec<&StandardRoleEntry> {
        // Find the role assignment for this person
        let assigned_role_names: Vec<&str> = self
            .role_assignments
            .iter()
            .filter(|a| a.person_id == person_id)
            .flat_map(|a| a.roles.iter().map(|s| s.as_str()))
            .collect();

        // Also check the people entries for assigned_roles
        let person_role_names: Vec<&str> = self
            .people
            .iter()
            .filter(|p| p.id == person_id)
            .flat_map(|p| p.assigned_roles.iter().map(|s| s.as_str()))
            .collect();

        // Combine both sources
        let all_role_names: Vec<&str> = assigned_role_names
            .into_iter()
            .chain(person_role_names)
            .collect();

        // Look up the role definitions
        self.standard_roles
            .iter()
            .filter(|r| all_role_names.contains(&r.name.as_str()))
            .collect()
    }

    /// Get all unique role names that are assigned to people
    pub fn get_assigned_role_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .role_assignments
            .iter()
            .flat_map(|a| a.roles.iter().map(|s| s.as_str()))
            .chain(
                self.people
                    .iter()
                    .flat_map(|p| p.assigned_roles.iter().map(|s| s.as_str())),
            )
            .collect();

        names.sort();
        names.dedup();
        names
    }

    /// Get the standard role entry by name
    pub fn get_role_by_name(&self, name: &str) -> Option<&StandardRoleEntry> {
        self.standard_roles.iter().find(|r| r.name == name)
    }

    /// Get all C-Level assignments as a vec of (role_name, person_ref)
    pub fn get_c_level_list(&self) -> Vec<(&'static str, Option<&PersonRef>)> {
        vec![
            ("CEO", self.c_level_assignments.ceo.as_ref()),
            ("COO", self.c_level_assignments.coo.as_ref()),
            ("CFO", self.c_level_assignments.cfo.as_ref()),
            ("CLO", self.c_level_assignments.clo.as_ref()),
            ("CSO", self.c_level_assignments.cso.as_ref()),
        ]
    }

    /// Calculate effective claims for a person (union of all role claims)
    pub fn get_effective_claims_for_person(&self, person_id: Uuid) -> Vec<&str> {
        let roles = self.get_roles_for_person(person_id);
        let mut claims: Vec<&str> = roles
            .iter()
            .flat_map(|r| r.claims.iter().map(|s| s.as_str()))
            .collect();

        claims.sort();
        claims.dedup();
        claims
    }

    /// Get claim categories with counts
    pub fn get_claim_category_summary(&self) -> Vec<(String, usize)> {
        self.claim_categories
            .iter()
            .map(|(cat, claims)| (cat.clone(), claims.len()))
            .collect()
    }

    /// Check if two roles are incompatible (separation of duties)
    pub fn are_roles_incompatible(&self, role_a: &str, role_b: &str) -> bool {
        self.separation_of_duties_rules.iter().any(|rule| {
            (rule.role_a == role_a && rule.conflicts_with.contains(&role_b.to_string()))
                || (rule.role_a == role_b && rule.conflicts_with.contains(&role_a.to_string()))
        })
    }

    /// Get all person IDs and their role entries (for badge generation)
    pub fn get_all_person_roles(&self) -> Vec<(Uuid, Vec<&StandardRoleEntry>)> {
        // Collect all unique person IDs from various sources
        let mut person_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

        // From role_assignments
        for assignment in &self.role_assignments {
            person_ids.insert(assignment.person_id);
        }

        // From people entries
        for person in &self.people {
            person_ids.insert(person.id);
        }

        // Get roles for each person
        person_ids
            .into_iter()
            .filter_map(|person_id| {
                let roles = self.get_roles_for_person(person_id);
                if roles.is_empty() {
                    None
                } else {
                    Some((person_id, roles))
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_policy_bootstrap() {
        let path = PolicyLoader::default_path();
        if path.exists() {
            let result = PolicyLoader::load_from_file(&path);
            assert!(result.is_ok(), "Failed to load policy bootstrap: {:?}", result.err());

            let data = result.unwrap();
            assert!(!data.standard_roles.is_empty(), "No standard roles loaded");
            assert!(!data.role_assignments.is_empty(), "No role assignments loaded");
        }
    }

    #[test]
    fn test_separation_class_color() {
        let role = StandardRoleEntry {
            name: "Test".to_string(),
            purpose: "Testing".to_string(),
            level: 3,
            separation_class: "Financial".to_string(),
            claims: vec![],
            incompatible_with: vec![],
        };

        let color = role.color();
        assert_eq!(color, (0.9, 0.7, 0.2)); // Gold
    }

    #[test]
    fn test_node_size_by_level() {
        for level in 0..=5 {
            let role = StandardRoleEntry {
                name: "Test".to_string(),
                purpose: "Testing".to_string(),
                level,
                separation_class: "Operational".to_string(),
                claims: vec![],
                incompatible_with: vec![],
            };

            let expected_size = match level {
                0 => 20.0,
                1 => 24.0,
                2 => 28.0,
                3 => 32.0,
                4 => 36.0,
                5 => 40.0,
                _ => 24.0,
            };

            assert_eq!(role.node_size(), expected_size);
        }
    }

    #[test]
    fn test_c_level_assignments() {
        let data = PolicyBootstrapData {
            schema: String::new(),
            version: String::new(),
            organization: OrganizationRef {
                id: Uuid::new_v4(),
                name: "Test".to_string(),
                display_name: "Test Org".to_string(),
                domain: None,
            },
            c_level_assignments: CLevelAssignments {
                ceo: Some(PersonRef {
                    person_id: Uuid::new_v4(),
                    person_name: "Test CEO".to_string(),
                    email: "ceo@test.com".to_string(),
                }),
                coo: None,
                cfo: None,
                clo: None,
                cso: None,
            },
            people: vec![],
            standard_roles: vec![],
            role_assignments: vec![],
            separation_of_duties_rules: vec![],
            claim_categories: HashMap::new(),
            metadata: PolicyMetadata::default(),
        };

        let c_levels = data.get_c_level_list();
        assert_eq!(c_levels.len(), 5);
        assert!(c_levels[0].1.is_some()); // CEO assigned
        assert!(c_levels[1].1.is_none()); // COO not assigned
    }
}
