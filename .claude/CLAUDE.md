# CIM Infrastructure Management - Claude Instructions

## 🔴 PRIME DIRECTIVE: CONTINUOUS LEARNING & BEST PRACTICES

**You are a constantly learning system. When you write your progress at natural breaks, you also:**

1. **ANALYZE** the instructions you were given and how well you performed them
2. **UPDATE** your "best practices" with new instructions to use in your memory going forward
3. **MAINTAIN** an ordered list of these best practices and shall not violate them when operating
4. **RECITE** these best practices when asked, and prove you are following them

### Current Best Practices (Updated Continuously):
1. **Infrastructure FIRST**: Always deploy NATS + PKI before domain work
2. **NixOS System**: Never use apt/yum/etc - only Nix package manager
3. **Date Handling**: Always use `$(date -I)`, never generate dates from memory
4. **No CRUD**: All infrastructure changes as immutable events
5. **Proxmox Current, NixOS Future**: Current deployment on Proxmox is fine, pure NixOS is roadmap
6. **PKI in cim-keys**: Never commit private keys to this repo
7. **Documentation First**: Document infrastructure before deploying
8. **Test Before Deploy**: Always verify in development first
9. **Network Isolation**: Respect VLAN boundaries and firewall rules
10. **Event Sourcing**: All configuration changes logged as events

## 🚨 CRITICAL: NIXOS SYSTEM - PACKAGE MANAGEMENT RULES 🚨

**THIS IS A NIXOS SYSTEM. NEVER USE ANY PACKAGE MANAGER OTHER THAN NIX!**

### FORBIDDEN COMMANDS (NEVER USE):
- ❌ `apt`, `apt-get`, `apt-cache`, `dpkg` (Debian/Ubuntu)
- ❌ `yum`, `dnf`, `rpm` (RedHat/Fedora)
- ❌ `pacman`, `yay` (Arch)
- ❌ `brew` (Homebrew)
- ❌ `snap`, `flatpak` (Universal packages)
- ❌ `pip install` at system level (use Nix shells)
- ❌ `npm install -g` (use Nix shells)
- ❌ `cargo install` at system level (use Nix shells)

### CORRECT NIX COMMANDS (ALWAYS USE):
- ✅ `nix develop` - Enter development shell with dependencies
- ✅ `nix search nixpkgs#<package>` - Search for packages
- ✅ `nix build` - Build packages
- ✅ `nix run nixpkgs#<package>` - Run packages directly
- ✅ `nix flake update` - Update flake dependencies

## CRITICAL: Date Handling Rules

**NEVER generate dates from memory. ALWAYS use:**
1. System date: `$(date -I)` or `$(date +%Y-%m-%d)`
2. Git commit dates: `$(git log -1 --format=%cd --date=short)`
3. Existing dates from files being read

**When updating progress.json:**
```bash
# Always capture system date
CURRENT_DATE=$(date -I)
# Then use $CURRENT_DATE in JSON updates
```

## Repository Purpose

This is your **CIM Infrastructure and Domain Management Repository**.

**Workflow Order** (do these in sequence):

1. **Setup Infrastructure** - Configure and deploy NATS cluster (do this FIRST)
2. **Deploy PKI** - Generate and commit security credentials to `cim-keys` (do this SECOND)
3. **Domain Development** - THEN observe, discover patterns, and implement domains (do this THIRD)
4. **Operations** - Continue managing your deployed CIM infrastructure (ongoing)

**What is a CIM?**: A **Deterministic, Content-Addressed, Event-Driven Architecture** running on NATS infrastructure.

## Current Infrastructure State

### Deployed (Proxmox-Based)
- **3 PVE Hosts**: pve1 (10.0.0.200), pve2 (10.0.0.201), pve3 (10.0.0.203)
- **NATS Cluster**: 3 LXC containers running NixOS
  - nats-1 (10.0.0.41, Container 201 on pve1)
  - nats-2 (10.0.0.42, Container 202 on pve2)
  - nats-3 (10.0.0.43, Container 203 on pve3)
- **cimstor**: NixOS server at 172.16.0.2 (NATS leaf node + IPLD storage)
- **Network**: UniFi Dream Machine Pro (10.0.0.1)
- **Storage**: Ceph distributed storage across PVE nodes

### Future (When More Hardware Available)
- Pure NixOS/nix-darwin bare metal deployment
- No virtualization layer
- See `domains/network-infrastructure/INFRASTRUCTURE-ROADMAP.md`

## Common Commands

### Development Environment
```bash
# Enter CIM infrastructure environment
nix develop

# Build NATS cluster configuration
nix build .#nats-cluster

# Deploy NATS to LXC container
./domains/network-infrastructure/nats-cluster/deploy-nats-lxc.sh

# Verify NATS cluster
./domains/network-infrastructure/nats-cluster/verify-cluster.sh
```

### NATS Management
```bash
# Check NATS server status
nats-server --signal status

# Monitor NATS cluster
nats server list

# View streams
nats stream list

# View KV stores
nats kv list

# Check object stores
nats object list
```

### Infrastructure Operations
```bash
# Deploy configuration updates
./domains/network-infrastructure/nats-cluster/deploy-config-update.sh

# Activate new configuration
./domains/network-infrastructure/nats-cluster/activate-config.sh

# Rebuild infrastructure
./domains/network-infrastructure/nats-cluster/deploy-rebuild.sh
```

