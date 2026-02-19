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
- DSL resource declarations → Rust codegen integration
- Cross-service composition planner

**Why backlogged**: Service layer works, but orchestration is XL scope and not needed
until infra provisioning is an active use case.

---

## Typed API Migration (H11 follow-up)

`TypedPort<T>`, `TypedInput<T>`, `TypedOutput<T>` exist and work. Legacy untyped
`Port` API still active. Full migration to typed-only would touch most builders.

**Why backlogged**: Typed wrappers are available for new code. Migration of existing
builders is mechanical but wide-blast-radius — lower priority than business flows.

---

## Resource Trait String Port Elimination (H7 follow-up)

`Resource` trait, `AccessMode`, `ManagedResource` all exist. String `res:*` ports
still coexist with the typed resource system.

Current guardrail state:
- wildcard file ports (for example `res:file:*`, `res:file:src/*`) are normalized
  to coarse `res:file` for resource accounting/admission.
- scheduler admission treats coarse `res:file` as conflicting with any specific
  `res:file:<path>` lock.
- generated makegen graphs now use coarse `res:file` directly.
- true glob-aware admission semantics are intentionally deferred.

**Why backlogged**: Resource trait works for new code. Full elimination of string ports
is a cross-cutting migration with many touchpoints.

### Deferred follow-up: Glob-aware Resource Admission

Define and implement wildcard semantics end-to-end for resource locks (not just file guard):
- canonical pattern model (`*`, prefix/suffix/infix) and conflict matrix.
- shared matcher used by scheduler admission + runtime guard.
- deterministic tie-breaking and fairness when wildcard and specific locks contend.
- decide migration path for legacy `res:file:*` acceptance in runtime guard.

**Why backlogged**: This is policy-sensitive concurrency behavior and needs explicit
design before enabling in runtime scheduling.

---

## Canonical Port Naming Invariants (R1 follow-up)

Some paths still rely on module-specific port aliases in normalization/parity logic
(for example makegen transport output aliases). The long-term direction is one
canonical port name per semantic role across lowering, runtime emission, and snapshots.

**Why backlogged**: Mechanical cleanup is easy, but cutover needs careful snapshot
and parity coordination to avoid false regressions across multiple toolchains.
