# R3 Verification Manager Brief

**Status:** PROPOSAL — manager brief authored at R3 spin-up (post-R2-close 2026-04-30 per [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §"Director closure acceptance"). Spawned per [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 2 (Director-locked 2026-04-28).

## Orient before reading

- **R3 structure authority:** [`docs/r3-structure.md`](../r3-structure.md). Names this manager owner of T-Verification-L4-L7-Direct + T-Verification-L5-Corpus + T-Free-Consequences-Demonstration + the `bridge_retirement_ledger_zero` audit gate of T-Bridge-Retirement.
- **Program scope source:** [`THESIS.md`](../../THESIS.md) §"Tier 3 — Verification from structure" (L4, L5, L7 verification-surface claims) + [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" rows for T-Verification-L4-L7-Direct / T-Verification-L5-Corpus / T-Free-Consequences-Demonstration / T-Bridge-Retirement.
- **Why a new manager (per `r3-structure.md` §"Manager structure" Item 2):** the R3 verification surface {L4, L5, L7} + free-consequences-demonstration is structural-acceptance-by-construction — its own discipline, not foldable into Substrate (different concern) or PB (different concern).
- **Cross-program producer:** **R2-Evaluator** gates lanes 1, 2, and 3 (Witness construction surface + cross-target equivalence harness primitives + consequence witnesses). R3-absorbed formal-grounding lane (TC1/TC2/TC3 bundling) consumes substrate primitives authored by Substrate Manager continuation. **PR-D semantic lock:** [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md) defines the L5 equality / corpus / oracle / float / effect policy this manager consumes.
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1): self-serve through the 3-step decision procedure before escalating substrate-shape questions to Director. Director ratified unified substrate-introduction for TC1/TC2/TC3 as `BinaryDimensionReportEquals` at [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427) + [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359); Substrate owns the predicate variant, while Verification supplies consuming coverage requirements.

## Owned program scope (3 lanes + 1 ledger gate, per `r3-structure.md` §"Manager structure" Item 2 authority)

| Item | Size | Status (at brief authoring) | Gates on |
|---|---|---|---|
| **Lane 1: T-V-L4-L7-Direct** | M | **Worker brief authored, standby** — [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md). Per-target equivalence harness using `DifferentialEquals` predicate (consumes Worker B PR-D scaffold per [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) §slice 1). NOT a `Lens<C>` instance per codex BLOCKING `f5f63c7d9` — runtime equivalence check, not structural fold. | R2-Evaluator PR-A.3 implementation carriers + PR-B body evaluator landing |
| **Lane 2: T-V-L5-Corpus** | M | **Worker brief authored, standby** — [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md). Cross-target equivalence corpus authoring (L5 only; L6 reclassified to R2-T-Ground-CrossTarget-Meta per [`r3-structure.md`](../r3-structure.md) §"Acceptance" T-Verification-L5-Corpus L6-reclassification note). Consumes PR-D semantic policy in [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md). | Lane 1 corpus existing + R2-Grounding-Rust + R2-Grounding-Python + R2-Grounding-Go (Shape A 3-target grounding precondition) |
| **Lane 3: T-Free-Consequences-Demonstration** | S-M | **Worker brief authored, standby** — [`r3-v-free-consequences-worker.md`](r3-v-free-consequences-worker.md). Small doc + testcase-driven demonstration of what guarantees the compiler actually provides: auto-parallelism (including effect/commutativity safety), auto-memoization, cross-target optimization, and space-bound CX status/reference (space-bound proofs remain NOT STARTED until the space lens is modeled). | R2-Evaluator witness construction + R2-T-Substrate-Lens-Primitive (`Lens<C>` shape) + T-CostLens-Composition |
| **Ledger gate: T-Bridge-Retirement (`bridge_retirement_ledger_zero`)** | S (audit cadence; no implementation) | **Bridge map row maintenance** — 5 named bridges per [`r3-structure.md`](../r3-structure.md) §"Lane structure" T-Bridge-Retirement row's distribution map. Verification owns the unified audit gate; retirement work distributes per natural-owner program. | Per-bridge: each bridge fires structurally in its owner program; ledger-zero gate fires when all 5 are green. |

### Absorbed cross-cutting responsibility — TC1/TC2/TC3 bundle (NOT a fourth owned lane)

Per [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L181 "TC3 ownership moves from PB to Verification" + R2-Evaluator residual transition for TC2 + #1179 ratification for TC1: the three formal-grounding `TestClaim`s are an **absorbed cross-cutting responsibility** of this manager, not a fourth owned lane. The structural authority `r3-structure.md` §"Manager structure" Item 2 names exactly **3 lanes + 1 ledger gate** for Verification scope after the 2026-04-30 expansion; this brief defers to that authority. Audit cadence + strict-fire activation tracking is folded into manager cadence (not a separate dispatch program). Worker brief at [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) authors the bundle as an absorbed-responsibility audit-cadence artifact.

**Unified-predicate disposition (Director-ratified):** PR #1309's TC1 analysis named Option 2 first — generalize `LensOutputEquals` into binary structural equality over `DimensionReport<C>`. PR #1316 independently converged on the same shape for TC2. Director ratified the unified substrate target at [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427) and [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359): one Substrate-owned `BinaryDimensionReportEquals` `TestPredicate` variant with reflection-aware modifiers absorbs TC1 eta-equivalence, TC2 strategy-order equality, and TC3 evaluation-step witnessing. Verification authors coverage requirements and consuming `TestClaim`s; Substrate authors the predicate variant / carrier. The 2026-04-30 lane expansion added T-Free-Consequences-Demonstration as Lane 3, but did not elevate the TC bundle into a lane.

## Bridge-retirement ledger — current state (2026-04-30 audit)

Per [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-Bridge-Retirement row's distribution map; cross-checked against closure ledger #1275 + landed PRs:

| # | Bridge | Owner | Status | Evidence |
|---|---|---|---|---|
| 1 | `SourceSpan.file` participation checks | **Substrate** | **R3-deferred** | #1273 STOP+PING audit landed; #1130 Director acceptance — partial string-check retirement rejected; structural prerequisites named (module/compilation-unit identity for lens reflection; typed authority/emit-scope carriers for lower/emit). Per [`r3-structure.md`](../r3-structure.md) §"Lane structure" T-Bridge-Retirement row item (1). |
| 2 | `mark_bootstrap_secret_nominal_opacity()` | **Substrate** | **retired** | #1272 (refactor(v3): retire Secret bootstrap opacity bridge). |
| 3 | Canonical lens-name dispatch | **PB** | **slice landed; ledger-zero pending** | #1183 — narrow-scope canonical lens-name dispatch slice. Broader exact-string lens-name patching not yet structurally retired. |
| 4 | `include_str!` side channels (e.g., `pipeline_authority.rs`) | **PB** | **outstanding-or-waiting** | #1171 suspended `reconcile_with_compile_body` rather than swapping `include_str!` for runtime file IO; **`bridge_include_str_side_channels_retired` still open** per [`design-emission-model.md`](../design-emission-model.md) §"Per Director directive 2026-04-28 (gpt-5-5-pro reflective analysis)" (`include_str!` retirement bullet). Awaits derivation or structural compile-body witness. |
| 5 | `patch_lower_helpers_*` residual | **PB** | **slice landed; narrow scope** | #1014 (first slice — generated field native) + #1192 (`bridge_lower_helpers_patch_zero_residual_test.rs` — narrow ratchet-zero). Broader exact-string patching outside this slice not claimed retired. |

**Net position:** 1 retired (#1272), 2 narrow slices landed (#1183 canonical lens / #1014 + #1192 lower-helper), 1 outstanding (#1171 suspended `reconcile_with_compile_body` only — `bridge_include_str_side_channels_retired` still open; tracked separately by closure-ledger update #1283), 1 R3-deferred (#1273). **Unified `bridge_retirement_ledger_zero` gate remains open** until all 5 fire structurally — row stays in-flight per closure ledger discipline.

## TestClaim author-now-fire-later state — audit (2026-04-30)

| TC | Fixture / authority | Strict-fire gate | Audit result |
|---|---|---|---|
| **TC1 — η-equivalence** | `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag` (#1179); `SubstrateResearchDeferredClaim` runner-valid only for this fixture per [`r2-closure-ledger.md`](../r2-closure-ledger.md) L220 | Unified `BinaryDimensionReportEquals` predicate with TC1 eta-equivalence modifier, plus T-Substrate-Lens-Primitive / lens producer prerequisites | **Consistent on main** — fixture exists; deferred-claim carrier authored per Director #1130 / dispatch #1139. Strict-fire path is now the unified predicate, not a TC1-specific predicate. |
| **TC2 — Church-Rosser / evaluation-order independence** | `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag` — `BinaryDimensionReportEquals` consumer with strategy-order role pairing (per [`r2-evaluator-manager.md`](r2-evaluator-manager.md) L127) | Runner strict-fire still needs **≥2 executable strategies** + `DimensionReport<C>` production (PR-B.1 lands one eager strategy; second strategy remains R3 residual) | **Consistent on main** — unified predicate consumer landed; evaluation remains NYI until report production lands. |
| **TC3 — strong normalization** | Stage-(a) fixture `src/v3/compiler/tests/fixtures/tc3_strong_normalization_deferred.dag`; declarative theorem text in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L145-215 | Unified `BinaryDimensionReportEquals` with TC3 evaluation-step role pairing; full strict-fire awaits T-FixedPoint (stage b) per bundle two-stage gate | **Consistent** — stage-(a) fixture on disk; PB does not re-author declarative shape post-transition L181-185. |

**No drift surfaced to Director.** All three claims maintain author-now-fire-later discipline; structural unblock conditions are category-tagged per dispatch contract.

## Cross-program dependencies

**Produces:**
- **L4-L7 verification surface** — Lane 1 lands per-target equivalence; Lane 2 lands cross-target equivalence corpus.
- **Free-consequences demonstration surface** — Lane 3 lands the design-free-consequences doc + 10-gate TestClaim suite for user-visible guarantees.
- **TC1/TC2/TC3 strict-fire activations** — absorbed-responsibility audit cadence strengthens deferred claims as the unified `BinaryDimensionReportEquals` substrate predicate and each TC's modifier/prerequisites land.
- **Unified bridge-retirement audit cadence** — periodic ledger-zero gate check; signals to Director when all 5 bridges fire.

**Consumes:**
- **R2-Evaluator** — PR-A.3 carriers (closed strategy + memoization), PR-B body evaluator (eager baseline), PR-D harness primitives (`DifferentialEquals` runner wiring), PR-E lens application (`fold_lens_over_reflected_program` integration seam).
- **R2-Grounding** — Shape A 3-target grounding (Rust + Python + Go) for L5 cross-target receipts.
- **PB Manager continuation** — T-FixedPoint completion (TC3 dependency); T-LensProducer-Retirement (consumed indirectly via TC1 substrate-research strengthening); 3 PB-side bridge slices landing toward ledger-zero.
- **Substrate Manager continuation** — T-Substrate-Lens-Primitive (TC1/TC2 strengthening; TC3 evaluation-step witness shape per [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L165; Lane 3 `Lens<C>` prerequisite); T-CostLens-Composition (Lane 3 cost-related claims); SourceSpan.file participation retirement (Bridge #1). **T-LBP option (b) + LAS #95:** carve scope (**#81**/**#82**/**#95**) is **canonical** in `r3-structure.md` §"Acceptance", `docs/r4-carve-out-routing.md`, and `docs/design-lens-application-surface.md` §**7** — Verification worker briefs **defer** (**INVARIANTS** §P2).

## Autonomous dispatch authority

- Authors all Verification sub-briefs without Director (per `feedback_standing_managers_need_owned_deliverables.md` discipline).
- Dispatches workers against Verification sub-briefs once cross-program prerequisites land.
- Resolves Verification-internal scope refinements; escalates substrate-shape questions / cross-program scope-changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline (carried into R3): every Verification worker brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance.
- **Unified predicate coverage proposal**: when TC1/TC2/TC3 coverage inputs mature, author the Verification-side requirements for `BinaryDimensionReportEquals`; Substrate owns the predicate variant / carrier through INVARIANTS §P1.

## Reporting cadence

- **Lane-close → R2 Release Manager continuation** (closure ledger maintenance via bold-lynx-173 #1135). Each lane's structural acceptance gate IS the demo per the structural-acceptance-per-lane-close discipline.
- **Cross-program signals** (e.g., bridge ledger-zero audit results) → cross-manager queue + Director.
- **TC strict-fire activation signals** → Director (gates R3 verification surface closure); unified-predicate coverage requirements route to Substrate when mature.
- **Blockers + scope changes** → Director (#828).
- **Brief-PR cadence** (per `feedback_brief_pr_cadence.md`): brief PRs only when carrying a new cross-manager signal; pure checkbox maintenance bundles into next signal PR or end-of-session sweep.

## Acceptance — `.dag` gates

Each lane closes under a structural acceptance gate authored as a `.dag` `TestClaim`:

- **Lane 1**: closes under both `l4_emit_eval_match` (per [`r3-structure.md`](../r3-structure.md) §"Acceptance" T-Verification-L4-L7-Direct gate definition — every `.dag` program in certification corpus has emit-target output equal to `.dag` eval output, algebraic equality) AND `l7_algebraic_laws_witnessed` (per [`r3-structure.md`](../r3-structure.md) §"Acceptance" T-Verification-L4-L7-Direct gate definition — every algebra × every applicable law has a runtime-constructed witness via `AlgebraicLaw` `TestPredicate`). Partial-coverage early slices do NOT close the lane; full coverage required.
- **Lane 2**: `l5_cross_target_consistency` (per [`r3-structure.md`](../r3-structure.md) §"Acceptance" T-Verification-L5-Corpus gate definition) — for every `.dag` program, emitted Rust/Python/Go produce equivalent runtime behavior on the certification corpus (algebraic equivalence over computational results, not byte identity).
- **Lane 3**: closes when [`docs/design-free-consequences.md`](../design-free-consequences.md) lands and the 10-gate suite from [`r3-v-free-consequences-worker.md`](r3-v-free-consequences-worker.md) is green: `auto_parallelism_independent_binds_emit_parallel`, `auto_parallelism_dependent_binds_emit_sequential`, `auto_parallelism_branch_arms_serialize`, `auto_loop_parallelism_provable_independence_emits_parallel`, `auto_loop_parallelism_unproven_falls_back_sequential`, `auto_loop_parallelism_dependence_emits_sequential`, `auto_memoization_repeated_pure_call_cached`, `auto_memoization_no_caching_for_one_shot`, `cross_target_optimization_constant_fold_consistent`, and `cross_target_optimization_cost_structurally_derived`.
- **Absorbed responsibility (TC bundle)**: TC1/TC2/TC3 strict-fire activation across the three deferred-claim fixtures via unified `BinaryDimensionReportEquals` once Substrate lands the predicate and each reflection-aware modifier is covered — tracked via audit cadence, not as a lane-close gate.
- **Ledger gate**: `bridge_retirement_ledger_zero` — unified ledger reports 0 named identity bridges remaining (per [`r3-structure.md`](../r3-structure.md) §"Acceptance" T-Bridge-Retirement gate definition).

## Sub-briefs (authored / pending)

**Authored / maintained:**
- [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md) — Lane 1 standby brief.
- [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md) — Lane 2 standby brief.
- [`r3-v-free-consequences-worker.md`](r3-v-free-consequences-worker.md) — Lane 3 standby brief.
- [`r3-v-pattern-a-tc1-v1-worker.md`](r3-v-pattern-a-tc1-v1-worker.md) — Pattern-A V1 (TC1 first executable slice; Q-PAFS Path A **ACCEPTED** 2026-05-06).
- [`r3-v-pattern-a-tc2-v1-worker.md`](r3-v-pattern-a-tc2-v1-worker.md) — Pattern-A TC2 (`tc2_church_rosser_executable`) dispatch-ready worker brief (**PRE-AUTH**; strategy-order / Church-Rosser slice).
- [`r3-v-pattern-a-tc3-v1-worker.md`](r3-v-pattern-a-tc3-v1-worker.md) — Pattern-A TC3 (`tc3_pattern_a_second_mover_executable`) dispatch-ready worker brief (**PRE-AUTH**; evaluation-step / second-mover; two-stage bundle).
- [`r3-v-pattern-a-rust-dag-isomorphism-v1-worker.md`](r3-v-pattern-a-rust-dag-isomorphism-v1-worker.md) — Pattern-A gate **#14** `rust_dag_isomorphism_executable` (**PRE-AUTH**; Dag-iso / shape-report consumer shell).
- [`r3-v-tests-as-data-v1-worker.md`](r3-v-tests-as-data-v1-worker.md) — T-Tests-As-Data-Completeness unified dispatch overlay (**PRE-AUTH**; facet-3 + quantifiers + cementing alignment).
- [`r3-v-t-lbp-narrowed-scope-partner-worker.md`](r3-v-t-lbp-narrowed-scope-partner-worker.md) — T-LBP Verification partner (**PRE-AUTH**; option **(b)** complexity+cost in R3; cementing + register **C3** receipts).
- [`r3-v-t-lens-application-surface-execution-split-worker.md`](r3-v-t-lens-application-surface-execution-split-worker.md) — T-LAS execution split (**PRE-AUTH**; substrate **88–91** + demos **92–94** for **R3**; **#95** **R4 C1** per structure/plan carve).
- [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) — absorbed-responsibility TC1/TC2/TC3 bundle (NOT a lane; audit-cadence artifact per `r3-structure.md` §"Manager structure" Item 2 3-lane authority).

**Pending (post-spawn manager authors autonomously):**
- Unified `BinaryDimensionReportEquals` coverage-requirements proposal (TC1/TC2/TC3 inputs; Substrate authors predicate variant when proposal is mature).
- Lane 1 / Lane 2 / Lane 3 implementation worker briefs (gated on R2-Evaluator / substrate prerequisites — convert from standby to dispatch-ready when prerequisites fire).

## Working state (fill on dispatch)

Lane status table refreshes here as work lands. Initial state: 3 lanes in standby + 1 ledger gate in audit cadence + TC bundle absorbed-responsibility audit cadence; bridge map row maintenance ongoing.

## Cross-refs

- Parent: [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 2 + §"Lane structure" rows for T-Verification-L4-L7-Direct / T-Verification-L5-Corpus / T-Free-Consequences-Demonstration / T-Bridge-Retirement
- Closure ledger predecessor: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) (R2 closed-with-residuals 2026-04-30)
- R2 Evaluator producer brief: [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md)
- TC3 upstream declarative shape: [`docs/briefs/r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3 — Strong-normalization TestClaim" (L145-215; ownership transitions to Verification per L181-185)
- Unified substrate-introduction ratification: [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427), [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359)
- TC1 deeper analysis: [PR #1309](https://github.com/gunb-ai/gunbc/pull/1309)
- TC2 independent coverage analysis: [PR #1316](https://github.com/gunb-ai/gunbc/pull/1316)
- Worker B PR-D scaffold consumer: [`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md)
- Bridge distribution map: [`docs/r3-structure.md`](../r3-structure.md) L98
- INVARIANTS substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1
