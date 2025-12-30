# Graph-First GUI User Guide

**Version:** 0.8.0
**Date:** 2025-11-21
**Status:** Production Ready

## Overview

The cim-keys Graph-First GUI is a pure FRP (Functional Reactive Programming) application for managing domain objects, relationships, and cryptographic infrastructure through an interactive graph interface.

### Key Features

- **10 Domain Types**: Person, Organization, Location, ServiceAccount, NATS entities, Certificates, YubiKeys
- **5 View Perspectives**: All, Organization, NATS, PKI, YubiKey
- **Event-Sourced**: Every operation emits immutable events with full audit trail
- **Graph-Based**: Nodes (entities) and Edges (relationships)
- **Zero Domain Coupling**: Generic architecture works with any aggregate type

---

## Quick Start

### Launch the GUI

```bash
# Native application
cargo run --bin cim-keys-gui --features gui --release -- ./output

# With custom config
cargo run --bin cim-keys-gui --features gui -- ./my-output --config config.toml
```

### WASM (Browser)

```bash
./build-wasm.sh
./serve.py
# Open http://localhost:8000
```

---

## User Interface Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ CIM Keys - Graph-First Architecture                             │
├─────────────────────────────────────────────────────────────────┤
│ View Switcher:                                                  │
│ [● All] [○ Organization] [○ NATS] [○ PKI] [○ YubiKey]          │
├─────────────────────────────────────────────────────────────────┤
│ Left Panel     │ Center Panel          │ Right Panel            │
│ (Actions)      │ (Property Editor)     │ (Relationships)        │
│                │                       │                        │
│ Domain:        │ Selected Node:        │ Relationships:         │
│ [+ Person]     │ Person (abc123...)    │ Alice --[reports_to]-> │
│ [+ Org]        │                       │ Bob                    │
│ [+ Location]   │ Properties:           │                        │
│                │ legal_name: Alice     │ Alice --[owns]->       │
│ NATS:          │ [Edit]                │ Key (xyz789...)        │
│ [+ Operator]   │                       │                        │
│ [+ Account]    │ active: true          │                        │
│ [+ User]       │ [Edit]                │                        │
│                │                       │                        │
│ Security:      │                       │                        │
│ [+ Cert]       │                       │                        │
│ [+ YubiKey]    │                       │                        │
│                │                       │                        │
│ File Ops:      │                       │                        │
│ [💾 Save]      │                       │                        │
│ [📂 Load]      │                       │                        │
│ [📋 Events]    │                       │                        │
│ [📤 Export]    │                       │                        │
└────────────────┴───────────────────────┴────────────────────────┘
```

---

## Complete Workflow Tutorial

### Example: Building an Organization with NATS Infrastructure

#### Step 1: Create Organization

1. Click **[+ Organization]**
2. Select the new "Organization (New Organization)" node
3. Click **[Edit]** next to "name" property
4. Change to "CowboyAI"
5. Click **[Save]**

**Event Emitted:**
```json
{
  "event_type": "DomainObjectCreated",
  "object_id": "01933e5f-a1b2-...",
  "aggregate_type": "Organization",
  "properties": { "name": "CowboyAI" }
}
```

#### Step 2: Add People

1. Click **[+ Person]** (creates Alice)
2. Edit `legal_name` → "Alice Smith"
3. Click **[+ Person]** (creates Bob)
4. Edit `legal_name` → "Bob Jones"

**Events:** 2× `DomainObjectCreated` + 2× `DomainObjectUpdated`

#### Step 3: Create Reporting Relationship

1. Select **Alice**
2. Click **[➕ Create Relationship]**
3. Select **Bob** (as target)
4. Click **[reports_to]** button

**Result:** Edge created: Alice --[reports_to]--> Bob

**Event Emitted:**
```json
{
  "event_type": "RelationshipEstablished",
  "source_id": "alice-uuid",
  "target_id": "bob-uuid",
  "relationship_type": "reports_to"
}
```

#### Step 4: Generate Cryptographic Key

1. Select **Alice**
2. Click **[🔑 Generate Key]**

**Result:**
- New Key node created
- Edge created: Alice --[owns_key]--> Key

**Events:** `DomainObjectCreated` (Key) + `RelationshipEstablished`

#### Step 5: Add NATS Infrastructure

1. Switch view: Click **[○ NATS]**
2. Click **[+ NATS Operator]**
3. Edit `name` → "CowboyAI Operator"
4. Click **[+ NATS Account]**
5. Edit `name` → "Engineering"
6. Create relationship: Operator --[contains]--> Account

**Events:** 2× `DomainObjectCreated` + 1× `RelationshipEstablished`

#### Step 6: Save & Export

1. Click **[💾 Save Graph]**
   - Saves to `./output/graph.json`
2. Click **[📋 Events (15)]**
   - View all events in console
3. Click **[📤 Export Events]**
   - Saves to `./output/events.json`

**Event Emitted:**
```json
{
  "event_type": "GraphSaved",
  "path": "./output/graph.json",
  "node_count": 7,
  "edge_count": 3
}
```

---

## View Perspectives

### 1. All Entities View

**Shows:** Everything
**Use Case:** Overview of entire domain

```
Nodes (All Entities)
  ► Person (Alice Smith)
    Person (Bob Jones)
    Organization (CowboyAI)
    Location (alice@example.com)
    NatsOperator (CowboyAI Operator)
    NatsAccount (Engineering)
    Key (ed25519-abc123)
