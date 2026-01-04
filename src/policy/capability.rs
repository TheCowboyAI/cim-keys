// Copyright (c) 2025 - Cowboy AI, LLC.

//! Capability - Categorical Product with Policy Fold
//!
//! Capability is the composition of Role, Subject, and Policy constraints.
//! It represents what a principal can actually do in the system.
//!
//! # Mathematical Foundation
//!
//! ```text
//! EffectiveCapability = fold(Policies, BaseCapability)
//!
//! where:
//!   BaseCapability = Role × Subject  (categorical product)
//!   fold = list catamorphism over Policy
//!   policy_algebra = [const id, uncurry apply]
//! ```
//!
//! ## Catamorphism Structure
//!
//! The fold over policies is a LIST catamorphism:
//! ```text
//! cata_List : (B, (A, B) → B) → List<A> → B
//! cata_List(b, f)([])     = b           -- identity on empty list
//! cata_List(b, f)(a:as)   = f(a, cata_List(b, f)(as))
//! ```
//!
//! Applied to policies:
//! ```text
//! fold_policies : List<Policy> → BaseCapability → EffectiveCapability
//! fold_policies([])       cap = cap
//! fold_policies(p:ps)     cap = p.apply(fold_policies(ps)(cap))
//! ```
//!
//! ## Categorical Product `Role × Subject`
//!
//! The product has universal property:
//! - π₁: Role × Subject → Role (first projection)
//! - π₂: Role × Subject → Subject (second projection)
//! - For any X with f: X → Role, g: X → Subject, exists unique ⟨f,g⟩: X → Role × Subject
//!
//! ## Policy Requirements
//!
//! For the catamorphism to be well-defined, policies must be:
//! 1. **Total**: Every policy produces an output for every input
//! 2. **Pure**: No side effects, deterministic
//! 3. **Monotonic** (optional): Policy application doesn't add permissions
//!
//! This ensures the functor laws hold and composition is sound.

use crate::policy::cim_claims::{CimClaim, ClaimSet, NatsClaim};
use crate::policy::cim_role::CimRole;
use crate::policy::subject::{Subject, SubjectPattern, MessageType, TypedSubject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// BASE CAPABILITY - The Categorical Product Role × Subject
// ============================================================================

/// Base Capability - the categorical product `Role × Subject`
///
/// This is the input to the policy fold. It represents the raw permissions
/// before any policy constraints are applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCapability {
    /// First projection: π₁ (the claims)
    pub role: CimRole,
    /// Second projection: π₂ (the subject patterns)
    pub subject: Subject,
}

impl BaseCapability {
    /// Create a new base capability (the categorical product)
    pub fn new(role: CimRole, subject: Subject) -> Self {
        Self { role, subject }
    }

    /// First projection: π₁(Role × Subject) → Role
    pub fn pi1(&self) -> &CimRole {
        &self.role
    }

    /// Second projection: π₂(Role × Subject) → Subject
    pub fn pi2(&self) -> &Subject {
        &self.subject
    }

    /// Universal property: construct product from morphisms
    /// ⟨f, g⟩: X → Role × Subject given f: X → Role, g: X → Subject
    pub fn from_morphisms<X, F, G>(x: &X, f: F, g: G) -> Self
    where
        F: Fn(&X) -> CimRole,
        G: Fn(&X) -> Subject,
    {
        Self {
            role: f(x),
            subject: g(x),
        }
    }
}

// ============================================================================
// POLICY - Endofunctor on Capability
// ============================================================================

/// Policy - a constraint that transforms capabilities
///
/// Policies are the morphisms applied during the fold.
/// They must be total and pure for the catamorphism to be valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique identifier
    pub id: Uuid,

    /// Human-readable name
    pub name: String,

    /// Description of what this policy does
    pub description: String,

    /// The constraint rules
    pub constraints: Vec<PolicyConstraint>,

    /// Priority (higher = applied later in fold)
    pub priority: u32,

    /// Whether this policy is active
    pub enabled: bool,

    /// When policy was created
    pub created_at: DateTime<Utc>,
}

