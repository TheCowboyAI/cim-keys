# CIM Keys Technical Documentation

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

Technical documentation for developers, maintainers, and contributors to cim-keys.

## Quick Navigation

### Architecture
- [System Overview](architecture/overview.md) - Complete system architecture with diagrams
- [MVI Pattern](architecture/mvi-pattern.md) - Model-View-Intent architecture
- [Hexagonal Architecture](architecture/hexagonal.md) - Ports and adapters
- [NATS Streaming](architecture/nats-streaming.md) - Event streaming infrastructure
- [Tooling](architecture/tooling.md) - Build and development tools
- [CIM Diagram](architecture/cim-diagram.md) - CIM ecosystem integration

### Axioms & Mathematical Foundations
- [FRP Axioms](axioms/frp-axioms.md) - 10 N-ary FRP axioms (MANDATORY)
- [FRP Guide](axioms/frp-guide.md) - Practical FRP implementation guide
- [Compliance Matrix](axioms/compliance-matrix.md) - Current axiom compliance status
- [Causality (A4)](axioms/causality.md) - UUID v7 timestamp axiom
- [Categorical Semantics](axioms/categorical-semantics.md) - Category theory foundations
- [FRP Getting Started](axioms/N_ARY_FRP_GETTING_STARTED.md) - Quick start for FRP
- [FRP Integration](axioms/N_ARY_FRP_INTEGRATION_GUIDE.md) - Integration patterns

### Methodology
- [Domain-Driven Design](methodology/domain-driven-design.md) - DDD patterns and guidelines
- [MVI Implementation](methodology/mvi-implementation.md) - MVI implementation details
- [MVI Integration](methodology/mvi-integration.md) - Integrating MVI with existing code
- [Testing Strategy](methodology/testing-strategy.md) - Unit, MVI, BDD, property tests

### Development
- [Contributing](development/contributing.md) - How to contribute
- [Testing](development/testing.md) - Running tests
- [Testing Status](development/testing-status.md) - Current test status
- [Fonts](development/fonts.md) - Font configuration

### Integration
- [CIM Registry](integration/cim-registry.md) - Integration with CIM ecosystem
- [NATS Integration](integration/nats-integration.md) - NATS connectivity
- [Event Publishing](integration/event-publishing.md) - Event publishing patterns

### Lessons Learned
- [Workflow Patterns](lessons-learned/workflow-patterns.md) - Patterns that work
- [Migration Guide](lessons-learned/migration-guide.md) - Adopting patterns in other modules

---

## Core Principles

### 1. Event Sourcing
All state changes are recorded as immutable events:
```
Command → Aggregate → Event → Projection
```

### 2. Pure Functions
The MVI update function is pure:
```rust
fn update(model: Model, intent: Intent) -> (Model, Task<Intent>)
```

### 3. Categorical Foundations
- **Coproduct**: DomainNode with injection functions
- **Functor**: LiftableDomain: Domain → Graph
- **Universal Property**: FoldDomainNode trait

### 4. FRP Axiom Compliance

| Axiom | Status | Description |
|-------|--------|-------------|
| A3: Decoupled | ✅ | Output depends only on input |
| A4: Causality | ✅ | UUID v7 + causation_id tracking |
| A5: Totality | ✅ | All functions are total |
| A7: Event Logs | ✅ | Events as timestamped prefixes |
| A9: Composition | ✅ | Associativity verified |

---

## Test Coverage

| Test Type | Count | Purpose |
|-----------|-------|---------|
| Unit Tests | 341 | Component correctness |
| MVI Tests | 33 | Pure function behavior |
| BDD Tests | 18 | Domain workflows |
| Property Tests | 7 | Invariant verification |
| **Total** | **399** | |

---

## Related Documentation

- **User Documentation**: [../user/](../user/README.md)
- **Archive (Historical)**: [../archive/](../archive/README.md)
