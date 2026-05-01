# R2 Evaluator Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), Goal 7 added 2026-04-28 via PR #1078). Spawns post-#1078-merge per Transition mechanics step 4. **No prior brief to migrate** — this is a genuinely new R2 manager.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of **7** standing R2 managers (alongside Substrate, Modeling, Grounding, Impossible-Bugs, Pure Bootstrap, R2 Release; cross-program coordination via Director). Manager count rose from 6 to 7 with this lane added per #1078.
- **Program scope source:** [`THESIS.md`](../../THESIS.md) §"Tier 3 — Verification from structure" (L4-L7 verification surface) + [`docs/r2-structure.md`](../r2-structure.md) §"Evaluator Manager (added 2026-04-28 amendment)".
- **Cross-program consumer:** **R2-Evaluator gates 7 of 10 R3 lanes** (T-Tier3-Dissolution, T-LensProducer-Retirement, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition). The Evaluator IS the runtime that R3's consequence layer falls out from. Without it, R3 dispatchers spin.
- **Demo coordination:** signal lane-close to R2 Release Manager (closure ledger; per the structural-acceptance-per-lane-close discipline locked in `r2-structure.md` — the demo IS the structural gate, not a separate artifact).
- **Closure residuals (R2 Release ledger, docs-only):** [`r2-evaluator-closure-residuals.md`](r2-evaluator-closure-residuals.md) — PR-D / PR-E / TC2 landed vs deferred wording for ledger consumers; does not expand implementation scope.
- **PR-C dissolution gates (docs-only):** [`r2-pr-c-reflection-dissolution-gates.md`](r2-pr-c-reflection-dissolution-gates.md) — complete reflection is landed (#1129 spec + #1170 implementation); remaining surface is structural-gate consumption / R2.5-R3 dissolution guidance, not reopened reflection implementation.
- **PR-E / R3 dispatch brief:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) — composes PR-A through PR-D into bounded R3-Evaluator implementation slices and STOP+PING boundaries.
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
| **Runtime value model** | M | **PR-A design slice authored** — [`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md) locks the runtime-value / evaluator-state split and opens slice-0 gates. **Naming drift note:** Director dispatch also called this "PR-B runtime-value model"; this manager brief keeps the live PR-A label. **Cross-program convergence target:** PB-Runtime ([`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §2 LANDED via #1176) — load-bearing distinction: PB-Runtime ≡ R2-Evaluator's runtime model expressed as `.dag` (dissolution-shaped, not parallel). PR-A's `Value` coproduct shape MUST match §3.2; the 5-primitive constraint per §3.1 (`Node | Conj | Disj | Cardinality | Bit` — DAG-processor execution vocabulary, distinct from the 5 L1 `Behavior` variants `Value | Transform | Branch | Loop | Bind` dispatched inside `Node`) constrains PR-A's design space. Closed-over environments are **evaluator-internal** `EvalFrame` / `EvalStateStack`, not observable `Value` variants. | Closed-over environments, lazy/eager evaluation strategy, memoization. Per #1078 design challenge #1: locked direction; implementation follows PR-A. |
| **Body evaluator** | L | NOT YET AUTHORED — gated on Runtime value model | Execute `.dag` function bodies structurally. Bounded forward execution per INVARIANTS P4. Termination by descent evidence (already in substrate per `dsl/std/termination.dag`). |
| **Lens application** | M | OPEN — complete reflection **landed** (#1170); **PR-E slice 1 landed** — `fold_lens_over_reflected_program` wires reflect → prepend carrier → `apply_lens_declaration` on one `lens_program` ([`docs/briefs/r2-pr-e-lens-application-over-reflected-program-dag.md`](r2-pr-e-lens-application-over-reflected-program-dag.md)) | `reflect_program_dag_nodes_in_file` is structurally complete per [design-reflection completeness](../design-reflection-completeness.md) §"Decision". **Lens application** sub-lane stays **OPEN** for full `Lens<C>` / `DimensionReport` / PB-Runtime fold semantics over that spine; slice 1 is the bounded **reflect+apply** seam only (deeper fold: **Evaluator / PR-E lane** per PR-E brief; **Worker B** for landed reflect+apply slices in `lens_apply.rs`; **Worker A** stays on PR-A runtime carriers only, not PR-E `lens_apply` ownership). |
| **Witness construction** | M | NOT YET AUTHORED | Runtime materialization of proof artifacts (`Witness::Inhabits` / `Witness::Violates` per `src/v3/std/dimensions.dag`); algebraic-law witnesses (associativity, commutativity, identity). |
| **Cross-target equivalence harness primitives** | S | **PR-D slice 0 + slice 1 landed** — named `.dag` claims + suite ([`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md)); **PR-D design lock introduced** at [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md) for semantic equality / corpus / oracle / float / effect policy. Slice 2 (`ForAllTargets`-class emit-scoped receipts) **remains gated** on LanguageSpec + Shape A grounding + L4/L7 corpus deps in that brief. **Cadence matrix:** [`docs/briefs/r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md). | For L5 verification in R3 (algebraic equivalence over a curated corpus, per #1078 design challenge #3 locked decision). Primitives only — corpus authoring is post-R2 (R3 lane T-Verification-L5-Corpus). |

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
| **PR-A** | (foundational) | — | Runtime value model — closed-over environments, lazy/eager strategy, memoization | **PR-A.0 DESIGN SLICE AUTHORED** — [`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md); slice-0 `TestClaim` fixtures introduced by PR-A.0 at `r2_evaluator_runtime_value_model.dag` + `tc2_evaluation_order_independence_deferred.dag`. **PR-A.1 VALUE CARRIER AUTHORED** — `src/v3/std/runtime.dag` declares bare runtime `Value` + `NamedField` after #1231 freed the flat name by renaming the L1 behavior marker to `ValueBehavior`; [`r2-pr-a1-runtime-value-dependency-audit.md`](r2-pr-a1-runtime-value-dependency-audit.md) records the resolved dependency. **PR-A.2 LANDED** — `EvalFrame` / `EvalStateStack` live in `src/v3/std/runtime.dag` ([PR #1255](https://github.com/gunb-ai/gunbc/pull/1255)); **Worker C / merry-heron-351**; merged dependency audit [PR #1222](https://github.com/gunb-ai/gunbc/pull/1222). **PR-A.3 STRATEGY/MEMO AUDIT AUTHORED; PARSER PREREQUISITE RESOLVED** — [`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) locks eager baseline, optional thunk boundary, closed strategy carrier, and structural memo-key identity; [`r2-pr-a3-implementation-blocker-audit.md`](r2-pr-a3-implementation-blocker-audit.md) remains the historical parser-gap receipt after #1286; Substrate owns carrier declarations, and [`r2-pr-a3-follow-on-test-surface.md`](r2-pr-a3-follow-on-test-surface.md) records Worker A's post-carrier test scope. |
| **PR-B** | PR-A | PR-C, PR-D | Body evaluator + witness construction surface — runtime materialization | **Body half: PR-B.0 design slice authored** — [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) locks the deterministic eager-baseline scope over bounded `.dag` bodies and names the R3 residual boundary (full execution gated on PR-A.3 implementation carriers + lazy/TC2/witness slices). **PR-B.1 implementation seed authored** — [`r2-pr-b-1-eager-evaluator-implementation-seed.md`](r2-pr-b-1-eager-evaluator-implementation-seed.md) extends PR-B.0 with a per-`Behavior`-variant checklist, frame push/pop discipline, `Map<PortId, Value>` lookup/update rules, and a single fail-closed catalog with dissolution triggers; hard-gated on the Substrate-owned PR-A.3 carriers landing, with Worker A owning follow-on carrier test coverage. **PR-B.2 / PR-B.3 / PR-B.4 docs scoping bundle authored** — [`r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md) is a single docs-only scoping bundle that carves three workstreams for separate implementation slices (provisional naming: W1/PR-B.2 `DifferentialEquals` lineage producers `rust_emit_output` + `dag_eval_output` for L4; W2/PR-B.3 `AlgebraicLaw` runner extension `Commutativity` + `Identity`, with `Distributivity` explicitly routed to P1 substrate-fact-introduction; W3/PR-B.4 `ForAllTargets` per-target structural value-domain observation for L5). All three remain fully Evaluator-owned. Per-workstream gates differ: **W1** gated on PR-B.1 + PR-A.3 for `dag_eval_output`. **W2** — `Commutativity` can proceed runner-side now; `Identity` waits on the lens identity-element edge; `Distributivity` routes to P1 substrate enum/fact introduction (none gated globally on PR-B.1 / PR-A.3). **W3** waits on the structural observation P1 carrier and per-target producer availability (not unlocked merely by PR-B.1). **Witness half: not yet authored** — separate worker brief once R3 residual is closed or earlier promotion is decided. |
| **PR-C** | (foundational; substrate-reflection-shape) | PR-A, PR-B, PR-D | Reflection completeness spec — what does "complete reflection" mean for `reflect_program_dag_nodes_in_file`? | **LANDED via #1129** at [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md); complete reflection implementation **landed via #1170**. Dissolution / structural-gate consumption is tracked in [`r2-pr-c-reflection-dissolution-gates.md`](r2-pr-c-reflection-dissolution-gates.md). |
| **PR-D** | (foundational; cross-target spec) | PR-A, PR-B, PR-C | L5 corpus type spec — algebraic equivalence over curated corpus (locked direction; specific design here) | **Design lock introduced** — [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md) defines semantic equality, corpus curation, oracle validity, float policy, and side-effect normalization for R3 L5 consumers. **Slice 0 + slice 1 landed** — worker brief [`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) + structural hooks; strict cross-target / `ForAllTargets` receipts follow LanguageSpec / grounding deps in that brief. **Evaluator cadence:** [`r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md). |
| **PR-E** | All of PR-A through PR-D | (synthesis; serializes after) | Final integration design — synthesizes PR-A through PR-D into the implementation roadmap | **AUTHORED** — [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) composes PR-A through PR-D into R3 implementation slices (Value, frames/bind, Transform, Branch, Loop, lens fold, witness construction, runner extensions, cross-target harness consumption) with per-slice STOP+PING boundaries. |

**Sequencing principle (per Director directive 2026-04-28):** "focus on dependencies — writing code is fast; we get stuck in review." Parallelize PR-A, PR-C, PR-D as independent foundational design locks; PR-B serializes after PR-A (witness construction uses runtime values); PR-E synthesizes after all four. Worker dispatch on implementation sub-lanes blocks on PR-E.

Plus **LanguageSpec parallel** (R2-T-Ground-LanguageSpec sub-lane) — Grounding Manager authors the LanguageSpec schema in parallel; Evaluator Manager consumes it for cross-target equivalence work. Independent of PR-A through PR-E.

**Timing — option (c) hybrid (Director-locked 2026-04-28 via dialogue; satisfied for PR-E authoring after PR-A through PR-D landed):**

- **PR-A through PR-D** = design-only (no worker dispatch). **Dispatch immediately post-#1078-merge** — these don't conflict with R1 closure work because no workers are running on them yet. Director writes design docs; the structural-discipline handshake lives where workers dispatch, not here.
- **PR-E (Final integration + worker dispatch brief)** = the load-bearing handshake. **PR-E is now authored** at [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) after PR-A through PR-D landed; implementation dispatch consumes that brief's slice boundaries and STOP+PING rules.
- **Worker dispatch on implementation sub-lanes** = consumes PR-E + the already-landed design locks jointly.

Rationale: artificially delaying PR-A through PR-D wouldn't preserve any real handshake invariant — those are design docs. The structural discipline (R1→R2 transition mechanics, manager spawn ordering, dispatch-discipline) fires at worker-dispatch time, which is exactly where PR-E + the joint wait gates it.

## Cross-program dependencies

**Produces:**
- **R3 lane gates** — 7 of 10 R3 lanes block on R2-Evaluator landing. The Evaluator's `Witness` runtime (per `src/v3/std/dimensions.dag`) is what R3-T-Tier3-Dissolution / R3-T-LensProducer-Retirement / R3-T-V-L4-L7-Direct / etc. consume.
- **`Lens<C>` runtime** — R2-T-Substrate-Lens-Primitive's generic `fold_lens<C>: Lens<C> → Dag → DimensionReport<C>` is implemented by the Evaluator. T-CostLens-Composition (R3, under Substrate continuation per #1078 lock) consumes this.
- **Runtime value model that PB-Runtime mirrors** — per [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §2 (LANDED via #1176): PB-Runtime ≡ R2-Evaluator's runtime model expressed as `.dag`. Evaluator's PR-A Value coproduct shape IS what PB-Runtime mirrors. Convergence is dissolution-shaped (not parallel runtimes); PB Manager's R3 T-LensProducer-Retirement consumes Evaluator's runtime-value design directly via the §3.2 `Value` shape match.

**Consumes:**
- **Substrate Manager** — additional carriers needed by runtime values (e.g., closed-over environment representation). Design-pass at lane spin-up identifies the dependency.
- **Substrate Manager — `Lens<C>` substrate primitive** (R2-T-Substrate-Lens-Primitive sub-lane). Evaluator implements `fold_lens<C>`; substrate declares the type.
- **PB Manager — PB-Runtime convergence path** (cross-program coordination per [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §5.4). **PB Manager owns** per-shim `BinShim` instance declarations + bin-shim emit pattern + retirement dispatch. **Substrate Manager owns** the `BinShim` carrier-type shape itself (generalized evolution escalates via §P1). **Evaluator owns** runtime-value model. Cross-coordination at PR-A authoring time so the Value shape lock matches PB-Runtime's mirror requirement.

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

- `evaluator_runtime_value_model_landed` — runtime value type declared in substrate; closed-over environment representation correctly implemented. **PR-A.1 status:** `src/v3/std/runtime.dag` declares the observable `Value` / `NamedField` carrier; `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs` validates the PB-Runtime section 3.2 coproduct shape with `runtime_value_carrier_*` tests. **PR-A.2 status:** `EvalFrame` / `EvalStateStack` live in `src/v3/std/runtime.dag` ([PR #1255](https://github.com/gunb-ai/gunbc/pull/1255)); merged dependency audit [PR #1222](https://github.com/gunb-ai/gunbc/pull/1222).
- `evaluation_order_independent_lens_results` — TC2 Church-Rosser / evaluation-order independence claim. **Slice-0 hook:** `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`; gated on PB-Runtime spec landing + T-Substrate-Lens-Primitive landing, then strengthens to strict strategy-output equality over `DimensionReport<C>`.
- `evaluator_body_evaluator_correctly_executes_std_termination` — Body evaluator correctly executes `dsl/std/termination.dag` body programs (representative test)
- `evaluator_lens_application_complete_reflection` — `reflect_program_dag_nodes_in_file` returns complete reflection (no shallow/lossy gaps) per [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) §"Decision" (5.1-5.3 sub-questions resolved). **Structural `.dag` claim hook (named target):** add `data evaluator_lens_application_complete_reflection: TestClaim = …` to `src/v3/compiler/tests/fixtures/r2_evaluator_lens_application.template.dag` when R2 gate splices land (see [`docs/briefs/r2-pr-e-lens-application-over-reflected-program-dag.md`](r2-pr-e-lens-application-over-reflected-program-dag.md) §Acceptance hook).
- `evaluator_witness_construction_per_lens_correct` — runtime witness materialization correct for at least 3 lens instances (complexity / tenant-flow / IFC per design-lens-framework.md)
- `evaluator_cross_target_equivalence_harness_primitives_landed` — primitives ready for R3-T-V-L5-Corpus consumer (no corpus authoring at R2; primitives only). **Design lock:** [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md) defines the semantic equality / oracle / float / effect policy that R3 consumes. **Structural `.dag` claim hook (named target):** `data evaluator_cross_target_equivalence_harness_primitives_landed: TestClaim = …` in [`src/v3/compiler/tests/fixtures/r2_evaluator_cross_target_equivalence_harness_primitives.dag`](../../src/v3/compiler/tests/fixtures/r2_evaluator_cross_target_equivalence_harness_primitives.dag) (suite `r2_evaluator_cross_target_equivalence_harness_primitives_suite`; slice 0 predicate `Compiles` — see [`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) §Next implementation slices).

**Plus:** `lens_complexity_n_log_n_fold_correct` + `lens_tenant_flow_aggregate_validate_fail_closed` + `lens_ifc_aggregate_validate_fail_closed` (TestClaims from `docs/design-lens-framework.md` I4 + I9).

## Sub-briefs (authored / pending)

**Authored:**
- [`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md) — runtime `Value` model / `EvalFrame` / `EvalStateStack` / lazy thunk + memoization boundary design lock.
- [`r2-pr-a1-runtime-value-dependency-audit.md`](r2-pr-a1-runtime-value-dependency-audit.md) — PR-A.1 blocker audit, resolved by #1231's `ValueBehavior` marker rename.
- [`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) — PR-A.3 strategy / memoization decision audit introduced by this slice; PR-A.2 `EvalFrame` / `EvalStateStack` carriers **landed** in `src/v3/std/runtime.dag` ([PR #1255](https://github.com/gunb-ai/gunbc/pull/1255)); parser prerequisite **resolved** by [PR #1286](https://github.com/gunb-ai/gunbc/pull/1286), with strategy/memo carrier declarations now owned by Substrate.
- [`r2-pr-a3-implementation-blocker-audit.md`](r2-pr-a3-implementation-blocker-audit.md) — PR-A.3 historical parser-gap receipt; still records the no-fake-variant discipline after #1286 resolved the syntax prerequisite.
- [`r2-pr-a3-follow-on-test-surface.md`](r2-pr-a3-follow-on-test-surface.md) — Worker A follow-on test plan for PR-A.3 carriers after Substrate lands the runtime declarations; no carrier-authoring scope.
- [`r2-pr-c-reflection-dissolution-gates.md`](r2-pr-c-reflection-dissolution-gates.md) — PR-C structural-gate consumption brief: #1129 spec + #1170 implementation are landed; dissolution waits on `.dag` acceptance wiring plus PR-B / PR-E consumers.

**Pending (post-spawn manager authors autonomously):**
- ~~PR-A worker brief — Runtime value model design lock~~ — **slice authored** at [`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md); implementation worker remains pending.
- PR-B worker brief — Witness construction surface design lock
- ~~PR-C worker brief — Reflection completeness spec~~ — **LANDED via #1129** at [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md); complete reflection implementation **landed via #1170**; structural-gate consumption tracked in [`r2-pr-c-reflection-dissolution-gates.md`](r2-pr-c-reflection-dissolution-gates.md).
- ~~PR-D worker brief — L5 corpus type spec~~ — **design lock introduced** at [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md); **slice 0 + slice 1 landed** in [`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md); **Evaluator cadence:** [`r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md) (strict L5 multi-target harness: follow §Next implementation slices in that brief)
- ~~PR-E worker brief — Final integration design synthesis~~ — **authored** at [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md); implementation workers consume its slice boundaries and STOP+PING rules.
- Implementation worker briefs (one per sub-lane: runtime value model implementation, body evaluator implementation, lens application implementation, witness construction implementation, cross-target equivalence primitives)
- **PR-E lens-application fold (slice 1 landed; deeper fold in flight)** — [`docs/briefs/r2-pr-e-lens-application-over-reflected-program-dag.md`](r2-pr-e-lens-application-over-reflected-program-dag.md); `fold_lens_over_reflected_program` in `src/v3/compiler/src/lens_apply.rs` runs **reflect → `apply_lens_declaration`** (single `lens_program` authority; reflected carrier as first lens arg). Deeper `Lens<C>` / `DimensionReport` / PB-Runtime integration in **Evaluator / PR-E lane** remains open per that brief (**Worker B** for landed slice-1 `lens_apply.rs`; **Worker A** owns PR-A runtime carriers, not PR-E fold ownership).

## Working state (fill on spawn)

Sub-lane status table refreshes here as work lands. Pre-spawn placeholder. Initial state will be PR-A through PR-E design-lock cadence in flight.

## Cross-refs

- **Evaluator cadence / convergence (Evaluator-side only):** [`docs/briefs/r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md) — lane-by-lane gate status, next allowed slices, forbidden widening; complements PB convergence audit [PR #1235](https://github.com/gunb-ai/gunbc/pull/1235) without replacing PB authority.
- **R3 Evaluator dispatch:** [`docs/briefs/r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) — PR-E implementation-slice authority after PR-A through PR-D land.
- **R2 Release closure residuals (Evaluator, docs-only):** [`r2-evaluator-closure-residuals.md`](r2-evaluator-closure-residuals.md) — PR-D / PR-E / TC2 landed vs deferred for ledger consumers.
- Parent: `docs/r2-structure.md` §"Evaluator Manager (added 2026-04-28 amendment)" (lane row in §"Manager structure" + §"Lane structure" table)
- R3 dependent lanes: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" — 7 of 10 lanes Evaluator-gated
- Lens framework consumer: [`docs/design-lens-framework.md`](../design-lens-framework.md) — `Lens<C>` primitive that Evaluator implements `fold_lens<C>` for
- 8 design questions disposition: [`docs/r2-structure.md`](../r2-structure.md) §4 + [`docs/design-emission-model.md`](../design-emission-model.md) Q1-Q5 + [`docs/design-lens-framework.md`](../design-lens-framework.md) Q6-Q8
- Pre-dispatch design lock cadence: [`docs/r3-structure.md`](../r3-structure.md) §"Pre-R2-Evaluator design lock cadence"
- INVARIANTS substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1
