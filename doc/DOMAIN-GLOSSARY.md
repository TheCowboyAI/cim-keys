<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# CIM-Keys Domain Glossary

This glossary defines the ubiquitous language used in the cim-keys system.
Terms are organized by bounded context.

## General Terms

| Term | Definition |
|------|------------|
| **CIM** | Composable Information Machine - A distributed system architecture using NATS messaging |
| **Bounded Context** | A logical boundary within which a particular domain model applies |
| **Published Language** | A well-documented set of types for cross-context communication |
| **Anti-Corruption Layer (ACL)** | A translation layer that prevents domain model pollution between contexts |
| **Aggregate** | A cluster of domain objects treated as a single unit |
| **Entity** | A domain object with a unique identity |
| **Value Object** | A domain object without identity, defined by its attributes |

## Organization Context

| Term | Definition | Type |
|------|------------|------|
| **Organization** | A company, department, or group that owns the CIM infrastructure | Entity |
| **OrganizationUnit** | A subdivision within an organization (department, team, project) | Entity |
| **Person** | An individual member of an organization who can own keys | Entity |
| **Location** | A physical or logical place where keys can be stored | Entity |
| **Role** | A named set of responsibilities and permissions within an organization | Entity |
| **Policy** | A set of rules governing access control and authorization | Entity |
| **KeyOwnerRole** | The PKI role of a person (RootAuthority, SecurityAdmin, Developer, etc.) | Value Object |

### Organization Reference Types (Published Language)

| Term | Definition |
|------|------------|
| **OrganizationReference** | Lightweight reference to an Organization for cross-context use |
| **PersonReference** | Lightweight reference to a Person for cross-context use |
| **LocationReference** | Lightweight reference to a Location for cross-context use |
| **RoleReference** | Lightweight reference to a Role for cross-context use |
| **OrganizationUnitReference** | Lightweight reference to an OrganizationUnit for cross-context use |

## PKI Context

| Term | Definition | Type |
|------|------------|------|
| **Certificate** | An X.509 certificate that binds a public key to an identity | Entity |
| **RootCertificate** | The top-level certificate in a PKI hierarchy (self-signed) | Entity |
| **IntermediateCertificate** | A certificate that chains to the root and can issue leaf certificates | Entity |
| **LeafCertificate** | An end-entity certificate for a person or service | Entity |
| **CryptographicKey** | A cryptographic key pair used for signing or encryption | Entity |
| **KeyOwnership** | The relationship between a key and its owner | Value Object |
| **KeyDelegation** | Authorization for one person to use another's keys | Value Object |
| **KeyPermission** | A specific capability granted by key ownership (Sign, Encrypt, Certify) | Value Object |
| **TrustChain** | The chain of certificates from leaf to root | Value Object |

### PKI Enumerations

| Term | Values | Definition |
|------|--------|------------|
| **CertificateType** | Root, Intermediate, Leaf, Policy | The type of certificate in the hierarchy |
| **CertificateStatus** | Pending, Active, Revoked, Expired, Suspended | The lifecycle state of a certificate |
| **KeyAlgorithm** | Ed25519, P256, RSA2048, RSA4096 | The cryptographic algorithm used |
| **KeyPurpose** | Signing, Encryption, Authentication, KeyAgreement | The intended use of a key |

### PKI Reference Types (Published Language)

| Term | Definition |
|------|------------|
| **KeyReference** | Lightweight reference to a CryptographicKey for cross-context use |
| **CertificateReference** | Lightweight reference to a Certificate for cross-context use |
| **KeyOwnershipReference** | Lightweight reference to key ownership relationship |
| **TrustChainReference** | Lightweight reference to a certificate trust chain |

## NATS Context

| Term | Definition | Type |
|------|------------|------|
| **NatsOperator** | The top-level NATS entity that manages accounts (maps to Organization) | Entity |
| **NatsAccount** | A NATS account for resource isolation (maps to OrganizationUnit) | Entity |
| **NatsUser** | A NATS user credential (maps to Person) | Entity |
| **NatsServiceAccount** | A NATS user for service-to-service communication | Entity |
| **NKey** | A NATS cryptographic key for signing JWTs | Value Object |
| **JWT** | A JSON Web Token for NATS authentication | Value Object |

### NATS Mappings

