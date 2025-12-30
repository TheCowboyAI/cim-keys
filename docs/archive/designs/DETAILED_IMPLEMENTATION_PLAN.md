# Detailed Implementation Plan: CIM-Keys Test & Functionality Guarantee

## Overview

This document provides a stage-by-stage implementation plan with clear evaluation criteria to ensure all three subdomains work correctly and cim-keys achieves its specified functionality as the genesis point for CIM infrastructures.

---

## STAGE 1: Foundation Repair (Days 1-3)
**Goal:** Fix compilation errors in all domain modules to establish a working baseline

### Stage 1.1: cim-domain-organization Fixes

#### Tasks
1. **Fix Missing Types** (2 hours)
   ```rust
   // Add to src/value_objects/mod.rs
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum SizeCategory {
       Startup,        // 1-10 people
       SmallBusiness,  // 11-50 people
       MediumBusiness, // 51-250 people
       Enterprise,     // 251-1000 people
       MegaCorp,       // 1000+ people
   }
   ```

2. **Update Test Structures** (3 hours)
   - Fix field mismatches in `tests/organization_tests.rs`
   - Update to new `AggregateRoot` trait API
   - Fix `EntityId` usage patterns

3. **Verify Basic Functionality** (1 hour)
   - Run: `cargo test --lib`
   - Run: `cargo test --test organization_tests`

#### Evaluation Criteria
- [ ] All library tests pass (4/4)
- [ ] Integration tests compile without errors
- [ ] At least 10/20 integration tests pass

#### Deliverables
- Fixed `organization_tests.rs`
- Updated value objects
- Test output log showing passes

---

### Stage 1.2: cim-domain-person Fixes

#### Tasks
1. **Fix Command Structures** (2 hours)
   ```rust
   // Fix RegisterComponent in src/commands.rs
   pub struct RegisterComponent {
       pub person_id: PersonId,
       pub component: ComponentData,
       pub registered_by: String, // Add missing field
       pub registration_reason: Option<String>,
   }
   ```

2. **Fix Workflow Manager** (3 hours)
   - Update lifecycle ownership patterns
   - Fix async trait implementations
   - Update state management

3. **Fix Component Store** (2 hours)
   - Fix move/clone issues
   - Update persistence layer

#### Evaluation Criteria
- [ ] Library tests compile
- [ ] At least 30/58 tests pass
- [ ] No compilation errors

#### Deliverables
- Fixed command structures
- Updated workflow manager
- Component store fixes
- Test results showing improvement

---

### Stage 1.3: cim-domain-location Fixes

#### Tasks
1. **Fix Workflow Tests** (2 hours)
   - Remove unnecessary mutability
   - Fix async workflow patterns

2. **Fix Spatial Tests** (2 hours)
   - Update coordinate handling
   - Fix comparison operators

3. **Fix Integration Points** (1 hour)
   - Update NATS subject generation
   - Fix event propagation

#### Evaluation Criteria
- [ ] Library tests compile
- [ ] At least 25/55 tests pass
- [ ] Spatial calculations work correctly

#### Deliverables
- Fixed workflow tests
- Spatial search corrections
- Integration test fixes

---

## STAGE 2: Core Functionality Implementation (Days 4-8)
**Goal:** Implement essential cim-keys functionality with comprehensive tests

### Stage 2.1: Domain Bootstrap System

#### Implementation Tasks
1. **Config Loader** (4 hours)
   ```rust
   pub struct DomainBootstrap {
       config_path: PathBuf,
       validator: ConfigValidator,
   }

   impl DomainBootstrap {
       pub fn load_config(&self) -> Result<DomainConfig, Error> { }
       pub fn validate_structure(&self) -> Result<(), ValidationError> { }
       pub fn create_organization(&self) -> Result<Organization, Error> { }
       pub fn create_people(&self) -> Result<Vec<Person>, Error> { }
   }
   ```

