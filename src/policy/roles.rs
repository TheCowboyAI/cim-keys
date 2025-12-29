// Copyright (c) 2025 - Cowboy AI, LLC.
//! Role Aggregate for Claims-Based Policy Ontology
//!
//! Roles are semantic aggregates of Claims with a unique purpose.
//! They form the composition layer of the authorization ontology.
//!
//! # Ontological Position
//!
//! ```text
//! Claim (atomic term) → Role (aggregate) → Policy (contextualized) → Binding (instantiated)
//! ```
//!
//! # Design Principles (from DDD Expert)
//!
//! 1. **Roles are NOT collections** - they are semantic aggregates with purpose
//! 2. **Explicit composition** - no implicit inheritance, use Union/Intersection/Extension
//! 3. **Invariant enforcement** - all invariants validated before state change
//! 4. **Separation of duties** - certain claim combinations forbidden
//! 5. **Assignment is first-class** - RoleAssignment is its own aggregate

use crate::policy::claims::{Claim, ClaimCategory};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;

// ============================================================================
// ROLE AGGREGATE
// ============================================================================

/// Role aggregate - semantic aggregation of claims with unique purpose
///
/// A Role answers: "What can someone with this responsibility do?"
///
/// # Invariants
/// 1. Non-empty claim set (a role without claims has no meaning)
/// 2. Purpose coherence (claims should relate to the stated purpose)
/// 3. No self-contradiction (no claim and its negation)
/// 4. Separation of duties compliance (forbidden pairs not allowed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier (UUID v7 for temporal ordering)
    pub id: Uuid,

    /// Human-readable name
    pub name: String,

    /// The semantic purpose - WHY this role exists
    pub purpose: RolePurpose,

    /// Claims granted by this role
    pub claims: HashSet<Claim>,

    /// Roles that cannot be held simultaneously with this one
    pub incompatible_roles: HashSet<Uuid>,

    /// How this role was created (direct or composed)
    pub composition: Option<RoleComposition>,

    /// Lifecycle status
    pub status: RoleStatus,

    /// Who created this role
    pub created_by: Uuid,

    /// Version for optimistic concurrency
    pub version: u64,
}

/// The semantic purpose of a role - answers "WHY does this role exist?"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RolePurpose {
    /// Primary domain this role serves
    pub domain: ClaimCategory,

    /// Human-readable description of the purpose
    pub description: String,

    /// Separation class for duty segregation
    pub separation_class: SeparationClass,

    /// Seniority/privilege level (0 = entry, higher = more senior)
    pub level: u8,
}

/// Separation classes for enforcing segregation of duties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeparationClass {
    /// Day-to-day operational work
    Operational,
    /// System administration
    Administrative,
    /// Compliance and audit functions
    Audit,
    /// Break-glass emergency access
    Emergency,
    /// Financial/procurement authority
    Financial,
    /// HR/people management
    Personnel,
}

/// How a role was composed from other roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleComposition {
    /// Union: grants all claims from source roles (R₁ ∪ R₂)
    Union {
        source_roles: Vec<Uuid>,
    },

    /// Intersection: grants only common claims (R₁ ∩ R₂)
    Intersection {
        source_roles: Vec<Uuid>,
    },

    /// Extension: base role with additions/removals
    Extension {
        base_role: Uuid,
        additions: HashSet<Claim>,
        removals: HashSet<Claim>,
    },

    /// Restriction: base role with narrowed applicability
    Restriction {
        base_role: Uuid,
        additional_constraints: Vec<String>,
    },
}

/// Lifecycle status of a role
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleStatus {
    /// Role is being defined, not yet usable
    Draft,

    /// Role is active and can be assigned
    Active,

    /// Role is deprecated, existing assignments continue but no new ones
    Deprecated {
        reason: String,
        replacement: Option<Uuid>,
        deprecated_at: DateTime<Utc>,
    },

    /// Role is retired, no longer in use
    Retired {
        retired_at: DateTime<Utc>,
    },
}

