# Single Passphrase to Complete PKI - Implementation Progress

## 🎯 Goal
Enable a single person to create, from a single master passphrase, an entire PKI for a small business running a CIM.

## ✅ Completed (Current Session)

### 1. MVI Architecture (100% Complete)
- **Intent Layer** (~260 lines): Unified event source abstraction
  - UI-originated intents (Ui*)
  - Domain events (Domain*)
  - Port responses (Port*)
  - System events (System*)

- **Model Layer** (~240 lines): Pure immutable state
  - Builder pattern with `with_*()` methods
  - No mutable state, clone-before-move pattern
  - Passphrase state management

- **Update Layer** (~450 lines): Pure state transitions
  - `(Model, Intent) → (Model, Task<Intent>)`
  - Port dependency injection
  - Async operations via Task::perform
  - Complete error handling

- **View Layer** (~450 lines): Pure rendering functions
  - `Model → Element<Intent>`
  - Tab-based navigation
  - Comprehensive passphrase UI

### 2. Crypto Module (100% Complete)

#### `src/crypto/seed_derivation.rs` (184 lines)
- **Argon2id KDF**: Memory-hard password derivation
  - Production: 1GB memory, 10 iterations
  - Testing: 64MB memory, 3 iterations
  - SHA-256 based deterministic salt from org_id
  - Base64 encoding without padding for SaltString

- **HKDF-SHA256**: Hierarchical key derivation
  - Domain separation via info parameter
  - Deterministic child seed generation
  - Cryptographically independent seeds

- **Tests**: 4/4 passing
  - Deterministic derivation
  - Different org → different seed
  - Child seed derivation
  - Hierarchical derivation

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
- **Tests**: 4/4 passing

#### `src/crypto/key_generation.rs` (113 lines)
- **Ed25519 Keypairs**: Deterministic from seeds
  - Same seed = same keypair (reproducible)
  - Sign/verify operations
  - 32-byte public/private keys

- **Tests**: 4/4 passing
  - Deterministic generation
  - Different seeds → different keypairs
  - Sign and verify
  - Child seed keypairs

**Total: 12/12 crypto tests passing** ✅

### 3. MVI-Crypto Integration (100% Complete)

#### Model State Extensions
```rust
pub passphrase: String,
pub passphrase_confirmed: String,
pub passphrase_strength: Option<PassphraseStrength>,
pub master_seed_derived: bool,
```

#### Intent Events
- `UiPassphraseChanged(String)` - Real-time input
- `UiPassphraseConfirmChanged(String)` - Confirmation input
- `UiDeriveMasterSeedClicked` - Trigger derivation
- `MasterSeedDerived { org_id, entropy }` - Success event
- `MasterSeedDerivationFailed { error }` - Failure event

#### Update Handlers
- **Real-time strength validation**: Updates as user types
- **Passphrase match verification**: Before derivation
- **Strength requirement enforcement**: Prevents weak passphrases
- **Async seed derivation**: Task::perform with Argon2id
- **Error feedback**: Clear messages to user

### 4. Passphrase UI (100% Complete)

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

## 📊 Architecture Statistics

### Lines of Code
- **MVI Framework**: ~1,400 lines
- **Crypto Module**: ~500 lines
- **Documentation**: ~1,500 lines (SINGLE_PASSPHRASE_WORKFLOW.md, etc.)
- **Total**: ~3,400 lines added this session

