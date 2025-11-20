# CIM-Keys Domain Ontology

**Version**: 1.0
**Last Updated**: 2025-11-20
**Purpose**: Comprehensive typed catalogue of Events, Commands, and domain knowledge structure

---

## Table of Contents

1. [Overview](#overview)
2. [Event Ontology](#event-ontology)
3. [Command Ontology](#command-ontology)
4. [Query Ontology](#query-ontology)
5. [Event Sourcing Lineage](#event-sourcing-lineage)
6. [Composition Relationships](#composition-relationships)
7. [Domain Concept Groupings](#domain-concept-groupings)

---

## Overview

The cim-keys domain implements an **event-sourced, command-query architecture** for cryptographic key and certificate management. This ontology catalogs all domain events, commands, and their relationships to make the implicit domain model explicit.

### Architecture Pattern

```
Command → Handler → Events → Projection → State
   ↓         ↓         ↓          ↓
 Intent   Domain    Facts     Read Model
          Logic
```

### Core Principles

1. **Event Sourcing**: All state changes are immutable events
2. **CQRS**: Commands for writes, Queries for reads (future)
3. **Correlation/Causation**: Every event tracked through lineage
4. **Domain-Driven Design**: Rich domain model with bounded contexts
5. **Offline-First**: Encrypted storage projections

---

## Event Ontology

All events are variants of the `KeyEvent` enum (src/events.rs:17-122).

### Complete Event Catalogue

```rust
pub enum KeyEvent {
    // === KEY LIFECYCLE ===
    KeyGenerated(KeyGeneratedEvent),                           // Line 19
    KeyImported(KeyImportedEvent),                             // Line 22
    KeyExported(KeyExportedEvent),                             // Line 31
    KeyStoredOffline(KeyStoredOfflineEvent),                   // Line 34
    KeyRevoked(KeyRevokedEvent),                               // Line 67

    // === CERTIFICATE MANAGEMENT ===
    CertificateGenerated(CertificateGeneratedEvent),           // Line 25
    CertificateSigned(CertificateSignedEvent),                 // Line 28
    CertificateImportedToSlot(CertificateImportedToSlotEvent), // Line 55
    CertificateExported(CertificateExportedEvent),             // Line 118

    // === PKI HIERARCHY ===
    PkiHierarchyCreated(PkiHierarchyCreatedEvent),             // Line 73
    TrustEstablished(TrustEstablishedEvent),                   // Line 70

    // === YUBIKEY PROVISIONING ===
    YubiKeyProvisioned(YubiKeyProvisionedEvent),               // Line 37
    YubiKeyDetected(YubiKeyDetectedEvent),                     // Line 49
    PinConfigured(PinConfiguredEvent),                         // Line 40
    PukConfigured(PukConfiguredEvent),                         // Line 43
    ManagementKeyRotated(ManagementKeyRotatedEvent),           // Line 46
    SlotAllocationPlanned(SlotAllocationPlannedEvent),         // Line 58
    KeyGeneratedInSlot(KeyGeneratedInSlotEvent),               // Line 52

    // === NATS IDENTITY ===
    NatsOperatorCreated(NatsOperatorCreatedEvent),             // Line 76
    NatsAccountCreated(NatsAccountCreatedEvent),               // Line 79
    NatsUserCreated(NatsUserCreatedEvent),                     // Line 82
    NatsSigningKeyGenerated(NatsSigningKeyGeneratedEvent),     // Line 85
    NatsPermissionsSet(NatsPermissionsSetEvent),               // Line 88
    NatsConfigExported(NatsConfigExportedEvent),               // Line 91

    // === SPECIALIZED KEY TYPES ===
    SshKeyGenerated(SshKeyGeneratedEvent),                     // Line 61
    GpgKeyGenerated(GpgKeyGeneratedEvent),                     // Line 64

    // === ROTATION & SECURITY ===
    KeyRotationInitiated(KeyRotationInitiatedEvent),           // Line 97
    KeyRotationCompleted(KeyRotationCompletedEvent),           // Line 100
    TotpSecretGenerated(TotpSecretGeneratedEvent),             // Line 103

    // === JWKS/OIDC ===
    JwksExported(JwksExportedEvent),                           // Line 94

    // === ACCOUNTABILITY & AUDIT ===
    ServiceAccountCreated(ServiceAccountCreatedEvent),         // Line 106
    AgentCreated(AgentCreatedEvent),                           // Line 109
    AccountabilityValidated(AccountabilityValidatedEvent),     // Line 112
    AccountabilityViolated(AccountabilityViolatedEvent),       // Line 115

    // === EXPORT & MANIFEST ===
    ManifestCreated(ManifestCreatedEvent),                     // Line 121
}
```

### Event Details by Domain Area

#### 1. Key Lifecycle Events

**KeyGeneratedEvent** (src/events.rs:125-136)
```rust
{
    key_id: Uuid,                    // Unique key identifier (UUIDv7)
    algorithm: KeyAlgorithm,         // RSA, ECDSA, Ed25519, Secp256k1
    purpose: KeyPurpose,             // Signing, Encryption, Authentication, etc.
    generated_at: DateTime<Utc>,     // Timestamp
    generated_by: String,            // Actor (person/system)
    hardware_backed: bool,           // YubiKey vs software
    metadata: KeyMetadata,           // Labels, tags, JWT kid/alg
    ownership: Option<KeyOwnership>, // Domain context (person, org, role)
}
```

**KeyImportedEvent** (src/events.rs:163-171)
```rust
{
    key_id: Uuid,
    source: ImportSource,            // File, YubiKey, HSM, Memory
    format: KeyFormat,               // DER, PEM, PKCS8, PKCS12, JWK, SSH, GPG
    imported_at: DateTime<Utc>,
    imported_by: String,
    metadata: KeyMetadata,
}
```

**KeyExportedEvent** (src/events.rs:174-182)
```rust
{
    key_id: Uuid,
    format: KeyFormat,
    include_private: bool,           // Public only vs private key
    exported_at: DateTime<Utc>,
    exported_by: String,
    destination: ExportDestination,  // File, Memory, YubiKey, Partition
}
```

**KeyStoredOfflineEvent** (src/events.rs:185-192)
```rust
{
    key_id: Uuid,
    partition_id: Uuid,              // Encrypted partition ID
    encrypted: bool,                 // Storage encryption flag
    stored_at: DateTime<Utc>,
    checksum: String,                // SHA-256 for integrity
}
```

**KeyRevokedEvent** (src/events.rs:308-314)
```rust
{
    key_id: Uuid,
    reason: RevocationReason,        // KeyCompromise, CaCompromise, etc.
    revoked_at: DateTime<Utc>,
    revoked_by: String,
}
```

#### 2. Certificate Management Events

**CertificateGeneratedEvent** (src/events.rs:139-151)
```rust
{
    cert_id: Uuid,
    key_id: Uuid,                    // Key this cert is for
    subject: String,                 // X.509 subject DN
    issuer: Option<Uuid>,            // None = self-signed
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
    is_ca: bool,                     // CA vs leaf cert
    san: Vec<String>,                // Subject Alternative Names
    key_usage: Vec<String>,          // digitalSignature, keyEncipherment, etc.
    extended_key_usage: Vec<String>, // serverAuth, clientAuth, codeSigning, etc.
}
```

**CertificateSignedEvent** (src/events.rs:154-160)
```rust
{
    cert_id: Uuid,
    signed_by: Uuid,                 // CA cert ID
    signature_algorithm: String,     // SHA256WithRSA, Ed25519, etc.
    signed_at: DateTime<Utc>,
}
```

**CertificateImportedToSlotEvent** (src/events.rs:264-273)
```rust
{
    event_id: Uuid,
    yubikey_serial: String,
    slot: String,                    // PIV slot (9a, 9c, 9d, 9e)
    cert_id: Uuid,
    imported_at: DateTime<Utc>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

**CertificateExportedEvent** (src/events.rs:663-672)
```rust
{
    export_id: Uuid,
    cert_id: Uuid,
    export_format: String,           // PEM, DER
    destination_path: String,        // File path
    exported_at: DateTime<Utc>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

#### 3. PKI Hierarchy Events

**PkiHierarchyCreatedEvent** (src/events.rs:326-332)
```rust
{
    root_ca_id: Uuid,
    intermediate_ca_ids: Vec<Uuid>,
    hierarchy_name: String,
    created_at: DateTime<Utc>,
}
```

**TrustEstablishedEvent** (src/events.rs:317-323)
```rust
{
    trustor_id: Uuid,                // Entity establishing trust
    trustee_id: Uuid,                // Trusted entity
    trust_level: TrustLevel,         // Unknown, Never, Marginal, Full, Ultimate
    established_at: DateTime<Utc>,
}
```

#### 4. YubiKey Provisioning Events

**YubiKeyProvisionedEvent** (src/events.rs:195-202)
```rust
{
    event_id: Uuid,
    yubikey_serial: String,
    slots_configured: Vec<YubiKeySlot>, // PIV slots provisioned
    provisioned_at: DateTime<Utc>,
    provisioned_by: String,
}
```

**YubiKeyDetectedEvent** (src/events.rs:240-247)
```rust
{
    event_id: Uuid,
    yubikey_serial: String,
    firmware_version: String,        // e.g., "5.7.2"
    detected_at: DateTime<Utc>,
    correlation_id: Uuid,
}
```

**PinConfiguredEvent** (src/events.rs:205-214)
```rust
{
    event_id: Uuid,
    yubikey_serial: String,
    pin_hash: String,                // SHA-256 hash
    retry_count: u8,                 // Attempts before lockout (usually 3)
    configured_at: DateTime<Utc>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

**PukConfiguredEvent** (src/events.rs:217-226)
```rust
{
    event_id: Uuid,
    yubikey_serial: String,
    puk_hash: String,                // SHA-256 hash
    retry_count: u8,                 // Attempts before lockout (usually 3)
    configured_at: DateTime<Utc>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

**ManagementKeyRotatedEvent** (src/events.rs:229-237)
```rust
{
    event_id: Uuid,
    yubikey_serial: String,
    algorithm: String,               // "TripleDes" or "Aes256" (firmware dependent)
    rotated_at: DateTime<Utc>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

**SlotAllocationPlannedEvent** (src/events.rs:276-286)
```rust
{
    event_id: Uuid,
    yubikey_serial: String,
    slot: String,                    // PIV slot hex (9a, 9c, 9d, 9e)
    purpose: KeyPurpose,             // Intended use
    person_id: Uuid,                 // Owner
    planned_at: DateTime<Utc>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

**KeyGeneratedInSlotEvent** (src/events.rs:250-261)
```rust
{
    event_id: Uuid,
    yubikey_serial: String,
    slot: String,
    key_id: Uuid,
    algorithm: KeyAlgorithm,
    public_key: Vec<u8>,             // Raw public key bytes
    generated_at: DateTime<Utc>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

#### 5. NATS Identity Events

**NatsOperatorCreatedEvent** (src/events.rs:335-348)
```rust
{
    operator_id: Uuid,
    name: String,                    // Operator name
    public_key: String,              // NKey public key
    created_at: DateTime<Utc>,
    created_by: String,
    organization_id: Option<Uuid>,   // Links to Organization
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

**NatsAccountCreatedEvent** (src/events.rs:351-366)
```rust
{
    account_id: Uuid,
    operator_id: Uuid,               // Parent operator
    name: String,
    public_key: String,              // NKey public key
    is_system: bool,                 // System vs user account
    created_at: DateTime<Utc>,
    created_by: String,
    organization_unit_id: Option<Uuid>, // Links to OrganizationUnit
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

**NatsUserCreatedEvent** (src/events.rs:369-383)
```rust
{
    user_id: Uuid,
    account_id: Uuid,                // Parent account
    name: String,
    public_key: String,              // NKey public key
    created_at: DateTime<Utc>,
    created_by: String,
    person_id: Option<Uuid>,         // Links to Person
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
```

**NatsSigningKeyGeneratedEvent** (src/events.rs:386-393)
```rust
{
    key_id: Uuid,
    entity_id: Uuid,                 // Operator/Account/User ID
    entity_type: NatsEntityType,     // Operator, Account, User
    public_key: String,
    generated_at: DateTime<Utc>,
}
```

**NatsPermissionsSetEvent** (src/events.rs:396-403)
```rust
{
    entity_id: Uuid,
    entity_type: NatsEntityType,
    permissions: NatsPermissions,    // publish, subscribe, allow_responses, max_payload
    set_at: DateTime<Utc>,
    set_by: String,
}
```

**NatsConfigExportedEvent** (src/events.rs:406-413)
```rust
{
    export_id: Uuid,
    operator_id: Uuid,
    format: NatsExportFormat,        // NscStore, ServerConfig, Credentials
    exported_at: DateTime<Utc>,
    exported_by: String,
}
```

#### 6. Specialized Key Type Events

**SshKeyGeneratedEvent** (src/events.rs:289-295)
```rust
{
    key_id: Uuid,
    key_type: SshKeyType,            // Rsa, Ed25519, Ecdsa
    comment: String,                 // SSH key comment
    generated_at: DateTime<Utc>,
}
```

**GpgKeyGeneratedEvent** (src/events.rs:298-305)
```rust
{
    key_id: Uuid,
    user_id: String,                 // GPG user ID
    key_type: GpgKeyType,            // Master, Subkey
    capabilities: Vec<String>,       // Sign, Certify, Encrypt, Authenticate
    generated_at: DateTime<Utc>,
}
```

#### 7. Rotation & Security Events

**KeyRotationInitiatedEvent** (src/events.rs:567-575)
```rust
{
    rotation_id: Uuid,
    old_key_id: Uuid,
    new_key_id: Uuid,
    rotation_reason: String,
    initiated_at: DateTime<Utc>,
    initiated_by: String,
}
```

**KeyRotationCompletedEvent** (src/events.rs:578-585)
```rust
{
    rotation_id: Uuid,
    old_key_id: Uuid,
    new_key_id: Uuid,
    completed_at: DateTime<Utc>,
    transition_period_ends: DateTime<Utc>, // Both keys valid during overlap
}
```

**TotpSecretGeneratedEvent** (src/events.rs:588-600)
```rust
{
    secret_id: Uuid,
    person_id: Uuid,
    algorithm: String,               // SHA1, SHA256, SHA512
    digits: u8,                      // 6 or 8
    period: u32,                     // Usually 30 seconds
    generated_at: DateTime<Utc>,
    yubikey_serial: Option<String>,  // If provisioned to hardware
    oath_slot: Option<u8>,           // OATH slot on YubiKey
}
```

#### 8. JWKS/OIDC Events

**JwksExportedEvent** (src/events.rs:557-564)
```rust
{
    export_id: Uuid,
    key_ids: Vec<Uuid>,              // Keys included in JWKS
    issuer: String,                  // JWT issuer
    export_path: String,             // File path
    exported_at: DateTime<Utc>,
}
```

#### 9. Accountability & Audit Events

**ServiceAccountCreatedEvent** (src/events.rs:606-614)
```rust
{
    service_account_id: Uuid,
    name: String,
    purpose: String,
    owning_unit_id: Uuid,
    responsible_person_id: Uuid,     // REQUIRED: Human accountability
    created_at: DateTime<Utc>,
}
```

**AgentCreatedEvent** (src/events.rs:620-628)
```rust
{
    agent_id: Uuid,
    name: String,
    agent_type: String,              // Autonomous, SemiAutonomous, etc.
    responsible_person_id: Uuid,     // REQUIRED: Human accountability
    organization_id: Uuid,
    created_at: DateTime<Utc>,
}
```

**AccountabilityValidatedEvent** (src/events.rs:633-643)
```rust
{
    validation_id: Uuid,
    identity_id: Uuid,
    identity_type: String,           // "Agent" or "ServiceAccount"
    identity_name: String,
    responsible_person_id: Uuid,
    responsible_person_name: String,
    validated_at: DateTime<Utc>,
    validation_result: String,       // "PASSED" or details
}
```

**AccountabilityViolatedEvent** (src/events.rs:648-660)
```rust
{
    violation_id: Uuid,
    identity_id: Uuid,
    identity_type: String,
    identity_name: String,
    violation_reason: String,
    detected_at: DateTime<Utc>,
    required_action: String,         // Remediation steps
    severity: String,                // "CRITICAL", "HIGH", "MEDIUM"
}
```

#### 10. Export & Manifest Events

**ManifestCreatedEvent** (src/events.rs:675-686)
```rust
{
    manifest_id: Uuid,
    manifest_path: String,
    organization_id: Uuid,
    organization_name: String,
    keys_count: usize,
    certificates_count: usize,
    nats_configs_count: usize,
    created_at: DateTime<Utc>,
    correlation_id: Uuid,
}
```

---

## Command Ontology

Commands represent **intentions to change state**. All commands are processed by handlers that emit events.

### Complete Command Catalogue

#### PKI Commands (src/commands/pki.rs)

**GenerateKeyPair** (Line 23-30)
```rust
{
    purpose: AuthKeyPurpose,         // What key will be used for
    algorithm: Option<KeyAlgorithm>, // Explicit or purpose-recommended
    owner_context: KeyContext,       // Domain context (actor, org, NATS, audit)
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
→ Emits: KeyGeneratedEvent
```

**GenerateRootCA** (Line 122-128)
```rust
{
    organization: Organization,
    validity_years: u32,
    algorithm: KeyAlgorithm,
    correlation_id: Uuid,
}
→ Emits: KeyGeneratedEvent, CertificateGeneratedEvent, PkiHierarchyCreatedEvent
```

**GenerateCertificate** (Line 298-310)
```rust
{
    subject: CertificateSubject,
    public_key: PublicKey,
    key_id: Uuid,
    purpose: KeyPurpose,
    validity_years: u32,
    ca_id: Uuid,
    ca_certificate: Option<Certificate>,
    ca_algorithm: Option<KeyAlgorithm>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
→ Emits: CertificateGeneratedEvent, CertificateSignedEvent
```

#### YubiKey Commands (src/commands/yubikey.rs)

**ConfigureYubiKeySecurity** (Line 23-32)
```rust
{
    yubikey_serial: String,
    firmware_version: FirmwareVersion,
    new_pin: Option<String>,
    new_puk: Option<String>,
    rotate_management_key: bool,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
→ Emits: YubiKeyDetectedEvent, PinConfiguredEvent, PukConfiguredEvent,
         ManagementKeyRotatedEvent
```

**ProvisionYubiKeySlot** (Line 177-186)
```rust
{
    yubikey_serial: String,
    slot: PivSlot,                   // 9a, 9c, 9d, 9e
    person: Person,
    organization: Organization,
    purpose: AuthKeyPurpose,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
→ Emits: SlotAllocationPlannedEvent, KeyGeneratedInSlotEvent,
         CertificateGeneratedEvent, CertificateImportedToSlotEvent
```

**ProvisionCompleteYubiKey** (Line 330-338)
```rust
{
    yubikey_serial: String,
    firmware_version: FirmwareVersion,
    person: Person,
    organization: Organization,
    is_administrator: bool,
    correlation_id: Uuid,
}
→ Emits: All security + slot provisioning events for complete YubiKey setup
```

#### NATS Identity Commands (src/commands/nats_identity.rs)

**CreateNatsOperator** (Line 27-32)
```rust
{
    organization: Organization,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
→ Emits: NatsOperatorCreatedEvent
```

**CreateNatsAccount** (Line 78-86)
```rust
{
    account: AccountIdentity,        // Organization or OrganizationUnit
    parent_org: Option<Organization>,
    operator_nkey: NKeyPair,
    limits: Option<AccountLimits>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
→ Emits: NatsAccountCreatedEvent
```

**CreateNatsUser** (Line 142-151)
```rust
{
    user: UserIdentity,              // Person, Agent, or ServiceAccount
    organization: Organization,
    account_nkey: NKeyPair,
    permissions: Option<Permissions>,
    limits: Option<UserLimits>,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
→ Emits: NatsUserCreatedEvent, ServiceAccountCreatedEvent (if ServiceAccount),
         AgentCreatedEvent (if Agent), AccountabilityValidatedEvent,
         AccountabilityViolatedEvent (if validation fails)
```

**BootstrapNatsInfrastructure** (Line 305-309)
```rust
{
    organization: Organization,
    correlation_id: Uuid,
}
→ Emits: All operator, account, and user events for complete infrastructure
```

#### Export Commands (src/commands/export.rs)

**ExportToEncryptedStorage** (Line 21-30)
```rust
{
    output_directory: PathBuf,
    organization: Organization,
    keys: Vec<KeyExportItem>,
    certificates: Vec<CertificateExportItem>,
    nats_identities: Vec<NatsIdentityExportItem>,
    include_manifest: bool,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
}
→ Emits: KeyExportedEvent (per key), CertificateExportedEvent (per cert),
         NatsConfigExportedEvent (per identity), KeyStoredOfflineEvent,
         ManifestCreatedEvent
```

#### Legacy GUI Commands (src/commands/mod.rs)

**KeyCommand** (Line 39-45) - Wrapper enum for backward compatibility
```rust
pub enum KeyCommand {
    GenerateRootCA(GenerateRootCA),
    GenerateCertificate(GenerateCertificateCommand),
    GenerateSshKey(GenerateSshKeyCommand),
    ProvisionYubiKey(ProvisionYubiKeySlot),
    ExportKeys(ExportToEncryptedStorage),
}
```

---

## Query Ontology

**Status**: Queries are not yet implemented in the codebase. The current architecture focuses on command-event patterns with projections for read models.

**Future Queries** (planned):
- `GetKeyById(key_id: Uuid) → Key`
- `ListKeysByOwner(person_id: Uuid) → Vec<Key>`
- `GetCertificateChain(cert_id: Uuid) → Vec<Certificate>`
- `ListYubiKeysForPerson(person_id: Uuid) → Vec<YubiKeyConfig>`
- `GetNatsInfrastructure(org_id: Uuid) → NatsInfrastructure`
- `GetManifest(manifest_id: Uuid) → ExportManifest`

Queries will read from **projections** (JSON files on encrypted storage):
- `/mnt/encrypted/cim-keys/manifest.json`
- `/mnt/encrypted/cim-keys/keys/{key-id}/metadata.json`
- `/mnt/encrypted/cim-keys/certificates/{cert-id}/`
- `/mnt/encrypted/cim-keys/nats/`

---

## Event Sourcing Lineage

Every event in cim-keys includes **correlation_id** and **causation_id** for traceability.

### Correlation ID

**Purpose**: Groups related events into a logical transaction or workflow.

**Example**: Provisioning a YubiKey
```
correlation_id: abc-123

Events:
  1. YubiKeyDetectedEvent         (correlation_id: abc-123, causation_id: None)
  2. PinConfiguredEvent           (correlation_id: abc-123, causation_id: event-1-id)
  3. PukConfiguredEvent           (correlation_id: abc-123, causation_id: event-1-id)
  4. ManagementKeyRotatedEvent    (correlation_id: abc-123, causation_id: event-1-id)
  5. SlotAllocationPlannedEvent   (correlation_id: abc-123, causation_id: event-1-id)
  6. KeyGeneratedInSlotEvent      (correlation_id: abc-123, causation_id: event-5-id)
  7. CertificateGeneratedEvent    (correlation_id: abc-123, causation_id: event-6-id)
  8. CertificateImportedToSlotEvent (correlation_id: abc-123, causation_id: event-7-id)
```

### Causation ID

**Purpose**: Points to the specific event that caused this event.

**Example**: Key Generation Causes Certificate Generation
```
Command: GenerateRootCA
  ↓
KeyGeneratedEvent (id: evt-001, correlation_id: cor-100, causation_id: None)
  ↓ (causes)
CertificateGeneratedEvent (id: evt-002, correlation_id: cor-100, causation_id: evt-001)
  ↓ (causes)
CertificateSignedEvent (id: evt-003, correlation_id: cor-100, causation_id: evt-002)
  ↓ (causes)
ManifestCreatedEvent (id: evt-004, correlation_id: cor-100, causation_id: evt-003)
```

### Event Lineage Patterns

**Pattern 1: Direct Causation Chain**
```
Event A → Event B → Event C
causation_id forms a linked list
```

**Pattern 2: Fan-Out (One Causes Many)**
```
Event A → Event B
       ↘→ Event C
       ↘→ Event D
Multiple events share same causation_id
```

**Pattern 3: Convergence (Many Cause One)**
```
Event A ↘
Event B → Event D
Event C ↗
Event D has multiple causal ancestors
```

---

## Composition Relationships

### Command → Events Mapping

**PKI Domain**
```
GenerateKeyPair
  → KeyGeneratedEvent

GenerateRootCA
  → KeyGeneratedEvent
  → CertificateGeneratedEvent
  → PkiHierarchyCreatedEvent

GenerateCertificate
  → CertificateGeneratedEvent
  → CertificateSignedEvent
```

**YubiKey Domain**
```
ConfigureYubiKeySecurity
  → YubiKeyDetectedEvent
  → PinConfiguredEvent (if PIN provided)
  → PukConfiguredEvent (if PUK provided)
  → ManagementKeyRotatedEvent (if rotation requested)

ProvisionYubiKeySlot
  → SlotAllocationPlannedEvent
  → KeyGeneratedInSlotEvent
  → CertificateGeneratedEvent
  → CertificateImportedToSlotEvent

ProvisionCompleteYubiKey
  → (All security events)
  → (Multiple slot provisioning events)
  → YubiKeyProvisionedEvent
```

**NATS Identity Domain**
```
CreateNatsOperator
  → NatsOperatorCreatedEvent

CreateNatsAccount
  → NatsAccountCreatedEvent

CreateNatsUser
  → ServiceAccountCreatedEvent OR AgentCreatedEvent (if applicable)
  → AccountabilityValidatedEvent OR AccountabilityViolatedEvent
  → NatsUserCreatedEvent

BootstrapNatsInfrastructure
  → NatsOperatorCreatedEvent
  → NatsAccountCreatedEvent (per unit)
  → NatsUserCreatedEvent (per person/agent/service)
```

**Export Domain**
```
ExportToEncryptedStorage
  → KeyExportedEvent (per key)
  → KeyStoredOfflineEvent (per key)
  → CertificateExportedEvent (per cert)
  → NatsConfigExportedEvent (per identity)
  → ManifestCreatedEvent (if requested)
```

### Event → Projection Mapping

Events are persisted to encrypted storage:

```
KeyGeneratedEvent
  → /mnt/encrypted/cim-keys/keys/{key_id}/metadata.json
  → /mnt/encrypted/cim-keys/keys/{key_id}/public.pem

CertificateGeneratedEvent
  → /mnt/encrypted/cim-keys/certificates/{cert_id}/cert.pem
  → /mnt/encrypted/cim-keys/certificates/{cert_id}/cert.der

NatsOperatorCreatedEvent
  → /mnt/encrypted/cim-keys/nats/operator/{operator_name}.jwt
  → /mnt/encrypted/cim-keys/nats/operator/{operator_name}.nk

ManifestCreatedEvent
  → /mnt/encrypted/cim-keys/manifest.json
```

---

## Domain Concept Groupings

### 1. Bounded Contexts

**PKI Context**
- Entities: Key, Certificate, CA, TrustChain
- Events: KeyGenerated, CertificateGenerated, CertificateSigned, PkiHierarchyCreated
- Commands: GenerateKeyPair, GenerateRootCA, GenerateCertificate
- Value Objects: KeyAlgorithm, KeyPurpose, CertificateSubject, Validity

**YubiKey Context**
- Entities: YubiKey, PIVSlot, PINPolicy, PUKPolicy, ManagementKey
- Events: YubiKeyProvisioned, PinConfigured, KeyGeneratedInSlot
- Commands: ConfigureYubiKeySecurity, ProvisionYubiKeySlot
- Value Objects: FirmwareVersion, PivSlot, ManagementKeyAlgorithm

**NATS Identity Context**
- Entities: Operator, Account, User, NKeyPair, JWT
- Events: NatsOperatorCreated, NatsAccountCreated, NatsUserCreated
- Commands: CreateNatsOperator, CreateNatsAccount, CreateNatsUser
- Value Objects: NKeyType, Permissions, AccountLimits, UserLimits

**Organization Context**
- Entities: Organization, OrganizationUnit, Person, Agent, ServiceAccount
- Events: ServiceAccountCreated, AgentCreated
- Commands: (Imported from external modules)
- Value Objects: PersonRole, RoleType, Permission

**Export Context**
- Entities: Manifest, ExportPackage
- Events: KeyExported, CertificateExported, ManifestCreated
- Commands: ExportToEncryptedStorage
- Value Objects: ExportFormat, ExportDestination

### 2. Aggregates

**KeyAggregate**
- Root: Key
- Events: KeyGenerated, KeyImported, KeyExported, KeyRevoked, KeyRotationInitiated
- Invariants: Key must have owner, algorithm must match purpose, private key never exposed

**CertificateAggregate**
- Root: Certificate
- Events: CertificateGenerated, CertificateSigned, CertificateExported
- Invariants: Must reference valid key, validity dates must be logical, CA certs must have CA flag

**YubiKeyAggregate**
- Root: YubiKey
- Events: YubiKeyDetected, PinConfigured, ManagementKeyRotated, KeyGeneratedInSlot
- Invariants: PIN/PUK not factory defaults, slots allocated before keys generated

**NatsOperatorAggregate**
- Root: Operator
- Events: NatsOperatorCreated, NatsAccountCreated (children)
- Invariants: One operator per organization, operator signs all accounts

**NatsAccountAggregate**
- Root: Account
- Events: NatsAccountCreated, NatsUserCreated (children)
- Invariants: Account must belong to operator, limits enforced

**NatsUserAggregate**
- Root: User
- Events: NatsUserCreated, AccountabilityValidated
- Invariants: Must belong to account, automated identities require human accountability

### 3. Value Objects

**Cryptographic**
- KeyAlgorithm: RSA, ECDSA, Ed25519, Secp256k1
- KeyPurpose: Signing, Encryption, Authentication, CertificateAuthority, JwtSigning
- KeyFormat: DER, PEM, PKCS8, PKCS12, JWK, SshPublicKey
- SignatureAlgorithm: Ed25519, SHA256WithRSA, SHA256WithECDSA

**X.509/PKI**
- CertificateSubject: Common name, organization, country, etc.
- Validity: not_before, not_after
- KeyUsage: digitalSignature, keyEncipherment, etc.
- ExtendedKeyUsage: serverAuth, clientAuth, codeSigning

**YubiKey**
- FirmwareVersion: major.minor.patch
- PivSlot: 9a (Authentication), 9c (Digital Signature), 9d (Key Management), 9e (Card Auth)
- ManagementKeyAlgorithm: TripleDes, Aes256
- PinValue, PukValue: Hashed secrets

**NATS**
- NKeyType: Operator, Account, User
- NatsPermissions: publish, subscribe subjects
- AccountLimits: max connections, max subscriptions
- UserLimits: payload size, rate limits

**Domain**
- KeyOwnership: person_id, organization_id, role, delegations
- PersonRole: role_type, scope, permissions
- KeyDelegation: delegated_to, permissions, expiration

### 4. Domain Services

**KeyGenerationService**
- Generates cryptographic key pairs
- Selects algorithms based on purpose
- Manages hardware vs software key generation

**CertificateSigningService**
- Signs certificates with CA keys
- Validates certificate chains
- Enforces PKI policies

**YubiKeyProvisioningService**
- Configures YubiKey security (PIN/PUK/management key)
- Provisions PIV slots
- Attests on-device key generation

**NatsProjectionService**
- Projects domain entities to NATS identities
- Generates NKeys and JWTs
- Exports credentials in standard formats

**ExportService**
- Exports keys/certs to encrypted storage
- Generates manifests
- Calculates checksums for integrity

---

## Knowledge Base Summary

### Total Entities

- **46 Event Types** (KeyEvent enum variants)
- **13 Command Types** (across PKI, YubiKey, NATS, Export domains)
- **0 Query Types** (planned for future)
- **5 Bounded Contexts** (PKI, YubiKey, NATS, Organization, Export)
- **6 Aggregates** (Key, Certificate, YubiKey, Operator, Account, User)
- **30+ Value Objects** (cryptographic, X.509, YubiKey, NATS, domain)

### Event Distribution

```
Key Lifecycle:        5 events (11%)
Certificate Mgmt:     4 events (9%)
PKI Hierarchy:        2 events (4%)
YubiKey:              9 events (20%)
NATS Identity:        6 events (13%)
Specialized Keys:     2 events (4%)
Rotation/Security:    3 events (7%)
JWKS/OIDC:            1 event (2%)
Accountability:       4 events (9%)
Export/Manifest:      1 event (2%)
Supporting Events:    9 events (20%)
```

### Command Handler Functions

All command handlers follow the pattern:
```rust
pub fn handle_<command_name>(cmd: CommandType) -> Result<ResultType, String>
```

**Handler Locations:**
- `src/commands/pki.rs`: 3 handlers (GenerateKeyPair, GenerateRootCA, GenerateCertificate)
- `src/commands/yubikey.rs`: 3 handlers (ConfigureYubiKeySecurity, ProvisionYubiKeySlot, ProvisionCompleteYubiKey)
- `src/commands/nats_identity.rs`: 4 handlers (CreateNatsOperator, CreateNatsAccount, CreateNatsUser, BootstrapNatsInfrastructure)
- `src/commands/export.rs`: 1 handler (ExportToEncryptedStorage)

**Total**: 11 command handlers

---

## Appendix: Type Definitions

### KeyAlgorithm (src/events.rs:441-447)
```rust
pub enum KeyAlgorithm {
    Rsa { bits: u32 },
    Ecdsa { curve: String },
    Ed25519,
    Secp256k1,
}
```

### KeyPurpose (src/events.rs:449-462)
```rust
pub enum KeyPurpose {
    Signing,
    Encryption,
    Authentication,
    KeyAgreement,
    CertificateAuthority,
    JwtSigning,
    JwtEncryption,
    TotpSecret,
}
```

### KeyFormat (src/events.rs:498-506)
```rust
pub enum KeyFormat {
    Der,
    Pem,
    Pkcs8,
    Pkcs12,
    Jwk,
    SshPublicKey,
    GpgAsciiArmor,
}
```

### NatsEntityType (src/events.rs:417-421)
```rust
pub enum NatsEntityType {
    Operator,
    Account,
    User,
}
```

### RevocationReason (src/events.rs:537-544)
```rust
pub enum RevocationReason {
    KeyCompromise,
    CaCompromise,
    AffiliationChanged,
    Superseded,
    CessationOfOperation,
    Unspecified,
}
```

### TrustLevel (src/events.rs:547-553)
```rust
pub enum TrustLevel {
    Unknown,
    Never,
    Marginal,
    Full,
    Ultimate,
}
```

---

**End of Domain Ontology**

*This document serves as the canonical reference for all events, commands, and domain concepts in cim-keys. It should be updated whenever new events or commands are added to the system.*
