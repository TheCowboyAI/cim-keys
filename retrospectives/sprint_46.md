# Sprint 46 Retrospective: Event Schema Simplification

**Date**: 2026-01-06
**Sprint Duration**: 2 sessions
**Status**: COMPLETED

## Sprint Goal

Remove unnecessary backward-compatibility complexity from event schemas. Since this is a new system with no existing serialized events to read, the legacy field coexistence pattern was identified as unnecessary complexity.

## Context

Sprint 45 completed the "dual-path" migration pattern with legacy string fields alongside typed ActorId fields. Upon review, we identified:
1. **Terminology Issue**: "Path" in the CIM domain means causation path (following causation_id), not migration patterns
2. **Unnecessary Complexity**: No old events exist to deserialize, making backward compatibility pointless

## Session 1: Key, Person, Organization Events

The first session simplified 21 event types across key, person, and organization modules by replacing the dual-path pattern with direct typed fields.

## Session 2: Certificate Events

The second session extended the simplification pattern to certificate events, which required more extensive refactoring:

1. **Actor Fields**: `CertificateRevokedEvent` and `CertificateRenewedEvent` used `String` for `revoked_by`/`renewed_by` - migrated to `ActorId`
2. **X.509 Value Objects**: `CertificateGeneratedEvent` had a complex dual-path structure with legacy string fields (`subject`, `san`, `key_usage`, etc.) alongside typed Optional fields - completely rewritten to use direct typed X.509 value objects

### CertificateGeneratedEvent Transformation

**Before (606+ lines of complexity):**
```rust
pub struct CertificateGeneratedEvent {
    #[deprecated]
    pub subject: String,
    pub subject_name: Option<SubjectName>,
    #[deprecated]
    pub san: Vec<String>,
    pub subject_alt_name: Option<SubjectAlternativeName>,
    // ... many more dual fields

    // Multiple constructors
    pub fn new_legacy(...) -> Self { ... }
    pub fn new_typed(...) -> Self { ... }

    // Value object accessors
    pub fn subject_value_object(&self) -> Result<SubjectName, ...> { ... }
    pub fn san_value_object(&self) -> Result<SubjectAlternativeName, ...> { ... }
}
```

**After (Direct typed fields):**
```rust
pub struct CertificateGeneratedEvent {
    pub cert_id: Uuid,
    pub key_id: Uuid,
    pub subject_name: SubjectName,
    pub subject_alt_name: Option<SubjectAlternativeName>,
    pub key_usage: KeyUsage,
    pub extended_key_usage: Option<ExtendedKeyUsage>,
    pub validity: CertificateValidity,
    pub basic_constraints: BasicConstraints,
    pub issuer: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}
```

### Session 2 Files Modified

| File | Change |
|------|--------|
| `src/events/certificate.rs` | Simplified 4 event types, direct typed fields |
| `src/crypto/x509.rs` | Updated certificate generation with value objects |
| `src/certificate_service.rs` | Use typed value object field access |
| `src/commands/pki.rs` | Renamed rcgen import, typed event construction |
| `src/commands/yubikey.rs` | Typed certificate event construction |
| `src/projections.rs` | Extract values from typed objects |
| `src/domain/aggregates.rs` | Use typed field access |
| `src/domain/nats/saga_command_handler.rs` | Update test assertions |
| `src/gui.rs` | Update metadata saving |
| `tests/certificate_events.rs` | Update sample helpers |
| `tests/complete_bootstrap_workflow.rs` | Update assertions |

### API Corrections Made

During Session 2, several API corrections were needed:

| Incorrect Usage | Correct Usage |
|-----------------|---------------|
| `SubjectName::builder()` | `SubjectName::new(CommonName).with_*()` |
| `KeyUsage::signing()` | `KeyUsage::code_signing()` |
| `KeyUsage::ca()` | `KeyUsage::ca_certificate()` |
| `ExtendedKeyUsage::server_auth()` | `ExtendedKeyUsage::tls_server()` |
| `SubjectAlternativeName::from_dns_names()` | `SubjectAlternativeName::new().with_dns_name()` |
| `cert.subject_name.common_name()` | `cert.subject_name.common_name.as_str()` |
| `key_usage.has_any()` | `key_usage.bits().count() > 0` |

## Changes Made (Combined)

### Event Simplification Pattern

**Before (Removed):**
```rust
pub struct KeyGeneratedEvent {
    #[deprecated]
    pub generated_by: String,               // Legacy field
    pub generated_by_actor: Option<ActorId>, // Typed field
    // ... other fields
}

impl KeyGeneratedEvent {
    pub fn new_legacy(...) -> Self { ... }
    pub fn new_typed(...) -> Self { ... }
    pub fn generated_by_value_object(&self) -> Result<ActorId, ...> { ... }
}
```

**After (Simplified):**
```rust
pub struct KeyGeneratedEvent {
    pub generated_by: ActorId,  // Direct typed field
    // ... other fields
}
```

### Files Modified

