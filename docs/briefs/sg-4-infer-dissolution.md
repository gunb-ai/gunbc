# SG-4 — Infer dissolution (full `infer.dag` authority) `(XXL)`

## Context

Inference is the one remaining pipeline stage not authored in `.dag`. Today:

- `src/v3/compiler/src/infer.rs` — hand-authored inference logic (~2-3K LOC)
- `src/v3/compiler/src/infer_helpers_generated.rs` — partial generation shipped (helper-level dissolution)
- SG-4-prep-a landed result_port read authority (substrate-level inference facts)
- Refined generics (DB-16) shipped — substrate supports refined-carrier substitution

The thesis claim: "analyses are lenses over physics" / "zero heuristics." Lenses already work over the DAG after inference. But **inference itself is still hand-authored Rust** — the stage that decides what the lenses ultimately see. Dissolving `infer.rs` to `infer.dag` makes the full compiler pipeline (tokenize → parse → lower → **infer** → emit) `.dag`-authored.

## Read first

- `src/v3/compiler/src/infer.rs` — the current inference implementation. Understand the major functions: type resolution, operator arrow resolution, predicate substitution, mutual-recursion cluster inference. Read in full.
- `src/v3/compiler/src/infer_helpers_generated.rs` + `src/v3/lenses/infer_helpers.dag` — the partial dissolution that already landed. Template for the shape of SG-4's output.
- `docs/design-db16-refined-generic-substitution.md` — how refined generics flow through inference (load-bearing for any algorithm in this space).
- `src/v3/std/substrate.dag` — `PortState`, `TypeShape`, `CardinalityBound` — the substrate facts inference writes.
- `src/v3/std/effects.dag` — `WorkflowEffect` and the Stage 2b effects authority — inference has to produce this fact on root nodes.
- `docs/design-mutual-recursion-lowering.md` — Cluster / Loop::Bound::Descent how the cluster fact flows through inference.
- `src/v3/std/verification.dag` — how testgen / verification lens consumes inference output (informs what inference must produce).
- `src/v3/compiler/src/dag.rs` — the `Dag` structure `infer.rs` mutates. Specifically `emit_anchors`, `stdlib_types`, `ports`, `diagnostics` — the write surface.
- SG-4-prep-a merge commit for the result_port read authority PR — understand what it moved into substrate and what stayed in `infer.rs`.

## Work

Multi-PR lane. Propose staged plan in first PR body; example structure:

**SG-4-a — Categorize `infer.rs` functions (~1 PR, ~1 week).**
Walk `infer.rs` function-by-function. For each, classify:
- **Structural reader** (consumes typed substrate facts, emits typed output) — candidate for `infer.dag` direct port
- **Helper logic** (list manipulation, scope threading, substitution) — candidate for helper-level generation like `infer_helpers_generated.rs`
- **Imperative state machine** (mutates `Dag`, threads diagnostics) — hardest category; may need substrate extensions to dissolve
- **Cross-stage glue** (calls out to lower, emits diagnostics, etc.) — likely stays as thin Rust shim

Output: `docs/infer-dissolution-plan.md` — every function classified + a migration path. The planning artifact for SG-4-b through SG-4-e.

**SG-4-b — Author `src/v3/compiler/infer.dag` (1-N PRs).**
Write the structural-reader portions as `.dag` declarations + consumer walks. Emit via `regen_infer` binary to `infer_generated.rs`. Pattern matches `infer_helpers_generated.rs` but covers full inference, not just helpers.

**SG-4-c — Migrate imperative state (1-N PRs).**
For mutable state (Dag mutation, diagnostic threading), either:
- Extend substrate to carry the fact as a typed carrier so `infer.dag` can produce it structurally, OR
- Keep as thin Rust shim in a slimmed-down `infer.rs` that calls into `infer_generated.rs` for structural parts

Worker proposes which path per state class.

**SG-4-d — Cutover (1 PR).**
Replace `infer.rs` consumers with the new structural path. Reduce `infer.rs` to whatever shim survives (or delete entirely if fully dissolved). Update SG-0 census, `compiler.dag::hand_maintained_src`, module declarations.

**SG-4-e — Regression proof (1 PR).**
All existing inference tests pass. `m1_substrate_test`, `m2_substrate_inhabitance_test`, `lane2_stage_2d_symbolic_cost_test`, `lane2_stage_2b_db18_test` — all green. Post-emit-verifier green. Fixed-point `self_host_fixed_point` still converges.

## Acceptance

- `src/v3/compiler/infer.dag` exists as the authoritative inference specification
- `src/v3/compiler/src/bin/regen_infer.rs` regenerates `infer_generated.rs` deterministically (bit-identical from same input)
- `src/v3/compiler/src/infer.rs` either deleted OR reduced to a shim under 200 LOC (worker argues the residual is genuinely non-dissolvable)
- SG-0 census -1 major file + possibly -5 helpers; `compiler.dag::hand_maintained_src` updated
- All existing inference tests pass (reference: `m1_substrate_test`, `m2_*`, `lane2_*`)
- DB-8 self-host fixed-point still converges bit-identically on `default_fixed_point_source`
- ROADMAP SG-4 status flipped from 🟡 to ✅; Lane 3c self-hosting readiness note updated

## STOP-AND-ESCALATE

- **If SG-4-a reveals that > 40% of `infer.rs` requires substrate extensions** — STOP. That's a signal we'd be inventing substrate to fit inference rather than expressing inference in existing substrate. Surface the count; may warrant an intermediate substrate-design PR.
- **If a specific function requires mutable graph walks that can't be expressed without a new connective** — STOP. Don't extend the substrate here; propose the extension separately per C1 stop-signal discipline.
- **If the migration would break DB-8 fixed-point** (bit-identical compile output would drift) — STOP. The `infer.dag` must produce byte-identical side effects on `Dag` as the current `infer.rs`. Drift is either a bug or a justification the ROADMAP should name before merging.
- **If the cutover requires simultaneous changes to emit / lower / bootstrap** — STOP. This lane owns inference; coupled changes are separate lanes. Surface the coupling.
- **If refined-generics substitution (DB-16) hits a gap while authoring `infer.dag`** — STOP. DB-16 shipped; but this lane is the first real stress test. Name the gap; may need DB-16-extension PR first.

## Non-goals

- **Not redesigning inference semantics** — the algorithm stays the same; only the authoring layer moves from Rust to `.dag`.
- **Not touching emission** — lenses + emit already read the DAG post-inference; they don't know whether `infer.rs` or `infer.dag` wrote it.
- **Not dissolving `lower.rs` or `parse.rs`** — those are separate lanes (SG-3b / SG-2b, post-SG-3f).
- **Not extending the substrate** — unless explicitly required and justified via STOP-AND-ESCALATE.
- **Not changing `PortState`, `CardinalityBound`, or `TypeShape`** — these are the substrate facts inference writes; they're stable.

## Size

XXL. SG-4-a alone is 1-2 weeks. Full lane is multi-week. May require 1-2 substrate extensions (tracked separately). Worker should be comfortable with the inference algorithm + the regen_* pattern + `.dag` expressive limits.

Expected LOC delta at close: **-2K to -3K** hand-Rust dissolved. Adds ~500-1000 LOC of `infer.dag` declarations + a regen binary.

## Dispatch note

Director reviews each sub-lane PR. SG-4-a (categorization) is the planning anchor — most scrutiny there. STOP-AND-ESCALATE is especially important on substrate-extension pressure; inventing substrate to fit inference violates the modeling discipline.
