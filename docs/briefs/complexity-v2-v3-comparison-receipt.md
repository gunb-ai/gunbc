# Complexity lens v2/v3 comparison + cementing test `(M)`

**Supersedes** `docs/briefs/lens-wire-in-audit-v2-comparison.md` (which was broader-audit-shaped). This is the focused thesis-receipt lane.

## Problem

The v3 complexity lens is structurally landed but has no receipt demonstrating it matches or beats v2's heuristic approach, and nothing *cements* its current output so regressions are caught.

Two specific gaps:

1. **No thesis receipt.** v3 `src/v3/lenses/complexity.dag` is 162 lines (🟢 TERMINAL, pure fold over `d.nodes`, fail-closed on malformed refs). v2 `src/v2/complexity.dag` is 5488 lines of heuristics, the majority of which (per `feedback_variant_provenance_shape`) is bridge code rebuilding facts dropped upstream. That's a ~13× LOC reduction with structurally stronger guarantees, but nothing in the repo demonstrates it concretely against the same input.

2. **No cementing test for v3 output.** Integration tests exist (`m1_3_lens_cost_test.rs`, `m2_lens_cost_migration_test.rs`) but they test the lens runs correctly — not that its specific output on a chosen fixture is locked in. An edit to `complexity.dag` that silently changes output would not trip a golden.

3. **Adjacent cosmetic: registry name mismatch.** `src/v3/compiler/regen.dag:34-38` declares `lens_cost_entry` whose `lens_file` is `src/v3/lenses/complexity.dag`. The "cost" registry name vs `complexity.dag` source file is confusing. Similarly, `cost.dag` is registered as `"cost_symbolic"`. Each future reader has to rediscover this.

## Read first

- `src/v3/lenses/complexity.dag` — the v3 authority (162 lines, pure catamorphism over `d.nodes`)
- `src/v3/lenses/cost.dag` — the symbolic-cost sibling (258 lines, different lens, related naming)
- `src/v3/compiler/regen.dag:34-62` — the registry with the "cost" vs `complexity.dag` name collision
- `src/v3/compiler/src/lens_cost_generated.rs` — generated projection
- `src/v3/compiler/tests/integration/m1_3_lens_cost_test.rs` — existing integration test (shape reference)
- `src/v2/complexity.dag` — v2 heuristic authority for comparison
- `src/v2/stage0/src/v2_compiler_complexity.rs` (or equivalent) — v2 stage0 Rust binding (for invoking v2 complexity on a fixture)
- `feedback_variant_provenance_shape` memory — context on why v2's complexity.dag is mostly bridge code
- INVARIANTS.md §P1 worked example on termination via descent evidence — P1 Modeling Faithfulness framing

## Work — two phases, Phase 2 is the payoff

### Phase 1 — Registry-name cleanup `(XS)`

Rename the registry entry for clarity. Options:

**(a) Rename the entry and generated file** to match the source:
- `lens_cost_entry` → `lens_complexity_entry`
- `name: "cost"` → `name: "complexity"`
- `lens_cost_generated.rs` → `lens_complexity_generated.rs`
- Update `m1_3_lens_cost_test.rs` → `m1_3_lens_complexity_test.rs` if the test name mirrors
- Update consumer imports (`use v3_compiler::lens_cost_generated::*;` etc.)

**(b) Keep the names but docstring the historical reason** in `regen.dag`. Cheaper if (a) is wide.

Prefer (a) — the confusion is ongoing debt. The rename surface is bounded (registry + generated file + consumers) and should be grep-able. Switch to (b) only if the consumer surface turns out to be wider than expected (STOP-AND-ESCALATE).

### Phase 2 — Comparison + cementing test `(M)`

1. **Pick a fixture.** Criteria:
   - Small but representative — a `.dag` file with at least one recursive function, one fold, and one simple arithmetic function
   - Reproducible by v2 — has to be parseable by v2 stage0 so v2 complexity can produce output
   - Already used by test infrastructure if possible — look at what existing complexity tests consume
   - Candidates: a fixture from `dsl/examples/` (simple, small), or a synthetic one purpose-built for this lane

2. **Capture v3 output as a golden.** Run v3 complexity lens on the fixture; serialize the per-port cost map (or equivalent structural output) to a golden file checked into the repo. Add an integration test that re-runs the lens and asserts byte-equality against the golden. This is the cementing test — any future edit to `complexity.dag` that changes output will trip this test, and the worker must either update the golden (if intentional) or fix the regression (if not).