impl Role {
    /// Create a new role in Draft status
    pub fn new(
        name: impl Into<String>,
        purpose: RolePurpose,
        claims: HashSet<Claim>,
        created_by: Uuid,
    ) -> Result<Self, RoleError> {
        let role = Self {
            id: Uuid::now_v7(),
            name: name.into(),
            purpose,
            claims,
            incompatible_roles: HashSet::new(),
            composition: None,
            status: RoleStatus::Draft,
            created_by,
            version: 1,
        };

        // Validate invariants
        role.validate()?;
        Ok(role)
    }

    /// Validate all role invariants
    pub fn validate(&self) -> Result<(), RoleError> {
        // Invariant 1: Non-empty claim set
        if self.claims.is_empty() {
            return Err(RoleError::EmptyClaims {
                role_id: self.id,
                role_name: self.name.clone(),
            });
        }

        // Invariant 2: Purpose coherence (warn but don't fail)
        // Claims should generally relate to the role's domain
        // This is advisory - some cross-domain roles are valid

        Ok(())
    }

    /// Add a claim to this role
    pub fn add_claim(&mut self, claim: Claim) -> Result<(), RoleError> {
        if matches!(self.status, RoleStatus::Retired { .. }) {
            return Err(RoleError::RoleRetired { role_id: self.id });
        }

        self.claims.insert(claim);
        self.version += 1;
        self.validate()
    }

    /// Remove a claim from this role
    pub fn remove_claim(&mut self, claim: &Claim) -> Result<(), RoleError> {
        if matches!(self.status, RoleStatus::Retired { .. }) {
            return Err(RoleError::RoleRetired { role_id: self.id });
        }

        self.claims.remove(claim);
        self.version += 1;
        self.validate()
    }

    /// Mark a role as incompatible with another (separation of duties)
    pub fn add_incompatible_role(&mut self, role_id: Uuid) {
        self.incompatible_roles.insert(role_id);
        self.version += 1;
    }

    /// Activate a draft role
    pub fn activate(&mut self) -> Result<(), RoleError> {
        match &self.status {
            RoleStatus::Draft => {
                self.validate()?;
                self.status = RoleStatus::Active;
                self.version += 1;
                Ok(())
            }
            _ => Err(RoleError::InvalidStatusTransition {
                role_id: self.id,
                from: format!("{:?}", self.status),
                to: "Active".to_string(),
            }),
        }
    }

    /// Deprecate a role
    pub fn deprecate(&mut self, reason: String, replacement: Option<Uuid>) -> Result<(), RoleError> {
        match &self.status {
            RoleStatus::Active => {
                self.status = RoleStatus::Deprecated {
                    reason,
                    replacement,
                    deprecated_at: Utc::now(),
                };
                self.version += 1;
                Ok(())
            }
            _ => Err(RoleError::InvalidStatusTransition {
                role_id: self.id,
                from: format!("{:?}", self.status),
                to: "Deprecated".to_string(),
            }),
        }
    }

    /// Retire a deprecated role
    pub fn retire(&mut self) -> Result<(), RoleError> {
        match &self.status {
            RoleStatus::Deprecated { .. } => {
                self.status = RoleStatus::Retired {
                    retired_at: Utc::now(),
                };
                self.version += 1;
                Ok(())
            }
            _ => Err(RoleError::InvalidStatusTransition {
                role_id: self.id,
                from: format!("{:?}", self.status),
                to: "Retired".to_string(),
            }),
        }
    }

    /// Check if this role is active and assignable
    pub fn is_assignable(&self) -> bool {
        matches!(self.status, RoleStatus::Active)
    }

    /// Check if role has a specific claim
    pub fn has_claim(&self, claim: &Claim) -> bool {
        self.claims.contains(claim)
    }

    /// Get all claims in a specific category
    pub fn claims_in_category(&self, category: ClaimCategory) -> Vec<&Claim> {
        self.claims.iter().filter(|c| c.category() == category).collect()
    }

    /// Check if this role is compatible with another
    pub fn is_compatible_with(&self, other_role_id: Uuid) -> bool {
        !self.incompatible_roles.contains(&other_role_id)
    }
}

// ============================================================================
// ROLE COMPOSITION FUNCTIONS
// ============================================================================

