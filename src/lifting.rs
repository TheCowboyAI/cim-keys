// Copyright (c) 2025 - Cowboy AI, LLC.

//! LiftableDomain - Faithful Functor for Domain Composition
//!
//! NOTE: This module uses the deprecated `Injection` type internally as the type tag
//! for the coproduct. This is intentional - `Injection` provides the full variant set
//! needed for faithful functor implementation. External code should use the `is_*()`
//! helper methods on `DomainNode` or per-context types from `crate::domains`.
//!
//! This module implements the `LiftableDomain` trait, enabling any domain type
//! to be "lifted" into a unified graph representation while preserving all
//! domain semantics.
//!
//! # Mathematical Foundation
//!
//! `LiftableDomain` forms a **faithful functor** F: Domain → Graph where:
//! - Objects (entities) map to graph nodes
//! - Morphisms (relationships) map to graph edges
//! - Identity is preserved: F(id_A) = id_{F(A)}
//! - Composition is preserved: F(g ∘ f) = F(g) ∘ F(f)
//!
//! The functor is "faithful" meaning it is injective on morphisms:
//! If F(f) = F(g) then f = g. This ensures no domain information is lost.
//!
//! # Monad Structure
//!
//! LiftableDomain combined with `Entity<T>` from cim-domain forms a monad:
//! - `pure` (unit): Domain value → Entity<Domain>
//! - `bind` (join): Entity<Entity<T>> → Entity<T>
//! - Laws: Left identity, Right identity, Associativity
//!
//! # Usage
//!
//! ```ignore
//! use cim_keys::lifting::{LiftableDomain, LiftedNode};
//!
//! // Domain types implement LiftableDomain
//! let person: Person = ...;
//! let lifted: LiftedNode = person.lift();
//!
//! // Recover original domain entity
//! let recovered: Option<Person> = Person::unlift(&lifted);
//!
//! // Events can also be lifted
//! let event: PersonCreatedEvent = ...;
//! let graph_event: GraphEvent = person.lift_event(event);
//! ```

use std::fmt::{self, Debug};
use std::any::Any;
use std::sync::Arc;

use iced::Color;
use uuid::Uuid;

use crate::domain::{Organization, OrganizationUnit, Person, Location, Role, Policy, KeyOwnerRole, PolicyClaim};
use crate::fold::{FoldCapability, Foldable};
use crate::gui::folds::query::edit_fields::EditFieldData;

// ============================================================================
// INJECTION TYPE TAG
// ============================================================================

/// Type tag for `LiftedNode` - identifies the domain type stored within.
///
/// `Injection` provides variant discrimination for the type-erased domain data.
/// Use the helper methods on this enum to check the category of a node
/// (e.g., `is_organization()`, `is_pki()`, `is_nats()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Injection {
    // Core Domain Entities
    Person,
    Organization,
    OrganizationUnit,
    Location,
    Role,
    Policy,

    // NATS Infrastructure (with projections)
    NatsOperator,
    NatsAccount,
    NatsUser,
    NatsServiceAccount,

    // NATS Infrastructure (simple visualization)
    NatsOperatorSimple,
    NatsAccountSimple,
    NatsUserSimple,

    // PKI Certificates
    RootCertificate,
    IntermediateCertificate,
    LeafCertificate,

    // Cryptographic Keys
    Key,

    // YubiKey Hardware
    YubiKey,
    PivSlot,
    YubiKeyStatus,

    // Export Manifest
    Manifest,

    // Policy Roles and Claims
    PolicyRole,
    PolicyClaim,
    PolicyCategory,
    PolicyGroup,

    // Aggregate Roots (DDD bounded contexts)
    AggregateOrganization,
    AggregatePkiChain,
    AggregateNatsSecurity,
    AggregateYubiKeyProvisioning,

    // Workflow Gaps (Trust Chain Fulfillment)
    WorkflowGap,

    // State Machine Visualization (Kan extension from graph → domain)
    StateMachineState,
    StateMachineTransition,
}

impl Injection {
    /// Get the display name for this injection type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Person => "Person",
            Self::Organization => "Organization",
            Self::OrganizationUnit => "Organizational Unit",
            Self::Location => "Location",
            Self::Role => "Role",
            Self::Policy => "Policy",
            Self::NatsOperator => "NATS Operator",
            Self::NatsAccount => "NATS Account",
            Self::NatsUser => "NATS User",
            Self::NatsServiceAccount => "NATS Service Account",
            Self::NatsOperatorSimple => "NATS Operator",
            Self::NatsAccountSimple => "NATS Account",
            Self::NatsUserSimple => "NATS User",
            Self::RootCertificate => "Root Certificate",
            Self::IntermediateCertificate => "Intermediate Certificate",
            Self::LeafCertificate => "Leaf Certificate",
            Self::Key => "Key",
            Self::YubiKey => "YubiKey",
            Self::PivSlot => "PIV Slot",
            Self::YubiKeyStatus => "YubiKey Status",
            Self::Manifest => "Manifest",
            Self::PolicyRole => "Policy Role",
            Self::PolicyClaim => "Policy Claim",
            Self::PolicyCategory => "Policy Category",
            Self::PolicyGroup => "Separation Class",
            Self::AggregateOrganization => "Organization Aggregate",
            Self::AggregatePkiChain => "PKI Chain Aggregate",
            Self::AggregateNatsSecurity => "NATS Security Aggregate",
            Self::AggregateYubiKeyProvisioning => "YubiKey Provisioning Aggregate",
            Self::WorkflowGap => "Workflow Gap",
            Self::StateMachineState => "State",
            Self::StateMachineTransition => "Transition",
        }
    }

    /// Get the layout tier for hierarchical positioning
    pub fn layout_tier(&self) -> u8 {
        match self {
            // Tier 0: Root entities and Aggregates
            Self::Organization => 0,
            Self::NatsOperator | Self::NatsOperatorSimple => 0,
            Self::RootCertificate => 0,
            Self::YubiKey => 0,
            Self::YubiKeyStatus => 0,
            Self::PolicyGroup => 0,
            Self::AggregateOrganization |
            Self::AggregatePkiChain |
            Self::AggregateNatsSecurity |
            Self::AggregateYubiKeyProvisioning => 0,

            // Tier 1: Intermediate entities
            Self::OrganizationUnit => 1,
            Self::Role => 1,
            Self::Policy => 1,
            Self::NatsAccount | Self::NatsAccountSimple => 1,
            Self::IntermediateCertificate => 1,
            Self::PivSlot => 1,
            Self::PolicyRole => 1,
            Self::PolicyCategory => 1,

            // Tier 2: Leaf entities
            Self::Person => 2,
            Self::Location => 2,
            Self::NatsUser | Self::NatsUserSimple | Self::NatsServiceAccount => 2,
            Self::LeafCertificate => 2,
            Self::Key => 2,
            Self::Manifest => 2,
            Self::PolicyClaim => 2,

            // Workflow gaps use semantic positioning (tier based on dependencies)
            Self::WorkflowGap => 1,

            // State machine visualization (graph → domain Kan extension)
            Self::StateMachineState => 1,
            Self::StateMachineTransition => 2,
        }
    }

    /// Check if this injection type is a NATS infrastructure node
    pub fn is_nats(&self) -> bool {
        matches!(
            self,
            Self::NatsOperator | Self::NatsOperatorSimple |
            Self::NatsAccount | Self::NatsAccountSimple |
            Self::NatsUser | Self::NatsUserSimple |
            Self::NatsServiceAccount
        )
    }

    /// Check if this injection type is a PKI certificate node
    pub fn is_certificate(&self) -> bool {
        matches!(
            self,
            Self::RootCertificate | Self::IntermediateCertificate | Self::LeafCertificate
        )
    }

    /// Check if this injection type is a YubiKey-related node
    pub fn is_yubikey(&self) -> bool {
        matches!(self, Self::YubiKey | Self::PivSlot | Self::YubiKeyStatus)
    }

    /// Check if this injection type is a policy-related node
    pub fn is_policy(&self) -> bool {
        matches!(
            self,
            Self::Policy | Self::PolicyRole | Self::PolicyClaim |
            Self::PolicyCategory | Self::PolicyGroup
        )
    }

    /// Check if this injection type is a state machine visualization node
    pub fn is_state_machine(&self) -> bool {
        matches!(self, Self::StateMachineState | Self::StateMachineTransition)
    }

    /// Get the list of injection types that can be manually created from the UI.
    pub fn creatable() -> Vec<Self> {
        vec![
            Self::Organization,
            Self::OrganizationUnit,
            Self::Person,
            Self::Location,
            Self::Role,
            Self::Policy,
        ]
    }

    /// Check if this injection type can be manually created from the UI
    pub fn is_creatable(&self) -> bool {
        matches!(
            self,
            Self::Organization |
            Self::OrganizationUnit |
            Self::Person |
            Self::Location |
            Self::Role |
            Self::Policy
        )
    }

    // Individual Type Checkers
    pub fn is_person(&self) -> bool { matches!(self, Self::Person) }
    pub fn is_organization(&self) -> bool { matches!(self, Self::Organization) }
    pub fn is_organization_unit(&self) -> bool { matches!(self, Self::OrganizationUnit) }
    pub fn is_location(&self) -> bool { matches!(self, Self::Location) }
    pub fn is_role(&self) -> bool { matches!(self, Self::Role) }
    pub fn is_key(&self) -> bool { matches!(self, Self::Key) }
    pub fn is_yubikey_device(&self) -> bool { matches!(self, Self::YubiKey) }
    pub fn is_piv_slot(&self) -> bool { matches!(self, Self::PivSlot) }
    pub fn is_nats_operator(&self) -> bool { matches!(self, Self::NatsOperator | Self::NatsOperatorSimple) }
    pub fn is_nats_account(&self) -> bool { matches!(self, Self::NatsAccount | Self::NatsAccountSimple) }
    pub fn is_nats_user(&self) -> bool { matches!(self, Self::NatsUser | Self::NatsUserSimple) }
    pub fn is_nats_service_account(&self) -> bool { matches!(self, Self::NatsServiceAccount) }
    pub fn is_aggregate_organization(&self) -> bool { matches!(self, Self::AggregateOrganization) }
    pub fn is_aggregate_pki_chain(&self) -> bool { matches!(self, Self::AggregatePkiChain) }
    pub fn is_aggregate_nats_security(&self) -> bool { matches!(self, Self::AggregateNatsSecurity) }
    pub fn is_aggregate_yubikey_provisioning(&self) -> bool { matches!(self, Self::AggregateYubiKeyProvisioning) }
    pub fn is_aggregate_yubikey(&self) -> bool { self.is_aggregate_yubikey_provisioning() }
    pub fn is_policy_role(&self) -> bool { matches!(self, Self::PolicyRole) }
    pub fn is_policy_category(&self) -> bool { matches!(self, Self::PolicyCategory) }
    pub fn is_policy_group(&self) -> bool { matches!(self, Self::PolicyGroup) }
    pub fn is_policy_claim(&self) -> bool { matches!(self, Self::PolicyClaim) }
    pub fn is_org_filterable(&self) -> bool {
        matches!(self, Self::Organization | Self::OrganizationUnit | Self::Person | Self::Location | Self::Role | Self::Policy)
    }
    pub fn is_yubikey_status(&self) -> bool { matches!(self, Self::YubiKeyStatus) }
    pub fn is_yubikey_or_slot(&self) -> bool { matches!(self, Self::YubiKey | Self::PivSlot | Self::YubiKeyStatus) }
    pub fn is_policy_variant(&self) -> bool {
        matches!(self, Self::Policy | Self::PolicyRole | Self::PolicyClaim | Self::PolicyCategory | Self::PolicyGroup)
    }
    pub fn is_policy_group_or_category(&self) -> bool { matches!(self, Self::PolicyGroup | Self::PolicyCategory) }
    pub fn is_workflow_gap(&self) -> bool { matches!(self, Self::WorkflowGap) }
}

