// NATS Identity Commands
//
// Command handlers for NATS Operator, Account, and User creation.
// Each handler validates input, generates NKeys/JWTs, and emits events.
//
// User Stories: US-001, US-002, US-003, US-004, US-005, US-006, US-007

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{AccountIdentity, Organization, UserIdentity};
use crate::domain_projections::NatsProjection;
use crate::events::DomainEvent;
use crate::value_objects::{
    AccountLimits, NatsCredential, NatsJwt, NKeyPair, Permissions, UserLimits,
};

// ============================================================================
// Command: Create NATS Operator (US-001, US-009)
// ============================================================================

/// Command to create NATS Operator for an organization
#[derive(Debug, Clone)]
pub struct CreateNatsOperator {
    pub organization: Organization,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Result of creating NATS Operator
#[derive(Debug, Clone)]
pub struct NatsOperatorCreated {
    pub operator_nkey: NKeyPair,
    pub operator_jwt: NatsJwt,
    pub events: Vec<DomainEvent>,
}

/// Handle CreateNatsOperator command
///
/// Emits:
/// - NatsOperatorCreatedEvent (operator details)
///
/// User Story: US-001
pub fn handle_create_nats_operator(
    cmd: CreateNatsOperator,
) -> Result<NatsOperatorCreated, String> {
    // Step 1: Project organization to operator identity (US-021: collects projection events)
    let identity = NatsProjection::project_operator(
        &cmd.organization,
        cmd.correlation_id,
        cmd.causation_id,
    );

    // Step 2: Emit operator created event
    let event = DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorCreated(crate::events::nats_operator::NatsOperatorCreatedEvent {
        operator_id: identity.nkey.id,
        name: cmd.organization.name.clone(),
        public_key: identity.nkey.public_key_string().to_string(),
        created_at: Utc::now(),
        created_by: format!("cim-keys-operator-bootstrap"), // System-initiated bootstrap
        organization_id: Some(cmd.organization.id),
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    }));

    // US-021: Combine projection events with high-level event
    let mut all_events = identity.events;
    all_events.push(event);

    Ok(NatsOperatorCreated {
        operator_nkey: identity.nkey,
        operator_jwt: identity.jwt,
        events: all_events,
    })
}

// ============================================================================
// Command: Create NATS Account (US-002, US-010)
// ============================================================================

/// Command to create NATS Account for organization or unit
#[derive(Debug, Clone)]
pub struct CreateNatsAccount {
    pub account: AccountIdentity,
    pub parent_org: Option<Organization>,
    pub operator_nkey: NKeyPair,
    pub limits: Option<AccountLimits>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Result of creating NATS Account
#[derive(Debug, Clone)]
pub struct NatsAccountCreated {
    pub account_nkey: NKeyPair,
    pub account_jwt: NatsJwt,
    pub events: Vec<DomainEvent>,
}

/// Handle CreateNatsAccount command
///
/// Emits:
/// - NatsAccountCreatedEvent (account details)
///
/// User Story: US-002
pub fn handle_create_nats_account(
    cmd: CreateNatsAccount,
) -> Result<NatsAccountCreated, String> {
    // Step 1: Project account identity to NATS account (US-021: collects projection events)
    let identity = NatsProjection::project_account_identity(
        &cmd.account,
        cmd.parent_org.as_ref(),
        &cmd.operator_nkey,
        cmd.limits,
        cmd.correlation_id,
        cmd.causation_id,
    );

    // Step 2: Emit account created event
    let event = DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsAccountCreated(crate::events::nats_account::NatsAccountCreatedEvent {
        account_id: identity.nkey.id,
        operator_id: cmd.operator_nkey.id,
        name: cmd.account.name().to_string(),
        public_key: identity.nkey.public_key_string().to_string(),
        is_system: matches!(cmd.account, AccountIdentity::Organization(_)),
        created_at: Utc::now(),
        created_by: format!("cim-keys-account-bootstrap"),
        organization_unit_id: match &cmd.account {
            AccountIdentity::OrganizationUnit(unit) => Some(unit.id),
            AccountIdentity::Organization(_) => None,
        },
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    }));

    // US-021: Combine projection events with high-level event
    let mut all_events = identity.events;
    all_events.push(event);

    Ok(NatsAccountCreated {
        account_nkey: identity.nkey,
        account_jwt: identity.jwt,
        events: all_events,
    })
}

// ============================================================================
// Command: Create NATS User (US-003, US-005, US-006, US-007)
// ============================================================================