3. **Capture v2 output for comparison.** Either:
   - **(a) Strongly preferred:** Run v2 stage0 `complexity` on the same fixture programmatically and capture output into a golden file; a second integration test asserts v2's output against that golden. Both sides are cemented — any drift on either side surfaces as a test failure.
   - **(b) Fallback only:** if (a) is genuinely un-wireable (no CLI surface, hermetic test harness missing), run v2 once manually and transcribe output into a comparison fixture file under `docs/history/` or `docs/perf/`. **This is a mild fail-open** — transcribed prose doesn't re-execute, so v2 could drift silently while v3 stays cemented. Option (b) must be justified explicitly in the PR body with the specific technical blocker that made (a) infeasible.

   Cementing symmetry is load-bearing: the thesis receipt is only meaningful if *both* sides are pinned to a re-executable baseline. (b) breaks the symmetry, so the bar for taking (b) should be high.

4. **Document the diff in the PR body.** Three possible outcomes:
   - **Identical (up to output-shape translation)**: thesis receipt. PR body shows both outputs side-by-side, notes the 13× LOC reduction, closes as a post-merge debt *closure* in ROADMAP.
   - **v3 produces richer or more precise output** (e.g., explicit fail-closed on malformed refs where v2 returns a heuristic guess): even better. PR body documents the specific delta.
   - **v3 misses something v2 catches**: specific gap to close. Each case is either a v3 modeling omission (fixable structurally) or a case where v2's heuristic was wrong but happened to work on this input. Each surface finds a separate sub-lane — not solved in this PR.

   PR body frames the finding explicitly — don't let the comparison land as prose-only; the specific diff (or equivalence) is the receipt.

## Acceptance

### Phase 1
- Registry entry name matches its source file (or is docstringed if (b) was taken)
- Generated file name is consistent
- Consumer imports updated
- No test breakage from the rename

### Phase 2
- One DSL fixture checked in, reachable by both v2 and v3 complexity analyses
- Golden file for v3 complexity output checked in
- Integration test asserts v3 output matches the golden byte-identically (cementing)
- v2 complexity output on the same fixture is either checked in (as a comparison fixture) or documented in the PR body with the command used to reproduce
- PR body contains an explicit **diff framing**: "identical", "v3 richer", or "v3 misses X" — pick one and justify it

## STOP-AND-ESCALATE

- **If Phase 1's rename pulls in a wide consumer surface** (more than ~3 files), switch to option (b) (docstring-only) rather than forcing a large rename. The cosmetic shouldn't dominate the lane.
- **If no existing fixture parseable by both v2 and v3 can be found**, STOP. Building a synthetic fixture is fine but only if it's representative; otherwise the comparison isn't meaningful. Surface the mismatch as its own finding.
- **If v2 complexity output on the fixture reveals a bug in v2** (not just heuristic-imprecision), STOP. That's a v2 bug lane, not a v3 receipt lane.
- **If the comparison reveals v3 misses a case v2 catches that requires new substrate carriers** (not just new lens rules), STOP. That's a substrate-modeling lane — surface the specific missing carrier (e.g., "v2 distinguishes X which v3 substrate doesn't carry").

## Non-goals

- Not touching v2 complexity itself — v2 is the reference/oracle, not the work.
- Not wiring `idempotency.dag` or `parallelism.dag` — those are separate lenses with their own lanes.
- Not reorganizing `cost.dag` / the `"cost_symbolic"` entry — if renaming `complexity.dag`'s registry entry creates a second mismatch there, flag it but don't fix in this PR.
- Not adding new complexity analysis features to v3 — this is a receipt + cementing lane, not a lens-extension lane.

## Size

M. Phase 1 is XS (minutes of rename). Phase 2 is the bulk: fixture selection (~30 min), v3 golden capture + cementing test (~1h), v2 run on same fixture + PR body comparison (~1–2h depending on v2 harness). Total expected ~half-day to day.

## Dispatch note

This is a **thesis receipt lane**, not a bug fix. The primary value is the comparison artifact — if it lands cleanly, it's reference material for future "the algebra does the work" arguments. The cementing test is load-bearing: without it, a future edit to `complexity.dag` could silently regress the structural approach without anyone noticing.

Director reviews the PR body's comparison framing specifically — the v2/v3 diff narrative is the load-bearing artifact. How the worker frames the finding ("identical", "v3 richer", or "v3 misses X") is the signal.

If the comparison comes out "identical" or "v3 richer": ROADMAP debt entries for v2 complexity can close. If "v3 misses X": new sub-lanes open, each a specific substrate or lens gap to close.
