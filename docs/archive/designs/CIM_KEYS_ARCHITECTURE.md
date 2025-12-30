# cim-keys: Complete Architecture for CIM Security Bootstrap

## Executive Summary

**cim-keys** is the genesis point for all CIM (Composable Information Machine) security infrastructure. It creates a tamper-proof, CID-verified PKI configuration that bootstraps an entire organization's cryptographic trust model.

**Key Differentiators**:
- **GUI-First**: Interactive graph as the primary interface (not bash scripts)
- **CID-Verified**: IPLD content addressing replaces PGP signatures for tamper detection
- **First-Time Friendly**: Wizard-style onboarding for users with no PKI experience
- **Encrypted Export**: All output to encrypted SD card with read-only mount
- **Complete Bootstrap**: Generates Organization + PKI + NATS hierarchy in one workflow

## Goals and Non-Goals

### Goals ✅
1. **Create initial CIM security configuration** for an organization starting from nothing
2. **Generate complete PKI hierarchy**: Root CA → Intermediate CAs → step-ca integration
3. **Bootstrap NATS infrastructure**: Operator → Accounts → Users with proper signing keys
4. **Ensure tamper-proof export**: CID verification prevents configuration modification
5. **Enable first-time users**: GUI-guided workflow with templates and validation
6. **Support offline/air-gapped operation**: No network dependencies during key generation

### Non-Goals ❌
1. **Not a key management service**: cim-keys creates the initial configuration, not ongoing operations
2. **Not a general-purpose PKI tool**: Specifically designed for CIM bootstrap
3. **Not cloud-hosted**: Designed for air-gapped, offline operation
4. **Not backward compatible**: New organizations only (migration tools are separate)

## Architecture Overview

### System Context

```
┌─────────────────────────────────────────────────────────────────┐
│                      cim-keys (This System)                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Interactive Graph Interface                    │ │
│  │  (Organizations → OrgUnits → People → Policies → Keys)     │ │
│  └─────────────────┬──────────────────────────────────────────┘ │
│                    │                                             │
│  ┌─────────────────▼──────────────────────────────────────────┐ │
│  │           Event-Sourced Domain Layer                        │ │
│  │  (Aggregates process Commands → emit Events → Project)     │ │
│  └─────────────────┬──────────────────────────────────────────┘ │
│                    │                                             │
│  ┌─────────────────▼──────────────────────────────────────────┐ │
│  │              PKI Generation Layer                           │ │
│  │  • Root CA (offline, Ed25519)                              │ │
│  │  • Intermediate CAs (per OrgUnit)                          │ │
│  │  • step-ca configuration                                   │ │
│  │  • SSH CA (host + user)                                    │ │
│  │  • PGP keys (per person)                                   │ │
│  │  • NATS hierarchy (operator/account/user)                  │ │
│  └─────────────────┬──────────────────────────────────────────┘ │
│                    │                                             │
│  ┌─────────────────▼──────────────────────────────────────────┐ │
│  │          CID Verification & Export Layer                    │ │
│  │  • IPLD DAG builder                                        │ │
│  │  • Content addressing (CID generation)                     │ │
│  │  • Manifest creation                                       │ │
│  │  • Encrypted SD card writer                                │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                  ┌───────────────────────┐
                  │  Encrypted SD Card    │
                  │  (Read-only mount)    │
                  │  CID-verified bundle  │
                  └───────────┬───────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │   CIM Infrastructure Nodes    │
              │  • Import configuration       │
              │  • Verify CIDs                │
              │  • Initialize NATS cluster    │
              │  • Deploy step-ca             │
              └───────────────────────────────┘
```

### Design Principles

#### 1. Graph-Centric Design
**The graph IS the organization.**

Every entity in the system exists as a node in the graph:
- Organizations (root nodes)
- Organizational Units (hierarchy)
- People (identity)
- Locations (key storage)
- Roles (positions)
- Policies (governance)
- Keys (cryptographic material)

Relationships are edges:
- ParentChild (org structure)
- MemberOf (person → unit)
- HasRole (person → role)
- OwnsKey (person → key)
- StoredAt (key → location)
- GovernsBy (entity → policy)

#### 2. Event Sourcing
**All state changes are immutable events.**

Never CRUD - always events:
```rust
// ❌ WRONG: Mutation
person.name = "New Name";

// ✅ CORRECT: Event emission
emit!(PersonNameChanged {
    person_id,
    old_name,
    new_name,
    changed_by,
    timestamp: Utc::now(),
    correlation_id,
    causation_id,
});
```

#### 3. CID-Based Tamper Detection
**IPLD content addressing replaces signatures.**

Instead of PGP signing each file:
```bash
# Old approach (pki_inception_ubuntu)
gpg --detach-sign root_ca.crt
# Creates root_ca.crt.asc (PGP signature)

# New approach (cim-keys)
# Generate CID for each file
cid_root_ca=$(ipfs add --only-hash --cid-version=1 root_ca.crt)
# Store in manifest: root_ca.crt -> bafybeig...
```

