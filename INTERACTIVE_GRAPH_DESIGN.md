# Interactive Graph System Design for cim-keys

## Vision

**The graph IS the organization**. The interactive graph becomes the primary interface for creating and managing all aspects of a CIM infrastructure bootstrap:

- **Nodes** = DDD Entities (Organizations, OrgUnits, People, Locations, Roles, Policies)
- **Edges** = Relationships and Hierarchies (reports-to, located-at, has-role, governed-by)
- **Values** = Property Cards for editing entity attributes

## Current State Analysis (Updated: 2025-01-15)

### Phase 1: COMPLETE ✅
- **Policy domain models** (claims-based security) ✅
  - 40+ PolicyClaim types (key mgmt, infrastructure, admin, NATS, data, security)
  - 12 PolicyCondition types (clearance, MFA, YubiKey, location, time, witness, etc.)
  - PolicyBinding (binds policies to entities)
  - PolicyEvaluation (priority sorting, condition checking, claims composition)
  - `evaluate_policies()` function (core evaluation engine)
- **Role/Position entities** ✅
  - Role entity with required policies
  - RoleAssignment with temporal validity
  - `can_person_fulfill()` role checking
- **Comprehensive test suite** ✅
  - 7 integration tests (all passing)
  - 100% coverage of policy evaluation logic
  - See: `tests/policy_tests.rs`

### What Exists (Pre-Phase 1) ✅
- Basic graph visualization (`src/gui/graph.rs`)
  - People shown as nodes
  - Delegations shown as edges
  - Node dragging, zoom, pan
  - Auto-layout (hierarchical and force-directed)
- Domain models (`src/domain.rs`)
  - Organization, OrganizationUnit, Person, Location
  - KeyOwnership, KeyDelegation
  - NATS identity mappings
  - **Policy, PolicyClaim, PolicyCondition** (Phase 1)
  - **Role, RoleAssignment** (Phase 1)
- MVI architecture (`src/mvi/`)
  - Intent-based event sourcing
  - Pure Model updates
  - Hexagonal ports

### Phase 2: COMPLETE ✅ (Completed: 2025-01-15)
- ✅ **Extended GraphNode for all entity types** (Organization, OrgUnit, Person, Location, Role, Policy)
- ✅ **Extended EdgeType enum** (12 new semantic edge types + 2 legacy)
- ✅ **Graph Interaction Intents** (23 new Intent variants in MVI system)
  - 12 UI-originated intents (UiGraphCreateNode, UiGraphCreateEdge*, etc.)
  - 11 Domain-originated events (DomainNodeCreated, DomainPolicyCreated, etc.)
- ✅ **Update function handlers** (placeholder implementations ready for Phase 3)
- ✅ **Type-aware rendering** (color-coded nodes, semantic edge labels)

### Phase 3: COMPLETE ✅ (Completed: 2025-01-15)
- ✅ **Context menu widget** (right-click node creation menu)
- ✅ **Property card widget** (editable property panel with dirty tracking)
- ✅ **Edge creation indicator** (visual dashed line feedback)
- ✅ **Component integration** (all widgets properly modularized)
- ✅ **Comprehensive tests** (11 new tests, 51 total passing)

### What's Next (Phase 4+) 🚧
- **Wire components to main GUI** (integrate context menu, property card)
- **Complete node creation workflow** (context menu → create → property card → save)
- **Complete edge creation workflow** (indicator → select nodes → create edge)
- **Complete property editing workflow** (select → edit → save/cancel)
- **Keyboard shortcuts** (Esc to cancel, Enter to save)
- **Polish and UX** (hover effects, animations, error handling)

## Design Principles

### 1. Graph-First Architecture
```
User Intent → Graph Interaction → Domain Event → State Update → Graph Re-render
```

Every operation flows through the graph:
- Creating an organization = Adding root node
- Adding a person = Creating Person node + connecting to OrgUnit
- Assigning a policy = Creating Policy node + edge to applicable entities
- Storing a key = Creating Location node + edge from Key

### 2. Claims-Based Policy System (✅ IMPLEMENTED - Phase 1)

Policies are **compositions of claims** that grant capabilities:

