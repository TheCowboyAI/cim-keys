# Sprint 61 Retrospective: Certificate Bounded Context Extraction

**Date**: 2026-01-06
**Sprint Duration**: 1 session
**Status**: COMPLETED

## Sprint Goal

Extract the Certificate bounded context from gui.rs into a dedicated domain module. Certificate handles X.509 certificate generation, metadata management, intermediate CA creation, server certificates with SANs, and certificate chain viewing.

## Context

Sprint 60 extracted Multi-Purpose Key. Sprint 61 extracts Certificate as a proper bounded context for comprehensive X.509 PKI certificate management.

## What Was Accomplished

### 1. Directory Structure Created

```
src/gui/
├── certificate/
│   ├── mod.rs                 # Module exports (22 lines)
│   └── management.rs          # Certificate bounded context (~530 lines)
```

### 2. CertificateMessage Enum

Created domain-specific message enum with 20 variants organized by sub-domain:

| Sub-domain | Message Count | Purpose |
|------------|---------------|---------|
| UI State | 3 | Section visibility toggles |
| Metadata Form | 6 | X.509 certificate fields |
| Intermediate CA | 3 | Create and select intermediate CAs |
| Server Certificate | 4 | Server cert with SANs |
| Client Certificate | 2 | mTLS client cert generation |
| Chain View | 1 | Certificate chain visualization |
| Loading | 1 | Certificates loaded from manifest |

### 3. CertificateState Struct

Created domain state struct with 18 fields:

```rust
pub struct CertificateState {
    // UI State (3 fields)
    pub certificates_collapsed: bool,
    pub intermediate_ca_collapsed: bool,
    pub server_cert_collapsed: bool,

    // Metadata Form (6 fields)
    pub organization: String,
    pub organizational_unit: String,
    pub locality: String,
    pub state_province: String,
    pub country: String,
    pub validity_days: String,

    // Intermediate CA (3 fields)
    pub intermediate_ca_name: String,
    pub selected_intermediate_ca: Option<String>,
    pub selected_unit_for_ca: Option<String>,

    // Server Certificate (3 fields)
    pub server_cn: String,
    pub server_sans: String,
    pub selected_location: Option<String>,

    // Chain View (1 field)
    pub selected_chain_cert: Option<Uuid>,

    // Loaded Data (2 fields)
    pub loaded_certificates: Vec<CertificateEntry>,
    pub certificates_generated: usize,
}
```

### 4. X.509 Metadata Support

Full X.509 Distinguished Name (DN) fields:
- Organization (O)
- Organizational Unit (OU)
- Locality (L)
- State/Province (ST)
- Country (C)
- Validity period in days

### 5. Helper Methods

Added utility methods to CertificateState:

- `new()` - Creates state with defaults (365 day validity)
- `is_metadata_valid()` - Check required O and C fields
- `is_intermediate_ca_ready()` - Check name + metadata
- `is_server_cert_ready()` - Check CN + CA + metadata
- `metadata_validation_error()` - Get metadata error
- `intermediate_ca_validation_error()` - Get intermediate CA error
- `server_cert_validation_error()` - Get server cert error
- `validity_days_parsed()` - Parse validity to u32
- `validity_days_or_default()` - Get validity or 365
- `clear_intermediate_ca_form()` - Reset intermediate CA form
- `clear_server_cert_form()` - Reset server cert form
- `certificate_count()` - Get loaded cert count
- `find_certificate(id)` - Find by UUID
- `ca_certificates()` - Get CA certificates
- `leaf_certificates()` - Get non-CA certificates
- `intermediate_cas()` - Get intermediate CAs (has issuer)
- `certificate_subjects()` - Get subjects for dropdown
- `parsed_sans()` - Parse comma-separated SANs

### Files Modified

| File | Change |
|------|--------|
| `src/gui/certificate/mod.rs` | NEW: Certificate module exports (22 lines) |
| `src/gui/certificate/management.rs` | NEW: Certificate bounded context (~530 lines) |
| `src/gui.rs` | Added certificate module, Certificate Message variant, handler |

## Design Decisions

### 1. X.509 Metadata as Strings

All X.509 fields stored as strings for form input:
```rust
pub validity_days: String,  // Parse when needed
```

### 2. Layered Validation

Validation checks build on each other:
```rust
// Server cert requires intermediate CA + metadata
pub fn is_server_cert_ready(&self) -> bool {
    !self.server_cn.is_empty()
        && self.selected_intermediate_ca.is_some()
        && self.is_metadata_valid()
}
```

### 3. SAN Parsing

