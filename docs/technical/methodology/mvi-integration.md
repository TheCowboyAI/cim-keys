# MVI Integration Guide for cim-keys

## Current Status (~85% Complete)

The project has **two complete but separate UI systems**:

### 1. MVI System (`src/mvi/`)
- ✅ **Intent Layer** (~295 lines): All events defined
- ✅ **Model Layer** (~332 lines): Complete state management
- ✅ **Update Layer** (~766 lines): All handlers implemented
- ✅ **View Layer** (~529 lines): Complete display with certificate hierarchy

### 2. Legacy GUI (`src/gui.rs`)
- ✅ **Message enum**: Certificate management variants
- ✅ **CimKeysApp struct**: Input fields for certificates
- ✅ **Message handlers**: Update local state (TODOs for MVI)
- ✅ **view_keys()**: Hierarchical PKI workflow with inputs

## Integration Strategy

### Option A: Quick Integration (Recommended for MVP)

**Goal**: Wire `gui.rs` Messages to MVI Intents with minimal changes

**Steps**:
1. Add MVI model instance to `CimKeysApp` struct
2. Add port instances (storage, x509, ssh, yubikey)
3. In Message handlers, call `mvi::update()` and use returned Model
4. Display certificates from MVI Model instead of local state

**Changes Required** (~50-100 lines):
```rust
// In gui.rs CimKeysApp struct:
struct CimKeysApp {
    // ... existing fields ...

    // MVI integration
    mvi_model: Model,
    storage_port: Arc<dyn StoragePort>,
    x509_port: Arc<dyn X509Port>,
    ssh_port: Arc<dyn SshKeyPort>,
    yubikey_port: Arc<dyn YubiKeyPort>,
}

// In Message::GenerateIntermediateCA handler:
Message::GenerateIntermediateCA => {
    let intent = Intent::UiGenerateIntermediateCAClicked {
        name: self.intermediate_ca_name_input.clone(),
    };

    let (new_model, task) = mvi::update(
        self.mvi_model.clone(),
        intent,
        self.storage_port.clone(),
        self.x509_port.clone(),
        self.ssh_port.clone(),
        self.yubikey_port.clone(),
    );

    self.mvi_model = new_model;

    // Map Task<Intent> to Task<Message>
    task.map(|intent| Message::MviIntent(intent))
}

// Add new Message variant:
enum Message {
    // ... existing variants ...
    MviIntent(Intent),  // NEW: for MVI intent results
}

// In Message::MviIntent handler:
Message::MviIntent(intent) => {
    let (new_model, task) = mvi::update(
        self.mvi_model.clone(),
        intent,
        self.storage_port.clone(),
        self.x509_port.clone(),
        self.ssh_port.clone(),
        self.yubikey_port.clone(),
    );

    self.mvi_model = new_model;
    task.map(|intent| Message::MviIntent(intent))
}
```

**Display Integration**:
```rust
// In view_keys(), display from MVI model:
for ca in &self.mvi_model.key_generation_status.intermediate_cas {
    text(format!("✓ {} - {}", ca.name, &ca.fingerprint[..16]))
}
```

**Pros**:
- ✅ Quick to implement (1-2 hours)
- ✅ Keeps existing GUI code
- ✅ Functional end-to-end flow immediately

**Cons**:
- 🔲 Maintains dual state (gui.rs fields + MVI model)
- 🔲 Input fields not in MVI view layer

---

### Option B: Full MVI Migration (Clean Architecture)

**Goal**: Migrate all GUI functionality into MVI system

**Steps**:
1. Add input fields to MVI view layer (text_input widgets)
2. Store input state in MVI Model
3. Remove gui.rs Message handlers
4. Use MviDemoApp pattern from examples/mvi_demo.rs
5. Delete legacy gui.rs code