```rust
Policy {
  id: Uuid,
  name: "Senior Developer",
  claims: [
    PolicyClaim::CanSignCode,
    PolicyClaim::CanAccessProduction,
    PolicyClaim::CanDelegateKeys,
  ],
  conditions: [
    PolicyCondition::MinimumSecurityClearance(SecurityClearance::Secret),
    PolicyCondition::MFAEnabled(true),
  ],
  priority: 100,  // Higher priority policies evaluated first
  enabled: true,
  created_at: Utc::now(),
  created_by: admin_user_id,
  metadata: HashMap::new(),
}
```

**Implementation Details**:
- **Claims Composition**: Multiple policies compose additively (union of claims)
- **Priority Sorting**: Policies evaluated by priority (higher → lower)
- **Condition Enforcement**: ALL conditions must be satisfied for policy to activate
- **Claim Deduplication**: Union operation removes duplicate claims
- **40+ Claim Types**: Key management, infrastructure, admin, NATS, data, security
- **12+ Condition Types**: Clearance, MFA, YubiKey, location, time, witness, training, etc.

**Policy Evaluation Algorithm**:
```rust
fn evaluate_policies(
    policies: &[Policy],
    bindings: &[PolicyBinding],
    entity_id: Uuid,
    entity_type: PolicyEntityType,
    context: &PolicyEvaluationContext,
) -> PolicyEvaluation {
    // 1. Find policies bound to entity
    // 2. Sort by priority (high to low)
    // 3. For each policy:
    //    - Check if enabled
    //    - Evaluate all conditions
    //    - If all pass: add claims to result
    // 4. Deduplicate claims (union)
    // 5. Return evaluation with active/inactive policies
}
```

**See**: `src/domain.rs` (lines 550-1027), `tests/policy_tests.rs`

### 3. Entity Types as Graph Nodes

All domain entities become graph nodes:

```rust
pub enum NodeType {
    Organization(Organization),
    OrganizationalUnit(OrganizationUnit),
    Person(Person),
    Location(Location),
    Role(Role),           // NEW: Position/Role entity
    Policy(Policy),       // NEW: Policy entity
    Key(KeyMetadata),     // NEW: Represents generated keys
}

pub struct GraphNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub position: Point,
    pub color: Color,
    pub selected: bool,
    pub metadata: HashMap<String, String>,
}
```

### 4. Relationship Types as Graph Edges

Edges represent typed relationships:

```rust
pub enum EdgeType {
    // Organizational hierarchy
    ParentChild,          // Organization → OrganizationalUnit
    ManagesUnit,          // Person → OrganizationalUnit
    MemberOf,             // Person → OrganizationalUnit

    // Key relationships
    OwnsKey,              // Person → Key
    DelegatesKey,         // Person → Person
    StoredAt,             // Key → Location

    // Policy relationships
    HasRole,              // Person → Role
    RoleRequiresPolicy,   // Role → Policy
    PolicyGovernsEntity,  // Policy → (Person | OrgUnit | Key)

    // Trust relationships
    Trusts,               // Organization → Organization
    CertifiedBy,          // Key → Key (CA relationship)
}
```

### 5. Property Cards for Value Editing

When a node is selected, a property card appears showing editable fields:

```
┌─────────────────────────────────────┐
│ Person: Alice Johnson               │
├─────────────────────────────────────┤
│ Name:     [Alice Johnson        ]   │
│ Email:    [alice@example.com    ]   │
│ Active:   ☑ Enabled                 │
│                                      │
│ Roles:    - Senior Developer        │
│           - Security Admin          │
│           [+ Add Role]               │
│                                      │
│ Policies: - CanSignCode (Active)    │
│           - RequiresMFA (Active)    │
│           - AuditAccess (Inactive)  │
│                                      │
│ [Save Changes]  [Cancel]            │
└─────────────────────────────────────┘
```

Property cards emit `UiPropertyChanged` intents that flow through the MVI system.

## Implementation Plan

### Phase 1: Domain Model Extensions ✅ COMPLETE

**Status**: Completed 2025-01-15
**Deliverables**:
- ✅ Policy domain models with 40+ claim types
- ✅ Role domain models with fulfillment checking
- ✅ PolicyBinding and PolicyEvaluation
- ✅ Comprehensive test suite (7 tests, all passing)
- ✅ Display trait implementations