| NATS Entity | Organization Entity | Relationship |
|-------------|---------------------|--------------|
| Operator | Organization | 1:1 - Organization becomes Operator |
| Account | OrganizationUnit | 1:1 - Each unit is an Account |
| User | Person | 1:1 - Each person is a User |

### NATS ACL Types

| Term | Definition |
|------|------------|
| **PersonContextPort** | Interface for NATS to access Organization context |
| **PkiContextPort** | Interface for NATS to access PKI context |
| **NatsUserContext** | NATS user context using Published Language references |
| **NatsAccountContext** | NATS account context mapped from organizational unit |

## YubiKey Context

| Term | Definition | Type |
|------|------------|------|
| **YubiKeyDevice** | A physical YubiKey hardware security module | Entity |
| **PivSlot** | A PIV (Personal Identity Verification) slot on a YubiKey | Entity |
| **YubiKeyStatus** | The provisioning status for a person's YubiKey | Value Object |

### PIV Slots

| Slot | Name | Typical Use |
|------|------|-------------|
| **9A** | PIV Authentication | General authentication |
| **9C** | Digital Signature | Document signing |
| **9D** | Key Management | Key encipherment |
| **9E** | Card Authentication | Physical access |

## Graph/Visualization Context

| Term | Definition | Type |
|------|------------|------|
| **LiftedNode** | A domain entity lifted into the graph representation | Entity |
| **LiftedGraph** | A graph of lifted nodes with typed edges | Entity |
| **Injection** | The coproduct tag identifying a domain type | Value Object |
| **LiftableDomain** | Trait for domain types that can be lifted to graph | Trait |

### Graph Operations

| Term | Definition |
|------|------------|
| **lift()** | Transform a domain entity into a LiftedNode |
| **unlift()** | Recover a domain entity from a LiftedNode |
| **fold()** | Apply a morphism to transform a node (universal property) |

## Event Sourcing Terms

| Term | Definition |
|------|------------|
| **DomainEvent** | An immutable fact about something that happened in the domain |
| **Command** | An intention to change state (may be rejected) |
| **Aggregate** | A consistency boundary that processes commands and emits events |
| **Projection** | A read model built from events |
| **EventStore** | Persistent storage for events |
| **Saga** | A long-running process coordinating multiple aggregates |

### Event Types

| Term | Definition |
|------|------------|
| **OrganizationCreated** | Event when an organization is created |
| **PersonJoinedOrganization** | Event when a person joins |
| **KeyGeneratedForPerson** | Event when a key is generated |
| **CertificateIssued** | Event when a certificate is issued |
| **YubiKeyProvisioned** | Event when a YubiKey is provisioned |

## Architectural Patterns

| Term | Definition |
|------|------------|
| **MorphismRegistry** | Registry of morphisms stored as DATA (HashMap) not CODE (match) |
| **Kan Extension** | Category theory pattern for lifting operations across functors |
| **Coproduct** | Sum type with injection functions and universal property (fold) |
| **Functor** | Structure-preserving map between categories |

## FRP (Functional Reactive Programming) Terms

| Term | Definition |
|------|------------|
| **Signal** | A time-varying value |
| **Event** | A discrete occurrence at a point in time |
| **Behavior** | A continuous value over time |
| **Decoupled** | Output at time t depends only on input before t |
| **Causality** | Effects cannot precede their causes |

## Abbreviations

| Abbreviation | Full Term |
|--------------|-----------|
| **ACL** | Anti-Corruption Layer |
| **CA** | Certificate Authority |
| **CIM** | Composable Information Machine |
| **DDD** | Domain-Driven Design |
| **FRP** | Functional Reactive Programming |
| **HSM** | Hardware Security Module |
| **JWT** | JSON Web Token |
| **MVI** | Model-View-Intent |
| **NATS** | Neural Autonomic Transport System |
| **NKey** | NATS Key |
| **PIV** | Personal Identity Verification |
| **PKI** | Public Key Infrastructure |
| **TEA** | The Elm Architecture |
| **TLS** | Transport Layer Security |

## Related Documentation

- [Context Map](architecture/context-map.md) - Bounded context relationships
- [N-ary FRP Axioms](../N_ARY_FRP_AXIOMS.md) - FRP axiom definitions
- [CLAUDE.md](../CLAUDE.md) - Development guidelines
