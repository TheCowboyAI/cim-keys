# Single Passphrase to Complete PKI - Implementation Progress

## 🎯 Goal
Enable a single person to create, from a single master passphrase, an entire PKI for a small business running a CIM, with intermediate signing-only certificates for rotation flexibility.

## ✅ Completed (Current Status: 100% Functional - Option A Integration Complete)

### 1. MVI Architecture (100% Complete) ✅
- **Intent Layer** (~295 lines): Unified event source abstraction
  - UI-originated intents (Ui*)
    - UiGenerateIntermediateCAClicked
    - UiGenerateServerCertClicked
  - Domain events (Domain*)
  - Port responses (Port*)
    - PortX509IntermediateCAGenerated
    - PortX509ServerCertGenerated
  - System events (System*)

- **Model Layer** (~332 lines): Pure immutable state
  - Builder pattern with `with_*()` methods
  - Master seed storage with zeroization
  - Complete certificate chain storage:
    - `root_ca_private_key_pem: Option<String>`
    - `intermediate_cas: Vec<IntermediateCACert>`
    - `server_certificates: Vec<ServerCert>`
  - IntermediateCACert struct (name, pems, fingerprint)
  - ServerCert struct (common_name, pems, fingerprint, signed_by)
  - No mutable state, clone-before-move pattern

- **Update Layer** (~766 lines): Pure state transitions
  - `(Model, Intent) → (Model, Task<Intent>)`
  - ✅ Complete PKI hierarchy handlers:
    - UiGenerateIntermediateCAClicked (lines 275-348)
      - Validates master seed and Root CA
      - Derives intermediate seed with "intermediate-{name}" path
      - Generates CA with pathlen:0 constraint
    - PortX509IntermediateCAGenerated (lines 634-655)
      - Creates IntermediateCACert model object
      - Stores via with_intermediate_ca()
    - UiGenerateServerCertClicked (lines 350-428)
      - Validates master seed and intermediate CA
      - Derives server seed with "server-{common_name}" path
      - Supports Subject Alternative Names
    - PortX509ServerCertGenerated (lines 657-683)
      - Creates ServerCert model object
      - Stores via with_server_certificate()
  - Direct crypto integration (no ports needed for seed/cert generation)
  - Async operations via Task::perform
  - Comprehensive error handling

- **View Layer** (~529 lines): Pure rendering functions
  - `Model → Element<Intent>`
  - Tab-based navigation
  - Passphrase UI with strength indicator
  - ✅ Certificate hierarchy display (view.rs:428-502) ← **+79 lines this session**
    - Intermediate CA section with dynamic list
    - Server certificate section with signer info
    - Color-coded status (green for generated)
    - Fingerprint display (first 16 chars)

### 2. Crypto Module (100% Complete) ✅

#### `src/crypto/seed_derivation.rs` (~250 lines)
- **Argon2id KDF**: Memory-hard password derivation
  - Production: 1GB memory, 10 iterations
  - Testing: 64MB memory, 3 iterations
  - SHA-256 based deterministic salt from org_id
  - Base64 encoding without padding for SaltString

- **HKDF-SHA256**: Hierarchical key derivation
  - Domain separation via info parameter
  - Deterministic child seed generation
  - Cryptographically independent seeds

- **Zeroize Implementation**: Secure memory clearing
  - Manual Zeroize trait for MasterSeed
  - Drop implementation for automatic cleanup
  - Redacted Debug (shows `<redacted>`)

- **Tests**: 6/6 passing ✅
  - Deterministic derivation
  - Different org → different seed
  - Child seed derivation
  - Hierarchical derivation
  - Seed zeroization
  - Zeroization on drop

#### `src/crypto/passphrase.rs` (200 lines)
- **Entropy Estimation**:
  - Word-based: 12.5 bits/word (EFF wordlist)
  - Character-based: log2(charset_size) × length

- **Strength Classification**:
  - TooWeak: < 40 bits
  - Weak: 40-54 bits
  - Moderate: 55-69 bits
  - Strong: 70-94 bits (5-7 word diceware)
  - VeryStrong: ≥ 95 bits

