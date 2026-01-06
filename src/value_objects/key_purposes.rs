// Copyright (c) 2025 - Cowboy AI, LLC.

//! Key Purpose Taxonomy
//!
//! Comprehensive key purposes mapping to authentication/authorization mechanisms.
//! Each purpose maps to specific YubiKey slots, GPG capabilities, and X.509 extensions.
//!
//! All types implement `cim_domain::ValueObject` marker trait.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::state_machines::{PivSlot, PinPolicy, TouchPolicy};

// Import DDD marker traits from cim-domain
use cim_domain::{DomainConcept, ValueObject};

// ============================================================================
// Key Purpose Taxonomy
// ============================================================================

/// Comprehensive key purpose for all authentication mechanisms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AuthKeyPurpose {
    // ========================================================================
    // SSO & Session Management
    // ========================================================================
    /// Single Sign-On session authentication
    /// Maps to: X.509 client cert, OIDC ID token signing
    SsoAuthentication,

    /// Session token signing (JWT)
    /// Maps to: Ed25519/RSA key for JWT signing
    SessionTokenSigning,

    // ========================================================================
    // SSH Access
    // ========================================================================
    /// SSH public key authentication
    /// Maps to: Ed25519/RSA SSH key, YubiKey slot 9a
    SshAuthentication,

    /// SSH certificate authority
    /// Maps to: SSH CA key for signing user/host certs
    SshCertificateAuthority,

    // ========================================================================
    // GPG Operations
    // ========================================================================
    /// GPG signing (git commits, emails, documents)
    /// Maps to: GPG signing subkey, YubiKey slot 9c with touch required
    GpgSigning,

    /// GPG encryption (email, file encryption)
    /// Maps to: GPG encryption subkey, YubiKey slot 9d
    GpgEncryption,

    /// GPG authentication (SSH via gpg-agent)
    /// Maps to: GPG authentication subkey, YubiKey slot 9a
    GpgAuthentication,

    /// GPG master key (certify only, kept offline)
    /// Maps to: GPG master key, NOT on YubiKey
    GpgMasterKey,

    // ========================================================================
    // X.509 Certificate Operations
    // ========================================================================
    /// X.509 TLS client authentication
    /// Maps to: Client certificate, YubiKey slot 9a
    X509ClientAuth,

    /// X.509 TLS server authentication
    /// Maps to: Server certificate with serverAuth extended key usage
    X509ServerAuth,

    /// X.509 code signing
    /// Maps to: Code signing certificate, YubiKey slot 9c
    X509CodeSigning,

    /// X.509 email protection (S/MIME)
    /// Maps to: Email certificate, slots 9c (sign) + 9d (encrypt)
    X509EmailProtection,

    // ========================================================================
    // OIDC & OAuth2
    // ========================================================================
    /// OIDC ID token signing
    /// Maps to: RSA/Ed25519 key for signing JWT ID tokens
    OidcIdTokenSigning,

    /// OIDC ID token encryption
    /// Maps to: RSA/ECDH key for encrypting JWT ID tokens
    OidcIdTokenEncryption,

    /// OAuth2 access token signing
    /// Maps to: RSA/Ed25519 key for signing JWT access tokens
    OAuth2AccessTokenSigning,

    /// OAuth2 refresh token encryption
    /// Maps to: AES/ChaCha20 key for encrypting refresh tokens
    OAuth2RefreshTokenEncryption,

    // ========================================================================
    // Passkeys & WebAuthn
    // ========================================================================
    /// WebAuthn/FIDO2 credential
    /// Maps to: FIDO2 resident credential on YubiKey
    WebAuthnCredential,

    /// FIDO U2F credential
    /// Maps to: U2F credential on YubiKey
    FidoU2fCredential,

    // ========================================================================
    // 2FA & OTP
    // ========================================================================
    /// TOTP (Time-based One-Time Password)
    /// Maps to: OATH-TOTP secret on YubiKey
    TotpSecret,

    /// HOTP (HMAC-based One-Time Password)
    /// Maps to: OATH-HOTP secret on YubiKey
    HotpSecret,

    /// Yubico OTP
    /// Maps to: Yubico OTP slot on YubiKey
    YubicoOtp,

    // ========================================================================
    // Touch Authorization
    // ========================================================================
    /// Touch-to-authorize for sensitive operations
    /// Maps to: Any key with touch policy = Always
    TouchAuthorization,

    // ========================================================================
    // CIM-Specific
    // ========================================================================
    /// NATS JWT signing (operator/account/user)
    /// Maps to: Ed25519 key for NATS credentials
    NatsJwtSigning,

    /// DID document signing
    /// Maps to: Ed25519 key for W3C DID operations
    DidDocumentSigning,

    /// Verifiable credential signing
    /// Maps to: Ed25519 key for W3C VC operations
    VerifiableCredentialSigning,
}

