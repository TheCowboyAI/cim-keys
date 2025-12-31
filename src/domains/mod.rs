// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Bounded Contexts
//!
//! This module organizes entity coproducts by DDD bounded context.
//! Each context owns its types and provides:
//!
//! 1. **Injection enum** - Tags for each entity type in the context
//! 2. **Data enum** - Inner data carrier for the coproduct
//! 3. **Entity struct** - The coproduct with injection functions
//! 4. **Fold trait** - Universal property for the coproduct
//!
//! ## Bounded Contexts
//!
//! - **Organization**: People, organizations, units, locations, roles, policies
//! - **PKI**: Certificates and cryptographic keys
//! - **NATS**: Operators, accounts, users, service accounts
//! - **YubiKey**: Hardware devices and PIV slots
//! - **Visualization**: Manifests, policy display types
//!
//! ## Categorical Structure
//!
//! Each context coproduct follows the same pattern:
//!
//! ```text
//! ContextEntity = A + B + C + ...
//!
//! Injections:
//!   inject_a: A → ContextEntity
//!   inject_b: B → ContextEntity
//!   ...
//!
//! Universal Property (fold):
//!   For any X with f_a: A → X, f_b: B → X, ...
//!   ∃! [f_a, f_b, ...]: ContextEntity → X
//! ```
//!
//! The `LiftableDomain` trait (see `composition` module) provides functors
//! from each context to the unified graph representation.

// Entity coproducts by bounded context
pub mod organization;
pub mod pki;
pub mod nats;
pub mod yubikey;
pub mod visualization;
pub mod typography;

// Aggregate state coproduct (separate from entities)
pub mod aggregates;

// Re-export entity types for convenience
pub use organization::{
    OrganizationEntity, OrganizationData, OrganizationInjection, FoldOrganizationEntity,
};
pub use pki::{
    PkiEntity, PkiData, PkiInjection, FoldPkiEntity,
};
pub use nats::{
    NatsEntity, NatsData, NatsInjection, FoldNatsEntity,
};
pub use yubikey::{
    YubiKeyEntity, YubiKeyData, YubiKeyInjection, FoldYubiKeyEntity,
};
pub use visualization::{
    VisualizationEntity, VisualizationData, VisualizationInjection, FoldVisualizationEntity,
};
pub use typography::{
    TypographyEntity, TypographyData, TypographyInjection, FoldTypographyEntity,
    VerifiedTheme, VerifiedFontFamily, VerifiedFontSet, FontFamily,
    VerifiedIcon, VerifiedIconSet, IconRepresentation, IconChain,
    LabelSpec, LabelCategory, LabelledElement,
    ColorPalette, ThemeMetrics,
};

// Re-export aggregate types
pub use aggregates::{
    AggregateState, AggregateStateData, AggregateInjection, FoldAggregateState,
    OrganizationAggregateState, PkiChainAggregateState,
    NatsSecurityAggregateState, YubiKeyProvisioningAggregateState,
};

/// Context tag for identifying which bounded context an entity belongs to.
///
/// Used for compositional routing at the context level rather than
/// entity-level pattern matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextTag {
    Organization,
    Pki,
    Nats,
    YubiKey,
    Visualization,
    Typography,
}

impl ContextTag {
    /// Display name for this context
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Organization => "Organization",
            Self::Pki => "PKI",
            Self::Nats => "NATS",
            Self::YubiKey => "YubiKey",
            Self::Visualization => "Visualization",
            Self::Typography => "Typography",
        }
    }

    /// All context tags
    pub fn all() -> Vec<Self> {
        vec![
            Self::Organization,
            Self::Pki,
            Self::Nats,
            Self::YubiKey,
            Self::Visualization,
            Self::Typography,
        ]
    }
}

impl std::fmt::Display for ContextTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
