# CIM Keys - Implementation Plan

**Status**: In Progress
**Started**: 2025-11-12
**Goal**: Make CIM Keys fully functional for offline key generation and domain bootstrapping

---

## Overview

This document outlines the step-by-step plan to complete the CIM Keys implementation. Each step will be implemented, tested, committed, and marked complete before moving to the next.

---

## Step 1: Add Master Passphrase Input ✅ COMPLETE

### Goal
Add passphrase input for encrypting the offline key storage on SD cards.

### Requirements
- [x] Add `master_passphrase: String` field to `CimKeysApp` state
- [x] Add `master_passphrase_confirm: String` for validation
- [x] Add `MasterPassphraseChanged(String)` message
- [x] Add `MasterPassphraseConfirmChanged(String)` message
- [x] Add passphrase input fields in domain creation form (Organization tab)
- [x] Add visual indicator if passphrases don't match
- [x] Validate passphrase strength (min 12 chars, complexity)

### Files to Modify
- `src/gui.rs` - Add state fields and message handlers
- `src/gui.rs` - Update `view_organization()` to show passphrase inputs

### Testing
- [x] Passphrase fields appear in domain creation form
- [x] Validation works (matching, strength)
- [x] Build succeeds with no errors

### Completed
✅ **Commit**: ac6da97 - feat: add master passphrase input for encrypted storage

### Commit Message Template
```
feat: add master passphrase input for encrypted storage

- Add passphrase and confirmation fields to app state
- Add passphrase input UI in domain creation form
- Implement passphrase validation (matching + strength check)
- Visual feedback for passphrase requirements

Part of comprehensive implementation plan.
```

---

## Step 2: Implement Domain Creation ✅ PENDING

### Goal
Wire up "Create New Domain" button to actually create an Organization entity.

### Requirements
- [ ] Validate organization name and domain inputs
- [ ] Validate passphrase is set and valid
- [ ] Create `Organization` entity with UUID
- [ ] Create initial `OrganizationUnit` for the root
- [ ] Emit `DomainCreated` event
- [ ] Persist organization to projection
- [ ] Update `domain_loaded` flag to true
- [ ] Show success message

### Files to Modify
- `src/gui.rs` - Update `CreateNewDomain` message handler
- `src/projections.rs` - Add methods to store organization
- `src/events.rs` - Add `OrganizationCreatedEvent` if missing

### Testing
- [ ] Click "Create New Domain" with valid inputs
- [ ] Organization appears in projection files
- [ ] Domain loaded state switches to true
- [ ] UI shows organization info

### Commit Message Template
```
feat: implement domain creation with Organization entity

- Wire CreateNewDomain to create actual Organization
- Create root OrganizationUnit
- Persist organization to projection
- Validate all required inputs
- Emit OrganizationCreatedEvent
- Update UI to show created domain

Part of comprehensive implementation plan.
```

---

## Step 3: Wire Person Addition ✅ PENDING

### Goal
Make "Add Person" button actually create Person entities in the organization.

### Requirements
- [ ] Validate person name and email inputs
- [ ] Validate selected role
- [ ] Create `Person` entity with UUID
- [ ] Link person to organization
- [ ] Add person to graph visualization
- [ ] Persist person to projection
- [ ] Clear input fields after successful addition
- [ ] Show success message

### Files to Modify
- `src/gui.rs` - Implement `AddPerson` message handler
- `src/projections.rs` - Add methods to store person
- `src/gui/graph.rs` - Update graph with new person node
- `src/events.rs` - Add `PersonAddedEvent` if missing

### Testing
- [ ] Add multiple people with different roles
- [ ] People appear in graph visualization
- [ ] People persist to projection files
- [ ] Can select people for key assignment

### Commit Message Template
```
feat: implement person addition to organization

- Wire AddPerson to create actual Person entities
- Add people to graph visualization
- Persist people to projection
- Validate all person inputs
- Emit PersonAddedEvent
- Clear form after successful addition

Part of comprehensive implementation plan.
```

---

## Step 4: Add Locations Management ✅ PENDING

### Goal
Add a "Locations" tab for managing corporate locations where keys/certs are stored.

### Requirements
- [ ] Add `Locations` tab to `Tab` enum (between Organization and Keys)
- [ ] Add location management state fields
- [ ] Add `view_locations()` function
- [ ] Add UI for adding locations (name, type, security level)
- [ ] Add `AddLocation`, `RemoveLocation` messages
- [ ] Persist locations to projection
- [ ] Show list of added locations

### Location Types to Support
- DataCenter
- Office
- CloudRegion
- SafeDeposit
- SecureStorage
- HardwareToken (YubiKey)

### Files to Modify
- `src/gui.rs` - Add Locations tab and view
- `src/projections.rs` - Add location storage methods
- `src/events.rs` - Add location events

### Testing
- [ ] Locations tab appears and works
- [ ] Can add locations with different types
- [ ] Locations persist and reload
- [ ] Locations appear in key storage selection

