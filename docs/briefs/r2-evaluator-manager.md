# R2 Evaluator Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), Goal 7 added 2026-04-28 via PR #1078). Spawns post-#1078-merge per Transition mechanics step 4. **No prior brief to migrate** — this is a genuinely new R2 manager.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of **7** standing R2 managers (alongside Substrate, Modeling, Grounding, Impossible-Bugs, Pure Bootstrap, R2 Release; cross-program coordination via Director). Manager count rose from 6 to 7 with this lane added per #1078.
- **Program scope source:** [`THESIS.md`](../../THESIS.md) §"Tier 3 — Verification from structure" (L4-L7 verification surface) + [`docs/r2-structure.md`](../r2-structure.md) §"Evaluator Manager (added 2026-04-28 amendment)".
- **Cross-program consumer:** **R2-Evaluator gates 7 of 10 R3 lanes** (T-Tier3-Dissolution, T-LensProducer-Retirement, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition). The Evaluator IS the runtime that R3's consequence layer falls out from. Without it, R3 dispatchers spin.
- **Demo coordination:** signal lane-close to R2 Release Manager (closure ledger; per the structural-acceptance-per-lane-close discipline locked in `r2-structure.md` — the demo IS the structural gate, not a separate artifact).
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1): self-serve through the 3-step decision procedure (DAG-ancestor → coproduct-vs-coordinate → primitive-vs-lens-extensible) before escalating substrate-shape questions to Director.

## Program scope (T-Evaluator XL)

R2's Goal 7 — **Runtime evaluator for `.dag` programs**. The Evaluator is the runtime layer that:
- Executes `.dag` function bodies structurally (not via the Rust mirrors)
- Applies lenses over reflected program DAGs
- Constructs runtime witnesses (proof artifacts)
- Provides cross-target equivalence harness primitives for L5 verification in R3

**Co-XL with T-Substrate** — largest single new lane in R2. Largest single concentration of new R2 work; gates R3 spin-up.

## Owned deliverables (through R2 close)