Benefits:
- **No secret key needed** for verification (CIDs are self-verifying)
- **Immutable references** (changing file changes CID)
- **Hierarchical verification** (manifest CID verifies entire tree)
- **IPFS-compatible** (can publish to IPFS if desired)

#### 4. Offline-First
**Never assume network connectivity.**

All operations work air-gapped:
- Key generation uses `/dev/urandom` (not network entropy)
- No certificate transparency logs
- No OCSP responders
- No external dependencies

#### 5. YubiKey Integration
**Hardware-backed keys for operational security.**

YubiKey slot allocation:
```
Slot 9A (PIV Authentication)
  ├─ Person operational signing key
  └─ Touch policy: REQUIRED

Slot 9C (Digital Signature)
  ├─ Person code signing key
  └─ Touch policy: REQUIRED

Slot 9D (Key Management)
  ├─ Person encryption key
  └─ Touch policy: OFF (for convenience)

Slot 9E (Card Authentication)
  ├─ Person SSH authentication key
  └─ Touch policy: REQUIRED
```

Root CA keys **NEVER** go on YubiKeys (offline storage only).

## Detailed Architecture

### Layer 1: Interactive Graph Interface

#### Technology Stack
- **Iced 0.13+** (Rust GUI framework)
- **Canvas rendering** (force-directed graph layout)
- **MVI pattern** (Model-View-Intent)
- **WASM-compatible** (can run in browser)

#### Graph Entity Types

```rust
pub enum NodeType {
    Organization {
        org: Organization,
        // Root of the hierarchy
    },

    OrganizationalUnit {
        unit: OrganizationUnit,
        parent_org_id: Uuid,
        unit_type: UnitType, // Division, Department, Team, etc.
    },

    Person {
        person: Person,
        roles: Vec<RoleAssignment>,
        units: Vec<Uuid>,
        clearance: SecurityClearance,
    },

    Location {
        location: Location,
        location_type: LocationType, // DataCenter, SafeDeposit, YubiKey
        security_level: SecurityLevel, // FIPS140Level4, etc.
    },

    Role {
        role: Role,
        required_policies: Vec<Uuid>,
        responsibilities: Vec<String>,
    },

    Policy {
        policy: Policy,
        claims: Vec<PolicyClaim>,
        conditions: Vec<PolicyCondition>,
        priority: i32,
    },

    Key {
        key_id: Uuid,
        key_type: KeyType, // RootCA, IntermediateCA, SSHUser, etc.
        algorithm: KeyAlgorithm, // Ed25519, etc.
        owner_id: Uuid,
        generated_at: DateTime<Utc>,
    },
}
```

#### User Workflows

**Workflow 1: First-Time Organization Setup**
1. User launches cim-keys GUI
2. Welcome screen with wizard:
   - "Create New Organization" button
   - "Import Existing Configuration" button (disabled for first-time)
3. Organization Creation Dialog:
   ```
   ┌────────────────────────────────────────┐
   │ Create Your Organization               │
   ├────────────────────────────────────────┤
   │ Name: [CowboyAI                   ]    │
   │ Domain: [cowboyai.com             ]    │
   │ Description: [                     ]   │
   │                                         │
   │ Country: [US ▼]                        │
   │ State/Province: [Texas            ]    │
   │ City: [Austin                     ]    │
   │                                         │
   │ [< Back]              [Create >]       │
   └────────────────────────────────────────┘
   ```
4. Organization node appears in graph (center)
5. Next: "Add your first organizational unit"
6. Right-click organization → "Add Organizational Unit"
7. OrgUnit dialog → "Engineering" created
8. Next: "Add your first person"
9. Right-click "Engineering" → "Add Person"
10. Person dialog → "Alice Johnson, alice@cowboyai.com"
11. Automatic edge creation: Alice → Engineering (MemberOf)

**Workflow 2: Define Security Policies**
1. User right-clicks canvas → "Create Policy"
2. Policy editor:
   ```
   ┌────────────────────────────────────────┐
   │ Policy Editor                          │
   ├────────────────────────────────────────┤
   │ Name: [Production Access Policy]       │
   │ Description: [Govern prod access  ]    │
   │                                         │
   │ Claims (Permissions):                  │
   │   ☑ CanAccessProduction               │
   │   ☑ CanSignCode                        │
   │   ☐ CanRevokeKeys                      │
   │   ☐ CanManageInfrastructure            │
   │                                         │
   │ Conditions (Requirements):             │
   │   • Minimum Clearance: Secret ▼        │
   │   • MFA Enabled: ☑                     │
   │   • YubiKey Required: ☑                │
   │                                         │
   │ Priority: [100              ]          │
   │                                         │
   │ [Cancel]                    [Save]     │
   └────────────────────────────────────────┘
   ```
