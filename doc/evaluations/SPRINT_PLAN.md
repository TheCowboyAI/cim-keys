# Unified Sprint Plan: cim-keys Anti-Pattern Remediation

**Date:** 2026-01-02
**Compiled from:** 8 Expert Evaluations
**Total Estimated Effort:** ~120 story points across 8 sprints

---

## Executive Summary

Eight domain experts evaluated cim-keys for pattern violations. Key findings:

| Expert | Compliance | Critical Issue |
|--------|------------|----------------|
| FRP Expert | 55% | A6 violation: 1000+ line match statement |
| ACT Expert | 45% | Coproduct universal property violated |
| DDD Expert | 55% | Anemic domain model, missing aggregate boundaries |
| Domain Expert | N/A | Context leakage between PKI/NATS and Organization |
| CIM Expert | 65% | Missing IPLD content addressing |
| Graph Expert | 45% | Embedded collections instead of edges |
| NATS Expert | 40% | No JetStream, no headers, no real NATS client |
| Subject Expert | N/A | Pattern matching instead of subject algebra |

**Priority 0 (CRITICAL):** Replace 1000+ line update.rs match with compositional routing

---

## Sprint Overview

| Sprint | Focus | Points | Duration |
|--------|-------|--------|----------|
| Sprint 11 | Categorical Foundations | 13 | 1 week |
| Sprint 12 | Routing Refactor | 16 | 1.5 weeks |
| Sprint 13 | DDD Foundations | 13 | 1 week |
| Sprint 14 | Bounded Context ACLs | 16 | 1 week |
| Sprint 15 | Graph Structure | 15 | 1 week |
| Sprint 16 | NATS Integration | 21 | 1.5 weeks |
| Sprint 17 | Event Hardening | 14 | 1 week |
| Sprint 18 | Verification & Docs | 12 | 1 week |

---

## Sprint 11: Categorical Foundations

**Goal:** Restore coproduct universal property and functor laws

### Tasks

| ID | Task | Points | Expert Source |
|----|------|--------|---------------|
| 11.1 | Implement `Coproduct` trait with `fold` | 3 | ACT |
| 11.2 | Add `arr` identity arrow to fold.rs | 2 | ACT |
| 11.3 | Implement functor law witnesses in LiftableDomain | 3 | ACT |
| 11.4 | Add Event Monoid implementation | 2 | ACT |
| 11.5 | Property tests for arrow laws | 3 | FRP |

**Acceptance Criteria:**
- [ ] `fold` is the unique morphism satisfying universal property
- [ ] Arrow identity and associativity laws verified
- [ ] Monoid identity and associativity for events

### Retrospective Template

```markdown
## Sprint 11 Retrospective

### What Worked
-

### What Didn't Work
-

### Lessons Learned
-

### Changes for Next Sprint
-
```

---

## Sprint 12: Routing Refactor

**Goal:** Replace 1000+ line update.rs match with compositional routing (FRP A6)

### Tasks

| ID | Task | Points | Expert Source |
|----|------|--------|---------------|
| 12.1 | Create `src/routing/` module with SubjectIntent | 3 | Subject |
| 12.2 | Implement arrow-based RouteArrow trait | 3 | Subject |
| 12.3 | Implement SubjectRouter with pattern matching | 3 | Subject |
| 12.4 | Extract UI handlers to `handlers/ui.rs` | 2 | FRP |
| 12.5 | Extract Domain handlers to `handlers/domain.rs` | 2 | FRP |
| 12.6 | Build hierarchical router, replace update() | 3 | Subject |

**Acceptance Criteria:**
- [ ] No match statement > 20 arms
- [ ] Arrow composition (>>>, ***, &&&) used for routing
- [ ] Subject patterns for all intent routing

### Retrospective Template

```markdown
## Sprint 12 Retrospective

### What Worked
-

### What Didn't Work
-

### Lessons Learned
-

### Changes for Next Sprint
-
```

---

## Sprint 13: DDD Foundations

**Goal:** Establish proper aggregate boundaries and value objects

### Tasks

