// CLAN Bootstrap Loader
//
// Loads clan-bootstrap.json configuration and converts it to domain structures
// for NSC export compatible with CLAN infrastructure.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::domain::{
    NatsPermissions, Organization, OrganizationUnit, OrganizationUnitType, Person, PersonRole,
    Permission, RoleScope, RoleType,
};

/// CLAN bootstrap configuration file structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClanBootstrapConfig {
    pub organization: OrganizationConfig,
    pub organizational_units: Vec<OrganizationalUnitConfig>,
    pub service_people: Vec<ServicePersonConfig>,
    #[serde(default)]
    pub nsc_export_config: Option<NscExportConfig>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Organization configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrganizationConfig {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
}

/// Organizational unit configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrganizationalUnitConfig {
    pub name: String,
    pub unit_type: String,
    #[serde(default)]
    pub description: Option<String>,
    pub nats_account_name: String,
    #[serde(default)]
    pub parent: Option<String>,
}

/// Service person configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServicePersonConfig {
    pub name: String,
    pub email: String,
    pub organizational_unit: String,
    pub role: String,
    pub nats_permissions: NatsPermissionsConfig,
    #[serde(default)]
    pub description: Option<String>,
}

/// NATS permissions configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NatsPermissionsConfig {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
    pub allow_responses: bool,
    pub max_payload: usize,
}

/// NSC export configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NscExportConfig {
    pub output_path: String,
    pub operator_name: String,
    pub generate_operator_jwt: bool,
    pub generate_account_jwts: bool,
    pub generate_user_creds: bool,
    #[serde(default)]
    pub secure_keys_directory: Option<bool>,
    #[serde(default)]
    pub key_permissions: Option<String>,
}

/// Bootstrap loader
pub struct ClanBootstrapLoader;

impl ClanBootstrapLoader {
    /// Load CLAN bootstrap configuration from JSON file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<ClanBootstrapConfig, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read bootstrap file: {}", e))?;

        let config: ClanBootstrapConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse bootstrap JSON: {}", e))?;

        Ok(config)
    }

    /// Convert bootstrap configuration to domain models
    pub fn to_domain_models(
        config: ClanBootstrapConfig,
    ) -> Result<(Organization, Vec<OrganizationUnit>, Vec<Person>), String> {
        // Create organization
        let org = Self::build_organization(&config.organization)?;

        // Create organizational units
        let units = Self::build_organizational_units(&config.organizational_units, &org)?;

        // Create service people
        let people = Self::build_service_people(&config.service_people, &org, &units)?;

        Ok((org, units, people))
    }

    /// Build organization from configuration
    fn build_organization(config: &OrganizationConfig) -> Result<Organization, String> {
        let mut metadata = HashMap::new();
        if let Some(domain) = &config.domain {
            metadata.insert("domain".to_string(), domain.clone());
        }

        Ok(Organization {
            id: Uuid::now_v7(),
            name: config.name.clone(),
            display_name: config.display_name.clone(),
            description: config.description.clone(),
            parent_id: None,
            units: Vec::new(), // Will be populated separately
            metadata,
        })
    }

    /// Build organizational units from configuration
    fn build_organizational_units(
        configs: &[OrganizationalUnitConfig],
        _org: &Organization,
    ) -> Result<Vec<OrganizationUnit>, String> {
        let mut units = Vec::new();

        for config in configs {
            let unit_type = Self::parse_unit_type(&config.unit_type)?;

            let unit = OrganizationUnit {
                id: Uuid::now_v7(),
                name: config.name.clone(),
                unit_type,
                parent_unit_id: None, // TODO: Handle parent relationships
                responsible_person_id: None,
                nats_account_name: Some(config.nats_account_name.clone()),
            };

            units.push(unit);
        }

        Ok(units)
    }

    /// Build service people from configuration
    fn build_service_people(
        configs: &[ServicePersonConfig],
        org: &Organization,
        units: &[OrganizationUnit],
    ) -> Result<Vec<Person>, String> {
        let mut people = Vec::new();

        for config in configs {
            // Find the organizational unit by name
            let unit = units
                .iter()
                .find(|u| u.name == config.organizational_unit)
                .ok_or_else(|| {
                    format!(
                        "Organizational unit '{}' not found for person '{}'",
                        config.organizational_unit, config.name
                    )
                })?;

            // Parse role type
            let role_type = Self::parse_role_type(&config.role)?;

            // Convert NATS permissions
            let nats_permissions = Some(NatsPermissions {
                publish: config.nats_permissions.publish.clone(),
                subscribe: config.nats_permissions.subscribe.clone(),
                allow_responses: config.nats_permissions.allow_responses,
                max_payload: Some(config.nats_permissions.max_payload),
            });

            let person = Person {
                id: Uuid::now_v7(),
                name: config.name.clone(),
                email: config.email.clone(),
                roles: vec![PersonRole {
                    role_type,
                    scope: RoleScope::Organization,
                    permissions: vec![Permission::CreateKeys, Permission::SignCertificates],
                }],
                organization_id: org.id,
                unit_ids: vec![unit.id],
                active: true,
                nats_permissions,
                owner_id: None,
            };

            people.push(person);
        }

        Ok(people)
    }

    /// Parse organizational unit type from string
    fn parse_unit_type(type_str: &str) -> Result<OrganizationUnitType, String> {
        match type_str.to_lowercase().as_str() {
            "division" => Ok(OrganizationUnitType::Division),
            "department" => Ok(OrganizationUnitType::Department),
            "team" => Ok(OrganizationUnitType::Team),
            "infrastructure" => Ok(OrganizationUnitType::Infrastructure),
            "service" => Ok(OrganizationUnitType::Service),
            "project" => Ok(OrganizationUnitType::Project),
            _ => Err(format!("Unknown unit type: {}", type_str)),
        }
    }

    /// Parse role type from string
    fn parse_role_type(role_str: &str) -> Result<RoleType, String> {
        match role_str.to_lowercase().as_str() {
            "executive" => Ok(RoleType::Executive),
            "administrator" => Ok(RoleType::Administrator),
            "developer" => Ok(RoleType::Developer),
            "operator" => Ok(RoleType::Operator),
            "auditor" => Ok(RoleType::Auditor),
            "service" => Ok(RoleType::Service),
            _ => Err(format!("Unknown role type: {}", role_str)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_clan_bootstrap_example() {
        let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json");
        assert!(config.is_ok(), "Failed to load clan-bootstrap.json: {:?}", config.err());

        let config = config.unwrap();
        assert_eq!(config.organization.name, "thecowboyai");
        assert_eq!(config.organizational_units.len(), 3);
        assert_eq!(config.service_people.len(), 11);
    }

    #[test]
    fn test_to_domain_models() {
        let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json").unwrap();
        let result = ClanBootstrapLoader::to_domain_models(config);

        assert!(result.is_ok(), "Failed to convert to domain models: {:?}", result.err());

        let (org, units, people) = result.unwrap();

        assert_eq!(org.name, "thecowboyai");
        assert_eq!(units.len(), 3);
        assert_eq!(people.len(), 11);

        // Verify NATS account names are set
        for unit in &units {
            assert!(unit.nats_account_name.is_some());
        }

        // Verify NATS permissions are set for service people
        for person in &people {
            assert!(person.nats_permissions.is_some());
            let perms = person.nats_permissions.as_ref().unwrap();
            assert!(!perms.publish.is_empty());
            assert!(!perms.subscribe.is_empty());
        }
    }
}
