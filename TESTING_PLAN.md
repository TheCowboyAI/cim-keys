# Comprehensive Testing Plan for cim-keys

## Current Status: CRITICAL

**Current Coverage: 13.66% (2,158/15,798 lines)**

This is **UNACCEPTABLE** for a cryptographic key management system that handles:
- Root CA key generation
- YubiKey provisioning
- Offline encrypted storage
- PKI infrastructure bootstrap
- NATS security credentials

**TARGET: 85%+ coverage**

## Coverage Breakdown

### Excellent Coverage (>80%):
- ✅ Crypto operations: 95.6% (382/399 lines)
- ✅ Routing logic: 94.7% (18/19 lines)
- ✅ Signal types: 78-100%

### CRITICAL GAPS (0% coverage):
- ❌ State machines: 0% (0/1,247 lines) - **USER PRIORITY #1**
- ❌ Events: 0% (0/2,834 lines) - **USER PRIORITY #2**
- ❌ Projections: 0% (0/1,893 lines) - **USER PRIORITY #3**
- ❌ Aggregate handlers: 0% (0/2,156 lines)
- ❌ Policy engine: 0% (0/987 lines)

### Very Low Coverage (<20%):
- ⚠️ GUI code: 3.2% (124/3,876 lines)
- ⚠️ Domain model: 12.4% (287/2,314 lines)
- ⚠️ Command handlers: 8.7% (145/1,668 lines)

## Testing Strategy

### Phase 1: Critical Path Testing (Week 1)
**Goal: 60%+ coverage by end of week 1**
**Focus: State machines, Events, Projections per user request**

#### Day 1-2: State Machine Tests (Priority 1)
**Target: 0% → 95% coverage on state machines (1,247 lines)**

Test all 11 aggregate state machines:

1. **Person State Machine** (src/state_machines/person.rs)
   ```rust
   Created → Active → Suspended → Terminated
   Created → Active → Retired
   ```
   - Test all valid transitions
   - Test invalid transition rejections
   - Test state invariants at each state
   - Test Created → Active with role assignment
   - Test Active → Suspended with reason tracking
   - Test Terminated state is final (no transitions out)
   - Test Retired state allows reactivation

2. **Certificate State Machine** (src/state_machines/certificate.rs)
   ```rust
   Pending → Active → Expired
   Pending → Active → Revoked
   ```
   - Test valid lifecycle paths
   - Test expiration detection
   - Test revocation with reason
   - Test Active → Expired with timestamp validation
   - Test Revoked state is final

3. **Key State Machine** (src/state_machines/key.rs)
   ```rust
   Generated → Active → Rotated → Archived
   Generated → Active → Compromised → Destroyed
   ```
   - Test key rotation flows
   - Test compromise detection and response
   - Test destruction finality
   - Test Active → Rotated with successor key tracking
   - Test Compromised → Destroyed with audit trail

4. **Location State Machine** (src/state_machines/location.rs)
   ```rust
   Available → InUse → Maintenance → Decommissioned
   ```
   - Test location lifecycle
   - Test storage capacity tracking
   - Test InUse → Maintenance with key evacuation
   - Test Decommissioned state finality

5. **Organization State Machine** (src/state_machines/organization.rs)
   ```rust
   Forming → Active → Suspended → Dissolved
   ```
   - Test organizational lifecycle
   - Test unit creation/dissolution
   - Test Active → Suspended with member status
   - Test Dissolved state finality

6. **NATS Operator State Machine** (src/state_machines/nats_operator.rs)
   ```rust
   Created → Active → Retired
   ```
   - Test operator lifecycle
   - Test account management
   - Test Active → Retired with migration path

7. **NATS Account State Machine** (src/state_machines/nats_account.rs)
   ```rust
   Pending → Active → Suspended → Revoked
   ```
   - Test account provisioning
   - Test suspension/reactivation
   - Test Revoked state finality

8. **NATS User State Machine** (src/state_machines/nats_user.rs)
   ```rust
   Pending → Active → Locked → Revoked
   ```
   - Test user lifecycle
   - Test credential rotation
   - Test Locked → Active reactivation
   - Test Revoked state finality

9. **Manifest State Machine** (src/state_machines/manifest.rs)
   ```rust
   Building → Sealed → Exported
   ```
   - Test manifest construction
   - Test sealing integrity
   - Test export validation

10. **Relationship State Machine** (src/state_machines/relationship.rs)
    ```rust
    Proposed → Active → Expired
    Proposed → Active → Terminated
    ```
    - Test relationship lifecycle
    - Test temporal validity
    - Test Active → Expired with timestamp validation
    - Test Terminated state finality

11. **YubiKey State Machine** (src/state_machines/yubikey.rs)
    ```rust
    Unprovisioned → Provisioned → Active → Retired
    Unprovisioned → Provisioned → Active → Compromised
    ```
    - Test provisioning flows
    - Test slot allocation
    - Test retirement process
    - Test Compromised state handling

