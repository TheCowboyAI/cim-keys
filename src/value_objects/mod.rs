// Copyright (c) 2025 - Cowboy AI, LLC.

//! Value Objects Module
//!
//! All cryptographic artifacts and security parameters as immutable value objects.
//! Values have no identity - they are defined entirely by their attributes.
//!
//! All value objects implement `cim_domain::ValueObject` marker trait.

pub mod core;
pub mod yubikey;
pub mod key_purposes;
pub mod nats;
pub mod x509;

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

// Re-export X.509 types
pub use x509::{
    CommonName,
    CountryCode,
    EmailAddress,
    LocalityName,
    OrganizationalUnitName,
    OrganizationName,
    StateName,
    SubjectName,
    SubjectNameError,
};
