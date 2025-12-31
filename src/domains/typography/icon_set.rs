// Copyright (c) 2025 - Cowboy AI, LLC.

//! Icon Set Types
//!
//! Defines verified icons with fallback chains that guarantee
//! at least one representation will render (no tofu).

use std::collections::HashMap;
use std::fmt;

/// Status icons for indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusIcon {
    Success,
    Warning,
    Error,
    Info,
    Locked,
    Unlocked,
    Active,
    Inactive,
    Pending,
    Complete,
}

/// Navigation icons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationIcon {
    Menu,
    Back,
    Forward,
    Up,
    Down,
    Home,
    Close,
    Expand,
    Collapse,
}

/// Action icons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionIcon {
    Save,
    Delete,
    Edit,
    Add,
    Remove,
    Refresh,
    Search,
    Filter,
    Copy,
    Paste,
}

/// Entity icons for domain objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityIcon {
    Person,
    Organization,
    Location,
    Key,
    Certificate,
    YubiKey,
    Account,
    Operator,
    Policy,
    Role,
}

/// Semantic icon categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticIcon {
    Status(StatusIcon),
    Navigation(NavigationIcon),
    Action(ActionIcon),
    Entity(EntityIcon),
}

impl SemanticIcon {
    /// Get the semantic name of this icon
    pub fn name(&self) -> &'static str {
        match self {
            Self::Status(s) => match s {
                StatusIcon::Success => "success",
                StatusIcon::Warning => "warning",
                StatusIcon::Error => "error",
                StatusIcon::Info => "info",
                StatusIcon::Locked => "locked",
                StatusIcon::Unlocked => "unlocked",
                StatusIcon::Active => "active",
                StatusIcon::Inactive => "inactive",
                StatusIcon::Pending => "pending",
                StatusIcon::Complete => "complete",
            },
            Self::Navigation(n) => match n {
                NavigationIcon::Menu => "menu",
                NavigationIcon::Back => "back",
                NavigationIcon::Forward => "forward",
                NavigationIcon::Up => "up",
                NavigationIcon::Down => "down",
                NavigationIcon::Home => "home",
                NavigationIcon::Close => "close",
                NavigationIcon::Expand => "expand",
                NavigationIcon::Collapse => "collapse",
            },
            Self::Action(a) => match a {
                ActionIcon::Save => "save",
                ActionIcon::Delete => "delete",
                ActionIcon::Edit => "edit",
                ActionIcon::Add => "add",
                ActionIcon::Remove => "remove",
                ActionIcon::Refresh => "refresh",
                ActionIcon::Search => "search",
                ActionIcon::Filter => "filter",
                ActionIcon::Copy => "copy",
                ActionIcon::Paste => "paste",
            },
            Self::Entity(e) => match e {
                EntityIcon::Person => "person",
                EntityIcon::Organization => "organization",
                EntityIcon::Location => "location",
                EntityIcon::Key => "key",
                EntityIcon::Certificate => "certificate",
                EntityIcon::YubiKey => "yubikey",
                EntityIcon::Account => "account",
                EntityIcon::Operator => "operator",
                EntityIcon::Policy => "policy",
                EntityIcon::Role => "role",
            },
        }
    }
}

/// How an icon can be represented
#[derive(Debug, Clone, PartialEq)]
pub enum IconRepresentation {
    /// Unicode emoji character
    Emoji(char),
    /// Material Icons ligature name
    MaterialIcon(String),
    /// Unicode symbol (non-emoji)
    UnicodeSymbol(char),
    /// Plain text fallback (always works)
    TextFallback(String),
}

impl IconRepresentation {
    /// Get display string for this representation
    pub fn display(&self) -> String {
        match self {
            Self::Emoji(c) => c.to_string(),
            Self::MaterialIcon(name) => name.clone(),
            Self::UnicodeSymbol(c) => c.to_string(),
            Self::TextFallback(text) => text.clone(),
        }
    }

    /// Check if this is the text fallback
    pub fn is_text_fallback(&self) -> bool {
        matches!(self, Self::TextFallback(_))
    }
}