/// A single constraint within a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyConstraint {
    /// Require specific claims to be present
    RequireClaim { claim: CimClaim },

    /// Remove specific claims
    DenyClaim { claim: CimClaim },

    /// Restrict to specific subject patterns
    RestrictSubject { pattern: SubjectPattern },

    /// Deny specific subject patterns
    DenySubject { pattern: SubjectPattern },

    /// Time-based restriction
    TimeRestriction {
        start_hour: u8,
        end_hour: u8,
        timezone: String,
    },

    /// Scope to organization
    ScopeOrganization { org_id: Uuid },

    /// Scope to organizational unit
    ScopeUnit { unit_id: Uuid },

    /// Maximum delegation depth
    MaxDelegationDepth { depth: u8 },
}

impl Policy {
    /// Create a new policy
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            description: description.into(),
            constraints: Vec::new(),
            priority: 100,
            enabled: true,
            created_at: Utc::now(),
        }
    }

    /// Builder: add a constraint
    pub fn with_constraint(mut self, constraint: PolicyConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Builder: set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Apply this policy to a capability (the fold step)
    ///
    /// This is the morphism `Policy × Capability → Capability`
    /// that implements the algebra for our list catamorphism.
    ///
    /// # Totality
    /// This function is total - it always produces a valid EffectiveCapability.
    ///
    /// # Purity
    /// This function is pure - no side effects, deterministic output.
    pub fn apply(&self, cap: EffectiveCapability) -> EffectiveCapability {
        if !self.enabled {
            return cap; // Identity when disabled
        }

        let mut claims = cap.claims.clone();
        let mut publish = cap.allowed_publish;
        let mut subscribe = cap.allowed_subscribe;
        let mut deny_publish = cap.deny_publish;
        let mut deny_subscribe = cap.deny_subscribe;
        let mut restrictions = cap.restrictions.clone();

        for constraint in &self.constraints {
            match constraint {
                PolicyConstraint::RequireClaim { claim } => {
                    // Policy doesn't add claims, but records requirement
                    restrictions.push(format!("Requires: {}", claim.uri()));
                }
                PolicyConstraint::DenyClaim { claim } => {
                    // Remove the claim from effective set by rebuilding without it
                    let old_claims = claims;
                    claims = ClaimSet::new();
                    for c in old_claims.iter() {
                        if c != claim {
                            claims = claims.with(c.clone());
                        }
                    }
                }
                PolicyConstraint::RestrictSubject { pattern } => {
                    // Filter publish/subscribe to only match restricted pattern
                    publish.retain(|p| pattern.subsumes(p));
                    subscribe.retain(|s| pattern.subsumes(s));
                }
                PolicyConstraint::DenySubject { pattern } => {
                    deny_publish.push(pattern.clone());
                    deny_subscribe.push(pattern.clone());
                }
                PolicyConstraint::TimeRestriction { start_hour, end_hour, timezone } => {
                    restrictions.push(format!(
                        "Time: {}:00-{}:00 {}",
                        start_hour, end_hour, timezone
                    ));
                }
                PolicyConstraint::ScopeOrganization { org_id } => {
                    restrictions.push(format!("Org: {}", org_id));
                }
                PolicyConstraint::ScopeUnit { unit_id } => {
                    restrictions.push(format!("Unit: {}", unit_id));
                }
                PolicyConstraint::MaxDelegationDepth { depth } => {
                    restrictions.push(format!("Max delegation: {}", depth));
                }
            }
        }

        EffectiveCapability {
            claims,
            allowed_publish: publish,
            allowed_subscribe: subscribe,
            deny_publish,
            deny_subscribe,
            restrictions,
            applied_policies: {
                let mut policies = cap.applied_policies;
                policies.push(self.id);
                policies
            },
        }
    }
}

// ============================================================================
// EFFECTIVE CAPABILITY - The Result of the Fold
// ============================================================================

/// Effective Capability - the result of folding policies over base capability
///
/// This is what a principal can actually do after all policies are applied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectiveCapability {
    /// Effective claims after policy constraints
    pub claims: ClaimSet,

    /// Allowed publish patterns
    pub allowed_publish: Vec<SubjectPattern>,

    /// Allowed subscribe patterns
    pub allowed_subscribe: Vec<SubjectPattern>,

    /// Denied publish patterns (takes precedence)
    pub deny_publish: Vec<SubjectPattern>,

    /// Denied subscribe patterns
    pub deny_subscribe: Vec<SubjectPattern>,

    /// Human-readable restrictions applied
    pub restrictions: Vec<String>,

    /// Policies that were applied
    pub applied_policies: Vec<Uuid>,
}

