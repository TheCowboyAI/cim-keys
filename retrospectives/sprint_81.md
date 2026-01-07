<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 81 Retrospective: IntermediateCA Crypto Wiring

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Wire IntermediateCA passphrase handler to generate_intermediate_ca() for real crypto operations

## What Was Accomplished

### Added Certificate Storage Fields to CimKeysApp

Extended `CimKeysApp` to store generated certificates for the signing chain:

```rust
// Generated certificate storage (Sprint 81)
// Store generated certificates for signing chain
generated_root_ca: Option<crate::crypto::x509::X509Certificate>,
generated_intermediate_cas: std::collections::HashMap<Uuid, crate::crypto::x509::X509Certificate>,
```

This enables:
- Root CA stored after generation for signing intermediate CAs
- Intermediate CAs stored for signing leaf certificates
- Certificate chain available for verification

### Added IntermediateCAGenerated Message Variant

Extended the Message enum with the async result handler:

```rust
// Sprint 81: Intermediate CA generation result
IntermediateCAGenerated(Result<(crate::crypto::x509::X509Certificate, Uuid), String>),
```

### Wired RootCAGenerated to Store Certificate

Updated the RootCAGenerated handler to persist the certificate:

```rust
Message::RootCAGenerated(result) => {
    match result {
        Ok(certificate) => {
            // ... create certificate node in graph ...

            // Sprint 81: Store the Root CA certificate for signing intermediate CAs
            self.generated_root_ca = Some(certificate.clone());
            tracing::info!("Root CA stored for intermediate CA signing");

            // ... update PKI state machine ...
        }
        Err(e) => { /* error handling */ }
    }
}
```

### Wired IntermediateCA Passphrase Handler

Connected the passphrase dialog to actual crypto operations:

```rust
passphrase_dialog::PassphrasePurpose::IntermediateCA => {
    // Sprint 81: Generate Intermediate CA signed by Root CA
    if let Some(ref root_ca) = self.generated_root_ca {
        self.status_message = "Intermediate CA generation in progress...".to_string();

        // Clone root CA data for async closure
        let root_ca_cert_pem = root_ca.certificate_pem.clone();
        let root_ca_key_pem = root_ca.private_key_pem.clone();

        // Get root CA ID from state machine
        let root_ca_id = match &self.pki_state {
            PKIBootstrapState::RootCAGenerated { root_ca_cert_id, .. } => {
                *root_ca_cert_id
            }
            // ... other states
        };

        return Task::perform(
            async move {
                // Derive seed and generate intermediate CA
                let (cert, _gen_event, _sign_event) = generate_intermediate_ca(
                    &intermediate_seed, params,
                    &root_ca_cert_pem, &root_ca_key_pem,
                    root_ca_id, correlation_id, None,
                )?;
                Ok((cert, intermediate_ca_id))
            },
            Message::IntermediateCAGenerated
        );
    } else {
        self.error_message = Some("Generate Root CA first...".to_string());
    }
}
```

### Added IntermediateCAGenerated Handler

Implemented the async result handler:

```rust
Message::IntermediateCAGenerated(result) => {
    match result {
        Ok((certificate, intermediate_ca_id)) => {
            // Create Certificate object for lifting
            let cert = crate::domain::Certificate::new(
                intermediate_ca_id, /* ... */
            );

            // Create Intermediate CA node in graph
            let custom_color = self.view_model.colors.cert_intermediate;
            let custom_label = format!("{} Intermediate CA", self.organization_name);
            let view = view_model::NodeView::new(intermediate_ca_id, position, custom_color, custom_label);

            // Store the intermediate CA for signing leaf certificates
            self.generated_intermediate_cas.insert(intermediate_ca_id, certificate.clone());

            // Update PKI state machine
            self.pki_state = PKIBootstrapState::IntermediateCAGenerated {
                intermediate_ca_ids: /* accumulated IDs */
            };
        }
        Err(e) => { /* error handling */ }
    }
}
```

### Added cert_intermediate Color to ColorPalette

Extended the view model's color palette:

```rust
// Certificate type colors
pub cert_root_ca: Color,    // Root CA - green (same as green_success)
pub cert_intermediate: Color, // Intermediate CA - purple (between root and leaf)
pub cert_leaf: Color,       // Leaf certificate - blue

// Default values:
cert_root_ca: Color::from_rgb(0.2, 0.8, 0.2),     // Root CA - green
cert_intermediate: Color::from_rgb(0.6, 0.4, 0.8), // Intermediate CA - purple
cert_leaf: Color::from_rgb(0.4, 0.6, 0.8),        // Leaf certificate - blue
```

