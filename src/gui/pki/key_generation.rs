// Copyright (c) 2025 - Cowboy AI, LLC.

//! PKI Key Generation Module
//!
//! Handles all PKI-related operations including:
//! - Certificate authority hierarchy (Root, Intermediate, Leaf)
//! - SSH key generation
//! - GPG key management
//! - Key recovery from deterministic seed
//! - Client certificates (mTLS)
//! - Multi-purpose key bundles

use iced::Task;
use uuid::Uuid;

use crate::domain::{InvariantKeyPurpose, OrganizationUnit};
use crate::ports::gpg::{GpgKeyInfo, GpgKeypair, GpgKeyType};
use crate::projections::CertificateEntry;
use crate::crypto::x509::X509Certificate;

use super::super::Message;

/// PKI domain messages
///
/// These messages handle all PKI-related UI interactions and
/// async operation results.
#[derive(Debug, Clone)]
pub enum PkiMessage {
    // ============================================================================
    // Root CA Operations
    // ============================================================================
    /// Generate Root CA certificate
    GenerateRootCA,
    /// Toggle Root CA section visibility
    ToggleRootCASection,
    /// Root CA generation completed
    RootCAGenerated(Result<X509Certificate, String>),

    // ============================================================================
    // Intermediate CA Operations
    // ============================================================================
    /// Intermediate CA name changed
    IntermediateCANameChanged(String),
    /// Select organizational unit for intermediate CA
    SelectUnitForCA(String),
    /// Generate intermediate CA
    GenerateIntermediateCA,
    /// Toggle Intermediate CA section visibility
    ToggleIntermediateCASection,

    // ============================================================================
    // Server Certificate Operations
    // ============================================================================
    /// Server cert common name changed
    ServerCertCNChanged(String),
    /// Server cert SANs changed
    ServerCertSANsChanged(String),
    /// Select intermediate CA for signing
    SelectIntermediateCA(String),
    /// Select storage location for certificate
    SelectCertLocation(String),
    /// Generate server certificate
    GenerateServerCert,
    /// Toggle Server Cert section visibility
    ToggleServerCertSection,

    // ============================================================================
    // Certificate Metadata Fields
    // ============================================================================
    /// Certificate organization changed
    CertOrganizationChanged(String),
    /// Certificate organizational unit changed
    CertOrganizationalUnitChanged(String),
    /// Certificate locality changed
    CertLocalityChanged(String),
    /// Certificate state/province changed
    CertStateProvinceChanged(String),
    /// Certificate country changed
    CertCountryChanged(String),
    /// Certificate validity days changed
    CertValidityDaysChanged(String),

    // ============================================================================
    // SSH Key Operations
    // ============================================================================
    /// Generate SSH keys
    GenerateSSHKeys,

    // ============================================================================
    // General Key Operations
    // ============================================================================
    /// Generate all keys
    GenerateAllKeys,
    /// Key generation progress update
    KeyGenerationProgress(f32),
    /// Keys generated result
    KeysGenerated(Result<usize, String>),
    /// Toggle Certificates section visibility
    ToggleCertificatesSection,
    /// Toggle Keys section visibility
    ToggleKeysSection,

    // ============================================================================
    // GPG Key Operations
    // ============================================================================
    /// GPG user ID changed
    GpgUserIdChanged(String),
    /// GPG key type selected
    GpgKeyTypeSelected(GpgKeyType),
    /// GPG key length changed
    GpgKeyLengthChanged(String),
    /// GPG expiration days changed
    GpgExpiresDaysChanged(String),
    /// Generate GPG key
    GenerateGpgKey,
    /// GPG key generated result
    GpgKeyGenerated(Result<GpgKeypair, String>),
    /// Toggle GPG section visibility
    ToggleGpgSection,
    /// List existing GPG keys
    ListGpgKeys,
    /// GPG keys listed result
    GpgKeysListed(Result<Vec<GpgKeyInfo>, String>),

    // ============================================================================
    // Key Recovery Operations
    // ============================================================================
    /// Toggle recovery section visibility
    ToggleRecoverySection,
    /// Recovery passphrase changed
    RecoveryPassphraseChanged(String),
    /// Recovery passphrase confirmation changed
    RecoveryPassphraseConfirmChanged(String),
    /// Recovery organization ID changed
    RecoveryOrganizationIdChanged(String),
    /// Verify recovery seed
    VerifyRecoverySeed,
    /// Recovery seed verified result
    RecoverySeedVerified(Result<String, String>),
    /// Recover keys from seed
    RecoverKeysFromSeed,
    /// Keys recovered result
    KeysRecovered(Result<usize, String>),

