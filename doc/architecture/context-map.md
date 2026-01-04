<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# CIM-Keys Context Map

This document describes the bounded context architecture of cim-keys,
following Domain-Driven Design (DDD) strategic patterns.

## Overview

CIM-Keys is organized into four primary bounded contexts, each with
clear responsibilities and well-defined boundaries:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CIM-Keys System                               │
│                                                                      │
│  ┌─────────────────┐     ┌─────────────────┐     ┌────────────────┐ │
│  │  ORGANIZATION   │     │      PKI        │     │     NATS       │ │
│  │    Context      │     │    Context      │     │   Context      │ │
│  │                 │     │                 │     │                │ │
│  │ • Organization  │     │ • Certificate   │     │ • Operator     │ │
│  │ • Person        │     │ • Key           │     │ • Account      │ │
│  │ • OrgUnit       │     │ • KeyOwnership  │     │ • User         │ │
│  │ • Location      │     │                 │     │ • ServiceAcct  │ │
│  │ • Role          │     │                 │     │                │ │
│  │ • Policy        │     │                 │     │                │ │
│  └────────┬────────┘     └────────┬────────┘     └───────┬────────┘ │
│           │                       │                      │          │
│           │    Published Language │                      │          │
│           ▼                       ▼                      ▼          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    SHARED KERNEL                              │  │
│  │  • EntityId<T> (phantom-typed IDs)                           │  │
│  │  • Uuid, DateTime<Utc>                                       │  │
│  │  • Serde traits                                              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌─────────────────┐                                                │
│  │    YUBIKEY      │                                                │
│  │    Context      │                                                │
│  │                 │                                                │
│  │ • YubiKeyDevice │                                                │
│  │ • PivSlot       │                                                │
│  │ • YubiKeyStatus │                                                │
│  └─────────────────┘                                                │
└─────────────────────────────────────────────────────────────────────┘
```

## Bounded Contexts

### 1. Organization Context (Upstream)

**Location:** `src/domain/organization.rs`, `src/domains/organization/`

**Responsibility:** Manages organizational structure, people, and policies.

**Entities:**
| Entity | Description |
|--------|-------------|
| Organization | Root entity representing an organization |
| OrganizationUnit | Departments, teams, projects within an org |
| Person | Individual members of the organization |
| Location | Physical or logical locations |
| Role | Organizational roles with responsibilities |
| Policy | Access control and authorization policies |

**Published Language:** `src/domains/organization/published.rs`
- `OrganizationReference`
- `PersonReference`
- `LocationReference`
- `RoleReference`
- `OrganizationUnitReference`

### 2. PKI Context (Downstream of Organization)

**Location:** `src/domain/pki.rs`, `src/domains/pki/`

**Responsibility:** Manages certificates, cryptographic keys, and key ownership.

**Entities:**
| Entity | Description |
|--------|-------------|
| Certificate | X.509 certificates (Root, Intermediate, Leaf) |
| CryptographicKey | Signing and encryption keys |
| KeyOwnership | Links keys to persons and organizations |

**Published Language:** `src/domains/pki/published.rs`
- `KeyReference`
- `CertificateReference`
- `KeyOwnershipReference`
- `TrustChainReference`

**Anti-Corruption Layer:** `src/domains/pki/acl.rs`
- `OrgContextPort` - Interface for accessing Organization context

### 3. NATS Context (Downstream of Organization and PKI)

**Location:** `src/domain/nats/`, `src/domains/nats/`

**Responsibility:** Manages NATS infrastructure entities mapped from organization.

**Entities:**
| Entity | Description |
|--------|-------------|
| NatsOperator | NATS operator (maps to Organization) |
| NatsAccount | NATS account (maps to OrganizationUnit) |
| NatsUser | NATS user (maps to Person) |
| NatsServiceAccount | NATS service account |

**Anti-Corruption Layer:** `src/domains/nats/acl.rs`
- `PersonContextPort` - Interface for accessing Organization context
- `PkiContextPort` - Interface for accessing PKI context

### 4. YubiKey Context (Downstream of PKI)

**Location:** `src/domain/yubikey.rs`, `src/domains/yubikey/`

**Responsibility:** Manages hardware security keys and PIV slots.

**Entities:**
| Entity | Description |
|--------|-------------|
| YubiKeyDevice | Physical YubiKey hardware |
| PivSlot | PIV slot on a YubiKey |
| YubiKeyStatus | Provisioning status for a person |

## Context Relationships

### Upstream/Downstream Relationships

```
                    UPSTREAM
                       │
           ┌───────────┴───────────┐
           │                       │
           ▼                       ▼
    ┌──────────────┐       ┌──────────────┐
    │ Organization │       │     PKI      │
    │   Context    │       │   Context    │
    └──────┬───────┘       └──────┬───────┘
           │                      │
           │    ┌─────────────────┤
           │    │                 │
           ▼    ▼                 ▼
    ┌──────────────┐       ┌──────────────┐
    │    NATS      │       │   YubiKey    │
    │   Context    │       │   Context    │
    └──────────────┘       └──────────────┘
                    DOWNSTREAM