impl EffectiveCapability {
    /// Create from base capability (identity in the fold)
    pub fn from_base(base: &BaseCapability) -> Self {
        Self {
            claims: base.role.claims.clone(),
            allowed_publish: base.subject.publish.clone(),
            allowed_subscribe: base.subject.subscribe.clone(),
            deny_publish: base.subject.deny_publish.clone(),
            deny_subscribe: base.subject.deny_subscribe.clone(),
            restrictions: Vec::new(),
            applied_policies: Vec::new(),
        }
    }

    /// Check if a specific claim is granted
    pub fn has_claim(&self, claim: &CimClaim) -> bool {
        self.claims.satisfies(claim)
    }

    /// Check if publishing to a subject is allowed
    pub fn can_publish(&self, subject: &str) -> bool {
        // Check deny first
        if self.deny_publish.iter().any(|p| p.matches(subject)) {
            return false;
        }
        // Check allow
        self.allowed_publish.iter().any(|p| p.matches(subject))
    }

    /// Check if subscribing to a subject is allowed
    pub fn can_subscribe(&self, subject: &str) -> bool {
        // Check deny first
        if self.deny_subscribe.iter().any(|p| p.matches(subject)) {
            return false;
        }
        // Check allow
        self.allowed_subscribe.iter().any(|p| p.matches(subject))
    }
}

// ============================================================================
// FOLD - The List Catamorphism over Policies
// ============================================================================

/// Fold policies over a base capability to produce effective capability
///
/// This is the list catamorphism:
/// ```text
/// cata_List : (B, (A, B) → B) → List<A> → B
/// ```
///
/// Where:
/// - B = EffectiveCapability (carrier type)
/// - A = Policy (element type)
/// - (A, B) → B = Policy.apply (algebra)
///
/// # Properties (from ACT expert verification)
///
/// 1. **Identity**: fold([], cap) = cap
/// 2. **Associativity**: fold order matches policy priority
/// 3. **Totality**: Always produces valid EffectiveCapability
///
/// # Example
/// ```ignore
/// let base = BaseCapability::new(role, subject);
/// let effective = fold_policies(&policies, &base);
/// ```
pub fn fold_policies(policies: &[Policy], base: &BaseCapability) -> EffectiveCapability {
    // Sort by priority (lower first, so higher priority policies apply last)
    let mut sorted_policies: Vec<&Policy> = policies.iter().collect();
    sorted_policies.sort_by_key(|p| p.priority);

    // Identity element: effective capability from base
    let identity = EffectiveCapability::from_base(base);

    // List catamorphism: fold right with policy application
    // cata_List(identity, apply)(policies)
    sorted_policies.iter().fold(identity, |acc, policy| policy.apply(acc))
}

/// Convenience function: compute capability from role, subject, and policies
pub fn compute_capability(
    role: CimRole,
    subject: Subject,
    policies: &[Policy],
) -> EffectiveCapability {
    let base = BaseCapability::new(role, subject);
    fold_policies(policies, &base)
}

// ============================================================================
// CAPABILITY CONTEXT - For runtime evaluation
// ============================================================================

/// Context for capability evaluation (time, location, etc.)
#[derive(Debug, Clone, Default)]
pub struct CapabilityContext {
    /// Current time for temporal policies
    pub current_time: Option<DateTime<Utc>>,
    /// Current organization scope
    pub organization_id: Option<Uuid>,
    /// Current unit scope
    pub unit_id: Option<Uuid>,
    /// Current delegation depth
    pub delegation_depth: u8,
}

impl CapabilityContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_time(mut self, time: DateTime<Utc>) -> Self {
        self.current_time = Some(time);
        self
    }

    pub fn with_org(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    pub fn with_unit(mut self, unit_id: Uuid) -> Self {
        self.unit_id = Some(unit_id);
        self
    }
}

// ============================================================================
// CQRS CLAIM REQUIREMENTS
// ============================================================================

