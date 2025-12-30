# CLAN NSC Export Gap Analysis

**Date**: 2025-11-24
**Status**: 🔴 **Analysis Complete - Implementation Needed**

## Executive Summary

cim-keys currently exports NATS credentials as flat `.creds` files. CLAN infrastructure requires NSC (NATS Security) hierarchical directory structure with separate Operator/Account/User JWTs.

## Current State

### Domain Model (src/domain.rs)

```rust
Organization {
    id: Uuid,
    name: String,
    units: Vec<OrganizationUnit>,
    // ...
}

OrganizationUnit {
    id: Uuid,
    name: String,
    unit_type: OrganizationUnitType, // Division, Department, Team, etc.
    parent_unit_id: Option<Uuid>,
    responsible_person_id: Option<Uuid>,
}

Person {
    id: Uuid,
    name: String,
    email: String,
    roles: Vec<PersonRole>, // RoleType: Executive, Administrator, Service, etc.
    unit_ids: Vec<Uuid>,
    organization_id: Uuid,
    active: bool,
}
```

**✅ Good**: Domain model already supports the hierarchical structure needed.

### Current Export (src/commands/export.rs)

**Export Structure**:
```
/output/
├── manifest.json
├── keys/
│   └── {key-id}/
├── certificates/
│   └── {cert-id}/
└── nats/
    └── {identity-id}.creds (FLAT - no hierarchy)
```

**NATS Credentials Format** (lines 389-402):
```rust
format!(
    "-----BEGIN NATS {TYPE} JWT-----\n{}\n------END NATS {TYPE} JWT------\n\n\
     -----BEGIN {TYPE} NKEY SEED-----\n{}\n------END {TYPE} NKEY SEED------\n",
    nats_item.jwt.token(),
    nats_item.nkey.seed_string(),
)
```

**🔴 Gap**: No NSC directory hierarchy, all credentials in flat structure.

### Manifest Structure (lines 467-502)

```rust
ExportManifest {
    manifest_id: Uuid,
    organization_id: Uuid,
    organization_name: String,
    keys: Vec<ManifestKeyEntry>,
    certificates: Vec<ManifestCertEntry>,
    nats_configs: Vec<ManifestNatsEntry>, // Flat list
    events_count: usize,
}
```

**🔴 Gap**: No concept of Operator → Account → User hierarchy.

## CLAN Requirements

### NSC Store Structure (Required)

```
nsc/stores/thecowboyai/
├── thecowboyai.jwt                    # Operator JWT (NEW)
├── accounts/                          # Account hierarchy (NEW)
│   ├── core/
│   │   ├── core.jwt                   # Account JWT (NEW)
│   │   └── users/
│   │       ├── organization-service.creds
│   │       ├── person-service.creds
│   │       ├── location-service.creds
│   │       ├── policy-service.creds
│   │       └── agent-service.creds
│   ├── media/
│   │   ├── media.jwt
│   │   └── users/
│   │       ├── audio-service.creds
│   │       ├── document-service.creds
│   │       └── video-service.creds
│   └── development/
│       ├── development.jwt
│       └── users/
│           ├── git-service.creds
│           ├── nix-service.creds
│           └── spaces-service.creds
└── keys/
    └── ... (private nkey seeds - SECURE)
```

### Mapping Logic (Required)

| CIM Domain Entity | NATS Entity | Example |
|-------------------|-------------|---------|
| **Organization** | NATS Operator | `thecowboyai` |
| **OrganizationUnit** (Service type) | NATS Account | `Core`, `Media`, `Development` |
| **Person** (Service role) | NATS User | `organization-service`, `person-service` |

### Domain Service Mapping (from CLAN-IMPLEMENTATION-COMPLETE.md)