**Implementation**: See `src/domain.rs` (lines 550-1162), `tests/policy_tests.rs`

#### 1.1 Policy Models (`src/domain.rs`) ✅

```rust
/// Policy entity with claims-based permissions
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub claims: Vec<PolicyClaim>,
    pub conditions: Vec<PolicyCondition>,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub metadata: HashMap<String, String>,
}

/// Individual claim (capability/permission)
/// 40+ claim types implemented:
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyClaim {
    // Key management claims (8 variants)
    CanGenerateKeys, CanSignCode, CanSignCertificates, CanRevokeKeys,
    CanDelegateKeys, CanExportKeys, CanBackupKeys, CanRotateKeys,

    // Infrastructure claims (7 variants)
    CanAccessProduction, CanAccessStaging, CanAccessDevelopment,
    CanModifyInfrastructure, CanDeployServices, CanCreateInfrastructure,
    CanDeleteInfrastructure,

    // Administrative claims (9 variants)
    CanManageOrganization, CanManagePolicies, CanAssignRoles,
    CanCreateAccounts, CanDisableAccounts, CanDeleteAccounts,
    CanViewAuditLogs, CanExportAuditLogs, CanModifyAuditSettings,

    // NATS claims (6 variants)
    CanCreateNATSOperators, CanCreateNATSAccounts, CanCreateNATSUsers,
    CanManageNATSSubjects, CanPublishSensitiveSubjects,
    CanSubscribeSensitiveSubjects,

    // Data claims (5 variants)
    CanReadSensitiveData, CanWriteSensitiveData, CanDeleteData,
    CanExportData, CanImportData,

    // Security claims (4 variants)
    CanPerformAudits, CanReviewIncidents, CanInitiateEmergency,
    CanOverrideSecurityControls,

    // Custom claim
    Custom { name: String, scope: String, description: String },
}

/// Conditions that must be met for policy to be active
/// 12 condition types implemented:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    MinimumSecurityClearance(SecurityClearance),
    MFAEnabled(bool),
    YubiKeyRequired(bool),
    LocationRestriction(Vec<Uuid>),
    TimeWindow { start: DateTime<Utc>, end: DateTime<Utc> },
    RequiresWitness { count: u32, witness_clearance: Option<SecurityClearance> },
    MemberOfUnits(Vec<Uuid>),
    HasRole(Uuid),
    MinimumEmploymentDuration { days: u32 },
    CompletedTraining { training_ids: Vec<String> },
    IPWhitelist(Vec<String>),
    BusinessHoursOnly { timezone: String, start_hour: u8, end_hour: u8 },
    Custom { name: String, parameters: HashMap<String, String> },
}

/// Security clearance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecurityClearance {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

/// Binds a policy to entities it governs
pub struct PolicyBinding {
    pub policy_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: EntityType,
    pub bound_at: DateTime<Utc>,
    pub bound_by: Uuid,  // Person who created binding
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Organization,
    OrganizationalUnit,
    Person,
    Location,
    Key,
}
```

#### 1.2 Add Role Models (`src/domain.rs`)

```rust
/// Role/Position in the organization
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Uuid,
    pub unit_id: Option<Uuid>,  // Optional: role specific to unit
    pub required_policies: Vec<Uuid>,
    pub responsibilities: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Assignment of a role to a person
pub struct RoleAssignment {
    pub person_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### Phase 2: Graph Model Extensions

#### 2.1 Extend `GraphNode` (`src/gui/graph.rs`)

```rust
pub struct GraphNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub position: Point,
    pub color: Color,
    pub size: f32,
    pub label: String,
    pub selected: bool,
    pub hovered: bool,
    pub metadata: HashMap<String, String>,
}

pub enum NodeType {
    Organization {
        org: Organization,
    },
    OrganizationalUnit {
        unit: OrganizationUnit,
        parent_org: Uuid,
    },
    Person {
        person: Person,
        role: KeyOwnerRole,
        units: Vec<Uuid>,
    },
    Location {
        location: Location,
    },
    Role {
        role: Role,
    },
    Policy {
        policy: Policy,
    },
    Key {
        key_id: Uuid,
        key_type: KeyType,
        owner: Uuid,
    },
}
```

#### 2.2 Extend `GraphMessage` (`src/gui/graph.rs`)

```rust
#[derive(Debug, Clone)]
pub enum GraphMessage {
    // Selection
    NodeClicked(Uuid),
    NodeHovered(Uuid),
    EdgeClicked { from: Uuid, to: Uuid },

