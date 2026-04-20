# SG-4a — Infer dissolution foundation `(L)`

## Context

`infer.rs` is the remaining pipeline stage not authored in `.dag`. A full cutover (SG-4 proper) is tempting but **the substrate expressivity pressure is uncharacterized**. The prior SG-manager STOP on full infer work was about whether the substrate can cleanly express what inference actually does — type resolution, operator arrow resolution, predicate substitution, mutual-recursion cluster inference, diagnostic threading, refined-generic substitution.

**Until we've mapped the expressivity gap, dispatching "full cutover" is committing before knowing.** SG-4a de-risks this: foundation + authority map + regen_infer prototype. After SG-4a closes, we reassess whether SG-4b (full dissolution) is a single XXL lane, multiple sequential lanes, or blocked on substrate extensions.

This is preflight, not cutover.

## Read first

- `src/v3/compiler/src/infer.rs` — current inference implementation. Understand the major functions: type resolution, operator arrow resolution, predicate substitution, mutual-recursion cluster inference. Read in full — this lane's deliverable is a map of it.
- `src/v3/compiler/src/infer_helpers_generated.rs` + `src/v3/lenses/infer_helpers.dag` — the partial dissolution that already landed. Template for the shape of what infer.dag would produce.
- `docs/design-db16-refined-generic-substitution.md` — how refined generics flow through inference (load-bearing for any algorithm in this space).
- `src/v3/std/substrate.dag` — `PortState`, `TypeShape`, `CardinalityBound` — the substrate facts inference writes.
- `src/v3/std/effects.dag` — `WorkflowEffect` and the Stage 2b effects authority — inference produces this on root nodes.
- `docs/design-mutual-recursion-lowering.md` — `Cluster` / `LoopBound::Descent` — how mutual recursion fact flows through inference.
- `src/v3/compiler/src/dag.rs` — the `Dag` structure infer.rs mutates. `emit_anchors`, `stdlib_types`, `ports`, `diagnostics` — the write surface.
- **The SG-manager's prior STOP on infer work** — the exact substrate-expressivity concerns that halted earlier dispatch. Worker should surface these from the chat history or ROADMAP debt rows if not already written down.

## Work

Not a cutover. Three deliverables, all foundation-level.

**Deliverable A — Authority map (`docs/infer-dissolution-plan.md`).**

Walk `infer.rs` function-by-function. For each:

1. **Function name + LOC**
2. **Classification**:
   - **Structural reader** (pure function over typed substrate facts → typed output) — SG-4b candidate for direct `.dag` port
   - **Helper logic** (list manipulation, scope threading, substitution mechanics) — SG-4b candidate for helper-level generation matching `infer_helpers_generated.rs` pattern
   - **Imperative state machine** (mutates `Dag`, threads diagnostics through local state) — SG-4c territory; may need substrate extension
   - **Cross-stage glue** (calls out to lower, emits diagnostics, populates emit_anchors) — likely residual Rust shim
3. **Substrate gap, if any** — if `infer.dag` would need a fact the substrate doesn't currently carry, name it. Examples: "inference produces a constraint-set fact the substrate has no carrier for" or "diagnostic threading relies on local state not expressible in `.dag`."
4. **Risk rating**: low / medium / high — how confident are we this function dissolves cleanly?

Output: structured document classifying every significant `infer.rs` function.

**Deliverable B — `regen_infer` prototype (non-ambitious).**

Extend `regen_infer_helpers.rs` (or fork a new `regen_infer.rs`) to emit a **minimal** `infer_generated.rs`-shaped output from a **small subset** of `infer.rs` — specifically, the Category-1 (structural reader) and Category-2 (helper logic) functions from Deliverable A. Target: ~10-20% of current `infer.rs` ported.

This is the proof-of-concept: regen pattern works at inference scope; a future SG-4b can extend it to the harder categories.

**Deliverable C — Substrate-gap enumeration (ROADMAP debt rows).**