```

### Context Map Patterns

| Relationship | Pattern | Implementation |
|--------------|---------|----------------|
| Organization → PKI | Published Language + ACL | `OrgContextPort` |
| Organization → NATS | Published Language + ACL | `PersonContextPort` |
| PKI → NATS | Published Language + ACL | `PkiContextPort` |
| PKI → YubiKey | Shared Kernel | Direct imports (same module) |

## Anti-Corruption Layers

### PKI ACL (`src/domains/pki/acl.rs`)

```rust
/// Port for accessing Organization context from PKI context.
pub trait OrgContextPort: Send + Sync {
    fn get_person(&self, person_id: Uuid) -> Option<PersonReference>;
    fn get_organization(&self, org_id: Uuid) -> Option<OrganizationReference>;
    fn get_location(&self, location_id: Uuid) -> Option<LocationReference>;
    fn get_role(&self, role_id: Uuid) -> Option<RoleReference>;
    fn person_has_role(&self, person_id: Uuid, role_name: &str) -> bool;
    fn get_organization_members(&self, org_id: Uuid) -> Vec<PersonReference>;
}
```

**Domain Concepts:**
- `KeyOwnerContext` - Key ownership using Published Language references

### NATS ACL (`src/domains/nats/acl.rs`)

```rust
/// Port for accessing Organization context from NATS context.
pub trait PersonContextPort: Send + Sync {
    fn get_person(&self, person_id: Uuid) -> Option<PersonReference>;
    fn get_organization(&self, org_id: Uuid) -> Option<OrganizationReference>;
    fn get_unit(&self, unit_id: Uuid) -> Option<OrganizationUnitReference>;
    fn get_person_role(&self, person_id: Uuid) -> Option<RoleReference>;
    fn get_unit_members(&self, unit_id: Uuid) -> Vec<PersonReference>;
    fn can_manage_nats(&self, person_id: Uuid) -> bool;
}

/// Port for accessing PKI context from NATS context.
pub trait PkiContextPort: Send + Sync {
    fn get_person_signing_key(&self, person_id: Uuid) -> Option<KeyReference>;
    fn get_server_certificate(&self, server_id: Uuid) -> Option<CertificateReference>;
    fn get_key_ownership(&self, key_id: Uuid) -> Option<KeyOwnershipReference>;
    fn is_key_valid_for_nats(&self, key_id: Uuid) -> bool;
}
```

**Domain Concepts:**
- `NatsUserContext` - NATS user using Published Language references
- `NatsAccountContext` - NATS account mapped from organizational unit

## Published Language Types

### Organization Published Language

| Type | Purpose | Fields |
|------|---------|--------|
| `OrganizationReference` | Reference to organization | id, name, display_name |
| `PersonReference` | Reference to person | id, display_name, email, active |
| `LocationReference` | Reference to location | id, name, location_type |
| `RoleReference` | Reference to role | id, name, level |
| `OrganizationUnitReference` | Reference to unit | id, name, unit_type, organization_id |

### PKI Published Language

| Type | Purpose | Fields |
|------|---------|--------|
| `KeyReference` | Reference to key | id, algorithm, fingerprint, purpose |
| `CertificateReference` | Reference to certificate | id, subject, cert_type, not_after, is_valid |
| `KeyOwnershipReference` | Reference to ownership | key_id, owner_id, organization_id, role |
| `TrustChainReference` | Reference to trust chain | root_cert_id, intermediate_cert_ids, is_valid |

## Usage Guidelines

### DO: Use Published Language for Cross-Context References

```rust
// Good: PKI context uses PersonReference
struct KeyOwnerContext {
    owner: PersonReference,  // From Published Language
    organization: OrganizationReference,
}
```

### DON'T: Import Internal Types Across Contexts

```rust
// Bad: PKI context importing internal Organization types
use crate::domain::Person;  // Direct import - AVOID