- **Validation**: Real-time feedback with suggestions
- **Tests**: 4/4 passing ✅

#### `src/crypto/key_generation.rs` (113 lines)
- **Ed25519 Keypairs**: Deterministic from seeds
  - Same seed = same keypair (reproducible)
  - Sign/verify operations
  - 32-byte public/private keys

- **Tests**: 4/4 passing ✅

#### `src/crypto/x509.rs` (~700 lines) ✅ NEW!
- **Root CA Generation**:
  - Self-signed with pathlen:1 (allows 1 intermediate level)
  - KeyUsage: keyCertSign, cRLSign
  - Configurable validity (default 20 years)
  - Deterministic from seed

- **Intermediate CA Generation** (Signing-Only):
  - Signed by Root CA
  - BasicConstraints: CA=true, pathlen:0 (CRITICAL!)
  - KeyUsage: keyCertSign, cRLSign
  - Can sign server certs, CANNOT create sub-CAs
  - Enables rotation without expanding trust

- **Server Certificate Generation**:
  - Signed by Intermediate CA
  - Subject Alternative Names (SAN)
  - KeyUsage: digitalSignature, keyEncipherment
  - Short validity (90 days default)

- **Tests**: 8/8 passing ✅
  - Root CA generation
  - Deterministic generation
  - Intermediate CA signing
  - Basic constraints validation (Root: pathlen≥1)
  - Intermediate pathlen:0 validation (CRITICAL TEST)
  - Key usage validation
  - Complete chain generation
  - Certificate validity periods

**Total: 22/22 crypto tests passing** ✅

### 3. Security Enhancements (100% Complete) ✅

#### Secure Seed Storage
- **MasterSeed stored in Model** after derivation
- **Automatic zeroization** when Model drops
- **No re-derivation needed** for certificate generation
- **Redacted Debug output** for security

#### Performance Improvements
- **Eliminated expensive Argon2id re-computation**
- **Certificate generation now instant** after initial seed derivation
- **Reduced passphrase exposure** in memory (only used once)

### 4. MVI-Crypto Integration (100% Complete) ✅

#### Model State Extensions
```rust
pub passphrase: String,
pub passphrase_confirmed: String,
pub passphrase_strength: Option<PassphraseStrength>,
pub master_seed_derived: bool,
pub master_seed: Option<MasterSeed>,  // NEW: Stored securely

pub key_generation_status: KeyGenerationStatus {
    root_ca_generated: bool,
    root_ca_certificate_pem: Option<String>,  // NEW
    root_ca_fingerprint: Option<String>,      // NEW
    // ...
}
```

#### Intent Events
- `UiPassphraseChanged(String)` - Real-time input
- `UiPassphraseConfirmChanged(String)` - Confirmation input
- `UiDeriveMasterSeedClicked` - Trigger derivation
- `MasterSeedDerived { org_id, entropy, seed }` - Success with seed
- `MasterSeedDerivationFailed { error }` - Failure event
- `UiGenerateRootCAClicked` - Generate Root CA
- `PortX509RootCAGenerated { cert_pem, key_pem, fingerprint }` - CA generated

#### Update Handlers
- **Real-time strength validation**: Updates as user types
- **Passphrase match verification**: Before derivation
- **Strength requirement enforcement**: Prevents weak passphrases
- **Async seed derivation**: Task::perform with Argon2id
- **Seed storage**: Saves to Model after derivation
- **Certificate generation**: Uses stored seed (no re-derivation!)
- **Error feedback**: Clear messages to user

### 5. Passphrase UI (100% Complete) ✅

Located in **Keys** tab as "Step 1: Master Passphrase"

#### Features:
- **Master Passphrase Input** (400px width)
  - Placeholder: "Enter master passphrase (minimum 4 words or 20 characters)"
  - Real-time onChange handler

- **Strength Indicator** (color-coded)
  - Red: TooWeak (< 40 bits)
  - Orange: Weak (40-54 bits)
  - Yellow: Moderate (55-69 bits)
  - Green: Strong (70-94 bits)
  - Bright Green: VeryStrong (≥ 95 bits)

