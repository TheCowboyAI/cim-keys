<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 87 Retrospective: YubiKey PIN Dialog Integration

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Integrate passphrase dialog for YubiKey PIN entry during certificate import

## What Was Accomplished

### Added YubiKeyImportCert Purpose to PassphraseDialog

Extended `PassphrasePurpose` enum with YubiKey-specific variant:

```rust
/// Sprint 87: YubiKey PIN for certificate import
/// Contains (serial, slot, certificate_bytes) for the pending import operation
YubiKeyImportCert {
    serial: String,
    slot: crate::ports::yubikey::PivSlot,
    certificate: Vec<u8>,
},
```

### Implemented Purpose-Specific Validation

Updated `is_valid()` to handle PIN vs passphrase requirements:

```rust
pub fn is_valid(&self) -> bool {
    if self.passphrase.is_empty() {
        return false;
    }

    match &self.purpose {
        PassphrasePurpose::YubiKeyImportCert { .. } => {
            // YubiKey PIN: 6-8 digits, no confirmation needed
            self.passphrase.len() >= 6 && self.passphrase.len() <= 8
        }
        _ => {
            // Standard passphrase: 12+ chars, must match confirmation
            self.passphrase == self.passphrase_confirm
                && self.passphrase.len() >= 12
        }
    }
}
```

### Updated Dialog UI for PIN Entry

1. **Label and placeholder** - Shows "PIN:" instead of "Passphrase:" for YubiKey operations
2. **No confirmation field** - PIN entry doesn't require double-entry
3. **No strength indicator** - PIN strength checking not applicable
4. **No "Generate Random" button** - Users must enter their actual PIN
5. **"Show PIN" checkbox** - Renamed from "Show passphrase" for PIN context

### Added ShowYubiKeyPinDialog Message

New message triggers PIN dialog for certificate import:

```rust
// Sprint 87: YubiKey PIN dialog for certificate import
ShowYubiKeyPinDialog {
    serial: String,
    slot: crate::ports::yubikey::PivSlot,
    certificate: Vec<u8>,
},
```

### Handler Implementation

```rust
Message::ShowYubiKeyPinDialog { serial, slot, certificate } => {
    // Show passphrase dialog configured for YubiKey PIN entry
    self.passphrase_dialog.show(passphrase_dialog::PassphrasePurpose::YubiKeyImportCert {
        serial: serial.clone(),
        slot,
        certificate,
    });
    self.status_message = format!("Enter PIN to import certificate to YubiKey {}", serial);
    Task::none()
}
```

### PIN Collected Callback

When user submits PIN, triggers certificate import:

```rust
passphrase_dialog::PassphrasePurpose::YubiKeyImportCert { serial, slot, certificate } => {
    // PIN collected, trigger the actual certificate import
    self.status_message = format!("Importing certificate to YubiKey {}...", serial);

    // Use the PIN entered in the dialog (Zeroizing<String> -> String)
    let pin: String = (*passphrase).clone();

    return Task::done(Message::YubiKeyStartCertificateImport {
        serial,
        slot,
        certificate,
        pin,
    });
}
```

## Certificate Import Flow (Updated)

