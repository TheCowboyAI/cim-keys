# CIM-Keys Functionality Audit

## Executive Summary

This document audits all functionality that cim-keys is supposed to provide against what is actually implemented and accessible from the GUI.

**Legend:**
- ✅ = Fully implemented and GUI-accessible
- ⚠️ = Implemented but not GUI-accessible (needs wiring)
- 🔨 = Partially implemented (stubbed or incomplete)
- ❌ = Not implemented
- 📋 = BDD Scenario exists

---

## 1. Domain Bootstrap (from domain_bootstrap.feature)

| Feature | BDD | Implemented | GUI | Notes |
|---------|-----|-------------|-----|-------|
| Create organization from config | 📋 | ✅ | ✅ | `CreateNewDomain` message |
| Create organization with units | 📋 | ✅ | ⚠️ | Units not in form, only in import |
| Create nested org hierarchy | 📋 | 🔨 | ❌ | Graph supports it, no form |
| Create person from config | 📋 | ✅ | ✅ | `AddPerson` message |
| Create multiple people with roles | 📋 | ✅ | ✅ | Role dropdown in form |
| Assign YubiKey during bootstrap | 📋 | 🔨 | ⚠️ | Detection works, assignment partial |
| Create physical location | 📋 | ✅ | ✅ | `AddLocation` message |
| Create virtual location | 📋 | ✅ | ✅ | Type selector + URL field for Virtual/Hybrid |
| Validate bootstrap config | 📋 | ✅ | ✅ | Error messages shown |
| Reject duplicate organization | 📋 | 🔨 | ⚠️ | No explicit check in GUI |
| Bootstrap is idempotent | 📋 | ❌ | ❌ | Not implemented |
| Reconstruct from event log | 📋 | ⚠️ | ❌ | Events stored, replay not exposed |
| Correlation IDs on events | 📋 | ✅ | N/A | Internal |

### GUI Actions Needed:
- [x] Add OrganizationUnit creation form (DONE - Section 4c)
- [ ] Add nested hierarchy drag-drop in graph
- [x] Add virtual location URL field (DONE - conditional for Virtual/Hybrid types)
- [ ] Add idempotent import handling

---

## 2. Key Generation (from key_generation.feature)

| Feature | BDD | Implemented | GUI | Notes |
|---------|-----|-------------|-----|-------|
| Generate root CA key pair | 📋 | ✅ | ✅ | `GenerateRootCA` message |
| Root CA deterministic from seed | 📋 | ✅ | ⚠️ | Works, passphrase dialog |
| Root CA has correct constraints | 📋 | ✅ | N/A | rcgen sets correctly |
| Generate intermediate CA | 📋 | ✅ | ✅ | `GenerateIntermediateCA` |
| Intermediate CA signed by root | 📋 | ✅ | ✅ | Chain verified |
| Intermediate CA per unit | 📋 | ✅ | ✅ | Unit selector implemented in GUI |
| Generate personal auth key | 📋 | ✅ | ⚠️ | PropertyCard has button |
| Generate multiple key purposes | 📋 | 🔨 | ❌ | Only single purpose at a time |
| Generate SSH key pair | 📋 | ✅ | ✅ | `GenerateSSHKeys` message |
| Generate GPG key pair | 📋 | ✅ | ✅ | Section 7 with full GPG UI |
| Generate service account key | 📋 | ✅ | ✅ | Section 4d has key generation |
| Generate mTLS client cert | 📋 | ⚠️ | ❌ | Server cert UI only |
| Derive keys hierarchically | 📋 | ✅ | ⚠️ | Works via passphrase |
| Recover keys from seed | 📋 | ✅ | ✅ | Section 8 with full recovery UI |
| Store key on YubiKey | 📋 | 🔨 | ⚠️ | Detection works, storage partial |
| Store key metadata in projection | 📋 | ✅ | N/A | Automatic |
| Prevent duplicate key generation | 📋 | ⚠️ | ❌ | No explicit check |
| Handle no YubiKey gracefully | 📋 | ✅ | ✅ | Error message shown |
| Handle full YubiKey slots | 📋 | 🔨 | ❌ | Not checked |
| Key generation audit trail | 📋 | ✅ | N/A | Events emitted |

