# Multi-Intermediate Trust Chain for Hosted Services

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

## Overview

For nginx reverse proxies in CowboyAI-hosted infrastructure, we use a **serial chain of intermediate CAs** - standard X.509 certification path as defined in [RFC 5280](https://datatracker.ietf.org/doc/html/rfc5280).

## Standards Compliance

This implementation follows:

- **[RFC 5280](https://datatracker.ietf.org/doc/html/rfc5280)** - Internet X.509 Public Key Infrastructure Certificate and Certificate Revocation List (CRL) Profile
  - Section 4.2.1.9: Basic Constraints (CA flag and pathLenConstraint)
  - Section 6: Certification Path Validation

The `pathLenConstraint` field (Section 4.2.1.9) specifies the maximum number of non-self-issued intermediate certificates that may follow this certificate in a valid certification path.

## The Trust Chain

```
CowboyAI Root CA (20 years, air-gapped)
    │
    ↓ signs
CowboyAI Hosting Intermediate CA (3 years, pathlen:1)
    │
    ↓ signs
Client Org Intermediate CA (3 years, pathlen:0)
    │
    ↓ signs
nginx Server Certificate (90 days)
```

## What This Proves

1. **CowboyAI Root** → ultimate trust anchor
2. **CowboyAI Hosting Intermediate** → CowboyAI operates this hosting infrastructure
3. **Client Intermediate** → This client is hosted by CowboyAI
4. **Server Cert** → This specific server serves this client

## Certificate Details

### CowboyAI Root CA
```
Subject: CN=CowboyAI Root CA, O=Cowboy AI LLC
Issuer: (self-signed)
Basic Constraints: CA:TRUE, pathlen:2
Key Usage: keyCertSign, cRLSign
Validity: 20 years
Storage: Air-gapped YubiKey
```

### CowboyAI Hosting Intermediate CA
```
Subject: CN=CowboyAI Hosting Intermediate CA, O=Cowboy AI LLC, OU=Hosting
Issuer: CN=CowboyAI Root CA
Basic Constraints: CA:TRUE, pathlen:1   ← Can sign ONE more intermediate level
Key Usage: keyCertSign, cRLSign
Validity: 3 years
```

### Client Organization Intermediate CA
```
Subject: CN=<ClientOrg> Hosted Intermediate CA, O=<ClientOrg>, OU=Hosted
Issuer: CN=CowboyAI Hosting Intermediate CA
Basic Constraints: CA:TRUE, pathlen:0   ← Can only sign leaf certs
Key Usage: keyCertSign, cRLSign
Validity: 3 years
```

### nginx Server Certificate
```
Subject: CN=app.clientorg.com, O=<ClientOrg>
Issuer: CN=<ClientOrg> Hosted Intermediate CA
Basic Constraints: CA:FALSE
Key Usage: digitalSignature, keyEncipherment
Extended Key Usage: serverAuth, clientAuth
SAN: app.clientorg.com, www.clientorg.com
Validity: 90 days
```

## Scope

**When to use this chain:**
- nginx reverse proxies for hosted organizations
- Public-facing services operated by CowboyAI for clients

**NOT needed for:**
- Client's internal applications (use their own PKI)
- NATS credentials (separate hierarchy)
- Personal certificates

## Implementation

The existing `generate_intermediate_ca` function already supports this - just call it twice:

```rust
// 1. Generate CowboyAI Hosting Intermediate (signed by CowboyAI Root)
let hosting_intermediate = generate_intermediate_ca(
    &hosting_seed,
    IntermediateCAParams {
        organization: "Cowboy AI, LLC".to_string(),
        organizational_unit: "Hosting".to_string(),
        common_name: "CowboyAI Hosting Intermediate CA".to_string(),
        validity_years: 3,
        ..Default::default()
    },
    &cowboyai_root.certificate_pem,
    &cowboyai_root.private_key_pem,
    cowboyai_root_id,
    correlation_id,
    causation_id,
)?;

// 2. Generate Client Intermediate (signed by CowboyAI Hosting Intermediate)
let client_intermediate = generate_intermediate_ca(
    &client_seed,
    IntermediateCAParams {
        organization: "ACME Corporation".to_string(),
        organizational_unit: "Hosted".to_string(),
        common_name: "ACME Hosted Intermediate CA".to_string(),
        validity_years: 3,
        ..Default::default()
    },
    &hosting_intermediate.certificate_pem,
    &hosting_intermediate.private_key_pem,
    hosting_intermediate_id,
    correlation_id,
    Some(hosting_event_id),
)?;

// 3. Generate server cert (signed by Client Intermediate)
let server_cert = generate_server_certificate(
    &server_seed,
    ServerCertParams {
        common_name: "app.acme.com".to_string(),
        san_entries: vec!["app.acme.com".to_string()],
        organization: "ACME Corporation".to_string(),
        ..Default::default()
    },
    &client_intermediate.certificate_pem,
    &client_intermediate.private_key_pem,
    client_intermediate_id,
    correlation_id,
    Some(client_event_id),
)?;
```

## CA Bundle for nginx

nginx needs the full chain (excluding root):

```nginx
ssl_certificate /etc/ssl/certs/server-chain.pem;
ssl_certificate_key /etc/ssl/private/server.key;
```

Where `server-chain.pem` contains (in order):
```
-----BEGIN CERTIFICATE-----
(server certificate)
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
(client intermediate CA)
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
(cowboyai hosting intermediate CA)
-----END CERTIFICATE-----
```

Clients need the CowboyAI Root CA in their trust store.

## pathlen Constraints

The `pathlen` constraint is critical:

| Certificate | pathlen | Meaning |
|-------------|---------|---------|
| CowboyAI Root | 2 | Can have 2 intermediate levels below |
| CowboyAI Hosting Intermediate | 1 | Can sign 1 more intermediate level |
| Client Intermediate | 0 | Can only sign leaf certificates |

This ensures clients can't create their own sub-CAs.

## Revocation

- Revoke **CowboyAI Hosting Intermediate** → All hosted clients affected
- Revoke **Client Intermediate** → Only that client affected
- Revoke **Server Cert** → Only that server affected

## Related

- [Certificate Hierarchy](../../user/concepts/certificate-hierarchy.md)
- [End-to-End Workflow](../../user/guides/end-to-end-workflow.md)