impl fmt::Display for IconRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// A chain of icon representations to try in order
#[derive(Debug, Clone)]
pub struct IconChain {
    /// Semantic name for this icon
    name: String,
    /// Representations to try, in priority order
    chain: Vec<IconRepresentation>,
}

impl IconChain {
    /// Create a new icon chain with a semantic name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            chain: Vec::new(),
        }
    }

    /// Add an emoji representation (first priority)
    pub fn try_emoji(mut self, c: char) -> Self {
        self.chain.push(IconRepresentation::Emoji(c));
        self
    }

    /// Add a Material Icon representation
    pub fn try_material(mut self, name: impl Into<String>) -> Self {
        self.chain.push(IconRepresentation::MaterialIcon(name.into()));
        self
    }

    /// Add a Unicode symbol representation
    pub fn try_symbol(mut self, c: char) -> Self {
        self.chain.push(IconRepresentation::UnicodeSymbol(c));
        self
    }

    /// Add the text fallback (always last, always works)
    pub fn fallback_text(mut self, text: impl Into<String>) -> Self {
        self.chain.push(IconRepresentation::TextFallback(text.into()));
        self
    }

    /// Get the semantic name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the chain of representations
    pub fn chain(&self) -> &[IconRepresentation] {
        &self.chain
    }

    /// Check if this chain has a text fallback
    pub fn has_fallback(&self) -> bool {
        self.chain.iter().any(|r| r.is_text_fallback())
    }
}

/// A verified icon that has been checked for renderability
#[derive(Debug, Clone)]
pub struct VerifiedIcon {
    /// The original chain definition
    chain: IconChain,
    /// The verified representation that will actually render
    verified_repr: IconRepresentation,
}

impl VerifiedIcon {
    /// Create a verified icon (called after verification passes)
    pub fn new(chain: IconChain, verified_repr: IconRepresentation) -> Self {
        Self {
            chain,
            verified_repr,
        }
    }

    /// Get the semantic name
    pub fn name(&self) -> &str {
        self.chain.name()
    }

    /// Get the verified representation
    pub fn representation(&self) -> &IconRepresentation {
        &self.verified_repr
    }

    /// Get the display string
    pub fn display(&self) -> String {
        self.verified_repr.display()
    }

    /// Check if we fell back to text
    pub fn is_text_fallback(&self) -> bool {
        self.verified_repr.is_text_fallback()
    }

    /// Get the full chain (for debugging/display)
    pub fn chain(&self) -> &IconChain {
        &self.chain
    }
}

impl fmt::Display for VerifiedIcon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// A complete set of verified icons
#[derive(Debug, Clone)]
pub struct VerifiedIconSet {
    /// Icons indexed by semantic name
    icons: HashMap<String, VerifiedIcon>,
}

impl VerifiedIconSet {
    /// Create a new empty icon set
    pub fn new() -> Self {
        Self {
            icons: HashMap::new(),
        }
    }

    /// Add a verified icon
    pub fn add(&mut self, icon: VerifiedIcon) {
        self.icons.insert(icon.name().to_string(), icon);
    }

    /// Get an icon by semantic name
    pub fn get(&self, name: &str) -> Option<&VerifiedIcon> {
        self.icons.get(name)
    }

    /// Get a status icon
    pub fn status(&self, icon: StatusIcon) -> Option<&VerifiedIcon> {
        let name = SemanticIcon::Status(icon).name();
        self.get(name)
    }

    /// Get a navigation icon
    pub fn navigation(&self, icon: NavigationIcon) -> Option<&VerifiedIcon> {
        let name = SemanticIcon::Navigation(icon).name();
        self.get(name)
    }

    /// Get an action icon
    pub fn action(&self, icon: ActionIcon) -> Option<&VerifiedIcon> {
        let name = SemanticIcon::Action(icon).name();
        self.get(name)
    }

    /// Get an entity icon
    pub fn entity(&self, icon: EntityIcon) -> Option<&VerifiedIcon> {
        let name = SemanticIcon::Entity(icon).name();
        self.get(name)
    }

    /// Count how many icons fell back to text
    pub fn text_fallback_count(&self) -> usize {
        self.icons.values().filter(|i| i.is_text_fallback()).count()
    }

