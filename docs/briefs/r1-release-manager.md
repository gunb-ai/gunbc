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
  `ROADMAP.md:362` (Duration/Money unit-mismatch enforcement
  consumer) or `ROADMAP.md:363` (`Secret<T>` nominal-wrapper
  graduation), escalate so R1 can be re-scoped or the row can be
  dispatched out of post-R1 ordering.

## Gap rows relevant to this slice

Both story-surfaced gaps now have ledger rows; this manager
monitors them as post-R1 follow-ups:
- **Unit-mismatch enforcement for typed value wrappers with phantom Unit / Currency parameters** (Duration/Money) — `ROADMAP.md:362`.
- **`Secret<T>` nominal-wrapper graduation** — `ROADMAP.md:363`.

Per the doc-authority single-ledger rule
([`doc-authority.md`](../thesis/doc-authority.md)), the ROADMAP
rows are the authority; this brief does not duplicate their
content.

## Working state

Lane-owner dispatch status (update as sub-deliverables close):

**T-Demo:**
- [x] `fixture_compiler_nerd_canonical` — Compiles (Day-1) **stage-(a)**
      landed in #686 (fixture-declaration compiles).
- [x] `fixture_compiler_nerd_canonical` — Compiles **stage-(b)** landed
      in #705 via `TestRunner::run_suite` (runner foundation from #688).
      Day-1 Compiles is now `[live]` end-to-end for this suite.
- [ ] `fixture_compiler_nerd_canonical` — lens-output demos evaluate
      (after T-LensAPI `lens_output_is_queryable_data`).
- [x] `fixture_integration_canonical` — Compiles (Day-1) **stage-(a)**
      landed in #686. Scope per `ROADMAP.md:71`: effects / idempotency /
      testgen.
- [x] `fixture_integration_canonical` — Compiles **stage-(b)** landed
      in #705 via `TestRunner::run_suite`. Day-1 Compiles `[live]`
      end-to-end.
- [ ] `fixture_integration_canonical` — lens-output demos evaluate
      (after T-LensAPI; `idempotency.dag` itself is already COMPLETE per
      `docs/v3-lens-capability-register.md:42`).
- [ ] Impossible-bugs suite (`impossible_bug_class_suite_r1`,
      `ROADMAP.md:72`) — **parked upstream-of-gates**. Re-dispatch
      when the first `[live]` R1 bug-class proof row lands:
  - Testgen runner + `MockBackedInvariant` wiring
    (`ROADMAP.md:51`, `:65`, `:235`). **Runner foundation `[live]`**
    via #688 + #705. `MockBackedInvariant` **dispatch + schema
    wiring** landed in #722, but the runner still returns
    `ClaimResult::NotYetImplemented` for mock simulation
    (`eval_mock_backed_invariant` in
    `src/v3/compiler/src/test_runner.rs`, currently `:621`).
    Trigger remains
    **half-met** until the runner actually evaluates a mock-backed
    invariant claim, not just routes it.
  - Substrate T-LaneE (`complexity_merge_sort_is_nlogn` /
    `lane_e_bundled_witness_host_emit_parity`) — pending.
  - Surface T-Emit (`emit_omni_demo_fixtures_green`) — pending.
  - PR #689 closed as premature (docs/thesis/ is director-owned per
    `doc-authority.md`). Narrative holds until at least one upstream
    row is fully `[live]` for a concrete bug class.

**T-Receipts (continuous):**
- [x] Receipt PR cadence established — bundle 1 (#685), bundle 2 (#701,
      replaced closed #687 with `-ne` enforcement tightening).
- [x] CI ratchet audit bundled — landed in #701; meta-ratchet freezes
      exemption count at 43 (equality check: fails on growth OR
      deletion-without-floor-drop). Follow-ups still open (fresh CI
      timings, per-exempt budgets, exemption deletions).
- [x] Stale-brief sweep in `docs/briefs/` — partial (#685).
- [x] INVARIANTS cross-ref cleanup — partial (#685).
- [ ] Scheduled-deletion work — bundled.
- [ ] `src/v3/compiler/tests/integration/common/mod.rs:319`
      narrow-scanner debt. Pre-existing: the scanner "deliberately does
      not model Rust character literals," forcing downstream test
      authors (e.g., the T-Demo integration tests at #686 / #705) into
      byte-constant workarounds like `DAG_ESCAPE_BYTE` /
      `DAG_QUOTE_BYTE`. Dissolution trigger: widen the scanner to model
      Rust char literals, or replace the scan with a structural reader.
      Tracked as a follow-up T-Receipts bundle row in `ROADMAP.md`
      (added in this brief-refresh PR).

**Dissolved in place (not ledger rows):**
Captures debt that was queued as a T-Receipts ledger row but whose
explicit dissolution trigger fired before the row was written. Not a
placeholder; a pattern for preventing ledger bloat when the queued
work dissolves within the same wave that surfaced it.

- T-Demo `.dag` text-slicing bridge (`find_string_field` /
  `read_dag_string_literal`) introduced in #686 dissolved in #705 when
  `TestRunner::run_suite` became the structurally-correct evaluator.
  Was queued as a T-Receipts row; never needed to be written down.

**Post-R1 gap monitoring (ROADMAP rows):**
- [ ] `ROADMAP.md:362` unit-mismatch enforcement — flag if R1 scope
      expands to depend on it
- [ ] `ROADMAP.md:363` `Secret<T>` nominal-wrapper — flag if R1
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
- **2026-04-24** — T-Receipts bundle 1 landed (#685); merge order per
  director for bundle 2 (#687, slow-test-exemption meta-ratchet):
  #685 → #687.
- **2026-04-24** — #687 closed and re-authored as #701 (same branch +
  `-ne` enforcement tightening). #701 landed. Process note logged on
  #701: future tightening should force-push onto the open PR rather
  than close-and-reopen.
- **2026-04-24** — T-Demo Day-1 Compiles `[live]` end-to-end for both
  canonical fixtures: stage-(a) in #686, stage-(b) in #705 via
  `TestRunner::run_suite`. Text-slicing bridge introduced in #686
  dissolved in place in #705 (runner evaluation fired its dissolution
  trigger). One receipt row dissolved before it needed to be written.
- **2026-04-24** — Impossible-bugs suite re-dispatch trigger "Testgen
  runner + MockBackedInvariant wiring" is now **half-met**: runner
  foundation from #688 + T-Demo consumer proof from #705.
  `MockBackedInvariant` wiring for idempotency-class proofs is the
  remaining piece. Notify Director when the other half lands so W3 can
  re-dispatch against honest claims.
- **2026-04-24** — `MockBackedInvariant` dispatch + schema wiring
  landed in #722, but runner mock simulation is still
  `NotYetImplemented` (`eval_mock_backed_invariant` in
  `test_runner.rs`). Initial brief refresh
  in #743 overstated this as "both halves live"; corrected after
  Codex review on #743 flagged P1 (Documentation Describes Live
  State). Trigger remains half-met until evaluation lands, not just
  dispatch.

Open questions for director:

- _(none yet — the two story-surfaced gaps are tracked as post-R1
  rows at `ROADMAP.md:362-363`; escalate only if R1 scope expands
  to depend on either)_

Cross-manager notifications queued:

- _(receive signals from all four other managers as their
  deliverables unblock demo-fixture evaluation)_
