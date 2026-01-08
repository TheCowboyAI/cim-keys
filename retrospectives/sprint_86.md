<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 86 Retrospective: Certificate Import Button & Attestation Storage

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Add UI button for certificate import, store attestation certificate bytes

## What Was Accomplished

### Added Attestation Certificate Storage

Added field to CimKeysApp to store attestation certificate bytes:

```rust
// Sprint 86: Attestation certificate storage
// Maps (serial, slot) -> attestation certificate bytes (DER/PEM)
attestation_certs: std::collections::HashMap<(String, crate::ports::yubikey::PivSlot), Vec<u8>>,
```

### Updated Attestation Handler to Store Bytes

Modified YubiKeyAttestationResult handler:

```rust
Message::YubiKeyAttestationResult(result) => {
    match result {
        Ok((serial, slot, attestation_cert)) => {
            // ... existing state transition code ...

            // Sprint 86: Store attestation certificate bytes for verification
            self.attestation_certs.insert(
                (serial.clone(), slot),
                attestation_cert.clone(),
            );
        }
        // ...
    }
}
```

### Added Certificate Import Button

Added "Import Certificate" button visible when `can_import_certs()` is true:

```rust
// Sprint 86: Import Certificate button - available after keys generated
if yubikey_state.map(|s| s.can_import_certs()).unwrap_or(false) {
    if let Some(leaf_cert) = &available_leaf_cert {
        // We have a certificate to import
        let cert_pem_bytes = leaf_cert.certificate_pem.as_bytes().to_vec();
        ops = ops.push(
            button("Import Certificate")
                .on_press(Message::YubiKeyStartCertificateImport {
                    serial: serial_for_import,
                    slot: PivSlot::Authentication,
                    certificate: cert_pem_bytes,
                    pin: "123456".to_string(), // Default PIN
                })
                .style(CowboyCustomTheme::primary_button())
        );
    } else {
        // No certificate available yet
        ops = ops.push(
            text("Generate leaf cert first")
                .color(self.view_model.colors.orange_warning)
        );
    }
}
```

## UI Button Visibility Order

Buttons now appear in provisioning flow order:

| State | Button | Style | Notes |
|-------|--------|-------|-------|
| KeysGenerated | "Import Certificate" | primary_button | Uses first available leaf cert |
| KeysGenerated | "Generate leaf cert first" | warning text | If no leaf cert available |
| CertificatesImported | "Get Attestation" | glass_button | Triggers attestation retrieval |
| Attested | "Seal Configuration" | security_button | Final seal operation |
| Sealed | "🔒 Sealed" | green text | Immutable state indicator |

## Attestation Storage Design

```rust
// Key: (serial, slot) tuple
// Value: Certificate bytes (DER or PEM format)

attestation_certs: HashMap<(String, PivSlot), Vec<u8>>

// Usage examples:
// Store: self.attestation_certs.insert((serial, slot), cert_bytes);
// Retrieve: self.attestation_certs.get(&(serial, slot));
// Check: self.attestation_certs.contains_key(&(serial, slot));
```

## Certificate Flow

```
                    Certificate Import Flow
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  [PKI Generation]                                                   │
│         │                                                           │
│         ▼                                                           │
│  generated_leaf_certs: HashMap<Uuid, X509Certificate>               │
│         │                                                           │
│         ▼                                                           │
│  [User clicks "Import Certificate"]                                 │
│         │                                                           │
│         ▼                                                           │
│  YubiKeyStartCertificateImport {                                    │
│      serial: String,                                                │
│      slot: PivSlot::Authentication,                                 │
│      certificate: leaf_cert.certificate_pem.as_bytes(),             │
│      pin: "123456" (default - TODO: passphrase dialog),             │
│  }                                                                  │
│         │                                                           │
│         ▼                                                           │
│  yubikey_port.import_certificate() → ykman piv certificates import │
│         │                                                           │
│         ▼                                                           │
│  State: CertificatesImported { slot_certs }                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Attestation Storage Flow

```
                    Attestation Storage Flow
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  [User clicks "Get Attestation"]                                    │
│         │                                                           │
│         ▼                                                           │
│  YubiKeyStartAttestation { serial, slot }                           │
│         │                                                           │
│         ▼                                                           │
│  yubikey_port.get_attestation() → ykman piv keys attest             │
│         │                                                           │
│         ▼                                                           │
│  YubiKeyAttestationResult(Ok((serial, slot, attestation_cert)))     │
│         │                                                           │
│         ├─────────────────────────────────────────────────────┐     │
│         ▼                                                     ▼     │
│  yubikey_states.insert(                          attestation_certs  │
│      serial,                                        .insert(        │
│      Attested { ... }                               (serial, slot), │
│  )                                                  cert_bytes      │
│                                                  )                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~50 |
| Tests passing | 1072 |
| Files modified | 1 |
| New struct fields | 1 |
| UI elements added | 2 |

## Files Modified

| File | Changes |
|------|---------|
| `src/gui.rs` | Added attestation_certs field, updated handler, added import button |

## What Went Well

1. **Tuple key for storage** - Using (serial, slot) as HashMap key provides precise lookup
2. **Conditional button rendering** - Button only shows when certificate is available
3. **Fallback text** - Clear guidance when leaf cert not yet generated
4. **Button order** - Matches provisioning flow (Import → Attest → Seal)

## Known Limitations

1. **Default PIN** - Import button uses "123456" default PIN
   - TODO: Integrate with passphrase dialog for PIN entry
2. **First cert only** - Uses first available leaf cert
   - Future: Allow selection of which certificate to import
3. **Single slot** - Always imports to Authentication slot (9a)
   - Future: Allow slot selection

## Technical Notes

### Certificate Selection

Currently selects first available leaf certificate:
```rust
let available_leaf_cert = self.generated_leaf_certs.values().next().cloned();
```

Future improvement: Add picker for which certificate to import to which YubiKey.

### Attestation Retrieval

Attestation certificates can now be verified against:
- YubiKey manufacturer root CA
- Device serial number embedded in certificate
- Key attestation extension confirming on-device generation

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
- Sprint 86: Added import button and attestation storage (this sprint)

## Next Steps

1. **Add PIN entry dialog** - Integrate passphrase dialog for YubiKey PIN
2. **Certificate selection UI** - Allow choosing which cert to import
3. **Slot selection UI** - Allow choosing which PIV slot
4. **Attestation verification** - Verify attestation against YubiKey root CA
5. **Export attestation certs** - Include attestation in export manifest

