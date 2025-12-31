// Copyright (c) 2025 - Cowboy AI, LLC.

//! Searchable Text Fold - Query Layer Natural Transformation
//!
//! This fold transforms domain nodes into searchable text for filtering.
//! It executes during query operations, producing pure selection data.
//!
//! ## FRP Pipeline Role
//!
//! ```text
//! Model + Query → [FoldSearchableText] → SearchableText → matches() → bool
//! ```
//!
//! ## Categorical Structure
//!
//! FoldSearchableText is a natural transformation:
//! ```text
//! η: DomainNode ⟹ SearchableText
//! ```

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    Person, Organization, OrganizationUnit, Location, Policy, Role, KeyOwnerRole,
};
use crate::domain::pki::{KeyAlgorithm, KeyPurpose};
use crate::domain::yubikey::PIVSlot;
use crate::domain::visualization::SeparationClass;
use crate::domain_projections::NatsIdentityProjection;
use crate::gui::domain_node::FoldDomainNode;

// ============================================================================
// OUTPUT TYPE
// ============================================================================

/// Searchable text extracted from a domain node.
///
/// Contains text fields and keywords for search matching.
#[derive(Debug, Clone)]
pub struct SearchableText {
    /// Text fields from the node data (name, email, subject, etc.)
    pub fields: Vec<String>,
    /// Type-specific keywords (e.g., "nats operator", "certificate", "yubikey")
    pub keywords: Vec<String>,
}

impl SearchableText {
    /// Check if any field or keyword contains the query (case-insensitive)
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.fields.iter().any(|f| f.to_lowercase().contains(&query_lower)) ||
        self.keywords.iter().any(|k| k.contains(&query_lower))
    }

    /// Create empty searchable text
    pub fn empty() -> Self {
        Self {
            fields: Vec::new(),
            keywords: Vec::new(),
        }
    }
}

// ============================================================================
// FOLD IMPLEMENTATION
// ============================================================================

/// Folder that transforms domain nodes into searchable text.
///
/// This is a QUERY fold - it executes for search/filter operations
/// and produces pure selection data with no side effects.
pub struct FoldSearchableText;

impl FoldDomainNode for FoldSearchableText {
    type Output = SearchableText;

    fn fold_person(&self, person: &Person, _role: &KeyOwnerRole) -> Self::Output {
        SearchableText {
            fields: vec![person.name.clone(), person.email.clone()],
            keywords: vec!["person".to_string(), "user".to_string()],
        }
    }

    fn fold_organization(&self, org: &Organization) -> Self::Output {
        SearchableText {
            fields: vec![
                org.name.clone(),
                org.display_name.clone(),
                org.description.clone().unwrap_or_default(),
            ],
            keywords: vec!["organization".to_string(), "org".to_string()],
        }
    }

    fn fold_organization_unit(&self, unit: &OrganizationUnit) -> Self::Output {
        SearchableText {
            fields: vec![unit.name.clone()],
            keywords: vec!["unit".to_string(), "department".to_string(), "team".to_string()],
        }
    }

    fn fold_location(&self, loc: &Location) -> Self::Output {
        SearchableText {
            fields: vec![loc.name.clone()],
            keywords: vec!["location".to_string(), format!("{:?}", loc.location_type).to_lowercase()],
        }
    }

    fn fold_role(&self, role: &Role) -> Self::Output {
        SearchableText {
            fields: vec![role.name.clone(), role.description.clone()],
            keywords: vec!["role".to_string()],
        }
    }

    fn fold_policy(&self, policy: &Policy) -> Self::Output {
        SearchableText {
            fields: vec![policy.name.clone(), policy.description.clone()],
            keywords: vec!["policy".to_string()],
        }
    }

    fn fold_nats_operator(&self, proj: &NatsIdentityProjection) -> Self::Output {
        SearchableText {
            fields: vec![proj.nkey.name.clone().unwrap_or_default()],
            keywords: vec!["nats".to_string(), "operator".to_string()],
        }
    }

    fn fold_nats_account(&self, proj: &NatsIdentityProjection) -> Self::Output {
        SearchableText {
            fields: vec![proj.nkey.name.clone().unwrap_or_default()],
            keywords: vec!["nats".to_string(), "account".to_string()],
        }
    }

    fn fold_nats_user(&self, proj: &NatsIdentityProjection) -> Self::Output {
        SearchableText {
            fields: vec![proj.nkey.name.clone().unwrap_or_default()],
            keywords: vec!["nats".to_string(), "user".to_string()],
        }
    }

    fn fold_nats_service_account(&self, proj: &NatsIdentityProjection) -> Self::Output {
        SearchableText {
            fields: vec![proj.nkey.name.clone().unwrap_or_default()],
            keywords: vec!["nats".to_string(), "service".to_string(), "account".to_string()],
        }
    }

