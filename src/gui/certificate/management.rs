// Copyright (c) 2025 - Cowboy AI, LLC.

//! Certificate Management Bounded Context
//!
//! This module implements the Certificate domain with:
//! - Message enum for all certificate operations
//! - State struct for certificate-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility toggles
//! 2. **Metadata Form**: X.509 certificate fields
//! 3. **Intermediate CA**: Create and select intermediate CAs
//! 4. **Server Certificate**: Generate server certificates with SANs
//! 5. **Client Certificate**: mTLS client certificate generation
//! 6. **Chain View**: Certificate chain visualization

use iced::Task;
use uuid::Uuid;

use crate::projections::CertificateEntry;

/// Certificate Message
///
/// Organized by sub-domain:
/// - UI State (3 messages)
/// - Metadata Form (6 messages)
/// - Intermediate CA (3 messages)
/// - Server Certificate (4 messages)
/// - Client Certificate (2 messages)
/// - Chain View (1 message)
/// - Loading (1 message)
#[derive(Debug, Clone)]
pub enum CertificateMessage {
    // === UI State ===
    /// Toggle certificates list section visibility
    ToggleCertificatesSection,
    /// Toggle intermediate CA subsection visibility
    ToggleIntermediateCA,
    /// Toggle server certificate subsection visibility
    ToggleServerCert,

    // === Metadata Form ===
    /// X.509 Organization field changed
    OrganizationChanged(String),
    /// X.509 Organizational Unit field changed
    OrganizationalUnitChanged(String),
    /// X.509 Locality field changed
    LocalityChanged(String),
    /// X.509 State/Province field changed
    StateProvinceChanged(String),
    /// X.509 Country field changed
    CountryChanged(String),
    /// Certificate validity period (days) changed
    ValidityDaysChanged(String),

    // === Intermediate CA ===
    /// Intermediate CA name input changed
    IntermediateCANameChanged(String),
    /// Select an existing intermediate CA
    SelectIntermediateCA(String),
    /// Generate a new intermediate CA
    GenerateIntermediateCA,

    // === Server Certificate ===
    /// Server certificate Common Name changed
    ServerCNChanged(String),
    /// Server certificate Subject Alternative Names changed
    ServerSANsChanged(String),
    /// Select storage location for certificate
    SelectLocation(String),
    /// Generate server certificate
    GenerateServerCert,

    // === Client Certificate (mTLS) ===
    /// Generate mTLS client certificate
    GenerateClientCert,
    /// Client certificate generation result
    ClientCertGenerated(Result<String, String>),

    // === Chain View ===
    /// Select certificate for chain visualization
    SelectForChainView(Uuid),

    // === Loading ===
    /// Certificates loaded from manifest
    CertificatesLoaded(Vec<CertificateEntry>),
}

/// Certificate State
///
/// Contains all state related to certificate management.
#[derive(Debug, Clone)]
pub struct CertificateState {
    // === UI State ===
    /// Whether the certificates list section is collapsed
    pub certificates_collapsed: bool,
    /// Whether the intermediate CA section is collapsed
    pub intermediate_ca_collapsed: bool,
    /// Whether the server certificate section is collapsed
    pub server_cert_collapsed: bool,

    // === Metadata Form ===
    /// X.509 Organization (O)
    pub organization: String,
    /// X.509 Organizational Unit (OU)
    pub organizational_unit: String,
    /// X.509 Locality (L)
    pub locality: String,
    /// X.509 State/Province (ST)
    pub state_province: String,
    /// X.509 Country (C)
    pub country: String,
    /// Certificate validity in days
    pub validity_days: String,

    // === Intermediate CA ===
    /// Name for new intermediate CA
    pub intermediate_ca_name: String,
    /// Currently selected intermediate CA
    pub selected_intermediate_ca: Option<String>,
    /// Selected organizational unit for CA
    pub selected_unit_for_ca: Option<String>,

    // === Server Certificate ===
    /// Server certificate Common Name (CN)
    pub server_cn: String,
    /// Server certificate Subject Alternative Names
    pub server_sans: String,
    /// Selected storage location
    pub selected_location: Option<String>,