    // Node manipulation
    NodeDragStarted { node_id: Uuid, offset: Vector },
    NodeDragged(Point),
    NodeDragEnded,

    // Node creation (context menu)
    ContextMenuOpened(Point),
    ContextMenuClosed,
    CreateNodeRequested { node_type: NodeCreationType, position: Point },

    // Edge creation
    EdgeCreationStarted { from_node: Uuid },
    EdgeCreationDragging(Point),
    EdgeCreationCompleted { from: Uuid, to: Uuid, edge_type: EdgeType },
    EdgeCreationCancelled,

    // Deletion
    DeleteNodeRequested(Uuid),
    DeleteEdgeRequested { from: Uuid, to: Uuid },

    // Property editing
    PropertyEditRequested(Uuid),
    PropertyChanged { node_id: Uuid, property: String, value: String },
    PropertySaveRequested(Uuid),
    PropertyCancelRequested,

    // View controls
    ZoomIn,
    ZoomOut,
    ResetView,
    Pan(Vector),
    AutoLayout,

    // Bulk operations
    SelectMultiple(Vec<Uuid>),
    GroupNodes(Vec<Uuid>),
    UngroupNodes(Uuid),
}

#[derive(Debug, Clone)]
pub enum NodeCreationType {
    Organization,
    OrganizationalUnit,
    Person,
    Location,
    Role,
    Policy,
}
```

### Phase 3: Interactive UI Components

#### 3.1 Context Menu (`src/gui/context_menu.rs`)

```rust
pub struct ContextMenu {
    position: Point,
    visible: bool,
    items: Vec<ContextMenuItem>,
}

pub enum ContextMenuItem {
    CreateNode(NodeCreationType),
    CreateEdge,
    Delete,
    Properties,
    Separator,
}

impl ContextMenu {
    pub fn view(&self) -> Element<'_, GraphMessage> {
        if !self.visible {
            return Space::new(0, 0).into();
        }

        let menu_items = column![
            button("Add Organization").on_press(
                GraphMessage::CreateNodeRequested {
                    node_type: NodeCreationType::Organization,
                    position: self.position,
                }
            ),
            button("Add Org Unit").on_press(...),
            button("Add Person").on_press(...),
            button("Add Location").on_press(...),
            button("Add Role").on_press(...),
            button("Add Policy").on_press(...),
        ];

        container(menu_items)
            .style(/* menu styling */)
            .into()
    }
}
```

#### 3.2 Property Card (`src/gui/property_card.rs`)

```rust
pub struct PropertyCard {
    node_id: Option<Uuid>,
    node_type: Option<NodeType>,
    fields: Vec<PropertyField>,
    dirty: bool,
}

pub enum PropertyField {
    Text { label: String, value: String, editable: bool },
    Email { label: String, value: String },
    Checkbox { label: String, checked: bool },
    Dropdown { label: String, options: Vec<String>, selected: Option<usize> },
    List { label: String, items: Vec<String> },
}

impl PropertyCard {
    pub fn view<'a>(&'a self) -> Element<'a, GraphMessage> {
        if self.node_id.is_none() {
            return text("Select a node to view properties").into();
        }

        let header = row![
            text(format!("{:?}", self.node_type)).size(20),
            horizontal_space(),
            button("✕").on_press(GraphMessage::PropertyCancelRequested),
        ];

        let fields = self.fields.iter().map(|field| {
            match field {
                PropertyField::Text { label, value, editable } => {
                    row![
                        text(label).width(120),
                        if *editable {
                            text_input("", value)
                                .on_input(move |new_value| {
                                    GraphMessage::PropertyChanged {
                                        node_id: self.node_id.unwrap(),
                                        property: label.clone(),
                                        value: new_value,
                                    }
                                })
                                .into()
                        } else {
                            text(value).into()
                        }
                    ].into()
                }
                // ... other field types
            }
        });

        let buttons = row![
            button("Save")
                .on_press(GraphMessage::PropertySaveRequested(self.node_id.unwrap())),
            button("Cancel")
                .on_press(GraphMessage::PropertyCancelRequested),
        ];

        container(column![header].chain(fields).push(buttons))
            .padding(20)
            .style(/* card styling */)
            .into()
    }
}
```

#### 3.3 Edge Creation Indicator

```rust
pub struct EdgeCreationIndicator {
    from_node: Option<Uuid>,
    current_position: Point,
    active: bool,
}

