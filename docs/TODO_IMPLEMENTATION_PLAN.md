# TODO Implementation Plan

**Total TODOs:** 87
**Date:** 2025-01-21
**Status:** Analysis Complete

This document provides a prioritized, phased plan to implement all TODO items in the codebase.

---

## Executive Summary

The codebase has 87 TODOs categorized into:
- **17 Critical Path** - Core functionality blockers
- **23 Security & Production** - Production readiness
- **28 Feature Completion** - Complete existing features
- **15 Polish & Enhancement** - User experience improvements
- **4 Documentation** - Code documentation

**Recommended Order:** Phases 1-3 (Critical + Security + Feature Completion) = Production Ready

---

## Phase 1: Critical Path (17 items) 🔴 HIGH PRIORITY

**Goal:** Unblock core functionality and make existing features work properly

**Estimated Effort:** 3-4 days

### 1.1 Aggregate Command Handling (3 items)
**Priority:** CRITICAL - Blocks command execution flow

**Files:**
- `src/aggregate.rs:8` - Fully refactor aggregate to coordinate new command handlers
- `src/aggregate.rs:35` - Implement actual command handling
- `src/aggregate.rs:91` - Re-implement command handlers for modular structure

**Implementation:**
```rust
// Current: Stub implementation
impl KeyManagementAggregate {
    pub fn handle_command(&mut self, cmd: KeyCommand) -> Result<Vec<KeyEvent>, Error> {
        // TODO: Implement
    }
}

// Target: Full command routing
impl KeyManagementAggregate {
    pub fn handle_command(&mut self, cmd: KeyCommand, ...) -> Result<Vec<KeyEvent>, Error> {
        match cmd {
            KeyCommand::CreateOrganization(cmd) => self.handle_create_org(cmd, ...),
            KeyCommand::CreatePerson(cmd) => self.handle_create_person(cmd, ...),
            KeyCommand::GenerateKey(cmd) => self.handle_generate_key(cmd, ...),
            // ... all command variants
        }
    }
}
```

**Success Criteria:**
- [ ] All KeyCommand variants have handlers
- [ ] Handlers emit proper events with correlation/causation
- [ ] Events applied to projection successfully
- [ ] Integration tests pass

---

### 1.2 Event Emitter Modernization (3 items)
**Priority:** HIGH - Needed for command flow

**Files:**
- `src/gui/event_emitter.rs:133` - Update to work with modular command structure
- `src/gui/event_emitter.rs:136` - Implement proper NATS subject building
- `src/gui/event_emitter.rs:365` - Update test to use current KeyCommand variants

**Implementation:**
```rust
// Target: Proper subject algebra
fn build_subject(&self, command: &KeyCommand) -> String {
    match command {
        KeyCommand::CreateOrganization(_) =>
            format!("{}.org.command.create", self.subject_prefix),
        KeyCommand::CreatePerson(_) =>
            format!("{}.person.command.create", self.subject_prefix),
        KeyCommand::GenerateKey(cmd) =>
            format!("{}.key.command.generate.{}", self.subject_prefix, cmd.key_type),
        // ... proper subject hierarchy
    }
}
```

**Success Criteria:**
- [ ] build_subject() handles all command types
- [ ] Subjects follow semantic naming: `org.unit.entity.operation`
- [ ] Tests updated and passing

---

### 1.3 Graph State Machine Population (1 item)
**Priority:** MEDIUM - Feature completion

**Files:**
- `src/gui.rs:790` - Populate Aggregates view with state machine nodes

**Implementation:**
```rust
GraphView::Aggregates => {
    self.org_graph.nodes.clear();
    self.org_graph.edges.clear();

    let state_machine = self.selected_aggregate.state_machine();
    populate_state_machine_graph(
        &mut self.org_graph,
        &state_machine,
        &StateMachineLayout::default()
    );

    self.status_message = format!(
        "State Machine: {} ({} states, {} transitions)",
        state_machine.name,
        state_machine.states.len(),
        state_machine.transitions.len()
    );
}
```

**Success Criteria:**
- [ ] State machine nodes render as circles
- [ ] Transitions render as directed edges with command labels
- [ ] Current state highlighted
- [ ] Terminal states visually distinct

---

### 1.4 Organization ID Resolution (2 items)
**Priority:** MEDIUM - Data consistency

