# Test Improvement Plan for CIM-Keys and Domain Modules

## Executive Summary
This plan addresses critical test failures across three domain modules and establishes comprehensive testing for cim-keys to guarantee its role as the genesis point for CIM infrastructures.

## Current State Analysis

### Test Status (as of 2025-10-22)
- **cim-domain-organization**: 4/20 tests passing (library only)
- **cim-domain-person**: 0/58 tests passing (compilation errors)
- **cim-domain-location**: 0/55 tests passing (compilation errors)
- **cim-keys**: 1 test only (insufficient coverage)

### Critical Gaps
1. API mismatches after cim-domain v0.7.8 migration
2. Missing integration tests between modules
3. No cryptographic operation tests
4. No event-sourcing validation tests
5. No YubiKey simulation/mock tests

## Phase 1: Fix Domain Module Tests (Week 1)

### 1.1 cim-domain-organization
**Priority: HIGH** - Required for organizational structure

#### Fix Compilation Errors
- [ ] Add missing `SizeCategory` enum
- [ ] Fix struct field mismatches in tests
- [ ] Update to new AggregateRoot trait API
- [ ] Fix EntityId usage patterns

#### Add Critical Tests
```rust
// Required test categories
- Organization hierarchy management
- Unit relationships and boundaries
- Role assignments and permissions
- Event propagation between units
- NATS subject generation for org structure
```

### 1.2 cim-domain-person
**Priority: HIGH** - Required for identity and key ownership

#### Fix Compilation Errors
- [ ] Fix `RegisterComponent` missing fields
- [ ] Update component registration flow
- [ ] Fix workflow manager tests
- [ ] Update persistence layer tests

#### Add Critical Tests
```rust
// Required test categories
- Person identity lifecycle
- Component attachment/detachment
- Workflow state transitions
- Key ownership assignment
- Delegation chain validation
```

### 1.3 cim-domain-location
**Priority: MEDIUM** - Required for key storage locations

#### Fix Compilation Errors
- [ ] Fix workflow manager mutability issues
- [ ] Update spatial search tests
- [ ] Fix coordinate system tests
- [ ] Update hierarchy tests

#### Add Critical Tests
```rust
// Required test categories
- Physical location validation
- Virtual location mapping
- Location hierarchy traversal
- Geo-coordinate calculations
- Storage location assignment
```

## Phase 2: Core cim-keys Functionality Tests (Week 2)

### 2.1 Domain Bootstrap Tests
```rust
#[cfg(test)]
mod domain_bootstrap_tests {
    // Test organization creation from JSON
    #[test]
    fn test_load_domain_bootstrap_config() { }

    // Test person-to-organization mapping
    #[test]
    fn test_create_organizational_graph() { }

    // Test role assignment validation
    #[test]
    fn test_assign_key_owner_roles() { }

    // Test YubiKey assignment to people
    #[test]
    fn test_yubikey_person_assignment() { }
}
```

### 2.2 Cryptographic Operations Tests
```rust
#[cfg(test)]
mod crypto_tests {
    // Root CA generation
    #[test]
    fn test_generate_root_ca() { }

    // Intermediate CA chain
    #[test]
    fn test_generate_intermediate_ca() { }

    // SSH key generation (Ed25519, RSA, ECDSA)
    #[test]
    fn test_generate_ssh_keys() { }

    // TLS certificate generation
    #[test]
    fn test_generate_tls_certificates() { }

    // GPG key generation
    #[test]
    fn test_generate_gpg_keys() { }

    // NATS operator/account/user keys
    #[test]
    fn test_generate_nats_hierarchy() { }
}
```

### 2.3 Event Sourcing Tests
```rust
#[cfg(test)]
mod event_sourcing_tests {
    // Command to Event flow
    #[test]
    fn test_command_generates_events() { }

    // Event correlation and causation
    #[test]
    fn test_event_correlation_chain() { }

    // Projection from events
    #[test]
    fn test_projection_from_event_stream() { }

    // Event replay and consistency
    #[test]
    fn test_event_replay_consistency() { }
}
```

### 2.4 Storage and Projection Tests
```rust
#[cfg(test)]
mod projection_tests {
    // Encrypted partition mock
    #[test]
    fn test_encrypted_storage_mock() { }

    // JSON projection structure
    #[test]
    fn test_project_to_json_structure() { }

    // Manifest generation
    #[test]
    fn test_generate_manifest() { }

    // Event log persistence
    #[test]
    fn test_persist_event_log() { }
}
```

### 2.5 YubiKey Integration Tests (with mocks)
```rust
#[cfg(test)]
mod yubikey_tests {
    use mockall::mock;

    // Mock YubiKey interface
    mock! {
        YubiKey {
            fn generate_key(&self, slot: PIVSlot) -> Result<PublicKey>;
            fn sign(&self, slot: PIVSlot, data: &[u8]) -> Result<Signature>;
        }
    }

    #[test]
    fn test_yubikey_slot_allocation() { }

    #[test]
    fn test_yubikey_key_generation() { }

    #[test]
    fn test_yubikey_certificate_storage() { }
}
```

## Phase 3: Integration Tests (Week 3)