Every Category-3 function (imperative state machine) in Deliverable A that reveals a substrate-expressivity gap becomes a named debt row in ROADMAP.md. Each row:
- Name of the function + its current shape
- Specific substrate fact that can't be expressed today
- Proposed extension shape (new carrier? new connective? refined existing?)
- Dissolution trigger for the gap

Result: a concrete list of substrate extensions required before SG-4b is viable.

## Acceptance

- `docs/infer-dissolution-plan.md` exists and classifies every function in `infer.rs` (or at minimum every function > 50 LOC)
- `regen_infer` (or equivalent) produces `infer_generated.rs` for the Category-1 + Category-2 subset; the generated code compiles + is consumed by `infer.rs` (partial dissolution pattern matches `infer_helpers_generated.rs`)
- Measurable reduction in hand-Rust inference LOC — target ~10-20% dissolved, but the real deliverable is the **plan**, not the LOC drop
- ROADMAP.md has named debt rows for every substrate-expressivity gap surfaced (possibly zero gaps, if the plan reveals everything is dissolvable — that's a valid outcome, just surface it)
- Post-SG-4a: Director + SG-manager can decide whether to dispatch SG-4b as a single XXL cutover, multiple sequential lanes, or hold on substrate extensions

## STOP-AND-ESCALATE

- **If Deliverable A reveals > 40% of `infer.rs` in Category 3 (imperative state machine)** — STOP. That's a signal the substrate as it stands can't express inference and SG-4b is not a compile-authority problem, it's a substrate-design problem. Surface the count; director decides whether to extend substrate (separate lane) or pause SG-4 indefinitely.
- **If the regen_infer prototype breaks DB-8 fixed-point** — STOP. Even a partial generated subset must produce bit-identical `Dag` output as the hand-Rust it replaces. Drift is the critical-path bug.
- **If a specific function's classification is ambiguous** (could be structural reader OR imperative state machine) — flag in Deliverable A with a risk rating of "high" and a one-paragraph "why ambiguous." Don't force a classification.
- **If ROADMAP debt rows for substrate gaps grow past 10** — STOP. That's a signal the scope grew past SG-4a's intent. Propose splitting: land the plan + prototype; handle substrate extensions as separate named lanes before SG-4b.

## Non-goals

- **Not full `infer.dag` authority** — that's SG-4b (possibly future, possibly blocked).
- **Not deleting `infer.rs`** — partial dissolution only; the file stays on SG-0 census with reduced LOC.
- **Not substrate extensions** — if gaps surface, they become separate named work via ROADMAP debt rows.
- **Not touching `parse.rs` / `lower.rs` / `emit.rs`** — this is inference-only scope.
- **Not changing `PortState`, `CardinalityBound`, `TypeShape`** — these are stable; extensions would need their own lane.

## Size

L (not XXL). 2-4 weeks single-worker. Deliverable A is the biggest — comprehensive function-by-function audit with structural classification. Deliverables B and C follow naturally from A.

Expected LOC delta: **-200 to -500** hand-Rust dissolved (Category-1/2 subset). New: `infer.dag` or extended `infer_helpers.dag` (+100-300 LOC of declarations). **Planning artifact is the real deliverable.**

## Dispatch note

Director reviews Deliverable A with heaviest scrutiny — it's the gating artifact for SG-4b dispatch. Deliverable B should not ship without A first. Deliverable C ensures substrate extensions become tracked work rather than silent overreach during SG-4b.

## Post-SG-4a handoff

When SG-4a closes, director + SG-manager decide one of:
1. Dispatch SG-4b as XXL full cutover (if plan shows no substrate gaps and prototype is clean)
2. Dispatch SG-4b as N sequential lanes (if gaps are bounded; one per category)
3. Hold SG-4 on substrate extensions landing (if gaps are structural)

Option determined by Deliverable A + Deliverable C. SG-4a exists to make that decision honest.