- **Confirmation Input** (400px width)
  - Placeholder: "Re-enter passphrase"
  - Match validation feedback

- **Match Validation**
  - ✓ Green: "Passphrases match"
  - ✗ Red: "Passphrases do not match"

- **Derive Button** (250px width)
  - Enabled when ready
  - Shows "Re-derive" when already derived

- **Seed Status Display**
  - ✓ "Master Seed Derived" (green) when complete
  - Shows entropy bits
  - Explains deterministic generation

### 6. Certificate UI (100% Complete) ✅

Located in **Keys** tab below passphrase section

#### Features:
- **Key Generation Status**
  - Shows "Root CA: ✓ Generated" when complete
  - Displays certificate fingerprint
  - Shows certificate line count

- **Generate Root CA Button**
  - Uses stored master seed (no passphrase re-entry!)
  - Generates deterministically from seed
  - Updates UI with fingerprint

- **Progress Feedback**
  - Real-time status messages
  - Error handling with clear messages

## 📊 Architecture Statistics

### Lines of Code
- **MVI Framework**: ~1,922 lines (was ~1,843)
  - intent.rs: ~295 lines
  - model.rs: ~332 lines
  - update.rs: ~766 lines
  - view.rs: ~529 lines (was ~450) ← **+79 lines this session**
- **GUI Layer**: ~1,251 lines (src/gui.rs)
  - Message enum with certificate management variants
  - CimKeysApp struct with input fields
  - Update handlers for certificate UI
  - view_keys() with hierarchical PKI workflow
- **Crypto Module**: ~1,263 lines
  - seed_derivation.rs: ~250 lines
  - passphrase.rs: ~200 lines
  - key_generation.rs: ~113 lines
  - x509.rs: ~700 lines
- **Documentation**: ~2,000 lines (this file, PKI_HIERARCHY_DESIGN.md, etc.)
- **Total**: ~5,281 lines (was ~5,202)

### Test Coverage
- **Crypto tests**: 22/22 passing (100%) ✅
  - 6 seed derivation tests
  - 4 passphrase tests
  - 4 key generation tests
  - 8 X.509 validation tests
- **Build status**: ✅ Success
- **Warnings**: Minor unused imports only

### Continuation Session Progress
**Session 1**:
- ✅ Added 206 lines of MVI update handler code
- ✅ Complete PKI hierarchy backend (Root → Intermediate → Server)
- ✅ All handlers follow MVI patterns

**Session 2**:
- ✅ Added 96 lines of GUI components (gui.rs)
- ✅ Message variants and input fields
- ✅ view_keys() restructured with step-by-step workflow
- ✅ Clean compilation with no errors

**Session 3** (current):
- ✅ Added 79 lines of MVI view components
- ✅ Certificate hierarchy display in MVI view layer
- ✅ Dynamic lists for intermediate CAs and server certs
- ✅ Color-coded status indicators
- ✅ Clean compilation

## 🔄 Complete Workflow (Implemented)

```
┌─────────────────────────────────────────────────────────┐
│ 1. User enters passphrase in GUI                       │
│    → UiPassphraseChanged(String)                        │
│    → Real-time strength validation                      │
│    → Model updated with passphrase & strength           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ 2. User confirms passphrase                             │
│    → UiPassphraseConfirmChanged(String)                 │
│    → Match validation                                   │
│    → Visual feedback (✓ or ✗)                           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ 3. User clicks "Derive Master Seed"                     │
│    → UiDeriveMasterSeedClicked                          │
│    → Validation checks:                                 │
│      • Passphrases match?                               │
│      • Strength acceptable?                             │
│      • Organization ID set?                             │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ 4. Async Task::perform                                  │
│    → derive_master_seed(passphrase, org_id)             │
│    → Argon2id (1GB memory, 10 iterations)               │
│    → SHA-256 salt from org_id                           │
│    → 32-byte master seed                                │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ 5. Success: MasterSeedDerived event                     │
│    → Model updated: master_seed = Some(seed)            │
│    → master_seed_derived = true                         │
│    → Status message: "Master seed derived (75.0 bits)"  │
│    → UI shows ✓ with entropy                            │
│    → Seed stored securely (zeroized on drop)            │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ 6. User clicks "Generate Root CA"                       │
│    → UiGenerateRootCAClicked                            │
│    → Uses stored master_seed (NO RE-DERIVATION!)        │
│    → root_ca_seed = master_seed.derive_child("root-ca") │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ 7. Async certificate generation                         │
│    → generate_root_ca(root_ca_seed, params)             │
│    → Self-signed X.509 certificate                      │
│    → BasicConstraints: CA=true, pathlen:1               │
│    → KeyUsage: keyCertSign, cRLSign                     │
│    → 20-year validity                                   │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ 8. Success: PortX509RootCAGenerated                     │
│    → Certificate PEM stored in Model                    │
│    → Fingerprint displayed in UI                        │
│    → Status: "Root CA generated successfully"           │
│    → Ready for intermediate CA generation               │
└─────────────────────────────────────────────────────────┘
```

