# R1 Testgen Manager Brief

## Orient before reading

- Product direction: [PR #672](https://github.com/gunb-ai/gunbc/pull/672)
  — `docs/thesis/compositional-modeling.md`. This manager's slice
  is load-bearing for the story doc's "tests fall out of the
  modeling" claim. Testgen is the runner that makes the release
  gates (including all the `[ext]` gates from the other four
  managers) actually evaluate as `.dag` programs. Lens API is the
  user-authored counterpart — letting consumers of the language
  extend the evaluation surface without compiler patches.
- Coordination context: [R1 Director Brief](r1-director-brief.md).
- Scope authority: [`THESIS.md`](../../THESIS.md) +
  [`ROADMAP.md`](../../ROADMAP.md). This brief does not author R1
  scope; it sequences and coordinates what those docs already name.

## Slice

This manager owns two lanes:

- **`T-TestGen`** (`ROADMAP.md:51`) — testgen runner, service
  simulation, first-class `TestClaim`. Size **L**. DB-15 follow-up
  (`ROADMAP.md:235`).
  - `testgen_structural_coverage` — `[ext]` gate.
  - `testgen_mock_backed_integration_safe` — `[ext]` gate;
    requires `MockBackedInvariant` wiring.
  - `testgen_manual_claim_is_first_class` — `[ext]` gate.
- **`T-LensAPI`** (`ROADMAP.md:52`, lens capability honesty pass
  at `:333`) — user-authored lenses + composition. Size **M-L**.
  - `user_authored_lens_compiles` — `[Day 1]` gate.
  - `lens_composition_associative` — `[ext]` gate; requires
    `AlgebraicLaw` predicate.
  - `lens_output_is_queryable_data` — `[ext]` gate.

## Framing question this manager answers

**Can the release gates evaluate as `.dag` programs end-to-end —
schema runner executes each predicate, `MockBackedInvariant` wires
service simulation, and user-authored lenses compose so the
evaluation surface is itself first-class `.dag`?**

Today:
- DB-15 schema landed (`ROADMAP.md:235`); generated runner
  execution remains follow-up. Translation: you can write a
  `TestClaim` in `.dag` and it compiles; you just can't yet
  *evaluate* it without the runner.
- The majority of R1 release gates are `[ext]` — they compile only
  after T-TestGen's schema extensions land (`ROADMAP.md:59`). That
  makes this lane the gate-enabling lane for R1.
- `MockBackedInvariant` service simulation is not yet wired.
- User-authored lenses (T-LensAPI) have DAY-1 compile support but
  composition / `AlgebraicLaw` / queryable-output are `[ext]`.

The ask: land the runner, wire service simulation, land lens
composition so the release-gate majority evaluates and user lenses
compose. When this closes, the R1 release gates themselves
(authored by other managers) become checkable.

## Sequence + dispatch

- **Day 1.** T-LensAPI `user_authored_lens_compiles` dispatches.
  `[Day 1]` gate; no schema extension needed. This gets user-
  authored lens compilation working early so consumers can start
  writing lenses while the evaluation surface is built.
- **Day 1.** T-TestGen runner foundation dispatches. Landing order
  for the three sub-deliverables is a lane-owner decision; the
  manager-level sequence is: runner first (so predicates have
  *something* to run against), then structural coverage, then
  mock-backed integration.
- **As runner lands.** `testgen_structural_coverage` predicate is
  the first to become evaluable. This evaluates "did testgen
  enumerate the structural transitions at a boundary?" — the
  claim from story-doc Part 6.
- **After runner foundation.** `testgen_mock_backed_integration_safe`
  dispatches. Requires `MockBackedInvariant` wiring — the
  non-trivial piece.
- **After runner foundation.** `testgen_manual_claim_is_first_class`
  dispatches. Manually-authored `TestClaim` values evaluate
  alongside generated ones.
- **Parallel (LensAPI).** `lens_composition_associative` requires
  `AlgebraicLaw` predicate in the T-TestGen schema extension.
  Coordinate internally so the two lanes land the shared extension
  once.
- **Parallel (LensAPI).** `lens_output_is_queryable_data`
  dispatches independently; don't block on compositional work.

## Hand-off points

- **Sideways to Self-hosting Manager.** T-TestGen runner readiness
  is the hand-off signal for T-PB-B landing. Self-hosting drafts
  `.dag` `TestClaim` declarations during Day-1, waits on this
  signal, then converts pipeline / contract tests. Notify
  Self-hosting when the runner lands.
- **Sideways to Surface Manager.** `emit_omni_demo_fixtures_green`
  and the three `emit_*` gates under T-Emit require `ExecuteCommand`
  + `ForAllTargets` predicates — these are T-TestGen `[ext]`
  extensions. Coordinate predicate shape with Surface early so
  T-Emit lane-owners aren't blocked.
- **Sideways to Release Manager.** T-Demo's `fixture_compiler_nerd_canonical`
  gate includes lens-output demos (`[ext]` on lens-output) — that's
  a T-LensAPI `lens_output_is_queryable_data` consumer. Notify
  Release when that lane lands.
- **Sideways to Substrate Manager.** When E-P lands (Substrate
  manager's sub-lane E-P), v3 cells produce `SubValueRelation`
  per-call. That's what `testgen_structural_coverage` should
  evaluate against — v3 authority rather than v2 oracle.
  Coordinate with Substrate on that cutover once E-P is near
  closure.
- **Up to director.** Any schema extension that looks like it
  needs to grow beyond what `ROADMAP.md:59` lists (new predicate
  kind, new `TestClaim` shape) is a scope question. Flag for
  director before landing.

## Working state

Lane-owner dispatch status (update as sub-deliverables close):

**T-TestGen:**
- [x] Schema extensions landed — `ExecuteCommand`, `ForAllTargets`,
      `LensOutputEquals`, `DifferentialEquals`, `AlgebraicLaw` variants
      added to `TestPredicate` (PR #678, merged 2026-04-24)
- [x] Runner foundation — schema predicates execute structurally
      (PR #688, merged 2026-04-24)
- [ ] `testgen_structural_coverage` gate compiles + evaluates
- [ ] `MockBackedInvariant` wiring
- [ ] `testgen_mock_backed_integration_safe` gate compiles + evaluates
- [x] `testgen_manual_claim_is_first_class` gate compiles + evaluates
      (`src/v3/std/r1_gates.dag` + `r1_manual_claim_gate_test.rs`)

**T-LensAPI:**
- [ ] `user_authored_lens_compiles` gate (Day-1) passes
      (PR #679 all checks green, merge pending)
- [ ] `AlgebraicLaw` predicate (schema extension, shared with
      lens_composition_associative) — predicate declared in #678;
      runner evaluation not yet wired
- [ ] `lens_composition_associative` gate compiles + evaluates
- [ ] `lens_output_is_queryable_data` gate compiles + evaluates

**Schema extensions owned here that other managers consume:**
- [x] `ExecuteCommand` predicate (Surface T-Emit consumer) — landed PR #678
- [x] `ForAllTargets` predicate (Surface T-Emit consumer) — landed PR #678
- [x] `LensOutputEquals` predicate (Substrate T-LaneE consumer
      — `complexity_merge_sort_is_nlogn`) — landed PR #678
- [x] `DifferentialEquals` predicate (Substrate T-LaneE consumer
      — `complexity_v3_matches_v2_oracle`) — landed PR #678

Decisions log (append as they happen):

- 2026-04-24: `ForAllTargets` self-referential variant dissolved to
  `{ command, args, expect_exit_code }` to preserve bounded-kernel
  invariant (Node is the only recursive type).

Open questions for director:

- _(none today)_

Cross-manager notifications queued:

- **Surface Manager**: `ExecuteCommand` + `ForAllTargets` predicates
  are on main (PR #678). T-Emit lane-owners can now author gates
  against those predicate shapes.
- **Substrate Manager**: `LensOutputEquals` + `DifferentialEquals`
  predicates are on main (PR #678). T-LaneE consumers can reference
  them once the runner lands.
- **Self-hosting Manager**: Runner foundation landed (PR #688,
  2026-04-24). T-PB-B is unblocked — begin `.dag` TestClaim
  conversion of pipeline / contract tests. ⬅ **SEND NOW**