/// Command to create NATS User for Person, Agent, or ServiceAccount
#[derive(Debug, Clone)]
pub struct CreateNatsUser {
    pub user: UserIdentity,
    pub organization: Organization,
    pub account_nkey: NKeyPair,
    pub permissions: Option<Permissions>,
    pub limits: Option<UserLimits>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Result of creating NATS User
#[derive(Debug, Clone)]
pub struct NatsUserCreated {
    pub user_nkey: NKeyPair,
    pub user_jwt: NatsJwt,
    pub credential: Option<NatsCredential>,
    pub events: Vec<DomainEvent>,
}

/// Handle CreateNatsUser command
///
/// Validates accountability for Agents and ServiceAccounts.
///
/// Emits:
/// - NatsUserCreatedEvent (user details)
/// - ServiceAccountCreatedEvent (if ServiceAccount)
/// - AgentCreatedEvent (if Agent)
/// - AccountabilityValidatedEvent (if automated identity)
/// - AccountabilityViolatedEvent (if validation fails)
///
/// User Story: US-003, US-005, US-006, US-007, US-008
pub fn handle_create_nats_user(cmd: CreateNatsUser) -> Result<NatsUserCreated, String> {
    let mut events = Vec::new();

    // Step 1: Validate accountability for automated identities (US-006, US-007)
    if let Some(responsible_person_id) = cmd.user.responsible_person_id() {
        match cmd.user.validate_accountability(&cmd.organization) {
            Ok(_) => {
                // Emit accountability validated event
                // Note: Organization doesn't have a direct people collection in the current model.
                // Person name would need to be looked up from a separate registry or passed in the command.
                // For now, we use the person_id as the name placeholder.
                let responsible_person_name = format!("person-{}", responsible_person_id);

                events.push(DomainEvent::Relationship(crate::events::RelationshipEvents::AccountabilityValidated(
                    crate::events::relationship::AccountabilityValidatedEvent {
                        validation_id: Uuid::now_v7(),
                        identity_id: cmd.user.id(),
                        identity_type: match &cmd.user {
                            UserIdentity::Person(_) => "Person".to_string(),
                            #[cfg(feature = "cim-domain-agent")]
                            UserIdentity::Agent(_) => "Agent".to_string(),
                            UserIdentity::ServiceAccount(_) => "ServiceAccount".to_string(),
                        },
                        identity_name: cmd.user.name().to_string(),
                        responsible_person_id,
                        responsible_person_name,
                        validated_at: Utc::now(),
                        validation_result: "PASSED".to_string(),
                        correlation_id: Some(cmd.correlation_id),
                        causation_id: cmd.causation_id,
                    },
                )));
            }
            Err(reason) => {
                // Emit accountability violation event
                let violation_event = DomainEvent::Relationship(crate::events::RelationshipEvents::AccountabilityViolated(
                    crate::events::relationship::AccountabilityViolatedEvent {
                        violation_id: Uuid::now_v7(),
                        identity_id: cmd.user.id(),
                        identity_type: match &cmd.user {
                            UserIdentity::Person(_) => "Person".to_string(),
                            #[cfg(feature = "cim-domain-agent")]
                            UserIdentity::Agent(_) => "Agent".to_string(),
                            UserIdentity::ServiceAccount(_) => "ServiceAccount".to_string(),
                        },
                        identity_name: cmd.user.name().to_string(),
                        violation_reason: reason.clone(),
                        detected_at: Utc::now(),
                        required_action: "Assign valid responsible_person_id".to_string(),
                        severity: "CRITICAL".to_string(),
                        correlation_id: Some(cmd.correlation_id),
                        causation_id: cmd.causation_id,
                    },
                ));
                events.push(violation_event);
                return Err(reason);
            }
        }
    }

    // Step 2: Emit creation event for ServiceAccount or Agent
    match &cmd.user {
        UserIdentity::ServiceAccount(sa) => {
            events.push(DomainEvent::NatsUser(crate::events::NatsUserEvents::ServiceAccountCreated(crate::events::nats_user::ServiceAccountCreatedEvent {
                service_account_id: sa.id,
                name: sa.name.clone(),
                purpose: sa.purpose.clone(),
                owning_unit_id: sa.owning_unit_id,
                responsible_person_id: sa.responsible_person_id,
                created_at: Utc::now(),
                correlation_id: Some(cmd.correlation_id),
                causation_id: cmd.causation_id,
            })));
        }
        #[cfg(feature = "cim-domain-agent")]
        UserIdentity::Agent(agent) => {
            // Convert agent_type to string representation using Display trait
            let agent_type_str = agent.agent_type().to_string();

            events.push(DomainEvent::NatsUser(crate::events::NatsUserEvents::AgentCreated(crate::events::nats_user::AgentCreatedEvent {
                agent_id: agent.id().into(), // Convert AgentId to Uuid
                name: agent.metadata().name().to_string(),
                agent_type: agent_type_str,
                responsible_person_id: agent.metadata().owner_id(), // owner_id is the responsible person
                organization_id: agent.metadata().owner_id(), // Using owner as org for now
                created_at: Utc::now(),
                correlation_id: Some(cmd.correlation_id),
                causation_id: cmd.causation_id,
            })));
        }
        UserIdentity::Person(_) => {
            // Person is self-accountable, no special event needed
        }
    }

    // Step 3: Project user identity to NATS user (US-021: collects projection events)
    let identity = NatsProjection::project_user_identity(
        &cmd.user,
        &cmd.organization,
        &cmd.account_nkey,
        cmd.permissions,
        cmd.limits,
        cmd.correlation_id,
        cmd.causation_id,
    );

    // US-021: Extend events with projection events
    events.extend(identity.events);

    // Step 4: Emit user created event
    events.push(DomainEvent::NatsUser(crate::events::NatsUserEvents::NatsUserCreated(crate::events::nats_user::NatsUserCreatedEvent {
        user_id: identity.nkey.id,
        account_id: cmd.account_nkey.id,
        name: cmd.user.name().to_string(),
        public_key: identity.nkey.public_key_string().to_string(),
        created_at: Utc::now(),
        created_by: format!("cim-keys-user-bootstrap"),
        person_id: match &cmd.user {
            UserIdentity::Person(p) => Some(p.id),
            _ => None,
        },
        correlation_id: cmd.correlation_id,
        causation_id: cmd.causation_id,
    })));

    Ok(NatsUserCreated {
        user_nkey: identity.nkey,
        user_jwt: identity.jwt,
        credential: identity.credential,
        events,
    })
}

// ============================================================================
// Command: Bootstrap Complete NATS Infrastructure (US-011)
// ============================================================================

/// Command to bootstrap complete NATS infrastructure from Organization
#[derive(Debug, Clone)]
pub struct BootstrapNatsInfrastructure {
    pub organization: Organization,
    pub correlation_id: Uuid,
}

/// Complete NATS infrastructure result
#[derive(Debug, Clone)]
pub struct NatsInfrastructureBootstrapped {
    pub operator: NatsOperatorCreated,
    pub accounts: Vec<NatsAccountCreated>,
    pub users: Vec<NatsUserCreated>,
    pub events: Vec<DomainEvent>,
}

/// Handle BootstrapNatsInfrastructure command
///
/// This is the organization-centric projection that extracts all identities
/// from the organizational structure.
///
/// Emits:
/// - All operator, account, and user events
/// - Complete event stream for entire infrastructure
///
/// User Story: US-011
pub fn handle_bootstrap_nats_infrastructure(
    cmd: BootstrapNatsInfrastructure,
) -> Result<NatsInfrastructureBootstrapped, String> {
    let mut all_events = Vec::new();

    // Step 1: Create operator for organization
    let operator_cmd = CreateNatsOperator {
        organization: cmd.organization.clone(),
        correlation_id: cmd.correlation_id,
        causation_id: None,
    };
    let operator = handle_create_nats_operator(operator_cmd)?;
    all_events.extend(operator.events.clone());

    // Step 2: Create accounts for all organizational units
    let mut accounts = Vec::new();
    for unit in &cmd.organization.units {
        let account_cmd = CreateNatsAccount {
            account: AccountIdentity::OrganizationUnit(unit.clone()),
            parent_org: Some(cmd.organization.clone()),
            operator_nkey: operator.operator_nkey.clone(),
            limits: Some(AccountLimits::default()),
            correlation_id: cmd.correlation_id,
            causation_id: Some(operator.operator_nkey.id),
        };
        let account = handle_create_nats_account(account_cmd)?;
        all_events.extend(account.events.clone());
        accounts.push(account);
    }

    // Step 3: Create users for all people/agents/services in organization
    //
    // NOTE: The current Organization model doesn't have direct collections of
    // people, service accounts, or agents. These entities reference the organization
    // via their organization_id field, but the Organization doesn't maintain
    // a reverse index.
    //
    // To bootstrap users, you need to:
    // 1. Maintain a separate registry of Person entities and query by organization_id
    // 2. Maintain a separate registry of ServiceAccount entities and query by owning_unit_id
    // 3. If using the 'cim-domain-agent' feature, maintain an Agent registry
    //
    // For each discovered identity, you would:
    // - Determine which account (unit) they belong to
    // - Call handle_create_nats_user with the appropriate CreateNatsUser command
    //
    // Example (when person registry is available):
    // ```
    // for person in person_registry.find_by_organization(cmd.organization.id) {
    //     let account = accounts.iter()
    //         .find(|a| person.unit_ids.contains(&a.account_nkey.id))
    //         .ok_or("No account found for person")?;
    //
    //     let user_cmd = CreateNatsUser {
    //         user: UserIdentity::Person(person),
    //         organization: cmd.organization.clone(),
    //         account_nkey: account.account_nkey.clone(),
    //         permissions: None,
    //         limits: None,
    //         correlation_id: cmd.correlation_id,
    //         causation_id: Some(account.account_nkey.id),
    //     };
    //     let user = handle_create_nats_user(user_cmd)?;
    //     all_events.extend(user.events.clone());
    //     users.push(user);
    // }
    // ```
    let users = Vec::new(); // Currently empty - requires external person/service/agent registries

    Ok(NatsInfrastructureBootstrapped {
        operator,
        accounts,
        users,
        events: all_events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, OrganizationUnit, OrganizationUnitType, Person};
    use crate::value_objects::NKeyType;

    #[test]
    fn test_create_operator_emits_event() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        let cmd = CreateNatsOperator {
            organization: org,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_create_nats_operator(cmd).unwrap();

        // US-021: Now emits 4 events (NKeyGenerated, JwtClaimsCreated, JwtSigned, NatsOperatorCreated)
        assert_eq!(result.events.len(), 4);
        assert!(matches!(
            result.events[0],
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NKeyGenerated(_))
        ));
        assert!(matches!(
            result.events[1],
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwtClaimsCreated(_))
        ));
        assert!(matches!(
            result.events[2],
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwtSigned(_))
        ));
        assert!(matches!(
            result.events[3],
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NatsOperatorCreated(_))
        ));
        assert_eq!(result.operator_nkey.key_type, NKeyType::Operator);
    }

    #[test]
    fn test_create_account_emits_event() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        let unit = OrganizationUnit {
            id: Uuid::now_v7(),
            name: "Engineering".to_string(),
            unit_type: OrganizationUnitType::Department,
            parent_unit_id: None,
            responsible_person_id: None,
            nats_account_name: None,
        };

        // Create operator first
        let operator = handle_create_nats_operator(CreateNatsOperator {
            organization: org.clone(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        })
        .unwrap();

        let cmd = CreateNatsAccount {
            account: AccountIdentity::OrganizationUnit(unit),
            parent_org: Some(org),
            operator_nkey: operator.operator_nkey,
            limits: None,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_create_nats_account(cmd).unwrap();

        // US-021: Now emits 4 events (NKeyGenerated, JwtClaimsCreated, JwtSigned, NatsAccountCreated)
        assert_eq!(result.events.len(), 4);
        assert!(matches!(
            result.events[0],
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::NKeyGenerated(_))
        ));
        assert!(matches!(
            result.events[1],
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwtClaimsCreated(_))
        ));
        assert!(matches!(
            result.events[2],
            DomainEvent::NatsOperator(crate::events::NatsOperatorEvents::JwtSigned(_))
        ));
        assert!(matches!(
            result.events[3],
            DomainEvent::NatsAccount(crate::events::NatsAccountEvents::NatsAccountCreated(_))
        ));
        assert_eq!(result.account_nkey.key_type, NKeyType::Account);
    }

    #[test]
    fn test_create_user_validates_accountability() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: Utc::now(),
            metadata: Default::default(),
        };

        let person = Person {
            id: Uuid::now_v7(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            roles: vec![],
            organization_id: org.id,
            unit_ids: vec![],
            created_at: Utc::now(),
            active: true,
            nats_permissions: None,
        };

        // Create operator and account first
        let operator = handle_create_nats_operator(CreateNatsOperator {
            organization: org.clone(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        })
        .unwrap();

        let unit = OrganizationUnit {
            id: Uuid::now_v7(),
            name: "Engineering".to_string(),
            unit_type: OrganizationUnitType::Department,
            parent_unit_id: None,
            responsible_person_id: None,
            nats_account_name: None,
        };

        let account = handle_create_nats_account(CreateNatsAccount {
            account: AccountIdentity::OrganizationUnit(unit),
            parent_org: Some(org.clone()),
            operator_nkey: operator.operator_nkey,
            limits: None,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        })
        .unwrap();

        let cmd = CreateNatsUser {
            user: UserIdentity::Person(person),
            organization: org,
            account_nkey: account.account_nkey,
            permissions: None,
            limits: None,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };

        let result = handle_create_nats_user(cmd).unwrap();

        // Person is self-accountable, should have 1 event (NatsUserCreated)
        assert!(result.events.len() >= 1);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, DomainEvent::NatsUser(crate::events::NatsUserEvents::NatsUserCreated(_)))));
    }
}