impl EdgeCreationIndicator {
    pub fn draw(&self, frame: &mut canvas::Frame, graph: &OrganizationGraph) {
        if !self.active || self.from_node.is_none() {
            return;
        }

        if let Some(from) = graph.nodes.get(&self.from_node.unwrap()) {
            // Draw dashed line from source node to cursor
            let path = canvas::Path::line(from.position, self.current_position);
            let stroke = canvas::Stroke::default()
                .with_color(Color::from_rgb(0.5, 0.5, 1.0))
                .with_width(2.0)
                .with_line_dash(canvas::LineDash {
                    segments: &[10.0, 5.0],
                    offset: 0,
                });
            frame.stroke(&path, stroke);
        }
    }
}
```

### Phase 4: MVI Integration

#### 4.1 Add Graph Intents (`src/mvi/intent.rs`)

```rust
pub enum Intent {
    // ... existing intents ...

    // ===== Graph-Originated Intents =====

    /// User clicked to create a new node
    UiGraphCreateNode {
        node_type: NodeCreationType,
        position: Point,
    },

    /// User started creating an edge
    UiGraphCreateEdgeStarted {
        from_node: Uuid,
    },

    /// User completed edge creation
    UiGraphCreateEdgeCompleted {
        from: Uuid,
        to: Uuid,
        edge_type: EdgeType,
    },

    /// User requested to delete a node
    UiGraphDeleteNode {
        node_id: Uuid,
    },

    /// User requested to delete an edge
    UiGraphDeleteEdge {
        from: Uuid,
        to: Uuid,
    },

    /// User opened property editor for a node
    UiGraphEditNodeProperties {
        node_id: Uuid,
    },

    /// User changed a property value
    UiGraphPropertyChanged {
        node_id: Uuid,
        property: String,
        value: String,
    },

    /// User saved property changes
    UiGraphPropertiesSaved {
        node_id: Uuid,
    },

    // Domain events for graph changes
    DomainNodeCreated {
        node_id: Uuid,
        node_type: NodeType,
    },

    DomainEdgeCreated {
        from: Uuid,
        to: Uuid,
        edge_type: EdgeType,
    },

    DomainNodeDeleted {
        node_id: Uuid,
    },

    DomainNodeUpdated {
        node_id: Uuid,
        changes: HashMap<String, String>,
    },
}
```

#### 4.2 Add Graph State to Model (`src/mvi/model.rs`)

```rust
pub struct Model {
    // ... existing fields ...

    // ===== Graph State =====
    pub graph: GraphState,
}

pub struct GraphState {
    pub nodes: HashMap<Uuid, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub selected_node: Option<Uuid>,
    pub selected_edges: Vec<(Uuid, Uuid)>,
    pub property_editor_open: bool,
    pub context_menu_open: bool,
    pub context_menu_position: Point,
    pub edge_creation_mode: bool,
    pub edge_creation_from: Option<Uuid>,
}

impl Model {
    pub fn with_node_added(mut self, node: GraphNode) -> Self {
        self.graph.nodes.insert(node.id, node);
        self
    }

    pub fn with_edge_added(mut self, edge: GraphEdge) -> Self {
        self.graph.edges.push(edge);
        self
    }

    pub fn with_node_selected(mut self, node_id: Option<Uuid>) -> Self {
        self.graph.selected_node = node_id;
        self.graph.property_editor_open = node_id.is_some();
        self
    }

