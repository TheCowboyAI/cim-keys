# Copyright (c) 2025 - Cowboy AI, LLC.
# Export Manifest Feature Specification
# BDD scenarios for exporting CIM domain data to encrypted storage

Feature: Export Manifest
  As a CIM administrator
  I want to export domain data to an encrypted manifest
  So that the domain can be imported into leaf nodes and clusters

  Background:
    Given a fully bootstrapped CIM domain
    And an encrypted output partition is mounted at "/mnt/encrypted"
    And the domain has:
      | entity_type | count |
      | organization| 1     |
      | units       | 3     |
      | people      | 5     |
      | keys        | 10    |
      | certificates| 8     |

  # ==========================================================================
  # Manifest Generation
  # ==========================================================================

  @manifest @generate @happy-path
  Scenario: Generate complete domain manifest
    When I generate a domain manifest
    Then a manifest.json file should be created
    And the manifest should have a valid UUID
    And the manifest should have a creation timestamp
    And the manifest should include version information
    And a ManifestGenerated event should be emitted

  @manifest @generate @contents
  Scenario: Manifest contains all domain artifacts
    When I generate a domain manifest
    Then the manifest should reference:
      | artifact_type     | path                      |
      | organization      | domain/organization.json  |
      | units             | domain/units/             |
      | people            | domain/people/            |
      | keys              | keys/                     |
      | certificates      | certificates/             |
      | events            | events/                   |
      | nats              | nats/                     |

  @manifest @generate @checksums
  Scenario: Manifest includes file checksums
    When I generate a domain manifest
    Then each referenced file should have a SHA-256 checksum
    And the checksums should be verifiable
    And tampered files should be detectable

  # ==========================================================================
  # Directory Structure Export
  # ==========================================================================

  @export @structure @complete
  Scenario: Export creates correct directory structure
    When I export the domain
    Then the following structure should exist:
      """
      /mnt/encrypted/cim-keys/
      ├── manifest.json
      ├── domain/
      │   ├── organization.json
      │   ├── units/
      │   ├── people/
      │   └── relationships.json
      ├── keys/
      │   └── {key-id}/
      │       ├── metadata.json
      │       └── public.pem
      ├── certificates/
      │   ├── root-ca/
      │   ├── intermediate-ca/
      │   └── leaf/
      ├── nats/
      │   ├── operator/
      │   ├── accounts/
      │   └── users/
      └── events/
          └── {date}/
      """

  @export @structure @keys
  Scenario: Key directory contains only public material
    When I export keys to the manifest
    Then each key directory should contain:
      | file           | contains          |
      | metadata.json  | key metadata      |
      | public.pem     | public key PEM    |
    And private key material should NOT be exported
    And the export should pass security audit

  @export @structure @certificates
  Scenario: Certificate directories organized by type
    When I export certificates
    Then root CA certificates should be in certificates/root-ca/
    And intermediate CA certificates should be in certificates/intermediate-ca/
    And leaf certificates should be in certificates/leaf/
    And each certificate should include the full chain

  # ==========================================================================
  # Event Log Export
  # ==========================================================================

  @export @events @complete
  Scenario: Export complete event history
    When I export the event log
    Then all domain events should be exported
    And events should be organized by date
    And events should be in chronological order
    And event integrity should be verifiable

  @export @events @format
  Scenario: Events exported in standard format
    When I export events
    Then each event file should contain:
      | field          | present |
      | event_id       | yes     |
      | event_type     | yes     |
      | correlation_id | yes     |
      | causation_id   | yes     |
      | timestamp      | yes     |
      | payload        | yes     |
    And the format should be JSON Lines (.jsonl)

  @export @events @incremental
  Scenario: Support incremental event export
    Given previous export was done at timestamp T1
    When I export events since T1
    Then only events after T1 should be exported
    And the incremental export should be appendable
    And the event sequence should remain continuous

  # ==========================================================================
  # NATS Credentials Export
  # ==========================================================================

  @export @nats @operator
  Scenario: Export NATS operator configuration
    When I export NATS configuration
    Then the operator JWT should be exported
    And the operator public key should be exported
    And the operator private key should NOT be exported

  @export @nats @accounts
  Scenario: Export NATS account configurations
    When I export NATS accounts
    Then each account should have:
      | file           | content              |
      | account.jwt    | signed account JWT   |
      | resolver.conf  | resolver preload     |
    And account relationships should be preserved

  @export @nats @users
  Scenario: Export NATS user credentials
    When I export NATS users
    Then each user should have a .creds file
    And the creds file should be in standard NATS format
    And credentials should be importable by nats-server

  # ==========================================================================
  # Manifest Verification
  # ==========================================================================

  @manifest @verify @integrity
  Scenario: Verify manifest integrity
    Given a complete export exists
    When I verify the manifest
    Then all referenced files should exist
    And all checksums should match
    And no unexpected files should be present
    And the verification should pass

  @manifest @verify @tampering
  Scenario: Detect manifest tampering
    Given a complete export exists
    And a file has been modified after export
    When I verify the manifest
    Then a checksum mismatch should be detected
    And the tampered file should be identified
    And the verification should fail

  @manifest @verify @missing
  Scenario: Detect missing files
    Given a complete export exists
    And a referenced file has been deleted
    When I verify the manifest
    Then the missing file should be detected
    And the verification should fail
    And recovery options should be suggested

  # ==========================================================================
  # Import Scenarios
  # ==========================================================================

  @manifest @import @clean
  Scenario: Import manifest to clean environment
    Given a clean CIM leaf node
    And a valid export manifest
    When I import the manifest
    Then all domain entities should be created
    And all keys should be registered
    And all certificates should be installed
    And NATS credentials should be configured

  @manifest @import @merge
  Scenario: Import manifest merging with existing data
    Given a CIM leaf node with existing domain data
    And a manifest with updated data
    When I import with merge strategy
    Then new entities should be added
    And existing entities should be updated
    And no data should be lost
    And conflicts should be reported

  @manifest @import @validate
  Scenario: Validate manifest before import
    Given a manifest file
    When I validate the manifest for import
    Then schema validation should pass
    And all referenced artifacts should be present
    And dependency order should be correct
    And the import should be approved

  # ==========================================================================
  # Partial Export Scenarios
  # ==========================================================================

  @export @partial @unit
  Scenario: Export single organizational unit
    Given I want to export only the "Engineering" unit
    When I export with unit filter "Engineering"
    Then only "Engineering" unit data should be exported
    And only people in "Engineering" should be included
    And only their keys should be exported
    And the manifest should indicate partial export

  @export @partial @person
  Scenario: Export single person's credentials
    Given I want to export credentials for "Alice"
    When I export with person filter "Alice"
    Then only Alice's data should be exported
    And her keys and certificates should be included
    And her NATS credentials should be included
    And organizational context should be minimal

  # ==========================================================================
  # Export Status and Progress
  # ==========================================================================

  @export @status @progress
  Scenario: Track export progress
    When I start a domain export
    Then progress should be reported for:
      | phase              | status     |
      | validating         | in_progress|
      | exporting_domain   | pending    |
      | exporting_keys     | pending    |
      | exporting_certs    | pending    |
      | exporting_nats     | pending    |
      | generating_manifest| pending    |
      | verifying          | pending    |
    And completion percentage should be updated

  @export @status @complete
  Scenario: Export completion status
    When an export completes successfully
    Then the status should be "completed"
    And the manifest path should be reported
    And export statistics should be provided:
      | metric             | present |
      | files_exported     | yes     |
      | total_size         | yes     |
      | duration           | yes     |
      | checksum           | yes     |

  @export @status @error
  Scenario: Export error handling
    Given the output partition has insufficient space
    When I attempt to export
    Then an ExportError should be returned
    And the partial export should be cleaned up
    And the error should include space requirements

  # ==========================================================================
  # Event Sourcing
  # ==========================================================================

  @manifest @eventsourcing @record
  Scenario: Export creates audit event
    When I export the domain
    Then a ManifestExported event should be emitted
    And the event should include:
      | field          | value                |
      | manifest_id    | uuid                 |
      | export_path    | /mnt/encrypted/...   |
      | artifacts      | count per type       |
      | checksum       | manifest checksum    |

  @manifest @eventsourcing @history
  Scenario: Track export history
    Given multiple exports have been performed
    When I query export history
    Then all ManifestExported events should be returned
    And exports should be in chronological order
    And each export should be uniquely identifiable
