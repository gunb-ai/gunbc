# R2 Structure

**Status:** `PROPOSAL` — pending user sign-off + R1 closure + promotion to `ROADMAP.md` as `## Release R2 Program` section.

**🔄 AMENDED 2026-04-28** — R2 expanded to include **Evaluator (Goal 7)** + full Grounding (Rust XL + Python L beyond original Pilot+Go scope). R3 reframed from *escape-hatch only* to a **structured Thesis Closure / Consequence Cycle** program — see [`docs/r3-structure.md`](r3-structure.md). Justification: at gunbc multi-agent dispatch velocity, the Evaluator + Rust + Python lanes are ~2-4 weeks of additional R2 work (vs. months at solo-dev sizing), and folding them into R2 gives a single shippable artifact with the *capacity layer* of the thesis fully landed. R3 then runs the *consequence layer* (Tier 3 dissolution, lens-producer retirement, L4-L7 verification, fixed-point, Int128, Shape B demos) as mechanical follow-through.

**🔄 AMENDED 2026-04-28 (engine reframe)** — `T-Ground-Engine` framing is superseded per [`docs/design-emission-model.md`](design-emission-model.md). THESIS:171 is explicit: "Coercion = emission. No separate coercion engine." The prior lane name implied an authority that picks between targets when structure under-determines — exactly what THESIS rules out. The reframe replaces the single Engine lane with **5 substrate-completion lanes** (T-Ground-Coercion-Fold + T-Ground-LanguageSpec + T-Ground-Lifetime-Analyzer + T-Ground-Diagnostic + T-Ground-CrossTarget-Meta) that surface the real modeling problems the engine framing was hiding. Net effect: lane count grows from 7 to 11; total scope is the same or slightly larger; *visibility* of scope is much higher. See [`docs/design-emission-model.md`](design-emission-model.md) for the modeling-problem decomposition + Open call 1 (Director sign-off needed).

**Authority:** single-source while open. Amendments before promotion land in this doc. After promotion, amendments follow the same discipline as R1's `## Release R1 Program` section (director-authored PRs with manager acknowledgement).

**Scope naming note:** `docs/db-history/db-18.md` uses "R2 carrier" as internal DB-stage nomenclature that predates release-level R# naming. Our release-level R2 (this doc) is unrelated to DB-18's stage label; no collision of meaning, just of string.

## Summary

R2 is the **capacity-layer close** — where "capacity" means *the substrate, runtime, and grounding that everything else falls out of*. R1 closes under all-R1-gates-green (see R1 closure criteria below), and that closure carries lens purity, self-hosting shim-floor close, E-family carrier closure, and tests-as-data closure out of R2's scope entirely. R2 is what's left of the capacity layer: **Grounding Completeness** (Tier 1 thesis claim — Rust + Python + Go target primitives structurally modeled), **Evaluator** (the runtime that executes `.dag` bodies and unblocks the consequence layer in R3), modeling-faithfulness dissolution, scoped substrate prereqs, remaining R2+ impossible-bug classes, and §6a per-method-metadata follow-through.

R3 ([`docs/r3-structure.md`](r3-structure.md)) is the **consequence layer** — every thesis claim that becomes mechanical once R2 lands: Tier 3 mirror dissolution, lens-producer retirement, L4-L7 verification harness, self-hosting facet 2 fixed-point, Tier 2 Int128, Shape B omni-emission demos, Anthropic typed wire. The R2/R3 split is structural: **R2 lands what requires new substrate / runtime / design; R3 runs what falls out mechanically.**

Two framing decisions drive scope + coordination:

1. **Anti-deferral principle.** If dissolution direction is clear and named, deferral is problem-finding, not problem-solving. R2 absorbs what has named dissolution directions, regardless of current execution velocity. (Velocity is a trailing observation; it can accelerate or slow between waves. The principle is what's load-bearing.)
2. **Light-touch throughput-oriented coordination.** Manager count = concurrent critical paths, not total scope.

## Program count — 3 active releases (R1 closing, R2 + R3 sequenced)

- **R1** closing.
- **R2** = **capacity-layer close** (substrate + Evaluator + Grounding + modeling + impossible-bug classes + framework).
- **R3** = **consequence-layer close** — Thesis Closure / Consequence Cycle per [`docs/r3-structure.md`](r3-structure.md). Runs Tier 3 mirror dissolution, lens-producer retirement, L4-L7 verification harness, self-hosting facet 2 fixed-point, Tier 2 Int128/Word128 substrate, Shape B omni-emission demos, Anthropic typed wire. Sequenced after R2; pre-R3 brief authoring during R2 final week mirroring R2's pre-R1-close pattern.

**🔄 SUPERSEDES** the prior framing of R3 as *"reserved as escape hatch only"*. Justification: thesis-claim mapping ([`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md), closing Open call 1 below) shows ~7 Tier-1/Tier-2/Tier-3 thesis claims become mechanical post-R2-Evaluator. Treating those as escape-hatch obscures the structural opportunity to close them as one cycle. R3 as a structured program is therefore named, scoped, and dispatchable — not an escape hatch.

Post-R3 is external work (adoption, documentation, community, ecosystem modeling) — not on the thesis-claim release ledger. The first named post-R3 stream is modeling what currently lives in `../ctrl/` (user-flagged 2026-04-24) as a practical pressure-test for whether the structural thesis R2+R3 closes actually holds when applied to real-program shapes. That pressure-test is not itself a thesis claim; it's a validation exercise against one.

## Goals

**Gate-ownership discipline.** Every R1 gate listed in `ROADMAP.md §"Lane acceptance — .dag gates"` closes in R1 under the locked all-R1-gates-green criterion (see R1 closure criteria below). That means concerns gated there — **lens purity** (`lens_producer_files_remaining` on T-PB-A via PR #752), **self-hosting shim-floor close** (T-PB-A `pb_hand_rust_at_shim_floor` + `pb_compiler_std_ratchet_zero` + T-PB-B `pb_rust_tests_outside_residual_zero`), and **E-family carrier port closure** (the T-LaneE critical path enabling `complexity_merge_sort_is_nlogn` + `complexity_merge_sort_v3_matches_v2_oracle` + `lane_e_bundled_witness_host_emit_parity`) — are **R1 scope, not R2**. R2 does not duplicate release authority over gates ROADMAP already assigns to R1 lanes.

Under that discipline, R2's goals are the Tier-1 thesis claims that are *not* gated in R1 today:

1. **Grounding Completeness** — target-side primitive types for Rust/Python/Go structurally declared; coercion via inhabitance search; Track-13 dissolution. Inherits from `ROADMAP.md §"Post-R1 Program — Grounding Completeness"` → promotes to R2 lane `T-Ground`. Not in R1's gate list today; R2 is where it lands.

2. **Modeling-faithfulness dissolution** — three Tier-1 type-refinement gaps close:
   - Surface int-literal magnitude at concept layer (P4 row on `ROADMAP.md`; originating analysis on PR #745)
   - `Secret<T>` nominal-opaque graduation (`ROADMAP.md` post-merge-debt section, 2026-04-23 thesis-doc surface)
   - `Dimension<Carrier>` typed value wrapper with phantom-parameter unit-mismatch enforcement (ibid.)

3. **Substrate prereqs** — named as explicit R2 sub-lanes with **scoped acceptance criteria** (sufficient-to-unblock, not full-capability). Each prereq is pinned to a specific R2 consumer (Goal 2 items + the tokenizer charclass closure inherited from R1 per the named exception in *"Lanes deliberately absent"* below); full substrate-capability lanes retain open design calls that may predate or postdate R2, and this structure does not commit R2 to close them all:

   - **Cardinality-substrate subset sufficient to close int-literal magnitude refinement** — enough cardinality modeling to let `IntLit` carry a magnitude that narrows to target int algebra at reconciliation. Consumer: T-Modeling int-lit (Goal 2). Does NOT commit to the full cardinality-substrate capability (fixed-width-types by-construction, container cardinality bounds in Grounding, etc. — those remain open design calls outside R2 scope unless additional R2 items demand them).
   - **Nominal-opaque substrate sufficient to graduate `Secret<T>`** — enough nominal-type modeling to carry construction-restriction (`where only X may construct`) semantics. Consumer: T-Modeling Secret<T> (Goal 2). Adjacent to DB-11 alias-RHS `where` (landed in R1 via PR #703); may or may not overlap DB-18 territory. Acceptance is `Secret<T>` graduation, not a general nominal-type program.
   - **Parametric algebra attachment subset sufficient to inhabit `Dimension<Carrier>` in an abelian group algebra** — enough substrate capability to let `Dimension<Unit>` carry phantom-parameter arithmetic (propagate through operations, compile error on unit-mismatch). Consumer: T-Modeling Dimensions (Goal 2). Primary authority is `ROADMAP.md §"Post-R1 Program — Grounding Completeness"` (the "Why post-R1" paragraph), which tags this dependency `DB-18 parametric algebra attachment` — but `docs/db-history/db-18.md` currently scopes DB-18 to workflow-effect carrier + Rust reflection (Part 2 shipped) + Go-accessor follow-up (Part 3), not parametric algebra attachment. That mismatch is an existing ROADMAP ↔ db-history inconsistency, not one introduced by this doc; a pre-promotion DB-lane rename or new DB number may be warranted. R2 acceptance is: `Dimension<Unit>` phantom-parameter arithmetic compiles with unit-mismatch errors, independent of the DB-tag the substrate ends up carrying.
   - **Top-level `ValueBody` list/sum subset + `std.unicode` bootstrap inclusion** — enough Class 5 Gap 3 substrate capability for `data ascii_scan_order: List<CharClass> = [Whitespace, Digit, IdentStart, IdentContinue]` (and similar list/sum top-level data declarations) to lower structurally (rather than fall to `ValueBody::Unparsed` and trigger R14 hard-fail), plus the `Dag::new()` bootstrap/load-set decision that makes `std.unicode::CharClass` and other extdeps-language declarations resolvable from compiler authorities. **Two named consumers, both of which share this list/sum substrate work**:
     1. **Tokenizer charclass phase-2** — original consumer per PR #762 reclassification (R1 T-Sub deferred this phase to substrate per Surface Manager handoff 2026-04-24). Shape: `data ascii_scan_order: List<CharClass> = [...]` (list of sum).
     2. **Engine sharpened-(b) full pilot enumeration** — surfaced by R2 Grounding loader-close worker (`clever-owl-123`) probe on PR #776 (`rust_pilot_primitives: List<RustPrimitive>` lowers as `Declaration` but `value_body` is `Unparsed(SourceSpan)`). Shape: list of sum (same substrate work as charclass).
     
     **Excluded from this sub-lane (substrate shape differs)**: `kernel_algebra_profile` mirror dissolution (existing P2 drift ratchet at `dag.rs:1530`) is a `Map<String, AlgebraProfile>` body — **map-shaped, not list-of-sum**. **Status (post-PR #1017, per PB Manager review 2026-04-28):** the `ValueBody::Map` carrier itself **landed** via #1017 — `kernel_algebra_profile` now lowers as `ValueBody::Map`. The remaining blockers are **map read-path/API decision** + **`std.computation` arrow-body evaluation** (Evaluator-gated), not the carrier. Tracked as a sibling sub-lane scope; not bundled here per substrate-shape honesty (codex BLOCKING on PR #782 caught this scope conflation; corrected 2026-04-25).
     
     Does NOT commit to the full Class 5 Gap 3 substrate-capability close; scoped to the list/sum subset for the two named consumers. ROI is 2× the original single-consumer scoping (tokenizer + Engine), not 3× as initially framed.

4. **Remaining R2+ impossible-bug classes** — three classes currently tagged `[R2+]` in `ROADMAP.md §"Lane acceptance — .dag gates"` (T-Demo row; THESIS §"Enumerable impossible-bug classes" is the authority on scheduling tags):
   - Nested-optional flatten
   - Unhandled diagnostic paths
   - Unenumerated effects

5. **§6a per-method-metadata** — **Design call closed** for R2 program purposes per [`docs/design-substrate-carrier-port-program.md` §6a](design-substrate-carrier-port-program.md): **Option 3, unified `MethodContract` carrier**, with dissolution trigger and live receipt recorded there. Options (0)–(3) remain in §6a for audit context only. **Receipt:** `src/v3/std/algebra.dag` declares `MethodContract`; `src/v3/lenses/cost.dag` imports it and defines `method_contract_cost_shape` as the minimal demo consumer (matches §6a **Live receipt** at design-doc HEAD). **R2 remainder** is **follow-through** only — bulk migration of `cost.dag` / `complexity.dag` to live call-site `MethodContract` lookup plus dissolution-trigger tracking — owned by R2 Release Manager per [`docs/briefs/r2-release-6a-follow-through-worker.md`](briefs/r2-release-6a-follow-through-worker.md), not a second open **pick**. If work blurs into parametric-algebra / substrate producer vs consumer-migration reframes, coordinate with Substrate Manager (see GitHub #856). The **pick** itself is not substrate-capability work; §6a explicitly rejects Option 1 for this lane.

6. **R2 closure demo** — simple "it runs" artifact per lane close. **R2 Release Manager-coordinated** (was Director-coordinated under prior 1-manager structure; reassigned under 2026-04-26 rework — single authority per `feedback_node_not_god_struct`). Not a lane — see Demo discipline below.

7. **Evaluator (added 2026-04-28 amendment)** — the runtime that executes `.dag` function bodies, applies lenses structurally, and constructs witnesses. Capacity-layer cornerstone: unblocks 7 of 10 R3 lanes (Tier 3 dissolution, lens-producer retirement, L4-L7 verification harness, fixed-point, Shape B demos). Not a thesis claim by itself; it's the *capacity* that makes those thesis claims mechanically reachable. **Standing as Goal 7 because it's the largest single piece of new substrate work in R2** and warrants its own manager + lane structure. Substrate Manager input required on which carriers the Evaluator needs (closed-over environments, witness construction, lazy/eager strategy) — see Open call 3 below for design challenges to resolve up-front.

## Manager structure

> **🔄 REVISED 2026-04-26.** The original "1 standing manager (Grounding) + Director ad-hoc" decision is **retracted.** Empirical signal: under that structure, Grounding Manager and (the prior) Zero-Floor Manager were idle while Director became the dispatch bottleneck for every other lane. Standing managers without owned deliverables degenerate into pass-through hops; concentrating brief-authoring on Director starved 3 of 4 non-Grounding lanes. Restoring R1's program-manager pattern: **N managers, each owning a complete program with autonomous brief-authoring + dispatch authority through R2 close.** Director coordinates *across* programs but does not author every brief.

**7 standing managers + Director coordinator** (count updated 2026-04-28 — Evaluator Manager added as Goal 7). Each manager owns a mutually-exclusive program with its own deliverables, sub-briefs, and worker dispatch through R2 completion. Director's role is cross-program coordination (handoffs, conflict resolution, scope-change escalation), not brief authoring.

**Cross-manager dependency discipline.** Where one program produces a substrate carrier another program consumes (e.g., T-Substrate ValueBody-list/sum unblocks T-Modeling tokenizer charclass + T-Ground Engine sharpened-(b)), the producing manager owns the carrier landing; consuming managers own the migration. Cross-program handoffs land via the R1 `Cross-manager notifications queued` brief pattern — producing manager signals readiness; consuming managers ack and dispatch consumer-migration work.

**Escalation signal channel.** When a brief instructs `STOP-AND-ESCALATE` or `surface to Director / specific manager`, the channel is: GitHub session-inbox issue comment for human-target escalations (Director's session inbox or the named manager's session inbox); cross-manager queue (per the R1 `Cross-manager notifications queued` brief pattern) for inter-manager signals. Escalation clauses in worker briefs do not need to restate the channel — this clause is the single authority for "where do I surface this?". Cross-cutting union map of all escalation clauses lives at [`docs/escalation-paths.md`](escalation-paths.md).

**Manager-brief authority matrix.** Every deliverable / signal / ledger entry owned by an R2 manager belongs to exactly one of 5 disjoint artifact categories (worker brief / decision brief / cross-manager signal / standing reporting duty / pre-spawn placeholder). Categories are defined at [`docs/briefs/r2-manager-brief-authority-matrix.md`](briefs/r2-manager-brief-authority-matrix.md), which also carries the per-manager deliverable inventory and a local review checklist. Manager briefs cite the matrix as authority for their deliverables' categorization and stop self-categorizing. A deliverable that doesn't cleanly fit one category is a category bug, not a sixth category — surface as matrix amendment.

**P5 dispatch-discipline applies to all manager-authored briefs.** Per INVARIANTS.md#p5-progress-is-dissolution "Dispatch-Discipline Mechanisms" (paired-dispatch + per-PR gate + velocity tripwire), the discipline applies uniformly across all 7 managers — not just Director's ad-hoc dispatches. **Each standing manager is responsible for enforcing P5 discipline on the briefs they author**: every brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance (paired-dispatch); every PR that introduces a hand-Rust file in `src/v3/` (including managers' worker dispatches) fills the per-PR gate naming what it deletes or explicitly defers. R2 Release Manager surfaces violations via the closure ledger + the velocity tripwire (≥3:1 ratio in any 7-day window across all managers), but the per-brief and per-PR enforcement happens at each manager's authoring point, not at a central choke. Codex P2 inline flagged this gap on PR #827 sha `d2bc1eca`; addressed by making P5 universally applicable here.

### 1. Grounding Manager

Owns **T-Ground** sub-program (Goal 1 — Grounding Completeness, the program with R2's only true critical path).

- **Critical path:** T-Ground-Pilot → T-Ground-Rust → T-Ground-LanguageSpec → T-Ground-Coercion-Fold → T-Ground-Tests → T-Ground-Dissolve (per `ROADMAP.md §"Post-R1 Grounding lanes"` after the engine-reframe cascade lands per [`docs/design-emission-model.md`](design-emission-model.md) §"Cascade across upstream docs"). Pre-cascade authority still names `T-Ground-Engine`; this doc supersedes that framing.
- **Fill queue:** T-Ground-Python, T-Ground-Go (2-way parallel after Pilot validates).
- **Cross-program consumer:** Engine sharpened-(b) full pilot enumeration consumes T-Substrate ValueBody-list/sum carrier.
- **Authority:** authors all T-Ground sub-briefs autonomously; dispatches workers; **signals lane-close to R2 Release Manager** (for closure ledger); escalates blockers and scope changes to Director.

### 2. Substrate Manager

Owns **T-Substrate** lane (Goal 3 — substrate prereqs) **and the B4 Identity-Carrier Substrate Pass program** (from #810 synthesis; see `docs/briefs/b4-identity-carrier-substrate-pass.md`). The largest single concentration of R2 work; also the program that unblocks the most consumers.

- **T-Substrate sub-lanes:**
  - Cardinality-substrate subset for int-literal magnitude (unblocks Modeling Manager's int-lit item)
  - Nominal-opaque subset for `Secret<T>` (unblocks Modeling Manager's `Secret<T>` item)
  - Parametric-algebra-attachment subset for `Dimension<Carrier>` (unblocks Modeling Manager's Dimensions item)
  - Top-level `ValueBody` list/sum + `std.unicode` bootstrap (unblocks Modeling Manager's tokenizer charclass phase-2 + Grounding Manager's Engine sharpened-(b))
  - **T-Substrate-Lens-Primitive (added 2026-04-28 per user direction "do this ASAP")** — declares `Lens<C>` parametric type in `dsl/std/lens.dag` combining (read function, monoid: unit/compose/branch/iterate, side-condition; **branch is BranchNode exclusive-choice with max/join semantics, NOT a phantom parallel primitive — v3 has no ParallelNode; auto-parallelism is emergent from BindNode dependency-graph analysis per THESIS §"Free consequences"**); generic fold machinery `fold_lens<C>: Lens<C> → Dag → DimensionReport<C>`; cost-basis-as-substrate-fact discipline. **Migrates 4 existing PROXY/STUB lenses to instances** (`cost.dag`, `complexity.dag`, `idempotency.dag`, `parallelism.dag`) — interface ON TOP of existing work + abstraction of patterns already there (`combine_max` / `combine_sequential` are already monoidal). Director-locked design decisions (per Director response on #1078): pure monoidal (not stateful — memory-peak is separate framework); reuses existing `DimensionReport<Carrier> = DimensionOk { dimension_name, composed, witnesses } \| DimensionFail { dimension_name, violations: List<Diagnostic>, witnesses }` from `src/v3/std/dimensions.dag:51-61` as fold result (no parallel `LensReport` carrier; no rename of existing variants); meta-lens deferred post-R3; cross-domain composition explicit only (`Lens<C> × Lens<D> = Lens<(C,D)>`); T-LensAPI rescope to lens-as-monoid in same wave; 3 worked instances (complexity / tenant-flow / IFC) sufficient for generality validation. **Why now**: anything pushed out compounds exponentially (per user direction 2026-04-28); R3 lanes (T-CostLens-Composition, T-Verification-L4-L7-Direct, L6 form-coverage fold) all consume `Lens<C>` — without it landing in R2, we get 3 ad-hoc fold shapes that need rework. **Spec authority**: [`docs/design-lens-framework.md`](design-lens-framework.md) (PROPOSAL skeleton authored 2026-04-28 with three worked instances + 6 locked design decisions + migration plan + up-front validation checklist; Director extends post-#1078-merge for full spec).
- **B4 Identity-Carrier Substrate Pass:** the 4 Phase 1 carriers (`DeclarationRef` consumer migration, fold-shape carrier, emit-helper carrier, extdeps-fixture-set carrier) and the 8 Phase 2 site dissolutions. Sub-briefs B4.1 through B4.12.
- **Watch condition (split trigger):** Substrate Manager's combined T-Substrate + B4 scope is intentionally heavy (8+ parallel slots). If Substrate becomes the new bottleneck the way Director was — measured as workers idle waiting for Substrate-authored briefs >7 days — split B4 into a dedicated standing **B4 Identity-Carrier Manager**. R2 Release Manager surfaces this signal via the velocity-tripwire reporting if it fires.
- **Cross-program producer:** all four T-Substrate sub-lanes produce carriers consumed by Modeling Manager + Grounding Manager.
- **Authority:** authors all T-Substrate + B4 sub-briefs autonomously; dispatches workers; signals readiness to consuming managers via cross-manager queue; **signals sub-lane / Phase close to R2 Release Manager** (for closure ledger); escalates blockers and scope changes to Director.

### 3. Modeling Manager

Owns **T-Modeling** lane (Goal 2 — modeling-faithfulness dissolution) and consumer-side migrations of T-Substrate carriers.

- **Items:**
  - Surface int-literal magnitude at concept layer (consumes T-Substrate cardinality subset)
  - `Secret<T>` nominal-opaque graduation (consumes T-Substrate nominal-opaque subset)
  - `Dimension<Carrier>` typed value wrapper with phantom-parameter unit-mismatch enforcement (consumes T-Substrate parametric-algebra subset)
  - Tokenizer charclass phase-2 (consumes T-Substrate ValueBody-list/sum subset)
- **Cross-program consumer:** waits on Substrate Manager's per-sub-lane readiness signals; each item dispatches as its substrate dependency lands.
- **Authority:** authors all T-Modeling sub-briefs autonomously; dispatches workers; **signals item-close to R2 Release Manager** (for closure ledger + demo coordination); escalates blockers and scope changes to Director.

### 4. Impossible-Bugs Manager

Owns **T-ImpossibleBugs** lane (Goal 4 — remaining R2+ impossible-bug classes per `THESIS.md §"Enumerable impossible-bug classes"`).

- **Classes:**
  - Nested-optional flatten (gated on cardinality refinement; coordinate with Substrate Manager)
  - Unhandled diagnostic paths (Tier 2 substrate; coordinate with Substrate Manager if substrate work surfaces)
  - Unenumerated effects (post-effects-design-doc per #808; closed-system effects model is the canonical reference)
- **Cross-program coordination:** classes that surface substrate gaps escalate to Substrate Manager rather than expanding T-ImpossibleBugs scope.
- **Authority:** authors all T-ImpossibleBugs sub-briefs autonomously; dispatches workers; **signals class-close to R2 Release Manager** (for closure ledger); escalates blockers and scope changes to Director.

### 5. Pure Bootstrap Manager

Owns the **post-R1 work** of the Pure Bootstrap to Zero program (per `docs/design-pure-bootstrap-zero.md` LIVE 2026-04-25). Replaces the prior idle Zero-Floor Manager with a manager that has owned deliverables and pull-not-push intake.

- **R1 vs R2 boundary — defers to ROADMAP gate authority.** R1's PB gates (`pb_hand_rust_at_shim_floor`, `pb_compiler_std_ratchet_zero`, `pb_rust_tests_outside_residual_zero`, `lens_producer_files_remaining`) are owned by R1 per `ROADMAP.md §"Lane acceptance — .dag gates"` and the T-PB-A / T-PB-B lane rows. Their target per the cascade promotion is **0** (non-test hand-Rust + Rust-authored tests both → 0 via SG-0 census). r2-structure.md does not reinterpret those gate semantics — ROADMAP is single authority on gate close, and R1 closure criterion (all-gates-green) governs when the census-driven dissolution work is "done" for R1's purposes.
- **R2 PB Manager scope = work that survives R1 close**, not a duplicate of R1 PB lanes. Specifically:
  - **Mirror dissolutions** (Tier 3 #10 + #12 from #810): termination / computation / induction / effect-carrier Rust mirrors of std `.dag` carriers. These dissolve as v3 lowers + evaluates `.dag` runtime values; not a hand-Rust-census concern, so not gated by R1's PB gates.
  - **Tier 2 carry-from-#810:** `patch_lower_helpers_generated_type_alias_refinement` retirement (PB-Tier1 priority hint per #810 §5) is **closed by PR #1014**. The known-fragile bridge no longer survives R1/R2; generated `lower_helpers` emits the refinement field natively.
  - **Post-R1 PB program emergence:** any new dissolution work surfaced post-R1 (e.g., new mirror dissolutions discovered during R2, new Rust scaffolds inadvertently introduced) — owned here so the PB program has a standing home rather than going through Director ad-hoc dispatch.
  - **kernel_algebra_profile mirror dissolution** — `ValueBody::Map` carrier **landed** via #1017 (per PB Manager review 2026-04-28); remaining substrate gaps are map read-path/API + `std.computation` arrow-body evaluation (Evaluator-gated). PB Manager consumes the carrier; the read-path/API decision is a Substrate Manager sub-task.
- **What R2 PB Manager does NOT own:** the R1 PB census-reduction work itself. That's R1 lane work driven by R1 dispatchers under R1's all-gates-green close criterion (per ROADMAP single authority).
- **Cross-program coordination:** B4's §0.7 file-preference rank carrier touches PB territory; coordinate with Substrate Manager on shared substrate dependencies.
- **Authority:** authors all post-R1 PB sub-briefs autonomously; dispatches workers; signals lane-close to R2 Release Manager (for closure ledger); escalates blockers and scope changes to Director.

### 6. R2 Release Manager

Owns release coordination + the smallest cross-cutting deliverables (Goal 5, Goal 6, the #810 discipline framework, B-wave Tier 0 dispatch coordination, R2 demo).

- **Owned deliverables:**
  - **Goal 5:** §6a per-method-metadata — **pick closed** (Option 3 + receipt per `docs/design-substrate-carrier-port-program.md` §6a); **follow-through** is bulk migration + dissolution tracking per `docs/briefs/r2-release-6a-follow-through-worker.md`.
  - **Goal 6:** R2 closure demo coordination — surface "it runs" artifacts at each lane close per Demo discipline section.
  - **B-wave Tier 0 coordination:** B1 (#820) / B2 (#817) / B3 (#821) implementation through-merge, including any audit-narrative iteration. Dispatch B5 (Loop construction-closure audit) + B6 (file-preference rank checklist fix) + B7 (priority-hint relay to Pure Bootstrap Manager).
  - **Discipline framework enforcement:** owns the **central reporting** layer of P5 (per INVARIANTS.md#p5-progress-is-dissolution "Dispatch-Discipline Mechanisms" — paired-dispatch + per-PR gate + velocity tripwire). Tracks velocity-tripwire ratio per integration-reflection cadence and surfaces ≥3:1 readings to Director. **Per-brief paired-dispatch and per-PR gate enforcement happens at each manager's authoring point** (see "P5 dispatch-discipline applies to all manager-authored briefs" in Manager structure above) — Release Manager is not the choke point for those, but is responsible for surfacing systemic violations through the closure ledger when patterns emerge across managers.
  - **Thesis-claim coverage mapping** (Open call 1 below): authors the table on R1 closure → R2 promotion transition.
  - **R2 closure ledger:** tracks lane-close green status; surfaces unblocked work to idle workers; coordinates v2-retirement post-R3 (per the R3 structured-program reframe; v2 was previously "post-R2" but moved to post-R3 when R3 became a structured Thesis Closure program).
- **Authority:** authors briefs for owned deliverables autonomously; dispatches workers; coordinates demo cadence with all other managers.

### 7. Evaluator Manager (added 2026-04-28 amendment)

Owns **T-Evaluator** lane (Goal 7 — the runtime that unblocks the R3 consequence layer).

- **Items:**
  - **Runtime value model** — typed runtime values for the 6 type connectives (Atom / Conj / Disj / Arrow / Cardinality / Instantiation) + 5 L1 behaviors (Value / Transform / Branch / Loop / Bind). Closed-over environments + binding scopes for `Loop` / `Bind`. Lazy/eager strategy decision (see Open call 3).
  - **Body evaluator** — execute `.dag` function bodies structurally. Bounded forward execution per P4. Termination by descent evidence (already in substrate per `dsl/std/termination.dag`).
  - **Lens application** — extend `reflect_program_dag_nodes_in_file` from "shallow/lossy" to complete reflection per [`docs/design-reflection-completeness.md`](design-reflection-completeness.md) (LOCKED 2026-04-29). Lens application = fold over reflected program DAG. No substrate-carrier change; existing `Behavior` / `LoopBound` / `BranchPath` / `Witness<Carrier>` shapes already total.
  - **Witness construction** — runtime materialization of proof artifacts (Witness::Inhabits / Witness::Violates per `src/v3/std/dimensions.dag`); algebraic-law witnesses (associativity, commutativity, identity).
  - **Cross-target equivalence harness primitives** — for L5 verification in R3 (algebraic equivalence over a curated corpus, not byte-equal).
- **Cross-program producer:** Evaluator produces the runtime consumed by the R3 consequence-cycle lanes. Per `docs/r3-structure.md` §"R3 worker dispatch precondition" (post-12-lane structure), **9 of 12 R3 lanes are Evaluator-gated** (T-Tier3-Dissolution, T-LensProducer-Retirement, T-Verification-L4-L7-Direct, T-Verification-L5-Corpus, T-FixedPoint, T-Omni-Shape-B, T-CostLens-Composition, T-V2-Retirement, T-Free-Consequences-Demonstration). **The 3 non-Evaluator-gated R3 lanes** (T-Numeric-Construction [reframed 2026-05-01 from T-Int128], T-Anthropic-Wire, T-Bridge-Retirement) consume R2 substrate carriers but not the Evaluator itself, so they may dispatch in parallel with R2-Evaluator work per scheduling preference (see [`docs/r3-structure.md`](r3-structure.md) §"R3 worker dispatch precondition" for the precise carve-out + T-Numeric-Construction's internal cascade gate on T-V2-Retirement).
- **Cross-program consumer:** waits on Substrate Manager for any additional carriers needed by runtime values (e.g., closed-over environment representation) — design pass at lane spin-up identifies the dependency.
- **Authority:** authors all T-Evaluator sub-briefs autonomously; dispatches workers; signals Evaluator-readiness to Director (which gates R3 spin-up); escalates blockers and scope changes to Director.

### Director (cross-program coordinator)

- **Cross-program conflict resolution.** When Substrate Manager's carrier shape conflicts with Modeling Manager's consumer needs (or any analogous cross-program collision), Director arbitrates.
- **Scope-change escalation.** If a manager discovers their program needs to expand (e.g., a class-of-pattern dissolution surfacing under what was a single-item brief), Director adjudicates whether to expand the program or split a new lane. **Decision artifact format:** Director's adjudication lands as either (a) an amendment PR to the brief that surfaced the question, with explicit justification in the PR description; or (b) a sibling brief if the decision creates a new program scope. Both reference the originating discovery PR. The decision artifact is what closes the escalation cycle — the originating brief stays open until the artifact lands.
- **R1 residual closure surveillance** (none expected per all-R1-gates-green closure criterion).
- **Weekly dependency health check:** which lanes are within 1 step of unblocking? Which managers are blocked on cross-program signals? Surface to user when a program goes >7 days without lane-close.
- **What Director no longer does:** authoring sub-briefs for individual lane work. That returns to the responsible manager. Director's bandwidth is conserved for cross-program coordination.

## Lane structure

| Lane | Size | Manager | Covers |
|---|---|---|---|
| T-Ground | XL | **Grounding Manager** | Full T-Ground-* sub-program (Goal 1). Per [`docs/design-emission-model.md`](design-emission-model.md) engine-reframe (2026-04-28; further corrected 2026-04-28 to drop annotations + add lifetime analyzer): Pilot ✓ + Rust XL + Python L + Go L + **Coercion-Fold S** + **LanguageSpec M** + **Lifetime-Analyzer M** (replaces the retracted "Annotation" lane — derives ownership/lifetime structurally from program use, no annotation surface) + **Diagnostic S** + **CrossTarget-Meta S** + Tests S + Dissolve S. Replaces the prior 7-lane "Pilot/Rust/Engine/Tests/Dissolve" framing with 11-lane structure that surfaces the modeling problems the Engine framing was hiding |
| T-Substrate | XL | **Substrate Manager** | Four T-Substrate scoped-subset sub-lanes (Goal 3) **plus the B4 Identity-Carrier Substrate Pass program** (4 Phase 1 carriers + 8 Phase 2 site dissolutions, sub-briefs B4.1–B4.12, per `docs/briefs/b4-identity-carrier-substrate-pass.md`). Largest single program in R2; produces carriers consumed by Modeling Manager (3 sub-lanes) + Grounding Manager (engine-reframe lanes). Note: `kernel_algebra_profile` mirror dissolution — `ValueBody::Map` carrier **landed** via #1017; remaining substrate gaps are map read-path/API + `std.computation` arrow-body evaluation (Evaluator-gated). |
| T-Modeling | M | **Modeling Manager** | int-lit magnitude / `Secret<T>` graduation / `Dimension<Carrier>` (Goal 2) **plus tokenizer charclass phase-2** (consumer of T-Substrate ValueBody-list/sum sub-lane). Each item dispatches as its T-Substrate dependency lands. |
| T-ImpossibleBugs | S | **Impossible-Bugs Manager** | nested-optional flatten / unhandled-diagnostic-paths / unenumerated-effects (Goal 4). Substrate-gap discoveries escalate to Substrate Manager. |
| T-PB | M | **Pure Bootstrap Manager** | **Post-R1 PB program work** that survives R1 close per ROADMAP gate authority. Covers Tier 2 `patch_lower_helpers_*` retirement (if it survives R1) + termination/computation/induction/effect-carrier mirror dissolutions (Tier 3 #10 + #12 from #810; for `kernel_algebra_profile`, `ValueBody::Map` carrier landed post-#1017 — remaining substrate gap is map read-path/API + `std.computation` arrow-body evaluation, not the carrier) + post-R1 emergent dissolutions. **Does NOT duplicate R1 T-PB-A / T-PB-B census-reduction work** — that's R1 lane work per ROADMAP single authority on gate semantics. |
| T-Release | M | **R2 Release Manager** | §6a follow-through after closed pick (Goal 5) + R2 demo coordination (Goal 6) + B-wave Tier 0/2 dispatch (B1/B2/B3 through-merge, B5/B6/B7 authoring) + #810 discipline framework enforcement (velocity tripwire reporting) + thesis-claim coverage mapping (Open call 1) + R2 closure ledger + v2 retirement coordination. |
| **T-Evaluator** | **XL** | **Evaluator Manager** (added 2026-04-28) | Runtime value model + body evaluator + lens application + witness construction + cross-target equivalence harness primitives (Goal 7). **Largest single new program in R2; gates R3 spin-up.** Co-XL with T-Substrate. |

**Goal 6 (R2 closure demo) is not a lane.** It is a cross-lane closure discipline (see "Demo discipline" below): each lane's closure PR ships its own simple "it runs" artifact; **R2 Release Manager coordinates surfacing** (single authority per the 2026-04-26 rework). No separate T-Demo lane owner, no separate demo-authoring critical path.

**Structural-acceptance-per-lane-close discipline (Director-locked 2026-04-28):** the demo IS the structural acceptance gate. Each lane's closure PR proves the lane closed correctly via a *structural acceptance gate* (e.g., `omni_layers_share_one_node_tree` for omni-emission; `bound_declaration_carries_three_variants_structurally` for substrate work; `lens_complexity_n_log_n_fold_correct` for lens framework). The demo is the gate's evidence — not a separate ad-hoc artifact. R2 Release Manager (and R3 Release Manager) coordinate visibility — surfacing structural-acceptance gates as they fire — but the *authoring* of each gate is owned by the lane's manager. This avoids "process gate" framing where demos exist as separate authoring work disconnected from lane closure (per Pattern C corroboration earlier in the loop).

**Net manager-count justification (Director-locked 2026-04-28; per R1 Closure Manager review):** R2 carries 7 standing managers — the original 6-manager structure (Substrate / Modeling / Grounding / Impossible-Bugs / Pure-Bootstrap / Release; locked 2026-04-26) plus Evaluator added 2026-04-28 as Goal 7. Original structure preserved + 1 added, not restructured. Manager count rises to 7 to reflect Evaluator's distinct program scope (largest single new lane; co-XL with T-Substrate); does NOT split or fold any existing manager.

**Lanes deliberately absent (R1 gates, closed by R1 lane acceptance):**
- T-LensMigration / `lens_producer_files_remaining` — R1 T-PB-A gate per PR #752. **Cascade-promotion update 2026-04-25:** Pure Bootstrap to Zero program (LIVE per `docs/design-pure-bootstrap-zero.md`) target is 0; R1 closes the gate per ROADMAP authority (single authority on gate semantics). Lens-producer file-by-file migration work runs as R1 T-PB-A lane work. Not in R2.
- ~~T-ShimFloor / `pb_hand_rust_at_shim_floor` / `pb_compiler_std_ratchet_zero` / `pb_rust_tests_outside_residual_zero` — R1 T-PB-A + T-PB-B gates.~~ **Cascade-promotion update 2026-04-25:** Pure Bootstrap to Zero program (LIVE) owns all shim-floor work; the program target is 0 per `docs/design-pure-bootstrap-zero.md`. R1's PB gates close per ROADMAP authority. Not in R2 — R1 owns the census-reduction work via T-PB-A / T-PB-B lanes per ROADMAP single authority on gate semantics. **R2 Pure Bootstrap Manager exists for post-R1 PB program work that survives R1 close** (mirror dissolutions, Tier 2 patch retirement, post-R1 emergent dissolutions); see Pure Bootstrap Manager section above.
- T-EFamilyClose — R1 T-LaneE's critical-path carrier work (E-T, E-C, E-I, E-P, E-M sub-lanes), enabling the R1 `complexity_merge_sort_is_nlogn` + `complexity_merge_sort_v3_matches_v2_oracle` + `lane_e_bundled_witness_host_emit_parity` gates. All E-family carrier-port work closes in R1; the §6a **carrier pick** is closed at HEAD (Option 3), and R2 inherits **§6a follow-through** only (Goal 5 — migration + dissolution tracking per `docs/briefs/r2-release-6a-follow-through-worker.md`).
- T-TestGen-tail (`testgen_mock_backed_integration_safe` / `MockBackedInvariant` wiring) — R1 T-TestGen gate per `ROADMAP.md §"Lane acceptance — .dag gates"`. Closes in R1.

R2 does not re-own R1 gate close authority; under all-R1-gates-green criterion, those gates ARE the close conditions per ROADMAP single authority. R2 inherits two named exceptions:

1. **Goal 5's §6a per-method-metadata** — was not an R1 gate; the **pick** is closed in-tree (Option 3 + receipt). **Follow-through** (bulk lens migration + dissolution tracking) inherits to R2 T-Release per `docs/briefs/r2-release-6a-follow-through-worker.md`, not as a reopened design call.
2. **`sub_charclass_in_std_unicode` phase-2** — was an R1 T-Sub gate, but reclassified to R2 substrate-capability per ROADMAP amendment (2026-04-24) following Surface Manager's handoff that the remaining work is Class 5 Gap 3 substrate-capability scope, not T-Sub-only surface fix. Now a 4th sub-lane under R2 T-Substrate (Goal 3); see lane row above.

Plus **post-R1 PB program work that survives R1 close** owned by R2 Pure Bootstrap Manager (per Pure Bootstrap Manager section above): mirror dissolutions, Tier 2 patch retirement, post-R1 emergent PB work. Not a duplicate of R1's PB census-reduction work — that's R1 lane scope per ROADMAP single authority on gate semantics.

## Dependency DAG

```
Grounding Manager (T-Ground):
    Pilot → Rust → Engine → Tests → Dissolve     (critical path)
    Python, Go run parallel after Pilot          (fill queue)
    Engine sharpened-(b) ← Substrate Manager: ValueBody-list/sum carrier

Substrate Manager (T-Substrate + B4):
    T-Substrate sub-lanes (independent of each other; all dispatchable in parallel):
        cardinality-for-int-lit                  → unblocks Modeling Manager: int-lit
        nominal-opaque-for-Secret                → unblocks Modeling Manager: Secret<T>
        parametric-algebra-for-Dimensions        → unblocks Modeling Manager: Dimensions
        ValueBody-list/sum + std.unicode bootstrap → unblocks Modeling Manager: tokenizer charclass
                                                  → unblocks Grounding Manager: Engine sharpened-(b)
    B4 Identity-Carrier Substrate Pass program (sub-briefs B4.1–B4.12):
        Phase 1 carriers (parallel after audits):
            B4.1 DeclarationRef consumer migration  (existing carrier; no landing; consumer migration only)
            B4.2 fold-shape carrier
            B4.3 emit-helper carrier
            B4.4 extdeps-fixture-set carrier
        Phase 2 site dissolutions (mechanical; dispatched as Phase 1 carriers land):
            B4.5–B4.12 (8 sites; consumer migration of new substrate)

Modeling Manager (T-Modeling):
    int-lit       ← Substrate Manager: cardinality-for-int-lit
    Secret<T>     ← Substrate Manager: nominal-opaque-for-Secret
    Dimensions    ← Substrate Manager: parametric-algebra-for-Dimensions
    Charclass-2   ← Substrate Manager: ValueBody-list/sum

Impossible-Bugs Manager (T-ImpossibleBugs):
    3 independent classes (parallel-dispatchable):
        Nested-optional flatten      (may surface cardinality substrate work — escalate to Substrate Manager)
        Unhandled diagnostic paths   (Tier 2 substrate; may escalate)
        Unenumerated effects         (closed-system effects model per #808)

Pure Bootstrap Manager (T-PB) — POST-R1 only (R1 owns census-reduction lanes per ROADMAP):
    Tier 3 mirror dissolutions (parallel-dispatchable):
        termination / computation / induction / effect-carrier Rust mirrors
        kernel_algebra_profile (ValueBody::Map carrier landed post-#1017; remaining gates are map read-path/API + std.computation arrow-body evaluation; Evaluator-gated)
    Tier 2 patch_lower_helpers_* retirement (closed by PR #1014)
    Post-R1 emergent dissolutions
    Cross-program: B4's §0.7 file-preference rank carrier touches PB territory; coordinate with Substrate Manager.

R2 Release Manager (T-Release):
    Cross-cutting (parallel-dispatchable):
        §6a follow-through              (Goal 5; pick closed — migration + dissolution tracking)
        B-wave Tier 0 through-merge     (B1/B2/B3 implementation iteration)
        B-wave Tier 2 brief authoring   (B5 Loop construction-closure audit; B6 checklist fix; B7 priority hint)
        Discipline-framework enforcement (velocity tripwire reporting per cadence)
        Thesis-claim coverage mapping   (Open call 1; closed via thesis/r2-r3-thesis-mapping.md)
        R2 demo coordination            (Goal 6; surface artifacts at each lane close)
        v2-retirement coordination      (post-R3; tracked but not gated)

Evaluator Manager (T-Evaluator) — added 2026-04-28:
    Sub-lanes (sequential; design pass first, then implementation):
        Runtime value model design  → consumed by all later sub-lanes
        Body evaluator              ← runtime value model
        Lens application            ← runtime value model (extends reflect_program_dag_nodes_in_file)
        Witness construction        ← runtime value model
        Cross-target equivalence harness primitives ← body evaluator + lens application

    Cross-program producer:
        Evaluator readiness         → unblocks R3 (7 of 10 R3 lanes gated on Evaluator)
    Cross-program consumer:
        Additional substrate carriers (if needed for runtime values)
                                    ← Substrate Manager (design pass identifies dependency)

Director (cross-program coordinator):
    Conflict resolution + scope-change escalation only — no brief authoring.
    R3 spin-up gate: Evaluator-readiness signal from Evaluator Manager triggers
    R3 brief-authoring window per docs/r3-structure.md transition mechanics.
```

**Parallel-capable work at steady state:** Grounding (1 critical-path slot + 2 fill) + Substrate (4 T-Substrate sub-lanes + 4 Phase 1 B4 carriers; up to 8 parallel) + Modeling (4 items, each pair-blocked on Substrate readiness) + ImpossibleBugs (3 independent classes) + PB (parallel per file/test) + Release (cross-cutting, 5+ parallel cross-cutting items) + **Evaluator (sequential design pass → 4 parallel implementation sub-lanes after design lands)**. **Aspirational dispatch ceiling: ~25+ concurrent worker slots across 7 programs** (capacity, not committed throughput — actual concurrency depends on idle-worker availability and cross-program unblock timing), vs. ~9–13 under the prior 1-manager structure where Director was the brief-authoring bottleneck.

## R1 closure criteria

**All R1 gates green per ROADMAP authority.** R1 closes when all 9 lane gates named in `ROADMAP.md §"Lane acceptance — .dag gates"` evaluate green per ROADMAP gate authority, including omni-emit (`emit_omni_demo_fixtures_green`), the T-PB-A self-hosting gates (including `lens_producer_files_remaining` added via PR #752), the T-PB-B tests-as-data gates, and the T-LaneE complexity-lens gates (which ride on the E-family carrier-port chain). No director-defined subset-close.

**ROADMAP is single authority on PB gate semantics.** Per `ROADMAP.md` T-PB-A / T-PB-B lane rows, those gates target **0** (non-test hand-Rust + Rust-authored tests via SG-0 census, per `docs/design-pure-bootstrap-zero.md` LIVE). r2-structure.md does not reinterpret those gate semantics. R2 Pure Bootstrap Manager's scope (above) is the post-R1 PB program work that survives R1 close — mirror dissolutions, Tier 2 patch retirement, post-R1 emergent dissolutions — **not** a duplicate of R1's census-reduction work.

**PB census Director-concession clause** (per R1 Closure Manager review 2026-04-28 + `docs/briefs/r1-closure-manager.md` cross-manager table): if PB census gates are RED at R1's all-green window, the team docs route via Pure Bootstrap pacing + possible Director concession. R3 additions in this doc do NOT soften R1's PB semantics — R1 closure under PB-census-concession is a separate Director call that lives in `r1-closure-manager.md`, not in r2-structure.md or r3-structure.md.

**`pb_self_compile_fixed_point` two-horizon semantics:** the same predicate name maps to two horizons — (i) R1 lane gate (Pass = current `verification.dag` + `test_runner` evaluation; this is R1's acceptance under the looser "compiler can compile itself" reading), (ii) R3 thesis facet 2 (closes under stronger interpretation: bit-identical fixed-point + SG-0 choreography per Director decisions). **R1 close does NOT wait on R3.** R1's gate Pass is whatever the current predicate evaluates to; R3's `T-FixedPoint` lane elevates the *release/thesis* bar without changing R1's acceptance criterion. See `r3-structure.md` §"T-FixedPoint" for the elevated-bar specifics.

Rationale: consistent with anti-deferral stance — tail-shaped work closes before R1 declares done; R2 doesn't inherit R1 residuals. Consequence: R2 scope does NOT include Lens Purity by Construction, self-hosting shim-floor close, or E-family carrier closure — those are all R1 gate concerns, closed by R1's own acceptance criterion. R2 is free to focus on the thesis claims R1's gates don't cover.

## Transition mechanics

1. **R1 close** — **R1 Closure Manager** declares R1 closed when **release acceptance** and **lane receipts** per `ROADMAP.md §"Release R1 Program"` are satisfied.
   Closure record: [`docs/briefs/r1-closure-manager.md`](briefs/r1-closure-manager.md) (completion section).
2. **R1 residual sweep** — every open R1 ledger row gets an R1-or-R2 assignment. No orphaning. Done in the R1 closure PR. Expected to be short under all-gates-green criterion.
3. **R1 manager dissolution** — all R1 standing managers (Surface, Substrate, Testgen, Self-hosting, Release) archive with closure banners. Their scopes are fully absorbed by R1's gate acceptance; no inheritance into R2 managers.
4. **R2 manager spin-up** — **seven standing managers** initialized per the revised manager structure above (count updated 2026-04-28 per R1 Closure Manager review — Evaluator Manager added as Goal 7). Each manager gets a dedicated brief (`docs/briefs/r2-grounding-manager.md`, `docs/briefs/r2-substrate-manager.md`, `docs/briefs/r2-modeling-manager.md`, `docs/briefs/r2-impossible-bugs-manager.md`, `docs/briefs/r2-pure-bootstrap-manager.md`, `docs/briefs/r2-release-manager.md`, `docs/briefs/r2-evaluator-manager.md`) naming program scope + owned deliverables + cross-program dependencies + autonomous dispatch authority + reporting cadence. **Pre-stage skeleton briefs before R1 closes** (Director authors skeletons during R1 final week; promotion PR fills in scope-final details) so the R1→R2 transition is not gated on seven fresh authoring cycles. Existing `docs/briefs/grounding-manager.md` migrates content into `r2-grounding-manager.md` and archives. Existing `docs/briefs/pure-bootstrap-zero-manager.md` migrates content into `r2-pure-bootstrap-manager.md` and archives.

   **🔄 REVISED 2026-04-26 (further updated 2026-04-28).** Of the seven R2 managers, only **Pure Bootstrap is definitionally R1-close-gated** — its scope is "what survives R1 close" (mirror dissolutions, Tier 2 patch retirement, post-R1 emergent dissolutions); R1's PB gates (T-PB-A + T-PB-B census reduction) own the R1-side work, and PB Manager picks up the residual. The other six managers (Grounding, Substrate, Modeling, Impossible-Bugs, Release, Evaluator) **may spawn pre-R1-close** when (a) their brief queue is authored, (b) at least one owned deliverable is dispatchable, and (c) spawning does not conflict with R1 closure work. In particular, **R2 Substrate Manager's ValueBody-list/sum sub-lane is a PREREQ for R1C-A Sub-deliverable A** (M1(2.8) list-body lowering depends on the substrate variant landing) — pre-R1-close Substrate spawn actively unblocks R1 closure rather than competing with it. **R2 Evaluator Manager's pre-dispatch design lock cadence (PR-A through PR-E + LanguageSpec parallel) may run during R1's final week** — design-only work, doesn't conflict with R1 closure. This refines the original "spawn at R1 close" framing of step 4 without superseding the dissolution discipline of step 3 (R1 manager dissolution still happens at R1 gates green).
5. **R2 open** — this doc promotes to `ROADMAP.md` as `## Release R2 Program` section. Promotion still gates on R1 close — pre-R1-close R2 manager activity is operating under PROPOSAL authority of this doc; the ROADMAP `## Release R2 Program` section lands at step 1 + step 5 sequencing.

**v2 retirement (explicit non-scoping note):** The v2 compiler at `src/v2/` persists as test oracle into R2 (and through R3) and is NOT on the R2 or R3 ledger. Its retirement is external **post-R3 operational cleanup** — no thesis claim depends on v2 being absent. Differential-test-oracle retirement is bounded by adoption/documentation concerns, not release-gate discipline. (Originally framed as "post-R2" before R3 became a structured Thesis Closure program; reframed to post-R3 in the 2026-04-28 R2/R3 expansion.)

## Demo discipline — visibility as structural requirement

Simple "look, it runs" or "before/after analysis" artifact ships with each lane closure PR. **R2 Release Manager coordinates surfacing to user** (single authority per the 2026-04-26 rework; was Director under the prior 1-manager structure — reassigned to keep release coordination concentrated in one role). No time-based cadence; the gate-close natural rhythm carries the visibility load directly — a demo lands whenever a lane closes, not on a schedule.

Forms that qualify:
- Running artifact + 1-paragraph "what this demonstrates"
- Before/after: "this program didn't compile; now it does"
- Census snapshot: "retired N hand-Rust files this milestone"
- Diagnostic demonstration: "here's a bad program, here's the error, here's the fix suggestion"

Purpose: proof-of-work visibility at director cadence. Without it, program slips invisibly over long horizons.

## Decisions locked

- **Modeling-faithfulness (Goal 2) in R2** (not R3+). Anti-deferral principle: dissolution directions are named and clear for all three items (int-lit via concept-layer magnitude decoupling; `Secret<T>` via nominal-opaque; Dimensions via phantom-parameter algebra attachment), so deferral would be scope-theater. Director's initial "defer to R3+" counter reviewed and conceded post-reframe.
- **R1 closure criteria = all-gates-green**. Same anti-deferral principle applied to omni-emit. **This decision is load-bearing for R2 scope**: by closing all R1 gates in R1 (including T-PB-A / T-PB-B self-hosting gates + T-LaneE complexity-lens gates), R2 is released from owning those concerns. R2 is what's left after R1's full close.
- **Gate-ownership single authority = ROADMAP `§"Lane acceptance — .dag gates"`**. R2 does not re-own gates assigned to R1 lanes. If a concern is R1-gated, it closes in R1. (Release-authority blocking review on sha `6fdd8341` / `24bc0027e` surfaced this; the rewrite landed in sha `0847b4796` removed goals that duplicated R1 gate ownership.)
- **Demo cadence = gate-close natural rhythm**. Simple artifact per close; no time-based schedule.
- ~~**Manager count = 1 + Director.** One standing manager (Grounding) matches R2's single critical path. T-Modeling / T-Substrate / T-ImpossibleBugs are parallel-capable with no critical-path coordination pressure; Director dispatches directly. Adjustable upward if parallel fill-queue depth becomes unmanageable in practice.~~ **🔄 RETRACTED 2026-04-26.** Empirically wrong: standing managers without owned deliverables sat idle while Director became the dispatch bottleneck for every other lane, starving 3 of 4 non-Grounding lanes. R1's program-manager pattern restored — see "Manager structure" above for the 7-manager structure (each owning a complete program with autonomous brief-authoring + dispatch authority through R2 close).
- **Manager count = 7 standing managers + Director coordinator** (locked 2026-04-26 at 6 managers; updated 2026-04-28 to 7 with Evaluator Manager added as Goal 7). Grounding / Substrate / Modeling / Impossible-Bugs / Pure Bootstrap / R2 Release / Evaluator. Each owns a mutually-exclusive program; cross-program dependencies handled via R1 `Cross-manager notifications queued` brief pattern. Director's role is cross-program conflict resolution + scope-change escalation, not brief authoring.
- **R2 includes substrate prereqs explicitly** per user's (i)-over-(ii) preference (honest scope over tight scope), with **scoped acceptance criteria** per Director refinement (each sub-lane closes on unblock of its paired Goal 2 item; full substrate-capability lanes are not R2-committed).
- **Anti-deferral principle is the frame, not velocity numbers.** Per Director observation: 16-hour R1 execution was a peak-day sample, not a baseline. The principle "if dissolution direction is clear and named, deferral is problem-finding not problem-solving" is what survives cadence shifts.
- **`sub_charclass_in_std_unicode` phase-2 reclassified R1 → R2** (2026-04-24, ROADMAP amendment paired with this revision). Surface Manager handoff (sub-child `quiet-gull-882` triage) confirmed the remaining work is Class 5 Gap 3 substrate-capability scope, not T-Sub-only surface fix; reclassifying lets R1 close on T-Sub Day-1 + DB-11 without waiting on substrate-capability work. Phase-2 lands in R2 as a 4th T-Substrate scoped sub-lane (consumer: tokenizer). Reclassification scope is bounded — full Class 5 Gap 3 substrate-capability close remains outside R2 unless additional R2 items demand it; only the tokenizer-charclass-unblock subset is R2-committed.
- ~~**Post-R2 stance = STRONG.** R2 is thesis close. Post-R2 work is external (adoption, documentation, community, ecosystem buildout). R3 is reserved as escape hatch only, not as a structural release program. User-locked 2026-04-24 after pressure-test framing: *"real programs are probably required to confirm the thesis is real, but we can keep R2 theoretical to stay fair — future work would pressure test the claims."* The R2 doc's "close-everything" claim is thus scoped to the **structural thesis** (what the compiler proves by construction); practical validation via modeling real programs (e.g., the user's `../ctrl/` follow-up) is a separate post-R2 stream that *tests whether the thesis holds in practice*, without itself being a thesis claim.~~ **🔄 SUPERSEDED 2026-04-28** by the next bullet (R3 reframed from escape-hatch to structured Thesis Closure program). The "post-R2 external-only" framing was retracted in the same direction-change. The "post-R3 external-only" stance is preserved (practical pressure-test on `../ctrl/` programs stays external; not a release-ledger item).
- **Pre-R1-close R2 manager spawn allowed for 6 of 7 managers** (locked 2026-04-26 per user direction; count updated 2026-04-28 with Evaluator Manager addition). Per-manager R1-gating audit found only Pure Bootstrap is definitionally R1-close-gated (scope = "what survives R1 close"); the other 6 (Grounding, Substrate, Modeling, Impossible-Bugs, Release, Evaluator) have no technical R1 dependency and may spawn pre-R1-close. R2 Substrate spawn actively *unblocks* R1 closure (ValueBody-list/sum is prereq for R1C-A Sub-deliverable A). R2 Evaluator's pre-dispatch design lock cadence (PR-A through PR-E + LanguageSpec parallel) is design-only and may run during R1's final week. This is the targeted exception to step 4's "spawn at R1 close" framing in Transition mechanics; refined inline at step 4. ROADMAP `## Release R2 Program` promotion (step 5) still gates on R1 close — pre-R1-close R2 manager activity operates under this doc's PROPOSAL authority, not under ROADMAP authority.
- **Evaluator (Goal 7) added to R2 (locked 2026-04-28 per user direction).** The runtime that executes `.dag` bodies, applies lenses, and constructs witnesses lands in R2 as the *capacity layer*. Justification: at gunbc multi-agent dispatch velocity, Evaluator is sized at ~2-3 weeks (vs. 2-4 months at solo-dev sizing). Folding it into R2 (rather than a separate post-R2 program) means R2 ships a complete capacity layer + R3 runs every consequence claim mechanically. The split is structural: R2 = enabling work that requires new design / substrate / runtime; R3 = mechanical follow-through that becomes obvious once R2 lands.
- **R3 reframed from escape-hatch to structured Thesis Closure / Consequence Cycle program (locked 2026-04-28 per user direction).** Supersedes the prior "R3 reserved as escape hatch only" decision. Per [`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md), ~7 thesis claims become mechanical post-R2-Evaluator (Tier 3 mirror dissolution × 4, lens-producer retirement, L4-L7 verification, fixed-point, Tier 2 Int128, Shape B demos × 2+, Anthropic typed wire). Treating those as an escape-hatch obscured the structural opportunity. R3 is now named, scoped, and dispatchable per [`docs/r3-structure.md`](r3-structure.md).
- **No separate coercion engine — engine framing replaced with 5 substrate-completion lanes (locked 2026-04-28 per user direction; further corrected later same day to drop annotations + add lifetime analyzer).** Per [`docs/design-emission-model.md`](design-emission-model.md), THESIS:171 is explicit: "Coercion = emission. No separate coercion engine." The `T-Ground-Engine` lane name implied an authority that "picks up slack" when structure under-determines — that violates fail-closed (P3) + parallel-authority discipline (P2). The reframe surfaces the real modeling problems the engine framing was hiding: refinement composition, structural axes that distinguish apparent multi-inhabitants (replaces "canonical choice"), program-intent derived structurally from program use (replaces "user annotation"), declared structural ordering for diagnostic enumeration, fail-closed diagnostic surface, language spec substrate, cross-target uniformity meta-spec, and post-R3 dogfooding. Each is a substrate-completion lane in its own right. Net effect on R2 scope: same or slightly larger total work, much higher *visibility* of work — the work was always there; the engine framing was hiding it. **Lane structure: T-Ground-Coercion-Fold S + T-Ground-LanguageSpec M + T-Ground-Lifetime-Analyzer M + T-Ground-Diagnostic S + T-Ground-CrossTarget-Meta S** (5 substrate-completion lanes; the prior "Annotation" lane was retracted in favor of structural derivation per Modeling problem 3 corrected). **Affects already-merged PR #989** (slice 1 of Phase 2 merged on main with engine framing); post-merge realignment queued per [`docs/design-emission-model.md`](design-emission-model.md) §"Affected lanes (post-merge realignment)" option (c) — hold further engine-framed slices + queue cleanup wave once LanguageSpec lands. **Director-coordinated**.

### R2-expansion items from Director review of #1078 (2026-04-28)

Five concrete findings from the gpt-5-5-pro exploratory analysis (cited by Director in #1078 review) are R2-expansion items — small, sharp, and substrate-completion-shaped. Each maps to a P-class invariant violation or duplicate authority that closes cleanly inside R2 rather than carrying over to R3:

- **N1: `analyze_symbolic_cost_dimension` fabricates `UnknownCost` on root miss** — `dimension.rs:67-79` produces `UnknownCost` when the root port has no symbolic cost. Per `algebra.dag:22-44`, `UnknownCost` is an algebra element, not a failure channel. P3 violation. **Fix:** consume `Lookup<SymbolicCost>` or partition `DimensionReport` into pass/fail variants. Owner: Cleanup Worker 1 #1003 (in flight) or post-merge follow-up. Lane home: T-Substrate residual / B-wave Tier 0.
- **N2: Operator missing-field fallback fabricates signatures** — `infer.rs:4195-4249`, `emit.rs:193-209`. Bool+Bool with `BooleanAlgebra` falls through to `(Bool,Bool)->Bool` even when no `add` field exists. Sharper than the existing ROADMAP "Operator ontology" row. **Fix:** distinguish `Resolved | NoAlgebraYet | AlgebraFoundButFieldMissing`; the third must fail-closed. Lane home: T-Substrate B-wave or T-Modeling.
- **N3: Shell `exit_success` / Boolean / typed-exit triple authority** across 6 extdeps files. `std/process.dag:39-41` already has `ProcessExit` carrier; the triple needs to consolidate. Closure-trigger-class consolidation similar to OpenAI/Anthropic wire work. Lane home: T-Substrate sub-lane.
- **N4: `Lookup<T>` algebra lifts hand-rolled 3x** in `cost.dag` (`combine_iterate` / `combine_max` / `combine_sequential`). Add generic `lookup_lift2` primitive in `std.lookup`. Small (1 primitive + 3 consumer rewrites). Lane home: T-Substrate or T-LaneE residual.
- **N5: `ExecuteCommandHostOutcome::Other(ClaimResult)` string authority** at `test_runner.rs:1091-1112` + substring assertions at `:3155-3161`. The #1063 helper landed the structural fix; this is the natural follow-up to expand `Other` into typed variants (`SpawnError` / `PolicyRejected` / `Timeout` / `SignalNoExitCode` / `ExitMismatch`). Sharper than ROADMAP "PB-Runtime / ExecuteCommand" row. Lane home: PB Manager or T-Release.

These 5 items are R2-close items, not R3 deferrals. Owners are existing managers; no new lane structure needed.

### R2 amendments to fold in (Director review of #1078 — 2026-04-28)

Three additional items the Director review identified as R2-expansion (not R3):

- **Diagnostic vocabulary CI sync** — every `DiagnosticKind` the test runner accepts must be declared in `src/v3/std/verification.dag` or marked internal-only. Currently `MagnitudeOutOfRange` / `MalformedIntegerRangeFact` may not be writable in `.dag` `TestClaim`, leaving Rust-side test authority leaking back. **Form:** could be a R2 `.dag` gate (`diagnostic_vocabulary_sync_zero_drift`), not just a CI script. Lane home: T-Release or T-Substrate.
- **Hand-rolled lattice data witnesses** (`DescentEvidence`, `Encoding`) — gated on aggregate value support, which is now available post-#1017 (ValueBody::Map). These should land in R2-Substrate as data witnesses (`data descent_evidence_lattice: BoundedLattice<DescentEvidence> = { meet, join, top, bottom }`), not deferred to R3-Tier3-Dissolution. Per Exploratory finding #5; existing scope was deferred to "post-aggregate-value-support" which has now landed.
- **Target primitive/range duplication retirement** (Reflective Pattern E) — Rust pilot constants + `.dag` aggregate rows + specialized range readers all orbit the same concept. Engine sharpened-(b) (PR #989) should consume `Dag::rust_pilot_primitives()` directly, retiring the `RUST_PILOT_PRIMITIVES` Rust mirror. Belongs in R2-Grounding as part of the engine-reframe work above (post-cascade). Note: this overlaps with `T-Ground-LanguageSpec` from the engine reframe — the LanguageSpec lane absorbs this as part of its scope.
- **Open call 1 (thesis-claim coverage mapping) closed via [`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md)** (2026-04-28). Per-claim disposition table covers every Tier-1 / Tier-2 / Tier-3 thesis claim with R1 / R2 / R3 / post-R3-external assignment + evidence pointer + status. Compromises (deferrals from R2 to R3 or post-R3) are documented inline. Design challenges to resolve before R3 dispatch are enumerated in [`docs/r3-structure.md`](r3-structure.md) §"Design challenges to resolve up-front."

## Open calls

### 1. ~~Pre-promotion thesis-claim coverage mapping (gate before ROADMAP promotion)~~ — **CLOSED 2026-04-28**

> **🔄 CLOSED** via [`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md). Per-claim disposition table covers every Tier-1 / Tier-2 / Tier-3 thesis claim with R1 / R2 / R3 / post-R3-external assignment + evidence pointer + status. Compromises (deferrals) documented inline. The doc closes this Open call and remains the live thesis-claim coverage authority for ongoing R2/R3 amendments.

~~Surfaced by codex API review on `6fdd8341`: the "close-everything/post-R2-external-only" framing requires an explicit mapping from THESIS tiers to concrete R1/R2/post-R2 disposition, so no thesis claim is implicitly-positioned. Otherwise "close-everything" is an assertion without audit.~~

~~THESIS authority (`THESIS.md:155-182`) lists:~~
- ~~**Tier 1 — Structural correctness** (type mismatches, CX termination, coercion = emission, ownership no-alias, **Grounding completeness**).~~
- ~~**Tier 2 — Runtime safety** (division-by-zero, integer overflow, out-of-bounds, force-unwrap, partial functions — proven safe or made total).~~
- ~~**Tier 3 — Verification from structure** (L4 emitted ≡ .dag, L5 cross-target parity, L6 structural-form coverage, L7 algebraic laws).~~

~~**Required before promotion:** a table mapping every Tier-1 / Tier-2 / Tier-3 claim to its R1-closed / R2-gated / post-R2-external disposition, with any gaps (claim named in THESIS but not mapped) flagged as pre-promotion blockers. Non-blocking for this PR; blocking for ROADMAP promotion.~~

~~**Audit format:** table with columns `Tier | Claim | Disposition (R1 / R2 / post-R2-external) | Gate or lane name | Evidence (PR# or gate name) | Status`. Gaps are rows with disposition column empty or claim not in THESIS list.~~

~~**Ownership:** Director authors as a sibling PR. PM reviews for completeness against `THESIS.md §"Thesis claims — complete list"`.~~

~~**Timeline:** lands as part of the R1 closure → R2 promotion transition (step 4 in Transition mechanics above). Not a separate program.~~

### 3. ~~Pre-dispatch Evaluator design questions~~ — **CLOSED 2026-04-28 per Director review of #1078**

> **🔄 CLOSED.** The 8 design challenges in [`docs/r3-structure.md`](r3-structure.md) §"Design challenges — direction ratified 2026-04-28; specific decisions split between DECIDED and SCHEDULED" were ratified in the Director review of #1078 at 2026-04-28T01:32:45Z, with **two distinct release states** (per gpt-5-5-pro meta-review feedback at 03:02Z): **DECIDED** for challenges #4-#8 (specific design final, no further design PR needed) and **DIRECTION RATIFIED, SPECIFIC DECISION SCHEDULED** for challenges #1-#3 (direction locked; specific design lands in named follow-up PRs PR-B/C/D before R2-Evaluator dispatch). The dispatch cadence (PR-A → PR-B → PR-C → PR-D → PR-E + LanguageSpec parallel) is named in [`docs/r3-structure.md`](r3-structure.md) §"Pre-R2-Evaluator design lock cadence."
>
> The original anchor reference (§"Design challenges to resolve up-front") no longer exists; the section was retitled to reflect the locked-decisions state.

### 4. Pre-T-Ground / pre-T-Substrate-Lens-Primitive design questions — **LOCKED 2026-04-28 (all 8 questions resolved via Director dialogue)**

**Status update 2026-04-28 (dialogue completed):** all 8 surfaced design questions resolved via Director dialogue. Per-question status:

| Q | Topic | Decision |
|---|---|---|
| Q1 | `BoundDeclaration` substrate type | `Interval<D>` shared parent (substrate consolidation prepended as PR-PreF); `BoundDeclaration = StaticBound(Interval<Int>) \| PlatformDependent`; asymmetric match rule (target's Unbounded universal-accepts) |
| Q2 | Per-target structural axes (Rust/Python/Go × primitive families) | (b3') Emission-biased non-violating minimal target modeling; `ReferenceModel<T>` shared parent; four-property verification framework (Faithful/Correct/Minimal/Performant) with Faithful + Performant as Lens<C> instances; Correct = L4 runtime check |
| Q3 | Per-primitive realization cost field shape | (c5) `RealizationCost` as record of `Cost<Unit>` coordinates; `Cost<Unit> = Dimension<Unit, SymbolicExpr>`; `Bits` and `CPUCycles` as substrate-declared primitives sibling to SI base units; sparse fail-closed access map |
| Q4 | L4 emit/eval-match acceptance corpus | (c)+(d) hybrid — generated cross-product + user-program corpus; universal four-property gate (Faithful + Correct + Minimal + Performant) per Q2 cascade; per-target compiler harness for non-violation gate |
| Q5 | L6 cardinality variant enumeration | (a) — cardinality is the connectives axis; collapses by construction with Q1's `Interval<Cardinal>` instances on each connective; PR-J = no-op |
| Q6 | `Witness<C>` generality for non-trivial monoids | (c)/(d) hybrid — Witness<C> stays as-is; rich structural validation failures encode into lens-local `Diagnostic.kind` declarations via lens-framework structural inhabitance, not into the closed compiler-core `CompilerDiagnosticKind` sum (uniform whether compiler-authored or user lens — compiler IS a user of its own system per `feedback_groundedness_gates_lenses`); arity-over-error-space pattern recognized observationally, no consolidation |
| Q7 | Error-recovery: report-all vs short-circuit | (d) — per-call validate yields one `OptionalDiagnostic`; fold accumulates `SomeDiagnostic.value` into `DimensionFail.violations: List<Diagnostic>`; no spec change. Arity-over-error-space makes report-all fall out by construction |
| Q8 | Cross-product validate with mixed presence | (a) — conjunctive; cross-product validate = `validate_C ∧ validate_D` with NoDiagnostic = TRUE; falls out of Q7's fold accumulation; no substrate change |

**Authority docs** for the dialogue (DECISION-locked paragraphs):
- Q1–Q5 (T-Ground / cross-target modeling): [`docs/design-emission-model.md`](design-emission-model.md) §"Design questions to surface and lock before dispatch"
- Q6–Q8 (lens-framework spec semantics): [`docs/design-lens-framework.md`](design-lens-framework.md) §"Design questions to lock before substrate dispatch"

**Meta-deliverable:** the modeling pattern that surfaced across Q1/Q2/Q3 (find shared parent → distinguish coproducts from coordinates → classify primitive vs lens-extensible) was promoted to [`INVARIANTS.md`](../INVARIANTS.md) §P1 as the "Substrate-fact introduction procedure" — a 3-step decision procedure workers can self-serve when introducing new substrate facts.

**Cadence PRs are ready to dispatch sequentially after #1078 merges.** Director signoff on this doc + the design docs is the green light.

**Surfaced:** the no-engine emission reframe + lens-framework introduction surfaced 8 modeling-hard design questions that need lock before per-target / per-lens dispatch. Director directive 2026-04-28: *"surface all design questions up front; modeling will be incredibly hard to get right per language; ensure verification (testing) discussion + discussion up front; otherwise leaving complex modeling problems open-ended will fail (per `feedback_holistic_over_patches.md`, `project_ownership_holistic.md` — alias/clone class)."*

**Authority docs** for the surfacing:
- Q1–Q5 (T-Ground / cross-target modeling): [`docs/design-emission-model.md`](design-emission-model.md) §"Design questions to surface and lock before dispatch"
- Q6–Q8 (lens-framework spec semantics): [`docs/design-lens-framework.md`](design-lens-framework.md) §"Design questions to lock before substrate dispatch"

**Required before dispatch:** Director sign-off on Q1–Q8 alternatives (accept recommendation, propose alternative, or surface sub-question). Each question includes: precise statement, alternatives enumerated, cascade implications listed, TestClaim shape proposed regardless of which alternative is chosen.

**Cadence PRs** (modeled on PR-A through PR-E that locked R2-Evaluator design):

| PR | Locks | Before dispatch of |
|---|---|---|
| **PR-PreF** *(prepended 2026-04-28 per Director directive)* | `Interval<D>` substrate consolidation — shared parent for CardinalityBound / SizeBound / LoopBound::Cardinality (additive retrofit; LoopBound::Descent and CostBound stay distinct). Sets up Q1's `BoundDeclaration = Interval<Int>` instance to consume a clean parent rather than introduce a fifth distinct bound type | All subsequent cadence PRs |
| **PR-F** | Q1 (BoundDeclaration = Interval<Int>; consumes PR-PreF parent) + Q2 Rust axes | T-Ground-Coercion-Fold + T-Ground-Rust |
| **PR-G** | Q2 Python axes | T-Ground-Python |
| **PR-H** | Q2 Go axes | T-Ground-Go |
| **PR-I** | Q3 (per-primitive realization cost) + Q4 (L4 corpus shape) | T-Ground-LanguageSpec + T-Verification-L4-L7-Direct |
| **PR-J** | Q5 (L6 cardinality enumeration — likely no-op given PR-PreF consolidation reinforces recommendation a) | T-Ground-CrossTarget-Meta |
| **PR-K** | Q6 + Q7 + Q8 (lens framework spec) | T-Substrate-Lens-Primitive |

**Ownership:** Director authors decisions inline in `design-emission-model.md` and `design-lens-framework.md` design-questions sections, OR as separate PRs PR-F through PR-K. Worker dispatch on the corresponding lane waits until the relevant cadence PR(s) merge.

**Non-blocking for #1078:** the surfacing itself is the deliverable; the locking is downstream.

~~Surfaced by 2026-04-28 R2 amendment folding Evaluator into R2 scope. The Evaluator is the largest single new lane; design questions affect downstream R3 consumers and should resolve before worker dispatch.~~

~~**Required before R2-Evaluator dispatch:** Director-authored decisions (or accept-as-recommended) on each of the 8 design challenges enumerated in `docs/r3-structure.md` §"Design challenges to resolve up-front". Specifically:~~

~~1. **Evaluator runtime-value representation** — closed-over environments, lazy/eager strategy, memoization, witness construction surface~~
~~2. **Lens reflection completeness scope** — what does "complete reflection" mean for `reflect_program_dag_nodes_in_file`?~~
~~3. **Cross-target equivalence semantics** — algebraic-equal vs byte-equal vs behavioral-equal under oracle?~~
~~4. **SG-0 zero requirement for fixed-point** — full hand-Rust retirement before fixed-point closes, or non-test subset only?~~
~~5. **T-Verification-L4L7 sequencing** — L4-L7 parallel or sequential within R3?~~
~~6. **Shape B target choice** — which 2 Shape B targets does R3 demo?~~
~~7. **Tier 3 mirror dissolution mechanics** — performance threshold for accepting evaluated paths over compiled-Rust mirrors?~~
~~8. **R3 Anthropic vs R2 OpenAI** — generalize provider pattern in R3, or replicate?~~

~~**Required:** Director reply on each (accept recommendation, propose alternative, or surface sub-question). Non-blocking for this PR; blocking for R2-Evaluator and R3 dispatch.~~

~~**Ownership:** Director authors as inline updates to `docs/r3-structure.md` §"Design challenges to resolve up-front" or as a sibling PR.~~

~~**Timeline:** R2-Evaluator design questions resolve before R2-Evaluator worker dispatch begins. R3-specific design questions (Shape B choice, L4-L7 sequencing, etc.) may resolve during R2's final week / R3 brief-authoring window.~~

### 4. Release-doc authority discipline (added 2026-04-28 per gpt-5-5-pro meta-review)

<!-- [retraction-context] retrospective discussion of superseded framings; not live release-control authority -->
The PR #1078 review loop surfaced a recurring pattern across **15+ review events / ~133 minutes / 7 codex passes / 1 cursor / 4 claude / 3 openai-pro** (as of the gpt-5-5-pro meta-review at 2026-04-28T03:47Z; loop continued past that point with additional iterations): reviewers kept finding the same P2 single-authority shape in new clothing (lane count drift, Evaluator gating count drift, T-Ground-Engine surviving the no-engine reframe, T-Ground-Annotation surviving the no-annotation reframe, "DECISIONS LOCKED" coexisting with "RECOMMENDATION", Shape B target locks not propagating to gates, escape-hatch language adjacent to structured-program language, ordering-as-emission contradicting diagnostic-only ordering, L6 living simultaneously in R2 and R3, consumer-not-wired-to-CI). [retraction-context] gpt-5-5-pro meta-reviewer at 2026-04-28T03:02:37Z verdict: PAUSE_AND_REGROUP — promote the pattern into a guardrail before iterating further.

**Release-doc authority discipline (specialization of P2 Boundary Discipline at the release-control surface):**

Every release-control fact lives in exactly one place with exactly one state. State drift across sibling docs (or within a single doc) is forbidden as live framing.

**Release-control facts** include (non-exhaustive): lane counts, Evaluator-gated counts, R2/R3 lane membership, target-pair locks (Shape B etc.), engine/annotation/canonical lane existence, design-call open/closed state, decision states (DECIDED vs DIRECTION-RATIFIED-PENDING-PR vs OPEN), gate names, gate predicates.

**State machine for each fact:**
- `OPEN` — under active design discussion
- `PROPOSED` — direction proposed, not yet ratified
- `DIRECTION-RATIFIED, SPECIFIC-DECISION-SCHEDULED` — direction locked; specific design lands in named follow-up PR
- `DECIDED` — specific design final, only implementation remains
- `CLOSED` — work complete or moved out of release scope
- `SUPERSEDED` — replaced by a named successor; original strikethrough'd inline
- `RETRACTED` — explicitly removed from release ledger; original strikethrough'd inline
- `DEFERRED` — moved to a future release with named target

**Discipline rules:**

1. **Single home.** Each release-control fact has one authoritative location named in this doc or [`docs/r3-structure.md`](r3-structure.md). Other docs project / cite, never re-author.
2. **Single state.** A fact may not simultaneously hold two states (e.g., `DECIDED` and `OPEN`). When superseding, the prior framing is strikethrough'd inline with a `🔄 SUPERSEDED <date>` marker pointing at the successor.
3. **Cascade discipline.** When a fact changes state, the change is propagated to every document that *projects* it in the same PR. Specifically the engine/annotation/lane-count/target-pair changes that recurred in the #1078 review loop must cascade across r2-structure.md, r3-structure.md, design-emission-model.md, and thesis/r2-r3-thesis-mapping.md before merge. **ROADMAP.md cascade is a *tracked follow-up bridge* (not before-merge)** because ROADMAP is the post-promotion authority surface; it amends *after* R2 promotes per [`docs/r2-structure.md` §"Transition mechanics" step 5](#transition-mechanics). The cascade is named in [`docs/design-emission-model.md`](design-emission-model.md) §"Cascade across upstream docs" (Open call 2) with explicit dissolution trigger: ROADMAP cascade lands as part of R2 promotion to ROADMAP `## Release R2 Program` section, owned by Director per pattern-A coordination scope.
4. **Forbidden-string consumer.** A doc-consistency consumer (script or CI gate) checks for forbidden stale lane/concept names appearing outside retraction-context markers. See §"Doc consistency check" below for the live consumer.
5. **State name correctness.** "DECISIONS LOCKED" cannot be used for items where the substantive decision is "scheduled in a follow-up PR." Such items are `DIRECTION-RATIFIED, SPECIFIC-DECISION-SCHEDULED`. [retraction-context: this rule references the retracted state name to define why it's retracted]

**Why this is a P2 specialization, not a separate principle.** P2 says every fact lives in exactly one authoritative place. The release-control surface adds: each fact also has exactly one state, and the state transitions are auditable. Both are single-authority discipline; the addition is the state-machine vocabulary at the release surface.

**Receipt:** an extended release-doc review loop is the empirical case study for this rule. Each round caught one stale string or one structured-state drift while another sibling table preserved a different one. Without this rule, the next release-doc PR will repeat the loop.

**Acknowledged gap (v2 guardrail follow-up)** [retraction-context: this paragraph documents the v1 consumer's input list; the listed forbidden strings appear as items being checked, not as live authority]: the v1 `scripts/check-release-doc-authority.sh` consumer catches forbidden-string drift (T-Ground-Engine, T-Ground-Annotation, canonical choice, @target, DECISIONS LOCKED, T-Verification-L4L7 in non-retraction context) but does NOT catch *structured-state drift* — lane counts, Evaluator-gated counts, dependency-graph membership, thesis-claim disposition, "implicit vs dedicated lane" state. The same review loop also identified a **retraction-pattern foot-gun**: v1's `[Rr]etracted` and "retracted" patterns are case-insensitive substring matches, so any prose containing the word "retracted" (anywhere on the line) exempts a forbidden string — a regression where stale framing slips in alongside an unrelated retraction discussion would not be caught.

**v2 guardrail requirements** (named explicitly to prevent drift):
1. **Structured release-state authority artifact** (`docs/release-state.yaml` or `docs/release-state.dag`) declaring lane → manager / disposition / dependencies / thesis-claim coverage in one machine-readable place
2. **Mechanical projection-check script** that parses the artifact and verifies r2-structure.md / r3-structure.md / thesis-mapping.md / design-emission-model.md projections match the single source
3. **Explicit-marker-only retraction-context detection** — replace v1's text-pattern matching (`[Rr]etracted`, "the retracted", etc.) with structured `[retraction-context]` marker-only detection, eliminating the substring foot-gun

Tracked as **follow-up post-merge of #1078** (Director-coordinated; bundles cleanly with the Director-authored `docs/design-lens-framework.md` work).

**Doc consistency check** (small consumer at `scripts/check-release-doc-authority.sh`; enforced by both CI and Makefile):

```bash
# Forbidden-string consumer — fails if any stale concept name appears in
# live (non-retraction-context) sections of release-control docs.
# Lives at scripts/check-release-doc-authority.sh.
#
# Enforcement paths (both wired):
#   - CI: .github/workflows/ci.yml step "Release-doc authority check
#     (P2 single-authority discipline)" — runs on every push/PR
#   - Makefile: `make verify` (alongside bootstrap-check + testgen-check)
#     and direct target `make release-doc-authority-check`
```

The script consumes a forbidden-string list (T-Ground-Engine outside retraction; T-Ground-Annotation outside retraction; "canonical choice" as live carrier; @target annotation outside retraction; "DECISIONS LOCKED" applied to DIRECTION-RATIFIED-SCHEDULED items, etc.) [retraction-context: documenting the consumer's input list] and reports violations. It's a small, mechanical consumer — not a full state-machine validator — but it catches the recurring pattern from the #1078 review loop with one bash invocation. **Enforcement path:** the CI workflow at `.github/workflows/ci.yml` invokes `bash scripts/check-release-doc-authority.sh` as a named step (alongside the existing fabrication-sentinel ratchet); failures surface as build errors, not advisory warnings. Local dev: `make verify` (or `make release-doc-authority-check` for the standalone check).

### 2. ~~Pre-promotion `≤5 irreducible-shim` gate-name review~~ — **RETRACTED (2026-04-25 cascade promotion)**

> **🔄 RETRACTED** — both Option A (sharpen the ≤5/3/2 trajectory) and Option B (rename gate to `pb_hand_rust_at_boundary_floor`) framed gate-name choices around fractional-shim numbers. Under the cascade-promoted 0-floor target per [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md) (LIVE 2026-04-25; supersedes the prior ≤5-floor framing in `docs/design-pure-bootstrap.md`), the gate threshold is **0**, not "boundary-determined 2-3". Both options below are moot. The predicate name `pb_hand_rust_at_shim_floor` is retained as housekeeping (semantic shifted: "shim floor" now reads as 0 under cascade promotion); a future predicate rename to `pb_hand_rust_zero` is post-cascade housekeeping, not a pre-promotion blocker. Section preserved below for audit-trail readability of how the design call was framed before cascade.

~~Surfaced by user question on 2026-04-24: the "≤5 irreducible-shim" floor in `docs/design-pure-bootstrap.md` is framed as a principled target but is actually a generous ceiling. The doc itself names the narrower case: *"If (4) and (5) can be generated, the shim is 3 files"* (§"Irreducible shim (target state)") — and v2 achieves the floor with 2 files (CLI + interpreter). The principled boundary floor is 2–3, not 5.~~

~~"≤5" reads as a principled absolute in gate names (e.g., `pb_hand_rust_at_shim_floor`) when it's really an upper bound carried for continuity with the 2024 design doc. Leaving the number unchallenged risks the "close-everything" claim of R2 resting on a shim-floor number that's looser than the principled floor.~~

~~**Required before promotion:** one of —~~
- ~~**Option A (sharpen-and-keep)** — edit `design-pure-bootstrap.md` to explicitly name the trajectory (`baseline → ~20 → 5 → 3 → 2`); keep `≤5` as the gate name but document that the principled boundary-determined floor is 2–3; promote R2 structure as-is.~~
- ~~**Option B (rename)** — rename the gate to `pb_hand_rust_at_boundary_floor` (threshold = boundary-determined, not a fixed number); land the doc edit + gate rename together; promote R2 structure after rename lands.~~

~~Authority: `docs/design-pure-bootstrap.md` §"Irreducible shim (target state)"; v2 empirical evidence (CLI + interpreter = 2 files per the same doc's "Inspiration: v2's model" section).~~

~~Non-blocking for this PR. Not done in this PR because the gate rename is a cross-doc edit with R1-lane-acceptance implications (T-PB-A's gate name would change); both land as prerequisites to the ROADMAP promotion.~~

## Cross-refs

- Parent: `ROADMAP.md` (sections: `## Release R1 Program`; `## Post-R1 Program — Grounding Completeness`; `## Tracked debts — 2026-04 analyses`).
- Substrate design: `docs/design-substrate-carrier-port-program.md` (E-family lanes + §6a per-method-metadata — **Decision:** Option 3 `MethodContract`; **receipt:** `src/v3/std/algebra.dag` + `src/v3/lenses/cost.dag`).
- Self-hosting anchor: [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md) (LIVE 2026-04-25; 0-floor target + SG census). Supersedes [`docs/design-pure-bootstrap.md`](design-pure-bootstrap.md) (now SUPERSEDED; ≤5-floor framing retracted).
- Thesis: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags authority); `THESIS.md §"Thesis claims — complete list"` (Tier-1 claim lineage).
- Lens capability: `docs/v3-lens-capability-register.md` (per-lens capability tracking).
- DB history: `docs/db-history/db-18.md` (DB-18 Part-2 shipped: workflow-effect carrier + Rust reflection; Part-3 queued: Go accessor). Note: `ROADMAP.md §"Post-R1 Program — Grounding Completeness"` tags "DB-18 parametric algebra attachment" as a post-R1 blocker; that label is not obviously aligned with db-history's DB-18 scope — a pre-promotion rename or new DB number may be warranted for the R2 parametric-algebra prereq.
- Related PRs: #745 (P4 int-literal row — substrate motivation for T-Modeling), #752 (T-PB-A lens-producer priority slice — R1 gate, not R2), #810 (debt-paydown synthesis B-wave + dispatch discipline framework), #812 (INVARIANTS §P5 dispatch-discipline mechanisms wiring).