    fn fold_nats_operator_simple(&self, name: &str, _organization_id: Option<Uuid>) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["nats".to_string(), "operator".to_string()],
        }
    }

    fn fold_nats_account_simple(&self, name: &str, _unit_id: Option<Uuid>, is_system: bool) -> Self::Output {
        let mut keywords = vec!["nats".to_string(), "account".to_string()];
        if is_system {
            keywords.push("system".to_string());
        }
        SearchableText {
            fields: vec![name.to_string()],
            keywords,
        }
    }

    fn fold_nats_user_simple(&self, name: &str, _person_id: Option<Uuid>, account_name: &str) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string(), account_name.to_string()],
            keywords: vec!["nats".to_string(), "user".to_string()],
        }
    }

    fn fold_root_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        _not_before: DateTime<Utc>,
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        SearchableText {
            fields: vec![subject.to_string(), issuer.to_string()],
            keywords: vec!["certificate".to_string(), "root".to_string(), "ca".to_string(), "pki".to_string()],
        }
    }

    fn fold_intermediate_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        _not_before: DateTime<Utc>,
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        SearchableText {
            fields: vec![subject.to_string(), issuer.to_string()],
            keywords: vec!["certificate".to_string(), "intermediate".to_string(), "ca".to_string(), "pki".to_string()],
        }
    }

    fn fold_leaf_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        _not_before: DateTime<Utc>,
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
        san: &[String],
    ) -> Self::Output {
        let mut fields = vec![subject.to_string(), issuer.to_string()];
        fields.extend(san.iter().cloned());

        SearchableText {
            fields,
            keywords: vec!["certificate".to_string(), "leaf".to_string(), "pki".to_string()],
        }
    }

    fn fold_key(
        &self,
        _key_id: Uuid,
        algorithm: &KeyAlgorithm,
        purpose: &KeyPurpose,
        _expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output {
        SearchableText {
            fields: vec![format!("{:?}", purpose), format!("{:?}", algorithm)],
            keywords: vec!["key".to_string(), "cryptographic".to_string()],
        }
    }

    fn fold_yubikey(
        &self,
        _device_id: Uuid,
        serial: &str,
        version: &str,
        _provisioned_at: Option<DateTime<Utc>>,
        _slots_used: &[String],
    ) -> Self::Output {
        SearchableText {
            fields: vec![serial.to_string(), version.to_string()],
            keywords: vec!["yubikey".to_string(), "hardware".to_string(), "token".to_string()],
        }
    }

    fn fold_piv_slot(
        &self,
        _slot_id: Uuid,
        slot_name: &str,
        yubikey_serial: &str,
        _has_key: bool,
        certificate_subject: Option<&String>,
    ) -> Self::Output {
        let mut fields = vec![slot_name.to_string(), yubikey_serial.to_string()];
        if let Some(subject) = certificate_subject {
            fields.push(subject.clone());
        }

        SearchableText {
            fields,
            keywords: vec!["piv".to_string(), "slot".to_string()],
        }
    }

    fn fold_yubikey_status(
        &self,
        _person_id: Uuid,
        yubikey_serial: Option<&String>,
        _slots_provisioned: &[PIVSlot],
        _slots_needed: &[PIVSlot],
    ) -> Self::Output {
        SearchableText {
            fields: vec![yubikey_serial.cloned().unwrap_or_default()],
            keywords: vec!["yubikey".to_string(), "status".to_string()],
        }
    }

    fn fold_manifest(
        &self,
        _manifest_id: Uuid,
        name: &str,
        destination: Option<&std::path::PathBuf>,
        _checksum: Option<&String>,
    ) -> Self::Output {
        let mut fields = vec![name.to_string()];
        if let Some(dest) = destination {
            fields.push(dest.display().to_string());
        }

        SearchableText {
            fields,
            keywords: vec!["manifest".to_string(), "export".to_string()],
        }
    }

    fn fold_policy_role(
        &self,
        _role_id: Uuid,
        name: &str,
        purpose: &str,
        _level: u8,
        _separation_class: SeparationClass,
        _claim_count: usize,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string(), purpose.to_string()],
            keywords: vec!["policy".to_string(), "role".to_string()],
        }
    }

    fn fold_policy_claim(
        &self,
        _claim_id: Uuid,
        name: &str,
        category: &str,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string(), category.to_string()],
            keywords: vec!["policy".to_string(), "claim".to_string()],
        }
    }

    fn fold_policy_category(
        &self,
        _category_id: Uuid,
        name: &str,
        _claim_count: usize,
        _expanded: bool,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["policy".to_string(), "category".to_string()],
        }
    }

    fn fold_policy_group(
        &self,
        _class_id: Uuid,
        name: &str,
        separation_class: SeparationClass,
        _role_count: usize,
        _expanded: bool,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string(), format!("{:?}", separation_class)],
            keywords: vec!["policy".to_string(), "group".to_string(), "separation".to_string()],
        }
    }

    fn fold_aggregate_organization(
        &self,
        name: &str,
        _version: u64,
        _people_count: usize,
        _units_count: usize,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["aggregate".to_string(), "organization".to_string()],
        }
    }

    fn fold_aggregate_pki_chain(
        &self,
        name: &str,
        _version: u64,
        _certificates_count: usize,
        _keys_count: usize,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["aggregate".to_string(), "pki".to_string(), "certificate".to_string()],
        }
    }

    fn fold_aggregate_nats_security(
        &self,
        name: &str,
        _version: u64,
        _operators_count: usize,
        _accounts_count: usize,
        _users_count: usize,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["aggregate".to_string(), "nats".to_string(), "security".to_string()],
        }
    }

    fn fold_aggregate_yubikey_provisioning(
        &self,
        name: &str,
        _version: u64,
        _devices_count: usize,
        _slots_provisioned: usize,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["aggregate".to_string(), "yubikey".to_string(), "provisioning".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_searchable_text_matches() {
        let text = SearchableText {
            fields: vec!["John Doe".to_string(), "john@example.com".to_string()],
            keywords: vec!["person".to_string()],
        };

        assert!(text.matches("john"));
        assert!(text.matches("JOHN")); // case insensitive
        assert!(text.matches("doe"));
        assert!(text.matches("example"));
        assert!(text.matches("person"));
        assert!(!text.matches("alice"));
    }

    #[test]
    fn test_searchable_text_empty() {
        let text = SearchableText::empty();
        assert!(text.fields.is_empty());
        assert!(text.keywords.is_empty());
        assert!(!text.matches("anything"));
    }
}
