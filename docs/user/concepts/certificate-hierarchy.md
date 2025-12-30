# PKI Hierarchy Design for CIM

## Overview

The CIM PKI follows industry best practices with **intermediate signing-only certificates** to enable certificate rotation without compromising root trust.

## Certificate Hierarchy

```
Root CA (Offline, Air-gapped)
  ├─ Stored on YubiKey #1 (Primary operator)
  ├─ Never signs server certificates directly
  ├─ Only signs intermediate CA certificates
  └─ Long-lived: 10-20 years

     ↓ signs

Intermediate Signing CA (Signing-Only, No Server Identity)
  ├─ Stored on YubiKey #2-N (per organizational unit)
  ├─ basicConstraints: CA:TRUE, pathlen:0
  ├─ keyUsage: keyCertSign, cRLSign (NO digitalSignature, NO keyEncipherment)
  ├─ Does NOT serve as server identity
  ├─ Can be rotated without touching root
  └─ Medium-lived: 2-5 years

     ↓ signs

Server/Leaf Certificates (End-entity, Rotatable)
  ├─ NATS servers, HTTPS endpoints, service identities
  ├─ basicConstraints: CA:FALSE
  ├─ keyUsage: digitalSignature, keyEncipherment
  ├─ extendedKeyUsage: serverAuth, clientAuth
  ├─ Can be rotated frequently
  └─ Short-lived: 90 days - 1 year
```

## Why Intermediate Signing-Only Certificates?

### Problem Without Intermediates
```
Root CA → Server Certs
   ↑
   │ If root compromised: ENTIRE PKI destroyed
   │ If need to rotate: ALL certificates must be re-issued
   └─ Root key must be online to sign new server certs (DANGEROUS!)
```

### Solution With Signing-Only Intermediates
```
Root CA (Offline) → Intermediate Signing CA (Online) → Server Certs
                         ↑
                         │ If intermediate compromised:
                         │   - Revoke intermediate
                         │   - Issue new intermediate from root
                         │   - Re-issue server certs from new intermediate
                         │   - Root CA trust remains intact!
```

## Certificate Types in CIM PKI

### 1. Root CA Certificate
**Purpose**: Ultimate trust anchor, offline storage

**Attributes**:
```
basicConstraints: CA:TRUE, pathlen:1  # Can sign CAs, max 1 level deep
keyUsage: keyCertSign, cRLSign
Validity: 20 years
Storage: YubiKey #1 (operator level)
```

**Generates Seed From**:
```rust
let root_ca_seed = master_seed.derive_child("root-ca");
```

**Usage**:
- Signs intermediate CA certificates ONLY
- NEVER brought online except for intermediate signing
- NEVER signs server certificates
- Stored on air-gapped YubiKey

### 2. Intermediate Signing CA Certificate (Per Organizational Unit)
**Purpose**: Online signing authority, rotatable

**Attributes**:
```
basicConstraints: CA:TRUE, pathlen:0  # Can sign certs, but NOT other CAs
keyUsage: keyCertSign, cRLSign        # SIGNING ONLY - no server auth!
Validity: 3 years
Storage: YubiKey #2-N (domain level)
```

**Generates Seed From**:
```rust
let intermediate_seed = root_ca_seed.derive_child("intermediate-engineering");
let intermediate_seed = root_ca_seed.derive_child("intermediate-operations");
```

**Usage**:
- Signs server/leaf certificates
- Signs user certificates (SSH, GPG, X.509)
- Can be kept online (but securely)
- Can be rotated without touching root CA
- **Does NOT serve as a server identity** - signing only!

**Critical**: `pathlen:0` means this intermediate CANNOT sign other CAs, preventing unauthorized CA creation.

### 3. Server/Leaf Certificates
**Purpose**: Actual service identities (NATS, HTTPS, etc.)

**Attributes**:
```
basicConstraints: CA:FALSE             # NOT a CA
keyUsage: digitalSignature, keyEncipherment
extendedKeyUsage: serverAuth, clientAuth
subjectAltName: DNS:nats.example.com, IP:10.0.0.5
Validity: 90 days (short-lived for security)
Storage: Filesystem or HSM
```

**Generates Seed From**:
```rust
let nats_server_seed = intermediate_seed.derive_child("nats-server-prod-01");
let https_server_seed = intermediate_seed.derive_child("api-gateway-prod-02");
```

**Usage**:
- Serve as TLS server identity
- Present to clients during TLS handshake
- Rotated frequently (90 days recommended)
- Cheap to replace if compromised

## Certificate Rotation Scenarios