impl AuthKeyPurpose {
    /// Get recommended PIV slot for this purpose
    pub fn recommended_piv_slot(&self) -> Option<PivSlot> {
        match self {
            // Authentication → Slot 9a
            AuthKeyPurpose::SsoAuthentication
            | AuthKeyPurpose::SshAuthentication
            | AuthKeyPurpose::GpgAuthentication
            | AuthKeyPurpose::X509ClientAuth => Some(PivSlot::Authentication),

            // Signing → Slot 9c
            AuthKeyPurpose::GpgSigning
            | AuthKeyPurpose::X509CodeSigning
            | AuthKeyPurpose::X509EmailProtection
            | AuthKeyPurpose::SessionTokenSigning
            | AuthKeyPurpose::OidcIdTokenSigning
            | AuthKeyPurpose::OAuth2AccessTokenSigning
            | AuthKeyPurpose::NatsJwtSigning
            | AuthKeyPurpose::DidDocumentSigning
            | AuthKeyPurpose::VerifiableCredentialSigning => Some(PivSlot::Signature),

            // Encryption → Slot 9d
            AuthKeyPurpose::GpgEncryption
            | AuthKeyPurpose::OidcIdTokenEncryption
            | AuthKeyPurpose::OAuth2RefreshTokenEncryption => Some(PivSlot::KeyManagement),

            // Card auth → Slot 9e (passwordless)
            AuthKeyPurpose::WebAuthnCredential
            | AuthKeyPurpose::FidoU2fCredential => Some(PivSlot::CardAuth),

            // Not stored in PIV slots
            AuthKeyPurpose::GpgMasterKey
            | AuthKeyPurpose::SshCertificateAuthority
            | AuthKeyPurpose::X509ServerAuth
            | AuthKeyPurpose::TotpSecret
            | AuthKeyPurpose::HotpSecret
            | AuthKeyPurpose::YubicoOtp
            | AuthKeyPurpose::TouchAuthorization => None,
        }
    }

    /// Get recommended key algorithm
    pub fn recommended_algorithm(&self) -> KeyAlgorithmRecommendation {
        match self {
            // Modern: Ed25519 for signing
            AuthKeyPurpose::SshAuthentication
            | AuthKeyPurpose::GpgSigning
            | AuthKeyPurpose::GpgAuthentication
            | AuthKeyPurpose::SessionTokenSigning
            | AuthKeyPurpose::NatsJwtSigning
            | AuthKeyPurpose::DidDocumentSigning
            | AuthKeyPurpose::VerifiableCredentialSigning => {
                KeyAlgorithmRecommendation::Ed25519
            }

            // Modern: X25519 for encryption
            AuthKeyPurpose::GpgEncryption => KeyAlgorithmRecommendation::X25519,

            // OIDC/OAuth2: RSA for compatibility, Ed25519 preferred
            AuthKeyPurpose::OidcIdTokenSigning
            | AuthKeyPurpose::OAuth2AccessTokenSigning => {
                KeyAlgorithmRecommendation::Ed25519OrRsa2048
            }

            // Encryption: RSA or ECDH
            AuthKeyPurpose::OidcIdTokenEncryption
            | AuthKeyPurpose::OAuth2RefreshTokenEncryption => {
                KeyAlgorithmRecommendation::EcdhP256OrRsa2048
            }

            // X.509: ECDSA P-256 for broad compatibility
            AuthKeyPurpose::SsoAuthentication
            | AuthKeyPurpose::X509ClientAuth
            | AuthKeyPurpose::X509ServerAuth
            | AuthKeyPurpose::X509CodeSigning
            | AuthKeyPurpose::X509EmailProtection => KeyAlgorithmRecommendation::EcdsaP256,

            // CA operations: ECDSA P-384 for higher security
            AuthKeyPurpose::SshCertificateAuthority
            | AuthKeyPurpose::GpgMasterKey => KeyAlgorithmRecommendation::EcdsaP384,

            // WebAuthn: ES256 (ECDSA P-256)
            AuthKeyPurpose::WebAuthnCredential
            | AuthKeyPurpose::FidoU2fCredential => KeyAlgorithmRecommendation::EcdsaP256,

            // OTP: Not applicable (symmetric secrets)
            AuthKeyPurpose::TotpSecret
            | AuthKeyPurpose::HotpSecret
            | AuthKeyPurpose::YubicoOtp
            | AuthKeyPurpose::TouchAuthorization => KeyAlgorithmRecommendation::NotApplicable,
        }
    }

