// Copyright (c) 2025 - Cowboy AI, LLC.

//! TrustChain Verification Bounded Context
//!
//! This module implements the TrustChain domain with:
//! - Message enum for all trust chain operations
//! - State struct for verification-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **Section Toggle**: UI visibility
//! 2. **Certificate Selection**: Select certificate for chain view
//! 3. **Verification**: Verify single or all chains
//! 4. **Status Tracking**: Verification status per certificate

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use iced::Task;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a certificate in the trust chain verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustChainStatus {
    /// Not yet verified
    Pending,
    /// Chain verified successfully to a trusted root
    Verified {
        chain_length: usize,
        root_subject: String,
    },
    /// Chain verification failed
    Failed { reason: String },
    /// Certificate is expired
    Expired { expired_at: DateTime<Utc> },
    /// Certificate is self-signed (root)
    SelfSigned,
    /// Issuer certificate not found
    IssuerNotFound { expected_issuer: String },
}

impl Default for TrustChainStatus {
    fn default() -> Self {
        TrustChainStatus::Pending
    }
}

impl TrustChainStatus {
    /// Check if verification was successful
    pub fn is_verified(&self) -> bool {
        matches!(self, TrustChainStatus::Verified { .. })
    }

    /// Check if verification failed
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            TrustChainStatus::Failed { .. }
                | TrustChainStatus::Expired { .. }
                | TrustChainStatus::IssuerNotFound { .. }
        )
    }

    /// Check if certificate is a root (self-signed)
    pub fn is_root(&self) -> bool {
        matches!(self, TrustChainStatus::SelfSigned)
    }

    /// Check if still pending verification
    pub fn is_pending(&self) -> bool {
        matches!(self, TrustChainStatus::Pending)
    }

    /// Get a human-readable status string
    pub fn display_status(&self) -> String {
        match self {
            TrustChainStatus::Pending => "⏳ Pending".to_string(),
            TrustChainStatus::Verified {
                chain_length,
                root_subject,
            } => format!("✅ Verified (chain: {}, root: {})", chain_length, root_subject),
            TrustChainStatus::Failed { reason } => format!("❌ Failed: {}", reason),
            TrustChainStatus::Expired { expired_at } => {
                format!("⚠️ Expired: {}", expired_at.format("%Y-%m-%d"))
            }
            TrustChainStatus::SelfSigned => "🔐 Self-Signed (Root)".to_string(),
            TrustChainStatus::IssuerNotFound { expected_issuer } => {
                format!("❓ Issuer not found: {}", expected_issuer)
            }
        }
    }
}

/// TrustChain Message
///
/// Organized by sub-domain:
/// - Section Toggle (1 message)
/// - Certificate Selection (1 message)
/// - Verification (3 messages)
#[derive(Debug, Clone)]
pub enum TrustChainMessage {
    // === Section Toggle ===
    /// Toggle trust chain section visibility
    ToggleTrustChainSection,

    // === Certificate Selection ===
    /// Select a certificate for trust chain view
    SelectCertForTrustChain(Uuid),

    // === Verification ===
    /// Verify a specific certificate's trust chain
    VerifyTrustChain(Uuid),
    /// Trust chain verification completed
    TrustChainVerified(Result<(Uuid, TrustChainStatus), String>),
    /// Verify all certificate trust chains
    VerifyAllTrustChains,
}

/// TrustChain State
///
/// Contains all state related to trust chain verification.
#[derive(Debug, Clone, Default)]
pub struct TrustChainState {
    // === UI State ===
    /// Whether trust chain section is collapsed
    pub trust_chain_section_collapsed: bool,

    // === Certificate Selection ===
    /// Certificate selected for chain view
    pub selected_trust_chain_cert: Option<Uuid>,

    // === Verification Status ===
    /// Verification status per certificate
    pub trust_chain_verification_status: HashMap<Uuid, TrustChainStatus>,
}

