# Tooling Architecture for CIM-Keys Implementation

## Core Architectural Principles Enforcement

### 1. Event Sourcing Validator Agent
**Purpose:** Ensures every state change goes through Command → Event → Projection
```yaml
name: event-sourcing-validator
triggers:
  - on_file_save: "*.rs"
  - on_commit: pre-commit hook
validations:
  - no_direct_state_mutation
  - all_events_have_correlation_id
  - all_events_have_causation_id
  - projections_are_idempotent
  - events_are_immutable
```

### 2. DDD Boundary Guardian Agent
**Purpose:** Enforces bounded context separation and aggregate consistency
```yaml
name: ddd-boundary-guardian
monitors:
  - aggregate_boundaries
  - entity_id_usage
  - value_object_immutability
  - domain_event_flow
prevents:
  - cross_aggregate_transactions
  - anemic_domain_models
  - leaking_domain_logic
```

### 3. FRP Pattern Enforcer
**Purpose:** Validates Functional Reactive Programming patterns
```yaml
name: frp-enforcer
checks:
  - pure_functions_in_aggregates
  - no_side_effects_in_domain
  - event_streams_are_composable
  - reactive_gui_updates
```

## MCP Servers Required

### 1. NixOS Development Environment MCP
```nix
# mcp-nixos-dev.nix
{
  name = "cim-keys-nixos-dev";

  capabilities = {
    # Manages Nix flakes and development shells
    flake_management = true;
    # Handles NixOS module testing
    nixos_test_runner = true;
    # Manages reproducible builds
    build_coordination = true;
  };

  tools = [
    "nix develop"
    "nix build"
    "nix flake check"
    "nixos-rebuild"
  ];

  functions = {
    enterDevShell = "Enters the Nix development environment";
    runNixOSTests = "Runs NixOS integration tests";
    validateFlake = "Validates flake.nix configuration";
    buildWASM = "Builds WASM target with Nix";
  };
}
```

### 2. Event Store MCP
```rust
// mcp-event-store.rs
pub struct EventStoreMCP {
    capabilities: Vec<Capability>,
}

impl EventStoreMCP {
    pub fn capabilities() -> Vec<Capability> {
        vec![
            Capability::AppendEvents,
            Capability::ReadEventStream,
            Capability::SubscribeToEvents,
            Capability::CreateSnapshots,
            Capability::ValidateEventChain,
            Capability::ReplayEvents,
        ]
    }

    pub fn append_event(&self, event: Event) -> Result<EventId> {
        // Validates correlation/causation chain
        // Ensures idempotency
        // Persists to encrypted storage
    }

    pub fn replay_from(&self, checkpoint: EventId) -> EventStream {
        // Replays events for projection rebuild
    }
}
```

### 3. Domain Model Validator MCP
```rust
// mcp-domain-validator.rs
pub struct DomainValidatorMCP {
    rules: DomainRules,
}

impl DomainValidatorMCP {
    pub fn validate_aggregate(&self, aggregate: &dyn AggregateRoot) -> ValidationResult {
        // Checks aggregate invariants
        // Validates entity relationships
        // Ensures consistency rules
    }

    pub fn validate_command(&self, cmd: Command) -> ValidationResult {
        // Pre-execution validation
        // Permission checks
        // Business rule validation
    }

    pub fn validate_projection(&self, projection: &Projection) -> ValidationResult {
        // Ensures deterministic projection
        // Validates manifest integrity
        // Checks storage structure
    }
}
```

## Specialized Agents

### 1. Test Orchestrator Agent (`test-orchestrator`)
```rust
pub struct TestOrchestratorAgent {
    plan: DetailedImplementationPlan,
    current_stage: Stage,
}

impl TestOrchestratorAgent {
    pub fn execute_stage(&mut self, stage: Stage) -> StageResult {
        match stage {
            Stage::FixDomainModules => self.fix_compilation_errors(),
            Stage::ImplementCore => self.implement_core_functionality(),
            Stage::IntegrationTests => self.run_integration_tests(),
            _ => self.execute_custom_stage(stage),
        }
    }

    pub fn validate_stage_completion(&self) -> bool {
        // Checks evaluation criteria from plan
        // Runs specified test suites
        // Validates deliverables
    }

    pub fn report_progress(&self) -> ProgressReport {
        // Current stage status
        // Tests passed/failed
        // Next steps
    }
}
```

### 2. Event Flow Visualizer Agent (`event-flow-viz`)
```rust
pub struct EventFlowVisualizerAgent {
    events: Vec<Event>,
    projections: HashMap<ProjectionId, Projection>,
}

impl EventFlowVisualizerAgent {
    pub fn generate_mermaid_diagram(&self) -> String {
        // Creates visual flow of events
        // Shows causation chains
        // Highlights projection updates
    }

    pub fn trace_command_flow(&self, cmd: Command) -> FlowTrace {
        // Shows: Command → Events → Projections → Side Effects
    }

    pub fn validate_event_choreography(&self) -> Vec<Issue> {
        // Detects missing events
        // Finds broken causation chains
        // Identifies orphaned projections
    }
}
```

### 3. NixOS Module Builder Agent (`nixos-module-builder`)
```nix
# agent-nixos-module-builder.nix
{
  buildCimKeysModule = {
    # Generates NixOS module from domain
    fromDomain = domain: {
      options.cim-keys = {
        enable = mkEnableOption "CIM Keys genesis system";
        domain = mkOption {
          type = types.attrs;
          description = "Domain configuration";
        };
        storage = mkOption {
          type = types.path;
          default = "/mnt/encrypted/cim-keys";
        };
      };

      config = mkIf cfg.enable {
        # Auto-generates systemd services
        # Sets up encrypted storage
        # Configures NATS integration
      };
    };
  };
}
```

