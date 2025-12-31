// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Bounded Context
//!
//! This module defines the coproduct for the NATS infrastructure
//! bounded context, handling operators, accounts, and users.
//!
//! ## Entities in this Context
//! - NatsOperator (tier 0)
//! - NatsAccount (tier 1)
//! - NatsUser (tier 2)
//! - NatsServiceAccount (tier 2)

use std::fmt;
use uuid::Uuid;

use crate::domain_projections::NatsIdentityProjection;

/// Injection tag for NATS bounded context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NatsInjection {
    /// Operator with full projection data
    Operator,
    /// Account with full projection data
    Account,
    /// User with full projection data
    User,
    /// Service account with full projection data
    ServiceAccount,
    /// Simple operator (visualization only)
    OperatorSimple,
    /// Simple account (visualization only)
    AccountSimple,
    /// Simple user (visualization only)
    UserSimple,
}

impl NatsInjection {
    /// Display name for this entity type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Operator | Self::OperatorSimple => "NATS Operator",
            Self::Account | Self::AccountSimple => "NATS Account",
            Self::User | Self::UserSimple => "NATS User",
            Self::ServiceAccount => "NATS Service Account",
        }
    }

    /// Layout tier for hierarchical visualization
    pub fn layout_tier(&self) -> u8 {
        match self {
            Self::Operator | Self::OperatorSimple => 0,
            Self::Account | Self::AccountSimple => 1,
            Self::User | Self::UserSimple | Self::ServiceAccount => 2,
        }
    }

    /// Check if this is a simple visualization variant
    pub fn is_simple(&self) -> bool {
        matches!(
            self,
            Self::OperatorSimple | Self::AccountSimple | Self::UserSimple
        )
    }
}

impl fmt::Display for NatsInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Inner data for NATS context entities
#[derive(Debug, Clone)]
pub enum NatsData {
    /// Operator with full projection
    Operator(NatsIdentityProjection),
    /// Account with full projection
    Account(NatsIdentityProjection),
    /// User with full projection
    User(NatsIdentityProjection),
    /// Service account with full projection
    ServiceAccount(NatsIdentityProjection),
    /// Simple operator for visualization
    OperatorSimple {
        name: String,
        organization_id: Option<Uuid>,
    },
    /// Simple account for visualization
    AccountSimple {
        name: String,
        unit_id: Option<Uuid>,
        is_system: bool,
    },
    /// Simple user for visualization
    UserSimple {
        name: String,
        person_id: Option<Uuid>,
        account_name: String,
    },
}

/// NATS Entity - Coproduct of NATS-related types
#[derive(Debug, Clone)]
pub struct NatsEntity {
    injection: NatsInjection,
    data: NatsData,
}

impl NatsEntity {
    // ========================================================================
    // Injection Functions
    // ========================================================================

    /// Inject Operator into coproduct
    pub fn inject_operator(projection: NatsIdentityProjection) -> Self {
        Self {
            injection: NatsInjection::Operator,
            data: NatsData::Operator(projection),
        }
    }

    /// Inject Account into coproduct
    pub fn inject_account(projection: NatsIdentityProjection) -> Self {
        Self {
            injection: NatsInjection::Account,
            data: NatsData::Account(projection),
        }
    }

    /// Inject User into coproduct
    pub fn inject_user(projection: NatsIdentityProjection) -> Self {
        Self {
            injection: NatsInjection::User,
            data: NatsData::User(projection),
        }
    }

    /// Inject Service Account into coproduct
    pub fn inject_service_account(projection: NatsIdentityProjection) -> Self {
        Self {
            injection: NatsInjection::ServiceAccount,
            data: NatsData::ServiceAccount(projection),
        }
    }

    /// Inject simple Operator for visualization
    pub fn inject_operator_simple(name: String, organization_id: Option<Uuid>) -> Self {
        Self {
            injection: NatsInjection::OperatorSimple,
            data: NatsData::OperatorSimple { name, organization_id },
        }
    }

    /// Inject simple Account for visualization
    pub fn inject_account_simple(name: String, unit_id: Option<Uuid>, is_system: bool) -> Self {
        Self {
            injection: NatsInjection::AccountSimple,
            data: NatsData::AccountSimple { name, unit_id, is_system },
        }
    }

