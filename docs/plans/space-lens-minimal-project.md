# Space lens — a minimal, confident memory prediction from the static `.dag`

> Plan doc, awaiting operator sign-off. Resumes `resource-aware-scheduler.md` **Node B** (`CostBasis::Predicted`) as a walking skeleton. DESIGN refs: §1 (time = cost/safety), §2 (Realization — one fold, N consumers), §3 (single authority — cost lives on the graph, width derives from it), §5 (fail-closed — *derive*, don't measure; refuse, don't guess), §7 (the lens is a pure reader over the same `Node` tree, sibling to the complexity/time lens).
>
> The referenced `docs/plans/derived-cost-input-envelope-roadmap.md` never landed on any branch (only a WIP commit names it); this doc supersedes that pointer rather than duplicating it.

## The ask, in one line

A lens over an arbitrary `.dag` graph that returns the **peak resident memory** computing it will need — derived from the static graph alone, confident by construction, fed to realization to pick shard width. The time analog already exists (the complexity lens derives loop/termination structure from the graph); this is the **space** sibling.

## Why this is a resume, not a greenfield

The architecture is already designed and signed. This doc only fills the one hollow spot.

| Piece | Where | Status |
| --- | --- | --- |
| `ResourceEnvelope` single authority, every knob derived | `compute-envelope-model.md`, `dag/product/compute_fabric.dag` | designed, signed |
| **"Derivation is the authority; measurement is the falsifier"** | `resource-aware-scheduler.md` (operator ruling 2026-06-25) | signed |
| `InputEnvelope = BoundedInput \| EnvelopeUnknown` + fail-closed admission | `dag/gunbc/ci_input_envelope.dag` | **landed** (P1) — this is "forcing modeling on inputs" |
| `CostAccount<S>{ time, space, power, basis }`, `CostBasis = Predicted \| Measured` | `dag/std/realization_schedule.dag` | carrier exists, **`space` always `byte_size(0)`** (`cost_account_predicted_zero`) |
| width formula `floor(budget ÷ per_shard_peak)` | `dag/std/realization_width.dag` | exists, reads `predicted_peak` from a **static data row** |
| per-shard RSS + closure-node-count measurement | #6425 (this session), `claim_executor` | **landed** — the *falsifier* half, explicitly "not the scheduler directly" |

**The gap is exactly one thing:** a fold that reads the static `.dag` closure and returns `CostAccount.space` with `basis = Predicted`, so `runnable_predicted_space` stops reading a hardcoded number. Everything above is scaffolding waiting for that fold.

## The seam (precise)

