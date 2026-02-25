# V4 Design Documents

The DSL design is the main document. Everything else is context.

## Task Tracking

- **[`TODO/tasks.md`](../../../TODO/tasks.md)** — The single execution queue. What to do next, dependency-ordered, parallelizable.
- **[`consolidated-worker-plan.md`](./consolidated-worker-plan.md)** — Architecture context: dependency DAG, cross-track relationships, wave decomposition. Informs the task order but is not the live task list.

## Primary

- **[`dsl-design.md`](./dsl-design.md)** — The language spec. Types, services, patterns, journeys, pipelines, compiler pipeline, emission targets, progress model, worked examples. This is the single source of truth for the DSL.
- **[`dsl-roadmap.md`](./dsl-roadmap.md)** — How to build it. Phased delivery (0-4), migration workstreams, modeling sweep, guardrails, definition of done.

## Architecture References

- **[`dsl-codegen-roadmap.md`](./dsl-codegen-roadmap.md)** — Track A (DONE): Computation → AbstractIR → language-tier lowering → rendering pipeline. 4 targets (Rust/Go/C/MIPS).
- **[`shared-abstractions.md`](./shared-abstractions.md)** — Cross-repo compatibility layer between gunbc and the-gunbai (EdgeKind, Effect, Value bridge, PortType).
- **[`workflow-modeling-preview.md`](./workflow-modeling-preview.md)** — Phase 0.5 deliverable: builder-shape vs DSL-shape parity proofs.
- **[`sandbox-replay-rfc.md`](./sandbox-replay-rfc.md)** — Runtime policy proposal for sandbox deny/allow/replay semantics.
- **[`gist-recent-credential-diagnostics.md`](./gist-recent-credential-diagnostics.md)** — Baseline trace + fallback analysis for credential resolution.
- **[`domain-hard-error-no-fallback-plan.md`](./domain-hard-error-no-fallback-plan.md)** — Domain-model plan for eliminating string tables, passthrough/stub fallbacks, and ad-hoc embed registries (post PR1-PR5 items 5/7/8).

## Background (how we got here)

- **[`bl1-retrospective.md`](./bl1-retrospective.md)** — What went wrong in the-gunbai's modeling layer. The diagnosis that led to V2/V3.
- **[`v3-contracts-minimal.md`](./v3-contracts-minimal.md)** — The conceptual core: one recursive type (`Node<T>/Dag<T>`), fractal DAG.
- **[`v2-contracts-design.md`](./v2-contracts-design.md)** — Full type system design for the modeling layer.
- **[`v3-worked-examples.md`](./v3-worked-examples.md)** — Concrete examples through the fractal lens.
- **[`v2-worked-examples.md`](./v2-worked-examples.md)** — Before/after comparisons.
- **[`dag-systems-overview.md`](./dag-systems-overview.md)** — The Go-era DAG system (historical).

## The arc

```
Go DAGs (dag-systems-overview)
  → "what went wrong in Rust" (bl1-retrospective)
    → "one recursive type" (v3-contracts-minimal)
      → "a language for it" (dsl-design)  ← you are here
        → "how to build it" (dsl-roadmap)
          → "what to do next" (TODO/tasks.md)
```