| ID | Task | Points | Expert Source |
|----|------|--------|---------------|
| 13.1 | Create phantom-typed EntityId in `src/domain/ids.rs` | 2 | DDD |
| 13.2 | Migrate all Uuid to phantom-typed IDs | 3 | DDD |
| 13.3 | Create value objects: OperatorName, AccountName | 2 | DDD |
| 13.4 | Extract CertificateAggregate | 3 | DDD |
| 13.5 | Extract IdentityAggregate | 3 | DDD |

**Acceptance Criteria:**
- [ ] Compiler prevents ID type confusion
- [ ] Value objects enforce invariants at construction
- [ ] Clear aggregate boundaries defined

### Retrospective Template

```markdown
## Sprint 13 Retrospective

### What Worked
-

### What Didn't Work
-

### Lessons Learned
-

### Changes for Next Sprint
-
```

---

## Sprint 14: Bounded Context ACLs

**Goal:** Eliminate context leakage with Anti-Corruption Layers

### Tasks

| ID | Task | Points | Expert Source |
|----|------|--------|---------------|
| 14.1 | Create Published Language types | 5 | Domain |
| 14.2 | Implement PKI ACL (OrgContextAdapter) | 4 | Domain |
| 14.3 | Implement NATS ACL (PersonContextAdapter) | 4 | Domain |
| 14.4 | Minimize shared kernel | 3 | Domain |

**Acceptance Criteria:**
- [ ] Zero direct cross-context imports
- [ ] All cross-context refs use Reference types
- [ ] Context boundary tests pass

### Retrospective Template

```markdown
## Sprint 14 Retrospective

### What Worked
-

### What Didn't Work
-

### Lessons Learned
-

### Changes for Next Sprint
-
```

---

## Sprint 15: Graph Structure

**Goal:** Replace embedded collections with edge-based graph

### Tasks

| ID | Task | Points | Expert Source |
|----|------|--------|---------------|
| 15.1 | Create DomainGraph with adjacency lists | 4 | Graph |
| 15.2 | Define Relationship as first-class edge | 2 | Graph |
| 15.3 | Remove Vec<> collections from domain entities | 3 | Graph |
| 15.4 | Implement BFS/DFS/TopSort algorithms | 4 | Graph |
| 15.5 | Add cycle detection for event causation | 2 | Graph |

**Acceptance Criteria:**
- [ ] O(degree) neighbor lookup
- [ ] All relationships as edges
- [ ] Event causation forms valid DAG

### Retrospective Template

```markdown
## Sprint 15 Retrospective

### What Worked
-

### What Didn't Work
-

### Lessons Learned
-

### Changes for Next Sprint
-
```

---

## Sprint 16: NATS Integration

**Goal:** Implement real NATS/JetStream event publishing

### Tasks

| ID | Task | Points | Expert Source |
|----|------|--------|---------------|
| 16.1 | Add async-nats dependency | 1 | NATS |
| 16.2 | Create CIM header specification | 2 | NATS |
| 16.3 | Define subject hierarchy constants | 2 | NATS |
| 16.4 | Implement JetStreamPort trait | 3 | NATS |
| 16.5 | Implement JetStreamAdapter | 4 | NATS |
| 16.6 | Configure KEYS_EVENTS stream | 3 | NATS |
| 16.7 | Add event publishing to aggregate | 3 | NATS |
| 16.8 | Create durable consumer | 3 | NATS |

**Acceptance Criteria:**
- [ ] Events publish to JetStream with CIM headers
- [ ] Deduplication via Nats-Msg-Id
- [ ] Consumer resumes from last position

### Retrospective Template

```markdown
## Sprint 16 Retrospective

### What Worked
-

### What Didn't Work
-

### Lessons Learned
-

### Changes for Next Sprint
-
```

---

## Sprint 17: Event Hardening

**Goal:** Pure projections and content addressing

### Tasks

