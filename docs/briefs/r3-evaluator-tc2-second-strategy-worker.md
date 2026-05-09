# R3 Evaluator — TC2 second-strategy substrate worker brief

**Status:** DRAFT — single-worker scope; **DISPATCH HARD-GATED on Substrate Mgr `InputEvaluationOrder::RightFirst` variant landing on `origin/main`** (per Substrate canvas + Director ratification path described below). Authored under R3 Evaluator Mgr standing authority pursuant to Director TC2 (a)-disposition AUTHORIZED at gunbc#828 c#4413613840 (delivered via crisp-bat-13 inbox #2065 c#4413613840).

**Authority anchors:**

- Director TC2 (a)-disposition AUTHORIZED for R3-window dispatch — gunbc#828 c#4413613840.
- TC2 (a)-disposition prior ratification — gunbc#828 c#4413594508 ("R3-eligible IF Evaluator dispatches second-strategy substrate").
- Lane tracker — `#1941` (R3 Evaluator Mgr lane through R3 close); inventory amendment c#4411535048.
- V-side anchors: `docs/briefs/r3-v-pattern-a-tc2-v1-worker.md` (Slice 1 substrate/evaluator receipt); `docs/briefs/r3-v-tc2-church-rosser-analysis.md` §1 (tractability default = second `InputEvaluationOrder` under eager applicative); `docs/briefs/r3-v-tc2-pattern-a-second-mover-audit.md`.
- Plan single-authority: `docs/r3-program-plan.md` §1.8 row #12 `tc2_church_rosser_executable` DECLARED; §10.3 Q-PAFS Path A (TC1 first slice policy; TC2 lands second-strategy under (a)-disposition).
- Substrate-side dependency: Substrate Mgr `RightFirst`-addition canvas (warm-wolf-698 / #2068) — variant + TERMINAL-comment retirement paired; canvas-ratification surface delegated by Director to Substrate Mgr.
- Verification-side dependency: G1.a static-representative `DimensionReport<C>` production sufficient to produce two typed reports under two strategies (wise-bear-525 / #2075).

## 1 Scope

Single Evaluator PR — **lands AFTER Substrate `RightFirst` variant is on `origin/main`**:

1. **Evaluator strategy-keyed input scheduling**: `eval_node` (and any input-collection helpers it dispatches to) honors `EvalStrategy::ApplicativeOrder { input_order: RightFirst }` by evaluating the n-ary transform's input ports in reverse declared order before the transform fires; `LeftFirst` retains exact pre-existing semantics (no-op refactor for that arm).
2. **Strategy-keyed `DimensionReport<C>` production**: when `fold_lens<C>` (G1.a static-representative path) executes under each of the two `InputEvaluationOrder` inhabitants on a fixed representative program + lens set, the resulting `DimensionReport<C>` is structurally producible from each strategy independently and routable to the V-side runner for paired equality assertion.
3. **Test fixture wiring**: extend (or pair-author with V-side) the fixed representative program at `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag` (or successor strict-fire fixture) so that the unified `BinaryDimensionReportEquals` predicate consumer in `r2-evaluator-manager.md` L127 is wired to two strategy-keyed report producers.

**Out of scope (defer past R3 unless Director re-ratifies):**

- Normal-order / thunk strategy (separate canvas; not the (a)-disposition scope).
- Parallel evaluation strategy variants.
- Memo-key plumbing changes (PR-A.3 closed; touching memo-key invariants requires separate brief).
- G1.b generic dispatch (canvas-deferred per #1972).
- Indirect-call substrate (canvas-deferred per #1972 / T-E-P-Producer-Broadening).

## 2 Hard preconditions before dispatch

1. **Substrate landing on `origin/main`**: `InputEvaluationOrder::RightFirst` variant exists in `src/v3/std/runtime.dag` AND `src/v3/compiler/src/lib.rs` (variant + bootstrap regen + parse-surface mirror); TERMINAL comments at `runtime.dag` L108 and `lib.rs` L143-149 retired in same Substrate PR. **Worker grep-verifies before authoring impl** (per `feedback_grep_verify_post_x_ready_briefs.md` + `feedback_substrate_grep_before_authoring.md`).
2. **G1.a static-representative `DimensionReport<C>` producer reachable on `origin/main`** — Verification cross-Mgr signal at #2075 c#4413603923 + c#4413611431. Worker grep-verifies the producer is callable from a public evaluator surface and yields a structurally-comparable report.
3. **No load-bearing-ratchet retirement** (per `feedback_load_bearing_ratchet_preservation.md`): the only existing `LeftFirst`-keyed assertions in `test_runner.rs` L542-543 and the prose at `lib.rs` L140 must be preserved or extended (not replaced) to cover both strategies.

## 3 Implementation guidance (non-authoritative)

- **Single substrate PR** lands variant; Evaluator PR depends on it. Do **not** parallel-declare or shadow the variant in Evaluator-crate code (per `feedback_import_not_redeclare_carriers.md`).
- **`eval_node` input-order branch**: a single match on `strategy.input_order()` at the n-ary input collection site is sufficient; `LeftFirst` arm keeps the existing iterator order; `RightFirst` arm reverses. No changes to per-input evaluation semantics, only collection order.
- **Strategy-keyed report producer**: the unified `BinaryDimensionReportEquals` consumer (per `r2-evaluator-manager.md` L127) already takes a `DimensionReport<C>` pair; the Evaluator side's job is to produce two reports keyed by strategy, not to author predicate logic.
- **Fixture pairing**: coordinate with bright-hawk-891 / V-Mgr (#2075) on the fixed representative program — the V-side scaffold-with-sentinel PR #2396 lands the runner shape; this Evaluator PR makes the runner non-vacuous when both producers land.

## 4 Acceptance criteria

1. `cargo test -p v3-compiler` green on both strategy-keyed paths through `fold_lens<C>` (G1.a static-rep path).
2. `tc2_church_rosser_executable` runner asserts strict structural equality between the two `DimensionReport<C>` outputs on the fixed representative program; gate #12 reaches CONSUMER_LANDED on landing and PASSING on green CI.
3. `cargo clippy --all-targets -- -D warnings` clean.
4. No new `EvalError` variants without explicit Director-ratified canvas pairing.
5. Sentinel-preserved scaffold PR #2396 auto-upgrades from sentinel to live assertion when this PR lands (V-side responsibility; this brief tracks the auto-upgrade as observability, not a separate landable).

## 5 Dispatch sequencing (lane-tracker view)

| Step | Owner | Status |
| --- | --- | --- |
| 1. Substrate `RightFirst`-addition canvas authoring | Substrate Mgr (warm-wolf-698 / #2068) — standing authority | TOKEN ISSUED 2026-05-09 (this brief) |
| 2. Director canvas ratification | Director (#828) | pending Substrate canvas land |
| 3. Substrate variant-addition PR (variant + TERMINAL-comment retire, paired) | Substrate Mgr | pending step 2 |
| 4. Evaluator second-strategy worker dispatch (this brief) | Evaluator Mgr (crisp-bat-13 / #2065) | HARD-GATED on step 3 land |
| 5. V-side runner auto-upgrade (PR #2396 sentinel removal) | Verification Mgr (wise-bear-525 / #2075) | tracked as observability post step 4 |

**Cross-Mgr coordination tokens issued (this brief authoring action):**

- **Substrate (#2068)**: canvas-authoring request for `InputEvaluationOrder::RightFirst` addition + paired TERMINAL-comment retirement at `runtime.dag:L108` + `lib.rs:L143-149`.
- **Verification (#2075)**: hard-precondition acknowledgment that G1.a static-rep `DimensionReport<C>` producer must be reachable on `origin/main` before this brief can dispatch.
- **PB-Runtime (#2074)**: surface — `eval_node` input-order plumbing scope check; flag if any of this work crosses into PB-Runtime carrier territory before Evaluator-side single-PR scope is authored.

## 6 Discipline notes (worker-tier)

- **Grep-verify Substrate landing on `origin/main`** before authoring impl (`feedback_grep_verify_post_x_ready_briefs.md`); pending unmerged Substrate PR is NOT sufficient.
- **Grep-verify G1.a producer reachability** before wiring fixture (`feedback_substrate_grep_before_authoring.md`).
- **Do not retire `LeftFirst`-keyed assertions** — extend, do not replace (`feedback_load_bearing_ratchet_preservation.md`).
- **Import — don't parallel-declare — `EvalStrategy`/`InputEvaluationOrder`** carriers from Substrate landing (`feedback_import_not_redeclare_carriers.md`).
- **4-axis grep** before authoring (`feedback_brief_author_4_axis_grep.md`): plan §1.8 row #12, §10.3 Q-PAFS, V-side TC2 brief, Substrate variant landing.

## 7 Out-of-band reactivation

If Director or Substrate Mgr re-disposes (e.g. canvas surfaces a different shape than enum-variant addition — separate carrier, alternate dispatch surface, etc.), this brief is **superseded by the canvas disposition**, not refreshed. Author a fresh worker brief at canvas-ratification land time.

If R3 close window passes without Substrate variant landing on `origin/main`, this brief HOLDS without dispatch; gate #12 stays DECLARED honestly per Director (b)-fallback equivalent at gunbc#828 c#4413612223.