| Domain Module | OrgUnit | NATS Account | Service Person |
|---------------|---------|--------------|----------------|
| cim-domain-organization | Core | `core` | organization-service |
| cim-domain-person | Core | `core` | person-service |
| cim-domain-location | Core | `core` | location-service |
| cim-domain-policy | Core | `core` | policy-service |
| cim-domain-agent | Core | `core` | agent-service |
| cim-domain-audio | Media | `media` | audio-service |
| cim-domain-document | Media | `media` | document-service |
| cim-domain-video | Media | `media` | video-service |
| cim-domain-git | Development | `development` | git-service |
| cim-domain-nix | Development | `development` | nix-service |
| cim-domain-spaces | Development | `development` | spaces-service |

### NATS Permissions (Required)

Each service needs NATS publish/subscribe permissions:

```json
{
  "publish": ["thecowboyai.org.{domain}.>"],
  "subscribe": ["thecowboyai.org.{domain}.>", "thecowboyai.org.{related-domain}.>"],
  "allow_responses": true,
  "max_payload": 1048576
}
```

Example: `person-service`:
- Publish: `thecowboyai.org.person.>`
- Subscribe: `thecowboyai.org.person.>`, `thecowboyai.org.location.>`

## Identified Gaps

### 1. **Missing: Operator JWT Generation** 🔴

**Current**: No operator-level JWT generated
**Required**: `thecowboyai.jwt` at root of NSC store
**Action**: Create `GenerateOperatorJwt` command that:
- Generates operator nkey
- Creates JWT with organization metadata
- Exports to `{nsc-store}/thecowboyai.jwt`

### 2. **Missing: Account JWT Generation** 🔴

**Current**: No account-level JWTs
**Required**: `{account-name}.jwt` in each account directory
**Action**: Create `GenerateAccountJwt` command that:
- Maps OrganizationUnit → NATS Account
- Generates account nkey
- Creates JWT with account permissions/limits
- Exports to `{nsc-store}/accounts/{account-name}/{account-name}.jwt`

### 3. **Missing: NSC Directory Structure** 🔴

**Current**: Flat directory with individual `.creds` files
**Required**: Hierarchical NSC store structure
**Action**: Update `handle_export_to_encrypted_storage` to:
- Create `nsc/stores/{operator}/` directory tree
- Place files in correct locations per NSC spec
- Generate proper manifest tracking hierarchy

### 4. **Missing: OrganizationUnit → Account Mapping** 🔴

**Current**: No explicit mapping logic
**Required**: Map OrgUnits to NATS Accounts with conventions
**Action**: Add mapping configuration:
```rust
pub struct NatsAccountMapping {
    pub org_unit_id: Uuid,
    pub account_name: String,  // "core", "media", "development"
    pub account_type: AccountType,
}
```

### 5. **Missing: Service Person Identification** 🔴

**Current**: All People treated equally
**Required**: Identify Service-role persons for NATS User generation
**Action**: Filter `Person` by `RoleType::Service` and map to NATS Users

### 6. **Missing: NATS Permissions Generation** 🔴

**Current**: No NATS-specific permissions
**Required**: Translate Person roles → NATS publish/subscribe subjects
**Action**: Create `NatsPermissionsGenerator` that:
- Reads Person roles/permissions
- Maps to NATS subject patterns
- Generates appropriate limits (max_payload, etc.)

### 7. **Missing: Private Key Storage** 🔴

**Current**: No separate private key storage
**Required**: `keys/` directory with secure nkey seeds
**Action**: Store private nkeys in `{nsc-store}/keys/` with:
- Proper permissions (0400 - read-only by owner)
- Separate from credentials
- Indexed by key ID

## Implementation Plan

### Phase 1: Domain Model Extensions (2-3 hours)

1. **Add NatsAccountMapping to OrganizationUnit**
   ```rust
   pub struct OrganizationUnit {
       // ... existing fields
       pub nats_account_name: Option<String>, // "core", "media", "development"
   }
   ```

