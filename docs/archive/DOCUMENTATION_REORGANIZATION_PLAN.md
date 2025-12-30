# Documentation Reorganization Plan

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

## Overview

Reorganize 150+ scattered markdown files into three logical groups:

1. **User Documentation** (`docs/user/`) - How to use cim-keys
2. **Technical Documentation** (`docs/technical/`) - How to build/maintain
3. **Archive** (`docs/archive/`) - Historical/transitional documents

---

## Group 1: User Documentation (`docs/user/`)

**Audience**: End users, security administrators, PKI operators

### Structure

```
docs/user/
├── README.md                    # User docs index
├── getting-started/
│   ├── installation.md          # How to install cim-keys
│   ├── quick-start.md           # 5-minute first use
│   └── prerequisites.md         # What you need before starting
├── guides/
│   ├── cli-reference.md         # Complete CLI command reference
│   ├── gui-user-guide.md        # Graph GUI usage guide
│   ├── end-to-end-workflow.md   # Complete PKI bootstrap workflow
│   └── yubikey-operations.md    # YubiKey-specific operations
├── concepts/
│   ├── pki-fundamentals.md      # What is PKI, why it matters
│   ├── certificate-hierarchy.md # Root CA → Intermediate → Leaf
│   ├── hardware-security.md     # YubiKey, HSM, air-gapped ops
│   ├── nats-identity.md         # NATS operators/accounts/users
│   └── key-ownership.md         # Person → Key → Organization model
└── ontology/
    ├── domain-model.md          # Organization, Person, Location
    ├── pki-ontology.md          # Certificate, Key, Trust relationships
    ├── claims-policy.md         # Authorization and claims model
    └── glossary.md              # Term definitions
```

### Source Files to Migrate

| Source | Destination |
|--------|-------------|
| `docs/QUICK_START.md` | `docs/user/getting-started/quick-start.md` |
| `docs/CLI_REFERENCE.md` | `docs/user/guides/cli-reference.md` |
| `docs/GRAPH_GUI_USER_GUIDE.md` | `docs/user/guides/gui-user-guide.md` |
| `docs/END_TO_END_USAGE_EXAMPLE.md` | `docs/user/guides/end-to-end-workflow.md` |
| `docs/NATS_HIERARCHY_GUIDE.md` | `docs/user/concepts/nats-identity.md` |
| `docs/DOMAIN_ONTOLOGY.md` | `docs/user/ontology/domain-model.md` |
| `docs/CLAIMS_POLICY_ONTOLOGY.md` | `docs/user/ontology/claims-policy.md` |
| `PKI_HIERARCHY_DESIGN.md` | `docs/user/concepts/certificate-hierarchy.md` |
| `SINGLE_PASSPHRASE_WORKFLOW.md` | `docs/user/guides/end-to-end-workflow.md` (merge) |

---

## Group 2: Technical Documentation (`docs/technical/`)

**Audience**: Developers, maintainers, contributors

### Structure

```
docs/technical/
├── README.md                    # Technical docs index
├── architecture/
│   ├── overview.md              # System architecture overview
│   ├── mvi-pattern.md           # Model-View-Intent architecture
│   ├── event-sourcing.md        # Event-driven design
│   ├── hexagonal.md             # Ports and adapters
│   ├── module-dependencies.md   # Crate structure
│   └── nats-streaming.md        # NATS JetStream integration
├── axioms/
│   ├── frp-axioms.md            # 10 N-ary FRP axioms
│   ├── causality.md             # A4 causality (UUID v7 + causation_id)
│   ├── categorical-semantics.md # Category theory foundations
│   └── compliance-matrix.md     # Current axiom compliance status
├── methodology/
│   ├── domain-driven-design.md  # DDD patterns used
│   ├── liftable-domain.md       # Domain → Graph functor
│   ├── bdd-specifications.md    # Gherkin scenario approach
│   └── testing-strategy.md      # Unit/MVI/BDD/Property tests
├── development/
│   ├── contributing.md          # How to contribute
│   ├── building.md              # Build instructions
│   ├── testing.md               # Running tests
│   └── nix-environment.md       # Nix flake setup
├── integration/
│   ├── cim-registry.md          # Integration with CIM ecosystem
│   ├── nats-ipld.md             # NATS + IPLD integration
│   └── event-publishing.md      # Event publishing patterns
└── lessons-learned/
    ├── workflow-patterns.md     # Extracted from retrospectives
    ├── anti-patterns.md         # What to avoid
    └── best-practices.md        # Proven approaches
```

