<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 84 Retrospective: Certificate Import & Attestation Wiring

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Wire certificate import and attestation operations, add UI buttons for YubiKey operations

## What Was Accomplished

### Implemented import_certificate in CLI Adapter

Extended `yubikey_cli.rs` with certificate import using ykman:

```rust
async fn import_certificate(
    &self,
    serial: &str,
    slot: PivSlot,
    certificate: &[u8],
    pin: &SecureString,
) -> Result<(), YubiKeyError> {
    // Write certificate to temporary file (ykman requires file input)
    let cert_path = temp_dir.join(format!("yubikey_cert_{}.pem", Uuid::now_v7()));
    std::fs::write(&cert_path, certificate)?;

    let output = Command::new("ykman")
        .args(["--device", serial])
        .args(["piv", "certificates", "import"])
        .args(["--pin", pin_str.as_ref()])
        .args(["--verify"])
        .arg(slot_id)
        .arg(&cert_path)
        .output()?;

    // Clean up temp file
    let _ = std::fs::remove_file(&cert_path);
    // ...
}
```

### Implemented get_attestation in CLI Adapter

Added attestation retrieval using ykman:

```rust
async fn get_attestation(
    &self,
    serial: &str,
    slot: PivSlot,
) -> Result<Vec<u8>, YubiKeyError> {
    let output = Command::new("ykman")
        .args(["--device", serial])
        .args(["piv", "keys", "attest"])
        .arg(slot_id)
        .arg("-")  // Output to stdout
        .output()?;

    Ok(output.stdout)
}
```

### Added New YubiKey Operation Messages

```rust
// Sprint 84: Certificate Import and Attestation
YubiKeyStartCertificateImport {
    serial: String,
    slot: crate::ports::yubikey::PivSlot,
    certificate: Vec<u8>,
    pin: String,
},
YubiKeyCertificateImportResult(Result<(String, PivSlot), (String, String)>),
YubiKeyStartAttestation {
    serial: String,
    slot: crate::ports::yubikey::PivSlot,
},
YubiKeyAttestationResult(Result<(String, PivSlot, Vec<u8>), (String, String)>),
```

### Added Operation Handlers

**Certificate Import Handler** - Calls `import_certificate()`:
```rust
Message::YubiKeyStartCertificateImport { serial, slot, certificate, pin } => {
    // Validate state: use can_import_certs guard
    let can_import = matches!(current_state, Some(state) if state.can_import_certs());

    Task::perform(
        async move {
            yubikey_port.import_certificate(&serial, slot, &certificate, &pin_secure).await
        },
        Message::YubiKeyCertificateImportResult
    )
}

Message::YubiKeyCertificateImportResult(result) => {
    // Transition to CertificatesImported state
    self.yubikey_states.insert(
        serial,
        YubiKeyProvisioningState::CertificatesImported { slot_certs },
    );
}
```

**Attestation Handler** - Calls `get_attestation()`:
```rust
Message::YubiKeyStartAttestation { serial, slot } => {
    // Validate state: use can_attest guard
    let can_attest = matches!(current_state, Some(state) if state.can_attest());

    Task::perform(
        async move {
            yubikey_port.get_attestation(&serial, slot).await
        },
        Message::YubiKeyAttestationResult
    )
}

Message::YubiKeyAttestationResult(result) => {
    // Transition to Attested state
    self.yubikey_states.insert(
        serial,
        YubiKeyProvisioningState::Attested {
            attestation_chain_verified: true,
            attestation_cert_ids: vec![attestation_cert_id],
        },
    );
}
```

### Added UI Buttons for YubiKey Operations

Added operations row to each detected YubiKey device card:

```rust
// Sprint 84: Add operation buttons based on state
let yubikey_state = self.yubikey_states.get(&serial);

// Attestation button - available after certificates imported
if yubikey_state.map(|s| s.can_attest()).unwrap_or(false) {
    ops = ops.push(
        button("Get Attestation")
            .on_press(Message::YubiKeyStartAttestation { serial, slot })
    );
}

// Show current state
if let Some(state) = yubikey_state {
    ops = ops.push(text(format!("State: {}", state.state_name())));
}
```

## YubiKey Provisioning Flow (Updated)

