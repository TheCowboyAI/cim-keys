# Copyright (c) 2025 - Cowboy AI, LLC.
# NATS Security Bootstrap Feature Specification
# BDD scenarios for NATS operator, account, and user security provisioning

Feature: NATS Security Bootstrap
  As a CIM infrastructure administrator
  I want to bootstrap NATS security credentials
  So that the messaging infrastructure is properly secured

  Background:
    Given a bootstrapped CIM domain with organization "CIMOrg"
    And a PKI hierarchy has been established
    And an encrypted output partition is mounted

  # ==========================================================================
  # NATS Operator Creation
  # ==========================================================================

  @nats @operator @happy-path
  Scenario: Create NATS operator for organization
    When I create a NATS operator for organization "CIMOrg"
    Then a NatsOperatorCreated event should be emitted
    And an operator NKey pair should be generated
    And the operator JWT should be created
    And the operator should be persisted to the projection

  @nats @operator @signing-key
  Scenario: Generate operator signing keys
    Given a NATS operator "CIMOrg Operator" exists
    When I generate signing keys for the operator
    Then operator signing NKey pairs should be created
    And the signing keys should be recorded
    And the operator JWT should be updated with signing key

  @nats @operator @system-account
  Scenario: Create system account with operator
    When I create a NATS operator with system account
    Then a system account should be automatically created
    And the system account should have system permissions
    And the operator should reference the system account

  # ==========================================================================
  # NATS Account Creation
  # ==========================================================================

  @nats @account @happy-path
  Scenario: Create NATS account for organizational unit
    Given a NATS operator exists
    And organizational unit "Engineering" exists
    When I create a NATS account for "Engineering"
    Then a NatsAccountCreated event should be emitted
    And an account NKey pair should be generated
    And the account JWT should be signed by the operator
    And the account should be associated with "Engineering" unit

  @nats @account @limits
  Scenario: Create account with resource limits
    Given a NATS operator exists
    When I create a NATS account with limits:
      | limit              | value     |
      | max_connections    | 100       |
      | max_payload        | 1048576   |
      | max_subscriptions  | 1000      |
      | max_data           | 10737418240|
    Then the account should have the specified limits
    And the limits should be encoded in the account JWT

  @nats @account @exports
  Scenario: Configure account exports
    Given a NATS account "Services" exists
    When I configure exports:
      | subject           | type    | public |
      | services.api.>    | service | true   |
      | events.internal.> | stream  | false  |
    Then the exports should be recorded in the account JWT
    And public exports should be available to all accounts
    And private exports should require import permission

  @nats @account @imports
  Scenario: Configure account imports
    Given NATS accounts exist:
      | name       |
      | Producer   |
      | Consumer   |
    And "Producer" exports stream "events.>"
    When I configure "Consumer" to import from "Producer"
    Then an import should be created
    And "Consumer" should be able to subscribe to imported subjects

  # ==========================================================================
  # NATS User Creation
  # ==========================================================================

  @nats @user @happy-path
  Scenario: Create NATS user for person
    Given a NATS account "Engineering" exists
    And person "Alice Engineer" exists in "Engineering" unit
    When I create a NATS user for "Alice Engineer"
    Then a NatsUserCreated event should be emitted
    And a user NKey pair should be generated
    And the user JWT should be signed by the account
    And the user should be associated with "Alice Engineer"

  @nats @user @permissions
  Scenario: Create user with specific permissions
    Given a NATS account exists
    When I create a NATS user with permissions:
      | type      | subjects           |
      | publish   | requests.>         |
      | subscribe | responses.>        |
      | subscribe | _INBOX.>           |
    Then the user should have the specified permissions
    And the permissions should be encoded in the user JWT

  @nats @user @deny
  Scenario: Create user with deny rules
    Given a NATS account exists
    When I create a NATS user with:
      | allow_publish | allow_subscribe | deny_publish  |
      | >             | >               | admin.>       |
    Then the user should be able to publish everywhere except admin.>
    And admin.> publications should be rejected

  @nats @user @bearer-token
  Scenario: Generate bearer token for user
    Given a NATS user "Service User" exists
    When I generate a bearer token for the user
    Then a bearer token should be created
    And the token should be usable for NATS authentication
    And the token should have appropriate expiration

  # ==========================================================================
  # Complete NATS Bootstrap
  # ==========================================================================

  @nats @bootstrap @complete
  Scenario: Bootstrap complete NATS security hierarchy
    Given an organization structure:
      | unit        | people                    |
      | Engineering | Alice, Bob                |
      | Security    | Carol                     |
      | Operations  | Dave                      |
    When I execute the complete NATS bootstrap
    Then one operator should be created for the organization
    And accounts should be created for each unit
    And users should be created for each person
    And all JWTs should be properly signed
    And credentials should be exportable

  @nats @bootstrap @idempotent
  Scenario: NATS bootstrap is idempotent
    Given a complete NATS security hierarchy exists
    When I execute the bootstrap again
    Then no duplicate entities should be created
    And existing credentials should remain valid
    And the operation should complete successfully

  # ==========================================================================
  # Credential Export
  # ==========================================================================

  @nats @export @creds
  Scenario: Export user credentials file
    Given a NATS user "Alice" exists
    When I export credentials for "Alice"
    Then a .creds file should be generated
    And the file should contain the user JWT
    And the file should contain the user NKey seed
    And the file should be in standard NATS format

  @nats @export @resolver
  Scenario: Export account resolver configuration
    Given multiple NATS accounts exist
    When I export the resolver configuration
    Then a resolver preload should be generated
    And all account JWTs should be included
    And the configuration should be valid for nats-server

  @nats @export @nsc-compatible
  Scenario: Export is NSC-compatible
    Given a complete NATS security hierarchy
    When I export the entire hierarchy
    Then the export should be importable by NSC
    And `nsc describe operator` should work
    And `nsc describe account` should work for all accounts

  # ==========================================================================
  # Key Rotation Scenarios
  # ==========================================================================

  @nats @rotation @operator
  Scenario: Rotate operator signing key
    Given a NATS operator with signing key SK1
    When I rotate the operator signing key
    Then a new signing key SK2 should be generated
    And SK1 should remain valid for grace period
    And new account JWTs should use SK2
    And a KeyRotated event should be emitted

  @nats @rotation @account
  Scenario: Rotate account signing key
    Given a NATS account with signing key
    When I rotate the account signing key
    Then existing users should remain valid
    And new users should use the new signing key
    And an AccountKeyRotated event should be emitted

  @nats @rotation @user
  Scenario: Revoke and reissue user credentials
    Given NATS user "Compromised User" exists
    When I revoke and reissue credentials for "Compromised User"
    Then the old NKey should be revoked
    And a new NKey should be generated
    And a new JWT should be issued
    And the old credentials should no longer work

  # ==========================================================================
  # Authorization Scenarios
  # ==========================================================================

  @nats @auth @subject-permissions
  Scenario: Verify subject-based authorization
    Given user "Restricted User" can only publish to "allowed.>"
    When "Restricted User" attempts to publish to "denied.test"
    Then the publication should be rejected
    And an authorization error should be logged

  @nats @auth @account-isolation
  Scenario: Verify account isolation
    Given accounts "Account A" and "Account B" exist
    And no imports/exports are configured
    When a user in "Account A" publishes to "internal.message"
    Then users in "Account B" should not receive the message
    And the message should only be visible within "Account A"

  # ==========================================================================
  # Integration with PKI
  # ==========================================================================

  @nats @pki @tls
  Scenario: Generate TLS certificates for NATS servers
    Given a PKI intermediate CA exists
    When I generate NATS server TLS certificates
    Then server certificates should be created
    And certificates should be signed by the intermediate CA
    And certificates should have NATS server SANs

  @nats @pki @mtls
  Scenario: Configure mutual TLS for NATS clients
    Given NATS server TLS is configured
    And user "mTLS User" exists
    When I generate client certificates for "mTLS User"
    Then client certificates should be created
    And the user should be able to connect with mTLS
    And certificate-based authentication should work

  # ==========================================================================
  # Event Sourcing
  # ==========================================================================

  @nats @eventsourcing @audit
  Scenario: Complete audit trail for NATS security
    Given NATS security bootstrap is executed
    When I review the event log
    Then all NATS events should have:
      | field          | present |
      | event_id       | yes     |
      | correlation_id | yes     |
      | causation_id   | yes     |
      | actor_id       | yes     |
      | timestamp      | yes     |
    And events should form a complete causation chain

  @nats @eventsourcing @replay
  Scenario: Reconstruct NATS security from events
    Given a NATS security hierarchy was created
    And all events were persisted
    When I replay events to reconstruct state
    Then all operators should be reconstructed
    And all accounts should be reconstructed
    And all users should be reconstructed
    And relationships should be preserved