    // === Chain View ===
    /// Certificate selected for chain view
    pub selected_chain_cert: Option<Uuid>,

    // === Loaded Data ===
    /// Certificates loaded from manifest
    pub loaded_certificates: Vec<CertificateEntry>,
    /// Count of certificates generated this session
    pub certificates_generated: usize,
}

impl Default for CertificateState {
    fn default() -> Self {
        Self {
            certificates_collapsed: true,
            intermediate_ca_collapsed: false,
            server_cert_collapsed: false,
            organization: String::new(),
            organizational_unit: String::new(),
            locality: String::new(),
            state_province: String::new(),
            country: String::new(),
            validity_days: "365".to_string(),
            intermediate_ca_name: String::new(),
            selected_intermediate_ca: None,
            selected_unit_for_ca: None,
            server_cn: String::new(),
            server_sans: String::new(),
            selected_location: None,
            selected_chain_cert: None,
            loaded_certificates: Vec::new(),
            certificates_generated: 0,
        }
    }
}

impl CertificateState {
    /// Create a new CertificateState with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if metadata form has required fields
    pub fn is_metadata_valid(&self) -> bool {
        !self.organization.is_empty() && !self.country.is_empty()
    }

    /// Check if ready to generate intermediate CA
    pub fn is_intermediate_ca_ready(&self) -> bool {
        !self.intermediate_ca_name.is_empty() && self.is_metadata_valid()
    }

    /// Check if ready to generate server certificate
    pub fn is_server_cert_ready(&self) -> bool {
        !self.server_cn.is_empty()
            && self.selected_intermediate_ca.is_some()
            && self.is_metadata_valid()
    }

    /// Get validation error for metadata
    pub fn metadata_validation_error(&self) -> Option<String> {
        if self.organization.is_empty() {
            return Some("Organization (O) is required".to_string());
        }
        if self.country.is_empty() {
            return Some("Country (C) is required".to_string());
        }
        None
    }

    /// Get validation error for intermediate CA
    pub fn intermediate_ca_validation_error(&self) -> Option<String> {
        if self.intermediate_ca_name.is_empty() {
            return Some("Intermediate CA name is required".to_string());
        }
        self.metadata_validation_error()
    }

    /// Get validation error for server certificate
    pub fn server_cert_validation_error(&self) -> Option<String> {
        if self.server_cn.is_empty() {
            return Some("Server Common Name (CN) is required".to_string());
        }
        if self.selected_intermediate_ca.is_none() {
            return Some("Please select a signing CA".to_string());
        }
        self.metadata_validation_error()
    }

    /// Parse validity days to u32
    pub fn validity_days_parsed(&self) -> Option<u32> {
        self.validity_days.parse().ok()
    }

    /// Get validity days or default
    pub fn validity_days_or_default(&self) -> u32 {
        self.validity_days_parsed().unwrap_or(365)
    }

    /// Clear intermediate CA form
    pub fn clear_intermediate_ca_form(&mut self) {
        self.intermediate_ca_name.clear();
        self.selected_unit_for_ca = None;
    }

    /// Clear server certificate form
    pub fn clear_server_cert_form(&mut self) {
        self.server_cn.clear();
        self.server_sans.clear();
        self.selected_location = None;
    }

    /// Get count of loaded certificates
    pub fn certificate_count(&self) -> usize {
        self.loaded_certificates.len()
    }

    /// Find certificate by ID
    pub fn find_certificate(&self, cert_id: Uuid) -> Option<&CertificateEntry> {
        self.loaded_certificates.iter().find(|c| c.cert_id == cert_id)
    }

    /// Get CA certificates
    pub fn ca_certificates(&self) -> Vec<&CertificateEntry> {
        self.loaded_certificates
            .iter()
            .filter(|c| c.is_ca)
            .collect()
    }

    /// Get leaf certificates (non-CA)
    pub fn leaf_certificates(&self) -> Vec<&CertificateEntry> {
        self.loaded_certificates
            .iter()
            .filter(|c| !c.is_ca)
            .collect()
    }

    /// Get intermediate CAs for selection (CAs that have an issuer)
    pub fn intermediate_cas(&self) -> Vec<&CertificateEntry> {
        self.loaded_certificates
            .iter()
            .filter(|c| c.is_ca && c.issuer.is_some())
            .collect()
    }