    // ============================================================================
    // Client Certificate (mTLS) Operations
    // ============================================================================
    /// Client cert common name changed
    ClientCertCNChanged(String),
    /// Client cert email changed
    ClientCertEmailChanged(String),
    /// Generate client certificate
    GenerateClientCert,
    /// Client cert generated result
    ClientCertGenerated(Result<String, String>),

    // ============================================================================
    // Multi-Purpose Key Operations
    // ============================================================================
    /// Toggle multi-purpose key section visibility
    ToggleMultiPurposeKeySection,
    /// Select person for multi-purpose keys
    MultiPurposePersonSelected(Uuid),
    /// Toggle a key purpose on/off
    ToggleKeyPurpose(InvariantKeyPurpose),
    /// Generate multi-purpose keys
    GenerateMultiPurposeKeys,
    /// Multi-purpose keys generated result
    MultiPurposeKeysGenerated(Result<(Uuid, Vec<String>), String>),

    // ============================================================================
    // Root Passphrase Operations
    // ============================================================================
    /// Root passphrase changed
    RootPassphraseChanged(String),
    /// Root passphrase confirmation changed
    RootPassphraseConfirmChanged(String),
    /// Toggle passphrase visibility
    TogglePassphraseVisibility,
    /// Generate random passphrase
    GenerateRandomPassphrase,

    // ============================================================================
    // Graph-Based PKI Operations
    // ============================================================================
    /// PKI certificates loaded from projection
    PkiCertificatesLoaded(Vec<CertificateEntry>),
    /// Generate PKI from graph
    GeneratePkiFromGraph,
    /// Personal keys generated result
    PersonalKeysGenerated(Result<(X509Certificate, Vec<String>), String>),
}

/// PKI domain state
///
/// Holds all state relevant to the PKI bounded context.
/// Field names match CimKeysApp for easy state synchronization.
#[derive(Debug, Clone, Default)]
pub struct PkiState {
    // Key generation progress
    pub key_generation_progress: f32,
    pub keys_generated: usize,
    pub total_keys_to_generate: usize,

    // Certificate generation fields
    pub intermediate_ca_name_input: String,
    pub server_cert_cn_input: String,
    pub server_cert_sans_input: String,
    pub selected_intermediate_ca: Option<String>,
    pub selected_cert_location: Option<String>,
    pub loaded_units: Vec<OrganizationUnit>,
    pub selected_unit_for_ca: Option<String>,

    // Certificate metadata fields
    pub cert_organization: String,
    pub cert_organizational_unit: String,
    pub cert_locality: String,
    pub cert_state_province: String,
    pub cert_country: String,
    pub cert_validity_days: String,

    // Loaded certificates
    pub loaded_certificates: Vec<CertificateEntry>,

    // Collapsible sections state
    pub root_ca_collapsed: bool,
    pub intermediate_ca_collapsed: bool,
    pub server_cert_collapsed: bool,
    pub certificates_collapsed: bool,
    pub keys_collapsed: bool,
    pub gpg_section_collapsed: bool,

    // GPG key generation state
    pub gpg_user_id: String,
    pub gpg_key_type: Option<GpgKeyType>,
    pub gpg_key_length: String,
    pub gpg_expires_days: String,
    pub generated_gpg_keys: Vec<GpgKeyInfo>,
    pub gpg_generation_status: Option<String>,

    // Key recovery state
    pub recovery_section_collapsed: bool,
    pub recovery_passphrase: String,
    pub recovery_passphrase_confirm: String,
    pub recovery_organization_id: String,
    pub recovery_status: Option<String>,
    pub recovery_seed_verified: bool,

    // Client certificate state
    pub client_cert_cn: String,
    pub client_cert_email: String,

    // Multi-purpose key state
    pub multi_purpose_key_section_collapsed: bool,
    pub multi_purpose_selected_person: Option<Uuid>,
    pub multi_purpose_selected_purposes: std::collections::HashSet<InvariantKeyPurpose>,

    // Root passphrase
    pub root_passphrase: String,
    pub root_passphrase_confirm: String,
    pub show_passphrase: bool,
}

impl PkiState {
    /// Create new PKI state with default values
    pub fn new() -> Self {
        Self {
            cert_validity_days: "365".to_string(),
            gpg_key_length: "4096".to_string(),
            gpg_expires_days: "365".to_string(),
            ..Default::default()
        }
    }