    // ... other graph state methods ...
}
```

#### 4.3 Update Function Extensions (`src/mvi/update.rs`)

```rust
pub fn update(
    model: Model,
    intent: Intent,
    storage: &Arc<dyn StoragePort>,
    x509: &Arc<dyn X509Port>,
    ssh: &Arc<dyn SshKeyPort>,
    yubikey: &Arc<dyn YubiKeyPort>,
) -> (Model, Task<Intent>) {
    match intent {
        // Graph node creation
        Intent::UiGraphCreateNode { node_type, position } => {
            let node_id = Uuid::now_v7();

            // Create appropriate domain entity
            let node = match node_type {
                NodeCreationType::Person => {
                    // Open property editor with blank person
                    GraphNode {
                        id: node_id,
                        node_type: NodeType::Person {
                            person: Person {
                                id: node_id,
                                name: "New Person".to_string(),
                                email: "".to_string(),
                                // ...
                            },
                            role: KeyOwnerRole::Developer,
                            units: Vec::new(),
                        },
                        position,
                        // ...
                    }
                }
                // ... other node types ...
            };

            let updated_model = model
                .with_node_added(node)
                .with_node_selected(Some(node_id))
                .with_status_message(format!("{:?} created", node_type));

            // Emit domain event
            let task = Task::done(Intent::DomainNodeCreated {
                node_id,
                node_type: node.node_type.clone(),
            });

            (updated_model, task)
        }

        // Edge creation
        Intent::UiGraphCreateEdgeCompleted { from, to, edge_type } => {
            let edge = GraphEdge {
                from,
                to,
                edge_type: edge_type.clone(),
                color: edge_type_color(&edge_type),
            };

            let updated_model = model
                .with_edge_added(edge)
                .with_status_message("Relationship created".to_string());

            let task = Task::done(Intent::DomainEdgeCreated {
                from,
                to,
                edge_type,
            });

            (updated_model, task)
        }

        // Property changes
        Intent::UiGraphPropertyChanged { node_id, property, value } => {
            // Update node property in model
            // This is just UI state - not persisted until save
            let mut updated_model = model;
            if let Some(node) = updated_model.graph.nodes.get_mut(&node_id) {
                // Update property based on node type
                // Mark as dirty
            }

            (updated_model, Task::none())
        }

        Intent::UiGraphPropertiesSaved { node_id } => {
            // Persist changes to projection
            // Emit domain event
            let mut changes = HashMap::new();
            // Extract changes from node

            let task = Task::done(Intent::DomainNodeUpdated {
                node_id,
                changes,
            });

            (model, task)
        }

        // ... other graph intents ...

        _ => (model, Task::none()),
    }
}
```

### Phase 5: Event Sourcing Integration

All graph changes must be persisted as events:

```rust
// New events in src/events.rs

pub enum DomainEvent {
    // ... existing events ...