**Changes Required** (~200-300 lines):
```rust
// In src/mvi/model.rs:
pub struct Model {
    // ... existing fields ...

    // Input state (NEW)
    pub intermediate_ca_name_input: String,
    pub server_cert_cn_input: String,
    pub server_cert_sans_input: String,
    pub selected_intermediate_ca: Option<String>,
}

// In src/mvi/intent.rs (already exists!):
// All UiIntermediateCA* intents already defined

// In src/mvi/view.rs, replace placeholders with inputs:
fn view_keys(model: &Model) -> Element<'_, Intent> {
    // ... existing sections ...

    // Intermediate CA section with input (NEW):
    column![
        text("Step 2: Intermediate CAs").size(18),
        text_input("CA Name (e.g., 'Engineering')", &model.intermediate_ca_name_input)
            .on_input(|s| Intent::UiIntermediateCANameChanged(s)),
        button(text("Generate Intermediate CA"))
            .on_press(Intent::UiGenerateIntermediateCAClicked {
                name: model.intermediate_ca_name_input.clone(),
            }),
        // Display generated CAs (already implemented)
    ]
}

// Main application (replace CimKeysApp):
struct CimKeysApp {
    model: Model,
    storage: Arc<dyn StoragePort>,
    x509: Arc<dyn X509Port>,
    ssh: Arc<dyn SshKeyPort>,
    yubikey: Arc<dyn YubiKeyPort>,
}

impl CimKeysApp {
    fn update(&mut self, intent: Intent) -> Task<Intent> {
        let (new_model, command) = mvi::update(
            self.model.clone(),
            intent,
            self.storage.clone(),
            self.x509.clone(),
            self.ssh.clone(),
            self.yubikey.clone(),
        );
        self.model = new_model;
        command
    }

    fn view(&self) -> Element<'_, Intent> {
        mvi::view(&self.model)
    }
}
```

**Pros**:
- ✅ Clean architecture (single source of truth)
- ✅ Pure MVI pattern throughout
- ✅ No dual state management
- ✅ Easier to test and maintain

**Cons**:
- 🔲 More work to implement (2-4 hours)
- 🔲 Requires careful migration of all GUI features

---

## Implementation Checklist

### For Option A (Quick Integration):
- [ ] Add `mvi_model: Model` to CimKeysApp struct
- [ ] Add port instances (use mock adapters initially)
- [ ] Add `MviIntent(Intent)` message variant
- [ ] Wire `GenerateIntermediateCA` → `UiGenerateIntermediateCAClicked`
- [ ] Wire `GenerateServerCert` → `UiGenerateServerCertClicked`
- [ ] Add `MviIntent` handler that calls `mvi::update()`
- [ ] Display certificates from `self.mvi_model.key_generation_status`
- [ ] Test: Enter CA name → Click button → See generated CA in list

### For Option B (Full Migration):
- [ ] Add input fields to `Model` struct
- [ ] Add `UiIntermediateCANameChanged(String)` handler to update.rs
- [ ] Add `UiServerCertCNChanged(String)` handler to update.rs
- [ ] Add `UiServerCertSANsChanged(String)` handler to update.rs
- [ ] Replace placeholder text in view.rs with actual text_input widgets
- [ ] Wire text inputs to Intent::Ui*Changed variants
- [ ] Replace CimKeysApp with MVI-style app structure
- [ ] Remove legacy Message enum and handlers
- [ ] Test complete flow

---

## Testing Strategy

### Manual Test Cases:
1. **Passphrase Entry**:
   - Enter passphrase → See strength indicator
   - Confirm passphrase → See match indicator
   - Click "Derive Master Seed" → See success message

2. **Root CA Generation**:
   - Click "Generate Root CA" → See Root CA in key status
   - Verify fingerprint displayed

3. **Intermediate CA Generation**:
   - Enter CA name "Engineering"
   - Click "Generate Intermediate CA"
   - Verify "✓ Engineering - fingerprint" appears in list

4. **Server Certificate Generation**:
   - Enter common name "nats.example.com"
   - Enter SANs "nats.example.com,192.168.1.10"
   - Select intermediate CA "Engineering"
   - Click "Generate Server Certificate"
   - Verify certificate appears with signer info

### Integration Test:
```bash
# Run MVI demo (already works):
cargo run --example mvi_demo --features gui

# After integration, test full app:
cargo run --bin cim-keys-gui --features gui -- /tmp/test-output
```

---

## Recommendation

**For next session: Implement Option A (Quick Integration)**

Rationale:
1. Fastest path to working end-to-end system
2. Validates that MVI backend works correctly
3. Allows user testing of PKI workflow
4. Can refactor to Option B later if needed

**Estimated time**: 1-2 hours
**Files to modify**: `src/gui.rs` only
**Lines to add**: ~50-100

This gets us from 85% → 100% functional (Option A complete) or 95% clean architecture (Option B).