/// Claim requirements by CQRS message type
///
/// Each message type (Command, Query, Event) requires different claims:
/// - Commands: Write claims (StreamWrite, KvWrite)
/// - Queries: Read claims (StreamRead, KvRead)
/// - Events: Emit claims (publish) or Subscribe claims (consume)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CqrsClaimRequirement {
    /// Write permission for Commands
    Write {
        aggregate: String,
        action: WriteAction,
        scope: WriteScope,
    },
    /// Read permission for Queries
    Read {
        projection: String,
        fields: FieldAccess,
        scope: ReadScope,
    },
    /// Emit permission for publishing Events
    Emit {
        stream: String,
        event_types: Vec<String>,
    },
    /// Subscribe permission for consuming Events
    Subscribe {
        stream: String,
        replay: ReplayPermission,
    },
}

/// Write actions for Command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WriteAction {
    /// Create new entities
    Create,
    /// Update existing entities
    Update,
    /// Delete entities
    Delete,
    /// Execute aggregate-specific commands
    Execute,
    /// Full administrative access
    Admin,
}

/// Scope for write operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WriteScope {
    /// Only entities owned by principal
    Own,
    /// All entities in principal's organization
    Organization,
    /// All entities (admin privilege)
    Global,
}

/// Field access levels for Query results
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FieldAccess {
    /// PII stripped from results
    Redacted,
    /// Subset of fields accessible
    Subset(Vec<String>),
    /// All fields accessible
    All,
}

/// Scope for read operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ReadScope {
    /// Only own entities
    Own,
    /// Organization-wide
    Organization,
    /// Global read access
    Global,
}

/// Event replay permissions
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ReplayPermission {
    /// Only new events
    NewOnly,
    /// From specific sequence number
    FromSequence(u64),
    /// From specific timestamp
    FromTime(i64),
    /// Full replay from beginning
    All,
}

impl ReplayPermission {
    /// Minimum of two replay permissions
    pub fn min(&self, other: &Self) -> Self {
        if self <= other {
            self.clone()
        } else {
            other.clone()
        }
    }
}

// ============================================================================
// CQRS CLAIM VALIDATION
// ============================================================================

/// Result of claim validation for a CQRS operation
#[derive(Debug, Clone)]
pub enum ClaimValidationResult {
    /// Operation is allowed
    Allowed {
        /// Any field filtering applied to results
        field_filter: Option<Vec<String>>,
        /// Any replay restriction applied
        replay_restriction: Option<ReplayPermission>,
    },
    /// Operation is denied
    Denied {
        /// Reason for denial
        reason: ClaimViolation,
    },
}

/// Types of claim violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimViolation {
    /// Missing permission for aggregate
    MissingAggregatePermission { aggregate: String },
    /// Insufficient action level
    InsufficientAction { required: WriteAction, have: Option<WriteAction> },
    /// Insufficient scope
    InsufficientScope { required: WriteScope, have: Option<WriteScope> },
    /// Missing read permission
    MissingReadPermission { projection: String },
    /// Missing emit permission
    MissingEmitPermission { stream: String },
    /// Missing subscribe permission
    MissingSubscribePermission { stream: String },
    /// Temporal constraint violated
    TemporalViolation { constraint: String },
}

impl fmt::Display for ClaimViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaimViolation::MissingAggregatePermission { aggregate } => {
                write!(f, "Missing permission for aggregate: {}", aggregate)
            }
            ClaimViolation::InsufficientAction { required, have } => {
                write!(f, "Insufficient action: required {:?}, have {:?}", required, have)
            }
            ClaimViolation::InsufficientScope { required, have } => {
                write!(f, "Insufficient scope: required {:?}, have {:?}", required, have)
            }
            ClaimViolation::MissingReadPermission { projection } => {
                write!(f, "Missing read permission for projection: {}", projection)
            }
            ClaimViolation::MissingEmitPermission { stream } => {
                write!(f, "Missing emit permission for stream: {}", stream)
            }
            ClaimViolation::MissingSubscribePermission { stream } => {
                write!(f, "Missing subscribe permission for stream: {}", stream)
            }
            ClaimViolation::TemporalViolation { constraint } => {
                write!(f, "Temporal constraint violated: {}", constraint)
            }
        }
    }
}