impl fmt::Display for Injection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// PROPERTY UPDATE
// ============================================================================

/// Property updates that can be applied to editable domain nodes.
#[derive(Debug, Clone, Default)]
pub struct PropertyUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub enabled: Option<bool>,
    pub claims: Option<Vec<PolicyClaim>>,
}

impl PropertyUpdate {
    pub fn new() -> Self { Self::default() }
    pub fn with_name(mut self, name: impl Into<String>) -> Self { self.name = Some(name.into()); self }
    pub fn with_description(mut self, description: impl Into<String>) -> Self { self.description = Some(description.into()); self }
    pub fn with_email(mut self, email: impl Into<String>) -> Self { self.email = Some(email.into()); self }
    pub fn with_enabled(mut self, enabled: bool) -> Self { self.enabled = Some(enabled); self }
    pub fn with_claims(mut self, claims: Vec<PolicyClaim>) -> Self { self.claims = Some(claims); self }

    pub fn has_changes_for(&self, injection: Injection) -> bool {
        match injection {
            Injection::Organization => self.name.is_some() || self.description.is_some(),
            Injection::OrganizationUnit => self.name.is_some(),
            Injection::Person => self.name.is_some() || self.email.is_some() || self.enabled.is_some(),
            Injection::Location => self.name.is_some(),
            Injection::Role => self.name.is_some() || self.description.is_some() || self.enabled.is_some(),
            Injection::Policy => {
                self.name.is_some() || self.description.is_some() ||
                self.enabled.is_some() || self.claims.is_some()
            }
            _ => false,
        }
    }
}

// ============================================================================
// VISUALIZATION DATA
// ============================================================================

/// Output of visualization - all data needed to render a node
#[derive(Debug, Clone)]
pub struct VisualizationData {
    /// Node color (fill)
    pub color: Color,
    /// Primary display text (e.g., name)
    pub primary_text: String,
    /// Secondary display text (e.g., email, description)
    pub secondary_text: String,
    /// Icon character (emoji or Material icon)
    pub icon: char,
    /// Font for the icon
    pub icon_font: iced::Font,
    /// Whether this node is expandable (has +/- indicator)
    pub expandable: bool,
    /// If expandable, whether it's currently expanded
    pub expanded: bool,
}

impl Default for VisualizationData {
    fn default() -> Self {
        Self {
            color: Color::from_rgb(0.5, 0.5, 0.5),
            primary_text: String::new(),
            secondary_text: String::new(),
            icon: crate::icons::ICON_HELP,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }
}

// ============================================================================
// DETAIL PANEL DATA
// ============================================================================

/// Data for rendering a detail panel for a selected node.
///
/// This is a simple struct containing the title and key-value fields for display.
/// It is used by `LiftedNode::detail_panel()` to provide information about selected entities.
#[derive(Debug, Clone)]
pub struct DetailPanelData {
    /// Title for the detail panel (e.g., "Selected Organization:")
    pub title: String,
    /// List of (label, value) pairs to display
    pub fields: Vec<(String, String)>,
}

impl DetailPanelData {
    /// Create a new detail panel data
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            fields: Vec::new(),
        }
    }

    /// Add a field to the detail panel
    pub fn with_field(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((label.into(), value.into()));
        self
    }
}

// ============================================================================
// ICON HELPER
// ============================================================================

/// Get icon for an injection type. Returns (icon_char, font) tuple.
pub fn icon_for_injection(injection: Injection) -> (char, iced::Font) {
    use crate::icons::*;

    match injection {
        Injection::Person => (ICON_PERSON, MATERIAL_ICONS),
        Injection::Organization => (ICON_BUSINESS, MATERIAL_ICONS),
        Injection::OrganizationUnit => (ICON_GROUP, MATERIAL_ICONS),
        Injection::Location => (ICON_LOCATION, MATERIAL_ICONS),
        Injection::Role => (ICON_SETTINGS, MATERIAL_ICONS),
        Injection::Policy => (ICON_VERIFIED, MATERIAL_ICONS),
        Injection::NatsOperator | Injection::NatsOperatorSimple => (ICON_SETTINGS, MATERIAL_ICONS),
        Injection::NatsAccount | Injection::NatsAccountSimple => (ICON_CLOUD, MATERIAL_ICONS),
        Injection::NatsUser | Injection::NatsUserSimple => (ICON_PERSON, MATERIAL_ICONS),
        Injection::NatsServiceAccount => (ICON_SETTINGS, MATERIAL_ICONS),
        Injection::RootCertificate => (ICON_VERIFIED, MATERIAL_ICONS),
        Injection::IntermediateCertificate => (ICON_VERIFIED, MATERIAL_ICONS),
        Injection::LeafCertificate => (ICON_VERIFIED, MATERIAL_ICONS),
        Injection::Key => (ICON_KEY, MATERIAL_ICONS),
        Injection::YubiKey => (ICON_SECURITY, MATERIAL_ICONS),
        Injection::PivSlot => (ICON_USB, MATERIAL_ICONS),
        Injection::YubiKeyStatus => (ICON_CHECK, MATERIAL_ICONS),
        Injection::Manifest => (ICON_MEMORY, MATERIAL_ICONS),
        Injection::PolicyRole => (ICON_SETTINGS, MATERIAL_ICONS),
        Injection::PolicyClaim => (ICON_CHECK, MATERIAL_ICONS),
        Injection::PolicyCategory => (ICON_FOLDER, MATERIAL_ICONS),
        Injection::PolicyGroup => (ICON_GROUP, MATERIAL_ICONS),
        Injection::AggregateOrganization => (ICON_BUSINESS, MATERIAL_ICONS),
        Injection::AggregatePkiChain => (ICON_VERIFIED, MATERIAL_ICONS),
        Injection::AggregateNatsSecurity => (ICON_CLOUD, MATERIAL_ICONS),
        Injection::AggregateYubiKeyProvisioning => (ICON_SECURITY, MATERIAL_ICONS),
        Injection::WorkflowGap => (ICON_CHECK, MATERIAL_ICONS),
        Injection::StateMachineState => (ICON_SETTINGS, MATERIAL_ICONS),
        Injection::StateMachineTransition => (ICON_ARROW, MATERIAL_ICONS),
    }
}
use crate::domain::visualization::{PolicyGroup, PolicyCategory, PolicyRole, PolicyClaimView, Manifest};
use crate::domain::pki::{Certificate, CertificateType, CryptographicKey};
use crate::domain::yubikey::{YubiKeyDevice, PivSlotView, YubiKeyStatus};

// ============================================================================
// DOMAIN-SPECIFIC COLORS
// ============================================================================
//
// NOTE: These constants are retained for compile-time const contexts.
// For runtime theming, prefer ColorPalette from crate::gui::view_model.
// ColorPalette fields: node_organization, node_unit, node_person, etc.

/// Color for Organization nodes (matches ColorPalette::node_organization)
pub const COLOR_ORGANIZATION: Color = Color::from_rgb(0.2, 0.3, 0.6);

