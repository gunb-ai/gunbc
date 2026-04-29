# R2 Closure Ledger

**Status:** LIVE (spawned with R2 Release Manager per [`docs/briefs/r2-release-manager.md`](briefs/r2-release-manager.md) owned deliverable #9 "Closure ledger" + acceptance gate `r2_closure_ledger_landed`).

**Authority:** R2 Release Manager is the single owner of this artifact. Other managers signal lane-close in to this ledger via the protocol below; they do not edit ledger rows directly. The ledger is the single sink for "is this lane structurally accepted yet?" across the 6 other R2 managers.

**Discipline source:** structural-acceptance-per-lane-close (Director-locked 2026-04-28, [`docs/r2-structure.md`](r2-structure.md) "Orient before reading"). **The demo IS the structural acceptance gate.** Each row's "Gate" column names the `.dag` gate whose firing demonstrates the lane closed correctly — not a separate process artifact.

**Scope of this doc:**
- Per-manager rows tracking lane-close / sub-lane / item / class status with structural-acceptance gate names.
- Sub-gate decomposition for `T-LensProducer-Retirement` (one R3-continuation lane carrying three internal sub-gates per Director cascade Item 8).
- Reserved incoming surface for R1 closure / R1C-B strict-receipt rows when Worker A+B land structural fixtures (sleek-pike #1164 / bold-wolf #1163), absorbed from R1 manager without table reshape.
- Signal-receiver protocol — how lane-close signals enter, what counts as a receipt, cadence touchpoints with velocity-tripwire reporting.

**Out of scope:** authoring lane-level structural gates (owned by lane-owning managers); per-brief paired-dispatch enforcement (lives at each manager's authoring point per [`docs/r2-structure.md`](r2-structure.md) "P5 dispatch-discipline applies to all manager-authored briefs"); script/CI automation of ledger rendering.

---

## Status legend

| Enum | Meaning |
|---|---|
| `not-started` | Lane scoped, no worker dispatched. |
| `in-flight` | Worker(s) dispatched; PR(s) open or iterating. |
| `green` | Structural-acceptance gate fires; lane is closed by construction. |
| `r3-continuation` | R2-scope work green; lane continues into R3 with R3-scoped sub-gates tracked here until R3 standup. |

Granularity column distinguishes whether the row tracks a full lane, a sub-lane, an item, or a class — matches the manager brief's own scope decomposition.

**Gate-string convention.** "Gate" cells are **descriptive identifiers** for the structural-acceptance gate the lane must fire to close. Until a ROADMAP gate-name alignment pass, treat unfamiliar strings as descriptive placeholders rather than ratchet names; the lane-owning manager is the authority on the canonical `.dag` gate string and updates this column on lane-close signal arrival.

**`in-flight` vs `not-started` convention.** `in-flight` requires an active worker PR (open or landed partial) or substrate landings cited in `Last signal`. A row whose only signal is "worker brief authored" stays `not-started` until the first worker PR opens.

---

## Per-manager rows

### Substrate Manager — T-Substrate + B4

**Manager brief:** [`r2-substrate-manager.md`](briefs/r2-substrate-manager.md). **R3 continuation:** T-CostLens-Composition sub-gate progress reported here through R3 standup.

| Identifier | Granularity | Scope | Gate (demo = gate) | Status | Last signal | Notes |
|---|---|---|---|---|---|---|
| T-Substrate-Cardinality | sub-lane | int-literal magnitude carrier (unblocks Modeling int-lit) | substrate-fact procedure landing per [`INVARIANTS.md`](../INVARIANTS.md) §P1 | not-started | — | Modeling int-lit consumer-side waits on this. |
| T-Substrate-NominalOpaque | sub-lane | `Secret<T>` nominal-opaque (unblocks Modeling `Secret<T>`) | `nominal_opacity_fail_closed` (#937 landed core; carrier completion pending) | in-flight | #937 (core fail-closed) | Modeling consumer brief authored. |
| T-Substrate-ParametricAlgebra | sub-lane | `Dimension<Carrier>` parametric attachment (unblocks Modeling Dimensions) | `dimension_phantom_unit_mismatch_structural` | not-started | — | Modeling phantom-worker brief authored. |
| T-Substrate-ValueBody-list/sum | sub-lane | top-level `ValueBody` list/sum + `std.unicode` bootstrap | `value_body_map_carrier_landed` (Map landed via #1017 + #1068; list/sum pending) | in-flight | #1017, #1068 | **Prereq for R1C-A Sub-deliverable A** — pre-R1-close spawn unblocks R1 closure (per `r2-structure.md` Transition mechanics step 4 carve-out). |
| T-Substrate-Lens-Primitive | sub-lane | `Lens<C>` parametric in `dsl/std/lens.dag` + `fold_lens<C>` machinery; migrate 4 PROXY/STUB lenses | `lens_primitive_landed` + `cost_lens_is_lens_instance` + `complexity_lens_is_lens_instance` + `idempotency_lens_is_lens_instance` + `parallelism_lens_is_lens_instance` | not-started | — | Spec at [`design-lens-framework.md`](design-lens-framework.md). Director-locked 2026-04-28: pure monoidal; reuses `DimensionReport<Carrier>`; meta-lens deferred post-R3. |
| B4 Phase 1 carriers | sub-lane | 4 carriers (`DeclarationRef` consumer migration, fold-shape, emit-helper, extdeps-fixture-set) | `b4_phase1_carriers_landed` | in-flight | B4.2 first-consumer wiring landed | Sub-briefs B4.1 through B4.4. |
| B4 Phase 2 site dissolutions | sub-lane | 8 site dissolutions | `b4_phase2_site_dissolutions_landed` | in-flight | #1069 (B4.8 site dissolution) | Sub-briefs B4.5 through B4.12. |
| **R3-continuation: T-CostLens-Composition** | sub-gate progress | composition lens over `Lens<C>` (consumes T-Substrate-Lens-Primitive) | tracked in R3 ledger at standup; reported here as **sub-gate progress** until then | not-started | — | Gated on T-Substrate-Lens-Primitive close. |

**Watch:** if Substrate workers idle >7 days waiting for Substrate-authored briefs, R2 Release Manager surfaces split-trigger to Director per [`r2-structure.md`](r2-structure.md) §"Watch condition (split trigger)" — recommend dedicated B4 Identity-Carrier Manager.

### Modeling Manager — T-Modeling

**Manager brief:** [`r2-modeling-manager.md`](briefs/r2-modeling-manager.md). **R3 disposition:** archives at R2 close.

| Identifier | Granularity | Scope | Gate (demo = gate) | Status | Last signal | Notes |
|---|---|---|---|---|---|---|
| Int-lit magnitude | item | surface int-literal magnitude at concept layer | `int_literal_magnitude_at_concept_layer` | not-started | worker brief authored ([`r2-modeling-int-lit-magnitude-worker.md`](briefs/r2-modeling-int-lit-magnitude-worker.md)) | Consumes T-Substrate-Cardinality. |
| `Secret<T>` graduation | item | nominal-opaque graduation | `secret_nominal_opacity_graduated` | not-started | worker brief authored ([`r2-modeling-secret-graduation-worker.md`](briefs/r2-modeling-secret-graduation-worker.md)) | Consumes T-Substrate-NominalOpaque. |
| `Dimension<Carrier>` phantom | item | typed value wrapper + phantom-parameter unit mismatch | `dimension_phantom_unit_mismatch_structural` | not-started | worker brief authored ([`r2-modeling-dimensions-phantom-worker.md`](briefs/r2-modeling-dimensions-phantom-worker.md)) | Consumes T-Substrate-ParametricAlgebra. |
| Tokenizer charclass phase-2 | item | consumer of T-Substrate ValueBody-list/sum | `tokenizer_charclass_phase2_structural` | not-started | worker brief authored ([`r2-modeling-tokenizer-charclass-phase2-worker.md`](briefs/r2-modeling-tokenizer-charclass-phase2-worker.md)) | Consumes T-Substrate-ValueBody-list/sum. |

### Grounding Manager — T-Ground

**Manager brief:** [`r2-grounding-manager.md`](briefs/r2-grounding-manager.md). **R3 disposition:** archives at R2 close (target primitives Tier-1 thesis claim closed).

11 lanes per engine-reframe ([`design-emission-model.md`](design-emission-model.md)).

| Identifier | Granularity | Scope | Gate (demo = gate) | Status | Last signal | Notes |
|---|---|---|---|---|---|---|
| T-Ground-Pilot | lane | pilot validates engine sharpened-(b) | `pilot_inhabitance_routing_stability_landed` | green | #765 (merged 2026-04-25) | Toy inhabitance-search engine for Rust integer family + bool + Unit; gates Rust/Python/Go. |
| T-Ground-Rust | lane | Rust target primitives | `rust_target_primitives_structural` | in-flight | #1005 (Rust IntegerRangeFact mirror dissolved) | XL. |
| T-Ground-Python | lane | Python target primitives | `python_target_primitives_structural` | in-flight | #1080 (`primitives.dag`) | L. |
| T-Ground-Go | lane | Go target primitives | `go_target_primitives_structural` | in-flight | `ac765ce10` + #1046 (tranche 1 + additional) | L. |
| T-Ground-LanguageSpec | lane | LanguageSpec language-agnostic | `language_spec_language_agnostic_structural` | not-started | — | M. |
| T-Ground-Coercion-Fold | lane | coercion fold | `coercion_fold_structural` | not-started | — | S. |
| T-Ground-Lifetime-Analyzer | lane | structural lifetime/ownership derivation (replaces retracted Annotation lane) | `lifetime_analyzer_structural` | not-started | — | M. No annotation surface. |
| T-Ground-Diagnostic | lane | diagnostic surface | `target_primitives_diagnostic_structural` | not-started | — | S. |
| T-Ground-CrossTarget-Meta | lane | cross-target meta | `cross_target_meta_structural` | not-started | — | S. |
| T-Ground-Tests | lane | grounding tests | `grounding_tests_structural` | not-started | — | S. |
| T-Ground-Dissolve | lane | dissolution sweep | `grounding_dissolve_structural` | not-started | — | S. |

### Impossible-Bugs Manager — T-ImpossibleBugs

**Manager brief:** [`r2-impossible-bugs-manager.md`](briefs/r2-impossible-bugs-manager.md). **R3 disposition:** archives at R2 close.

| Identifier | Granularity | Scope | Gate (demo = gate) | Status | Last signal | Notes |
|---|---|---|---|---|---|---|
| Nested-optional flatten | class | gated on cardinality refinement | `nested_optional_flatten_compile_error` in `t_impossiblebugs_nested_optional_flatten.dag` (runner: `t_impossiblebugs_nested_optional_flatten_suite_passes_through_runner`) | green | #890 + #962 (impl) + #1173 (class-close / structural test) | **Class closed.** Substrate work #890 + #962; audit + runner-backed TestClaim #1173. No substrate gaps surfaced. |
| Unhandled diagnostic paths | class | Tier 2 substrate | `unhandled_diagnostic_paths_impossible_structural` | not-started | worker brief authored ([`r2-impossible-bugs-unhandled-diagnostic-paths-worker.md`](briefs/r2-impossible-bugs-unhandled-diagnostic-paths-worker.md)) | Substrate-gap escalations route to Substrate Manager. Flips to `in-flight` when first worker PR opens. |
| Unenumerated effects | class | post-effects-design-doc per #808 | `unenumerated_effects_impossible_structural` | green | #971 (unenumerated effects lens landing) | Closed-system effects model is canonical reference. |

### Pure Bootstrap Manager — post-R1 PB

**Manager brief:** [`r2-pure-bootstrap-manager.md`](briefs/r2-pure-bootstrap-manager.md). **R3 continuation:** T-LensProducer-Retirement (3 sub-gates), T-FixedPoint, T-Tier3-Dissolution, 3 distributed bridge retirements — sub-gate progress reported here until R3 standup.

| Identifier | Granularity | Scope | Gate (demo = gate) | Status | Last signal | Notes |
|---|---|---|---|---|---|---|
| Tier 3 mirror dissolutions | sub-lane | termination / computation / induction / effect-carrier Rust mirrors | `tier3_mirror_dissolutions_structural` | in-flight | worker briefs authored ([`r2-pb-tier3-mirror-dissolution-workers.md`](briefs/r2-pb-tier3-mirror-dissolution-workers.md), [`r2-pb-tier3-worker1-termination-mirror-audit.md`](briefs/r2-pb-tier3-worker1-termination-mirror-audit.md)) | Not gated by R1's PB census. |
| Tier 2 patch-lower-helpers retirement | sub-lane | `patch_lower_helpers_generated_type_alias_refinement` | `patch_lower_helpers_retired` | green | #1014 | B7 priority-hint relay completed. |
| `kernel_algebra_profile` mirror dissolution | sub-lane | consumer plumbing on landed `ValueBody::Map` carrier | `kernel_algebra_profile_consumer_plumbed` | in-flight | #1017, #1068 (carrier landed); read-path/API + arrow-body evaluation are Substrate + Evaluator gated | Substrate sub-task + Evaluator-gated on `std.computation` arrow-body. |
| Post-R1 emergent dissolutions | standing | dissolutions surfaced post-R1 spawn | per-emergence gate | not-started | — | Standing intake row. |
| **R3-continuation: T-LensProducer-Retirement** | lane (3 internal sub-gates) | one program; sub-gate granularity per Director cascade Item 8 | `lens_producer_retirement_subgate_1_structural` + `lens_producer_retirement_subgate_2_structural` + `lens_producer_retirement_subgate_3_structural` | not-started | — | **Reports as one row with three sub-gates, NOT three lanes** (per dispatch directive). |
| **R3-continuation: T-FixedPoint** | sub-gate progress | fixed-point lane | tracked in R3 ledger at standup | not-started | — | Evaluator-gated. |
| **R3-continuation: T-Tier3-Dissolution** | sub-gate progress | Tier 3 dissolution at R3 scope | tracked in R3 ledger at standup | not-started | — | Evaluator-gated. |
| **R3-continuation: 3 distributed bridge retirements** | sub-gate progress | 3 PB-side bridges from T-Bridge-Retirement distribution map | tracked in R3 ledger at standup | not-started | — | 2 sibling Substrate-side bridges live in Substrate row. |

### Evaluator Manager — T-Evaluator

**Manager brief:** [`r2-evaluator-manager.md`](briefs/r2-evaluator-manager.md). **R3 disposition:** Evaluator-readiness signal to Director gates R3 spin-up; manager continues into R3 if scope warrants (TBD at R2 close).

5 sub-lanes + PR-A through PR-E design-lock cadence.

| Identifier | Granularity | Scope | Gate (demo = gate) | Status | Last signal | Notes |
|---|---|---|---|---|---|---|
| Runtime value model | sub-lane | typed runtime values for 6 connectives + 5 L1 behaviors | `runtime_value_model_structural` | not-started | — | Lazy/eager strategy = Open call 3. |
| Body evaluator | sub-lane | execute `.dag` function bodies structurally | `body_evaluator_structural` | not-started | — | Bounded forward execution per P4. |
| Lens application | sub-lane | extend `reflect_program_dag_nodes_in_file` to complete reflection | `lens_application_complete_reflection` | not-started | — | Per [`design-reflection-completeness.md`](design-reflection-completeness.md) (LOCKED 2026-04-29). |
| Witness construction | sub-lane | runtime materialization of `Witness::Inhabits` / `Witness::Violates` + algebraic-law witnesses | `witness_construction_structural` | not-started | — | — |
| Cross-target equivalence harness primitives | sub-lane | for L5 verification in R3 | `cross_target_equivalence_harness_structural` | not-started | — | Algebraic equivalence over curated corpus. |
| **PR-A through PR-E design-lock cadence** | progress track | pre-dispatch design lock cadence | per-PR design-lock landing | not-started | — | May run during R1's final week per `r2-structure.md` Transition mechanics step 4. |

---

## Incoming surface — R1 closure absorption (reserved)

When R1 closure work lands structural fixtures, the following rows merge into this ledger **without table reshape** (same column shape: identifier / granularity / scope / gate / status / last signal / notes). Reserved here so the structure can absorb the rows the moment they ship.

**Sources tracked for absorption:**
- **R1C-B T-P0 fixtures** ([`r1c-b-t-p0-fixtures-worker.md`](briefs/r1c-b-t-p0-fixtures-worker.md)) — strict-receipt fixtures `p0_repeat_string_correct` (structural), `p0_repeat_string_v2_oracle_rust_bridge` (interim), `p0_no_fabrication_sentinel`, `p0_rest_ops_aligned`. Worker A (sleek-pike #1164) and Worker B (bold-wolf #1163) land structural fixtures — at that point the R1C-B row absorbs in as a `green` (or `r3-continuation` if interim oracle bridge lingers) sub-lane row under a new "**R1 Residual (absorbed)**" section.
- **R1 closure ledger residuals** — any R1 ledger rows that survive R1 close per [`r2-structure.md`](r2-structure.md) Transition mechanics step 2 ("R1 residual sweep — every open R1 ledger row gets an R1-or-R2 assignment"). R2-assigned residuals merge in under "**R1 Residual (absorbed)**".

**Absorption protocol:** R2 Release Manager opens a section titled `## R1 Residual (absorbed)` below this one when the first R1 row arrives, and adds rows under it using the same column structure. No retroactive reshape of per-manager rows above. The R1 manager signals absorption via the receiver protocol below; absorption is recorded in "Last signal" with the R1 closure PR number.

*(Section currently reserved — empty by design. The first absorbed row creates the section header.)*

---

## Signal-receiver protocol

**Channel.** Lane-close signals flow on the **cross-manager queue** per [`r2-structure.md`](r2-structure.md) "Cross-manager dependency discipline" + the R1 `Cross-manager notifications queued` brief pattern. Mechanically: a comment on the R2 Release Manager's session inbox issue (the standing inbox of the session running the Release Manager role); for human escalations, the GitHub session-inbox issue per [`docs/escalation-paths.md`](escalation-paths.md). **R2 Release Manager remains the single owner of this ledger** — managers do not edit ledger rows directly.

**What managers signal.** Each lane-owning manager sends one of:
- **Lane-close** — the lane's structural-acceptance gate fires; PR landed on `main`. Cite the PR + the gate name.
- **Sub-lane / item / class close** — same as above at finer granularity (matches the manager's own brief decomposition; row above shows which granularity applies per manager).
- **R3 continuation sub-gate progress** — for PB / Substrate R3-continuation rows: report sub-gate landings as they occur; ledger updates the sub-gate row.
- **STOP-AND-ESCALATE** — per [`docs/escalation-paths.md`](escalation-paths.md); routed to Director, but mirrored here as a `last signal` annotation on the affected row so the ledger reflects current state.

**Signal payload (minimum).** Identifier (matches a row in this ledger or under "R1 Residual (absorbed)"); status transition (`not-started` → `in-flight` → `green`, or `green` → `r3-continuation`); PR or brief reference; one-line note for "R3 continuation readiness" where applicable.

**What constitutes a receipt.** A signal is **received** when R2 Release Manager:
1. Updates the affected row's `Status`, `Last signal`, and `Notes` columns in this doc on `main` (single PR or batched per cadence — see below).
2. Acks on the cross-manager queue ("received; ledger updated in PR #N").

Until both happen, the signal is in-flight. A signal that sits >7 days unreceived is itself a velocity-tripwire signal and surfaces to Director.

**Cadence.**
- **On signal arrival** — small batches (1–3 rows) update on the next ledger PR within ≤3 business days; urgent signals (R2-close gate, STOP-AND-ESCALATE mirror) update same-day.
- **Integration-reflection cadence** — per `ROADMAP.md` "Integration-reflection cadence", R2 Release Manager runs a sweep: aggregates pending signals, updates ledger rows in one PR, and **emits a velocity-tripwire reading** (introduction:dissolution PR ratio across all manager-authored work in the cadence window). **≥3:1 fires an alert to Director** per [`INVARIANTS.md`](../INVARIANTS.md) §P5(c). Manual sweep for dissolution-bearing feature PRs runs first per the calibration caveat in the Release Manager brief.
- **Weekly health check** — closure-ledger summary to Director: lanes within 1 step of unblocking; managers blocked on cross-program signals; lanes >7 days without movement; bottleneck-watch reading on Substrate Manager.

**Authority discipline.** Release Manager is the **single ledger owner**, not a parallel decision-maker:
- Does not author or contest lane-level structural-acceptance gates — those are owned by lane-owning managers per the structural-acceptance-per-lane-close discipline.
- Does not adjudicate cross-program scope conflicts — those route to Director per [`r2-structure.md`](r2-structure.md) "Director (cross-program coordinator)".
- Does not relitigate R1 PB census semantics on absorbed R1C rows — ROADMAP is single authority on R1 gate close per [`r2-structure.md`](r2-structure.md) §"Pure Bootstrap Manager" ("R1 vs R2 boundary — defers to ROADMAP gate authority").

**R2-close signal emission.** When **all 6 other managers' R2-scope lanes** are `green` (Modeling and Impossible-Bugs fully green; Substrate / PB green on their R2-scope lanes with R3-continuation rows still active; Grounding 11 lanes green; Evaluator 5 sub-lanes green), R2 Release Manager fires `r2_close_signal_to_director_authored` per the Release Manager brief acceptance gate, with R3 continuation readiness summarized from the `r3-continuation`-status rows in this ledger. **R3-continuation rows do NOT gate `r2_close_signal_to_director_authored`** — they are R3-scope work tracked here until R3 standup, surfaced as readiness signal alongside the R2-close fire, not as a blocker.

---

## Audit debt — `std.verification` deferral predicates (non-blocking)

**Recorded:** 2026-04-29 (Director clarification: gunb-ai/gunbc#1130; #1179 direction ratified — separate TC1 fixture + `SubstrateResearchDeferredClaim`).

| Topic | Status | Notes |
|---|---|---|
| Unify vs split deferral carriers | open (design hygiene) | Today: `ReleaseDeferredClaim` is runner-valid only for `r1_release_acceptance.dag` (R1 release-acceptance discipline); `SubstrateResearchDeferredClaim` is runner-valid only for `tc1_substrate_lens_eta_equivalence_deferred.dag` (TC1 / R2 substrate research). **Open question for a future R2 pass:** collapse into one `DeferredAcceptanceClaim` with a discriminator (fixture / authority lane), or keep separate predicates with fixture-scoped runner gates. Either path preserves current fail-closed behavior. **Not a merge blocker for #1179.** |

---

## Cross-refs

- Parent: [`docs/r2-structure.md`](r2-structure.md) §"R2 Release Manager" + §"Manager structure" + §"Cross-manager dependency discipline" + §"Escalation signal channel".
- Owner brief: [`docs/briefs/r2-release-manager.md`](briefs/r2-release-manager.md) — owned deliverable #9 "Closure ledger" + acceptance gate `r2_closure_ledger_landed`.
- Discipline framework: [`INVARIANTS.md`](../INVARIANTS.md) §P5 "Dispatch-Discipline Mechanisms" (a)/(b)/(c).
- Cadence wiring: `ROADMAP.md` "Integration-reflection cadence" + velocity-tripwire reporting.
- Escalation channel: [`docs/escalation-paths.md`](escalation-paths.md).
- Manager-brief authority matrix: [`docs/briefs/r2-manager-brief-authority-matrix.md`](briefs/r2-manager-brief-authority-matrix.md) — ledger entries are the "standing reporting duty" artifact category.
- R3 continuation pattern: [`docs/r3-structure.md`](r3-structure.md) §"Manager structure".
- R1 closure absorption source: [`docs/briefs/r1-closure-manager.md`](briefs/r1-closure-manager.md) (R1C-B fixtures + R1 residual sweep).