3. Policy node created in graph (gold pentagon)
4. User drags edge from Policy → "Engineering" OrgUnit
5. All members of Engineering now governed by policy
6. Policy evaluation shown in person property cards

**Workflow 3: Generate PKI**
1. User clicks "Keys" tab (graph still visible)
2. Sidebar shows:
   ```
   ┌────────────────────────────────────────┐
   │ PKI Generation Wizard                  │
   ├────────────────────────────────────────┤
   │ Step 1: Master Passphrase              │
   │ ☑ Completed                            │
   │                                         │
   │ Step 2: Generate Root CA               │
   │ ☐ Pending                              │
   │   [Generate Root CA]                   │
   │                                         │
   │ Step 3: Intermediate CAs               │
   │ ☐ Pending (1 per org unit)             │
   │   Engineering CA                       │
   │   Operations CA                        │
   │   [Generate Intermediate CAs]          │
   │                                         │
   │ Step 4: Person Keys                    │
   │ ☐ Pending (3 people)                   │
   │   Alice Johnson                        │
   │   Bob Smith                            │
   │   Carol White                          │
   │   [Generate Person Keys]               │
   │                                         │
   │ Step 5: NATS Hierarchy                 │
   │ ☐ Pending                              │
   │   [Generate NATS Hierarchy]            │
   │                                         │
   │ Step 6: Export                         │
   │ ☐ Pending                              │
   │   [Export to SD Card]                  │
   └────────────────────────────────────────┘
   ```
3. User clicks "Generate Root CA"
4. Progress dialog:
   ```
   ┌────────────────────────────────────────┐
   │ Generating Root CA...                  │
   ├────────────────────────────────────────┤
   │ [████████████████░░░░░░░░░░░░] 60%    │
   │                                         │
   │ Creating Ed25519 keypair...            │
   │ Building X.509 certificate...          │
   │ Computing fingerprints...              │
   │                                         │
   │ This may take a moment...              │
   └────────────────────────────────────────┘
   ```
5. Key node appears in graph (green circle with lock icon)
6. Automatic edge: Root CA → (stored at) Encrypted Storage location
7. Graph visualizes entire key hierarchy

### Layer 2: Event-Sourced Domain

#### Domain Events

All changes flow through immutable events:

```rust
// src/events.rs

pub enum DomainEvent {
    // Organization events
    OrganizationCreated {
        id: Uuid,
        name: String,
        domain: String,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    OrganizationalUnitCreated {
        id: Uuid,
        name: String,
        parent_org_id: Uuid,
        unit_type: OrganizationUnitType,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    // Person events
    PersonCreated {
        id: Uuid,
        name: String,
        email: String,
        organization_id: Uuid,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    PersonAssignedToUnit {
        person_id: Uuid,
        unit_id: Uuid,
        assigned_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    // Role events
    RoleCreated {
        id: Uuid,
        name: String,
        description: String,
        organization_id: Uuid,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    RoleAssignedToPerson {
        role_id: Uuid,
        person_id: Uuid,
        assigned_by: Uuid,
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    // Policy events
    PolicyCreated {
        id: Uuid,
        name: String,
        claims: Vec<PolicyClaim>,
        conditions: Vec<PolicyCondition>,
        priority: i32,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    PolicyBoundToEntity {
        policy_id: Uuid,
        entity_id: Uuid,
        entity_type: EntityType,
        bound_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    PolicyEnabled {
        policy_id: Uuid,
        enabled_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    // Key generation events
    RootCAGenerated {
        key_id: Uuid,
        algorithm: KeyAlgorithm,
        public_key_fingerprint: String,
        organization_id: Uuid,
        generated_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    IntermediateCAGenerated {
        key_id: Uuid,
        algorithm: KeyAlgorithm,
        public_key_fingerprint: String,
        organizational_unit_id: Uuid,
        signed_by_key_id: Uuid, // Root CA key ID
        generated_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    PersonKeyGenerated {
        key_id: Uuid,
        key_type: PersonKeyType, // SSH, PGP, X509
        algorithm: KeyAlgorithm,
        public_key_fingerprint: String,
        person_id: Uuid,
        generated_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    KeyStoredAtLocation {
        key_id: Uuid,
        location_id: Uuid,
        storage_type: KeyStorageType, // YubiKey, EncryptedFile, etc.
        stored_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    // NATS hierarchy events
    NATSOperatorCreated {
        operator_id: Uuid,
        organization_id: Uuid,
        signing_key_id: Uuid,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },

    NATSAccountCreated {
        account_id: Uuid,
        organizational_unit_id: Uuid,
        operator_id: Uuid,
        signing_key_id: Uuid,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    NATSUserCreated {
        user_id: Uuid,
        person_id: Uuid,
        account_id: Uuid,
        signing_key_id: Uuid,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    // Export events
    ConfigurationExported {
        export_id: Uuid,
        manifest_cid: String, // IPLD CID of the manifest
        output_path: PathBuf,
        exported_by: Uuid,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
    },
}
```

