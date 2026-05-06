# Lens wire-in audit + v2/v3 complexity comparison `(M)`

> **Superseded by [`complexity-v2-v3-comparison-receipt.md`](complexity-v2-v3-comparison-receipt.md)** — that brief is the focused thesis-receipt lane. This doc remains for the broader three-phase audit (registry cleanup + v2/v3 comparison + orphaned lens wire-in for `idempotency` / `parallelism`); the supersession narrows scope to just the comparison + cementing test. Pick the narrower brief for dispatch.

## Problem

The v3 lens infrastructure is in a mixed state. Some lenses are fully registry-driven (`.dag` authority → `regen_lens` → generated Rust, with freshness ratchet tests); others are hand-written Rust with `.dag` siblings that aren't yet registered; others have no `.dag` authority at all. There's no single artifact that *demonstrates* the "algebra does the work" end-to-end or *compares* the v3 structural approach to v2's heuristic complexity analysis.

Two concrete questions to answer:
1. **Does v3's structural complexity lens produce the same (or better) output as v2's 5488-line heuristic `complexity.dag`?** If yes, it's a thesis receipt worth publicizing. If no, there's a gap to name.
2. **What's the cleanup path for the mid-migration lenses (idempotency, parallelism) whose `.dag` authorities exist but aren't wired through the registry?**

## Current wire-in state (inventory)

**Registry-driven (`.dag` → `regen_lens` → generated Rust, freshness-tested):**
- `complexity.dag` → `lens_cost_generated.rs` (registered under the name "cost" — misleading name)
- `cost.dag` → `lens_cost_symbolic_generated.rs` (registered as "cost_symbolic")
- `provenance.dag` → `lens_provenance_generated.rs`
- `structural_resolution.dag` → `lens_structural_resolution_generated.rs`
- `unused_parameters.dag` → `lens_unused_parameters_generated.rs`
- `infer_helpers.dag` → `infer_helpers_generated.rs`
- `variant_payload.dag` → `variant_payload_generated.rs`
- `lower_helpers.dag` → `lower_helpers_generated.rs`

**Hand-Rust with `.dag` sibling (partial wire-in, bridge state):**
- `idempotency.dag` + hand `lens_idempotency.rs`
- `parallelism.dag` + hand `lens_parallelism.rs`

**Hand-Rust only (pre-`.dag`-authority):**
- `lens_depth.rs`
- `lens_testgen.rs`

## Read first

- `src/v3/compiler/regen.dag` — lens registry (`LensRegistryEntry` rows)
- `src/v3/compiler/src/bin/regen_lens.rs` — the driver that reads the registry and emits generated Rust
- `src/v3/lenses/complexity.dag` — 162 lines, 🟢 TERMINAL, pure catamorphism over `d.nodes`. Source for `lens_cost_generated.rs` despite the registry name mismatch
- `src/v3/lenses/cost.dag` — 258 lines, sourced for the "cost_symbolic" registry entry
- `src/v3/compiler/tests/integration/m1_3_lens_cost_test.rs` — existing integration test for the v3 cost/complexity lens
- `src/v2/complexity.dag` — 5488 lines, heuristic approach (per `feedback_variant_provenance_shape`, the majority is bridge code reconstructing facts dropped upstream)
- INVARIANTS.md#p1-modeling-faithfulness Modeling Faithfulness — the "heuristics indicate lost structure" framing this lane is meant to demonstrate concretely

## Work — three phases, each landable independently

### Phase 1: Registry-name cleanup `(S)`

The "cost" registry entry sources `complexity.dag` and emits `lens_cost_generated.rs`. The name collision with `cost.dag` (which is the actual symbolic-cost source, registered as "cost_symbolic") is confusing.

Either:
- **(a)** Rename the entry to "complexity" and rename `lens_cost_generated.rs` → `lens_complexity_generated.rs`. Update consumers. Prefer this if the rename touches a bounded consumer surface.
- **(b)** Leave the registry name but add a docstring in `regen.dag` explaining the historical name collision and why the source file is `complexity.dag`. Cheaper if (a) is a large rename.

Phase 1 is a cleanup, not a prerequisite for Phase 2. If scope pressure is high, skip to Phase 2.

### Phase 2: v2 vs v3 complexity comparison `(M)`

**Goal:** pick a canonical DSL fixture (small, representative — ideally already used by v2 complexity tests), run both v2 and v3 complexity analyses on it, and compare structural output.

