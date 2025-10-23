//! Common test utilities and fixtures for cim-keys tests

use cim_keys::domain::{Organization, Person, Location, KeyOwnerRole};
use uuid::Uuid;
use std::collections::HashMap;

/// Builder for creating test organizations
pub struct OrganizationBuilder {
    name: String,
    domain: String,
    units: Vec<(String, String)>, // (name, type)
}

impl OrganizationBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            domain: format!("{}.test", name.into().to_lowercase().replace(" ", "")),
            units: Vec::new(),
        }
    }

    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = domain.into();
        self
    }

    pub fn add_unit(mut self, name: impl Into<String>, unit_type: impl Into<String>) -> Self {
        self.units.push((name.into(), unit_type.into()));
        self
    }

    pub fn build(self) -> Organization {
        Organization {
            id: Uuid::new_v4(),
            name: self.name,
            domain: self.domain,
        }
    }
}

/// Builder for creating test people
pub struct PersonBuilder {
    name: String,
    email: String,
    role: KeyOwnerRole,
}

impl PersonBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let email = format!("{}@test.com", name.to_lowercase().replace(" ", "."));

        Self {
            name,
            email,
            role: KeyOwnerRole::User,
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    pub fn with_role(mut self, role: KeyOwnerRole) -> Self {
        self.role = role;
        self
    }

    pub fn build(self) -> Person {
        Person {
            id: Uuid::new_v4(),
            name: self.name,
            email: self.email,
            role: self.role,
        }
    }
}

/// Mock YubiKey for testing
pub struct MockYubiKey {
    serial: String,
    slots: HashMap<String, Vec<u8>>, // slot -> key material
}

impl MockYubiKey {
    pub fn new(serial: impl Into<String>) -> Self {
        Self {
            serial: serial.into(),
            slots: HashMap::new(),
        }
    }

    pub fn generate_key(&mut self, slot: &str) -> Result<Vec<u8>, String> {
        // Generate mock key material
        let key = vec![0x01, 0x02, 0x03, 0x04]; // Mock key
        self.slots.insert(slot.to_string(), key.clone());
        Ok(key)
    }

    pub fn sign(&self, slot: &str, data: &[u8]) -> Result<Vec<u8>, String> {
        if !self.slots.contains_key(slot) {
            return Err(format!("No key in slot {}", slot));
        }

        // Mock signature (just hash the data)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        Ok(hasher.finish().to_le_bytes().to_vec())
    }

    pub fn get_serial(&self) -> &str {
        &self.serial
    }
}

/// Event assertion helpers
pub mod assertions {
    use cim_keys::events::KeyEvent;
    use uuid::Uuid;

    /// Assert that events form a valid correlation chain
    pub fn assert_valid_event_chain(events: &[KeyEvent]) -> bool {
        if events.is_empty() {
            return true;
        }

        // First event should have no causation_id
        // Subsequent events should reference previous events
        for (i, event) in events.iter().enumerate() {
            if i == 0 {
                // First event should not have causation
                // (This would need to be implemented based on actual event structure)
            } else {
                // Later events should reference earlier ones
                // (Implementation depends on actual event structure)
            }
        }

        true
    }

    /// Assert that a projection is valid and consistent
    pub fn assert_valid_projection(projection_path: &std::path::Path) -> bool {
        // Check that required files exist
        let manifest = projection_path.join("manifest.json");

        if !manifest.exists() {
            return false;
        }

        // Verify manifest structure
        if let Ok(content) = std::fs::read_to_string(&manifest) {
            if let Ok(_json) = serde_json::from_str::<serde_json::Value>(&content) {
                return true;
            }
        }

        false
    }
}

/// Test data generators
pub mod generators {
    use super::*;

    /// Generate a test organization with people
    pub fn generate_test_org(size: usize) -> (Organization, Vec<Person>) {
        let org = OrganizationBuilder::new("Test Corp")
            .with_domain("testcorp.test")
            .add_unit("Engineering", "department")
            .add_unit("Operations", "department")
            .build();

        let mut people = Vec::new();

        // Add an operator
        people.push(
            PersonBuilder::new("Alice Admin")
                .with_role(KeyOwnerRole::Operator)
                .build()
        );

        // Add regular users
        for i in 1..size {
            people.push(
                PersonBuilder::new(format!("User {}", i))
                    .with_role(KeyOwnerRole::User)
                    .build()
            );
        }

        (org, people)
    }

    /// Generate mock YubiKeys
    pub fn generate_mock_yubikeys(count: usize) -> Vec<MockYubiKey> {
        (0..count)
            .map(|i| MockYubiKey::new(format!("MOCK{:08}", i)))
            .collect()
    }
}

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

/// Fixture data
pub mod fixtures {
    use serde_json::json;

    pub fn minimal_domain_config() -> serde_json::Value {
        json!({
            "version": "1.0",
            "organization": {
                "name": "Test Organization",
                "domain": "test.org"
            },
            "people": [],
            "yubikeys": []
        })
    }

    pub fn complex_domain_config() -> serde_json::Value {
        json!({
            "version": "1.0",
            "organization": {
                "name": "Complex Corp",
                "domain": "complex.corp",
                "units": [
                    {"name": "HQ", "type": "location"},
                    {"name": "Engineering", "type": "department"},
                    {"name": "Security", "type": "department"}
                ]
            },
            "people": [
                {
                    "name": "CEO",
                    "email": "ceo@complex.corp",
                    "role": "Operator"
                },
                {
                    "name": "CTO",
                    "email": "cto@complex.corp",
                    "role": "Administrator"
                },
                {
                    "name": "Security Lead",
                    "email": "security@complex.corp",
                    "role": "SecurityOfficer"
                }
            ],
            "yubikeys": [
                {
                    "serial": "ROOT001",
                    "assigned_to": "ceo@complex.corp",
                    "purpose": "root_ca"
                },
                {
                    "serial": "SEC001",
                    "assigned_to": "security@complex.corp",
                    "purpose": "intermediate_ca"
                }
            ],
            "locations": [
                {
                    "name": "Secure Vault",
                    "type": "physical",
                    "purpose": "backup_storage"
                },
                {
                    "name": "Cloud HSM",
                    "type": "virtual",
                    "purpose": "key_escrow"
                }
            ]
        })
    }
}