| File | Change |
|------|--------|
| `src/events/key.rs` | Simplified 10 event types (447 → 211 lines) |
| `src/events/person.rs` | Simplified 2 event types (345 → 217 lines) |
| `src/events/organization.rs` | Simplified 10 event types (529 → 322 lines) |
| `src/commands/organization.rs` | Use direct struct init with ActorId |
| `src/commands/pki.rs` | Use direct struct init with ActorId |
| `src/commands/export.rs` | Use direct ActorId field |
| `src/domain/nats/saga_command_handler.rs` | Use direct ActorId field |
| `src/projections.rs` | Convert ActorId to string for file storage |
| `src/event_store.rs` | Update test events with ActorId |
| `src/projection/jetstream.rs` | Update test events with ActorId |
| `src/adapters/nats_client.rs` | Update test events with ActorId |
| `src/domain/nats/publisher.rs` | Update test events with ActorId |
| `tests/key_events.rs` | Update sample event helpers with ActorId |

### Events Simplified

| Event Type | Fields Removed |
|------------|----------------|
| KeyGeneratedEvent | `generated_by: String`, `generated_by_actor: Option<ActorId>` → `generated_by: ActorId` |
| KeyImportedEvent | `imported_by: String`, `imported_by_actor: Option<ActorId>` → `imported_by: ActorId` |
| KeyExportedEvent | `exported_by: String`, `exported_by_actor: Option<ActorId>` → `exported_by: ActorId` |
| KeyRevokedEvent | `revoked_by: String`, `revoked_by_actor: Option<ActorId>` → `revoked_by: ActorId` |
| KeyRotationInitiatedEvent | `initiated_by: String`, `initiated_by_actor: Option<ActorId>` → `initiated_by: ActorId` |
| SshKeyGeneratedEvent | `generated_by: String`, `generated_by_actor: Option<ActorId>` → `generated_by: ActorId` |
| GpgKeyGeneratedEvent | `generated_by: String`, `generated_by_actor: Option<ActorId>` → `generated_by: ActorId` |
| PersonCreatedEvent | `created_by: Option<String>`, `created_by_actor: Option<ActorId>` → `created_by: ActorId` |
| PersonUpdatedEvent | `updated_by: String`, `updated_by_actor: Option<ActorId>` → `updated_by: ActorId` |
| OrganizationUpdatedEvent | Similar pattern |
| OrganizationalUnitCreatedEvent | Similar pattern |
| OrganizationalUnitUpdatedEvent | Similar pattern |
| OrganizationalUnitDissolvedEvent | Similar pattern |
| RoleCreatedEvent | Similar pattern |
| RoleUpdatedEvent | Similar pattern |
| RoleDeletedEvent | Similar pattern |
| PolicyCreatedEvent | Similar pattern |
| PolicyUpdatedEvent | Similar pattern |
| PolicyRevokedEvent | Similar pattern |
| **Session 2 (Certificate Events)** | |
| CertificateGeneratedEvent | Complete rewrite: dual-path → direct typed X.509 value objects |
| CertificateRevokedEvent | `revoked_by: String` → `revoked_by: ActorId` |
| CertificateRenewedEvent | `renewed_by: String` → `renewed_by: ActorId` |
| PkiHierarchyCreatedEvent | `created_by: String` → `created_by: ActorId` |

## Removed Code

- `#[deprecated]` attributes on legacy fields
- `#[allow(deprecated)]` on event construction sites
- `new_legacy()` / `new_typed()` constructors
- `*_value_object()` accessor methods
- `#[serde(default)]` / `#[serde(skip_serializing_if)]` for migration

## Metrics

| Metric | Session 1 | Session 2 | Total |
|--------|-----------|-----------|-------|
| Lines Removed (net) | ~500 | ~600 | ~1100 |
| Events Simplified | 21 | 4 | 25 |
| Files Modified | 13 | 11 | 24 |
| Tests Passing | 985 lib + 24 int | All passing | All passing |

## What Worked Well

1. **Correct Domain Language**: User caught terminology confusion ("path" = causation path)
2. **Pragmatic Simplification**: No backward compatibility needed = remove the complexity
3. **Comprehensive Testing**: All tests updated and passing
4. **Clear Pattern**: Direct `ActorId` fields are simpler to understand and use
5. **Consistent Application**: Session 2 applied same pattern to certificate events
6. **Type-Safe X.509**: Value objects provide compile-time guarantees for certificate data

## Lessons Learned

1. **Question Backward Compatibility Early**: For new systems, backward compat may be YAGNI
2. **Domain Language Matters**: Use established domain terms, avoid confusing naming
3. **Complexity Has Cost**: Migration patterns add significant code overhead
4. **Test-First Helps**: Comprehensive tests made the refactoring safe
5. **API Discovery**: Read actual value object APIs before assuming builder patterns
6. **Consistent Refactoring**: Apply simplification patterns uniformly across all event types

## Best Practices Updated

37. **New Systems Don't Need Backward Compat**: If no old events exist, don't add migration complexity
38. **Use Domain Terminology**: "Causation path" not "dual-path" in CIM context
39. **Direct Typed Fields**: Prefer `field: Type` over `field_legacy: String` + `field_typed: Option<Type>`
40. **Use Correct Value Object APIs**: Check actual API (e.g., `SubjectName::new()` not `::builder()`)
41. **Field Access Over Methods**: Use `struct.field.as_str()` not `struct.field()`

## Follow-up Items

None - Sprint 46 completes the event schema simplification across ALL event types (key, person, organization, certificate).
