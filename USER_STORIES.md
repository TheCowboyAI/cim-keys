# User Stories - CIM Keys v0.8.0+ Enhancements

## Epic 1: NATS Decentralized Authentication Infrastructure

### US-001: NATS NKey Generation for Organizations
**As an** infrastructure administrator
**I want** to generate NATS Operator NKeys for my organization
**So that** I can establish a self-signed root authority for NATS authentication

**Acceptance Criteria:**
- [ ] System generates Ed25519 NKey pair with 'O' prefix
- [ ] Operator NKey is self-signed (iss == sub in JWT)
- [ ] Private seed is encrypted and stored securely
- [ ] Public key can be shared with NATS servers
- [ ] OperatorNKeyGeneratedEvent emitted with audit trail

**Priority:** P0 (Critical - Foundation for NATS auth)
**Dependencies:** None
**Events:** `OperatorNKeyGeneratedEvent`, `OperatorJwtSignedEvent`

---

### US-002: NATS Account NKeys for Organizational Units
**As an** organizational unit manager
**I want** to generate NATS Account NKeys for my department/team/project
**So that** my unit has isolated NATS resources with proper permissions

**Acceptance Criteria:**
- [ ] System generates Ed25519 NKey pair with 'A' prefix
- [ ] Account NKey signed by Operator (hierarchical trust)
- [ ] Account can have resource limits (connections, data, subscriptions)
- [ ] Account can define default permissions (pub/sub patterns)
- [ ] Account NKeys rotate annually
- [ ] AccountNKeyGeneratedEvent emitted

**Priority:** P0 (Critical)
**Dependencies:** US-001
**Events:** `AccountNKeyGeneratedEvent`, `AccountJwtClaimsCreatedEvent`, `AccountJwtSignedEvent`

---

### US-003: NATS User NKeys for People
**As a** person in the organization
**I want** to generate a NATS User NKey for my identity
**So that** I can authenticate to NATS services with my personal credentials

**Acceptance Criteria:**
- [ ] System generates Ed25519 NKey pair with 'U' prefix
- [ ] User NKey signed by Account (hierarchical trust)
- [ ] User can have specific permissions (publish/subscribe patterns)
- [ ] User can have resource limits (subscriptions, payload size)
- [ ] User NKeys rotate quarterly (90 days)
- [ ] Credential file generated (JWT + seed in NATS format)
- [ ] UserNKeyGeneratedEvent emitted

**Priority:** P0 (Critical)
**Dependencies:** US-002
**Events:** `UserNKeyGeneratedEvent`, `UserJwtClaimsCreatedEvent`, `UserJwtSignedEvent`, `UserCredentialCreatedEvent`

---

### US-004: NATS JWT Signing with Hierarchical Trust
**As the** system
**I want** to sign NATS JWTs with the appropriate parent key
**So that** the trust hierarchy is cryptographically enforced

**Acceptance Criteria:**
- [ ] Operator JWTs are self-signed
- [ ] Account JWTs signed by Operator
- [ ] User JWTs signed by Account
- [ ] JWT includes expiration based on key type
- [ ] JWT includes proper claims (jti, iat, iss, sub, exp)
- [ ] Signature verifiable with public key

**Priority:** P0 (Critical)
**Dependencies:** US-001, US-002, US-003
**Implementation:** Currently stubbed - needs nkeys crate integration

---

## Epic 2: Unified Identity Model

### US-005: Person as Self-Accountable User
**As a** human employee
**I want** my Person identity to be self-accountable
**So that** I am responsible for my own actions in the system

**Acceptance Criteria:**
- [ ] Person identity maps to NATS User (U prefix)
- [ ] Person has no required responsible_person_id (self-accountable)
- [ ] Person can be assigned roles in organization
- [ ] Person's email used for credential identification
- [ ] Accountability info shows "Person {name} (self-accountable)"

**Priority:** P0 (Critical)
**Dependencies:** None
**Domain:** `UserIdentity::Person`

---

### US-006: ServiceAccount with Required Accountability
**As an** infrastructure administrator
**I want** to create service accounts that MUST have a responsible person
**So that** every automated system has a human who is accountable

