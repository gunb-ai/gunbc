# TODO — Active Plan Index

Active and planned work items live in this folder. When a plan is complete,
move it to `TODO/TODONE/`.

Last reconciled: 2026-02-14

## Active Plans

| Plan | Status | Notes |
|------|--------|-------|
| `TODO/TODO_URGENT_logging_consolidation.md` | Active | CI/local display + logging parity and failure-first output hardening |
| `TODO/TODO_hacks.md` | Active | Consolidated runtime/design debt tracker |
| `TODO/TODO_gcp_infra_parity.md` | In Progress | Primary implementation checklist for GCP infra parity |
| `TODO/TODO_credential_lifecycle.md` | Draft | Architecture reset for auth/credential lifecycle |
| `TODO/TODO_workflow_audit.md` | Draft | Workflow consolidation, purity/resource, parallelization roadmap |
| `TODO/consolidation.md` | Ongoing | Broad consolidation + remaining design-dependent tasks |
| `TODO/llm-code-review-pipeline.md` | V0 complete, Track 1 open | Resource abstraction track still pending |
| `TODO/design-codegen-quality.md` | Active (ongoing concern) | Generated code quality and backend idiom coverage |
| `TODO/TODO_transport_dag_migration.md` | Draft | Recommended migration plan, not implemented end-to-end |
| `TODO/TODO_testgen_seed_policy_postmortem.md` | Partially complete | Core fix landed; follow-up hardening still open |
| `TODO/TODO_remove_disallowed_methods_script.md` | Pending | Script/allowlist removal task |

## Source Of Truth Notes

- For GCP infra execution progress, treat `TODO/TODO_gcp_infra_parity.md` as
  the canonical tracker. `docs/design/gcp-service-modeling.md` is architecture
  reference.
- For completed work, prefer links under `TODO/TODONE/`; stale `TODO/...`
  references should be updated during doc edits.
- Hack debt is primarily tracked in `TODO/TODO_hacks.md`; root-level
  `TODO_hacks` contains additional historical notes that may still have open
  items.

## Recently Moved To TODONE

- `TODO/TODONE/TODO_ci_timeout_fermi.md` (moved 2026-02-14)

## Plan Template

```markdown
# [Feature Name]

**Status**: Draft | In Progress | Completed
**Date**: YYYY-MM-DD

## Goal

[What we're trying to achieve]

## Design

[Architecture, data models, diagrams]

## Tasks

- [ ] Task A
- [ ] Task B
- [ ] Task C

## Notes

[Implementation notes, decisions made]
```
