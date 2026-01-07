// Copyright (c) 2025 - Cowboy AI, LLC.

//! PKI Message Definitions
//!
//! This module defines the message types for the PKI bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **Root CA**: Root certificate authority generation
//! 2. **Intermediate CA**: Intermediate CA management
//! 3. **Server Certificates**: Server certificate generation
//! 4. **SSH Keys**: SSH key generation
//! 5. **GPG Keys**: GPG key management
//! 6. **Key Recovery**: Recovery from seed
//! 7. **Client Certificates**: mTLS client certs

use uuid::Uuid;

use crate::crypto::x509::X509Certificate;
use crate::domain::InvariantKeyPurpose;
use crate::ports::gpg::{GpgKeyInfo, GpgKeypair, GpgKeyType};
use crate::projections::CertificateEntry;

/// PKI domain messages
#[derive(Debug, Clone)]
pub enum PkiMessage {
    // === Root CA Operations ===
    GenerateRootCA,
    ToggleRootCASection,
    RootCAGenerated(Result<X509Certificate, String>),

    // === Intermediate CA Operations ===
    IntermediateCANameChanged(String),
    SelectUnitForCA(String),
    GenerateIntermediateCA,
    ToggleIntermediateCASection,

    // === Server Certificate Operations ===
    ServerCertCNChanged(String),
    ServerCertSANsChanged(String),
    SelectIntermediateCA(String),
    SelectCertLocation(String),
    GenerateServerCert,
    ToggleServerCertSection,

    // === Certificate Metadata Fields ===
    CertOrganizationChanged(String),
    CertOrganizationalUnitChanged(String),
    CertLocalityChanged(String),
    CertStateProvinceChanged(String),
    CertCountryChanged(String),
    CertValidityDaysChanged(String),

    // === SSH Key Operations ===
    GenerateSSHKeys,

    // === General Key Operations ===
    GenerateAllKeys,
    KeyGenerationProgress(f32),
    KeysGenerated(Result<usize, String>),
    ToggleCertificatesSection,
    ToggleKeysSection,

    // === GPG Key Operations ===
    GpgUserIdChanged(String),
    GpgKeyTypeSelected(GpgKeyType),
    GpgKeyLengthChanged(String),
    GpgExpiresDaysChanged(String),
    GenerateGpgKey,
    GpgKeyGenerated(Result<GpgKeypair, String>),
    ToggleGpgSection,
    ListGpgKeys,
    GpgKeysListed(Result<Vec<GpgKeyInfo>, String>),

    // === Key Recovery Operations ===
    ToggleRecoverySection,
    RecoveryPassphraseChanged(String),
    RecoveryPassphraseConfirmChanged(String),
    RecoveryOrganizationIdChanged(String),
    VerifyRecoverySeed,
    RecoverySeedVerified(Result<String, String>),
    RecoverKeysFromSeed,
    KeysRecovered(Result<usize, String>),

    // === Client Certificate (mTLS) Operations ===
    ClientCertCNChanged(String),
    ClientCertEmailChanged(String),
    GenerateClientCert,
    ClientCertGenerated(Result<String, String>),

    // === Multi-Purpose Key Operations ===
    ToggleMultiPurposeKeySection,
    MultiPurposePersonSelected(Uuid),
    ToggleKeyPurpose(InvariantKeyPurpose),
    GenerateMultiPurposeKeys,
    MultiPurposeKeysGenerated(Result<(Uuid, Vec<String>), String>),

    // === Root Passphrase Operations ===
    RootPassphraseChanged(String),
    RootPassphraseConfirmChanged(String),
    TogglePassphraseVisibility,
    GenerateRandomPassphrase,

    // === Graph-Based PKI Operations ===
    PkiCertificatesLoaded(Vec<CertificateEntry>),
    GeneratePkiFromGraph,
    PersonalKeysGenerated(Result<(X509Certificate, Vec<String>), String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pki_message_variants() {
        let _ = PkiMessage::GenerateRootCA;
        let _ = PkiMessage::ToggleRootCASection;
        let _ = PkiMessage::IntermediateCANameChanged("CA".to_string());
        let _ = PkiMessage::SelectUnitForCA("Unit".to_string());
        let _ = PkiMessage::GenerateIntermediateCA;
        let _ = PkiMessage::ServerCertCNChanged("cn".to_string());
        let _ = PkiMessage::ServerCertSANsChanged("*.example.com".to_string());
        let _ = PkiMessage::SelectIntermediateCA("CA".to_string());
        let _ = PkiMessage::SelectCertLocation("loc".to_string());
        let _ = PkiMessage::CertOrganizationChanged("Org".to_string());
        let _ = PkiMessage::CertValidityDaysChanged("365".to_string());
        let _ = PkiMessage::GenerateSSHKeys;
        let _ = PkiMessage::GenerateAllKeys;
        let _ = PkiMessage::KeyGenerationProgress(0.5);
        let _ = PkiMessage::ToggleCertificatesSection;
        let _ = PkiMessage::ToggleKeysSection;
        let _ = PkiMessage::GpgUserIdChanged("user".to_string());
        let _ = PkiMessage::GpgKeyLengthChanged("4096".to_string());
        let _ = PkiMessage::ToggleGpgSection;
        let _ = PkiMessage::ListGpgKeys;
        let _ = PkiMessage::ToggleRecoverySection;
        let _ = PkiMessage::RecoveryPassphraseChanged("pass".to_string());
        let _ = PkiMessage::VerifyRecoverySeed;
        let _ = PkiMessage::RecoverKeysFromSeed;
        let _ = PkiMessage::ClientCertCNChanged("cn".to_string());
        let _ = PkiMessage::GenerateClientCert;
        let _ = PkiMessage::ToggleMultiPurposeKeySection;
        let _ = PkiMessage::MultiPurposePersonSelected(Uuid::nil());
        let _ = PkiMessage::ToggleKeyPurpose(InvariantKeyPurpose::Signing);
        let _ = PkiMessage::RootPassphraseChanged("pass".to_string());
        let _ = PkiMessage::TogglePassphraseVisibility;
        let _ = PkiMessage::GenerateRandomPassphrase;
        let _ = PkiMessage::PkiCertificatesLoaded(vec![]);
        let _ = PkiMessage::GeneratePkiFromGraph;
    }
}
