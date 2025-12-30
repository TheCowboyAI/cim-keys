# CIM Keys Archive

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

Historical and transitional documentation from the cim-keys development process.

> **Note**: This is archived documentation. For current documentation, see:
> - [User Documentation](../user/README.md)
> - [Technical Documentation](../technical/README.md)

---

## Archive Contents

### Retrospectives
Sprint retrospectives documenting lessons learned:

- **[RETROSPECTIVE_SYNTHESIS.md](retrospectives/RETROSPECTIVE_SYNTHESIS.md)** - Compiled lessons from all sprints (START HERE)
- `sprint_0.md` through `sprint_10.md` - Individual sprint retrospectives
- `STATE_MACHINE_PHASE*_RETROSPECTIVE.md` - State machine implementation phases
- `EXPERT_RETROSPECTIVE_SPRINT17.md` - Expert consultation outcomes

#### NodeType Migration Retrospectives
Located in `retrospectives/nodetype-migration/`:
- `NODETYPE_MIGRATION_SPRINT6_RETROSPECTIVE.md` through `NODETYPE_MIGRATION_SPRINT17_RETROSPECTIVE.md`

### Progress Tracking
Implementation progress and completion reports:

- Phase completion: `PHASE1_COMPLETE.md` through `PHASE4_COMPLETE.md`
- Implementation summaries: `IMPLEMENTATION_*.md`
- Progress tracking: `*_PROGRESS.md`
- Feature completion: `FEATURES_COMPLETE_README.md`
- User stories: `USER_STORIES.md`, `USER_STORY_COVERAGE.md`
- Test planning: `UI_TEST_*.md`, `TEST_IMPROVEMENT_PLAN.md`
- FRP summaries: `N_ARY_FRP_INDEX.md`, `N_ARY_FRP_SUMMARY.md`

### Sessions
Session and continuation notes:

- `SESSION_SUMMARY.md` - Session summaries
- `SESSION_5_INTEGRATION_COMPLETE.md` - Integration milestone
- `CONTINUATION_SESSION_SUMMARY.md` - Continuation notes
- `PROGRESS_LOG.md` - Progress log

### Migrations
Migration and cleanup tracking:

- `MIGRATION_PLAN.md`, `MIGRATION_COMPLETION_SUMMARY.md`
- `ICED_0.13_MIGRATION.md` - Iced framework migration
- `CODE_CLEANUP_SUMMARY.md`, `GUI_CLEANUP_COMPLETE.md`
- `FRP_COMPLIANCE_UPDATE.md`, `FRP_VIOLATION_REPORT.md`
- `WARNING_CLEANUP_STATUS.md`

### Analysis
Technical analysis and assessments:

- `DDD_HEXAGONAL_ARCHITECTURE_ASSESSMENT.md`
- `CIM_INTEGRATION_ASSESSMENT.md`
- `CLAN-NSC-GAP-ANALYSIS.md`
- `INTENT_SIGNAL_KIND_ANALYSIS.md`

### Designs
Original design documents:

- `ARCHITECTURE_DESIGN.md`, `CIM_KEYS_ARCHITECTURE.md`
- `INTERACTIVE_GRAPH_DESIGN.md`
- `CQRS_GRAPH_DESIGN.md`
- `GRAPH_FIRST_UI_ARCHITECTURE.md`, `GRAPH_INTERACTION_PATTERNS.md`
- `GUI_VISUAL_MOCKUP.md`, `GUI_ENHANCEMENTS_ARCHITECTURE.md`
- `FIREFLY_SWARM_MODELS.md`, `FIREFLY_DEBUG_SUMMARY.md`
- `DETAILED_IMPLEMENTATION_PLAN.md`, `REFACTORING_PLAN.md`

---

## Key Insights from Retrospectives

### Patterns That Worked
1. **Expert Consultation First** - Consult DDD, ACT, FRP experts before coding
2. **Foundation Before Migration** - Build infrastructure before migrating
3. **Incremental Migration** - Use conversion functions, not big-bang
4. **Test-Driven Verification** - Comprehensive tests as safety net
5. **Fold Pattern** - Replace giant match statements with folds
6. **Builder Pattern** - Immutable `with_*` methods

### Anti-Patterns to Avoid
1. Big-bang replacement
2. Coding without mathematical foundation
3. Giant match statements on type tags
4. Mutable state in update functions
5. Manual `causation_id: None`

See [RETROSPECTIVE_SYNTHESIS.md](retrospectives/RETROSPECTIVE_SYNTHESIS.md) for complete analysis.

---

## Organization Plan

This archive was created as part of documentation reorganization.
See [DOCUMENTATION_REORGANIZATION_PLAN.md](DOCUMENTATION_REORGANIZATION_PLAN.md) for the migration plan.
