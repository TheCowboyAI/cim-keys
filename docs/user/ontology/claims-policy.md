<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->
# Claims-Based Policy Ontology

## Overview

The CIM authorization system uses a **Claims-Based Policy Ontology** - a formal knowledge representation that defines permissions, roles, and access control through compositional semantics.

## Ontological Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CLAIMS-POLICY ONTOLOGY                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  VOCABULARY LAYER (Terms)                                                   │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                          │
│  │ Claim   │ │ Claim   │ │ Claim   │ │ Claim   │  ... 200+ atomic claims  │
│  │CreateUser│ │ReadRepo │ │SignCode │ │ViewLogs │                          │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘                          │
│       │           │           │           │                                │
│  ─────┼───────────┼───────────┼───────────┼────────────────────────────── │
│       │           │           │           │                                │
│  COMPOSITION LAYER (Aggregates)                                            │
│  ┌────┴───────────┴───────────┴───────────┴────┐                          │
│  │                    Role                      │                          │
│  │  "Junior Developer"                         │                          │
│  │  Purpose: Entry-level code contribution     │                          │
│  │  Claims: {ReadRepo, WriteRepo, CreateBranch}│                          │
│  └──────────────────────┬──────────────────────┘                          │
│                         │                                                  │
│  ─────────────────────────────────────────────────────────────────────────│
│                         │                                                  │
│  CONTEXTUALIZATION LAYER (Conditions)                                      │
│  ┌──────────────────────┴──────────────────────┐                          │
│  │                   Policy                     │                          │
│  │  Role + Conditions + Priority               │                          │
│  │  Conditions: {MFA=true, TimeWindow=9-17}    │                          │
│  │  Priority: 100                              │                          │
│  └──────────────────────┬──────────────────────┘                          │
│                         │                                                  │
│  ─────────────────────────────────────────────────────────────────────────│
│                         │                                                  │
│  INSTANTIATION LAYER (Bindings)                                            │
│  ┌──────────────────────┴──────────────────────┐                          │
│  │               PolicyBinding                  │                          │
│  │  Policy → Entity (Person, Org, Location)   │                          │
│  │  "Alice has Junior Developer policy"        │                          │
│  └──────────────────────┬──────────────────────┘                          │
│                         │                                                  │
│  ─────────────────────────────────────────────────────────────────────────│
│                         ▼                                                  │
│  INFERENCE LAYER (Evaluation)                                              │
│  ┌─────────────────────────────────────────────┐                          │
│  │             PolicyEvaluation                 │                          │
│  │  Effective Claims = ∪(active policy claims) │                          │
│  │  At time T, Alice can: {ReadRepo, ...}      │                          │
│  └─────────────────────────────────────────────┘                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### 1. Claim (Atomic Permission)

A **Claim** is the atomic unit of permission - a single capability that can be granted.

```rust
pub enum Claim {
    // Identity
    CreateUser,
    ReadUser,
    DeleteUser,

    // Infrastructure
    CreateServer,
    DeployToProduction,

    // Security
    GenerateKey,
    SignCode,

    // Custom extension
    Custom { domain, resource, action, scope },
}
```

**Properties of Claims:**
- **Category**: Which domain (Identity, Security, Data, etc.)
- **Read-only**: Does not modify state
- **Destructive**: Permanently removes resources
- **Elevation-required**: Needs additional authorization

### 2. Role (Claim Aggregate)

A **Role** is a meaningful aggregation of claims with a unique purpose.

```rust
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub purpose: String,           // WHY this combination
    pub claims: Vec<Claim>,        // WHAT permissions
    pub incompatible_roles: Vec<Uuid>,  // Separation of duties
}
```

**Role Composition:**
- Roles do NOT inherit from each other
- Roles can be explicitly composed: `RoleA ∪ RoleB = combined claims`
- Incompatible roles enforce separation of duties

### 3. Policy (Contextualized Role)

A **Policy** adds conditions to when a Role's claims are active.

```rust
pub struct Policy {
    pub id: Uuid,
    pub role_id: Uuid,
    pub conditions: Vec<Condition>,
    pub priority: i32,
    pub enabled: bool,
}
```

**Conditions include:**
- Time windows
- Location restrictions
- MFA requirements
- Security clearance levels
- Training completion
- Witness requirements

### 4. PolicyBinding (Instantiation)

A **PolicyBinding** connects a Policy to an entity.

```rust
pub struct PolicyBinding {
    pub policy_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: EntityType,  // Person, Org, Location, Key
}
```

### 5. PolicyEvaluation (Inference)

**PolicyEvaluation** computes effective permissions at runtime.

```rust
pub struct PolicyEvaluation {
    pub entity_id: Uuid,
    pub granted_claims: Vec<Claim>,
    pub active_policies: Vec<Uuid>,
    pub inactive_policies: Vec<(Uuid, Vec<String>)>,  // + reasons
}
```

## Claim Categories