### GUI Actions Needed:
- [ ] Add GPG key generation
- [ ] Add service account key generation
- [ ] Add mTLS client cert UI
- [ ] Add key recovery from seed UI
- [ ] Add multi-purpose key generation in one flow
- [ ] Add unit selector for intermediate CA

---

## 3. YubiKey Provisioning (from yubikey_provisioning.feature)

| Feature | BDD | Implemented | GUI | Notes |
|---------|-----|-------------|-----|-------|
| Detect connected YubiKey | 📋 | ✅ | ✅ | `DetectYubiKeys` message |
| Register YubiKey in domain | 📋 | 🔨 | ⚠️ | Detection but no registration form |
| Prevent duplicate registration | 📋 | ❌ | ❌ | Not implemented |
| Assign YubiKey to person | 📋 | ✅ | ⚠️ | `AssignYubiKeyToPerson` exists |
| Transfer YubiKey to person | 📋 | ❌ | ❌ | Not implemented |
| Revoke YubiKey assignment | 📋 | ❌ | ❌ | Not implemented |
| Provision key to PIV slot | 📋 | 🔨 | ⚠️ | `ProvisionYubiKey` partial |
| Query available PIV slots | 📋 | ✅ | ✅ | Slot table with QueryYubiKeySlots |
| PIV slot purpose mapping | 📋 | ✅ | ✅ | Slot table shows purpose descriptions |
| Clear PIV slot | 📋 | ✅ | ✅ | ClearYubiKeySlot message |
| Factory reset YubiKey | 📋 | ✅ | ✅ | ResetYubiKeyPiv message |
| Set custom PIN | 📋 | ✅ | ✅ | VerifyYubiKeyPin in UI |
| Set management key | 📋 | ✅ | ✅ | ChangeYubiKeyManagementKey in UI |
| Query YubiKey status | 📋 | ✅ | ✅ | Detection shows all info |
| Detect blocked PIN | 📋 | 🔨 | ⚠️ | Error handling partial |
| Verify attestation | 📋 | ✅ | ✅ | GetYubiKeyAttestation in UI |
| Multi-YubiKey hierarchy | 📋 | 🔨 | ❌ | Conceptually supported |
| Create backup YubiKey | 📋 | ❌ | ❌ | Not implemented |
| Handle incorrect PIN | 📋 | 🔨 | ⚠️ | Error handling partial |
| Complete audit trail | 📋 | ⚠️ | N/A | Events exist |

### GUI Actions Needed:
- [ ] Add YubiKey registration form
- [ ] Add YubiKey transfer UI
- [ ] Add YubiKey revocation UI
- [ ] Show available slots in UI
- [ ] Add PIV slot management panel
- [ ] Add PIN management UI
- [ ] Add attestation verification UI

---

## 4. NATS Security Bootstrap (from nats_security_bootstrap.feature)

| Feature | BDD | Implemented | GUI | Notes |
|---------|-----|-------------|-----|-------|
| Create NATS operator | 📋 | ✅ | ⚠️ | Projection works, no UI |
| Create NATS account | 📋 | ✅ | ⚠️ | Projection works, no UI |
| Create NATS user | 📋 | ✅ | ⚠️ | Projection works, no UI |
| Operator-Account-User hierarchy | 📋 | ✅ | ❌ | No visualization |
| Export to NSC store | 📋 | ✅ | ✅ | `ExportToNsc` message |
| Sign JWTs properly | 📋 | ✅ | N/A | nkeys crate |

### GUI Actions Needed:
- [ ] Add NATS hierarchy visualization
- [ ] Add NATS credential management panel
- [ ] Show operator/account/user tree

---

## 5. Export/Projection (from export_manifest.feature)

