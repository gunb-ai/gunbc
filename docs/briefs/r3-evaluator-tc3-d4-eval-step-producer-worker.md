# R3 Evaluator — TC3 D4 evaluation-step / bounded-step producer worker brief

**Status:** DRAFT — single-worker scope; **DISPATCH HARD-GATED on D3 T-FixedPoint termination semantics + producer-shape ratification per `r3-v-tc3-pattern-a-second-mover-conformance-audit.md` §"Strict-Fire Preconditions"** items 1-5. Authored under R3 Evaluator Mgr standing authority pursuant to Director TC3 (a)-disposition AUTHORIZE at gunbc#828 c#4413696757 + Verification Mgr cross-Mgr token c#4413701849 (delivered via crisp-bat-13 inbox #2065 c#4413701849).

**Authority anchors:**

- Director TC3 (a)-disposition AUTHORIZED for R3-window dispatch — gunbc#828 c#4413696757.
- Verification Mgr cross-Mgr token (D4 brief authoring under standing authority) — #2075 c#4413701849.
- Lane tracker — `#1941` (R3 Evaluator Mgr lane through R3 close).
- V-side anchors: `docs/briefs/r3-v-pattern-a-tc3-v1-worker.md` §Strict-Fire Preconditions D1-D5 (D4 = "E5 `Descent` execution proof + evaluator evaluation-step producer; Evaluator + Substrate"); `docs/briefs/r3-v-tc3-pattern-a-second-mover-conformance-audit.md` §"Witness-Shape Commitments" + §"Strict-Fire Preconditions" items 1-5; `docs/briefs/r3-v-pattern-a-coverage-rollup.md` §TC3 row.
- Plan single-authority: `docs/r3-program-plan.md` §1.8 row #13 `tc3_pattern_a_second_mover_executable` DECLARED (runtime prereq: Descent execution proof (E5) + eval-step producer); §10.3 Pattern-A policy + per-instance ratification posture.
- Sister Evaluator-side brief: `docs/briefs/r3-pr-e6-g1a-static-lens-fold-worker.md` (G1.a static-lens-fold producer-surface-wiring; whole-`Dag` traversal authority — distinct producer surface from this brief).
- E5 descent contract on main: PR #2147 (carrier) + #2190 (consumer) MERGED 2026-05-08 — `descent_execution_proof` token consumable.

## 1 Scope

Single Evaluator PR — implements the evaluation-step / bounded-step producer surface for TC3 second-mover, producing two `DimensionReport<Dag>` values consumable by V-side `BinaryDimensionReportEquals`:

1. **Producer A — baseline `tc3_evaluation_step_baseline_dimension_report`**: emits a `DimensionReport<Dag>` for evaluation-step semantics over the well-typed `.dag` fragment representative selected per Director/Substrate/Evaluator ratification of universal-fragment coverage shape (item 5 of conformance audit §Strict-Fire Preconditions).
2. **Producer B — compare `tc3_evaluation_step_compare_dimension_report`**: emits a `DimensionReport<Dag>` from bounded forward execution / termination evidence on the same representative; consumes `descent_execution_proof` token (E5 contract carrier already on main per PR #2147 + #2190).
3. **Common envelope**: both reports use the same `Dag` carrier + same structural envelope (`DimensionOk { dimension_name, composed, witnesses }` / `DimensionFail { dimension_name, violations, witnesses }`) per conformance audit §Witness-Shape Commitments items 3-4.
4. **Test fixture wiring**: the V-side TC3 V1 fixture (lively-raven-404 PR #2435 sentinel-preserved scaffold) becomes non-vacuous when both producers land on `origin/main`; runner auto-upgrades from `NotYetImplemented` to live `BinaryDimensionReportEquals` strict-fire (V-side responsibility; observability here, not separate landable).

**Out of scope (defer past R3 unless Director re-ratifies):**

- Universal-fragment coverage proof (structural induction OR generated exhaustive producer OR bounded representative harness — per conformance audit §Strict-Fire Preconditions item 5; this is a separate ratification surface).
- T-FixedPoint termination semantics implementation (per conformance audit item 3; separate Substrate+PB lane — Cluster M Phase 3 / #2087 cascade).
- Generic `fold_lens<C>` (canvas-deferred per #1972).
- Serialized normal forms, step trace strings, byte snapshots, per-program sample corpora (explicitly forbidden by conformance audit §Witness-Shape Commitments).
- New `DimensionReport<C>` carrier shape (existing variant set is the contract).
- New `Value` variants, parser/lowerer changes, new substrate carrier shapes.

## 2 Hard preconditions before dispatch

Per `r3-v-tc3-pattern-a-second-mover-conformance-audit.md` §"Strict-Fire Preconditions" items 1-5, this brief HOLDS dispatch until:

1. **Item 1**: B5 loop construction-closure green at HEAD. Worker grep-verifies before authoring impl.
2. **Item 2**: G1.a static-lens-fold producer-surface-wiring (sister brief at `r3-pr-e6-g1a-static-lens-fold-worker.md`) landed on `origin/main` so generic `DimensionReport<Dag>` production is a callable evaluator surface (NOT a `NotYetImplemented` returner) per `test_runner.rs:2627-2630` retirement.
3. **Item 3**: T-FixedPoint termination semantics — Cluster M Phase 3 / #2087 lane status. If T-FixedPoint not landed in R3 window, this brief HOLDS at "Producer B" sub-scope. Producer A (baseline evaluation-step) may still be R3-eligible as a partial slice if Director ratifies a milestone narrowing per V-side §"Strict-Fire Preconditions" item 2 ("D3 + D4 land for full strict-fire — no PASSING without (b) unless Director narrows milestone").
4. **Item 4** (this brief's scope): the producer surface itself.
5. **Item 5**: universal-fragment coverage shape ratification — Director + Substrate + Evaluator. **Open canvas-shape question** (see §6).
6. **E5 descent contract reachable**: `descent_execution_proof` token consumable from evaluator (PR #2147 + #2190 already on main; verify at dispatch time per `feedback_grep_verify_post_x_ready_briefs.md`).

Dispatch fires when items 1, 2, 4, 6 are satisfied AND item 5 has Director-ratified shape AND (item 3 satisfied OR Director-narrowed milestone for Producer A partial slice).

## 3 Implementation guidance (non-authoritative)

- **Reuse G1.a static-lens-fold consumer-wiring infrastructure**: once G1.a producer-surface-wiring lands, this brief's two producers reuse the same `Witness<C>` / `OptionalDiagnostic` / `DimensionReport<C>` construction path via `Value::RecordValue` / `Value::VariantValue`. Do not duplicate report construction logic.
- **Producer A (baseline)**: walks the representative `Dag` under the existing evaluator entry; emits `DimensionOk` when evaluation-step semantics hold structurally. No new traversal authority — reuse G1.a's whole-`Dag` / named-decl-ref traversal.
- **Producer B (bounded-step)**: consumes `descent_execution_proof` (existing E5 contract carrier) + folds bounded forward execution evidence into the report; on `Missing | Unknown | Incomplete | NonStrict` residuals (per descent contract substrate type), emits `DimensionFail` with the residual partition encoded as a `Witness<Dag>` violation (NOT serialized prose).
- **Carrier discipline**: `Dag` IS the reflected program per Q-Reification Option A RATIFIED (`docs/r3-program-plan.md` §10.3 Q-Reification row, MERGED 2026-05-07 via #2096). Both producers operate over `Dag`-as-carrier; do NOT introduce a parallel report-of-trace / serialized-normal-form carrier (per `feedback_import_not_redeclare_carriers.md`).
- **No load-bearing ratchet retirement** (per `feedback_load_bearing_ratchet_preservation.md`): the `BinaryDimensionReportEquals` `NotYetImplemented` runner sentinel at `test_runner.rs:2627` will retire when ALL producer-side surfaces land (G1.a + this brief + TC2 second-strategy + future per-instance ratifications). Do not partial-retire the sentinel.
- **Sister-brief coordination**: G1.a static-lens-fold worker brief (`r3-pr-e6-g1a-static-lens-fold-worker.md`) lands FIRST per item 2 hard precondition; this brief's authoring happens AFTER G1.a is on main so report-construction infrastructure is reusable.

## 4 Acceptance criteria

1. `cargo test -p v3-compiler` green: V-side TC3 V1 fixture (lively-raven-404 PR #2435 sentinel-preserved scaffold) auto-upgrades from `NotYetImplemented` to live `BinaryDimensionReportEquals` Pass on the representative selected per item-5 ratification.
2. Both producers (`tc3_evaluation_step_baseline_dimension_report` + `tc3_evaluation_step_compare_dimension_report`) emit structurally well-formed `DimensionReport<Dag>` values via `Value::RecordValue` / `Value::VariantValue` (no serialized prose, no per-program sample corpora).
3. Producer B's `DimensionFail` violations encode descent-contract residual variants as `Witness<Dag>` (not strings).
4. `cargo clippy --all-targets -- -D warnings` clean.
5. No new `EvalError` variants without explicit Director-ratified canvas pairing.
6. SG-0 hand-Rust delta = 0 for any non-test code (this brief is a producer-surface-wiring slice, not a P0 stage0 addition; per `feedback_p0_zero_hand_rust_check_before_authoring.md`).
7. Gate #13 `tc3_pattern_a_second_mover_executable` reaches CONSUMER_LANDED on landing + PASSING on green CI (subject to item-3 T-FixedPoint precondition for full strict-fire; if Director-narrowed milestone lands Producer A only, gate reaches partial PASSING per Director ratification scope).

## 5 Dispatch sequencing (lane-tracker view)

| Step | Owner | Status |
| --- | --- | --- |
| 1. G1.a static-lens-fold producer-surface-wiring landed on `origin/main` | Evaluator Mgr (PM #846 spawn-authority queue P0 per c#4413664759) | OUTSTANDING — dispatch authorized but worker not yet spawned |
| 2. T-FixedPoint termination semantics landing (Cluster M Phase 3 / #2087) | Substrate Mgr / PB Mgr | OUTSTANDING — separate cascade |
| 3. Universal-fragment coverage shape ratification (item 5) | Director + Substrate + Evaluator | **OPEN canvas-shape question — see §6** |
| 4. E5 descent contract reachable | Substrate (#2147 + #2190 MERGED 2026-05-08) | DONE |
| 5. B5 loop construction-closure green | (lane-owner unspecified per audit) | grep-verify at dispatch time |
| 6. **TC3 D4 worker dispatch (this brief)** | Evaluator Mgr | HARD-GATED on steps 1, 2, 3, 4, 5 per §2 hard preconditions |
| 7. V-side runner auto-upgrade (PR #2435 sentinel removal) | V-Mgr (#2075) | post step 6 |

## 6 Item-5 ratification — (γ) bounded representative harness RATIFIED

Per Director (γ) RATIFICATION at gunbc#828 c#4413725564: TC3 D4 baseline emits a single-representative `DimensionReport<Dag>` mirroring G1.a static-representative pattern (Q-PAFS Path A ACCEPTED 2026-05-06; PR #1824 merge record). Strict-mirror discipline preserves Pattern-A architectural consistency across TC1/TC2/TC3.

Director delegated specific-representative selection + scope statement to Mgr-tier canvas authoring (this section), with Director ratification at canvas-surface time per `feedback_substrate_shape_belongs_in_mgr_canvas`.

### Canvas grep — candidate subject programs at HEAD

Constraint: the representative must (i) be small + Director-visible; (ii) exercise non-trivial bounded-step termination semantics so Producer B has substantive bounded-forward-execution evidence to fold; (iii) mirror G1.a's single-static-top-level-subject pattern; (iv) avoid `LoopBound::Descent` content if possible (descent contract on main but adds an extra coupling axis — Cardinality is sufficient and simpler).

Candidates considered:

| Candidate | Locus | Fit |
| --- | --- | --- |
| Reuse TC1 η-pair `eta_subject_f` / `eta_subject_f_eta` from `tc1_substrate_lens_eta_equivalence_strict_fire.dag` | existing fixture | **Reject** — η-pair has no bounded loop / step structure; Producer B has nothing substantive to fold (`x + 1` is trivial) |
| Existing `tc3_strong_normalization_deferred.dag` subject body | existing fixture | **Reject** — fixture's `source: ""` is structural-claim convention; no subject program declared |
| Author fresh `tc3_strong_normalization_strict_fire.dag` (mirrors TC1 `_deferred` / `_strict_fire` pair pattern) with smallest bounded-loop subject | new fixture (parallel to TC1 strict-fire fixture pattern) | **Recommended** — mirrors TC1 V1 strict-fire fixture authoring precedent; Director-visible; minimal scope |
| Reuse existing `r3_pb_runtime_evaluator_corpus_seeds.dag` or other corpus-seeded program | existing fixture | **Reject** — corpus-seeded programs are not Director-visible bounded-representative; scope statement would be larger than (γ) intent |

### Specific-representative recommendation (Mgr canvas-tier — surfacing for Director ratification)

**Author fresh `src/v3/compiler/tests/fixtures/tc3_strong_normalization_strict_fire.dag`** mirroring TC1 V1 strict-fire fixture pattern. Subject body proposal: smallest non-trivial `LoopBound::Cardinality`-bounded computation — e.g. a bounded fold over a 3-element literal list summing to a known constant. Concrete shape (illustrative; final wording is canvas-tier):

```
fn tc3_subject_bounded_sum() -> Int =
  fold([1, 2, 3], 0, lambda acc x. acc + x)   // bounded by Cardinality(3); converges to 6
```

Producer A (baseline `tc3_evaluation_step_baseline_dimension_report`): walks the subject structurally; emits `DimensionOk` if the bounded-step semantics composition holds (reuses G1.a `Witness<C>` / `OptionalDiagnostic` envelope).

Producer B (compare `tc3_evaluation_step_compare_dimension_report`): bounded forward execution evidence over the same subject — folds the literal list cardinality into a `Witness<Dag>` partition; emits `DimensionOk` when bounded-step convergence agrees with baseline structural shape.

Both reports use `Dag` as carrier per Q-Reification Option A.

### Scope statement (per Director "scope-statement to your canvas-tier authoring")

This representative is the **single canonical TC3 second-mover bounded-representative for R3 close**. Universal-fragment coverage is **deferred-not-blocked** to a post-R3 cycle (where (α)/(β) novel-substrate-introduction can be canvas-authored separately). Per `feedback_pattern_a_scaffold_sentinel_per_instance_ratification`: per-instance ratification preserved — gate #13 PASSING is reactive-not-predetermined on this representative + producer surface landing + B5 + G1.a in R3 window.

The representative is **not** a coverage proof; it is a strict-fire witness for second-mover disposition under (γ). Mirror to G1.a TC1 V1 strict-fire fixture authority shape.

### Director ratification

**RATIFIED** at gunbc#828 c#4413738978 (2026-05-09): specific-representative selection + scope statement above are dispatch-locked. The illustrative subject body (`fn tc3_subject_bounded_sum() -> Int = fold([1, 2, 3], 0, lambda acc x. acc + x)`) is **illustrative-not-binding**; final fixture wording is canvas-tier authoring scope at worker-dispatch time per `feedback_substrate_shape_belongs_in_mgr_canvas`. Bounded-`Cardinality(3)` shape is ratified as the binding structural constraint.

Cross-Pattern-A consistency preserved across the family (TC1/TC2/TC3/TC4/#19) — strict-mirror discipline maintained; (α)/(β) novel-substrate-introduction formerly carved to R4+ via Class P partition (Debt-Paydown PR #2437), AMENDED 2026-05-09 by carve-promotion ratification + scope statement.

## 7 Discipline notes (worker-tier)

- **Grep-verify all six hard preconditions** at dispatch time per `feedback_grep_verify_post_x_ready_briefs.md` + `feedback_grep_consumer_surfaces_for_lane_completeness.md`. Pending unmerged PRs are NOT sufficient.
- **Reuse G1.a infrastructure** — do not parallel-declare report-construction helpers (`feedback_import_not_redeclare_carriers.md`).
- **No load-bearing ratchet partial-retire** (`feedback_load_bearing_ratchet_preservation.md`): runner sentinel retires only when full producer cascade lands.
- **4-axis grep before authoring impl** (`feedback_brief_author_4_axis_grep.md`): plan §1.8 row #13, §10.3 Pattern-A policy, V-side TC3 V1 brief, sister Evaluator G1.a brief on main.
- **Substrate-grep before authoring** (`feedback_substrate_grep_before_authoring.md`): confirm `descent_execution_proof` token signature stable + G1.a producer-surface API stable + T-FixedPoint substrate (when item 3 lands) consumable.

## 8 Out-of-band reactivation

If Director ratifies item-5 shape differently than nominated (α/β/γ) — e.g. surfaces a fourth shape — this brief is **superseded**, not refreshed. Author fresh worker brief at canvas-ratification land time.

If R3 close window passes without G1.a landing (step 1) OR item-5 ratification (step 3), this brief HOLDS without dispatch; gate #13 stays DECLARED honestly per Director (b)-fallback equivalent.

If Director narrows milestone for Producer A partial slice (per V-side §"Strict-Fire Preconditions" item 2 reservation), author fresh narrowed brief; do not stretch this brief's scope.
