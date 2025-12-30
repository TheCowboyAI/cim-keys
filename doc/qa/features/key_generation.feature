# Copyright (c) 2025 - Cowboy AI, LLC.
# Key Generation Feature Specification
# BDD scenarios for cryptographic key generation in CIM

Feature: Key Generation
  As a CIM security administrator
  I want to generate cryptographic keys for people and services
  So that they can securely authenticate and sign artifacts

  Background:
    Given a bootstrapped CIM domain with organization "SecureCorp"
    And person "Key Owner" exists with Administrator role
    And an encrypted output partition is mounted
    And a secure random source is available

  # ==========================================================================
  # Root CA Key Generation
  # ==========================================================================

  @key @root-ca @happy-path
  Scenario: Generate root CA key pair
    When I generate a root CA key for organization "SecureCorp"
    Then a RootCAKeyGenerated event should be emitted
    And the key should use algorithm "Ed25519" or "ECDSA P-384"
    And the public key should be stored in the projection
    And the private key should be stored on the designated YubiKey
    And a self-signed root certificate should be generated

  @key @root-ca @deterministic
  Scenario: Root CA generation is deterministic from seed
    Given a master seed "0x1234...abcd"
    When I generate a root CA key twice with the same seed
    Then both generations should produce identical key material
    And the key fingerprints should match exactly

  @key @root-ca @constraints
  Scenario: Root CA certificate has correct constraints
    When I generate a root CA key
    Then the certificate should have:
      | constraint          | value          |
      | Basic Constraints   | CA:TRUE        |
      | Key Usage           | keyCertSign    |
      | Key Usage           | cRLSign        |
      | Path Length         | 2              |
    And the validity period should be 10 years

  # ==========================================================================
  # Intermediate CA Key Generation
  # ==========================================================================

  @key @intermediate-ca @happy-path
  Scenario: Generate intermediate CA signed by root
    Given a root CA exists for organization "SecureCorp"
    When I generate an intermediate CA for unit "Engineering"
    Then an IntermediateCAKeyGenerated event should be emitted
    And the intermediate certificate should be signed by the root CA
    And the issuer should be the root CA subject
    And the certificate chain should be valid

  @key @intermediate-ca @constraints
  Scenario: Intermediate CA has restricted constraints
    Given a root CA exists
    When I generate an intermediate CA
    Then the certificate should have:
      | constraint          | value          |
      | Basic Constraints   | CA:TRUE        |
      | Key Usage           | keyCertSign    |
      | Key Usage           | cRLSign        |
      | Path Length         | 1              |
    And the validity period should not exceed the root CA

  @key @intermediate-ca @per-unit
  Scenario: Generate intermediate CA per organizational unit
    Given a root CA exists
    And organizational units exist:
      | name        |
      | Engineering |
      | Security    |
      | Operations  |
    When I generate intermediate CAs for each unit
    Then each unit should have its own intermediate CA
    And all intermediate CAs should chain to the root
    And each should have unit-specific subject DN

  # ==========================================================================
  # Personal Key Generation
  # ==========================================================================

  @key @personal @happy-path
  Scenario: Generate personal authentication key
    Given person "Alice Developer" exists
    And an intermediate CA exists for their unit
    When I generate a personal key for "Alice Developer" with purpose "authentication"
    Then a PersonalKeyGenerated event should be emitted
    And the key should be associated with "Alice Developer"
    And a leaf certificate should be signed by the intermediate CA
    And the certificate should contain the person's email as SAN

  @key @personal @multiple-purposes
  Scenario: Generate multiple keys for different purposes
    Given person "Bob Signer" exists
    When I generate keys for purposes:
      | purpose        |
      | authentication |
      | signing        |
      | encryption     |
    Then separate keys should be generated for each purpose
    And each key should have appropriate key usage extensions
    And all keys should be associated with "Bob Signer"

  @key @personal @ssh
  Scenario: Generate SSH key pair
    Given person "SSH User" exists
    When I generate an SSH key for "SSH User"
    Then an SSHKeyGenerated event should be emitted
    And the public key should be in OpenSSH format
    And the key should be associated with the person
    And the comment should include the person's email

  @key @personal @gpg
  Scenario: Generate GPG key pair
    Given person "GPG User" exists
    When I generate a GPG key for "GPG User"
    Then a GPGKeyGenerated event should be emitted
    And the key should have the person's name and email as UID
    And subkeys should be generated for signing and encryption
    And the key should be exportable in armored format

  # ==========================================================================
  # Service Account Key Generation
  # ==========================================================================

  @key @service @happy-path
  Scenario: Generate service account key
    Given a service account "api-gateway" exists
    When I generate a key for service "api-gateway"
    Then a ServiceKeyGenerated event should be emitted
    And the certificate should have DNS SANs for the service
    And the certificate should not have personal email SANs

  @key @service @mtls
  Scenario: Generate mTLS client certificate for service
    Given service "backend-service" needs to authenticate to "api-gateway"
    When I generate an mTLS client certificate for "backend-service"
    Then the certificate should have clientAuth extended key usage
    And the certificate should be valid for mTLS handshakes

  # ==========================================================================
  # Key Derivation Scenarios
  # ==========================================================================

  @key @derivation @hierarchical
  Scenario: Derive keys hierarchically from master seed
    Given a master seed for organization "SecureCorp"
    When I derive keys for the hierarchy:
      | level        | path                    |
      | root         | m/0'                    |
      | intermediate | m/0'/1'                 |
      | personal     | m/0'/1'/person_index'   |
    Then each derived key should be deterministic
    And child keys should not reveal parent private keys
    And the derivation path should be recorded

  @key @derivation @backup-recovery
  Scenario: Recover keys from backup seed
    Given keys were previously generated from seed "backup_seed_123"
    And the original keys are lost
    When I recover keys using the same seed
    Then all recovered keys should match the originals
    And certificates can be re-issued with same public keys

  # ==========================================================================
  # Key Storage Scenarios
  # ==========================================================================

  @key @storage @yubikey
  Scenario: Store generated key on YubiKey
    Given a YubiKey with serial "12345678" is connected
    And PIV slot 9A is available
    When I generate a key and store it on the YubiKey
    Then the private key should be written to slot 9A
    And the public key should be recorded in projection
    And the key should not be exportable from the device

  @key @storage @projection
  Scenario: Store key metadata in projection
    When I generate any key
    Then the following should be stored in projection:
      | field           | stored   |
      | key_id          | yes      |
      | public_key      | yes      |
      | algorithm       | yes      |
      | created_at      | yes      |
      | owner_id        | yes      |
      | purpose         | yes      |
      | private_key     | NO       |
    And the private key should never touch the projection

  # ==========================================================================
  # Validation Scenarios
  # ==========================================================================

  @key @validation @duplicate
  Scenario: Prevent duplicate key generation for same purpose
    Given person "Duplicate Key Person" already has an authentication key
    When I attempt to generate another authentication key for them
    Then a KeyAlreadyExistsError should be returned
    And no new key should be generated

  @key @validation @no-yubikey
  Scenario: Fail gracefully when YubiKey not available
    Given no YubiKey is connected
    When I attempt to generate a key requiring YubiKey storage
    Then a YubiKeyNotAvailableError should be returned
    And helpful instructions should be provided

  @key @validation @slot-full
  Scenario: Handle full YubiKey slots
    Given all YubiKey PIV slots are occupied
    When I attempt to store a new key
    Then a NoAvailableSlotError should be returned
    And the existing keys should not be affected

  # ==========================================================================
  # Event Sourcing Scenarios
  # ==========================================================================

  @key @eventsourcing @audit
  Scenario: Key generation creates complete audit trail
    When I generate a key
    Then the event should contain:
      | field          | present |
      | event_id       | yes     |
      | correlation_id | yes     |
      | causation_id   | yes     |
      | timestamp      | yes     |
      | actor_id       | yes     |
    And the event should be persisted to the event log

  @key @eventsourcing @chain
  Scenario: Key generation events have correct causation chain
    Given I bootstrap a domain (event E1)
    And I create a person (event E2, caused by E1)
    When I generate a key for the person (event E3)
    Then E3 should have causation_id pointing to E2
    And all events should share the same correlation_id
