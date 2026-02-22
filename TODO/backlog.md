# Backlog — Feature Ideas (Not Scheduled)

These are large-scope or speculative features parked for future consideration.
They are **not on the active roadmap** — move to `tasks.md` when prioritized.

---

## Display Reactive DSL (was H1)

**Size**: XL | **Design**: `docs/design/horizon/h1-display-reactive-dsl.md`

Channel-driven event loop with `on`/`tick` triggers for display orchestration.
Requires new DSL parser primitives (`reactive`, `on`, `tick`), IR nodes
(`ReactiveSubDag`, typed channels), and a runtime scheduler.

**Why backlogged**: Requires significant new DSL infrastructure that doesn't exist.
Core process needs stress-testing with existing primitives first.

---

## Compute Stack Provision/Apply Orchestration (was H10, remaining work)

**Size**: L | **Design**: `docs/design/horizon/h10-compute-stack-services.md`

Service trait definitions and REST adapters for Cloud Run, GCS, LB, and Compute Engine
are implemented (`lib/gcp-ops/src/services/`). Discovery DAG exists. What's missing:

- Provision/apply DAG builder (create/update/release lifecycle)
- DSL resource declarations -> Rust codegen integration
- Cross-service composition planner

**Why backlogged**: Service layer works, but orchestration is XL scope and not needed
until infra provisioning is an active use case.

---

## Glob-aware Resource Admission

Define and implement wildcard semantics end-to-end for resource locks (not just file guard):
- canonical pattern model (`*`, prefix/suffix/infix) and conflict matrix.
- shared matcher used by scheduler admission + runtime guard.
- deterministic tie-breaking and fairness when wildcard and specific locks contend.
- decide migration path for legacy `res:file:*` acceptance in runtime guard.

**Why backlogged**: This is policy-sensitive concurrency behavior and needs explicit
design before enabling in runtime scheduling.

---

## Completed (removed from backlog 2026-02-22)

The following former backlog items were completed as part of Lane 4 (Codebase Polish):

- **Typed API Migration** (was H11 follow-up) -> Completed as CU-7: all untyped `port(...)` calls migrated to `typed_port::<T>()`.
- **Resource Trait String Port Elimination** (was H7 follow-up) -> Completed as CU-8: all `res:*` string constructors normalized to typed resource system.
- **Canonical Port Naming Invariants** (was R1 follow-up) -> Completed as CU-9: standardized on canonical `file:write` + `return` across lowering/runtime/snapshots.