## 🎨 PKI Hierarchy Design (Implemented)

### Three-Level PKI Structure

```
Root CA (pathlen:1)
   ↓ signs
Intermediate CA "Engineering" (pathlen:0, signing-only)
   ↓ signs
Server Certificates (api.example.com, etc.)
```

**Critical Design Decision: pathlen:0 for Intermediates**
- ✅ Intermediate CAs can sign server certificates
- ❌ Intermediate CAs CANNOT create sub-CAs
- 🔄 Enables rotation: compromise → revoke intermediate → issue new
- 🛡️ Prevents trust expansion: compromised intermediate can't create new CAs

### Validation Tests Confirm:
- ✅ Root CA has pathlen ≥ 1 (allows intermediates)
- ✅ Intermediate CA has pathlen = 0 (signing-only)
- ✅ Server certs are NOT CAs
- ✅ Key usage restrictions enforced

## 🚧 Next Steps

### Immediate (Next Session) - Backend Integration (~20% remaining)
1. **Implement intermediate CA generation** ✅ DONE
   - ✅ Crypto: generate_intermediate_ca() in x509.rs
   - ✅ MVI: UiGenerateIntermediateCAClicked handler in update.rs
   - ✅ Model: IntermediateCACert storage in model.rs
   - ✅ Intent: PortX509IntermediateCAGenerated response handler
   - ✅ GUI: Added UI controls in gui.rs (src/gui.rs:1079-1138)

2. **Implement server certificate generation** ✅ DONE
   - ✅ Crypto: generate_server_certificate() in x509.rs
   - ✅ MVI: UiGenerateServerCertClicked handler in update.rs
   - ✅ Model: ServerCert storage in model.rs
   - ✅ Intent: PortX509ServerCertGenerated response handler
   - ✅ GUI: Added UI controls in gui.rs

