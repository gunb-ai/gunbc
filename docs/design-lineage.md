# Design Lineage: gunb.ai → the-gunbai → gunbc

> Part of: [THESIS.md](../THESIS.md) > [src/v3/spec.dag](../src/v3/spec.dag)

How the current v3 design emerged across four repos. The same
principles appear at every stage — they just get sharper.

---

## Stage 1: gunb.ai — DAG as orchestration

**What it was:** A pseudo-static-analysis system that schedules
work (CI, infrastructure, LLM tasks) as DAGs with contracts.

**Core concepts established:**

- **Node = identity + dependencies.** The entire interface:
  `NodeID()` and `NodeDependsOn()`. Everything else is optional.
  (gunb.ai/OaaS_v2/pkg/dag/dag.go)

- **Contract = provides + requires.** Every node declares what
  data it exports and what data it imports. The framework validates
  that every import is satisfied by a prior node's export.
  (gunb.ai/OaaS_v2/pkg/dag/contract.go)

- **Waves = parallelism from structure.** Nodes with no dependency
  between them land in the same wave. Waves execute in parallel.
  No parallelism annotations. The dependency structure IS the
  schedule. (gunb.ai/docs/architecture/dag-patterns.md)

- **L0/L1/L2 layering.** Start minimal, extract upward. L0 is
  identity + dependencies. L1 is composable appendages (state
  machines, leases, phases). L2 is composed patterns. Don't
  over-engineer before seeing real patterns emerge.
  (gunb.ai/docs/UNIFIED_DAG_PHASE1_SPEC.md)

- **Validate early, fail loudly.** Catch errors at definition
  time, not at runtime. Missing dependencies, cycles, missing
  required outputs — all build-time errors.
  (gunb.ai/docs/architecture/dag-patterns.md)

- **L1 behavior patterns.** Loop, Poll, Retry, Transform,
  Pipeline, Spawn — fundamental execution patterns that compose.
  L2 (Freshen) composes L1 (Loop + Transform + domain check).
  (gunb.ai/OaaS_v2/pkg/dag/patterns/)

**What carried forward:** Node + Contract + Waves + L-layering +
validate-early. ALL of these survive into v3.

---

## Stage 2: the-gunbai — DAG as mini compiler

**What it was:** A Rust-based compiler that generates code from
DAG specifications. The shift: from runtime orchestration to
compile-time code generation.

**What was added:**

- **Codegen layering.** Three-layer poset: Contracts < Codegen <
  Full. Build dependencies flow downward, generated outputs flow
  upward. The monotonicity rule `layer(node) >= layer(dep)` makes
  cycles unrepresentable by construction.
  (the-gunbai/spec/codegen-layering.md)

- **Bootstrap tiers.** Foundation (no generated sources) →
  Bootstrap (must have committed targets) → Generated (can be
  generated-only). Derived from dependency graph closure.
  (the-gunbai/spec/bootstrap-tier.md)

- **Behavior patterns as first-class.** Upsert, Transaction,
  Retry, CRUD — modeled as typed abstractions with phase tagging.
  Each phase enforces properties (Check → ReadOnly). Patterns
  generate contract tests automatically.
  (the-gunbai/spec/behavior-patterns.md)

- **Demand-driven freshness.** Consumer tracking replaces manual
  regeneration. Staleness detection per consumer.
  (the-gunbai/spec/demand-driven-codegen.md)

- **Producer-centric model.** Make the producer the primary entity.
  Duplicates become unrepresentable.
  (the-gunbai/spec/producer-centric-generators.md)

- **Set theory rigor.** SetSpec<T> = Empty | Universal | These(&[T]).
  Eliminated ambiguous empty-slice semantics. Formal intersection,
  union, partition validation.
  (the-gunbai/spec/set-theory-rigor.md)

- **Multi-layer verification.** MockUnit < Simulated < Integration
  < E2E < Manual. Evidence tracking. CI enforces minimum tier.
  (the-gunbai/spec/confidence-verification-levels.md)

**What carried forward:** Codegen layering → emission architecture.
Bootstrap tiers → self-hosting cycle. Behavior patterns → std/
type system. Set theory rigor → closed system design. All survive
into v3.

---

## Stage 3: gunbc v2 — DAG as full compiler

**What it was:** A self-hosted compiler in .dag that compiles
.dag source to Rust/Python/Go.

**What was added:**

- **Self-hosting.** The compiler compiles itself. Bootstrap
  converges.

- **Bounded computation model.** Three iteration primitives
  (fold/descend/repeat). All programs terminate by construction.
  Decidability is not checked — it's unrepresentable to violate.

- **Structural type system.** Types are Node trees with
  connectives (Conj/Disj/Arrow). Inference is reconciliation.