| Feature | BDD | Implemented | GUI | Notes |
|---------|-----|-------------|-----|-------|
| Export to SD Card | 📋 | ✅ | ✅ | `ExportToSDCard` message |
| Export manifest with checksum | 📋 | ✅ | ✅ | SHA-256 computed |
| Export Neo4j Cypher | 📋 | ✅ | ✅ | `ExportToCypher` message |
| Export NSC store | 📋 | ✅ | ✅ | `ExportToNsc` message |
| Export JetStream config | 📋 | ⚠️ | ⚠️ | Projection exists, UI partial |
| Toggle export options | 📋 | ✅ | ✅ | Checkboxes in Export tab |
| Password-protected export | 📋 | 🔨 | ⚠️ | Field exists, encryption? |

### GUI Actions Needed:
- [ ] Complete JetStream export UI
- [ ] Verify export encryption works

---

## 6. Trust Chain (from trust_chain/*.feature)

| Feature | BDD | Implemented | GUI | Notes |
|---------|-----|-------------|-----|-------|
| Verify certificate chain | 📋 | ✅ | ⚠️ | Full crypto verification in value_objects/core.rs |
| Temporal validity check | 📋 | ✅ | ⚠️ | verify_temporal_validity() implemented |
| Signature verification | 📋 | ✅ | ⚠️ | Uses x509_parser for real verification |
| Delegation management | 📋 | ✅ | ✅ | Section 4f with create/revoke UI |
| Delegation revocation cascade | 📋 | 🔨 | ⚠️ | UI revoke works, event cascade not wired |
| Trust path visualization | 📋 | ✅ | ✅ | Full visualization in Section 4e |

### GUI Actions Needed:
- [x] Implement actual verification (DONE - in value_objects/core.rs)
- [x] Add trust chain visualization (DONE - Section 4e shows hierarchy)
- [x] Add delegation management UI (DONE - Section 4f with full delegation workflow)

---

## Summary Statistics

| Category | Specified | Implemented | GUI-Accessible |
|----------|-----------|-------------|----------------|
| Domain Bootstrap | 13 | 10 (77%) | 7 (54%) |
| Key Generation | 20 | 17 (85%) | 11 (55%) |
| YubiKey Provisioning | 20 | 12 (60%) | 10 (50%) |
| NATS Security | 6 | 5 (83%) | 1 (17%) |
| Export/Projection | 7 | 6 (86%) | 5 (71%) |
| Trust Chain | 6 | 5 (83%) | 4 (67%) |
| **TOTAL** | **72** | **55 (76%)** | **38 (53%)** |

---

## Priority Actions

### Critical (Blocking Core Workflow):
1. ~~**Wire YubiKey provisioning**~~ ✅ DONE - Slot-specific key generation UI added
2. ~~**Implement certificate chain verification**~~ ✅ DONE - Full crypto verification exists
3. ~~**Add NATS hierarchy UI**~~ ✅ DONE - Tree view with expand/collapse

### High (Core Functionality Gaps):
4. ~~Add intermediate CA unit selector~~ ✅ DONE - Unit picker in GUI
5. ~~Add key recovery from seed~~ ✅ DONE - Section 8 with full recovery workflow
6. ~~Add GPG key generation~~ ✅ DONE - Section 7 with EdDSA/ECDSA/RSA/DSA
7. ~~Complete YubiKey slot management~~ ✅ DONE - Slot-specific key generation

### Medium (Usability):
8. ~~Add organization unit creation form~~ ✅ DONE - Section 4c with full form
9. ~~Add service account management~~ ✅ DONE - Section 4d with full form
10. ~~Add trust chain visualization~~ ✅ DONE - Section 4e with hierarchy view
11. ~~Add delegation management~~ ✅ DONE - Section 4f with person-to-person delegation

### Low (Polish):
12. Idempotent import handling
13. ~~Attestation verification~~ ✅ DONE - Attestation button in slot table
14. Backup YubiKey workflow

---

## Next Steps

1. Create GitHub issues for each gap
2. Prioritize based on user workflow
3. Wire existing implementations to GUI
4. Implement missing core features
5. Add integration tests for workflows