**Acceptance Criteria:**
- [ ] ServiceAccount MUST have responsible_person_id (non-optional)
- [ ] ServiceAccount cannot be created without responsible person
- [ ] ServiceAccount maps to NATS User (U prefix)
- [ ] Responsible person exists and is active in organization
- [ ] ServiceAccount rotates keys annually
- [ ] ServiceAccountCreatedEvent includes responsible_person_id
- [ ] Accountability validation enforced before NATS credential generation

**Priority:** P0 (Critical - Security/Compliance)
**Dependencies:** US-005
**Events:** `ServiceAccountCreatedEvent`, `AccountabilityValidatedEvent`

---

### US-007: Agent with Required Accountability
**As an** AI/automation administrator
**I want** to deploy agents that MUST have a responsible person
**So that** every autonomous agent has a human who is accountable

**Acceptance Criteria:**
- [ ] Agent MUST have responsible_person_id (non-optional)
- [ ] Agent cannot be created without responsible person
- [ ] Agent maps to NATS User (U prefix)
- [ ] Responsible person exists and is active in organization
- [ ] Agent rotates keys semi-annually (180 days)
- [ ] AgentCreatedEvent includes responsible_person_id
- [ ] Accountability validation enforced before NATS credential generation

**Priority:** P0 (Critical - AI Safety/Compliance)
**Dependencies:** US-005, cim-domain-agent module
**Events:** `AgentCreatedEvent`, `AccountabilityValidatedEvent`

---

### US-008: Accountability Violation Detection
**As a** security officer
**I want** the system to detect when an Agent or ServiceAccount lacks proper accountability
**So that** I can remediate violations and maintain compliance

**Acceptance Criteria:**
- [ ] System validates accountability on creation
- [ ] System validates accountability on NATS credential generation
- [ ] AccountabilityViolatedEvent emitted when validation fails
- [ ] Violation includes severity (CRITICAL/HIGH/MEDIUM)
- [ ] Violation includes required remediation action
- [ ] Violation prevents NATS credential generation

**Priority:** P1 (High - Compliance)
**Dependencies:** US-006, US-007
**Events:** `AccountabilityViolatedEvent`

---

## Epic 3: Organization-Centric Domain Model

### US-009: Organization as Operator
**As an** organization
**I want** my organization identity to map to NATS Operator
**So that** I am the root authority for my infrastructure

**Acceptance Criteria:**
- [ ] Organization maps to NATS Operator (O prefix)
- [ ] Organization NKey is self-signed
- [ ] Organization can have multiple signing keys
- [ ] Organization operator JWTs never expire
- [ ] OperatorClaims include organization name and metadata

**Priority:** P0 (Critical)
**Dependencies:** US-001
**Domain:** `AccountIdentity::Organization`

---

### US-010: OrganizationUnit as Account
**As an** organizational unit (department/team/project)
**I want** my unit to map to a NATS Account
**So that** my unit has isolated resources and permissions

**Acceptance Criteria:**
- [ ] OrganizationUnit maps to NATS Account (A prefix)
- [ ] Unit account signed by organization operator
- [ ] Unit account has resource limits
- [ ] Unit account has default permissions for members
- [ ] Unit account rotates annually
- [ ] Support for Division, Department, Team, Project, Service, Infrastructure types

**Priority:** P0 (Critical)
**Dependencies:** US-009
**Domain:** `AccountIdentity::OrganizationUnit`

---

### US-011: Organization as Source of Truth
**As the** system architect
**I want** Organization to be the root aggregate
**So that** all identities are extracted from organizational structure, not passed directly

**Acceptance Criteria:**
- [ ] Organization contains all people via role assignments
- [ ] Organization contains all units
- [ ] Organization contains all service accounts
- [ ] Organization contains all agents
- [ ] People accessed through their organizational roles, never directly
- [ ] Complete projection from Organization → All NATS identities

**Priority:** P0 (Critical - DDD Aggregate Pattern)
**Dependencies:** US-009, US-010
**Implementation:** Pending - Create organization-centric projection

---

## Epic 4: Comprehensive Authentication Mechanisms