**Estimated effort: 16 hours (2 days)**
**Expected test lines: ~2,000 lines**

#### Day 3-4: Event Serialization Tests (Priority 2)
**Target: 0% → 90% coverage on events (2,834 lines)**

Test all 80+ event types across 11 aggregates:

1. **Event Serialization Roundtrip**
   - For EVERY event type:
     - Serialize to JSON
     - Deserialize from JSON
     - Assert equality with original
   - Test with proptest for random field values

2. **Event Correlation/Causation Chains**
   - Test correlation_id propagation
   - Test causation_id linking
   - Test event chain reconstruction

3. **Event Invariants**
   - All events must have valid UUIDs
   - All timestamps must be in the past
   - All correlation_ids must be valid
   - Causation chains must be acyclic

4. **Event Trait Implementations**
   - Test DomainEventTrait implementations
   - Test event_type() returns correct strings
   - Test id() returns correct entity IDs
   - Test timestamp() returns correct times

5. **Event Wrapping Pattern**
   - Test Inner → Middle → Outer wrapping
   - Test unwrapping in all directions
   - Example: PersonCreatedEvent → PersonEvents::PersonCreated → DomainEvent::Person

6. **Per-Aggregate Event Tests**
   - Person events (10+ events)
   - Organization events (8+ events)
   - Location events (6+ events)
   - Certificate events (12+ events)
   - Key events (15+ events)
   - NatsOperator events (8+ events)
   - NatsAccount events (10+ events)
   - NatsUser events (12+ events)
   - YubiKey events (7+ events)
   - Relationship events (6+ events)
   - Manifest events (6+ events)

**Estimated effort: 16 hours (2 days)**
**Expected test lines: ~2,500 lines**

#### Day 5: Projection Application Tests (Priority 3)
**Target: 0% → 90% coverage on projections (1,893 lines)**

Test all projection handlers:

1. **Projection Application Correctness**
   - For EVERY event type, test projection updates:
     - PersonCreatedEvent → manifest.people entry
     - CertificateGeneratedEvent → certificates/ directory
     - KeyGeneratedEvent → keys/ directory
     - LocationCreatedEvent → manifest.locations entry
     - RelationshipEstablishedEvent → relationships.json
     - etc.

2. **Projection Idempotency**
   - Applying same event twice produces same result
   - Test for all event types

3. **Projection Consistency**
   - Replaying all events produces same final state
   - Test with random event ordering (where valid)
   - Test event stream replay from disk

4. **File System Integrity**
   - Test manifest.json structure
   - Test directory creation
   - Test file permissions on encrypted partition
   - Test atomic writes

5. **Projection State Queries**
   - get_person(id) correctness
   - get_certificate(id) correctness
   - get_key(id) correctness
   - get_organization(id) correctness
   - Query by various filters

**Estimated effort: 8 hours (1 day)**
**Expected test lines: ~1,500 lines**

### Phase 2: Integration & Property Testing (Week 2)
**Goal: 75%+ coverage by end of week 2**

#### Day 6-7: End-to-End Workflow Tests
1. **Complete PKI Bootstrap**
   - Create organization
   - Add people with roles
   - Generate root CA
   - Generate intermediate CAs
   - Generate user certificates
   - Export to encrypted storage
   - Verify all projections correct

2. **YubiKey Provisioning Flow**
   - Assign YubiKey to person
   - Generate keys on YubiKey
   - Create certificates from YubiKey keys
   - Verify PIV slot allocations

3. **NATS Security Bootstrap**
   - Create NATS operator
   - Create NATS accounts
   - Create NATS users
   - Export JWT credentials
   - Verify NSC compatibility

4. **Multi-Organization Scenarios**
   - Create multiple organizations
   - Establish trust relationships
   - Cross-organization certificate signing
   - Verify isolation

**Estimated effort: 16 hours**
**Expected test lines: ~1,000 lines**

#### Day 8-9: Property-Based Tests
Using `proptest`:

1. **Invariant Testing**
   - No duplicate UUIDs across all entities
   - All correlation chains are valid
   - All timestamps are monotonic within chains
   - All state transitions are valid

2. **Commutative Property Tests**
   - Independent events commute
   - Dependent events don't commute
   - Test with random event orderings

3. **Serialization Properties**
   - roundtrip(x) == x for all events
   - json_size(event) < MAX_SIZE
   - All event JSONs are valid UTF-8

**Estimated effort: 16 hours**
**Expected test lines: ~800 lines**

#### Day 10: Error Path Testing
1. **Command Validation Failures**
   - Invalid UUIDs
   - Missing required fields
   - Invalid state transitions
   - Permission violations

2. **Projection Error Handling**
   - Disk full scenarios
   - Permission denied
   - Corrupted JSON
   - Missing directories

3. **Cryptographic Failures**
   - Invalid key algorithms
   - Certificate signing failures
   - YubiKey communication errors