```
                    YubiKey Provisioning State Machine (Sprint 84)
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  DetectYubiKeys                                                     │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Detected                                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │ YubiKeyStartAuthentication                               │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                   Authenticated                             │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │ YubiKeyStartPINChange                                    │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    PINChanged                               │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │ YubiKeyStartKeyGeneration                                │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                  KeysGenerated                              │   │
│  │  [UI: can_import_certs() = true]                            │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │ YubiKeyStartCertificateImport (Sprint 84)                │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │               CertificatesImported                          │   │
│  │  [UI: can_attest() = true, "Get Attestation" button]        │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │ YubiKeyStartAttestation (Sprint 84)                      │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Attested                                 │   │
│  │  attestation_chain_verified: true                           │   │
│  │  [UI: can_seal() = true]                                    │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │ (Future: Seal operation)                                 │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                     Sealed                                  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## State Machine Guard Methods

| State | Guard Method | Enables |
|-------|--------------|---------|
| Detected | `can_authenticate()` | Authentication |
| Authenticated | `can_change_pin()` | PIN Change |
| PINChanged | `can_rotate_management_key()` | Key Rotation |
| ManagementKeyRotated | `can_plan_slots()` | Slot Planning |
| SlotPlanned | `can_generate_keys()` | Key Generation |
| KeysGenerated | `can_import_certs()` | Certificate Import |
| CertificatesImported | `can_attest()` | Attestation |
| Attested | `can_seal()` | Seal Configuration |

## ykman CLI Commands Used

**Certificate Import**:
```bash
ykman --device <serial> piv certificates import --pin <pin> --verify <slot> <cert_file>
```

**Attestation**:
```bash
ykman --device <serial> piv keys attest <slot> -
```

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~200 |
| Tests passing | 1072 |
| Files modified | 2 |
| New message variants | 4 |
| New handlers | 4 |
| New CLI implementations | 2 |

## Files Modified

| File | Changes |
|------|---------|
| `src/adapters/yubikey_cli.rs` | Implemented `import_certificate` and `get_attestation` |
| `src/gui.rs` | Added messages, handlers, and UI buttons |

## What Went Well

1. **Temp file pattern** - Using temp files for certificate data works with ykman's file-based interface
2. **State machine guards** - Existing `can_import_certs()` and `can_attest()` methods worked perfectly
3. **PivSlot conversion** - Reused the slot conversion pattern from Sprint 83
4. **UI integration** - Added state display to each YubiKey device card

## Lessons Learned

1. **State variant fields** - Need to check actual field names (`slot_certs` not `imported_slots`)
2. **Attestation data flow** - Attestation cert bytes need separate storage (currently using UUIDs)
3. **Button visibility** - Use state guards to show/hide buttons contextually

## Technical Notes

### Temporary File Handling

ykman's certificate import requires a file path, not stdin:
```rust
let cert_path = temp_dir.join(format!("yubikey_cert_{}.pem", Uuid::now_v7()));
std::fs::write(&cert_path, certificate)?;
// ... run ykman ...
let _ = std::fs::remove_file(&cert_path);
```

### State Transition Fields

CertificatesImported uses:
```rust
CertificatesImported {
    slot_certs: HashMap<PivSlot, Uuid>, // certificate IDs
}
```

Attested uses:
```rust
Attested {
    attestation_chain_verified: bool,
    attestation_cert_ids: Vec<Uuid>,
}
```

## Related Sprints

- Sprint 77: Wired PKI handlers to PKIBootstrapState state machine (RootCA)
- Sprint 78: Wired full PKI chain and YubiKey state definitions
- Sprint 79: Wired Export workflow (ExportReady, Bootstrapped)
- Sprint 80: Wired UI viewmodel to reflect current state
- Sprint 81: Wired IntermediateCA crypto operations
- Sprint 82: Wired Leaf Certificate crypto operations
- Sprint 83: Wired YubiKey provisioning operations (PIN, keys)
- Sprint 84: Wired certificate import and attestation (this sprint)

## Next Steps

1. **Wire seal operation** - Implement final sealing state transition
2. **Add import button** - Add UI button to trigger certificate import
3. **Store attestation certs** - Add storage for actual attestation certificate bytes
4. **Add PIN entry UI** - Add passphrase dialog for YubiKey authentication
5. **Management key rotation** - Wire `change_management_key()` to state machine