### Layer 3: PKI Generation

#### PKI Hierarchy (Adapted from pki_inception_ubuntu)

```
Root CA (Offline, Ed25519)
  ├─ Key stored: Encrypted file (NEVER on YubiKey)
  ├─ Certificate validity: 20 years
  ├─ Usage: Signs intermediate CAs only
  └─ Location: Air-gapped storage, SD card

Intermediate CAs (Per Organizational Unit, Ed25519)
  ├─ Engineering CA
  │   ├─ Signed by: Root CA
  │   ├─ Certificate validity: 10 years
  │   ├─ Usage: Signs server certificates, user certificates
  │   └─ Deployed to: step-ca on engineering infrastructure
  │
  ├─ Operations CA
  │   ├─ Signed by: Root CA
  │   ├─ Usage: Signs infrastructure certificates
  │   └─ Deployed to: step-ca on operations infrastructure
  │
  └─ Security CA
      ├─ Signed by: Root CA
      ├─ Usage: Signs security tooling certificates
      └─ Deployed to: step-ca on security infrastructure

Leaf Certificates (ACME, RSA 2048 or Ed25519)
  ├─ Server certificates (TLS)
  │   ├─ Issued by: Intermediate CAs via step-ca
  │   ├─ Certificate validity: 90 days
  │   ├─ Renewal: Automated via ACME
  │   └─ Usage: HTTPS, gRPC, service mesh
  │
  ├─ User certificates (S/MIME, code signing)
  │   ├─ Issued by: Intermediate CAs via step-ca
  │   ├─ Certificate validity: 1 year
  │   └─ Storage: YubiKey PIV slots
  │
  └─ Device certificates (IoT, edge)
      ├─ Issued by: Intermediate CAs via step-ca
      ├─ Certificate validity: 1 year
      └─ Storage: TPM, secure enclave
```

#### step-ca Integration

**Configuration Generation**:

```json
// ca_infra/secrets/step_ca/deployment/config/ca.json
{
  "root": "/var/lib/step-ca/certs/root_ca.crt",
  "crt": "/var/lib/step-ca/certs/intermediate_ca.crt",
  "key": "/var/lib/step-ca/secrets/intermediate_ca.key",
  "password": "<from encrypted source>",

  "address": ":8443",
  "dnsNames": ["ca.cowboyai.com"],

  "logger": {
    "format": "json"
  },

  "db": {
    "type": "badgerv2",
    "dataSource": "/var/lib/step-ca/db"
  },

  "authority": {
    "provisioners": [
      {
        "type": "ACME",
        "name": "acme"
      },
      {
        "type": "JWK",
        "name": "engineering-admin",
        "key": {
          "use": "sig",
          "kty": "EC",
          "kid": "<generated>",
          "crv": "P-256",
          "alg": "ES256",
          "x": "<from JWKS>",
          "y": "<from JWKS>"
        },
        "encryptedKey": "<encrypted private key>"
      }
    ]
  },

  "tls": {
    "cipherSuites": [
      "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
      "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256"
    ],
    "minVersion": 1.3,
    "maxVersion": 1.3
  },

  "templates": {
    "ssh": {
      "user": [{
        "name": "user.tpl",
        "type": "snippet",
        "template": "/var/lib/step-ca/templates/ssh/user.tpl",
        "path": "~/.ssh/authorized_keys"
      }],
      "host": [{
        "name": "host.tpl",
        "type": "snippet",
        "template": "/var/lib/step-ca/templates/ssh/host.tpl",
        "path": "/etc/ssh/ssh_host_ecdsa_key-cert.pub"
      }]
    }
  }
}
```

**SSH CA Integration**:

```bash
# SSH Host CA (signs server SSH host keys)
ssh-keygen -t ed25519 \
  -f ca_infra/secrets/ssh_ca/host_ca/ssh_host_ca.key \
  -C "CowboyAI SSH Host CA"

# SSH User CA (signs user SSH keys for authentication)
ssh-keygen -t ed25519 \
  -f ca_infra/secrets/ssh_ca/user_ca/ssh_user_ca.key \
  -C "CowboyAI SSH User CA"

# step-ca references these in ca.json:
{
  "ssh": {
    "hostKey": "/var/lib/step-ca/secrets/ssh_host_ca.key",
    "userKey": "/var/lib/step-ca/secrets/ssh_user_ca.key"
  }
}
```

**Per-Person Key Generation**:

For each person in the organization:

```rust
// PGP keys (email encryption, code signing)
generate_pgp_key(PersonKeyRequest {
    person_id,
    email: "alice@cowboyai.com",
    name: "Alice Johnson",
    algorithm: KeyAlgorithm::Ed25519,
    usage: vec![KeyUsage::Sign, KeyUsage::Encrypt, KeyUsage::Certify],
    subkeys: vec![
        SubKey { usage: KeyUsage::Sign, algorithm: KeyAlgorithm::Ed25519 },
        SubKey { usage: KeyUsage::Encrypt, algorithm: KeyAlgorithm::Cv25519 },
        SubKey { usage: KeyUsage::Authenticate, algorithm: KeyAlgorithm::Ed25519 },
    ],
    expiration: Some(Duration::days(730)), // 2 years
    passphrase: derived_from_master_passphrase,
});

// SSH keys (server authentication)
generate_ssh_key(PersonKeyRequest {
    person_id,
    key_type: SSHKeyType::Ed25519,
    comment: "alice@cowboyai.com",
    passphrase: derived_from_master_passphrase,
});

// PIV/X.509 keys (YubiKey slots)
generate_piv_key(PersonKeyRequest {
    person_id,
    slot: PIVSlot::Authentication, // 9A
    algorithm: KeyAlgorithm::EcdsaP256,
    subject: "CN=Alice Johnson,O=CowboyAI",
    signed_by: intermediate_ca_for_unit,
    touch_policy: TouchPolicy::Always,
    pin_policy: PINPolicy::Always,
});
```

### Layer 4: CID Verification & Export

#### IPLD DAG Structure

The export creates a content-addressed DAG (Directed Acyclic Graph):

```
Root Manifest (CID: bafybeig...)
├─ organization.json (CID: bafybeid...)
├─ people.json (CID: bafybeif...)
├─ locations.json (CID: bafybeih...)
├─ policies.json (CID: bafybeim...)
├─ events/
│  ├─ 2025-01-15.ndjson (CID: bafybeik...)
│  ├─ 2025-01-16.ndjson (CID: bafybein...)
│  └─ manifest.json (CID: bafybeip...)
├─ pki/
│  ├─ root_ca/
│  │  ├─ root_ca.crt (CID: bafybeir...)
│  │  ├─ root_ca.key.enc (CID: bafybeis...) # Encrypted
│  │  ├─ fingerprints.json (CID: bafybeit...)
│  │  └─ manifest.json (CID: bafybeiu...)
│  ├─ intermediate_cas/
│  │  ├─ engineering_ca.crt (CID: bafybeiv...)
│  │  ├─ engineering_ca.key.enc (CID: bafybeiw...)
│  │  ├─ operations_ca.crt (CID: bafybeix...)
│  │  └─ manifest.json (CID: bafybeiy...)
│  ├─ ssh_ca/
│  │  ├─ ssh_host_ca.pub (CID: bafybeiz...)
│  │  ├─ ssh_user_ca.pub (CID: bafybeja...)
│  │  └─ manifest.json (CID: bafybejb...)
│  └─ manifest.json (CID: bafybejc...)
├─ people_keys/
│  ├─ alice@cowboyai.com/
│  │  ├─ pgp.pub (CID: bafybejd...)
│  │  ├─ pgp.key.enc (CID: bafybeje...)
│  │  ├─ ssh.pub (CID: bafybejf...)
│  │  └─ manifest.json (CID: bafybejg...)
│  └─ manifest.json (CID: bafybejh...)
├─ nats/
│  ├─ operator/
│  │  ├─ operator.jwt (CID: bafybeji...)
│  │  ├─ operator.nk.enc (CID: bafybejj...)
│  │  └─ manifest.json (CID: bafybejk...)
│  ├─ accounts/
│  │  ├─ engineering.jwt (CID: bafybejl...)
│  │  ├─ operations.jwt (CID: bafybejm...)
│  │  └─ manifest.json (CID: bafybejn...)
│  ├─ users/
│  │  ├─ alice.jwt (CID: bafybejo...)
│  │  └─ manifest.json (CID: bafybejp...)
│  └─ manifest.json (CID: bafybejq...)
└─ step_ca/
   ├─ config/
   │  ├─ ca.json (CID: bafybejr...)
   │  └─ manifest.json (CID: bafybejs...)
   └─ manifest.json (CID: bafybejt...)
```

Each `manifest.json` contains CIDs of its children:

```json
{
  "version": "1.0.0",
  "type": "cim-keys-export",
  "generated_at": "2025-01-15T10:30:00Z",
  "generated_by": "cim-keys v0.1.0",

  "organization": {
    "name": "CowboyAI",
    "domain": "cowboyai.com",
    "id": "01933e5c-8f91-7890-a1b2-c3d4e5f6g7h8"
  },

  "contents": {
    "organization.json": {
      "cid": "bafybeid...",
      "size": 1234,
      "type": "application/json"
    },
    "people.json": {
      "cid": "bafybeif...",
      "size": 5678,
      "type": "application/json"
    },
    "events/": {
      "cid": "bafybeip...",
      "type": "directory"
    },
    "pki/": {
      "cid": "bafybejc...",
      "type": "directory"
    },
    "people_keys/": {
      "cid": "bafybejh...",
      "type": "directory"
    },
    "nats/": {
      "cid": "bafybejq...",
      "type": "directory"
    },
    "step_ca/": {
      "cid": "bafybejt...",
      "type": "directory"
    }
  },

  "integrity": {
    "algorithm": "cid-v1-sha256",
    "root_cid": "bafybeig...",
    "verification_instructions": "ipfs dag get <root_cid>"
  }
}
```