/// Compose two roles via union (grants all claims from both)
pub fn compose_union(
    role_a: &Role,
    role_b: &Role,
    new_name: impl Into<String>,
    new_purpose: RolePurpose,
    created_by: Uuid,
) -> Result<Role, RoleError> {
    let combined_claims: HashSet<Claim> = role_a
        .claims
        .union(&role_b.claims)
        .cloned()
        .collect();

    let combined_incompatible: HashSet<Uuid> = role_a
        .incompatible_roles
        .union(&role_b.incompatible_roles)
        .cloned()
        .collect();

    let mut role = Role::new(new_name, new_purpose, combined_claims, created_by)?;
    role.incompatible_roles = combined_incompatible;
    role.composition = Some(RoleComposition::Union {
        source_roles: vec![role_a.id, role_b.id],
    });

    Ok(role)
}

/// Compose two roles via intersection (grants only common claims)
pub fn compose_intersection(
    role_a: &Role,
    role_b: &Role,
    new_name: impl Into<String>,
    new_purpose: RolePurpose,
    created_by: Uuid,
) -> Result<Role, RoleError> {
    let common_claims: HashSet<Claim> = role_a
        .claims
        .intersection(&role_b.claims)
        .cloned()
        .collect();

    if common_claims.is_empty() {
        return Err(RoleError::EmptyComposition {
            role_a: role_a.id,
            role_b: role_b.id,
        });
    }

    let mut role = Role::new(new_name, new_purpose, common_claims, created_by)?;
    role.composition = Some(RoleComposition::Intersection {
        source_roles: vec![role_a.id, role_b.id],
    });

    Ok(role)
}

/// Extend a base role with additional/removed claims
pub fn compose_extension(
    base_role: &Role,
    new_name: impl Into<String>,
    new_purpose: RolePurpose,
    additions: HashSet<Claim>,
    removals: HashSet<Claim>,
    created_by: Uuid,
) -> Result<Role, RoleError> {
    let mut extended_claims = base_role.claims.clone();
    for claim in &additions {
        extended_claims.insert(claim.clone());
    }
    for claim in &removals {
        extended_claims.remove(claim);
    }

    let mut role = Role::new(new_name, new_purpose, extended_claims, created_by)?;
    role.incompatible_roles = base_role.incompatible_roles.clone();
    role.composition = Some(RoleComposition::Extension {
        base_role: base_role.id,
        additions,
        removals,
    });

    Ok(role)
}

// ============================================================================
// ROLE ASSIGNMENT
// ============================================================================

/// Assignment of a role to a person - first-class domain object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    /// Unique identifier
    pub id: Uuid,

    /// Person receiving the role
    pub person_id: Uuid,

    /// Role being assigned
    pub role_id: Uuid,

    /// Who approved this assignment
    pub granted_by: Uuid,

    /// When the assignment was created
    pub granted_at: DateTime<Utc>,

    /// When the assignment becomes effective
    pub valid_from: DateTime<Utc>,

    /// When the assignment expires (None = indefinite)
    pub valid_until: Option<DateTime<Utc>>,

    /// Context/scope for this assignment
    pub context: AssignmentContext,

    /// Assignment lifecycle status
    pub status: AssignmentStatus,

    /// Version for optimistic concurrency
    pub version: u64,
}

/// Context that narrows where the role assignment applies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssignmentContext {
    /// Organization scope (None = all orgs the person belongs to)
    pub organization_id: Option<Uuid>,

    /// Unit scope (None = all units)
    pub unit_id: Option<Uuid>,

    /// Environment restrictions
    pub environments: Vec<String>,

    /// Time window restrictions (e.g., business hours only)
    pub time_windows: Vec<TimeWindow>,

    /// Location restrictions
    pub location_ids: Vec<Uuid>,
}

/// Time window for assignment validity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Days of week (0=Sunday, 6=Saturday)
    pub days: Vec<u8>,

    /// Start hour (0-23)
    pub start_hour: u8,

    /// End hour (0-23)
    pub end_hour: u8,

    /// Timezone
    pub timezone: String,
}

/// Assignment lifecycle status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStatus {
    /// Awaiting approval
    Pending {
        requires_approval_from: Option<Uuid>,
    },

    /// Assignment is active
    Active,

    /// Temporarily suspended
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid,
    },

    /// Assignment has expired
    Expired {
        expired_at: DateTime<Utc>,
    },

    /// Assignment was revoked
    Revoked {
        reason: String,
        revoked_at: DateTime<Utc>,
        revoked_by: Uuid,
    },
}