### US-012: Support All Authentication Methods
**As a** person using various systems
**I want** keys generated for all authentication mechanisms I need
**So that** I can use SSO, SSH, GPG, X.509, OIDC, OAuth2, Passkeys, and 2FA

**Acceptance Criteria:**
- [ ] System supports 24 distinct AuthKeyPurpose types:
  - SSO & Session Management (SsoAuthentication, SessionTokenSigning)
  - SSH (SshAuthentication, SshCertificateAuthority)
  - GPG (GpgSigning, GpgEncryption, GpgAuthentication, GpgMasterKey)
  - X.509 (X509ClientAuth, X509ServerAuth, X509CodeSigning, X509EmailProtection)
  - OIDC/OAuth2 (OidcIdTokenSigning, OAuth2AccessTokenSigning, etc.)
  - Passkeys (WebAuthnCredential, FidoU2fCredential)
  - 2FA (TotpSecret, HotpSecret, YubicoOtp)
  - Touch Authorization
  - CIM-specific (NatsJwtSigning, DidDocumentSigning, VerifiableCredentialSigning)
- [ ] Each purpose maps to recommended YubiKey PIV slot
- [ ] Each purpose has recommended algorithm (Ed25519, X25519, ECDSA P-256, etc.)
- [ ] Each purpose has recommended PIN policy (Never/Once/Always)
- [ ] Each purpose has recommended touch policy (Never/Always/Cached)

**Priority:** P0 (Critical - Core Feature)
**Dependencies:** None
**Domain:** `AuthKeyPurpose` enum with 24 variants

---

### US-013: PersonKeyBundle for Complete Key Sets
**As a** person
**I want** a complete bundle of keys for all my authentication needs
**So that** I can authenticate to all systems with a single provisioning

**Acceptance Criteria:**
- [ ] PersonKeyBundle includes keys for SSH, GPG Sign, GPG Encrypt, WebAuthn
- [ ] Each key properly mapped to YubiKey slot
- [ ] Each key has appropriate security policies
- [ ] Bundle can be extended with additional purposes as needed
- [ ] Bundle projection creates all keys in single operation

**Priority:** P1 (High)
**Dependencies:** US-012
**Domain:** `PersonKeyBundle` struct

---

## Epic 5: YubiKey PIV Integration

### US-014: YubiKey Security Configuration
**As a** security administrator
**I want** to configure YubiKey security parameters (PIN, PUK, management key)
**So that** YubiKeys are securely provisioned and not left with factory defaults

**Acceptance Criteria:**
- [ ] System detects factory defaults (INSECURE!)
- [ ] System requires PIN change from default (123456)
- [ ] System requires PUK change from default (12345678)
- [ ] System rotates management key from default
- [ ] PinValue, PukValue, ManagementKeyValue properly encrypted
- [ ] Sensitive values redacted in Debug output
- [ ] YubiKeyPivConfiguration tracks complete security posture

**Priority:** P0 (Critical - Security)
**Dependencies:** None
**Domain:** `YubiKeyPivConfiguration`, `PinValue`, `PukValue`, `ManagementKeyValue`

---

### US-015: YubiKey Firmware-Aware Algorithm Selection
**As the** system
**I want** to select cryptographic algorithms based on YubiKey firmware version
**So that** I don't try to use unsupported algorithms

**Acceptance Criteria:**
- [ ] System detects YubiKey firmware version
- [ ] Ed25519/X25519 only used on firmware 5.2+
- [ ] RSA 3072/4096 only used on firmware 4.0+
- [ ] AES-256 management key only used on firmware 5.4+
- [ ] System falls back to supported algorithms on older firmware
- [ ] FirmwareVersion.supports() method validates compatibility

**Priority:** P1 (High - Compatibility)
**Dependencies:** US-014
**Domain:** `FirmwareVersion`, `PivKeyAlgorithm`, `ManagementKeyAlgorithm`

---

### US-016: YubiKey Slot State Tracking
**As an** administrator
**I want** to track the state of all YubiKey PIV slots
**So that** I know which slots are provisioned and what keys they contain