    /// Get certificate subjects for dropdown
    pub fn certificate_subjects(&self) -> Vec<String> {
        self.loaded_certificates
            .iter()
            .map(|c| c.subject.clone())
            .collect()
    }

    /// Parse SANs from comma-separated string
    pub fn parsed_sans(&self) -> Vec<String> {
        self.server_sans
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update certificate state based on message
///
/// This function handles certificate domain messages. Note that generation
/// operations require crypto context and will be delegated to the main update.
pub fn update(state: &mut CertificateState, message: CertificateMessage) -> Task<Message> {
    use CertificateMessage::*;

    match message {
        // === UI State ===
        ToggleCertificatesSection => {
            state.certificates_collapsed = !state.certificates_collapsed;
            Task::none()
        }

        ToggleIntermediateCA => {
            state.intermediate_ca_collapsed = !state.intermediate_ca_collapsed;
            Task::none()
        }

        ToggleServerCert => {
            state.server_cert_collapsed = !state.server_cert_collapsed;
            Task::none()
        }

        // === Metadata Form ===
        OrganizationChanged(org) => {
            state.organization = org;
            Task::none()
        }

        OrganizationalUnitChanged(ou) => {
            state.organizational_unit = ou;
            Task::none()
        }

        LocalityChanged(locality) => {
            state.locality = locality;
            Task::none()
        }

        StateProvinceChanged(st) => {
            state.state_province = st;
            Task::none()
        }

        CountryChanged(country) => {
            state.country = country;
            Task::none()
        }

        ValidityDaysChanged(days) => {
            state.validity_days = days;
            Task::none()
        }

        // === Intermediate CA ===
        IntermediateCANameChanged(name) => {
            state.intermediate_ca_name = name;
            Task::none()
        }

        SelectIntermediateCA(ca_name) => {
            state.selected_intermediate_ca = if ca_name.is_empty() {
                None
            } else {
                Some(ca_name)
            };
            Task::none()
        }

        GenerateIntermediateCA => {
            // Delegated to main for crypto operations
            Task::none()
        }

        // === Server Certificate ===
        ServerCNChanged(cn) => {
            state.server_cn = cn;
            Task::none()
        }

        ServerSANsChanged(sans) => {
            state.server_sans = sans;
            Task::none()
        }

        SelectLocation(location) => {
            state.selected_location = if location.is_empty() {
                None
            } else {
                Some(location)
            };
            Task::none()
        }

        GenerateServerCert => {
            // Delegated to main for crypto operations
            Task::none()
        }

        // === Client Certificate ===
        GenerateClientCert => {
            // Delegated to main for crypto operations
            Task::none()
        }

        ClientCertGenerated(_result) => {
            // Delegated to main for status handling
            Task::none()
        }

        // === Chain View ===
        SelectForChainView(cert_id) => {
            state.selected_chain_cert = Some(cert_id);
            Task::none()
        }

        // === Loading ===
        CertificatesLoaded(certificates) => {
            state.loaded_certificates = certificates;
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_state_default() {
        let state = CertificateState::default();
        assert!(state.certificates_collapsed);
        assert!(!state.intermediate_ca_collapsed);
        assert!(!state.server_cert_collapsed);
        assert!(state.organization.is_empty());
        assert_eq!(state.validity_days, "365");
        assert!(state.loaded_certificates.is_empty());
    }

    #[test]
    fn test_certificate_state_new() {
        let state = CertificateState::new();
        assert!(state.certificates_collapsed);
        assert_eq!(state.validity_days, "365");
    }

    #[test]
    fn test_toggle_certificates_section() {
        let mut state = CertificateState::new();
        assert!(state.certificates_collapsed);

        let _ = update(&mut state, CertificateMessage::ToggleCertificatesSection);
        assert!(!state.certificates_collapsed);

        let _ = update(&mut state, CertificateMessage::ToggleCertificatesSection);
        assert!(state.certificates_collapsed);
    }

    #[test]
    fn test_toggle_intermediate_ca() {
        let mut state = CertificateState::new();
        let initial = state.intermediate_ca_collapsed;

        let _ = update(&mut state, CertificateMessage::ToggleIntermediateCA);
        assert_eq!(state.intermediate_ca_collapsed, !initial);
    }

    #[test]
    fn test_toggle_server_cert() {
        let mut state = CertificateState::new();
        let initial = state.server_cert_collapsed;

        let _ = update(&mut state, CertificateMessage::ToggleServerCert);
        assert_eq!(state.server_cert_collapsed, !initial);
    }

    #[test]
    fn test_organization_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::OrganizationChanged("Cowboy AI".to_string()),
        );
        assert_eq!(state.organization, "Cowboy AI");
    }

    #[test]
    fn test_organizational_unit_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::OrganizationalUnitChanged("Engineering".to_string()),
        );
        assert_eq!(state.organizational_unit, "Engineering");
    }

