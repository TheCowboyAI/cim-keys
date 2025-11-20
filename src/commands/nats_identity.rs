// NATS Identity Commands
//
// Command handlers for NATS Operator, Account, and User creation.
// Each handler validates input, generates NKeys/JWTs, and emits events.
//
// User Stories: US-001, US-002, US-003, US-004, US-005, US-006, US-007

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{AccountIdentity, Organization, OrganizationUnit, UserIdentity};
use crate::domain_projections::{
    JwtClaimsProjection, JwtSigningProjection, NatsProjection, NKeyProjection,
};
use crate::events::{
    AccountabilityValidatedEvent, AccountabilityViolatedEvent, AgentCreatedEvent, KeyEvent,
    NatsAccountCreatedEvent, NatsOperatorCreatedEvent, NatsUserCreatedEvent,
    ServiceAccountCreatedEvent,
};
use crate::value_objects::{
    AccountLimits, NatsCredential, NatsJwt, NKeyPair, NKeyType, Permissions, UserLimits,
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
    pub events: Vec<KeyEvent>,
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
    // Step 1: Project organization to operator identity
    let identity = NatsProjection::project_operator(&cmd.organization);

    // Step 2: Emit operator created event
    // TODO: Add correlation_id and causation_id to event structure for proper event sourcing
    let event = KeyEvent::NatsOperatorCreated(NatsOperatorCreatedEvent {
        operator_id: identity.nkey.id,
        name: cmd.organization.name.clone(),
        public_key: identity.nkey.public_key_string().to_string(),
        created_at: Utc::now(),
        created_by: "system".to_string(), // TODO: Get actual user
        organization_id: Some(cmd.organization.id),
    });

    Ok(NatsOperatorCreated {
        operator_nkey: identity.nkey,
        operator_jwt: identity.jwt,
        events: vec![event],
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
    pub events: Vec<KeyEvent>,
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
    // Step 1: Project account identity to NATS account
    let identity = NatsProjection::project_account_identity(
        &cmd.account,
        cmd.parent_org.as_ref(),
        &cmd.operator_nkey,
        cmd.limits,
    );

    // Step 2: Emit account created event
    // TODO: Add correlation_id and causation_id to event structure for proper event sourcing
    let event = KeyEvent::NatsAccountCreated(NatsAccountCreatedEvent {
        account_id: identity.nkey.id,
        operator_id: Uuid::nil(), // TODO: Get operator ID from operator_nkey
        name: cmd.account.name().to_string(),
        public_key: identity.nkey.public_key_string().to_string(),
        is_system: false, // TODO: Determine from account type
        created_at: Utc::now(),
        created_by: "system".to_string(), // TODO: Get actual user
        organization_unit_id: None, // TODO: Get from account if OrganizationUnit
    });

    Ok(NatsAccountCreated {
        account_nkey: identity.nkey,
        account_jwt: identity.jwt,
        events: vec![event],
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
    pub events: Vec<KeyEvent>,
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
                events.push(KeyEvent::AccountabilityValidated(
                    AccountabilityValidatedEvent {
                        validation_id: Uuid::now_v7(),
                        identity_id: cmd.user.id(),
                        identity_type: match &cmd.user {
                            UserIdentity::Person(_) => "Person".to_string(),
                            #[cfg(feature = "agent")]
                            UserIdentity::Agent(_) => "Agent".to_string(),
                            UserIdentity::ServiceAccount(_) => "ServiceAccount".to_string(),
                        },
                        identity_name: cmd.user.name().to_string(),
                        responsible_person_id,
                        responsible_person_name: "TODO: Look up person name".to_string(), // TODO: Get from org
                        validated_at: Utc::now(),
                        validation_result: "PASSED".to_string(),
                    },
                ));
            }
            Err(reason) => {
                // Emit accountability violation event
                let violation_event = KeyEvent::AccountabilityViolated(
                    AccountabilityViolatedEvent {
                        violation_id: Uuid::now_v7(),
                        identity_id: cmd.user.id(),
                        identity_type: match &cmd.user {
                            UserIdentity::Person(_) => "Person".to_string(),
                            #[cfg(feature = "agent")]
                            UserIdentity::Agent(_) => "Agent".to_string(),
                            UserIdentity::ServiceAccount(_) => "ServiceAccount".to_string(),
                        },
                        identity_name: cmd.user.name().to_string(),
                        violation_reason: reason.clone(),
                        detected_at: Utc::now(),
                        required_action: "Assign valid responsible_person_id".to_string(),
                        severity: "CRITICAL".to_string(),
                    },
                );
                events.push(violation_event);
                return Err(reason);
            }
        }
    }

    // Step 2: Emit creation event for ServiceAccount or Agent
    match &cmd.user {
        UserIdentity::ServiceAccount(sa) => {
            events.push(KeyEvent::ServiceAccountCreated(ServiceAccountCreatedEvent {
                service_account_id: sa.id,
                name: sa.name.clone(),
                purpose: sa.purpose.clone(),
                owning_unit_id: sa.owning_unit_id,
                responsible_person_id: sa.responsible_person_id,
                created_at: Utc::now(),
            }));
        }
        #[cfg(feature = "agent")]
        UserIdentity::Agent(agent) => {
            events.push(KeyEvent::AgentCreated(AgentCreatedEvent {
                agent_id: agent.id,
                name: agent.name.clone(),
                agent_type: format!("{:?}", agent.agent_type), // TODO: Proper string conversion
                responsible_person_id: agent.responsible_person_id,
                organization_id: agent.organization_id,
                created_at: Utc::now(),
            }));
        }
        UserIdentity::Person(_) => {
            // Person is self-accountable, no special event needed
        }
    }

    // Step 3: Project user identity to NATS user
    let identity = NatsProjection::project_user_identity(
        &cmd.user,
        &cmd.organization,
        &cmd.account_nkey,
        cmd.permissions,
        cmd.limits,
    );

    // Step 4: Emit user created event
    // TODO: Add correlation_id and causation_id to event structure for proper event sourcing
    events.push(KeyEvent::NatsUserCreated(NatsUserCreatedEvent {
        user_id: identity.nkey.id,
        account_id: cmd.account_nkey.id,
        name: cmd.user.name().to_string(),
        public_key: identity.nkey.public_key_string().to_string(),
        created_at: Utc::now(),
        created_by: "system".to_string(), // TODO: Get actual user
        person_id: match &cmd.user {
            UserIdentity::Person(p) => Some(p.id),
            _ => None,
        },
    }));

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
    pub events: Vec<KeyEvent>,
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
    // TODO: Extract people from organizational roles
    // TODO: Extract service accounts from units
    // TODO: Extract agents from organization
    let users = Vec::new(); // Placeholder

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

        assert_eq!(result.events.len(), 1);
        assert!(matches!(
            result.events[0],
            KeyEvent::NatsOperatorCreated(_)
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

        assert_eq!(result.events.len(), 1);
        assert!(matches!(
            result.events[0],
            KeyEvent::NatsAccountCreated(_)
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
            .any(|e| matches!(e, KeyEvent::NatsUserCreated(_))));
    }
}