| Category | Description | Example Claims |
|----------|-------------|----------------|
| **Identity** | User, group, role, session management | CreateUser, AssignRole, TerminateSession |
| **Infrastructure** | Servers, containers, networks, storage | CreateServer, ManageFirewall, ConfigureVPN |
| **Development** | Code, CI/CD, deployments, releases | WriteRepository, DeployToProduction, MergeBranch |
| **Security** | Keys, certificates, incidents, compliance | GenerateKey, SignCode, OverrideSecurityControl |
| **Data** | Databases, backups, PII, exports | ExecuteSQL, ReadRestrictedData, AnonymizeData |
| **Observability** | Logs, metrics, alerts, traces | ViewAuditLogs, CreateAlertRule, ConfigureTracing |
| **Communication** | Email, chat, documents, wikis | SendOrganizationEmail, ShareDocumentExternal |
| **Project** | Tasks, sprints, roadmaps, budgets | CreateSprint, ApproveTimesheet, EditRoadmap |
| **Finance** | Invoices, expenses, procurement | ApproveExpense, SignContract, ViewFinancialReport |
| **HR** | Employees, onboarding, reviews, training | UpdateCompensation, InitiateOffboarding |
| **NATS** | Messaging infrastructure | CreateNATSOperator, ManageJetStream |
| **Organization** | Structure management | CreateOrganizationalUnit, ManageOrganizationSettings |
| **Policy** | Meta-management | CreatePolicy, BindPolicy |
| **Emergency** | Break-glass operations | SuperAdmin, OverrideSecurityControl |

## Standard Roles

### Development Track

| Role | Purpose | Key Claims |
|------|---------|------------|
| **Junior Developer** | Entry-level code contribution | ReadRepository, WriteRepository, CreateBranch |
| **Developer** | Full development workflow | + MergeBranch, TriggerPipeline, DeployToDevelopment |
| **Senior Developer** | Code review and mentorship | + ApprovePullRequest, ManageRepositorySettings |
| **Lead Developer** | Technical leadership | + DeployToStaging, ConfigurePipeline |
| **Principal Engineer** | Architecture decisions | + DeployToProduction, ManagePipelineSecrets |

### Operations Track

| Role | Purpose | Key Claims |
|------|---------|------------|
| **Junior SRE** | Basic operational monitoring | ViewLogs, ViewMetrics, ViewAlerts |
| **SRE** | Incident response | + AcknowledgeAlert, RestartServer, ViewContainerLogs |
| **Senior SRE** | Infrastructure management | + CreateServer, ManageFirewall, ConfigureLoadBalancer |
| **Platform Engineer** | Platform development | + ManageNamespace, DeployHelmChart, ConfigureTracing |

### Security Track

| Role | Purpose | Key Claims |
|------|---------|------------|
| **Security Analyst** | Threat monitoring | ViewSecurityAlerts, ViewVulnerabilityReport |
| **Security Engineer** | Security implementation | + RunSecurityScan, ConfigureCompliancePolicy |
| **Security Architect** | Security design | + CreatePolicy, ManagePipelineSecrets |
| **CISO** | Security leadership | + OverrideSecurityControl (break-glass only) |

### C-Level Executive Track

| Role | Purpose | Key Claims |
|------|---------|------------|
| **CEO** | Ultimate organizational authority | ManageOrganizationSettings, SignContract, AccessEmergencyControl |
| **COO** | Daily operations & efficiency | ManageCloudAccount, ApproveExpense, OverrideSecurityControl |
| **CFO** | Financial management | ViewFinancialReport, ApproveInvoice, UpdateCompensation |
| **CLO** | Legal affairs & compliance | SignContract, ConfigureCompliancePolicy, ManageDataRetention |
| **CSO** | Research & scientific strategy | DeployToProduction, CreateDatabase, GenerateKey |

#### Cowboy AI C-Level Assignments

| Role | Person |
|------|--------|
| CEO | Jace |
| COO | David |
| CFO | (Vacant) |
| CLO | Mark Sonnenklar |
| CSO | Steele |

## Ontological Properties

### Separation of Duties

Certain roles are **incompatible** - a person cannot hold both:

- **Developer** ⊥ **Auditor**: Cannot audit own code
- **Requestor** ⊥ **Approver**: Cannot approve own requests
- **Key Generator** ⊥ **Key Auditor**: Cannot audit own keys

### Least Privilege

Roles grant the **minimum claims necessary** for their purpose:

```
JuniorDeveloper ⊂ Developer ⊂ SeniorDeveloper (claim containment)
```

### Temporal Validity

All bindings have temporal bounds:
- `valid_from`: When the binding becomes active
- `valid_until`: When the binding expires (optional)

### Condition Composition

Conditions are AND-composed within a policy:
- All conditions must be satisfied
- If any fails, the policy is inactive

## Event Sourcing

All changes are recorded as immutable events:

```rust
enum RoleEvent {
    RoleCreated { id, name, purpose, claims },
    ClaimAddedToRole { role_id, claim },
    ClaimRemovedFromRole { role_id, claim },
    RoleDeactivated { role_id, reason },
}

enum PolicyEvent {
    PolicyCreated { id, role_id, conditions },
    PolicyBound { policy_id, entity_id, entity_type },
    PolicyUnbound { policy_id, entity_id },
    PolicyEvaluated { entity_id, granted_claims, timestamp },
}
```

## Integration with NATS

Policy events are published to NATS subjects:

```
cim.security.policy.created
cim.security.policy.bound
cim.security.role.created
cim.security.claim.evaluated
```

## Mathematical Foundation

The ontology forms a **partially ordered set (poset)** on claims:

- **Claim ordering**: `c1 ≤ c2` if `c1` is implied by `c2`
- **Role regions**: Roles define convex regions in claim space
- **Policy lattice**: Policies form a lattice under composition

This enables:
- Efficient subsumption checking
- Automatic role suggestions
- Conflict detection
- Minimal permission sets

## See Also

- `src/policy/claims.rs` - Claim vocabulary implementation
- `src/policy/roles.rs` - Role aggregate implementation
- `src/domain.rs` - Policy and PolicyBinding types
- `N_ARY_FRP_AXIOMS.md` - Signal flow for policy evaluation