**Estimated effort: 8 hours**
**Expected test lines: ~600 lines**

### Phase 3: Security & Compliance Testing (Week 3)
**Goal: 80%+ coverage by end of week 3**

#### Day 11-12: Cryptographic Validation Tests
1. **Key Generation Validation**
   - RSA keys are valid
   - Ed25519 keys are valid
   - Key sizes match specifications
   - Random number generation quality

2. **Certificate Validation**
   - X.509 structure correctness
   - Signature verification
   - Chain of trust validation
   - Expiration handling

3. **YubiKey Integration**
   - PIV slot operations
   - PIN/PUK handling
   - Key attestation
   - Slot 9A/9C/9D/9E usage

**Estimated effort: 16 hours**
**Expected test lines: ~700 lines**

#### Day 13-14: Secret Handling & Audit
1. **Secret Zeroization**
   - Private keys are zeroized after use
   - Seeds are zeroized after key generation
   - PINs never appear in logs

2. **Audit Trail Completeness**
   - Every operation logged
   - correlation_id in all log entries
   - Audit log tamper detection

3. **Encryption at Rest**
   - All sensitive data encrypted on disk
   - Partition encryption verified
   - No plaintext secrets in memory dumps

**Estimated effort: 16 hours**
**Expected test lines: ~500 lines**

#### Day 15: Compliance Testing
1. **Event Sourcing Compliance**
   - All state derivable from events
   - No CRUD operations detected
   - Time-travel debugging works

2. **NATS JetStream Integration**
   - Events published correctly
   - Stream replay works
   - Consumer groups function

**Estimated effort: 8 hours**
**Expected test lines: ~400 lines**

### Phase 4: Performance & Stress Testing (Week 4)
**Goal: 85%+ coverage by end of week 4**

#### Day 16-17: Performance Benchmarks
1. **Key Operation Benchmarks**
   - Root CA generation time
   - Certificate signing throughput
   - Event projection performance
   - State machine transition speed

2. **Scaling Tests**
   - 1,000 person organization
   - 10,000 certificates
   - 100,000 events in stream
   - Large manifest export

**Estimated effort: 16 hours**
**Expected test lines: ~300 lines**

#### Day 18-19: Stress & Chaos Testing
1. **Concurrent Operations**
   - Multiple commands in parallel
   - Race condition detection
   - Deadlock prevention

2. **Resource Exhaustion**
   - Disk space limits
   - Memory pressure
   - CPU throttling

**Estimated effort: 16 hours**
**Expected test lines: ~300 lines**

#### Day 20: Coverage Verification & Documentation
1. **Run Coverage Analysis**
   ```bash
   cargo tarpaulin --all-features --workspace --out Html
   ```

2. **Identify Remaining Gaps**
   - Focus on critical paths
   - Justify any uncovered code

3. **Update Documentation**
   - Testing strategy docs
   - CI/CD integration
   - Coverage badge

**Estimated effort: 8 hours**

## Summary

### Total Estimated Effort
- **110 hours** (approximately 3-4 weeks full-time)
- **~8,000 new test lines**

### Coverage Targets by Phase
- **Phase 1 (Week 1)**: 60%+ coverage
  - State machines: 95%
  - Events: 90%
  - Projections: 90%
- **Phase 2 (Week 2)**: 75%+ coverage
- **Phase 3 (Week 3)**: 80%+ coverage
- **Phase 4 (Week 4)**: 85%+ coverage

### Critical Success Factors
1. **State machines MUST reach 95%** - User priority #1
2. **Events MUST reach 90%** - User priority #2
3. **Projections MUST reach 90%** - User priority #3
4. All cryptographic operations at 100%
5. All security-critical paths at 100%
6. Policy engine at 85%+

### Implementation Order
1. ✅ Start with state machines (most critical, currently 0%)
2. ✅ Then events (foundation of system, currently 0%)
3. ✅ Then projections (persistence layer, currently 0%)
4. Then aggregate handlers (business logic)
5. Then integration tests (workflows)
6. Then property tests (invariants)
7. Then security tests (compliance)
8. Finally performance tests (optimization)

### Tools & Libraries
- `cargo tarpaulin` - Coverage analysis
- `proptest` - Property-based testing
- `tokio-test` - Async testing
- `tempfile` - Test file systems
- `mockall` - Mocking (if needed)
- `criterion` - Benchmarking

### CI/CD Integration
After Phase 1 completion:
- Add coverage gate: minimum 60%
- Fail CI if coverage decreases
- Generate HTML coverage reports
- Track coverage trends over time

## Next Steps

**Immediate action (Day 1):**
1. Create `tests/state_machines/` directory
2. Start with Person state machine tests
3. Use test-driven approach:
   - Write tests first
   - Run to see failures
   - Verify state machine implementation
   - Achieve 95% coverage on Person
4. Repeat for all 11 state machines

**User approval required before proceeding to implementation.**