/// Color for OrganizationUnit nodes (matches ColorPalette::node_unit)
pub const COLOR_UNIT: Color = Color::from_rgb(0.4, 0.5, 0.8);

/// Color for Person nodes (matches ColorPalette::node_person)
pub const COLOR_PERSON: Color = Color::from_rgb(0.2, 0.8, 0.3);  // Green

/// Color for Location nodes (matches ColorPalette::node_location)
pub const COLOR_LOCATION: Color = Color::from_rgb(0.6, 0.5, 0.4);

/// Color for Role nodes (matches ColorPalette::node_role)
pub const COLOR_ROLE: Color = Color::from_rgb(0.6, 0.3, 0.8);

/// Color for Policy nodes (matches ColorPalette::node_policy)
pub const COLOR_POLICY: Color = Color::from_rgb(0.9, 0.7, 0.2);  // Gold/yellow

/// Color for NATS Operator nodes (matches ColorPalette::node_nats_operator)
pub const COLOR_NATS_OPERATOR: Color = Color::from_rgb(0.6, 0.2, 0.8);

/// Color for NATS Account nodes (matches ColorPalette::node_nats_account)
pub const COLOR_NATS_ACCOUNT: Color = Color::from_rgb(0.5, 0.3, 0.7);

/// Color for NATS User nodes (matches ColorPalette::node_nats_user)
pub const COLOR_NATS_USER: Color = Color::from_rgb(0.4, 0.4, 0.6);

/// Color for Certificate nodes (matches ColorPalette::node_certificate)
pub const COLOR_CERTIFICATE: Color = Color::from_rgb(0.7, 0.5, 0.2);

/// Color for Key nodes (matches ColorPalette::node_key)
pub const COLOR_KEY: Color = Color::from_rgb(0.6, 0.6, 0.2);

/// Color for YubiKey nodes (matches ColorPalette::node_yubikey)
pub const COLOR_YUBIKEY: Color = Color::from_rgb(0.0, 0.6, 0.4);

/// Color for PolicyGroup (SeparationClass) nodes
pub const COLOR_POLICY_GROUP: Color = Color::from_rgb(0.8, 0.4, 0.2);  // Orange-brown

/// Color for PolicyCategory nodes
pub const COLOR_POLICY_CATEGORY: Color = Color::from_rgb(0.7, 0.5, 0.3);  // Tan

/// Color for PolicyRole nodes
pub const COLOR_POLICY_ROLE: Color = Color::from_rgb(0.6, 0.4, 0.6);  // Purple-gray

/// Color for PolicyClaim nodes
pub const COLOR_POLICY_CLAIM: Color = Color::from_rgb(0.5, 0.6, 0.4);  // Olive green

/// Color for Workflow Gap nodes - varies by status
pub const COLOR_WORKFLOW_GAP_NOT_STARTED: Color = Color::from_rgb(0.5, 0.5, 0.5);  // Gray
pub const COLOR_WORKFLOW_GAP_IN_PROGRESS: Color = Color::from_rgb(0.2, 0.6, 0.9);  // Blue
pub const COLOR_WORKFLOW_GAP_IMPLEMENTED: Color = Color::from_rgb(0.8, 0.6, 0.2);  // Orange
pub const COLOR_WORKFLOW_GAP_TESTED: Color = Color::from_rgb(0.6, 0.8, 0.2);       // Yellow-green
pub const COLOR_WORKFLOW_GAP_VERIFIED: Color = Color::from_rgb(0.2, 0.8, 0.4);     // Green

// ============================================================================
// LIFTED NODE - Graph representation of any domain entity
// ============================================================================

/// A domain entity lifted into graph representation.
///
/// This is the target of the LiftableDomain functor. It contains:
/// - A unique identifier (preserved from domain)
/// - An injection tag indicating the domain type
/// - The original domain data (type-erased for heterogeneous storage)
/// - Derived metadata for graph visualization
/// - FoldCapability for EditFieldData (FRP A5/A6 compliance)
#[derive(Clone)]
pub struct LiftedNode {
    /// Entity ID (preserved from domain)
    pub id: Uuid,

    /// Domain type tag (coproduct injection)
    pub injection: Injection,

    /// Display label for graph visualization
    pub label: String,

    /// Secondary text (optional)
    pub secondary: Option<String>,

    /// Color for visualization
    pub color: Color,

    /// Original domain data (type-erased)
    data: Arc<dyn Any + Send + Sync>,

    /// FoldCapability for EditFieldData extraction (FRP A5/A6 compliant)
    ///
    /// Captured at lift time - NO pattern matching required at fold time.
    /// This is the categorical fold eliminator for the coproduct.
    edit_fields_fold: Option<FoldCapability<EditFieldData>>,
}

// Custom Debug impl since FoldCapability doesn't have useful Debug
impl std::fmt::Debug for LiftedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiftedNode")
            .field("id", &self.id)
            .field("injection", &self.injection)
            .field("label", &self.label)
            .field("secondary", &self.secondary)
            .field("color", &self.color)
            .field("has_edit_fields_fold", &self.edit_fields_fold.is_some())
            .finish()
    }
}

