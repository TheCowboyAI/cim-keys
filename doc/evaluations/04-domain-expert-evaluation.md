# Bounded Context Evaluation: cim-keys

**Evaluator:** Domain Expert
**Date:** 2026-01-02

---

## 1. Current Architecture Analysis

### 1.1 Domain Structure Overview

```
src/
├── domain/           # Core shared kernel
├── domains/          # Bounded contexts
│   ├── organization/ # Organization context
│   ├── pki/          # PKI context
│   ├── nats/         # NATS context
│   └── typography/   # Typography context
├── lifting.rs        # Context mapping layer
└── fold.rs           # Functional folding primitives
```

### 1.2 Bounded Contexts Identified

| Context | Purpose | Key Types |
|---------|---------|-----------|
| **Organization** | Domain entities | Organization, Person, OrganizationUnit, Location, Role, Policy |
| **PKI** | Cryptographic identity | Certificate, Key, YubiKey, PIVSlot |
| **NATS** | Messaging infrastructure | Operator, Account, User, NatsConfig |
| **Typography** | Visualization theming | FontSet, VerifiedTheme |

---

## 2. Anti-Patterns Identified

### 2.1 Context Leakage (HIGH SEVERITY)

**Problem:** PKI context directly references Organization types

```rust
pub struct KeyOwnership {
    pub key_id: Uuid,
    pub person_id: Uuid,        // Direct reference to Organization context!
    pub organization_id: Uuid,  // Direct reference to Organization context!
}
```

### 2.2 Missing Anti-Corruption Layer (HIGH SEVERITY)

**Problem:** NATS context maps directly to Organization entities without translation.

### 2.3 Shared Kernel Overreach (MEDIUM SEVERITY)

**Problem:** `src/domain/mod.rs` includes too much context-specific logic.

### 2.4 Implicit Context Mapping (MEDIUM SEVERITY)

**Problem:** `lifting.rs` maps contexts to graph without explicit context map documentation.

### 2.5 Missing Published Language (LOW SEVERITY)

**Problem:** No explicit DTOs or events for cross-context communication.

---

## 3. Recommended Context Map

```
Organization Context [Upstream]
        │
        ▼ publishes
┌─────────────────────────────┐
│   Published Language        │
│   - OrganizationReference   │
│   - PersonReference         │
│   - KeyReference            │
└─────────────────────────────┘
        │
        ▼ consumes via ACL
┌───────────────┬───────────────┐
│ PKI Context   │ NATS Context  │
│ [Downstream]  │ [Downstream]  │
└───────────────┴───────────────┘
```

---

## 4. Corrective Action Plan

### Story 11.1: Define Published Language (5 points)

1. [ ] Create `src/domains/organization/published.rs`
   - `OrganizationReference { id: Uuid, name: String }`
   - `PersonReference { id: Uuid, display_name: String }`
   - `LocationReference { id: Uuid, name: String }`

2. [ ] Create `src/domains/pki/published.rs`
   - `KeyReference { id: Uuid, algorithm: String, fingerprint: String }`
   - `CertificateReference { id: Uuid, subject: String, not_after: DateTime }`

### Story 11.2: Implement Anti-Corruption Layer for PKI (8 points)

1. [ ] Create `src/domains/pki/acl/mod.rs`
2. [ ] Create `OrgContextAdapter` trait
3. [ ] Refactor `KeyOwnership` to use `PersonReference`

### Story 11.3: Implement Anti-Corruption Layer for NATS (5 points)

1. [ ] Create `src/domains/nats/acl/mod.rs`
2. [ ] Create `PersonContextAdapter` trait
3. [ ] Refactor `NatsUser` to use `PersonReference`

### Story 11.4: Minimize Shared Kernel (3 points)

1. [ ] Move context-specific imports out of `domain/mod.rs`
2. [ ] Create `src/shared_kernel/mod.rs`

### Story 11.5: Document Context Map (2 points)

1. [ ] Create `doc/architecture/context-map.md`

### Story 11.6: Update Lifting Layer (3 points)

1. [ ] Update `LiftableDomain` implementations to use published types

### Story 11.7: Add Context Boundary Tests (3 points)

1. [ ] Create `tests/context_boundaries.rs`
2. [ ] Add compilation tests that fail if contexts import directly

---

## 5. Sprint Planning Summary

**Sprint 11 Total Points: 29**

| Story | Points | Priority | Depends On |
|-------|--------|----------|------------|
| 11.1 Published Language | 5 | HIGH | - |
| 11.2 PKI ACL | 8 | HIGH | 11.1 |
| 11.3 NATS ACL | 5 | HIGH | 11.1 |
| 11.4 Minimize Shared Kernel | 3 | MEDIUM | - |
| 11.5 Context Map Documentation | 2 | MEDIUM | 11.1, 11.2, 11.3 |
| 11.6 Update Lifting Layer | 3 | MEDIUM | 11.1 |
| 11.7 Context Boundary Tests | 3 | MEDIUM | 11.2, 11.3 |

### Recommended Sprint Split

**Sprint 11A (Week 1): Foundation** - 8 pts
**Sprint 11B (Week 2): ACL Implementation** - 13 pts
**Sprint 11C (Week 3): Validation & Documentation** - 8 pts

---

## 6. Success Metrics

1. **Zero direct cross-context imports** (measurable via lint rules)
2. **100% coverage of context boundary tests**
3. **Published language documentation complete**
4. **Context map reviewed and approved by team**
