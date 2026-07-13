# Cost / risk / benefit — the floor's unmodeled optimization

*Captured 2026-07-12 from an operator working session; expanded 2026-07-13 into a migration plan; reconciled to the post-#6506 baseline 2026-07-13. Status: **model + directive + staged plan**. **Stages 0–2 have landed as an approximation in #6506** (hermetic-by-default floor, 5s fast-lane budget, `test/claim/long/` cadence, memory governor); **Stages 3–5 are open**. #6506 is a good-enough interim — a hand-set budget and hand-moved files that stand in for the derived cost/benefit model — explicitly **not the final state**.*

This doc covers the whole arc the operator asked for: **where we were** (the cruft), **where we are now** (the #6506 approximation), **where we're headed** (the model), **how each piece reconciles or dies**, and the **staged path with relief points** that drags the tree onto the model without a fail-open window.

---

## 1. Thesis — one optimization, many disguises

"What do we actually run on the floor" is **one optimization**: maximize `benefit − cost` subject to a **budget** (§1 — time is finite), with the fail-closed invariant in §2. Every mechanism the floor currently uses to answer that question is an *unmodeled proxy* for it — a hand-tuned stand-in that got its own vocabulary because cost, risk, and benefit were never made first-class:

- **affected-set** — don't pay to re-run a claim whose benefit (chance of catching a *new* regression) is ~0 because its inputs didn't move.
- **wet / dry (mocks)** — trade a sliver of fidelity (risk) for a large cost cut, so the freed budget buys more coverage. *"Mocks let you run more tests while keeping ~99.99% of the behavioral guarantee"* — the operator's framing, and the canonical case.
- **receipt / `long` / `manual/`** — defer a claim whose PR-marginal benefit doesn't clear its cost.
- **`ReadsLiveTree` (never-skip)** — a claim whose inputs the diff can't bound; really a *cost* term plus a fidelity choice, not a kind.
- **`WitnessKind` / `TestClassification` / the test-kind names (`test`/`receipt`/`demo`/`integration`)** — lossy compressions of *where a claim lands on the cost-benefit plane*, mistaken for intrinsic properties.

None is a primitive. Each is `argmax(benefit − cost)` evaluated at a different lever. They feel like a pile of separate policies only because the three quantities are unquantified — so each seam grew its own proxy, and each decision gets re-litigated by a human every conversation (the "ask once, compile forever" tax). Most of it is, bluntly, **astrology**: qualitative buckets standing in for numbers nobody computed.

The root anemia: **`test fn` bakes in `cost ≈ 0, run always`.** That is false (`s1_closure` ≈ 10 min wet). Every category we keep inventing patches a place that assumption breaks. Decompress the assumption instead of patching around it (DESIGN §2: decompress → map → reduce).

## 2. The invariant — traded risk must stay observable

The mock example fixes the one rule that keeps this from being corner-cutting. **"99.99%" is real only if the 0.01% is instrumented.** A mock spends a small risk budget to buy cost; that trade is honest *only* when the residual divergence is a **counted, cadenced signal** (the nightly affected-set falsifier; fixture staleness) — not an assumption. If the deferred risk is silent, you don't have 99.99%, you have an unmeasured hole — the §5 absorbing-fallback trap (frequency zeroed by construction, the cost surfacing later as a budget break).

So: **you may trade fidelity for cost, but the fidelity you gave up must remain observable.** This reframes the tree's deepest current default, which is the actual bug — **wet is treated as sacred** (`ReadsLiveTree` never skips; wet is "truth," dry is suspect). Wet is just the highest-cost point on a fidelity curve, usually *not worth it*. Fail-closed does not require always paying for wet; it requires that when you take the cheaper point, the deferred risk is **loud and detected, not ignored**. Deferred-and-detected is fail-closed; silently-skipped is fail-open. The current design conflates the two.

---

## 3. Where we were — the cruft inventory (grounded)

This is the pre-#6506 baseline: four-plus uncoordinated carriers of "what kind of verification is this," none of which drove cadence, plus a wet-is-sacred default. It is what has to be reconciled or killed, and it is the "before" that §3a's approximation and §5's fates are measured against:

| Artifact | File | What it really encodes | Problem |
|---|---|---|---|
| **`LiveTreeDisposition`** (`ReadsLiveTree \| SubstrateInputsOnly`) | `src/v2/std/live_tree.dag`; branch at `cli_run.rs:2246` (`if reads_live_tree { return false }`) and `floor_row_would_skip` (`ReadsLiveTree => false`) in `src/v2/workflow/affected_set_floor_runner.dag` | input-boundedness for skip, *conflated with* fidelity/cost | boolean; forces never-skip; misnamed — all 30 rows read the **checkout** (deterministic), none touch network/clock |
| **`WitnessKind`** (`CorpusWitnessKind \| ExecutionWitnessKind`) | `dag/std/realization_schedule.dag`; on `CommitWitnessClaim` | run-shape / resource profile | a cost input masquerading as a kind |
| **`TestClassification { tier, layer }`**, `TestgenLayer = Unit\|Integration\|Boundary` | `src/v2/std/verification.dag` | scope + depth | unfinished testgen lens (operator: "never got there"); `Integration` half-modeled and abandoned |
| **naming / placement convention** | `witness_discovery_scan_dirs = ["dag/test/claim","src/v2/test/claim/manual"]` (`dag/gunbc/ci_layer_roots.dag:9`); `*_test.dag`, `manual/`, `*receipt*`, `*demo*` | *everything*, implicitly | 80 `test fn`s in `manual/` swept onto the PR floor by path, not by cost |
| **execution mode = `Wet`, hardcoded** | `claim_executor.rs` (`run_shared_entry_claims:510`, `run_discovery_batch_node:826`) | fidelity choice | no hermetic-on-PR path; every claim pays wet even when a mock would do |

Concrete blast radius that drove the fix: the floor timed out at **270 min** (a `#[floor-memory]` heartbeat with no witness progress for ~4h — a hang/cost blowout, not OOM). `s1_closure` alone was a ~10-min `ReadsLiveTree` receipt pinned onto every PR by the never-skip rule. The 30 `ReadsLiveTree` witnesses are *all* checkout-deterministic — the label is "can't prove input-bounded," not "non-hermetic."

## 3a. Where we are now — the #6506 approximation

#6506 landed the first mechanisms. Treat them as the current baseline, and as *approximations* of the model — good-enough interims, each standing in for a modeled quantity that Stages 3–5 will derive. **Grounding caveat: every path and symbol named in this section is introduced by #6506 and is absent from the current tree until that PR merges** — `git grep` at main/PR-head will not find them; they are cited as the assume-merged baseline, not as present-tree facts.

- **Explicit budget exists, but hand-set.** #6506 introduces `gunbc_ci_fast_lane_witness_eval_budget = 5s` in `ci_floor_plan.dag`, enforced fail-closed as a typed `EvalBudgetExceeded` (in-eval poll + completion-side) — both symbols are absent from the current tree and land only when #6506 merges. This *approximates* the cost axis: a single flat threshold stands in for a per-claim cost estimate. It is the §5 smuggled-heuristic (a bare threshold selecting an arm), honest only because it refuses loudly and countably — retired by Stage 4 (derived cost).
- **`long/` cadence exists, but populated by hand.** 26 over-budget files moved to `test/claim/long/` (+ 12 hermetic-refusers to `test/claim/execution/`). This *approximates* cost-derived deferral: membership was decided by running the 5s budget once and moving what breached, not by a standing cost model. Right shape (counted, cadenced, not silent), interim mechanism.
- **Hermetic-by-default floor exists, fidelity axis not yet modeled.** #6506 re-homes `ExecutionMode` (its PR target is `dag/std/execution_mode.dag`; the current in-tree module is `src/v2/std/execution_mode.dag` — the `dag/std/` path lands only when #6506 merges); discovery/falsifier run Hermetic; wet lanes declared in `ci_layer_roots.dag`. This *approximates* the fidelity axis — but ε (the mock↔wet divergence, §2's observability requirement) is still carried by the nightly falsifier as a coarse whole-corpus check, not a per-claim instrumented counter.
- **Memory governor** (AIMD vs cgroup budget, pinned width constants deleted). The space half of cost, live as a safety net; its own dissolve-on is a graph-derived `CostAccount.space`.

Net: the acute 270-min timeout is addressed, and the *directions* are all correct — but every one is a hand-set stand-in for a number that isn't modeled yet. That gap is Stages 3–5.

## 4. Where we're headed — the target model

Two quantitative axes and a budget; the qualitative kinds become **derived regions**, not primitives:

- **cost** — time to evaluate a claim. Near-term: measured (`wall_nanos`, already emitted). North star: **derived from the graph** (§4) — effect counts × per-effect latency, fold/loop bounds (`loop_bound_edge`, `DescentEvidence`). Measurement becomes the *oracle* that validates the derivation.
- **risk / fidelity** — how much behavioral guarantee a given execution point yields (wet = 1.0; a mock = 1−ε, with ε the instrumented divergence). Choosing a cheaper point is allowed iff ε stays observable (§2).
- **benefit** — displaced cost (§6): the pain a red catches, weighted by how often it fires. Hard axis; start with the §5 lower bound (no discriminating red ⟹ ~0 gate-value) + observed red-frequency; declared interim for the rest.
- **budget** — an explicit PR wall-clock number (replacing the implicit, wrong 270-min timeout).

**Admission derives:** run the highest `benefit/cost` claims that fit the budget at the fidelity point whose traded risk is instrumented; everything declined is a **counted, typed `Deferred{cost, cadence, reason}`** row (never a silent skip), run on `long`/nightly. `affected-set`, `wet/dry`, `receipt/long`, `ReadsLiveTree`, and the test-kind names all fall out as *readings* of this, and the vocabulary is deleted rather than extended.

## 5. Reconciliation map — each proxy's fate

#6506 already moves several of these toward their fate *approximately* (§3a): `execution mode hardcoded Wet` → per-lane hermetic/wet; `manual/` scan-dir → `long/` by budget. The "How" column remains the *final* form each must reach — a derived reading, not a hand-set lane.

| Today | Fate | How |
|---|---|---|
| `LiveTreeDisposition` boolean | **replace** with an `input-domain` axis (`BoundedByGraph \| ReadsCheckout \| DependsOnUnfixedState`) feeding cost + skip | all 30 current rows re-stamp to `ReadsCheckout` (deterministic); `never-skip` special-case deleted — high cost + unbounded input just *ranks low* for PR and defers, observably |
| `ReadsLiveTree => false` (never-skip) | **delete** | subsumed by cost ranking + the §2 invariant (deferred-and-detected via the falsifier) |
| execution mode hardcoded `Wet` | **make a per-claim fidelity choice** | mock where a published case exists (cheaper, ε instrumented); wet only where fidelity is worth its cost |
| `WitnessKind` (Corpus/Execution) | **demote to a cost input** | run-shape feeds the cost estimate; not a standalone kind |
| `TestClassification` / `TestgenLayer.Integration` | **park / derive** | retract reliance (unfinished lens); `Integration` becomes derived (`DependsOnUnfixedState ∨ expensive`), not authored |
| `manual/` in `witness_discovery_scan_dirs` | **delete the path-based rule** | admission by measured cost replaces "scan this dir"; the 80 receipts defer by cost, not by folder |
| `receipt` / `demo` names | **derive / drop** | `receipt` = high-cost, low PR-marginal-benefit → `long`; pure `demo` (no discriminating red) = §5 zero-value → **dropped**, not cadenced |

## 6. Migration — staged, each stage a relief point

Every stage is independently landable, keeps the §2 invariant (no silent drops), and delivers CI relief. Ordered so the acute timeout dies first and the model hardens under it.

> **Landed (assume merged): PR #6506** implements the mechanism side of **Stages 0–2** as an approximation (see §3a) — hermetic-by-default floor, 5s fast-lane budget, `test/claim/long/`, memory governor. This doc is the *model* those mechanisms should derive from; #6506 is the *mechanism*, and its stand-ins (hand-set budget, hand-moved files, coarse ε) are exactly what Stages 3–5 retire. The **live forward work is Stages 3–5** (reconcile the kind-classifiers, derive cost from the graph, model benefit + instrument fidelity ε). Stages 0–2 remain in the list as the "done-as-approximation" record.

- **Stage 0 — Relief now (no new model). RELIEF: unbreaks the 270-min timeout.** *(landed as approximation: #6506 — `test/claim/long/`, populated by hand from the 5s budget.)*
  Take the expensive/broken receipts off the *PR* floor and run them on a `long` cadence (main/nightly). Concretely: drop `src/v2/test/claim/manual` from `witness_discovery_scan_dirs`, stand up a `long` runner. **Invariant-preserving:** the removed set is a *declared, counted* deferral with a reason (cost) and a cadence — not a silent scan-dir deletion — and the existing nightly falsifier keeps running the full corpus cold, so nothing goes dark. This is the hand-applied conclusion of the model; Stages 1–2 make it *derived*.

- **Stage 1 — Cost first-class (measured) + explicit budget. RELIEF: floor size tracks a real number, not a folder.** *(landed as approximation: #6506 — the flat 5s budget stands in for a per-claim cost estimate.)*
  Carry a per-claim measured cost receipt (from `wall_nanos` the floor already emits; BatchRecord/`emit_gantt`/`write_resolve_receipt` are the source). Declare a PR wall-clock budget. Admission = fit by cost; declined = counted `Deferred{cost, cadence}`. Stage 0's `manual/` un-scoping now becomes *derived from cost*, and the scan-dir/roster hand-list (first cruft) is deleted.

- **Stage 2 — Collapse `LiveTreeDisposition` into input-domain + fidelity. RELIEF: wet stops being mandatory.** *(landed as approximation: #6506 — hermetic-by-default + declared wet lanes; ε still coarse, carried by the nightly falsifier not a per-claim counter.)*
  Replace the boolean with the `input-domain` axis; delete the never-skip special case; introduce the per-claim fidelity choice (mock vs wet) with the §2 instrumentation requirement (a wet→mock swap must have its divergence on the falsifier). Kills the wet-is-sacred default and the second cruft carrier.

- **Stage 3 — Reconcile the kind-classifiers. RELIEF: one authority, deletes 3 forks.**
  `WitnessKind` → cost input; `TestClassification`/`Integration` → derived or parked; `receipt`/`demo`/`manual` naming → derived from the axes. The §5 truth-teller (discriminating-red present?) becomes a *value lower bound*: no-red ⟹ drop (that's a demo), not cadence.

- **Stage 4 — Cost from the graph (north star). RELIEF: cost known before running.**
  Derive cost from effect counts × per-effect latency + fold bounds; measurement (Stage 1) becomes the oracle — measured-vs-predicted divergence is a counted signal (same falsifier pattern). Fail-closed: unknown fold bound ⇒ counts as expensive.

- **Stage 5 — Benefit axis. RELIEF: admission by value-per-cost; last proxies collapse.**
  §5 lower bound + observed red-frequency first; declared interim for the rest. Once benefit exists, `receipt` vs `test` fully dissolves and admission is a clean knapsack.

## 7. Missing pieces (highlighted, not papered over)

Post-#6506, the acute holes are patched by approximations (§3a); these are what the approximations still stand in for:

- **No per-effect cost model** — latency expectations per effect kind (file read / compute / network) are declared nowhere; #6506's flat 5s budget is the stand-in.
- **Cost not derived, only bounded by the budget** — a claim is "cheap" iff it finishes under 5s, not because its graph was priced. Where a fold/loop bound isn't statically known, cost is a *bound*; fail-closed = unknown ⇒ expensive.
- **Explicit budget exists but is flat + hand-set** — 5s for every claim; the real budget is per-run wall-clock to be *allocated* across claims by value/cost, not a uniform per-claim cap.
- **`long/` membership is hand-moved, not derived** — a standing cost model would re-derive it every run instead of freezing a one-time move.
- **Benefit largely unmodeled** — only footholds are the §5 no-red lower bound and observed red-frequency.
- **Fidelity/ε not per-claim** — the "99.99%" is carried coarsely by the nightly falsifier; the divergence a mock trades away needs a real per-claim counter.

## 8. North star

The run / defer / drop decision *derives* from cost, risk, and benefit under a budget, with any traded-away risk kept observable — so it stops being re-argued by hand every conversation, and the qualitative vocabulary (`ReadsLiveTree`, `WitnessKind`, `receipt`, `demo`, `integration`) is **deleted, not extended**.