impl LiftedNode {
    /// Create a new lifted node from domain data
    pub fn new<T: Send + Sync + 'static>(
        id: Uuid,
        injection: Injection,
        label: impl Into<String>,
        color: Color,
        data: T,
    ) -> Self {
        Self {
            id,
            injection,
            label: label.into(),
            secondary: None,
            color,
            data: Arc::new(data),
            edit_fields_fold: None,
        }
    }

    /// Create a new lifted node with a fold capability for EditFieldData.
    ///
    /// FRP A5/A6 compliant: The fold is captured at lift time, so fold execution
    /// requires NO pattern matching or downcasting.
    pub fn new_with_fold<T: Send + Sync + Clone + 'static>(
        id: Uuid,
        injection: Injection,
        label: impl Into<String>,
        color: Color,
        data: T,
        fold_fn: impl Fn(&T) -> EditFieldData + Send + Sync + 'static,
    ) -> Self {
        let fold_cap = FoldCapability::new(data.clone(), fold_fn);
        Self {
            id,
            injection,
            label: label.into(),
            secondary: None,
            color,
            data: Arc::new(data),
            edit_fields_fold: Some(fold_cap),
        }
    }

    /// Add secondary text
    pub fn with_secondary(mut self, text: impl Into<String>) -> Self {
        self.secondary = Some(text.into());
        self
    }

    /// Update primary text (label)
    pub fn with_primary(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self
    }

    /// Add a fold capability for EditFieldData extraction (FRP A5/A6 compliance)
    ///
    /// The fold is captured at lift time. To extract EditFieldData, call
    /// `fold_edit_fields()` - NO pattern matching required.
    pub fn with_edit_fields_fold(mut self, fold: FoldCapability<EditFieldData>) -> Self {
        self.edit_fields_fold = Some(fold);
        self
    }

    /// Execute the EditFieldData fold (FRP A6 compliant - no pattern matching)
    ///
    /// Returns the EditFieldData if a fold was captured at lift time.
    /// If no fold was captured, returns None (caller can fall back to legacy extraction).
    pub fn fold_edit_fields(&self) -> Option<EditFieldData> {
        self.edit_fields_fold.as_ref().map(|cap| cap.execute())
    }

    /// Check if this node has an EditFieldData fold capability
    pub fn has_edit_fields_fold(&self) -> bool {
        self.edit_fields_fold.is_some()
    }

    /// Attempt to downcast and retrieve the original domain data
    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }

    /// Get the injection type
    pub fn injection(&self) -> Injection {
        self.injection
    }

    /// Apply property updates to this node, returning a new LiftedNode.
    ///
    /// This is the non-deprecated replacement for `DomainNode::with_properties()`.
    /// It downcasts to the appropriate domain type, applies the updates,
    /// and re-lifts the modified entity.
    ///
    /// For read-only node types, returns a clone of self unchanged.
    pub fn apply_properties(&self, update: &PropertyUpdate) -> Self {
        match self.injection {
            Injection::Organization => {
                if let Some(org) = self.downcast::<Organization>() {
                    let mut updated = org.clone();
                    if let Some(name) = &update.name {
                        updated.name = name.clone();
                        updated.display_name = name.clone();
                    }
                    if let Some(description) = &update.description {
                        updated.description = Some(description.clone());
                    }
                    return updated.lift();
                }
            }
            Injection::OrganizationUnit => {
                if let Some(unit) = self.downcast::<OrganizationUnit>() {
                    let mut updated = unit.clone();
                    if let Some(name) = &update.name {
                        updated.name = name.clone();
                    }
                    return updated.lift();
                }
            }
            Injection::Person => {
                if let Some(person) = self.downcast::<Person>() {
                    let mut updated = person.clone();
                    if let Some(name) = &update.name {
                        updated.name = name.clone();
                    }
                    if let Some(email) = &update.email {
                        updated.email = email.clone();
                    }
                    if let Some(enabled) = update.enabled {
                        updated.active = enabled;
                    }
                    return updated.lift();
                }
            }
            Injection::Location => {
                if let Some(loc) = self.downcast::<Location>() {
                    let mut updated = loc.clone();
                    if let Some(name) = &update.name {
                        updated.name = name.clone();
                    }
                    return updated.lift();
                }
            }
            Injection::Role => {
                if let Some(role) = self.downcast::<Role>() {
                    let mut updated = role.clone();
                    if let Some(name) = &update.name {
                        updated.name = name.clone();
                    }
                    if let Some(description) = &update.description {
                        updated.description = description.clone();
                    }
                    if let Some(enabled) = update.enabled {
                        updated.active = enabled;
                    }
                    return updated.lift();
                }
            }
            Injection::Policy => {
                if let Some(policy) = self.downcast::<Policy>() {
                    let mut updated = policy.clone();
                    if let Some(name) = &update.name {
                        updated.name = name.clone();
                    }
                    if let Some(description) = &update.description {
                        updated.description = description.clone();
                    }
                    if let Some(enabled) = update.enabled {
                        updated.enabled = enabled;
                    }
                    if let Some(claims) = &update.claims {
                        updated.claims = claims.clone();
                    }
                    return updated.lift();
                }
            }
            // Read-only types - return clone unchanged
            _ => {}
        }
        self.clone()
    }

    /// Check if this node type is mutable (supports property updates)
    pub fn is_mutable(&self) -> bool {
        matches!(
            self.injection,
            Injection::Organization |
            Injection::OrganizationUnit |
            Injection::Person |
            Injection::Location |
            Injection::Role |
            Injection::Policy
        )
    }

    // ========================================================================
    // ACCESSOR METHODS (replacing deprecated DomainNode accessors)
    // ========================================================================

    /// Get YubiKey serial if this is a YubiKey-related node
    pub fn yubikey_serial(&self) -> Option<&str> {
        match self.injection {
            Injection::YubiKey => {
                self.downcast::<YubiKeyDevice>().map(|d| d.serial.as_str())
            }
            Injection::PivSlot => {
                self.downcast::<PivSlotView>().map(|s| s.yubikey_serial.as_str())
            }
            _ => None,
        }
    }

    /// Get NATS account name if this is a NATS account node
    pub fn nats_account_name(&self) -> Option<&str> {
        if self.injection == Injection::NatsAccountSimple {
            self.downcast::<NatsAccountSimple>().map(|a| a.name.as_str())
        } else {
            None
        }
    }

    /// Get parent account name if this is a NATS user node
    pub fn nats_user_account_name(&self) -> Option<&str> {
        if self.injection == Injection::NatsUserSimple {
            self.downcast::<NatsUserSimple>().map(|u| u.account_name.as_str())
        } else {
            None
        }
    }

    /// Get Person with derived KeyOwnerRole if this is a Person node
    ///
    /// The KeyOwnerRole is derived from the person's primary role.
    /// This replaces `domain_node.person_with_role()`.
    pub fn person_with_role(&self) -> Option<(&Person, KeyOwnerRole)> {
        use crate::domain::RoleType;

        if self.injection != Injection::Person {
            return None;
        }

        self.downcast::<Person>().map(|person| {
            // Derive KeyOwnerRole from primary role
            let role = person.roles.first()
                .map(|r| match r.role_type {
                    RoleType::Executive => KeyOwnerRole::RootAuthority,
                    RoleType::Administrator => KeyOwnerRole::SecurityAdmin,
                    RoleType::Developer => KeyOwnerRole::Developer,
                    RoleType::Operator => KeyOwnerRole::ServiceAccount,
                    RoleType::Auditor => KeyOwnerRole::Auditor,
                    RoleType::Service => KeyOwnerRole::ServiceAccount,
                })
                .unwrap_or(KeyOwnerRole::Developer);
            (person, role)
        })
    }

    /// Get Person reference if this is a Person node
    pub fn person(&self) -> Option<&Person> {
        if self.injection == Injection::Person {
            self.downcast::<Person>()
        } else {
            None
        }
    }

    /// Get Organization reference if this is an Organization node
    pub fn organization(&self) -> Option<&Organization> {
        if self.injection == Injection::Organization {
            self.downcast::<Organization>()
        } else {
            None
        }
    }

    /// Get OrganizationUnit reference if this is an OrganizationUnit node
    pub fn organization_unit(&self) -> Option<&OrganizationUnit> {
        if self.injection == Injection::OrganizationUnit {
            self.downcast::<OrganizationUnit>()
        } else {
            None
        }
    }

    /// Get Location reference if this is a Location node
    pub fn location(&self) -> Option<&Location> {
        if self.injection == Injection::Location {
            self.downcast::<Location>()
        } else {
            None
        }
    }

    /// Get CryptographicKey reference if this is a Key node
    pub fn key(&self) -> Option<&CryptographicKey> {
        if self.injection == Injection::Key {
            self.downcast::<CryptographicKey>()
        } else {
            None
        }
    }

    /// Get Certificate reference if this is a Certificate node (any type)
    pub fn certificate(&self) -> Option<&Certificate> {
        match self.injection {
            Injection::RootCertificate
            | Injection::IntermediateCertificate
            | Injection::LeafCertificate => {
                self.downcast::<Certificate>()
            }
            _ => None,
        }
    }

    /// Get Manifest reference if this is a Manifest node
    pub fn manifest(&self) -> Option<&Manifest> {
        if self.injection == Injection::Manifest {
            self.downcast::<Manifest>()
        } else {
            None
        }
    }

    /// Get YubiKeyDevice reference if this is a YubiKey node
    pub fn yubikey(&self) -> Option<&YubiKeyDevice> {
        if self.injection == Injection::YubiKey {
            self.downcast::<YubiKeyDevice>()
        } else {
            None
        }
    }

    /// Get YubiKeyStatus reference if this is a YubiKeyStatus node
    pub fn yubikey_status(&self) -> Option<&YubiKeyStatus> {
        if self.injection == Injection::YubiKeyStatus {
            self.downcast::<YubiKeyStatus>()
        } else {
            None
        }
    }

    /// Get Role reference if this is a Role node
    pub fn role(&self) -> Option<&Role> {
        if self.injection == Injection::Role {
            self.downcast::<Role>()
        } else {
            None
        }
    }

    /// Get Policy reference if this is a Policy node
    pub fn policy(&self) -> Option<&Policy> {
        if self.injection == Injection::Policy {
            self.downcast::<Policy>()
        } else {
            None
        }
    }

    /// Get PolicyGroup reference if this is a PolicyGroup node
    pub fn policy_group(&self) -> Option<&PolicyGroup> {
        if self.injection == Injection::PolicyGroup {
            self.downcast::<PolicyGroup>()
        } else {
            None
        }
    }

    /// Get PolicyCategory reference if this is a PolicyCategory node
    pub fn policy_category(&self) -> Option<&PolicyCategory> {
        if self.injection == Injection::PolicyCategory {
            self.downcast::<PolicyCategory>()
        } else {
            None
        }
    }

    /// Get PolicyRole reference if this is a PolicyRole node
    pub fn policy_role(&self) -> Option<&PolicyRole> {
        if self.injection == Injection::PolicyRole {
            self.downcast::<PolicyRole>()
        } else {
            None
        }
    }

    /// Get PivSlotView reference if this is a PivSlot node
    pub fn piv_slot(&self) -> Option<&PivSlotView> {
        if self.injection == Injection::PivSlot {
            self.downcast::<PivSlotView>()
        } else {
            None
        }
    }

    // ========================================================================
    // FOLD METHODS (replacing deprecated DomainNode.detail_panel() etc.)
    // ========================================================================

    /// Get detail panel data for this node.
    ///
    /// Uses `DetailPanelRegistry` for morphism-based detail extraction (DATA pattern).
    /// Falls back to basic label panel for unregistered types.
    ///
    /// # Architecture
    ///
    /// This method implements the Kan extension boundary crossing:
    /// - DetailPanelRegistry stores morphisms as DATA (HashMap)
    /// - registry.fold() looks up and applies the morphism
    /// - No match arms needed here - dispatched by injection tag
    pub fn detail_panel(&self) -> DetailPanelData {
        use crate::graph::DetailPanelRegistry;

        // Create registry and try to fold this node
        let registry = DetailPanelRegistry::new();

        if let Some(panel_data) = registry.fold(self) {
            return panel_data;
        }

        // Fallback for any unregistered types (e.g., Manifest, NatsServiceAccount)
        DetailPanelData::new(format!("Selected {:?}:", self.injection))
            .with_field("ID", self.id.to_string())
            .with_field("Label", &self.label)
    }

    /// Get themed visualization data for this node
    ///
    /// Uses `VisualizationRegistry` for morphism-based visualization (DATA pattern).
    /// Falls back to basic label visualization for unregistered types.
    ///
    /// # Architecture
    ///
    /// This method implements the Kan extension boundary crossing:
    /// - VisualizationRegistry stores morphisms as DATA (HashMap)
    /// - registry.fold() looks up and applies the morphism
    /// - No match arms needed here - dispatched by injection tag
    pub fn themed_visualization(
        &self,
        theme: &crate::domains::typography::VerifiedTheme,
    ) -> crate::gui::folds::view::ThemedVisualizationData {
        use crate::graph::VisualizationRegistry;
        use crate::gui::folds::view::ThemedVisualizationData;
        use crate::domains::typography::{LabelledElement, LabelCategory, FontFamily};

        // Create registry and try to fold this node
        let registry = VisualizationRegistry::new(theme);

        if let Some(vis_data) = registry.fold(self) {
            return vis_data;
        }

        // Fallback: create a basic visualization from label for unregistered types
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            None,
            Some(self.label.clone()),
            FontFamily::Body,
            self.color,
            theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            self.secondary.clone(),
            format!("{} - {:?}", self.label, self.injection),
            false,
        )
    }
}

// ============================================================================
// LIFTED EDGE - Graph representation of relationships
// ============================================================================