### Scenario 1: Routine Server Certificate Rotation (Every 90 Days)
```
1. Generate new server keypair from seed:
   new_server_seed = intermediate.derive_child("nats-server-prod-01-2025-02")

2. Intermediate CA signs new certificate (kept online for this)

3. Deploy new cert to server

4. Old cert expires naturally (no revocation needed)

✅ Root CA never involved
✅ Intermediate CA never touched
✅ Zero trust disruption
```

### Scenario 2: Intermediate CA Rotation (Every 2-3 Years)
```
1. Generate new intermediate keypair from seed:
   new_intermediate = root_ca_seed.derive_child("intermediate-engineering-2027")

2. Root CA signs new intermediate (air-gapped operation)

3. New intermediate signs new server certs

4. Distribute new intermediate cert to trust stores

5. Revoke old intermediate via CRL

✅ Root CA briefly online (air-gapped)
✅ All server certs must be re-issued
✅ Root trust intact
```

### Scenario 3: Root CA Compromise (Catastrophic, Rare)
```
1. Generate entirely new root from NEW passphrase

2. New root signs new intermediates

3. New intermediates sign new server certs

4. Distribute new root to all trust stores

5. Revoke old root in external PKI (if cross-signed)

❌ Complete PKI rebuild required
❌ All certificates invalidated
❌ But: Deterministic reproduction from passphrase makes this less painful!
```

## Implementation in cim-keys

### Seed Derivation Hierarchy
```rust
// Step 1: Master seed from passphrase
let master_seed = derive_master_seed(passphrase, org_id)?;

// Step 2: Root CA seed
let root_ca_seed = master_seed.derive_child("root-ca");

// Step 3: Intermediate CA seeds (one per organizational unit)
let eng_intermediate = root_ca_seed.derive_child("intermediate-engineering");
let ops_intermediate = root_ca_seed.derive_child("intermediate-operations");

// Step 4: Server certificate seeds
let nats_prod_1 = eng_intermediate.derive_child("nats-server-prod-01");
let nats_prod_2 = eng_intermediate.derive_child("nats-server-prod-02");
let api_gateway = ops_intermediate.derive_child("api-gateway-prod-01");
```

### X.509 Certificate Generation

#### Root CA Certificate
```rust
fn generate_root_ca(root_ca_seed: &MasterSeed, org_name: &str) -> Certificate {
    let keypair = KeyPair::from_seed(root_ca_seed);

    Certificate::from_params(CertificateParams {
        distinguished_name: DistinguishedName {
            common_name: format!("{} Root CA", org_name),
            organization: Some(org_name.to_string()),
            ..Default::default()
        },
        key_usages: vec![KeyUsage::KeyCertSign, KeyUsage::CrlSign],
        is_ca: IsCa::Ca(BasicConstraints {
            path_len_constraint: Some(1),  // Can sign intermediates
        }),
        validity_period: ValidityPeriod {
            not_before: Utc::now(),
            not_after: Utc::now() + Duration::days(365 * 20),  // 20 years
        },
        ..Default::default()
    })
}
```

#### Intermediate Signing-Only CA Certificate
```rust
fn generate_intermediate_ca(
    intermediate_seed: &MasterSeed,
    root_ca_cert: &Certificate,
    root_ca_keypair: &KeyPair,
    unit_name: &str,
) -> Certificate {
    let keypair = KeyPair::from_seed(intermediate_seed);

    Certificate::from_params(CertificateParams {
        distinguished_name: DistinguishedName {
            common_name: format!("{} Intermediate CA", unit_name),
            organizational_unit: Some(unit_name.to_string()),
            ..Default::default()
        },
        // CRITICAL: Signing only, no server authentication!
        key_usages: vec![KeyUsage::KeyCertSign, KeyUsage::CrlSign],
        // NO digitalSignature, NO keyEncipherment

        is_ca: IsCa::Ca(BasicConstraints {
            path_len_constraint: Some(0),  // Cannot sign other CAs
        }),
        validity_period: ValidityPeriod {
            not_before: Utc::now(),
            not_after: Utc::now() + Duration::days(365 * 3),  // 3 years
        },
        ..Default::default()
    })
    .signed_by(root_ca_cert, root_ca_keypair)  // Signed by root
}
```

#### Server/Leaf Certificate
```rust
fn generate_server_certificate(
    server_seed: &MasterSeed,
    intermediate_cert: &Certificate,
    intermediate_keypair: &KeyPair,
    server_name: &str,
    san_entries: Vec<String>,
) -> Certificate {
    let keypair = KeyPair::from_seed(server_seed);

    Certificate::from_params(CertificateParams {
        distinguished_name: DistinguishedName {
            common_name: server_name.to_string(),
            ..Default::default()
        },
        key_usages: vec![KeyUsage::DigitalSignature, KeyUsage::KeyEncipherment],
        extended_key_usages: vec![
            ExtendedKeyUsage::ServerAuth,
            ExtendedKeyUsage::ClientAuth,
        ],
        is_ca: IsCa::NotCa,  // NOT a CA!
        subject_alt_names: san_entries,
        validity_period: ValidityPeriod {
            not_before: Utc::now(),
            not_after: Utc::now() + Duration::days(90),  // Short-lived!
        },
        ..Default::default()
    })
    .signed_by(intermediate_cert, intermediate_keypair)  // Signed by intermediate
}
```

