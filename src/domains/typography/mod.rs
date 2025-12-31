// Copyright (c) 2025 - Cowboy AI, LLC.

//! Typography Bounded Context
//!
//! This module treats typography and visual rendering as a domain concern,
//! not an implementation detail. The core insight is that "tofu" (missing
//! glyph boxes) represents a **domain invariant violation** - a label that
//! cannot communicate its intent has failed its fundamental purpose.
//!
//! ## Why Typography is a Bounded Context
//!
//! 1. **Own Ubiquitous Language**: Glyph, Font Family, Icon Set, Rendering Capability
//! 2. **Invariants**: Every requested glyph MUST render (no tofu)
//! 3. **Aggregates**: Theme is a consistency boundary over fonts, icons, colors
//! 4. **Value Objects**: TextStyle, IconSpec, VerifiedGlyph
//!
//! ## Aggregate Root: VerifiedTheme
//!
//! The `VerifiedTheme` aggregate ensures that all referenced glyphs will render.
//! This is enforced at construction time (initialization), not at render time.
//!
//! ## Categorical Structure
//!
//! ```text
//! TypographyEntity = Font + Icon + Label
//!
//! Injections:
//!   inject_font: VerifiedFont → TypographyEntity
//!   inject_icon: VerifiedIcon → TypographyEntity
//!   inject_label: LabelledElement → TypographyEntity
//!
//! Universal Property (fold):
//!   For any X with morphisms from each component:
//!   ∃! [f_font, f_icon, f_label]: TypographyEntity → X
//! ```

mod injection;
mod entity;
mod fold;
mod verified_theme;
mod font_set;
mod icon_set;
mod verification;
pub mod labels;

// Re-export primary types
pub use injection::{TypographyInjection, TypographyCategory};
pub use entity::{TypographyEntity, TypographyData};
pub use fold::{FoldTypographyEntity, IdentityFold, ToStringFold};
pub use verified_theme::{VerifiedTheme, ColorPalette, ThemeMetrics, BackgroundColors, VerificationSummary};
pub use font_set::{VerifiedFontFamily, VerifiedFontSet, FontFamily};
pub use icon_set::{
    VerifiedIcon, VerifiedIconSet, IconRepresentation, IconChain, SemanticIcon,
    StatusIcon, NavigationIcon, ActionIcon, EntityIcon, CimIconSetBuilder,
};
pub use verification::{TypographyVerificationError, GlyphCoverage, UnicodeBlock, Verified};
pub use labels::{LabelSpec, LabelCategory, LabelledElement};
