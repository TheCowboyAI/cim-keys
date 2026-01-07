<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 82 Retrospective: Leaf Certificate Crypto Wiring

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Wire Leaf Certificate generation to complete the PKI signing chain (Root CA → Intermediate CA → Leaf Cert)

## What Was Accomplished

### Added Leaf Certificate Storage to CimKeysApp

Extended `CimKeysApp` to store the complete certificate chain:

```rust
// Generated certificate storage (Sprint 81, extended Sprint 82)
// Store generated certificates for signing chain
generated_root_ca: Option<crate::crypto::x509::X509Certificate>,
generated_intermediate_cas: std::collections::HashMap<Uuid, crate::crypto::x509::X509Certificate>,
generated_leaf_certs: std::collections::HashMap<Uuid, crate::crypto::x509::X509Certificate>,
```

### Added LeafCertGenerated Message Variant

Extended the Message enum with the async result handler:

```rust
// Sprint 82: Leaf certificate generation result
LeafCertGenerated(Result<(crate::crypto::x509::X509Certificate, Uuid, String), String>), // (cert, leaf_cert_id, person_name)
```

### Wired PersonalKeys Passphrase Handler

Replaced placeholder implementation with actual `generate_server_certificate()` call:

```rust
passphrase_dialog::PassphrasePurpose::PersonalKeys => {
    // Sprint 82: Generate Leaf Certificate signed by Intermediate CA
    if let Some((intermediate_ca_id, intermediate_ca)) = self.generated_intermediate_cas.iter().next() {
        self.status_message = "Leaf certificate generation in progress...".to_string();

        // Clone intermediate CA data for async closure
        let intermediate_ca_cert_pem = intermediate_ca.certificate_pem.clone();
        let intermediate_ca_key_pem = intermediate_ca.private_key_pem.clone();

        return Task::perform(
            async move {
                use crate::crypto::x509::{generate_server_certificate, ServerCertParams};

                let leaf_cert_params = ServerCertParams {
                    common_name: format!("{}", person_name_clone),
                    san_entries: vec![/* SAN entries */],
                    organization: org_name.clone(),
                    organizational_unit: Some("Personal".to_string()),
                    validity_days: 365, // 1 year for personal certs
                };

                let (cert, _gen_event, _sign_event) = generate_server_certificate(
                    &leaf_seed,
                    leaf_cert_params,
                    &intermediate_ca_cert_pem,
                    &intermediate_ca_key_pem,
                    intermediate_ca_id,
                    correlation_id,
                    None,
                )?;

                Ok((cert, leaf_cert_id, person_name_clone))
            },
            Message::LeafCertGenerated
        );
    } else {
        self.error_message = Some("Generate Intermediate CA first...".to_string());
    }
}
```

### Added LeafCertGenerated Handler

Implemented the async result handler:

```rust
Message::LeafCertGenerated(result) => {
    match result {
        Ok((certificate, leaf_cert_id, person_name)) => {
            // Create Leaf Certificate node in graph
            let cert = PkiCertificate::leaf(
                cert_id, subject, issuer, not_before, not_after, key_usage, san
            );

            // Add to graph with cert_leaf color
            let custom_color = self.view_model.colors.cert_leaf;
            let custom_label = format!("{} Certificate", person_name);

            // Store the leaf certificate
            self.generated_leaf_certs.insert(leaf_cert_id, certificate.clone());

            // Update PKI state machine to LeafCertsGenerated
            self.pki_state = PKIBootstrapState::LeafCertsGenerated { leaf_cert_ids };
        }
        Err(e) => { /* error handling */ }
    }
}
```

## Complete PKI Chain

