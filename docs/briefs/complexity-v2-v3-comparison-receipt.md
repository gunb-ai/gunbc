# Complexity lens v2/v3 comparison + cementing test `(M)`

**Supersedes** `docs/briefs/lens-wire-in-audit-v2-comparison.md` (which was broader-audit-shaped). This is the focused thesis-receipt lane.

## Problem

The v3 complexity lens is structurally landed but has no receipt demonstrating it matches or beats v2's heuristic approach, and nothing *cements* its current output so regressions are caught.

Two core gaps plus an adjacent cosmetic:

1. **No thesis receipt.** v3 `src/v3/lenses/complexity.dag` is 162 lines (🟢 TERMINAL, pure fold over `d.nodes`, fail-closed on malformed refs). v2 `src/v2/complexity.dag` is 5488 lines of heuristics, the majority of which (per `feedback_variant_provenance_shape`) is bridge code rebuilding facts dropped upstream. That's a ~13× LOC reduction with structurally stronger guarantees, but nothing in the repo demonstrates it concretely against the same input.

2. **No cementing test for v3 output.** Integration tests exist (`m1_3_lens_cost_test.rs`, `m2_lens_cost_migration_test.rs`) but they test the lens runs correctly — not that its specific output on a chosen fixture is locked in. An edit to `complexity.dag` that silently changes output would not trip a golden.

3. **Adjacent cosmetic (not a core gap): registry name mismatch.** `src/v3/compiler/regen.dag:34-38` declares `lens_cost_entry` whose `lens_file` is `src/v3/lenses/complexity.dag`. The "cost" registry name vs `complexity.dag` source file is confusing. Similarly, `cost.dag` is registered as `"cost_symbolic"`. Each future reader has to rediscover this. Phase 1 and Phase 2 have **no structural dependency** — the worker should land them as separate PRs. Phase 2 is the load-bearing thesis receipt; bundling the cosmetic rename risks the churn obscuring the diff artifact. If scope pressure is high, skip Phase 1 entirely and do it later as an independent micro-lane.

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

Prefer (a) — the confusion is ongoing debt. The rename surface is bounded (registry + generated file + consumers) and should be grep-able. Rename churn that stays under ~10 files is expected and worth eating; only if the consumer surface exceeds that and the rename is blowing up scope should the worker switch to (b) or STOP-AND-ESCALATE. Three files is well within the "do it" zone.

### Phase 2 — Comparison + cementing test `(M)`

1. **Pick a fixture.** Criteria, in priority order:
   - **Representative first** — a `.dag` file with at least one recursive function, one fold, and one simple arithmetic function. Representativeness is the load-bearing property; a purpose-built fixture that hits the right shapes is strictly better than a convenient one that doesn't.
   - Reproducible by v2 — has to be parseable by v2 stage0 so v2 complexity can produce output
   - Small — keep the fixture tight so the diff is readable
   - Reused only if it satisfies the above — if an existing test fixture happens to be representative, reuse is a nice-to-have. Do not couple the receipt to whatever fixture was convenient.
   - Candidates: a fixture from `dsl/examples/` (if representative), or a synthetic one purpose-built for this lane

2. **Capture v3 output as a golden.** Run v3 complexity lens on the fixture; serialize the per-port cost map (or equivalent structural output) to a golden file checked into the repo. Add an integration test that re-runs the lens and asserts **semantic equality** against the golden — parse both into the same typed carrier (e.g., `Map<PortId, CostLookup>`) and compare the data structures, or canonicalize both serializations (sort by `PortId`, normalize whitespace) before byte-compare. **Do not** assert raw byte-identity against a free-form serialization: formatting or iteration-order changes would produce false regressions that pin representation rather than behavior (TESTING.md prefers semantic carriers over representation pinning). This is the cementing test — any future edit to `complexity.dag` that changes *structural output* will trip this test, and the worker must either update the golden (if intentional) or fix the regression (if not).

3. **Capture v2 output programmatically.** Run v2 stage0 `complexity` on the same fixture programmatically and capture output into a golden file; a second integration test asserts v2's output against that golden via the same semantic-equality discipline. Both sides are cemented — any drift on either side surfaces as a test failure.

   **Cementing symmetry is load-bearing.** The thesis receipt is only meaningful if both sides are pinned to a re-executable baseline. Transcribed prose that doesn't re-execute is not a substitute — it introduces asymmetric cementing (v3 cemented, v2 frozen prose that could drift silently) and defeats the point of the receipt. If v2 stage0 has no programmatic surface that allows running complexity on an arbitrary fixture, **STOP-AND-ESCALATE**: that's its own finding (v2 observability gap), not a fallback this lane can absorb.

4. **Document the diff in the PR body.** Three possible outcomes:
   - **Identical (up to output-shape translation)**: thesis receipt. PR body shows both outputs side-by-side, notes the 13× LOC reduction, closes as a post-merge debt *closure* in ROADMAP.
   - **v3 produces richer or more precise output** (e.g., explicit fail-closed on malformed refs where v2 returns a heuristic guess): even better. PR body documents the specific delta.
   - **v3 misses something v2 catches**: specific gap to close. Each case is either a v3 modeling omission (fixable structurally) or a case where v2's heuristic was wrong but happened to work on this input. Each surface finds a separate sub-lane — not solved in this PR.

   PR body frames the finding explicitly — don't let the comparison land as prose-only; the specific diff (or equivalence) is the receipt.

## Acceptance

### Phase 1 (ship as a separate PR from Phase 2)
- Registry entry name matches its source file (or is docstringed if (b) was taken)
- Generated file name is consistent
- Consumer imports updated
- No test breakage from the rename

### Phase 2 (the thesis-receipt PR)
- One DSL fixture checked in, representative (recursion, fold, arithmetic), reachable programmatically by both v2 and v3 complexity analyses
- Golden file for v3 complexity output checked in
- Integration test asserts v3 output matches the golden via **semantic equality** — parse both into the same typed carrier and compare the data structure, or canonicalize both serializations (sort by `PortId`, normalize whitespace) before byte-compare. Raw byte-identity against a free-form serialization is explicitly disallowed (pins representation, not behavior).
- Golden file for v2 complexity output on the same fixture checked in; **second** integration test asserts v2 output against its golden via the same semantic-equality discipline. Both sides cemented symmetrically; transcribed-prose fallbacks are disallowed.
- PR body contains an explicit **diff framing**: "identical", "v3 richer", or "v3 misses X" — pick one and justify it

## STOP-AND-ESCALATE

- **If Phase 1's rename pulls in a wide consumer surface** (more than ~10 files), switch to option (b) (docstring-only) rather than forcing a large rename. Below ~10 files the rename is expected scope; above it the cosmetic shouldn't dominate.
- **If v2 stage0 has no programmatic surface for running complexity on an arbitrary fixture**, STOP. Asymmetric cementing (v3 cemented vs v2 transcribed-prose) defeats the receipt; surface the v2-observability gap as a separate lane rather than absorbing it here.
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