## Crypto Generation Flow

```
                    IntermediateCA Crypto Wiring (Sprint 81)
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  User clicks "Generate Intermediate CA" button                      │
│                      │                                              │
│                      ▼                                              │
│  PkiExecuteIntermediateCAGeneration                                 │
│                      │                                              │
│                      ▼ can_generate_intermediate_ca()               │
│  PassphraseDialog shown (PassphrasePurpose::IntermediateCA)         │
│                      │                                              │
│                      ▼ User enters passphrase                       │
│  DialogSubmitted(Purpose::IntermediateCA)                           │
│                      │                                              │
│                      ▼ Validate generated_root_ca exists            │
│  Task::perform(async { generate_intermediate_ca(...) })             │
│                      │                                              │
│                      ▼ Crypto operations complete                   │
│  IntermediateCAGenerated(Ok(certificate, intermediate_ca_id))       │
│                      │                                              │
│                      ▼                                              │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ • Create Certificate node in graph                          │   │
│  │ • Store certificate for leaf signing                        │   │
│  │ • Update state machine to IntermediateCAGenerated           │   │
│  │ • Log success                                               │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Certificate Storage Pattern

```rust
// Storage enables full certificate chain
CimKeysApp {
    // Root CA (always single)
    generated_root_ca: Option<X509Certificate>,

    // Intermediate CAs (can have multiple per org unit)
    generated_intermediate_cas: HashMap<Uuid, X509Certificate>,

    // Future: Leaf certificates
    // generated_leaf_certs: HashMap<Uuid, X509Certificate>,
}

// Chain signing:
// Root CA (stored) --> signs --> Intermediate CA (stored) --> signs --> Leaf Cert
```

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~150 |
| Tests passing | 326+ |
| Files modified | 2 |
| New fields | 2 |
| New message variants | 1 |
| New handler methods | 1 |
| New color fields | 1 |

## Files Modified

| File | Changes |
|------|---------|
| `src/gui.rs` | Added storage fields, IntermediateCAGenerated message and handler, wired passphrase handler |
| `src/gui/view_model.rs` | Added cert_intermediate color to ColorPalette |

## What Went Well

1. **Storage pattern** - HashMap enables multiple intermediate CAs per organization
2. **Color distinction** - Purple intermediate CA sits visually between green root and blue leaf
3. **Async pattern** - Task::perform handles non-blocking crypto operations
4. **State machine integration** - Handler properly updates PKI state

## Lessons Learned

1. **Certificate chain storage is essential** - Can't sign without parent certificate
2. **OR patterns must bind same variables** - `|` patterns require consistent bindings
3. **Color semantics matter** - Visual hierarchy: green (root) → purple (intermediate) → blue (leaf)

## Technical Notes

### Certificate Storage vs State Machine

The state machine tracks IDs:
```rust
PKIBootstrapState::RootCAGenerated { root_ca_cert_id, root_ca_key_id, .. }
PKIBootstrapState::IntermediateCAGenerated { intermediate_ca_ids }
```

But storage holds actual certificates:
```rust
generated_root_ca: Option<X509Certificate>
generated_intermediate_cas: HashMap<Uuid, X509Certificate>
```

This separation allows:
- State machine for workflow validation
- Storage for cryptographic operations
- Graph visualization from IDs

### Root CA ID Extraction

When generating intermediate CA, we need the root CA's ID:
```rust
let root_ca_id = match &self.pki_state {
    PKIBootstrapState::RootCAGenerated { root_ca_cert_id, .. } => *root_ca_cert_id,
    PKIBootstrapState::IntermediateCAPlanned { .. } |
    PKIBootstrapState::IntermediateCAGenerated { .. } => {
        // Fallback - should track root_ca_id in future states
        Uuid::now_v7()
    }
    _ => Uuid::now_v7()
};
```

## Related Sprints

- Sprint 77: Wired PKI handlers to PKIBootstrapState state machine (RootCA)
- Sprint 78: Wired full PKI chain (IntermediateCA, LeafCerts, YubiKey)
- Sprint 79: Wired Export workflow (ExportReady, Bootstrapped)
- Sprint 80: Wired UI viewmodel to reflect current state
- Sprint 81: Wired IntermediateCA crypto operations (this sprint)

## Next Steps

1. **Wire LeafCert generation** - Connect passphrase handler to generate_leaf_certificate()
2. **Track root_ca_id in later states** - Avoid fallback UUIDs
3. **Certificate chain verification** - Verify signatures at each level
4. **YubiKey provisioning wiring** - Connect to real YubiKey operations
5. **Bounded context refactoring** - Extract PKI domain from gui.rs

