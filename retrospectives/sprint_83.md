<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 83 Retrospective: YubiKey Provisioning Wiring

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Wire YubiKey provisioning state machine to real PIV operations via YubiKeyPort

## What Was Accomplished

### Added change_pin to YubiKeyPort Trait

Extended the `YubiKeyPort` trait with PIN change operation:

```rust
/// Change PIN
///
/// **Functor Mapping**: (device, old_pin, new_pin) → ()
/// Changes the PIV PIN from old to new value
async fn change_pin(
    &self,
    serial: &str,
    old_pin: &SecureString,
    new_pin: &SecureString,
) -> Result<(), YubiKeyError>;
```

### Implemented change_pin in All Adapters

**Mock Adapter** (`yubikey_mock.rs`):
```rust
async fn change_pin(&self, serial: &str, old_pin: &SecureString, new_pin: &SecureString) -> Result<(), YubiKeyError> {
    let mut pins = self.pins.write().unwrap();
    let stored_pin = pins.get(serial).ok_or_else(|| YubiKeyError::DeviceNotFound(serial.to_string()))?;
    if &old_pin_str != stored_pin {
        return Err(YubiKeyError::InvalidPin);
    }
    pins.insert(serial.to_string(), new_pin_str);
    Ok(())
}
```

**CLI Adapter** (`yubikey_cli.rs`):
```rust
async fn change_pin(&self, serial: &str, old_pin: &SecureString, new_pin: &SecureString) -> Result<(), YubiKeyError> {
    let output = Command::new("ykman")
        .args(["--device", serial])
        .args(["piv", "access", "change-pin"])
        .args(["--pin", old_pin_str.as_ref()])
        .args(["--new-pin", new_pin_str.as_ref()])
        .output()?;
    // ... error handling
}
```

**Hardware Adapter** (`yubikey_hardware.rs`):
- Returns `NotSupported` error (use CLI adapter for real hardware)

### Wired Detection to State Machine

Connected `YubiKeysDetected` handler to emit `YubiKeyDetected` for each device:

```rust
Message::YubiKeysDetected(result) => {
    match result {
        Ok(devices) => {
            // ... update detected_yubikeys ...

            // Sprint 83: Update YubiKey provisioning state machine for each device
            let tasks: Vec<_> = devices.iter().map(|d| {
                Task::done(Message::YubiKeyDetected {
                    serial: d.serial.clone(),
                    firmware_version: d.version.clone(),
                })
            }).collect();

            return Task::batch(tasks);
        }
        // ...
    }
}
```

### Added New YubiKey Operation Messages

```rust
// Sprint 83: YubiKey Operations (trigger actual PIV operations)
YubiKeyStartAuthentication { serial: String, pin: String },
YubiKeyAuthenticationResult(Result<(String, u8), (String, String)>),
YubiKeyStartPINChange { serial: String, old_pin: String, new_pin: String },
YubiKeyPINChangeResult(Result<String, (String, String)>),
YubiKeyStartKeyGeneration { serial: String, slot: PivSlot, algorithm: KeyAlgorithm },
YubiKeyKeyGenerationResult(Result<(String, PivSlot, Vec<u8>), (String, String)>),
```

### Added Operation Handlers

**Authentication Handler** - Calls `verify_pin()`:
```rust
Message::YubiKeyStartAuthentication { serial, pin } => {
    // Validate state: must be in Detected state
    if !matches!(current_state, Some(state) if state.can_authenticate()) {
        return error;
    }

    Task::perform(
        async move {
            yubikey_port.verify_pin(&serial, &pin_secure).await
        },
        Message::YubiKeyAuthenticationResult
    )
}

Message::YubiKeyAuthenticationResult(result) => {
    match result {
        Ok((serial, pin_retries)) => {
            return Task::done(Message::YubiKeyAuthenticated { serial, pin_retries });
        }
        // ...
    }
}
```

**PIN Change Handler** - Calls `change_pin()`:
```rust
Message::YubiKeyStartPINChange { serial, old_pin, new_pin } => {
    // Validate state: must be in Authenticated state
    if !matches!(current_state, Some(state) if state.can_change_pin()) {
        return error;
    }

    Task::perform(
        async move {
            yubikey_port.change_pin(&serial, &old_pin_secure, &new_pin_secure).await
        },
        Message::YubiKeyPINChangeResult
    )
}
```

**Key Generation Handler** - Calls `generate_key_in_slot()`:
```rust
Message::YubiKeyStartKeyGeneration { serial, slot, algorithm } => {
    Task::perform(
        async move {
            yubikey_port.generate_key_in_slot(&serial, slot, algorithm, &mgmt_key).await
        },
        Message::YubiKeyKeyGenerationResult
    )
}

Message::YubiKeyKeyGenerationResult(result) => {
    match result {
        Ok((serial, slot, public_key)) => {
            // Convert ports::PivSlot to workflows::PivSlot
            let workflow_slot = match slot { /* ... */ };

            // Update state machine to KeysGenerated
            self.yubikey_states.insert(serial, YubiKeyProvisioningState::KeysGenerated { slot_keys });
        }
        // ...
    }
}
```

