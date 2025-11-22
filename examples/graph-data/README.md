# Example Graph Data Files

This directory contains example graph data files that you can load into the Graph-First GUI for testing and learning.

## Files

### 1. simple-graph.json
**Description:** A minimal example with basic organization structure.

**Contains:**
- 1 Organization (Example Corp)
- 2 People (Alice Smith, Bob Jones)
- 1 Location (alice@example.com)
- 2 Relationships (reports_to, located_at)

**Use Case:** Perfect for first-time users to understand the basic graph structure.

### 2. organization-example.json
**Description:** A complete organizational example with people, locations, and keys.

**Contains:**
- 1 Organization (CowboyAI)
- 3 People (Alice, Bob, Charlie) with roles
- 2 Locations (email addresses)
- 2 Keys (ed25519 signing keys)
- 6 Relationships (reports_to, located_at, owns_key)

**Use Case:** Demonstrates organizational hierarchy and key ownership patterns.

### 3. nats-infrastructure.json
**Description:** NATS messaging infrastructure example.

**Contains:**
- 1 NatsOperator (CowboyAI Operator)
- 2 NatsAccounts (Engineering, Operations)
- 3 NatsUsers (alice.engineering, bob.engineering, ops.service)
- 5 Relationships (all `contains` type showing hierarchy)

**Use Case:** View in "NATS Infrastructure" perspective to see messaging hierarchy.

### 4. pki-hierarchy.json
**Description:** Complete PKI certificate authority hierarchy.

**Contains:**
- 1 Root CA Certificate
- 2 Intermediate CA Certificates (Engineering, Operations)
- 2 Leaf Certificates (Alice, Bob)
- 1 Key (Root CA key)
- 1 YubiKey (hardware token)
- 8 Relationships (signs, trusts, uses, stores)

**Use Case:** View in "PKI / Certificates" perspective to see trust chains.

## How to Use

### Method 1: Load from GUI

1. Run the graph-first GUI:
   ```bash
   cargo run --bin cim-keys-gui --features gui -- ./output
   ```

2. Click **[📂 Load Graph]** button

3. Navigate to `examples/graph-data/` directory

4. Select a JSON file (e.g., `simple-graph.json`)

5. The graph will load and display nodes/edges

### Method 2: Command Line Copy

```bash
# Copy example to output directory
cp examples/graph-data/organization-example.json ./output/graph.json

# Run GUI (it will auto-load graph.json)
cargo run --bin cim-keys-gui --features gui -- ./output
```

### Method 3: Test Multiple Examples

```bash
# Load simple example
cargo run --bin cim-keys-gui --features gui -- ./output
# Click Load → simple-graph.json

# Switch view to "NATS Infrastructure"
# Click Load → nats-infrastructure.json

# Switch view to "PKI / Certificates"
# Click Load → pki-hierarchy.json
```

## Learning Path

**Recommended order for learning:**

1. **Start with `simple-graph.json`**
   - Understand basic nodes and edges
   - Practice selecting nodes
   - View properties
   - Try editing a property

2. **Load `organization-example.json`**
   - See organizational hierarchy
   - Understand `reports_to` relationships
   - See how keys are owned by people
   - Practice creating new relationships

3. **Load `nats-infrastructure.json`**
   - Switch to "NATS Infrastructure" view
   - See operator → account → user hierarchy
   - Understand `contains` relationships

4. **Load `pki-hierarchy.json`**
   - Switch to "PKI / Certificates" view
   - See root → intermediate → leaf chain
   - Understand `signs` and `trusts` relationships
   - See how YubiKey stores keys

## Creating Your Own Data

After loading examples, try creating your own:

1. **Start Fresh:**
   - Delete `output/graph.json`
   - Run GUI
   - Click **[+ Person]**, **[+ Organization]**, etc.
   - Build your own domain

2. **Modify Examples:**
   - Load an example
   - Add new nodes
   - Create new relationships
   - Click **[💾 Save Graph]**
   - Your changes are saved

3. **Export Events:**
   - Click **[📋 Events]** to view event log
   - Click **[📤 Export Events]** to save `events.json`
   - Review the complete audit trail

## File Format Reference

All files follow this structure:

```json
{
  "nodes": {
    "uuid-here": {
      "id": "uuid-here",
      "aggregate_type": "Person|Organization|Location|...",
      "properties": {
        "property_name": "value"
      },
      "version": 1
    }
  },
  "edges": [
    {
      "source_id": "source-uuid",
      "target_id": "target-uuid",
      "relationship_type": "reports_to|owns|uses|contains|signs|trusts"
    }
  ]
}
```

**UUID Format:** All IDs use UUID v7 (time-ordered)

**Aggregate Types:**
- Person, Organization, Location, ServiceAccount
- NatsOperator, NatsAccount, NatsUser
- Key, Certificate, YubiKey

**Relationship Types:**
- `reports_to` - Organizational hierarchy
- `owns`, `owns_key` - Ownership
- `uses` - Usage relationship
- `contains` - Containment (operator contains accounts)
- `signs` - Certificate signing
- `trusts` - Trust relationship
- `stores` - Hardware storage
- `located_at` - Location relationship

## Troubleshooting

**Problem:** "Failed to load graph"
- **Solution:** Check JSON syntax (use `jq` to validate)

**Problem:** Nodes not visible after loading
- **Solution:** Switch to "All Entities" view

**Problem:** Can't see relationships
- **Solution:** Check that both source and target nodes exist

**Problem:** Events not loading
- **Solution:** Events are separate from graph - use **[📤 Export Events]** to create `events.json`

## Next Steps

After exploring these examples:

1. Read [GRAPH_GUI_USER_GUIDE.md](../../docs/GRAPH_GUI_USER_GUIDE.md) for complete documentation
2. Read [MIGRATION_PLAN.md](../../docs/MIGRATION_PLAN.md) to understand the architecture
3. Start building your own domain models
4. Integrate with real NATS infrastructure
5. Export to encrypted SD cards for production use

## License

These example files are provided as-is for testing and learning purposes.