### 4. YubiKey Mock Agent (`yubikey-mock`)
```rust
pub struct YubiKeyMockAgent {
    slots: HashMap<PIVSlot, MockKey>,
}

impl YubiKeyMockAgent {
    pub fn provision_slot(&mut self, slot: PIVSlot, key_type: KeyType) -> Result<()> {
        // Simulates YubiKey operations for testing
        // Maintains same API as real YubiKey
        // Generates deterministic keys for testing
    }

    pub fn sign_with_slot(&self, slot: PIVSlot, data: &[u8]) -> Result<Signature> {
        // Mock signing operation
        // Validates slot usage
        // Returns test signature
    }
}
```

## Development Workflow Tools

### 1. Plan Tracker CLI (`cim-plan`)
```bash
#!/usr/bin/env bash
# cim-plan - Tracks implementation plan progress

cim-plan current
# Output: Stage 1.1.2: Fix field mismatches in organization_tests.rs

cim-plan validate
# Runs validation for current stage
# Shows: ✓ 4/6 criteria met

cim-plan next
# Advances to next stage after validation

cim-plan report
# Generates progress report with timeline
```

### 2. Event Sourcing REPL (`cim-repl`)
```rust
// Interactive REPL for testing event flows
pub struct CimRepl {
    aggregate: KeyManagementAggregate,
    event_store: EventStore,
    projection: Projection,
}

impl CimRepl {
    pub fn commands() -> Vec<ReplCommand> {
        vec![
            ReplCommand::new("emit", "Emit a command"),
            ReplCommand::new("replay", "Replay events from checkpoint"),
            ReplCommand::new("project", "Show current projection"),
            ReplCommand::new("trace", "Trace event causation chain"),
            ReplCommand::new("validate", "Validate aggregate state"),
        ]
    }
}
```

### 3. Domain Language Validator (`domain-lint`)
```rust
// Ensures ubiquitous language consistency
pub struct DomainLinter {
    dictionary: DomainDictionary,
}

impl DomainLinter {
    pub fn lint_file(&self, path: &Path) -> Vec<LintIssue> {
        // Checks for consistent terminology
        // Validates event naming (past tense)
        // Ensures command naming (imperative)
        // Validates aggregate boundaries
    }
}
```

## CI/CD Integration Tools

### 1. Nix Flake Checker
```yaml
name: nix-flake-check
on: [push, pull_request]
jobs:
  check:
    runs-on: nixos-latest
    steps:
      - uses: actions/checkout@v3
      - run: nix flake check
      - run: nix develop --command cargo test
      - run: nix build .#cim-keys
```

### 2. Event Sourcing Test Suite
```rust
#[test_suite]
mod event_sourcing_invariants {
    #[invariant_test]
    fn events_are_immutable() { }

    #[invariant_test]
    fn projections_are_deterministic() { }

    #[invariant_test]
    fn replay_produces_same_state() { }

    #[property_test]
    fn command_idempotency(cmd: Command) { }
}
```

## Integration with Existing CIM Agents

### Required Updates to Existing Agents:

1. **sage.md** - Add CIM-Keys specific orchestration:
   - Bootstrap workflow coordination
   - Test suite execution monitoring
   - Integration validation

2. **ddd-expert.md** - Enhance for CIM-Keys:
   - Aggregate boundary validation
   - Entity relationship checking
   - Value object immutability enforcement

3. **event-storming-expert.md** - Add:
   - Key lifecycle event discovery
   - YubiKey provisioning flow mapping
   - Certificate chain event choreography

4. **nix-expert.md** - Extend with:
   - CIM-Keys flake management
   - NixOS module generation
   - Reproducible build validation

5. **nats-expert.md** - Include:
   - NATS hierarchy generation validation
   - Subject naming convention enforcement
   - Event routing configuration

## Automation Scripts

### 1. Stage Executor (`execute-stage.sh`)
```bash
#!/usr/bin/env bash
# Executes a specific stage from DETAILED_IMPLEMENTATION_PLAN.md

STAGE=$1
source .claude/scripts/stage-executor.sh

execute_stage "$STAGE" || {
    echo "Stage $STAGE failed validation"
    show_failures
    exit 1
}

advance_to_next_stage
```

### 2. Principle Validator (`validate-principles.sh`)
```bash
#!/usr/bin/env bash
# Validates code against architectural principles

check_event_sourcing() {
    rg "setState|this\.\w+ =" src/ && {
        echo "ERROR: Direct state mutation detected"
        return 1
    }
}

check_ddd_boundaries() {
    # Validates aggregate boundaries
    # Checks for anemic models
    # Ensures ubiquitous language
}

check_frp_patterns() {
    # Validates pure functions
    # Checks for side effects
    # Ensures reactive patterns
}
```

## Success Metrics Dashboard

```yaml
dashboard:
  implementation_progress:
    - current_stage: "1.1.2"
    - stages_completed: 1
    - stages_remaining: 35

  principle_adherence:
    - event_sourcing: 95%
    - ddd_boundaries: 88%
    - frp_patterns: 92%
    - nixos_compatibility: 100%

  test_coverage:
    - domain_modules: 45%
    - cim_keys_core: 12%
    - integration: 0%

  quality_metrics:
    - events_with_correlation: 100%
    - deterministic_projections: 100%
    - aggregate_consistency: 100%
```

## Immediate Actions

1. **Create `.claude/agents/cim-keys-guardian.md`** - Agent to enforce our principles
2. **Implement `cim-plan` CLI** - Track our progress automatically
3. **Set up event-sourcing linter** - Catch violations early
4. **Create YubiKey mock library** - Enable testing without hardware
5. **Build NixOS test harness** - Validate in target environment

This tooling architecture ensures we stay focused on our core principles while efficiently implementing the plan.