    #[test]
    fn test_locality_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::LocalityChanged("Austin".to_string()),
        );
        assert_eq!(state.locality, "Austin");
    }

    #[test]
    fn test_state_province_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::StateProvinceChanged("Texas".to_string()),
        );
        assert_eq!(state.state_province, "Texas");
    }

    #[test]
    fn test_country_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::CountryChanged("US".to_string()),
        );
        assert_eq!(state.country, "US");
    }

    #[test]
    fn test_validity_days_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::ValidityDaysChanged("730".to_string()),
        );
        assert_eq!(state.validity_days, "730");
    }

    #[test]
    fn test_intermediate_ca_name_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::IntermediateCANameChanged("Engineering CA".to_string()),
        );
        assert_eq!(state.intermediate_ca_name, "Engineering CA");
    }

    #[test]
    fn test_select_intermediate_ca() {
        let mut state = CertificateState::new();

        let _ = update(
            &mut state,
            CertificateMessage::SelectIntermediateCA("Root CA".to_string()),
        );
        assert_eq!(state.selected_intermediate_ca, Some("Root CA".to_string()));

        // Empty string clears selection
        let _ = update(
            &mut state,
            CertificateMessage::SelectIntermediateCA(String::new()),
        );
        assert!(state.selected_intermediate_ca.is_none());
    }

    #[test]
    fn test_server_cn_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::ServerCNChanged("api.example.com".to_string()),
        );
        assert_eq!(state.server_cn, "api.example.com");
    }

    #[test]
    fn test_server_sans_changed() {
        let mut state = CertificateState::new();
        let _ = update(
            &mut state,
            CertificateMessage::ServerSANsChanged("api.example.com, *.api.example.com".to_string()),
        );
        assert_eq!(state.server_sans, "api.example.com, *.api.example.com");
    }

    #[test]
    fn test_select_location() {
        let mut state = CertificateState::new();

        let _ = update(
            &mut state,
            CertificateMessage::SelectLocation("YubiKey-001".to_string()),
        );
        assert_eq!(state.selected_location, Some("YubiKey-001".to_string()));

        // Empty string clears selection
        let _ = update(
            &mut state,
            CertificateMessage::SelectLocation(String::new()),
        );
        assert!(state.selected_location.is_none());
    }

    #[test]
    fn test_select_for_chain_view() {
        let mut state = CertificateState::new();
        let cert_id = Uuid::now_v7();

        let _ = update(&mut state, CertificateMessage::SelectForChainView(cert_id));
        assert_eq!(state.selected_chain_cert, Some(cert_id));
    }

    #[test]
    fn test_is_metadata_valid() {
        let mut state = CertificateState::new();
        assert!(!state.is_metadata_valid());

        state.organization = "Cowboy AI".to_string();
        assert!(!state.is_metadata_valid());

        state.country = "US".to_string();
        assert!(state.is_metadata_valid());
    }

    #[test]
    fn test_is_intermediate_ca_ready() {
        let mut state = CertificateState::new();
        assert!(!state.is_intermediate_ca_ready());

        state.intermediate_ca_name = "Engineering CA".to_string();
        assert!(!state.is_intermediate_ca_ready());

        state.organization = "Cowboy AI".to_string();
        state.country = "US".to_string();
        assert!(state.is_intermediate_ca_ready());
    }

    #[test]
    fn test_is_server_cert_ready() {
        let mut state = CertificateState::new();
        assert!(!state.is_server_cert_ready());

        state.server_cn = "api.example.com".to_string();
        assert!(!state.is_server_cert_ready());

        state.selected_intermediate_ca = Some("Engineering CA".to_string());
        assert!(!state.is_server_cert_ready());

        state.organization = "Cowboy AI".to_string();
        state.country = "US".to_string();
        assert!(state.is_server_cert_ready());
    }

    #[test]
    fn test_metadata_validation_error() {
        let mut state = CertificateState::new();
        assert_eq!(
            state.metadata_validation_error(),
            Some("Organization (O) is required".to_string())
        );

        state.organization = "Cowboy AI".to_string();
        assert_eq!(
            state.metadata_validation_error(),
            Some("Country (C) is required".to_string())
        );

        state.country = "US".to_string();
        assert!(state.metadata_validation_error().is_none());
    }

    #[test]
    fn test_intermediate_ca_validation_error() {
        let mut state = CertificateState::new();
        assert_eq!(
            state.intermediate_ca_validation_error(),
            Some("Intermediate CA name is required".to_string())
        );

        state.intermediate_ca_name = "Engineering CA".to_string();
        // Now metadata error takes over
        assert_eq!(
            state.intermediate_ca_validation_error(),
            Some("Organization (O) is required".to_string())
        );
    }

    #[test]
    fn test_server_cert_validation_error() {
        let mut state = CertificateState::new();
        assert_eq!(
            state.server_cert_validation_error(),
            Some("Server Common Name (CN) is required".to_string())
        );

        state.server_cn = "api.example.com".to_string();
        assert_eq!(
            state.server_cert_validation_error(),
            Some("Please select a signing CA".to_string())
        );
    }

    #[test]
    fn test_validity_days_parsed() {
        let mut state = CertificateState::new();
        assert_eq!(state.validity_days_parsed(), Some(365));

        state.validity_days = "730".to_string();
        assert_eq!(state.validity_days_parsed(), Some(730));

        state.validity_days = "invalid".to_string();
        assert!(state.validity_days_parsed().is_none());
    }

    #[test]
    fn test_validity_days_or_default() {
        let mut state = CertificateState::new();
        assert_eq!(state.validity_days_or_default(), 365);

        state.validity_days = "invalid".to_string();
        assert_eq!(state.validity_days_or_default(), 365);

        state.validity_days = "730".to_string();
        assert_eq!(state.validity_days_or_default(), 730);
    }

    #[test]
    fn test_clear_intermediate_ca_form() {
        let mut state = CertificateState::new();
        state.intermediate_ca_name = "Engineering CA".to_string();
        state.selected_unit_for_ca = Some("Engineering".to_string());

        state.clear_intermediate_ca_form();

        assert!(state.intermediate_ca_name.is_empty());
        assert!(state.selected_unit_for_ca.is_none());
    }

    #[test]
    fn test_clear_server_cert_form() {
        let mut state = CertificateState::new();
        state.server_cn = "api.example.com".to_string();
        state.server_sans = "*.api.example.com".to_string();
        state.selected_location = Some("YubiKey-001".to_string());

        state.clear_server_cert_form();

        assert!(state.server_cn.is_empty());
        assert!(state.server_sans.is_empty());
        assert!(state.selected_location.is_none());
    }

    #[test]
    fn test_parsed_sans() {
        let mut state = CertificateState::new();
        state.server_sans = "api.example.com, *.api.example.com, localhost".to_string();

        let sans = state.parsed_sans();
        assert_eq!(sans.len(), 3);
        assert_eq!(sans[0], "api.example.com");
        assert_eq!(sans[1], "*.api.example.com");
        assert_eq!(sans[2], "localhost");
    }

    #[test]
    fn test_parsed_sans_empty() {
        let state = CertificateState::new();
        assert!(state.parsed_sans().is_empty());
    }

    #[test]
    fn test_parsed_sans_with_empty_entries() {
        let mut state = CertificateState::new();
        state.server_sans = "api.example.com, , *.api.example.com".to_string();

        let sans = state.parsed_sans();
        assert_eq!(sans.len(), 2);
    }
}