Concrete work:
1. Select a fixture. Candidates: a simple arithmetic function, a Node-recursive structure, a fold-over-list. Choose one whose v2 complexity output is already checked in so the comparison is reproducible.
2. Add an integration test that runs v3 complexity lens on the fixture and asserts the structural output (per-port cost map, or equivalent).
3. Either reuse an existing v2 stage0 binary to produce v2 complexity output on the same fixture, *or* encode the expected v2 output as a fixture file. The former is more defensible; the latter is cheaper.
4. Diff the two. Three outcomes:
   - **Identical** (up to representation) — thesis receipt. Document as a post-merge debt *closure* in ROADMAP.
   - **v3 produces richer output** (e.g., more precise bounds, explicit fail-closed on malformed refs where v2 would return a heuristic guess) — even better. Document the *delta* explicitly.
   - **v3 misses something v2 catches** — surface the specific case. The gap is either a v3 modeling omission (fixable by adding structural carriers) or a case where v2's heuristic was wrong but happened to handle this input. Each case is a separate sub-lane.
5. The integration test lives under `src/v3/compiler/tests/integration/` following the `m1_3_lens_cost_test.rs` pattern.

**Acceptance:** one fixture with both v2 and v3 complexity output captured; test asserts the v3 output structurally; diff vs v2 is documented (either "identical", or a specific named delta).

### Phase 3: Unwired `.dag` lens audit `(S–M, optional)`

For `idempotency.dag` and `parallelism.dag`:
1. Read each and verify it's a structurally-valid lens (TERMINAL, pure fold, fail-closed on malformed refs).
2. Add a `LensRegistryEntry` for each in `regen.dag` pointing at a `lens_idempotency_generated.rs` / `lens_parallelism_generated.rs` target.
3. Run `cargo run -p v3-compiler --bin regen_lens` to produce the Rust projection.
4. Verify the generated Rust matches the hand-written `lens_idempotency.rs` / `lens_parallelism.rs` (or diagnose the delta).
5. If they match, the hand-written Rust is now redundant — delete it and point consumers at the generated module.
6. If they don't match, document the delta as a separate dissolution item (the `.dag` authority might be incomplete, or the hand Rust does something the `.dag` doesn't model).

Phase 3 is not a prerequisite for Phase 2. Can land separately.

## Acceptance

- **Phase 1:** registry entry name resolves the "cost vs complexity" confusion. Either renamed (with consumer updates) or explicitly docstringed.
- **Phase 2:** one DSL fixture, v3 complexity output captured in a test, v2 complexity output captured (either by running v2 or as a checked-in fixture), diff documented.
- **Phase 3 (optional):** at least one of `idempotency` / `parallelism` wired into the registry; hand Rust deleted if it matches, or delta documented.

## STOP-AND-ESCALATE

- **If Phase 2 reveals v3 complexity misses cases v2 catches** that require *new substrate carriers* (not just new lens rules), STOP. That's a substrate-modeling lane, not a lens-wiring lane. Surface the specific missing carrier (e.g., "v2 complexity distinguishes X which v3 substrate doesn't carry").
- **If Phase 1's rename turns into a wide consumer-touching refactor**, prefer option (b) — docstring explanation — rather than forcing the rename.
- **If Phase 3 reveals that `idempotency.dag` or `parallelism.dag` is structurally incomplete** (the hand-Rust does real work that the `.dag` doesn't model), don't delete the hand Rust. Surface the gap as a separate `.dag`-authoring lane.

## Non-goals

- Not wiring `lens_depth.rs` or `lens_testgen.rs` — those have no `.dag` authority yet, so they'd need authoring work first.
- Not restructuring `complexity.dag` itself — the lens is already TERMINAL and pure. Work is downstream.
- Not fixing CI regen drift from #625 — that's a separate blocking issue.

## Size

M. Phase 1 alone is S. Phase 2 is the bulk (fixture selection, v2 bin execution or fixture transcription, integration test authoring, diff documentation). Phase 3 is S per lens if the generated matches hand-written, M if they diverge and the gap needs authoring.

Expected deliverable: ~1 small PR for Phase 1 + ~1 medium PR for Phase 2 + ~1 small PR per lens for Phase 3. Could combine phases if the worker has bandwidth.

## Dispatch note

This lane is a **thesis receipt candidate**, not a bug fix. The primary value is demonstrating concretely that v3's structural approach (162 lines + fold over `d.nodes`) matches or beats v2's heuristic approach (5488 lines). If Phase 2 lands cleanly with matching output, it's the kind of artifact that becomes reference material for future "the algebra does the work" arguments. If it reveals a specific delta, that's equally valuable as a specific gap to close.

Director reviews Phase 2 output specifically — the v2/v3 diff is the load-bearing artifact. Framing of the diff in the PR body (as "v3 matches v2" vs "v3 improves over v2" vs "v3 misses case X") matters for how the receipt lands in the repo's thesis narrative.
