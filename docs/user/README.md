# CIM Keys User Documentation

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

Welcome to the CIM Keys user documentation. This guide will help you understand and use cim-keys for cryptographic key management and PKI infrastructure.

## Quick Navigation

### Getting Started
- [Quick Start Guide](getting-started/quick-start.md) - Get up and running in 5 minutes

### User Guides
- [CLI Reference](guides/cli-reference.md) - Complete command-line interface reference
- [GUI User Guide](guides/gui-user-guide.md) - Graph-based visual interface guide
- [End-to-End Workflow](guides/end-to-end-workflow.md) - Complete PKI bootstrap workflow

### Concepts
- [Certificate Hierarchy](concepts/certificate-hierarchy.md) - Root CA → Intermediate → Leaf
- [NATS Identity](concepts/nats-identity.md) - Operators, accounts, and users
- [Passphrase Workflow](concepts/passphrase-workflow.md) - Single passphrase to full PKI

### Ontology & Domain Model
- [Domain Model](ontology/domain-model.md) - Organization, Person, Location entities
- [Claims & Policy](ontology/claims-policy.md) - Authorization and claims model

---

## What is CIM Keys?

CIM Keys is an **air-gapped, offline-first** PKI and cryptographic key management system designed for:

- **Organizations** managing their own PKI infrastructure
- **Security teams** requiring hardware-backed key storage (YubiKey)
- **NATS deployments** needing operator/account/user credentials
- **Air-gapped environments** where keys never touch networked systems

## Key Features

| Feature | Description |
|---------|-------------|
| Offline PKI | Generate complete certificate hierarchies offline |
| YubiKey Support | Hardware-backed key storage on PIV-compliant tokens |
| NATS Integration | Generate NATS operator, account, and user credentials |
| Graph Visualization | Visual interface for understanding key relationships |
| Event Sourcing | Complete audit trail of all operations |

## Getting Help

- **Technical Documentation**: [../technical/](../technical/README.md)
- **Archive (Historical)**: [../archive/](../archive/README.md)
- **GitHub Issues**: Report bugs or request features