### Source Files to Migrate

| Source | Destination |
|--------|-------------|
| `ARCHITECTURE.md` | `docs/technical/architecture/overview.md` |
| `MVI_ARCHITECTURE_DIAGRAM.md` | `docs/technical/architecture/mvi-pattern.md` |
| `HEXAGONAL_ARCHITECTURE.md` | `docs/technical/architecture/hexagonal.md` |
| `N_ARY_FRP_AXIOMS.md` | `docs/technical/axioms/frp-axioms.md` |
| `N_ARY_FRP_COMPLIANCE_ANALYSIS.md` | `docs/technical/axioms/compliance-matrix.md` |
| `CATEGORICAL_FRP_SEMANTICS.md` | `docs/technical/axioms/categorical-semantics.md` |
| `docs/UUID_V7_TIMESTAMP_AXIOM.md` | `docs/technical/axioms/causality.md` |
| `CIM-DEVELOPMENT-GUIDELINES.md` | `docs/technical/methodology/domain-driven-design.md` |
| `CONTRIBUTING.md` | `docs/technical/development/contributing.md` |
| `docs/NATS_STREAMING_ARCHITECTURE.md` | `docs/technical/architecture/nats-streaming.md` |
| `docs/NATS_IPLD_IMPLEMENTATION_SUMMARY.md` | `docs/technical/integration/nats-ipld.md` |
| `docs/CIM_REGISTRY_INTEGRATION.md` | `docs/technical/integration/cim-registry.md` |
| `docs/TEST_COVERAGE.md` | `docs/technical/methodology/testing-strategy.md` |

---

## Group 3: Archive (`docs/archive/`)

**Purpose**: Preserve historical documents, transitional work, retrospectives

### Structure

```
docs/archive/
├── README.md                    # Archive index with context
├── retrospectives/
│   ├── RETROSPECTIVE_SYNTHESIS.md  # Compiled lessons from all sprints
│   ├── sprint-00-foundation.md
│   ├── sprint-01-domain.md
│   ├── sprint-02-terminology.md
│   ├── sprint-03-injection.md
│   ├── sprint-04-mvi-intent.md
│   ├── sprint-05-pure-update.md
│   ├── sprint-06-conceptual-spaces.md
│   ├── sprint-07-liftable-domain.md
│   ├── sprint-08-test-infrastructure.md
│   ├── sprint-09-bdd-specs.md
│   ├── sprint-10-documentation.md
│   └── nodetype-migration/       # NodeType removal sprints 6-17
├── progress/
│   ├── phase-1-completion.md
│   ├── phase-2-completion.md
│   ├── phase-3-completion.md
│   ├── phase-4-completion.md
│   └── implementation-timeline.md
├── sessions/
│   ├── session-summaries.md      # Consolidated session notes
│   └── continuation-notes.md
├── migrations/
│   ├── iced-0.13-migration.md
│   ├── frp-compliance-updates.md
│   └── gui-cleanup-history.md
├── analysis/
│   ├── ddd-assessment.md
│   ├── gap-analyses.md
│   └── violation-reports.md
└── designs/
    ├── original-designs.md       # Early design documents
    ├── interactive-graph.md
    └── firefly-swarm.md
```

### Files to Archive