### Test Coverage
- **Crypto tests**: 12/12 passing (100%)
- **Build status**: ✅ Success
- **Warnings**: Minor unused imports only

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
│    → Model updated: master_seed_derived = true          │
│    → Status message: "Master seed derived (75.0 bits)"  │
│    → UI shows ✓ with entropy                            │
│    → Ready for hierarchical key generation              │
└─────────────────────────────────────────────────────────┘
```

## 🎨 UI Screenshots (Text Description)

### Keys Tab - Step 1: Master Passphrase

```
┌────────────────────────────────────────────────────────────┐
│ Step 1: Master Passphrase                              │18│
│ All keys are derived from a single master passphrase   │11│
│ using Argon2id (1GB memory, 10 iterations)                 │
│                                                             │
│ Master Passphrase:                                     │12│
│ ┌──────────────────────────────────────────────────────┐   │
│ │ correct horse battery staple mountain river          │   │
│ └──────────────────────────────────────────────────────┘   │
│ Strength: Strong (excellent)                           │12│
│                                                             │
│ Confirm Passphrase:                                    │12│
│ ┌──────────────────────────────────────────────────────┐   │
│ │ correct horse battery staple mountain river          │   │
│ └──────────────────────────────────────────────────────┘   │
│ ✓ Passphrases match                                    │11│
│                                                             │
│ ┌──────────────────────────┐                               │
│ │  Derive Master Seed  │14 │                               │
│ └──────────────────────────┘                               │
│                                                             │
│ ✓ Master Seed Derived                                  │14│
│ All keys will be deterministically generated           │11│
└────────────────────────────────────────────────────────────┘
```

## 🚧 Next Steps

### Immediate (Next Session)
1. **Store master seed securely** - Current TODO in update.rs line 180
   - Consider using secrecy crate
   - Zeroize on drop
   - Memory protection

2. **Connect seed to key generation**
   - Update UiGenerateRootCAClicked to use derived seed
   - Implement HKDF child derivation for Root CA
   - Generate Ed25519 keypair from seed
   - Create X.509 certificate with generated key

### Short-term
3. **Implement hierarchical key derivation**
   - Root CA seed: `master.derive_child("root-ca")`
   - Intermediate CA seeds: `root.derive_child("intermediate-engineering")`
   - User key seeds: `intermediate.derive_child("user-alice")`

4. **YubiKey integration**
   - Store derived keys in PIV slots
   - 9A: Authentication
   - 9C: Digital Signature
   - 9D: Key Management
   - 9E: Card Authentication

5. **SD Card export**
   - LUKS encryption setup
   - Export domain + keys to encrypted partition
   - Manifest generation

### Long-term
6. **NATS credential generation**
   - Operator keys from seed
   - Account keys from operator
   - User keys from accounts

7. **Complete PKI hierarchy**
   - Root CA → Intermediate CAs → Leaf certificates
   - All deterministic from single passphrase
   - Reproducible on any machine with same passphrase + org_id

## 🔐 Security Properties

### Current Implementation
- ✅ **Memory-hard KDF**: Argon2id with 1GB memory cost
- ✅ **Time-hard KDF**: 10 iterations (production)
- ✅ **Deterministic salt**: SHA-256(org_id)
- ✅ **Cryptographic separation**: HKDF domain separation
- ✅ **Strong entropy**: 70+ bits for Strong classification
- ✅ **Real-time validation**: Prevents weak passphrases

### Future Enhancements
- 🔲 Passphrase masking (if Iced adds `.secure()` method)
- 🔲 Zeroize passphrases in memory
- 🔲 Memory protection (mlock/mprotect)
- 🔲 Hardware-backed storage (YubiKey)

## 📈 Progress Metrics

- **Completion**: ~40% of single-passphrase-to-PKI workflow
- **Core crypto**: 100% ✅
- **MVI architecture**: 100% ✅
- **Passphrase UI**: 100% ✅
- **Key generation from seed**: 0%
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
7. **Test-Driven Crypto**: Write tests first, fix implementation
8. **Production vs Test Params**: Use cfg(test) for faster test execution
9. **Salt Generation**: Hash input + Base64 encode without padding
10. **Strength Thresholds**: Adjust for word-based vs character-based

## 📝 Documentation

- ✅ `SINGLE_PASSPHRASE_WORKFLOW.md` (560 lines)
- ✅ `MVI_IMPLEMENTATION_GUIDE.md` (500+ lines)
- ✅ `MVI_ARCHITECTURE_DIAGRAM.md`
- ✅ `PASSPHRASE_TO_PKI_PROGRESS.md` (this file)
- ✅ Comprehensive code comments
- ✅ Test documentation

## 🔗 Related Files

### Source Code
- `src/crypto/seed_derivation.rs` - Argon2id + HKDF
- `src/crypto/passphrase.rs` - Validation & entropy
- `src/crypto/key_generation.rs` - Ed25519 keypairs
- `src/mvi/model.rs` - State management
- `src/mvi/intent.rs` - Event types
- `src/mvi/update.rs` - State transitions
- `src/mvi/view.rs` - UI rendering

### Tests
- `src/crypto/seed_derivation.rs` - 4 tests
- `src/crypto/passphrase.rs` - 4 tests
- `src/crypto/key_generation.rs` - 4 tests

### Configuration
- `Cargo.toml` - Added argon2, hkdf dependencies
- `flake.nix` - Nix development environment

---

**Last Updated**: 2025-11-09
**Session Duration**: ~2 hours
**Commits**: 2 major commits
**Status**: ✅ Core foundation complete, ready for key generation integration