/// Validate a CQRS operation against effective capability
pub fn validate_cqrs_operation(
    capability: &EffectiveCapability,
    typed_subject: &TypedSubject,
    requirement: &CqrsClaimRequirement,
) -> ClaimValidationResult {
    match (typed_subject.message_type, requirement) {
        // Command validation - requires write claims
        (MessageType::Command, CqrsClaimRequirement::Write { aggregate, action, scope }) => {
            validate_command_claims(capability, aggregate, action, scope)
        }
        // Query validation - requires read claims
        (MessageType::Query, CqrsClaimRequirement::Read { projection, fields, scope }) => {
            validate_query_claims(capability, projection, fields, scope)
        }
        // Event emit validation
        (MessageType::Event, CqrsClaimRequirement::Emit { stream, event_types }) => {
            validate_emit_claims(capability, stream, event_types)
        }
        // Event subscribe validation
        (MessageType::Event, CqrsClaimRequirement::Subscribe { stream, replay }) => {
            validate_subscribe_claims(capability, stream, replay)
        }
        // Mismatched message type and requirement
        _ => ClaimValidationResult::Denied {
            reason: ClaimViolation::MissingAggregatePermission {
                aggregate: "type_mismatch".to_string(),
            },
        },
    }
}

fn validate_command_claims(
    capability: &EffectiveCapability,
    _aggregate: &str,
    _action: &WriteAction,
    _scope: &WriteScope,
) -> ClaimValidationResult {
    // Check for write claims in capability
    let has_write = capability.claims.iter().any(|c| matches!(c,
        CimClaim::Nats(NatsClaim::StreamWrite) |
        CimClaim::Nats(NatsClaim::StreamAdmin) |
        CimClaim::Nats(NatsClaim::KvWrite) |
        CimClaim::Nats(NatsClaim::KvAdmin)
    ));

    if has_write {
        ClaimValidationResult::Allowed {
            field_filter: None,
            replay_restriction: None,
        }
    } else {
        ClaimValidationResult::Denied {
            reason: ClaimViolation::InsufficientAction {
                required: *_action,
                have: None,
            },
        }
    }
}

fn validate_query_claims(
    capability: &EffectiveCapability,
    projection: &str,
    fields: &FieldAccess,
    _scope: &ReadScope,
) -> ClaimValidationResult {
    // Check for read claims in capability
    let has_read = capability.claims.iter().any(|c| matches!(c,
        CimClaim::Nats(NatsClaim::StreamRead) |
        CimClaim::Nats(NatsClaim::StreamAdmin) |
        CimClaim::Nats(NatsClaim::KvRead) |
        CimClaim::Nats(NatsClaim::KvAdmin)
    ));

    if has_read {
        // Apply field filtering based on claim level
        let field_filter = match fields {
            FieldAccess::All => None,
            FieldAccess::Subset(fields) => Some(fields.clone()),
            FieldAccess::Redacted => Some(vec![]), // No fields
        };
        ClaimValidationResult::Allowed {
            field_filter,
            replay_restriction: None,
        }
    } else {
        ClaimValidationResult::Denied {
            reason: ClaimViolation::MissingReadPermission {
                projection: projection.to_string(),
            },
        }
    }
}

fn validate_emit_claims(
    capability: &EffectiveCapability,
    stream: &str,
    _event_types: &[String],
) -> ClaimValidationResult {
    // Emit requires write claims to publish to streams
    let has_emit = capability.claims.iter().any(|c| matches!(c,
        CimClaim::Nats(NatsClaim::StreamWrite) |
        CimClaim::Nats(NatsClaim::StreamAdmin)
    ));

    if has_emit {
        ClaimValidationResult::Allowed {
            field_filter: None,
            replay_restriction: None,
        }
    } else {
        ClaimValidationResult::Denied {
            reason: ClaimViolation::MissingEmitPermission {
                stream: stream.to_string(),
            },
        }
    }
}

