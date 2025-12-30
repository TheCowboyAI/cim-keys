# CIM Keys GUI Visual Representation

## 🖼️ GUI Layout (iced 0.13 with Canvas)

```
┌─────────────────────────────────────────────────────────────────────┐
│ 🔐 CIM Keys - Offline Key Management System                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────── Tab Bar ─────────────────────────┐  │
│  │  [Welcome]  [Organization]  [Keys]  [Export]                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ╔══════════════════ Organization Graph View ═══════════════════╗  │
│  ║                                                               ║  │
│  ║  ┌──────────────────────────────────────────────────────┐    ║  │
│  ║  │ [Zoom In] [Zoom Out] [Reset] [Auto Layout]           │    ║  │
│  ║  └──────────────────────────────────────────────────────┘    ║  │
│  ║                                                               ║  │
│  ║  ┌─────────────── Canvas (500px height) ─────────────────┐   ║  │
│  ║  │                                                        │   ║  │
│  ║  │           🔴 Alice (Root Authority)                   │   ║  │
│  ║  │              ╱        │         ╲                     │   ║  │
│  ║  │             ╱         │          ╲                    │   ║  │
│  ║  │            ↓          ↓           ↓                   │   ║  │
│  ║  │      🔵 Bob    🔵 Charlie   🟢 SvcAcct               │   ║  │
│  ║  │   (Sec Admin)  (Sec Admin)  (Service)                │   ║  │
│  ║  │         │           │                                 │   ║  │
│  ║  │         ↓           ↓                                 │   ║  │
│  ║  │    🟦 Dave     🟦 Eve                                 │   ║  │
│  ║  │   (Developer) (Developer)                             │   ║  │
│  ║  │                                                        │   ║  │
│  ║  └────────────────────────────────────────────────────────┘  ║  │
│  ║                                                               ║  │
│  ║  Selected Node Details:                                      ║  │
│  ║  ┌──────────────────────────────────────────────────────┐   ║  │
│  ║  │ Name: Alice Johnson                                  │   ║  │
│  ║  │ Email: alice@cowboyai.com                            │   ║  │
│  ║  │ Role: Root Authority                                 │   ║  │
│  ║  │ Keys Owned: 3                                        │   ║  │
│  ║  └──────────────────────────────────────────────────────┘   ║  │
│  ╚═══════════════════════════════════════════════════════════╝  │
│                                                                     │
│  Status: Graph layout updated                                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## 🎨 Visual Elements

### Node Colors (by KeyOwnerRole):
- 🔴 **Red** - Root Authority
- 🔵 **Blue** - Security Admin
- 🟦 **Light Blue** - Developer
- 🟢 **Green** - Service Account
- 🟠 **Orange** - Backup Holder
- 🟡 **Yellow** - Auditor

### Edge Types:
- **Blue lines** → Hierarchical relationships
- **Green lines** → Key delegations
- **Orange lines** → Trust relationships
- **Arrows** → Direction of relationship/delegation

### Interactive Features:
1. **Click on nodes** - Selects node and shows details
2. **Selected node** - Yellow ring highlight around it
3. **Mouse wheel** - Zoom in/out on the graph
4. **Zoom buttons** - Alternative zoom controls
5. **Auto Layout** - Arranges nodes in circular pattern
6. **Pan** - Drag to move the viewport (when implemented)

## 📱 Screen Flow

### 1. Welcome Tab
```
Welcome to CIM Keys!

Organization: [CowboyAI___________]
Domain: [cowboyai.com_________]

[Load Existing Domain]  [Create New Domain]
```

### 2. Organization Tab (Graph View)
- Interactive canvas with nodes and edges
- Real-time visualization of org structure
- Click to select people
- Visual delegation paths

### 3. Keys Tab
```
Generate Keys for Organization

[Generate Root CA]
[Generate SSH Keys for All]
[Provision YubiKeys]

Progress: [████████░░░░░░░] 12 of 20 keys generated
```

### 4. Export Tab
```
Export Domain Configuration

Output Directory: /tmp/test-output
☑ Include public keys
☑ Include certificates
☑ Generate NATS configuration
☐ Include private keys (requires password)

[Export to Encrypted SD Card]
```

## 🔧 Technical Implementation

The GUI uses:
- **iced 0.13** - Native Rust GUI framework
- **Canvas widget** - For graph visualization
- **canvas::Program trait** - Custom rendering logic
- **Event-driven updates** - Commands → Events → Projections
- **Async runtime** - Tokio for background operations

### Canvas Rendering Code:
```rust
impl canvas::Program<GraphMessage> for OrganizationGraph {
    fn draw() {
        // Draw edges with arrows
        // Draw nodes as colored circles
        // Draw text labels
        // Apply zoom/pan transformations
    }

    fn update() {
        // Handle mouse clicks
        // Handle scroll events
        // Return GraphMessage events
    }
}
```

The graph provides a powerful visual way to understand and manage the organizational structure and key delegation relationships in the CIM system.