    /// Get recommended PIN policy
    pub fn recommended_pin_policy(&self) -> PinPolicy {
        match self {
            // Always require PIN for signing operations
            AuthKeyPurpose::GpgSigning
            | AuthKeyPurpose::X509CodeSigning
            | AuthKeyPurpose::NatsJwtSigning
            | AuthKeyPurpose::DidDocumentSigning
            | AuthKeyPurpose::VerifiableCredentialSigning => PinPolicy::Always,

            // Once per session for authentication
            AuthKeyPurpose::SsoAuthentication
            | AuthKeyPurpose::SshAuthentication
            | AuthKeyPurpose::GpgAuthentication
            | AuthKeyPurpose::X509ClientAuth
            | AuthKeyPurpose::SessionTokenSigning
            | AuthKeyPurpose::OidcIdTokenSigning
            | AuthKeyPurpose::OAuth2AccessTokenSigning
            | AuthKeyPurpose::GpgEncryption => PinPolicy::Once,

            // Never for passwordless/card auth
            AuthKeyPurpose::WebAuthnCredential
            | AuthKeyPurpose::FidoU2fCredential => PinPolicy::Never,

            // Once for everything else
            _ => PinPolicy::Once,
        }
    }

    /// Get recommended touch policy
    pub fn recommended_touch_policy(&self) -> TouchPolicy {
        match self {
            // ALWAYS require touch for signing operations (non-repudiation)
            AuthKeyPurpose::GpgSigning
            | AuthKeyPurpose::X509CodeSigning
            | AuthKeyPurpose::TouchAuthorization => TouchPolicy::Always,

            // Cached for WebAuthn (good UX while still secure)
            AuthKeyPurpose::WebAuthnCredential
            | AuthKeyPurpose::FidoU2fCredential => TouchPolicy::Cached,

            // Never for convenience (PIN provides security)
            AuthKeyPurpose::SshAuthentication
            | AuthKeyPurpose::GpgAuthentication
            | AuthKeyPurpose::GpgEncryption
            | AuthKeyPurpose::X509ClientAuth
            | AuthKeyPurpose::SsoAuthentication => TouchPolicy::Never,

            // Cached for everything else (good balance)
            _ => TouchPolicy::Cached,
        }
    }

    /// Check if this purpose requires YubiKey storage
    pub fn requires_hardware_storage(&self) -> bool {
        matches!(
            self,
            AuthKeyPurpose::GpgSigning
                | AuthKeyPurpose::GpgEncryption
                | AuthKeyPurpose::GpgAuthentication
                | AuthKeyPurpose::X509ClientAuth
                | AuthKeyPurpose::X509CodeSigning
                | AuthKeyPurpose::WebAuthnCredential
                | AuthKeyPurpose::FidoU2fCredential
                | AuthKeyPurpose::TouchAuthorization
        )
    }