**Files:**
- `src/gui.rs:1053` - Use actual org ID from config (not Uuid::now_v7())
- `src/domain.rs:399` - Get org from unit (proper hierarchy)

**Implementation:**
```rust
// Store org_id in CimKeysApp state
pub struct CimKeysApp {
    organization_id: Option<Uuid>, // Already exists
    // ...
}

// Use it consistently
CreatePersonCommand {
    organization_id: self.organization_id.unwrap_or_else(|| {
        // Fallback: get from domain config
        self.bootstrap_config.as_ref().map(|c| c.organization.id)
            .unwrap_or_else(Uuid::now_v7)
    }),
    // ...
}
```

**Success Criteria:**
- [ ] Single source of truth for organization_id
- [ ] All commands use consistent org_id
- [ ] No more Uuid::now_v7() for organization references

---

### 1.5 MVI Pattern Completion (10 items)
**Priority:** LOW-MEDIUM - Architecture consistency

**Files:**
- `src/mvi/update.rs:72` - Parse domain data properly
- `src/mvi/update.rs:336, 417` - Use actual CA IDs from model
- `src/mvi/update.rs:534` - Prompt user for YubiKey PIN
- `src/mvi/update.rs:649, 672, 700` - Save certificates via storage port
- `src/mvi/update.rs:805-889` - Complete graph interaction handlers (9 items)

**Implementation Strategy:**
Complete the MVI (Model-View-Intent) pattern for consistency. This is lower priority because the current implementation works, but MVI provides better architecture.

**Defer to Phase 3** unless blocking other work.

---

## Phase 2: Security & Production (23 items) 🟠 MEDIUM-HIGH PRIORITY

**Goal:** Make the system production-ready and secure

**Estimated Effort:** 5-7 days

### 2.1 Certificate Storage & Persistence (6 items)
**Priority:** HIGH - Security requirement

**Files:**
- `src/gui.rs:1641` - Store certificate PEM data in projection
- `src/gui.rs:1642` - Store private key securely (encrypted!)
- `src/gui.rs:1643` - Emit CertificateGeneratedEvent
- `src/mvi/update.rs:649, 672, 700` - Save via storage port (3 items)

**Implementation:**
```rust
// Secure storage pattern
pub fn store_certificate_with_private_key(
    &mut self,
    cert_pem: &str,
    private_key_pem: &str,
    passphrase: &SecureString,
) -> Result<(), ProjectionError> {
    // Encrypt private key with passphrase
    let encrypted_key = self.encrypt_private_key(private_key_pem, passphrase)?;

    // Store certificate (public, can be plaintext)
    let cert_path = self.root_path.join("certificates").join(format!("{}.pem", cert_id));
    fs::write(&cert_path, cert_pem)?;

    // Store encrypted private key
    let key_path = self.root_path.join("keys").join(format!("{}.encrypted", cert_id));
    fs::write(&key_path, encrypted_key)?;

    // Update manifest
    self.manifest.certificates.push(CertificateEntry { ... });
    self.save_manifest()?;

    Ok(())
}
```

**Success Criteria:**
- [ ] Certificates stored as PEM files
- [ ] Private keys encrypted with Argon2id + passphrase
- [ ] Manifest tracks all certificates
- [ ] Events emitted for audit trail
- [ ] Keys can be retrieved and decrypted

---

### 2.2 PKI Certificate Signing Chain (6 items)
**Priority:** HIGH - Core PKI functionality

**Files:**
- `src/adapters/x509_rcgen.rs:88, 140` - Use provided key (not generate new)
- `src/adapters/x509_rcgen.rs:177, 192, 253` - Implement proper CA signing (3 items)
- `src/adapters/x509_rcgen.rs:222, 281` - Sign with parent CA (2 items)

**Implementation:**
```rust
// Proper certificate signing
pub fn sign_certificate_request(
    &self,
    csr: &CertificateSigningRequest,
    ca_cert: &Certificate,
    ca_private_key: &PrivateKey,
    validity_days: u32,
) -> Result<Certificate, X509Error> {
    // Parse CSR
    let subject = csr.subject();
    let public_key = csr.public_key();

    // Build certificate from CA template
    let mut params = CertificateParams::default();
    params.subject_alt_names = csr.subject_alt_names();
    params.key_usages = csr.key_usages();

    // Sign with CA's private key
    let cert = params.signed_by(&ca_private_key, &ca_cert, &ring::signature::RSA_PKCS1_SHA256)?;

    Ok(cert)
}
```