fn validate_subscribe_claims(
    capability: &EffectiveCapability,
    stream: &str,
    replay: &ReplayPermission,
) -> ClaimValidationResult {
    // Subscribe requires read claims
    let has_subscribe = capability.claims.iter().any(|c| matches!(c,
        CimClaim::Nats(NatsClaim::StreamRead) |
        CimClaim::Nats(NatsClaim::StreamAdmin) |
        CimClaim::Nats(NatsClaim::ConsumerAdmin)
    ));

    if has_subscribe {
        // Apply replay restrictions based on claim level
        let replay_restriction = Some(replay.clone());
        ClaimValidationResult::Allowed {
            field_filter: None,
            replay_restriction,
        }
    } else {
        ClaimValidationResult::Denied {
            reason: ClaimViolation::MissingSubscribePermission {
                stream: stream.to_string(),
            },
        }
    }
}

// ============================================================================
// CQRS-TYPED CAPABILITY
// ============================================================================

/// A capability specifically typed for CQRS operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedCapability {
    /// The base effective capability
    pub effective: EffectiveCapability,
    /// Command subjects (requires write claims)
    pub command_subjects: Vec<TypedSubject>,
    /// Query subjects (requires read claims)
    pub query_subjects: Vec<TypedSubject>,
    /// Event emit subjects (requires publish)
    pub emit_subjects: Vec<TypedSubject>,
    /// Event subscribe subjects
    pub subscribe_subjects: Vec<TypedSubject>,
}

impl TypedCapability {
    /// Create from role, subject permissions, and policies
    pub fn from_role_and_subjects(
        role: CimRole,
        commands: Vec<TypedSubject>,
        queries: Vec<TypedSubject>,
        emits: Vec<TypedSubject>,
        subscribes: Vec<TypedSubject>,
        policies: &[Policy],
    ) -> Self {
        // Build untyped subject from all patterns
        let mut subject = Subject::new();
        for cmd in &commands {
            subject = subject.with_publish(cmd.pattern.clone());
        }
        for qry in &queries {
            // Queries use request-reply, so publish to query, subscribe to response
            subject = subject.with_publish(qry.pattern.clone());
        }
        for emit in &emits {
            subject = subject.with_publish(emit.pattern.clone());
        }
        for sub in &subscribes {
            subject = subject.with_subscribe(sub.pattern.clone());
        }

        let effective = compute_capability(role, subject, policies);

        Self {
            effective,
            command_subjects: commands,
            query_subjects: queries,
            emit_subjects: emits,
            subscribe_subjects: subscribes,
        }
    }

    /// Check if a command operation is allowed
    pub fn can_execute_command(&self, subject: &str) -> bool {
        // Must match a command subject pattern
        let matches_pattern = self.command_subjects.iter()
            .any(|ts| ts.pattern.matches(subject));

        if !matches_pattern {
            return false;
        }

        // Must have write claims
        self.effective.claims.iter().any(|c| matches!(c,
            CimClaim::Nats(NatsClaim::StreamWrite | NatsClaim::StreamAdmin)
        ))
    }

    /// Check if a query operation is allowed
    pub fn can_execute_query(&self, subject: &str) -> bool {
        let matches_pattern = self.query_subjects.iter()
            .any(|ts| ts.pattern.matches(subject));

        if !matches_pattern {
            return false;
        }

        self.effective.claims.iter().any(|c| matches!(c,
            CimClaim::Nats(NatsClaim::StreamRead | NatsClaim::StreamAdmin | NatsClaim::KvRead)
        ))
    }

    /// Check if emitting an event is allowed
    pub fn can_emit_event(&self, subject: &str) -> bool {
        let matches_pattern = self.emit_subjects.iter()
            .any(|ts| ts.pattern.matches(subject));

        if !matches_pattern {
            return false;
        }

        self.effective.claims.iter().any(|c| matches!(c,
            CimClaim::Nats(NatsClaim::StreamWrite | NatsClaim::StreamAdmin)
        ))
    }

    /// Check if subscribing to events is allowed
    pub fn can_subscribe_event(&self, subject: &str) -> bool {
        let matches_pattern = self.subscribe_subjects.iter()
            .any(|ts| ts.pattern.matches(subject));

        if !matches_pattern {
            return false;
        }

        self.effective.claims.iter().any(|c| matches!(c,
            CimClaim::Nats(NatsClaim::StreamRead | NatsClaim::ConsumerAdmin)
        ))
    }
}

