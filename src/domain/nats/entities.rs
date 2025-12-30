// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Bounded Context
//!
//! This module provides the NATS bounded context for cim-keys.
//! It includes NATS operators, accounts, users, and credential management.
//!
//! ## Domain Types
//!
//! **NATS Security Hierarchy**:
//! - Operator (O prefix NKey) - Maps to Organization
//! - Account (A prefix NKey) - Maps to Organizational Unit
//! - User (U prefix NKey) - Maps to Person or Service Account
//!
//! **Credential Types**:
//! - JWTs for authorization
//! - NKeys for authentication
//! - Permissions for pub/sub access
//!
//! ## Bounded Context Separation
//!
//! This context is responsible for:
//! - NATS operator/account/user hierarchy
//! - NKey generation and management
//! - JWT credential issuance
//! - Subject permissions
//!
//! It does NOT handle:
//! - Organizational structure (see `organization` context)
//! - Certificate PKI (see `pki` context)
//! - Hardware tokens (see `yubikey` context)
//!
//! ## NATS Hierarchy Mapping
//!
//! ```text
//! Organization -> NATS Operator (O prefix NKey)
//! ├── Department -> NATS Account (A prefix NKey)
//! │   ├── Person -> NATS User (U prefix NKey)
//! │   └── Service -> NATS User (U prefix NKey, with owner)
//! └── Team -> NATS Account
//!     └── Person -> NATS User
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export phantom-typed IDs for NATS (go up two levels: nats -> domain)
pub use crate::domain::ids::{
    NatsOperatorId,
    NatsOperatorMarker,
    NatsAccountId,
    NatsAccountMarker,
    NatsUserId,
    NatsUserMarker,
};

// Re-export NATS types from bootstrap
pub use crate::domain::bootstrap::{
    NatsPermissions,
    NatsIdentity,
    UserIdentity,
    AccountIdentity,
    ServiceAccount,
};

/// NATS operator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorInfo {
    pub id: NatsOperatorId,
    pub name: String,
    pub organization_id: Uuid,
    pub signing_key_id: Option<Uuid>,
    pub system_account_id: Option<NatsAccountId>,
}

impl NatsOperatorInfo {
    /// Create new operator info for an organization
    pub fn new(name: String, organization_id: Uuid) -> Self {
        Self {
            id: NatsOperatorId::new(),
            name,
            organization_id,
            signing_key_id: None,
            system_account_id: None,
        }
    }
}

/// NATS account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountInfo {
    pub id: NatsAccountId,
    pub name: String,
    pub operator_id: NatsOperatorId,
    pub unit_id: Uuid,
    pub signing_key_id: Option<Uuid>,
    pub exports: Vec<NatsExport>,
    pub imports: Vec<NatsImport>,
    pub limits: AccountLimits,
}

impl NatsAccountInfo {
    /// Create new account info for an organizational unit
    pub fn new(name: String, operator_id: NatsOperatorId, unit_id: Uuid) -> Self {
        Self {
            id: NatsAccountId::new(),
            name,
            operator_id,
            unit_id,
            signing_key_id: None,
            exports: Vec::new(),
            imports: Vec::new(),
            limits: AccountLimits::default(),
        }
    }
}

/// NATS user configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserInfo {
    pub id: NatsUserId,
    pub name: String,
    pub account_id: NatsAccountId,
    pub person_id: Option<Uuid>,
    pub service_account_id: Option<Uuid>,
    pub permissions: NatsPermissions,
    pub signing_key_id: Option<Uuid>,
}

impl NatsUserInfo {
    /// Create new user info for a person
    pub fn new_for_person(name: String, account_id: NatsAccountId, person_id: Uuid) -> Self {
        Self {
            id: NatsUserId::new(),
            name,
            account_id,
            person_id: Some(person_id),
            service_account_id: None,
            permissions: NatsPermissions {
                publish: Vec::new(),
                subscribe: Vec::new(),
                allow_responses: true,
                max_payload: None,
            },
            signing_key_id: None,
        }
    }

    /// Create new user info for a service account
    pub fn new_for_service(
        name: String,
        account_id: NatsAccountId,
        service_account_id: Uuid,
    ) -> Self {
        Self {
            id: NatsUserId::new(),
            name,
            account_id,
            person_id: None,
            service_account_id: Some(service_account_id),
            permissions: NatsPermissions {
                publish: Vec::new(),
                subscribe: Vec::new(),
                allow_responses: false,
                max_payload: None,
            },
            signing_key_id: None,
        }
    }
}

/// NATS service/stream export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsExport {
    pub name: String,
    pub subject: String,
    pub export_type: ExportType,
    pub token_required: bool,
}

/// Type of NATS export
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportType {
    /// Publish/subscribe stream
    Stream,
    /// Request/reply service
    Service,
}

/// NATS service/stream import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsImport {
    pub name: String,
    pub subject: String,
    pub account: String,
    pub local_subject: Option<String>,
}

/// Account resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLimits {
    pub max_connections: Option<i64>,
    pub max_subscriptions: Option<i64>,
    pub max_payload: Option<i64>,
    pub max_data: Option<i64>,
    pub max_imports: Option<i64>,
    pub max_exports: Option<i64>,
}

impl Default for AccountLimits {
    fn default() -> Self {
        Self {
            max_connections: Some(100),
            max_subscriptions: Some(1000),
            max_payload: Some(1024 * 1024), // 1MB
            max_data: None,
            max_imports: Some(10),
            max_exports: Some(10),
        }
    }
}

impl std::fmt::Display for ExportType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExportType::Stream => write!(f, "Stream"),
            ExportType::Service => write!(f, "Service"),
        }
    }
}
