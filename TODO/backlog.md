# Backlog — Prioritized

**Last updated**: 2026-02-25
**Policy**: Items reviewed quarterly. P3 items not promoted within 2 quarters are deleted.
**Promotion**: Move to `TODO/tasks.md` when scheduled for active work.

## Priority Tiers

- **P1**: Feeds active lanes or unblocks near-term work. Promote next.
- **P2**: Valuable, clear use case, but not urgent. Promote when capacity opens.
- **P3**: Speculative or low-urgency. Subject to deletion if stale.

---

## P1 — Promote Next

| ID | Item | Size | Design Doc | Feeds | Added |
|----|------|------|------------|-------|-------|
| **NF-7** | **Lowerer extern func wiring**: `TypedItemSignature::ExternFunc` + `lower_extern_call()` for same-module calls from function bodies. Design complete. Blocks Phases 5-8 extern bridge elimination. | L | `docs/design/v4/externcall-same-module-port-wiring.md` | Extern bridge elimination | 2026-02 |
| **DG1** | **Daggen (dynamic DAG generation)**: `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | — | Lane 2 (compiled pipeline) | 2026-02 |

## P2 — Promote When Capacity Opens

| ID | Item | Size | Design Doc | Feeds | Added |
|----|------|------|------------|-------|-------|
| **H10** | **Compute stack orchestration**: Provision/apply DAG builder for Cloud Run, GCS, LB lifecycle. Service layer exists; orchestration missing. | L | `docs/design/horizon/h10-compute-stack-services.md` | Lane 2 (SDLC infra) | 2026-02 |
| **S12-E** | **Multi-worker CAS**: `GcsClaimStore` with generation-based CAS (`x-goog-if-generation-match`). DSL exists; wiring deferred until cloud_run profile needed. | M | — | Lane 2 (cloud deployment) | 2026-02 |

## P3 — Speculative (Review 2026-Q3, Delete if Not Promoted)

| ID | Item | Size | Design Doc | Notes | Added |
|----|------|------|------------|-------|-------|
| **H1** | **Display reactive DSL**: Channel-driven event loop with `on`/`tick` triggers. Requires new DSL parser primitives, IR nodes, runtime scheduler. | XL | `docs/design/horizon/h1-display-reactive-dsl.md` | No current use case. Requires significant new DSL infra. | 2026-02 |

---

## Archived (2026-02-25)

Items removed from backlog because they shipped:
- **H2** (Testgen dynamic targets), **H3** (Makegen tool registry), **H4** (Loop extra inputs), **H7** (Resource abstraction trait), **H8** (Justfile rendering), **H9** (GitHub Actions rendering), **H11** (DAG typing hardening) — all completed, see `TODO/TODONE/tasks-completed.md`.