## CIM Infrastructure Workflow

### Phase 1: Setup Infrastructure

**1. Configure Your Environment**
```bash
# Clone this repository
git clone https://github.com/thecowboyai/cim
cd cim

# Enter development shell
nix develop
```

**2. Define Infrastructure Topology**
- Edit `domains/network-infrastructure/nats-cluster/flake.nix`
- Configure network settings
- Define NATS cluster topology (1 node = leaf, 3+ nodes = cluster)
- Setup storage backends (IPLD object store)

**3. Configure NATS Security**
- Generate PKI keys using NSC (NATS Security)
- Store keys in `../cim-keys/` repository
- Configure operators, accounts, and users
- Setup TLS certificates

### Phase 2: Deploy NATS

**1. Deploy NATS Leaf Node or Cluster**
```bash
# Single leaf node deployment
./domains/network-infrastructure/nats-cluster/deploy-nats-lxc.sh --node leaf-01

# Multi-node cluster deployment
./domains/network-infrastructure/nats-cluster/deploy-cluster.sh --nodes 3
```

**2. Verify Deployment**
```bash
# Verify cluster health
./domains/network-infrastructure/nats-cluster/verify-cluster.sh

# Check JetStream
nats stream list

# Verify object store integration
./domains/network-infrastructure/nats-cluster/verify-cimstor-integration.sh
```

### Phase 3: Commit PKI to cim-keys

**1. Initialize cim-keys Repository**
```bash
# Create cim-keys repo (if not exists)
cd ..
git clone https://github.com/yourusername/cim-keys
cd cim-keys
```

**2. Commit PKI Credentials**
```bash
# Copy NSC keys
cp -r ~/.nsc/* ./nsc/

# Commit to git
git add .
git commit -m "feat: Add PKI credentials for CIM infrastructure"
git push
```

**3. Reference from CIM Infrastructure**
- Update NATS configurations to reference PKI paths
- Configure credential resolution
- Setup automatic key rotation policies

### Phase 4: Manage Deployed CIM

**1. Monitor Infrastructure**
```bash
# View NOC dashboard
cd domains/network-infrastructure/noc-dashboard
nix run

# Monitor NATS metrics
nats server report jetstream
nats server report connections
```

**2. Deploy Configuration Updates**
```bash
# Update configuration
vim domains/network-infrastructure/nats-cluster/config/nats.conf

# Deploy update
./domains/network-infrastructure/nats-cluster/deploy-config-update.sh

# Verify update
nats server list
```

**3. Scale Infrastructure**
```bash
# Add new leaf node
./domains/network-infrastructure/nats-cluster/deploy-nats-lxc.sh --node leaf-02

# Verify cluster membership
nats server list
```

### Phase 5: Domain Development (After Infrastructure Ready)

Once your NATS infrastructure and PKI are deployed, you can begin domain development:

**1. Observe Information Landscape**
```bash
# Read the essential guides
cat doc/CIM-INFORMATION-PHILOSOPHY.md
cat doc/CIM-PURPOSE-AND-PATTERN-DISCOVERY.md
```

- What information exists in this domain?
- Where does it live? (External systems, APIs, databases)
- Who are the authoritative sources?
- What information do we need to reference vs. create?

**2. Define Domain Purpose and Boundaries**
- What business domain will this CIM serve?
- What are the domain boundaries?
- Who are the domain experts?

**3. Discover Patterns Through Observation**
```bash
# Run event storming sessions
# See doc/event-storming-guide.md
```

**4. Model and Implement**
```bash
# Follow the actual workflow guide
cat doc/ACTUAL-CIM-WORKFLOW.md

# Create domain implementation using cim-domain library
# See /git/thecowboyai/cim-domain and /git/thecowboyai/cim-domain-person for examples
```

**5. Deploy to Infrastructure**
```bash
# Deploy domain services to your NATS infrastructure
nix build .#your-domain-service
# Deploy to leaf nodes using your established infrastructure
```

## Special Considerations

### Date Handling
- **NEVER generate dates from memory**
- Always use system date: `$(date -I)`
- Use git commit dates: `$(git log -1 --format=%cd --date=short)`

### Infrastructure Changes
- Always test in development first
- Document topology changes
- Update network diagrams in `domains/network-infrastructure/`
- Commit configuration before deploying

### Security Best Practices
- Never commit private keys to this repo
- Always use `cim-keys` repo for PKI
- Rotate credentials regularly
- Use TLS for all NATS connections
- Enable JetStream encryption at rest

## Documentation Reference

Essential reading:
- `CLAUDE.md` (this repository root) - Complete infrastructure workflow
- `doc/START-HERE.md` - Complete learning path
- `doc/WHAT-IS-A-CIM.md` - CIM definition
- `doc/HEXAGONAL-ARCHITECTURE-CATEGORY-THEORY.md` - Architectural foundation
- `domains/network-infrastructure/README.md` - Network infrastructure operations
- `domains/network-infrastructure/INFRASTRUCTURE-ROADMAP.md` - Proxmox now, NixOS future

## Context Awareness

**Check which repository you're in:**
- If in `cim` → You're in the INFRASTRUCTURE repository (this one)
- If in `cim-*` → You're in a MODULE (provides specific functionality)
- If in `cim-domain-*` → You're in a DOMAIN (assembles modules for business)

This repository is your operational center for CIM infrastructure - where theory meets production deployment.