### 3.1 Cross-Domain Integration
```rust
#[cfg(test)]
mod integration_tests {
    // Organization -> Person -> Location flow
    #[test]
    fn test_complete_domain_setup() { }

    // Key ownership through org hierarchy
    #[test]
    fn test_key_ownership_hierarchy() { }

    // Delegation chain validation
    #[test]
    fn test_delegation_chain() { }

    // NATS subject generation from domain
    #[test]
    fn test_nats_subject_generation() { }
}
```

### 3.2 End-to-End Scenarios
```rust
#[cfg(test)]
mod e2e_tests {
    // Complete bootstrap scenario
    #[test]
    fn test_bootstrap_new_organization() {
        // 1. Create org structure
        // 2. Add people with roles
        // 3. Assign YubiKeys
        // 4. Generate all keys
        // 5. Project to storage
        // 6. Verify manifest
    }

    // Key rotation scenario
    #[test]
    fn test_key_rotation_workflow() { }

    // Disaster recovery scenario
    #[test]
    fn test_restore_from_backup() { }
}
```

## Phase 4: Performance and Property-Based Tests (Week 4)

### 4.1 Performance Tests
```rust
#[cfg(test)]
mod performance_tests {
    use criterion::{black_box, criterion_group, Criterion};

    fn benchmark_key_generation(c: &mut Criterion) {
        c.bench_function("generate_rsa_4096", |b| {
            b.iter(|| generate_rsa_key(black_box(4096)))
        });
    }

    fn benchmark_event_projection(c: &mut Criterion) {
        // Benchmark projection of 10k events
    }
}
```

### 4.2 Property-Based Tests
```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_event_ordering_preserved(
            events in prop::collection::vec(event_strategy(), 1..100)
        ) {
            // Verify event ordering is maintained
        }

        #[test]
        fn test_delegation_transitivity(
            delegations in delegation_chain_strategy()
        ) {
            // Verify delegation chains are transitive
        }
    }
}
```

## Implementation Roadmap

### Week 1: Domain Module Fixes
- Day 1-2: Fix cim-domain-organization tests
- Day 3-4: Fix cim-domain-person tests
- Day 5: Fix cim-domain-location tests

### Week 2: Core cim-keys Tests
- Day 1: Domain bootstrap tests
- Day 2: Cryptographic operation tests
- Day 3: Event sourcing tests
- Day 4: Storage/projection tests
- Day 5: YubiKey mock tests

### Week 3: Integration Testing
- Day 1-2: Cross-domain integration
- Day 3-4: End-to-end scenarios
- Day 5: Test documentation

### Week 4: Advanced Testing
- Day 1-2: Performance benchmarks
- Day 3-4: Property-based tests
- Day 5: CI/CD pipeline setup

## Test Infrastructure Requirements

### 1. Mock Services
```toml
[dev-dependencies]
mockall = "0.13"
fake = "2.10"
tempfile = "3.14"
```

### 2. Test Fixtures
```
tests/fixtures/
├── domain-bootstrap.json
├── sample-org.json
├── sample-people.json
├── mock-yubikeys.json
└── expected-projections/
    ├── manifest.json
    └── events/
```

### 3. Test Utilities
```rust
// tests/common/mod.rs
pub mod builders {
    pub fn organization_builder() -> OrganizationBuilder { }
    pub fn person_builder() -> PersonBuilder { }
    pub fn yubikey_mock() -> MockYubiKey { }
}

pub mod assertions {
    pub fn assert_valid_event_chain(events: &[Event]) { }
    pub fn assert_valid_projection(projection: &Projection) { }
}
```

## Success Criteria

### Coverage Goals
- Domain modules: ≥80% code coverage
- cim-keys core: ≥90% code coverage
- Integration tests: All critical paths covered

### Quality Metrics
- All tests pass in CI/CD
- No flaky tests
- Test execution < 60 seconds (excluding performance tests)
- Property tests run 100+ cases each

### Functional Guarantees
1. **Domain Bootstrap**: Can load and validate domain configuration
2. **Key Generation**: All key types generate correctly
3. **Event Sourcing**: Events maintain consistency and ordering
4. **Storage**: Projections are deterministic and recoverable
5. **Integration**: Domain modules integrate seamlessly
6. **YubiKey**: Operations work with real hardware and mocks
7. **NATS**: Correct subject generation and message routing

## Risk Mitigation

### Technical Risks
1. **YubiKey Hardware**: Use mocks for CI, real hardware for manual tests
2. **Encrypted Storage**: Use temp directories with mock encryption
3. **Network Dependencies**: All tests run offline
4. **Platform Differences**: Test on Linux, macOS, Windows

### Process Risks
1. **Time Constraints**: Prioritize critical path tests
2. **API Changes**: Version lock dependencies during testing
3. **Flaky Tests**: Implement retry logic and deterministic seeds

## Continuous Improvement

### Monitoring
- Track test execution times
- Monitor coverage trends
- Log flaky test occurrences

### Documentation
- Maintain test documentation
- Create test writing guidelines
- Document mock behaviors

### Review Process
- Code review all test changes
- Quarterly test effectiveness review
- Annual test strategy assessment

## Conclusion

This comprehensive test plan ensures that:
1. All domain modules have reliable, passing tests
2. cim-keys core functionality is thoroughly validated
3. Integration between components is verified
4. The system can serve as a reliable genesis point for CIM infrastructures

The phased approach allows for incremental improvement while maintaining development velocity. By the end of Week 4, we will have a robust test suite that guarantees the functionality specified in cim-keys.