// Domain Projections Module
//
// Projections are functors that map domain compositions to library-native formats.
// These are pure transformations - no side effects, no I/O.
//
// In Category Theory terms:
// - Domain Category: Our domain objects (Organization, Person, KeyContext, etc.)
// - Target Category: Library formats (CSR, X509Params, PIVSlots, JWT claims, etc.)
// - Projection: A functor F: Domain → Target
//
// Each projection module provides mappings for specific operations:
// - certificate: Domain → CSR → X509 params
// - yubikey: Domain → PIV provisioning params
// - nats: Domain → JWT claims / NATS config
// - ssi: Domain → DID documents / Verifiable Credentials

pub mod certificate;
pub mod yubikey;
pub mod nats;
pub mod ssi;

// Re-export key types
pub use certificate::{
    CertificateRequestProjection,
    CertificateSigningRequest,
    X509Extensions,
};

pub use yubikey::{
    YubiKeyProvisioningProjection,
    PivSlotConfiguration,
};

pub use nats::{
    JwtClaimsProjection,
    JwtSigningProjection,
    NatsIdentityProjection,
    NatsProjection,
    NKeyGenerationParams,
    NKeyProjection,
    OrganizationBootstrap,
};

pub use ssi::{
    DidDocumentProjection,
    VerifiableCredentialProjection,
};