impl TrustChainState {
    /// Create a new TrustChainState with sensible defaults
    pub fn new() -> Self {
        Self {
            trust_chain_section_collapsed: true,
            selected_trust_chain_cert: None,
            trust_chain_verification_status: HashMap::new(),
        }
    }

    /// Get verification status for a certificate
    pub fn get_status(&self, cert_id: &Uuid) -> &TrustChainStatus {
        self.trust_chain_verification_status
            .get(cert_id)
            .unwrap_or(&TrustChainStatus::Pending)
    }

    /// Set verification status for a certificate
    pub fn set_status(&mut self, cert_id: Uuid, status: TrustChainStatus) {
        self.trust_chain_verification_status.insert(cert_id, status);
    }

    /// Count certificates by status
    pub fn count_by_status(&self) -> (usize, usize, usize, usize) {
        let mut verified = 0;
        let mut failed = 0;
        let mut pending = 0;
        let mut roots = 0;

        for status in self.trust_chain_verification_status.values() {
            match status {
                TrustChainStatus::Verified { .. } => verified += 1,
                TrustChainStatus::Failed { .. }
                | TrustChainStatus::Expired { .. }
                | TrustChainStatus::IssuerNotFound { .. } => failed += 1,
                TrustChainStatus::Pending => pending += 1,
                TrustChainStatus::SelfSigned => roots += 1,
            }
        }

        (verified, failed, pending, roots)
    }

    /// Check if all certificates have been verified (no pending)
    pub fn all_verified(&self) -> bool {
        !self.trust_chain_verification_status.is_empty()
            && self
                .trust_chain_verification_status
                .values()
                .all(|s| !s.is_pending())
    }

    /// Get total certificate count
    pub fn total_certificates(&self) -> usize {
        self.trust_chain_verification_status.len()
    }

