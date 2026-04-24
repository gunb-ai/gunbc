# R1 Release Manager Brief

## Orient before reading

- Product direction: [PR #672](https://github.com/gunb-ai/gunbc/pull/672)
  — `docs/thesis/compositional-modeling.md`. This manager's slice
  is where the story stops being internal grounding and starts
  being something you can point at: a realistic end-to-end
  workflow whose code size, error-class coverage, and testgen
  output *actually match* what the story doc claims at R1 scope.
  Also owns the continuous debt-paydown discipline so nothing
  upstream drifts while the other four managers work.
- Coordination context: [R1 Director Brief](r1-director-brief.md).
- Scope authority: [`THESIS.md`](../../THESIS.md) +
  [`ROADMAP.md`](../../ROADMAP.md). This brief does not author R1
  scope; it sequences and coordinates what those docs already name.

## Slice

This manager owns two lanes (one scoped, one continuous):

- **`T-Demo`** (`ROADMAP.md:55`) — two canonical fixtures +
  impossible-bugs suite + narrative. Size **M**.
  - `fixture_compiler_nerd_canonical` — `[Day 1 (Compiles)]` +
    `[ext]` for lens-output demos. Demonstrates: complexity,
    ownership, parallelism.
  - Additional fixture(s) per `ROADMAP.md:55+` (flesh out from
    lane authority).
  - Impossible-bugs narrative — the part that makes the reward-
    structure pitch concrete for a reader.
- **`T-Receipts`** (continuous track, `ROADMAP.md:28`) — debt
  paydown continues in parallel. R1 does not freeze the tracked-
  debt ledger. Standing preference: bundle 2–4 items per PR. CI
  ratchet audit, stale-brief sweep, INVARIANTS cross-ref cleanup,
  scheduled-deletion work.

## Framing question this manager answers

**Does the realistic end-to-end workflow compose the other
managers' deliveries convincingly (the "scale" example from Part 7
of the story doc), and are tracked debts continuously paid down
without blocking any gate?**

Today:
- Story-doc Part 7 describes a five-service integration
  (`docs/thesis/compositional-modeling.md`), tagged `[target]` with
  composite gap pointers. R1's T-Demo scope is narrower than a
  five-service demo; the lane authority at `ROADMAP.md:55` defines
  the two canonical fixtures in play.
- The impossible-bugs-suite claim from the story doc is partially
  a T-Demo deliverable (narrative + fixtures) and partially a
  composition-of-other-managers' deliveries (the bug classes
  themselves are prevented by their lens / compiler / testgen
  work).
- Debt ledger at `ROADMAP.md:248-400+` has ongoing rows; T-Receipts
  bundles them into continuous PRs. Receipt volume is a drift
  indicator — if it spikes, upstream managers are producing
  receipts faster than the continuous track absorbs.

The ask: land a demo that a principal engineer can read in one
sitting and say "yes, this is what they claim, and here's the
evidence." Keep debt paydown flowing so nothing upstream rots
while the other managers ship.

## Sequence + dispatch

- **Day 1.** T-Receipts dispatches immediately. Bundle the
  existing tracked-debt ledger rows into continuous PRs at 2–4
  items per PR. The standing preference is explicit.
- **Gated on other managers, T-Demo fixture drafting.** Start
  drafting the `.dag` fixture declarations for the two canonical
  fixtures in parallel with other managers' work. Some predicates
  compile Day-1 (`[Day 1 (Compiles)]` gates); the lens-output
  demos are `[ext]` on T-LensAPI's `lens_output_is_queryable_data`
  closure.
- **Gated on Testgen Manager.** `fixture_compiler_nerd_canonical`
  lens-output predicates evaluate after T-LensAPI `lens_output_is_queryable_data`
  lands. Don't block early draft work; just land evaluation when
  ready.
- **Gated on convergence.** Final T-Demo landing waits on
  Substrate (lens authority for complexity / parallelism /
  ownership demos), Self-hosting (fixed-point self-compile so the
  demo runs in the shipped binary), Surface (emission green across
  targets so demo fixtures compile under external toolchains),
  and Testgen (runner so gate predicates evaluate).
- **Impossible-bugs narrative.** This is the story-doc-adjacent
  piece. Can be drafted conversationally once the other managers
  name which bug classes are actually `[live]` at R1. Hold the
  narrative until those claims are honest — no "stop existing"
  language about bugs that still exist.

## Hand-off points

- **Up the chain, continuously, from all managers.** Receipt
  candidates from other managers surface to Release for bundling.
  Receipts aren't scope questions; they're bookkeeping. The
  director promotes to ledger; this manager bundles landed
  receipts into PRs.
- **Sideways from Substrate Manager.** E-P closure enables
  lens-authority complexity / cost / parallelism demos. The demo
  fixtures showcase those lenses once they compile against v3
  authority.
- **Sideways from Surface Manager.** T-Emit omni-target green is
  the precondition for demo fixtures running under external
  toolchains.