**Success Criteria:**
- [ ] Root CA self-signed correctly
- [ ] Intermediate CAs signed by root
- [ ] Leaf certificates signed by intermediate
- [ ] Certificate chain validates
- [ ] Trust path verifiable from leaf to root

---

### 2.3 NATS JWT Signing (4 items)
**Priority:** MEDIUM-HIGH - NATS security

**Files:**
- `src/adapters/nsc.rs:173` - Use operator's signing key (not account's seed)
- `src/adapters/nsc.rs:242` - Use account's signing key (not user's seed)
- `src/domain_projections/nats.rs:706, 757, 948` - Load signing keys from metadata (3 items)

**Implementation:**
```rust
// Proper JWT signing hierarchy
pub fn create_account_jwt(
    &self,
    account: &NatsAccount,
    operator_signing_key: &SigningKey, // CRITICAL: Use operator's key
) -> Result<String, NatsError> {
    let claims = AccountClaims {
        aud: account.id.to_string(),
        iat: Utc::now().timestamp(),
        iss: operator.id.to_string(), // Issued by operator
        sub: account.id.to_string(),
        // ... claims
    };

    // Sign with OPERATOR's key, not account's
    let jwt = sign_claims(&claims, operator_signing_key)?;
    Ok(jwt)
}
```

**Success Criteria:**
- [ ] Operator JWTs self-signed with operator key
- [ ] Account JWTs signed with operator key
- [ ] User JWTs signed with account key
- [ ] Signing keys loaded from secure storage
- [ ] JWT validation succeeds

---

### 2.4 YubiKey Security & Attestation (4 items)
**Priority:** MEDIUM - Hardware security

**Files:**
- `src/commands/yubikey.rs:434` - Attest keys generated on device
- `src/commands/yubikey.rs:438` - Mark YubiKey as sealed/immutable
- `src/adapters/yubikey_cli.rs:143` - Implement certificate import
- `src/adapters/yubikey_cli.rs:154` - Implement signing

**Implementation:**
```rust
// YubiKey attestation
pub fn attest_key_generation(
    &self,
    slot: PIVSlot,
) -> Result<AttestationCertificate, YubiKeyError> {
    // Request attestation from YubiKey
    let attestation = self.yubikey.attest(slot)?;

    // Verify attestation chain to Yubico root
    self.verify_attestation_chain(&attestation)?;

    // Prove key was generated on-device (not imported)
    assert!(attestation.key_generated_on_device());

    Ok(attestation)
}

// Seal YubiKey (prevent further modifications)
pub fn seal_yubikey(&mut self) -> Result<(), YubiKeyError> {
    // Set management key to random value and discard
    let random_mgm_key = generate_random_management_key();
    self.yubikey.set_management_key(&random_mgm_key)?;

    // Don't store it - key is now immutable
    self.sealed = true;

    Ok(())
}
```

**Success Criteria:**
- [ ] Key attestation certificates generated
- [ ] Attestation chain verified to Yubico root
- [ ] Sealed YubiKeys cannot be modified
- [ ] Certificates imported to slots
- [ ] Signing operations work

---

### 2.5 Certificate Chain Verification (2 items)
**Priority:** MEDIUM - Trust validation

**Files:**
- `src/adapters/x509_rcgen.rs:307` - Parse certificate structure
- `src/adapters/x509_rcgen.rs:356` - Implement chain verification

**Implementation:**
```rust
pub fn verify_certificate_chain(
    &self,
    leaf: &Certificate,
    intermediates: &[Certificate],
    root: &Certificate,
) -> Result<(), X509Error> {
    // Build chain: leaf <- intermediates <- root
    let mut chain = vec![leaf];
    chain.extend(intermediates);
    chain.push(root);

    // Verify each link
    for i in 0..chain.len()-1 {
        let cert = &chain[i];
        let issuer = &chain[i+1];

        // Verify signature
        cert.verify_signature(issuer.public_key())?;

        // Verify validity dates
        cert.check_validity()?;

        // Verify constraints (CA can sign, path length)
        issuer.check_ca_constraints()?;
    }

    Ok(())
}
```

**Success Criteria:**
- [ ] Leaf certificates verify against intermediate
- [ ] Intermediate certificates verify against root
- [ ] Validity dates checked
- [ ] CA constraints enforced
- [ ] Invalid chains rejected

---