| Sub-lane | Size | Status (at brief authoring) | Description |
|---|---|---|---|
| **Runtime value model** | M | NOT YET AUTHORED — gated on PR-A design lock | Closed-over environments, lazy/eager evaluation strategy, memoization. Per #1078 design challenge #1: locked direction; specific design lands in PR-A. |
| **Body evaluator** | L | NOT YET AUTHORED — gated on Runtime value model | Execute `.dag` function bodies structurally. Bounded forward execution per INVARIANTS P4. Termination by descent evidence (already in substrate per `dsl/std/termination.dag`). |
| **Lens application** | M | NOT YET AUTHORED — Reflection completeness spec LANDED via #1129 ([`docs/design-reflection-completeness.md`](../design-reflection-completeness.md)) | Extend `reflect_program_dag_nodes_in_file` from "shallow/lossy" to complete reflection per [`docs/design-reflection-completeness.md` §"Decision"](../design-reflection-completeness.md). Lens application = fold over reflected program DAG via `Lens<C>` framework (lands as R2-T-Substrate-Lens-Primitive sub-lane). |
| **Witness construction** | M | NOT YET AUTHORED | Runtime materialization of proof artifacts (`Witness::Inhabits` / `Witness::Violates` per `src/v3/std/dimensions.dag`); algebraic-law witnesses (associativity, commutativity, identity). |
| **Cross-target equivalence harness primitives** | S | NOT YET AUTHORED | For L5 verification in R3 (algebraic equivalence over a curated corpus, per #1078 design challenge #3 locked decision). Primitives only — corpus authoring is post-R2 (R3 lane T-Verification-L5-Corpus). |

## Pre-dispatch design lock cadence (per #1078 locked structure)

Before worker dispatch begins on the implementation sub-lanes above, **5 design PRs land per their dependency graph** (parallelize where independent; serialize where there's a real dependency):

```
                    ┌───────→ PR-B (Witness construction surface)
                    │              │
       PR-A ────────┤              ↓
       (Runtime     │              │
        value       ├───────→ PR-E ←─── PR-C (Reflection completeness spec)
        model)      │       (Final         │
                    │       integration)   ↓
                    └─────────────────────┴── (consumed by R3-T-LensProducer-Retirement)

       PR-D (L5 corpus type spec) ──→ PR-E (independent of A/B/C; cross-target equivalence)
                                          (consumed by R3-T-V-L5-Corpus)
```

| PR | Depends on | Parallelizable with | Locks | Status |
|---|---|---|---|---|
| **PR-A** | (foundational) | — | Runtime value model — closed-over environments, lazy/eager strategy, memoization | NOT YET AUTHORED |
| **PR-B** | PR-A | PR-C, PR-D | Witness construction surface — concrete shape for runtime materialization | NOT YET AUTHORED |
| **PR-C** | (foundational; substrate-reflection-shape) | PR-A, PR-B, PR-D | Reflection completeness spec — what does "complete reflection" mean for `reflect_program_dag_nodes_in_file`? | **LANDED via #1129** at [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) (consumed by R3-T-LensProducer-Retirement per §"Cascade and gates") |
| **PR-D** | (foundational; cross-target spec) | PR-A, PR-B, PR-C | L5 corpus type spec — algebraic equivalence over curated corpus (locked direction; specific design here) | NOT YET AUTHORED (consumed by R3-T-V-L5-Corpus) |
| **PR-E** | All of PR-A through PR-D | (synthesis; serializes after) | Final integration design — synthesizes PR-A through PR-D into the implementation roadmap | NOT YET AUTHORED |

**Sequencing principle (per Director directive 2026-04-28):** "focus on dependencies — writing code is fast; we get stuck in review." Parallelize PR-A, PR-C, PR-D as independent foundational design locks; PR-B serializes after PR-A (witness construction uses runtime values); PR-E synthesizes after all four. Worker dispatch on implementation sub-lanes blocks on PR-E.

Plus **LanguageSpec parallel** (R2-T-Ground-LanguageSpec sub-lane) — Grounding Manager authors the LanguageSpec schema in parallel; Evaluator Manager consumes it for cross-target equivalence work. Independent of PR-A through PR-E.

**Timing — option (c) hybrid (Director-locked 2026-04-28 via dialogue):**

- **PR-A through PR-D** = design-only (no worker dispatch). **Dispatch immediately post-#1078-merge** — these don't conflict with R1 closure work because no workers are running on them yet. Director writes design docs; the structural-discipline handshake lives where workers dispatch, not here.
- **PR-E (Final integration + worker dispatch brief)** = the load-bearing handshake. **Wait on R1-Closure-Manager-signals-R1-close** before authoring + dispatching. R1 closure window is small (fixture authoring only — single `r1_release_acceptance.dag` + R1C-B's 3 fixtures); the wait is days, not weeks.
- **Worker dispatch on implementation sub-lanes** = blocks on PR-E + R1 close signal jointly.

Rationale: artificially delaying PR-A through PR-D wouldn't preserve any real handshake invariant — those are design docs. The structural discipline (R1→R2 transition mechanics, manager spawn ordering, dispatch-discipline) fires at worker-dispatch time, which is exactly where PR-E + the joint wait gates it.

## Cross-program dependencies

**Produces:**
- **R3 lane gates** — 7 of 10 R3 lanes block on R2-Evaluator landing. The Evaluator's `Witness` runtime (per `src/v3/std/dimensions.dag`) is what R3-T-Tier3-Dissolution / R3-T-LensProducer-Retirement / R3-T-V-L4-L7-Direct / etc. consume.
- **`Lens<C>` runtime** — R2-T-Substrate-Lens-Primitive's generic `fold_lens<C>: Lens<C> → Dag → DimensionReport<C>` is implemented by the Evaluator. T-CostLens-Composition (R3, under Substrate continuation per #1078 lock) consumes this.

**Consumes:**
- **Substrate Manager** — additional carriers needed by runtime values (e.g., closed-over environment representation). Design-pass at lane spin-up identifies the dependency.
- **Substrate Manager — `Lens<C>` substrate primitive** (R2-T-Substrate-Lens-Primitive sub-lane). Evaluator implements `fold_lens<C>`; substrate declares the type.

## Locked design decisions consumed (per #1078 8-question dialogue)

The Evaluator's substrate decisions inherit from #1078's locked design questions. Worker briefs MUST consume these without re-litigation:

- **Q1**: `Interval<D>` shared parent in substrate; `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent` (asymmetric match rule)
- **Q3**: `Cost<Unit> = Dimension<Unit, SymbolicExpr>`; `RealizationCost { storage: Cost<Bits>, access: Map<AlgebraOp, Cost<CPUCycles>> }`
- **Q6 + Q6.5 (LANDED via #1129)**: `Witness<C>` stays as-is; structural validation failures encode via two-layer diagnostic-kind authority per [`docs/design-lens-framework.md` §"Q6.5 — Two-layer authority for diagnostic kinds"](../design-lens-framework.md). Layer 1 = `CompilerDiagnosticKind` (Substrate-owned, untouched); Layer 2 = lens-instance kinds declared in lens's own `.dag` via structural inhabitance (Evaluator authors per-lens kinds without Substrate handoff). Anti-shadowing: Layer-2 names cannot reuse Layer-1 variants.
- **Q7**: per-call validate yields one `OptionalDiagnostic`; fold accumulates into `DimensionFail.violations: List<Diagnostic>`
- **Q8**: cross-product validate is conjunctive (`Lens<C> × Lens<D>` runs both; conjunctive fold)

Full disposition table: [`docs/r2-structure.md`](../r2-structure.md) §4 + [`docs/design-lens-framework.md`](../design-lens-framework.md) §"Design questions to lock before substrate dispatch".

## Pre-spawn vs post-spawn authority

- **Pre-spawn (this brief authored post-#1078-merge by PM/Director):** brief authoring + PR-A through PR-E design-lock cadence sequence locked. Manager spawns once a design-lock PR is dispatchable. PR-A/B/C/D dispatch immediately (design-only, no workers); PR-E + worker dispatch wait on R1 close signal per option (c) hybrid above.
- **Post-spawn (Evaluator Manager active):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

**R1 test-infrastructure precedent** (per Director coordination 2026-04-28; acknowledged in PR-E worker-dispatch brief — NOT substrate input): R1 closure produced reusable test-infrastructure patterns that PR-E should reference where applicable for R2-Evaluator's own structural-acceptance fixtures:
- **TestPredicate variants from R1C-D** (`CensusBoundCheck` / `CensusSubsetCount` / `RatchetZero` / `GeneratedFromDag` / `FixedPointConverges`) — reusable test-predicate pattern; relevant for Evaluator's `.dag` TestClaim acceptance gates (Acceptance section above).
- **Concession-encoding pattern from `r1_release_acceptance.dag`** (in flight via still-seal-529) — structural-fact-cites-lane-authority shape. Reusable for R2 lane-close fixtures with similar evaluator-ready-vs-substrate-pending disposition.

**Explicitly NOT from R1** (Director-confirmed 2026-04-28): reflection completeness now lives in [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) (LANDED via #1129; was PR-C placeholder); witness construction is R2-Substrate authority (`Witness<C>` + `DimensionReport` already exist in `src/v3/std/dimensions.dag`); Evaluator constructs witnesses by evaluating `.dag` bodies through the lens framework. R1 doesn't produce substrate facts R2-Evaluator consumes directly.

## Autonomous dispatch authority

- Authors all T-Evaluator sub-briefs without Director (after PR-A through PR-E design lock).
- Dispatches workers against T-Evaluator sub-briefs.
- Resolves T-Evaluator-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every T-Evaluator worker brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; every PR introducing hand-Rust under `src/v3/` fills the per-PR gate.
- **Cross-program signal authority:** Evaluator-readiness signal goes to Director (gates R3 spin-up); per-sub-lane closure goes to R2 Release Manager (closure ledger).

## Reporting cadence

- **Lane-close → R2 Release Manager** (closure ledger). Each sub-lane's structural acceptance gate (e.g., `evaluator_runtime_value_model_landed`, `body_evaluator_executes_std_dag_correctly`) IS the demo per the structural-acceptance-per-lane-close discipline.
- **Cross-program signals** (e.g., `Lens<C>` runtime ready for T-CostLens-Composition consumer) → cross-manager queue.
- **Evaluator-readiness signal** (R3 spin-up gate) → Director.
- **Blockers + scope changes** → Director.
- **Weekly health surfacing to Director:** which sub-lanes within 1 step of unblocking, which workers fill vs. ready, which PR-A through PR-E design locks are landed vs pending.

## Acceptance — `.dag` gates

Each sub-lane closes under a structural acceptance gate authored as a `.dag` `TestClaim`:

- `evaluator_runtime_value_model_landed` — runtime value type declared in substrate; closed-over environment representation correctly implemented
- `evaluator_body_evaluator_correctly_executes_std_termination` — Body evaluator correctly executes `dsl/std/termination.dag` body programs (representative test)
- `evaluator_lens_application_complete_reflection` — `reflect_program_dag_nodes_in_file` returns complete reflection (no shallow/lossy gaps) per [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) §"Decision" (5.1-5.3 sub-questions resolved)
- `evaluator_witness_construction_per_lens_correct` — runtime witness materialization correct for at least 3 lens instances (complexity / tenant-flow / IFC per design-lens-framework.md)
- `evaluator_cross_target_equivalence_harness_primitives_landed` — primitives ready for R3-T-V-L5-Corpus consumer (no corpus authoring at R2; primitives only)

**Plus:** `lens_complexity_n_log_n_fold_correct` + `lens_tenant_flow_aggregate_validate_fail_closed` + `lens_ifc_aggregate_validate_fail_closed` (TestClaims from `docs/design-lens-framework.md` I4 + I9).

## Sub-briefs (authored / pending)

**Authored:** none yet. Brief is the parent skeleton.

**Pending (post-spawn manager authors autonomously):**
- PR-A worker brief — Runtime value model design lock
- PR-B worker brief — Witness construction surface design lock
- ~~PR-C worker brief — Reflection completeness spec~~ — **LANDED via #1129** at [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md)
- PR-D worker brief — L5 corpus type spec
- PR-E worker brief — Final integration design synthesis
- Implementation worker briefs (one per sub-lane: runtime value model implementation, body evaluator implementation, lens application implementation, witness construction implementation, cross-target equivalence primitives)

## Working state (fill on spawn)

Sub-lane status table refreshes here as work lands. Pre-spawn placeholder. Initial state will be PR-A through PR-E design-lock cadence in flight.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Evaluator Manager (added 2026-04-28 amendment)" (lane row in §"Manager structure" + §"Lane structure" table)
- R3 dependent lanes: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" — 7 of 10 lanes Evaluator-gated
- Lens framework consumer: [`docs/design-lens-framework.md`](../design-lens-framework.md) — `Lens<C>` primitive that Evaluator implements `fold_lens<C>` for
- 8 design questions disposition: [`docs/r2-structure.md`](../r2-structure.md) §4 + [`docs/design-emission-model.md`](../design-emission-model.md) Q1-Q5 + [`docs/design-lens-framework.md`](../design-lens-framework.md) Q6-Q8
- Pre-dispatch design lock cadence: [`docs/r3-structure.md`](../r3-structure.md) §"Pre-R2-Evaluator design lock cadence"
- INVARIANTS substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1
