# Copyright (c) 2025 - Cowboy AI, LLC.
# Domain Bootstrap Feature Specification
# BDD scenarios for initializing a CIM domain from configuration

Feature: Domain Bootstrap
  As a CIM administrator
  I want to bootstrap a domain from configuration files
  So that I can initialize the organizational structure for key management

  Background:
    Given a clean CIM environment
    And an encrypted output partition is mounted

  # ==========================================================================
  # Organization Creation Scenarios
  # ==========================================================================

  @organization @happy-path
  Scenario: Create organization from bootstrap configuration
    Given a domain-bootstrap.json configuration with organization "CowboyAI"
    When I execute the bootstrap command
    Then an OrganizationCreated event should be emitted
    And the organization should have name "CowboyAI"
    And the organization should have a valid UUID v7 identifier
    And the organization should be persisted to the projection

  @organization @units
  Scenario: Create organization with units
    Given a domain-bootstrap.json with organization "TechCorp" containing:
      | unit_name  | unit_type  |
      | Engineering| Department |
      | Security   | Department |
      | DevOps     | Team       |
    When I execute the bootstrap command
    Then OrganizationUnitCreated events should be emitted for each unit
    And the organization should have 3 organizational units
    And each unit should reference the parent organization

  @organization @hierarchy
  Scenario: Create nested organizational hierarchy
    Given a domain-bootstrap.json with nested units:
      | parent       | child        | type       |
      | root         | Engineering  | Department |
      | Engineering  | Backend      | Team       |
      | Engineering  | Frontend     | Team       |
      | root         | Operations   | Department |
    When I execute the bootstrap command
    Then the hierarchy should be correctly established
    And Backend team should have Engineering as parent
    And Frontend team should have Engineering as parent

  # ==========================================================================
  # Person Creation Scenarios
  # ==========================================================================

  @person @happy-path
  Scenario: Create person from bootstrap configuration
    Given a domain-bootstrap.json with person:
      | name         | email                | role          |
      | Steele Hazel | steele@cowboyai.com  | Administrator |
    When I execute the bootstrap command
    Then a PersonCreated event should be emitted
    And the person should have name "Steele Hazel"
    And the person should be assigned to the organization

  @person @multiple
  Scenario: Create multiple people with roles
    Given a domain-bootstrap.json with people:
      | name         | email              | role          | unit        |
      | Alice Admin  | alice@corp.com     | Administrator | Security    |
      | Bob Dev      | bob@corp.com       | Developer     | Engineering |
      | Carol Ops    | carol@corp.com     | Operator      | Operations  |
    When I execute the bootstrap command
    Then PersonCreated events should be emitted for 3 people
    And each person should be assigned to their respective unit
    And each person should have their role recorded

  @person @yubikey-assignment
  Scenario: Assign YubiKey to person during bootstrap
    Given a domain-bootstrap.json with person "Security Admin" assigned YubiKey serial "12345678"
    When I execute the bootstrap command
    Then the person should be created
    And a YubiKeyAssigned event should be emitted
    And the YubiKey assignment should reference the person

  # ==========================================================================
  # Location Creation Scenarios
  # ==========================================================================

  @location @physical
  Scenario: Create physical storage location
    Given a domain-bootstrap.json with location:
      | name        | type     | address              |
      | HQ Vault    | Physical | 123 Security Lane    |
    When I execute the bootstrap command
    Then a LocationCreated event should be emitted
    And the location should be of type Physical
    And the location should be available for key storage

  @location @virtual
  Scenario: Create virtual storage location
    Given a domain-bootstrap.json with virtual location:
      | name          | type    | uri                    |
      | Cloud HSM     | Virtual | hsm://vault.corp.com   |
    When I execute the bootstrap command
    Then a LocationCreated event should be emitted
    And the location should be of type Virtual
    And the URI should be validated and stored

  # ==========================================================================
  # Validation Scenarios
  # ==========================================================================

  @validation @error
  Scenario: Reject invalid bootstrap configuration
    Given a domain-bootstrap.json with missing required fields
    When I attempt to execute the bootstrap command
    Then a validation error should be returned
    And no events should be emitted
    And the projection should remain unchanged

  @validation @duplicate
  Scenario: Reject duplicate organization creation
    Given an organization "ExistingOrg" already exists in the projection
    And a domain-bootstrap.json with organization "ExistingOrg"
    When I attempt to execute the bootstrap command
    Then an OrganizationAlreadyExists error should be returned
    And no duplicate organization should be created

  @validation @idempotent
  Scenario: Bootstrap is idempotent for existing entities
    Given a fully bootstrapped domain with organization "IdempotentOrg"
    When I execute the same bootstrap command again
    Then no new entities should be created
    And the existing entities should remain unchanged
    And the operation should complete successfully

  # ==========================================================================
  # Event Sourcing Scenarios
  # ==========================================================================

  @eventsourcing @replay
  Scenario: Reconstruct domain from event log
    Given a bootstrapped domain with 5 people and 3 units
    And all events have been persisted
    When I replay all events from the beginning
    Then the reconstructed projection should match the original
    And all relationships should be preserved
    And all timestamps should be preserved

  @eventsourcing @correlation
  Scenario: All bootstrap events share correlation ID
    Given a domain-bootstrap.json configuration
    When I execute the bootstrap command
    Then all emitted events should share the same correlation_id
    And each event should have a unique event_id
    And causation chains should be correctly established