### 2.6 NATS Directory Structure (1 item)
**Priority:** LOW-MEDIUM - Deployment

**Files:**
- `src/bin/cim-keys.rs:256` - Create NATS server-compatible directory structure

**Implementation:**
```bash
# Target directory structure
$OUTPUT_DIR/
├── nsc/
│   ├── operators/
│   │   └── $OPERATOR_NAME/
│   │       ├── $OPERATOR_NAME.jwt
│   │       ├── accounts/
│   │       │   └── $ACCOUNT_NAME/
│   │       │       ├── $ACCOUNT_NAME.jwt
│   │       │       └── users/
│   │       │           └── $USER_NAME.jwt
│   └── keys/
│       ├── operators/
│       │   └── $OPERATOR_SEED.nk
│       └── accounts/
│           └── $ACCOUNT_SEED.nk
```

**Success Criteria:**
- [ ] Directory structure matches NSC/NATS expectations
- [ ] JWTs in correct locations
- [ ] Seeds secured properly
- [ ] NATS server can read configuration

---

## Phase 3: Feature Completion (28 items) 🟡 MEDIUM PRIORITY

**Goal:** Complete partially implemented features

**Estimated Effort:** 4-6 days

### 3.1 NATS Identity Generation (5 items)
**Priority:** MEDIUM - Core feature

**Files:**
- `src/gui.rs:1685` - Create NATS identity nodes
- `src/gui.rs:1686` - Store keys in projection
- `src/gui.rs:3449` - Generate NATS keys (Operator, Account, User)
- `src/gui/graph_nats.rs:246` - Generate actual NKeys and JWTs
- `src/domain_projections/nats.rs:1125` - Extract service accounts from org metadata

**Implementation:**
```rust
// Complete NATS identity workflow
pub fn generate_nats_identities(
    &mut self,
    org: &Organization,
) -> Result<NatsIdentitySet, Error> {
    // 1. Generate operator key
    let operator_key = self.nats_port.generate_operator_key()?;
    let operator_jwt = self.nats_port.create_operator_jwt(&operator_key, org)?;

    // 2. Generate account keys per organizational unit
    let mut accounts = Vec::new();
    for unit in &org.units {
        let account_key = self.nats_port.generate_account_key()?;
        let account_jwt = self.nats_port.create_account_jwt(
            &account_key,
            unit,
            &operator_key
        )?;
        accounts.push((unit.clone(), account_key, account_jwt));
    }

    // 3. Generate user keys per person
    let mut users = Vec::new();
    for person in &org.people {
        let user_key = self.nats_port.generate_user_key()?;
        let user_jwt = self.nats_port.create_user_jwt(
            &user_key,
            person,
            &find_account_for_person(person, &accounts)?
        )?;
        users.push((person.clone(), user_key, user_jwt));
    }

    // 4. Store in projection
    self.projection.store_nats_identities(operator_key, accounts, users)?;

    // 5. Create graph nodes
    self.create_nats_graph_nodes(&operator_key, &accounts, &users)?;

    Ok(NatsIdentitySet { operator_key, accounts, users })
}
```

**Success Criteria:**
- [ ] Operator key generated with org name
- [ ] Account keys generated per unit
- [ ] User keys generated per person
- [ ] JWTs properly signed
- [ ] Keys stored in projection
- [ ] NATS graph nodes created and visible

---

### 3.2 Person & Entity Management (3 items)
**Priority:** MEDIUM - UI completion

**Files:**
- `src/gui.rs:1266` - Remove person from graph and domain
- `src/gui.rs:2778` - Implement Role node type
- `src/gui.rs:2779` - Add NATS, PKI, YubiKey node types to context menu

**Implementation:**
```rust
// Person deletion with cascade
Message::DeletePerson(person_id) => {
    // 1. Find all relationships
    let relationships = self.org_graph.find_edges_for_node(person_id);

    // 2. Find all owned keys
    let keys = self.projection.find_keys_for_person(person_id);

    // 3. Emit domain events
    let events = vec![
        KeyEvent::PersonRemoved { person_id, ... },
        // Cascade: keys, relationships
    ];

    // 4. Update graph
    self.org_graph.remove_node(person_id);

    // 5. Update projection
    self.projection.apply_events(&events)?;

    Task::none()
}
```

**Success Criteria:**
- [ ] Person deletion works
- [ ] Cascade deletes keys, relationships
- [ ] Role nodes created via context menu
- [ ] All entity types in context menu

