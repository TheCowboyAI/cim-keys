// Copyright (c) 2025 - Cowboy AI, LLC.

//! Typography Verification
//!
//! This module provides the verification infrastructure that ensures
//! fonts and glyphs will render correctly (no tofu boxes).
//!
//! ## Verification Strategy
//!
//! 1. **Font Loading**: Verify fonts are successfully loaded
//! 2. **Glyph Coverage**: Check which Unicode ranges a font supports
//! 3. **Fallback Chains**: Build fallback sequences ending in ASCII text
//! 4. **Initialization-Time Check**: Fail fast if verification fails

use std::collections::HashSet;
use std::fmt;

/// Error during typography verification
#[derive(Debug, Clone)]
pub enum TypographyVerificationError {
    /// A required font failed to load
    FontLoadFailed {
        font_name: String,
        reason: String,
    },

    /// A glyph has no renderable representation
    NoRenderableGlyph {
        semantic_name: String,
        attempted_codepoints: Vec<char>,
    },

    /// An icon's fallback chain is exhausted
    FallbackChainExhausted {
        icon_name: String,
        chain: Vec<String>,
    },

    /// Font doesn't cover required Unicode block
    MissingUnicodeBlock {
        font_name: String,
        block_name: String,
    },
}

impl fmt::Display for TypographyVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FontLoadFailed { font_name, reason } => {
                write!(f, "Font '{}' failed to load: {}", font_name, reason)
            }
            Self::NoRenderableGlyph { semantic_name, attempted_codepoints } => {
                write!(
                    f,
                    "No renderable glyph for '{}', tried: {:?}",
                    semantic_name, attempted_codepoints
                )
            }
            Self::FallbackChainExhausted { icon_name, chain } => {
                write!(
                    f,
                    "Fallback chain exhausted for '{}': {:?}",
                    icon_name, chain
                )
            }
            Self::MissingUnicodeBlock { font_name, block_name } => {
                write!(
                    f,
                    "Font '{}' missing Unicode block '{}'",
                    font_name, block_name
                )
            }
        }
    }
}

impl std::error::Error for TypographyVerificationError {}

/// Glyph coverage information for a font
///
/// Tracks which Unicode codepoints a font can render.
#[derive(Debug, Clone, Default)]
pub struct GlyphCoverage {
    /// Unicode blocks this font supports
    blocks: Vec<UnicodeBlock>,
    /// Specific codepoints verified to render
    verified_codepoints: HashSet<char>,
}

impl GlyphCoverage {
    /// Create empty coverage
    pub fn new() -> Self {
        Self::default()
    }

    /// Create coverage for basic Latin (ASCII)
    pub fn basic_latin() -> Self {
        Self {
            blocks: vec![UnicodeBlock::BasicLatin],
            verified_codepoints: HashSet::new(),
        }
    }

    /// Create coverage for emoji
    pub fn emoji() -> Self {
        Self {
            blocks: vec![
                UnicodeBlock::Emoticons,
                UnicodeBlock::MiscSymbols,
                UnicodeBlock::Dingbats,
                UnicodeBlock::SupplementalSymbols,
            ],
            verified_codepoints: HashSet::new(),
        }
    }

    /// Create coverage for Material Icons (uses ligatures, covers Latin)
    pub fn material_icons() -> Self {
        Self {
            blocks: vec![UnicodeBlock::BasicLatin],
            verified_codepoints: HashSet::new(),
        }
    }

    /// Add a specific verified codepoint
    pub fn add_verified(&mut self, c: char) {
        self.verified_codepoints.insert(c);
    }

    /// Check if this coverage includes a character
    pub fn contains(&self, c: char) -> bool {
        self.verified_codepoints.contains(&c) ||
        self.blocks.iter().any(|b| b.contains(c))
    }

    /// Merge coverage from another source
    pub fn merge(&mut self, other: &GlyphCoverage) {
        self.blocks.extend(other.blocks.iter().cloned());
        self.verified_codepoints.extend(other.verified_codepoints.iter().cloned());
    }
}

/// Unicode block ranges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnicodeBlock {
    /// Basic Latin (U+0000 - U+007F) - ASCII
    BasicLatin,
    /// Latin-1 Supplement (U+0080 - U+00FF)
    Latin1Supplement,
    /// General Punctuation (U+2000 - U+206F)
    GeneralPunctuation,
    /// Miscellaneous Symbols (U+2600 - U+26FF)
    MiscSymbols,
    /// Dingbats (U+2700 - U+27BF)
    Dingbats,
    /// Emoticons (U+1F600 - U+1F64F)
    Emoticons,
    /// Miscellaneous Symbols and Pictographs (U+1F300 - U+1F5FF)
    MiscSymbolsPictographs,
    /// Supplemental Symbols and Pictographs (U+1F900 - U+1F9FF)
    SupplementalSymbols,
    /// Transport and Map Symbols (U+1F680 - U+1F6FF)
    TransportMapSymbols,
}

impl UnicodeBlock {
    /// Check if a character is in this block
    pub fn contains(&self, c: char) -> bool {
        let code = c as u32;
        match self {
            Self::BasicLatin => code <= 0x007F,
            Self::Latin1Supplement => (0x0080..=0x00FF).contains(&code),
            Self::GeneralPunctuation => (0x2000..=0x206F).contains(&code),
            Self::MiscSymbols => (0x2600..=0x26FF).contains(&code),
            Self::Dingbats => (0x2700..=0x27BF).contains(&code),
            Self::Emoticons => (0x1F600..=0x1F64F).contains(&code),
            Self::MiscSymbolsPictographs => (0x1F300..=0x1F5FF).contains(&code),
            Self::SupplementalSymbols => (0x1F900..=0x1F9FF).contains(&code),
            Self::TransportMapSymbols => (0x1F680..=0x1F6FF).contains(&code),
        }
    }

    /// Display name for this block
    pub fn name(&self) -> &'static str {
        match self {
            Self::BasicLatin => "Basic Latin",
            Self::Latin1Supplement => "Latin-1 Supplement",
            Self::GeneralPunctuation => "General Punctuation",
            Self::MiscSymbols => "Miscellaneous Symbols",
            Self::Dingbats => "Dingbats",
            Self::Emoticons => "Emoticons",
            Self::MiscSymbolsPictographs => "Miscellaneous Symbols and Pictographs",
            Self::SupplementalSymbols => "Supplemental Symbols and Pictographs",
            Self::TransportMapSymbols => "Transport and Map Symbols",
        }
    }
}

/// Marker type indicating verification has passed
///
/// This is a phantom type used to ensure VerifiedTheme can only
/// be constructed through the verification process.
#[derive(Debug, Clone, Copy)]
pub struct Verified;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_block_contains() {
        assert!(UnicodeBlock::BasicLatin.contains('A'));
        assert!(UnicodeBlock::BasicLatin.contains('z'));
        assert!(UnicodeBlock::BasicLatin.contains('!'));
        assert!(!UnicodeBlock::BasicLatin.contains('é'));

        assert!(UnicodeBlock::Emoticons.contains('😀'));
        assert!(!UnicodeBlock::Emoticons.contains('A'));
    }

    #[test]
    fn test_glyph_coverage() {
        let mut coverage = GlyphCoverage::basic_latin();
        assert!(coverage.contains('A'));
        assert!(!coverage.contains('😀'));

        coverage.add_verified('😀');
        assert!(coverage.contains('😀'));
    }
}
