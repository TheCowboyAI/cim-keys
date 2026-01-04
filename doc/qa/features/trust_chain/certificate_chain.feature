# Copyright (c) 2025 - Cowboy AI, LLC.

Feature: Certificate Chain Cryptographic Verification
  As a PKI administrator
  I need certificate chains to be cryptographically verified
  So that trust relationships are mathematically proven

  Background:
    Given a root CA certificate "RootCA" that is self-signed
    And the root CA has a validity period of 10 years
    And the root CA is in the trusted roots list

  # ============================================================================
  # Temporal Validity Scenarios
  # ============================================================================

  Scenario: Valid certificate within validity period
    Given a certificate "LeafCert" with the following validity:
      | not_before | 30 days ago |
      | not_after  | 335 days from now |
    When I verify temporal validity at the current time
    Then verification should succeed

  Scenario: Expired certificate in chain
    Given an intermediate CA "ExpiredCA" signed by "RootCA"
    And "ExpiredCA" has the following validity:
      | not_before | 60 days ago |
      | not_after  | 1 day ago |
    When I verify the certificate chain containing "ExpiredCA"
    Then verification should fail with error type "Expired"
    And the error should identify "ExpiredCA" as the expired certificate
    And the error should include the expiration timestamp

  Scenario: Certificate not yet valid
    Given a certificate "FutureCert" with the following validity:
      | not_before | 1 day from now |
      | not_after  | 365 days from now |
    When I verify temporal validity at the current time
    Then verification should fail with error type "NotYetValid"
    And the error should include the not_before timestamp

  Scenario: Certificate valid at specific point in time
    Given a certificate "PastCert" with the following validity:
      | not_before | 100 days ago |
      | not_after  | 50 days ago |
    When I verify temporal validity at "75 days ago"
    Then verification should succeed

  # ============================================================================
  # Chain Structure Scenarios
  # ============================================================================

  Scenario: Valid two-tier certificate chain (root + leaf)
    Given a root CA certificate "RootCA" that is self-signed
    And a leaf certificate "LeafCert" signed by "RootCA"
    When I verify the certificate chain [LeafCert, RootCA]
    Then verification should succeed
    And the trust path should contain 2 verified links
    And the trust path should show "LeafCert" issued by "RootCA"
    And the trust path should show "RootCA" as self-signed

  Scenario: Valid three-tier certificate chain
    Given a root CA certificate "RootCA" that is self-signed
    And an intermediate CA "IntermediateCA" signed by "RootCA"
    And a leaf certificate "LeafCert" signed by "IntermediateCA"
    When I verify the certificate chain [LeafCert, IntermediateCA, RootCA]
    Then verification should succeed
    And the trust path should contain 3 verified links
    And the chain depth should be 3

  Scenario: Valid four-tier certificate chain
    Given a root CA certificate "RootCA" that is self-signed
    And an intermediate CA "Level1CA" signed by "RootCA"
    And an intermediate CA "Level2CA" signed by "Level1CA"
    And a leaf certificate "LeafCert" signed by "Level2CA"
    When I verify the certificate chain [LeafCert, Level2CA, Level1CA, RootCA]
    Then verification should succeed
    And the trust path should contain 4 verified links

  # ============================================================================
  # Signature Verification Scenarios
  # ============================================================================

  Scenario: Invalid signature in chain - Ed25519
    Given a certificate "FakeCert" claiming to be signed by "RootCA"
    But "FakeCert" has an invalid Ed25519 signature
    When I verify the certificate chain [FakeCert, RootCA]
    Then verification should fail with error type "InvalidSignature"
    And the error should identify "FakeCert" as having the invalid signature
    And the error should reference "RootCA" as the expected issuer

  Scenario: Invalid signature in chain - RSA
    Given certificates using RSA-SHA256 signatures
    And a certificate "FakeRSACert" claiming to be signed by "RSACA"
    But "FakeRSACert" has an invalid RSA signature
    When I verify the certificate chain [FakeRSACert, RSACA]
    Then verification should fail with error type "InvalidSignature"

  Scenario: Invalid signature in chain - ECDSA P-256
    Given certificates using ECDSA P-256 signatures
    And a certificate "FakeECCert" claiming to be signed by "ECCA"
    But "FakeECCert" has an invalid ECDSA signature
    When I verify the certificate chain [FakeECCert, ECCA]
    Then verification should fail with error type "InvalidSignature"

  Scenario: Valid signature with Ed25519
    Given a root CA "Ed25519Root" using Ed25519 algorithm
    And a leaf certificate "Ed25519Leaf" properly signed by "Ed25519Root"
    When I verify the certificate chain [Ed25519Leaf, Ed25519Root]
    Then verification should succeed
    And the trust level for all links should be "Complete"

  # ============================================================================
  # Issuer Chain Verification Scenarios
  # ============================================================================

  Scenario: Issuer DN mismatch
    Given a root CA certificate "RootCA" with subject CN "Root CA"
    And a leaf certificate "BadIssuer" with issuer CN "Different CA"
    When I verify the certificate chain [BadIssuer, RootCA]
    Then verification should fail with error type "IssuerMismatch"
    And the error should show expected issuer "Root CA"
    And the error should show actual issuer "Different CA"

  Scenario: Correct issuer chain through intermediates
    Given a root CA "RootCA" with subject CN "Root CA"
    And an intermediate "IntCA" with subject CN "Intermediate CA" and issuer CN "Root CA"
    And a leaf "Leaf" with subject CN "Leaf Cert" and issuer CN "Intermediate CA"
    When I verify the certificate chain [Leaf, IntCA, RootCA]
    Then verification should succeed

  # ============================================================================
  # Root Certificate Scenarios
  # ============================================================================

  Scenario: Root certificate is not self-signed
    Given a certificate "FakeRoot" where subject CN differs from issuer CN
    When I verify that "FakeRoot" is self-signed
    Then verification should fail with error type "RootNotSelfSigned"

  Scenario: Untrusted root certificate
    Given a self-signed certificate "UntrustedRoot"
    And "UntrustedRoot" is NOT in the trusted roots list
    And a leaf certificate "UntrustedLeaf" signed by "UntrustedRoot"
    When I verify the chain against trusted roots
    Then verification should fail with error type "UntrustedRoot"
    And the error should include the root certificate fingerprint

  Scenario: Trusted root certificate
    Given a self-signed certificate "TrustedRoot"
    And "TrustedRoot" IS in the trusted roots list
    And a leaf certificate "TrustedLeaf" signed by "TrustedRoot"
    When I verify the chain against trusted roots
    Then verification should succeed

  # ============================================================================
  # Edge Cases and Error Handling
  # ============================================================================

  Scenario: Empty certificate chain
    Given an empty certificate chain
    When I attempt to verify the chain
    Then verification should fail with error type "EmptyChain"

  Scenario: Single certificate chain (self-signed leaf)
    Given a self-signed certificate "SelfSignedLeaf"
    When I verify the certificate as both leaf and root
    Then verification should succeed if self-signature is valid

  Scenario: Expired intermediate with valid leaf and root
    Given a valid root CA "ValidRoot"
    And an expired intermediate CA "ExpiredInt" signed by "ValidRoot"
    And a valid leaf certificate "ValidLeaf" signed by "ExpiredInt"
    When I verify the certificate chain
    Then verification should fail with error type "Expired"
    And the error should identify "ExpiredInt" specifically

  Scenario: Future-dated root with valid leaf
    Given a root CA "FutureRoot" with not_before 1 year from now
    And a leaf certificate "CurrentLeaf" signed by "FutureRoot"
    When I verify the certificate chain
    Then verification should fail with error type "NotYetValid"
    And the error should identify "FutureRoot" specifically

  # ============================================================================
  # Trust Path Result Scenarios
  # ============================================================================

  Scenario: Trust path contains correct fingerprints
    Given a certificate chain with known certificate fingerprints
    When I verify the certificate chain
    Then the trust path should contain the exact fingerprints of each certificate
    And fingerprints should be SHA-256 hashes in hexadecimal format
    And each fingerprint should be exactly 64 characters

  Scenario: Trust path records verification timestamp
    Given a valid certificate chain
    When I verify the certificate chain
    Then the trust path should record the verification timestamp
    And the timestamp should be within 1 second of the current time

  Scenario: Trust path shows issuer relationships
    Given a three-tier certificate chain [Leaf, Int, Root]
    When I verify the certificate chain
    Then the trust path should show:
      | certificate | issuer        |
      | Leaf        | Int           |
      | Int         | Root          |
      | Root        | (self-signed) |

  # ============================================================================
  # Time-Based Verification Scenarios
  # ============================================================================

  Scenario: Verify chain at historical time
    Given a certificate chain that was valid 6 months ago
    But the leaf certificate has since expired
    When I verify the chain at "6 months ago"
    Then verification should succeed

  Scenario: Verify chain at future time
    Given a certificate chain that is currently valid
    But the leaf certificate will expire in 30 days
    When I verify the chain at "60 days from now"
    Then verification should fail with error type "Expired"

  Scenario: Certificate rotation - old chain still valid during overlap
    Given an "OldCA" certificate expiring in 30 days
    And a "NewCA" certificate valid starting today
    And a leaf certificate signed by "OldCA"
    When I verify the chain today
    Then verification should succeed with "OldCA"

  # ============================================================================
  # Algorithm Support Scenarios
  # ============================================================================

  Scenario Outline: Verify chain with different signature algorithms
    Given a root CA using <algorithm> signatures
    And a leaf certificate properly signed with <algorithm>
    When I verify the certificate chain
    Then verification should succeed

    Examples:
      | algorithm     |
      | Ed25519       |
      | ECDSA-P256    |
      | RSA-SHA256    |
      | RSA-SHA512    |

  Scenario: Unsupported signature algorithm
    Given a certificate signed with an unsupported algorithm
    When I verify the certificate signature
    Then verification should fail with error type "UnsupportedAlgorithm"
    And the error should identify the unsupported algorithm
