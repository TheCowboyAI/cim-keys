// Copyright (c) 2025 - Cowboy AI, LLC.

//! TrustChain Message Definitions
//!
//! This module defines the message types for the TrustChain bounded context.
//! Handlers are in gui.rs - this module provides message organization and
//! the TrustChainStatus value type.
//!
//! ## Sub-domains
//!
//! 1. **Section Toggle**: UI visibility
//! 2. **Certificate Selection**: Select certificate for chain view
//! 3. **Verification**: Verify single or all chains

use chrono::{DateTime, Utc};
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

    #[test]
    fn test_trust_chain_message_variants() {
        let _ = TrustChainMessage::ToggleTrustChainSection;
        let _ = TrustChainMessage::SelectCertForTrustChain(Uuid::nil());
        let _ = TrustChainMessage::VerifyTrustChain(Uuid::nil());
        let _ = TrustChainMessage::TrustChainVerified(Ok((Uuid::nil(), TrustChainStatus::Pending)));
        let _ = TrustChainMessage::VerifyAllTrustChains;
    }
}