2. **Domain Builder** (4 hours)
   ```rust
   pub struct DomainBuilder {
       organization: Option<Organization>,
       people: Vec<Person>,
       locations: Vec<Location>,
       yubikeys: Vec<YubiKeyAssignment>,
   }

   impl DomainBuilder {
       pub fn build(&self) -> Result<Domain, Error> { }
       pub fn validate_relationships(&self) -> Result<(), Error> { }
   }
   ```

3. **Tests** (3 hours)
   ```rust
   #[test]
   fn test_load_minimal_config() { }

   #[test]
   fn test_load_complex_config() { }

   #[test]
   fn test_validate_relationships() { }

   #[test]
   fn test_circular_dependency_detection() { }
   ```

#### Evaluation Criteria
- [ ] Can load JSON config files
- [ ] Validates organization structure
- [ ] Creates person entities with roles
- [ ] Assigns YubiKeys correctly
- [ ] All 4 bootstrap tests pass

#### Deliverables
- `src/bootstrap/` module
- Config validation logic
- 10+ passing tests

---

### Stage 2.2: Cryptographic Key Generation

#### Implementation Tasks
1. **Key Generator Factory** (6 hours)
   ```rust
   pub enum KeyType {
       RootCA,
       IntermediateCA,
       SSH(SshKeyType),
       TLS,
       GPG,
       NATS(NatsKeyLevel),
   }

   pub struct KeyGenerator {
       yubikey: Option<YubiKeyHandle>,
       software_backend: SoftwareKeyBackend,
   }

   impl KeyGenerator {
       pub async fn generate(&self, key_type: KeyType) -> Result<KeyPair, Error> { }
       pub fn generate_on_yubikey(&self, slot: PIVSlot) -> Result<PublicKey, Error> { }
   }
   ```

2. **Certificate Chain Builder** (4 hours)
   ```rust
   pub struct CertificateChain {
       root_ca: Certificate,
       intermediates: Vec<Certificate>,
   }

   impl CertificateChain {
       pub fn build_chain(&self) -> Result<Vec<Certificate>, Error> { }
       pub fn validate_chain(&self) -> Result<(), ValidationError> { }
   }
   ```

3. **NATS Key Hierarchy** (4 hours)
   ```rust
   pub struct NatsKeyHierarchy {
       operator: OperatorKeyPair,
       accounts: HashMap<String, AccountKeyPair>,
       users: HashMap<String, UserKeyPair>,
   }

   impl NatsKeyHierarchy {
       pub fn generate_hierarchy(&mut self) -> Result<(), Error> { }
       pub fn export_jwts(&self) -> Result<NatsJWTs, Error> { }
   }
   ```

4. **Tests** (4 hours)
   ```rust
   #[test]
   fn test_generate_root_ca() { }

   #[test]
   fn test_certificate_chain_validation() { }

   #[test]
   fn test_ssh_key_formats() { }

   #[test]
   fn test_nats_hierarchy() { }
   ```

#### Evaluation Criteria
- [ ] Generates all key types
- [ ] Certificate chains validate
- [ ] SSH keys in correct format
- [ ] NATS JWTs are valid
- [ ] 15+ crypto tests pass

#### Deliverables
- `src/crypto/` module
- Key generation for all types
- Certificate chain logic
- NATS hierarchy implementation

---

### Stage 2.3: Event Sourcing & Projection

#### Implementation Tasks
1. **Event Store** (4 hours)
   ```rust
   pub struct EventStore {
       events: Vec<DomainEvent>,
       snapshots: HashMap<Uuid, Snapshot>,
   }

   impl EventStore {
       pub fn append(&mut self, event: DomainEvent) -> Result<(), Error> { }
       pub fn replay_from(&self, position: usize) -> Vec<DomainEvent> { }
       pub fn create_snapshot(&self) -> Snapshot { }
   }
   ```