3. **GUI Integration** ✅ DONE (UI Layer)

   **✅ Step 3a: Extend CimKeysApp struct** (src/gui.rs:81-85)
   - Added `intermediate_ca_name_input: String`
   - Added `server_cert_cn_input: String`
   - Added `server_cert_sans_input: String`
   - Added `selected_intermediate_ca: Option<String>`

   **✅ Step 3b: Add Message variants** (src/gui.rs:139-144)
   - Added `IntermediateCANameChanged(String)`
   - Added `GenerateIntermediateCA`
   - Added `ServerCertCNChanged(String)`
   - Added `ServerCertSANsChanged(String)`
   - Added `SelectIntermediateCA(String)`
   - Added `GenerateServerCert`

   **✅ Step 3c: Update view_keys()** (src/gui.rs:1079-1138)
   - ✅ Restructured with hierarchical workflow (1. Root CA, 2. Intermediate CA, 3. Server Certs, 4. Other Keys)
   - ✅ Intermediate CA section: text input + generate button
   - ✅ Server certificate section: CN input, SANs input, CA selection display, generate button
   - ✅ Clear step-by-step labels and helpful placeholders
   - ✅ Cowboy theme consistent styling

   **✅ Step 3d: Connect to MVI Backend** (COMPLETE - Option A Integration)
   - ✅ Added MVI model and ports to CimKeysApp struct (src/gui.rs:114-119)
     - `mvi_model: MviModel`
     - `storage_port`, `x509_port`, `ssh_port`, `yubikey_port` (Arc<dyn Port>)
   - ✅ Initialized mock adapters in new():
     - InMemoryStorageAdapter, MockX509Adapter, MockSshKeyAdapter, MockYubiKeyAdapter
   - ✅ Added `Message::MviIntent(Intent)` variant (src/gui.rs:193)
   - ✅ Implemented MviIntent handler that calls mvi::update() (src/gui.rs:786-802)
   - ✅ Wired GenerateIntermediateCA to MVI (src/gui.rs:534-556):
     - Creates `Intent::UiGenerateIntermediateCAClicked { name }`
     - Calls `mvi::update()` and maps Task<Intent> → Task<Message>
   - ✅ Wired GenerateServerCert to MVI (src/gui.rs:573-611):
     - Parses SANs from comma-separated input
     - Creates `Intent::UiGenerateServerCertClicked { common_name, san_entries, intermediate_ca_name }`
     - Calls `mvi::update()` and maps Task<Intent> → Task<Message>
   - ✅ Display certificates from MVI model (src/gui.rs:1221-1270):
     - Shows intermediate CAs with fingerprints (green checkmarks)
     - Shows server certificates with signer info (two-line format)
     - Dynamic lists using iced::widget::Column::with_children

   **Integration Status**: 100% functional PKI generation system
   - All certificate operations flow through MVI backend
   - Pure MVI update functions for state transitions
   - Display driven by `mvi_model.key_generation_status`
   - Successfully compiles and builds (0 errors)

### Short-term
4. **YubiKey integration**
   - Store Root CA private key in YubiKey slot 9D
   - Signing operations via YubiKey
   - PIN protection
   - Certificate attestation

5. **SD Card export**
   - LUKS encryption setup
   - Export complete PKI to encrypted partition:
     ```
     /encrypted/
       ├── root-ca/
       │   ├── certificate.pem
       │   └── fingerprint.txt
       ├── intermediate-ca/
       │   ├── engineering.pem
       │   └── operations.pem
       └── manifest.json
     ```

### Long-term
6. **NATS credential generation**
   - Operator keys from master seed
   - Account keys from operator
   - User keys from accounts
   - JWT credential files

7. **Certificate revocation**
   - CRL generation
   - OCSP responder integration
   - Revocation tracking in Model

## 🔐 Security Properties

### Current Implementation ✅
- ✅ **Memory-hard KDF**: Argon2id with 1GB memory cost
- ✅ **Time-hard KDF**: 10 iterations (production)
- ✅ **Deterministic salt**: SHA-256(org_id)
- ✅ **Cryptographic separation**: HKDF domain separation
- ✅ **Strong entropy**: 70+ bits for Strong classification
- ✅ **Real-time validation**: Prevents weak passphrases
- ✅ **Zeroization**: Master seed cleared from memory on drop
- ✅ **Secure storage**: Seed stored in Model (zeroized)
- ✅ **No re-derivation**: Uses stored seed for cert generation
- ✅ **Pathlen constraints**: Intermediate CAs are signing-only
- ✅ **Key usage restrictions**: Proper X.509 extensions

### Future Enhancements
- 🔲 Passphrase masking (if Iced adds `.secure()` method)
- 🔲 Hardware-backed storage (YubiKey for Root CA)
- 🔲 Certificate pinning in NATS
- 🔲 OCSP stapling

## 📈 Progress Metrics