## Trust Chain Validation

### Client Validation Flow
```
1. Client receives server certificate during TLS handshake
2. Client validates:
   ├─ Server cert signed by intermediate?
   ├─ Intermediate cert signed by root?
   ├─ Root cert in trust store?
   ├─ Certificates not expired?
   ├─ Certificates not revoked (CRL/OCSP)?
   └─ Subject Alternative Name matches server hostname?

3. If all checks pass: Trust established ✅
```

### Trust Store Distribution
```
Root CA Certificate
  ├─ Distributed to ALL clients (browsers, apps, systems)
  ├─ Rarely changes (20-year lifetime)
  └─ High-trust, high-security

Intermediate CA Certificates
  ├─ Distributed via certificate chain during TLS handshake
  ├─ Can be rotated without client reconfiguration
  └─ Medium-trust, operational flexibility

Server Certificates
  ├─ Never distributed to trust stores
  ├─ Validated against intermediate during connection
  └─ Rotated frequently
```

## Operational Benefits

### Security
- **Root offline**: Compromising intermediate doesn't compromise root
- **Limited blast radius**: Intermediate compromise only affects its subtree
- **Rotation without trust loss**: Replace intermediate without touching root
- **Short-lived server certs**: 90-day rotation limits exposure window

### Operational
- **Automated rotation**: Server certs rotated via cron/systemd timer
- **Deterministic reproduction**: Lost intermediate? Re-derive from seed!
- **Organizational isolation**: Each unit has own intermediate
- **Compliance**: Meets enterprise PKI requirements

### Development
- **Test environments**: Separate intermediate for dev/staging/prod
- **Principle of least privilege**: Dev intermediate can't sign prod certs
- **Audit trail**: Clear certificate lineage

## NATS-Specific Considerations

### NATS Server Certificates
```
Intermediate CA (engineering)
  ├─ nats-server-prod-01.example.com
  ├─ nats-server-prod-02.example.com
  └─ nats-server-prod-03.example.com (cluster)

SAN entries:
  - DNS: nats-server-prod-01.example.com
  - DNS: nats-server-prod-01.internal
  - IP: 10.0.0.5
  - DNS: *.nats.cluster.local (for cluster mesh)
```

### NATS Client Certificates
```
Intermediate CA (engineering)
  ├─ user-alice-nats-client
  ├─ service-api-gateway-nats-client
  └─ service-data-processor-nats-client

Key Usage: clientAuth
```

### NATS Operator/Account Keys (Ed25519, not X.509)
NATS uses **NKeys** (Ed25519) for operator/account/user authentication, separate from TLS PKI:
```
Root NKey Seed = master_seed.derive_child("nats-operator")
  ↓
Operator NKey = generate_nkey(Root NKey Seed)
  ↓
Account NKeys = generate_nkey(operator_seed.derive_child("account-engineering"))
  ↓
User NKeys = generate_nkey(account_seed.derive_child("user-alice"))
```

## Summary

The **intermediate signing-only CA** pattern provides:

1. **Security**: Root CA remains offline, reducing attack surface
2. **Flexibility**: Rotate server certs without touching root
3. **Isolation**: Organizational units have independent signing authority
4. **Compliance**: Meets enterprise and regulatory requirements
5. **Recoverability**: Deterministic reproduction from master passphrase

**Critical**: Intermediate CAs are **signing-only** (keyCertSign, cRLSign), they do NOT serve as server identities. This separation enables rotation without service disruption.

## Implementation Checklist

- [x] Master seed derivation (Argon2id)
- [x] HKDF hierarchical derivation
- [x] Ed25519 keypair generation from seeds
- [ ] X.509 certificate generation with proper extensions
- [ ] Root CA generation (pathlen:1)
- [ ] Intermediate CA generation (pathlen:0, signing-only)
- [ ] Server certificate generation (CA:FALSE)
- [ ] Certificate chain validation
- [ ] YubiKey storage for Root and Intermediate CAs
- [ ] SD card export with certificate chain
- [ ] Certificate rotation workflows

---

**References**:
- RFC 5280: Internet X.509 Public Key Infrastructure
- Mozilla PKI Policy
- NIST SP 800-57: Key Management Guidelines