    /// Check if this purpose supports offline operation
    pub fn supports_offline(&self) -> bool {
        !matches!(
            self,
            AuthKeyPurpose::WebAuthnCredential
                | AuthKeyPurpose::FidoU2fCredential
                | AuthKeyPurpose::TotpSecret
                | AuthKeyPurpose::HotpSecret
        )
    }

    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            AuthKeyPurpose::SsoAuthentication => "Single Sign-On authentication",
            AuthKeyPurpose::SessionTokenSigning => "Session token (JWT) signing",
            AuthKeyPurpose::SshAuthentication => "SSH public key authentication",
            AuthKeyPurpose::SshCertificateAuthority => "SSH certificate authority",
            AuthKeyPurpose::GpgSigning => "GPG signing (commits, emails, documents)",
            AuthKeyPurpose::GpgEncryption => "GPG encryption (emails, files)",
            AuthKeyPurpose::GpgAuthentication => "GPG authentication (SSH via gpg-agent)",
            AuthKeyPurpose::GpgMasterKey => "GPG master key (offline, certify only)",
            AuthKeyPurpose::X509ClientAuth => "X.509 TLS client authentication",
            AuthKeyPurpose::X509ServerAuth => "X.509 TLS server authentication",
            AuthKeyPurpose::X509CodeSigning => "X.509 code signing",
            AuthKeyPurpose::X509EmailProtection => "X.509 email protection (S/MIME)",
            AuthKeyPurpose::OidcIdTokenSigning => "OIDC ID token signing",
            AuthKeyPurpose::OidcIdTokenEncryption => "OIDC ID token encryption",
            AuthKeyPurpose::OAuth2AccessTokenSigning => "OAuth2 access token signing",
            AuthKeyPurpose::OAuth2RefreshTokenEncryption => "OAuth2 refresh token encryption",
            AuthKeyPurpose::WebAuthnCredential => "WebAuthn/FIDO2 passkey",
            AuthKeyPurpose::FidoU2fCredential => "FIDO U2F second factor",
            AuthKeyPurpose::TotpSecret => "TOTP (time-based OTP)",
            AuthKeyPurpose::HotpSecret => "HOTP (counter-based OTP)",
            AuthKeyPurpose::YubicoOtp => "Yubico OTP",
            AuthKeyPurpose::TouchAuthorization => "Touch-to-authorize",
            AuthKeyPurpose::NatsJwtSigning => "NATS JWT credential signing",
            AuthKeyPurpose::DidDocumentSigning => "DID document signing",
            AuthKeyPurpose::VerifiableCredentialSigning => "Verifiable credential signing",
        }
    }
}

impl fmt::Display for AuthKeyPurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Key algorithm recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAlgorithmRecommendation {
    /// Ed25519 (modern, fast, secure)
    Ed25519,
    /// X25519 (ECDH for encryption)
    X25519,
    /// ECDSA P-256 (broad compatibility)
    EcdsaP256,
    /// ECDSA P-384 (higher security)
    EcdsaP384,
    /// Ed25519 preferred, RSA 2048 acceptable
    Ed25519OrRsa2048,
    /// ECDH P-256 preferred, RSA 2048 acceptable
    EcdhP256OrRsa2048,
    /// Not applicable (symmetric key or OTP)
    NotApplicable,
}

// ============================================================================
// Key Bundle - Complete set of keys for a person
// ============================================================================

/// Complete key bundle for a person supporting all auth mechanisms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonKeyBundle {
    /// Person identifier
    pub person_id: uuid::Uuid,
    /// Person name
    pub person_name: String,
    /// YubiKey serial (if using hardware)
    pub yubikey_serial: Option<String>,
    /// All keys organized by purpose
    pub keys: Vec<KeyAssignment>,
}

/// Key assignment to slot/purpose
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyAssignment {
    pub purpose: AuthKeyPurpose,
    pub key_id: uuid::Uuid,
    pub piv_slot: Option<PivSlot>,
    pub pin_policy: PinPolicy,
    pub touch_policy: TouchPolicy,
}