    /// Clear all verification status (for re-verification)
    pub fn clear_verification_status(&mut self) {
        self.trust_chain_verification_status.clear();
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update trust chain state based on message
///
/// This function handles trust chain domain messages. Note that actual
/// verification requires access to loaded_certificates and will be
/// delegated to the main update function.
pub fn update(state: &mut TrustChainState, message: TrustChainMessage) -> Task<Message> {
    use TrustChainMessage::*;

    match message {
        // === Section Toggle ===
        ToggleTrustChainSection => {
            state.trust_chain_section_collapsed = !state.trust_chain_section_collapsed;
            Task::none()
        }

        // === Certificate Selection ===
        SelectCertForTrustChain(cert_id) => {
            state.selected_trust_chain_cert = Some(cert_id);
            Task::none()
        }

        // === Verification (delegated to main for certificate access) ===
        VerifyTrustChain(_cert_id) => {
            // Verification logic requires loaded_certificates - delegated to main
            Task::none()
        }

        TrustChainVerified(result) => {
            match result {
                Ok((cert_id, status)) => {
                    state.set_status(cert_id, status);
                }
                Err(_) => {
                    // Error handling done in main app
                }
            }
            Task::none()
        }

        VerifyAllTrustChains => {
            // Batch verification requires loaded_certificates - delegated to main
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_chain_status_default() {
        let status = TrustChainStatus::default();
        assert!(status.is_pending());
    }

    #[test]
    fn test_trust_chain_status_checks() {
        let verified = TrustChainStatus::Verified {
            chain_length: 3,
            root_subject: "Root CA".to_string(),
        };
        assert!(verified.is_verified());
        assert!(!verified.is_failed());
        assert!(!verified.is_root());
        assert!(!verified.is_pending());

        let failed = TrustChainStatus::Failed {
            reason: "Invalid signature".to_string(),
        };
        assert!(!failed.is_verified());
        assert!(failed.is_failed());

        let root = TrustChainStatus::SelfSigned;
        assert!(root.is_root());
        assert!(!root.is_verified());
        assert!(!root.is_failed());

        let expired = TrustChainStatus::Expired {
            expired_at: Utc::now(),
        };
        assert!(expired.is_failed());

        let missing = TrustChainStatus::IssuerNotFound {
            expected_issuer: "Unknown CA".to_string(),
        };
        assert!(missing.is_failed());
    }

    #[test]
    fn test_trust_chain_state_default() {
        let state = TrustChainState::default();
        assert!(!state.trust_chain_section_collapsed);
        assert!(state.selected_trust_chain_cert.is_none());
        assert!(state.trust_chain_verification_status.is_empty());
    }

    #[test]
    fn test_trust_chain_state_new() {
        let state = TrustChainState::new();
        assert!(state.trust_chain_section_collapsed);
        assert!(state.selected_trust_chain_cert.is_none());
    }

    #[test]
    fn test_toggle_section() {
        let mut state = TrustChainState::new();
        assert!(state.trust_chain_section_collapsed);

        let _ = update(&mut state, TrustChainMessage::ToggleTrustChainSection);
        assert!(!state.trust_chain_section_collapsed);

        let _ = update(&mut state, TrustChainMessage::ToggleTrustChainSection);
        assert!(state.trust_chain_section_collapsed);
    }

    #[test]
    fn test_select_certificate() {
        let mut state = TrustChainState::new();
        let cert_id = Uuid::now_v7();

        let _ = update(
            &mut state,
            TrustChainMessage::SelectCertForTrustChain(cert_id),
        );
        assert_eq!(state.selected_trust_chain_cert, Some(cert_id));
    }

    #[test]
    fn test_verification_result() {
        let mut state = TrustChainState::new();
        let cert_id = Uuid::now_v7();

        // Initially pending
        assert!(state.get_status(&cert_id).is_pending());

        // Set verified
        let _ = update(
            &mut state,
            TrustChainMessage::TrustChainVerified(Ok((
                cert_id,
                TrustChainStatus::Verified {
                    chain_length: 2,
                    root_subject: "Root CA".to_string(),
                },
            ))),
        );

        assert!(state.get_status(&cert_id).is_verified());
    }

    #[test]
    fn test_count_by_status() {
        let mut state = TrustChainState::new();

        // Add various statuses
        state.set_status(
            Uuid::now_v7(),
            TrustChainStatus::Verified {
                chain_length: 2,
                root_subject: "Root".to_string(),
            },
        );
        state.set_status(
            Uuid::now_v7(),
            TrustChainStatus::Verified {
                chain_length: 3,
                root_subject: "Root".to_string(),
            },
        );
        state.set_status(
            Uuid::now_v7(),
            TrustChainStatus::Failed {
                reason: "Invalid".to_string(),
            },
        );
        state.set_status(Uuid::now_v7(), TrustChainStatus::SelfSigned);
        state.set_status(Uuid::now_v7(), TrustChainStatus::Pending);

        let (verified, failed, pending, roots) = state.count_by_status();
        assert_eq!(verified, 2);
        assert_eq!(failed, 1);
        assert_eq!(pending, 1);
        assert_eq!(roots, 1);
    }

    #[test]
    fn test_all_verified() {
        let mut state = TrustChainState::new();
        assert!(!state.all_verified()); // Empty is not "all verified"

        state.set_status(
            Uuid::now_v7(),
            TrustChainStatus::Verified {
                chain_length: 2,
                root_subject: "Root".to_string(),
            },
        );
        assert!(state.all_verified());

        state.set_status(Uuid::now_v7(), TrustChainStatus::Pending);
        assert!(!state.all_verified()); // Has pending
    }

    #[test]
    fn test_display_status() {
        assert!(TrustChainStatus::Pending
            .display_status()
            .contains("Pending"));
        assert!(TrustChainStatus::SelfSigned
            .display_status()
            .contains("Self-Signed"));
        assert!(TrustChainStatus::Verified {
            chain_length: 2,
            root_subject: "Root".to_string()
        }
        .display_status()
        .contains("Verified"));
    }
}
