# Backlog — Prioritized

**Last updated**: 2026-02-23
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
| **H11** | **DAG typing hardening**: Typed node I/O wrappers at DAG boundaries, fail-closed semantic carrier refinement. | M | `docs/design/horizon/h11-dag-typing-hardening.md` | Lane 1 (type system), Lane 3 (M8 metadata) | 2026-02 |
| **H2** | **Testgen dynamic targets**: Generate test targets by iterating `DagSpecDef` inventory at codegen time instead of manual enumeration. | M | `docs/design/horizon/h2-testgen-dynamic-targets.md` | M14 (single inventory), M20 (repo self-model) | 2026-02 |
| **H3** | **Makegen tool registry**: Make `#[tool_target]` the single source of truth for tool registry instead of hand-maintained lists. | S | `docs/design/horizon/h3-makegen-tool-registry.md` | M14 (single inventory), M20 (repo self-model) | 2026-02 |
| **DG1** | **Daggen (dynamic DAG generation)**: `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | — | Lane 2 (compiled pipeline) | 2026-02 |

## P2 — Promote When Capacity Opens

| ID | Item | Size | Design Doc | Feeds | Added |
|----|------|------|------------|-------|-------|
| **H8** | **Justfile rendering**: Adopt Justfile as a second workflow renderer to validate model portability. | M | `docs/design/horizon/h8-workflow-rendering-justfile.md` | M18 (projection-only surfaces) | 2026-02 |
| **H9** | **GitHub Actions rendering**: GitHub Actions YAML as additional CI provider generated from shared `WorkflowSpec`. | M | `docs/design/horizon/h9-workflow-rendering-github-actions.md` | M18 (projection-only surfaces) | 2026-02 |
| **H10** | **Compute stack orchestration**: Provision/apply DAG builder for Cloud Run, GCS, LB lifecycle. Service layer exists; orchestration missing. | L | `docs/design/horizon/h10-compute-stack-services.md` | Lane 2 (SDLC infra) | 2026-02 |
| **S12-E** | **Multi-worker CAS**: `GcsClaimStore` with generation-based CAS (`x-goog-if-generation-match`). DSL exists; wiring deferred until cloud_run profile needed. | M | — | Lane 2 (cloud deployment) | 2026-02 |
| **H4** | **Loop extra inputs passthrough**: Support additional context (config, auth, branch) through loop bodies. | S | `docs/design/horizon/h4-loop-extra-inputs-passthrough.md` | — | 2026-02 |

## P3 — Speculative (Review 2026-Q3, Delete if Not Promoted)

| ID | Item | Size | Design Doc | Notes | Added |
|----|------|------|------------|-------|-------|
| **H1** | **Display reactive DSL**: Channel-driven event loop with `on`/`tick` triggers. Requires new DSL parser primitives, IR nodes, runtime scheduler. | XL | `docs/design/horizon/h1-display-reactive-dsl.md` | No current use case. Requires significant new DSL infra. | 2026-02 |
| **H7** | **Resource abstraction trait**: Full `Resource` trait migration eliminating string `res:*` ports. Includes glob-aware resource admission (wildcard semantics, conflict matrix, deterministic tie-breaking). | L | `docs/design/horizon/h7-resource-abstraction-trait.md` | Typed resource system works for new code. Migration is wide-blast-radius. Tracked as CU-8 in Lane 4 for incremental work. | 2026-02 |

---

## Cross-References

Some backlog items overlap with Lane 4 (polish) tasks that handle incremental progress:

| Backlog Item | Lane 4 Task | Relationship |
|--------------|-------------|-------------|
| H11 (DAG typing) | CU-7 (Typed API migration) | CU-7 is the mechanical migration; H11 is the full design. |
| H7 (Resource trait) | CU-8 (Resource string port elimination) | CU-8 is incremental cleanup; H7 is the full replacement. |
| — | CU-9 (Canonical port naming) | Standalone polish, no backlog equivalent. |
