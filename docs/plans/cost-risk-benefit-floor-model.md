# Cost / risk / benefit — the floor's unmodeled optimization

*Captured 2026-07-12 from an operator working session. This is the frame; the mechanism is a later lane. Status: **model + directive**, no code yet.*

## Thesis

"What do we actually run on the floor" is **one optimization**: maximize `benefit − cost` subject to a **budget** (§1 — time is finite), with the invariant below. Every mechanism the floor currently uses to answer that question is an *unmodeled proxy* for it — a hand-tuned stand-in that got its own vocabulary because cost, risk, and benefit were never made first-class:

- **affected-set** — don't pay to re-run a claim whose benefit (chance of catching a *new* regression) is ~0 because its inputs didn't move.
- **wet / dry (mocks)** — trade a sliver of fidelity (risk) for a large cost cut, so the freed budget buys more coverage. *"Mocks let you run more tests while keeping ~99.99% of the behavioral guarantee"* — the operator's framing, and the canonical case.
- **receipt / `long` / `manual/`** — defer a claim whose PR-marginal benefit doesn't clear its cost.
- **`ReadsLiveTree` (never-skip)** — a claim whose inputs the diff can't bound; really a *cost* term (and a fidelity choice), not a kind.
- **`WitnessKind` / `TestClassification` / test-kind names (`test`/`receipt`/`demo`/`integration`)** — lossy compressions of *where a claim lands on the cost-benefit plane*, mistaken for intrinsic properties.

None of these is a primitive. Each is `argmax(benefit − cost)` evaluated at a different lever. They feel like a pile of separate policies only because the three quantities are unquantified — so each seam grew its own proxy, and each decision gets re-litigated by a human every conversation (the "ask once, compile forever" tax, DESIGN open threads). Most of it is, bluntly, **astrology**: qualitative buckets standing in for numbers nobody computed.

The root anemia: **`test fn` bakes in `cost ≈ 0, run always`.** That assumption is simply false (`s1_closure` = ~10 min wet). Every category we keep inventing is a patch for where that false assumption breaks. Decompress the assumption instead of patching around it (DESIGN §2: decompress → map → reduce).

## The invariant — traded risk must stay observable

The mock example also fixes the one rule that keeps this from being corner-cutting. **"99.99%" is real only if the 0.01% is instrumented.** A mock spends a small risk budget to buy cost; that trade is honest *only* when the residual divergence is a **counted, cadenced signal** (the nightly falsifier, fixture staleness) — not an assumption. If the deferred risk is silent, you don't have 99.99%, you have an unmeasured hole — the §5 absorbing-fallback trap (the deficit's frequency zeroed by construction, the cost surfacing later as a budget break).

So: **you may trade fidelity for cost, but the fidelity you gave up must remain observable.** This reframes the tree's deepest current default, which is the actual bug — **wet is treated as sacred** (`ReadsLiveTree` never skips; wet is "truth," dry is suspect). Wet is just the highest-cost point on a fidelity curve, and it is *usually not worth it*. Fail-closed does not require always paying for wet; it requires that when you take the cheaper point, the deferred risk is **loud and detected, not ignored**. Deferred-and-detected is fail-closed; silently-skipped is fail-open. The current design conflates the two.

## What we have *right now* (the cost axis is bootstrappable)

Cost is the tractable axis, and in a closed bounded DAG (§4) it is ultimately **derivable from the code itself**. Today we already have most of the inputs:

- **Expected internal work is inspectable.** A claim declares its inputs and expected output (the claim's lhs/rhs nodes), so the *shape and size* of the computation it drives is a node-graph fact, not a mystery.
- **Effects are enumerable.** Which service ops a claim invokes (file read / compute / network) and — via fold/loop bounds (`loop_bound_edge`, `DescentEvidence`) — *how many times*, are readable off the graph.
- **Per-effect latency expectations.** Each effect kind has a characteristic cost (a file read ~X, a network round-trip ~Y, pure compute ~ node-count). Attach a latency expectation per effect and cost estimates compose.
- **Measurement already exists.** The floor emits `wall_nanos`, cgroup peak, and resolve/eval nanos per claim. So cost can be a **measured receipt now** and migrate to **graph-derived** when the cost lens lands — no need to block on derivation.

## Missing pieces (highlight, don't paper over)

- **No cost model on effects** — latency expectations per effect kind are not declared anywhere.
- **Bounds, not always points** — where a fold/loop bound isn't statically known, cost is a *bound* not a point; fail-closed = unknown bound counts as expensive, never as cheap.
- **No explicit floor budget** — the 270-min CI timeout is an implicit, wrong budget; §1's finite time should be a declared number the admission decision optimizes against.
- **Benefit is unmodeled** — no way yet to denominate a claim's value (displaced cost, §6). Two footholds only: (1) a claim with **no discriminating red** has provably ~0 gate-value (a §5 lower bound — this is what a pure "demo" is, and it should be *dropped*, not cadenced); (2) **frequency-of-red is observable** (history / the falsifier record it) — a claim that has never fired and guards nothing ranks low. The rest stays a declared estimate.
- **Proxies not yet reconciled** — `ReadsLiveTree`, `WitnessKind`, `TestClassification`, and the `manual/` naming convention are three-plus uncoordinated carriers (§3 fork cruft) that should become *derived readings* of the axes, not primitives.

## Sequencing

1. **Model cost first** (compute / resources), starting from what's derivable/measurable now (measured `wall_nanos` receipt → graph-derived estimate). Make the floor **budget-aware**, and make every claim the budget declines to run a **counted, typed deferral** carrying its cost and cadence — never a silent skip.
2. **Benefit second** — the harder axis. Start with the §5 lower bound (no-red ⟹ drop) and observed red-frequency; leave the rest a declared interim until it can be grounded.
3. **Reconcile the proxies** — once cost (and eventually benefit) are first-class, affected-set / wet-dry / cadence / test-kinds collapse into derived readings of one optimization, and the qualitative vocabulary is deleted rather than extended.

The north star: the run / defer / drop decision *derives* from cost, risk, and benefit under a budget, with any traded-away risk kept observable — so it stops being re-argued by hand every time.
