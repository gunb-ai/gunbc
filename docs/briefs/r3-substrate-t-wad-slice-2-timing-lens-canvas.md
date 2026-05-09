---
status: canvas (Substrate Mgr authority under Director standing-authority resolve at gunbc#1955 c#4411918059)
authority parent: R3 Substrate Manager (#1939 / #2068 warm-wolf-698)
ratification ask: Director (zesty-bear-812 inbox #828) — 4 substrate-shape questions named below; no ratification needed on whether to draft (per Director resolve note "canvas-drafting is Mgr-tier work")
roadmap row: §1.8 ledger rows #54 `timing_lens_carrier_landed` + #55 `shared_external_attachment_pattern_documented`
authority docs:
  - `docs/r3-structure.md:41` (T-Workflow-As-Data lane scope)
  - `docs/r3-structure.md:168-169` (gates #54/#55 closure conditions)
  - `docs/r3-structure.md:215` (lane row + gates summary)
  - `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` §"Slice scope (binding per Director)" (Slice 2 carved as separate canvas gated on T-LBP COMPLETE)
  - `gunb-ai/gunbc#1130` #issuecomment-4374109666 (Substrate Mgr design stance — Shared External Attachment Pattern)
  - `gunb-ai/gunbc#828` inbox-4374342708 (Director ratification of T-WAD lane 2026-05-04)
  - `docs/design-lens-application-surface.md` (Output projection precedent)
  - INVARIANTS.md #P5 (dissolution discipline) + #C-8 (fail-closed)
  - feedback_fail_closed_discipline (memory)
worker assignment: tidy-raven-610 / #2359 (#1955 carrier authoring)
---

# R3 Substrate canvas — T-WAD Slice 2 timing-lens substrate shape

## Why canvas

Slice 2 of T-Workflow-As-Data (per `r3-substrate-t-workflow-as-data-slice-1-worker.md` §"Slice scope") introduces an **observation-driven lens-shape class** parallel to the existing **structural-static** `Lens<C>` instances (complexity, cost). This is a substrate-shape novelty that warrants Director disposition before worker carrier-authoring locks shapes:

- The existing `Lens<C>` carriers consume static program structure (Dag + Behavior). Timing-lens consumes **external observations** captured at workflow execution. The substrate shape for "lens reads external data" is not yet declared.
- `WorkflowObservationAnchor` (gate #55) factors the external-data attachment as a **reusable** primitive (six invariants per Substrate Mgr design stance gunbc#1130 c#4374109666). First consumer is timing; second consumer is named (likely `ProofReceipt` per ctrl#369). Carrier shape needs to be picked correctly so the second-consumer promotion is not a substrate-rewrite.
- Output projection `Observed | Missing | Ambiguous | Stale` (per r3-structure.md:168) needs explicit declaration shape; fail-closed enforcement on non-Observed states per `feedback_fail_closed_discipline`.
- Carrier placement (`dsl/std/` cross-provider universal vs `dsl/extdeps/<provider>/` provider-scoped) is a routing decision with strong precedent both ways.

Worker dispatch is in flight (tidy-raven-610 spawned 2026-05-09); canvas surfaces the shape decisions so worker can grep against locked authority rather than re-derive.

## Carrier inventory (binding per #1955 brief + r3-structure.md:168-169)

Four substrate carriers gate-named:
1. **`TimingMeasurement`** — single observation: subject identity + observed timing payload + observer attestation.
2. **`TimingObservationSet`** — collection of `TimingMeasurement` over a workflow run; the data the lens folds over.
3. **`WorkflowObservationAnchor`** — generic external-data attachment carrier (factored separately; six invariants below).
4. **`TimingBudget`** — declared budget for `Enforce`-mode lens application (parallel to `AsymptoticClass` in complexity, `SymbolicCost` in cost).

Lens declaration: `Lens<TimingMeasurement>` (or `Lens<TimingObservationSet>` if `C` is the fold-input collection — Question 1 below).

## Question 1 (Q-WAD-S2-LensC) — `Lens<C>` instantiation: per-measurement vs per-set

Two options for `C` in `Lens<C>`:

### Option (a) — `C = TimingMeasurement` (per-observation)

`Lens<TimingMeasurement>.read: fn(Dag, Behavior) -> Witness<TimingMeasurement>` returns a single measurement per Behavior. Aggregation across Behaviors handled by the Output projection.

- **Pro**: mirrors structural-static `Lens<C>` shape (`Lens<ComplexitySummary>`, `Lens<SymbolicCost>` — `C` is per-Behavior summary). Minimal new substrate.
- **Con**: a single observation per Behavior may not match real-world workflow timing (e.g., CI workflow has many timing observations: per-job, per-step, per-runner-resource).

### Option (b) — `C = TimingObservationSet` (per-set, collection-typed)

`Lens<TimingObservationSet>.read: fn(Dag, Behavior) -> Witness<TimingObservationSet>` returns the full set of observations attached to that Behavior.

- **Pro**: fits real-world workflow shape (multiple timing observations per Behavior); set-typed `C` is precedented (e.g., `List<EmissionProvenance>` framing in T-EmissionProvenance discussion).
- **Con**: set-typed `C` introduces fold-aggregation question (does the lens framework's `LensCompose` work over set-typed `C`? Substrate Mgr precedent: `feedback_strict_mirror_vs_novel_substrate_fact` — set-typed `C` was previously handled via per-Behavior aggregation patterns).

**Recommended (Mgr-tier preliminary)**: **(b)** — workflow timing is intrinsically multi-observation per Behavior (CI workflow ≠ single-call complexity computation). Per-set `C` matches the data shape; precedent exists for set-typed `C`.

## Question 2 (Q-WAD-S2-Anchor) — `WorkflowObservationAnchor` shape: timing-specific vs generic

Two options for the anchor carrier:

### Option (a) — Timing-specific now; promote-to-generic on second consumer

Author `WorkflowObservationAnchor` with timing-shaped fields directly (e.g., `subject: BehaviorId`, `observed: TimingMeasurement`, etc.). Promote to `ExternalDataAnchor<Subject, Source>` parametric carrier when second consumer (`ProofReceipt` or other) lands.

- **Pro**: minimal-novel-substrate per `feedback_strict_mirror_vs_novel_substrate_fact`; deferred-genericization is the precedent pattern.
- **Con**: the **six invariants** (gates #55 acceptance) call for shape that's intrinsically anchor-generic, not timing-specific. Authoring timing-shaped now means the carrier rename + parameterization happens at second-consumer dispatch.

### Option (b) — Generic from authoring; timing is first instantiation

Author `WorkflowObservationAnchor<Subject, Source>` (or `ExternalDataAnchor<Subject, Source>` directly) parametrically; timing is `WorkflowObservationAnchor<BehaviorId, TimingPayload>` instantiation, where `TimingPayload` is the observed-value payload (e.g., `{ nanoseconds: Int }`) **without** subject identity baked in. Subject identity is owned solely by the anchor's `Subject` parameter (P2 single-authority discipline) — observation payloads carry observed values, not subject facts.

- **Pro**: gate #55 closure predicate ("`WorkflowObservationAnchor` factored separately as reusable external-data attachment primitive") is naturally satisfied; second-consumer promotion is zero-substrate-edit.
- **Con**: novel-substrate authoring at first consumer; per `feedback_strict_mirror_vs_novel_substrate_fact` this needs canvas justification (which this canvas provides).

**Recommended (Mgr-tier preliminary)**: **(b)** — gate #55 acceptance language ("factored separately as reusable") points to (b); the six invariants are intrinsically anchor-generic; second-consumer scenario is named (ctrl#369 `ProofReceipt`); zero-substrate-edit promotion is materially better than rename-on-second-consumer.

## Question 3 (Q-WAD-S2-Output) — Output projection shape

Per r3-structure.md:168, `Output` is "projection/report distinguishing `Observed | Missing | Ambiguous | Stale` (fail-closed enforcement on non-observed states)".

### Option (a) — Sum type: `TimingObservationOutcome = Observed | Missing | Ambiguous | Stale`

Direct sum-type encoding. Each variant carries shape-appropriate payload (`Observed { value: TimingMeasurement }`, `Missing { reason }`, `Ambiguous { candidates }`, `Stale { observed_at, expired_after }`).

**Dissolution classification (INVARIANTS P5 / modeling-discipline.md §coproduct dissolution): 🟢 GREEN (terminal).** No richer source exists at the workflow-observation boundary. The four variants exhaust the externally-attested observation states: a fact was attached cleanly (Observed), no fact attached (Missing), multiple conflicting facts attached (Ambiguous), a previously-attached fact has expired its validity window (Stale). All variants trace to the substrate's external-data attachment surface (six invariants per gate #55); none has a richer-source-extraction path that would dissolve to a finer coproduct. Worker `.dag` declarations MUST carry the 🟢 marker comment per modeling-discipline.md:132 checkpoint discipline.

- **Pro**: direct match to gate #55 invariant 5 ("stale/ambiguous/missing/observed report states").
- **Con**: `LensEnforcement<Output, Budget>.violates: fn(Budget, Budget) -> Bool` (per `src/v3/std/lens_application.dag:100`) does NOT see raw Output — it sees projected Budget values. Fail-closed on non-Observed states requires either (i) the projection step (Output → Budget) fabricating a max-violating Budget for non-Observed variants, or (ii) substrate extension of `violates` signature to `fn(Output, Budget, Budget) -> Bool`. (i) is workable but pushes the report-state semantics one level out of the lens-framework view; (ii) is a substrate-shape change to `LensEnforcement` itself (not just the Output type).

### Option (b) — `Result<TimingMeasurement, ObservationFailure>` with `ObservationFailure` enum

Two-level: outcome is `Result`, failure is enum. Shifts the ambiguous/stale/missing distinctions one level down.

- **Pro**: matches stdlib-Result-style intuition.
- **Con**: extra indirection; gate #55 acceptance language reads as flat-sum (Observed *and* the failure variants are peer report states); fail-closed is more verbose to express.

**Recommended (Mgr-tier preliminary)**: **(a)** with projection-step report-state fold (option (a)(i) — projection fabricates violating Budget for non-Observed). NOT option (a)(ii) — substrate extension of `LensEnforcement.violates` signature is canvas-tier and would cascade across all lens-application consumers (T-LBP / T-CostLens / T-LAS), not just timing.

### Option (c) — Output-folded-into-carrier (worker tidy-raven-610 PR #2360 shape)

Worker authored `TimingMeasurement = Observed { nanoseconds: Int } | Missing | Ambiguous | Stale` directly — the carrier IS the report state, no separate Output projection.

**Dissolution classification: 🟢 GREEN (terminal)** — same rationale as (a); the four variants exhaust the externally-attested observation states. Worker `.dag` MUST carry the 🟢 marker comment. `Lens<TimingMeasurement>` is per-observation; `TimingObservationSet` is separate aggregation. Budget projection (TimingMeasurement → TimingBudget compare-shape) handles the (a)(i) fabrication naturally: non-Observed variants project to a violating Budget value.

- **Pro**: avoids the `violates: (Budget, Budget) -> Bool` signature collision entirely — projection step IS where Observed-vs-non-Observed semantics live, no fabrication ambiguity. Substrate carrier count is lower (no separate Output projection type).
- **Con**: \"per-observation\" framing means TimingObservationSet lens-fold semantics need explicit aggregation logic (sequential / parallel composition over per-observation TimingMeasurement values).
- **Status**: shape ratification pending — Director call between (a)(i) separate-projection vs (c) folded-carrier per #828 c#4412018726.

## Question 4 (Q-WAD-S2-Placement) — Carrier placement: `dsl/std/` vs `dsl/extdeps/`

Per Director STOP-and-PING standby (#828 c#4411989488): "carrier-placement decisions are well-precedented (compare T-LBP / T-CostLens / T-LAS history); no pre-ratification needed."

Two options:

### Option (a) — `dsl/std/timing_lens.dag` (cross-provider universal)

Place all four carriers (`TimingMeasurement`, `TimingObservationSet`, `WorkflowObservationAnchor`, `TimingBudget`) + lens declaration in a new `dsl/std/timing_lens.dag` file.

- **Pro**: timing-as-observation is a substrate-universal concept (any workflow runner produces timing observations); not provider-specific. T-LBP / T-CostLens / T-LAS lens carriers all live in `dsl/std/`. Promotes the "lens framework is cross-provider" claim.
- **Con**: WorkflowObservationAnchor's first concrete *workflow* is GitHub Actions CI (per Slice 1 placement at `dsl/extdeps/github/actions.dag`); having anchor in `dsl/std/` while workflow grammar lives in `dsl/extdeps/` could feel split.

### Option (b) — `dsl/extdeps/github/actions.dag` (extends Slice 1 placement)

Fold the four carriers into the existing `dsl/extdeps/github/actions.dag` adjacent to Slice 1 carriers.

- **Pro**: workflow-coherent; all workflow-related substrate in one file.
- **Con**: timing-lens carriers are NOT GitHub-Actions-specific; placing in `dsl/extdeps/github/` would require a future migration when second runner (e.g., GitLab CI, Buildkite) lands.

**Recommended (Mgr-tier preliminary)**: **(a)** — `dsl/std/timing_lens.dag`. Lens carriers are cross-provider universal per existing T-LBP / T-CostLens / T-LAS precedent. WorkflowObservationAnchor is anchor-generic (per Q-WAD-S2-Anchor (b)), not GitHub-specific. Workflow grammar (GitHub-specific) staying at `dsl/extdeps/github/actions.dag` is correct; lens substrate (cross-provider) belongs in `dsl/std/`.

## Six invariants (gate #55 acceptance — `WorkflowObservationAnchor`)

Per r3-structure.md:169 (verbatim, for worker brief reference):

1. **Stable subject identity** — not span-based; identity is structural (e.g., BehaviorId), not source-position.
2. **Observed-artifact identity/digest** — content-addressed; stale detection works against digest, not timestamp alone.
3. **Producer/observer/prover identity** — three roles distinguishable; producer = workflow runner, observer = measurement-capture process, prover = signature/attestation source.
4. **Attachment timestamp + run id** — both fields required; run id provides workflow-scope, timestamp provides ordering.
5. **Stale/ambiguous/missing/observed report states** — per Q-WAD-S2-Output (a) above; flat sum.
6. **Fail-closed enforcement on non-observed/non-valid states** — `LensEnforcement.violates = Output != Observed` per `feedback_fail_closed_discipline` + INVARIANTS C-8.

## Acceptance gates (worker dispatch — same-canvas)

After Director ratifies the four shape questions, worker tidy-raven-610 (#2359) executes:

1. Carrier landings per Q-WAD-S2-Placement disposition.
2. `Lens<C>` declaration per Q-WAD-S2-LensC disposition.
3. `WorkflowObservationAnchor` shape per Q-WAD-S2-Anchor disposition (with six invariants documented inline).
4. Output projection per Q-WAD-S2-Output disposition.
5. §1.8 ledger advancement: gate #54 `timing_lens_carrier_landed` + gate #55 `shared_external_attachment_pattern_documented` → CONSUMER_LANDED.
6. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean.
7. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.

## STOP / PING criteria

- **STOP** if Director disposition on any of Q-WAD-S2-LensC / Anchor / Output / Placement diverges from Mgr-tier preliminary recommendation — re-grep substrate impact + re-author worker brief acceptance language.
- **STOP** if worker grep finds prior unmerged authoring of any of the four carriers (collision with concurrent T-WAD work) — surface to Mgr immediately.
- **STOP** if six invariants surface a substrate-fact gap (e.g., "producer/observer/prover identity" requires a substrate carrier not yet declared) — fold the gap into this canvas and re-ratify.
- **PING** Verification Mgr (#2075) at PR-open for §1.8 ledger ratchet authoring on gates #54/#55.
- **PING** PB Mgr (#2074) at carrier-landing if `WorkflowObservationAnchor`'s second-consumer (`ProofReceipt`) cascades into PB-Runtime work.

## Provenance

Drafted 2026-05-09 by warm-wolf-698 (Substrate Mgr) per Director standing-authority resolve at gunbc#1955 c#4411918059. Worker tidy-raven-610 (#2359) spawned independently 2026-05-09T07:46Z; canvas surfaces shape decisions ahead of worker carrier-authoring decision points.

Mgr-tier recommendations on all 4 questions are preliminary; Director ratification needed before worker locks shapes. Canvas authoring under standing-authority does NOT bypass Director substrate-shape ratification per `feedback_canvas_two_axis_verification`.