Subject Alternative Names stored as comma-separated string:
```rust
pub fn parsed_sans(&self) -> Vec<String> {
    self.server_sans
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

### 4. CA Detection via is_ca Field

Using `is_ca` boolean from CertificateEntry:
```rust
pub fn intermediate_cas(&self) -> Vec<&CertificateEntry> {
    self.loaded_certificates.iter()
        .filter(|c| c.is_ca && c.issuer.is_some())
        .collect()
}
```

## Tests Added

| Test | Purpose |
|------|---------|
| `test_certificate_state_default` | Default state values |
| `test_certificate_state_new` | Constructor defaults |
| `test_toggle_certificates_section` | Toggle visibility |
| `test_toggle_intermediate_ca` | Toggle intermediate CA section |
| `test_toggle_server_cert` | Toggle server cert section |
| `test_organization_changed` | Organization field update |
| `test_organizational_unit_changed` | OU field update |
| `test_locality_changed` | Locality field update |
| `test_state_province_changed` | State/Province update |
| `test_country_changed` | Country field update |
| `test_validity_days_changed` | Validity period update |
| `test_intermediate_ca_name_changed` | CA name update |
| `test_select_intermediate_ca` | CA selection |
| `test_server_cn_changed` | Server CN update |
| `test_server_sans_changed` | SANs update |
| `test_select_location` | Location selection |
| `test_select_for_chain_view` | Chain view selection |
| `test_is_metadata_valid` | Metadata validation |
| `test_is_intermediate_ca_ready` | CA readiness |
| `test_is_server_cert_ready` | Server cert readiness |
| `test_metadata_validation_error` | Metadata error messages |
| `test_intermediate_ca_validation_error` | CA error messages |
| `test_server_cert_validation_error` | Server cert error messages |
| `test_validity_days_parsed` | Parse validity |
| `test_validity_days_or_default` | Default validity |
| `test_clear_intermediate_ca_form` | Clear CA form |
| `test_clear_server_cert_form` | Clear server cert form |
| `test_parsed_sans` | Parse SANs |
| `test_parsed_sans_empty` | Empty SANs |
| `test_parsed_sans_with_empty_entries` | SANs with empty entries |

## Metrics

| Metric | Value |
|--------|-------|
| New files created | 2 |
| Lines added | ~550 |
| Tests passing | 1178 (up from 1148) |
| Message variants extracted | 20 |
| State fields extracted | 18 |
| Certificate-specific tests | 30 |

## Bug Fix During Sprint

**Issue**: Used non-existent `common_name` and `cert_type` fields
- CertificateEntry has `subject` not `common_name`
- CertificateEntry uses `is_ca` boolean, not `cert_type` string
- Fixed to use correct field names from projections.rs

## What Worked Well

1. **Comprehensive X.509 Support**: Full DN field coverage
2. **Layered Validation**: Each level builds on previous
3. **SAN Parsing**: Simple comma-separated format
4. **CA Detection**: Using existing is_ca + issuer pattern

## Lessons Learned

1. **Struct Field Verification**: Always check actual struct definitions
2. **Boolean vs Enum**: is_ca boolean simpler than cert_type string
3. **Optional Issuer**: Issuer presence distinguishes intermediate from root CAs

## Best Practices Updated

84. **Struct Field Audit**: Verify actual field names before using in helpers
85. **Boolean Filters**: Prefer boolean fields over string type comparisons
86. **Issuer-Based Classification**: Use issuer presence to detect intermediate CAs

## Progress Summary

| Sprint | Type | Module | Messages | State Fields | Tests |
|--------|------|--------|----------|--------------|-------|
| 48 | Domain | Organization | 50+ | 30+ | 991 |
| 49 | Domain | PKI | 55+ | 45+ | 998 |
| 50 | Domain | YubiKey | 40+ | 25+ | 1005 |
| 51 | Domain | NATS | 20+ | 14+ | 1014 |
| 52 | Port | Export | 15+ | 9+ | 1024 |
| 53 | Domain | Delegation | 9 | 6 | 1035 |
| 54 | Domain | TrustChain | 5 | 3 | 1053 |
| 55 | Domain | Location | 10 | 9 | 1068 |
| 56 | Domain | ServiceAccount | 11 | 6 | 1077 |
| 57 | Domain | GPG | 9 | 7 | 1096 |
| 58 | Domain | Recovery | 8 | 6 | 1112 |
| 59 | Domain | OrgUnit | 9 | 7 | 1130 |
| 60 | Domain | MultiKey | 5 | 4 | 1148 |
| 61 | Domain | Certificate | 20 | 18 | 1178 |
| **Total** | **13 domains, 1 port** | | **266+** | **189+** | **1178** |

## Next Steps (Sprint 62+)

1. **Event Log domain**: Event replay functionality
2. **Review gui.rs size**: Measure cumulative reduction
3. **Consolidation review**: Most major domains now extracted

## Sprint Summary

Sprint 61 successfully extracted the Certificate bounded context:
- Created certificate module with 20 message variants and 18 state fields
- Comprehensive X.509 DN field support (O, OU, L, ST, C)
- Layered validation for metadata, intermediate CA, and server certificates
- SAN parsing for Subject Alternative Names
- Added 30 new tests (total: 1178 passing)

Fourteen bounded contexts (Organization + PKI + YubiKey + NATS + Delegation + TrustChain + Location + ServiceAccount + GPG + Recovery + OrgUnit + MultiKey + Certificate) plus one Port (Export) now have clean separation from the main gui.rs module.