#### Verification Algorithm

When CIM nodes import the configuration:

```rust
fn verify_export(mount_path: &Path) -> Result<(), VerificationError> {
    // 1. Read root manifest
    let manifest_path = mount_path.join("manifest.json");
    let manifest: RootManifest = serde_json::from_reader(
        File::open(&manifest_path)?
    )?;

    // 2. Compute CID of manifest itself
    let manifest_bytes = std::fs::read(&manifest_path)?;
    let computed_manifest_cid = compute_cid_v1(&manifest_bytes);

    // 3. Expected manifest CID should be known (from secure channel)
    // For now, we trust the manifest on first import and store its CID
    store_trusted_manifest_cid(&computed_manifest_cid)?;

    // 4. Verify each file's CID matches manifest
    for (file_path, file_info) in manifest.contents {
        let full_path = mount_path.join(&file_path);

        if file_info.type_ == "directory" {
            // Recursively verify directory manifest
            verify_directory_manifest(&full_path, &file_info.cid)?;
        } else {
            // Verify file CID
            let file_bytes = std::fs::read(&full_path)?;
            let computed_cid = compute_cid_v1(&file_bytes);

            if computed_cid != file_info.cid {
                return Err(VerificationError::CIDMismatch {
                    file: file_path,
                    expected: file_info.cid,
                    actual: computed_cid,
                });
            }
        }
    }

    Ok(())
}

fn compute_cid_v1(data: &[u8]) -> String {
    // Use IPLD CID v1 with SHA-256
    // Format: base32(multibase) + version + codec + hash
    use libipld::cid::Cid;
    use libipld::multihash::{Code, MultihashDigest};

    let hash = Code::Sha2_256.digest(data);
    let cid = Cid::new_v1(0x55, hash); // 0x55 = raw codec
    cid.to_string() // Returns "bafybeig..." format
}
```

**Tamper Detection**:
- If ANY file is modified, its CID changes
- Manifest CID verification fails → reject entire import
- Atomic trust: either entire export is valid, or none of it is

**Advantages over PGP Signatures**:
- No private key needed for verification
- Self-verifying (CID is derived from content)
- Works with IPFS pinning services
- Immutable references
- Efficient for large files (chunked hashing)

#### Encrypted SD Card Export

**Partition Structure**:

```bash
/dev/sda (SD Card)
├─ /dev/sda1 (500MB) - Unencrypted boot partition
│  └─ Contains: verification tools, public keys, README
│
└─ /dev/sda2 (Remaining space) - LUKS encrypted partition
   └─ Contains: CID-verified export tree
```

**Encryption Setup**:

```rust
// Use master passphrase to derive encryption key
let encryption_key = derive_key_from_passphrase(
    master_passphrase,
    salt: b"cim-keys-sd-card-encryption-v1",
    iterations: 100_000,
    algorithm: Argon2id,
);

// LUKS format partition
luks_format(
    device: "/dev/sda2",
    key: &encryption_key,
    cipher: "aes-xts-plain64",
    key_size: 512, // bits
    hash: "sha256",
);

// Mount encrypted partition
luks_open(
    device: "/dev/sda2",
    name: "cim-keys-export",
    key: &encryption_key,
);

// Write CID-verified tree
write_export_tree("/dev/mapper/cim-keys-export")?;

// Generate manifest CID
let manifest_cid = generate_root_manifest("/dev/mapper/cim-keys-export")?;

// Write verification script to boot partition
write_verification_script(
    "/dev/sda1/verify.sh",
    expected_manifest_cid: &manifest_cid,
)?;

// Unmount and close
luks_close("cim-keys-export");
```

**Read-Only Mount on CIM Nodes**:

```bash
# On CIM infrastructure node
sudo cryptsetup open /dev/sdb2 cim-keys-import --readonly
sudo mount -o ro /dev/mapper/cim-keys-import /mnt/cim-config

# Verify CIDs
/mnt/cim-config/verify.sh

# If verification passes, import configuration
cim-import --config /mnt/cim-config --verify-cids

# After import, unmount
sudo umount /mnt/cim-config
sudo cryptsetup close cim-keys-import
```

## Implementation Roadmap

### Phase 1: Enhanced Domain Models (Week 1)
**Goal**: Extend domain models with Policy and Role entities

