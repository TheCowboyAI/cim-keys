# Sprint 28 Retrospective: Saga Coordinator for Multi-Aggregate Workflows

**Date**: 2026-01-03
**Sprint Goal**: Implement JetStream-backed saga execution with KV persistence and recovery

## What We Built

### 1. JetStreamSagaExecutor (`src/domain/nats/saga_executor.rs`)

A complete saga execution framework for multi-aggregate workflows:

- **AsyncSagaExecutor Trait**: Defines the contract for async saga step execution
- **JetStreamSagaExecutor**: Orchestrates saga lifecycle using JetStream KV
- **PersistedSagaState**: Serializable wrapper for saga state with metadata
- **SagaRecovery**: Recovery system for resuming failed/stalled sagas
- **StepExecutionResult**: Typed results for step outcomes (Complete, Compensating, Retry, Error)

### 2. JetStream KV Port Extensions (`src/ports/nats.rs`)

Added KV operations to the JetStreamPort trait:

```rust
async fn kv_get(&self, bucket: &str, key: &str) -> Result<Option<Vec<u8>>, JetStreamError>;
async fn kv_put(&self, bucket: &str, key: &str, value: &[u8]) -> Result<u64, JetStreamError>;
async fn kv_delete(&self, bucket: &str, key: &str) -> Result<(), JetStreamError>;
async fn kv_keys(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, JetStreamError>;
async fn kv_create_bucket(&self, bucket: &str, config: &KvBucketConfig) -> Result<(), JetStreamError>;
```

### 3. JetStreamAdapter KV Implementation (`src/adapters/nats_client.rs`)

Full implementation of KV operations for the NATS client adapter:

- Bucket creation with configurable max_bytes, TTL, replicas
- Key-value get/put/delete operations
- Key listing with prefix filtering
- Proper error handling with KvError variant

### 4. Comprehensive Test Suite (`tests/saga_executor_tests.rs`)

16 tests covering:

- Configuration builder patterns
- PersistedSagaState creation and mutation
- Saga lifecycle (start, load, execute, compensate, delete)
- Event publishing during saga execution
- Active saga listing and filtering

## Architecture Decisions

### 1. KV-Based State Persistence

Sagas are persisted to JetStream KV using the pattern:
```
{saga_type}.{saga_id} -> PersistedSagaState<S>
```

Benefits:
- Automatic replication across cluster
- Point-in-time recovery
- Simple key-based lookup

### 2. Event-Driven Saga Progress

Each saga step publishes events to JetStream:
- `saga.{type}.started` - Initial saga creation
- `saga.{type}.step.{step_name}` - Step completions
- `saga.{type}.compensating` - Compensation triggered
- `saga.{type}.completed` - Final completion
- `saga.{type}.failed` - Terminal failure

### 3. Recovery Strategy

The `SagaRecovery` struct provides:
- Stalled saga detection (configurable timeout)
- Resume with retry limit
- Automatic compensation after max retries

### 4. Separation of Concerns

- **Domain Layer**: `AsyncSagaExecutor` trait defines business logic
- **Infrastructure Layer**: `JetStreamSagaExecutor` handles persistence
- **Port Layer**: `JetStreamPort` abstracts NATS operations

## Metrics

| Metric | Value |
|--------|-------|
| New Files | 1 (saga_executor.rs) |
| Modified Files | 4 |
| New Tests | 16 |
| Total Tests Passing | 788+ |
| Lines of Code | ~600 (saga_executor) |

## What Went Well

1. **Clean Trait Design**: `AsyncSagaExecutor` cleanly separates saga logic from infrastructure
2. **Mock Testing**: `MockJetStreamWithKv` enabled comprehensive testing without real NATS
3. **Error Handling**: Proper error types with `thiserror` for clear error messages
4. **Port Pattern**: KV operations fit naturally into existing JetStreamPort trait

## Challenges and Solutions

### Challenge 1: Field Name Mismatches
The `KvBucketConfig` struct had different field names than I initially implemented.

**Solution**: Read the actual struct definition and use `max_bytes` instead of `max_value_size`.

### Challenge 2: Mock Implementation for Tests
Adding KV methods to the trait required updating all mock implementations.

**Solution**: Added stub implementations to `MockJetStreamPort` in publisher.rs returning sensible defaults.

### Challenge 3: State Serialization
Saga state needs to be serializable with metadata like version and timestamps.

**Solution**: Created `PersistedSagaState<S>` wrapper that adds metadata while preserving type safety.

## Lessons Learned

1. **Check Struct Definitions**: Always verify field names before implementing adapters
2. **Mock All Trait Methods**: When extending a trait, update all mock implementations
3. **Event Sourcing Compatibility**: Saga events integrate naturally with existing event infrastructure
4. **Recovery is Essential**: Production sagas need recovery mechanisms from day one

## Integration with Existing Systems

The saga executor integrates with:
- **Event Publisher**: Uses same JetStreamPort for saga events
- **Event Replay**: Saga events can be replayed for recovery
- **Domain Sagas**: Existing `SagaState` trait from `src/domain/sagas/`

## Next Steps (Sprint 29)

1. **Command Handler Integration**: Connect sagas to command processing pipeline
2. **Saga Repository Pattern**: Abstract saga persistence behind repository trait
3. **Distributed Saga Coordination**: Handle sagas spanning multiple services
4. **Saga Visualization**: Add mermaid diagrams for saga state machines

## Conclusion

Sprint 28 successfully delivered a production-ready saga execution framework. The architecture follows hexagonal patterns, enabling easy testing and future infrastructure changes. The KV-based persistence provides durability while the event-driven progress enables observability and recovery.