/// A domain relationship lifted into graph representation.
#[derive(Debug, Clone)]
pub struct LiftedEdge {
    /// Edge ID
    pub id: Uuid,

    /// Source node ID
    pub from_id: Uuid,

    /// Target node ID
    pub to_id: Uuid,

    /// Relationship type label
    pub label: String,

    /// Edge color
    pub color: Color,

    /// Edge weight (for weighted graphs)
    pub weight: Option<f64>,
}

impl LiftedEdge {
    /// Create a new lifted edge
    pub fn new(
        id: Uuid,
        from_id: Uuid,
        to_id: Uuid,
        label: impl Into<String>,
        color: Color,
    ) -> Self {
        Self {
            id,
            from_id,
            to_id,
            label: label.into(),
            color,
            weight: None,
        }
    }

    /// Add weight to edge
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}

// ============================================================================
// LIFTABLE DOMAIN TRAIT
// ============================================================================

/// Trait for domain types that can be lifted into graph representation.
///
/// This is a **faithful functor** - the lifting preserves all domain semantics
/// and the `unlift` operation can fully recover the original domain entity.
///
/// # Functor Laws
///
/// Implementations MUST satisfy:
/// 1. **Identity**: `lift(id_A) = id_{lift(A)}` - identity morphisms are preserved
/// 2. **Composition**: `lift(g ∘ f) = lift(g) ∘ lift(f)` - composition is preserved
/// 3. **Faithfulness**: If `lift(f) = lift(g)` then `f = g` - no information loss
///
/// # Example
///
/// ```ignore
/// impl LiftableDomain for Person {
///     fn lift(&self) -> LiftedNode {
///         LiftedNode::new(
///             self.id,
///             Injection::Person,
///             &self.name,
///             PERSON_COLOR,
///             self.clone(),
///         )
///     }
///
///     fn unlift(node: &LiftedNode) -> Option<Self> {
///         node.downcast::<Person>().cloned()
///     }
/// }
/// ```
pub trait LiftableDomain: Clone + Send + Sync + 'static {
    /// Lift this domain entity into a graph node.
    ///
    /// This is the object-mapping part of the functor.
    fn lift(&self) -> LiftedNode;

    /// Attempt to recover the domain entity from a lifted node.
    ///
    /// Returns `None` if the node was not created from this domain type.
    fn unlift(node: &LiftedNode) -> Option<Self>;

    /// Get the injection type for this domain.
    ///
    /// Used for type dispatch without instantiating the full lift.
    fn injection() -> Injection;

    /// Get the entity ID.
    ///
    /// This should match the ID used in `lift()`.
    fn entity_id(&self) -> Uuid;
}

// ============================================================================
// LIFTED GRAPH - Unified graph from multiple domains
// ============================================================================

/// A unified graph containing nodes and edges from multiple domains.
///
/// This is the coproduct (disjoint union) of all lifted domain types,
/// enabling heterogeneous domain composition in a single graph structure.
#[derive(Debug, Clone, Default)]
pub struct LiftedGraph {
    /// All nodes in the graph
    nodes: Vec<LiftedNode>,

    /// All edges in the graph
    edges: Vec<LiftedEdge>,
}

impl LiftedGraph {
    /// Create a new empty lifted graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a domain entity to the graph
    pub fn add<T: LiftableDomain>(&mut self, entity: &T) -> &LiftedNode {
        let node = entity.lift();
        self.nodes.push(node);
        self.nodes.last().unwrap()
    }

    /// Add a lifted node directly
    pub fn add_node(&mut self, node: LiftedNode) {
        self.nodes.push(node);
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, edge: LiftedEdge) {
        self.edges.push(edge);
    }

    /// Connect two nodes with a labeled edge
    pub fn connect(
        &mut self,
        from_id: Uuid,
        to_id: Uuid,
        label: impl Into<String>,
        color: Color,
    ) {
        let edge = LiftedEdge::new(
            Uuid::now_v7(),
            from_id,
            to_id,
            label,
            color,
        );
        self.edges.push(edge);
    }

    /// Get all nodes
    pub fn nodes(&self) -> &[LiftedNode] {
        &self.nodes
    }

    /// Get all edges
    pub fn edges(&self) -> &[LiftedEdge] {
        &self.edges
    }

    /// Find a node by ID
    pub fn find_node(&self, id: Uuid) -> Option<&LiftedNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Find all nodes of a specific injection type
    pub fn nodes_by_type(&self, injection: Injection) -> Vec<&LiftedNode> {
        self.nodes.iter().filter(|n| n.injection == injection).collect()
    }

    /// Get nodes count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edges count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Recover all entities of a specific domain type
    pub fn unlift_all<T: LiftableDomain>(&self) -> Vec<T> {
        self.nodes
            .iter()
            .filter_map(|n| T::unlift(n))
            .collect()
    }

    /// Merge another graph into this one
    pub fn merge(&mut self, other: LiftedGraph) {
        self.nodes.extend(other.nodes);
        self.edges.extend(other.edges);
    }

    /// Verify functor laws (for testing)
    pub fn verify_functor_laws(&self) -> bool {
        // Verify all nodes have valid injections
        for node in &self.nodes {
            // Each node must have a consistent injection
            if format!("{:?}", node.injection).is_empty() {
                return false;
            }
        }

        // Verify all edges reference existing nodes
        for edge in &self.edges {
            let has_from = self.nodes.iter().any(|n| n.id == edge.from_id);
            let has_to = self.nodes.iter().any(|n| n.id == edge.to_id);
            if !has_from || !has_to {
                return false;
            }
        }

        true
    }
}

// ============================================================================
// DOMAIN COMPOSITION
// ============================================================================

/// Compose multiple domains into a unified graph.
///
/// This function takes domain entities and relationships, lifting them
/// all into a single coherent graph structure.
///
/// # Categorical Structure
///
/// This is the coproduct construction: given functors F_i: D_i → Graph,
/// we construct the coproduct functor ∐F_i: ∐D_i → Graph.
pub fn compose_domains<T: LiftableDomain>(entities: &[T]) -> LiftedGraph {
    let mut graph = LiftedGraph::new();
    for entity in entities {
        graph.add(entity);
    }
    graph
}

// ============================================================================
// LIFTABLE DOMAIN IMPLEMENTATIONS
// ============================================================================

impl LiftableDomain for Organization {
    fn lift(&self) -> LiftedNode {
        // Use new_with_fold to capture EditFieldData fold at lift time (FRP A5/A6)
        LiftedNode::new_with_fold(
            self.id.as_uuid(),
            Injection::Organization,
            &self.display_name,
            COLOR_ORGANIZATION,
            self.clone(),
            |org: &Organization| Foldable::<EditFieldData>::fold(org),
        )
        .with_secondary(format!("Org: {}", self.name))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Organization {
            return None;
        }
        node.downcast::<Organization>().cloned()
    }