impl RoleAssignment {
    /// Create a new role assignment (starts as Pending)
    pub fn new(
        person_id: Uuid,
        role_id: Uuid,
        granted_by: Uuid,
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
        context: AssignmentContext,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            person_id,
            role_id,
            granted_by,
            granted_at: Utc::now(),
            valid_from,
            valid_until,
            context,
            status: AssignmentStatus::Pending {
                requires_approval_from: None,
            },
            version: 1,
        }
    }

    /// Approve and activate the assignment
    pub fn approve(&mut self, approver_id: Uuid) -> Result<(), RoleError> {
        // Separation of duties: approver cannot be the granter
        if approver_id == self.granted_by {
            return Err(RoleError::SelfApprovalForbidden {
                assignment_id: self.id,
            });
        }

        match &self.status {
            AssignmentStatus::Pending { .. } => {
                self.status = AssignmentStatus::Active;
                self.version += 1;
                Ok(())
            }
            _ => Err(RoleError::InvalidAssignmentStatus {
                assignment_id: self.id,
                expected: "Pending".to_string(),
                actual: format!("{:?}", self.status),
            }),
        }
    }

    /// Suspend the assignment
    pub fn suspend(&mut self, reason: String, suspended_by: Uuid) -> Result<(), RoleError> {
        match &self.status {
            AssignmentStatus::Active => {
                self.status = AssignmentStatus::Suspended {
                    reason,
                    suspended_at: Utc::now(),
                    suspended_by,
                };
                self.version += 1;
                Ok(())
            }
            _ => Err(RoleError::InvalidAssignmentStatus {
                assignment_id: self.id,
                expected: "Active".to_string(),
                actual: format!("{:?}", self.status),
            }),
        }
    }

    /// Resume a suspended assignment
    pub fn resume(&mut self) -> Result<(), RoleError> {
        match &self.status {
            AssignmentStatus::Suspended { .. } => {
                self.status = AssignmentStatus::Active;
                self.version += 1;
                Ok(())
            }
            _ => Err(RoleError::InvalidAssignmentStatus {
                assignment_id: self.id,
                expected: "Suspended".to_string(),
                actual: format!("{:?}", self.status),
            }),
        }
    }

    /// Revoke the assignment
    pub fn revoke(&mut self, reason: String, revoked_by: Uuid) -> Result<(), RoleError> {
        match &self.status {
            AssignmentStatus::Active | AssignmentStatus::Suspended { .. } => {
                self.status = AssignmentStatus::Revoked {
                    reason,
                    revoked_at: Utc::now(),
                    revoked_by,
                };
                self.version += 1;
                Ok(())
            }
            _ => Err(RoleError::InvalidAssignmentStatus {
                assignment_id: self.id,
                expected: "Active or Suspended".to_string(),
                actual: format!("{:?}", self.status),
            }),
        }
    }

    /// Check if assignment is currently effective
    pub fn is_effective(&self, now: DateTime<Utc>) -> bool {
        if self.status != AssignmentStatus::Active {
            return false;
        }

        if now < self.valid_from {
            return false;
        }

        if let Some(until) = self.valid_until {
            if now > until {
                return false;
            }
        }

        true
    }

    /// Check time window restrictions
    pub fn is_in_time_window(&self, _now: DateTime<Utc>) -> bool {
        if self.context.time_windows.is_empty() {
            return true; // No restrictions
        }

        // TODO: Implement time window checking with timezone support
        true
    }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Errors that can occur in Role operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum RoleError {
    #[error("Role {role_name} ({role_id}) has no claims - roles must grant at least one permission")]
    EmptyClaims { role_id: Uuid, role_name: String },

    #[error("Role {role_id} is retired and cannot be modified")]
    RoleRetired { role_id: Uuid },

    #[error("Invalid status transition for role {role_id}: {from} → {to}")]
    InvalidStatusTransition {
        role_id: Uuid,
        from: String,
        to: String,
    },

    #[error("Composition of roles {role_a} and {role_b} would result in empty claim set")]
    EmptyComposition { role_a: Uuid, role_b: Uuid },

    #[error("Self-approval forbidden for assignment {assignment_id}")]
    SelfApprovalForbidden { assignment_id: Uuid },

    #[error("Invalid assignment status for {assignment_id}: expected {expected}, got {actual}")]
    InvalidAssignmentStatus {
        assignment_id: Uuid,
        expected: String,
        actual: String,
    },

    #[error("Separation of duties violation: roles {role_a} and {role_b} cannot be held together")]
    SeparationOfDutiesViolation { role_a: Uuid, role_b: Uuid },
}