- [ ] Add `Policy`, `PolicyClaim`, `PolicyCondition` to `src/domain.rs`
- [ ] Add `Role`, `RoleAssignment` to `src/domain.rs`
- [ ] Add `PolicyBinding` and `PolicyEvaluation` logic
- [ ] Write unit tests for policy composition
- [ ] Write unit tests for role hierarchies

**Deliverables**:
- `src/domain.rs` with complete entity models
- `tests/domain_tests.rs` with comprehensive coverage

### Phase 2: Graph Interaction Intents (Week 2)
**Goal**: Add MVI Intents for all graph operations

- [ ] Add graph node creation Intents (`UiGraphCreateNode`)
- [ ] Add graph edge creation Intents (`UiGraphCreateEdge`)
- [ ] Add property editing Intents (`UiGraphPropertyChanged`)
- [ ] Add graph deletion Intents (`UiGraphDeleteNode`, `UiGraphDeleteEdge`)
- [ ] Add domain event Intents (`DomainNodeCreated`, etc.)
- [ ] Update `src/mvi/intent.rs` with all new variants

**Deliverables**:
- `src/mvi/intent.rs` with complete Intent definitions
- Documentation for each Intent variant

### Phase 3: Interactive Graph UI (Week 3)
**Goal**: Make graph fully interactive

- [ ] Implement `PropertyCard` component (`src/gui/property_card.rs`)
- [ ] Implement `ContextMenu` component (`src/gui/context_menu.rs`)
- [ ] Implement edge creation visual feedback
- [ ] Add node type-specific colors and shapes
- [ ] Add edge type-specific rendering
- [ ] Wire up all graph interactions to emit Intents

**Deliverables**:
- `src/gui/property_card.rs` - Property editing component
- `src/gui/context_menu.rs` - Right-click menu
- Updated `src/gui/graph.rs` with full interactivity

### Phase 4: PKI Generation Integration (Week 4)
**Goal**: Integrate step-ca patterns from pki_inception_ubuntu

- [ ] Create `src/pki/` module hierarchy
- [ ] Implement `RootCAGenerator` (Ed25519, offline)
- [ ] Implement `IntermediateCAGenerator` (per OrgUnit)
- [ ] Implement `StepCAConfigGenerator` (ca.json generation)
- [ ] Implement `SSHCAGenerator` (host + user CAs)
- [ ] Implement `PersonKeyGenerator` (PGP, SSH, PIV)
- [ ] Add YubiKey provisioning support
- [ ] Wire PKI generation to domain events

**Deliverables**:
- `src/pki/root_ca.rs` - Root CA generation
- `src/pki/intermediate_ca.rs` - Intermediate CA generation
- `src/pki/step_ca.rs` - step-ca configuration
- `src/pki/ssh_ca.rs` - SSH CA generation
- `src/pki/person_keys.rs` - Per-person key generation
- `tests/pki_tests.rs` - Integration tests

### Phase 5: CID Verification System (Week 5)
**Goal**: Implement IPLD content addressing

- [ ] Add `libipld` dependency
- [ ] Implement `compute_cid_v1()` function
- [ ] Implement manifest generation
- [ ] Implement manifest verification
- [ ] Create DAG builder for export tree
- [ ] Write CID verification tests

**Deliverables**:
- `src/cid/mod.rs` - CID computation and verification
- `src/cid/manifest.rs` - Manifest generation and parsing
- `src/cid/dag.rs` - IPLD DAG builder
- `tests/cid_tests.rs` - Verification test suite

### Phase 6: Encrypted Export (Week 6)
**Goal**: Export to encrypted SD card with CID verification

- [ ] Implement LUKS encryption wrapper
- [ ] Implement export tree writer
- [ ] Generate verification scripts
- [ ] Create import validator
- [ ] Add progress tracking for export
- [ ] Add export to GUI

**Deliverables**:
- `src/export/sd_card.rs` - SD card export
- `src/export/encryption.rs` - LUKS wrapper
- `src/export/verification.rs` - CID verification scripts
- `scripts/import-cim-config.sh` - Import script for CIM nodes

### Phase 7: NATS Hierarchy Generation (Week 7)
**Goal**: Generate NATS operator/account/user JWTs

- [ ] Add `nkeys` dependency (NATS key generation)
- [ ] Implement NKeysGenerator (ED25519 keypairs)
- [ ] Implement JWTGenerator (operator, account, user)
- [ ] Map Organization → Operator
- [ ] Map OrgUnits → Accounts
- [ ] Map People → Users
- [ ] Generate NATS configuration files

**Deliverables**:
- `src/nats/nkeys.rs` - NATS key generation
- `src/nats/jwt.rs` - JWT generation
- `src/nats/hierarchy.rs` - Org → NATS mapping
- `tests/nats_tests.rs` - NATS integration tests

### Phase 8: Documentation & Testing (Week 8)
**Goal**: Complete documentation and end-to-end testing