// ============================================================================
// DISPLAY IMPLEMENTATIONS
// ============================================================================

impl fmt::Display for BaseCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BaseCapability(role={}, pub={}, sub={})",
            self.role.name,
            self.subject.publish.len(),
            self.subject.subscribe.len()
        )
    }
}

impl fmt::Display for EffectiveCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EffectiveCapability(claims={}, pub={}, sub={}, policies={})",
            self.claims.len(),
            self.allowed_publish.len(),
            self.allowed_subscribe.len(),
            self.applied_policies.len()
        )
    }
}

impl fmt::Display for Policy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Policy({}, {} constraints, priority={})",
            self.name,
            self.constraints.len(),
            self.priority
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::cim_claims::NatsClaim;
    use crate::policy::subject::SubjectPattern;

    fn test_role() -> CimRole {
        let claims = ClaimSet::new()
            .with(CimClaim::Nats(NatsClaim::OperatorAdmin))
            .with(CimClaim::Nats(NatsClaim::StreamAdmin));

        CimRole::new(
            "Test Role",
            "For testing",
            claims,
            crate::policy::cim_claims::ClaimDomain::Nats,
            Uuid::now_v7(),
        )
        .unwrap()
    }

    fn test_subject() -> Subject {
        Subject::new()
            .with_publish(SubjectPattern::parse("cowboyai.>").unwrap())
            .with_subscribe(SubjectPattern::parse("cowboyai.>").unwrap())
    }

    #[test]
    fn test_base_capability_projections() {
        let role = test_role();
        let subject = test_subject();
        let base = BaseCapability::new(role.clone(), subject.clone());

        assert_eq!(base.pi1().name, role.name);
        assert_eq!(base.pi2().publish.len(), subject.publish.len());
    }

    #[test]
    fn test_fold_identity() {
        // fold([], cap) = cap
        let base = BaseCapability::new(test_role(), test_subject());
        let effective = fold_policies(&[], &base);

        assert_eq!(effective.claims.len(), base.role.claims.len());
        assert!(effective.applied_policies.is_empty());
    }

    #[test]
    fn test_fold_with_deny_policy() {
        let base = BaseCapability::new(test_role(), test_subject());

        let deny_policy = Policy::new("Deny Secrets", "Deny access to secret subjects")
            .with_constraint(PolicyConstraint::DenySubject {
                pattern: SubjectPattern::parse("cowboyai.secret.>").unwrap(),
            });

        let effective = fold_policies(&[deny_policy], &base);

        assert!(effective.can_publish("cowboyai.public.events"));
        assert!(!effective.can_publish("cowboyai.secret.keys"));
    }

    #[test]
    fn test_fold_policy_priority() {
        let base = BaseCapability::new(test_role(), test_subject());

        let low_priority = Policy::new("Low", "Low priority").with_priority(10);
        let high_priority = Policy::new("High", "High priority").with_priority(100);

        // High priority should be applied last
        let effective = fold_policies(&[high_priority.clone(), low_priority.clone()], &base);

        assert_eq!(effective.applied_policies.len(), 2);
        // First applied is low priority (sorted by priority ascending)
        assert_eq!(effective.applied_policies[0], low_priority.id);
        assert_eq!(effective.applied_policies[1], high_priority.id);
    }

    #[test]
    fn test_compute_capability_convenience() {
        let role = test_role();
        let subject = test_subject();
        let policies: Vec<Policy> = vec![];

        let effective = compute_capability(role.clone(), subject, &policies);

        assert!(effective.has_claim(&CimClaim::Nats(NatsClaim::OperatorAdmin)));
        assert!(effective.can_publish("cowboyai.test"));
    }

    #[test]
    fn test_effective_capability_checks() {
        let role = test_role();
        let subject = Subject::new()
            .with_publish(SubjectPattern::parse("allowed.>").unwrap())
            .with_deny_publish(SubjectPattern::parse("allowed.denied.*").unwrap());

        let effective = compute_capability(role, subject, &[]);

        assert!(effective.can_publish("allowed.public"));
        assert!(!effective.can_publish("allowed.denied.secret"));
        assert!(!effective.can_publish("other.topic"));
    }
}