impl PersonKeyBundle {
    /// Create a standard key bundle for a person
    pub fn standard_bundle(person_id: uuid::Uuid, person_name: String) -> Self {
        Self {
            person_id,
            person_name,
            yubikey_serial: None,
            keys: vec![
                // Slot 9a: SSH + TLS client auth
                KeyAssignment {
                    purpose: AuthKeyPurpose::SshAuthentication,
                    key_id: uuid::Uuid::now_v7(),
                    piv_slot: Some(PivSlot::Authentication),
                    pin_policy: PinPolicy::Once,
                    touch_policy: TouchPolicy::Never,
                },
                // Slot 9c: GPG + code signing
                KeyAssignment {
                    purpose: AuthKeyPurpose::GpgSigning,
                    key_id: uuid::Uuid::now_v7(),
                    piv_slot: Some(PivSlot::Signature),
                    pin_policy: PinPolicy::Always,
                    touch_policy: TouchPolicy::Always,
                },
                // Slot 9d: GPG encryption
                KeyAssignment {
                    purpose: AuthKeyPurpose::GpgEncryption,
                    key_id: uuid::Uuid::now_v7(),
                    piv_slot: Some(PivSlot::KeyManagement),
                    pin_policy: PinPolicy::Once,
                    touch_policy: TouchPolicy::Never,
                },
                // Slot 9e: WebAuthn passkeys
                KeyAssignment {
                    purpose: AuthKeyPurpose::WebAuthnCredential,
                    key_id: uuid::Uuid::now_v7(),
                    piv_slot: Some(PivSlot::CardAuth),
                    pin_policy: PinPolicy::Never,
                    touch_policy: TouchPolicy::Cached,
                },
            ],
        }
    }

    /// Get key assignment for a specific purpose
    pub fn get_key_for_purpose(&self, purpose: AuthKeyPurpose) -> Option<&KeyAssignment> {
        self.keys.iter().find(|k| k.purpose == purpose)
    }
}

// ============================================================================
// DDD Marker Trait Implementations
// ============================================================================

impl DomainConcept for AuthKeyPurpose {}
impl ValueObject for AuthKeyPurpose {}

impl DomainConcept for KeyAlgorithmRecommendation {}
impl ValueObject for KeyAlgorithmRecommendation {}

impl DomainConcept for PersonKeyBundle {}
impl ValueObject for PersonKeyBundle {}

impl DomainConcept for KeyAssignment {}
impl ValueObject for KeyAssignment {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purpose_slot_mapping() {
        assert_eq!(
            AuthKeyPurpose::SshAuthentication.recommended_piv_slot(),
            Some(PivSlot::Authentication)
        );
        assert_eq!(
            AuthKeyPurpose::GpgSigning.recommended_piv_slot(),
            Some(PivSlot::Signature)
        );
        assert_eq!(
            AuthKeyPurpose::GpgEncryption.recommended_piv_slot(),
            Some(PivSlot::KeyManagement)
        );
    }

    #[test]
    fn test_purpose_security_policies() {
        // Signing should always require PIN + touch
        assert_eq!(
            AuthKeyPurpose::GpgSigning.recommended_pin_policy(),
            PinPolicy::Always
        );
        assert_eq!(
            AuthKeyPurpose::GpgSigning.recommended_touch_policy(),
            TouchPolicy::Always
        );

        // WebAuthn should never require PIN but use cached touch
        assert_eq!(
            AuthKeyPurpose::WebAuthnCredential.recommended_pin_policy(),
            PinPolicy::Never
        );
        assert_eq!(
            AuthKeyPurpose::WebAuthnCredential.recommended_touch_policy(),
            TouchPolicy::Cached
        );
    }

    #[test]
    fn test_standard_bundle() {
        let bundle = PersonKeyBundle::standard_bundle(
            uuid::Uuid::now_v7(),
            "Alice".to_string(),
        );

        assert_eq!(bundle.keys.len(), 4);
        assert!(bundle
            .get_key_for_purpose(AuthKeyPurpose::SshAuthentication)
            .is_some());
        assert!(bundle
            .get_key_for_purpose(AuthKeyPurpose::GpgSigning)
            .is_some());
    }
}