**Retrospectives** (move from `retrospectives/` and `docs/*_RETROSPECTIVE.md`):
- All `retrospectives/sprint_*.md` files
- All `docs/*_RETROSPECTIVE.md` files

**Progress/Phase Completion**:
- `PHASE1_COMPLETE.md`, `PHASE2_COMPLETE.md`, etc.
- `PHASE_1_COMPLETION_SUMMARY.md`, `PHASE_1_MILESTONE.md`
- `IMPLEMENTATION_COMPLETE.md`, `IMPLEMENTATION_SUMMARY.md`
- `*_PROGRESS.md` files

**Session/Continuation**:
- `SESSION_SUMMARY.md`, `SESSION_5_INTEGRATION_COMPLETE.md`
- `CONTINUATION_SESSION_SUMMARY.md`
- `PROGRESS_LOG.md`

**Cleanup/Migration Tracking**:
- `CODE_CLEANUP_SUMMARY.md`, `GUI_CLEANUP_COMPLETE.md`
- `FINAL_GUI_CLEANUP.md`, `WARNING_CLEANUP_STATUS.md`
- `MIGRATION_GUIDE.md`, `ICED_0.13_MIGRATION.md`
- `FRP_COMPLIANCE_UPDATE.md`, `FRP_VIOLATION_REPORT.md`

**Analysis/Assessment**:
- `DDD_HEXAGONAL_ARCHITECTURE_ASSESSMENT.md`
- `CLAN-NSC-GAP-ANALYSIS.md`
- `INTENT_SIGNAL_KIND_ANALYSIS.md`

**Design Documents**:
- `INTERACTIVE_GRAPH_DESIGN.md`
- `FIREFLY_SWARM_MODELS.md`, `FIREFLY_DEBUG_SUMMARY.md`
- `GUI_VISUAL_MOCKUP.md`, `GRAPH_BASED_NODE_CREATION.md`
- `CQRS_GRAPH_DESIGN.md`

---

## Execution Steps

### Step 1: Create Directory Structure
```bash
mkdir -p docs/user/{getting-started,guides,concepts,ontology}
mkdir -p docs/technical/{architecture,axioms,methodology,development,integration,lessons-learned}
mkdir -p docs/archive/{retrospectives/nodetype-migration,progress,sessions,migrations,analysis,designs}
```

### Step 2: Compile Retrospectives
- Read all 11 sprint retrospectives
- Read all 22 NodeType migration retrospectives
- Extract workflow patterns, anti-patterns, best practices
- Create `RETROSPECTIVE_SYNTHESIS.md`

### Step 3: Migrate User Documentation
- Move and rename files per mapping table
- Update internal links
- Create `docs/user/README.md` index

### Step 4: Migrate Technical Documentation
- Move and rename files per mapping table
- Consolidate overlapping content
- Create `docs/technical/README.md` index

### Step 5: Archive Historical Documents
- Move all transitional files
- Rename with consistent prefixes
- Create `docs/archive/README.md` with context

### Step 6: Update Root Files
- Update main `README.md` with links to new structure
- Update `CLAUDE.md` references
- Remove migrated files from root

### Step 7: Verify and Commit
- Check all internal links work
- Run any doc tests
- Commit with comprehensive message

---

## Files to Keep at Root

- `README.md` - Project overview (link to docs)
- `CLAUDE.md` - Claude Code instructions
- `CONTRIBUTING.md` - Redirect to `docs/technical/development/contributing.md`
- `ARCHITECTURE.md` - Keep as quick reference, link to full docs
- `Cargo.toml`, `flake.nix`, etc. - Build files

---

## Timeline

| Step | Description | Files Affected |
|------|-------------|----------------|
| 1 | Create directories | 0 |
| 2 | Compile retrospectives | 33 read, 1 created |
| 3 | User docs | ~15 files |
| 4 | Technical docs | ~25 files |
| 5 | Archive | ~60 files |
| 6 | Update root | ~5 files |
| 7 | Commit | All |

**Total**: ~100+ files reorganized
