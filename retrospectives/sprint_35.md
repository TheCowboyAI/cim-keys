# Sprint 35 Retrospective: Bounded Context ACLs

**Date:** 2026-01-03
**Status:** Complete

## Sprint Goal
Eliminate context leakage by implementing Anti-Corruption Layers (ACLs) and Published Language types for cross-context communication.

## What Was Accomplished

### 1. Published Language Types (`src/domains/*/published.rs`)

**Organization Context Published Language (5 types):**
- `OrganizationReference` - Reference to Organization without importing internal type
- `PersonReference` - Reference to Person without importing internal type
- `LocationReference` - Reference to Location without importing internal type
- `RoleReference` - Reference to Role without importing internal type
- `OrganizationUnitReference` - Reference to OrganizationUnit without importing internal type

**PKI Context Published Language (4 types):**
- `KeyReference` - Reference to CryptographicKey without importing internal type
- `CertificateReference` - Reference to Certificate without importing internal type
- `KeyOwnershipReference` - Reference to key ownership relationship
- `TrustChainReference` - Reference to certificate trust chain

All Published Language types:
- Are self-contained (no imports from internal types)
- Implement Serialize/Deserialize
- Implement Hash/Eq for use in collections
- Have comprehensive test coverage

### 2. PKI Anti-Corruption Layer (`src/domains/pki/acl.rs`)

**Port (Interface):**
- `OrgContextPort` trait - Interface for accessing Organization context
  - `get_person()`, `get_organization()`, `get_location()`, `get_role()`
  - `person_has_role()` - Authorization check
  - `get_organization_members()` - List members

**Adapter (Implementation):**
- `MockOrgContextAdapter` - Mock for testing
- Builder pattern: `.with_person()`, `.with_organization()`, etc.

**Domain Concept:**
- `KeyOwnerContext` - PKI domain type using Published Language references

### 3. NATS Anti-Corruption Layer (`src/domains/nats/acl.rs`)

**Ports (Interfaces):**
- `PersonContextPort` trait - Interface for accessing Organization context from NATS
  - `get_person()`, `get_unit()`, `get_person_role()`
  - `get_unit_members()`, `can_manage_nats()`
- `PkiContextPort` trait - Interface for accessing PKI context from NATS
  - `get_person_signing_key()`, `get_server_certificate()`
  - `get_key_ownership()`, `is_key_valid_for_nats()`

**Adapters (Implementations):**
- `MockPersonContextAdapter` - Mock for Organization context
- `MockPkiContextAdapter` - Mock for PKI context

**Domain Concepts:**
- `NatsUserContext` - NATS user using Published Language references
- `NatsAccountContext` - NATS account mapped from organizational unit

### 4. Context Boundary Tests (`tests/context_boundaries.rs`)

12 integration tests verifying:
- Published Language types are self-contained
- ACL ports/adapters work correctly
- Cross-context workflows use only Published Language
- No direct imports of internal types across contexts

## Architecture Achievement

```
Before (Context Leakage):
┌──────────────────┐     direct import     ┌──────────────────┐
│  PKI Context     │ ─────────────────────► │  Organization    │
│  (KeyOwnership)  │     Person, Org        │  Context         │
└──────────────────┘                        └──────────────────┘

After (ACL Pattern):
┌──────────────────┐     OrgContextPort     ┌──────────────────┐
│  PKI Context     │ ◄──────────────────────│  Published Lang  │
│  (KeyOwnerCtx)   │     PersonReference    │  (References)    │
│                  │     OrgReference       └──────────────────┘
└──────────────────┘                                │
                                                    │
                                            ┌───────▼──────────┐
                                            │  Organization    │
                                            │  Context         │
                                            │  (internal)      │
                                            └──────────────────┘
```

## DDD Pattern Summary

| Pattern | Implementation | Purpose |
|---------|----------------|---------|
| Published Language | `*/published.rs` | Stable cross-context types |
| Anti-Corruption Layer | `*/acl.rs` | Translation between contexts |
| Ports & Adapters | `*ContextPort` traits | Dependency inversion |
| Context Map | Module structure | Clear boundaries |

## Test Results

| Category | Tests |
|----------|-------|
| Organization Published Language | 8 |
| PKI Published Language | 5 |
| PKI ACL | 7 |
| NATS ACL | 7 |
| Context Boundary Integration | 12 |
| **New Tests This Sprint** | **39** |
| **Total Library Tests** | 633 |

## Files Created/Modified

### New Files
- `src/domains/organization/published.rs` - Organization Published Language
- `src/domains/pki/published.rs` - PKI Published Language
- `src/domains/pki/acl.rs` - PKI Anti-Corruption Layer
- `src/domains/nats/acl.rs` - NATS Anti-Corruption Layer
- `tests/context_boundaries.rs` - Integration tests

### Restructured
- `src/domains/organization.rs` → `src/domains/organization/mod.rs`
- `src/domains/pki.rs` → `src/domains/pki/mod.rs`
- `src/domains/nats.rs` → `src/domains/nats/mod.rs`

## What Went Well

1. **Clear Pattern**: ACL + Published Language is a well-understood DDD pattern
2. **Test-First**: Wrote tests that verify boundaries before implementation
3. **Builder Pattern**: MockAdapters use builder pattern for easy test setup
4. **Self-Contained Types**: Published Language types need no imports

## Lessons Learned

1. **Module Structure**: Had to restructure `.rs` files to directories to support submodules
2. **Reference vs Entity**: Published Language types are lightweight references, not full entities
3. **Adapter Granularity**: One adapter per source context keeps responsibilities clear
4. **Port Design**: Ports should be minimal - only what the consumer needs

## Success Metrics

| Metric | Before | After |
|--------|--------|-------|
| Cross-context direct imports | Many | Possible to eliminate |
| Published Language types | 0 | 9 |
| ACL Port traits | 0 | 3 |
| Context boundary tests | 0 | 12 |
| Total tests | 606 | 633 (+27) |

## Next Steps

1. **Migrate Existing Code**: Update existing cross-context references to use Published Language
2. **Real Adapters**: Implement real adapters (not just mocks) for production use
3. **Documentation**: Create `doc/architecture/context-map.md`
4. **Lint Rules**: Add compile-time checks for cross-context imports

## FRP Axiom Compliance

| Axiom | Status | Notes |
|-------|--------|-------|
| A3 (Decoupling) | ✅ | ACLs decouple contexts |
| A5 (Totality) | ✅ | Ports return Option, never panic |
| A6 (Explicit Routing) | ✅ | Explicit port method calls |
| A9 (Composition) | ✅ | Contexts compose through ACLs |

## Commits

1. `feat(acl): add Published Language and Anti-Corruption Layers for bounded contexts`