    /// Check if all icons render without text fallback
    pub fn all_graphical(&self) -> bool {
        self.text_fallback_count() == 0
    }

    /// Get list of icons using text fallback
    pub fn text_fallback_icons(&self) -> Vec<&str> {
        self.icons
            .values()
            .filter(|i| i.is_text_fallback())
            .map(|i| i.name())
            .collect()
    }

    /// Get total icon count
    pub fn len(&self) -> usize {
        self.icons.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }
}

impl Default for VerifiedIconSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating the default CIM icon set
pub struct CimIconSetBuilder;

impl CimIconSetBuilder {
    /// Build the default icon chains for CIM
    ///
    /// These are the icon chains before verification - verification
    /// will select which representation actually renders.
    pub fn default_chains() -> Vec<IconChain> {
        vec![
            // Status icons
            IconChain::new("success")
                .try_emoji('✓')
                .try_symbol('✓')
                .fallback_text("[OK]"),
            IconChain::new("warning")
                .try_emoji('⚠')
                .try_symbol('!')
                .fallback_text("[WARN]"),
            IconChain::new("error")
                .try_emoji('✗')
                .try_symbol('×')
                .fallback_text("[ERR]"),
            IconChain::new("info")
                .try_emoji('ℹ')
                .try_symbol('i')
                .fallback_text("[INFO]"),
            IconChain::new("locked")
                .try_emoji('🔒')
                .try_material("lock")
                .try_symbol('⚿')
                .fallback_text("[LOCK]"),
            IconChain::new("unlocked")
                .try_emoji('🔓')
                .try_material("lock_open")
                .fallback_text("[OPEN]"),
            IconChain::new("active")
                .try_emoji('●')
                .try_symbol('●')
                .fallback_text("[ON]"),
            IconChain::new("inactive")
                .try_emoji('○')
                .try_symbol('○')
                .fallback_text("[OFF]"),
            IconChain::new("pending")
                .try_emoji('⏳')
                .try_material("hourglass_empty")
                .fallback_text("[...]"),
            IconChain::new("complete")
                .try_emoji('✔')
                .try_symbol('✔')
                .fallback_text("[DONE]"),

            // Entity icons
            IconChain::new("person")
                .try_emoji('👤')
                .try_material("person")
                .fallback_text("[USR]"),
            IconChain::new("organization")
                .try_emoji('🏢')
                .try_material("business")
                .fallback_text("[ORG]"),
            IconChain::new("location")
                .try_emoji('📍')
                .try_material("location_on")
                .fallback_text("[LOC]"),
            IconChain::new("key")
                .try_emoji('🔑')
                .try_material("vpn_key")
                .try_symbol('⚿')
                .fallback_text("[KEY]"),
            IconChain::new("certificate")
                .try_emoji('📜')
                .try_material("verified")
                .fallback_text("[CRT]"),
            IconChain::new("yubikey")
                .try_emoji('🔐')
                .try_material("security")
                .fallback_text("[YBK]"),
            IconChain::new("account")
                .try_emoji('👥')
                .try_material("group")
                .fallback_text("[ACC]"),
            IconChain::new("operator")
                .try_emoji('⚙')
                .try_material("settings")
                .fallback_text("[OPR]"),
            IconChain::new("policy")
                .try_emoji('📋')
                .try_material("policy")
                .fallback_text("[POL]"),
            IconChain::new("role")
                .try_emoji('🎭')
                .try_material("badge")
                .fallback_text("[ROL]"),

            // Navigation icons
            IconChain::new("menu")
                .try_emoji('☰')
                .try_material("menu")
                .try_symbol('≡')
                .fallback_text("[=]"),
            IconChain::new("back")
                .try_emoji('←')
                .try_material("arrow_back")
                .try_symbol('←')
                .fallback_text("[<]"),
            IconChain::new("forward")
                .try_emoji('→')
                .try_material("arrow_forward")
                .try_symbol('→')
                .fallback_text("[>]"),
            IconChain::new("close")
                .try_emoji('✕')
                .try_material("close")
                .try_symbol('×')
                .fallback_text("[X]"),

            // Action icons
            IconChain::new("save")
                .try_emoji('💾')
                .try_material("save")
                .fallback_text("[SAV]"),
            IconChain::new("delete")
                .try_emoji('🗑')
                .try_material("delete")
                .fallback_text("[DEL]"),
            IconChain::new("edit")
                .try_emoji('✏')
                .try_material("edit")
                .fallback_text("[EDT]"),
            IconChain::new("add")
                .try_emoji('➕')
                .try_material("add")
                .try_symbol('+')
                .fallback_text("[+]"),
            IconChain::new("refresh")
                .try_emoji('🔄')
                .try_material("refresh")
                .fallback_text("[REF]"),
            IconChain::new("search")
                .try_emoji('🔍')
                .try_material("search")
                .fallback_text("[?]"),

            // Additional icons for GUI compatibility
            IconChain::new("visibility")
                .try_emoji('👁')
                .try_material("visibility")
                .fallback_text("[VIS]"),
            IconChain::new("visibility_off")
                .try_emoji('🙈')
                .try_material("visibility_off")
                .fallback_text("[HID]"),
            IconChain::new("cloud")
                .try_emoji('☁')
                .try_material("cloud")
                .fallback_text("[CLD]"),
            IconChain::new("folder")
                .try_emoji('📁')
                .try_material("folder")
                .fallback_text("[DIR]"),
            IconChain::new("folder_open")
                .try_emoji('📂')
                .try_material("folder_open")
                .fallback_text("[DIR]"),
            IconChain::new("usb")
                .try_emoji('🔌')
                .try_material("usb")
                .fallback_text("[USB]"),
            IconChain::new("download")
                .try_emoji('📥')
                .try_material("download")
                .fallback_text("[DL]"),
            IconChain::new("help")
                .try_emoji('❓')
                .try_material("help")
                .fallback_text("[?]"),
            IconChain::new("group")
                .try_emoji('👥')
                .try_material("group")
                .fallback_text("[GRP]"),
            IconChain::new("check")
                .try_emoji('✓')
                .try_symbol('✓')
                .fallback_text("[OK]"),
            IconChain::new("check_circle")
                .try_emoji('✅')
                .try_material("check_circle")
                .fallback_text("[OK]"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_chain_builder() {
        let chain = IconChain::new("lock")
            .try_emoji('🔒')
            .try_material("lock")
            .try_symbol('⚿')
            .fallback_text("[LOCK]");

        assert_eq!(chain.name(), "lock");
        assert_eq!(chain.chain().len(), 4);
        assert!(chain.has_fallback());
    }

    #[test]
    fn test_verified_icon() {
        let chain = IconChain::new("test")
            .try_emoji('🔒')
            .fallback_text("[LOCK]");
        let icon = VerifiedIcon::new(chain, IconRepresentation::Emoji('🔒'));

        assert_eq!(icon.name(), "test");
        assert_eq!(icon.display(), "🔒");
        assert!(!icon.is_text_fallback());
    }

    #[test]
    fn test_verified_icon_text_fallback() {
        let chain = IconChain::new("test")
            .try_emoji('🔒')
            .fallback_text("[LOCK]");
        let icon = VerifiedIcon::new(chain, IconRepresentation::TextFallback("[LOCK]".to_string()));

        assert!(icon.is_text_fallback());
        assert_eq!(icon.display(), "[LOCK]");
    }

    #[test]
    fn test_icon_set() {
        let mut set = VerifiedIconSet::new();

        let chain = IconChain::new("success").fallback_text("[OK]");
        set.add(VerifiedIcon::new(chain, IconRepresentation::TextFallback("[OK]".to_string())));

        assert_eq!(set.len(), 1);
        assert!(set.get("success").is_some());
        assert!(set.status(StatusIcon::Success).is_some());
    }

    #[test]
    fn test_default_chains() {
        let chains = CimIconSetBuilder::default_chains();

        // Should have all the essential icons
        assert!(chains.iter().any(|c| c.name() == "lock" || c.name() == "locked"));
        assert!(chains.iter().any(|c| c.name() == "person"));
        assert!(chains.iter().any(|c| c.name() == "organization"));
        assert!(chains.iter().any(|c| c.name() == "key"));

        // All should have fallbacks
        assert!(chains.iter().all(|c| c.has_fallback()));
    }
}
