# Copyright (c) 2025 - Cowboy AI, LLC.
# Person Management Feature Specification
# BDD scenarios for managing people within a CIM domain

Feature: Person Management
  As a CIM administrator
  I want to manage people within the organization
  So that I can assign keys, roles, and permissions appropriately

  Background:
    Given a bootstrapped CIM domain with organization "TechCorp"
    And organizational units exist:
      | name        | type       |
      | Engineering | Department |
      | Security    | Department |
    And an encrypted output partition is mounted

  # ==========================================================================
  # Person Creation Scenarios
  # ==========================================================================

  @person @create @happy-path
  Scenario: Create a new person in the organization
    When I create a person with:
      | name       | email            | role      |
      | John Smith | john@techcorp.com| Developer |
    Then a PersonCreated event should be emitted
    And the person should have a valid UUID v7 identifier
    And the person should be associated with organization "TechCorp"
    And the person should be persisted to the projection

  @person @create @with-unit
  Scenario: Create person assigned to organizational unit
    When I create a person with:
      | name        | email             | role      | unit        |
      | Jane Doe    | jane@techcorp.com | Engineer  | Engineering |
    Then a PersonCreated event should be emitted
    And a PersonAssignedToUnit event should be emitted
    And the person should be a member of "Engineering" unit

  @person @create @multiple-units
  Scenario: Create person assigned to multiple units
    Given I create a person "Alex Multi" with email "alex@techcorp.com"
    When I assign the person to units:
      | unit        |
      | Engineering |
      | Security    |
    Then PersonAssignedToUnit events should be emitted for each unit
    And the person should be a member of both units

  # ==========================================================================
  # Person Update Scenarios
  # ==========================================================================

  @person @update @name
  Scenario: Update person's name
    Given a person "Original Name" exists with email "person@techcorp.com"
    When I update the person's name to "Updated Name"
    Then a PersonNameUpdated event should be emitted
    And the projection should reflect the new name
    And the person's other attributes should remain unchanged

  @person @update @role
  Scenario: Update person's role
    Given a person "Role Changer" exists with role "Developer"
    When I update the person's role to "Senior Developer"
    Then a PersonRoleUpdated event should be emitted
    And the person's role should be "Senior Developer"

  @person @update @email
  Scenario: Update person's email address
    Given a person "Email Updater" exists with email "old@techcorp.com"
    When I update the person's email to "new@techcorp.com"
    Then a PersonEmailUpdated event should be emitted
    And the projection should reflect the new email

  # ==========================================================================
  # Person Relationship Scenarios
  # ==========================================================================

  @person @relationship @delegation
  Scenario: Establish key delegation between people
    Given person "Delegator" exists with Administrator role
    And person "Delegate" exists with Developer role
    When "Delegator" delegates key signing authority to "Delegate"
    Then a DelegationEstablished event should be emitted
    And "Delegate" should have delegated signing authority
    And the delegation should have temporal bounds

  @person @relationship @supervisor
  Scenario: Establish supervisor relationship
    Given person "Manager" exists in "Engineering" unit
    And person "Developer" exists in "Engineering" unit
    When I establish "Manager" as supervisor of "Developer"
    Then a SupervisorRelationshipEstablished event should be emitted
    And "Developer" should have "Manager" as supervisor

  @person @relationship @revoke
  Scenario: Revoke delegation
    Given person "Delegator" has delegated authority to "Delegate"
    When "Delegator" revokes the delegation
    Then a DelegationRevoked event should be emitted
    And "Delegate" should no longer have delegated authority

  # ==========================================================================
  # Person Removal Scenarios
  # ==========================================================================

  @person @remove @soft-delete
  Scenario: Deactivate a person (soft delete)
    Given person "Departing Employee" exists
    When I deactivate the person
    Then a PersonDeactivated event should be emitted
    And the person should be marked as inactive
    And the person's keys should be marked for revocation
    And the person should still exist in the audit trail

  @person @remove @key-transfer
  Scenario: Transfer keys before deactivation
    Given person "Key Holder" exists with assigned keys
    And person "Key Recipient" exists
    When I transfer keys from "Key Holder" to "Key Recipient"
    And I deactivate "Key Holder"
    Then KeyOwnershipTransferred events should be emitted
    And "Key Recipient" should own the transferred keys
    And "Key Holder" should have no active keys

  # ==========================================================================
  # Person Query Scenarios
  # ==========================================================================

  @person @query @by-unit
  Scenario: Query people by organizational unit
    Given people exist in various units:
      | name   | unit        |
      | Alice  | Engineering |
      | Bob    | Engineering |
      | Carol  | Security    |
    When I query people in "Engineering" unit
    Then the result should contain "Alice" and "Bob"
    And the result should not contain "Carol"

  @person @query @by-role
  Scenario: Query people by role
    Given people exist with various roles:
      | name   | role          |
      | Admin1 | Administrator |
      | Admin2 | Administrator |
      | Dev1   | Developer     |
    When I query people with role "Administrator"
    Then the result should contain "Admin1" and "Admin2"
    And the result should not contain "Dev1"

  @person @query @with-keys
  Scenario: Query people with assigned keys
    Given "KeyHolder1" has 2 assigned keys
    And "KeyHolder2" has 1 assigned key
    And "NoKeys" has no assigned keys
    When I query people with assigned keys
    Then the result should contain "KeyHolder1" and "KeyHolder2"
    And the result should not contain "NoKeys"

  # ==========================================================================
  # Validation Scenarios
  # ==========================================================================

  @person @validation @duplicate-email
  Scenario: Reject duplicate email address
    Given person "Existing" exists with email "taken@techcorp.com"
    When I attempt to create a person with email "taken@techcorp.com"
    Then a DuplicateEmailError should be returned
    And no PersonCreated event should be emitted

  @person @validation @invalid-email
  Scenario: Reject invalid email format
    When I attempt to create a person with email "not-an-email"
    Then an InvalidEmailError should be returned
    And no PersonCreated event should be emitted

  @person @validation @required-fields
  Scenario: Require name and email for person creation
    When I attempt to create a person without a name
    Then a ValidationError should be returned
    And no PersonCreated event should be emitted

  # ==========================================================================
  # Event Sourcing Scenarios
  # ==========================================================================

  @person @eventsourcing @history
  Scenario: Retrieve complete person history
    Given person "History Person" exists
    And the person has undergone multiple updates:
      | update_type | old_value  | new_value    |
      | name        | Old Name   | History Person|
      | role        | Junior     | Senior       |
      | email       | old@x.com  | new@x.com    |
    When I retrieve the person's event history
    Then all historical events should be returned in order
    And each event should have correct causation chain

  @person @eventsourcing @point-in-time
  Scenario: Reconstruct person state at point in time
    Given person "Time Traveler" exists
    And the person was updated at various timestamps
    When I reconstruct the person's state at a specific past timestamp
    Then the state should reflect the person as of that timestamp
    And future events should not be included