## YubiKey Provisioning Flow

```
                    YubiKey Provisioning State Machine (Sprint 83)
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  DetectYubiKeys (button click)                                      │
│         │                                                           │
│         ▼                                                           │
│  yubikey_port.list_devices()                                        │
│         │                                                           │
│         ▼                                                           │
│  YubiKeysDetected → for each device → YubiKeyDetected               │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Detected                                 │   │
│  │  serial: "12345678"                                         │   │
│  │  firmware_version: "5.4.3"                                  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼ YubiKeyStartAuthentication { serial, pin }                │
│  yubikey_port.verify_pin()                                          │
│         │                                                           │
│         ▼ YubiKeyAuthenticationResult(Ok) → YubiKeyAuthenticated    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                   Authenticated                             │   │
│  │  pin_retries_remaining: 3                                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼ YubiKeyStartPINChange { serial, old_pin, new_pin }        │
│  yubikey_port.change_pin()                                          │
│         │                                                           │
│         ▼ YubiKeyPINChangeResult(Ok) → YubiKeyPINChanged            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    PINChanged                               │   │
│  │  new_pin_hash: "sha256:..."                                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼ YubiKeyStartKeyGeneration { serial, slot, algorithm }     │
│  yubikey_port.generate_key_in_slot()                                │
│         │                                                           │
│         ▼ YubiKeyKeyGenerationResult(Ok)                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                  KeysGenerated                              │   │
│  │  slot_keys: { Authentication: [pub_key_bytes] }             │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## State Machine Guard Pattern

Each operation validates state machine guards before calling port:

```rust
// Pattern: Validate → Call Port → Update State Machine
Message::YubiKeyStartAuthentication { serial, pin } => {
    // 1. Validate state guard
    let current_state = self.yubikey_states.get(&serial);
    if !matches!(current_state, Some(state) if state.can_authenticate()) {
        return error;
    }

    // 2. Call port operation (async)
    Task::perform(yubikey_port.verify_pin(...))
}

Message::YubiKeyAuthenticationResult(result) => {
    // 3. Update state machine on success
    return Task::done(Message::YubiKeyAuthenticated { ... });
}
```

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~200 |
| Tests passing | 326+ |
| Files modified | 4 |
| New port methods | 1 |
| New message variants | 6 |
| New handlers | 6 |

## Files Modified

| File | Changes |
|------|---------|
| `src/ports/yubikey.rs` | Added `change_pin` trait method |
| `src/adapters/yubikey_mock.rs` | Implemented `change_pin` |
| `src/adapters/yubikey_cli.rs` | Implemented `change_pin` using ykman |
| `src/adapters/yubikey_hardware.rs` | Added `change_pin` stub |
| `src/gui.rs` | Added operation messages and handlers |

## What Went Well

1. **Port abstraction** - YubiKeyPort trait cleanly separates hardware access
2. **State machine integration** - Guard methods prevent invalid transitions
3. **Async pattern** - Task::perform handles non-blocking hardware operations
4. **PivSlot conversion** - Clean mapping between port and workflow slot types

## Lessons Learned

1. **Dual PivSlot types** - `ports::PivSlot` vs `workflows::PivSlot` require explicit conversion
2. **Management key vs PIN** - Key generation uses management key, not PIN
3. **State chaining** - Result handlers emit state transition messages for clean separation

## Technical Notes

### PivSlot Type Conversion

Two separate `PivSlot` enums exist:
- `crate::ports::yubikey::PivSlot` - For port operations
- `crate::state_machines::workflows::PivSlot` - For state machine data

Explicit conversion needed:
```rust
let workflow_slot = match slot {
    crate::ports::yubikey::PivSlot::Authentication =>
        crate::state_machines::workflows::PivSlot::Authentication,
    // ... other variants
};
```

### ykman CLI Commands

PIN change uses:
```bash
ykman --device <serial> piv access change-pin --pin <old> --new-pin <new>
```

Key generation uses (already implemented):
```bash
ykman --device <serial> piv keys generate --algorithm <alg> <slot>
```

## Related Sprints

- Sprint 77: Wired PKI handlers to PKIBootstrapState state machine (RootCA)
- Sprint 78: Wired full PKI chain and YubiKey state definitions
- Sprint 79: Wired Export workflow (ExportReady, Bootstrapped)
- Sprint 80: Wired UI viewmodel to reflect current state
- Sprint 81: Wired IntermediateCA crypto operations
- Sprint 82: Wired Leaf Certificate crypto operations
- Sprint 83: Wired YubiKey provisioning operations (this sprint)

## Next Steps

1. **Wire certificate import** - Connect `import_certificate()` to state machine
2. **Wire attestation** - Connect `get_attestation()` for hardware key verification
3. **Add UI buttons** - Add buttons to trigger new YubiKey operation messages
4. **Management key rotation** - Wire `change_management_key()` to state machine
5. **Seal configuration** - Implement final sealing state transition