- **Sideways from Self-hosting Manager.** T-PB-A fixed-point
  closure is what lets the demo binary be the shipped compiler.
  T-PB-B tests-as-data convert the demo's own tests into `.dag`
  data (meta-demo).
- **Sideways from Testgen Manager.** T-LensAPI
  `lens_output_is_queryable_data` enables lens-output evaluation
  in demo fixtures. T-TestGen runner evaluates all gate predicates
  in the demo.
- **Up to director.** Any proposed T-Demo scope expansion (third
  fixture, five-service integration, etc.) routes to director.
  R1's T-Demo is explicitly scoped to two canonical fixtures +
  impossible-bugs suite per `ROADMAP.md:55`.
- **Up to director.** If at any point R1 scope expands to depend
  on either of the two story-surfaced gaps tracked at
  `ROADMAP.md:364` (Duration/Money unit-mismatch enforcement
  consumer) or `ROADMAP.md:365` (`Secret<T>` nominal-wrapper
  graduation), escalate so R1 can be re-scoped or the row can be
  dispatched out of post-R1 ordering.

## Gap rows relevant to this slice

Both story-surfaced gaps now have ledger rows; this manager
monitors them as post-R1 follow-ups:
- **Unit-mismatch enforcement for typed value wrappers with phantom Unit / Currency parameters** (Duration/Money) — `ROADMAP.md:364`.
- **`Secret<T>` nominal-wrapper graduation** — `ROADMAP.md:365`.

Per the doc-authority single-ledger rule
([`doc-authority.md`](../thesis/doc-authority.md)), the ROADMAP
rows are the authority; this brief does not duplicate their
content.

## Working state

Lane-owner dispatch status (update as sub-deliverables close):

**T-Demo:**
- [ ] `fixture_compiler_nerd_canonical` — Compiles (Day-1) — in flight on
      PR #686 (W2 / quiet-lynx-432); draft, revising per manager
      feedback (real source, not placeholders; fixture 2 pending)
- [ ] `fixture_compiler_nerd_canonical` — lens-output demos evaluate
      (after T-LensAPI)
- [ ] `fixture_integration_canonical` — Compiles (Day-1). Scope per
      `ROADMAP.md:71`: effects / idempotency / testgen. Staffed under
      PR #686 as the second canonical fixture (not drafter's-choice —
      director clarified `ROADMAP.md:70-73` names both fixtures
      explicitly).
- [ ] `fixture_integration_canonical` — lens-output demos evaluate
      (after T-LensAPI + live idempotency lens)
- [ ] Impossible-bugs suite (`impossible_bug_class_suite_r1`,
      `ROADMAP.md:72`) — **parked upstream-of-gates**. Re-dispatch when
      the first `[live]` R1 bug-class proof row lands: Substrate
      T-LaneE (`complexity_merge_sort_is_nlogn` /
      `complexity_v3_matches_v2_oracle`), Testgen + live idempotency
      lens (currently STUB per `ROADMAP.md:333`), or Surface T-Emit
      (`emit_omni_demo_fixtures_green`). PR #689 closed as premature
      (docs/thesis/ is director-owned per `doc-authority.md` — tracking
      scaffolding is not the lane deliverable). Narrative holds until
      at least one upstream row is honest.

**T-Receipts (continuous):**
- [ ] Receipt PR cadence established (2–4 items per PR)
- [ ] CI ratchet audit bundled
- [ ] Stale-brief sweep in `docs/briefs/` bundled
- [ ] INVARIANTS cross-ref cleanup bundled
- [ ] Scheduled-deletion work bundled

**Post-R1 gap monitoring (ROADMAP rows):**
- [ ] `ROADMAP.md:364` unit-mismatch enforcement — flag if R1 scope
      expands to depend on it
- [ ] `ROADMAP.md:365` `Secret<T>` nominal-wrapper — flag if R1
      scope expands to depend on it

Decisions log (append as they happen):

- **2026-04-24** — Director: W2 demo fixtures stay at
  `src/v3/compiler/tests/t_demo/`; demos are `TestClaim` evaluations per
  `ROADMAP.md:21-26`, not a new authority surface.
- **2026-04-24** — Director: second fixture is
  `fixture_integration_canonical` per `ROADMAP.md:71`, not
  drafter's-choice.
- **2026-04-24** — Director: `docs/thesis/` edits (impossible-bugs
  narrative) are director-owned per `doc-authority.md` (PR #672). W3
  parked upstream-of-gates; re-dispatch on first R1 `[live]` proof row.
- **2026-04-24** — T-Receipts bundle 1 landed (#685); bundle 2 (#687,
  slow-test-exemption meta-ratchet) green-lit pending rebase on #685.

Open questions for director:

- _(none yet — the two story-surfaced gaps are tracked as post-R1
  rows at `ROADMAP.md:364-365`; escalate only if R1 scope expands
  to depend on either)_

Cross-manager notifications queued:

- _(receive signals from all four other managers as their
  deliverables unblock demo-fixture evaluation)_