2. **Add NATS permissions to Person**
   ```rust
   pub struct Person {
       // ... existing fields
       pub nats_permissions: Option<NatsPermissions>,
   }

   pub struct NatsPermissions {
       pub publish: Vec<String>,
       pub subscribe: Vec<String>,
       pub allow_responses: bool,
       pub max_payload: Option<usize>,
   }
   ```

### Phase 2: Operator/Account JWT Generation (4-5 hours)

1. **Create `GenerateNatsOperator` command**
   - Input: Organization
   - Output: Operator JWT + nkey
   - Event: `NatsOperatorCreatedEvent`

2. **Create `GenerateNatsAccount` command**
   - Input: OrganizationUnit with account mapping
   - Output: Account JWT + nkey
   - Event: `NatsAccountCreatedEvent`

3. **Update `GenerateNatsUser` command**
   - Input: Person (Service role) with permissions
   - Output: User JWT + nkey + .creds file
   - Event: `NatsUserCreatedEvent`

### Phase 3: NSC Export Refactoring (3-4 hours)

1. **Create `NscExportAdapter`**
   ```rust
   pub struct NscExportAdapter {
       pub base_path: PathBuf,  // /path/to/nsc/stores/
   }

   impl NscExportAdapter {
       pub fn export_operator(&self, org: &Organization) -> Result<()>;
       pub fn export_account(&self, unit: &OrganizationUnit) -> Result<()>;
       pub fn export_user(&self, person: &Person, account: &str) -> Result<()>;
       pub fn create_nsc_structure(&self, operator: &str) -> Result<()>;
   }
   ```

2. **Update `handle_export_to_encrypted_storage`**
   - Add `export_as_nsc: bool` flag
   - If true, use `NscExportAdapter` instead of flat export
   - Generate complete NSC hierarchy

3. **Update manifest structure**
   ```rust
   pub struct NscManifest {
       operator: OperatorInfo,
       accounts: Vec<AccountInfo>,
       users: Vec<UserInfo>,
       // ... hierarchy metadata
   }
   ```

### Phase 4: Integration Testing (2-3 hours)

1. **Create test organization with 11 services**
   - 3 OrgUnits (Core, Media, Development)
   - 11 Service persons mapped to correct accounts
   - Proper NATS permissions for each

2. **Generate NSC export**
   - Verify directory structure matches NSC spec
   - Validate JWT formats
   - Check file permissions

3. **Test with CLAN infrastructure**
   - Import credentials into CLAN
   - Verify services can connect to NATS
   - Test publish/subscribe permissions

### Phase 5: Documentation & Examples (1-2 hours)

1. **Create example configuration** (`examples/clan-bootstrap.json`)
2. **Update README with NSC export instructions**
3. **Add CLAN integration guide**

## Breaking Changes

1. **Export format change**: New `--export-format nsc` flag required for NSC output
2. **Manifest structure**: New NSC-specific manifest format
3. **Domain model**: OrgUnit and Person gain optional NATS metadata fields

## Backward Compatibility

**Preserve existing flat export**: Keep current export as default, add NSC as opt-in:

```rust
pub enum ExportFormat {
    Flat,        // Current format (default)
    NscStore,    // New NSC hierarchical format
}
```

## Success Criteria

- [ ] NSC store structure matches spec exactly
- [ ] Operator/Account/User JWTs validate with `nsc`
- [ ] All 11 domain services have proper credentials
- [ ] CLAN infrastructure can consume exports
- [ ] Services connect to NATS with correct permissions
- [ ] Backward compatibility maintained for existing exports

## Next Steps

1. ✅ Complete gap analysis (this document)
2. ⏳ Review with team / get approval
3. ⏳ Implement Phase 1 (domain extensions)
4. ⏳ Implement Phase 2 (JWT generation)
5. ⏳ Implement Phase 3 (NSC export)
6. ⏳ Integration testing with CLAN
7. ⏳ Documentation updates

---

**Recommendation**: Implement NSC export as a new export format option rather than replacing the existing flat export. This preserves backward compatibility while enabling CLAN integration.
