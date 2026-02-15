# V4 Design Documents

The DSL design is the main document. Everything else is context.

## Primary

- **[`dsl-design.md`](./dsl-design.md)** — The language spec. Types, services, patterns, journeys, pipelines, compiler pipeline, emission targets, progress model, worked examples. This is the single source of truth for the DSL.
- **[`dsl-roadmap.md`](./dsl-roadmap.md)** — How to build it. Phased delivery (0-4), migration workstreams, modeling sweep, guardrails, definition of done.

## Background (how we got here)

Read these to understand *why* the DSL looks the way it does.

- **[`bl1-retrospective.md`](./bl1-retrospective.md)** — What went wrong in the-gunbai's modeling layer. String-typed semantics, missing behavior sub-DAG, bottom-up vocabulary. The diagnosis that led to V2/V3.
- **[`v3-contracts-minimal.md`](./v3-contracts-minimal.md)** — The conceptual core: one recursive type (`Node<T>/Dag<T>`), fractal DAG, the tower of abstraction levels. Traces the design to the Abstraction Calculus. The DSL is this idea applied to workflow authoring.
- **[`v2-contracts-design.md`](./v2-contracts-design.md)** — Full type system design for the modeling layer. Patterns as sub-DAG templates, Lane A/B/C extension model, typed semantic channels. The DSL's design principles (C1-C11) descend from V2's P1-P4.

## Reference

- **[`v3-worked-examples.md`](./v3-worked-examples.md)** — Concrete examples (zstd, git, tectonic) through the fractal `Node<T>/Dag<T>` lens.
- **[`v2-worked-examples.md`](./v2-worked-examples.md)** — Before/after comparisons showing what typed contracts replace.
- **[`dag-systems-overview.md`](./dag-systems-overview.md)** — The Go-era DAG system (gunb.ai). Historical reference for the `Contractor`/`NodeContract` pattern that started all of this.

## The arc

```
Go DAGs (dag-systems-overview)
  → "what went wrong in Rust" (bl1-retrospective)
    → "one recursive type" (v3-contracts-minimal)
      → "a language for it" (dsl-design)  ← you are here
        → "how to build it" (dsl-roadmap)
```

The key insight connecting V3 to the DSL: V2/V3 solved the modeling layer (how to describe tool behaviors as typed causal DAGs). The DSL extends the same principles to the workflow authoring layer (how to *write* workflows as typed causal DAGs, with the compiler generating everything else).