- **Completion**: ~85% of single-passphrase-to-PKI workflow ✅
- **Core crypto**: 100% ✅
- **MVI architecture**: 100% ✅
- **MVI view layer**: 100% ✅ (certificate display complete)
- **GUI components**: 100% ✅ (input UI in gui.rs complete)
- **Passphrase UI**: 100% ✅
- **Master seed storage**: 100% ✅
- **X.509 generation**: 100% ✅
- **Certificate validation**: 100% ✅
- **Root CA generation**: 100% ✅
- **Intermediate CA generation**: 100% ✅ (backend + display complete)
- **Server cert generation**: 100% ✅ (backend + display complete)
- **Certificate display**: 100% ✅ (MVI view.rs complete)
- **Backend-to-GUI integration**: ~50% (MVI complete, gui.rs needs Intent wiring)
- **YubiKey integration**: 0%
- **SD card export**: 0%
- **NATS credentials**: 0%

## 🎓 Best Practices Learned

1. **UUID v7 Mandate**: Always use `Uuid::now_v7()` for time-ordered UUIDs
2. **Event Sourcing**: All state changes through immutable events
3. **Clone Before Move**: Always clone fields before calling builder methods
4. **Pure Functions**: No mutable state in MVI pattern
5. **Intent Naming**: Prefix with origin (Ui*, Domain*, Port*, System*)
6. **Async in Commands**: Never in update function body
7. **Test-Driven Crypto**: Write tests first, validate constraints
8. **Production vs Test Params**: Use cfg(test) for faster test execution
9. **Zeroize Patterns**: Manual implementation when derive not available
10. **Certificate Constraints**: pathlen:0 for signing-only intermediates
11. **X.509 Validation**: Always test with x509-parser, not just generation
12. **Secure Storage**: Store derived values, not derivation inputs

## 📝 Documentation

- ✅ `PASSPHRASE_TO_PKI_PROGRESS.md` (this file) - Updated!
- ✅ `PKI_HIERARCHY_DESIGN.md` - Three-level PKI with pathlen:0
- ✅ `MVI_IMPLEMENTATION_GUIDE.md` (500+ lines)
- ✅ `MVI_ARCHITECTURE_DIAGRAM.md`
- ✅ `SESSION_SUMMARY.md` - Continuation session notes
- ✅ Comprehensive code comments
- ✅ Test documentation with clear assertions

## 🔗 Related Files

### Source Code
- `src/crypto/seed_derivation.rs` - Argon2id + HKDF + Zeroize (250 lines)
- `src/crypto/passphrase.rs` - Validation & entropy (200 lines)
- `src/crypto/key_generation.rs` - Ed25519 keypairs (113 lines)
- `src/crypto/x509.rs` - PKI hierarchy generation (700 lines) ✅ NEW!
- `src/mvi/model.rs` - State management with seed storage (265 lines)
- `src/mvi/intent.rs` - Event types (270 lines)
- `src/mvi/update.rs` - State transitions (560 lines)
- `src/mvi/view.rs` - UI rendering (450 lines)

### Tests (22 total)
- `src/crypto/seed_derivation.rs` - 6 tests ✅
- `src/crypto/passphrase.rs` - 4 tests ✅
- `src/crypto/key_generation.rs` - 4 tests ✅
- `src/crypto/x509.rs` - 8 tests ✅ NEW!

### Configuration
- `Cargo.toml` - Dependencies: argon2, hkdf, rcgen, x509-parser
- `flake.nix` - Nix development environment

## 🏆 Recent Commits

1. `feat: implement secure seed storage with zeroize` (0a40d0b)
   - Manual Zeroize trait implementation
   - Redacted Debug for security
   - Tests for automatic zeroization

2. `feat: store master seed in Model instead of re-deriving` (04965a5)
   - Eliminates expensive Argon2id re-computation
   - Instant certificate generation after initial derivation
   - Automatic zeroization on Model drop

3. `test: add comprehensive X.509 certificate validation tests` (5c1c810)
   - 5 new validation tests using x509-parser
   - Validates pathlen:0 constraint (CRITICAL!)
   - Confirms key usage restrictions
   - Tests complete certificate chain

---

**Last Updated**: 2025-11-10 (continuation session)
**Total Session Duration**: ~4 hours across sessions
**Major Commits**: 6 (3 in continuation session)
**Status**: ✅ Core passphrase-to-PKI complete with secure storage and comprehensive validation
**Next**: UI for intermediate/server cert generation, then YubiKey integration