- `runnable_predicted_space(profile) -> ByteSize` (`realization_schedule.dag:168`) returns `profile.memory.peak.predicted_peak`. Today `predicted_peak` is stuffed from a static row (`gunbc_ci_floor_per_shard_peak_samples`, the row this session's PR annotated `STALE-PENDING-RECALIBRATION`). **This is the injection point.** Replace the row read with the fold's output.
- `InputEnvelope` already declares input **size as a node count** — `InputSizeAxis = WitnessCount | SourceNodeCount | CorpusNodeCount`, each with a `max: Measure<Count, One, Nat>` ceiling, and `input_admitted` refuses `EnvelopeUnknown` fail-closed. So the input-size axis the space lens multiplies is *the axis the envelope already bounds*. No new input modeling — P1 did it.
- #6425 emits `roster_closure_nodes` (measured 274 at `spawn_width=4`) alongside per-shard RSS. That measured node-count is (a) the `SourceNodeCount` the envelope declares a ceiling for, and (b) one half of the calibration pair for the per-node constant below.

So the whole lens reduces to: **declared node-count ceiling × calibrated bytes-per-node + base**, evaluated statically, refusing when the input is undeclared.

## What the prediction feeds — two knobs, not one

The affected-set lane (loyal-wren-398) root-caused the host OOM as **systematic, not intermittent**: a `width == 1` shard at ~6.7 GiB is killed under its own 24 GiB slot cap purely by **host-level** oversubscription (the kernel evicts the biggest resident process host-wide; ~25 × 24 GiB committed on 134 GiB physical). The load-bearing consequence for this lens: when the kill is host-level, **narrowing width does nothing** — width is the per-shard lever, but no width choice fits a host that is already oversubscribed. So the per-shard prediction must feed **two** knobs (`compute-envelope-model.md`'s split), and the systematic case is served by the second:

1. **Within-job width** — `floor(slot_budget ÷ predicted_per_shard)`. Handles the cap-relative case.
2. **Cross-job packing / admission** — how many big residents co-reside on a host. When `predicted_per_shard` exceeds available host headroom, the remedy is to **refuse co-residence** (fail-closed admission), not to pick a smaller width. This is the CTRL arm (runner-slot converge to live==declared + `MemoryMax` de-drift) that the affected-set lane recommends first, and it is what the systematic kill actually requires.

So slice 4 below wires the prediction into the width formula *and* records that the same value is the packing/admission input; the admission-refusal path is the systematic-case remedy, not an optional extra. Calibration input for this comes directly from the affected-set lane's per-PR flip receipts (per-shard RSS + `roster_closure_nodes`), **including RSS-at-kill as a lower bound** on runs that 137 before completing — the flipped corpus peak has otherwise never been observed.

## The one irreducible empirical coefficient (and its dissolution)

`bytes_per_node` — how many bytes of resident memory one typed node occupies — is **not** a graph property; it is a property of how a node is *represented* in memory. Today a typed node is a Rust struct, so its byte cost must be measured. This is the sole non-derived input, and it is quarantined to one declared row.

- **Home:** one `Scaffold`-disposition constant (mirroring the `ci_input_envelope` ceiling rows, which are `Scaffold` for the same reason).
- **Dissolution trigger:** the **model↔realization grounding** open thread (DESIGN "Open threads"; `Value::Null` split / primitive grounding). Once a typed node *is* a substrate value with a size computable from its own declared type, `bytes_per_node` is derived from the node's type, not measured, and the Scaffold row is deleted. Until then it is a calibrated constant, honestly marked — not smeared through code.
- **Refinement path (no rework of the fold):** flat `bytes_per_node` → per-`InputSizeAxis`/per-node-**kind** size model (a function-item vs a type-decl vs an import cost differently, relative sizes readable from the Rust struct shapes). Crude first, structural later; the fold's shape is unchanged.

## Why "confident about peaks from the static `.dag` only" holds

Three properties, each fail-closed, together are the confidence — not a trusted number:

1. **Total over the closure.** The fold visits every module in the transitive closure (a pure import walk, no run, no typecheck). Nothing is sampled or skipped. This is sound *because* the dominant workload holds the whole closure resident during typecheck (the code's own note: the transient whole-tree resolved graph is the floor's dominant RSS), so `peak ≈ Σ closure node sizes` is the model, not an approximation of a scheduling peak.
2. **Bounded by a declared envelope.** The prediction is evaluated at the `InputEnvelope` ceiling; `EnvelopeUnknown → RefusedUndeclared`. There is no unbounded input on which the lens guesses — an undeclared input is refused, exactly as an unbounded loop without `DescentEvidence` is refused (§5, the InputEnvelope's stated rationale).
3. **Continuously falsified.** `Measured.space` (from #6425's plumbing) must equal `Predicted.space` within tolerance; a divergence is a **typed, located error**, not a silent re-fit. The lens cannot quietly drift from reality — a wrong prediction stops the line (§5 factory model).

## Minimal project — five slices, each green-by-execution

Each slice lands on carriers that already exist; each is proven by a real consumer plus a discriminating RED.

1. **`node_size_model`** — a `Scaffold` `bytes_per_node: ByteSize` constant + a `base: ByteSize`, with the dissolution trigger above spelled out in-row. Calibrated from ≥2 measured `(node_count, rss)` pairs (this session's emission is the first point; a second width gives the second). RED: revert the constant → the falsifier gate (slice 5) goes red.
2. **`predicted_space_from_nodes(node_count) -> ByteSize`** — `base + bytes_per_node × node_count`. Pure. The fold that produces `node_count` is a `fold_node` reader over the closure `Node` tree (the complexity/time lens is the exact template). RED: a fixture with a known node count → asserted bytes; a wrong per-node constant flips it.
3. **Bind to `InputEnvelope`** — evaluate at the declared `SourceNodeCount`/`CorpusNodeCount` bound; reuse `input_admitted` so `EnvelopeUnknown → Refused`. RED: an undeclared-axis input must refuse, not predict (the admission test already has this shape).
4. **Populate `CostAccount.space` / `predicted_peak`; wire to width; delete the static row.** `runnable_predicted_space` returns the fold output with `basis = Predicted`; `spawn_width` consumes it unchanged. Delete `gunbc_ci_floor_per_shard_peak_samples`. RED: the width formula's output must move when the predicted space moves (a bigger closure → narrower width), and the deleted-row must not be referenced anywhere (compile wall).
5. **Falsifier gate** — assert `|Measured.space − Predicted.space| ≤ tolerance`; divergence is a typed error carrying both values and the closure identity. RED: plant a prediction off by > tolerance → the gate fires; a matching pair → green. This is the end-to-end control the #6425 review flagged as owed (it also finally discriminates closure-from-counter on the `width==1` path, since prediction is diff-independent by construction).

The skeleton (all five at their crudest — flat constant, single axis) proves the whole loop end-to-end on one corpus. Then slice 1's size model refines flat → per-kind, and grounds away entirely when the model↔realization thread lands. No downstream slice reworks.

## Sequencing caveat worth an explicit call

Today each shard builds its **own private copy** of the shared std/spec prefix (`Rc<ResolvedGraph>` is `!Send`; cross-shard sharing is the deferred "S2b/Arc frontier"). So a large part of `bytes_per_node × node_count` is that prefix, duplicated N times resident. If cross-shard sharing lands, the per-shard marginal roughly halves and `bytes_per_node` (or the model's prefix term) re-calibrates. **Decision for the operator:** calibrate against today's duplicated reality (useful now, re-measure after S2b), or gate slice 1 on S2b. This changes the coefficient, not the fold.

## Ownership / homing

This lane is operator-signed and multi-owner (`compute-envelope-model.md` names warm-lark-306, quick-ant-298, sharp-stag-782, calm-carp-204, bright-stag-194) and it touches load-bearing substrate (`std.realization*`, the scheduler, `CostAccount`). Per project discipline it should be homed as a proper work item under those owners, not run out of a closed "ci floor measurement" session. This doc is the model-first artifact for that sign-off; no load-bearing `.dag` file is edited until it is signed.

## Interim floor calibration emission (landed #6425)

**Authority:** `roster_import_closure_nodes_pre_resolve` in `src/v1/stage0/src/cli_run.rs` — deduped transitive import-closure of discovery rows plus prefix-context entries, counted at module-path grain via the pure import walk (no typecheck). The space-lens predictor and floor calibration emission bind here.

| Label | Carrier | Meaning |
|---|---|---|
| `roster_import_closure_nodes` | stderr, pre-resolve | Import-closure module count |
| `floor_peak_pre` / `floor_peak_post` | cgroup `memory.peak` steps in `gunbc.ci_workflow` | Job-scoped peak; censored lower bound on OOM kill |
| `floor_outcome` | post step | Step outcome paired with peak |

**Dissolve-on:** skip-before-resolve (count selected resident subset); bash-emit (#5828) for cgroup shell; resolver graph-major / S2a node-keyed store shrinks the closure the calibration measures.
