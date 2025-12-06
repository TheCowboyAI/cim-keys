# CIM Keys - DGX Phoenix Cluster Security

**Created**: 2025-12-05
**Operator**: dgx-phoenix
**Security Model**: NATS JWT/NKeys

## Overview

This repository contains all PKI credentials and security configurations for the DGX Phoenix NATS cluster. These credentials enable secure, authenticated access to the NATS infrastructure.

**⚠️ CRITICAL**: This repository contains sensitive cryptographic material. Never commit to public repositories.

## Security Architecture

### Operator
- **Name**: dgx-phoenix
- **Purpose**: Root of trust for the DGX cluster
- **Scope**: Manages all accounts and users in the Phoenix datacenter

### Accounts

| Account | Purpose | Users |
|---------|---------|-------|
| **SYS** | System account for NATS cluster management | sys-admin |
| **ADMIN** | Administrative access for operators | admin |
| **MONITORING** | Metrics collection and monitoring | prometheus |

### Users

| User | Account | Purpose | Credentials File |
|------|---------|---------|------------------|
| sys-admin | SYS | Cluster administration | `creds/SYS/sys-admin.creds` |
| admin | ADMIN | General administrative tasks | `creds/ADMIN/admin.creds` |
| prometheus | MONITORING | Prometheus metrics scraping | `creds/MONITORING/prometheus.creds` |

## Using Credentials

### Connect with NATS CLI

```bash
# As sys-admin
nats --creds=/path/to/cim-keys/creds/SYS/sys-admin.creds server list

# As admin
nats --creds=/path/to/cim-keys/creds/ADMIN/admin.creds pub test.subject "hello"

# As prometheus (monitoring)
nats --creds=/path/to/cim-keys/creds/MONITORING/prometheus.creds sub "metrics.>"
```

### Use in Applications

```go
// Go example
nc, _ := nats.Connect("nats://10.0.20.1:4222",
    nats.UserCredentials("/path/to/creds/ADMIN/admin.creds"))
```

```rust
// Rust example
let nc = nats::Options::with_credentials("/path/to/creds/ADMIN/admin.creds")
    .connect("nats://10.0.20.1:4222")?;
```

## Deployment to NATS Servers

The resolver configuration has been generated and saved to:
- `/git/thecowboyai/cim-dgx/nats-configs/resolver.conf`

To deploy security to the NATS cluster, include this configuration in your NATS server config files.

## Security Best Practices

1. **Never commit unencrypted credentials to git**
2. **Rotate credentials regularly** (every 90 days recommended)
3. **Use principle of least privilege** - grant minimum necessary permissions
4. **Audit access logs** regularly
5. **Keep backups** of NSC store in secure offline location

## Credential Rotation

To rotate a user's credentials:

```bash
# Set NSC home
export NSC_HOME=/git/thecowboyai/cim-keys/nsc

# Revoke old credentials
nsc revoke user --account ADMIN --name admin

# Generate new credentials  
nsc generate user --account ADMIN --name admin

# Update resolver configuration
nsc generate config --mem-resolver --sys-account SYS > ../cim-dgx/nats-configs/resolver.conf

# Deploy updated resolver to NATS servers
# (see deployment documentation)
```

## Emergency Access Revocation

If credentials are compromised:

```bash
export NSC_HOME=/git/thecowboyai/cim-keys/nsc

# Revoke the compromised user
nsc revoke user --account <ACCOUNT> --name <USER>

# Regenerate resolver config
nsc generate config --mem-resolver --sys-account SYS

# Immediately deploy to all NATS servers
# The revoked credentials will be rejected within seconds
```

## Files to Protect

These files contain sensitive cryptographic material and must be protected:

- `nsc/stores/**/keys/**/*.nk` - Private keys (NEVER share)
- `creds/**/*.creds` - User credential files (share only with authorized users)
- `.nkeys/**/*` - NKey seeds (NEVER share)

## Reference

- [NATS Security](https://docs.nats.io/running-a-nats-service/configuration/securing_nats)
- [NSC Tool Documentation](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/auth_intro/nsc_intro)
- [JWT Authentication](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/auth_intro/jwt)
