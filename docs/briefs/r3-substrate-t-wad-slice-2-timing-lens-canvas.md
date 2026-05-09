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

## Ratified final shape (AUTHORITATIVE — supersedes all sections below)

After 6+ Director ratification iterations 2026-05-09, the locked substrate-shape disposition is:

| Question | Final ratified shape | Director comment |
|---|---|---|
| **Q-WAD-S2-LensC** | per-observation `Lens<TimingMeasurement>` (worker shape; NOT canvas (b) per-set) | #828 c#4412301889 + c#4413322671 |
| **Q-WAD-S2-Anchor** | (a) timing-specific concrete fields on `WorkflowObservationAnchor` (NOT (b) generic parametric) — 4 invariants on anchor (subject_stable_id / artifact_digest / producer-observer-prover / attached_at + run_id); promotion to ProofReceipt-second-consumer is bounded refactor | #828 c#4412301889 + c#4413322671 |
| **Q-WAD-S2-Output** | folded carrier `TimingMeasurement = Observed { nanoseconds: Int } \| Missing \| Ambiguous \| Stale` (NOT separate-projection sum, NOT Result-typed). Carrier IS the report-state. Invariant 5 lives on TimingMeasurement variants. | #828 c#4412301889 + c#4413322671 (A-modified) |
| **Q-WAD-S2-Placement** | `src/v3/std/timing_lens.dag` per `src/v3/std/lens.dag:17-21` v3-only-carriers convention (NOT `dsl/std/`) | #828 c#4413159089 |
| **Q3 sub-disposition (LensEnforcement extension)** | (a)(ii) full-signature change: `violates: fn(Output, Budget, Budget) -> Bool` (NOT (a)(i) Budget fabrication; NOT (a)(iii) auto-violate-bit). `violates` body pattern-matches Output variants for fail-closed without Budget fabrication. | #828 c#4413284764 |

**Gate #55 split**:
- **#55a** — worker scope: anchor invariants 1-4 + carrier landing (PR #2360 folded-carrier ships as-is per (A)-modified disposition).
- **#55b** — Substrate Mgr scope: (a)(ii) `LensEnforcement` extension PR (signature change cascading T-LBP/T-CostLens/T-LAS impls + timing-lens fail-closed body). #55a + #55b co-close in the (a)(ii) PR per #828 c#4413322671.

**Sections below are RETAINED for the design-question history + reasoning trail** (Mgr-tier preliminary recommendations + reviewer-corrections + Director ratification iterations). Worker brief authoring should consume the table above as the binding shape; the per-question Pro/Con sections show how the shape was reached, not what to author.

## Why canvas

Slice 2 of T-Workflow-As-Data (per `r3-substrate-t-workflow-as-data-slice-1-worker.md` §"Slice scope") introduces an **observation-driven lens-shape class** parallel to the existing **structural-static** `Lens<C>` instances (complexity, cost). This is a substrate-shape novelty that warrants Director disposition before worker carrier-authoring locks shapes:

- The existing `Lens<C>` carriers consume static program structure (Dag + Behavior). Timing-lens consumes **external observations** captured at workflow execution. The substrate shape for "lens reads external data" is not yet declared.
- `WorkflowObservationAnchor` (gate #55) factors the external-data attachment as a **reusable** primitive (six invariants per Substrate Mgr design stance gunbc#1130 c#4374109666). First consumer is timing; second consumer is named (likely `ProofReceipt` per ctrl#369). Carrier shape needs to be picked correctly so the second-consumer promotion is not a substrate-rewrite.
- Output projection `Observed | Missing | Ambiguous | Stale` (per r3-structure.md:168) needs explicit declaration shape; fail-closed enforcement on non-Observed states per `feedback_fail_closed_discipline`.
- Carrier placement (`dsl/std/` cross-provider universal vs `dsl/extdeps/<provider>/` provider-scoped) is a routing decision with strong precedent both ways.

Worker dispatch is in flight (tidy-raven-610 spawned 2026-05-09); canvas surfaces the shape decisions so worker can grep against locked authority rather than re-derive.

## Carrier inventory (binding per #1955 brief + r3-structure.md:168-169 + Director ratification at #828 c#4412301889)

**Director ratified worker tidy-raven-610's shape as-shipped on PR #2360** (re-ratification at #828 c#4412301889) — reversing this canvas's prior (b)/(b)/(a) preliminary recommendations on Q1/Q2/Q3. Final ratified carrier shape:

- **`TimingMeasurement = Observed { nanoseconds: Int } | Missing | Ambiguous | Stale`** — fused carrier-as-report-state. Per Director's reasoning (`feedback_no_rejected_patterns` + `feedback_state_space_vs_behavioral_invariants`): the carrier admits exactly the legal states; folding the 4 variants into the carrier dissolves the artificial separation between "value carrier" and "outcome projection." Owns invariant 5 (report state) as variant-level structure.
- **`WorkflowObservationAnchor`** — timing-specific concrete-field carrier (Q-WAD-S2-Anchor (a) ratified, NOT generic parametric (b)): `subject_stable_id` + `artifact_digest` + `producer_id` + `observer_id` + `prover_id` + `attached_at_epoch_ns` + `workflow_run_id`. Owns invariants 1-4 (subject identity / artifact digest / producer-observer-prover identity / attachment timestamp + run id). Promotion-to-generic on second-consumer (ProofReceipt per ctrl#369) landing is bounded refactor (rename + add `<Subject, Source>` parameters); per `feedback_strict_mirror_vs_novel_substrate_fact` defer-shape pattern is correct here.
- **`TimingObservationSet`** — collection-typed carrier; aggregation surface separate from per-observation `Lens<TimingMeasurement>`.
- **`TimingBudget`** — declared budget for `Enforce`-mode lens application (parallel to `AsymptoticClass`, `SymbolicCost`).

Lens declaration per Q-WAD-S2-LensC ratification: `Lens<TimingMeasurement>` (per-observation; aggregation handled by `TimingObservationSet` separately).

**Invariant 6 (fail-closed) carriage**: structurally NOT on either anchor or `TimingMeasurement`; lives at `LensEnforcement.violates` layer. Worker scope (gate #55a) lands invariants 1-5 on anchor + TimingMeasurement; **gate #55b** (LensEnforcement substrate-extension for fail-closed-correct enforcement on non-Observed variants) is separate Substrate-Mgr-scope dispatch (see Gate #55 closure-predicate split section below). The `LensEnforcement<Output, Budget>.violates: fn(Budget, Budget) -> Bool` signature per `src/v3/std/lens_application.dag:99-101` cannot pattern-match `TimingMeasurement` variants; substrate-extension is required for fail-closed-correct observation-driven lens classes (timing first; ProofReceipt second).

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

Author parametrically with the six invariant fields as concrete fields on the carrier and `Subject` + `Source` as type parameters for the variable identity / payload parts:

```
type WorkflowObservationAnchor<Subject, Source> {
  subject:               Subject              // invariant 1: stable subject identity (NOT span)
  artifact_digest:       ContentDigest        // invariant 2: observed-artifact identity/digest
  producer_id:           ProducerIdentity     // invariant 3a: producer identity
  observer_id:           ObserverIdentity     // invariant 3b: observer identity
  prover_id:             ProverIdentity       // invariant 3c: prover identity
  attached_at:           Timestamp            // invariant 4a: attachment timestamp
  workflow_run_id:       RunId                // invariant 4b: run id
  observation_outcome:   ObservationOutcome<Source>  // invariant 5: report state (Observed/Missing/Ambiguous/Stale)
  // invariant 6 (fail-closed) lives at the LensEnforcement.violates layer, not here
}
```

Timing instantiation: `WorkflowObservationAnchor<BehaviorId, TimingPayload>` with `TimingPayload = { nanoseconds: Int }`. Subject identity owned solely by the anchor's `subject` field (P2 single-authority); observation payloads carry observed values only, not subject facts. **5 of 6 gate #55 invariants are concrete fields on the anchor (1-5); invariant 6 (fail-closed enforcement on non-Observed) is NOT an anchor field — it lives at the `LensEnforcement.violates` layer and is only satisfiable once Q-WAD-S2-Output (a)(ii) or (a)(iii) substrate-extension lands.** Without that substrate-extension, gate #55 cannot close: closing on 5/6 invariants would let consumers attach observation facts without fail-closed evidence, violating P2 facts-flow-forward + P3/C-8 fail-closed. **Gate #55 closure depends on Director-tier `LensEnforcement` substrate-extension disposition** (canvas:118 escalation thread).

- **Pro**: gate #55 closure predicate ("`WorkflowObservationAnchor` factored separately as reusable external-data attachment primitive") is satisfied; second-consumer promotion (ProofReceipt per ctrl#369) is zero-substrate-edit (rebind type parameters); six invariants are explicitly carried on the anchor (not abstracted away).
- **Con**: novel-substrate authoring at first consumer; per `feedback_strict_mirror_vs_novel_substrate_fact` this needs canvas justification (which this canvas provides). `ProducerIdentity` / `ObserverIdentity` / `ProverIdentity` / `ContentDigest` / `RunId` types may need substrate sub-carriers if not already declared (worker grep at dispatch).

**RATIFIED (Director #828 c#4412301889 + re-confirmed at c#4413322671)**: **(a) timing-specific concrete fields**. Director's reasoning per `feedback_strict_mirror_vs_novel_substrate_fact` + `feedback_construction_over_ratchets`: introducing parametric novel substrate NOW (when ProofReceipt's actual substrate shape hasn't been authored yet) risks parameterization that doesn't fit second-consumer needs. Worker tidy-raven-610 PR #2360's flat shape is the ratified posture; promotion-to-generic on ProofReceipt landing is bounded refactor (rename + add `<Subject, Source>` parameters; no field redesign).

Mgr-tier preliminary (b) recommendation was over-engineered — chasing hypothetical second-consumer parameterization. Discipline lesson: defer-shape pattern is structurally appropriate when (1) field-level reusability is satisfied (worker's anchor fields are already generic external-data-attachment fields modulo type-rename) + (2) promotion is bounded refactor + (3) second consumer is named-but-not-authored.

**Invariant 5 (report state) NOT on anchor — RATIFIED posture**. Director's (a) ratification accepts that invariant 5 lives on the lens Output type (`TimingMeasurement` variants in worker's PR #2360), not as an `observation_outcome` field on the anchor. The single-authority concern is resolved at the LensEnforcement layer per (a)(ii) full-signature extension (c#4413284764) — `violates: fn(Output, Budget, Budget) -> Bool` pattern-matches Output variants for fail-closed enforcement directly. No `ObservationOutcome` projection wrapper needed; folded carrier IS the report state. See Q-WAD-S2-Output below for (a)(ii) ratification + folded-carrier compatibility (Director #828 c#4413322671).

This collapses Q-WAD-S2-Output (c) folded-into-payload back into anchor-side ObservationOutcome carriage — payload is the Observed value, `ObservationOutcome<Source>` wraps it on the anchor. Per canvas:118 fix, downstream `violates`-path resolution is (a)(ii) or (a)(iii) substrate-extension (NOT the rejected (a)(i) Budget fabrication). Trade-off: (c) was simpler at carrier-count but split P2 invariant 5 authority; canvas-(b)-revised + anchor-side ObservationOutcome keeps single-authority on anchor at cost of one extra type + Director-tier `LensEnforcement` substrate change.

## Question 3 (Q-WAD-S2-Output) — Output projection shape

Per r3-structure.md:168, `Output` is "projection/report distinguishing `Observed | Missing | Ambiguous | Stale` (fail-closed enforcement on non-observed states)".

### Option (a) — Sum type: `TimingObservationOutcome = Observed | Missing | Ambiguous | Stale`

Direct sum-type encoding. Each variant carries shape-appropriate payload (`Observed { value: Source }`, `Missing { reason }`, `Ambiguous { candidates }`, `Stale { observed_at, expired_after }`). For timing instantiation, `Source = TimingPayload { nanoseconds: Int }` — the **payload-only** observed value, NOT the full `TimingMeasurement` (which would re-introduce the variants and split P2 single-authority for invariant 5). Per Q-WAD-S2-Anchor revised convergence: subject identity / attestation / report-state all live on the anchor; payload carries only the observed-value-when-present.

**Dissolution classification (INVARIANTS P5 / modeling-discipline.md §coproduct dissolution): 🟢 GREEN (terminal).** No richer source exists at the workflow-observation boundary. The four variants exhaust the externally-attested observation states: a fact was attached cleanly (Observed), no fact attached (Missing), multiple conflicting facts attached (Ambiguous), a previously-attached fact has expired its validity window (Stale). All variants trace to the substrate's external-data attachment surface (six invariants per gate #55); none has a richer-source-extraction path that would dissolve to a finer coproduct. Worker `.dag` declarations MUST carry the 🟢 marker comment per modeling-discipline.md:132 checkpoint discipline.

- **Pro**: direct match to gate #55 invariant 5 ("stale/ambiguous/missing/observed report states").
- **Con**: `LensEnforcement<Output, Budget>.violates: fn(Budget, Budget) -> Bool` (per `src/v3/std/lens_application.dag:100`) does NOT see raw Output — it sees projected Budget values. Three sub-options:
  - **(i) Projection fabricates a violating Budget** for non-Observed variants. **REJECTED per INVARIANTS P3/C-8 fail-closed discipline**: erases the typed report-state failure evidence (Missing / Ambiguous / Stale) into a plausible-looking Budget value. Consumers downstream of `violates` cannot distinguish "budget was exceeded by an observed value" from "no observation existed" — failure-cause introspection is destroyed at the projection boundary. Per PR #2333 inline review at canvas:118.
  - **(ii) Substrate-extend `violates` signature** to `fn(Output, Budget, Budget) -> Bool` so the per-lens body can pattern-match on Output report-state and return true (violate) for any non-Observed variant with the typed evidence preserved up to the call site. Canvas-tier substrate change to `LensEnforcement` carrier — cascades across all lens consumers (T-LBP / T-CostLens / T-LAS). **Director-tier disposition required** if this is the chosen path.
  - **(iii) Result-typed projection**: projection step returns `Result<Budget, ObservationFailure>` and the lens framework adds an `auto_violate_on_left: Bool` bit (or equivalent fail-closed-by-construction wiring) so non-Observed Result-Left automatically counts as violate without a Budget value. Smaller substrate change than (ii) — adds one bit to `LensEnforcement` rather than changing `violates` signature.

### Option (b) — `Result<TimingMeasurement, ObservationFailure>` with `ObservationFailure` enum

Two-level: outcome is `Result`, failure is enum. Shifts the ambiguous/stale/missing distinctions one level down.

- **Pro**: matches stdlib-Result-style intuition.
- **Con**: extra indirection; gate #55 acceptance language reads as flat-sum (Observed *and* the failure variants are peer report states); fail-closed is more verbose to express.

**Recommended (Mgr-tier — corrected per PR #2333 inline review at canvas:118)**: **(a) with sub-option pending Director disposition** between (a)(ii) full-signature-extension (canvas-tier; cascades across all lens consumers) and (a)(iii) Result-typed projection with `auto_violate_on_left` bit (smaller substrate change). **(a)(i) Budget fabrication is REJECTED** per P3/C-8 fail-closed — erases typed report-state failure evidence. Director-tier disposition required.

### Option (c) — Output-folded-into-carrier (worker tidy-raven-610 PR #2360 shape)

Worker authored `TimingMeasurement = Observed { nanoseconds: Int } | Missing | Ambiguous | Stale` directly — the carrier IS the report state, no separate Output projection.

**Dissolution classification: 🟢 GREEN (terminal)** — the four variants exhaust the externally-attested observation states. Worker `.dag` MUST carry the 🟢 marker comment. `Lens<TimingMeasurement>` is per-observation; `TimingObservationSet` is separate aggregation.

- **Pro**: state-space discipline — carrier admits exactly the legal states (no illegal-state-via-projection-mismatch surface). Substrate carrier count is lower (no separate Output projection type). Per Director ratification reasoning (`feedback_no_rejected_patterns` + `feedback_state_space_vs_behavioral_invariants`): the carrier IS the coordinates the projection would have produced; folding dissolves an artificial separation.
- **Con (fail-closed structural)**: does NOT sidestep the `violates: fn(Budget, Budget) -> Bool` signature collision. `TimingMeasurement` variants reach `LensEnforcement.project: fn(Output) -> Budget` first; `violates` body cannot pattern-match the variants because the signature only sees the projected Budget. Per P3/C-8 fail-closed: non-Observed variants need typed-evidence preservation through to the violation-decision boundary, NOT Budget fabrication. **Therefore (c) STILL requires a substrate-extension path** — either (a)(ii) violates-signature extension or (a)(iii) auto-violate-bit on `LensEnforcement` — for fail-closed-correct enforcement on non-Observed.
- **Status**: **(c) RATIFIED by Director at #828 c#4412301889** as the carrier-shape disposition. Worker scope (gate #55a) lands the carrier shape; the substrate-extension for fail-closed-correct enforcement is **gate #55b** (separate Substrate-Mgr-scope dispatch; Director-tier sub-disposition between (a)(ii) and (a)(iii) still pending — see escalation thread at #828 c#4412879231 / c#4413178095).

## Question 4 (Q-WAD-S2-Placement) — Carrier placement: `src/v3/std/` vs `dsl/extdeps/`

Per Director STOP-and-PING standby (#828 c#4411989488): "carrier-placement decisions are well-precedented (compare T-LBP / T-CostLens / T-LAS history); no pre-ratification needed."

**Critical convention** (per `src/v3/std/lens.dag:17-21`): v3-only carriers stay under **`src/v3/std/`**, NOT `dsl/std/`, until substrate graduation. The lens-framework carrier `Lens<C>` itself lives at `src/v3/std/lens.dag` for this reason. T-LBP / T-CostLens / T-LAS lens substrate live in `src/v3/std/` (e.g., `src/v3/std/lens_application.dag`, `src/v3/lenses/cost.dag`). The earlier canvas claim that "T-LBP / T-CostLens / T-LAS lens carriers all live in `dsl/std/`" was **wrong** — they live in `src/v3/std/` and `src/v3/lenses/`.

Two options:

### Option (a) — `src/v3/std/timing_lens.dag` (correct v3 layer)

Place all four carriers (`TimingMeasurement`, `TimingObservationSet`, `WorkflowObservationAnchor`, `TimingBudget`) + lens declaration in a new `src/v3/std/timing_lens.dag` file.

- **Pro**: matches the explicit v3-only-carriers convention at `src/v3/std/lens.dag:17-21`; precedent from T-LBP / T-CostLens / T-LAS lens substrate location. Substrate-graduation trigger (when v3 substrate types graduate into shared `dsl/std/`) is the future move-to-dsl-std event, not Slice 2's call.
- **Con**: WorkflowObservationAnchor's first concrete *workflow* is GitHub Actions CI (per Slice 1 placement at `dsl/extdeps/github/actions.dag`); split between substrate (v3) and workflow grammar (extdeps) is structural — workflow grammar IS provider-specific, lens substrate is NOT.

### Option (b) — `dsl/extdeps/github/actions.dag` (extends Slice 1 placement)

Fold the four carriers into the existing `dsl/extdeps/github/actions.dag` adjacent to Slice 1 carriers.

- **Pro**: workflow-coherent; all workflow-related substrate in one file.
- **Con**: timing-lens carriers are NOT GitHub-Actions-specific; placing in `dsl/extdeps/github/` would require a future migration when second runner (e.g., GitLab CI, Buildkite) lands.

**Recommended (Mgr-tier — corrected per PR #2333 inline review)**: **(a)** — `src/v3/std/timing_lens.dag`. Matches the v3-only-carriers convention at `src/v3/std/lens.dag:17-21` and existing T-LBP / T-CostLens / T-LAS lens substrate placement. Workflow grammar (GitHub-specific) staying at `dsl/extdeps/github/actions.dag` remains correct; lens substrate (v3-internal) belongs in `src/v3/std/`. Worker tidy-raven-610 PR #2360 already placed at `src/v3/std/timing_lens.dag` ✓ — matches corrected recommendation.

## Six invariants (gate #55 acceptance — `WorkflowObservationAnchor`)

Per r3-structure.md:169 (verbatim, for worker brief reference):

1. **Stable subject identity** — not span-based; identity is structural (e.g., BehaviorId), not source-position.
2. **Observed-artifact identity/digest** — content-addressed; stale detection works against digest, not timestamp alone.
3. **Producer/observer/prover identity** — three roles distinguishable; producer = workflow runner, observer = measurement-capture process, prover = signature/attestation source.
4. **Attachment timestamp + run id** — both fields required; run id provides workflow-scope, timestamp provides ordering.
5. **Stale/ambiguous/missing/observed report states** — per Q-WAD-S2-Output (a) above; flat sum.
6. **Fail-closed enforcement on non-observed/non-valid states** — does NOT live on the anchor; lives at the `LensEnforcement` layer per Q-WAD-S2-Output (a)(ii) signature-extension or (a)(iii) auto-violate-bit substrate-extension. Per `feedback_fail_closed_discipline` + INVARIANTS C-8.

## Gate #55 closure-predicate split

Per PR #2333 inline review at canvas (b526a26f finding 1): gate #55 mixes anchor-fact-carriage with fail-closed semantics. Split:

- **Gate #55a** (anchor-fact-carriage): `WorkflowObservationAnchor` declared with invariants 1-4 as concrete fields (subject identity / artifact digest / producer-observer-prover identity / attachment timestamp + run id) per Director-ratified (a) timing-specific shape. Invariant 5 (report state Observed/Missing/Ambiguous/Stale) lives on the lens Output type `TimingMeasurement` variants, NOT on the anchor — single-authority preserved at the lens layer (consumed by `violates` per (a)(ii)). Closes when worker carrier lands per PR #2360.
- **Gate #55b** (fail-closed enforcement): `LensEnforcement` substrate-extension landed per Q-WAD-S2-Output (a)(ii) Director ratification at #828 c#4413284764 — `violates: fn(Output, Budget, Budget) -> Bool` signature change cascading across T-LBP / T-CostLens / T-LAS impls (semantic-equivalent updates) + timing-lens `violates` body pattern-matches `TimingMeasurement` variants, returning true for Missing/Ambiguous/Stale (fail-closed without Budget fabrication, preserving typed report-state evidence per P3/C-8).

Per Director (A)-modified disposition at #828 c#4413322671: gates #55a + #55b co-close in the LensEnforcement (a)(ii) extension PR (Substrate Mgr scope) once PR #2360 (folded-carrier worker scope) merges first. Worker scope = #55a anchor + carrier landing; Mgr-tier (a)(ii) extension PR adds the `violates` Output-arg signature + timing-lens fail-closed body. No separate dispatch for #55b; same PR as the substrate-extension.

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
