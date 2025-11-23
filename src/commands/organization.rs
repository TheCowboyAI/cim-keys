//! Organizational Domain Commands
//!
//! Commands for creating and managing organizational entities:
//! - Organizations, OrganizationalUnits
//! - People, Locations
//! - Roles, Policies
//! - Relationships between entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::policy_types::{
    KeyDelegation,
    PolicyClaim, PolicyCondition,
};
use crate::events::DomainEvent;

// ============================================================================
// Entity Creation Commands
// ============================================================================

/// Command to create a new person in the organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerson {
    pub command_id: Uuid,
    pub person_id: Uuid,
    pub name: String,
    pub email: String,
    pub title: Option<String>,
    pub department: Option<String>,
    pub organization_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

/// Command to create a new location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLocation {
    pub command_id: Uuid,
    pub location_id: Uuid,
    pub name: String,
    pub location_type: String, // "Physical", "Virtual", "Vault", "Cloud"
    pub address: Option<String>,
    pub coordinates: Option<(f64, f64)>,
    pub organization_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

/// Command to create a new organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganization {
    pub command_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

/// Command to create an organizational unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganizationalUnit {
    pub command_id: Uuid,
    pub unit_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>, // Organization or parent unit
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

/// Command to create a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRole {
    pub command_id: Uuid,
    pub role_id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

/// Command to create a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicy {
    pub command_id: Uuid,
    pub policy_id: Uuid,
    pub name: String,
    pub description: String,
    pub claims: Vec<PolicyClaim>,
    pub conditions: Vec<PolicyCondition>,
    pub organization_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Relationship Commands
// ============================================================================

/// Command to establish a relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstablishRelationship {
    pub command_id: Uuid,
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub relationship_type: RelationshipType,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

/// Types of relationships between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Key delegation from one person to another
    KeyDelegation(KeyDelegation),
    /// Storage location for keys
    StoredAt,
    /// Role assignment
    HasRole,
    /// Policy governance
    PolicyGovernsEntity,
    /// Policy requirement for role
    RoleRequiresPolicy,
    /// Organizational hierarchy
    ParentChild,
    /// Membership in organizational unit
    MemberOf,
    /// Management of organizational unit
    ManagesUnit,
    /// Trust relationship between organizations
    Trusts,
    /// YubiKey ownership
    OwnsYubiKey,
    /// YubiKey assignment
    AssignedTo,
}

// ============================================================================
// Command Handlers
// ============================================================================

/// Handle CreatePerson command
pub async fn handle_create_person(
    cmd: CreatePerson,
) -> Result<Vec<DomainEvent>, crate::aggregate::KeyManagementError> {
    // Validate command
    if cmd.name.is_empty() {
        return Err(crate::aggregate::KeyManagementError::InvalidCommand(
            "Person name cannot be empty".to_string(),
        ));
    }

    if cmd.email.is_empty() || !cmd.email.contains('@') {
        return Err(crate::aggregate::KeyManagementError::InvalidCommand(
            "Valid email address required".to_string(),
        ));
    }

    // Emit PersonCreated event
    let event = DomainEvent::Person(crate::events::PersonEvents::PersonCreated(crate::events::person::PersonCreatedEvent {
        person_id: cmd.person_id,
        name: cmd.name,
        email: Some(cmd.email),
        title: cmd.title,
        department: cmd.department,
        organization_id: cmd.organization_id.unwrap_or_else(|| Uuid::now_v7()),
        created_at: cmd.timestamp,
        created_by: Some("system".to_string()),
        correlation_id: cmd.correlation_id,
        causation_id: Some(cmd.command_id),
    }));

    Ok(vec![event])
}

/// Handle CreateLocation command
pub async fn handle_create_location(
    cmd: CreateLocation,
) -> Result<Vec<DomainEvent>, crate::aggregate::KeyManagementError> {
    // Validate command
    if cmd.name.is_empty() {
        return Err(crate::aggregate::KeyManagementError::InvalidCommand(
            "Location name cannot be empty".to_string(),
        ));
    }

    // Emit LocationCreated event
    let event = DomainEvent::Location(crate::events::LocationEvents::LocationCreated(crate::events::location::LocationCreatedEvent {
        location_id: cmd.location_id,
        name: cmd.name,
        location_type: cmd.location_type,
        address: cmd.address,
        coordinates: cmd.coordinates,
        organization_id: cmd.organization_id,
        created_at: cmd.timestamp,
        created_by: "system".to_string(),
        correlation_id: cmd.correlation_id,
        causation_id: Some(cmd.command_id),
    }));

    Ok(vec![event])
}

/// Handle CreateOrganization command
pub async fn handle_create_organization(
    cmd: CreateOrganization,
) -> Result<Vec<DomainEvent>, crate::aggregate::KeyManagementError> {
    // Validate command
    if cmd.name.is_empty() {
        return Err(crate::aggregate::KeyManagementError::InvalidCommand(
            "Organization name cannot be empty".to_string(),
        ));
    }

    // Emit OrganizationCreated event
    let event = DomainEvent::Organization(crate::events::OrganizationEvents::OrganizationCreated(crate::events::organization::OrganizationCreatedEvent {
        organization_id: cmd.organization_id,
        name: cmd.name,
        domain: cmd.domain,
        created_at: cmd.timestamp,
        correlation_id: cmd.correlation_id,
        causation_id: Some(cmd.command_id),
    }));

    Ok(vec![event])
}

/// Handle EstablishRelationship command
pub async fn handle_establish_relationship(
    cmd: EstablishRelationship,
) -> Result<Vec<DomainEvent>, crate::aggregate::KeyManagementError> {
    // Validate command
    if cmd.from_id == cmd.to_id {
        return Err(crate::aggregate::KeyManagementError::InvalidCommand(
            "Cannot create relationship to same entity".to_string(),
        ));
    }

    // Emit RelationshipEstablished event
    let event = DomainEvent::Relationship(crate::events::RelationshipEvents::RelationshipEstablished(crate::events::relationship::RelationshipEstablishedEvent {
        from_id: cmd.from_id,
        to_id: cmd.to_id,
        relationship_type: cmd.relationship_type,
        established_at: cmd.timestamp,
        correlation_id: cmd.correlation_id,
        causation_id: Some(cmd.command_id),
        relationship_id: uuid::Uuid::now_v7(),
        established_by: "system".to_string(),
        valid_from: cmd.timestamp,
        valid_until: None,
    }));

    Ok(vec![event])
}