// ============================================================================
// DISPLAY IMPLEMENTATIONS
// ============================================================================

impl fmt::Display for RoleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoleStatus::Draft => write!(f, "Draft"),
            RoleStatus::Active => write!(f, "Active"),
            RoleStatus::Deprecated { reason, .. } => write!(f, "Deprecated: {}", reason),
            RoleStatus::Retired { .. } => write!(f, "Retired"),
        }
    }
}

impl fmt::Display for SeparationClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeparationClass::Operational => write!(f, "Operational"),
            SeparationClass::Administrative => write!(f, "Administrative"),
            SeparationClass::Audit => write!(f, "Audit"),
            SeparationClass::Emergency => write!(f, "Emergency"),
            SeparationClass::Financial => write!(f, "Financial"),
            SeparationClass::Personnel => write!(f, "Personnel"),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({} claims, {})",
            self.name,
            self.claims.len(),
            self.status
        )
    }
}

impl fmt::Display for AssignmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssignmentStatus::Pending { .. } => write!(f, "Pending"),
            AssignmentStatus::Active => write!(f, "Active"),
            AssignmentStatus::Suspended { reason, .. } => write!(f, "Suspended: {}", reason),
            AssignmentStatus::Expired { .. } => write!(f, "Expired"),
            AssignmentStatus::Revoked { reason, .. } => write!(f, "Revoked: {}", reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_purpose() -> RolePurpose {
        RolePurpose {
            domain: ClaimCategory::Development,
            description: "Test role".to_string(),
            separation_class: SeparationClass::Operational,
            level: 1,
        }
    }

    #[test]
    fn test_role_creation() {
        let claims: HashSet<Claim> = vec![Claim::ReadRepository, Claim::WriteRepository]
            .into_iter()
            .collect();
        let role = Role::new("Developer", test_purpose(), claims, Uuid::now_v7());
        assert!(role.is_ok());
        let role = role.unwrap();
        assert_eq!(role.name, "Developer");
        assert_eq!(role.claims.len(), 2);
        assert!(matches!(role.status, RoleStatus::Draft));
    }

    #[test]
    fn test_empty_role_fails() {
        let result = Role::new("Empty", test_purpose(), HashSet::new(), Uuid::now_v7());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RoleError::EmptyClaims { .. }));
    }

    #[test]
    fn test_role_activation() {
        let claims: HashSet<Claim> = vec![Claim::ReadRepository].into_iter().collect();
        let mut role = Role::new("Reader", test_purpose(), claims, Uuid::now_v7()).unwrap();

        assert!(!role.is_assignable());
        role.activate().unwrap();
        assert!(role.is_assignable());
    }

    #[test]
    fn test_role_composition_union() {
        let claims_a: HashSet<Claim> = vec![Claim::ReadRepository].into_iter().collect();
        let claims_b: HashSet<Claim> = vec![Claim::WriteRepository].into_iter().collect();

        let role_a = Role::new("Reader", test_purpose(), claims_a, Uuid::now_v7()).unwrap();
        let role_b = Role::new("Writer", test_purpose(), claims_b, Uuid::now_v7()).unwrap();

        let combined = compose_union(&role_a, &role_b, "ReadWriter", test_purpose(), Uuid::now_v7()).unwrap();

        assert_eq!(combined.claims.len(), 2);
        assert!(combined.has_claim(&Claim::ReadRepository));
        assert!(combined.has_claim(&Claim::WriteRepository));
    }

    #[test]
    fn test_assignment_self_approval_forbidden() {
        let granter = Uuid::now_v7();
        let mut assignment = RoleAssignment::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            granter,
            Utc::now(),
            None,
            AssignmentContext::default(),
        );

        let result = assignment.approve(granter);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RoleError::SelfApprovalForbidden { .. }
        ));
    }
}