2. **Projection Engine** (6 hours)
   ```rust
   pub struct ProjectionEngine {
       projectors: Vec<Box<dyn Projector>>,
       output_path: PathBuf,
   }

   impl ProjectionEngine {
       pub fn project(&self, events: &[DomainEvent]) -> Result<(), Error> { }
       pub fn write_json(&self, data: &ProjectionData) -> Result<(), Error> { }
       pub fn generate_manifest(&self) -> Result<Manifest, Error> { }
   }
   ```

3. **Storage Layout** (3 hours)
   ```
   output/
   ├── manifest.json          # Master index
   ├── domain/
   │   ├── organization.json  # Org structure
   │   ├── people.json       # People registry
   │   └── locations.json    # Storage locations
   ├── keys/
   │   └── {key-id}/
   │       ├── metadata.json
   │       └── public.pem
   ├── certificates/
   │   ├── root-ca/
   │   └── intermediate/
   ├── events/
   │   └── {date}/
   │       └── {sequence}.json
   └── nats/
       ├── operator.jwt
       └── accounts/
   ```

4. **Tests** (3 hours)
   ```rust
   #[test]
   fn test_event_ordering() { }

   #[test]
   fn test_projection_determinism() { }

   #[test]
   fn test_manifest_generation() { }

   #[test]
   fn test_storage_structure() { }
   ```

#### Evaluation Criteria
- [ ] Events maintain order
- [ ] Projections are deterministic
- [ ] Manifest accurately indexes content
- [ ] Can replay from events
- [ ] 10+ event tests pass

#### Deliverables
- Event store implementation
- Projection engine
- Storage layout implementation
- Manifest generator

---

## STAGE 3: Integration & GUI (Days 9-12)
**Goal:** Integrate all components and ensure GUI functionality

### Stage 3.1: Component Integration

#### Tasks
1. **Domain Module Integration** (4 hours)
   ```rust
   pub struct DomainIntegration {
       organization: cim_domain_organization::Organization,
       people: Vec<cim_domain_person::Person>,
       locations: Vec<cim_domain_location::Location>,
   }

   impl DomainIntegration {
       pub fn link_people_to_org(&mut self) -> Result<(), Error> { }
       pub fn assign_locations(&mut self) -> Result<(), Error> { }
       pub fn validate_consistency(&self) -> Result<(), Error> { }
   }
   ```

2. **Cross-Domain Operations** (3 hours)
   - Person → Organization mapping
   - Location → Key storage assignment
   - Role → Permission validation

3. **Integration Tests** (3 hours)
   ```rust
   #[test]
   fn test_complete_domain_setup() { }

   #[test]
   fn test_key_ownership_chain() { }

   #[test]
   fn test_delegation_validation() { }
   ```

#### Evaluation Criteria
- [ ] Domains integrate without conflicts
- [ ] Cross-references work correctly
- [ ] Permissions cascade properly
- [ ] 5+ integration tests pass

---

### Stage 3.2: GUI Functionality

#### Tasks
1. **Fix Iced 0.13 Compatibility** (4 hours)
   - Update to new Application API
   - Fix container styling
   - Update event handling

2. **Graph Visualization** (6 hours)
   ```rust
   pub struct OrganizationGraphView {
       nodes: HashMap<Uuid, PersonNode>,
       edges: Vec<RelationshipEdge>,
   }

   impl OrganizationGraphView {
       pub fn render(&self) -> Element<Message> { }
       pub fn handle_interaction(&mut self, event: GraphEvent) { }
   }
   ```

3. **Workflow Implementation** (4 hours)
   - Welcome → Load/Create flow
   - Organization setup wizard
   - People & role assignment
   - Key generation workflow
   - Export functionality

#### Evaluation Criteria
- [ ] GUI compiles and runs
- [ ] Graph renders correctly
- [ ] Can complete full workflow
- [ ] Export produces valid output

---

## STAGE 4: YubiKey & Hardware (Days 13-15)
**Goal:** Implement YubiKey support with proper mocking

### Stage 4.1: YubiKey Abstraction