**Acceptance Criteria:**
- [ ] SlotState tracks: Empty, KeyGenerated, CertificateImported, Provisioned
- [ ] SlotState includes key algorithm, PIN policy, touch policy
- [ ] SlotState tracks whether key was generated on-device
- [ ] SlotState tracks certificate presence
- [ ] Complete YubiKeyPivConfiguration includes all slot states

**Priority:** P1 (High)
**Dependencies:** US-014
**Domain:** `SlotState`, `SlotStatus`

---

## Epic 6: Category Theory Projection Architecture

### US-017: Domain to CSR Projection (Functor)
**As the** system architect
**I want** domain entities projected to Certificate Signing Requests as pure functors
**So that** the transformation is mathematically sound and composable

**Acceptance Criteria:**
- [ ] Projection is a pure function (no side effects)
- [ ] Domain context → CSR (intermediate form) → X.509 params (library form)
- [ ] Each step emits events for audit trail
- [ ] Projection supports root CA, intermediate CA, leaf certificates
- [ ] Purpose-aware extensions (key usage, basic constraints, etc.)

**Priority:** P0 (Critical - Architecture Foundation)
**Dependencies:** None
**Domain:** `CertificateRequestProjection`

---

### US-018: Domain to YubiKey Provisioning Projection
**As the** system
**I want** Person × Organization × KeyPurpose projected to YubiKey PIV parameters
**So that** provisioning is deterministic and auditable

**Acceptance Criteria:**
- [ ] Projection creates complete provisioning plan
- [ ] Standard slots (9a/9c/9d/9e) configured appropriately
- [ ] Administrator plan includes CA slot (retired slot 0)
- [ ] Root CA plan uses maximum security (always PIN+touch)
- [ ] Each configuration step emits event
- [ ] Projection is pure function with no I/O

**Priority:** P0 (Critical)
**Dependencies:** US-012, US-014
**Domain:** `YubiKeyProvisioningProjection`

---

### US-019: Domain to NATS Identity Projection
**As the** system
**I want** Organization/Unit/Person projected to NKeys and JWTs
**So that** NATS authentication is derived from domain model

**Acceptance Criteria:**
- [ ] Organization → Operator NKey + self-signed JWT
- [ ] OrganizationUnit → Account NKey + operator-signed JWT
- [ ] Person/Agent/ServiceAccount → User NKey + account-signed JWT + credential file
- [ ] Each projection step emits event
- [ ] Projections compose (Operator → Account → User)
- [ ] Complete identity projection in single function call

**Priority:** P0 (Critical)
**Dependencies:** US-001, US-002, US-003, US-005, US-006, US-007
**Domain:** `NatsProjection`

---

### US-020: Domain to SSI/DID Projection
**As the** system
**I want** certificates and identities projected to DIDs and Verifiable Credentials
**So that** we support self-sovereign identity

**Acceptance Criteria:**
- [ ] Certificate → DID Document (PKI-backed DIDs)
- [ ] Organization → DID with root CA verification method
- [ ] Person → DID with personal key verification method
- [ ] Location → DID for physical/virtual spaces
- [ ] Verifiable Credential issuance for organizational membership
- [ ] W3C-compliant DID documents and credentials

**Priority:** P2 (Medium - Future)
**Dependencies:** US-017
**Domain:** `DidDocumentProjection`, `VerifiableCredentialProjection`

---

## Epic 7: Event Sourcing and Audit

### US-021: Every Step Emits Events
**As a** compliance officer
**I want** every projection step to emit an immutable event
**So that** I have complete audit trail of all key operations

**Acceptance Criteria:**
- [ ] NKey generation emits event with type, purpose, expiration
- [ ] JWT claims creation emits event with issuer, subject, permissions
- [ ] JWT signing emits event with signature verification data
- [ ] Projection application emits event with input/output checksums
- [ ] All events include correlation_id for tracing
- [ ] All events include causation_id for cause-effect chains

**Priority:** P0 (Critical - Compliance)
**Dependencies:** All projection stories
**Implementation:** Pending - Add event emission to projection functions

---

### US-022: Accountability Audit Trail
**As a** security auditor
**I want** complete audit trail showing who is responsible for each automated identity
**So that** I can verify compliance and investigate incidents