### Commit Message Template
```
feat: add Locations management tab

- Add Locations tab between Organization and Keys
- Implement location addition/removal UI
- Support all location types (DataCenter, Office, etc.)
- Persist locations to projection
- Display location list with type and security level

Part of comprehensive implementation plan.
```

---

## Step 5: Test YubiKey Integration ✅ PENDING

### Goal
Test YubiKey detection, selection, and provisioning with real hardware.

### Requirements
- [ ] Connect physical YubiKey to test machine
- [ ] Test YubiKey detection (list available keys)
- [ ] Test YubiKey selection for person assignment
- [ ] Test "Provision YubiKey" button
- [ ] Verify PIV slot allocation
- [ ] Test key generation to YubiKey
- [ ] Handle YubiKey errors gracefully

### Files to Modify
- `src/adapters/yubikey_adapter.rs` - Replace mock with real implementation
- `src/gui.rs` - Add YubiKey detection UI
- May need to add system dependencies

### Testing Checklist
- [ ] YubiKey appears in detection list
- [ ] Can assign YubiKey to person
- [ ] Provisioning writes to correct slots
- [ ] Keys are hardware-backed
- [ ] Errors show meaningful messages

### Notes
- Requires `pcscd` daemon running on Linux
- Requires YubiKey Manager libraries
- May need elevated permissions

### Commit Message Template
```
feat: implement and test YubiKey integration

- Replace mock YubiKey adapter with real implementation
- Add YubiKey detection and selection UI
- Implement PIV slot provisioning
- Test hardware-backed key generation
- Add error handling for YubiKey operations

Part of comprehensive implementation plan.
```

---

## Step 6: End-to-End Workflow Test ✅ PENDING

### Goal
Validate the complete workflow from domain creation to export.

### Workflow Steps to Test
1. Create new domain with organization name + passphrase
2. Add 3-5 people with different roles
3. Add 2-3 locations (office, datacenter, yubikey)
4. Generate Root CA
5. Generate Intermediate CA
6. Generate server certificates
7. Assign YubiKeys to people
8. Generate SSH keys for people
9. Export to encrypted SD card
10. Verify all artifacts in export directory

### Requirements
- [ ] Complete workflow runs without errors
- [ ] All entities are created correctly
- [ ] Keys and certificates are generated
- [ ] Export contains all expected files
- [ ] Manifest is complete and valid
- [ ] Can reload domain from export

### Files to Review
- All projection files in output directory
- Manifest.json structure
- Certificate chain validity
- Key ownership mappings

### Testing Checklist
- [ ] Domain creation works
- [ ] People addition works
- [ ] Location management works
- [ ] Root CA generates successfully
- [ ] Intermediate CA generates and signs correctly
- [ ] Server certs generate with proper SAN
- [ ] YubiKey provisioning works
- [ ] SSH keys generate for all people
- [ ] Export creates all expected files
- [ ] Can load exported domain

### Issues to Document
- Any errors encountered
- Missing features discovered
- Performance concerns
- UX improvements needed

### Commit Message Template
```
test: validate complete end-to-end workflow

- Document complete workflow test results
- Verify domain creation through export
- Test all key generation types
- Validate YubiKey integration
- Confirm export structure and contents

Part of comprehensive implementation plan.
```

---

## Progress Tracking

### Completed Steps
- ✅ Step 1: Add Master Passphrase Input (Commit ac6da97)

### Current Step
Step 2: Implement Domain Creation

### Blockers
- None currently

### Next Session
Continue with Step 1 implementation

---

## Success Criteria

### Phase 1 (Steps 1-3): Core Functionality
- ✅ Can create domain with organization
- ✅ Can add people to organization
- ✅ Domain persists and reloads

### Phase 2 (Step 4): Infrastructure
- ✅ Can manage corporate locations
- ✅ Locations available for key storage selection

### Phase 3 (Step 5): Hardware Integration
- ✅ YubiKey detection works
- ✅ Can provision keys to YubiKey
- ✅ Hardware-backed keys function correctly

### Phase 4 (Step 6): Complete Workflow
- ✅ End-to-end workflow completes successfully
- ✅ All artifacts export correctly
- ✅ Exported domain can be imported by CIM infrastructure

---

## Notes

### Architecture Patterns
- Use event sourcing for all state changes
- Emit events before updating UI
- Persist to projection after events
- Keep UI reactive to projection state

### Code Quality
- Run `cargo build` after each change
- Run `cargo clippy` to check warnings
- Update CLAUDE.md with new patterns learned
- Keep commits atomic and focused

### Testing Strategy
- Manual testing for now (no automated tests yet)
- Test on actual YubiKey hardware when available
- Validate exported files manually
- Document any edge cases found

---

## Future Enhancements (Post-MVP)

- [ ] Import existing domain from JSON
- [ ] Certificate revocation functionality
- [ ] Automated key rotation
- [ ] Backup and recovery procedures
- [ ] Multi-signature key operations
- [ ] Integration with NATS infrastructure
- [ ] Real-time sync between multiple instances
- [ ] Audit log viewing and export
- [ ] Key usage tracking
- [ ] Certificate expiry notifications