#### Implementation
1. **YubiKey Interface** (4 hours)
   ```rust
   #[async_trait]
   pub trait YubiKeyInterface {
       async fn list_devices(&self) -> Result<Vec<YubiKeyInfo>, Error>;
       async fn connect(&self, serial: &str) -> Result<YubiKeyHandle, Error>;
       async fn generate_key(&self, slot: PIVSlot) -> Result<PublicKey, Error>;
       async fn sign(&self, slot: PIVSlot, data: &[u8]) -> Result<Signature, Error>;
   }
   ```

2. **Mock Implementation** (3 hours)
   ```rust
   pub struct MockYubiKey {
       keys: HashMap<PIVSlot, MockKeyPair>,
   }

   #[async_trait]
   impl YubiKeyInterface for MockYubiKey {
       // Mock implementations
   }
   ```

3. **Real Implementation** (4 hours)
   ```rust
   #[cfg(feature = "yubikey-support")]
   pub struct RealYubiKey {
       device: YubiKeyDevice,
   }
   ```

#### Evaluation Criteria
- [ ] Mock YubiKey works in tests
- [ ] Real YubiKey works when available
- [ ] Graceful fallback when no hardware
- [ ] 10+ YubiKey tests pass

---

## STAGE 5: Validation & Performance (Days 16-18)
**Goal:** Ensure system meets performance and reliability requirements

### Stage 5.1: End-to-End Testing

#### Test Scenarios
1. **Complete Bootstrap** (2 hours)
   ```rust
   #[test]
   fn test_bootstrap_small_org() {
       // 10 people, 1 YubiKey
   }

   #[test]
   fn test_bootstrap_medium_org() {
       // 100 people, 5 YubiKeys
   }

   #[test]
   fn test_bootstrap_large_org() {
       // 1000 people, 20 YubiKeys
   }
   ```

2. **Disaster Recovery** (2 hours)
   ```rust
   #[test]
   fn test_restore_from_events() { }

   #[test]
   fn test_restore_from_manifest() { }
   ```

3. **Key Rotation** (2 hours)
   ```rust
   #[test]
   fn test_rotate_intermediate_ca() { }

   #[test]
   fn test_rotate_user_keys() { }
   ```

#### Evaluation Criteria
- [ ] Can bootstrap orgs of any size
- [ ] Recovery works from events
- [ ] Key rotation maintains consistency
- [ ] All E2E tests pass

---

### Stage 5.2: Performance Optimization

#### Benchmarks
1. **Key Generation Performance**
   ```rust
   #[bench]
   fn bench_generate_rsa_2048(b: &mut Bencher) { }

   #[bench]
   fn bench_generate_ed25519(b: &mut Bencher) { }
   ```

2. **Event Processing Performance**
   ```rust
   #[bench]
   fn bench_project_1000_events(b: &mut Bencher) { }

   #[bench]
   fn bench_replay_10000_events(b: &mut Bencher) { }
   ```

#### Performance Targets
- [ ] RSA-2048 generation < 100ms
- [ ] Ed25519 generation < 10ms
- [ ] 1000 events projection < 1s
- [ ] 10000 events replay < 5s

---

### Stage 5.3: Property-Based Testing

#### Property Tests
```rust
proptest! {
    #[test]
    fn prop_event_ordering(events in vec(event_strategy(), 1..100)) {
        // Events maintain causal ordering
    }

    #[test]
    fn prop_projection_determinism(events in vec(event_strategy(), 1..100)) {
        // Same events produce same projection
    }

    #[test]
    fn prop_delegation_transitivity(chain in delegation_chain_strategy()) {
        // A delegates to B, B to C => A transitively delegates to C
    }
}
```

#### Evaluation Criteria
- [ ] 100+ property test runs pass
- [ ] No counter-examples found
- [ ] Edge cases handled

---

## STAGE 6: Documentation & Deployment (Days 19-20)
**Goal:** Complete documentation and deployment readiness

### Stage 6.1: Documentation

