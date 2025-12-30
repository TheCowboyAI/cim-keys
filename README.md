# CIM Keys

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

**Event-Sourced Cryptographic Key Management for CIM Infrastructure**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-proprietary-blue.svg)](LICENSE)

## Overview

CIM Keys is the genesis point for CIM (Composable Information Machine) infrastructures. It provides:

- **Domain Bootstrap**: Create organizations, people, locations, and their relationships
- **PKI Generation**: Root CA, intermediate CAs, personal keys, and certificates
- **NATS Security**: Operators, accounts, users, and JWT credentials
- **YubiKey Integration**: Hardware token provisioning and management
- **Offline-First**: Air-gapped operation with encrypted SD card projections

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         CIM Keys                                 │
├──────────────┬──────────────┬──────────────┬───────────────────┤
│   Domain     │    Events    │  Projections │      GUI          │
│  Bootstrap   │   (CQRS)     │  (JSON/SD)   │  (Iced 0.13+)     │
├──────────────┼──────────────┼──────────────┼───────────────────┤
│ Organization │ DomainEvent  │ OfflineKey   │ MVI Architecture  │
│ Person       │ KeyEvent     │ Projection   │ Pure Update Fn    │
│ Location     │ YubiKeyEvent │              │ Intent Routing    │
└──────────────┴──────────────┴──────────────┴───────────────────┘
```

## Quick Start

```bash
# Enter Nix development shell
nix develop

# Build native application
cargo build --release --features gui

# Run GUI
cargo run --bin cim-keys-gui -- /path/to/output

# Run tests
cargo test --features gui
```

## Key Features

### Event-Sourced FRP Architecture

All state changes flow through immutable events:

```rust
Command → Aggregate → Events → Projection → GUI
```

- **No CRUD operations** - Only domain events
- **Offline-first** - Events stored as JSON on encrypted partitions
- **Replay capability** - Reconstruct state from event history

### MVI (Model-View-Intent) GUI

Pure functional reactive GUI using Iced 0.13+:

```rust
fn update(model: Model, intent: Intent) -> (Model, Task<Intent>) {
    // Pure function - no side effects
    match intent {
        Intent::UiTabSelected(tab) => (model.with_tab(tab), Task::none()),
        Intent::PortFileLoaded(data) => (model.with_data(data), Task::none()),
        // ...
    }
}
```

### LiftableDomain Pattern

Faithful functor for domain composition:

```rust
pub trait LiftableDomain {
    fn lift(&self) -> LiftedNode;
    fn unlift(node: &LiftedNode) -> Option<Self>;
    fn injection() -> Injection;
}

// Implemented for: Organization, OrganizationUnit, Person, Location
```

## Project Structure

```
src/
├── aggregate.rs       # Command processing (CQRS)
├── commands/          # Domain commands by aggregate
├── events/            # Immutable domain events
├── projections.rs     # State materialization to JSON
├── domain.rs          # Organization, Person, Location
├── gui.rs             # Iced GUI application
├── mvi/               # Model-View-Intent architecture
│   ├── model.rs       # Immutable application state
│   ├── intent.rs      # User intents (Ui*, Port*, Domain*)
│   └── update.rs      # Pure update function
├── lifting.rs         # LiftableDomain trait
└── crypto/            # Key generation, X.509, seeds

tests/
├── mvi_tests.rs       # MVI property tests (33 tests)
├── bdd_tests.rs       # BDD step definitions (18 tests)
└── bdd/               # Step definition modules

doc/qa/features/       # Gherkin specifications (112 scenarios)
├── domain_bootstrap.feature
├── person_management.feature
├── key_generation.feature
├── yubikey_provisioning.feature
├── nats_security_bootstrap.feature
└── export_manifest.feature
```

## Testing

### Test Summary

| Type | Count | Location |
|------|-------|----------|
| Library tests | 341 | `src/**/*.rs` |
| MVI tests | 33 | `tests/mvi_tests.rs` |
| BDD tests | 18 | `tests/bdd_tests.rs` |
| Gherkin specs | 112 | `doc/qa/features/` |
| Workflow tests | ~50 | `tests/*_state_machine.rs` |

### Run Tests

```bash
# All tests
cargo test --features gui

# Library only
cargo test --features gui --lib

# MVI tests
cargo test --features gui --test mvi_tests

# BDD tests
cargo test --features gui --test bdd_tests
```

## Configuration

### Domain Bootstrap

Create `secrets/domain-bootstrap.json`:

```json
{
  "organization": {
    "name": "CowboyAI",
    "domain": "cowboyai.com"
  },
  "units": [
    { "name": "Engineering", "type": "Department" }
  ],
  "people": [
    { "name": "Admin", "email": "admin@cowboyai.com", "role": "Administrator" }
  ]
}
```

### Export Structure

```
/mnt/encrypted/cim-keys/
├── manifest.json
├── domain/
│   ├── organization.json
│   ├── people/
│   └── relationships.json
├── keys/
│   └── {key-id}/
│       ├── metadata.json
│       └── public.pem
├── certificates/
│   ├── root-ca/
│   ├── intermediate-ca/
│   └── leaf/
├── nats/
│   ├── operator/
│   ├── accounts/
│   └── users/
└── events/
    └── {date}/
```

## Development

### Prerequisites

- Rust 1.75+
- Nix (recommended)
- YubiKey (optional, for hardware key storage)

### Best Practices

1. **UUID v7** - Always use `Uuid::now_v7()` for time-ordered IDs
2. **Pure Functions** - Update functions must be pure
3. **Immutable Models** - Use `with_*` methods, never mutate
4. **Event Sourcing** - All state changes through events
5. **Intent Naming** - Prefix with origin: `Ui*`, `Port*`, `Domain*`

See [CLAUDE.md](CLAUDE.md) for comprehensive development guidelines.

## Documentation

| Document | Purpose |
|----------|---------|
| [CLAUDE.md](CLAUDE.md) | Development guidelines |
| [REFACTORING_PLAN.md](REFACTORING_PLAN.md) | Architecture refactoring plan |
| [N_ARY_FRP_AXIOMS.md](N_ARY_FRP_AXIOMS.md) | FRP axiom specification |
| [retrospectives/](retrospectives/) | Sprint retrospectives |

## License

Copyright (c) 2025 - Cowboy AI, LLC. All rights reserved.

## NATS Credentials (DGX Phoenix)

For NATS cluster credentials, see [creds/](creds/) directory.

```bash
# Connect as admin
nats --creds=creds/ADMIN/admin.creds server list
```
