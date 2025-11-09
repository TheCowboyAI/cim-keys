# .claude Directory Index - CIM Infrastructure Management

## Overview

This directory contains all instructions, patterns, and standards for Claude AI when working on CIM infrastructure management. The focus is on deploying and operating NATS clusters, managing PKI, and maintaining production infrastructure.

## Quick Start

**New to CIM Infrastructure?**
1. Read [CLAUDE.md](./CLAUDE.md) - Start here for infrastructure workflow
2. Read [/CLAUDE.md](../CLAUDE.md) - Repository root for complete picture
3. Review `domains/network-infrastructure/README.md` - Operations guide

**Working on infrastructure:**
1. Follow Phase 1-4 in [CLAUDE.md](./CLAUDE.md)
2. Deploy NATS cluster
3. Setup PKI in cim-keys
4. Monitor and manage

## Directory Structure

### 📋 Primary Instructions

- **[CLAUDE.md](./CLAUDE.md)** - Infrastructure management instructions
  - Infrastructure-first workflow
  - Phase-by-phase deployment
  - NATS cluster operations
  - PKI management
  - Current state (Proxmox) and future (NixOS)

- **[instructions.md](./instructions.md)** - Agent orchestration (legacy)
  - SAGE agent system
  - Expert agent coordination
  - Keep for reference

- **[unified-conversation-model.md](./unified-conversation-model.md)** - CIM conversation model
  - Event-driven patterns
  - Mathematical foundations

### 🔒 Security

- **[security/](./security/)** - Security configurations
  - Permission settings
  - PKI references

### 📜 Scripts

- **[scripts/detect-context.sh](./scripts/detect-context.sh)** - Detect repository type
  - Identifies if you're in cim (infrastructure), cim-* (module), or cim-domain-* (domain)

### 🎭 Agents (Legacy)

- **[agents/](./agents/)** - Specialized expert agent definitions
  - SAGE orchestrator
  - Infrastructure experts (NATS, network, nix)
  - Domain experts (DDD, event storming)
  - Development experts (TDD, BDD, QA)

### 📋 Rules (Legacy)

- **[rules/](./rules/)** - Legacy rule definitions
  - Keep for reference

## Infrastructure Workflow (Primary Focus)

This repository is for **infrastructure management**, not domain development:

### 1. Setup Infrastructure (FIRST)
```bash
cd domains/network-infrastructure/nats-cluster
./deploy-option2.sh  # Deploy NATS with cimstor
./verify-cluster.sh
```

### 2. Deploy PKI (SECOND)
```bash
# Generate NSC credentials
nsc add operator <org>
nsc add account platform
nsc add user <service>

# Commit to cim-keys (separate repo)
cd ../cim-keys
git add .
git commit -m "feat: Add PKI"
git push
```

### 3. Domain Development (THIRD)
After infrastructure is ready:
- Observe information landscape
- Discover patterns
- Implement domains (see cim-domain-person for examples)

### 4. Operations (ONGOING)
- Monitor NATS cluster
- Deploy configuration updates
- Scale infrastructure
- Manage domains

## Current Infrastructure State

### Production (Proxmox-Based)
- 3 PVE hosts running NATS cluster in LXC containers
- cimstor for IPLD object storage
- UniFi network management
- See `domains/network-infrastructure/` for details

### Future (Roadmap)
- Pure NixOS/nix-darwin bare metal
- No virtualization layer
- See `domains/network-infrastructure/INFRASTRUCTURE-ROADMAP.md`

## Key Principles for Infrastructure Management

1. **Infrastructure FIRST** - Deploy NATS + PKI before domain work
2. **NixOS Only** - Never use other package managers
3. **Event-Driven** - All infrastructure changes as immutable events
4. **Security First** - PKI in cim-keys, never commit private keys here
5. **Test Before Deploy** - Always verify in development
6. **Document Everything** - Infrastructure as code AND documentation
7. **Monitor Always** - Use NOC dashboard and NATS metrics
8. **No Rush** - Proxmox works fine, NixOS migration when hardware available

## Documentation Map

### Infrastructure Documentation
- `domains/network-infrastructure/README.md` - Operations guide
- `domains/network-infrastructure/nats-cluster/README.md` - NATS deployment
- `domains/network-infrastructure/nats-cluster/ARCHITECTURE.md` - Architecture details
- `domains/network-infrastructure/INFRASTRUCTURE-ROADMAP.md` - Migration plan

### CIM Philosophy Documentation
- `doc/START-HERE.md` - Complete learning path
- `doc/WHAT-IS-A-CIM.md` - CIM definition
- `doc/HEXAGONAL-ARCHITECTURE-CATEGORY-THEORY.md` - Mathematical foundations
- `doc/CIM-INFORMATION-PHILOSOPHY.md` - External-first information approach
- `doc/CONCEPTUAL-SPACES-DOMAIN-INTEGRATION.md` - Semantic reasoning
- `doc/ACTUAL-CIM-WORKFLOW.md` - From idea to deployed code

### Repository Root
- `/CLAUDE.md` - Complete workflow (Infrastructure → PKI → Domains → Operations)
- `/README.md` - Repository overview

## Context Awareness

**Always check which repository you're in:**

```bash
# Run context detection
./.claude/scripts/detect-context.sh
```

**Repository Types:**
- `cim` (this one) → Infrastructure management
- `cim-*` → Module (functionality building block)
- `cim-domain-*` → Domain implementation (business-specific)

## Navigation by Task

### Deploying NATS Cluster
1. Review `domains/network-infrastructure/nats-cluster/README.md`
2. Follow deployment scripts in `domains/network-infrastructure/nats-cluster/`
3. Verify with health checks

### Managing PKI
1. Use NSC to generate credentials
2. Commit to `cim-keys` repository (separate, private)
3. Reference from NATS configuration

### Monitoring Infrastructure
1. Use NOC dashboard: `cd domains/network-infrastructure/noc-dashboard && nix run`
2. Check NATS metrics: `nats server list`, `nats stream list`
3. Review logs and health checks

### Scaling Infrastructure
1. Add new leaf node with deployment scripts
2. Verify cluster membership
3. Update documentation

### Understanding CIM
1. Start with `doc/START-HERE.md`
2. Read `doc/WHAT-IS-A-CIM.md`
3. Understand infrastructure-first approach

## Critical Reminders

- ❌ **Never use apt/yum/etc** - Only Nix on this NixOS system
- ❌ **Never generate dates from memory** - Use `$(date -I)`
- ❌ **Never commit private keys here** - Use cim-keys repository
- ✅ **Always test before deploy** - Verify in development
- ✅ **Always document changes** - Infrastructure as code
- ✅ **Always update progress** - Document state changes

## Remember

This repository is your **operational center for CIM infrastructure management**:
- Deploy and manage NATS clusters
- Configure and maintain PKI
- Monitor and scale infrastructure
- Prepare for domain development

Domain implementation happens in `cim-domain-*` repositories after infrastructure is ready.