#### Deliverables
1. **API Documentation**
   - All public APIs documented
   - Examples for each major function
   - Error handling guide

2. **User Guide**
   - Installation instructions
   - Configuration guide
   - Workflow walkthroughs

3. **Architecture Documentation**
   - System design
   - Event flow diagrams
   - Security model

#### Evaluation Criteria
- [ ] 100% public API documented
- [ ] All examples compile and run
- [ ] User guide covers all workflows

---

### Stage 6.2: CI/CD Pipeline

#### Setup Tasks
1. **GitHub Actions Workflow**
   ```yaml
   name: Test Suite
   on: [push, pull_request]
   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v2
         - run: cargo test --all-features
         - run: cargo test --no-default-features
   ```

2. **Release Process**
   - Version tagging
   - Binary builds
   - Docker images

#### Evaluation Criteria
- [ ] CI runs on every commit
- [ ] All tests pass in CI
- [ ] Release artifacts build

---

## Success Metrics Summary

### Quantitative Metrics
- **Test Coverage**: ≥80% for domain modules, ≥90% for cim-keys core
- **Test Count**: 100+ tests across all modules
- **Performance**: All benchmarks meet targets
- **Documentation**: 100% public API coverage

### Qualitative Metrics
- **Reliability**: No flaky tests
- **Usability**: GUI intuitive for non-technical users
- **Security**: All cryptographic operations validated
- **Maintainability**: Clear separation of concerns

### Functional Requirements Met
1. ✅ Domain bootstrap from configuration
2. ✅ Complete PKI hierarchy generation
3. ✅ YubiKey integration with fallback
4. ✅ Event-sourced architecture
5. ✅ Deterministic projections
6. ✅ NATS key hierarchy
7. ✅ Offline operation
8. ✅ GUI for management

---

## Risk Management

### Technical Risks
| Risk | Mitigation | Contingency |
|------|------------|-------------|
| YubiKey unavailable | Mock implementation | Software-only mode |
| Platform differences | CI on multiple OS | Platform-specific fixes |
| Crypto library issues | Multiple backends | Fallback implementations |
| Performance issues | Early benchmarking | Async/parallel processing |

### Schedule Risks
| Risk | Mitigation | Contingency |
|------|------------|-------------|
| Scope creep | Fixed requirements | Defer to v2 |
| Integration issues | Early integration | Simplify interfaces |
| Test failures | Incremental fixes | Focus on critical path |

---

## Timeline Summary

**Week 1 (Days 1-5)**: Foundation
- Fix all compilation errors
- Establish baseline functionality

**Week 2 (Days 6-10)**: Core Implementation
- Domain bootstrap
- Cryptographic operations
- Event sourcing

**Week 3 (Days 11-15)**: Integration
- Component integration
- GUI implementation
- YubiKey support

**Week 4 (Days 16-20)**: Validation
- E2E testing
- Performance optimization
- Documentation

---

## Evaluation Checkpoints

### Day 3 Checkpoint
- [ ] All modules compile
- [ ] Basic tests pass
- [ ] Development environment stable

### Day 8 Checkpoint
- [ ] Core functionality implemented
- [ ] 50+ tests passing
- [ ] Key generation working

### Day 12 Checkpoint
- [ ] Integration complete
- [ ] GUI functional
- [ ] 75+ tests passing

### Day 15 Checkpoint
- [ ] YubiKey support working
- [ ] E2E tests passing
- [ ] Performance acceptable

### Day 20 - Final
- [ ] All requirements met
- [ ] 100+ tests passing
- [ ] Documentation complete
- [ ] Ready for production

---

## Conclusion

This detailed plan provides:
1. **Clear stages** with specific tasks
2. **Measurable criteria** for evaluation
3. **Concrete deliverables** at each stage
4. **Risk mitigation** strategies
5. **Realistic timeline** with checkpoints

Following this plan ensures that cim-keys will successfully serve as the genesis point for CIM infrastructures with robust testing and guaranteed functionality.