- [ ] Update INTERACTIVE_GRAPH_DESIGN.md
- [ ] Create STEP_CA_INTEGRATION.md
- [ ] Create CID_VERIFICATION.md
- [ ] Create EXPORT_IMPORT_WORKFLOW.md
- [ ] Create USER_GUIDE.md
- [ ] Write end-to-end integration test
- [ ] Record demo video

**Deliverables**:
- Complete documentation suite
- End-to-end integration test
- Demo video showing full workflow

## Security Considerations

### Threat Model

**Threats In Scope**:
1. **Tampered Configuration**: Attacker modifies exported files → CID verification fails
2. **Stolen SD Card**: Attacker has physical SD card → LUKS encryption protects
3. **Malicious Import**: Attacker provides fake configuration → CID verification rejects
4. **Key Extraction**: Attacker compromises running system → Keys encrypted at rest
5. **Supply Chain Attack**: Malicious cim-keys binary → Code signing, reproducible builds

**Threats Out of Scope**:
1. **Evil Maid Attack on Air-Gapped System**: Assume air-gapped system is physically secure
2. **Quantum Computing**: Post-quantum algorithms not yet integrated
3. **Side-Channel Attacks**: Not mitigated in software (rely on hardware tokens)
4. **Social Engineering**: Out of scope for technical controls

### Cryptographic Standards

**Key Algorithms**:
- **Root CA**: Ed25519 (256-bit security)
- **Intermediate CAs**: Ed25519 or ECDSA P-256
- **Person Keys (SSH)**: Ed25519
- **Person Keys (PGP)**: Ed25519 + Cv25519 (encryption)
- **Person Keys (PIV)**: ECDSA P-256 (YubiKey compatibility)
- **NATS Keys**: Ed25519 (nkeys format)

**Encryption**:
- **SD Card**: LUKS with AES-256-XTS
- **Key Files**: AES-256-GCM with Argon2id-derived keys
- **Passphrases**: Argon2id (100k iterations minimum)

**Hashing**:
- **CID**: SHA-256 (via IPLD multihash)
- **Fingerprints**: SHA-256
- **Password Derivation**: Argon2id

### Operational Security

**Air-Gapped Operation**:
1. Run cim-keys on dedicated machine (never networked)
2. Transfer master passphrase via physical paper
3. Export to SD card via USB
4. Transport SD card via secure courier
5. Import on CIM nodes in secure facility

**Key Rotation**:
- Root CA: Never rotated (20-year validity)
- Intermediate CAs: Rotated every 10 years
- Person keys: Rotated every 1-2 years
- NATS keys: Rotated every 90 days (automated)

**Backup Strategy**:
- Multiple SD card copies (at least 3)
- Geographic distribution (different physical locations)
- Safe deposit boxes or secure vaults
- Never digital backups (to prevent remote theft)

## FAQ

### Why Ed25519 instead of RSA?
- **Smaller keys**: 256 bits vs 2048+ bits
- **Faster**: Signature generation and verification
- **Quantum-resistant (partially)**: Better than RSA against quantum attacks
- **Side-channel resistant**: Constant-time operations
- **Modern**: Widely supported (OpenSSH, GnuPG, NATS)

### Why not use existing tools like HashiCorp Vault?
- **Air-gapped requirement**: Vault requires network for clustering
- **Simplicity**: cim-keys is single-purpose (bootstrap only)
- **CID integration**: Vault doesn't support IPLD content addressing
- **Event sourcing**: cim-keys captures full audit trail
- **Graph-centric**: Vault is key-value, not graph-based

### Why LUKS encryption instead of GPG?
- **Full-disk encryption**: LUKS protects entire partition
- **Performance**: Block-level encryption is faster
- **Key derivation**: Argon2id integrated
- **Standard**: Linux kernel crypto subsystem
- **Read-only mount**: Easy to enforce with LUKS

### Can I import the same SD card multiple times?
Yes! CID verification is idempotent. Same CIDs = same content.

### What if I lose the master passphrase?
**You cannot recover**. The master passphrase is the root of trust. Without it:
- Cannot decrypt SD card
- Cannot decrypt key files
- Cannot derive encryption keys

**Mitigation**: Distribute passphrase shares using Shamir's Secret Sharing.

### How do I add a new person after initial export?
**Option A**: Re-run cim-keys, add person, re-export (new manifest CID)
**Option B**: Use operational tooling (not cim-keys) to issue new certs from existing CAs

cim-keys is for **initial bootstrap**, not ongoing operations.

### Can cim-keys run in a web browser?
Yes! The GUI compiles to WASM. However:
- **Security risk**: Browser environment less secure than air-gapped machine
- **No SD card access**: Must download files instead of direct SD write
- **Recommended**: Native application only for production use

---

**Document Version**: 1.0.0
**Last Updated**: 2025-01-15
**Author**: CIM Development Team
**Status**: Living Document (update as implementation proceeds)