    /// Inject simple User for visualization
    pub fn inject_user_simple(name: String, person_id: Option<Uuid>, account_name: String) -> Self {
        Self {
            injection: NatsInjection::UserSimple,
            data: NatsData::UserSimple { name, person_id, account_name },
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the injection tag
    pub fn injection(&self) -> NatsInjection {
        self.injection
    }

    /// Get reference to inner data
    pub fn data(&self) -> &NatsData {
        &self.data
    }

    /// Get entity ID
    pub fn id(&self) -> Uuid {
        match &self.data {
            NatsData::Operator(p) => p.nkey.id,
            NatsData::Account(p) => p.nkey.id,
            NatsData::User(p) => p.nkey.id,
            NatsData::ServiceAccount(p) => p.nkey.id,
            NatsData::OperatorSimple { organization_id, .. } => organization_id.unwrap_or_else(Uuid::now_v7),
            NatsData::AccountSimple { unit_id, .. } => unit_id.unwrap_or_else(Uuid::now_v7),
            NatsData::UserSimple { person_id, .. } => person_id.unwrap_or_else(Uuid::now_v7),
        }
    }

    /// Get entity name
    pub fn name(&self) -> String {
        match &self.data {
            NatsData::Operator(p) => p.nkey.name.clone().unwrap_or_else(|| "Operator".to_string()),
            NatsData::Account(p) => p.nkey.name.clone().unwrap_or_else(|| "Account".to_string()),
            NatsData::User(p) => p.nkey.name.clone().unwrap_or_else(|| "User".to_string()),
            NatsData::ServiceAccount(p) => p.nkey.name.clone().unwrap_or_else(|| "Service Account".to_string()),
            NatsData::OperatorSimple { name, .. } => name.clone(),
            NatsData::AccountSimple { name, .. } => name.clone(),
            NatsData::UserSimple { name, .. } => name.clone(),
        }
    }

    // ========================================================================
    // Universal Property (Fold)
    // ========================================================================

    /// Apply a fold to this entity
    pub fn fold<F: FoldNatsEntity>(&self, folder: &F) -> F::Output {
        match &self.data {
            NatsData::Operator(p) => folder.fold_operator(p),
            NatsData::Account(p) => folder.fold_account(p),
            NatsData::User(p) => folder.fold_user(p),
            NatsData::ServiceAccount(p) => folder.fold_service_account(p),
            NatsData::OperatorSimple { name, organization_id } => {
                folder.fold_operator_simple(name, *organization_id)
            }
            NatsData::AccountSimple { name, unit_id, is_system } => {
                folder.fold_account_simple(name, *unit_id, *is_system)
            }
            NatsData::UserSimple { name, person_id, account_name } => {
                folder.fold_user_simple(name, *person_id, account_name)
            }
        }
    }
}

/// Universal property trait for NatsEntity coproduct
pub trait FoldNatsEntity {
    type Output;

    fn fold_operator(&self, projection: &NatsIdentityProjection) -> Self::Output;
    fn fold_account(&self, projection: &NatsIdentityProjection) -> Self::Output;
    fn fold_user(&self, projection: &NatsIdentityProjection) -> Self::Output;
    fn fold_service_account(&self, projection: &NatsIdentityProjection) -> Self::Output;
    fn fold_operator_simple(&self, name: &str, organization_id: Option<Uuid>) -> Self::Output;
    fn fold_account_simple(&self, name: &str, unit_id: Option<Uuid>, is_system: bool) -> Self::Output;
    fn fold_user_simple(&self, name: &str, person_id: Option<Uuid>, account_name: &str) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct InjectionFolder;

    impl FoldNatsEntity for InjectionFolder {
        type Output = NatsInjection;

        fn fold_operator(&self, _: &NatsIdentityProjection) -> Self::Output {
            NatsInjection::Operator
        }
        fn fold_account(&self, _: &NatsIdentityProjection) -> Self::Output {
            NatsInjection::Account
        }
        fn fold_user(&self, _: &NatsIdentityProjection) -> Self::Output {
            NatsInjection::User
        }
        fn fold_service_account(&self, _: &NatsIdentityProjection) -> Self::Output {
            NatsInjection::ServiceAccount
        }
        fn fold_operator_simple(&self, _: &str, _: Option<Uuid>) -> Self::Output {
            NatsInjection::OperatorSimple
        }
        fn fold_account_simple(&self, _: &str, _: Option<Uuid>, _: bool) -> Self::Output {
            NatsInjection::AccountSimple
        }
        fn fold_user_simple(&self, _: &str, _: Option<Uuid>, _: &str) -> Self::Output {
            NatsInjection::UserSimple
        }
    }
}