---

### 3.3 PKI Enhancements (3 items)
**Priority:** LOW-MEDIUM - Polish

**Files:**
- `src/gui.rs:3435` - Generate proper leaf cert signed by intermediate CA
- `src/gui/graph.rs:780` - Add SAN extraction from certificate extensions
- `src/gui/graph_pki.rs:148` - Implement topological sort for nested org units

**Defer to Phase 4** unless needed for other work.

---

### 3.4 Domain Validation (3 items)
**Priority:** LOW - Data integrity

**Files:**
- `src/domain.rs:331` - Check if responsible_person_id exists
- `src/domain.rs:454` - Check if agent owner_id exists
- `src/domain.rs:1035` - Implement timezone-aware business hours

**Defer to Phase 4** unless needed for production.

---

### 3.5 Graph & UI Enhancements (8 items)
**Priority:** LOW - UX polish

**Files:**
- `src/gui.rs:1920` - Implement SSH key generation
- `src/gui.rs:2622` - Track window size for bounds checking
- `src/gui.rs:2825` - Persist graph_events to NATS/IPLD
- `src/gui.rs:3019, 3055` - Publish to NATS (2 items)
- `src/gui.rs:4310` - Replace with real event store
- `src/gui.rs:4728` - Re-implement location type picker
- `src/gui.rs:6113, 6134` - Map org units to NATS accounts (2 items)

**Defer to Phase 4** (polish phase).

---

### 3.6 Adapter Implementations (6 items)
**Priority:** LOW - Optional adapters

**Files:**
- `src/adapters/mod.rs:32` - Implement real adapters for production
- `src/adapters/nats_publisher_stub.rs:6` - Full implementation
- `src/adapters/yubikey_hardware.rs:73` - Implement PIV key generation
- `src/gui/graph.rs:619, 643` - Connect based on org unit membership (2 items)
- `src/value_objects/core.rs:201` - Implement proper extension checking

**Defer to Phase 5** (optional).

---

## Phase 4: Polish & Enhancement (15 items) 🟢 LOW PRIORITY

**Goal:** Improve user experience and add nice-to-have features

**Estimated Effort:** 2-3 days

### 4.1 UX Improvements
- Window size tracking for proper bounds checking
- SSH key generation UI
- Location type picker with proper Display trait
- Graph event persistence to NATS/IPLD
- Real event store integration

### 4.2 NATS Publishing
- Publish events to NATS on graph changes
- Real-time collaboration support preparation
- Event streaming to other CIM components

### 4.3 Domain Validation
- Timezone-aware business hours checking
- Responsible person existence validation
- Agent owner validation
- Enhanced data integrity checks

### 4.4 PKI Polish
- SAN extraction from certificate extensions
- Topological sort for nested organizational units
- Enhanced certificate display information

---

## Phase 5: Documentation & Cleanup (4 items) 📝 LOW PRIORITY

**Goal:** Complete code documentation

**Estimated Effort:** 1 day

**Files:**
- `src/lib.rs:39` - Re-enable missing_docs warnings
- `src/lib.rs:101, 111, 140` - Re-export command types (3 items)

**Implementation:**
```rust
// Enable full documentation enforcement
#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]

// Re-export all command types
pub use commands::{
    KeyCommand,
    organization::{CreatePerson, CreateOrganization, /* ... */},
    pki::{GenerateCertificate, GenerateKeyPair, /* ... */},
    yubikey::{ProvisionYubiKeySlot, /* ... */},
    nats_identity::{CreateNatsOperator, /* ... */},
};
```

**Success Criteria:**
- [ ] All public API documented
- [ ] Examples in doc comments
- [ ] cargo doc builds without warnings
- [ ] Command types properly exported

---

## Implementation Strategy

### Recommended Approach

**Week 1: Critical Path (Phase 1)**
- Day 1-2: Aggregate command handling
- Day 3: Event emitter modernization
- Day 4: State machine population + org ID resolution

**Week 2: Security & Production (Phase 2)**
- Day 1-2: Certificate storage & persistence
- Day 3-4: PKI certificate signing chain
- Day 5: NATS JWT signing + YubiKey attestation

**Week 3: Feature Completion (Phase 3)**
- Day 1-3: NATS identity generation (complete workflow)
- Day 4: Person management + entity types
- Day 5: Review and integration testing