struct KeyOwnership {
    owner: Person,  // Internal type - AVOID
}
```

### DO: Use ACL Ports for Cross-Context Operations

```rust
// Good: Use port interface
fn get_key_owner(
    key_id: Uuid,
    org_port: &impl OrgContextPort,
) -> Option<KeyOwnerContext> {
    let person = org_port.get_person(owner_id)?;
    let org = org_port.get_organization(org_id)?;
    Some(KeyOwnerContext::new(person, org))
}
```

### DON'T: Bypass ACLs with Direct Queries

```rust
// Bad: Bypassing ACL
fn get_key_owner(key_id: Uuid, db: &Database) -> Option<Person> {
    db.query_person(owner_id)  // Direct query - AVOID
}
```

## Testing Context Boundaries

Context boundary tests are in `tests/context_boundaries.rs`:

```rust
#[test]
fn cross_context_workflow_uses_published_language() {
    // Setup adapters
    let person_adapter = MockPersonContextAdapter::new()
        .with_person(person);
    let pki_adapter = MockPkiContextAdapter::new()
        .with_person_key(person_id, key);

    // Workflow uses only Published Language
    let person_ref = person_adapter.get_person(person_id)?;
    let key_ref = pki_adapter.get_person_signing_key(person_id)?;

    let context = NatsUserContext::new(person_ref, org_ref)
        .with_signing_key(key_ref);
}
```

## Module Structure

```
src/
├── domain/                    # Core domain types
│   ├── organization.rs        # Organization internal types
│   ├── pki.rs                 # PKI internal types
│   ├── nats/                  # NATS internal types
│   └── yubikey.rs             # YubiKey internal types
│
├── domains/                   # Bounded context modules
│   ├── organization/
│   │   ├── mod.rs             # Entity coproduct
│   │   └── published.rs       # Published Language
│   ├── pki/
│   │   ├── mod.rs             # Entity coproduct
│   │   ├── published.rs       # Published Language
│   │   └── acl.rs             # Anti-Corruption Layer
│   ├── nats/
│   │   ├── mod.rs             # Entity coproduct
│   │   └── acl.rs             # Anti-Corruption Layer
│   └── yubikey/
│       └── mod.rs             # Entity coproduct
│
└── tests/
    └── context_boundaries.rs  # Context boundary tests
```

## Evolution Strategy

### Adding New Contexts

1. Create `src/domain/new_context.rs` for internal types
2. Create `src/domains/new_context/mod.rs` for entity coproduct
3. If downstream, create `src/domains/new_context/acl.rs` for ACL
4. If upstream, create `src/domains/new_context/published.rs` for Published Language
5. Add context boundary tests

### Migrating Existing Cross-Context References

1. Identify direct imports across contexts
2. Create Published Language types for referenced entities
3. Create ACL port trait
4. Implement mock adapter for testing
5. Update consuming code to use port interface
6. Add context boundary tests

## Related Documentation

- [DDD Expert Evaluation](../evaluations/03-ddd-expert-evaluation.md)
- [Domain Expert Evaluation](../evaluations/04-domain-expert-evaluation.md)
- [Sprint 35 Retrospective](../../retrospectives/sprint_35.md)