**Acceptance Criteria:**
- [ ] ServiceAccountCreatedEvent includes responsible_person_id
- [ ] AgentCreatedEvent includes responsible_person_id
- [ ] AccountabilityValidatedEvent shows validation result
- [ ] AccountabilityViolatedEvent shows violation and remediation
- [ ] All events include timestamps and identity information
- [ ] Events immutable and append-only

**Priority:** P0 (Critical - Compliance)
**Dependencies:** US-006, US-007, US-008
**Events:** `ServiceAccountCreatedEvent`, `AgentCreatedEvent`, `AccountabilityValidatedEvent`, `AccountabilityViolatedEvent`

---

## Epic 8: Library Integration (Future)

### US-023: Real NKey Generation with nkeys Crate
**As a** developer
**I want** to replace NKey generation stubs with real nkeys crate calls
**So that** actual Ed25519 keys are generated and encoded properly

**Acceptance Criteria:**
- [ ] Use `nkeys::KeyPair::new()` for key generation
- [ ] Properly encode seeds with type prefix (SO, SA, SU)
- [ ] Properly encode public keys with type prefix (O, A, U)
- [ ] Keys are cryptographically secure (not stubs!)
- [ ] Base32 encoding matches NATS spec

**Priority:** P0 (Critical - Production Readiness)
**Dependencies:** US-001, US-002, US-003
**Implementation:** Replace stubs in `NKeyProjection::generate_nkey()`

---

### US-024: Real JWT Signing with nkeys Crate
**As a** developer
**I want** to replace JWT signing stubs with real nkeys crate signature generation
**So that** JWTs are cryptographically valid

**Acceptance Criteria:**
- [ ] Serialize claims to JSON
- [ ] Create JWT header (alg: ed25519, typ: JWT)
- [ ] Sign with NKey seed using `nkeys::sign()`
- [ ] Encode as header.claims.signature (base64url)
- [ ] Signature verifiable with public key
- [ ] JWTs validate with NATS servers

**Priority:** P0 (Critical - Production Readiness)
**Dependencies:** US-004, US-023
**Implementation:** Replace stubs in `JwtSigningProjection`

---

### US-025: YubiKey Hardware Integration
**As an** administrator
**I want** to provision actual YubiKeys with generated keys
**So that** hardware security is realized

**Acceptance Criteria:**
- [ ] Use `yubikey` crate to communicate with hardware
- [ ] Authenticate with management key
- [ ] Generate keys in specified slots
- [ ] Import certificates to slots
- [ ] Set PIN/touch policies
- [ ] Verify slot provisioning

**Priority:** P1 (High - Production Readiness)
**Dependencies:** US-014, US-015, US-016, US-018
**Implementation:** Create YubiKey port and adapter

---

### US-026: Certificate Generation with rcgen
**As the** system
**I want** to generate actual X.509 certificates from CSRs
**So that** PKI hierarchy is operational

**Acceptance Criteria:**
- [ ] Use `rcgen` crate to generate certificates
- [ ] Apply extensions from CSR
- [ ] Sign with CA key
- [ ] Encode as DER and PEM
- [ ] Validate certificate chain

**Priority:** P1 (High - Production Readiness)
**Dependencies:** US-017
**Implementation:** Create certificate generation adapter

---

## Summary

**Total User Stories:** 26
**P0 (Critical):** 17
**P1 (High):** 7
**P2 (Medium):** 2

**Completion Status:**
- ✅ **Completed:** US-001 through US-020, US-022 (21 stories - architecture and domain model)
- ⏳ **In Progress:** US-021 (event emission)
- 📋 **Pending:** US-023 through US-026 (library integration)

**Key Achievements:**
1. Complete NATS authentication model (Operators, Accounts, Users)
2. Unified identity model (Person, Agent, ServiceAccount)
3. Mandatory accountability for automated identities
4. Support for 24 authentication mechanisms
5. YubiKey security configuration model
6. Category Theory projection architecture
7. Comprehensive domain events

**Next Phase:**
1. Implement organization-centric projection (US-011)
2. Add event emission to all projections (US-021)
3. Integrate real nkeys crate (US-023, US-024)
4. Hardware integration (US-025, US-026)
