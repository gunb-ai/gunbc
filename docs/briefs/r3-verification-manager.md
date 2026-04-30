# R3 Verification Manager Brief

**Status:** PROPOSAL — manager brief authored at R3 spin-up (post-R2-close 2026-04-30 per [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §"Director closure acceptance"). Spawned per [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 2 (Director-locked 2026-04-28).

## Orient before reading

- **R3 structure authority:** [`docs/r3-structure.md`](../r3-structure.md). Names this manager owner of T-Verification-L4-L7-Direct + T-Verification-L5-Corpus + the `bridge_retirement_ledger_zero` audit gate of T-Bridge-Retirement.
- **Program scope source:** [`THESIS.md`](../../THESIS.md) §"Tier 3 — Verification from structure" (L4, L5, L7 verification-surface claims) + [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" rows for T-Verification-L4-L7-Direct / T-Verification-L5-Corpus / T-Bridge-Retirement.
- **Why a new manager (per `r3-structure.md` L108):** the R3 verification surface {L4, L5, L7} is structural-acceptance-by-construction — its own discipline, not foldable into Substrate (different concern) or PB (different concern).
- **Cross-program producer:** **R2-Evaluator** gates lanes 1 and 2 (Witness construction surface + cross-target equivalence harness primitives). R3-absorbed formal-grounding lane (TC1/TC2/TC3 bundling) consumes substrate primitives authored by Substrate Manager continuation.
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1): self-serve through the 3-step decision procedure before escalating substrate-shape questions to Director. Director ratified unified substrate-introduction for TC1/TC2/TC3 as `BinaryDimensionReportEquals` at [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427) + [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359); Substrate owns the predicate variant, while Verification supplies consuming coverage requirements.

## Owned program scope (2 lanes + 1 ledger gate, per `r3-structure.md` L108 authority)

| Item | Size | Status (at brief authoring) | Gates on |
|---|---|---|---|
| **Lane 1: T-V-L4-L7-Direct** | M | **Worker brief authored, standby** — [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md). Per-target equivalence harness using `DifferentialEquals` predicate (consumes Worker B PR-D scaffold per [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) §slice 1). NOT a `Lens<C>` instance per codex BLOCKING `f5f63c7d9` — runtime equivalence check, not structural fold. | R2-Evaluator PR-A.3 implementation carriers + PR-B body evaluator landing |
| **Lane 2: T-V-L5-Corpus** | M | **Worker brief authored, standby** — [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md). Cross-target equivalence corpus authoring (L5 only; L6 reclassified to R2-T-Ground-CrossTarget-Meta per [`r3-structure.md`](../r3-structure.md) L92-93). | Lane 1 corpus existing + R2-Grounding-Rust + R2-Grounding-Python + R2-Grounding-Go (Shape A 3-target grounding precondition) |
| **Ledger gate: T-Bridge-Retirement (`bridge_retirement_ledger_zero`)** | S (audit cadence; no implementation) | **Bridge map row maintenance** — 5 named bridges per [`r3-structure.md`](../r3-structure.md) L98 distribution map. Verification owns the unified audit gate; retirement work distributes per natural-owner program. | Per-bridge: each bridge fires structurally in its owner program; ledger-zero gate fires when all 5 are green. |

### Absorbed cross-cutting responsibility — TC1/TC2/TC3 bundle (NOT a new lane)

Per [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L181 "TC3 ownership moves from PB to Verification" + R2-Evaluator residual transition for TC2 + #1179 ratification for TC1: the three formal-grounding `TestClaim`s are an **absorbed cross-cutting responsibility** of this manager, not a third owned lane. The structural authority `r3-structure.md` L108 names exactly **2 lanes + 1 ledger gate** for Verification scope; this brief defers to that authority. Audit cadence + strict-fire activation tracking is folded into manager cadence (not a separate dispatch program). Worker brief at [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) authors the bundle as an absorbed-responsibility audit-cadence artifact.

**Unified-predicate disposition (Director-ratified):** PR #1309's TC1 analysis named Option 2 first — generalize `LensOutputEquals` into binary structural equality over `DimensionReport<C>`. PR #1316 independently converged on the same shape for TC2. Director ratified the unified substrate target at [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427) and [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359): one Substrate-owned `BinaryDimensionReportEquals` `TestPredicate` variant with reflection-aware modifiers absorbs TC1 eta-equivalence, TC2 strategy-order equality, and TC3 evaluation-step witnessing. Verification authors coverage requirements and consuming `TestClaim`s; Substrate authors the predicate variant / carrier. If Director ratifies a third lane in `r3-structure.md` itself, this brief updates accordingly; until then, 2-lane scope is the authority.

## Bridge-retirement ledger — current state (2026-04-30 audit)

Per [`docs/r3-structure.md`](../r3-structure.md) L98 distribution map; cross-checked against closure ledger #1275 + landed PRs:

| # | Bridge | Owner | Status | Evidence |
|---|---|---|---|---|
| 1 | `SourceSpan.file` participation checks | **Substrate** | **R3-deferred** | #1273 STOP+PING audit landed; #1130 Director acceptance — partial string-check retirement rejected; structural prerequisites named (module/compilation-unit identity for lens reflection; typed authority/emit-scope carriers for lower/emit). Per [`r3-structure.md`](../r3-structure.md) L79. |
| 2 | `mark_bootstrap_secret_nominal_opacity()` | **Substrate** | **retired** | #1272 (refactor(v3): retire Secret bootstrap opacity bridge). |
| 3 | Canonical lens-name dispatch | **PB** | **slice landed; ledger-zero pending** | #1183 — narrow-scope canonical lens-name dispatch slice. Broader exact-string lens-name patching not yet structurally retired. |
| 4 | `include_str!` side channels (e.g., `pipeline_authority.rs`) | **PB** | **outstanding-or-waiting** | #1171 suspended `reconcile_with_compile_body` rather than swapping `include_str!` for runtime file IO; **`bridge_include_str_side_channels_retired` still open** per [`docs/design-emission-model.md:944`](../design-emission-model.md). Awaits derivation or structural compile-body witness. |
| 5 | `patch_lower_helpers_*` residual | **PB** | **slice landed; narrow scope** | #1014 (first slice — generated field native) + #1192 (`bridge_lower_helpers_patch_zero_residual_test.rs` — narrow ratchet-zero). Broader exact-string patching outside this slice not claimed retired. |

**Net position:** 1 retired (#1272), 2 narrow slices landed (#1183 canonical lens / #1014 + #1192 lower-helper), 1 outstanding (#1171 suspended `reconcile_with_compile_body` only — `bridge_include_str_side_channels_retired` still open; tracked separately by closure-ledger update #1283), 1 R3-deferred (#1273). **Unified `bridge_retirement_ledger_zero` gate remains open** until all 5 fire structurally — row stays in-flight per closure ledger discipline.

## TestClaim author-now-fire-later state — audit (2026-04-30)

| TC | Fixture / authority | Strict-fire gate | Audit result |
|---|---|---|---|
| **TC1 — η-equivalence** | `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag` (#1179); `SubstrateResearchDeferredClaim` runner-valid only for this fixture per [`r2-closure-ledger.md`](../r2-closure-ledger.md) L220 | T-Substrate-Lens-Primitive + lens producer retirement | **Consistent on main** — fixture exists; deferred-claim carrier authored per Director #1130 / dispatch #1139. |
| **TC2 — Church-Rosser / evaluation-order independence** | `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag` slice-0 hook (per [`r2-evaluator-manager.md`](r2-evaluator-manager.md) L127) | Strict claim activation needs **≥2 executable strategies** (R3-deferred from R2-Evaluator; PR-B.1 lands a single eager strategy per [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L154) | **Consistent on main** — slice-0 deferred fixture exists; strengthens to strict strategy-output equality over `DimensionReport<C>` once second strategy lands. |
| **TC3 — strong normalization** | **Text-form only** in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L145-215; **no fixture on disk** | Unified `BinaryDimensionReportEquals` predicate required with TC3 evaluation-step modifier; no TC3-specific `MetaTheoremOverTypedFragment` predicate after Director ratification | **Consistent — declaration-as-PROPOSAL preserved.** TC3 ownership transitions from PB to Verification per L181-185 contract. Verification will author TC3 coverage requirements for the unified predicate when B5 + T-Substrate-Lens-Primitive land; PB does not re-author. |

**No drift surfaced to Director.** All three claims maintain author-now-fire-later discipline; structural unblock conditions are category-tagged per dispatch contract.

## Cross-program dependencies

**Produces:**
- **L4-L7 verification surface** — Lane 1 lands per-target equivalence; Lane 2 lands cross-target equivalence corpus.
- **TC1/TC2/TC3 strict-fire activations** — absorbed-responsibility audit cadence strengthens deferred claims as the unified `BinaryDimensionReportEquals` substrate predicate and each TC's modifier/prerequisites land.
- **Unified bridge-retirement audit cadence** — periodic ledger-zero gate check; signals to Director when all 5 bridges fire.

**Consumes:**
- **R2-Evaluator** — PR-A.3 carriers (closed strategy + memoization), PR-B body evaluator (eager baseline), PR-D harness primitives (`DifferentialEquals` runner wiring), PR-E lens application (`fold_lens_over_reflected_program` integration seam).
- **R2-Grounding** — Shape A 3-target grounding (Rust + Python + Go) for L5 cross-target receipts.
- **PB Manager continuation** — T-FixedPoint completion (TC3 dependency); T-LensProducer-Retirement (consumed indirectly via TC1 substrate-research strengthening); 3 PB-side bridge slices landing toward ledger-zero.
- **Substrate Manager continuation** — T-Substrate-Lens-Primitive (TC1/TC2 strengthening; TC3 evaluation-step witness shape per [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L165); SourceSpan.file participation retirement (Bridge #1).

## Autonomous dispatch authority

- Authors all Verification sub-briefs without Director (per `feedback_standing_managers_need_owned_deliverables.md` discipline).
- Dispatches workers against Verification sub-briefs once cross-program prerequisites land.
- Resolves Verification-internal scope refinements; escalates substrate-shape questions / cross-program scope-changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline (carried into R3): every Verification worker brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance.
- **TC3 substrate-fact-introduction**: when B5 + T-Substrate-Lens-Primitive land, author substrate path through INVARIANTS §P1 procedure; cross-coordinate with Substrate Manager for the meta-theorem quantifier shape.

## Reporting cadence

- **Lane-close → R2 Release Manager continuation** (closure ledger maintenance via bold-lynx-173 #1135). Each lane's structural acceptance gate IS the demo per the structural-acceptance-per-lane-close discipline.
- **Cross-program signals** (e.g., bridge ledger-zero audit results) → cross-manager queue + Director.
- **TC strict-fire activation signals** → Director (gates R3 verification surface closure); unified-predicate coverage requirements route to Substrate when mature.
- **Blockers + scope changes** → Director (#828).
- **Brief-PR cadence** (per `feedback_brief_pr_cadence.md`): brief PRs only when carrying a new cross-manager signal; pure checkbox maintenance bundles into next signal PR or end-of-session sweep.

## Acceptance — `.dag` gates

Each lane closes under a structural acceptance gate authored as a `.dag` `TestClaim`:

- **Lane 1**: closes under both `l4_emit_eval_match` (per [`r3-structure.md`](../r3-structure.md) L53 — every `.dag` program in certification corpus has emit-target output equal to `.dag` eval output, algebraic equality) AND `l7_algebraic_laws_witnessed` (per [`r3-structure.md`](../r3-structure.md) L54 — every algebra × every applicable law has a runtime-constructed witness via `AlgebraicLaw` `TestPredicate`). Partial-coverage early slices do NOT close the lane; full coverage required.
- **Lane 2**: `l5_cross_target_consistency` (per [`r3-structure.md`](../r3-structure.md) L56) — for every `.dag` program, emitted Rust/Python/Go produce equivalent runtime behavior on the certification corpus (algebraic equivalence over computational results, not byte identity).
- **Absorbed responsibility (TC bundle)**: TC1/TC2/TC3 strict-fire activation across the three deferred-claim fixtures via unified `BinaryDimensionReportEquals` once Substrate lands the predicate and each reflection-aware modifier is covered — tracked via audit cadence, not as a lane-close gate.
- **Ledger gate**: `bridge_retirement_ledger_zero` — unified ledger reports 0 named identity bridges remaining (per [`r3-structure.md`](../r3-structure.md) L84).

## Sub-briefs (authored / pending)

**Authored at spawn (this PR):**
- [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md) — Lane 1 standby brief.
- [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md) — Lane 2 standby brief.
- [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) — absorbed-responsibility TC1/TC2/TC3 bundle (NOT a lane; audit-cadence artifact per `r3-structure.md` L108 2-lane authority).

**Pending (post-spawn manager authors autonomously):**
- Unified `BinaryDimensionReportEquals` coverage-requirements proposal (TC1/TC2/TC3 inputs; Substrate authors predicate variant when proposal is mature).
- Lane 1 / Lane 2 implementation worker briefs (gated on R2-Evaluator landing — convert from standby to dispatch-ready when prerequisites fire).

## Working state (fill on dispatch)

Lane status table refreshes here as work lands. Initial state: 2 lanes in standby + 1 ledger gate in audit cadence + TC bundle absorbed-responsibility audit cadence; bridge map row maintenance ongoing.

## Cross-refs

- Parent: [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 2 + §"Lane structure" rows for T-Verification-L4-L7-Direct / T-Verification-L5-Corpus / T-Bridge-Retirement
- Closure ledger predecessor: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) (R2 closed-with-residuals 2026-04-30)
- R2 Evaluator producer brief: [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md)
- TC3 upstream declarative shape: [`docs/briefs/r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3 — Strong-normalization TestClaim" (L145-215; ownership transitions to Verification per L181-185)
- Unified substrate-introduction ratification: [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427), [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359)
- TC1 deeper analysis: [PR #1309](https://github.com/gunb-ai/gunbc/pull/1309)
- TC2 independent coverage analysis: [PR #1316](https://github.com/gunb-ai/gunbc/pull/1316)
- Worker B PR-D scaffold consumer: [`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md)
- Bridge distribution map: [`docs/r3-structure.md`](../r3-structure.md) L98
- INVARIANTS substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1
