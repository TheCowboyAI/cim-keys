# Copyright (c) 2025 - Cowboy AI, LLC.

Feature: Delegation Revocation Cascade
  As a security administrator
  I need delegation revocations to cascade to dependent delegations
  So that revoked authority doesn't persist through transitive delegations

  Background:
    Given an organization "CowboyAI" with PKI infrastructure
    And a person "Alice" with full PKI administrator permissions
    And correlation tracking is enabled for all events

  # ============================================================================
  # Basic Delegation Scenarios
  # ============================================================================

  Scenario: Single delegation creation
    Given "Alice" has key-generation permissions
    When "Alice" delegates key-generation to "Bob"
    Then a DelegationCreated event should be emitted
    And the event should have a unique delegation_id
    And the event should reference Alice as the delegator
    And the event should reference Bob as the delegate
    And "Bob" should have key-generation permissions

  Scenario: Delegation with limited scope
    Given "Alice" has full PKI administrator permissions
    When "Alice" delegates only key-generation to "Bob"
    Then "Bob" should have key-generation permissions
    And "Bob" should NOT have certificate-signing permissions
    And "Bob" should NOT have key-revocation permissions

  # ============================================================================
  # Transitive Delegation Scenarios
  # ============================================================================

  Scenario: Two-level delegation chain
    Given "Alice" has full PKI administrator permissions
    And "Alice" delegates key-generation to "Bob"
    When "Bob" sub-delegates key-generation to "Charlie"
    Then a DelegationCreated event should be emitted for Bob→Charlie
    And the delegation should reference Bob→Charlie derives from Alice→Bob
    And "Charlie" should have key-generation permissions

  Scenario: Three-level delegation chain
    Given "Alice" has full PKI administrator permissions
    And "Alice" delegates key-generation to "Bob"
    And "Bob" sub-delegates key-generation to "Charlie"
    When "Charlie" sub-delegates key-generation to "Dave"
    Then "Dave" should have key-generation permissions
    And the delegation chain should be: Alice → Bob → Charlie → Dave

  Scenario: Cannot delegate more than you have
    Given "Alice" has full PKI administrator permissions
    And "Alice" delegates only key-generation to "Bob"
    When "Bob" attempts to sub-delegate certificate-signing to "Charlie"
    Then the delegation should be rejected
    And an error should indicate "InsufficientDelegatorPermissions"

  # ============================================================================
  # Revocation Cascade Scenarios
  # ============================================================================

  Scenario: Single revocation - no cascade needed
    Given "Alice" delegates key-generation to "Bob"
    And there are no further sub-delegations from "Bob"
    When "Alice" revokes the delegation to "Bob"
    Then a DelegationRevoked event should be emitted
    And exactly one revocation event should be emitted
    And "Bob" should no longer have key-generation permissions

  Scenario: Two-level cascade revocation
    Given "Alice" delegates key-generation to "Bob"
    And "Bob" sub-delegates key-generation to "Charlie"
    When "Alice" revokes the delegation to "Bob"
    Then a DelegationRevoked event should be emitted for Alice→Bob
    And a CascadeRevoked event should be emitted for Bob→Charlie
    And the causation_id of Charlie's revocation should reference Alice's revocation
    And "Bob" should no longer have key-generation permissions
    And "Charlie" should no longer have key-generation permissions

  Scenario: Three-level cascade revocation
    Given the delegation chain: Alice → Bob → Charlie → Dave
    When "Alice" revokes the delegation to "Bob"
    Then the following events should be emitted in order:
      | event_type     | delegation     | caused_by      |
      | Revoked        | Alice → Bob    | (original)     |
      | CascadeRevoked | Bob → Charlie  | Alice → Bob    |
      | CascadeRevoked | Charlie → Dave | Bob → Charlie  |
    And neither Bob, Charlie, nor Dave should have permissions

  Scenario: Mid-chain revocation
    Given the delegation chain: Alice → Bob → Charlie → Dave
    When "Bob" revokes the delegation to "Charlie"
    Then a DelegationRevoked event should be emitted for Bob→Charlie
    And a CascadeRevoked event should be emitted for Charlie→Dave
    But "Bob" should still have key-generation permissions
    And neither Charlie nor Dave should have permissions

  # ============================================================================
  # Sibling Delegation Scenarios
  # ============================================================================

  Scenario: Revocation does not affect sibling delegations
    Given "Alice" delegates to both "Bob" and "Carol"
    When "Alice" revokes only Bob's delegation
    Then exactly one Revoked event should be emitted
    And "Bob" should no longer have permissions
    And "Carol" should still have permissions

  Scenario: Multiple siblings with their own delegations
    Given "Alice" delegates to "Bob" and "Carol"
    And "Bob" sub-delegates to "Bob-Jr"
    And "Carol" sub-delegates to "Carol-Jr"
    When "Alice" revokes Bob's delegation
    Then Bob and Bob-Jr should lose permissions
    And Carol and Carol-Jr should retain permissions

  # ============================================================================
  # Revocation Reason Propagation
  # ============================================================================

  Scenario: Revocation reason is recorded
    Given "Alice" delegates key-generation to "Bob"
    When "Alice" revokes Bob's delegation with reason "Security policy violation"
    Then the revocation event should include reason "Security policy violation"
    And the audit log should record the reason

  Scenario: Cascade revocation includes parent reason
    Given "Alice" delegates to "Bob" who sub-delegates to "Charlie"
    When "Alice" revokes Bob's delegation with reason "Termination"
    Then Charlie's CascadeRevoked event should reference:
      | field             | value                                            |
      | reason            | Parent delegation revoked: Termination           |
      | caused_by         | (Alice→Bob revocation event id)                  |

  # ============================================================================
  # Event Sourcing Integrity
  # ============================================================================

  Scenario: All revocations have correlation IDs
    Given a complex delegation tree:
      """
      Alice → Bob → [Charlie, Dave]
      Alice → Eve → [Frank]
      """
    When "Alice" revokes Bob's delegation
    Then all revocation events should share the same correlation_id
    And each event should have a unique event_id
    And causation_ids should form a valid tree

  Scenario: Revocation events are idempotent
    Given "Alice" delegates to "Bob"
    And "Alice" has already revoked Bob's delegation
    When "Alice" attempts to revoke Bob's delegation again
    Then no new events should be emitted
    Or an IdempotentOperation response should be returned

  Scenario: Concurrent revocation handling
    Given "Alice" delegates to "Bob" who sub-delegates to "Charlie"
    When "Alice" revokes Bob's delegation
    And simultaneously "Bob" attempts to revoke Charlie's delegation
    Then only the cascade from Alice's revocation should apply
    And no duplicate revocations should occur

  # ============================================================================
  # Permission Verification After Revocation
  # ============================================================================

  Scenario: Revoked delegate cannot perform actions
    Given "Alice" delegates key-generation to "Bob"
    And "Alice" revokes Bob's delegation
    When "Bob" attempts to generate a key
    Then the operation should be denied
    And the error should indicate "Delegation revoked"

  Scenario: Cascade-revoked delegate cannot perform actions
    Given "Alice" delegates to "Bob" who sub-delegates to "Charlie"
    And "Alice" revokes Bob's delegation
    When "Charlie" attempts to generate a key
    Then the operation should be denied
    And the error should indicate "Delegation revoked (cascade)"

  # ============================================================================
  # Edge Cases
  # ============================================================================

  Scenario: Self-delegation is not allowed
    When "Alice" attempts to delegate to herself
    Then the delegation should be rejected
    And an error should indicate "SelfDelegationNotAllowed"

  Scenario: Circular delegation detection
    Given "Alice" delegates to "Bob"
    And "Bob" delegates to "Charlie"
    When "Charlie" attempts to delegate to "Alice"
    Then the delegation should be rejected
    And an error should indicate "CircularDelegationDetected"

  Scenario: Delegation to non-existent person
    When "Alice" attempts to delegate to non-existent person "Ghost"
    Then the delegation should be rejected
    And an error should indicate "DelegateNotFound"

  Scenario: Revocation by non-delegator
    Given "Alice" delegates to "Bob"
    When "Carol" attempts to revoke Alice→Bob delegation
    Then the revocation should be denied
    And an error should indicate "UnauthorizedRevocation"

  # ============================================================================
  # Temporal Aspects
  # ============================================================================

  Scenario: Delegation with expiration
    Given "Alice" delegates to "Bob" with expiration in 30 days
    When 31 days pass
    Then "Bob" should automatically lose permissions
    And a DelegationExpired event should be emitted

  Scenario: Revocation timestamp is recorded
    Given "Alice" delegates to "Bob"
    When "Alice" revokes Bob's delegation
    Then the revocation event should have a timestamp
    And the timestamp should be within 1 second of the current time

  Scenario: Cascade revocations have sequential timestamps
    Given "Alice" delegates to "Bob" who sub-delegates to "Charlie"
    When "Alice" revokes Bob's delegation
    Then CascadeRevoked timestamp should be >= Revoked timestamp
    And all timestamps should be recorded with UTC timezone
