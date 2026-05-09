# Worker brief — Substrate T-LBP complexity-lens substrate completion

**Sub-issue**: TBD (parented under R3 Substrate Mgr lane #1939; new sub-issue created at dispatch time).
**Authority — live design doc**: [`docs/design-complexity-lens-behavioral-completeness.md`](../design-complexity-lens-behavioral-completeness.md) — comprehensive R3 design that resolves substrate-shape questions for `lenses/complexity.dag` PROXY → COMPLETE. **All implementation decisions trace to this doc**; brief delegates carrier shape, dimension wiring, and consumer-rewrite to its §§1.1–1.6 + §3.
**Authority — lane row**: [`docs/r3-structure.md`](../r3-structure.md) row 146 (T-Lens-Behavioral-Parity slice 1 — complexity).
**Authority — Director ratification**: Q-Complexity-Composition-Layering canvas (`docs/proposals/q-complexity-composition-layering-canvas.md`) Q1/Q2/Q3 RATIFIED at gunb-ai/gunbc#828 #issuecomment-4402714255 — finding: ε precedent does NOT apply (no target-context axis); slice-tier dispatch authorized; existing substrate authorities (`SymbolicCost`, `CostBound`, `ComplexitySummary`, `AsymptoticClass`) consumed, NOT re-introduced.
**Closure predicate**: §1.8 row #79 `complexity_lens_behaviorally_complete` (work/span split + asymptotic classification + cementing test); cementing test (#1950) is a separate downstream brief.

## Important framing — this is NOT substrate-fact-introduction

Per the live design doc §1, **all six substrate carriers already exist or are minor enrichments**, not new P1 facts:

- §1.1 `SymbolicCost` 7-variant — already at `src/v3/std/algebra.dag` (DB-7); no substrate change
- §1.2 `SizeVariable` — enrichment (add `display_name: String?`), not new type
- §1.3 work/span — TWO `AnalysisDimension<SymbolicCost>` instances (`work_dimension` + `span_dimension`) per DB-3; the `AnalysisDimension` parent already declared in `src/v3/std/dimensions.dag`
- §1.4 `AsymptoticClass` — 8-variant sum + `BoundedLattice<AsymptoticClass>` instance, declared in `src/v3/std/algebra.dag` per design §1.4 spec (NB: `data` form gated on class-5 record bodies; Rust-execution authority pattern applies until grammar lands)
- §1.5 `Certainty = Proven | Conservative` — complexity-lens-local sum type; declared in `lenses/complexity.dag` per design §1.5
- §1.6 `RecurrenceForm` + `CostBound` — both already at `src/v3/std/induction.dag`; missing piece is the `cost_bound_to_symbolic` projection (design §1.6 spec)

Plus §1.7 `ComplexitySummary` record (work + span + cost + asymptotic + certainty coordinates) at `lenses/complexity.dag`.

**Use existing carriers verbatim**; do NOT invent parallel names (e.g., `ComplexityCost`). Per `feedback_parallel_representation_debt`: `CostBound` is canonical for the proof; `SymbolicCost` is the display/composition algebra; consume both, do not re-author.

## Hard prerequisite — T-E-P-Producer-Broadening must be COMPLETE

Per design doc §"Authority discipline": *"Implementation lane is T-Lens-Behavioral-Parity slice 1 (complexity); cascade-gated on T-E-P-Producer-Broadening"*. Per `docs/r3-structure.md` T-Lens-Behavioral-Parity row, complexity-lens BEHAVIORAL COMPLETION cannot land before T-E-P-Producer-Broadening covers all live call sites.

**Live status of T-E-P P1**:
- Slices 1-4 (#2167/#2178/#2182/#2192): direct-call sub-class closed
- Slice 5+ (Indirect/`TransformDispatch::Indirect` / `ArrowPortRef`): in flight via eager-bat-178 (current PR #2198 is repositioned arithmetic-Div follow-up; real Slice 5 indirect-call work is next per worker disposition at gunb-ai/gunbc#2166 c#4402742599)
- Additional sub-classes per design §3 may surface (parser/worklist/fold-body classes named in eager-bat-178's prior planning framing)

**Worker action**: STOP and surface to Mgr at slice authoring time IF T-E-P P1 is not yet declared COMPLETE across all sub-classes the design doc names. `complexity_lens_behaviorally_complete` cannot ship while T-E-P producer carriers are partial — `DescentEvidence` / `CallPattern` / `SubValueRelation` reads at recursive-call sites would have gaps that propagate into the lens output. Prior framing (in earlier draft of this brief) treating producer coverage as optional corpus scope was wrong; this brief retracts that position.

## Scope (binding per design doc + Director ratification)

Three coordinated authoring deltas — all named in the design doc, all consume existing substrate:

### Deliverable 1 — Substrate enrichments per design §§1.1–1.6

Author the named substrate enrichments per design doc verbatim:

- **§1.2**: extend `SizeVariable` in `src/v3/std/algebra.dag` with `display_name: String?` field; add `size_variable_eq` (equality on `source_port` only).
- **§1.3**: declare `work_dimension` + `span_dimension` data declarations in `lenses/complexity.dag` per design §1.3 spec; respect class-5 record-body cascade gate (Rust-execution authority pattern via `v3_compiler::analyze_symbolic_cost_dimension` until class-5 grammar lands; substrate-data-spec stays in `.dag` per P5 dissolution-trigger discipline).
- **§1.4**: declare `AsymptoticClass` 8-variant + `BoundedLattice<AsymptoticClass>` instance + `meet_asymptotic_class` / `join_asymptotic_class` / `classify(SymbolicCost) -> AsymptoticClass` in `src/v3/std/algebra.dag` per design §1.4 spec.
- **§1.5**: declare `Certainty = Proven | Conservative` in `lenses/complexity.dag` per design §1.5 spec; **no `BoundedLattice<Certainty>` declaration** (composition is cost-aware via §3.1 `compose_summary_*` family, not lattice-fold).
- **§1.6**: add `cost_bound_to_symbolic(CostBound) -> SymbolicCost` projection in `src/v3/std/induction.dag` per design §1.6 spec.

### Deliverable 2 — `ComplexitySummary` record + lens widening

Per design §1.7 + §3:

- Declare `ComplexitySummary { work, span, cost, asymptotic, certainty }` record in `lenses/complexity.dag`.
- Widen lens output type from `Lookup<Int>` to `Lookup<ComplexitySummary>`; preserve forward-fold catamorphism shape (substrate invariant on `d.nodes` ordering).
- Implement `compose_summary_*` family per design §3.1 — sequential / iterate / branch-max compositions; per-dimension certainty projection consuming the cost-aware composition shape from §1.5.
- Status header refines: `STRUCTURALLY TERMINAL; BEHAVIORALLY PROXY` → `STRUCTURALLY TERMINAL; BEHAVIORALLY COMPLETE`.

### Deliverable 3 — T-E-P P1 carrier consumption (recurrence wiring)

Per design §1.6 + §3: consume `RecurrenceForm` + `master_theorem` (E-I lane substrate already at HEAD) at recursive-call sites; classify per call-pattern via T-E-P P1 `DescentEvidence` / `CallPattern` / `SubValueRelation`. Routes through `cost_bound_to_symbolic` projection (Deliverable 1 §1.6).

## Out-of-scope

- **Cementing test (#1950)**: separate downstream brief at `docs/briefs/r3-substrate-t-lbp-complexity-lens-cementing-test-worker.md`; gates additionally on frozen v2-oracle snapshot capture (Q4 RATIFIED 2026-05-08 — PB Manager owns; #828 c#4402740072).
- **`Lens<C>` generic carrier-shape refactor**: β-extended DEFERRED to N=2 trigger per `q-lens-target-context-canvas.md`; STOP-and-PING if surfaces.
- **Cost-lens substrate completion**: separate active stream (fierce-ram-21 ε path); not absorbed.
- **Class-5 record bodies in `data` declarations**: cascade-gated per design §1.3 + §1.4 NB; stay on Rust-execution authority pattern until grammar lands; do not block this slice on class-5.

## Acceptance gates (same-slice, all must pass)

1. T-E-P-Producer-Broadening declared COMPLETE before this slice opens (worker grep at slice-authoring time per Hard prerequisite section above).
2. All §§1.1–1.7 substrate enrichments authored per design doc verbatim; carrier names match design doc (NO renames — `ComplexitySummary` / `AsymptoticClass` / `Certainty` / `cost_bound_to_symbolic`).
3. `complexity.dag` advanced from `STRUCTURALLY TERMINAL; BEHAVIORALLY PROXY` → `STRUCTURALLY TERMINAL; BEHAVIORALLY COMPLETE`; status header updated in the file itself.
4. Lens output type widened: `Lookup<Int>` → `Lookup<ComplexitySummary>`; forward-fold catamorphism preserved.
5. **#79 satisfied by construction**: lens output documented to carry symbolic CostExpr + work/span split + asymptotic classification. Code-level cite in PR description.
6. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean.
7. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.
8. **r3-structure.md row 146 receipt-text refresh** + `docs/v3-lens-capability-register.md` complexity-lens row update reflecting BEHAVIORAL COMPLETE; cite this PR's # as the receipt.

## STOP / PING criteria

- **STOP** if T-E-P-Producer-Broadening is not yet COMPLETE across all live call-site sub-classes — surface to Mgr; coordinate sequencing with eager-bat-178.
- **STOP** if any §§1.1–1.6 design spec doesn't map structurally onto substrate at HEAD (e.g., grammar gap beyond class-5; algebra primitive missing) — surface to Mgr for canvas-tier re-open.
- **STOP** if `Lens<C>` generic carrier-shape refactor surfaces — β-extended DEFERRED to N=2 trigger; do NOT absorb.
- **PING** Verification Mgr (#2075 / wise-bear-525) at PR-open per Pattern-A executable-gate ratchet for #79 PROXY → COMPLETE transition.

## Worker pin

Fresh-pool pick at dispatch time (NOT eager-bat-178 per Director Q3 framing — different problem class than T-E-P producer-broadening, plus they own the prerequisite Slice 5+ work; NOT fierce-ram-21 — busy with cost-lens ε Slice 1a→1b chain).

## Cross-Mgr coordination

- **Verification Mgr (#2075)**: PR-open ping per Pattern-A executable-gate.
- **Eager-bat-178**: T-E-P P1 sequencing — this slice gates on Slice 5+ Indirect-variant land + any further sub-classes.
- **PB Manager (#2074)**: separate stream for v2-oracle snapshot capture (Q4 RATIFIED); not on this slice's critical path but on downstream cementing (#1950).
- **#1950 downstream**: complexity-lens cementing test gates on this slice's BEHAVIORAL COMPLETION + frozen v2-oracle snapshot.

## Auto-spawn caveat

L-sized substrate-behavioral-completion threshold; HOLD dispatch on this brief until auto-spawn fix lands per ctrl#217 + same-window-dispatch discipline applies post-canvas-merge (PR #2197) AND post-T-E-P P1 COMPLETE.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-08 per Director Q1/Q2/Q3 RATIFIED at gunb-ai/gunbc#828 #issuecomment-4402714255; **rewritten 2026-05-08 post-codex BLOCKING at #2199** to consume existing design authority `docs/design-complexity-lens-behavioral-completeness.md` verbatim (prior draft had three BLOCKING violations: missing-canvas reconciliation, optional-producer-coverage, stale-row receipt; all three addressed in this rewrite). Sibling canvas at `docs/proposals/q-complexity-composition-layering-canvas.md`.