- **Algebraic type inhabitants.** Types carry algebraic structure
  (Monoid, Ring, Lattice). Operations emerge from inhabitation.

- **Interpreter.** `dag run` executes validated IR directly.
  Reads the same transport specs as the emitter. Proves the IR
  is a complete computational description.

**What went wrong:**

- **Node became a god struct.** 17 fields mixing authored syntax,
  semantic facts, and compiler state. Any field change is
  cross-cutting across 38K lines.

- **TypeBinding threw away provenance.** Inference computed rich
  facts, stored only the resolved type. Every downstream consumer
  reconstructed what was discarded.

- **21 ExprData variants.** 665 match arms across consumers.
  Only 40-50 truly distinct. 11 "interaction" variants are the
  same structural shape with different data.

- **Complexity and ownership bolted on.** Separate analysis passes
  that re-walk the IR to reconstruct discarded structure.

**Diagnosis:** The physics (the DAG structure) was incomplete.
The lenses (complexity, ownership, effects) couldn't read what
they needed, so they reconstructed it with heuristics.

---

## Stage 4: definitely-not-agi — epistemological foundation

**What it is:** Papers on G-B logic, intersubjectivity, set
theory, abstraction calculus.

**What it contributed:**

- **Intersubjectivity.** Models are shared agreements, not
  objective truths. This IS .dag's modeling philosophy: "shared
  facts, not preferences."

- **Conceptual lenses.** From G-B set theory: sets are not
  containers but lenses you look through. Membership is
  context-dependent. Directly informs: complexity, ownership,
  effects are lenses over the DAG, not properties OF the DAG.

- **Epistemic stacking.** Build from simplest primitives, layer
  composition. Each layer makes sense on its own. Maps directly
  to L1/L2 behavior layering.

- **Abstraction calculus.** A primitive is (Universe, Codomain,
  Lens, Mode). Abstraction is the kernel quotient. Re-priming
  treats the first abstraction's carrier as the next universe.
  This is the formal basis for "design the physics, analyses
  are lenses."

---

## Stage 5: gunbc v3 — physics + lenses

**What it is:** The v3 specification, grounded in all prior work.

**Design DNA from each stage:**

| Principle | Origin | v3 form |
|-----------|--------|---------|
| Node = identity + dependencies | gunb.ai L0 | Still the foundation. Behaviors have id + ports. |
| Contract = provides + requires | gunb.ai contracts | NodeContract on each behavior. Compiler validates. |
| Waves = parallelism from structure | gunb.ai waves | Independent behaviors execute concurrently. No annotations. |
| L0/L1/L2 layering | gunb.ai UNIFIED_DAG spec | L1 behaviors (Value, Transform, Branch, Loop, Bind). L2 composes. |
| Start minimal, extract upward | gunb.ai design philosophy | Don't over-engineer lenses. Let patterns emerge. |
| Validate early, fail loudly | gunb.ai dag-patterns | Compiler catches all errors. Emission is mechanical. |
| Codegen layering | the-gunbai | Emission reads specs. No cycles. |
| Bootstrap tiers | the-gunbai | Self-hosting cycle is architecture, not tooling. |
| Set theory rigor | the-gunbai | Closed system. Finite types. No ambiguity. |
| Behavior patterns | the-gunbai | L1 behaviors are the patterns: Transform, Loop, Branch. |
| Multi-layer verification | the-gunbai | L0-L7 verification tiers from THESIS.md. |
| Bounded computation | gunbc v2 | All iteration bounded. Decidability by construction. |
| Conceptual lenses | definitely-not-agi | Analyses are lightweight views over the physics. |
| Epistemic stacking | definitely-not-agi | L1 → L2 → properties. Each layer self-contained. |
| Intersubjectivity | definitely-not-agi | Models are shared facts. Compiler validates consistency. |

**The v3 test (sustainability):** can we add a lens we didn't
think of at design time, and what does it cost? Answer: define
what you're measuring, define how it composes, done. Zero compiler
changes. Zero heuristics. Because the physics is complete.

---

## What changed at each transition

```
gunb.ai           → the-gunbai         → gunbc v2           → v3 spec
─────────────────────────────────────────────────────────────────────────
Runtime DAG         Compile-time DAG     Self-hosted compiler  Physics + lenses
Go execution        Rust codegen         .dag → Rust/Py/Go    L1 behaviors
Scheduling          Code generation      Full type system      5 behaviors
Wave parallelism    Layer acyclicity     Bounded computation   Lenses over DAG
Contracts           Behavior patterns    Algebraic types       Zero heuristics
```

Each transition preserved the core (Node + Contract + Waves) and
added a new capability. The v3 transition's addition: the physics
is rich enough that ALL analyses are lenses. Nothing is bolted on.