```
                    PKI Certificate Chain (Complete)
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              Root CA (Green - 20 year validity)              │  │
│  │              stored in: generated_root_ca                    │  │
│  │              Sprint 81: generate_root_ca()                   │  │
│  └─────────────────────────────┬────────────────────────────────┘  │
│                                │ signs                              │
│                                ▼                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │           Intermediate CA (Purple - 3 year validity)         │  │
│  │           stored in: generated_intermediate_cas              │  │
│  │           Sprint 81: generate_intermediate_ca()              │  │
│  └─────────────────────────────┬────────────────────────────────┘  │
│                                │ signs                              │
│                                ▼                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │            Leaf Certificate (Blue - 1 year validity)         │  │
│  │            stored in: generated_leaf_certs                   │  │
│  │            Sprint 82: generate_server_certificate()          │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Certificate Generation Flow

```
                    Leaf Certificate Generation (Sprint 82)
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  User clicks "Generate Personal Keys" button                        │
│                      │                                              │
│                      ▼                                              │
│  PropertyCardMessage::GeneratePersonalKeys                          │
│                      │                                              │
│                      ▼                                              │
│  PassphraseDialog shown (PassphrasePurpose::PersonalKeys)           │
│                      │                                              │
│                      ▼ User enters passphrase                       │
│  DialogSubmitted(Purpose::PersonalKeys)                             │
│                      │                                              │
│                      ▼ Validate generated_intermediate_cas exists   │
│  Task::perform(async { generate_server_certificate(...) })          │
│                      │                                              │
│                      ▼ Crypto operations complete                   │
│  LeafCertGenerated(Ok(certificate, leaf_cert_id, person_name))      │
│                      │                                              │
│                      ▼                                              │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ • Create Leaf Certificate node in graph                     │   │
│  │ • Store certificate in generated_leaf_certs                 │   │
│  │ • Update state machine to LeafCertsGenerated                │   │
│  │ • Switch to PKI view                                        │   │
│  │ • Log success                                               │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## State Machine Progression

| State | Guard | Sprint |
|-------|-------|--------|
| Uninitialized | - | 77 |
| RootCAPlanned | can_plan_root_ca() | 77 |
| RootCAGenerated | can_generate_root_ca() | 77/81 |
| IntermediateCAPlanned | can_plan_intermediate_ca() | 78 |
| IntermediateCAGenerated | can_generate_intermediate_ca() | 78/81 |
| **LeafCertsGenerated** | can_generate_leaf_cert() | **78/82** |
| YubiKeysProvisioned | can_provision_yubikey() | 78 |
| ExportReady | can_prepare_export() | 79 |
| Bootstrapped | can_export() | 79 |

## Visual Color Hierarchy

```
Root CA:        Green   - Color::from_rgb(0.2, 0.8, 0.2)
Intermediate:   Purple  - Color::from_rgb(0.6, 0.4, 0.8)
Leaf Cert:      Blue    - Color::from_rgb(0.4, 0.6, 0.8)
```

The color progression (green → purple → blue) creates visual distinction in the PKI Graph view.

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~100 |
| Tests passing | 326+ |
| Files modified | 1 |
| New storage fields | 1 |
| New message variants | 1 |
| New handler methods | 1 |

## Files Modified

| File | Changes |
|------|---------|
| `src/gui.rs` | Added leaf cert storage, LeafCertGenerated message, handler, wired passphrase |

## What Went Well

1. **Consistent pattern** - Same structure as IntermediateCA wiring
2. **Certificate chain complete** - Full Root → Intermediate → Leaf chain now operational
3. **Color consistency** - Visual hierarchy maintained across certificate types
4. **State machine integration** - Proper transition to LeafCertsGenerated state

## Lessons Learned

1. **Check function signatures** - `PkiCertificate::leaf()` takes 7 params (including SAN), not 6
2. **SAN generation** - Leaf certs need SAN entries for modern TLS compatibility
3. **Placeholder removal** - Original placeholder was generating a self-signed cert using Root CA params

## Technical Notes

### Certificate Chain Validation

With Sprint 82 complete, the full chain can be verified:
```
Leaf Cert.issuer == Intermediate CA.subject
Intermediate CA.issuer == Root CA.subject
```

### SAN Generation

For personal certificates, SAN is derived from person name and org:
```rust
format!("{}.{}",
    person_name.to_lowercase().replace(' ', "."),
    org_name.to_lowercase().replace(' ', "-")
)
// Example: "john.doe.cowboy-ai"
```

### Validity Periods

| Certificate Type | Validity |
|------------------|----------|
| Root CA | 20 years |
| Intermediate CA | 3 years |
| Leaf Certificate | 1 year (365 days) |

## Related Sprints

- Sprint 77: Wired PKI handlers to PKIBootstrapState state machine (RootCA)
- Sprint 78: Wired full PKI chain (IntermediateCA, LeafCerts, YubiKey)
- Sprint 79: Wired Export workflow (ExportReady, Bootstrapped)
- Sprint 80: Wired UI viewmodel to reflect current state
- Sprint 81: Wired IntermediateCA crypto operations
- Sprint 82: Wired Leaf Certificate crypto operations (this sprint)

## Next Steps

1. **Wire YubiKey provisioning** - Connect state machine to real YubiKey operations
2. **Certificate chain verification** - Implement verification of signature chain
3. **Certificate export** - Export certificates to projection for SD card backup
4. **NATS key integration** - Generate NATS keys alongside leaf certificates
5. **Bounded context refactoring** - Extract PKI domain from gui.rs per plan