```
                    Certificate Import Flow (Sprint 87)
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  [User clicks "Import Certificate" button]                         │
│         │                                                           │
│         ▼                                                           │
│  Message::ShowYubiKeyPinDialog {                                    │
│      serial: String,                                                │
│      slot: PivSlot::Authentication,                                 │
│      certificate: leaf_cert.certificate_pem.as_bytes(),             │
│  }                                                                  │
│         │                                                           │
│         ▼                                                           │
│  passphrase_dialog.show(YubiKeyImportCert { ... })                  │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              PIN Entry Dialog                               │   │
│  │  ┌─────────────────────────────────────────────────────┐   │   │
│  │  │ YubiKey PIN                                         │   │   │
│  │  │ Enter your YubiKey PIN to import the certificate.   │   │   │
│  │  │                                                     │   │   │
│  │  │ PIN: [******]                                       │   │   │
│  │  │ ☐ Show PIN                                          │   │   │
│  │  │                                                     │   │   │
│  │  │            [OK]  [Cancel]                           │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼ (User clicks OK)                                          │
│  PassphraseDialogMessage::Submit                                    │
│         │                                                           │
│         ▼                                                           │
│  Match purpose: YubiKeyImportCert { serial, slot, certificate }     │
│         │                                                           │
│         ▼                                                           │
│  Task::done(Message::YubiKeyStartCertificateImport {                │
│      serial,                                                        │
│      slot,                                                          │
│      certificate,                                                   │
│      pin: (*passphrase).clone(),                                    │
│  })                                                                 │
│         │                                                           │
│         ▼                                                           │
│  yubikey_port.import_certificate() → ykman piv certificates import │
│         │                                                           │
│         ▼                                                           │
│  State: CertificatesImported { slot_certs }                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Validation Differences: PIN vs Passphrase

| Aspect | Passphrase | PIN |
|--------|------------|-----|
| Minimum length | 12 characters | 6 digits |
| Maximum length | Unlimited | 8 digits |
| Requires confirmation | Yes | No |
| Strength indicator | Yes | No |
| Generate random | Yes | No |
| Show toggle label | "Show passphrase" | "Show PIN" |
| Input label | "Passphrase:" | "PIN:" |

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~80 |
| Tests passing | 27 (doctests) |
| Files modified | 2 |
| New message variants | 1 |
| New handlers | 2 |
| New PassphrasePurpose variants | 1 |

## Files Modified

| File | Changes |
|------|---------|
| `src/gui/passphrase_dialog.rs` | Added YubiKeyImportCert purpose, PIN-specific validation and UI |
| `src/gui.rs` | Added ShowYubiKeyPinDialog message and handlers, wired import button |

## What Went Well

1. **requires_confirmation() reuse** - Existing method provided natural extension point
2. **Pattern matching on purpose** - Clean separation of PIN vs passphrase logic
3. **Minimal UI changes** - Dialog structure remained same, just conditional sections
4. **Zeroizing preserved** - PIN still handled with Zeroizing<String> for security

## Lessons Learned

1. **Type annotations needed** - Zeroizing<String> requires explicit type for clone
2. **Conditional UI rendering** - Used `is_pin_entry` flag for cleaner view logic
3. **Purpose carries data** - Embedding (serial, slot, certificate) in purpose enum cleanly passes context

## Technical Notes

### Zeroizing<String> Access

Converting from Zeroizing<String> to String for message:
```rust
// Correct: dereference and clone
let pin: String = (*passphrase).clone();

// Error: as_ref() needs type annotation
// let pin = passphrase.as_ref().to_string();  // Ambiguous
```

### PIN Validation Logic

YubiKey PIV PIN requirements:
- Default PIN: 123456
- Minimum length: 6 digits
- Maximum length: 8 digits
- Can be changed by user
- Blocks after 3 incorrect attempts (resets with PUK)

## Related Sprints

- Sprint 77: Wired PKI handlers to PKIBootstrapState state machine (RootCA)
- Sprint 78: Wired full PKI chain and YubiKey state definitions
- Sprint 79: Wired Export workflow (ExportReady, Bootstrapped)
- Sprint 80: Wired UI viewmodel to reflect current state
- Sprint 81: Wired IntermediateCA crypto operations
- Sprint 82: Wired Leaf Certificate crypto operations
- Sprint 83: Wired YubiKey provisioning operations (PIN, keys)
- Sprint 84: Wired certificate import and attestation
- Sprint 85: Wired seal operation
- Sprint 86: Added import button and attestation storage
- Sprint 87: Added PIN dialog integration (this sprint)

## Security Considerations

1. **PIN not stored** - PIN is collected, used once, then Zeroizing clears memory
2. **No default PIN** - User must enter their actual PIN each time
3. **Visibility toggle** - PIN hidden by default with option to show
4. **Short-lived PIN** - PIN only held in Zeroizing<String> during operation

## Next Steps

1. **PUK handling** - Add dialog for PUK entry when PIN is blocked
2. **PIN change dialog** - Allow users to change their YubiKey PIN
3. **Management key dialog** - Add dialog for management key operations
4. **Multiple certificate selection** - Allow choosing which certificate to import
5. **Slot selection UI** - Allow choosing which PIV slot for import
