// Copyright (c) 2025 - Cowboy AI, LLC.

//! Value Objects Module
//!
//! All cryptographic artifacts and security parameters as immutable value objects.
//! Values have no identity - they are defined entirely by their attributes.
//!
//! All value objects implement `cim_domain::ValueObject` marker trait.
//!
//! ## Domain Ownership
//!
//! Value objects belong to their owning domains:
//! - **Person domain** (`cim-domain-person`): PersonName, EmailAddress, PhoneNumber
//! - **Location domain** (`cim-domain-location`): VirtualUrl, IpAddress, VirtualLocation
//! - **Organization domain** (`cim-domain-organization`): Organization-specific types
//!
//! ## PKI Bounded Context
//!
//! This module contains PKI-specific value objects that compose from domain types:
//! - **X.509 types**: SubjectName, KeyUsage, ExtendedKeyUsage, BasicConstraints
//! - **NATS credentials**: NKey types, JWT claims
//! - **YubiKey**: PIV configuration and slot management
//!
//! When building a CSR or Certificate, the bounded context composes:
//! - Subject identity from Person domain
//! - Organization from Organization domain
//! - Location (email, address) from Location domain
//! - PKI-specific extensions from this module

pub mod core;
pub mod key_purposes;
pub mod nats;
pub mod x509;
pub mod yubikey;

// Re-export core types
pub use core::*;

// Re-export YubiKey types
pub use yubikey::{
    FirmwareVersion,
    ManagementKeyAlgorithm,
    ManagementKeyValue,
    PinValue,
    PukValue,
    SlotKeyAlgorithm,
    SlotPinPolicy,
    SlotState,
    SlotStatus,
    SlotTouchPolicy,
    YubiKeyPivConfiguration,
};

// Re-export Key Purpose types
pub use key_purposes::{
    AuthKeyPurpose,
    KeyAssignment,
    PersonKeyBundle,
};

// Re-export NATS types
pub use nats::{
    AccountClaims,
    AccountData,
    AccountLimits,
    NatsCredential,
    NatsJwt,
    NatsJwtHeader,
    NKeyPair,
    NKeyPublic,
    NKeySeed,
    NKeyType,
    OperatorClaims,
    OperatorData,
    Permissions,
    UserClaims,
    UserData,
    UserLimits,
};

// Re-export X.509 types (PKI bounded context)
pub use x509::{
    BasicConstraints,
    CertificateValidity,
    CommonName,
    CountryCode,
    DnsName,
    EmailAddress,
    ExtendedKeyUsage,
    ExtendedKeyUsagePurpose,
    KeyUsage,
    KeyUsageBit,
    LocalityName,
    OrganizationalUnitName,
    OrganizationName,
    SanEmail,
    SanEntry,
    SanError,
    SanIpAddress,
    SanUri,
    StateName,
    SubjectAlternativeName,
    SubjectName,
    SubjectNameError,
    ValidityError,
};

// ============================================================================
// Domain Re-exports (when features enabled)
// ============================================================================

// Re-export from cim-domain-location (always available)
pub use cim_domain_location::value_objects::{
    IpAddress, IpAddressType, NetworkInfo, PortMapping, UrlType, VirtualLocation,
    VirtualLocationType, VirtualUrl,
};

// Re-export from cim-domain-person (when feature enabled)
#[cfg(feature = "cross-domain")]
pub use cim_domain_person::value_objects::{
    EmailAddress as PersonEmailAddress, PersonName as DomainPersonName, PhoneNumber,
};
