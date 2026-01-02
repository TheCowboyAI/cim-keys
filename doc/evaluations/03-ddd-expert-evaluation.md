# DDD Compliance Evaluation: cim-keys

**Evaluator:** DDD Expert
**Date:** 2026-01-02
**Overall DDD Compliance Score: 55/100**

---

## 1. Analysis of Core Domain Files

### 1.1 Domain Module Structure

**Findings:**
- Domain types include: `DomainBootstrap`, `OperatorDefinition`, `AccountDefinition`, `UserDefinition`
- The `Foldable` trait is implemented for graph traversal

**Issues Identified:**
1. **Domain types are configuration DTOs, not true aggregates**
2. **No explicit aggregate roots**
3. **Missing phantom-typed IDs**

### 1.2 Bootstrap Definitions

```rust
pub struct DomainBootstrap {
    pub operator: OperatorDefinition,
    pub accounts: Vec<AccountDefinition>,
}
```

**DDD Violations:**

1. **Anemic Domain Model** - Pure data structures with no behavior
2. **No Value Objects** - `name: String` should be `OperatorName` value object
3. **Missing Aggregate Invariants** - No business rules enforced
4. **CRUD-Style Structure** - Looks like database schema, not domain model

### 1.3 Aggregate Module

**DDD Compliance Issues:**

1. **Single Monolithic Aggregate** - `KeyAggregate` handles all domain operations
2. **Port Dependencies in Aggregate** - `yubikey_port` should be in command handlers
3. **Missing Aggregate Root Pattern**
4. **Async in Aggregate** - Aggregates should be synchronous pure functions

---

## 2. DDD Anti-Patterns Identified

### 2.1 Anemic Domain Model (Severity: HIGH)

```rust
// bootstrap.rs - Pure data, no behavior
pub struct AccountDefinition {
    pub name: String,
    pub signing_key: Option<SigningKeyConfig>,
    pub users: Vec<UserDefinition>,
}
```

### 2.2 Missing Aggregate Boundaries (Severity: HIGH)

Single `KeyAggregate` handles: Certificates, NATS credentials, YubiKey operations, PKI hierarchy

### 2.3 Downcast Chains (Severity: HIGH)

Bypasses aggregate encapsulation. Domain objects should be addressed through their aggregate root.

### 2.4 Missing Phantom-Typed Entity IDs (Severity: MEDIUM)

```rust
// Current pattern
pub person_id: Uuid,

// Required pattern
pub type PersonId = EntityId<PersonMarker>;
```

---

## 3. Recommended Better Patterns

### 3.1 Proper Aggregate Design

```rust
pub mod certificate {
    pub struct CertificateAggregate {
        id: CertificateId,
        hierarchy: CertificateHierarchy,
    }

    impl CertificateAggregate {
        pub fn issue_intermediate(
            &self,
            cmd: IssueIntermediateCertificate
        ) -> Result<Vec<CertificateEvent>, CertificateError> {
            // Pure, synchronous, no port dependencies
        }
    }
}
```

### 3.2 Value Objects with Invariant Enforcement

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorName(String);

impl OperatorName {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let name = name.into();
        if name.is_empty() {
            return Err(DomainError::InvalidOperatorName("cannot be empty"));
        }
        Ok(Self(name))
    }
}
```

### 3.3 Phantom-Typed Entity IDs

```rust
pub struct EntityId<T> {
    id: Uuid,
    _marker: PhantomData<T>,
}

pub type CertificateId = EntityId<CertificateMarker>;
pub type PersonId = EntityId<PersonMarker>;
```

---

## 4. Corrective Action Plan

### Sprint 1: Foundation - Phantom-Typed IDs and Value Objects (3-5 days)

1. [ ] Create `src/domain/ids.rs` with phantom-typed ID definitions
2. [ ] Migrate all `Uuid` IDs to phantom-typed versions
3. [ ] Create value objects: `OperatorName`, `AccountName`, `KeyPurpose`, `CertificateSubject`

### Sprint 2: Aggregate Boundary Definition (5-7 days)

1. [ ] Extract `CertificateAggregate`
2. [ ] Extract `IdentityAggregate`
3. [ ] Extract `NatsInfrastructureAggregate`
4. [ ] Define per-aggregate event and command types

### Sprint 3: LiftableDomain Implementation (5-7 days)

1. [ ] Refactor GUI graph code to use `LiftableDomain`
2. [ ] Remove `Injection` enum and downcast chains

### Sprint 4: Event Sourcing Purity (3-5 days)

1. [ ] Move YubiKey port calls to application service layer
2. [ ] Make all aggregate methods synchronous
3. [ ] Introduce `CommandHandler` application service pattern

### Sprint 5: Domain Language Alignment (3-5 days)

1. [ ] Create domain glossary in `doc/DOMAIN-GLOSSARY.md`
2. [ ] Rename types to match business language

---

## 5. Compliance Verification Checklist

### Aggregate Boundaries
- [ ] Each aggregate has single responsibility
- [ ] Aggregates don't share mutable state
- [ ] Cross-aggregate references use IDs only

### Value Objects
- [ ] All domain values are immutable
- [ ] Validation at construction time
- [ ] No invalid states representable

### Entity Identity
- [ ] All entity IDs are phantom-typed
- [ ] Compiler prevents ID type confusion
- [ ] IDs use UUIDv7 for time-ordering

### Domain Events
- [ ] Events capture business intent
- [ ] Events are immutable
- [ ] Events have correlation/causation IDs

**Estimated total effort: 19-29 days** across 5 sprints.