    /// Check if passphrase is valid
    pub fn is_passphrase_valid(&self) -> bool {
        let matches = self.root_passphrase == self.root_passphrase_confirm;
        let long_enough = self.root_passphrase.len() >= 12;
        let has_upper = self.root_passphrase.chars().any(|c| c.is_uppercase());
        let has_lower = self.root_passphrase.chars().any(|c| c.is_lowercase());
        let has_digit = self.root_passphrase.chars().any(|c| c.is_numeric());
        let has_special = self.root_passphrase.chars().any(|c| !c.is_alphanumeric());

        matches && long_enough && has_upper && has_lower && has_digit && has_special
    }
}

/// Update function for PKI messages
///
/// Handles simple state mutations. Complex operations that need
/// aggregate/projection access return Task::none() and are handled
/// by the main update() in gui.rs.
pub fn update(state: &mut PkiState, message: PkiMessage) -> Task<Message> {
    match message {
        // ========================================================================
        // Root CA Operations
        // ========================================================================
        PkiMessage::GenerateRootCA => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        PkiMessage::ToggleRootCASection => {
            state.root_ca_collapsed = !state.root_ca_collapsed;
            Task::none()
        }
        PkiMessage::RootCAGenerated(_) => {
            // Handled by main update - needs graph access
            Task::none()
        }

        // ========================================================================
        // Intermediate CA Operations
        // ========================================================================
        PkiMessage::IntermediateCANameChanged(name) => {
            state.intermediate_ca_name_input = name;
            Task::none()
        }
        PkiMessage::SelectUnitForCA(unit) => {
            state.selected_unit_for_ca = Some(unit);
            Task::none()
        }
        PkiMessage::GenerateIntermediateCA => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        PkiMessage::ToggleIntermediateCASection => {
            state.intermediate_ca_collapsed = !state.intermediate_ca_collapsed;
            Task::none()
        }

        // ========================================================================
        // Server Certificate Operations
        // ========================================================================
        PkiMessage::ServerCertCNChanged(cn) => {
            state.server_cert_cn_input = cn;
            Task::none()
        }
        PkiMessage::ServerCertSANsChanged(sans) => {
            state.server_cert_sans_input = sans;
            Task::none()
        }
        PkiMessage::SelectIntermediateCA(ca) => {
            state.selected_intermediate_ca = Some(ca);
            Task::none()
        }
        PkiMessage::SelectCertLocation(loc) => {
            state.selected_cert_location = Some(loc);
            Task::none()
        }
        PkiMessage::GenerateServerCert => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        PkiMessage::ToggleServerCertSection => {
            state.server_cert_collapsed = !state.server_cert_collapsed;
            Task::none()
        }

        // ========================================================================
        // Certificate Metadata Fields
        // ========================================================================
        PkiMessage::CertOrganizationChanged(org) => {
            state.cert_organization = org;
            Task::none()
        }
        PkiMessage::CertOrganizationalUnitChanged(ou) => {
            state.cert_organizational_unit = ou;
            Task::none()
        }
        PkiMessage::CertLocalityChanged(locality) => {
            state.cert_locality = locality;
            Task::none()
        }
        PkiMessage::CertStateProvinceChanged(sp) => {
            state.cert_state_province = sp;
            Task::none()
        }
        PkiMessage::CertCountryChanged(country) => {
            state.cert_country = country;
            Task::none()
        }
        PkiMessage::CertValidityDaysChanged(days) => {
            state.cert_validity_days = days;
            Task::none()
        }

        // ========================================================================
        // SSH Key Operations
        // ========================================================================
        PkiMessage::GenerateSSHKeys => {
            // Handled by main update - needs port access
            Task::none()
        }

        // ========================================================================
        // General Key Operations
        // ========================================================================
        PkiMessage::GenerateAllKeys => {
            // Handled by main update - needs port access
            Task::none()
        }
        PkiMessage::KeyGenerationProgress(progress) => {
            state.key_generation_progress = progress;
            Task::none()
        }
        PkiMessage::KeysGenerated(result) => {
            if let Ok(count) = result {
                state.keys_generated = count;
            }
            state.key_generation_progress = 0.0;
            Task::none()
        }
        PkiMessage::ToggleCertificatesSection => {
            state.certificates_collapsed = !state.certificates_collapsed;
            Task::none()
        }
        PkiMessage::ToggleKeysSection => {
            state.keys_collapsed = !state.keys_collapsed;
            Task::none()
        }

        // ========================================================================
        // GPG Key Operations
        // ========================================================================
        PkiMessage::GpgUserIdChanged(user_id) => {
            state.gpg_user_id = user_id;
            Task::none()
        }
        PkiMessage::GpgKeyTypeSelected(key_type) => {
            state.gpg_key_type = Some(key_type);
            Task::none()
        }
        PkiMessage::GpgKeyLengthChanged(length) => {
            state.gpg_key_length = length;
            Task::none()
        }
        PkiMessage::GpgExpiresDaysChanged(days) => {
            state.gpg_expires_days = days;
            Task::none()
        }
        PkiMessage::GenerateGpgKey => {
            // Handled by main update - needs port access
            Task::none()
        }
        PkiMessage::GpgKeyGenerated(result) => {
            match result {
                Ok(_keypair) => {
                    state.gpg_generation_status = Some("GPG key generated successfully".to_string());
                }
                Err(e) => {
                    state.gpg_generation_status = Some(format!("GPG key generation failed: {}", e));
                }
            }
            Task::none()
        }
        PkiMessage::ToggleGpgSection => {
            state.gpg_section_collapsed = !state.gpg_section_collapsed;
            Task::none()
        }
        PkiMessage::ListGpgKeys => {
            // Handled by main update - needs port access
            Task::none()
        }
        PkiMessage::GpgKeysListed(result) => {
            if let Ok(keys) = result {
                state.generated_gpg_keys = keys;
            }
            Task::none()
        }

        // ========================================================================
        // Key Recovery Operations
        // ========================================================================
        PkiMessage::ToggleRecoverySection => {
            state.recovery_section_collapsed = !state.recovery_section_collapsed;
            Task::none()
        }
        PkiMessage::RecoveryPassphraseChanged(passphrase) => {
            state.recovery_passphrase = passphrase;
            Task::none()
        }
        PkiMessage::RecoveryPassphraseConfirmChanged(passphrase) => {
            state.recovery_passphrase_confirm = passphrase;
            Task::none()
        }
        PkiMessage::RecoveryOrganizationIdChanged(org_id) => {
            state.recovery_organization_id = org_id;
            Task::none()
        }
        PkiMessage::VerifyRecoverySeed => {
            // Handled by main update - needs crypto access
            Task::none()
        }
        PkiMessage::RecoverySeedVerified(result) => {
            match result {
                Ok(fingerprint) => {
                    state.recovery_seed_verified = true;
                    state.recovery_status = Some(format!("Seed verified: {}", fingerprint));
                }
                Err(e) => {
                    state.recovery_seed_verified = false;
                    state.recovery_status = Some(format!("Verification failed: {}", e));
                }
            }
            Task::none()
        }
        PkiMessage::RecoverKeysFromSeed => {
            // Handled by main update - needs crypto access
            Task::none()
        }
        PkiMessage::KeysRecovered(result) => {
            match result {
                Ok(count) => {
                    state.recovery_status = Some(format!("Recovered {} keys", count));
                }
                Err(e) => {
                    state.recovery_status = Some(format!("Recovery failed: {}", e));
                }
            }
            Task::none()
        }

        // ========================================================================
        // Client Certificate (mTLS) Operations
        // ========================================================================
        PkiMessage::ClientCertCNChanged(cn) => {
            state.client_cert_cn = cn;
            Task::none()
        }
        PkiMessage::ClientCertEmailChanged(email) => {
            state.client_cert_email = email;
            Task::none()
        }
        PkiMessage::GenerateClientCert => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        PkiMessage::ClientCertGenerated(_) => {
            // Status handled by main update
            Task::none()
        }

        // ========================================================================
        // Multi-Purpose Key Operations
        // ========================================================================
        PkiMessage::ToggleMultiPurposeKeySection => {
            state.multi_purpose_key_section_collapsed = !state.multi_purpose_key_section_collapsed;
            Task::none()
        }
        PkiMessage::MultiPurposePersonSelected(person_id) => {
            state.multi_purpose_selected_person = Some(person_id);
            Task::none()
        }
        PkiMessage::ToggleKeyPurpose(purpose) => {
            if state.multi_purpose_selected_purposes.contains(&purpose) {
                state.multi_purpose_selected_purposes.remove(&purpose);
            } else {
                state.multi_purpose_selected_purposes.insert(purpose);
            }
            Task::none()
        }
        PkiMessage::GenerateMultiPurposeKeys => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        PkiMessage::MultiPurposeKeysGenerated(result) => {
            if result.is_ok() {
                // Clear selections on success
                state.multi_purpose_selected_person = None;
                state.multi_purpose_selected_purposes.clear();
            }
            Task::none()
        }

        // ========================================================================
        // Root Passphrase Operations
        // ========================================================================
        PkiMessage::RootPassphraseChanged(passphrase) => {
            state.root_passphrase = passphrase;
            Task::none()
        }
        PkiMessage::RootPassphraseConfirmChanged(passphrase) => {
            state.root_passphrase_confirm = passphrase;
            Task::none()
        }
        PkiMessage::TogglePassphraseVisibility => {
            state.show_passphrase = !state.show_passphrase;
            Task::none()
        }
        PkiMessage::GenerateRandomPassphrase => {
            // Generate a random passphrase (handled locally)
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%^&*"
                .chars()
                .collect();
            let passphrase: String = (0..20)
                .map(|_| chars[rng.gen_range(0..chars.len())])
                .collect();
            state.root_passphrase = passphrase.clone();
            state.root_passphrase_confirm = passphrase;
            Task::none()
        }

        // ========================================================================
        // Graph-Based PKI Operations
        // ========================================================================
        PkiMessage::PkiCertificatesLoaded(certs) => {
            state.loaded_certificates = certs;
            Task::none()
        }
        PkiMessage::GeneratePkiFromGraph => {
            // Handled by main update - needs graph access
            Task::none()
        }
        PkiMessage::PersonalKeysGenerated(_) => {
            // Status handled by main update
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pki_state_default() {
        let state = PkiState::default();
        assert!(!state.root_ca_collapsed);
        assert!(!state.gpg_section_collapsed);
        assert!(!state.recovery_seed_verified);
        assert!(state.cert_validity_days.is_empty());
    }

    #[test]
    fn test_pki_state_new() {
        let state = PkiState::new();
        assert_eq!(state.cert_validity_days, "365");
        assert_eq!(state.gpg_key_length, "4096");
        assert_eq!(state.gpg_expires_days, "365");
    }

    #[test]
    fn test_passphrase_validation() {
        let mut state = PkiState::default();

        // Too short
        state.root_passphrase = "Short1!".to_string();
        state.root_passphrase_confirm = "Short1!".to_string();
        assert!(!state.is_passphrase_valid());

        // Missing special char
        state.root_passphrase = "LongEnough123".to_string();
        state.root_passphrase_confirm = "LongEnough123".to_string();
        assert!(!state.is_passphrase_valid());

        // Valid
        state.root_passphrase = "ValidPass123!@#".to_string();
        state.root_passphrase_confirm = "ValidPass123!@#".to_string();
        assert!(state.is_passphrase_valid());

        // Mismatch
        state.root_passphrase_confirm = "Different123!@#".to_string();
        assert!(!state.is_passphrase_valid());
    }

    #[test]
    fn test_toggle_sections() {
        let mut state = PkiState::default();

        let _ = update(&mut state, PkiMessage::ToggleRootCASection);
        assert!(state.root_ca_collapsed);

        let _ = update(&mut state, PkiMessage::ToggleGpgSection);
        assert!(state.gpg_section_collapsed);

        let _ = update(&mut state, PkiMessage::ToggleRecoverySection);
        assert!(state.recovery_section_collapsed);
    }

    #[test]
    fn test_cert_metadata_updates() {
        let mut state = PkiState::default();

        let _ = update(&mut state, PkiMessage::CertOrganizationChanged("CowboyAI".to_string()));
        assert_eq!(state.cert_organization, "CowboyAI");

        let _ = update(&mut state, PkiMessage::CertCountryChanged("US".to_string()));
        assert_eq!(state.cert_country, "US");

        let _ = update(&mut state, PkiMessage::CertValidityDaysChanged("730".to_string()));
        assert_eq!(state.cert_validity_days, "730");
    }

    #[test]
    fn test_multi_purpose_key_toggle() {
        let mut state = PkiState::default();

        let _ = update(&mut state, PkiMessage::ToggleKeyPurpose(InvariantKeyPurpose::Signing));
        assert!(state.multi_purpose_selected_purposes.contains(&InvariantKeyPurpose::Signing));

        let _ = update(&mut state, PkiMessage::ToggleKeyPurpose(InvariantKeyPurpose::Signing));
        assert!(!state.multi_purpose_selected_purposes.contains(&InvariantKeyPurpose::Signing));
    }

    #[test]
    fn test_key_generation_progress() {
        let mut state = PkiState::default();

        let _ = update(&mut state, PkiMessage::KeyGenerationProgress(0.5));
        assert_eq!(state.key_generation_progress, 0.5);

        let _ = update(&mut state, PkiMessage::KeysGenerated(Ok(10)));
        assert_eq!(state.keys_generated, 10);
        assert_eq!(state.key_generation_progress, 0.0); // Reset after completion
    }
}
