# TODO — DSL Program Index

This folder is now organized around the DSL adoption program.
Primary roadmap reference: `docs/design/v4/dsl-roadmap.md`.
**Consolidated execution plan**: [`docs/design/v4/consolidated-worker-plan.md`](../docs/design/v4/consolidated-worker-plan.md) — unified dependency DAG, wave decomposition, and task assignments across all tracks.

Last reconciled: 2026-02-18

## Execution Order

1. Track A: DSL core compiler capabilities
2. Track B: DSL migration targets (move existing Rust DAG graphs to `.dag`)
3. Track C: modeling foundations needed for non-fragile DSL migration
4. Track D: runtime/test hardening required for confident rollout
5. Track E: domain parity and adjacent programs
6. Track F: general debt ledger

## Track Definitions

| Track | Purpose |
|------|---------|
| A — DSL Core | Language/compiler/runtime features needed by the roadmap |
| B — Migration Targets | Existing workflows that should be rewritten in DSL now |
| C — Modeling Foundation | Canonical models (platform, env, transport, composition) that remove stringly/manual wiring |
| D — Runtime/Test Hardening | Logging, testgen, codegen quality, and execution safety for DSL-generated flows |
| E — Domain Parity | Product/domain workstreams that should become DSL consumers over time |
| F — Debt Ledger | Generic cleanup and fallback debt not tied to one feature |

## Active Docs By Track

| Doc | Track | DSL Alignment | Status |
|-----|-------|---------------|--------|
| `TODO_URGENT_type_dispatch_boilerplate.md` | B | Critical prerequisite — blocks DSL migration | Active (BLOCKER) |
| `TODO_URGENT_dsl_migration.md` | B | Primary migration backlog | Active |
| `TODO_workflow_audit.md` | B | Migration inventory + sequencing input | Draft |
| `TODO_URGENT_anemic_modeling_audit.md` | C | Cross-cutting model consolidation | Active |
| `TODO_URGENT_platform_toolchain_modeling.md` | C | Target/platform/toolchain canonicalization | Partial (Phase 1 done) |
| `TODO_URGENT_logging_consolidation.md` | D | Execution observability hardening | Partial (5 fixed, 4 remain) |
| `TODO_testgen_seed_policy_postmortem.md` | D | Testgen semantic input correctness | Partial (core fix + 1/4 follow-ups) |
| `TODO_credential_lifecycle.md` | E | Credential/service modeling for DSL consumers | Draft |
| `TODO_gcp_infra_parity.md` | E | Domain parity backlog (DSL consumer target) | Partial (Phase 1 mostly done) |
| `consolidation.md` | F | Generic consolidation backlog | Ongoing (~12 done, ~30 remain) |
| `TODO_hacks.md` | F | Fallback/debt register | Partial (25 resolved, 7 open) |

## Recently Moved to TODONE (2026-02-18)

| Doc | Track | Reason |
|-----|-------|--------|
| `TODO_URGENT_browser_modeling.md` | C | Implementation complete; DSL surfacing is future scope |
| `TODO_transport_dag_migration.md` | C | C4.1-C4.4 complete; no Value extension needed |
| `design-codegen-quality.md` | D | Rust IR complete; cross-language is future scope |
| `llm-code-review-pipeline.md` | E | V0 complete; Tracks 2-6 done; Track 1 is future scope |

## Conventions

- Every active TODO doc should include:
  - `Status`
  - `Date` (or status date)
  - `DSL Alignment`
  - `Track`
- Use `TODO_URGENT_*.md` only for blockers or high-priority prerequisites.
- When complete, move docs to `TODO/TODONE/` and note completion date.
- If a doc is superseded, keep a short pointer at the top to the new source.

## Short Template

```markdown
# [Title]

**Status**: Draft | Active | In Progress | Completed
**Date**: YYYY-MM-DD
**DSL Alignment**: [one line]
**Track**: A | B | C | D | E | F

## Goal

[Outcome]

## Tasks

- [ ] Task A
- [ ] Task B
```