**Week 4: Polish & Documentation (Phases 4-5)**
- Day 1-2: UX improvements
- Day 3: Domain validation
- Day 4-5: Documentation + cleanup

---

## Testing Strategy

### Test Coverage Goals

| Phase | Existing Tests | New Tests Needed | Target Coverage |
|-------|---------------|------------------|-----------------|
| Phase 1 | 239 passing | +20 integration | 95% |
| Phase 2 | - | +30 security | 98% |
| Phase 3 | - | +15 feature | 98% |
| Phase 4 | - | +10 UX | 99% |
| Phase 5 | - | +5 integration | 99% |

### Test Types Needed

**Phase 1:**
- Aggregate command handler tests
- Event emission verification
- State machine rendering tests

**Phase 2:**
- Certificate encryption/decryption tests
- PKI chain validation tests
- JWT signing verification tests
- YubiKey attestation tests

**Phase 3:**
- NATS identity generation end-to-end tests
- Person management workflow tests
- Entity creation/deletion tests

---

## Risk Analysis

### High Risk Items

1. **Certificate Storage (Phase 2)** - Security critical
   - **Risk:** Private keys exposed
   - **Mitigation:** Argon2id encryption + secure zeroization

2. **PKI Signing Chain (Phase 2)** - Core functionality
   - **Risk:** Invalid certificates generated
   - **Mitigation:** Comprehensive chain validation tests

3. **Aggregate Refactoring (Phase 1)** - Major refactor
   - **Risk:** Breaking existing functionality
   - **Mitigation:** Incremental migration + extensive testing

### Medium Risk Items

4. **NATS JWT Signing (Phase 2)** - Security boundary
   - **Risk:** Wrong signing key used
   - **Mitigation:** Clear separation + validation tests

5. **YubiKey Operations (Phase 2)** - Hardware dependency
   - **Risk:** Device compatibility issues
   - **Mitigation:** Mock adapters + hardware testing matrix

---

## Success Metrics

### Phase Completion Criteria

**Phase 1 Complete When:**
- [ ] All KeyCommand variants handled
- [ ] Events emit with proper correlation/causation
- [ ] State machines render in Aggregates view
- [ ] 259/259 tests passing

**Phase 2 Complete When:**
- [ ] Certificates stored encrypted
- [ ] PKI chains validate correctly
- [ ] JWTs sign with proper keys
- [ ] YubiKey attestation works
- [ ] 289/289 tests passing

**Phase 3 Complete When:**
- [ ] NATS identities generate complete workflow
- [ ] Person management fully functional
- [ ] All entity types supported
- [ ] 304/304 tests passing

**Phases 4-5 Complete When:**
- [ ] UX polish complete
- [ ] Documentation 100%
- [ ] 319/319 tests passing
- [ ] cargo doc builds clean

---

## Next Steps

1. **Review this plan with stakeholders**
2. **Prioritize based on business needs**
3. **Create GitHub issues for each phase**
4. **Begin Phase 1 implementation**
5. **Daily standups to track progress**

---

## Appendix: Full TODO Inventory

### By File

**Aggregate (3 TODOs):**
- Refactor aggregate coordinator
- Implement command handling
- Re-implement modular command structure

**Adapters (13 TODOs):**
- Implement real adapters
- NSC key signing fixes (2)
- X509 signing chain (7)
- YubiKey operations (3)
- Extension checking (1)

**Commands (3 TODOs):**
- YubiKey attestation
- YubiKey sealing
- PKI implementation notes

**Domain (4 TODOs):**
- Person/agent existence checks (2)
- Unit-to-org lookup
- Timezone-aware hours

**GUI (41 TODOs):**
- Event emitter (3)
- Graph operations (6)
- MVI pattern (10)
- Certificate storage (3)
- NATS operations (5)
- UI enhancements (8)
- Person management (1)
- State machine population (1)
- YubiKey provisioning (1)
- SSH generation (1)
- Real event store (1)
- Location picker (1)

**Domain Projections (4 TODOs):**
- Load signing keys from metadata (3)
- Extract service accounts (1)

**Lib.rs (4 TODOs):**
- Documentation warnings
- Command re-exports (3)

**Secrets Loader (1 TODO):**
- Parse organizational units

**Total: 87 TODOs**

---

**Document Version:** 1.0
**Last Updated:** 2025-01-21
**Next Review:** After Phase 1 completion