| ID | Task | Points | Expert Source |
|----|------|--------|---------------|
| 17.1 | Convert projections to pure `apply_event_pure(self)` | 3 | CIM |
| 17.2 | Add #[non_exhaustive] to EventEnvelope | 1 | CIM |
| 17.3 | Implement IPLD serialization for events | 4 | CIM |
| 17.4 | Generate CID for each event | 3 | CIM |
| 17.5 | Store events by CID | 3 | CIM |

**Acceptance Criteria:**
- [ ] Projections are pure functions
- [ ] Events have CID identifiers
- [ ] IPLD format on disk

### Retrospective Template

```markdown
## Sprint 17 Retrospective

### What Worked
-

### What Didn't Work
-

### Lessons Learned
-

### Changes for Next Sprint
-
```

---

## Sprint 18: Verification & Documentation

**Goal:** Verify all changes, document architecture

### Tasks

| ID | Task | Points | Expert Source |
|----|------|--------|---------------|
| 18.1 | Context boundary integration tests | 3 | Domain |
| 18.2 | Functor/arrow law property tests | 3 | ACT |
| 18.3 | Create doc/architecture/context-map.md | 2 | Domain |
| 18.4 | Create doc/DOMAIN-GLOSSARY.md | 2 | DDD |
| 18.5 | Update CLAUDE.md with new patterns | 2 | All |

**Acceptance Criteria:**
- [ ] All tests pass
- [ ] Documentation complete
- [ ] Team reviewed and approved

### Retrospective Template

```markdown
## Sprint 18 Retrospective

### What Worked
-

### What Didn't Work
-

### Lessons Learned
-

### Final Compliance Scores
| Expert | Before | After |
|--------|--------|-------|
| FRP | 55% | |
| ACT | 45% | |
| DDD | 55% | |
| CIM | 65% | |
| Graph | 45% | |
| NATS | 40% | |
```

---

## Dependency Graph

```
Sprint 11 (Categorical Foundations)
    │
    ├──► Sprint 12 (Routing Refactor) ──► Sprint 18
    │
    ├──► Sprint 13 (DDD Foundations)
    │        │
    │        └──► Sprint 14 (Context ACLs) ──► Sprint 18
    │
    ├──► Sprint 15 (Graph Structure) ──► Sprint 18
    │
    └──► Sprint 16 (NATS Integration)
             │
             └──► Sprint 17 (Event Hardening) ──► Sprint 18
```

---

## Risk Matrix

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking changes during routing refactor | HIGH | HIGH | Feature flags, incremental migration |
| Performance regression from pure projections | LOW | MEDIUM | Profile critical paths |
| Team unfamiliarity with ACT patterns | MEDIUM | MEDIUM | Pair programming, documentation |
| NATS integration complexity | MEDIUM | HIGH | Start with mock, add real NATS later |
| Scope creep | HIGH | HIGH | Strict story boundaries, PR reviews |

---

## Success Metrics

### Target Compliance After Completion

| Expert Area | Current | Target |
|-------------|---------|--------|
| FRP Axioms | 55% | 90%+ |
| Category Theory | 45% | 85%+ |
| DDD Patterns | 55% | 90%+ |
| Context Isolation | N/A | 100% |
| CIM Architecture | 65% | 95%+ |
| Graph Theory | 45% | 90%+ |
| NATS Integration | 40% | 95%+ |
| Subject Algebra | N/A | 90%+ |

---

## Evaluation Files Location

All detailed expert evaluations are available at:

```
/git/thecowboyai/cim-keys/doc/evaluations/
├── 01-frp-expert-evaluation.md
├── 02-act-expert-evaluation.md
├── 03-ddd-expert-evaluation.md
├── 04-domain-expert-evaluation.md
├── 05-cim-expert-evaluation.md
├── 06-graph-expert-evaluation.md
├── 07-nats-expert-evaluation.md
├── 08-subject-expert-evaluation.md
└── SPRINT_PLAN.md (this file)
```

---

## Next Steps

1. **Review** this sprint plan with stakeholders
2. **Prioritize** if full scope is too large
3. **Create tickets** in issue tracker
4. **Begin Sprint 11** with categorical foundations
5. **Hold retrospective** after each sprint