```

### 2. Organization View

**Shows:** Person, Organization, Location, ServiceAccount
**Use Case:** Organizational structure and human resources

**Filtered:** NATS, Certificates, YubiKeys hidden

### 3. NATS View

**Shows:** NatsOperator, NatsAccount, NatsUser
**Use Case:** NATS infrastructure and messaging hierarchy

**Example:**
```
Nodes (NATS Infrastructure)
  NatsOperator (CowboyAI Operator)
  NatsAccount (Engineering)
  NatsAccount (Operations)
  NatsUser (alice.engineering)

Relationships:
  Operator --[contains]--> Engineering
  Operator --[contains]--> Operations
  Engineering --[contains]--> alice.engineering
```

### 4. PKI View

**Shows:** Certificate, Key
**Use Case:** Certificate authority hierarchy and trust chains

**Example:**
```
Nodes (PKI / Certificates)
  Certificate (Root CA)
  Certificate (Intermediate CA)
  Certificate (Leaf - Alice)
  Key (root-key)

Relationships:
  Root CA --[signs]--> Intermediate CA
  Intermediate CA --[signs]--> Leaf
  Root CA --[trusts]--> Intermediate CA
```

### 5. YubiKey View

**Shows:** YubiKey
**Use Case:** Hardware security token provisioning

---

## Domain Types Reference

### Person

**Properties:**
- `legal_name` (String) - Full legal name
- `active` (Boolean) - Employment status

**Typical Relationships:**
- `reports_to` → Person
- `owns_key` → Key
- `uses` → YubiKey
- `located_at` → Location

**Example:**
```json
{
  "id": "01933e5f-...",
  "aggregate_type": "Person",
  "properties": {
    "legal_name": "Alice Smith",
    "active": true
  }
}
```

### Organization

**Properties:**
- `name` (String) - Organization name
- `display_name` (String) - Display name

**Typical Relationships:**
- `contains` → OrganizationUnit
- `operates` → NatsOperator

### Location

**Properties:**
- `location_type` (String) - email, address, etc.
- `address` (String) - Location identifier

**Typical Relationships:**
- Email locations for Person entities

### ServiceAccount

**Properties:**
- `name` (String) - Service account name
- `purpose` (String) - automated_service, etc.
- `active` (Boolean)

**Typical Relationships:**
- `uses` → NatsUser
- `owns_key` → Key

### NatsOperator

**Properties:**
- `name` (String) - Operator name
- `operator_id` (String) - NATS operator ID
- `created_at` (Timestamp)

**Typical Relationships:**
- `contains` → NatsAccount

### NatsAccount

**Properties:**
- `name` (String) - Account name
- `account_id` (String) - NATS account ID
- `created_at` (Timestamp)

**Typical Relationships:**
- `contains` → NatsUser

### NatsUser

**Properties:**
- `name` (String) - User name
- `user_id` (String) - NATS user ID
- `created_at` (Timestamp)

**Typical Relationships:**
- Owned by Person or ServiceAccount

### Certificate

**Properties:**
- `common_name` (String) - CN field
- `certificate_type` (String) - root, intermediate, leaf
- `valid_from` (Timestamp)
- `valid_until` (Timestamp)

**Typical Relationships:**
- `signs` → Certificate
- `trusts` → Certificate

### YubiKey

**Properties:**
- `serial_number` (String) - Hardware serial
- `firmware_version` (String) - e.g., "5.4.3"
- `status` (String) - unprovisioned, provisioned, sealed

**Typical Relationships:**
- `assigned_to` → Person
- `stores` → Key

### Key (Generated)

**Properties:**
- `key_type` (String) - ed25519, rsa, etc.
- `purpose` (String) - signing, encryption
- `generated_at` (Timestamp)

**Typical Relationships:**
- `owned_by` → Person

---

## Relationship Types

### Organizational

- **`reports_to`**: Hierarchical reporting (Person → Person)
- **`contains`**: Containment (Org → Unit, Operator → Account)
- **`located_at`**: Physical/logical location

### Ownership

- **`owns`**: Generic ownership
- **`owns_key`**: Cryptographic key ownership (Person → Key)

### Usage

- **`uses`**: Generic usage relationship
- **`assigned_to`**: Hardware assignment (YubiKey → Person)

### PKI

- **`signs`**: Certificate signing (CA → Certificate)
- **`trusts`**: Trust relationship (Root → Intermediate)

---

## Event System

### Event Types

#### 1. DomainObjectCreated

**Emitted:** On every node creation
**Contains:** object_id, aggregate_type, properties, timestamp

```json
{
  "event_type": "DomainObjectCreated",
  "object_id": "01933e5f-a1b2-c3d4-e5f6-0123456789ab",
  "aggregate_type": "Person",
  "properties": {
    "legal_name": "Alice Smith",
    "active": true
  },
  "timestamp": "2025-11-21T14:30:00.000Z"
}
```

#### 2. DomainObjectUpdated

**Emitted:** On property edit
**Contains:** object_id, property_key, old_value, new_value, timestamp

```json
{
  "event_type": "DomainObjectUpdated",
  "object_id": "01933e5f-...",
  "property_key": "legal_name",
  "old_value": "Alice",
  "new_value": "Alice Smith",
  "timestamp": "2025-11-21T14:31:00.000Z"
}
```

#### 3. DomainObjectDeleted

**Emitted:** On node deletion
**Contains:** object_id, aggregate_type, timestamp

```json
{
  "event_type": "DomainObjectDeleted",
  "object_id": "01933e5f-...",
  "aggregate_type": "Person",
  "timestamp": "2025-11-21T14:32:00.000Z"
}
```

#### 4. RelationshipEstablished

**Emitted:** On edge creation
**Contains:** source_id, target_id, relationship_type, timestamp

```json
{
  "event_type": "RelationshipEstablished",
  "source_id": "01933e5f-...",
  "target_id": "01933e60-...",
  "relationship_type": "reports_to",
  "timestamp": "2025-11-21T14:33:00.000Z"
}
```

#### 5. RelationshipRemoved

**Emitted:** On edge deletion (cascade from node deletion)
**Contains:** source_id, target_id, relationship_type, timestamp

#### 6. GraphSaved

**Emitted:** On save operation
**Contains:** path, node_count, edge_count, timestamp

```json
{
  "event_type": "GraphSaved",
  "path": "./output/graph.json",
  "node_count": 7,
  "edge_count": 3,
  "timestamp": "2025-11-21T14:34:00.000Z"
}
```

#### 7. GraphLoaded

**Emitted:** On load operation
**Contains:** path, node_count, edge_count, timestamp

### Event Log Usage

**View Events:**
1. Click **[📋 Events (N)]** to print all events to console
2. Events shown in chronological order
3. Includes full event details

**Export Events:**
1. Click **[📤 Export Events]**
2. Saves to `./output/events.json`
3. JSON array of all events
4. Use for audit trails, compliance, debugging

**Event Persistence:**
- Auto-saved to `events.json` on every event
- Append-only (immutable)
- Complete audit trail
- Event sourcing foundation

---

## Keyboard Shortcuts

*(Future enhancement)*

- **Ctrl+S**: Save graph
- **Ctrl+O**: Load graph
- **Delete**: Delete selected node
- **Ctrl+E**: Export events

---

## File Format

### graph.json

```json
{
  "nodes": {
    "01933e5f-a1b2-c3d4-e5f6-0123456789ab": {
      "id": "01933e5f-a1b2-c3d4-e5f6-0123456789ab",
      "aggregate_type": "Person",
      "properties": {
        "legal_name": "Alice Smith",
        "active": true
      },
      "version": 1
    }
  },
  "edges": [
    {
      "source_id": "01933e5f-...",
      "target_id": "01933e60-...",
      "relationship_type": "reports_to"
    }
  ]
}
```

### events.json

```json
[
  {
    "event_type": "DomainObjectCreated",
    "object_id": "01933e5f-...",
    "aggregate_type": "Person",
    "properties": { "legal_name": "Alice Smith" },
    "timestamp": "2025-11-21T14:30:00.000Z"
  },
  {
    "event_type": "RelationshipEstablished",
    "source_id": "01933e5f-...",
    "target_id": "01933e60-...",
    "relationship_type": "reports_to",
    "timestamp": "2025-11-21T14:33:00.000Z"
  }
]
```

---

## Best Practices

### 1. Organize by View

- Use **Organization** view for people/structure
- Use **NATS** view for messaging infrastructure
- Use **PKI** view for certificates
- Switch views to focus on specific concerns

### 2. Use Meaningful Names

- Edit default names immediately
- Use descriptive property values
- Follow naming conventions (e.g., "firstname.lastname" for NATS users)

### 3. Save Often

- Click **[💾 Save]** after major changes
- Events auto-save, but graph needs manual save
- Use **[📤 Export Events]** for audit trails

### 4. Leverage Event Log

- Review events to understand what changed
- Export events for compliance/audit
- Use events to debug issues

### 5. Delete Carefully

- Deletion cascades to all connected edges
- Check relationships before deleting
- Events provide deletion audit trail

---

## Troubleshooting

### Graph Won't Load

**Problem:** Clicking [📂 Load] does nothing
**Solution:** Ensure `graph.json` exists in output directory

### Events Not Saving

**Problem:** Events not in `events.json`
**Solution:** Check output directory permissions

### Node Not Visible After Creation

**Problem:** Created node doesn't appear in list
**Solution:** Check current view filter - switch to "All Entities"

### Can't Create Relationship

**Problem:** Clicking node after "Create Relationship" doesn't show options
**Solution:** Ensure target node is different from source node

---

## Architecture Benefits

### Pure FRP

- No mutable state
- All updates pure functions
- Immutable events
- Time-travel debugging possible

### Generic Design

- Works with ANY aggregate type
- Add new types = 0 UI changes
- No domain-specific code
- Infinite extensibility

### Event-Sourced

- Complete audit trail
- Compliance-ready
- Temporal queries
- State reconstruction

---

## Next Steps

After mastering the GUI:

1. **Integrate with cim-domain modules**
   - Import real Person/Organization from cim-domain-person
   - Use actual aggregate validation

2. **Connect to NATS**
   - Publish events to NATS subjects
   - Real-time collaboration
   - Distributed event sourcing

3. **Add custom aggregate types**
   - Define new domain objects
   - Generic rendering handles automatically

4. **Build workflows**
   - Chain operations
   - Automate common tasks
   - Template-based creation

---

**Version:** 0.8.0
**Last Updated:** 2025-11-21
**For Support:** See GitHub issues at github.com/thecowboyai/cim-keys