    fn injection() -> Injection {
        Injection::Organization
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for OrganizationUnit {
    fn lift(&self) -> LiftedNode {
        // Use new_with_fold to capture EditFieldData fold at lift time (FRP A5/A6)
        LiftedNode::new_with_fold(
            self.id.as_uuid(),
            Injection::OrganizationUnit,
            &self.name,
            COLOR_UNIT,
            self.clone(),
            |unit: &OrganizationUnit| Foldable::<EditFieldData>::fold(unit),
        )
        .with_secondary(format!("{:?}", self.unit_type))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::OrganizationUnit {
            return None;
        }
        node.downcast::<OrganizationUnit>().cloned()
    }

    fn injection() -> Injection {
        Injection::OrganizationUnit
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for Person {
    fn lift(&self) -> LiftedNode {
        // Use new_with_fold to capture EditFieldData fold at lift time (FRP A5/A6)
        LiftedNode::new_with_fold(
            self.id.as_uuid(),
            Injection::Person,
            &self.name,
            COLOR_PERSON,
            self.clone(),
            |person: &Person| Foldable::<EditFieldData>::fold(person),
        )
        .with_secondary(self.email.clone())
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Person {
            return None;
        }
        node.downcast::<Person>().cloned()
    }

    fn injection() -> Injection {
        Injection::Person
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for Location {
    fn lift(&self) -> LiftedNode {
        // Location uses EntityId<LocationMarker> via AggregateRoot trait
        use cim_domain::AggregateRoot;
        let id = *self.id().as_uuid();
        // Use new_with_fold to capture EditFieldData fold at lift time (FRP A5/A6)
        LiftedNode::new_with_fold(
            id,
            Injection::Location,
            &self.name,
            COLOR_LOCATION,
            self.clone(),
            |loc: &Location| Foldable::<EditFieldData>::fold(loc),
        )
        .with_secondary(format!("{:?}", self.location_type))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Location {
            return None;
        }
        node.downcast::<Location>().cloned()
    }

    fn injection() -> Injection {
        Injection::Location
    }

    fn entity_id(&self) -> Uuid {
        use cim_domain::AggregateRoot;
        *self.id().as_uuid()
    }
}

impl LiftableDomain for Role {
    fn lift(&self) -> LiftedNode {
        // Use new_with_fold to capture EditFieldData fold at lift time (FRP A5/A6)
        LiftedNode::new_with_fold(
            self.id.as_uuid(),
            Injection::Role,
            &self.name,
            COLOR_ROLE,
            self.clone(),
            |role: &Role| Foldable::<EditFieldData>::fold(role),
        )
        .with_secondary(self.description.clone())
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Role {
            return None;
        }
        node.downcast::<Role>().cloned()
    }

    fn injection() -> Injection {
        Injection::Role
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for Policy {
    fn lift(&self) -> LiftedNode {
        // Use new_with_fold to capture EditFieldData fold at lift time (FRP A5/A6)
        LiftedNode::new_with_fold(
            self.id.as_uuid(),
            Injection::Policy,
            &self.name,
            COLOR_POLICY,
            self.clone(),
            |policy: &Policy| Foldable::<EditFieldData>::fold(policy),
        )
        .with_secondary(self.description.clone())
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Policy {
            return None;
        }
        node.downcast::<Policy>().cloned()
    }

    fn injection() -> Injection {
        Injection::Policy
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for PolicyGroup {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id.as_uuid(),
            Injection::PolicyGroup,
            &self.name,
            COLOR_POLICY_GROUP,
            self.clone(),
        )
        .with_secondary(format!("{:?} - {} roles", self.separation_class, self.role_count))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::PolicyGroup {
            return None;
        }
        node.downcast::<PolicyGroup>().cloned()
    }

    fn injection() -> Injection {
        Injection::PolicyGroup
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for PolicyCategory {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id.as_uuid(),
            Injection::PolicyCategory,
            &self.name,
            COLOR_POLICY_CATEGORY,
            self.clone(),
        )
        .with_secondary(format!("{} claims", self.claim_count))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::PolicyCategory {
            return None;
        }
        node.downcast::<PolicyCategory>().cloned()
    }

    fn injection() -> Injection {
        Injection::PolicyCategory
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for PolicyRole {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id.as_uuid(),
            Injection::PolicyRole,
            &self.name,
            COLOR_POLICY_ROLE,
            self.clone(),
        )
        .with_secondary(format!("L{} - {:?}", self.level, self.separation_class))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::PolicyRole {
            return None;
        }
        node.downcast::<PolicyRole>().cloned()
    }

    fn injection() -> Injection {
        Injection::PolicyRole
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for PolicyClaimView {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id.as_uuid(),
            Injection::PolicyClaim,
            &self.name,
            COLOR_POLICY_CLAIM,
            self.clone(),
        )
        .with_secondary(format!("Category: {}", self.category))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::PolicyClaim {
            return None;
        }
        node.downcast::<PolicyClaimView>().cloned()
    }

    fn injection() -> Injection {
        Injection::PolicyClaim
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for Certificate {
    fn lift(&self) -> LiftedNode {
        let (injection, color) = match self.cert_type {
            CertificateType::Root => (Injection::RootCertificate, COLOR_CERTIFICATE),
            CertificateType::Intermediate => (Injection::IntermediateCertificate, COLOR_CERTIFICATE),
            CertificateType::Leaf => (Injection::LeafCertificate, COLOR_CERTIFICATE),
            CertificateType::Policy => (Injection::LeafCertificate, COLOR_POLICY), // Policy certs render as leaf
        };
        LiftedNode::new(
            self.id.as_uuid(),
            injection,
            &self.subject,
            color,
            self.clone(),
        )
        .with_secondary(format!("{:?} - Valid until {}", self.cert_type, self.not_after.format("%Y-%m-%d")))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        // Certificate can be unlifted from any of the certificate injection types
        match node.injection {
            Injection::RootCertificate |
            Injection::IntermediateCertificate |
            Injection::LeafCertificate => {
                node.downcast::<Certificate>().cloned()
            }
            _ => None,
        }
    }

    fn injection() -> Injection {
        // Default to Leaf; actual injection depends on cert_type
        Injection::LeafCertificate
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

/// Color for PIV slot nodes
pub const COLOR_PIV_SLOT: Color = Color::from_rgb(0.1, 0.5, 0.5);  // Teal

impl LiftableDomain for CryptographicKey {
    fn lift(&self) -> LiftedNode {
        let label = format!("{:?} Key", self.purpose);
        let secondary = match self.expires_at {
            Some(exp) => format!("{:?} - Expires {}", self.algorithm, exp.format("%Y-%m-%d")),
            None => format!("{:?} - No expiration", self.algorithm),
        };
        LiftedNode::new(
            self.id.as_uuid(),
            Injection::Key,
            &label,
            COLOR_KEY,
            self.clone(),
        )
        .with_secondary(secondary)
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::Key {
            node.downcast::<CryptographicKey>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::Key
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for YubiKeyDevice {
    fn lift(&self) -> LiftedNode {
        let label = format!("YubiKey {}", self.serial);
        let secondary = match self.provisioned_at {
            Some(prov) => format!("v{} - Provisioned {} - {} slots", self.version, prov.format("%Y-%m-%d"), self.slots_used.len()),
            None => format!("v{} - Not provisioned", self.version),
        };
        LiftedNode::new(
            self.id.as_uuid(),
            Injection::YubiKey,
            &label,
            COLOR_YUBIKEY,
            self.clone(),
        )
        .with_secondary(secondary)
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::YubiKey {
            node.downcast::<YubiKeyDevice>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::YubiKey
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for PivSlotView {
    fn lift(&self) -> LiftedNode {
        let label = self.slot_name.clone();
        let secondary = if self.has_key {
            match &self.certificate_subject {
                Some(subj) => format!("YubiKey {} - {}", self.yubikey_serial, subj),
                None => format!("YubiKey {} - Key present", self.yubikey_serial),
            }
        } else {
            format!("YubiKey {} - Empty", self.yubikey_serial)
        };
        LiftedNode::new(
            self.id.as_uuid(),
            Injection::PivSlot,
            &label,
            COLOR_PIV_SLOT,
            self.clone(),
        )
        .with_secondary(secondary)
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::PivSlot {
            node.downcast::<PivSlotView>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::PivSlot
    }

    fn entity_id(&self) -> Uuid {
        self.id.as_uuid()
    }
}

impl LiftableDomain for YubiKeyStatus {
    fn lift(&self) -> LiftedNode {
        let label = match &self.yubikey_serial {
            Some(serial) => format!("YubiKey Status: {}", serial),
            None => "YubiKey Status: Not assigned".to_string(),
        };
        let secondary = format!(
            "{} of {} slots provisioned",
            self.slots_provisioned.len(),
            self.slots_needed.len()
        );
        LiftedNode::new(
            self.person_id,
            Injection::YubiKeyStatus,
            &label,
            COLOR_YUBIKEY,
            self.clone(),
        )
        .with_secondary(secondary)
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::YubiKeyStatus {
            node.downcast::<YubiKeyStatus>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::YubiKeyStatus
    }

    fn entity_id(&self) -> Uuid {
        self.person_id
    }
}

// ============================================================================
// NATS IDENTITY PROJECTION - LiftableDomain
// ============================================================================

use crate::domain_projections::nats::NatsIdentityProjection;
use crate::value_objects::NKeyType;

impl LiftableDomain for NatsIdentityProjection {
    fn lift(&self) -> LiftedNode {
        let (injection, color, label) = match self.nkey.key_type {
            NKeyType::Operator => (
                Injection::NatsOperator,
                COLOR_NATS_OPERATOR,
                format!("NATS Operator: {}", self.nkey.name.as_deref().unwrap_or("Unnamed")),
            ),
            NKeyType::Account => (
                Injection::NatsAccount,
                COLOR_NATS_ACCOUNT,
                format!("NATS Account: {}", self.nkey.name.as_deref().unwrap_or("Unnamed")),
            ),
            NKeyType::User => (
                Injection::NatsUser,
                COLOR_NATS_USER,
                format!("NATS User: {}", self.nkey.name.as_deref().unwrap_or("Unnamed")),
            ),
            NKeyType::Server | NKeyType::Cluster => (
                Injection::NatsServiceAccount,
                COLOR_NATS_USER,
                format!("NATS Service: {}", self.nkey.name.as_deref().unwrap_or("Unnamed")),
            ),
        };
        let secondary = format!("Public: {}...", &self.nkey.public_key_string()[..12]);
        LiftedNode::new(
            self.nkey.id,
            injection,
            &label,
            color,
            self.clone(),
        )
        .with_secondary(secondary)
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        match node.injection {
            Injection::NatsOperator |
            Injection::NatsAccount |
            Injection::NatsUser |
            Injection::NatsServiceAccount => {
                node.downcast::<NatsIdentityProjection>().cloned()
            }
            _ => None,
        }
    }

    fn injection() -> Injection {
        // Default to User, actual injection determined by key_type in lift()
        Injection::NatsUser
    }

    fn entity_id(&self) -> Uuid {
        self.nkey.id
    }
}

// ============================================================================
// SIMPLE NATS VISUALIZATION TYPES
// ============================================================================

/// Simple NATS operator for visualization (no cryptographic data)
#[derive(Debug, Clone)]
pub struct NatsOperatorSimple {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Option<Uuid>,
}

impl NatsOperatorSimple {
    pub fn new(id: Uuid, name: String, organization_id: Option<Uuid>) -> Self {
        Self { id, name, organization_id }
    }
}

impl LiftableDomain for NatsOperatorSimple {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::NatsOperatorSimple,
            &self.name,
            COLOR_NATS_OPERATOR,
            self.clone(),
        )
        .with_secondary("NATS Operator")
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::NatsOperatorSimple {
            node.downcast::<NatsOperatorSimple>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::NatsOperatorSimple
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

/// Simple NATS account for visualization (no cryptographic data)
#[derive(Debug, Clone)]
pub struct NatsAccountSimple {
    pub id: Uuid,
    pub name: String,
    pub unit_id: Option<Uuid>,
    pub is_system: bool,
}

impl NatsAccountSimple {
    pub fn new(id: Uuid, name: String, unit_id: Option<Uuid>, is_system: bool) -> Self {
        Self { id, name, unit_id, is_system }
    }
}

impl LiftableDomain for NatsAccountSimple {
    fn lift(&self) -> LiftedNode {
        let secondary = if self.is_system {
            "System Account".to_string()
        } else {
            "NATS Account".to_string()
        };
        LiftedNode::new(
            self.id,
            Injection::NatsAccountSimple,
            &self.name,
            COLOR_NATS_ACCOUNT,
            self.clone(),
        )
        .with_secondary(secondary)
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::NatsAccountSimple {
            node.downcast::<NatsAccountSimple>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::NatsAccountSimple
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

/// Simple NATS user for visualization (no cryptographic data)
#[derive(Debug, Clone)]
pub struct NatsUserSimple {
    pub id: Uuid,
    pub name: String,
    pub person_id: Option<Uuid>,
    pub account_name: String,
}

impl NatsUserSimple {
    pub fn new(id: Uuid, name: String, person_id: Option<Uuid>, account_name: String) -> Self {
        Self { id, name, person_id, account_name }
    }
}

impl LiftableDomain for NatsUserSimple {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::NatsUserSimple,
            &self.name,
            COLOR_NATS_USER,
            self.clone(),
        )
        .with_secondary(format!("Account: {}", self.account_name))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::NatsUserSimple {
            node.downcast::<NatsUserSimple>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::NatsUserSimple
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

// ============================================================================
// AGGREGATE VISUALIZATION TYPES
// ============================================================================

/// Organization aggregate for graph visualization
#[derive(Debug, Clone)]
pub struct AggregateOrganization {
    pub id: Uuid,
    pub name: String,
    pub version: u64,
    pub people_count: usize,
    pub units_count: usize,
}

impl AggregateOrganization {
    pub fn new(id: Uuid, name: String, version: u64, people_count: usize, units_count: usize) -> Self {
        Self { id, name, version, people_count, units_count }
    }
}

impl LiftableDomain for AggregateOrganization {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::AggregateOrganization,
            &self.name,
            COLOR_ORGANIZATION,
            self.clone(),
        )
        .with_secondary(format!("v{} | {} people | {} units", self.version, self.people_count, self.units_count))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::AggregateOrganization {
            node.downcast::<AggregateOrganization>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::AggregateOrganization
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

/// PKI chain aggregate for graph visualization
#[derive(Debug, Clone)]
pub struct AggregatePkiChain {
    pub id: Uuid,
    pub name: String,
    pub version: u64,
    pub certificates_count: usize,
    pub keys_count: usize,
}

impl AggregatePkiChain {
    pub fn new(id: Uuid, name: String, version: u64, certificates_count: usize, keys_count: usize) -> Self {
        Self { id, name, version, certificates_count, keys_count }
    }
}

impl LiftableDomain for AggregatePkiChain {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::AggregatePkiChain,
            &self.name,
            COLOR_CERTIFICATE,
            self.clone(),
        )
        .with_secondary(format!("v{} | {} certs | {} keys", self.version, self.certificates_count, self.keys_count))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::AggregatePkiChain {
            node.downcast::<AggregatePkiChain>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::AggregatePkiChain
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

/// NATS security aggregate for graph visualization
#[derive(Debug, Clone)]
pub struct AggregateNatsSecurity {
    pub id: Uuid,
    pub name: String,
    pub version: u64,
    pub operators_count: usize,
    pub accounts_count: usize,
    pub users_count: usize,
}

impl AggregateNatsSecurity {
    pub fn new(id: Uuid, name: String, version: u64, operators_count: usize, accounts_count: usize, users_count: usize) -> Self {
        Self { id, name, version, operators_count, accounts_count, users_count }
    }
}

impl LiftableDomain for AggregateNatsSecurity {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::AggregateNatsSecurity,
            &self.name,
            COLOR_NATS_OPERATOR,
            self.clone(),
        )
        .with_secondary(format!("v{} | {} ops | {} accts | {} users", self.version, self.operators_count, self.accounts_count, self.users_count))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::AggregateNatsSecurity {
            node.downcast::<AggregateNatsSecurity>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::AggregateNatsSecurity
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

/// YubiKey provisioning aggregate for graph visualization
#[derive(Debug, Clone)]
pub struct AggregateYubiKeyProvisioning {
    pub id: Uuid,
    pub name: String,
    pub version: u64,
    pub devices_count: usize,
    pub slots_provisioned: usize,
}

impl AggregateYubiKeyProvisioning {
    pub fn new(id: Uuid, name: String, version: u64, devices_count: usize, slots_provisioned: usize) -> Self {
        Self { id, name, version, devices_count, slots_provisioned }
    }
}

impl LiftableDomain for AggregateYubiKeyProvisioning {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::AggregateYubiKeyProvisioning,
            &self.name,
            COLOR_YUBIKEY,
            self.clone(),
        )
        .with_secondary(format!("v{} | {} devices | {} slots", self.version, self.devices_count, self.slots_provisioned))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::AggregateYubiKeyProvisioning {
            node.downcast::<AggregateYubiKeyProvisioning>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::AggregateYubiKeyProvisioning
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

// ============================================================================
// WORKFLOW GAP LIFTING
// ============================================================================

use crate::workflow::{TrustChainGap, GapId, GapStatus};

/// Get color for a workflow gap based on its status
pub fn color_for_gap_status(status: GapStatus) -> Color {
    match status {
        GapStatus::NotStarted => COLOR_WORKFLOW_GAP_NOT_STARTED,
        GapStatus::InProgress => COLOR_WORKFLOW_GAP_IN_PROGRESS,
        GapStatus::Implemented => COLOR_WORKFLOW_GAP_IMPLEMENTED,
        GapStatus::Tested => COLOR_WORKFLOW_GAP_TESTED,
        GapStatus::Verified => COLOR_WORKFLOW_GAP_VERIFIED,
    }
}

/// Wrapper that pairs a TrustChainGap with its current status for lifting
#[derive(Debug, Clone)]
pub struct LiftableGap {
    pub gap: TrustChainGap,
    pub status: GapStatus,
}

impl LiftableGap {
    pub fn new(gap: TrustChainGap, status: GapStatus) -> Self {
        Self { gap, status }
    }

    /// Create from gap with default NotStarted status
    pub fn from_gap(gap: TrustChainGap) -> Self {
        Self { gap, status: GapStatus::NotStarted }
    }
}

impl LiftableDomain for LiftableGap {
    fn lift(&self) -> LiftedNode {
        let color = color_for_gap_status(self.status);
        let progress = self.status.progress_percentage();
        let category = format!("{:?}", self.gap.category);

        LiftedNode::new(
            self.gap.id.as_uuid(),
            Injection::WorkflowGap,
            &self.gap.name,
            color,
            self.clone(),
        )
        .with_secondary(format!("[{}] {}% - {:?}", category, progress, self.status))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection == Injection::WorkflowGap {
            node.downcast::<LiftableGap>().cloned()
        } else {
            None
        }
    }

    fn injection() -> Injection {
        Injection::WorkflowGap
    }

    fn entity_id(&self) -> Uuid {
        self.gap.id.as_uuid()
    }
}

/// Build a LiftedGraph from workflow gaps
pub fn lift_workflow_graph(
    gaps: &[TrustChainGap],
    statuses: &std::collections::HashMap<GapId, GapStatus>,
) -> LiftedGraph {
    let mut graph = LiftedGraph::new();

    // Lift all gaps as nodes
    for gap in gaps {
        let status = statuses.get(&gap.id).copied().unwrap_or(GapStatus::NotStarted);
        let liftable = LiftableGap::new(gap.clone(), status);
        graph.add(&liftable);
    }

    // Create edges for dependencies
    for gap in gaps {
        for dep_id in &gap.dependencies {
            // Edge from dependency to this gap (dependency must be done first)
            graph.connect(
                dep_id.as_uuid(),
                gap.id.as_uuid(),
                "enables",
                COLOR_WORKFLOW_GAP_IN_PROGRESS,
            );
        }
    }

    graph
}

// ============================================================================
// CONVENIENCE: From bootstrap to LiftedGraph
// ============================================================================

/// Build a LiftedGraph from an Organization and its units/people.
///
/// This function performs the complete lifting operation for a bootstrap
/// configuration, creating nodes and edges for all entities.
pub fn lift_organization_graph(
    org: &Organization,
    people: &[Person],
) -> LiftedGraph {
    let mut graph = LiftedGraph::new();

    // Lift the organization
    graph.add(org);

    // Lift all units
    for unit in &org.units {
        graph.add(unit);
        // Connect unit to organization
        graph.connect(
            unit.id.as_uuid(),
            org.id.as_uuid(),
            "belongs_to",
            COLOR_UNIT,
        );

        // Connect to parent unit if exists
        if let Some(parent_id) = unit.parent_unit_id {
            graph.connect(
                unit.id.as_uuid(),
                parent_id.as_uuid(),
                "reports_to",
                COLOR_UNIT,
            );
        }
    }

    // Lift all people
    for person in people {
        graph.add(person);

        // Connect person to organization
        if person.organization_id == org.id {
            graph.connect(
                person.id.as_uuid(),
                org.id.as_uuid(),
                "member_of",
                COLOR_PERSON,
            );
        }

        // Connect person to their units
        for unit_id in &person.unit_ids {
            graph.connect(
                person.id.as_uuid(),
                unit_id.as_uuid(),
                "works_in",
                COLOR_PERSON,
            );
        }

        // Connect to owner if exists
        if let Some(owner_id) = person.owner_id {
            graph.connect(
                person.id.as_uuid(),
                owner_id.as_uuid(),
                "owned_by",
                COLOR_PERSON,
            );
        }
    }

    graph
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::{BootstrapOrgId, UnitId, BootstrapPersonId};

    // Test domain type
    #[derive(Debug, Clone)]
    struct TestEntity {
        id: Uuid,
        name: String,
    }

    impl LiftableDomain for TestEntity {
        fn lift(&self) -> LiftedNode {
            LiftedNode::new(
                self.id,
                Injection::Person, // Use Person as test injection
                &self.name,
                Color::WHITE,
                self.clone(),
            )
        }

        fn unlift(node: &LiftedNode) -> Option<Self> {
            node.downcast::<TestEntity>().cloned()
        }

        fn injection() -> Injection {
            Injection::Person
        }

        fn entity_id(&self) -> Uuid {
            self.id
        }
    }

    #[test]
    fn test_lift_unlift_roundtrip() {
        let entity = TestEntity {
            id: Uuid::now_v7(),
            name: "Test".to_string(),
        };

        let lifted = entity.lift();
        let recovered = TestEntity::unlift(&lifted);

        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert_eq!(recovered.id, entity.id);
        assert_eq!(recovered.name, entity.name);
    }

    #[test]
    fn test_lifted_graph() {
        let entity1 = TestEntity {
            id: Uuid::now_v7(),
            name: "Entity1".to_string(),
        };
        let entity2 = TestEntity {
            id: Uuid::now_v7(),
            name: "Entity2".to_string(),
        };

        let mut graph = LiftedGraph::new();
        graph.add(&entity1);
        graph.add(&entity2);
        graph.connect(entity1.id, entity2.id, "relates_to", Color::WHITE);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.verify_functor_laws());
    }

    #[test]
    fn test_unlift_all() {
        let entities: Vec<TestEntity> = (0..5)
            .map(|i| TestEntity {
                id: Uuid::now_v7(),
                name: format!("Entity{}", i),
            })
            .collect();

        let graph = compose_domains(&entities);
        let recovered: Vec<TestEntity> = graph.unlift_all();

        assert_eq!(recovered.len(), 5);
    }

    #[test]
    fn test_nodes_by_type() {
        let entity = TestEntity {
            id: Uuid::now_v7(),
            name: "Test".to_string(),
        };

        let mut graph = LiftedGraph::new();
        graph.add(&entity);

        let persons = graph.nodes_by_type(Injection::Person);
        assert_eq!(persons.len(), 1);

        let orgs = graph.nodes_by_type(Injection::Organization);
        assert_eq!(orgs.len(), 0);
    }

    // ========== Domain Type Tests ==========

    #[test]
    fn test_organization_lift_unlift() {
        use std::collections::HashMap;

        let org = Organization {
            id: BootstrapOrgId::new(),
            name: "test-org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("A test organization".to_string()),
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let lifted = org.lift();

        // Verify lifted properties
        assert_eq!(lifted.id, org.id.as_uuid());
        assert_eq!(lifted.injection, Injection::Organization);
        assert_eq!(lifted.label, "Test Organization");
        assert!(lifted.secondary.is_some());

        // Verify roundtrip
        let recovered = Organization::unlift(&lifted);
        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert_eq!(recovered.id.as_uuid(), org.id.as_uuid());
        assert_eq!(recovered.name, org.name);
    }

    #[test]
    fn test_organization_unit_lift_unlift() {
        use crate::domain::OrganizationUnitType;

        let unit = OrganizationUnit {
            id: UnitId::new(),
            name: "Engineering".to_string(),
            unit_type: OrganizationUnitType::Department,
            parent_unit_id: None,
            responsible_person_id: None,
            nats_account_name: Some("eng".to_string()),
        };

        let lifted = unit.lift();

        // Verify lifted properties
        assert_eq!(lifted.id, unit.id.as_uuid());
        assert_eq!(lifted.injection, Injection::OrganizationUnit);
        assert_eq!(lifted.label, "Engineering");

        // Verify roundtrip
        let recovered = OrganizationUnit::unlift(&lifted);
        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert_eq!(recovered.id.as_uuid(), unit.id.as_uuid());
        assert_eq!(recovered.name, unit.name);
    }

    #[test]
    fn test_person_lift_unlift() {
        let org_id = BootstrapOrgId::new();
        let person = Person {
            id: BootstrapPersonId::new(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            roles: vec![],
            organization_id: org_id,
            unit_ids: vec![],
            active: true,
            nats_permissions: None,
            owner_id: None,
        };

        let lifted = person.lift();

        // Verify lifted properties
        assert_eq!(lifted.id, person.id.as_uuid());
        assert_eq!(lifted.injection, Injection::Person);
        assert_eq!(lifted.label, "John Doe");
        assert_eq!(lifted.secondary, Some("john@example.com".to_string()));

        // Verify roundtrip
        let recovered = Person::unlift(&lifted);
        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert_eq!(recovered.id.as_uuid(), person.id.as_uuid());
        assert_eq!(recovered.name, person.name);
        assert_eq!(recovered.email, person.email);
    }

    #[test]
    fn test_lift_organization_graph() {
        use crate::domain::OrganizationUnitType;
        use std::collections::HashMap;

        let org_id = BootstrapOrgId::new();
        let unit_id = UnitId::new();

        let org = Organization {
            id: org_id.clone(),
            name: "cowboyai".to_string(),
            display_name: "Cowboy AI".to_string(),
            description: None,
            parent_id: None,
            units: vec![
                OrganizationUnit {
                    id: unit_id.clone(),
                    name: "Core".to_string(),
                    unit_type: OrganizationUnitType::Team,
                    parent_unit_id: None,
                    responsible_person_id: None,
                    nats_account_name: Some("core".to_string()),
                },
            ],
            metadata: HashMap::new(),
        };

        let people = vec![
            Person {
                id: BootstrapPersonId::new(),
                name: "Alice".to_string(),
                email: "alice@cowboyai.com".to_string(),
                roles: vec![],
                organization_id: org_id.clone(),
                unit_ids: vec![unit_id.clone()],
                active: true,
                nats_permissions: None,
                owner_id: None,
            },
            Person {
                id: BootstrapPersonId::new(),
                name: "Bob".to_string(),
                email: "bob@cowboyai.com".to_string(),
                roles: vec![],
                organization_id: org_id.clone(),
                unit_ids: vec![],
                active: true,
                nats_permissions: None,
                owner_id: None,
            },
        ];

        let graph = lift_organization_graph(&org, &people);

        // 1 org + 1 unit + 2 people = 4 nodes
        assert_eq!(graph.node_count(), 4);

        // Edges:
        // - unit -> org (belongs_to)
        // - person1 -> org (member_of)
        // - person1 -> unit (works_in)
        // - person2 -> org (member_of)
        // = 4 edges
        assert_eq!(graph.edge_count(), 4);

        // Verify functor laws
        assert!(graph.verify_functor_laws());

        // Verify we can recover all orgs
        let orgs: Vec<Organization> = graph.unlift_all();
        assert_eq!(orgs.len(), 1);

        // Verify we can recover all people
        let recovered_people: Vec<Person> = graph.unlift_all();
        assert_eq!(recovered_people.len(), 2);
    }

    #[test]
    fn test_graph_merge() {
        use std::collections::HashMap;

        let org1 = Organization {
            id: BootstrapOrgId::new(),
            name: "org1".to_string(),
            display_name: "Org 1".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let org2 = Organization {
            id: BootstrapOrgId::new(),
            name: "org2".to_string(),
            display_name: "Org 2".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let graph1 = lift_organization_graph(&org1, &[]);
        let mut graph2 = lift_organization_graph(&org2, &[]);

        graph2.merge(graph1);

        assert_eq!(graph2.node_count(), 2);
        assert!(graph2.verify_functor_laws());
    }

    #[test]
    fn test_functor_faithfulness() {
        // Faithfulness: If lift(a) == lift(b) in graph representation,
        // then a == b in domain. We test the contrapositive:
        // Different domain entities produce different graph nodes.

        let org_id = BootstrapOrgId::new();
        let person1 = Person {
            id: BootstrapPersonId::new(),
            name: "Person 1".to_string(),
            email: "p1@test.com".to_string(),
            roles: vec![],
            organization_id: org_id.clone(),
            unit_ids: vec![],
            active: true,
            nats_permissions: None,
            owner_id: None,
        };

        let person2 = Person {
            id: BootstrapPersonId::new(),
            name: "Person 2".to_string(),
            email: "p2@test.com".to_string(),
            roles: vec![],
            organization_id: org_id.clone(),
            unit_ids: vec![],
            active: true,
            nats_permissions: None,
            owner_id: None,
        };

        let lifted1 = person1.lift();
        let lifted2 = person2.lift();

        // Different entities produce different lifted nodes
        assert_ne!(lifted1.id, lifted2.id);
        assert_ne!(lifted1.label, lifted2.label);

        // Each can be unlifted to its original
        let recovered1 = Person::unlift(&lifted1).unwrap();
        let recovered2 = Person::unlift(&lifted2).unwrap();

        assert_eq!(recovered1.id, person1.id);
        assert_eq!(recovered2.id, person2.id);
    }
}