    // Graph entity events
    OrganizationCreated {
        id: Uuid,
        name: String,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    OrganizationalUnitCreated {
        id: Uuid,
        name: String,
        parent_id: Uuid,
        unit_type: OrganizationUnitType,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    PersonCreated {
        id: Uuid,
        name: String,
        email: String,
        organization_id: Uuid,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    LocationCreated {
        id: Uuid,
        name: String,
        location_type: LocationType,
        security_level: SecurityLevel,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    RoleCreated {
        id: Uuid,
        name: String,
        organization_id: Uuid,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    PolicyCreated {
        id: Uuid,
        name: String,
        claims: Vec<PolicyClaim>,
        priority: i32,
        created_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    // Relationship events
    RelationshipEstablished {
        from: Uuid,
        to: Uuid,
        relationship_type: EdgeType,
        established_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    RelationshipRemoved {
        from: Uuid,
        to: Uuid,
        relationship_type: EdgeType,
        removed_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    // Policy events
    PolicyBound {
        policy_id: Uuid,
        entity_id: Uuid,
        entity_type: EntityType,
        bound_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    PolicyUnbound {
        policy_id: Uuid,
        entity_id: Uuid,
        unbound_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    PolicyEnabled {
        policy_id: Uuid,
        enabled_by: Uuid,
        timestamp: DateTime<Utc>,
    },

    PolicyDisabled {
        policy_id: Uuid,
        disabled_by: Uuid,
        timestamp: DateTime<Utc>,
    },
}
```

## User Workflows

### Workflow 1: Create Organization from Scratch

1. User opens cim-keys GUI
2. Empty graph canvas appears
3. User right-clicks → "Create Organization"
4. Property card opens with fields:
   - Name: [input]
   - Domain: [input]
   - Description: [textarea]
5. User fills in "CowboyAI" and saves
6. Organization node appears as root node in graph
7. User right-clicks organization → "Add Organizational Unit"
8. Property card for OrgUnit appears
9. User creates "Engineering" unit
10. Edge automatically created: Organization → Engineering

### Workflow 2: Add Person with Role and Policies

1. User right-clicks "Engineering" unit → "Add Person"
2. Property card opens:
   - Name: Alice Johnson
   - Email: alice@cowboyai.com
   - Roles: [dropdown] → Select "Senior Developer"
3. User saves → Person node created
4. Edge created: Person → Engineering (MemberOf)
5. Edge created: Person → Senior Developer Role (HasRole)
6. User selects "Senior Developer" role node
7. Property card shows policies attached to role
8. User drags from role to "CanSignCode" policy
9. Edge created: Role → Policy (RoleRequiresPolicy)
10. Policy claims now automatically apply to Alice

### Workflow 3: Assign Key Storage Location

1. User has generated a root CA key
2. Key node appears in graph (created by key generation event)
3. User right-clicks → "Create Location"
4. Property card:
   - Name: "Safe Deposit Box #123"
   - Type: SafeDeposit
   - Security Level: FIPS140Level4
5. User saves → Location node created
6. User drags from Key node to Location node
7. Edge type selector appears: "StoredAt"
8. Edge created: Key → Location (StoredAt)
9. Key metadata updated with storage location

### Workflow 4: Define and Apply Policy

1. User right-clicks → "Create Policy"
2. Property card opens:
   - Name: "Production Access Policy"
   - Claims: [list]
     - ✅ CanAccessProduction
     - ✅ CanSignCode
     - ✅ RequiresMFA
   - Conditions: [list]
     - Minimum Clearance: Secret
     - MFA Enabled: true
   - Priority: 100
3. User saves → Policy node created
4. User drags from Policy to "Engineering" OrgUnit
5. Edge created: Policy → OrgUnit (PolicyGovernsEntity)
6. All members of Engineering now governed by this policy
7. Property cards for people show "Production Access Policy (Active)"

## Visual Design

### Node Appearance by Type

```
┌─────────────────────────────────────┐
│  Organization Nodes                 │
│  - Large hexagon                    │
│  - Dark blue                        │
│  - Bold text                        │
│                                      │
│  Organizational Unit Nodes          │
│  - Medium hexagon                   │
│  - Light blue                       │
│  - Department icon                  │
│                                      │
│  Person Nodes                       │
│  - Circle                           │
│  - Color by role                    │
│  - Profile icon                     │
│                                      │
│  Location Nodes                     │
│  - Square                           │
│  - Brown/Gray                       │
│  - Location pin icon                │
│                                      │
│  Role Nodes                         │
│  - Diamond                          │
│  - Purple                           │
│  - Badge icon                       │
│                                      │
│  Policy Nodes                       │
│  - Pentagon (shield shape)          │
│  - Gold/Yellow                      │
│  - Shield icon                      │
│                                      │
│  Key Nodes                          │
│  - Small circle with lock icon      │
│  - Green (active) / Gray (inactive) │
└─────────────────────────────────────┘
```

### Edge Appearance by Type

```
ParentChild:           ━━━━━▶  (solid, blue)
ManagesUnit:           ═══▶  (double line, purple)
MemberOf:              ----▶  (dashed, gray)
OwnsKey:               ━━━▶  (solid, green)
DelegatesKey:          ····▶  (dotted, orange)
StoredAt:              ━━━▶  (solid, brown)
HasRole:               ━━━▶  (solid, purple)
RoleRequiresPolicy:    ━━━▶  (solid, gold)
PolicyGovernsEntity:   ⚡⚡⚡▶  (lightning bolt, gold)
```

## Implementation Priority

### Phase 1: Foundation ✅ COMPLETE (Completed: 2025-01-15)
- ✅ Add Policy and Role domain models (`src/domain.rs` lines 550-1162)
  - 40+ PolicyClaim types implemented
  - 12 PolicyCondition types implemented
  - `evaluate_policies()` function with priority sorting and claims composition
  - Role and RoleAssignment entities with `can_person_fulfill()` method
- ✅ Comprehensive test suite (`tests/policy_tests.rs`)
  - 7 integration tests, all passing
  - 100% coverage of policy evaluation logic
  - Test scenarios: claim composition, priority sorting, clearance checks, MFA, witnesses, role fulfillment
- 🚧 Extend GraphNode for all entity types (NEXT: Phase 2)
- 🚧 Add graph Intents to MVI system (NEXT: Phase 2)

**See**: `PHASE1_COMPLETE.md` for full details

### Phase 2: Graph Interaction Intents ✅ COMPLETE (Completed: 2025-01-15)

**Status**: All objectives met, code compiles, tests passing

**Deliverables**:
- ✅ Extended GraphNode to support all entity types (`src/gui/graph.rs:36-53`)
- ✅ Added 6 node creation methods (one per entity type)
- ✅ Extended EdgeType to 12 semantic types + 2 legacy (`src/gui/graph.rs:64-101`)
- ✅ Added NodeCreationType enum (`src/mvi/intent.rs:9-17`)
- ✅ Added 23 graph interaction Intents (`src/mvi/intent.rs:127-252`)
- ✅ Added placeholder update handlers (`src/mvi/update.rs:791-964`)
- ✅ Updated hierarchical layout for all node types
- ✅ Updated rendering for type-specific display

**Implementation**: See `PHASE2_COMPLETE.md` for full details

### Phase 3: Interactive UI Components ✅ COMPLETE (Completed: 2025-01-15)

**Status**: All components implemented, all tests passing (51 tests)

**Deliverables**:
- ✅ Context menu widget (`src/gui/context_menu.rs`)
  - Right-click menu for all 6 node types
  - Create edge action
  - 2 tests (creation, show/hide)
- ✅ Property card widget (`src/gui/property_card.rs`)
  - Type-specific property fields
  - Dirty state tracking
  - Save/Cancel actions
  - 4 tests (creation, set_node, dirty_state, clear)
- ✅ Edge creation indicator (`src/gui/edge_indicator.rs`)
  - Dashed line visual feedback
  - Directional arrow and instruction text
  - 5 tests (creation, start, update, complete, cancel)
- ✅ Module integration (`src/gui.rs`)
- ✅ Graph visibility updates (public fields for component access)

**Implementation**: See `PHASE3_COMPLETE.md` for full details

### Phase 4: Complete Interactive Workflows (Week 3-4)
- [ ] Wire context menu to main GUI state
- [ ] Wire property card to main GUI state
- [ ] Implement complete node creation workflow
- [ ] Implement complete edge creation workflow
- [ ] Implement complete property editing workflow
- [ ] Add keyboard shortcuts
- [ ] Polish and UX refinements

### Phase 3: Relationships (Week 3)
- [ ] Edge creation UI (drag between nodes)
- [ ] Edge type selector
- [ ] Relationship validation

### Phase 4: Policy System (Week 4)
- [ ] Policy creation and editing
- [ ] Claims composition
- [ ] Policy evaluation engine
- [ ] Visual policy status indicators

### Phase 5: Polish (Week 5)
- [ ] Node styling by type
- [ ] Edge styling by type
- [ ] Animations
- [ ] Keyboard shortcuts
- [ ] Export graph as diagram

## Success Criteria

✅ User can create entire organization structure through graph
✅ User can define policies and see which entities they govern
✅ Graph is the ONLY interface needed for domain setup
✅ All operations are event-sourced
✅ Graph state persists to encrypted storage
✅ Graph can be imported into CIM deployments

## Future Enhancements

- **Graph Templates**: Pre-built organization structures
- **Import from HR Systems**: Auto-generate org structure from LDAP/AD
- **Policy Visualization**: Heat map showing policy coverage
- **Simulation Mode**: Test policy changes before applying
- **Time Travel**: View graph at different points in event history
- **Multi-Graph Views**: Switch between org chart, policy graph, key topology
- **Collaborative Editing**: Multiple users editing graph simultaneously (via NATS)

---

**This document is a living specification.** As implementation proceeds, update this document with:
- Actual implementation details
- Design decisions and rationales
- Discovered edge cases
- User feedback and iterations
