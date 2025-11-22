//! NATS Identity Types
//!
//! These types provide semantic wrappers around DomainObject for NATS identity projections.
//! They distinguish between different entity roles in the NATS infrastructure:
//! - AccountIdentity: Organizations and OrganizationUnits map to NATS Accounts
//! - UserIdentity: People and ServiceAccounts map to NATS Users

use cim_graph::functors::domain_functor::DomainObject;
use uuid::Uuid;

/// Identity that maps to a NATS Account
///
/// NATS Accounts can be:
/// - Organizations (top-level accounts)
/// - OrganizationUnits (sub-accounts within an organization)
#[derive(Debug, Clone)]
pub enum AccountIdentity {
    Organization(DomainObject),
    OrganizationUnit(DomainObject),
}

impl AccountIdentity {
    /// Get the ID of this account identity
    pub fn id(&self) -> Uuid {
        match self {
            AccountIdentity::Organization(o) | AccountIdentity::OrganizationUnit(o) => o.id,
        }
    }

    /// Get the name of this account identity
    pub fn name(&self) -> &str {
        match self {
            AccountIdentity::Organization(o) | AccountIdentity::OrganizationUnit(o) => {
                o.properties
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unnamed")
            }
        }
    }

    /// Get the underlying DomainObject
    pub fn as_domain_object(&self) -> &DomainObject {
        match self {
            AccountIdentity::Organization(o) | AccountIdentity::OrganizationUnit(o) => o,
        }
    }
}

/// Identity that maps to a NATS User
///
/// NATS Users can be:
/// - People (individual users)
/// - ServiceAccounts (automated systems)
#[derive(Debug, Clone)]
pub enum UserIdentity {
    Person(DomainObject),
    ServiceAccount(DomainObject),
}

impl UserIdentity {
    /// Get the ID of this user identity
    pub fn id(&self) -> Uuid {
        match self {
            UserIdentity::Person(p) | UserIdentity::ServiceAccount(p) => p.id,
        }
    }

    /// Get the name of this user identity
    pub fn name(&self) -> &str {
        match self {
            UserIdentity::Person(p) | UserIdentity::ServiceAccount(p) => {
                p.properties
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unnamed")
            }
        }
    }

    /// Get email for person or purpose for service account
    pub fn description(&self) -> Option<String> {
        match self {
            UserIdentity::Person(p) => p
                .properties
                .get("email")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            UserIdentity::ServiceAccount(s) => s
                .properties
                .get("purpose")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }

    /// Get the responsible person ID for service accounts
    pub fn responsible_person_id(&self) -> Option<Uuid> {
        match self {
            UserIdentity::Person(_) => None,
            UserIdentity::ServiceAccount(s) => s
                .properties
                .get("responsible_person_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok()),
        }
    }

    /// Validate that this identity has proper accountability
    pub fn validate_accountability(&self) -> Result<(), String> {
        match self {
            UserIdentity::Person(_) => Ok(()),
            UserIdentity::ServiceAccount(s) => {
                if s.properties.contains_key("responsible_person_id") {
                    Ok(())
                } else {
                    Err("ServiceAccount must have responsible_person_id".to_string())
                }
            }
        }
    }

    /// Get the underlying DomainObject
    pub fn as_domain_object(&self) -> &DomainObject {
        match self {
            UserIdentity::Person(p) | UserIdentity::ServiceAccount(p) => p,
        }
    }
}
