# Copyright (c) 2025 - Cowboy AI, LLC.
# YubiKey Provisioning Feature Specification
# BDD scenarios for YubiKey hardware token management in CIM

Feature: YubiKey Provisioning
  As a CIM security administrator
  I want to provision and manage YubiKey hardware tokens
  So that cryptographic keys are stored securely in hardware

  Background:
    Given a bootstrapped CIM domain with organization "SecureCorp"
    And an encrypted output partition is mounted
    And the YubiKey management tooling is available

  # ==========================================================================
  # YubiKey Detection and Registration
  # ==========================================================================

  @yubikey @detection @happy-path
  Scenario: Detect connected YubiKey
    Given a YubiKey with serial "12345678" is physically connected
    When I scan for connected YubiKeys
    Then the YubiKey should be detected
    And the serial number should be "12345678"
    And the firmware version should be reported
    And available PIV slots should be enumerated

  @yubikey @registration @happy-path
  Scenario: Register new YubiKey in domain
    Given a YubiKey with serial "NEW12345" is connected
    And the YubiKey is not yet registered in the domain
    When I register the YubiKey with label "Admin Primary"
    Then a YubiKeyRegistered event should be emitted
    And the YubiKey should be recorded in the projection
    And the registration should include:
      | field          | value       |
      | serial         | NEW12345    |
      | label          | Admin Primary|
      | status         | registered  |

  @yubikey @registration @duplicate
  Scenario: Prevent duplicate YubiKey registration
    Given a YubiKey with serial "EXISTING123" is already registered
    When I attempt to register the same YubiKey again
    Then a YubiKeyAlreadyRegisteredError should be returned
    And the existing registration should remain unchanged

  # ==========================================================================
  # YubiKey Assignment to Person
  # ==========================================================================

  @yubikey @assignment @happy-path
  Scenario: Assign YubiKey to person
    Given a registered YubiKey with serial "ASSIGN123"
    And person "Token Holder" exists
    When I assign the YubiKey to "Token Holder"
    Then a YubiKeyAssignedToPerson event should be emitted
    And "Token Holder" should be recorded as the YubiKey owner
    And the assignment should have a valid timestamp

  @yubikey @assignment @transfer
  Scenario: Transfer YubiKey to different person
    Given YubiKey "TRANSFER123" is assigned to "Original Owner"
    And person "New Owner" exists
    When I transfer the YubiKey from "Original Owner" to "New Owner"
    Then a YubiKeyTransferred event should be emitted
    And "New Owner" should now own the YubiKey
    And "Original Owner" should no longer own it
    And the transfer should be recorded in audit trail

  @yubikey @assignment @revoke
  Scenario: Revoke YubiKey assignment
    Given YubiKey "REVOKE123" is assigned to "Departing Employee"
    When I revoke the YubiKey assignment
    Then a YubiKeyAssignmentRevoked event should be emitted
    And the YubiKey should no longer be assigned to anyone
    And the YubiKey should be marked as "available"

  # ==========================================================================
  # PIV Slot Management
  # ==========================================================================

  @yubikey @slot @provision
  Scenario: Provision key to specific PIV slot
    Given a registered YubiKey with available slot 9A
    And a generated key pair awaiting storage
    When I provision the key to slot 9A
    Then a SlotProvisioned event should be emitted
    And slot 9A should be marked as occupied
    And the slot should reference the stored key

  @yubikey @slot @available-slots
  Scenario Outline: Query available PIV slots
    Given a YubiKey with the following slot status:
      | slot | status   |
      | 9A   | <slot_9a>|
      | 9C   | <slot_9c>|
      | 9D   | <slot_9d>|
      | 9E   | <slot_9e>|
    When I query available slots
    Then available slots should be <available_slots>

    Examples:
      | slot_9a  | slot_9c  | slot_9d  | slot_9e  | available_slots |
      | empty    | empty    | empty    | empty    | 9A, 9C, 9D, 9E  |
      | occupied | empty    | empty    | empty    | 9C, 9D, 9E      |
      | occupied | occupied | occupied | empty    | 9E              |
      | occupied | occupied | occupied | occupied | none            |

  @yubikey @slot @purpose-mapping
  Scenario: PIV slots have designated purposes
    Given a fresh YubiKey is connected
    When I query slot purposes
    Then the slots should have purposes:
      | slot | purpose            |
      | 9A   | Authentication     |
      | 9C   | Digital Signature  |
      | 9D   | Key Management     |
      | 9E   | Card Authentication|

  @yubikey @slot @clear
  Scenario: Clear a PIV slot
    Given slot 9A contains a key
    When I clear slot 9A
    Then a SlotCleared event should be emitted
    And slot 9A should be marked as empty
    And the key should be marked as destroyed

  # ==========================================================================
  # YubiKey Initialization
  # ==========================================================================

  @yubikey @init @factory-reset
  Scenario: Factory reset YubiKey
    Given a YubiKey with existing keys in slots
    When I perform a factory reset
    Then a YubiKeyFactoryReset event should be emitted
    And all PIV slots should be cleared
    And the PIN should be reset to default
    And the PUK should be reset to default
    And the management key should be reset to default

  @yubikey @init @set-pin
  Scenario: Set custom PIN
    Given a YubiKey with default PIN
    When I set the PIN to a custom value
    Then a PINChanged event should be emitted
    And the new PIN should be required for slot access
    And the old PIN should no longer work

  @yubikey @init @set-management-key
  Scenario: Set custom management key
    Given a YubiKey with default management key
    When I set a custom management key
    Then a ManagementKeyChanged event should be emitted
    And the new management key should be required for provisioning
    And the management key should not be stored in projection

  # ==========================================================================
  # YubiKey Status and Health
  # ==========================================================================

  @yubikey @status @query
  Scenario: Query YubiKey status
    Given a provisioned YubiKey with serial "STATUS123"
    When I query the YubiKey status
    Then the status should include:
      | field              | present |
      | serial             | yes     |
      | firmware_version   | yes     |
      | form_factor        | yes     |
      | pin_retries        | yes     |
      | slots_occupied     | yes     |
      | last_used          | yes     |

  @yubikey @status @pin-blocked
  Scenario: Detect blocked PIN
    Given a YubiKey with 0 PIN retries remaining
    When I query the YubiKey status
    Then the PIN should be reported as blocked
    And a warning should be issued
    And PUK unlock should be suggested

  @yubikey @status @attestation
  Scenario: Verify YubiKey attestation
    Given a YubiKey with attestation capability
    When I verify the attestation certificate
    Then the attestation should be validated against Yubico root
    And the key should be confirmed as hardware-generated
    And the serial number should match the reported serial

  # ==========================================================================
  # Multi-YubiKey Scenarios
  # ==========================================================================

  @yubikey @multi @hierarchy
  Scenario: Provision YubiKey hierarchy for organization
    Given YubiKeys are registered:
      | serial    | label           | level    |
      | ROOT001   | Root Authority  | operator |
      | INT001    | Engineering CA  | domain   |
      | USER001   | Alice Token     | user     |
    When I provision the PKI hierarchy to these YubiKeys
    Then the root key should be on ROOT001
    And the intermediate key should be on INT001
    And Alice's personal key should be on USER001
    And each level should have appropriate trust

  @yubikey @multi @backup
  Scenario: Create backup YubiKey for disaster recovery
    Given a primary YubiKey "PRIMARY001" with root CA key
    And a backup YubiKey "BACKUP001" is available
    When I create a backup using key splitting
    Then both YubiKeys should be required to recover the root key
    And neither YubiKey alone should expose the private key

  # ==========================================================================
  # Error Handling Scenarios
  # ==========================================================================

  @yubikey @error @not-connected
  Scenario: Handle YubiKey not connected
    Given no YubiKey is physically connected
    When I attempt any YubiKey operation
    Then a YubiKeyNotConnectedError should be returned
    And helpful troubleshooting steps should be provided

  @yubikey @error @wrong-pin
  Scenario: Handle incorrect PIN entry
    Given a YubiKey is connected
    When I enter an incorrect PIN
    Then an IncorrectPINError should be returned
    And the remaining retry count should be reported
    And the key operation should not proceed

  @yubikey @error @pin-locked
  Scenario: Handle locked PIN
    Given a YubiKey with PIN locked (0 retries)
    When I attempt to use a PIN-protected slot
    Then a PINLockedError should be returned
    And instructions for PUK unlock should be provided

  # ==========================================================================
  # Audit and Event Sourcing
  # ==========================================================================

  @yubikey @audit @complete
  Scenario: Complete audit trail for YubiKey lifecycle
    Given a YubiKey goes through its lifecycle:
      | action             |
      | registered         |
      | assigned to Alice  |
      | key provisioned    |
      | transferred to Bob |
      | key cleared        |
      | deregistered       |
    When I retrieve the YubiKey audit history
    Then all lifecycle events should be present
    And events should be in chronological order
    And each event should have proper causation chain

  @yubikey @eventsourcing @projection
  Scenario: Reconstruct YubiKey state from events
    Given a YubiKey with extensive event history
    When I replay all events
    Then the current projection should be accurately reconstructed
    And slot occupancy should match current physical state
    And ownership should be correctly determined
